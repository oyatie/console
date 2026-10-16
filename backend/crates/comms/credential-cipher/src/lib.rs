//! Envelope AEAD for SMTP/IMAP credentials, implementing the application-owned port.
//!
//! New writes use AES-256-GCM with a fresh 256-bit DEK per secret and independent
//! provider-generated 96-bit payload and KEK-wrap nonces. Both stages bind the
//! versioned domain, distinct purpose and exact caller row/field AAD.
//!
//! The per-field v2 tuple is (12-byte nonce, 12-byte wrap nonce, 49-byte wrapped
//! DEK): the wrap starts with format byte 2 followed by 32 ciphertext bytes and
//! a 16-byte tag. Legacy (24, 24, 48) XChaCha bundles remain readable with their
//! original AAD. Other tuples fail closed; authentication failure never falls
//! back. The shared SMTP/IMAP KEK version remains 1, independently of format.
//!
//! The master KEK comes from base64 `CONSOLE_MAIL_MASTER_KEY`. Working secrets
//! are zeroized on failure/drop; this crate never logs credentials or keys.
//!
//! # Deployment and key lifetime
//!
//! Drain every old API/send/sync/worker reader before enabling AES writes. Every
//! rollback artifact must also read AES; no ciphertext migration or KEK rotation
//! occurs here. Randomized GCM wrapping requires at most 2^32 invocations per
//! KEK across its entire fleet lifetime, including retries/discarded writes and
//! restores. Operators must bound that aggregate before exposure; this provider
//! does not enforce a fleet counter and key rotation is not implemented here.
//! AES-256-GCM is a NIST-approved algorithm; the workspace's non-FIPS provider
//! configuration does not establish FIPS module certification.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use aws_lc_rs::aead::{AES_256_GCM, Aad as ProviderAad, Nonce, RandomizedNonceKey};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use secrecy::{ExposeSecret, SecretBox};
use zeroize::{Zeroize, Zeroizing};

pub use console_comms_application::credential_cipher::{
    Aad, CipherError, CredentialCipher, SealedCredential,
};

/// The environment variable carrying the base64-encoded 32-byte master KEK.
pub const MASTER_KEY_ENV: &str = "CONSOLE_MAIL_MASTER_KEY";

/// The key-derivation version stamped onto every freshly encrypted row. Bumped
/// (with a re-wrap job) on KEK rotation.
pub const CURRENT_KEY_VERSION: i16 = 1;

const KEY_LEN: usize = 32;

/// AES-256-GCM writer and compatible legacy reader with a protected master KEK.
pub struct EnvelopeCredentialCipher {
    /// The master key-encryption key (32 bytes), zeroized on drop.
    kek: SecretBox<[u8; KEY_LEN]>,
    key_version: i16,
}

impl EnvelopeCredentialCipher {
    /// Build the cipher from the base64-encoded 32-byte KEK in the
    /// `CONSOLE_MAIL_MASTER_KEY` environment variable.
    pub fn from_env() -> Result<Self, CipherError> {
        let encoded =
            Zeroizing::new(std::env::var(MASTER_KEY_ENV).map_err(|_| CipherError::MasterKey)?);
        Self::from_base64_key(&encoded)
    }

    /// Build the cipher from a base64 (standard alphabet) encoding of exactly
    /// 32 key bytes. The decoded buffer is zeroized after the key is copied in.
    pub fn from_base64_key(encoded: &str) -> Result<Self, CipherError> {
        let decoded = Zeroizing::new(
            BASE64
                .decode(encoded.trim())
                .map_err(|_| CipherError::MasterKey)?,
        );
        Self::from_key_bytes(&decoded)
    }

    /// Build the cipher directly from raw key bytes (must be exactly 32).
    pub fn from_key_bytes(bytes: &[u8]) -> Result<Self, CipherError> {
        if bytes.len() != KEY_LEN {
            return Err(CipherError::MasterKey);
        }
        let mut key = [0u8; KEY_LEN];
        key.copy_from_slice(bytes);
        let kek = SecretBox::new(Box::new(key));
        // `key` is a Copy array on the stack; overwrite our local copy too.
        key.zeroize();
        Ok(Self {
            kek,
            key_version: CURRENT_KEY_VERSION,
        })
    }
}

