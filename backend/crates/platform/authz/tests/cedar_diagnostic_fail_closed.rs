#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Candidate tests against actual existing Cedar adapter code, base58b6f758.
//! No HTTP, DB-loading, policy-admission or full information-flow claim.
use console_kernel_core::{BranchId, BranchScope, OrgId, UserId};
use console_platform_authz::cedar_pbac::{
    authoring::{self, AuthoredPolicy, SimRequest, SimResource, SimSubject},
    engine,
};
use console_platform_authz::{
    Action, AuthorizationRequest, AuthorizationResource, CedarEvaluation, Feature, Principal, Role,
    SubjectFreshness,
};
use std::collections::BTreeSet;

const PERMIT: &str = r#"permit(principal, action == Action::"role_manage", resource);"#;
const OVERFLOW_FORBID: &str = r#"forbid(principal, action == Action::"role_manage", resource)
when { principal.subject_version + 1 < 0 };"#;

fn request(subject_version: u64) -> AuthorizationRequest {
    let org = OrgId::knl();
    let branch = BranchId::new();
    AuthorizationRequest::new(
        Principal::new(
            UserId::new(),
            org,
            BTreeSet::from([Role::SuperAdmin]),
            BranchScope::single(branch),
        ),
        Action::new(Feature::RoleManage),
        AuthorizationResource::branch(org, branch, "policy_role"),
    )
    .with_subject_freshness(SubjectFreshness {
        policy_version: 1,
        subject_version,
        session_generation: 1,
        step_up_generation: None,
    })
}

fn compile(source: &str) -> engine::CompiledBundle {
    engine::compile_bundle_from_sources(
        OrgId::knl(),
        1,
        engine::ROLE_MANAGE_SCHEMA_VERSION,
        engine::ROLE_MANAGE_SCHEMA,
        source,
    )
    .expect("fixture must pass actual strict compiler before runtime diagnostic assertions")
}

#[test]
fn clean_compiled_permit_with_nonoverflowing_forbid_allows() {
    let bundle = compile(&format!("{PERMIT}\n{OVERFLOW_FORBID}"));
    assert!(matches!(
        engine::evaluate(&request(1), &bundle),
        CedarEvaluation::Allow { .. }
    ));
}

#[test]
fn compiled_allow_with_runtime_overflow_diagnostic_fails_closed() {
    let req = request(i64::MAX as u64);
    assert!(
        matches!(
            engine::evaluate(&req, &compile(PERMIT)),
            CedarEvaluation::Allow { .. }
        ),
        "maximum subject version itself remains valid under a clean permit"
    );
    // Production fail-closed diagnostic path witnesses the failing policy alone.
    // A compile/type error above is a fixture failure, not this test's intended RED.
    let witness = engine::evaluate(&req, &compile(OVERFLOW_FORBID));
    match witness {
        CedarEvaluation::Deny { reason, .. } => assert!(
            reason.contains("evaluation errors"),
            "fixture must reach an evaluation diagnostic, not ordinary deny: {reason}"
        ),
        CedarEvaluation::Error { reason } => assert!(
            !reason.is_empty(),
            "typed diagnostic failure must have protected diagnostic evidence"
        ),
        other => panic!("overflow fixture did not establish runtime failure: {other:?}"),
    }
    let bundle = compile(&format!("{PERMIT}\n{OVERFLOW_FORBID}"));
    assert!(
        !matches!(
            engine::evaluate(&req, &bundle),
            CedarEvaluation::Allow { .. }
        ),
        "a matching permit must not erase a second policy's runtime diagnostic"
    );
}

fn subject() -> SimSubject {
    SimSubject {
        org: OrgId::knl(),
        user_id: "diagnostic-subject".to_owned(),
        roles: vec![],
        clearance_keys: vec![],
    }
}
fn resource(owner: Option<&str>) -> SimResource {
    SimResource {
        org: OrgId::knl(),
        resource_type: "payroll_line".to_owned(),
        resource_id: Some("synthetic-line".to_owned()),
        owner: owner.map(str::to_owned),
        branch: None,
        legal_hold: None,
    }
}
fn permit(action: &str) -> AuthoredPolicy {
    AuthoredPolicy::new(
        "permit_clean",
        format!("permit(principal, action == Action::\"{action}\", resource);"),
    )
}
fn missing_owner_forbid(action: &str) -> AuthoredPolicy {
    AuthoredPolicy::new(
        "forbid_owner",
        format!(
            "forbid(principal, action == Action::\"{action}\", resource) when {{ resource.owner == principal.user_id }};"
        ),
    )
}
fn assert_diagnostic(policies: &[AuthoredPolicy], action: &str) {
    let out = authoring::simulate(
        policies,
        &SimRequest {
            subject: subject(),
            action: action.to_owned(),
            resource: resource(None),
            purpose: None,
            field: (action == "read_field").then(|| "salary".to_owned()),
        },
    );
    assert!(
        !out.errors.is_empty(),
        "fixture must reach an actual Cedar diagnostic before the visibility assertion: {out:?}"
    );
}

#[test]
fn valid_field_policy_controls_show_clean_allow_and_matching_forbid() {
    assert!(authoring::property_field_visible(
        &[permit("read_field")],
        subject(),
        resource(None),
        "salary"
    ));
    assert!(!authoring::property_field_visible(
        &[permit("read_field"), missing_owner_forbid("read_field")],
        subject(),
        resource(Some("diagnostic-subject")),
        "salary"
    ));
}

#[test]
fn field_visibility_fails_closed_when_allow_has_diagnostics() {
    // An adversarial persisted-source boundary, not a claim that strict no-code
    // authoring admits an unguarded optional-attribute policy.
    let policies = [permit("read_field"), missing_owner_forbid("read_field")];
    assert_diagnostic(&policies, "read_field");
    assert!(
        !authoring::property_field_visible(&policies, subject(), resource(None), "salary"),
        "field helper must not discard evaluation errors from a matching Allow"
    );
}

#[test]
fn row_visibility_fails_closed_when_allow_has_diagnostics() {
    assert!(authoring::object_row_visible(
        &[permit("view")],
        subject(),
        resource(None)
    ));
    let policies = [permit("view"), missing_owner_forbid("view")];
    assert_diagnostic(&policies, "view");
    assert!(
        !authoring::object_row_visible(&policies, subject(), resource(None)),
        "row helper must not discard evaluation errors from a matching Allow"
    );
}
