use super::*;

/// A deterministic 32-byte test KEK, base64-encoded.
fn test_key_b64() -> String {
    let key = [7u8; KEY_LEN];
    BASE64.encode(key)
}

fn cipher() -> EnvelopeCredentialCipher {
    EnvelopeCredentialCipher::from_base64_key(&test_key_b64()).unwrap()
}

fn aad<'a>() -> Aad<'a> {
    Aad {
        org_id: "11111111-1111-1111-1111-111111111111",
        account_id: "22222222-2222-2222-2222-222222222222",
        field: "smtp_password",
    }
}

#[test]
fn round_trip_recovers_plaintext() {
    let c = cipher();
    let secret = b"super-secret-smtp-pw";
    let sealed = c.encrypt(secret, aad()).unwrap();
    let out = c.decrypt(&sealed, aad()).unwrap();
    assert_eq!(out.expose_secret().as_slice(), secret);
}

#[test]
fn ciphertext_is_not_plaintext_and_aes_v2_format_is_explicit() {
    let c = cipher();
    let secret = b"another-pw";
    let sealed = c.encrypt(secret, aad()).unwrap();
    assert_ne!(sealed.ciphertext, secret);
    assert_eq!(sealed.nonce.len(), 12);
    assert_eq!(sealed.dek_nonce.len(), 12);
    assert_eq!(sealed.key_version, CURRENT_KEY_VERSION);
    assert_eq!(sealed.key_version, 1);
    // One format byte, 32 encrypted DEK bytes and a 16-byte GCM tag.
    assert_eq!(sealed.dek_wrapped.len(), 1 + KEY_LEN + 16);
    assert_eq!(sealed.dek_wrapped[0], 2);
}

#[test]
fn fresh_dek_and_nonce_per_call_yield_distinct_ciphertext() {
    let c = cipher();
    let secret = b"same-input";
    let a = c.encrypt(secret, aad()).unwrap();
    let b = c.encrypt(secret, aad()).unwrap();
    // Random DEK + random nonces => different ciphertext for identical input.
    assert_ne!(a.ciphertext, b.ciphertext);
    assert_ne!(a.nonce, b.nonce);
    assert_ne!(a.dek_wrapped, b.dek_wrapped);
    // Both still decrypt to the same plaintext.
    assert_eq!(
        c.decrypt(&a, aad()).unwrap().expose_secret().as_slice(),
        secret
    );
    assert_eq!(
        c.decrypt(&b, aad()).unwrap().expose_secret().as_slice(),
        secret
    );
}

#[test]
fn wrong_kek_fails_to_decrypt() {
    let c = cipher();
    let sealed = c.encrypt(b"pw", aad()).unwrap();
    let other = EnvelopeCredentialCipher::from_key_bytes(&[9u8; KEY_LEN]).unwrap();
    assert!(matches!(
        other.decrypt(&sealed, aad()),
        Err(CipherError::Decrypt)
    ));
}

#[test]
fn tampered_ciphertext_fails_auth() {
    let c = cipher();
    let mut sealed = c.encrypt(b"pw", aad()).unwrap();
    sealed.ciphertext[0] ^= 0xff;
    assert!(matches!(
        c.decrypt(&sealed, aad()),
        Err(CipherError::Decrypt)
    ));
}

#[test]
fn tampered_nonce_fails_auth() {
    let c = cipher();
    let mut sealed = c.encrypt(b"pw", aad()).unwrap();
    sealed.nonce[0] ^= 0xff;
    assert!(matches!(
        c.decrypt(&sealed, aad()),
        Err(CipherError::Decrypt)
    ));
}

#[test]
fn tampered_wrapped_dek_fails_auth() {
    let c = cipher();
    let mut sealed = c.encrypt(b"pw", aad()).unwrap();
    sealed.dek_wrapped[0] ^= 0xff;
    assert!(matches!(
        c.decrypt(&sealed, aad()),
        Err(CipherError::Decrypt)
    ));
}

#[test]
fn tampered_dek_nonce_fails_auth() {
    let c = cipher();
    let mut sealed = c.encrypt(b"pw", aad()).unwrap();
    sealed.dek_nonce[0] ^= 0xff;
    assert!(matches!(
        c.decrypt(&sealed, aad()),
        Err(CipherError::Decrypt)
    ));
}

