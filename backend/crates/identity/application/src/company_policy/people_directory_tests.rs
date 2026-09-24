//! Explicit pure decoder/source vectors, never SQL custody or authentication proof.
use super::*;
use serde_json::{Value, json};
fn id(n: u128) -> Uuid {
    Uuid::from_u128(n)
}
fn at() -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(1_790_000_000).unwrap()
}
fn binding() -> NativePolicySourceBinding {
    NativePolicySourceBinding {
        account: AccountId::from_uuid(id(1)).unwrap(),
        session_id: id(2),
        account_security_generation: 1,
        source_xid: 9001,
        source_backend_pid: 123,
        observed_at: at(),
    }
}
fn row(kind: DirectoryActionV1) -> NativePeopleDirectoryProjectionRow {
    // Fixture keys are independent literals, not implementation property_keys().
    let keys: &[&str] = match kind {
        DirectoryActionV1::Read => &[
            "person.employee_id",
            "person.person_id",
            "person.legal_name",
            "person.employee_number",
            "person.person_version",
            "person.directory_registered_at",
        ],
        DirectoryActionV1::Create => &["person.legal_name", "person.employee_number"],
    };
    let action = json!({"org_id":id(11).to_string(),"object_type_id":id(20).to_string(),"action_type_id":id(if kind==DirectoryActionV1::Read {21}else{22}).to_string(),"registration_revision":"1","manifest_digest":"591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e"});
    let mut fields = Vec::new();
    let mut named = Vec::new();
    for (i, key) in keys.iter().enumerate() {
        let p = json!({"org_id":id(11).to_string(),"object_type_id":id(20).to_string(),"property_id":id(100+i as u128).to_string(),"schema_revision":"1"});
        fields.push(p.clone());
        let mut n = p;
        n["key"] = json!(key);
        named.push(n);
    }
    let clause = json!([{"kind":"COMPANY_CAPABILITY_CLAUSE_V1","action":action,"resource":{"kind":"COMPANY","org_id":id(11).to_string()},"fields":fields,"valid_from":"2026-01-01T00:00:00.000000Z","valid_until":null,"delegable":false}]);
    NativePeopleDirectoryProjectionRow {
        account_id: id(1),
        session_id: id(2),
        account_security_generation: 1,
        org_id: id(11),
        company_epoch: 3,
        current_policy_receipt_id: id(16),
        assignment_id: id(41),
        assignment_revision: 7,
        role_id: id(42),
        role_revision: 1,
        registered_clauses: clause.to_string(),
        assignment_valid_from: at(),
        assignment_valid_until: at() + Duration::days(1),
        action_reference: action.to_string(),
        named_properties: Value::Array(named).to_string(),
        action: kind,
        observed_at: at(),
        source_xid: "9001".into(),
        source_backend_pid: 123,
    }
}
fn checked(
    r: NativePeopleDirectoryProjectionRow,
) -> Result<CurrentPeopleDirectoryAuthority, CompanyPolicyError> {
    CurrentPeopleDirectoryAuthority::from_retained_projection(&binding(), r)
}
#[test]
fn native_people_exact_named_catalog_matches_both_actions_and_unordered_input() {
    for (kind, count) in [(DirectoryActionV1::Read, 6), (DirectoryActionV1::Create, 2)] {
        let a = checked(row(kind)).unwrap();
        assert_eq!(a.named_properties().len(), count);
        assert!(a.property("person.legal_name").is_some());
        assert!(a.property("salary.amount").is_none());
        assert_eq!(a.source().action, kind);
    }
}
#[test]
fn native_people_checks_retained_coordinates_company_revisions_and_interval() {
    for change in 0..16 {
        let mut r = row(DirectoryActionV1::Read);
        match change {
            0 => r.account_id = id(99),
            1 => r.session_id = id(99),
            2 => r.account_security_generation = 2,
            3 => r.source_xid = "09001".into(),
            4 => r.source_backend_pid = 124,
            5 => r.observed_at += Duration::microseconds(1),
            6 => r.observed_at += Duration::nanoseconds(1),
            7 => r.org_id = *OrgId::platform().as_uuid(),
            8 => r.current_policy_receipt_id = Uuid::nil(),
            9 => r.assignment_id = Uuid::nil(),
            10 => r.assignment_revision = 0,
            11 => r.role_revision = 2,
            12 => r.company_epoch = 1,
            13 => r.assignment_valid_until = r.assignment_valid_from,
            14 => r.assignment_valid_until += Duration::days(30),
            15 => r.assignment_valid_from += Duration::nanoseconds(1),
            _ => unreachable!(),
        };
        assert!(checked(r).is_err(), "change {change}");
    }
}
#[test]
fn native_people_named_property_checks_detect_same_count_swaps_and_aliases() {
    for change in 0..9 {
        let mut r = row(DirectoryActionV1::Read);
        let mut n: Value = serde_json::from_str(&r.named_properties).unwrap();
        match change {
            0 => n[0]["key"] = json!("salary.amount"),
            1 => n[0]["key"] = n[1]["key"].clone(),
            2 => n[0]["property_id"] = n[1]["property_id"].clone(),
            3 => n[0]["property_id"] = json!(id(999).to_string()),
            4 => n[0]["org_id"] = json!(id(99).to_string()),
            5 => n[0]["object_type_id"] = json!(id(99).to_string()),
            6 => n[0]["schema_revision"] = json!("2"),
            7 => n[0]["extra"] = json!(true),
            8 => {
                n.as_array_mut().unwrap().pop();
            }
            _ => unreachable!(),
        };
        r.named_properties = n.to_string();
        assert!(checked(r).is_err(), "change {change}");
    }
}
#[test]
fn native_people_strict_clause_and_action_are_not_length_only_checks() {
    for change in 0..13 {
        let mut r = row(DirectoryActionV1::Create);
        let mut c: Value = serde_json::from_str(&r.registered_clauses).unwrap();
        match change {
            0 => c[0]["fields"][0]["property_id"] = json!(id(999).to_string()),
            1 => c[0]["action"]["action_type_id"] = json!(id(999).to_string()),
            2 => c[0]["action"]["manifest_digest"] = json!("00".repeat(32)),
            3 => c[0]["delegable"] = json!(true),
            4 => c[0]["valid_until"] = json!("2027-01-01T00:00:00Z"),
            5 => c[0]["resource"]["org_id"] = json!(id(99).to_string()),
            6 => c[0]["extra"] = json!(true),
            7 => c[0]["action"]["registration_revision"] = json!("01"),
            8 => c[0]["fields"][0]["schema_revision"] = json!(1),
            9 => {
                c[0].as_object_mut().unwrap().remove("valid_until");
            }
            10 => {
                let duplicate = c[0].clone();
                c.as_array_mut().unwrap().push(duplicate);
            }
            11 => c[0]["kind"] = json!("OTHER"),
            12 => r.action = DirectoryActionV1::Read,
            _ => unreachable!(),
        };
        r.registered_clauses = c.to_string();
        assert!(checked(r).is_err(), "change {change}");
    }
}
#[test]
fn native_people_duplicate_json_members_and_bounded_raw_sources_refuse() {
    for target in 0..3 {
        let mut r = row(DirectoryActionV1::Read);
        let raw = match target {
            0 => &mut r.registered_clauses,
            1 => &mut r.action_reference,
            _ => &mut r.named_properties,
        };
        *raw = raw.replacen(
            "\"org_id\":",
            &format!("\"org_id\":\"{}\",\"org_id\":", id(11)),
            1,
        );
        assert!(checked(r).is_err());
    }
    for target in 0..3 {
        let mut r = row(DirectoryActionV1::Read);
        let raw = match target {
            0 => &mut r.registered_clauses,
            1 => &mut r.action_reference,
            _ => &mut r.named_properties,
        };
        raw.push_str(&" ".repeat(32769));
        assert!(checked(r).is_err());
    }
}
#[test]
fn native_people_request_modes_and_non_nil_selectors_are_closed() {
    let actor = AccountId::from_uuid(id(1)).unwrap();
    let c = OrgId::from_uuid(id(11));
    for (a, r) in [
        (
            DirectoryActionV1::Read,
            NativePeopleDirectoryResource::Collection,
        ),
        (
            DirectoryActionV1::Read,
            NativePeopleDirectoryResource::Entry(id(8)),
        ),
        (
            DirectoryActionV1::Create,
            NativePeopleDirectoryResource::Request(id(9)),
        ),
    ] {
        assert!(NativePeopleDirectoryRequestV1::new(c, actor, a, r).is_ok());
    }
    for (a, r) in [
        (
            DirectoryActionV1::Read,
            NativePeopleDirectoryResource::Request(id(9)),
        ),
        (
            DirectoryActionV1::Create,
            NativePeopleDirectoryResource::Collection,
        ),
        (
            DirectoryActionV1::Create,
            NativePeopleDirectoryResource::Entry(id(8)),
        ),
        (
            DirectoryActionV1::Read,
            NativePeopleDirectoryResource::Entry(Uuid::nil()),
        ),
        (
            DirectoryActionV1::Create,
            NativePeopleDirectoryResource::Request(Uuid::nil()),
        ),
    ] {
        assert!(NativePeopleDirectoryRequestV1::new(c, actor, a, r).is_err());
    }
    assert!(
        NativePeopleDirectoryRequestV1::new(
            OrgId::platform(),
            actor,
            DirectoryActionV1::Read,
            NativePeopleDirectoryResource::Collection
        )
        .is_err()
    );
}

#[test]
fn native_people_navigation_is_a_distinct_non_object_selector_for_both_actions() {
    for action in [DirectoryActionV1::Read, DirectoryActionV1::Create] {
        let request = NativePeopleDirectoryRequestV1::new(
            OrgId::from_uuid(id(11)),
            AccountId::from_uuid(id(1)).unwrap(),
            action,
            NativePeopleDirectoryResource::Navigation,
        )
        .unwrap();
        assert_eq!(request.action(), action);
        assert_eq!(
            request.resource(),
            NativePeopleDirectoryResource::Navigation
        );
    }
}
