//! External parser-only proposal. All rows are synthetic in-memory test values.
//! PostgreSQL canonical serialization, installed custody, actual Auth/Cedar and
//! failed-transaction recovery remain integration requirements.
use super::owner_jsonb_contract_tests::{fixture, literal_bytes};
use super::{material, snapshot_shapes};
use console_identity_application::group_process::*;
use serde_json::{Value, json};
use uuid::Uuid;

fn id(n: u128) -> String {
    Uuid::from_u128(n).to_string()
}
fn unavailable<T>(value: Result<T, GroupProcessError>, label: &str) {
    assert!(
        matches!(value, Err(GroupProcessError::Unavailable)),
        "{label}"
    );
}
fn bytea(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2 + 2);
    output.push_str("\\x");
    for byte in bytes {
        use std::fmt::Write;
        write!(&mut output, "{byte:02x}").unwrap();
    }
    output
}
// Convert only the independent known timestamp fixture strings, preserving
// exact microseconds. This is not an implementation of PostgreSQL jsonb::text.
fn owner_timestamp_spelling(value: &mut Value) {
    match value {
        Value::Object(object) => {
            for value in object.values_mut() {
                owner_timestamp_spelling(value)
            }
        }
        Value::Array(array) => {
            for value in array {
                owner_timestamp_spelling(value)
            }
        }
        Value::String(text)
            if text.starts_with("2026-") && text.ends_with('Z') && text.contains('T') =>
        {
            let utc = &text[..text.len() - 1];
            let canonical = if utc.contains('.') {
                utc.trim_end_matches('0').trim_end_matches('.')
            } else {
                utc
            };
            *text = format!("{canonical}+00:00");
        }
        _ => {}
    }
}
fn designation(f: &Value, revoked: bool) -> Value {
    let mut head = f["designation"].clone();
    if revoked {
        head["revision"] = json!(2);
        head["receipt_id"] = json!(id(60));
    }
    json!({"head":head,"receipt":{
        "account_id":id(1),"command_id":id(61),"database_name":"constructor_fixture","database_oid":42,
        "expected_revision":if revoked{1}else{0},"expected_security_generation":if revoked{Value::Null}else{json!(1)},
        "kind":if revoked{"REVOKE"}else{"DESIGNATE"},"reason":if revoked{json!("reviewed designation revoked")}else{Value::Null},
        "receipt_id":if revoked{id(60)}else{id(9)},"recorded_at":"2026-10-01T00:59:50+00:00",
        "revision":if revoked{2}else{1},"system_identifier":"123456789"}})
}
fn snapshot(name: &str, designation_kind: &str) -> Value {
    let f = fixture();
    let birth_request = json!({"account_id":id(1),"codec_version":1,"command_id":id(10),
        "committed_receipt_id":id(11),"created_at":"2026-10-01T00:59:49+00:00",
        "designation_receipt_id":id(9),"expires_at":"2026-10-02T01:00:00+00:00",
        "input_bytes":null,"input_digest":bytea(&[0x21;32]),"state":"COMMITTED","terminal_at":"2026-10-01T00:59:50+00:00"});
    let birth_receipt = json!({"account_id":id(1),"action_refs":[],"administrative_account_id":id(1),
        "catalog_version":"COMPANY_ENROLLMENT_V1","codec_version":1,"command_id":id(10),
        "committed_at":"2026-10-01T00:59:50+00:00","designation_receipt_id":id(9),"group_id":id(3),
        "input_digest":bytea(&[0x21;32]),"manifest_digest":bytea(&[0x22;32]),"org_id":id(50),
        "property_refs":[],"receipt_id":id(11),"root_assignment_id":id(51),"root_revision":1,"session_id":id(8)});
    let (policy, head, version, history) = if name == "empty" {
        (Value::Null, Value::Null, Value::Null, json!([]))
    } else {
        let history = if name == "first" {
            json!([{"head":f["rows"][name]["head"],"reason":null}])
        } else {
            json!([{"head":f["prior_head"],"reason":null},{"head":f["rows"][name]["head"],"reason":f["reason"]}])
        };
        (
            f["rows"][name]["policy"].clone(),
            f["rows"][name]["head"].clone(),
            f["rows"][name]["version"].clone(),
            history,
        )
    };
    let current_designation = match designation_kind {
        "none" => Value::Null,
        "revoked" => designation(&f, true),
        "active" => designation(&f, false),
        _ => panic!("unknown fixture"),
    };
    let mut value = json!({"protocol":"GROUP_PROCESS_NAVIGATION_SOURCE_V1",
        "account":{"actor_account_id":id(1),"session_id":id(8),
            "root":{"id":id(1),"created_at":"2026-10-01T00:59:40+00:00"},
            "security":{"account_id":id(1),"context_generation":1,"revision":1,"security_generation":1,"security_state":"ACTIVE","updated_at":"2026-10-01T00:59:40+00:00"},
            "family":{"account_security_generation":1,"assurance":"password","auth_time":"2026-10-01T00:59:45+00:00","created_at":"2026-10-01T00:59:45+00:00","org_id":null,"protocol":"ACCOUNT_V1","revoked_at":null,"user_id":id(1)},
            "enrollment":{"account_id":id(1),"actor_account_id":id(1),"evidence_ref":{},"id":id(52),"kind":"ENROLLED","occurred_at":"2026-10-01T00:59:40+00:00","payload":{},"session_id":id(8)},
            "consent":[{"accepted_at":"2026-10-01T00:59:40+00:00","content_sha256":bytea(&[0x23;32]),"manifest_sha256":bytea(&[0x24;32]),"terms_kind":"privacy"},
                {"accepted_at":"2026-10-01T00:59:40+00:00","content_sha256":bytea(&[0x25;32]),"manifest_sha256":bytea(&[0x24;32]),"terms_kind":"service"}],
            "terms_head":{"manifest_sha256":bytea(&[0x24;32]),"release_receipt_id":id(53),"revision":1}},
        "designation":current_designation,"groups":[{"group_id":id(3),"group_incarnation":id(4),
            "group":f["group"],"topology":f["topology"],"birth_request":birth_request,"birth_receipt":birth_receipt,
            "birth_designation_receipt":designation(&f,false)["receipt"],"source":f["source"],
            "policy":policy,"head":head,"version":version,"history":history}]});
    owner_timestamp_spelling(&mut value);
    if name == "empty" {
        value["groups"][0]["source"]["registration_manifest_digest"] =
            json!(format!("\\x{}", REGISTRATION_SHA256));
    }
    value
}
fn envelope(snapshot: &Value) -> Value {
    let candidates:Vec<_>=snapshot["groups"].as_array().into_iter().flatten().map(|group|
        json!({"group_id":group["group_id"],"group_incarnation":group["group_incarnation"]})).collect();
    json!({"protocol":"GROUP_PROCESS_NAVIGATION_V1","actor_account_id":id(1),"session_id":id(8),
        "status":"MATCH","candidates":candidates,"snapshot":bytea(&serde_json::to_vec(snapshot).unwrap())})
}
fn parse(
    snapshot: &Value,
) -> Result<(GroupProcessNavigationCandidatesV1, Vec<(Vec<u8>, [u8; 32])>), GroupProcessError> {
    material::navigation(
        &envelope(snapshot),
        Uuid::from_u128(1),
        Uuid::from_u128(8),
        false,
    )
}
fn expected_checks(f: &Value, name: &str) -> Vec<(Vec<u8>, [u8; 32])> {
    let stored = if name == "empty" {
        REGISTRATION_SHA256
    } else {
        f["source"]["registration_manifest_digest"]
            .as_str()
            .unwrap()
            .strip_prefix("\\x")
            .unwrap()
    };
    let mut checks = vec![(
        literal_bytes(REGISTRATION_HEX),
        literal_bytes(stored).try_into().unwrap(),
    )];
    if name != "empty" {
        let mut oracles = vec![
            &f["oracles"][name]["policy"],
            &f["oracles"][name]["head"],
            &f["oracles"][name]["version"],
        ];
        if name == "suspend" {
            oracles.push(&f["oracles"]["prior"]["head"])
        }
        oracles.push(&f["oracles"][name]["head"]);
        checks.extend(oracles.into_iter().map(|o| {
            (
                literal_bytes(o["hex"].as_str().unwrap()),
                literal_bytes(o["sha256"].as_str().unwrap())
                    .try_into()
                    .unwrap(),
            )
        }));
    }
    checks
}