#[test]
fn wrong_aad_org_fails_auth() {
    let c = cipher();
    let sealed = c.encrypt(b"pw", aad()).unwrap();
    let mut bad = aad();
    bad.org_id = "99999999-9999-9999-9999-999999999999";
    assert!(matches!(c.decrypt(&sealed, bad), Err(CipherError::Decrypt)));
}

#[test]
fn wrong_aad_account_fails_auth() {
    let c = cipher();
    let sealed = c.encrypt(b"pw", aad()).unwrap();
    let mut bad = aad();
    bad.account_id = "00000000-0000-0000-0000-000000000000";
    assert!(matches!(c.decrypt(&sealed, bad), Err(CipherError::Decrypt)));
}

#[test]
fn wrong_aad_field_fails_auth() {
    // The crux of envelope AAD-binding: a ciphertext sealed for
    // `smtp_password` must NOT decrypt under the `imap_password` field.
    let c = cipher();
    let sealed = c.encrypt(b"pw", aad()).unwrap();
    let mut bad = aad();
    bad.field = "imap_password";
    assert!(matches!(c.decrypt(&sealed, bad), Err(CipherError::Decrypt)));
}

#[test]
fn aad_encoding_is_unambiguous() {
    // Length-prefixing keeps ("ab","c") distinct from ("a","bc").
    let one = Aad {
        org_id: "ab",
        account_id: "c",
        field: "f",
    }
    .encode();
    let two = Aad {
        org_id: "a",
        account_id: "bc",
        field: "f",
    }
    .encode();
    assert_ne!(one, two);
}

#[test]
fn wrong_key_version_is_rejected() {
    let c = cipher();
    let mut sealed = c.encrypt(b"pw", aad()).unwrap();
    sealed.key_version = 99;
    assert!(matches!(
        c.decrypt(&sealed, aad()),
        Err(CipherError::KeyVersion)
    ));
}

#[test]
fn bad_master_key_inputs_are_rejected() {
    assert!(matches!(
        EnvelopeCredentialCipher::from_base64_key("not!base64!"),
        Err(CipherError::MasterKey)
    ));
    // Valid base64 but wrong length (16 bytes, not 32).
    let short = BASE64.encode([1u8; 16]);
    assert!(matches!(
        EnvelopeCredentialCipher::from_base64_key(&short),
        Err(CipherError::MasterKey)
    ));
    assert!(matches!(
        EnvelopeCredentialCipher::from_key_bytes(&[0u8; 31]),
        Err(CipherError::MasterKey)
    ));
}

#[test]
fn secret_debug_is_redacted() {
    // `SecretBox`'s Debug never prints the secret bytes.
    let c = cipher();
    let sealed = c.encrypt(b"top-secret-value", aad()).unwrap();
    let recovered = c.decrypt(&sealed, aad()).unwrap();
    let dbg = format!("{recovered:?}");
    assert!(
        dbg.contains("REDACTED"),
        "secret Debug must redact, got: {dbg}"
    );
    assert!(!dbg.contains("top-secret-value"));
}

#[test]
fn empty_plaintext_round_trips() {
    let c = cipher();
    let sealed = c.encrypt(b"", aad()).unwrap();
    let out = c.decrypt(&sealed, aad()).unwrap();
    assert!(out.expose_secret().is_empty());
}

#[test]
fn wrong_nonce_length_is_rejected() {
    let c = cipher();
    let current = c.encrypt(b"pw", aad()).unwrap();
    for length in [0, 1, 11, 13, 23, 24, 25] {
        let mut payload_bad = current.clone();
        payload_bad.nonce.resize(length, 0);
        assert!(matches!(
            c.decrypt(&payload_bad, aad()),
            Err(CipherError::Decrypt)
        ));
        let mut wrap_bad = current.clone();
        wrap_bad.dek_nonce.resize(length, 0);
        assert!(matches!(
            c.decrypt(&wrap_bad, aad()),
            Err(CipherError::Decrypt)
        ));
    }
    // Preserve the original legacy 24-to-12 truncation refusal for both fields.
    for (field, _, sealed) in legacy_fixtures() {
        let context = Aad { field, ..aad() };
        let mut payload_bad = sealed.clone();
        payload_bad.nonce.truncate(12);
        assert!(matches!(
            c.decrypt(&payload_bad, context),
            Err(CipherError::Decrypt)
        ));
        let mut wrap_bad = sealed;
        wrap_bad.dek_nonce.truncate(12);
        assert!(matches!(
            c.decrypt(&wrap_bad, context),
            Err(CipherError::Decrypt)
        ));
    }
}

