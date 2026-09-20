// Include inside platform/authz/src/company_policy.rs's cfg(test) module.
// Pure inputs are NOT authenticated fixtures or committed database authority.
use super::CompanyPolicy;
use console_identity_application::company_policy::{
    AccountId, ActionRef, CompanyPolicyDecision, CompanyPolicyDecisionPort, CompanyPolicyError,
    CompanyPolicyRequest, CompanyProjectionRow, CurrentCompanyAuthority, PropertyRef,
};
use console_kernel_core::OrgId;
use serde_json::{Value, json};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use uuid::Uuid;

fn input_row() -> (AccountId, OrgId, OffsetDateTime, CompanyProjectionRow) {
    let org = Uuid::from_u128(1);
    let account = AccountId::from_uuid(Uuid::from_u128(2)).unwrap();
    let company_type = Uuid::from_u128(3);
    let assignment_type = Uuid::from_u128(4);
    let digest = "0d3d0c3bc0357c0394b02400295f77231178cd5dc22a668a90880fc92a089935";
    let fields = |object: Uuid, start: u128, count: u128| {
        (start..start+count).map(|id|json!({
        "org_id":org,"object_type_id":object,"property_id":Uuid::from_u128(id),"schema_revision":"1"})).collect::<Vec<_>>()
    };
    let mut clauses = Vec::new();
    for (action, object, field_set, delegable) in [
        (11, company_type, fields(company_type, 21, 2), false),
        (12, company_type, fields(company_type, 21, 2), false),
        (13, assignment_type, fields(assignment_type, 31, 8), false),
        (14, assignment_type, vec![], false),
        (15, assignment_type, vec![], false),
        (11, company_type, fields(company_type, 21, 2), true),
        (12, company_type, fields(company_type, 21, 2), true),
    ] {
        clauses.push(json!({"kind":"COMPANY_CAPABILITY_CLAUSE_V1","action":{
            "org_id":org,"object_type_id":object,"action_type_id":Uuid::from_u128(action),"registration_revision":"1","manifest_digest":digest},
            "resource":{"kind":"COMPANY","org_id":org},"fields":field_set,
            "valid_from":"2026-09-20T00:00:00.000000Z","valid_until":null,"delegable":delegable}));
    }
    (
        account,
        OrgId::from_uuid(org),
        OffsetDateTime::parse("2026-09-20T00:00:01.000000Z", &Rfc3339).unwrap(),
        CompanyProjectionRow {
            company_epoch: 1,
            context_generation: 2,
            assignment_id: Uuid::from_u128(78),
            assignment_revision: 1,
            role_id: Uuid::from_u128(77),
            role_revision: 1,
            registered_clauses: serde_json::to_string(&clauses).unwrap(),
            company_name: "긴 한국어 회사 이름".into(),
            company_slug: "current-company".into(),
        },
    )
}

fn authority() -> CurrentCompanyAuthority {
    let (a, c, t, row) = input_row();
    CurrentCompanyAuthority::from_initial_projection(a, c, t, row).unwrap()
}
fn request_for(a: &CurrentCompanyAuthority, index: usize) -> CompanyPolicyRequest {
    let clause = &a.clauses()[index];
    let object = if !(2..=4).contains(&index) {
        *a.company().as_uuid()
    } else {
        a.assignment_id()
    };
    CompanyPolicyRequest::new(
        a.company(),
        clause.action().object_type_id(),
        object,
        clause.action().clone(),
        clause.fields().to_vec(),
    )
    .unwrap()
}

