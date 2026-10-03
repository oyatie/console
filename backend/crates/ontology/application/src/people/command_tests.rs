//! Exact command bytes, selector validation, and strict decoding.
use crate::people::DirectoryRegistrationInput;
use crate::people::{DirectoryExpectationsV1, NativeDirectoryCommandV1 as Command};
use console_kernel_core::{AccountId, OrgId};
use uuid::Uuid;

const IDS: [&str; 8] = [
    "11111111-1111-4111-8111-111111111111",
    "22222222-2222-4222-8222-222222222222",
    "33333333-3333-4333-8333-333333333333",
    "44444444-4444-4444-8444-444444444444",
    "55555555-5555-4555-8555-555555555555",
    "66666666-6666-4666-8666-666666666666",
    "77777777-7777-4777-8777-777777777777",
    "88888888-8888-4888-8888-888888888888",
];
fn id(index: usize) -> Uuid {
    Uuid::parse_str(IDS[index]).unwrap()
}
fn actor() -> AccountId {
    AccountId::from_uuid(id(0)).unwrap()
}
fn expected() -> DirectoryExpectationsV1 {
    DirectoryExpectationsV1 {
        company_epoch: 7,
        object_type_id: id(4),
        action_type_id: id(5),
        action_revision: 3,
        schema_revision: 5,
        legal_name_property_id: id(6),
        employee_number_property_id: id(7),
    }
}
fn command(name: &str, number: &str) -> Command {
    Command::new(
        id(2),
        OrgId::from_uuid(id(1)),
        id(3),
        expected(),
        DirectoryRegistrationInput::new(name, number).unwrap(),
    )
    .unwrap()
}
fn unhex(text: &str) -> Vec<u8> {
    text.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
        .collect()
}
// Literal fixtures generated independently from the published field/offset spec.
const ASCII: &str = "636f6e736f6c652e70656f706c652e6469726563746f72792d72656769737465720000011111111111114111811111111111111122222222222242228222222222222222333333333333433383333333333333334444444444444444844444444444444400000000000000075555555555554555855555555555555566666666666646668666666666666666000000000000000300000000000000057777777777774777877777777777777788888888888848888888888888888888591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e00034b696d00054e2d303031";
const KOREAN: &str = "636f6e736f6c652e70656f706c652e6469726563746f72792d72656769737465720000011111111111114111811111111111111122222222222242228222222222222222333333333333433383333333333333334444444444444444844444444444444400000000000000075555555555554555855555555555555566666666666646668666666666666666000000000000000300000000000000057777777777774777877777777777777788888888888848888888888888888888591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e001beab980ed9598eb8a98203cec97b0eab5ac202620ec9ab4ec98813e000d55492dec82aceb9e8c2d303031";
fn text_bytes(name: &[u8], number: &[u8]) -> Vec<u8> {
    let mut b = unhex(ASCII)[..220].to_vec();
    for value in [name, number] {
        b.extend_from_slice(&(value.len() as u16).to_be_bytes());
        b.extend_from_slice(value);
    }
    b
}

#[test]
fn independent_directory_vectors_preserve_bytes_and_all_expectations() {
    for (name, number, golden, length) in [
        ("Kim", "N-001", ASCII, 232),
        ("김하늘 <연구 & 운영>", "UI-사람-001", KOREAN, 264),
    ] {
        let bytes = unhex(golden);
        let candidate = command(name, number);
        assert_eq!(bytes.len(), length);
        assert_eq!(candidate.encode(actor()), bytes);
        let (who, decoded) = Command::decode(&bytes).unwrap();
        assert_eq!(who, actor());
        assert_eq!(decoded, candidate);
        assert_eq!(decoded.encode(who), bytes);
        assert_eq!(decoded.expected(), expected());
        assert_eq!(decoded.employee_id(), id(3));
        let mut effect = b"console.people.directory-effect\0\0\x01".to_vec();
        effect.extend_from_slice(&bytes);
        assert_eq!(decoded.effect_payload(who), effect);
        assert_eq!(bytes, unhex(golden));
    }
}