fn legacy_fixtures() -> Vec<(&'static str, &'static [u8], SealedCredential)> {
    vec![
        (
            "smtp_password",
            b"legacy-smtp-password",
            SealedCredential {
                ciphertext: BASE64
                    .decode("vEr0rLa/+JI8bmO6smU6xEDa9Wh4gVipdPs9D4MkIDqYqw4G")
                    .unwrap(),
                nonce: BASE64.decode("wu7cXtK9xJKjW6diml3rpuW3cl9fgpLw").unwrap(),
                dek_wrapped: BASE64
                    .decode("TER/qSD1DJy8+RrVZC6mxJYaTRG9PUwGHy2KSmBJQn6HdN8vMhciP8M5gU2keMEp")
                    .unwrap(),
                dek_nonce: BASE64.decode("fN3zCjVAQgIlUQcpqEGCTLrL+aa4UStt").unwrap(),
                key_version: 1,
            },
        ),
        (
            "imap_password",
            b"legacy-imap-password",
            SealedCredential {
                ciphertext: BASE64
                    .decode("kFDw/oHqyqsQhqCSdGYNz6qQPX9/MPwhj+3aRbpzrHc5D6LF")
                    .unwrap(),
                nonce: BASE64.decode("rL3JrimiNn9EZYyhr8zFDMHOtOFxkwh3").unwrap(),
                dek_wrapped: BASE64
                    .decode("Ei/Z4VML2WNOkQQSpDcBme0RnPvA2ZWnkTsmNKVJ99HgQkayJGTp0sMizzYYnqmo")
                    .unwrap(),
                dek_nonce: BASE64.decode("NE5ctAFFuaGKXNrDItn7xY4n4GlbZPxi").unwrap(),
                key_version: 1,
            },
        ),
    ]
}

fn v2_test_aad(purpose: u8, context: Aad<'_>) -> Vec<u8> {
    let mut bytes = b"console.mail.credential\0\x02".to_vec();
    bytes.push(purpose);
    bytes.extend_from_slice(&context.encode());
    bytes
}

#[test]
fn frozen_legacy_smtp_and_imap_remain_readable_without_mutation() {
    let c = cipher();
    for (field, secret, sealed) in legacy_fixtures() {
        let context = Aad { field, ..aad() };
        let original = sealed.clone();
        assert_eq!(sealed.nonce.len(), 24);
        assert_eq!(sealed.dek_nonce.len(), 24);
        assert_eq!(sealed.dek_wrapped.len(), 48);
        assert_eq!(sealed.key_version, 1);
        assert_ne!(sealed.ciphertext, secret);
        for _ in 0..2 {
            assert_eq!(
                c.decrypt(&sealed, context)
                    .unwrap()
                    .expose_secret()
                    .as_slice(),
                secret
            );
            assert_eq!(sealed, original);
        }
        let wrong = Aad {
            field: if field == "smtp_password" {
                "imap_password"
            } else {
                "smtp_password"
            },
            ..context
        };
        assert!(matches!(
            c.decrypt(&sealed, wrong),
            Err(CipherError::Decrypt)
        ));
        assert_eq!(sealed, original);
    }
}

