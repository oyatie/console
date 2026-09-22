// Pure source vectors only; never authenticated SQL/current-custody evidence.
// Mount as a cfg(test) child of CompanyPolicy's actual module.
#[path = "native_workflow_usecases_tests.rs"]
mod workflow;
use super::*;
use console_identity_application::company_policy::business::NativeBusinessOperationV1;
use console_identity_application::company_policy::{
    AccountId, ActionRef, CurrentNativeBootstrapAuthority, CurrentPayrollReadAuthority,
    NativeBootstrapProjectionRow, NativeBootstrapRequestV1, NativePayrollReadProjectionRow,
    NativePolicySourceBinding, PropertyRef,
};
use serde_json::Value;
use std::sync::atomic::Ordering;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

const BOOT_SCHEMA: &str = include_str!("company_policy/native-business-bootstrap-v1.cedarschema");
const BOOT_POLICY: &str = include_str!("company_policy/native-business-bootstrap-v1.cedar");
const READ_SCHEMA: &str =
    include_str!("company_policy/native-payroll-collection-read-v1.cedarschema");
const READ_POLICY: &str = include_str!("company_policy/native-payroll-collection-read-v1.cedar");
const DIGEST: &str = "07781514029d5f8f7e96221d6504387324c8c0f2513ded2b214d0bd683ce3ddd";
const KEYS: [&str; 18] = [
    "pay_run.id",
    "pay_run.period_start",
    "pay_run.period_end",
    "pay_run.source_label",
    "pay_run.status",
    "pay_run.calculation_enabled",
    "pay_run.created_by",
    "pay_run.approved_by",
    "pay_run.approved_at",
    "pay_run.close_receipt",
    "pay_run.submitted_by",
    "pay_run.submitted_at",
    "pay_run.decided_by",
    "pay_run.decided_at",
    "pay_run.decision_reason",
    "pay_run.approval_ref",
    "pay_run.created_at",
    "pay_run.updated_at",
];
fn id(n: u128) -> Uuid {
    Uuid::from_u128(n)
}
fn digest() -> [u8; 32] {
    hex::decode(DIGEST).unwrap().try_into().unwrap()
}
fn at() -> OffsetDateTime {
    time::macros::datetime!(2026-09-21 00:00:00 UTC)
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
fn bootstrap_row() -> NativeBootstrapProjectionRow {
    NativeBootstrapProjectionRow {
        actor_account_id: id(1),
        session_id: id(2),
        account_security_generation: 1,
        designation_system_identifier: "7610000000000000001".into(),
        designation_database_name: "console_test".into(),
        designation_database_oid: 16384,
        designation_revision: 3,
        designation_receipt_id: id(4),
        org_id: id(11),
        current_group_id: id(12),
        group_revision: 2,
        group_incarnation: id(13),
        membership_id: id(14),
        membership_revision: 2,
        membership_incarnation: id(15),
        company_epoch: 3,
        current_policy_receipt_id: Some(id(16)),
        // Deliberate legitimate-predecessor vector: no birth=current predicate.
        origin_account_id: id(31),
        origin_command_id: id(32),
        origin_receipt_id: id(33),
        administrative_account_id: id(34),
        company_actor_admission_receipt_id: id(33),
        birth_assignment_id: id(35),
        birth_role_id: id(36),
        observed_at: at(),
        source_xid: "9001".into(),
        source_backend_pid: 123,
    }
}
fn bootstrap() -> CurrentNativeBootstrapAuthority {
    CurrentNativeBootstrapAuthority::from_retained_projection(&binding(), bootstrap_row()).unwrap()
}
fn bootstrap_request(operation: NativeBusinessOperationV1) -> NativeBootstrapRequestV1 {
    NativeBootstrapRequestV1::new(
        OrgId::from_uuid(id(11)),
        id(12),
        AccountId::from_uuid(id(34)).unwrap(),
        operation,
        digest(),
    )
    .unwrap()
}
fn payroll_row() -> NativePayrollReadProjectionRow {
    let org = OrgId::from_uuid(id(11));
    NativePayrollReadProjectionRow {
        account_id: id(1),
        session_id: id(2),
        account_security_generation: 1,
        org_id: id(11),
        company_epoch: 3,
        current_policy_receipt_id: id(16),
        assignment_id: id(41),
        assignment_revision: 7,
        assignment_state: "ACTIVE".into(),
        role_id: id(42),
        role_revision: 1,
        role_state: "ACTIVE".into(),
        valid_from: at(),
        valid_until: at() + Duration::days(1),
        delegable: false,
        action: ActionRef::new(org, id(21), id(20), 1, digest()).unwrap(),
        properties: (0..18)
            .map(|i| PropertyRef::new(org, id(20), id(100 + i), 1).unwrap())
            .collect(),
        observed_at: at(),
        source_xid: "9001".into(),
        source_backend_pid: 123,
    }
}
fn payroll() -> CurrentPayrollReadAuthority {
    CurrentPayrollReadAuthority::from_retained_projection(&binding(), payroll_row()).unwrap()
}

#[test]
fn frozen_business_resource_bytes_and_strict_compilation() {
    for (bytes, expected) in [
        (
            BOOT_SCHEMA,
            "58d413453cca9e8f67206a73c95dcb33fc020a794a3615f96da2107bac7d40b7",
        ),
        (
            BOOT_POLICY,
            "25ec7cbd092432c38ed25944edd609e67d2da529e21ac47b5dd9146fd451db82",
        ),
        (
            READ_SCHEMA,
            "5eba76196c0203980310cad04bc64ffa0cb65fac3bc18b9a1416fb84c213ac34",
        ),
        (
            READ_POLICY,
            "f861ad6a2b9e41471a9c17dc880ef925bcab32c7c0e074b8bd6aa0686e83cfa8",
        ),
    ] {
        assert_eq!(hex::encode(Sha256::digest(bytes)), expected);
    }
    sdk_identity().unwrap();
    for (schema, policy, name) in [
        (BOOT_SCHEMA, BOOT_POLICY, "native-business-bootstrap-v1"),
        (
            READ_SCHEMA,
            READ_POLICY,
            "native-payroll-collection-read-v1",
        ),
    ] {
        let bundle =
            compile_bundle_from_sources(OrgId::from_uuid(id(11)), 3, name, schema, policy).unwrap();
        assert_eq!(bundle.key.org_id, OrgId::from_uuid(id(11)));
        assert_eq!(bundle.key.policy_version, 3);
        assert_eq!(bundle.key.schema_version, name);
        assert_eq!(bundle.key.cedar_language_version, CEDAR_LANGUAGE_VERSION);
        assert_eq!(bundle.key.cedar_sdk_version, CEDAR_SDK_VERSION);
        let mut hash = Sha256::new();
        hash.update(schema);
        hash.update(policy);
        assert_eq!(bundle.key.bundle_digest, hex::encode(hash.finalize()));
        assert!(
            compile_bundle_from_sources(
                OrgId::from_uuid(id(11)),
                3,
                name,
                schema,
                "permit INVALID;"
            )
            .is_err()
        );
    }
    CompanyPolicy::new().unwrap();
}

#[test]
fn production_native_bootstrap_uses_sdk_for_operator_and_successor_without_birth_equality() {
    let policy = CompanyPolicy::new().unwrap();
    for same_birth_actor in [true, false] {
        let mut row = bootstrap_row();
        if same_birth_actor {
            row.origin_account_id = id(1);
        }
        let authority =
            CurrentNativeBootstrapAuthority::from_retained_projection(&binding(), row).unwrap();
        for operation in [
            NativeBusinessOperationV1::Install,
            NativeBusinessOperationV1::Grant,
            NativeBusinessOperationV1::Revoke,
        ] {
            policy.sdk_calls.store(0, Ordering::SeqCst);
            assert_eq!(
                policy.decide_native_bootstrap(&authority, &bootstrap_request(operation)),
                Ok(CompanyPolicyDecision::Allow)
            );
            assert_eq!(
                policy.sdk_calls.load(Ordering::SeqCst),
                1,
                "bootstrap bypassed actual Cedar authorizer"
            );
        }
    }
}

#[test]
fn production_bootstrap_denies_wrong_company_group_recipient_and_manifest() {
    let policy = CompanyPolicy::new().unwrap();
    let authority = bootstrap();
    for changed in 0..4 {
        let request = NativeBootstrapRequestV1::new(
            OrgId::from_uuid(id(if changed == 0 { 91 } else { 11 })),
            id(if changed == 1 { 92 } else { 12 }),
            AccountId::from_uuid(id(if changed == 2 { 93 } else { 34 })).unwrap(),
            NativeBusinessOperationV1::Grant,
            if changed == 3 { [0; 32] } else { digest() },
        )
        .unwrap();
        policy.sdk_calls.store(0, Ordering::SeqCst);
        assert_eq!(
            policy.decide_native_bootstrap(&authority, &request),
            Ok(CompanyPolicyDecision::Deny)
        );
        assert_eq!(policy.sdk_calls.load(Ordering::SeqCst), 1);
    }
}

#[test]
fn checked_bootstrap_material_rejects_wrong_auth_transaction_and_provenance_correlations() {
    for change in 0..14 {
        let mut row = bootstrap_row();
        match change {
            0 => row.actor_account_id = id(90),
            1 => row.session_id = id(90),
            2 => row.account_security_generation = 2,
            3 => row.source_xid = "9002".into(),
            4 => row.source_backend_pid = 124,
            5 => row.observed_at = at() + Duration::microseconds(1),
            6 => row.company_actor_admission_receipt_id = id(90),
            7 => row.current_policy_receipt_id = None,
            8 => row.designation_revision = 0,
            9 => row.group_incarnation = Uuid::nil(),
            10 => row.membership_incarnation = Uuid::nil(),
            11 => row.source_xid = "09001".into(),
            12 => row.designation_database_oid = -1,
            13 => row.designation_receipt_id = Uuid::nil(),
            _ => unreachable!(),
        }
        assert!(
            matches!(
                CurrentNativeBootstrapAuthority::from_retained_projection(&binding(), row),
                Err(CompanyPolicyError::MaterialUnavailable)
            ),
            "bad source vector {change}"
        );
    }
    let mut birth = bootstrap_row();
    birth.company_epoch = 1;
    birth.current_policy_receipt_id = None;
    assert!(CurrentNativeBootstrapAuthority::from_retained_projection(&binding(), birth).is_ok());
}

#[test]
fn production_payroll_sdk_requires_all18_fields_and_finite_non_delegable_source() {
    let policy = CompanyPolicy::new().unwrap();
    policy.sdk_calls.store(0, Ordering::SeqCst);
    assert_eq!(
        policy.decide_native_payroll_collection(&payroll()),
        Ok(CompanyPolicyDecision::Allow)
    );
    assert_eq!(policy.sdk_calls.load(Ordering::SeqCst), 1);
    for missing in 0..18 {
        let mut row = payroll_row();
        row.properties.remove(missing);
        assert!(
            matches!(
                CurrentPayrollReadAuthority::from_retained_projection(&binding(), row),
                Err(CompanyPolicyError::MaterialUnavailable)
            ),
            "missing {} admitted",
            KEYS[missing]
        );
    }
    for changed in 0..9 {
        let mut row = payroll_row();
        match changed {
            0 => row.delegable = true,
            1 => row.role_revision = 2,
            2 => row.assignment_state = "REVOKED".into(),
            3 => row.role_state = "INACTIVE".into(),
            4 => {
                row.properties[0] =
                    PropertyRef::new(OrgId::from_uuid(id(90)), id(20), id(100), 1).unwrap()
            }
            5 => row.properties[1] = row.properties[0].clone(),
            6 => {
                row.action =
                    ActionRef::new(OrgId::from_uuid(id(11)), id(21), id(20), 1, [0; 32]).unwrap()
            }
            7 => row.valid_until = at() + Duration::days(31),
            8 => row.valid_until += Duration::nanoseconds(1),
            _ => unreachable!(),
        }
        assert!(
            matches!(
                CurrentPayrollReadAuthority::from_retained_projection(&binding(), row),
                Err(CompanyPolicyError::MaterialUnavailable)
            ),
            "invalid read source {changed}"
        );
    }
}

#[test]
fn production_payroll_interval_is_inclusive_start_exclusive_end() {
    let policy = CompanyPolicy::new().unwrap();
    for (now, expected) in [
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
        let mut row = payroll_row();
        row.observed_at = now;
        let mut source = binding();
        source.observed_at = now;
        let authority =
            CurrentPayrollReadAuthority::from_retained_projection(&source, row).unwrap();
        assert_eq!(
            policy.decide_native_payroll_collection(&authority),
            Ok(expected)
        );
    }
}

#[test]
fn payroll_uses_latest_independent_binding_clock_after_acquisition() {
    let policy = CompanyPolicy::new().unwrap();
    let row = payroll_row();
    let mut current = binding();
    current.observed_at = row.valid_until;
    let authority = CurrentPayrollReadAuthority::from_retained_projection(&current, row).unwrap();
    assert_eq!(authority.observed_at(), current.observed_at);
    assert_eq!(
        policy.decide_native_payroll_collection(&authority),
        Ok(CompanyPolicyDecision::Deny)
    );
}

#[test]
fn installation_permission_does_not_supply_a_payroll_assignment() {
    let policy = CompanyPolicy::new().unwrap();
    assert_eq!(
        policy.decide_native_bootstrap(
            &bootstrap(),
            &bootstrap_request(NativeBusinessOperationV1::Install)
        ),
        Ok(CompanyPolicyDecision::Allow)
    );
    let mut no_grant = payroll_row();
    no_grant.assignment_state = "ABSENT".into();
    assert!(matches!(
        CurrentPayrollReadAuthority::from_retained_projection(&binding(), no_grant),
        Err(CompanyPolicyError::MaterialUnavailable)
    ));
}

#[test]
fn native_authorizer_failure_never_releases_allow_or_panics() {
    let mut policy = CompanyPolicy::new().unwrap();
    policy.panic_at_authorizer = true;
    assert_eq!(
        policy.decide_native_bootstrap(
            &bootstrap(),
            &bootstrap_request(NativeBusinessOperationV1::Grant)
        ),
        Err(CompanyPolicyError::EvaluatorUnavailable)
    );
    assert_eq!(
        policy.decide_native_payroll_collection(&payroll()),
        Err(CompanyPolicyError::EvaluatorUnavailable)
    );
}

// Independent literal-resource evaluation covers predicates that SQL27's
// already-validated projection intentionally cannot carry as arbitrary flags.
fn literal_decision(
    schema: &str,
    policy: &str,
    action: &str,
    resource_type: &str,
    entities: Value,
    context: Value,
) -> Result<Decision, CompanyPolicyError> {
    fn fail<T>(_: T) -> CompanyPolicyError {
        CompanyPolicyError::EvaluatorUnavailable
    }
    let bundle = compile_bundle_from_sources(
        OrgId::from_uuid(id(11)),
        3,
        "literal-resource-vector",
        schema,
        policy,
    )
    .map_err(fail)?;
    let principal = EntityUid::from_str(&format!("Account::\"{}\"", id(1))).map_err(fail)?;
    let action = EntityUid::from_str(&format!("Action::\"{action}\"")).map_err(fail)?;
    let resource = EntityUid::from_str(&format!("{resource_type}::\"resource\"")).map_err(fail)?;
    let entities = Entities::from_json_value(entities, Some(&bundle.schema)).map_err(fail)?;
    let context =
        Context::from_json_value(context, Some((&bundle.schema, &action))).map_err(fail)?;
    let request =
        Request::new(principal, action, resource, context, Some(&bundle.schema)).map_err(fail)?;
    let response = Authorizer::new().is_authorized(&request, &bundle.policies, &entities);
    if response.diagnostics().errors().next().is_some() {
        return Err(CompanyPolicyError::EvaluatorUnavailable);
    }
    Ok(response.decision())
}
fn boot_entities() -> Value {
    json!([
        {"uid":{"type":"Account","id":id(1).to_string()},"parents":[],"attrs":{
            "account_id":id(1).to_string(),"session_id":id(2).to_string(),"security_generation":"1","state":"ACTIVE",
            "deployment_system_identifier":"7610000000000000001","deployment_database_name":"console_test","deployment_database_oid":"16384",
            "designation_account_id":id(1).to_string(),"designation_revision":"3","designation_receipt_id":id(4).to_string(),"designation_state":"ACTIVE"}},
        {"uid":{"type":"NativeCompany","id":"resource"},"parents":[],"attrs":{
            "org_id":id(11).to_string(),"state":"ACTIVE","deployment_system_identifier":"7610000000000000001",
            "deployment_database_name":"console_test","deployment_database_oid":"16384","current_group_id":id(12).to_string(),
            "group_state":"ACTIVE","group_revision":"2","group_incarnation":id(13).to_string(),"membership_id":id(14).to_string(),
            "membership_state":"ACTIVE","membership_revision":"2","membership_incarnation":id(15).to_string(),"company_epoch":"3",
            "origin_account_id":id(31).to_string(),"origin_command_id":id(32).to_string(),"origin_receipt_id":id(33).to_string(),
            "administrative_account_id":id(34).to_string(),"company_actor_admission_receipt_id":id(33).to_string(),
            "birth_assignment_id":id(35).to_string(),"birth_role_id":id(36).to_string(),"manifest_digest":DIGEST}}
    ])
}
fn boot_context() -> Value {
    json!({"requested_org_id":id(11).to_string(),"requested_group_id":id(12).to_string(),"recipient_account_id":id(34).to_string(),"manifest_digest":DIGEST})
}
#[test]
fn literal_bootstrap_denies_revoked_designation_inactive_resources_and_wrong_deployment() {
    let run = |entities| {
        literal_decision(
            BOOT_SCHEMA,
            BOOT_POLICY,
            "GrantPayrollReadV1",
            "NativeCompany",
            entities,
            boot_context(),
        )
    };
    assert_eq!(run(boot_entities()), Ok(Decision::Allow));
    // A distinct recipient's inactive Account is not the maintenance principal.
    // SQL must still classify its grant eligibility before A / as a B rejection.
    let mut with_inactive_recipient = boot_entities();
    let mut recipient = with_inactive_recipient[0].clone();
    recipient["uid"]["id"] = json!(id(34).to_string());
    recipient["attrs"]["account_id"] = json!(id(34).to_string());
    recipient["attrs"]["session_id"] = json!(id(89).to_string());
    recipient["attrs"]["state"] = json!("INACTIVE");
    recipient["attrs"]["designation_state"] = json!("REVOKED");
    recipient["attrs"]["designation_account_id"] = json!(id(34).to_string());
    with_inactive_recipient
        .as_array_mut()
        .unwrap()
        .push(recipient);
    assert_eq!(run(with_inactive_recipient), Ok(Decision::Allow));
    for (entity, key, value) in [
        (0, "state", "INACTIVE"),
        (0, "designation_state", "REVOKED"),
        (
            0,
            "designation_account_id",
            "00000000-0000-0000-0000-000000000099",
        ),
        (1, "state", "INACTIVE"),
        (1, "group_state", "INACTIVE"),
        (1, "membership_state", "INACTIVE"),
        (1, "deployment_database_name", "other_database"),
        (1, "deployment_database_oid", "99"),
        (1, "deployment_system_identifier", "99"),
    ] {
        let mut entities = boot_entities();
        entities[entity]["attrs"][key] = json!(value);
        assert_eq!(
            run(entities),
            Ok(Decision::Deny),
            "predicate {entity}/{key}"
        );
    }
}
fn literal_fields() -> Vec<String> {
    (0..18)
        .map(|i| format!("{}/{}/{}/1/{DIGEST}", id(11), id(20), id(100 + i)))
        .collect()
}
fn read_entities() -> Value {
    let micros = i64::try_from(at().unix_timestamp_nanos() / 1000).unwrap();
    json!([
        {"uid":{"type":"Account","id":id(1).to_string()},"parents":[],"attrs":{"account_id":id(1).to_string(),"state":"ACTIVE"}},
        {"uid":{"type":"PayrollCollection","id":"resource"},"parents":[],"attrs":{
            "org_id":id(11).to_string(),"object_type_id":id(20).to_string(),"registered_action_id":id(21).to_string(),
            "registration_revision":"1","manifest_digest":DIGEST,"recipient_account_id":id(1).to_string(),
            "assignment_id":id(41).to_string(),"assignment_revision":"7","assignment_state":"ACTIVE",
            "valid_from_us":micros,"valid_until_us":micros+86_400_000_000_i64,"role_id":id(42).to_string(),
            "role_revision":"1","role_state":"ACTIVE","company_epoch":"3","delegable":false,"allowed_fields":literal_fields()}}
    ])
}
fn read_context() -> Value {
    json!({"requested_org_id":id(11).to_string(),"object_type_id":id(20).to_string(),"registered_action_id":id(21).to_string(),
        "requested_fields":literal_fields(),"observed_at_us":i64::try_from(at().unix_timestamp_nanos()/1000).unwrap()})
}
#[test]
fn literal_collection_requires_exact18field_set_and_denies_unsupported_actions() {
    let run = |entities, context| {
        literal_decision(
            READ_SCHEMA,
            READ_POLICY,
            "payroll.collection.read",
            "PayrollCollection",
            entities,
            context,
        )
    };
    assert_eq!(run(read_entities(), read_context()), Ok(Decision::Allow));
    for missing in 0..18 {
        let mut context = read_context();
        context["requested_fields"]
            .as_array_mut()
            .unwrap()
            .remove(missing);
        assert_eq!(
            run(read_entities(), context),
            Ok(Decision::Deny),
            "request missing {}",
            KEYS[missing]
        );
        let mut entities = read_entities();
        entities[1]["attrs"]["allowed_fields"]
            .as_array_mut()
            .unwrap()
            .remove(missing);
        assert_eq!(
            run(entities, read_context()),
            Ok(Decision::Deny),
            "grant missing {}",
            KEYS[missing]
        );
    }
    for action in [
        "payroll.export",
        "payroll.salary.read",
        "payroll.detail.read",
        "GrantPayrollReadV1",
    ] {
        assert!(
            literal_decision(
                READ_SCHEMA,
                READ_POLICY,
                action,
                "PayrollCollection",
                read_entities(),
                read_context()
            )
            .is_err(),
            "closed schema accepted {action}"
        );
    }
    let mut absent = read_entities();
    absent[1]["attrs"]["assignment_state"] = json!("ABSENT");
    assert_eq!(run(absent, read_context()), Ok(Decision::Deny));
    let mut wrong_recipient = read_entities();
    wrong_recipient[1]["attrs"]["recipient_account_id"] = json!(id(90).to_string());
    assert_eq!(run(wrong_recipient, read_context()), Ok(Decision::Deny));
    let mut delegated = read_entities();
    delegated[1]["attrs"]["delegable"] = json!(true);
    assert_eq!(run(delegated, read_context()), Ok(Decision::Deny));
}