impl CredentialCipher for EnvelopeCredentialCipher {
    fn encrypt(&self, plaintext: &[u8], aad: Aad<'_>) -> Result<SealedCredential, CipherError> {
        let mut dek = Zeroizing::new([0u8; KEY_LEN]);
        aws_lc_rs::rand::fill(dek.as_mut()).map_err(|_| CipherError::Encrypt)?;
        let (ciphertext, nonce) = aes_seal(dek.as_ref(), plaintext, &versioned_aad(1, aad))?;
        let (wrapped, dek_nonce) = aes_seal(
            self.kek.expose_secret().as_slice(),
            dek.as_ref(),
            &versioned_aad(2, aad),
        )?;
        let mut dek_wrapped = Vec::with_capacity(49);
        dek_wrapped.push(2);
        dek_wrapped.extend_from_slice(&wrapped);
        Ok(SealedCredential {
            ciphertext,
            nonce,
            dek_wrapped,
            dek_nonce,
            key_version: self.key_version,
        })
    }

    fn decrypt(
        &self,
        sealed: &SealedCredential,
        aad: Aad<'_>,
    ) -> Result<SecretBox<Vec<u8>>, CipherError> {
        if sealed.key_version != self.key_version {
            return Err(CipherError::KeyVersion);
        }
        let mut plaintext = match (
            sealed.nonce.len(),
            sealed.dek_nonce.len(),
            sealed.dek_wrapped.len(),
        ) {
            (12, 12, 49) if sealed.dek_wrapped[0] == 2 => {
                let dek = aes_open(
                    self.kek.expose_secret().as_slice(),
                    &sealed.dek_nonce,
                    &sealed.dek_wrapped[1..],
                    &versioned_aad(2, aad),
                )?;
                aes_open(
                    &dek,
                    &sealed.nonce,
                    &sealed.ciphertext,
                    &versioned_aad(1, aad),
                )?
            }
            (24, 24, 48) => {
                let aad_bytes = aad.encode();
                let dek = legacy_open(
                    self.kek.expose_secret().as_slice(),
                    &sealed.dek_nonce,
                    &sealed.dek_wrapped,
                    &aad_bytes,
                )?;
                legacy_open(&dek, &sealed.nonce, &sealed.ciphertext, &aad_bytes)?
            }
            _ => return Err(CipherError::Decrypt),
        };
        Ok(SecretBox::new(Box::new(std::mem::take(&mut *plaintext))))
    }
}

fn versioned_aad(purpose: u8, aad: Aad<'_>) -> Vec<u8> {
    let mut bytes = b"console.mail.credential\0\x02".to_vec();
    bytes.push(purpose);
    bytes.extend_from_slice(&aad.encode());
    bytes
}

fn aes_seal(key: &[u8], plaintext: &[u8], aad: &[u8]) -> Result<(Vec<u8>, Vec<u8>), CipherError> {
    let key = RandomizedNonceKey::new(&AES_256_GCM, key).map_err(|_| CipherError::Encrypt)?;
    // Reserve the tag before copying plaintext; appending it must not free an
    // allocation that still contains a plaintext copy.
    let capacity = plaintext
        .len()
        .checked_add(16)
        .ok_or(CipherError::Encrypt)?;
    let mut buffer = Zeroizing::new(Vec::with_capacity(capacity));
    buffer.extend_from_slice(plaintext);
    let nonce = key
        .seal_in_place_append_tag(ProviderAad::from(aad), &mut *buffer)
        .map_err(|_| CipherError::Encrypt)?;
    Ok((std::mem::take(&mut *buffer), nonce.as_ref().to_vec()))
}

fn aes_open(
    key: &[u8],
    nonce: &[u8],
    ciphertext: &[u8],
    aad: &[u8],
) -> Result<Zeroizing<Vec<u8>>, CipherError> {
    let key = RandomizedNonceKey::new(&AES_256_GCM, key).map_err(|_| CipherError::Decrypt)?;
    let nonce = Nonce::try_assume_unique_for_key(nonce).map_err(|_| CipherError::Decrypt)?;
    let mut buffer = Zeroizing::new(ciphertext.to_vec());
    let length = key
        .open_in_place(nonce, ProviderAad::from(aad), &mut buffer)
        .map_err(|_| CipherError::Decrypt)?
        .len();
    buffer.truncate(length);
    Ok(buffer)
}

fn legacy_open(
    key: &[u8],
    nonce: &[u8],
    ciphertext: &[u8],
    aad: &[u8],
) -> Result<Zeroizing<Vec<u8>>, CipherError> {
    if key.len() != KEY_LEN || nonce.len() != 24 {
        return Err(CipherError::Decrypt);
    }
    XChaCha20Poly1305::new(Key::from_slice(key))
        .decrypt(
            XNonce::from_slice(nonce),
            Payload {
                msg: ciphertext,
                aad,
            },
        )
        .map(Zeroizing::new)
        .map_err(|_| CipherError::Decrypt)
}

#[cfg(test)]
#[path = "aes_tests.rs"]
mod tests;
