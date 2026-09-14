//! Conserved public API behavior across the Ring -> AWS-LC migration.
//! RFC8032 section 7.1 TEST1/TEST2 supply independent raw Ed25519 vectors:
//! https://www.rfc-editor.org/rfc/rfc8032#section-7.1
//! RING_TRANSCRIPT was generated separately with Ring 0.17.14 from the public
//! RFC TEST1 seed and the fixed synthetic message below. No signing key is
//! retained here. These tests deliberately have no provider-specific imports.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use console_platform_audit_chain::{
    ExternalSealSigner, InMemoryEd25519Signer, SealSignError, SealSignTransport, SealSigner,
};
use std::collections::BTreeMap;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

const RFC1_PK: &str = "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a";
const RFC1_SIG: &str = concat!(
    "e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e06522490155",
    "5fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b"
);
const RFC2_PK: &str = "3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c";
const RFC2_SIG: &str = concat!(
    "92a009a9f0d4cab8720e820b5f642540a2b27b5416503f8fb3762223ebdb69da",
    "085ac1e43e15996e458f3613d0f11d8c387b2eaeb4302aeeb00d291612bb0c00"
);
const RING_MESSAGE: &[u8] = b"console:audit-chain:compatibility:2026-09-14";
const RING_TRANSCRIPT: &str = concat!(
    "03b3981abe6f318dd9e6327a89884c51e66118b4e22a2d8ac0e3e61424f230a1a",
    "d1eabe17f56fccb12cd349607464b178bc9684aaaa33b52c93508607361730b"
);
const PINNED: &str = "external:compatibility:rfc1";

#[derive(Default)]
struct OfflineTransport {
    calls: AtomicUsize,
}
impl SealSignTransport for OfflineTransport {
    fn sign(&self, _: &str, _: &[u8]) -> Result<Vec<u8>, SealSignError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Err(SealSignError::Unavailable(
            "synthetic offline custody".into(),
        ))
    }
}
fn external(public_key: Vec<u8>) -> (ExternalSealSigner, Arc<OfflineTransport>) {
    let transport = Arc::new(OfflineTransport::default());
    let signer = ExternalSealSigner::new(
        PINNED.into(),
        transport.clone(),
        BTreeMap::from([(PINNED.into(), public_key)]),
    )
    .unwrap();
    (signer, transport)
}
fn assert_both(public_key: &[u8], message: &[u8], signature: &[u8], expected: bool) {
    let (external, transport) = external(public_key.to_vec());
    let dev = InMemoryEd25519Signer::generate().unwrap();
    let stored_dev_ref = format!("test:ed25519:{}", hex::encode(public_key));
    assert_eq!(
        external.verify(message, signature, PINNED).unwrap(),
        expected,
        "external anchor verdict"
    );
    assert_eq!(
        dev.verify(message, signature, &stored_dev_ref).unwrap(),
        expected,
        "dev stored raw-key verdict"
    );
    assert_eq!(
        transport.calls.load(Ordering::SeqCst),
        0,
        "verification must not depend on signing transport"
    );
}