#[test]
fn checked_initial_projection_rejects_invalid_revision_and_correlated_material() {
    assert_eq!(authority().clauses().len(), 7);
    for field in [
        "role_revision",
        "assignment_revision",
        "company_epoch",
        "context_generation",
    ] {
        let (a, c, t, mut row) = input_row();
        match field {
            "role_revision" => row.role_revision = 2,
            "assignment_revision" => row.assignment_revision = 2,
            "company_epoch" => row.company_epoch = 2,
            "context_generation" => row.context_generation = 0,
            _ => unreachable!(),
        };
        assert!(
            matches!(
                CurrentCompanyAuthority::from_initial_projection(a, c, t, row),
                Err(CompanyPolicyError::MaterialUnavailable)
            ),
            "bad {field}"
        );
    }
    for corruption in [
        "extra",
        "missing_nullable",
        "duplicate_field",
        "wrong_company",
        "wrong_type",
        "wrong_manifest",
        "wrong_registration",
        "delegation_as_use",
        "later_validity",
        "wrong_action_alias",
        "field_order",
        "missing_clause",
    ] {
        let (a, c, t, mut row) = input_row();
        let mut clauses: Value = serde_json::from_str(&row.registered_clauses).unwrap();
        match corruption {
            "extra" => clauses[0]["authorized"] = json!(true),
            "missing_nullable" => {
                clauses[0].as_object_mut().unwrap().remove("valid_until");
            }
            "duplicate_field" => clauses[0]["fields"][1] = clauses[0]["fields"][0].clone(),
            "wrong_company" => clauses[0]["action"]["org_id"] = json!(Uuid::from_u128(99)),
            "wrong_type" => clauses[0]["fields"][0]["object_type_id"] = json!(Uuid::from_u128(4)),
            "wrong_manifest" => clauses[0]["action"]["manifest_digest"] = json!("0".repeat(64)),
            "wrong_registration" => clauses[0]["action"]["registration_revision"] = json!("2"),
            "delegation_as_use" => clauses[5]["delegable"] = json!(false),
            "later_validity" => clauses[0]["valid_until"] = json!("2026-09-21T00:00:00.000000Z"),
            "wrong_action_alias" => clauses[5]["action"] = clauses[2]["action"].clone(),
            "field_order" => clauses[0]["fields"].as_array_mut().unwrap().reverse(),
            "missing_clause" => {
                clauses.as_array_mut().unwrap().pop();
            }
            _ => unreachable!(),
        }
        row.registered_clauses = serde_json::to_string(&clauses).unwrap();
        assert!(
            matches!(
                CurrentCompanyAuthority::from_initial_projection(a, c, t, row),
                Err(CompanyPolicyError::MaterialUnavailable)
            ),
            "accepted {corruption}"
        );
    }
    let (a, c, t, mut row) = input_row();
    row.registered_clauses = row.registered_clauses.replacen(
        "\"delegable\":false",
        "\"delegable\":false,\"delegable\":false",
        1,
    );
    assert!(
        matches!(
            CurrentCompanyAuthority::from_initial_projection(a, c, t, row),
            Err(CompanyPolicyError::MaterialUnavailable)
        ),
        "duplicate input was collapsed before strict conversion"
    );
}

#[test]
fn actual_cedar_initial_use_and_denials_reach_sdk_boundary() {
    use std::sync::atomic::Ordering;
    let a = authority();
    let policy = CompanyPolicy::new().unwrap();
    for index in 0..5 {
        let request = request_for(&a, index);
        policy.sdk_calls.store(0, Ordering::SeqCst);
        assert!(matches!(
            policy.decide(&a, &request),
            Ok(CompanyPolicyDecision::Allow)
        ));
        assert_eq!(
            policy.sdk_calls.load(Ordering::SeqCst),
            1,
            "real authorizer bypassed"
        );
    }
    let read = &a.clauses()[1];
    for request in [
        CompanyPolicyRequest::new(
            OrgId::from_uuid(Uuid::from_u128(99)),
            read.action().object_type_id(),
            Uuid::from_u128(99),
            read.action().clone(),
            read.fields().to_vec(),
        )
        .unwrap(),
        CompanyPolicyRequest::new(
            a.company(),
            read.action().object_type_id(),
            Uuid::from_u128(99),
            read.action().clone(),
            read.fields().to_vec(),
        )
        .unwrap(),
        CompanyPolicyRequest::new(
            a.company(),
            read.action().object_type_id(),
            *a.company().as_uuid(),
            read.action().clone(),
            vec![
                PropertyRef::new(
                    a.company(),
                    read.action().object_type_id(),
                    Uuid::from_u128(99),
                    1,
                )
                .unwrap(),
            ],
        )
        .unwrap(),
    ] {
        policy.sdk_calls.store(0, Ordering::SeqCst);
        assert!(
            matches!(policy.decide(&a, &request), Ok(CompanyPolicyDecision::Deny)),
            "Cedar scope/field mismatch allowed"
        );
        assert_eq!(
            policy.sdk_calls.load(Ordering::SeqCst),
            1,
            "denial came only from another guard; SDK oracle not exercised"
        );
    }
    // Changing only the request's manifest must not remap labels or field IDs.
    let mut digest = *read.action().manifest_digest();
    digest[0] ^= 1;
    let action = ActionRef::new(
        a.company(),
        read.action().action_type_id(),
        read.action().object_type_id(),
        1,
        digest,
    )
    .unwrap();
    let request = CompanyPolicyRequest::new(
        a.company(),
        read.action().object_type_id(),
        *a.company().as_uuid(),
        action,
        read.fields().to_vec(),
    )
    .unwrap();
    assert!(matches!(
        policy.decide(&a, &request),
        Ok(CompanyPolicyDecision::Deny)
    ));
}

#[test]
fn actual_cedar_panic_does_not_turn_into_allow_or_successful_projection() {
    let a = authority();
    let mut policy = CompanyPolicy::new().unwrap();
    policy.panic_at_authorizer = true;
    assert!(matches!(
        policy.decide(&a, &request_for(&a, 1)),
        Err(CompanyPolicyError::EvaluatorUnavailable)
    ));
}