#[test]
fn aes256_gcm_known_answer_opens_through_production_primitive() {
    // NIST AES-256-GCM: zero 256-bit key, zero 96-bit IV, empty AAD,
    // 16 zero plaintext bytes. Ciphertext and 128-bit tag are fixed oracle bytes.
    let ciphertext = [
        0xce, 0xa7, 0x40, 0x3d, 0x4d, 0x60, 0x6b, 0x6e, 0x07, 0x4e, 0xc5, 0xd3, 0xba, 0xf3, 0x9d,
        0x18, 0xd0, 0xd1, 0xc8, 0xa7, 0x99, 0x99, 0x6b, 0xf0, 0x26, 0x5b, 0x98, 0xb5, 0xd4, 0x8a,
        0xb9, 0x19,
    ];
    assert_eq!(
        aes_open(&[0; 32], &[0; 12], &ciphertext, b"")
            .unwrap()
            .as_slice(),
        &[0; 16]
    );
    for offset in [0, 15, 16, 31] {
        let mut corrupt = ciphertext;
        corrupt[offset] ^= 1;
        assert!(matches!(
            aes_open(&[0; 32], &[0; 12], &corrupt, b""),
            Err(CipherError::Decrypt)
        ));
    }
    assert!(aes_open(&[1; 32], &[0; 12], &ciphertext, b"").is_err());
    assert!(aes_open(&[0; 32], &[1; 12], &ciphertext, b"").is_err());
    assert!(aes_open(&[0; 32], &[0; 12], &ciphertext, b"unexpected-aad").is_err());
    for length in [0, 16, 31, 33] {
        assert!(aes_open(&vec![0; length], &[0; 12], &ciphertext, b"").is_err());
    }
    for length in [0, 11, 13, 24] {
        assert!(aes_open(&[0; 32], &vec![0; length], &ciphertext, b"").is_err());
    }
    for end in 0..ciphertext.len() {
        assert!(aes_open(&[0; 32], &[0; 12], &ciphertext[..end], b"").is_err());
    }
}

#[test]
fn aes_v2_binds_domain_row_field_and_distinct_payload_wrap_purposes() {
    let c = cipher();
    let secret = b"purpose-bound-password";
    let sealed = c.encrypt(secret, aad()).unwrap();
    assert_eq!(sealed.dek_wrapped.len(), 49);
    assert_eq!(sealed.dek_wrapped[0], 2);
    let wrap_aad = v2_test_aad(2, aad());
    let payload_aad = v2_test_aad(1, aad());
    let dek = aes_open(
        &[7; 32],
        &sealed.dek_nonce,
        &sealed.dek_wrapped[1..],
        &wrap_aad,
    )
    .unwrap();
    assert_eq!(dek.len(), 32);
    assert_eq!(
        aes_open(&dek, &sealed.nonce, &sealed.ciphertext, &payload_aad)
            .unwrap()
            .as_slice(),
        secret
    );
    assert!(
        aes_open(
            &[7; 32],
            &sealed.dek_nonce,
            &sealed.dek_wrapped[1..],
            &payload_aad
        )
        .is_err()
    );
    assert!(aes_open(&dek, &sealed.nonce, &sealed.ciphertext, &wrap_aad).is_err());
    for purpose in [0, 1, 2, 3] {
        if purpose != 2 {
            assert!(
                aes_open(
                    &[7; 32],
                    &sealed.dek_nonce,
                    &sealed.dek_wrapped[1..],
                    &v2_test_aad(purpose, aad())
                )
                .is_err()
            );
        }
        if purpose != 1 {
            assert!(
                aes_open(
                    &dek,
                    &sealed.nonce,
                    &sealed.ciphertext,
                    &v2_test_aad(purpose, aad())
                )
                .is_err()
            );
        }
    }
    let mut corrupt_domain = wrap_aad.clone();
    corrupt_domain[0] ^= 1;
    assert!(
        aes_open(
            &[7; 32],
            &sealed.dek_nonce,
            &sealed.dek_wrapped[1..],
            &corrupt_domain
        )
        .is_err()
    );
    assert!(
        aes_open(
            &[7; 32],
            &sealed.dek_nonce,
            &sealed.dek_wrapped[1..],
            &aad().encode()
        )
        .is_err()
    );
    assert!(aes_open(&dek, &sealed.nonce, &sealed.ciphertext, &aad().encode()).is_err());
}