#[test]
fn rfc8032_empty_message_vector_verifies_through_both_public_apis() {
    assert_both(
        &hex::decode(RFC1_PK).unwrap(),
        b"",
        &hex::decode(RFC1_SIG).unwrap(),
        true,
    );
}
#[test]
fn rfc8032_one_byte_message_vector_verifies_through_both_public_apis() {
    assert_both(
        &hex::decode(RFC2_PK).unwrap(),
        &[0x72],
        &hex::decode(RFC2_SIG).unwrap(),
        true,
    );
}
#[test]
fn frozen_old_ring_transcript_remains_verifiable_without_ring_test_dependency() {
    assert_both(
        &hex::decode(RFC1_PK).unwrap(),
        RING_MESSAGE,
        &hex::decode(RING_TRANSCRIPT).unwrap(),
        true,
    );
}
#[test]
fn wrong_messages_and_swapped_keys_are_bad_signatures_not_transport_errors() {
    let pk = hex::decode(RFC1_PK).unwrap();
    let signature = hex::decode(RING_TRANSCRIPT).unwrap();
    assert_both(&pk, RING_MESSAGE, &signature, true);
    for message in [
        b"".as_slice(),
        b"console:audit-chain:compatibility:2026-09-15".as_slice(),
        b"console:audit-chain:compatibility:2026-09-14\0".as_slice(),
    ] {
        assert_both(&pk, message, &signature, false);
    }
    assert_both(
        &hex::decode(RFC2_PK).unwrap(),
        RING_MESSAGE,
        &signature,
        false,
    );
}
#[test]
fn flipping_each_signature_byte_is_rejected() {
    let pk = hex::decode(RFC1_PK).unwrap();
    let signature = hex::decode(RING_TRANSCRIPT).unwrap();
    assert_eq!(signature.len(), 64);
    for index in 0..signature.len() {
        let mut damaged = signature.clone();
        damaged[index] ^= 1;
        assert_both(&pk, RING_MESSAGE, &damaged, false);
    }
}
#[test]
fn truncated_and_extended_signatures_do_not_change_raw_wire_contract() {
    let pk = hex::decode(RFC1_PK).unwrap();
    let signature = hex::decode(RING_TRANSCRIPT).unwrap();
    for length in [0, 1, 31, 32, 63] {
        assert_both(&pk, RING_MESSAGE, &signature[..length], false);
    }
    for tail in [vec![0], vec![0xff], vec![0; 64]] {
        let mut extended = signature.clone();
        extended.extend(tail);
        assert_both(&pk, RING_MESSAGE, &extended, false);
    }
}
#[test]
fn nonraw_public_key_encodings_are_rejected_including_valid_der_spki_wrapper() {
    let pk = hex::decode(RFC1_PK).unwrap();
    let signature = hex::decode(RING_TRANSCRIPT).unwrap();
    assert_both(&pk, RING_MESSAGE, &signature, true);
    // RFC8410 Ed25519 SPKI: SEQUENCE(AlgorithmIdentifier id-Ed25519,
    // BIT STRING raw32). A valid DER container is not this API's raw key.
    let mut spki = hex::decode("302a300506032b6570032100").unwrap();
    spki.extend_from_slice(&pk);
    let mut extended = pk.clone();
    extended.push(0);
    for encoded in [Vec::new(), pk[..31].to_vec(), extended, spki] {
        assert_both(&encoded, RING_MESSAGE, &signature, false);
    }
}
#[test]
fn trusted_key_bytes_do_not_make_an_unpinned_key_reference_trusted() {
    let (signer, transport) = external(hex::decode(RFC1_PK).unwrap());
    let signature = hex::decode(RING_TRANSCRIPT).unwrap();
    assert!(signer.verify(RING_MESSAGE, &signature, PINNED).unwrap());
    for key_ref in [
        "external:attacker:rfc1".to_owned(),
        format!("test:ed25519:{RFC1_PK}"),
        String::new(),
    ] {
        assert!(!signer.verify(RING_MESSAGE, &signature, &key_ref).unwrap());
    }
    assert_eq!(transport.calls.load(Ordering::SeqCst), 0);
    assert!(matches!(
        ExternalSealSigner::new(
            "missing".into(),
            transport,
            BTreeMap::from([(PINNED.into(), hex::decode(RFC1_PK).unwrap())])
        ),
        Err(SealSignError::KeyRef(_))
    ));
}
#[test]
fn pinned_old_anchor_survives_rotation_but_cannot_be_substituted_for_active_key() {
    let transport = Arc::new(OfflineTransport::default());
    let active = "external:compatibility:rfc2";
    let signer = ExternalSealSigner::new(
        active.into(),
        transport.clone(),
        BTreeMap::from([
            (PINNED.into(), hex::decode(RFC1_PK).unwrap()),
            (active.into(), hex::decode(RFC2_PK).unwrap()),
        ]),
    )
    .unwrap();
    assert_eq!(signer.key_ref(), active);
    assert!(
        signer
            .verify(b"", &hex::decode(RFC1_SIG).unwrap(), PINNED)
            .unwrap()
    );
    assert!(
        signer
            .verify(&[0x72], &hex::decode(RFC2_SIG).unwrap(), active)
            .unwrap()
    );
    assert!(
        !signer
            .verify(b"", &hex::decode(RFC1_SIG).unwrap(), active)
            .unwrap()
    );
    assert_eq!(transport.calls.load(Ordering::SeqCst), 0);
}
#[test]
fn generated_dev_signer_preserves_raw_key_signature_and_key_reference_shapes() {
    let signer = InMemoryEd25519Signer::generate().unwrap();
    let pk = signer.public_key();
    assert_eq!(pk.len(), 32);
    assert_eq!(
        signer.key_ref(),
        format!("test:ed25519:{}", hex::encode(&pk))
    );
    let signature = signer.sign(RING_MESSAGE).unwrap();
    assert_eq!(signature.len(), 64);
    assert_eq!(
        signature,
        signer.sign(RING_MESSAGE).unwrap(),
        "Ed25519 signing stays deterministic for the same key/message"
    );
    assert!(
        signer
            .verify(RING_MESSAGE, &signature, signer.key_ref())
            .unwrap()
    );
    let (external, _) = external(pk);
    assert!(external.verify(RING_MESSAGE, &signature, PINNED).unwrap());
    assert!(!external.verify(b"different", &signature, PINNED).unwrap());
}
#[test]
fn malformed_dev_key_references_retain_typed_error_instead_of_panicking() {
    let signer = InMemoryEd25519Signer::generate().unwrap();
    let signature = hex::decode(RFC1_SIG).unwrap();
    for key_ref in [
        "",
        "external:compatibility:rfc1",
        "test:ed25519:not-hex",
        "test:ed25519:0",
    ] {
        assert!(matches!(
            signer.verify(b"", &signature, key_ref),
            Err(SealSignError::KeyRef(_))
        ));
    }
}
#[test]
fn offline_external_signing_fails_closed_while_fixed_signature_verification_remains_available() {
    let (signer, transport) = external(hex::decode(RFC1_PK).unwrap());
    assert!(matches!(
        signer.sign(RING_MESSAGE),
        Err(SealSignError::Unavailable(_))
    ));
    assert_eq!(transport.calls.load(Ordering::SeqCst), 1);
    assert!(
        signer
            .verify(RING_MESSAGE, &hex::decode(RING_TRANSCRIPT).unwrap(), PINNED)
            .unwrap()
    );
    assert_eq!(transport.calls.load(Ordering::SeqCst), 1);
}

