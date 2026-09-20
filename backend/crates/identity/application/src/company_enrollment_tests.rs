// Proposed tests for the actual identity application CompanyEnrollmentV1 boundary.
// No parallel fake endpoint, synthetic success owner, or production stub.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use super::CompanyEnrollmentV1;
use serde_json::{Value, json};
use uuid::Uuid;

const ACCOUNT: &str = "11111111-1111-4111-8111-111111111111";
const COMMAND: &str = "22222222-2222-4222-8222-222222222222";
const ADMIN: &str = "33333333-3333-4333-8333-333333333333";
fn account() -> Uuid {
    Uuid::parse_str(ACCOUNT).unwrap()
}
fn input() -> Value {
    json!({"command_id":COMMAND,"group_id":null,"slug":"han","name":"Han Company","administrative_account_id":ADMIN})
}
fn parse(value: &Value) -> Result<CompanyEnrollmentV1, console_kernel_core::KernelError> {
    CompanyEnrollmentV1::from_json_slice(&serde_json::to_vec(value).unwrap())
}
fn unhex(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0);
    value
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}
// Independently encoded, fixed vectors; expected bytes never call the owner encoder.
const VECTORS: &[(&str, &str, &str)] = &[
    (
        "ascii",
        r###"{"command_id":"22222222-2222-4222-8222-222222222222","group_id":null,"slug":"han","name":"Han Company","administrative_account_id":"33333333-3333-4333-8333-333333333333"}"###,
        "636f6e736f6c652e636f6d70616e792e656e726f6c6c6d656e74000001111111111111411181111111111111112222222222224222822222222222222200333333333333433383333333333333330000000368616e0000000b48616e20436f6d70616e79",
    ),
    (
        "korean",
        r###"{"command_id":"22222222-2222-4222-8222-222222222222","group_id":null,"slug":"han-seoul","name":"한 회사","administrative_account_id":"33333333-3333-4333-8333-333333333333"}"###,
        "636f6e736f6c652e636f6d70616e792e656e726f6c6c6d656e74000001111111111111411181111111111111112222222222224222822222222222222200333333333333433383333333333333330000000968616e2d73656f756c0000000aed959c20ed9a8cec82ac",
    ),
    (
        "escaped",
        r###"{"command_id":"22222222-2222-4222-8222-222222222222","group_id":null,"slug":"han-2","name":"회사 \"한\" \\ 본사","administrative_account_id":"33333333-3333-4333-8333-333333333333"}"###,
        "636f6e736f6c652e636f6d70616e792e656e726f6c6c6d656e74000001111111111111411181111111111111112222222222224222822222222222222200333333333333433383333333333333330000000568616e2d3200000015ed9a8cec82ac2022ed959c22205c20ebb3b8ec82ac",
    ),
    (
        "preserved_spaces",
        r###"{"command_id":"22222222-2222-4222-8222-222222222222","group_id":null,"slug":"han","name":"  한 회사  ","administrative_account_id":"33333333-3333-4333-8333-333333333333"}"###,
        "636f6e736f6c652e636f6d70616e792e656e726f6c6c6d656e74000001111111111111411181111111111111112222222222224222822222222222222200333333333333433383333333333333330000000368616e0000000e2020ed959c20ed9a8cec82ac2020",
    ),
    (
        "nonnull_group",
        r###"{"command_id":"22222222-2222-4222-8222-222222222222","group_id":"44444444-4444-4444-8444-444444444444","slug":"han","name":"Han Company","administrative_account_id":"33333333-3333-4333-8333-333333333333"}"###,
        "636f6e736f6c652e636f6d70616e792e656e726f6c6c6d656e7400000111111111111141118111111111111111222222222222422282222222222222220144444444444444448444444444444444333333333333433383333333333333330000000368616e0000000b48616e20436f6d70616e79",
    ),
];

