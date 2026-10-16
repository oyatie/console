//! Real Cedar unit evaluation over explicit pure source vectors; not SQL custody proof.
use super::*;
use console_identity_application::company_policy::people_business::DirectoryActionV1;
use console_identity_application::company_policy::{
    AccountId, CurrentPeopleDirectoryAuthority, NativePeopleDirectoryProjectionRow,
    NativePeopleDirectoryRequestV1, NativePeopleDirectoryResource, NativePolicySourceBinding,
};
use serde_json::Value;
use std::sync::atomic::Ordering;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;
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
fn request(
    action: DirectoryActionV1,
    resource: NativePeopleDirectoryResource,
) -> NativePeopleDirectoryRequestV1 {
    NativePeopleDirectoryRequestV1::new(
        OrgId::from_uuid(id(11)),
        AccountId::from_uuid(id(1)).unwrap(),
        action,
        resource,
    )
    .unwrap()
}
#[test]
fn people_business_bundle_compiles_strictly_and_matches_frozen_bytes() {
    let schema = include_str!("company_policy/native-people-directory-business-v1.cedarschema");
    let policy = include_str!("company_policy/native-people-directory-business-v1.cedar");
    assert_eq!(
        hex::encode(Sha256::digest(schema)),
        "071d02880c01c1c57cbe38c91a600d3d2ffa73681fccac8dcdc1eb75ae407390"
    );
    assert_eq!(
        hex::encode(Sha256::digest(policy)),
        "85361d5590088c9e39cb5e17c84685a8b51752d6bb6647f6d51952de280ab8db"
    );
    let b = compile_bundle_from_sources(
        OrgId::from_uuid(id(11)),
        3,
        "native-people-directory-business-v1",
        schema,
        policy,
    )
    .unwrap();
    assert_eq!(b.key.cedar_sdk_version, CEDAR_SDK_VERSION);
    assert_eq!(b.key.cedar_language_version, CEDAR_LANGUAGE_VERSION);
    assert!(
        compile_bundle_from_sources(
            OrgId::from_uuid(id(11)),
            3,
            "native-people-directory-business-v1",
            schema,
            "permit INVALID;"
        )
        .is_err()
    );
}
#[test]
fn people_business_real_sdk_accepts_read_collection_entry_and_create_request() {
    let p = CompanyPolicy::new().unwrap();
    for (action, resource) in [
        (
            DirectoryActionV1::Read,
            NativePeopleDirectoryResource::Collection,
        ),
        (
            DirectoryActionV1::Read,
            NativePeopleDirectoryResource::Entry(id(81)),
        ),
        (
            DirectoryActionV1::Create,
            NativePeopleDirectoryResource::Request(id(82)),
        ),
    ] {
        let a = CurrentPeopleDirectoryAuthority::from_retained_projection(&binding(), row(action))
            .unwrap();
        p.sdk_calls.store(0, Ordering::SeqCst);
        assert_eq!(
            p.decide_native_people_directory(&a, &request(action, resource)),
            Ok(CompanyPolicyDecision::Allow)
        );
        assert_eq!(p.sdk_calls.load(Ordering::SeqCst), 1);
    }
}
#[test]
fn people_business_wrong_actor_company_and_other_business_action_deny() {
    let p = CompanyPolicy::new().unwrap();
    for grant in [DirectoryActionV1::Read, DirectoryActionV1::Create] {
        let a = CurrentPeopleDirectoryAuthority::from_retained_projection(&binding(), row(grant))
            .unwrap();
        for change in 0..3 {
            let action = if change == 2 {
                if grant == DirectoryActionV1::Read {
                    DirectoryActionV1::Create
                } else {
                    DirectoryActionV1::Read
                }
            } else {
                grant
            };
            let resource = if action == DirectoryActionV1::Read {
                NativePeopleDirectoryResource::Collection
            } else {
                NativePeopleDirectoryResource::Request(id(81))
            };
            let req = NativePeopleDirectoryRequestV1::new(
                OrgId::from_uuid(id(if change == 0 { 99 } else { 11 })),
                AccountId::from_uuid(id(if change == 1 { 99 } else { 1 })).unwrap(),
                action,
                resource,
            )
            .unwrap();
            assert_eq!(
                p.decide_native_people_directory(&a, &req),
                Ok(CompanyPolicyDecision::Deny),
                "change {change}"
            );
        }
    }
}
#[test]
fn people_business_effective_interval_and_fresh_binding_time_control_disclosure() {
    let p = CompanyPolicy::new().unwrap();
    for action in [DirectoryActionV1::Read, DirectoryActionV1::Create] {
        let resource = if action == DirectoryActionV1::Read {
            NativePeopleDirectoryResource::Collection
        } else {
            NativePeopleDirectoryResource::Request(id(81))
        };
        for (now, decision) in [
            (
                at() - Duration::microseconds(1),
                CompanyPolicyDecision::Deny,
            ),
            (at(), CompanyPolicyDecision::Allow),
            (
                at() + Duration::days(1) - Duration::microseconds(1),
                CompanyPolicyDecision::Allow,
            ),
            (at() + Duration::days(1), CompanyPolicyDecision::Deny),
        ] {
            let mut r = row(action);
            r.observed_at = now;
            let mut b = binding();
            b.observed_at = now;
            let a = CurrentPeopleDirectoryAuthority::from_retained_projection(&b, r).unwrap();
            assert_eq!(
                p.decide_native_people_directory(&a, &request(action, resource)),
                Ok(decision)
            );
        }
        let r = row(action);
        let mut b = binding();
        b.observed_at = r.assignment_valid_until;
        let a = CurrentPeopleDirectoryAuthority::from_retained_projection(&b, r).unwrap();
        assert_eq!(
            p.decide_native_people_directory(&a, &request(action, resource)),
            Ok(CompanyPolicyDecision::Deny)
        );
    }
}
#[test]
fn people_business_authorizer_panic_is_unavailable_not_allow_or_process_panic() {
    let mut p = CompanyPolicy::new().unwrap();
    p.panic_at_authorizer = true;
    let a = CurrentPeopleDirectoryAuthority::from_retained_projection(
        &binding(),
        row(DirectoryActionV1::Read),
    )
    .unwrap();
    assert_eq!(
        p.decide_native_people_directory(
            &a,
            &request(
                DirectoryActionV1::Read,
                NativePeopleDirectoryResource::Collection
            )
        ),
        Err(CompanyPolicyError::EvaluatorUnavailable)
    );
}

