//! External test-only proposal. Synthetic in-memory rows are never stored.
//! These tests cover the owner-jsonb mapper and checked application closure.
//! They do not establish Auth, Cedar, installed custody, SHA execution, recovery,
//! navigation's complete protected snapshot validation, or release acceptance.
use super::{material, rows};
use console_identity_application::group_process::*;
use serde_json::{Value, json};
use uuid::Uuid;

const OBSERVED_US: i64 = 1_790_816_401_234_567;
const FINAL_US: i64 = 1_790_816_403_456_789;

pub(super) fn fixture() -> Value {
    serde_json::from_str(FIXTURE_JSON).unwrap()
}
fn id(n: u128) -> Uuid {
    Uuid::from_u128(n)
}
fn group() -> GroupId {
    GroupId::from_uuid(id(3)).unwrap()
}
fn incarnation() -> GroupIncarnation {
    GroupIncarnation::from_uuid(id(4)).unwrap()
}
fn binding(at: i64) -> GroupProcessRetainedBindingV1 {
    GroupProcessRetainedBindingV1::new(AccountId::from_uuid(id(1)).unwrap(), id(8), 1, at, 21, 123)
        .unwrap()
}
pub(super) fn literal_bytes(text: &str) -> Vec<u8> {
    assert_eq!(text.len() % 2, 0);
    text.as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}
fn assert_checks(actual: &[(Vec<u8>, [u8; 32])], expected: &[&Value]) {
    assert_eq!(
        actual.len(),
        expected.len(),
        "a protected row or repeated history preimage was omitted"
    );
    for ((bytes, digest), oracle) in actual.iter().zip(expected) {
        assert_eq!(bytes, &literal_bytes(oracle["hex"].as_str().unwrap()));
        assert_eq!(bytes.len(), oracle["bytes"].as_u64().unwrap() as usize);
        assert_eq!(
            digest.as_slice(),
            literal_bytes(oracle["sha256"].as_str().unwrap())
        );
    }
}
fn unavailable<T>(value: Result<T, GroupProcessError>, label: &str) {
    assert!(
        matches!(value, Err(GroupProcessError::Unavailable)),
        "{label}"
    );
}
fn original(f: &Value, name: &str, terminal: bool) -> Value {
    json!({"accepted":f["rows"][name]["accepted"],
        "terminal":if terminal {f["rows"][name]["result"].clone()} else {Value::Null}})
}
fn locator(f: &Value, name: &str) -> GroupProcessLocator {
    rows::accepted(&f["rows"][name]["accepted"], &mut Vec::new())
        .unwrap()
        .locator()
}
fn current(f: &Value, name: &str, mode: i16, original: Value, before: bool) -> Value {
    let (policy, head, version, history) = if before && name == "first" {
        (Value::Null, Value::Null, Value::Null, json!([]))
    } else if before {
        (
            f["rows"]["suspend"]["policy"].clone(),
            f["prior_head"].clone(),
            f["rows"]["suspend"]["version"].clone(),
            json!([{"head":f["prior_head"],"reason":null}]),
        )
    } else {
        let history = if name == "first" {
            json!([{"head":f["rows"][name]["head"],"reason":null}])
        } else {
            json!([{"head":f["prior_head"],"reason":null},
                {"head":f["rows"][name]["head"],"reason":f["reason"]}])
        };
        (
            f["rows"][name]["policy"].clone(),
            f["rows"][name]["head"].clone(),
            f["rows"][name]["version"].clone(),
            history,
        )
    };
    let mut account = f["account"].clone();
    account["observed_at_us"] = json!(if before { OBSERVED_US } else { FINAL_US });
    json!({"variant":"CurrentMutation","mode":mode,"account":account,
        "group":f["group"],"topology":f["topology"],"designation":f["designation"],
        "group_deployment":f["group_deployment"],"source":f["source"],
        "policy":policy,"head":head,"version":version,"history":history,"original":original})
}
fn own(f: &Value, name: &str, terminal: bool) -> Value {
    json!({"variant":"OwnReceipt","mode":5,"account":f["account"],
        "group_id":group().as_uuid().to_string(),
        "group_incarnation":incarnation().as_uuid().to_string(),
        "original":original(f,name,terminal),"source":f["source"]})
}
fn checked_current(row: &Value) -> Result<CurrentGroupProcessAuthority, GroupProcessError> {
    let request = GroupProcessScopeRequest::Current {
        group: group(),
        incarnation: incarnation(),
    };
    let (projection, _) = material::projection(row, &request)?;
    CurrentGroupProcessAuthority::from_retained_projection(binding(FINAL_US), &request, projection)
}

#[test]
fn group_owner_jsonb_positive_rows_keep_every_literal_preimage_and_receipt() {
    let f = fixture();
    for name in ["first", "suspend"] {
        let mut checks = Vec::new();
        let status = rows::status(&original(&f, name, true), &mut checks)
            .unwrap()
            .unwrap();
        assert_checks(
            &checks,
            &[
                &f["oracles"][name]["command"],
                &f["oracles"][name]["result"],
            ],
        );
        let GroupProcessStatus::Terminal(terminal) = status else {
            panic!("lost terminal")
        };
        assert_eq!(terminal.accepted.accepted_at_us, 1_790_816_400_123_456);
        assert_eq!(terminal.result.executed_at_us(), 1_790_816_402_345_678);
        assert_eq!(terminal.result.requested_process_id(), id(5));

        let row = current(&f, name, 1, Value::Null, false);
        let request = GroupProcessScopeRequest::Current {
            group: group(),
            incarnation: incarnation(),
        };
        let (projection, checks) = material::projection(&row, &request).unwrap();
        let mut expected = vec![
            &f["oracles"][name]["policy"],
            &f["oracles"][name]["head"],
            &f["oracles"][name]["version"],
        ];
        if name == "suspend" {
            expected.push(&f["oracles"]["prior"]["head"]);
        }
        expected.push(&f["oracles"][name]["head"]);
        assert_checks(&checks, &expected);
        let authority = CurrentGroupProcessAuthority::from_retained_projection(
            binding(FINAL_US),
            &request,
            projection,
        )
        .unwrap();
        let view = authority.current_view().unwrap();
        assert_eq!(view.observed_at_us, FINAL_US);
        assert_eq!(view.history.len(), if name == "first" { 1 } else { 2 });
        assert_eq!(
            view.history.last().unwrap().reason.as_deref(),
            if name == "first" {
                None
            } else {
                f["reason"].as_str()
            }
        );
        assert_eq!(
            view.head.unwrap().reference.head_revision(),
            if name == "first" { 1 } else { 2 }
        );

        for terminal in [false, true] {
            let row = own(&f, name, terminal);
            let request = GroupProcessScopeRequest::Status(locator(&f, name));
            let (projection, checks) = material::projection(&row, &request).unwrap();
            let mut expected = vec![&f["oracles"][name]["command"]];
            if terminal {
                expected.push(&f["oracles"][name]["result"]);
            }
            assert_checks(&checks, &expected);
            let authority = CurrentGroupProcessAuthority::from_retained_projection(
                binding(FINAL_US),
                &request,
                projection,
            )
            .unwrap();
            assert!(authority.original_status().is_some());
            unavailable(
                authority.current_view(),
                "own receipt fabricated current context",
            );
        }
    }
    let mut checks = Vec::new();
    assert_eq!(rows::status(&Value::Null, &mut checks).unwrap(), None);
    assert!(checks.is_empty());
}

fn object_paths(value: &Value, path: &str, output: &mut Vec<String>) {
    match value {
        Value::Object(object) => {
            output.push(path.into());
            for (key, value) in object {
                object_paths(value, &format!("{path}/{key}"), output);
            }
        }
        Value::Array(array) => {
            for (index, value) in array.iter().enumerate() {
                object_paths(value, &format!("{path}/{index}"), output);
            }
        }
        _ => {}
    }
}
#[test]
fn group_owner_jsonb_rejects_each_omitted_and_unknown_object_member() {
    let f = fixture();
    for name in ["first", "suspend"] {
        for own_receipt in [false, true] {
            let row = if own_receipt {
                own(&f, name, true)
            } else {
                current(&f, name, 4, original(&f, name, true), false)
            };
            let request = if own_receipt {
                GroupProcessScopeRequest::Status(locator(&f, name))
            } else {
                GroupProcessScopeRequest::Execute(locator(&f, name))
            };
            material::projection(&row, &request).unwrap(); // meaningful positive parser control
            let mut paths = Vec::new();
            object_paths(&row, "", &mut paths);
            for path in paths {
                let members: Vec<_> = row
                    .pointer(&path)
                    .unwrap()
                    .as_object()
                    .unwrap()
                    .keys()
                    .cloned()
                    .collect();
                for member in members {
                    let mut corrupt = row.clone();
                    corrupt
                        .pointer_mut(&path)
                        .unwrap()
                        .as_object_mut()
                        .unwrap()
                        .remove(&member);
                    unavailable(
                        material::projection(&corrupt, &request),
                        &format!("omitted {path}/{member}"),
                    );
                }
                let mut corrupt = row.clone();
                corrupt
                    .pointer_mut(&path)
                    .unwrap()
                    .as_object_mut()
                    .unwrap()
                    .insert("unexpected_member".into(), Value::Null);
                unavailable(
                    material::projection(&corrupt, &request),
                    &format!("unknown member at {path}"),
                );
            }
        }
    }
}

