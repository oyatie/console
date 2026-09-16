//! Additive protocol and schema refusal cases; no authorized context producer.
use super::{
    BoundedValue, NormalizedOwnerSubmission, decode_owner_submission, encode_owner_submission,
};
use serde::de::DeserializeSeed;
use serde_json::{Value, json};
use std::io::Cursor;

fn positive() -> Vec<u8> {
    include_bytes!("codec-goldens/owner-gated-calculate.bin").to_vec()
}
fn frame(body: &[u8]) -> Vec<u8> {
    let mut bytes = b"console.owner28.submission.v1\0".to_vec();
    bytes.extend_from_slice(&u32::try_from(body.len()).unwrap().to_be_bytes());
    bytes.extend_from_slice(body);
    bytes
}
fn mutated(change: impl FnOnce(&mut Value)) -> Vec<u8> {
    let mut value: Value = serde_json::from_slice(&positive()[34..]).unwrap();
    change(&mut value);
    frame(&serde_json::to_vec(&value).unwrap())
}
#[test]
fn owner_protocol_truncation_every_boundary_and_trailing_bytes() {
    let complete = positive();
    assert!(decode_owner_submission(&mut Cursor::new(&complete)).is_ok());
    for end in 0..complete.len() {
        assert!(
            decode_owner_submission(&mut Cursor::new(&complete[..end])).is_err(),
            "accepted truncation {end}"
        );
    }
    let mut trailing = complete;
    trailing.push(0);
    assert!(decode_owner_submission(&mut Cursor::new(trailing)).is_err());
}
#[test]
fn owner_protocol_rejects_tag_oversize_and_noncanonical_json() {
    let mut tag = positive();
    tag[0] ^= 1;
    assert!(decode_owner_submission(&mut Cursor::new(tag)).is_err());
    let mut oversized = positive();
    oversized[30..34].copy_from_slice(&u32::MAX.to_be_bytes());
    assert!(decode_owner_submission(&mut Cursor::new(oversized)).is_err());
    let mut padded = b" ".to_vec();
    padded.extend_from_slice(&positive()[34..]);
    assert!(decode_owner_submission(&mut Cursor::new(frame(&padded))).is_err());
    assert!(decode_owner_submission(&mut Cursor::new(frame(&[0xff]))).is_err());
}
#[test]
fn owner_protocol_schema_and_identity_refusals_do_not_use_fixture_matching() {
    for change in [
        |v: &mut Value| {
            v["body"]["unknown_field"] = json!(true);
            v["expected"]["registered_input_digest"] =
                json!("0c86b3e7b5fb9dfd689d943134d109e3e4ddeb1a98fc18ce800c5561903f15b2");
        },
        |v: &mut Value| {
            v["input_schema_ref"]["revision"] = json!("9223372036854775808");
        },
        |v: &mut Value| {
            v["org_id"] = json!("00000000-0000-0000-0000-000000000000");
        },
        |v: &mut Value| {
            v["input_schema_ref"]["org_id"] = json!("11111111-1111-1111-1111-111111111111");
        },
        |v: &mut Value| {
            v["owner_key"] = json!("unregistered-owner");
        },
        |v: &mut Value| {
            v["owner_action"] = json!("arbitrary.execute");
        },
        |v: &mut Value| {
            v["reason"] = json!("mismatched reason");
        },
        |v: &mut Value| {
            v["expected"]["registered_input_digest"] = json!("f".repeat(64));
        },
        |v: &mut Value| {
            v["target"]["object_kind"] = json!("arbitrary-kind");
        },
    ] {
        assert!(decode_owner_submission(&mut Cursor::new(mutated(change))).is_err());
    }
    // A changed plain command identity remains valid and gets a new fingerprint;
    // UUID literals are not a fixture allowlist and never authorize an action.
    let changed = mutated(|v| v["command_id"] = json!("11111111-1111-1111-1111-111111111111"));
    let command = decode_owner_submission(&mut Cursor::new(changed))
        .unwrap()
        .command;
    let original = decode_owner_submission(&mut Cursor::new(positive()))
        .unwrap()
        .command;
    let a = encode_owner_submission(&command, &mut Vec::new()).unwrap();
    let b = encode_owner_submission(&original, &mut Vec::new()).unwrap();
    assert_ne!(a.command_fingerprint, b.command_fingerprint);
}
#[test]
fn owner_protocol_duplicate_nested_keys_and_container_limits_refuse() {
    let body = String::from_utf8(positive()[34..].to_vec()).unwrap();
    let duplicate = body.replacen(
        "\"run_revision\":\"1\"",
        "\"run_revision\":\"1\",\"run_revision\":\"1\"",
        1,
    );
    assert_ne!(duplicate, body);
    assert!(decode_owner_submission(&mut Cursor::new(frame(duplicate.as_bytes()))).is_err());
    // Exercise the actual pre-schema parser directly, with accepted boundaries.
    fn bounded(text: &str) -> bool {
        let mut remaining = super::MAX_FRAME;
        BoundedValue {
            depth: 1,
            remaining: &mut remaining,
        }
        .deserialize(&mut serde_json::Deserializer::from_str(text))
        .is_ok()
    }
    assert!(bounded(&format!("{}null{}", "[".repeat(8), "]".repeat(8))));
    assert!(!bounded(&format!("{}null{}", "[".repeat(9), "]".repeat(9))));
    assert!(bounded(&serde_json::to_string(&vec!["x"; 2000]).unwrap()));
    assert!(!bounded(&serde_json::to_string(&vec!["x"; 2001]).unwrap()));
    let fields =
        |count| Value::Object((0..count).map(|i| (format!("f{i}"), Value::Null)).collect());
    assert!(bounded(&serde_json::to_string(&fields(128)).unwrap()));
    assert!(!bounded(&serde_json::to_string(&fields(129)).unwrap()));
    assert!(serde_json::from_str::<NormalizedOwnerSubmission>("{\"bad\\u0000key\":null}").is_err());
}
#[test]
fn owner_protocol_shared_parse_budget_and_numeric_diagnostics_are_bounded() {
    use serde::de::DeserializeSeed;
    // Two short strings fit only when their shared container/string budget fits.
    fn parse(remaining: &mut usize) -> bool {
        super::BoundedValue {
            depth: 1,
            remaining,
        }
        .deserialize(&mut serde_json::Deserializer::from_str("[\"aa\",\"bb\"]"))
        .is_ok()
    }
    assert!(parse(&mut 10));
    assert!(!parse(&mut 9));
    for input in ["987654321", "-987654321", "987654321.25"] {
        let error = serde_json::from_str::<super::NormalizedOwnerSubmission>(input).unwrap_err();
        assert!(!error.to_string().contains("987654321"));
    }
}