#[test]
fn accepted_vectors_match_exact_independent_bytes_and_decode() {
    for (label, source, hex) in VECTORS {
        let input = CompanyEnrollmentV1::from_json_slice(source.as_bytes()).unwrap();
        let expected = unhex(hex);
        assert_eq!(input.encode(account()).unwrap(), expected, "{label}");
        let (decoded_account, decoded) = CompanyEnrollmentV1::decode(&expected).unwrap();
        assert_eq!(decoded_account, account(), "{label}");
        assert_eq!(decoded, input, "{label}");
        assert_eq!(
            decoded.encode(decoded_account).unwrap(),
            expected,
            "{label}"
        );
    }
}
#[test]
fn json_shape_rejects_missing_duplicate_unknown_or_wrong_members() {
    let valid = input();
    for key in [
        "command_id",
        "group_id",
        "slug",
        "name",
        "administrative_account_id",
    ] {
        let mut value = valid.clone();
        value.as_object_mut().unwrap().remove(key);
        assert!(parse(&value).is_err(), "missing {key}");
        let original = serde_json::to_string(&valid).unwrap();
        let duplicate = format!(
            "{{{}:{},{}",
            serde_json::to_string(key).unwrap(),
            valid[key],
            &original[1..]
        );
        assert!(
            CompanyEnrollmentV1::from_json_slice(duplicate.as_bytes()).is_err(),
            "duplicate {key}"
        );
    }
    let mut extra = valid.clone();
    extra["authorized"] = json!(true);
    assert!(parse(&extra).is_err());
    // serde struct sequence deserialization must not admit a five-value array.
    assert!(parse(&json!([COMMAND, null, "han", "Han Company", ADMIN])).is_err());
    let original = serde_json::to_string(&valid).unwrap();
    let escaped_duplicate = format!(r#"{{"na\u006de":"Other",{}"#, &original[1..]);
    assert!(CompanyEnrollmentV1::from_json_slice(escaped_duplicate.as_bytes()).is_err());
    let escaped_unique = original.replacen("\"command_id\"", r#""\u0063ommand_id""#, 1);
    assert_eq!(
        CompanyEnrollmentV1::from_json_slice(escaped_unique.as_bytes()).unwrap(),
        parse(&valid).unwrap()
    );
    for value in [json!(null), json!([]), json!(true), json!("input")] {
        assert!(parse(&value).is_err());
    }
    for (key, value) in [
        ("slug", json!(null)),
        ("name", json!(7)),
        ("command_id", json!([])),
        ("administrative_account_id", json!(false)),
        ("group_id", json!(7)),
    ] {
        let mut bad = valid.clone();
        bad[key] = value;
        assert!(parse(&bad).is_err(), "type {key}");
    }
}
#[test]
fn uuid_wire_is_canonical_non_nil_and_null_group_remains_distinct() {
    for key in ["command_id", "administrative_account_id", "group_id"] {
        for value in [
            "00000000-0000-0000-0000-000000000000",
            "AAAAAAAA-AAAA-4AAA-8AAA-AAAAAAAAAAAA",
            "11111111111141118111111111111111",
            "{11111111-1111-4111-8111-111111111111}",
            "11111111-1111-4111-8111-111111111111\n",
            "not-a-uuid",
            "",
        ] {
            let mut bad = input();
            bad[key] = json!(value);
            assert!(parse(&bad).is_err(), "{key}={value:?}");
        }
    }
    let mut nil_admin = input();
    nil_admin["administrative_account_id"] = Value::Null;
    assert!(parse(&nil_admin).is_err());
    assert!(parse(&input()).is_ok());
    let valid = parse(&input()).unwrap();
    assert!(valid.encode(Uuid::nil()).is_err());
}
#[test]
fn names_are_utf8_byte_bounded_nonblank_control_free_and_unmodified() {
    for name in [
        "A".repeat(256),
        format!("{}A", "한".repeat(85)),
        "  한 회사  ".to_owned(),
        "회사 \"한\" \\ 본사".to_owned(),
    ] {
        let mut value = input();
        value["name"] = json!(name);
        let command = parse(&value).unwrap();
        let encoded = command.encode(account()).unwrap();
        assert!(encoded.ends_with(name.as_bytes()));
        assert_eq!(CompanyEnrollmentV1::decode(&encoded).unwrap().1, command);
    }
    for name in [
        String::new(),
        " \t ".to_owned(),
        "\u{00a0}".to_owned(),
        "A".repeat(257),
        format!("{}AA", "한".repeat(85)),
        "한\n회사".to_owned(),
        "한\0회사".to_owned(),
        "한\u{007f}회사".to_owned(),
        "한\u{0085}회사".to_owned(),
    ] {
        let mut bad = input();
        bad["name"] = json!(name);
        assert!(parse(&bad).is_err(), "{name:?}");
    }
}
#[test]
fn slugs_use_exact_ascii_grammar_and_byte_limit() {
    for slug in [
        "a".to_owned(),
        "0".to_owned(),
        "a-0".to_owned(),
        "a".repeat(63),
    ] {
        let mut value = input();
        value["slug"] = json!(slug);
        assert!(parse(&value).is_ok());
    }
    for slug in [
        String::new(),
        "a".repeat(64),
        "-a".to_owned(),
        "a-".to_owned(),
        "A".to_owned(),
        "한".to_owned(),
        "a_b".to_owned(),
        " a".to_owned(),
        "a ".to_owned(),
        "a\n".to_owned(),
        "a/b".to_owned(),
    ] {
        let mut value = input();
        value["slug"] = json!(slug);
        assert!(parse(&value).is_err(), "{slug:?}");
    }
}
#[test]
fn input_parse_enforces_exact_4096_byte_budget_and_json_exhaustion() {
    let source = serde_json::to_vec(&input()).unwrap();
    let mut exact = source.clone();
    exact.resize(4096, b' ');
    assert!(CompanyEnrollmentV1::from_json_slice(&exact).is_ok());
    exact.push(b' ');
    assert!(CompanyEnrollmentV1::from_json_slice(&exact).is_err());
    let mut two = source.clone();
    two.extend_from_slice(&source);
    assert!(CompanyEnrollmentV1::from_json_slice(&two).is_err());
    let mut invalid = source;
    invalid.push(0xff);
    assert!(CompanyEnrollmentV1::from_json_slice(&invalid).is_err());
}
#[test]
fn codec_refuses_every_truncation_and_any_trailing_bytes() {
    for (label, _, hex) in VECTORS {
        let valid = unhex(hex);
        for len in 0..valid.len() {
            assert!(
                CompanyEnrollmentV1::decode(&valid[..len]).is_err(),
                "{label}/{len}"
            );
        }
        for suffix in [&[0u8][..], &[0, 1, 2][..], &b" "[..]] {
            let mut extra = valid.clone();
            extra.extend_from_slice(suffix);
            assert!(CompanyEnrollmentV1::decode(&extra).is_err(), "{label}");
        }
    }
}
#[test]
fn codec_refuses_invalid_header_discriminator_ids_lengths_utf8_and_fields() {
    // Offsets are independent format constants from golden-vectors.json layout.
    // header=27 bytes incl NUL; version=2; account=16; command=16;
    // group-present=1; admin=16; slug length=4; slug=3; name length=4.
    let valid = unhex(VECTORS[0].2);
    for (offset, replacement) in [
        (0, b'X'),
        (26, b'X'),
        (27, 1),
        (28, 2),
        (61, 2),
        (82, b'A'),
        (89, 0xff),
        (89, 0),
    ] {
        let mut bad = valid.clone();
        bad[offset] = replacement;
        assert!(
            CompanyEnrollmentV1::decode(&bad).is_err(),
            "offset {offset}"
        );
    }
    for start in [29, 45, 62] {
        let mut nil = valid.clone();
        nil[start..start + 16].fill(0);
        assert!(CompanyEnrollmentV1::decode(&nil).is_err(), "nil at {start}");
    }
    for start in [78, 85] {
        for length in [0u32, u32::MAX, 65536] {
            let mut bad = valid.clone();
            bad[start..start + 4].copy_from_slice(&length.to_be_bytes());
            assert!(
                CompanyEnrollmentV1::decode(&bad).is_err(),
                "length {start}/{length}"
            );
        }
    }
    let mut nil_group = unhex(VECTORS[4].2);
    nil_group[62..78].fill(0);
    assert!(CompanyEnrollmentV1::decode(&nil_group).is_err());
}