#[test]
fn group_owner_jsonb_rejects_wrong_scalars_nullable_arms_and_stored_byte_mismatches() {
    let f = fixture();
    let row = current(&f, "first", 1, Value::Null, false);
    checked_current(&row).unwrap();
    let cases = [
        ("/mode", json!(1.5)),
        ("/mode", json!(9)),
        ("/account/account_security_generation", json!(0)),
        ("/account/account_security_generation", json!("1")),
        ("/account/source_xid", json!("021")),
        ("/account/source_xid", json!(21)),
        ("/account/source_backend_pid", json!(4_294_967_296_u64)),
        ("/account/source_backend_pid", json!(2_147_483_648_u64)),
        ("/account/actor_account_id", json!(Uuid::nil().to_string())),
        (
            "/group/origin_command_id",
            json!(id(10).to_string().to_uppercase()),
        ),
        ("/topology/group_id", json!(id(99).to_string())),
        ("/topology/state", json!("INACTIVE")),
        ("/designation/singleton", json!(0)),
        ("/designation/database_oid", json!(0)),
        (
            "/group_deployment/database_name",
            json!("different_database"),
        ),
        ("/version/schema_id", json!("native-group-process-v1")),
        ("/version/operator_responsibility", json!(true)),
        ("/version/operator_responsibility", json!(0)),
        ("/head/state", json!("READY")),
        (
            "/source/registered_actions/0/action_id",
            json!(Uuid::nil().to_string()),
        ),
        ("/policy/registered_actions/0/revision", json!(2)),
    ];
    for (path, replacement) in cases {
        let mut corrupt = row.clone();
        *corrupt.pointer_mut(path).unwrap() = replacement;
        unavailable(checked_current(&corrupt), path);
    }
    for path in [
        "/original/accepted/command_id",
        "/original/accepted/group_id",
        "/original/accepted/group_incarnation",
        "/original/terminal/command_id",
        "/original/terminal/result_receipt_id",
        "/original/terminal/execution_session_id",
    ] {
        let mut corrupt = own(&f, "first", true);
        *corrupt.pointer_mut(path).unwrap() = json!(id(99).to_string());
        unavailable(
            material::projection(
                &corrupt,
                &GroupProcessScopeRequest::Status(locator(&f, "first")),
            ),
            path,
        );
    }
    let nullable_cases = [
        ("/original/accepted/accepted_policy_revision", json!(1)),
        ("/original/terminal/policy_before_revision", json!(1)),
        (
            "/original/terminal/policy_before_head_digest",
            f["rows"]["first"]["policy"]["head_digest"].clone(),
        ),
        ("/original/terminal/policy_after_tag", json!(2)),
        ("/original/terminal/policy_after_revision", Value::Null),
        (
            "/original/terminal/before_process_id",
            json!(id(5).to_string()),
        ),
        ("/original/terminal/after_head_digest", Value::Null),
        ("/original/terminal/effect_census/results", json!(0)),
        ("/original/terminal/effect_census/results", json!("1")),
        ("/original/terminal/layout_version", json!(1)),
        ("/original/accepted/codec_version", json!(2)),
        ("/original/accepted/operation", json!(6)),
        ("/original/terminal/operation", json!(6)),
        ("/original/terminal/terminal_code", json!("REPLACED")),
    ];
    for (path, replacement) in nullable_cases {
        let mut corrupt = own(&f, "first", true);
        *corrupt.pointer_mut(path).unwrap() = replacement;
        unavailable(
            material::projection(
                &corrupt,
                &GroupProcessScopeRequest::Status(locator(&f, "first")),
            ),
            path,
        );
    }
}

#[test]
fn group_owner_jsonb_preserves_exact_timestamp_precision_and_bounded_bytea_grammar() {
    for (text, expected) in [
        ("2026-10-01T01:00:00.123456Z", 1_790_816_400_123_456),
        ("2026-10-01T10:00:00.123456+09:00", 1_790_816_400_123_456),
        ("2026-10-01T01:00:00.123456000+00:00", 1_790_816_400_123_456),
        ("2026-10-01T01:00:00Z", 1_790_816_400_000_000),
        ("1969-12-31T23:59:59.999999Z", -1),
        ("1969-12-31T23:59:58.999999Z", -1_000_001),
    ] {
        assert_eq!(
            rows::timestamp(&json!({"at":text}), "at").unwrap(),
            expected
        );
    }
    for text in [
        "2026-10-01T01:00:00.123456001Z",
        "2026-10-01T01:00:00",
        "infinity",
        "-infinity",
        "0001-12-31 15:00:00+00 BC",
        "2026-02-30T01:00:00Z",
    ] {
        unavailable(rows::timestamp(&json!({"at":text}), "at"), text);
    }
    assert_eq!(rows::decimal("18446744073709551615").unwrap(), u64::MAX);
    for text in ["", "0", "01", "+1", " 1", "1.0", "18446744073709551616"] {
        unavailable(rows::decimal(text), text);
    }
    let f = fixture();
    let good = f["rows"]["first"]["accepted"]["input_bytes"]
        .as_str()
        .unwrap();
    assert_eq!(
        rows::bytes(&json!({"v":good}), "v", 188, GROUP_PROCESS_ADOPT_MAX_BYTES).unwrap(),
        literal_bytes(f["oracles"]["first"]["command"]["hex"].as_str().unwrap())
    );
    for text in [
        good[2..].to_owned(),
        good.to_uppercase(),
        format!("{}0", good),
        "\\x00".into(),
        format!("\\x{}", "00".repeat(GROUP_PROCESS_ADOPT_MAX_BYTES + 1)),
    ] {
        unavailable(
            rows::bytes(&json!({"v":text}), "v", 188, GROUP_PROCESS_ADOPT_MAX_BYTES),
            "bytea grammar",
        );
    }
    for text in [
        "\\x00".to_owned(),
        format!("\\x{}", "00".repeat(31)),
        format!("\\x{}", "00".repeat(33)),
    ] {
        unavailable(
            rows::digest(&json!({"v":text}), "v"),
            "exact 32-byte digest",
        );
    }
    let mut corrupt = f["rows"]["first"]["accepted"].clone();
    corrupt["input_bytes"] = json!(format!("{good}00"));
    unavailable(
        rows::accepted(&corrupt, &mut Vec::new()),
        "command trailing byte",
    );
    assert!(rows::document("{}", 2).is_ok());
    unavailable(rows::document("{}", 1), "document exceeds admitted bound");
    unavailable(rows::document("", 2), "empty document");
    unavailable(rows::document("{", 2), "invalid JSON");
    // Digest comparison is deferred: the queue must retain corrupted digest
    // bytes rather than falsely accepting the row as cryptographically checked.
    let mut corrupt = f["rows"]["first"]["accepted"].clone();
    corrupt["input_digest"] = json!(format!("\\x{}", "11".repeat(32)));
    let mut checks = Vec::new();
    rows::accepted(&corrupt, &mut checks).unwrap();
    assert_eq!(checks.len(), 1);
    assert_eq!(
        checks[0].0,
        literal_bytes(f["oracles"]["first"]["command"]["hex"].as_str().unwrap())
    );
    assert_eq!(checks[0].1, [0x11; 32]);
    assert_ne!(
        checks[0].1.as_slice(),
        literal_bytes(f["oracles"]["first"]["command"]["sha256"].as_str().unwrap())
    );
}