#[test]
fn group_navigation_snapshot_keeps_complete_denied_and_current_preimage_work() {
    let f = fixture();
    for name in ["empty", "first", "suspend"] {
        for designation in ["none", "active", "revoked"] {
            let value = snapshot(name, designation);
            let raw = serde_json::to_vec(&value).unwrap();
            let (candidates, checks) = parse(&value).unwrap();
            assert_eq!(candidates.candidates().len(), 1);
            assert_eq!(candidates.snapshot().source_bytes(), raw);
            assert_eq!(
                checks,
                expected_checks(&f, name),
                "denied designation omitted a source/policy/content/history preimage"
            );
            assert_eq!(
                checks.len(),
                match name {
                    "empty" => 1,
                    "first" => 5,
                    "suspend" => 6,
                    _ => unreachable!(),
                }
            );
        }
    }
    let mut value = snapshot("empty", "none");
    value["groups"] = json!([]);
    let (candidates, checks) = parse(&value).unwrap();
    assert!(candidates.candidates().is_empty());
    assert!(checks.is_empty());
}
fn fixed_object_paths(value: &Value, path: &str, output: &mut Vec<String>) {
    // payload/evidence_ref are deliberately open objects in the SQL roster;
    // their interior is not falsely promoted into a closed ABI schema.
    if path.ends_with("/payload") || path.ends_with("/evidence_ref") {
        return;
    }
    match value {
        Value::Object(object) => {
            output.push(path.into());
            for (key, value) in object {
                fixed_object_paths(value, &format!("{path}/{key}"), output);
            }
        }
        Value::Array(array) => {
            for (index, value) in array.iter().enumerate() {
                fixed_object_paths(value, &format!("{path}/{index}"), output);
            }
        }
        _ => {}
    }
}
#[test]
fn group_navigation_snapshot_rejects_omitted_and_unknown_closed_members() {
    assert_eq!(
        serde_json::from_str::<Value>(snapshot_shapes::SHAPES).unwrap(),
        serde_json::from_str::<Value>(SQL_SHAPES).unwrap(),
        "frozen SQL ABI roster drift"
    );
    for name in ["first", "suspend"] {
        let value = snapshot(name, "active");
        parse(&value).unwrap();
        let mut paths = Vec::new();
        fixed_object_paths(&value, "", &mut paths);
        for path in paths {
            let members: Vec<_> = value
                .pointer(&path)
                .unwrap()
                .as_object()
                .unwrap()
                .keys()
                .cloned()
                .collect();
            for member in members {
                let mut corrupt = value.clone();
                corrupt
                    .pointer_mut(&path)
                    .unwrap()
                    .as_object_mut()
                    .unwrap()
                    .remove(&member);
                unavailable(parse(&corrupt), &format!("omitted {path}/{member}"));
            }
            let mut corrupt = value.clone();
            corrupt
                .pointer_mut(&path)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .insert("unexpected_member".into(), Value::Null);
            unavailable(parse(&corrupt), &format!("unknown member {path}"));
        }
    }
}
#[test]
fn group_navigation_snapshot_rejects_protocol_identity_roster_sort_and_nullable_corruption() {
    let value = snapshot("suspend", "active");
    parse(&value).unwrap();
    let independent_shapes: Value = serde_json::from_str(SQL_SHAPES).unwrap();
    for (path, shape_name) in [
        ("/account/root", "accounts"),
        ("/account/security", "account_security"),
        ("/account/family", "family"),
        ("/account/enrollment", "account_security_events"),
        ("/account/consent/0", "consent"),
        ("/account/consent/1", "consent"),
        ("/account/terms_head", "terms_head"),
        ("/designation/head", "deployment_operator_head"),
        ("/designation/receipt", "deployment_operator_receipts"),
        ("/groups/0/group", "groups"),
        ("/groups/0/topology", "group_authority_heads"),
        ("/groups/0/birth_request", "company_enrollment_requests"),
        ("/groups/0/birth_receipt", "company_enrollment_receipts"),
        (
            "/groups/0/birth_designation_receipt",
            "deployment_operator_receipts",
        ),
        ("/groups/0/source", "source"),
        ("/groups/0/policy", "native_group_identity_policy_heads_v1"),
        ("/groups/0/head", "native_group_process_heads_v1"),
        ("/groups/0/version", "native_group_process_versions_v1"),
        (
            "/groups/0/history/0/head",
            "native_group_process_head_revisions_v1",
        ),
        (
            "/groups/0/history/1/head",
            "native_group_process_head_revisions_v1",
        ),
    ] {
        for (key, rule) in independent_shapes[shape_name].as_object().unwrap() {
            let member_path = format!("{path}/{key}");
            let wrong_type = match rule[0].as_str().unwrap() {
                "uuid" | "string" => json!(1),
                "timestamp" => json!(false),
                "integer" => json!("1"),
                "bytea" | "object" => json!([]),
                "array" => json!({}),
                _ => panic!("unknown independent SQL type"),
            };
            let mut corrupt = value.clone();
            *corrupt.pointer_mut(&member_path).unwrap() = wrong_type;
            unavailable(parse(&corrupt), &format!("wrong SQL type {member_path}"));
            if rule[1] == json!(false) {
                let mut corrupt = value.clone();
                *corrupt.pointer_mut(&member_path).unwrap() = Value::Null;
                unavailable(
                    parse(&corrupt),
                    &format!("forbidden SQL null {member_path}"),
                );
            }
        }
    }
    let cases = [
        ("/protocol", json!("GROUP_PROCESS_NAVIGATION_SOURCE_V2")),
        ("/account/actor_account_id", json!(id(99))),
        ("/account/session_id", json!(id(99))),
        ("/account/root/id", json!(id(99))),
        ("/account/security/account_id", json!(id(99))),
        ("/account/family/user_id", json!(id(99))),
        ("/account/enrollment/account_id", json!(id(99))),
        ("/account/enrollment/kind", json!("SUSPENDED")),
        ("/account/family/org_id", json!(false)),
        ("/account/family/revoked_at", json!(false)),
        (
            "/designation/receipt/expected_security_generation",
            json!("1"),
        ),
        ("/groups/0/birth_request/input_bytes", json!(false)),
        ("/groups/0/group_id", json!(Uuid::nil().to_string())),
        ("/groups/0/topology/incarnation", json!(id(99))),
        ("/groups/0/group/origin_command_id", json!(id(99))),
        ("/groups/0/birth_request/state", json!("PENDING")),
        ("/groups/0/birth_receipt/group_id", json!(id(99))),
        (
            "/groups/0/source/registered_actions/0/key",
            json!("unknown.action/1"),
        ),
        ("/groups/0/source/registered_actions/0/fields", json!([1])),
        ("/groups/0/policy", Value::Null),
        ("/groups/0/head", Value::Null),
        ("/groups/0/version", Value::Null),
        ("/groups/0/history", json!([])),
        (
            "/groups/0/history/1/head/before_head_digest",
            json!(format!("\\x{}", "00".repeat(32))),
        ),
        (
            "/groups/0/history/0/reason",
            json!("not an adoption reason"),
        ),
        ("/groups/0/history/1/reason", Value::Null),
        ("/account/consent/0/terms_kind", json!("Privacy")),
    ];
    for (path, replacement) in cases {
        let mut corrupt = value.clone();
        *corrupt.pointer_mut(path).unwrap() = replacement;
        unavailable(parse(&corrupt), path);
    }
    for kind in 0..5 {
        let mut corrupt = value.clone();
        match kind {
            0 => {
                corrupt["account"]["consent"]
                    .as_array_mut()
                    .unwrap()
                    .swap(0, 1);
            }
            1 => {
                corrupt["account"]["consent"][1] = corrupt["account"]["consent"][0].clone();
            }
            2 => {
                corrupt["account"]["consent"] = json!([]);
            }
            3 => {
                let repeated = corrupt["account"]["consent"][0].clone();
                corrupt["account"]["consent"] = json!(vec![repeated; 9]);
            }
            4 => {
                corrupt["groups"][0]["source"]["registered_actions"]
                    .as_array_mut()
                    .unwrap()
                    .swap(0, 1);
                corrupt["groups"][0]["policy"]["registered_actions"]
                    .as_array_mut()
                    .unwrap()
                    .swap(0, 1);
            }
            _ => unreachable!(),
        }
        unavailable(parse(&corrupt), &format!("sort/roster {kind}"));
    }
}
#[test]
fn group_navigation_envelope_rejects_overflow_duplicates_and_wrong_conflict_grammar() {
    let snapshot = snapshot("empty", "none");
    let value = envelope(&snapshot);
    material::navigation(&value, Uuid::from_u128(1), Uuid::from_u128(8), false).unwrap();
    for path in ["", "/candidates/0"] {
        let members: Vec<_> = value
            .pointer(path)
            .unwrap()
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect();
        for member in members {
            let mut corrupt = value.clone();
            corrupt
                .pointer_mut(path)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .remove(&member);
            unavailable(
                material::navigation(&corrupt, Uuid::from_u128(1), Uuid::from_u128(8), false),
                &format!("omitted envelope member {path}/{member}"),
            );
        }
        let mut corrupt = value.clone();
        corrupt
            .pointer_mut(path)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("unexpected_member".into(), Value::Null);
        unavailable(
            material::navigation(&corrupt, Uuid::from_u128(1), Uuid::from_u128(8), false),
            &format!("unknown envelope member {path}"),
        );
    }
    for (path, replacement) in [
        ("/protocol", json!("bad")),
        ("/actor_account_id", json!(id(99))),
        ("/session_id", json!(id(99))),
        ("/status", json!("unknown")),
        ("/snapshot", Value::Null),
        ("/candidates/0/group_id", json!(Uuid::nil().to_string())),
    ] {
        let mut corrupt = value.clone();
        *corrupt.pointer_mut(path).unwrap() = replacement;
        unavailable(
            material::navigation(&corrupt, Uuid::from_u128(1), Uuid::from_u128(8), false),
            path,
        );
    }
    for count in [2, 257] {
        let mut corrupt = value.clone();
        corrupt["candidates"] = json!(vec![value["candidates"][0].clone(); count]);
        unavailable(
            material::navigation(&corrupt, Uuid::from_u128(1), Uuid::from_u128(8), false),
            "duplicate/overflow",
        );
    }
    let mut corrupt = value.clone();
    corrupt["candidates"] = json!([
        {"group_id":id(99),"group_incarnation":id(4)},{"group_id":id(3),"group_incarnation":id(4)}]);
    unavailable(
        material::navigation(&corrupt, Uuid::from_u128(1), Uuid::from_u128(8), false),
        "unsorted candidates",
    );
    // SQL accepts at most one captured incarnation per Group. A pair-sorted
    // constructor would mistakenly admit these two complete empty snapshots.
    let mut duplicate_group_snapshot = snapshot.clone();
    let mut second = duplicate_group_snapshot["groups"][0].clone();
    second["group_incarnation"] = json!(id(5));
    second["topology"]["incarnation"] = json!(id(5));
    duplicate_group_snapshot["groups"]
        .as_array_mut()
        .unwrap()
        .push(second);
    let duplicate_group = envelope(&duplicate_group_snapshot);
    unavailable(
        material::navigation(
            &duplicate_group,
            Uuid::from_u128(1),
            Uuid::from_u128(8),
            false,
        ),
        "duplicate Group with different captured incarnation",
    );
    let mut corrupt = value.clone();
    corrupt["candidates"] = json!([]);
    unavailable(
        material::navigation(&corrupt, Uuid::from_u128(1), Uuid::from_u128(8), false),
        "snapshot count mismatch",
    );
    let mut conflict = value.clone();
    conflict["status"] = json!("CONFLICT");
    conflict["candidates"] = json!([]);
    conflict["snapshot"] = Value::Null;
    assert!(matches!(
        material::navigation(&conflict, Uuid::from_u128(1), Uuid::from_u128(8), true),
        Err(GroupProcessError::Conflict)
    ));
    unavailable(
        material::navigation(&conflict, Uuid::from_u128(1), Uuid::from_u128(8), false),
        "enumeration conflict",
    );
    for (path, replacement) in [
        ("/snapshot", value["snapshot"].clone()),
        ("/candidates", value["candidates"].clone()),
    ] {
        let mut corrupt = conflict.clone();
        *corrupt.pointer_mut(path).unwrap() = replacement;
        unavailable(
            material::navigation(&corrupt, Uuid::from_u128(1), Uuid::from_u128(8), true),
            "conflict carries data",
        );
    }
    for raw in [vec![0xff], vec![], vec![b' '; 1_048_577], b"{}".to_vec()] {
        let mut corrupt = value.clone();
        corrupt["snapshot"] = json!(bytea(&raw));
        unavailable(
            material::navigation(&corrupt, Uuid::from_u128(1), Uuid::from_u128(8), false),
            "snapshot encoding/size",
        );
    }
}
// Offsets independently declared by the frozen V1 field layouts: head prefix32
// + Group/incarnation/process UUID48 + head/content revisions16 => content96.
// Version prefix35 + UUID48 + revision8 + schema text29 + actor/designation UUID32
// + designation/policy revisions16 => policy-head digest168.
fn replace_single_literal_digest(
    preimage: &mut [u8],
    original: &[u8],
    replacement: &[u8],
    offset: usize,
) {
    assert_eq!(original.len(), 32);
    assert_eq!(replacement.len(), 32);
    let sites: Vec<_> = preimage
        .windows(32)
        .enumerate()
        .filter_map(|(index, bytes)| (bytes == original).then_some(index))
        .collect();
    assert_eq!(
        sites,
        vec![offset],
        "independent literal field offset/occurrence drift"
    );
    preimage[offset..offset + 32].copy_from_slice(replacement);
}
#[test]
fn group_navigation_snapshot_retains_every_corrupt_digest_for_independent_batch() {
    let f = fixture();
    let value = snapshot("suspend", "none");
    let (_, checks) = parse(&value).unwrap();
    assert_eq!(checks, expected_checks(&f, "suspend"));
    let mut empty = snapshot("empty", "none");
    empty["groups"][0]["source"]["registration_manifest_digest"] =
        json!(format!("\\x{}", "11".repeat(32)));
    let (_, registration_checks) = parse(&empty).unwrap();
    assert_eq!(registration_checks.len(), 1);
    assert_eq!(registration_checks[0].0, literal_bytes(REGISTRATION_HEX));
    assert_eq!(registration_checks[0].1, [0x11; 32]);
    for path in [
        "/groups/0/policy/head_digest",
        "/groups/0/head/head_digest",
        "/groups/0/version/content_digest",
        "/groups/0/history/0/head/head_digest",
    ] {
        let mut corrupt = value.clone();
        let bad = json!(format!("\\x{}", "11".repeat(32)));
        // Preserve cross-reference/chain closure so only queued SHA work can
        // detect this corruption, rather than a cheap reference mismatch.
        match path {
            "/groups/0/policy/head_digest" => {
                corrupt["groups"][0]["policy"]["head_digest"] = bad.clone();
                corrupt["groups"][0]["version"]["policy_head_digest"] = bad.clone();
            }
            "/groups/0/head/head_digest" => {
                corrupt["groups"][0]["head"]["head_digest"] = bad.clone();
                corrupt["groups"][0]["history"][1]["head"]["head_digest"] = bad.clone();
            }
            "/groups/0/version/content_digest" => {
                corrupt["groups"][0]["version"]["content_digest"] = bad.clone();
                corrupt["groups"][0]["head"]["content_digest"] = bad.clone();
                for entry in corrupt["groups"][0]["history"].as_array_mut().unwrap() {
                    entry["head"]["content_digest"] = bad.clone();
                }
            }
            "/groups/0/history/0/head/head_digest" => {
                corrupt["groups"][0]["history"][0]["head"]["head_digest"] = bad.clone();
                corrupt["groups"][0]["head"]["before_head_digest"] = bad.clone();
                corrupt["groups"][0]["history"][1]["head"]["before_head_digest"] = bad.clone();
            }
            _ => unreachable!(),
        }
        let mut expected = expected_checks(&f, "suspend");
        let old_policy = literal_bytes(
            f["oracles"]["suspend"]["policy"]["sha256"]
                .as_str()
                .unwrap(),
        );
        let old_version = literal_bytes(
            f["oracles"]["suspend"]["version"]["sha256"]
                .as_str()
                .unwrap(),
        );
        let bad_digest = [0x11; 32];
        // Exact independent queue order is registration0, policy1, currenthead2,
        // version3, historicalprior4, historicalcurrent5. Every unrelated tuple
        // remains identical; references included by an encoder alter its bytes.
        match path {
            "/groups/0/policy/head_digest" => {
                expected[1].1 = bad_digest;
                replace_single_literal_digest(&mut expected[3].0, &old_policy, &bad_digest, 168);
            }
            "/groups/0/head/head_digest" => {
                expected[2].1 = bad_digest;
                expected[5].1 = bad_digest;
            }
            "/groups/0/version/content_digest" => {
                expected[3].1 = bad_digest;
                for index in [2, 4, 5] {
                    replace_single_literal_digest(
                        &mut expected[index].0,
                        &old_version,
                        &bad_digest,
                        96,
                    );
                }
            }
            "/groups/0/history/0/head/head_digest" => {
                expected[4].1 = bad_digest;
                // before_head_digest is deliberately outside the head preimage.
            }
            _ => unreachable!(),
        }
        let (_, mutated) = parse(&corrupt).unwrap();
        assert_eq!(
            mutated, expected,
            "corruption queue changed/omitted/duplicated a protected preimage: {path}"
        );
    }
}
#[test]
fn group_navigation_snapshot_enforces_fixed_sql_integer_digest_and_timestamp_rosters() {
    let value = snapshot("empty", "active");
    parse(&value).unwrap();
    // These are intentionally RED on frozen V3's generic row helper. Exact SQL
    // owner ABI supplies the oracle; root owns any independently reviewed fix.
    for (path, replacement) in [
        ("/account/security/revision", json!(-1)),
        (
            "/account/root/created_at",
            json!("0000-01-01T00:00:00+00:00"),
        ),
        ("/account/security/security_generation", json!(0)),
        (
            "/account/terms_head/manifest_sha256",
            json!(bytea(&[0x24; 31])),
        ),
        ("/account/consent/0/content_sha256", json!(bytea(&[]))),
        ("/account/terms_head/revision", json!(1.0)),
        ("/account/root/created_at", json!("2026-10-01T00:59:40Z")),
        (
            "/account/root/created_at",
            json!("2026-10-01T09:59:40+09:00"),
        ),
        (
            "/account/root/created_at",
            json!("2026-10-01T00:59:40.000000+00:00"),
        ),
        (
            "/account/root/created_at",
            json!("2026-10-01T00:59:40.120000+00:00"),
        ),
    ] {
        let mut corrupt = value.clone();
        *corrupt.pointer_mut(path).unwrap() = replacement;
        unavailable(parse(&corrupt), path);
    }
    for timestamp in [
        "0001-01-01T00:00:00+00:00",
        "9999-12-31T23:59:59.999999+00:00",
        "2026-10-01T00:59:40+00:00",
        "2026-10-01T00:59:40.12+00:00",
        "2026-10-01T00:59:40.123456+00:00",
    ] {
        let mut positive = value.clone();
        positive["account"]["root"]["created_at"] = json!(timestamp);
        parse(&positive).unwrap();
    }
    let mut positive = value.clone();
    positive["designation"]["receipt"]["expected_revision"] = json!(0);
    parse(&positive).unwrap();
}