#[test]
fn external_signing_preserves_exact_transport_key_message_and_signature_bytes() {
    struct RecordedTranscript {
        calls: std::sync::Mutex<Vec<(String, Vec<u8>)>>,
    }
    impl SealSignTransport for RecordedTranscript {
        fn sign(&self, key_ref: &str, message: &[u8]) -> Result<Vec<u8>, SealSignError> {
            self.calls
                .lock()
                .unwrap()
                .push((key_ref.into(), message.to_vec()));
            // Frozen response from an external signer, not a replacement
            // cryptographic implementation or test-generated signature.
            Ok(hex::decode(RING_TRANSCRIPT).unwrap())
        }
    }
    let transport = Arc::new(RecordedTranscript {
        calls: std::sync::Mutex::new(Vec::new()),
    });
    let signer = ExternalSealSigner::new(
        PINNED.into(),
        transport.clone(),
        BTreeMap::from([(PINNED.into(), hex::decode(RFC1_PK).unwrap())]),
    )
    .unwrap();
    let signature = signer.sign(RING_MESSAGE).unwrap();
    assert_eq!(signature, hex::decode(RING_TRANSCRIPT).unwrap());
    assert_eq!(
        *transport.calls.lock().unwrap(),
        vec![(PINNED.into(), RING_MESSAGE.to_vec())]
    );
    assert!(signer.verify(RING_MESSAGE, &signature, PINNED).unwrap());
    assert_eq!(
        transport.calls.lock().unwrap().len(),
        1,
        "verify must not contact external custody"
    );
}
