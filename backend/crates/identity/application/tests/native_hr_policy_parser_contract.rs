use console_identity_application::company_policy::decode_native_hr_policy_command_v1;

#[path = "hr_policy_fixtures/literals.rs"]
mod literals;
#[macro_use]
#[path = "hr_policy_fixtures/assert_fields.rs"]
mod assert_fields;
use literals::{EXPIRY, Literal, VECTORS};

const PROTOCOL: &str = "COMPANY_HR_APPOINTMENT_CAPABILITY_V1";
fn uuid(n: u128) -> uuid::Uuid {
    uuid::Uuid::from_u128(n)
}
fn reject(bytes: &[u8]) {
    let error = match decode_native_hr_policy_command_v1(PROTOCOL, 1, bytes) {
        Err(error) => error,
        Ok(_) => panic!("malformed literal accepted"),
    };
    assert_eq!(error.kind, console_kernel_core::ErrorKind::Validation);
    assert_eq!(error.message, "invalid native HR policy command");
}
fn changed(v: &Literal, field: &str, value: &[u8]) {
    let mut bytes = v.bytes();
    let start = v.at(field);
    bytes[start..start + value.len()].copy_from_slice(value);
    reject(&bytes);
}

#[test]
fn hr_policy_setup_decodes_exact_fields() {
    assert_eq!(VECTORS[..2].len(), 2);
    for v in &VECTORS[..2] {
        assert_literal!(v);
    }
}
#[test]
#[allow(non_snake_case)]
fn hr_policy_B_grants_decode_without_qualification_pair() {
    assert_eq!(VECTORS[2..6].len(), 4);
    for v in &VECTORS[2..6] {
        assert_literal!(v);
    }
}
#[test]
#[allow(non_snake_case)]
fn hr_policy_C_grants_decode_complete_snapshot_pair() {
    assert_eq!(VECTORS[6..22].len(), 16);
    for v in &VECTORS[6..22] {
        assert_literal!(v);
    }
}
#[test]
fn hr_policy_revoke_decodes_without_qualification() {
    assert_eq!(VECTORS[22..].len(), 4);
    for v in &VECTORS[22..] {
        assert_literal!(v);
    }
}
#[test]
fn hr_policy_protocol_header_and_flags_reject_corruption() {
    let bytes = VECTORS[0].bytes();
    for protocol in [
        "",
        "NATIVE_POLICY",
        "COMPANY_IDENTITY_CAPABILITY_V1",
        "COMPANY_HR_APPOINTMENT_CAPABILITY_V2",
    ] {
        for codec in [1, 0, 2, -1, i16::MAX] {
            let e = decode_native_hr_policy_command_v1(protocol, codec, &bytes)
                .err()
                .unwrap();
            assert_eq!(e.kind, console_kernel_core::ErrorKind::Validation);
            assert_eq!(e.message, "invalid native HR policy command");
        }
    }
    for codec in [0, 2, 3, 4, -1, i16::MAX] {
        let e = decode_native_hr_policy_command_v1(PROTOCOL, codec, &bytes)
            .err()
            .unwrap();
        assert_eq!(e.kind, console_kernel_core::ErrorKind::Validation);
        assert_eq!(e.message, "invalid native HR policy command");
    }
    for v in VECTORS {
        for offset in 0..44 {
            let mut b = v.bytes();
            b[offset] ^= 1;
            reject(&b);
        }
        for operation in [0, 4, 255] {
            changed(v, "operation", &[operation]);
        }
        for operation in [1, 2, 3] {
            if operation != v.operation {
                changed(v, "operation", &[operation]);
            }
        }
        changed(v, "manifest", &[0; 32]);
        for &(name, _, _) in v.fields.iter().filter(|x| x.0.ends_with(".flag")) {
            changed(v, name, &[2]);
            changed(v, name, &[255]);
        }
        if v.operation != 1 {
            for role in [0, 4, 255] {
                changed(v, "role", &[role]);
            }
            if v.recipient == 1 {
                changed(v, "role", &[2]);
            }
            if v.recipient == 7 {
                changed(v, "role", &[1]);
            }
        }
        for codec in 1..=4 {
            assert!(console_identity_application::company_policy::workflow::NativePolicyCommand::decode(codec, &v.bytes()).is_err());
        }
    }
    changed(&VECTORS[0], "epoch", &2u64.to_be_bytes());
    changed(&VECTORS[1], "epoch", &1u64.to_be_bytes());
    for v in &VECTORS[2..] {
        changed(v, "epoch", &1u64.to_be_bytes());
    }
    for v in &VECTORS[2..22] {
        for micros in [
            i64::MIN,
            i64::MAX,
            -62135596800000001,
            253402268340000001,
            1791849600000001,
        ] {
            changed(v, "expiry", &micros.to_be_bytes());
        }
        if v.recipient == 1 {
            changed(v, "pair.flag", &[1]);
            changed(v, "recipient", uuid(7).as_bytes());
        } else {
            changed(v, "pair.flag", &[0]);
            changed(v, "recipient", uuid(1).as_bytes());
        }
    }
    // Nonmanifest digests are parsed data; current factual comparisons belong to owners.
    for digest in [[0; 32], [0xff; 32]] {
        let mut b = VECTORS[2].bytes();
        let offset = VECTORS[2].at("recipient_source_digest");
        b[offset..offset + 32].copy_from_slice(&digest);
        assert_eq!(
            decode_native_hr_policy_command_v1(PROTOCOL, 1, &b)
                .unwrap()
                .1
                .recipient_source_digest,
            Some(digest)
        );
    }
}
#[test]
fn hr_policy_admin_source_and_assignment_refs_reject_corruption() {
    for v in VECTORS {
        for &(field, _, width) in v
            .fields
            .iter()
            .filter(|x| !x.0.starts_with("B.") && !x.0.starts_with("C."))
        {
            if width == 16 {
                changed(v, field, &[0; 16]);
            }
            if width == 8 && field != "expiry" {
                changed(v, field, &0u64.to_be_bytes());
                changed(v, field, &(i64::MAX as u64 + 1).to_be_bytes());
            }
        }
        changed(v, "admin_source.tag", &[0]);
        changed(v, "admin_source.tag", &[2]);
        changed(v, "actor", uuid(7).as_bytes());
        changed(v, "admin_source.administrator_b", uuid(7).as_bytes());
        changed(v, "company", uuid(0xface).as_bytes());
        for field in [
            "root_assignment_revision",
            "root_role_revision",
            "assign_identity.registration_revision",
            "read_policy.registration_revision",
        ] {
            changed(v, &format!("admin_source.{field}"), &2u64.to_be_bytes());
        }
        for action in ["assign_identity", "read_policy"] {
            changed(
                v,
                &format!("admin_source.{action}.manifest_digest"),
                &[0xff; 32],
            );
        }
        changed(
            v,
            "admin_assignment.flag",
            &[if v.operation == 1 { 1 } else { 0 }],
        );
        if v.operation != 2 {
            changed(v, "target_assignment.flag", &[if v.target { 0 } else { 1 }]);
        }
    }
}
#[test]
fn hr_policy_qualification_snapshot_pair_rejects_corruption() {
    for v in &VECTORS[6..22] {
        for who in ["B", "C"] {
            let field = |tail: &str| format!("{who}.{tail}");
            for bad in [0u16, 600, 602, 608, 610, 65535] {
                changed(v, &field("length"), &bad.to_be_bytes());
            }
            let valid = if who == "B" { v.bvalid } else { v.cvalid };
            changed(
                v,
                &field("length"),
                &(if valid { 601u16 } else { 609u16 }).to_be_bytes(),
            );
            for offset in 0..40 {
                let mut b = v.bytes();
                b[v.at(&field("prefix")) + offset] ^= 1;
                reject(&b);
            }
            changed(v, &field("company"), uuid(8).as_bytes());
            changed(
                v,
                &field("account"),
                uuid(if who == "B" { 7 } else { 1 }).as_bytes(),
            );
            changed(v, &field("human_anchor"), uuid(9).as_bytes());
            changed(v, &field("qualification.person_id"), uuid(9).as_bytes());
            for &(name, _, width) in v
                .fields
                .iter()
                .filter(|x| x.0.starts_with(&format!("{who}.")))
            {
                if width == 16 {
                    changed(v, name, &[0; 16]);
                }
                if width == 8 && !name.ends_with("individual_valid_until.value") {
                    changed(v, name, &0u64.to_be_bytes());
                    changed(v, name, &(i64::MAX as u64 + 1).to_be_bytes());
                }
            }
            changed(
                v,
                &field("individual_valid_until.flag"),
                &[u8::from(!valid)],
            );
            if valid {
                for micros in [i64::MIN, i64::MAX, -62135596800000001, 253402300800000000] {
                    changed(
                        v,
                        &field("individual_valid_until.value"),
                        &micros.to_be_bytes(),
                    );
                }
            }
        }
        let mut equal_people = v.bytes();
        for name in ["human_anchor", "qualification.person_id"] {
            let offset = v.at(&format!("C.{name}"));
            equal_people[offset..offset + 16].copy_from_slice(uuid(32).as_bytes());
        }
        reject(&equal_people);
        // Change a syntactically valid factual digest and require it to be preserved.
        let mut b = v.bytes();
        let offset = v.at("B.qualification.observation_digest");
        b[offset..offset + 32].fill(0xff);
        let pair = decode_native_hr_policy_command_v1(PROTOCOL, 1, &b)
            .unwrap()
            .1
            .qualification_pair
            .unwrap();
        assert_eq!(pair.0.qualification.observation_digest, [0xff; 32]);
    }
}
#[test]
fn hr_policy_all_truncations_and_trailing_bytes_reject() {
    assert_eq!(VECTORS.len(), 26);
    for v in VECTORS {
        let b = v.bytes();
        for end in 0..b.len() {
            reject(&b[..end]);
        }
        for extra in [&[0u8][..], &[0xff][..], &[0, 1][..]] {
            let mut trailing = b.clone();
            trailing.extend_from_slice(extra);
            reject(&trailing);
        }
    }
}
