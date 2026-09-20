// Pure finite Company Cedar prerequisite probes, not an enrollment implementation.
// Include as a child of cedar_pbac::engine so its existing compiled artifacts
// can be evaluated without exposing private production bundle fields.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use super::{CompiledBundle, compile_bundle_from_sources};
use cedar_policy::{Authorizer, Context, Decision, Entities, EntityUid, Request};
use console_kernel_core::OrgId;
use serde_json::{Value, json};
use std::str::FromStr;

const SCHEMA: &str = r###"entity Account;
entity CompanyResource = {
  "org_id": String,
  "object_type_id": String
};

action "context.discover" appliesTo {
  principal: [Account], resource: [CompanyResource],
  context: { "requested_fields": Set<String> }
};
action "company.identity.read" appliesTo {
  principal: [Account], resource: [CompanyResource],
  context: { "requested_fields": Set<String> }
};
action "company.policy.read" appliesTo {
  principal: [Account], resource: [CompanyResource],
  context: { "requested_fields": Set<String> }
};
action "company.policy.assign" appliesTo {
  principal: [Account], resource: [CompanyResource],
  context: { "requested_fields": Set<String> }
};
action "company.policy.revoke" appliesTo {
  principal: [Account], resource: [CompanyResource],
  context: { "requested_fields": Set<String> }
};
"###;
const SCHEMA_ID: &str = "native-company-authorization-2026-09-19.1";
const ACCOUNT: &str = "11111111-1111-4111-8111-111111111111";
const OTHER_ACCOUNT: &str = "11111111-1111-4111-8111-111111111112";
const COMPANY: &str = "22222222-2222-4222-8222-222222222222";
const OTHER_COMPANY: &str = "22222222-2222-4222-8222-222222222223";
const WORKSPACE_TYPE: &str = "33333333-3333-4333-8333-333333333333";
const ASSIGNMENT_TYPE: &str = "33333333-3333-4333-8333-333333333334";
const OBJECT: &str = "55555555-5555-4555-8555-555555555555";
const FIELD: &str = "22222222-2222-4222-8222-222222222222/33333333-3333-4333-8333-333333333333/44444444-4444-4444-8444-444444444444/1/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn quote(value: &str) -> String {
    serde_json::to_string(value).unwrap()
}
fn policy(action: &str, object_type: &str, fields: &[&str]) -> String {
    // Cedar strict validation forbids empty set literals. Empty permission
    // still permits only an empty requested field set, never a wildcard.
    let field_condition = if fields.is_empty() {
        "context.requested_fields.isEmpty()".to_owned()
    } else {
        format!(
            "{}.containsAll(context.requested_fields)",
            serde_json::to_string(fields).unwrap()
        )
    };
    // Fixed test template, not a production native compiler.
    format!(
        "permit(principal == Account::{}, action == Action::{}, resource) when {{ resource.org_id == {} && resource.object_type_id == {} && {} }};",
        quote(ACCOUNT),
        quote(action),
        quote(COMPANY),
        quote(object_type),
        field_condition
    )
}
fn compile(source: &str) -> CompiledBundle {
    compile_bundle_from_sources(
        OrgId::from_uuid(uuid::Uuid::parse_str(COMPANY).unwrap()),
        1,
        SCHEMA_ID,
        SCHEMA,
        source,
    )
    .unwrap()
}
fn decide(
    bundle: &CompiledBundle,
    account: &str,
    company: &str,
    object_type: &str,
    action: &str,
    context: Value,
) -> Result<(Decision, usize), String> {
    let resource_id = format!("{company}/{object_type}/{OBJECT}");
    let principal =
        EntityUid::from_str(&format!("Account::{}", quote(account))).map_err(|e| e.to_string())?;
    let action_uid =
        EntityUid::from_str(&format!("Action::{}", quote(action))).map_err(|e| e.to_string())?;
    let resource = EntityUid::from_str(&format!("CompanyResource::{}", quote(&resource_id)))
        .map_err(|e| e.to_string())?;
    let entities = Entities::from_json_value(json!([
        {"uid":{"type":"Account","id":account},"attrs":{},"parents":[]},
        {"uid":{"type":"CompanyResource","id":resource_id},"attrs":{"org_id":company,"object_type_id":object_type},"parents":[]}
    ]), Some(&bundle.schema)).map_err(|e| e.to_string())?;
    let context = Context::from_json_value(context, Some((&bundle.schema, &action_uid)))
        .map_err(|e| e.to_string())?;
    let request = Request::new(
        principal,
        action_uid,
        resource,
        context,
        Some(&bundle.schema),
    )
    .map_err(|e| e.to_string())?;
    let response = Authorizer::new().is_authorized(&request, &bundle.policies, &entities);
    Ok((response.decision(), response.diagnostics().errors().count()))
}
fn request(
    bundle: &CompiledBundle,
    account: &str,
    company: &str,
    object_type: &str,
    action: &str,
    fields: &[&str],
) -> (Decision, usize) {
    decide(
        bundle,
        account,
        company,
        object_type,
        action,
        json!({"requested_fields":fields}),
    )
    .unwrap()
}

