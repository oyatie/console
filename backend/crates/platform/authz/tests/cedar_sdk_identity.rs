#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Real compiled SDK identity and mixed-version boundary regression.
//! No database migration, stored-record rewrite or native Company policy claim.
use std::collections::BTreeSet;

use console_kernel_core::{BranchScope, OrgId, UserId};
use console_platform_authz::cedar_pbac::engine;
use console_platform_authz::{
    Action, AuthorizationRequest, AuthorizationResource, CedarEvaluation, CoexistenceMapEntry,
    CompiledBundleCacheKey, DecisionEffect, DecisionReason, DualEngineMode, Feature, Principal,
    RlsScopeProof, Role, SubjectFreshness, SubjectFreshnessRequirement,
    evaluate_cedar_pbac_boundary, observe_cedar_pbac_decision,
};

#[test]
fn compiled_bundle_keys_name_the_actual_linked_cedar_sdk() {
    let actual = cedar_policy::get_sdk_version().to_string();
    let org = OrgId::knl();
    let bundles = [
        engine::compile_bundle(org, 7).expect("real role bundle strict compilation"),
        engine::compile_bundle_for_feature(org, 7, Feature::WorkOrderReadAll)
            .expect("real per-feature bundle strict compilation"),
        engine::compile_audit_stream_bundle(org, 7)
            .expect("real audit-stream bundle strict compilation"),
    ];
    for bundle in bundles {
        assert_eq!(
            bundle.key.cedar_sdk_version, actual,
            "compiled bundle identity must name its actual linked Cedar SDK"
        );
    }
    assert_eq!(engine::CEDAR_SDK_VERSION, actual);
}

#[test]
fn current_sdk_evaluation_rejects_historical_key_without_relabeling_it() {
    let org = OrgId::knl();
    let bundle = engine::compile_bundle(org, 7).expect("real role bundle strict compilation");
    let request = AuthorizationRequest::new(
        Principal::new(
            UserId::new(),
            org,
            BTreeSet::from([Role::SuperAdmin]),
            BranchScope::All,
        ),
        Action::new(Feature::RoleManage),
        AuthorizationResource::org_wide(org, "policy_role"),
    )
    .with_policy_domain("identity.policy")
    .with_subject_freshness(SubjectFreshness {
        policy_version: 7,
        subject_version: 1,
        session_generation: 1,
        step_up_generation: None,
    })
    .requiring_freshness(SubjectFreshnessRequirement {
        min_policy_version: 7,
        min_subject_version: 1,
        min_session_generation: 1,
        required_step_up_generation: None,
    })
    .with_rls_scope_proof(RlsScopeProof::runtime_role_guc(org));
    let evaluated = engine::evaluate(&request, &bundle);
    assert!(
        matches!(evaluated, CedarEvaluation::Allow { .. }),
        "real policy permits the otherwise valid request"
    );
    let mut current = bundle.key.clone();
    current.cedar_sdk_version = cedar_policy::get_sdk_version().to_string();
    let entry = |key| {
        CoexistenceMapEntry::new(
            "identity.policy.role_manage",
            "identity.policy",
            Feature::RoleManage,
            "policy_role",
            DualEngineMode::CedarOnly,
            Some(key),
        )
    };
    let current_entry = entry(current.clone());
    let positive = evaluate_cedar_pbac_boundary(&request, Some(&current_entry), evaluated.clone());
    assert_eq!(
        positive.effect,
        DecisionEffect::Allow,
        "actual SDK registration must match the evaluated bundle"
    );
    assert_eq!(positive.reason, DecisionReason::CedarAllowed);

    // A manually authored prior key is valid serialized evidence. Its SDK field
    // must remain 4.11.2; reading it cannot convert it into current authority.
    let mut historical = current.clone();
    historical.cedar_sdk_version = "4.11.2".to_owned();
    let bytes = serde_json::to_vec(&historical).unwrap();
    let decoded: CompiledBundleCacheKey = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(decoded.cedar_sdk_version, "4.11.2");
    assert_ne!(decoded, current, "only the historical SDK identity differs");
    let historical_entry = entry(decoded);
    let denied = evaluate_cedar_pbac_boundary(&request, Some(&historical_entry), evaluated.clone());
    assert_eq!(denied.effect, DecisionEffect::Deny);
    assert_eq!(denied.reason, DecisionReason::StalePolicyBundle);
    let observation =
        observe_cedar_pbac_decision(&request, Some(&historical_entry), Some(&evaluated), denied);
    let observed_historical = observation.bundle_key.as_ref().unwrap();
    assert_eq!(serde_json::to_vec(observed_historical).unwrap(), bytes);
    assert_eq!(observation.evaluated_bundle_key.as_ref(), Some(&current));
    assert_eq!(serde_json::to_vec(&historical).unwrap(), bytes);
}