#[test]
fn directory_decoder_rejects_every_truncation_trailing_bytes_and_wrong_protocol() {
    for original in [unhex(ASCII), unhex(KOREAN)] {
        for end in 0..original.len() {
            assert!(Command::decode(&original[..end]).is_err(), "prefix {end}");
        }
        for tail in [0, 1, 255] {
            let mut b = original.clone();
            b.push(tail);
            assert!(Command::decode(&b).is_err());
        }
        for offset in 0..36 {
            let mut b = original.clone();
            b[offset] ^= 1;
            assert!(Command::decode(&b).is_err(), "prefix byte {offset}");
        }
        for offset in 188..220 {
            let mut b = original.clone();
            b[offset] ^= 1;
            assert!(Command::decode(&b).is_err(), "manifest byte {offset}");
        }
        assert_eq!(
            original,
            if original.len() == 232 {
                unhex(ASCII)
            } else {
                unhex(KOREAN)
            }
        );
    }
    for prefix in [
        b"console.company.people-policy\0\0\x01".as_slice(),
        b"console.company.payroll-policy\0\0\x01".as_slice(),
    ] {
        let mut b = prefix.to_vec();
        b.extend_from_slice(&unhex(ASCII)[36..]);
        assert!(Command::decode(&b).is_err());
    }
}

#[test]
fn directory_decoder_rejects_nil_identities_bad_revisions_and_property_alias() {
    for start in [36, 52, 68, 84, 108, 124, 156, 172] {
        let mut b = unhex(ASCII);
        b[start..start + 16].fill(0);
        assert!(Command::decode(&b).is_err(), "uuid {start}");
    }
    for start in [100, 140, 148] {
        for value in [0u64, 1u64 << 63, u64::MAX] {
            let mut b = unhex(ASCII);
            b[start..start + 8].copy_from_slice(&value.to_be_bytes());
            assert!(Command::decode(&b).is_err());
        }
        for value in [1u64, i64::MAX as u64] {
            let mut b = unhex(ASCII);
            b[start..start + 8].copy_from_slice(&value.to_be_bytes());
            let (who, parsed) = Command::decode(&b).unwrap();
            assert_eq!(parsed.encode(who), b);
        }
    }
    let mut b = unhex(ASCII);
    let name = b[156..172].to_vec();
    b[172..188].copy_from_slice(&name);
    assert!(Command::decode(&b).is_err());
}

#[test]
fn directory_decoder_refuses_noncanonical_text_without_normalizing_history() {
    for value in [
        "",
        " ",
        " Kim",
        "Kim ",
        "\u{2003}Kim",
        "Kim\u{a0}",
        "\tKim",
        "Ki\n m",
        "\0",
        "\u{7f}",
        "\u{85}",
        "\u{9f}",
    ] {
        assert!(
            Command::decode(&text_bytes(value.as_bytes(), b"1")).is_err(),
            "name {value:?}"
        );
        assert!(
            Command::decode(&text_bytes(b"Name", value.as_bytes())).is_err(),
            "number {value:?}"
        );
    }
    for raw in [
        vec![0xff],
        vec![0xc0, 0xaf],
        vec![0xed, 0xa0, 0x80],
        vec![0xf4, 0x90, 0x80, 0x80],
    ] {
        assert!(Command::decode(&text_bytes(&raw, b"1")).is_err());
        assert!(Command::decode(&text_bytes(b"Name", &raw)).is_err());
    }
    for text in ["é", "e\u{301}", "Ａ", "A", "ab", "AB", "a b", "ab"] {
        let bytes = text_bytes(text.as_bytes(), text.as_bytes());
        let (who, parsed) = Command::decode(&bytes).unwrap();
        assert_eq!(parsed.input().legal_name(), text);
        assert_eq!(parsed.input().employee_number(), text);
        assert_eq!(parsed.encode(who), bytes);
    }
}