// Semantic successor: checks original serialized material before any lossy Value
// conversion by the product parser. Value below authors controls, never authority.
#[test]
fn checked_initial_projection_requires_closed_maps_canonical_scalars_and_bounded_json() {
    let (_, _, _, row) = input_row();
    let original = row.registered_clauses;
    let mut cases: Vec<(String, String)> = Vec::new();
    for pointer in ["/0/action", "/0/resource", "/0/fields/0"] {
        let value: Value = serde_json::from_str(&original).unwrap();
        let object = value.pointer(pointer).unwrap().as_object().unwrap();
        // Every required nested member, not just a selected representative.
        for key in object.keys() {
            let mut changed = value.clone();
            changed
                .pointer_mut(pointer)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .remove(key);
            cases.push((
                format!("missing {pointer}/{key}"),
                serde_json::to_string(&changed).unwrap(),
            ));
        }
        let mut changed = value.clone();
        changed
            .pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("authorized".into(), json!(true));
        cases.push((
            format!("unknown key {pointer}"),
            serde_json::to_string(&changed).unwrap(),
        ));
        // Same-value duplicates remain invalid. Construct raw bytes AFTER normal
        // fixture serialization so no Value parser can collapse the control.
        let encoded = serde_json::to_string(value.pointer(pointer).unwrap()).unwrap();
        let (key, field) = object.iter().next().unwrap();
        let duplicate = format!(
            "{{{}:{},{}",
            serde_json::to_string(key).unwrap(),
            serde_json::to_string(field).unwrap(),
            &encoded[1..]
        );
        let changed = original.replacen(&encoded, &duplicate, 1);
        assert_ne!(
            changed, original,
            "nested duplicate control did not replace input"
        );
        cases.push((format!("duplicate key {pointer}/{key}"), changed));
    }
    // Resource has only two fields. Both declaration orders are tested, so a
    // permissive serde struct sequence cannot escape via field-order mismatch.
    for reversed in [false, true] {
        let mut changed: Value = serde_json::from_str(&original).unwrap();
        let resource = &changed[0]["resource"];
        let mut values = vec![resource["kind"].clone(), resource["org_id"].clone()];
        if reversed {
            values.reverse();
        }
        changed[0]["resource"] = Value::Array(values);
        cases.push((
            format!("resource array reversed={reversed}"),
            serde_json::to_string(&changed).unwrap(),
        ));
    }
    for (name, old, new) in [
        (
            "registration leading zero",
            "\"registration_revision\":\"1\"",
            "\"registration_revision\":\"01\"",
        ),
        (
            "schema leading zero",
            "\"schema_revision\":\"1\"",
            "\"schema_revision\":\"01\"",
        ),
        (
            "registration number",
            "\"registration_revision\":\"1\"",
            "\"registration_revision\":1",
        ),
        (
            "schema number",
            "\"schema_revision\":\"1\"",
            "\"schema_revision\":1",
        ),
        (
            "timestamp offset alias",
            "2026-09-20T00:00:00.000000Z",
            "2026-09-20T00:00:00.000000+00:00",
        ),
        (
            "timestamp fraction alias",
            "2026-09-20T00:00:00.000000Z",
            "2026-09-20T00:00:00Z",
        ),
        (
            "uppercase UUID alias",
            "00000000-0000-0000-0000-00000000000b",
            "00000000-0000-0000-0000-00000000000B",
        ),
        (
            "unhyphenated UUID alias",
            "00000000-0000-0000-0000-00000000000b",
            "0000000000000000000000000000000b",
        ),
    ] {
        // Replace every occurrence, preserving semantic correlation. Denial
        // must detect spelling/type rather than unequal alias/delegation pairs.
        let changed = original.replace(old, new);
        assert_ne!(changed, original, "canonical control did not replace input");
        cases.push((name.into(), changed));
    }
    cases.push(("trailing JSON".into(), format!("{original} null")));
    assert!(original.len() < 32768);
    cases.push((
        "32769-byte valid JSON".into(),
        format!("{}{}", original, " ".repeat(32769 - original.len())),
    ));
    for (name, raw) in cases {
        let (account, company, observed, mut row) = input_row();
        row.registered_clauses = raw;
        assert!(
            matches!(
                CurrentCompanyAuthority::from_initial_projection(account, company, observed, row),
                Err(CompanyPolicyError::MaterialUnavailable)
            ),
            "accepted {name}"
        );
    }
    // Valid control at the admitted byte boundary. JSON trailing whitespace is
    // permitted; bytes must be measured before trimming, so32769above differs.
    let (account, company, observed, mut row) = input_row();
    row.registered_clauses = format!("{}{}", original, " ".repeat(32768 - original.len()));
    assert_eq!(row.registered_clauses.len(), 32768);
    assert!(
        CurrentCompanyAuthority::from_initial_projection(account, company, observed, row).is_ok()
    );
}
