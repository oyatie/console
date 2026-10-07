//! Pure codec compatibility only; no SQL/current-authority or UI acceptance.
//! Design V2 SHA256 999c457cae8cbae8640c6dc55e0819e0e2ee0cbba36e0ec67ed895b630255c8a.
use super::{org_unit_codec3_literal_bytes, org_unit_codec3_literals};
use crate::company_policy::{
    AccountId, business::NativeBusinessOperationV1, workflow::NativePolicyCommand,
};
use console_kernel_core::{ErrorKind, OrgId};
use serde_json::Value;
use std::collections::BTreeSet;
use time::OffsetDateTime;
use uuid::Uuid;

#[path = "company_information_codec4_literal_fixtures.rs"]
mod literals;

fn positives() -> Vec<Value> {
    let values: Vec<Value> = serde_json::from_str(literals::POSITIVE_JSON).unwrap();
    assert_eq!(values.len(), 8, "exact positive codec4 census changed");
    values
}

fn bytes(value: &Value) -> Vec<u8> {
    let input = org_unit_codec3_literal_bytes(value["hex"].as_str().unwrap());
    assert_eq!(
        input.len(),
        usize::try_from(value["bytes"].as_u64().unwrap()).unwrap()
    );
    input
}

fn assert_literal(value: &Value) {
    let input = bytes(value);
    let (actor, command) = NativePolicyCommand::decode(4, &input).expect(
        "COMPANY_INFORMATION_CODEC4_LITERAL: existing shared decoder must decode the approved finite literal",
    );
    let expected = &value["expected"];
    assert_eq!(
        actor,
        AccountId::from_uuid(
            Uuid::parse_str(expected["actor_account_id"].as_str().unwrap()).unwrap()
        )
        .unwrap()
    );
    assert_eq!(command.codec_version(), 4);
    assert_eq!(
        command.company(),
        OrgId::from_uuid(Uuid::parse_str(expected["company_id"].as_str().unwrap()).unwrap())
    );
    assert_eq!(
        command.command_id(),
        Uuid::parse_str(expected["command_id"].as_str().unwrap()).unwrap()
    );
    assert_eq!(
        command.expected_company_epoch(),
        expected["epoch"].as_u64().unwrap()
    );
    assert_eq!(
        command.catalog_version(),
        "native-company-identity-2026-09-19.1"
    );
    assert_eq!(
        command.manifest_digest().as_slice(),
        org_unit_codec3_literal_bytes(
            "0d3d0c3bc0357c0394b02400295f77231178cd5dc22a668a90880fc92a089935"
        )
        .as_slice()
    );
    let operation = match expected["operation"].as_str().unwrap() {
        "Grant" => NativeBusinessOperationV1::Grant,
        "Revoke" => NativeBusinessOperationV1::Revoke,
        _ => panic!("literal contains an undeclared operation"),
    };
    assert_eq!(command.operation(), operation);
    let recipient = expected["recipient_account_id"]
        .as_str()
        .map(|id| AccountId::from_uuid(Uuid::parse_str(id).unwrap()).unwrap());
    assert_eq!(command.recipient_account_id(), recipient);
    let expiry = expected["expires_at_microseconds"]
        .as_i64()
        .map(|value| OffsetDateTime::from_unix_timestamp_nanos(i128::from(value) * 1_000).unwrap());
    assert_eq!(command.expires_at(), expiry);
    // This includes every exact reason, parent/child, field, action and time byte.
    assert_eq!(
        command.encode(actor),
        input,
        "decoder changed the accepted literal"
    );
}

#[test]
fn company_information_codec4_exact_literals_preserve_reason_refs_and_microseconds() {
    let values = positives();
    let mut names = BTreeSet::new();
    for value in values {
        assert!(names.insert(value["name"].as_str().unwrap().to_owned()));
        assert_literal(&value);
    }
    assert_eq!(names.len(), 8);
}

#[test]
fn company_information_codec4_malformed_literals_are_validation_failures() {
    let values = positives();
    // Refusals cannot pass merely because codec4 is unsupported or all input fails.
    for value in &values {
        assert_literal(value);
    }
    let refusals: Vec<Value> = serde_json::from_str(literals::REFUSAL_JSON).unwrap();
    assert_eq!(refusals.len(), 244, "finite refusal census changed");
    let mut names = BTreeSet::new();
    for value in &refusals {
        let name = value["name"].as_str().unwrap();
        assert!(names.insert(name), "duplicate refusal: {name}");
        let codec = i16::try_from(value["codec"].as_i64().unwrap()).unwrap();
        let error = NativePolicyCommand::decode(codec, &bytes(value)).unwrap_err();
        assert_eq!(
            error.kind,
            ErrorKind::Validation,
            "wrong refusal kind: {name}"
        );
    }
    let mut checked = 0;
    for value in &values {
        let input = bytes(value);
        for end in 0..input.len() {
            assert_eq!(
                NativePolicyCommand::decode(4, &input[..end])
                    .unwrap_err()
                    .kind,
                ErrorKind::Validation,
                "truncation of {} at {end} admitted",
                value["name"]
            );
            checked += 1;
        }
    }
    assert_eq!(checked, 8392, "full truncation census changed");
}

#[test]
fn company_information_codec4_keeps_all_predecessor_literal_bytes_and_grammars() {
    let mut checked = 0;
    for (raw, expected_count) in [
        (org_unit_codec3_literals::PREDECESSOR_JSON, 11),
        (org_unit_codec3_literals::POSITIVE_JSON, 10),
    ] {
        let previous: Vec<Value> = serde_json::from_str(raw).unwrap();
        assert_eq!(previous.len(), expected_count);
        for value in previous {
            let codec = i16::try_from(value["codec"].as_i64().unwrap()).unwrap();
            assert!(matches!(codec, 1..=3));
            let input = org_unit_codec3_literal_bytes(value["hex"].as_str().unwrap());
            let (actor, command) = NativePolicyCommand::decode(codec, &input).unwrap();
            assert_eq!(command.codec_version(), codec);
            assert_eq!(command.encode(actor), input);
            assert_eq!(
                NativePolicyCommand::decode(4, &input).unwrap_err().kind,
                ErrorKind::Validation
            );
            checked += 1;
        }
    }
    assert_eq!(checked, 21);
    for value in positives() {
        let input = bytes(&value);
        for codec in 1..=3 {
            assert_eq!(
                NativePolicyCommand::decode(codec, &input).unwrap_err().kind,
                ErrorKind::Validation
            );
        }
    }
}