#[test]
fn aes_v2_fresh_wrapping_nonces_and_actual_deks_are_independent() {
    let c = cipher();
    let a = c.encrypt(b"same-input", aad()).unwrap();
    let b = c.encrypt(b"same-input", aad()).unwrap();
    let wrap_aad = v2_test_aad(2, aad());
    let first_dek = aes_open(&[7; 32], &a.dek_nonce, &a.dek_wrapped[1..], &wrap_aad).unwrap();
    let second_dek = aes_open(&[7; 32], &b.dek_nonce, &b.dek_wrapped[1..], &wrap_aad).unwrap();
    assert_ne!(a.dek_nonce, b.dek_nonce);
    assert_ne!(a.nonce, a.dek_nonce);
    assert_ne!(b.nonce, b.dek_nonce);
    assert_ne!(first_dek.as_slice(), second_dek.as_slice());
    assert_eq!(a.key_version, 1);
    assert_eq!(b.key_version, 1);
}

#[test]
fn aes_v2_rejects_mixed_formats_unknown_markers_and_every_bundle_corruption() {
    let c = cipher();
    let current = c.encrypt(b"new-smtp-password", aad()).unwrap();
    let (_, _, legacy) = legacy_fixtures().remove(0);
    let original = current.clone();
    assert_eq!(
        c.decrypt(&current, aad())
            .unwrap()
            .expose_secret()
            .as_slice(),
        b"new-smtp-password"
    );
    for bits in 1..8 {
        let mut mixed = current.clone();
        if bits & 1 != 0 {
            mixed.nonce = legacy.nonce.clone();
        }
        if bits & 2 != 0 {
            mixed.dek_nonce = legacy.dek_nonce.clone();
        }
        if bits & 4 != 0 {
            mixed.dek_wrapped = legacy.dek_wrapped.clone();
        }
        assert!(matches!(
            c.decrypt(&mixed, aad()),
            Err(CipherError::Decrypt)
        ));
    }
    for marker in 0..=u8::MAX {
        if marker == 2 {
            continue;
        }
        let mut corrupt = current.clone();
        corrupt.dek_wrapped[0] = marker;
        assert!(matches!(
            c.decrypt(&corrupt, aad()),
            Err(CipherError::Decrypt)
        ));
    }
    // Each retained byte, including the AES tag and wrapped-DEK body, is authenticated.
    for field in 0..4 {
        let length = match field {
            0 => current.ciphertext.len(),
            1 => current.nonce.len(),
            2 => current.dek_wrapped.len(),
            _ => current.dek_nonce.len(),
        };
        for offset in 0..length {
            let mut corrupt = current.clone();
            let bytes = match field {
                0 => &mut corrupt.ciphertext,
                1 => &mut corrupt.nonce,
                2 => &mut corrupt.dek_wrapped,
                _ => &mut corrupt.dek_nonce,
            };
            bytes[offset] ^= 1;
            assert!(matches!(
                c.decrypt(&corrupt, aad()),
                Err(CipherError::Decrypt)
            ));
        }
        let mut extended = current.clone();
        match field {
            0 => extended.ciphertext.push(0),
            1 => extended.nonce.push(0),
            2 => extended.dek_wrapped.push(0),
            _ => extended.dek_nonce.push(0),
        }
        assert!(matches!(
            c.decrypt(&extended, aad()),
            Err(CipherError::Decrypt)
        ));
        for length in 0..length {
            let mut corrupt = current.clone();
            match field {
                0 => corrupt.ciphertext.truncate(length),
                1 => corrupt.nonce.truncate(length),
                2 => corrupt.dek_wrapped.truncate(length),
                _ => corrupt.dek_nonce.truncate(length),
            }
            assert!(matches!(
                c.decrypt(&corrupt, aad()),
                Err(CipherError::Decrypt)
            ));
        }
    }
    for key_version in [0, 2, 99] {
        for sealed in [&current, &legacy] {
            let mut wrong_version = sealed.clone();
            wrong_version.key_version = key_version;
            assert!(matches!(
                c.decrypt(&wrong_version, aad()),
                Err(CipherError::KeyVersion)
            ));
        }
    }
    assert_eq!(current, original);
    assert_eq!(
        c.decrypt(&current, aad())
            .unwrap()
            .expose_secret()
            .as_slice(),
        b"new-smtp-password"
    );
}