#[test]
fn people_navigation_uses_real_cedar_and_preserves_actor_company_action_and_time() {
    let p = CompanyPolicy::new().unwrap();
    for grant in [DirectoryActionV1::Read, DirectoryActionV1::Create] {
        let a = CurrentPeopleDirectoryAuthority::from_retained_projection(&binding(), row(grant))
            .unwrap();
        for change in 0..4 {
            let action = if change == 3 {
                if grant == DirectoryActionV1::Read {
                    DirectoryActionV1::Create
                } else {
                    DirectoryActionV1::Read
                }
            } else {
                grant
            };
            let req = NativePeopleDirectoryRequestV1::new(
                OrgId::from_uuid(id(if change == 1 { 99 } else { 11 })),
                AccountId::from_uuid(id(if change == 2 { 99 } else { 1 })).unwrap(),
                action,
                NativePeopleDirectoryResource::Navigation,
            )
            .unwrap();
            p.sdk_calls.store(0, Ordering::SeqCst);
            assert_eq!(
                p.decide_native_people_directory(&a, &req),
                Ok(if change == 0 {
                    CompanyPolicyDecision::Allow
                } else {
                    CompanyPolicyDecision::Deny
                })
            );
            assert_eq!(p.sdk_calls.load(Ordering::SeqCst), 1);
        }
        let mut b = binding();
        b.observed_at = row(grant).assignment_valid_until;
        let a = CurrentPeopleDirectoryAuthority::from_retained_projection(&b, row(grant)).unwrap();
        assert_eq!(
            p.decide_native_people_directory(
                &a,
                &request(grant, NativePeopleDirectoryResource::Navigation)
            ),
            Ok(CompanyPolicyDecision::Deny)
        );
    }
}

#[test]
fn people_business_navigation_v2_is_pinned_and_distinct_from_preserved_v1() {
    let schema = include_str!("company_policy/native-people-directory-business-v1.cedarschema");
    let policy = include_str!("company_policy/native-people-directory-business-v2.cedar");
    let prior = include_str!("company_policy/native-people-directory-business-v1.cedar");
    assert_eq!(
        hex::encode(Sha256::digest(schema)),
        "071d02880c01c1c57cbe38c91a600d3d2ffa73681fccac8dcdc1eb75ae407390"
    );
    assert_eq!(
        hex::encode(Sha256::digest(policy)),
        "79470085d87325678949f3a18b9d68f4f62fa51d4f97de60cd5684898a92900c"
    );
    let current = compile_bundle_from_sources(
        OrgId::from_uuid(id(11)),
        3,
        "native-people-directory-business-v2",
        schema,
        policy,
    )
    .unwrap();
    let historical = compile_bundle_from_sources(
        OrgId::from_uuid(id(11)),
        3,
        "native-people-directory-business-v1",
        schema,
        prior,
    )
    .unwrap();
    assert_eq!(
        current.key.schema_version,
        "native-people-directory-business-v2"
    );
    assert_eq!(
        historical.key.schema_version,
        "native-people-directory-business-v1"
    );
    assert_ne!(current.key.bundle_digest, historical.key.bundle_digest);
    assert_eq!(current.key.cedar_sdk_version, CEDAR_SDK_VERSION);
    assert_eq!(current.key.cedar_language_version, CEDAR_LANGUAGE_VERSION);
    assert!(
        compile_bundle_from_sources(
            OrgId::from_uuid(id(11)),
            3,
            "native-people-directory-business-v2",
            schema,
            "permit INVALID;"
        )
        .is_err()
    );
}