#[test]
fn group_owner_jsonb_rejects_incomplete_substituted_and_inconsistent_history() {
    let f = fixture();
    let row = current(&f, "suspend", 1, Value::Null, false);
    checked_current(&row).unwrap();
    for corruption in 0..9 {
        let mut corrupt = row.clone();
        match corruption {
            0 => {
                corrupt["history"] = json!([]);
            }
            1 => {
                corrupt["history"].as_array_mut().unwrap().remove(0);
            }
            2 => {
                corrupt["history"][0]["head"]["before_head_digest"] =
                    f["prior_head"]["head_digest"].clone();
            }
            3 => {
                corrupt["history"][1]["head"]["before_head_digest"] =
                    f["rows"]["suspend"]["head"]["head_digest"].clone();
            }
            4 => {
                corrupt["history"][1]["head"]["last_input_digest"] =
                    f["rows"]["first"]["accepted"]["input_digest"].clone();
            }
            5 => {
                corrupt["history"][0]["reason"] = f["reason"].clone();
            }
            6 => {
                corrupt["history"][1]["reason"] = Value::Null;
            }
            7 => {
                corrupt["history"][0]["head"]["content_version"] = json!(2);
            }
            8 => {
                corrupt["version"]["process_id"] = json!(id(99).to_string());
            }
            _ => unreachable!(),
        }
        unavailable(
            checked_current(&corrupt),
            &format!("history corruption {corruption}"),
        );
    }
}

#[test]
fn group_owner_jsonb_reconstructs_same_transaction_postimage_without_resealing() {
    let f = fixture();
    for name in ["first", "suspend"] {
        let request = GroupProcessScopeRequest::Execute(locator(&f, name));
        let before = current(&f, name, 4, original(&f, name, false), true);
        let (projection, checks) = material::projection(&before, &request).unwrap();
        let mut expected = vec![&f["oracles"][name]["command"]];
        if name == "suspend" {
            expected.extend([
                &f["oracles"]["prior"]["policy"],
                &f["oracles"]["prior"]["head"],
                &f["oracles"]["prior"]["version"],
                &f["oracles"]["prior"]["head"],
            ]);
        }
        assert_checks(&checks, &expected);
        let authority = CurrentGroupProcessAuthority::from_retained_projection(
            binding(OBSERVED_US),
            &request,
            projection,
        )
        .unwrap();
        let after = current(&f, name, 4, original(&f, name, true), false);
        let (projection, checks) = material::projection(&after, &request).unwrap();
        let mut expected = vec![
            &f["oracles"][name]["command"],
            &f["oracles"][name]["result"],
            &f["oracles"][name]["policy"],
            &f["oracles"][name]["head"],
            &f["oracles"][name]["version"],
        ];
        if name == "suspend" {
            expected.push(&f["oracles"]["prior"]["head"]);
        }
        expected.push(&f["oracles"][name]["head"]);
        assert_checks(&checks, &expected);
        unavailable(
            CurrentGroupProcessAuthority::from_retained_projection(
                binding(FINAL_US),
                &request,
                projection,
            ),
            "ordinary construction admitted already terminal Execute",
        );
        let (projection, _) = material::projection(&after, &request).unwrap();
        let GroupProcessRetainedProjectionV1::Current(projected) = projection else {
            panic!("lost current")
        };
        let GroupProcessStatus::Terminal(terminal) =
            rows::status(&original(&f, name, true), &mut Vec::new())
                .unwrap()
                .unwrap()
        else {
            panic!("lost terminal")
        };
        let execution = GroupProcessExecution {
            inserted: true,
            terminal,
        };
        authority
            .execution_postimage(binding(FINAL_US), &request, projected, &execution)
            .unwrap();
        let mut corrupt = after.clone();
        let coherent = &f["coherent_first_content"]["oracles"];
        if name == "suspend" {
            // The reason is outside the head/result digest; the exact accepted
            // command still owns which reason may be projected after B.
            corrupt["history"][1]["reason"] = f["coherent_suspend_reason"].clone();
        } else {
            // Every altered digest and stored byte string is independently
            // declared in the already approved first_content oracle. A stale
            // digest cannot mask the accepted-content closure failure.
            corrupt["version"]["title"] = f["coherent_first_content"]["title"].clone();
            let version_digest = json!(format!(
                "\\x{}",
                coherent["version"]["sha256"].as_str().unwrap()
            ));
            let head_digest = json!(format!(
                "\\x{}",
                coherent["head"]["sha256"].as_str().unwrap()
            ));
            corrupt["version"]["content_digest"] = version_digest.clone();
            corrupt["head"]["content_digest"] = version_digest.clone();
            corrupt["head"]["head_digest"] = head_digest.clone();
            corrupt["history"][0]["head"]["content_digest"] = version_digest.clone();
            corrupt["history"][0]["head"]["head_digest"] = head_digest.clone();
            corrupt["original"]["terminal"]["after_content_digest"] = version_digest;
            corrupt["original"]["terminal"]["after_head_digest"] = head_digest;
            corrupt["original"]["terminal"]["result_bytes"] = json!(format!(
                "\\x{}",
                coherent["result"]["hex"].as_str().unwrap()
            ));
            corrupt["original"]["terminal"]["result_digest"] = json!(format!(
                "\\x{}",
                coherent["result"]["sha256"].as_str().unwrap()
            ));
        }
        let (projection, checks) = material::projection(&corrupt, &request).unwrap();
        if name == "first" {
            assert_checks(
                &checks,
                &[
                    &coherent["command"],
                    &coherent["result"],
                    &coherent["policy"],
                    &coherent["head"],
                    &coherent["version"],
                    &coherent["head"],
                ],
            );
        } else {
            assert_checks(&checks, &expected);
        }
        let GroupProcessRetainedProjectionV1::Current(projected) = projection else {
            panic!("lost current")
        };
        let GroupProcessStatus::Terminal(terminal) =
            rows::status(&corrupt["original"], &mut Vec::new())
                .unwrap()
                .unwrap()
        else {
            panic!("lost corrupt terminal")
        };
        let corrupt_execution = GroupProcessExecution {
            inserted: true,
            terminal,
        };
        unavailable(
            authority.execution_postimage(
                binding(FINAL_US),
                &request,
                projected,
                &corrupt_execution,
            ),
            "coherently hashed content/reason escaped exact accepted postimage closure",
        );
    }
}