#[test]
fn directory_codec_enforces_scalar_and_byte_limits_and_exact_lengths() {
    let max = command(&"🧑".repeat(200), &"🧑".repeat(64)).encode(actor());
    assert_eq!(max.len(), 1280);
    assert_eq!(Command::decode(&max).unwrap().1.encode(actor()), max);
    for character in ['a', '가', '🧑'] {
        assert!(
            Command::decode(&text_bytes(
                character.to_string().repeat(201).as_bytes(),
                b"1"
            ))
            .is_err()
        );
        assert!(
            Command::decode(&text_bytes(
                b"Name",
                character.to_string().repeat(65).as_bytes()
            ))
            .is_err()
        );
    }
    for length in [0u16, 2, 4, 801, u16::MAX] {
        let mut b = unhex(ASCII);
        b[220..222].copy_from_slice(&length.to_be_bytes());
        assert!(Command::decode(&b).is_err());
    }
    for length in [0u16, 4, 6, 257, u16::MAX] {
        let mut b = unhex(ASCII);
        b[225..227].copy_from_slice(&length.to_be_bytes());
        assert!(Command::decode(&b).is_err());
    }
}

#[test]
fn directory_each_selector_remains_in_identity_and_allocation_is_not_replaced() {
    let original = unhex(ASCII);
    for start in [36, 52, 68, 84, 108, 124, 156, 172] {
        let mut b = original.clone();
        b[start] ^= 0x04;
        let (who, parsed) = Command::decode(&b).unwrap();
        assert_ne!(parsed.encode(who), original);
        assert_eq!(parsed.encode(who), b);
    }
    for start in [100, 140, 148] {
        let mut b = original.clone();
        b[start + 7] += 1;
        let (who, parsed) = Command::decode(&b).unwrap();
        assert_eq!(parsed.encode(who), b);
        assert_ne!(b, original);
    }
    let trimmed = command(" \u{2003}Kim\u{a0} ", " N-001 ");
    assert_eq!(trimmed.encode(actor()), original);
    // Trimming is accepted-input construction, never persisted-byte decoding.
    assert_eq!(Command::decode(&original).unwrap().1.employee_id(), id(3));
}

#[test]
fn directory_constructor_rechecks_mutable_expectations_and_nonnil_identities() {
    let input = DirectoryRegistrationInput::new("Kim", "1").unwrap();
    for (command_id, company, employee) in [
        (Uuid::nil(), id(1), id(3)),
        (id(2), Uuid::nil(), id(3)),
        (id(2), id(1), Uuid::nil()),
    ] {
        assert!(
            Command::new(
                command_id,
                OrgId::from_uuid(company),
                employee,
                expected(),
                input.clone()
            )
            .is_err()
        );
    }
    for field in 0..7 {
        let mut e = expected();
        match field {
            0 => e.company_epoch = 0,
            1 => e.object_type_id = Uuid::nil(),
            2 => e.action_type_id = Uuid::nil(),
            3 => e.action_revision = u64::MAX,
            4 => e.schema_revision = 0,
            5 => e.legal_name_property_id = Uuid::nil(),
            _ => e.employee_number_property_id = e.legal_name_property_id,
        }
        assert!(Command::new(id(2), OrgId::from_uuid(id(1)), id(3), e, input.clone()).is_err());
    }
}

#[test]
fn directory_platform_sentinel_is_not_a_company_in_construction_or_decoding() {
    let platform = OrgId::from_uuid(Uuid::from_u128(0xface));
    assert_eq!(platform, OrgId::platform());
    assert!(!platform.as_uuid().is_nil());
    assert!(
        Command::new(
            id(2),
            platform,
            id(3),
            expected(),
            DirectoryRegistrationInput::new("Kim", "N-001").unwrap()
        )
        .is_err()
    );
    let mut wire = unhex(ASCII);
    wire[52..68].copy_from_slice(Uuid::from_u128(0xface).as_bytes());
    assert!(Command::decode(&wire).is_err());
    assert!(Command::decode(&unhex(ASCII)).is_ok());
    assert!(Command::decode(&unhex(KOREAN)).is_ok());
}