#[test]
fn exact_five_action_schema_and_templates_compile_strictly_at_existing_owner() {
    for (action, object_type) in [
        ("context.discover", WORKSPACE_TYPE),
        ("company.identity.read", WORKSPACE_TYPE),
        ("company.policy.read", ASSIGNMENT_TYPE),
        ("company.policy.assign", ASSIGNMENT_TYPE),
        ("company.policy.revoke", ASSIGNMENT_TYPE),
    ] {
        let bundle = compile(&policy(action, object_type, &[]));
        assert_eq!(
            request(&bundle, ACCOUNT, COMPANY, object_type, action, &[]),
            (Decision::Allow, 0),
            "{action}"
        );
    }
}
#[test]
fn exact_identity_permit_denies_other_account_company_type_action_and_field() {
    let bundle = compile(&policy("company.identity.read", WORKSPACE_TYPE, &[FIELD]));
    assert_eq!(
        request(
            &bundle,
            ACCOUNT,
            COMPANY,
            WORKSPACE_TYPE,
            "company.identity.read",
            &[FIELD]
        ),
        (Decision::Allow, 0)
    );
    for (account, company, object_type, action, field) in [
        (
            OTHER_ACCOUNT,
            COMPANY,
            WORKSPACE_TYPE,
            "company.identity.read",
            FIELD,
        ),
        (
            ACCOUNT,
            OTHER_COMPANY,
            WORKSPACE_TYPE,
            "company.identity.read",
            FIELD,
        ),
        (
            ACCOUNT,
            COMPANY,
            ASSIGNMENT_TYPE,
            "company.identity.read",
            FIELD,
        ),
        (
            ACCOUNT,
            COMPANY,
            WORKSPACE_TYPE,
            "company.policy.revoke",
            FIELD,
        ),
        (
            ACCOUNT,
            COMPANY,
            WORKSPACE_TYPE,
            "company.identity.read",
            "unknown-field",
        ),
        (
            ACCOUNT,
            COMPANY,
            WORKSPACE_TYPE,
            "company.identity.read",
            "*",
        ),
        (
            ACCOUNT,
            COMPANY,
            WORKSPACE_TYPE,
            "company.identity.read",
            "field\" || true",
        ),
    ] {
        assert_eq!(
            request(&bundle, account, company, object_type, action, &[field]),
            (Decision::Deny, 0),
            "{account}/{company}/{object_type}/{action}/{field}"
        );
    }
}
#[test]
fn empty_field_permission_is_not_a_field_wildcard() {
    let bundle = compile(&policy("company.identity.read", WORKSPACE_TYPE, &[]));
    assert_eq!(
        request(
            &bundle,
            ACCOUNT,
            COMPANY,
            WORKSPACE_TYPE,
            "company.identity.read",
            &[]
        ),
        (Decision::Allow, 0)
    );
    assert_eq!(
        request(
            &bundle,
            ACCOUNT,
            COMPANY,
            WORKSPACE_TYPE,
            "company.identity.read",
            &[FIELD]
        ),
        (Decision::Deny, 0)
    );
    // The downstream projection must separately return zero properties on empty request.
}
#[test]
fn exact_joint_field_set_does_not_union_independent_permits() {
    let second = FIELD.replace("444444444444/1/", "444444444445/1/");
    let source = format!(
        "{}\n{}",
        policy("company.identity.read", WORKSPACE_TYPE, &[FIELD]),
        policy("company.identity.read", WORKSPACE_TYPE, &[&second])
    );
    let bundle = compile(&source);
    assert_eq!(
        request(
            &bundle,
            ACCOUNT,
            COMPANY,
            WORKSPACE_TYPE,
            "company.identity.read",
            &[FIELD]
        ),
        (Decision::Allow, 0)
    );
    assert_eq!(
        request(
            &bundle,
            ACCOUNT,
            COMPANY,
            WORKSPACE_TYPE,
            "company.identity.read",
            &[&second]
        ),
        (Decision::Allow, 0)
    );
    assert_eq!(
        request(
            &bundle,
            ACCOUNT,
            COMPANY,
            WORKSPACE_TYPE,
            "company.identity.read",
            &[FIELD, &second]
        ),
        (Decision::Deny, 0)
    );
}
#[test]
fn invalid_context_or_unknown_action_cannot_become_allow() {
    let bundle = compile(&policy("company.identity.read", WORKSPACE_TYPE, &[FIELD]));
    for context in [
        json!({}),
        json!({"requested_fields":"*"}),
        json!({"requested_fields":[1]}),
    ] {
        assert!(
            decide(
                &bundle,
                ACCOUNT,
                COMPANY,
                WORKSPACE_TYPE,
                "company.identity.read",
                context
            )
            .is_err()
        );
    }
    assert!(
        decide(
            &bundle,
            ACCOUNT,
            COMPANY,
            WORKSPACE_TYPE,
            "future.unknown",
            json!({"requested_fields":[]})
        )
        .is_err()
    );
}
#[test]
fn malformed_schema_policy_and_warning_only_bundle_are_refused_by_existing_owner() {
    let org = OrgId::from_uuid(uuid::Uuid::parse_str(COMPANY).unwrap());
    let valid = policy("company.identity.read", WORKSPACE_TYPE, &[FIELD]);
    for (schema, policies) in [
        ("entity {", valid.as_str()),
        (SCHEMA, "permit("),
        (
            SCHEMA,
            "permit(principal, action, resource) when { false };",
        ),
    ] {
        assert!(compile_bundle_from_sources(org, 1, SCHEMA_ID, schema, policies).is_err());
    }
    let warning_source = "permit(principal, action, resource) when { false };";
    let warning_schema = cedar_policy::Schema::from_str(SCHEMA).unwrap();
    let warning_policies = cedar_policy::PolicySet::from_str(warning_source).unwrap();
    let validation = cedar_policy::Validator::new(warning_schema)
        .validate(&warning_policies, cedar_policy::ValidationMode::Strict);
    assert_eq!(
        validation.validation_errors().count(),
        0,
        "warning fixture has validation errors"
    );
    assert!(
        validation.validation_warnings().next().is_some(),
        "warning fixture has no warning"
    );
    assert!(compile_bundle_from_sources(org, 1, SCHEMA_ID, SCHEMA, warning_source).is_err());
    let unknown = valid.replace("company.identity.read", "future.unknown");
    assert!(compile_bundle_from_sources(org, 1, SCHEMA_ID, SCHEMA, &unknown).is_err());
}
#[test]
fn legacy_feature_bundle_still_compiles_unchanged_beside_native_schema() {
    assert!(super::compile_bundle(OrgId::knl(), 1).is_ok());
    assert!(super::compile_bundle_for_feature(OrgId::knl(), 1, crate::Feature::RoleManage).is_ok());
}