const FIXTURE_JSON: &str = r###"{"source":{"schema_id":"native-group-process-v1","schema_digest":"\\x7e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683","policy_digest":"\\x573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727","codec_contract_digest":"\\x338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e5129769","registration_manifest_version":1,"registration_manifest_digest":"\\x4172e4cf6afcbfb97e879eff3d02061db35cf4a0a02042724b710351dcc60b25","cedar_sdk_version":"4.13.0","cedar_language_version":"4.5","registered_actions":[{"key":"identity.verifier.process.adopt/1","action_id":"00000000-0000-0000-0000-00000000001f","revision":1,"fields":["process_id","process_version","process_digest","process_head_revision","process_head_digest","state","expiry","title","method","intended_claimant_matching_procedure","account_possession_procedure","physical_human_evidence_procedure","duplicate_contradictory_claim_procedure","qualification_criteria_instruction","escalation_adjudication_procedure","evidence_minimization_retention_description","recipient_responsibility","receipt_locator"]},{"key":"identity.verifier.process.suspend/1","action_id":"00000000-0000-0000-0000-000000000020","revision":1,"fields":["process_id","process_version","process_digest","process_head_revision","process_head_digest","state","expiry","reason","receipt_locator"]},{"key":"identity.verifier.process.read/1","action_id":"00000000-0000-0000-0000-000000000021","revision":1,"fields":["group_context","process_id","process_version","process_digest","process_head_revision","process_head_digest","state","expiry","title","method","intended_claimant_matching_procedure","account_possession_procedure","physical_human_evidence_procedure","duplicate_contradictory_claim_procedure","qualification_criteria_instruction","escalation_adjudication_procedure","evidence_minimization_retention_description","recipient_responsibility","history","allowed_actions"]},{"key":"identity.verifier.process.receipt.read-own/1","action_id":"00000000-0000-0000-0000-000000000022","revision":1,"fields":["command_id","input_digest","intake_receipt_id","result_receipt_id","terminal_code","accepted_at","executed_at","process_id","before_head","after_head","original_content","original_reason","receipt_locator"]}]},"account":{"actor_account_id":"00000000-0000-0000-0000-000000000001","session_id":"00000000-0000-0000-0000-000000000008","account_security_generation":1,"account_state":"ACTIVE","registration_receipt":"00000000-0000-0000-0000-00000000000c","current_terms_receipt":"00000000-0000-0000-0000-00000000000d","observed_at_us":1790816403456789,"source_xid":"21","source_backend_pid":123},"group":{"id":"00000000-0000-0000-0000-000000000003","name":"실제 절차를 검사하는 단위 테스트","slug":"constructor-fixture","status":"ACTIVE","created_at":"2026-10-01T00:59:50Z","updated_at":"2026-10-01T00:59:50Z","origin_account_id":"00000000-0000-0000-0000-000000000001","origin_command_id":"00000000-0000-0000-0000-00000000000a","origin_receipt_id":"00000000-0000-0000-0000-00000000000b"},"topology":{"group_id":"00000000-0000-0000-0000-000000000003","incarnation":"00000000-0000-0000-0000-000000000004","revision":7,"state":"ACTIVE"},"designation":{"singleton":1,"account_id":"00000000-0000-0000-0000-000000000001","revision":1,"receipt_id":"00000000-0000-0000-0000-000000000009","system_identifier":"123456789","database_name":"constructor_fixture","database_oid":42},"group_deployment":{"system_identifier":"123456789","database_name":"constructor_fixture","database_oid":42},"rows":{"first":{"policy":{"schema_id":"native-group-process-v1","schema_digest":"\\x7e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683","policy_digest":"\\x573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727","codec_contract_digest":"\\x338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e5129769","registration_manifest_version":1,"registration_manifest_digest":"\\x4172e4cf6afcbfb97e879eff3d02061db35cf4a0a02042724b710351dcc60b25","group_id":"00000000-0000-0000-0000-000000000003","group_incarnation":"00000000-0000-0000-0000-000000000004","revision":1,"first_actor_account_id":"00000000-0000-0000-0000-000000000001","first_command_id":"00000000-0000-0000-0000-000000000002","first_input_digest":"\\xffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe","activation_receipt_id":"00000000-0000-0000-0000-000000000007","activated_at":"2026-10-01T01:00:02.345678Z","head_digest":"\\xc4507daab34ef2a018b2f906e310eac2d76a3f01763e5b9a5268eba72694111d","registered_actions":[{"key":"identity.verifier.process.adopt/1","action_id":"00000000-0000-0000-0000-00000000001f","revision":1,"fields":["process_id","process_version","process_digest","process_head_revision","process_head_digest","state","expiry","title","method","intended_claimant_matching_procedure","account_possession_procedure","physical_human_evidence_procedure","duplicate_contradictory_claim_procedure","qualification_criteria_instruction","escalation_adjudication_procedure","evidence_minimization_retention_description","recipient_responsibility","receipt_locator"]},{"key":"identity.verifier.process.suspend/1","action_id":"00000000-0000-0000-0000-000000000020","revision":1,"fields":["process_id","process_version","process_digest","process_head_revision","process_head_digest","state","expiry","reason","receipt_locator"]},{"key":"identity.verifier.process.read/1","action_id":"00000000-0000-0000-0000-000000000021","revision":1,"fields":["group_context","process_id","process_version","process_digest","process_head_revision","process_head_digest","state","expiry","title","method","intended_claimant_matching_procedure","account_possession_procedure","physical_human_evidence_procedure","duplicate_contradictory_claim_procedure","qualification_criteria_instruction","escalation_adjudication_procedure","evidence_minimization_retention_description","recipient_responsibility","history","allowed_actions"]},{"key":"identity.verifier.process.receipt.read-own/1","action_id":"00000000-0000-0000-0000-000000000022","revision":1,"fields":["command_id","input_digest","intake_receipt_id","result_receipt_id","terminal_code","accepted_at","executed_at","process_id","before_head","after_head","original_content","original_reason","receipt_locator"]}]},"version":{"group_id":"00000000-0000-0000-0000-000000000003","group_incarnation":"00000000-0000-0000-0000-000000000004","process_id":"00000000-0000-0000-0000-000000000005","version":1,"schema_id":"GROUP_VERIFIER_PROCESS_V1","actor_account_id":"00000000-0000-0000-0000-000000000001","designation_receipt_id":"00000000-0000-0000-0000-000000000009","designation_revision":1,"policy_revision":1,"policy_head_digest":"\\xc4507daab34ef2a018b2f906e310eac2d76a3f01763e5b9a5268eba72694111d","adopt_command_id":"00000000-0000-0000-0000-000000000002","input_digest":"\\xffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe","admitted_at":"2026-10-01T01:00:00.123456Z","expires_at":"2026-10-02T01:00:00.000000Z","operator_responsibility":1,"title":"독립 검증 절차","method":"ATTENDED_ACCOUNT_AND_DOCUMENTARY_REVIEW_V1","intended_claimant_matching_procedure":"claimant match","account_possession_procedure":"account possession","physical_human_evidence_procedure":"physical evidence","duplicate_contradictory_claim_procedure":"contradictory claim","qualification_criteria_instruction":"qualification instruction","escalation_adjudication_procedure":"escalation adjudication","evidence_minimization_retention_description":"minimal retention","recipient_responsibility":"recipient responsibility","content_digest":"\\xc01f8908bef0d77d52b989ae9e8d4142c21e49d3fb6422e6a48e6b7dbfc90a9e"},"head":{"group_id":"00000000-0000-0000-0000-000000000003","group_incarnation":"00000000-0000-0000-0000-000000000004","process_id":"00000000-0000-0000-0000-000000000005","head_revision":1,"content_version":1,"content_digest":"\\xc01f8908bef0d77d52b989ae9e8d4142c21e49d3fb6422e6a48e6b7dbfc90a9e","state":"ACTIVE","expires_at":"2026-10-02T01:00:00.000000Z","last_actor_account_id":"00000000-0000-0000-0000-000000000001","last_command_id":"00000000-0000-0000-0000-000000000002","last_input_digest":"\\xffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe","result_receipt_id":"00000000-0000-0000-0000-000000000007","updated_at":"2026-10-01T01:00:02.345678Z","head_digest":"\\x9c5ee0e6463543162b3686b799f4e726d4e4b55801e9729e3211624152f1cd29","before_head_digest":null},"result":{"schema_id":"native-group-process-v1","schema_digest":"\\x7e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683","policy_digest":"\\x573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727","codec_contract_digest":"\\x338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e5129769","registration_manifest_version":1,"registration_manifest_digest":"\\x4172e4cf6afcbfb97e879eff3d02061db35cf4a0a02042724b710351dcc60b25","cedar_sdk_version":"4.13.0","cedar_language_version":"4.5","actor_account_id":"00000000-0000-0000-0000-000000000001","command_id":"00000000-0000-0000-0000-000000000002","group_id":"00000000-0000-0000-0000-000000000003","group_incarnation":"00000000-0000-0000-0000-000000000004","operation":1,"input_digest":"\\xffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe","intake_receipt_id":"00000000-0000-0000-0000-000000000006","result_receipt_id":"00000000-0000-0000-0000-000000000007","terminal_code":"ADOPTED","accepted_at":"2026-10-01T01:00:00.123456Z","executed_at":"2026-10-01T01:00:02.345678Z","execution_session_id":"00000000-0000-0000-0000-000000000008","account_security_generation":1,"designation_receipt_id":"00000000-0000-0000-0000-000000000009","designation_revision":1,"observed_group_revision":7,"policy_before_tag":0,"policy_before_revision":null,"policy_before_head_digest":null,"policy_after_tag":1,"policy_after_revision":1,"policy_after_head_digest":"\\xc4507daab34ef2a018b2f906e310eac2d76a3f01763e5b9a5268eba72694111d","requested_process_id":"00000000-0000-0000-0000-000000000005","effect_xid":"21","effect_backend_pid":123,"layout_version":2,"before_process_id":null,"before_head_revision":null,"before_content_version":null,"before_content_digest":null,"before_head_digest":null,"before_state":null,"before_expires_at":null,"after_process_id":"00000000-0000-0000-0000-000000000005","after_head_revision":1,"after_content_version":1,"after_content_digest":"\\xc01f8908bef0d77d52b989ae9e8d4142c21e49d3fb6422e6a48e6b7dbfc90a9e","after_head_digest":"\\x9c5ee0e6463543162b3686b799f4e726d4e4b55801e9729e3211624152f1cd29","after_state":"ACTIVE","after_expires_at":"2026-10-02T01:00:00.000000Z","effect_id":"00000000-0000-0000-0000-000000000011","audit_id":"00000000-0000-0000-0000-000000000012","effect_census":{"policy_heads":1,"versions":1,"heads":1,"head_revisions":1,"effects":1,"results":1,"audits":1},"result_bytes":"\\x434f4e534f4c452e4944454e544954592e50524f434553532e524553554c54000002000000000000000000000000000000010000000000000000000000000000000200000000000000000000000000000003000000000000000000000000000000040001ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe0000000000000000000000000000000600000000000000000000000000000007000100065cbcef63264000065cbcef850ece000000000000000000000000000000080000000000000001000000000000000000000000000000090000000000000001000000000000000700010000000000000001c4507daab34ef2a018b2f906e310eac2d76a3f01763e5b9a5268eba72694111d000000176e61746976652d67726f75702d70726f636573732d76317e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e512976900000000000000014172e4cf6afcbfb97e879eff3d02061db35cf4a0a02042724b710351dcc60b2500000006342e31332e3000000003342e350000000000000000000000000000000500010000000000000000000000000000000500000000000000010000000000000001c01f8908bef0d77d52b989ae9e8d4142c21e49d3fb6422e6a48e6b7dbfc90a9e9c5ee0e6463543162b3686b799f4e726d4e4b55801e9729e3211624152f1cd29000100065cd10d38a40000000000000000150000007b","result_digest":"\\x265fc32fe611ae01977cf3a8689cfbf7b91aa39b852b0d94b2ce6d1c4863559d"},"accepted":{"schema_id":"native-group-process-v1","schema_digest":"\\x7e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683","policy_digest":"\\x573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727","codec_contract_digest":"\\x338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e5129769","registration_manifest_version":1,"registration_manifest_digest":"\\x4172e4cf6afcbfb97e879eff3d02061db35cf4a0a02042724b710351dcc60b25","cedar_sdk_version":"4.13.0","cedar_language_version":"4.5","actor_account_id":"00000000-0000-0000-0000-000000000001","command_id":"00000000-0000-0000-0000-000000000002","group_id":"00000000-0000-0000-0000-000000000003","group_incarnation":"00000000-0000-0000-0000-000000000004","operation":1,"codec_version":1,"input_bytes":"\\x434f4e534f4c452e4944454e544954592e47524f55500000010001000000000000000000000000000000010000000000000000000000000000000200000000000000000000000000000003000000000000000000000000000000040000000000000007000000000000000000000000000000000000000000000005000000000000000000065cd10d38a400000100000014eb8f85eba6bd20eab280eca69d20eca088ecb0a800010000000e636c61696d616e74206d61746368000000126163636f756e7420706f7373657373696f6e00000011706879736963616c2065766964656e636500000013636f6e747261646963746f727920636c61696d000000197175616c696669636174696f6e20696e737472756374696f6e00000017657363616c6174696f6e2061646a756469636174696f6e000000116d696e696d616c20726574656e74696f6e00000018726563697069656e7420726573706f6e736962696c697479","input_digest":"\\xffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe","intake_receipt_id":"00000000-0000-0000-0000-000000000006","accepted_at":"2026-10-01T01:00:00.123456Z","accepted_session_id":"00000000-0000-0000-0000-000000000008","account_security_generation":1,"designation_receipt_id":"00000000-0000-0000-0000-000000000009","designation_revision":1,"observed_group_revision":7,"accepted_policy_tag":0,"accepted_policy_revision":null,"accepted_policy_head_digest":null,"acceptance_xid":"21","acceptance_backend_pid":123,"audit_id":"00000000-0000-0000-0000-000000000010"}},"suspend":{"policy":{"schema_id":"native-group-process-v1","schema_digest":"\\x7e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683","policy_digest":"\\x573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727","codec_contract_digest":"\\x338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e5129769","registration_manifest_version":1,"registration_manifest_digest":"\\x4172e4cf6afcbfb97e879eff3d02061db35cf4a0a02042724b710351dcc60b25","group_id":"00000000-0000-0000-0000-000000000003","group_incarnation":"00000000-0000-0000-0000-000000000004","revision":1,"first_actor_account_id":"00000000-0000-0000-0000-000000000001","first_command_id":"00000000-0000-0000-0000-00000000000f","first_input_digest":"\\x19a911a6eb9b79ff986bddb45fc24ea17f6e01b2589f33ebcefc3fcebf63e128","activation_receipt_id":"00000000-0000-0000-0000-00000000000e","activated_at":"2026-10-01T00:59:51.000000Z","head_digest":"\\x748a626ff665ced3a5bd1165facad0ef297af222154b051246bf7abe25c838b0","registered_actions":[{"key":"identity.verifier.process.adopt/1","action_id":"00000000-0000-0000-0000-00000000001f","revision":1,"fields":["process_id","process_version","process_digest","process_head_revision","process_head_digest","state","expiry","title","method","intended_claimant_matching_procedure","account_possession_procedure","physical_human_evidence_procedure","duplicate_contradictory_claim_procedure","qualification_criteria_instruction","escalation_adjudication_procedure","evidence_minimization_retention_description","recipient_responsibility","receipt_locator"]},{"key":"identity.verifier.process.suspend/1","action_id":"00000000-0000-0000-0000-000000000020","revision":1,"fields":["process_id","process_version","process_digest","process_head_revision","process_head_digest","state","expiry","reason","receipt_locator"]},{"key":"identity.verifier.process.read/1","action_id":"00000000-0000-0000-0000-000000000021","revision":1,"fields":["group_context","process_id","process_version","process_digest","process_head_revision","process_head_digest","state","expiry","title","method","intended_claimant_matching_procedure","account_possession_procedure","physical_human_evidence_procedure","duplicate_contradictory_claim_procedure","qualification_criteria_instruction","escalation_adjudication_procedure","evidence_minimization_retention_description","recipient_responsibility","history","allowed_actions"]},{"key":"identity.verifier.process.receipt.read-own/1","action_id":"00000000-0000-0000-0000-000000000022","revision":1,"fields":["command_id","input_digest","intake_receipt_id","result_receipt_id","terminal_code","accepted_at","executed_at","process_id","before_head","after_head","original_content","original_reason","receipt_locator"]}]},"version":{"group_id":"00000000-0000-0000-0000-000000000003","group_incarnation":"00000000-0000-0000-0000-000000000004","process_id":"00000000-0000-0000-0000-000000000005","version":1,"schema_id":"GROUP_VERIFIER_PROCESS_V1","actor_account_id":"00000000-0000-0000-0000-000000000001","designation_receipt_id":"00000000-0000-0000-0000-000000000009","designation_revision":1,"policy_revision":1,"policy_head_digest":"\\x748a626ff665ced3a5bd1165facad0ef297af222154b051246bf7abe25c838b0","adopt_command_id":"00000000-0000-0000-0000-00000000000f","input_digest":"\\x19a911a6eb9b79ff986bddb45fc24ea17f6e01b2589f33ebcefc3fcebf63e128","admitted_at":"2026-10-01T00:59:50.000000Z","expires_at":"2026-10-02T01:00:00.000000Z","operator_responsibility":1,"title":"독립 검증 절차","method":"ATTENDED_ACCOUNT_AND_DOCUMENTARY_REVIEW_V1","intended_claimant_matching_procedure":"claimant match","account_possession_procedure":"account possession","physical_human_evidence_procedure":"physical evidence","duplicate_contradictory_claim_procedure":"contradictory claim","qualification_criteria_instruction":"qualification instruction","escalation_adjudication_procedure":"escalation adjudication","evidence_minimization_retention_description":"minimal retention","recipient_responsibility":"recipient responsibility","content_digest":"\\x98bd7ea46629f0169a6eac2015c367196bc7e30431b5af269323e5d189b9b4e0"},"head":{"group_id":"00000000-0000-0000-0000-000000000003","group_incarnation":"00000000-0000-0000-0000-000000000004","process_id":"00000000-0000-0000-0000-000000000005","head_revision":2,"content_version":1,"content_digest":"\\x98bd7ea46629f0169a6eac2015c367196bc7e30431b5af269323e5d189b9b4e0","state":"SUSPENDED","expires_at":"2026-10-02T01:00:00.000000Z","last_actor_account_id":"00000000-0000-0000-0000-000000000001","last_command_id":"00000000-0000-0000-0000-000000000002","last_input_digest":"\\x0cc7fc6d11ad78000890d6bdf1d509207c57b8aa972ca073fb1e8abe9e8d978b","result_receipt_id":"00000000-0000-0000-0000-000000000007","updated_at":"2026-10-01T01:00:02.345678Z","head_digest":"\\x69ab1c9d0b376e732e0d0f02bb4ce3675b87146f6e7f2bf0d342ecbf524fa59d","before_head_digest":"\\x938698d8c6ad82400ead9e9c9d887179904ab0fa743182fe98c15c9fab5a1c28"},"result":{"schema_id":"native-group-process-v1","schema_digest":"\\x7e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683","policy_digest":"\\x573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727","codec_contract_digest":"\\x338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e5129769","registration_manifest_version":1,"registration_manifest_digest":"\\x4172e4cf6afcbfb97e879eff3d02061db35cf4a0a02042724b710351dcc60b25","cedar_sdk_version":"4.13.0","cedar_language_version":"4.5","actor_account_id":"00000000-0000-0000-0000-000000000001","command_id":"00000000-0000-0000-0000-000000000002","group_id":"00000000-0000-0000-0000-000000000003","group_incarnation":"00000000-0000-0000-0000-000000000004","operation":6,"input_digest":"\\x0cc7fc6d11ad78000890d6bdf1d509207c57b8aa972ca073fb1e8abe9e8d978b","intake_receipt_id":"00000000-0000-0000-0000-000000000006","result_receipt_id":"00000000-0000-0000-0000-000000000007","terminal_code":"SUSPENDED","accepted_at":"2026-10-01T01:00:00.123456Z","executed_at":"2026-10-01T01:00:02.345678Z","execution_session_id":"00000000-0000-0000-0000-000000000008","account_security_generation":1,"designation_receipt_id":"00000000-0000-0000-0000-000000000009","designation_revision":1,"observed_group_revision":7,"policy_before_tag":1,"policy_before_revision":1,"policy_before_head_digest":"\\x748a626ff665ced3a5bd1165facad0ef297af222154b051246bf7abe25c838b0","policy_after_tag":1,"policy_after_revision":1,"policy_after_head_digest":"\\x748a626ff665ced3a5bd1165facad0ef297af222154b051246bf7abe25c838b0","requested_process_id":"00000000-0000-0000-0000-000000000005","effect_xid":"21","effect_backend_pid":123,"layout_version":2,"before_process_id":"00000000-0000-0000-0000-000000000005","before_head_revision":1,"before_content_version":1,"before_content_digest":"\\x98bd7ea46629f0169a6eac2015c367196bc7e30431b5af269323e5d189b9b4e0","before_head_digest":"\\x938698d8c6ad82400ead9e9c9d887179904ab0fa743182fe98c15c9fab5a1c28","before_state":"ACTIVE","before_expires_at":"2026-10-02T01:00:00.000000Z","after_process_id":"00000000-0000-0000-0000-000000000005","after_head_revision":2,"after_content_version":1,"after_content_digest":"\\x98bd7ea46629f0169a6eac2015c367196bc7e30431b5af269323e5d189b9b4e0","after_head_digest":"\\x69ab1c9d0b376e732e0d0f02bb4ce3675b87146f6e7f2bf0d342ecbf524fa59d","after_state":"SUSPENDED","after_expires_at":"2026-10-02T01:00:00.000000Z","effect_id":"00000000-0000-0000-0000-000000000011","audit_id":"00000000-0000-0000-0000-000000000012","effect_census":{"policy_heads":0,"versions":0,"heads":1,"head_revisions":1,"effects":1,"results":1,"audits":1},"result_bytes":"\\x434f4e534f4c452e4944454e544954592e50524f434553532e524553554c540000020000000000000000000000000000000100000000000000000000000000000002000000000000000000000000000000030000000000000000000000000000000400060cc7fc6d11ad78000890d6bdf1d509207c57b8aa972ca073fb1e8abe9e8d978b0000000000000000000000000000000600000000000000000000000000000007000300065cbcef63264000065cbcef850ece0000000000000000000000000000000800000000000000010000000000000000000000000000000900000000000000010000000000000007010000000000000001748a626ff665ced3a5bd1165facad0ef297af222154b051246bf7abe25c838b0010000000000000001748a626ff665ced3a5bd1165facad0ef297af222154b051246bf7abe25c838b0000000176e61746976652d67726f75702d70726f636573732d76317e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e512976900000000000000014172e4cf6afcbfb97e879eff3d02061db35cf4a0a02042724b710351dcc60b2500000006342e31332e3000000003342e350000000000000000000000000000000501000000000000000000000000000000050000000000000001000000000000000198bd7ea46629f0169a6eac2015c367196bc7e30431b5af269323e5d189b9b4e0938698d8c6ad82400ead9e9c9d887179904ab0fa743182fe98c15c9fab5a1c28000100065cd10d38a40001000000000000000000000000000000050000000000000002000000000000000198bd7ea46629f0169a6eac2015c367196bc7e30431b5af269323e5d189b9b4e069ab1c9d0b376e732e0d0f02bb4ce3675b87146f6e7f2bf0d342ecbf524fa59d000200065cd10d38a40000000000000000150000007b","result_digest":"\\x830a587fa92fea0be151fb5a5a0461635d2ffea4760498be6d4269815b9d5d46"},"accepted":{"schema_id":"native-group-process-v1","schema_digest":"\\x7e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683","policy_digest":"\\x573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727","codec_contract_digest":"\\x338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e5129769","registration_manifest_version":1,"registration_manifest_digest":"\\x4172e4cf6afcbfb97e879eff3d02061db35cf4a0a02042724b710351dcc60b25","cedar_sdk_version":"4.13.0","cedar_language_version":"4.5","actor_account_id":"00000000-0000-0000-0000-000000000001","command_id":"00000000-0000-0000-0000-000000000002","group_id":"00000000-0000-0000-0000-000000000003","group_incarnation":"00000000-0000-0000-0000-000000000004","operation":6,"codec_version":1,"input_bytes":"\\x434f4e534f4c452e4944454e544954592e47524f55500000010006000000000000000000000000000000010000000000000000000000000000000200000000000000000000000000000003000000000000000000000000000000040000000000000007000000000000000100000000000000000000000000000005000000000000000198bd7ea46629f0169a6eac2015c367196bc7e30431b5af269323e5d189b9b4e00000000000000001938698d8c6ad82400ead9e9c9d887179904ab0fa743182fe98c15c9fab5a1c280000001eeab280ed86a0ed959c20ec9b90ebacb820eca491eca78020ec82acec9ca0","input_digest":"\\x0cc7fc6d11ad78000890d6bdf1d509207c57b8aa972ca073fb1e8abe9e8d978b","intake_receipt_id":"00000000-0000-0000-0000-000000000006","accepted_at":"2026-10-01T01:00:00.123456Z","accepted_session_id":"00000000-0000-0000-0000-000000000008","account_security_generation":1,"designation_receipt_id":"00000000-0000-0000-0000-000000000009","designation_revision":1,"observed_group_revision":7,"accepted_policy_tag":1,"accepted_policy_revision":1,"accepted_policy_head_digest":"\\x748a626ff665ced3a5bd1165facad0ef297af222154b051246bf7abe25c838b0","acceptance_xid":"21","acceptance_backend_pid":123,"audit_id":"00000000-0000-0000-0000-000000000010"}}},"oracles":{"first":{"command":{"hex":"434f4e534f4c452e4944454e544954592e47524f55500000010001000000000000000000000000000000010000000000000000000000000000000200000000000000000000000000000003000000000000000000000000000000040000000000000007000000000000000000000000000000000000000000000005000000000000000000065cd10d38a400000100000014eb8f85eba6bd20eab280eca69d20eca088ecb0a800010000000e636c61696d616e74206d61746368000000126163636f756e7420706f7373657373696f6e00000011706879736963616c2065766964656e636500000013636f6e747261646963746f727920636c61696d000000197175616c696669636174696f6e20696e737472756374696f6e00000017657363616c6174696f6e2061646a756469636174696f6e000000116d696e696d616c20726574656e74696f6e00000018726563697069656e7420726573706f6e736962696c697479","sha256":"ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe","bytes":356},"policy":{"hex":"434f4e534f4c452e4944454e544954592e47524f55502e504f4c4943592e4845414400000100000000000000000000000000000003000000000000000000000000000000040000000000000001000000176e61746976652d67726f75702d70726f636573732d76317e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e51297694172e4cf6afcbfb97e879eff3d02061db35cf4a0a02042724b710351dcc60b250000000000000000000000000000000100000000000000000000000000000002ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe0000000000000000000000000000000700065cbcef850ece","sha256":"c4507daab34ef2a018b2f906e310eac2d76a3f01763e5b9a5268eba72694111d","bytes":320},"version":{"hex":"434f4e534f4c452e4944454e544954592e50524f434553532e56455253494f4e00000100000000000000000000000000000003000000000000000000000000000000040000000000000000000000000000000500000000000000010000001947524f55505f56455249464945525f50524f434553535f5631000000000000000000000000000000010000000000000000000000000000000900000000000000010000000000000001c4507daab34ef2a018b2f906e310eac2d76a3f01763e5b9a5268eba72694111d00000000000000000000000000000002ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe00065cbcef63264000065cd10d38a400000100000014eb8f85eba6bd20eab280eca69d20eca088ecb0a800010000000e636c61696d616e74206d61746368000000126163636f756e7420706f7373657373696f6e00000011706879736963616c2065766964656e636500000013636f6e747261646963746f727920636c61696d000000197175616c696669636174696f6e20696e737472756374696f6e00000017657363616c6174696f6e2061646a756469636174696f6e000000116d696e696d616c20726574656e74696f6e00000018726563697069656e7420726573706f6e736962696c697479","sha256":"c01f8908bef0d77d52b989ae9e8d4142c21e49d3fb6422e6a48e6b7dbfc90a9e","bytes":481},"head":{"hex":"434f4e534f4c452e4944454e544954592e50524f434553532e4845414400000100000000000000000000000000000003000000000000000000000000000000040000000000000000000000000000000500000000000000010000000000000001c01f8908bef0d77d52b989ae9e8d4142c21e49d3fb6422e6a48e6b7dbfc90a9e000100065cd10d38a4000000000000000000000000000000000100000000000000000000000000000002ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe0000000000000000000000000000000700065cbcef850ece","sha256":"9c5ee0e6463543162b3686b799f4e726d4e4b55801e9729e3211624152f1cd29","bytes":226},"result":{"hex":"434f4e534f4c452e4944454e544954592e50524f434553532e524553554c54000002000000000000000000000000000000010000000000000000000000000000000200000000000000000000000000000003000000000000000000000000000000040001ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe0000000000000000000000000000000600000000000000000000000000000007000100065cbcef63264000065cbcef850ece000000000000000000000000000000080000000000000001000000000000000000000000000000090000000000000001000000000000000700010000000000000001c4507daab34ef2a018b2f906e310eac2d76a3f01763e5b9a5268eba72694111d000000176e61746976652d67726f75702d70726f636573732d76317e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e512976900000000000000014172e4cf6afcbfb97e879eff3d02061db35cf4a0a02042724b710351dcc60b2500000006342e31332e3000000003342e350000000000000000000000000000000500010000000000000000000000000000000500000000000000010000000000000001c01f8908bef0d77d52b989ae9e8d4142c21e49d3fb6422e6a48e6b7dbfc90a9e9c5ee0e6463543162b3686b799f4e726d4e4b55801e9729e3211624152f1cd29000100065cd10d38a40000000000000000150000007b","sha256":"265fc32fe611ae01977cf3a8689cfbf7b91aa39b852b0d94b2ce6d1c4863559d","bytes":596}},"suspend":{"command":{"hex":"434f4e534f4c452e4944454e544954592e47524f55500000010006000000000000000000000000000000010000000000000000000000000000000200000000000000000000000000000003000000000000000000000000000000040000000000000007000000000000000100000000000000000000000000000005000000000000000198bd7ea46629f0169a6eac2015c367196bc7e30431b5af269323e5d189b9b4e00000000000000001938698d8c6ad82400ead9e9c9d887179904ab0fa743182fe98c15c9fab5a1c280000001eeab280ed86a0ed959c20ec9b90ebacb820eca491eca78020ec82acec9ca0","sha256":"0cc7fc6d11ad78000890d6bdf1d509207c57b8aa972ca073fb1e8abe9e8d978b","bytes":237},"policy":{"hex":"434f4e534f4c452e4944454e544954592e47524f55502e504f4c4943592e4845414400000100000000000000000000000000000003000000000000000000000000000000040000000000000001000000176e61746976652d67726f75702d70726f636573732d76317e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e51297694172e4cf6afcbfb97e879eff3d02061db35cf4a0a02042724b710351dcc60b25000000000000000000000000000000010000000000000000000000000000000f19a911a6eb9b79ff986bddb45fc24ea17f6e01b2589f33ebcefc3fcebf63e1280000000000000000000000000000000e00065cbceed7efc0","sha256":"748a626ff665ced3a5bd1165facad0ef297af222154b051246bf7abe25c838b0","bytes":320},"version":{"hex":"434f4e534f4c452e4944454e544954592e50524f434553532e56455253494f4e00000100000000000000000000000000000003000000000000000000000000000000040000000000000000000000000000000500000000000000010000001947524f55505f56455249464945525f50524f434553535f5631000000000000000000000000000000010000000000000000000000000000000900000000000000010000000000000001748a626ff665ced3a5bd1165facad0ef297af222154b051246bf7abe25c838b00000000000000000000000000000000f19a911a6eb9b79ff986bddb45fc24ea17f6e01b2589f33ebcefc3fcebf63e12800065cbceec8ad8000065cd10d38a400000100000014eb8f85eba6bd20eab280eca69d20eca088ecb0a800010000000e636c61696d616e74206d61746368000000126163636f756e7420706f7373657373696f6e00000011706879736963616c2065766964656e636500000013636f6e747261646963746f727920636c61696d000000197175616c696669636174696f6e20696e737472756374696f6e00000017657363616c6174696f6e2061646a756469636174696f6e000000116d696e696d616c20726574656e74696f6e00000018726563697069656e7420726573706f6e736962696c697479","sha256":"98bd7ea46629f0169a6eac2015c367196bc7e30431b5af269323e5d189b9b4e0","bytes":481},"head":{"hex":"434f4e534f4c452e4944454e544954592e50524f434553532e484541440000010000000000000000000000000000000300000000000000000000000000000004000000000000000000000000000000050000000000000002000000000000000198bd7ea46629f0169a6eac2015c367196bc7e30431b5af269323e5d189b9b4e0000200065cd10d38a40000000000000000000000000000000001000000000000000000000000000000020cc7fc6d11ad78000890d6bdf1d509207c57b8aa972ca073fb1e8abe9e8d978b0000000000000000000000000000000700065cbcef850ece","sha256":"69ab1c9d0b376e732e0d0f02bb4ce3675b87146f6e7f2bf0d342ecbf524fa59d","bytes":226},"result":{"hex":"434f4e534f4c452e4944454e544954592e50524f434553532e524553554c540000020000000000000000000000000000000100000000000000000000000000000002000000000000000000000000000000030000000000000000000000000000000400060cc7fc6d11ad78000890d6bdf1d509207c57b8aa972ca073fb1e8abe9e8d978b0000000000000000000000000000000600000000000000000000000000000007000300065cbcef63264000065cbcef850ece0000000000000000000000000000000800000000000000010000000000000000000000000000000900000000000000010000000000000007010000000000000001748a626ff665ced3a5bd1165facad0ef297af222154b051246bf7abe25c838b0010000000000000001748a626ff665ced3a5bd1165facad0ef297af222154b051246bf7abe25c838b0000000176e61746976652d67726f75702d70726f636573732d76317e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e512976900000000000000014172e4cf6afcbfb97e879eff3d02061db35cf4a0a02042724b710351dcc60b2500000006342e31332e3000000003342e350000000000000000000000000000000501000000000000000000000000000000050000000000000001000000000000000198bd7ea46629f0169a6eac2015c367196bc7e30431b5af269323e5d189b9b4e0938698d8c6ad82400ead9e9c9d887179904ab0fa743182fe98c15c9fab5a1c28000100065cd10d38a40001000000000000000000000000000000050000000000000002000000000000000198bd7ea46629f0169a6eac2015c367196bc7e30431b5af269323e5d189b9b4e069ab1c9d0b376e732e0d0f02bb4ce3675b87146f6e7f2bf0d342ecbf524fa59d000200065cd10d38a40000000000000000150000007b","sha256":"830a587fa92fea0be151fb5a5a0461635d2ffea4760498be6d4269815b9d5d46","bytes":742}},"prior":{"command":{"hex":"434f4e534f4c452e4944454e544954592e47524f55500000010001000000000000000000000000000000010000000000000000000000000000000f00000000000000000000000000000003000000000000000000000000000000040000000000000007000000000000000000000000000000000000000000000005000000000000000000065cd10d38a400000100000014eb8f85eba6bd20eab280eca69d20eca088ecb0a800010000000e636c61696d616e74206d61746368000000126163636f756e7420706f7373657373696f6e00000011706879736963616c2065766964656e636500000013636f6e747261646963746f727920636c61696d000000197175616c696669636174696f6e20696e737472756374696f6e00000017657363616c6174696f6e2061646a756469636174696f6e000000116d696e696d616c20726574656e74696f6e00000018726563697069656e7420726573706f6e736962696c697479","sha256":"19a911a6eb9b79ff986bddb45fc24ea17f6e01b2589f33ebcefc3fcebf63e128","bytes":356},"policy":{"hex":"434f4e534f4c452e4944454e544954592e47524f55502e504f4c4943592e4845414400000100000000000000000000000000000003000000000000000000000000000000040000000000000001000000176e61746976652d67726f75702d70726f636573732d76317e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e51297694172e4cf6afcbfb97e879eff3d02061db35cf4a0a02042724b710351dcc60b25000000000000000000000000000000010000000000000000000000000000000f19a911a6eb9b79ff986bddb45fc24ea17f6e01b2589f33ebcefc3fcebf63e1280000000000000000000000000000000e00065cbceed7efc0","sha256":"748a626ff665ced3a5bd1165facad0ef297af222154b051246bf7abe25c838b0","bytes":320},"version":{"hex":"434f4e534f4c452e4944454e544954592e50524f434553532e56455253494f4e00000100000000000000000000000000000003000000000000000000000000000000040000000000000000000000000000000500000000000000010000001947524f55505f56455249464945525f50524f434553535f5631000000000000000000000000000000010000000000000000000000000000000900000000000000010000000000000001748a626ff665ced3a5bd1165facad0ef297af222154b051246bf7abe25c838b00000000000000000000000000000000f19a911a6eb9b79ff986bddb45fc24ea17f6e01b2589f33ebcefc3fcebf63e12800065cbceec8ad8000065cd10d38a400000100000014eb8f85eba6bd20eab280eca69d20eca088ecb0a800010000000e636c61696d616e74206d61746368000000126163636f756e7420706f7373657373696f6e00000011706879736963616c2065766964656e636500000013636f6e747261646963746f727920636c61696d000000197175616c696669636174696f6e20696e737472756374696f6e00000017657363616c6174696f6e2061646a756469636174696f6e000000116d696e696d616c20726574656e74696f6e00000018726563697069656e7420726573706f6e736962696c697479","sha256":"98bd7ea46629f0169a6eac2015c367196bc7e30431b5af269323e5d189b9b4e0","bytes":481},"head":{"hex":"434f4e534f4c452e4944454e544954592e50524f434553532e484541440000010000000000000000000000000000000300000000000000000000000000000004000000000000000000000000000000050000000000000001000000000000000198bd7ea46629f0169a6eac2015c367196bc7e30431b5af269323e5d189b9b4e0000100065cd10d38a400000000000000000000000000000000010000000000000000000000000000000f19a911a6eb9b79ff986bddb45fc24ea17f6e01b2589f33ebcefc3fcebf63e1280000000000000000000000000000000e00065cbceed7efc0","sha256":"938698d8c6ad82400ead9e9c9d887179904ab0fa743182fe98c15c9fab5a1c28","bytes":226}}},"prior_head":{"group_id":"00000000-0000-0000-0000-000000000003","group_incarnation":"00000000-0000-0000-0000-000000000004","process_id":"00000000-0000-0000-0000-000000000005","head_revision":1,"content_version":1,"content_digest":"\\x98bd7ea46629f0169a6eac2015c367196bc7e30431b5af269323e5d189b9b4e0","state":"ACTIVE","expires_at":"2026-10-02T01:00:00.000000Z","last_actor_account_id":"00000000-0000-0000-0000-000000000001","last_command_id":"00000000-0000-0000-0000-00000000000f","last_input_digest":"\\x19a911a6eb9b79ff986bddb45fc24ea17f6e01b2589f33ebcefc3fcebf63e128","result_receipt_id":"00000000-0000-0000-0000-00000000000e","updated_at":"2026-10-01T00:59:51Z","head_digest":"\\x938698d8c6ad82400ead9e9c9d887179904ab0fa743182fe98c15c9fab5a1c28","before_head_digest":null},"reason":"검토한 원문 중지 사유","coherent_first_content":{"title":"다른 검증 절차","oracles":{"command":{"hex":"434f4e534f4c452e4944454e544954592e47524f55500000010001000000000000000000000000000000010000000000000000000000000000000200000000000000000000000000000003000000000000000000000000000000040000000000000007000000000000000000000000000000000000000000000005000000000000000000065cd10d38a400000100000014eb8f85eba6bd20eab280eca69d20eca088ecb0a800010000000e636c61696d616e74206d61746368000000126163636f756e7420706f7373657373696f6e00000011706879736963616c2065766964656e636500000013636f6e747261646963746f727920636c61696d000000197175616c696669636174696f6e20696e737472756374696f6e00000017657363616c6174696f6e2061646a756469636174696f6e000000116d696e696d616c20726574656e74696f6e00000018726563697069656e7420726573706f6e736962696c697479","sha256":"ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe","bytes":356},"policy":{"hex":"434f4e534f4c452e4944454e544954592e47524f55502e504f4c4943592e4845414400000100000000000000000000000000000003000000000000000000000000000000040000000000000001000000176e61746976652d67726f75702d70726f636573732d76317e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e51297694172e4cf6afcbfb97e879eff3d02061db35cf4a0a02042724b710351dcc60b250000000000000000000000000000000100000000000000000000000000000002ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe0000000000000000000000000000000700065cbcef850ece","sha256":"c4507daab34ef2a018b2f906e310eac2d76a3f01763e5b9a5268eba72694111d","bytes":320},"version":{"hex":"434f4e534f4c452e4944454e544954592e50524f434553532e56455253494f4e00000100000000000000000000000000000003000000000000000000000000000000040000000000000000000000000000000500000000000000010000001947524f55505f56455249464945525f50524f434553535f5631000000000000000000000000000000010000000000000000000000000000000900000000000000010000000000000001c4507daab34ef2a018b2f906e310eac2d76a3f01763e5b9a5268eba72694111d00000000000000000000000000000002ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe00065cbcef63264000065cd10d38a400000100000014eb8ba4eba5b820eab280eca69d20eca088ecb0a800010000000e636c61696d616e74206d61746368000000126163636f756e7420706f7373657373696f6e00000011706879736963616c2065766964656e636500000013636f6e747261646963746f727920636c61696d000000197175616c696669636174696f6e20696e737472756374696f6e00000017657363616c6174696f6e2061646a756469636174696f6e000000116d696e696d616c20726574656e74696f6e00000018726563697069656e7420726573706f6e736962696c697479","sha256":"0a37de595498f20741b24df2bb6ddfb9f4c446a81486f004d2423f741d1b0182","bytes":481},"head":{"hex":"434f4e534f4c452e4944454e544954592e50524f434553532e48454144000001000000000000000000000000000000030000000000000000000000000000000400000000000000000000000000000005000000000000000100000000000000010a37de595498f20741b24df2bb6ddfb9f4c446a81486f004d2423f741d1b0182000100065cd10d38a4000000000000000000000000000000000100000000000000000000000000000002ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe0000000000000000000000000000000700065cbcef850ece","sha256":"8b5b9c4c7cc311fee6ef90bbdda6b6feb536f37dad46cda706fad367bb59d377","bytes":226},"result":{"hex":"434f4e534f4c452e4944454e544954592e50524f434553532e524553554c54000002000000000000000000000000000000010000000000000000000000000000000200000000000000000000000000000003000000000000000000000000000000040001ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe0000000000000000000000000000000600000000000000000000000000000007000100065cbcef63264000065cbcef850ece000000000000000000000000000000080000000000000001000000000000000000000000000000090000000000000001000000000000000700010000000000000001c4507daab34ef2a018b2f906e310eac2d76a3f01763e5b9a5268eba72694111d000000176e61746976652d67726f75702d70726f636573732d76317e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e512976900000000000000014172e4cf6afcbfb97e879eff3d02061db35cf4a0a02042724b710351dcc60b2500000006342e31332e3000000003342e3500000000000000000000000000000005000100000000000000000000000000000005000000000000000100000000000000010a37de595498f20741b24df2bb6ddfb9f4c446a81486f004d2423f741d1b01828b5b9c4c7cc311fee6ef90bbdda6b6feb536f37dad46cda706fad367bb59d377000100065cd10d38a40000000000000000150000007b","sha256":"40d926ec48f807d4503cb601a1cc1a4065ce0072a05ff05465a8193c4fcd2e58","bytes":596}}},"coherent_suspend_reason":"제출하지 않은 다른 사유"}"###;