const REGISTRATION_SHA256: &str =
    "f746e5d23756ddf8b0d6424f3072cf84b9827da681e0ba951804823c7f2923e8";
const REGISTRATION_HEX: &str = "434f4e534f4c452e4944454e544954592e47524f55502e50524f434553532e524547495354524154494f4e00000100000000000000000000000000000003000000000000000000000000000000040000000000000001000000176e61746976652d67726f75702d70726f636573732d76317e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e51297690004000000216964656e746974792e76657269666965722e70726f636573732e61646f70742f310000000000000000000000000000001f000000000000000100120000000a70726f636573735f69640000000f70726f636573735f76657273696f6e0000000e70726f636573735f6469676573740000001570726f636573735f686561645f7265766973696f6e0000001370726f636573735f686561645f64696765737400000005737461746500000006657870697279000000057469746c65000000066d6574686f6400000024696e74656e6465645f636c61696d616e745f6d61746368696e675f70726f6365647572650000001c6163636f756e745f706f7373657373696f6e5f70726f63656475726500000021706879736963616c5f68756d616e5f65766964656e63655f70726f636564757265000000276475706c69636174655f636f6e747261646963746f72795f636c61696d5f70726f636564757265000000227175616c696669636174696f6e5f63726974657269615f696e737472756374696f6e00000021657363616c6174696f6e5f61646a756469636174696f6e5f70726f6365647572650000002b65766964656e63655f6d696e696d697a6174696f6e5f726574656e74696f6e5f6465736372697074696f6e00000018726563697069656e745f726573706f6e736962696c6974790000000f726563656970745f6c6f6361746f72000000236964656e746974792e76657269666965722e70726f636573732e73757370656e642f3100000000000000000000000000000020000000000000000100090000000a70726f636573735f69640000000f70726f636573735f76657273696f6e0000000e70726f636573735f6469676573740000001570726f636573735f686561645f7265766973696f6e0000001370726f636573735f686561645f6469676573740000000573746174650000000665787069727900000006726561736f6e0000000f726563656970745f6c6f6361746f72000000206964656e746974792e76657269666965722e70726f636573732e726561642f3100000000000000000000000000000021000000000000000100140000000d67726f75705f636f6e746578740000000a70726f636573735f69640000000f70726f636573735f76657273696f6e0000000e70726f636573735f6469676573740000001570726f636573735f686561645f7265766973696f6e0000001370726f636573735f686561645f64696765737400000005737461746500000006657870697279000000057469746c65000000066d6574686f6400000024696e74656e6465645f636c61696d616e745f6d61746368696e675f70726f6365647572650000001c6163636f756e745f706f7373657373696f6e5f70726f63656475726500000021706879736963616c5f68756d616e5f65766964656e63655f70726f636564757265000000276475706c69636174655f636f6e747261646963746f72795f636c61696d5f70726f636564757265000000227175616c696669636174696f6e5f63726974657269615f696e737472756374696f6e00000021657363616c6174696f6e5f61646a756469636174696f6e5f70726f6365647572650000002b65766964656e63655f6d696e696d697a6174696f6e5f726574656e74696f6e5f6465736372697074696f6e00000018726563697069656e745f726573706f6e736962696c69747900000007686973746f72790000000f616c6c6f7765645f616374696f6e730000002c6964656e746974792e76657269666965722e70726f636573732e726563656970742e726561642d6f776e2f31000000000000000000000000000000220000000000000001000d0000000a636f6d6d616e645f69640000000c696e7075745f64696765737400000011696e74616b655f726563656970745f696400000011726573756c745f726563656970745f69640000000d7465726d696e616c5f636f64650000000b61636365707465645f61740000000b65786563757465645f61740000000a70726f636573735f69640000000b6265666f72655f686561640000000a61667465725f68656164000000106f726967696e616c5f636f6e74656e740000000f6f726967696e616c5f726561736f6e0000000f726563656970745f6c6f6361746f72";
const SQL_SHAPES: &str = r###"{"account_security":{"account_id":["uuid",false],"context_generation":["integer",false],"revision":["integer",false],"security_generation":["integer",false],"security_state":["string",false],"updated_at":["timestamp",false]},"account_security_events":{"account_id":["uuid",false],"actor_account_id":["uuid",false],"evidence_ref":["object",false],"id":["uuid",false],"kind":["string",false],"occurred_at":["timestamp",false],"payload":["object",false],"session_id":["uuid",false]},"accounts":{"created_at":["timestamp",false],"id":["uuid",false]},"company_enrollment_receipts":{"account_id":["uuid",false],"action_refs":["array",false],"administrative_account_id":["uuid",false],"catalog_version":["string",false],"codec_version":["integer",false],"command_id":["uuid",false],"committed_at":["timestamp",false],"designation_receipt_id":["uuid",false],"group_id":["uuid",false],"input_digest":["bytea",false],"manifest_digest":["bytea",false],"org_id":["uuid",false],"property_refs":["array",false],"receipt_id":["uuid",false],"root_assignment_id":["uuid",false],"root_revision":["integer",false],"session_id":["uuid",false]},"company_enrollment_requests":{"account_id":["uuid",false],"codec_version":["integer",false],"command_id":["uuid",false],"committed_receipt_id":["uuid",false],"created_at":["timestamp",false],"designation_receipt_id":["uuid",false],"expires_at":["timestamp",false],"input_bytes":["bytea",true],"input_digest":["bytea",false],"state":["string",false],"terminal_at":["timestamp",false]},"consent":{"accepted_at":["timestamp",false],"content_sha256":["bytea",false],"manifest_sha256":["bytea",false],"terms_kind":["string",false]},"deployment_operator_head":{"account_id":["uuid",false],"database_name":["string",false],"database_oid":["integer",false],"receipt_id":["uuid",false],"revision":["integer",false],"singleton":["integer",false],"system_identifier":["string",false]},"deployment_operator_receipts":{"account_id":["uuid",false],"command_id":["uuid",false],"database_name":["string",false],"database_oid":["integer",false],"expected_revision":["integer",false],"expected_security_generation":["integer",true],"kind":["string",false],"reason":["string",true],"receipt_id":["uuid",false],"recorded_at":["timestamp",false],"revision":["integer",false],"system_identifier":["string",false]},"family":{"account_security_generation":["integer",false],"assurance":["string",false],"auth_time":["timestamp",false],"created_at":["timestamp",false],"org_id":["uuid",true],"protocol":["string",false],"revoked_at":["timestamp",true],"user_id":["uuid",false]},"group_authority_heads":{"group_id":["uuid",false],"incarnation":["uuid",false],"revision":["integer",false],"state":["string",false]},"groups":{"created_at":["timestamp",false],"id":["uuid",false],"name":["string",false],"origin_account_id":["uuid",false],"origin_command_id":["uuid",false],"origin_receipt_id":["uuid",false],"slug":["string",false],"status":["string",false],"updated_at":["timestamp",false]},"native_group_identity_policy_heads_v1":{"activated_at":["timestamp",false],"activation_receipt_id":["uuid",false],"codec_contract_digest":["bytea",false],"first_actor_account_id":["uuid",false],"first_command_id":["uuid",false],"first_input_digest":["bytea",false],"group_id":["uuid",false],"group_incarnation":["uuid",false],"head_digest":["bytea",false],"policy_digest":["bytea",false],"registered_actions":["array",false],"registration_manifest_digest":["bytea",false],"registration_manifest_version":["integer",false],"revision":["integer",false],"schema_digest":["bytea",false],"schema_id":["string",false]},"native_group_process_head_revisions_v1":{"before_head_digest":["bytea",true],"content_digest":["bytea",false],"content_version":["integer",false],"expires_at":["timestamp",false],"group_id":["uuid",false],"group_incarnation":["uuid",false],"head_digest":["bytea",false],"head_revision":["integer",false],"last_actor_account_id":["uuid",false],"last_command_id":["uuid",false],"last_input_digest":["bytea",false],"process_id":["uuid",false],"result_receipt_id":["uuid",false],"state":["string",false],"updated_at":["timestamp",false]},"native_group_process_heads_v1":{"before_head_digest":["bytea",true],"content_digest":["bytea",false],"content_version":["integer",false],"expires_at":["timestamp",false],"group_id":["uuid",false],"group_incarnation":["uuid",false],"head_digest":["bytea",false],"head_revision":["integer",false],"last_actor_account_id":["uuid",false],"last_command_id":["uuid",false],"last_input_digest":["bytea",false],"process_id":["uuid",false],"result_receipt_id":["uuid",false],"state":["string",false],"updated_at":["timestamp",false]},"native_group_process_versions_v1":{"account_possession_procedure":["string",false],"actor_account_id":["uuid",false],"admitted_at":["timestamp",false],"adopt_command_id":["uuid",false],"content_digest":["bytea",false],"designation_receipt_id":["uuid",false],"designation_revision":["integer",false],"duplicate_contradictory_claim_procedure":["string",false],"escalation_adjudication_procedure":["string",false],"evidence_minimization_retention_description":["string",false],"expires_at":["timestamp",false],"group_id":["uuid",false],"group_incarnation":["uuid",false],"input_digest":["bytea",false],"intended_claimant_matching_procedure":["string",false],"method":["string",false],"operator_responsibility":["integer",false],"physical_human_evidence_procedure":["string",false],"policy_head_digest":["bytea",false],"policy_revision":["integer",false],"process_id":["uuid",false],"qualification_criteria_instruction":["string",false],"recipient_responsibility":["string",false],"schema_id":["string",false],"title":["string",false],"version":["integer",false]},"source":{"cedar_language_version":["string",false],"cedar_sdk_version":["string",false],"codec_contract_digest":["bytea",false],"policy_digest":["bytea",false],"registered_actions":["array",false],"registration_manifest_digest":["bytea",false],"registration_manifest_version":["integer",false],"schema_digest":["bytea",false],"schema_id":["string",false]},"terms_head":{"manifest_sha256":["bytea",false],"release_receipt_id":["uuid",false],"revision":["integer",false]}}"###;
