//! Supplemental pure predicate evidence, never authentication, locking or owner admission.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use super::*;
use crate::Role;
use crate::cedar_pbac::engine::{CompiledBundle, compile_bundle_from_sources};
use cedar_policy::{
    Authorizer, Context, Decision, Entities, EntityUid, PolicySet, Request, Schema, ValidationMode,
    Validator,
};
use console_kernel_core::{OrgId, UserId};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, str::FromStr};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

const POLICY_ID: &str = "legacy-platform-authority-v1";
const SCHEMA_ID: &str = "legacy-platform-subject-resource-v1";
const COMPANY: &str = "11111111-1111-4111-8111-111111111111";
const OTHER_COMPANY: &str = "22222222-2222-4222-8222-222222222222";
const SUBJECT: &str = "33333333-3333-4333-8333-333333333333";
const FAMILY: &str = "44444444-4444-4444-8444-444444444444";
const GROUP: &str = "55555555-5555-4555-8555-555555555555";
type FactMutation = (&'static str, fn(&mut PlatformPolicyInput));

const ROLES: [&str; 6] = [
    "MEMBER",
    "RECEPTIONIST",
    "MECHANIC",
    "ADMIN",
    "EXECUTIVE",
    "SUPER_ADMIN",
];
fn oracle() -> Value {
    serde_json::from_str(include_str!("truth-table.json")).unwrap()
}
fn uid(s: &str) -> Uuid {
    Uuid::parse_str(s).unwrap()
}
fn org(s: &str) -> OrgId {
    OrgId::from_uuid(uid(s))
}
fn platform() -> OrgId {
    org("00000000-0000-0000-0000-00000000face")
}
fn now() -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(1_790_000_000).unwrap()
}
fn feature(s: &str) -> PlatformFeature {
    match s {
        "tenant_create" => PlatformFeature::TenantCreate,
        "tenant_list" => PlatformFeature::TenantList,
        "tenant_suspend" => PlatformFeature::TenantSuspend,
        "tenant_remove" => PlatformFeature::TenantRemove,
        "tenant_health_read" => PlatformFeature::TenantHealthRead,
        "tenant_manage" => PlatformFeature::TenantManage,
        "group_manage" => PlatformFeature::GroupManage,
        "platform_audit_read" => PlatformFeature::PlatformAuditRead,
        _ => panic!("oracle action not in frozen roster: {s}"),
    }
}
fn action(f: PlatformFeature) -> &'static str {
    match f {
        PlatformFeature::TenantCreate => "tenant_create",
        PlatformFeature::TenantList => "tenant_list",
        PlatformFeature::TenantSuspend => "tenant_suspend",
        PlatformFeature::TenantRemove => "tenant_remove",
        PlatformFeature::TenantHealthRead => "tenant_health_read",
        PlatformFeature::TenantManage => "tenant_manage",
        PlatformFeature::GroupManage => "group_manage",
        PlatformFeature::PlatformAuditRead => "platform_audit_read",
    }
}
fn target(s: &str) -> PlatformTarget {
    match s {
        "companies" => PlatformTarget::Companies,
        "company" => PlatformTarget::Company(org(COMPANY)),
        "groups" => PlatformTarget::Groups,
        "group" => PlatformTarget::Group(uid(GROUP)),
        "operations" => PlatformTarget::Operations,
        "platform_audit" => PlatformTarget::PlatformAudit,
        _ => panic!("oracle target not in frozen roster: {s}"),
    }
}
fn operation(s: &str) -> PlatformUse {
    match s {
        "read" => PlatformUse::ReadProjection,
        "effect" => PlatformUse::Effect,
        "mint" => PlatformUse::ContextMint,
        _ => panic!("unknown oracle use: {s}"),
    }
}
fn base(action_name: &str, bound: bool) -> PlatformPolicyInput {
    let table = oracle();
    let row = table["features"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["action"] == action_name)
        .unwrap();
    PlatformPolicyInput {
        credential: PlatformCredentialFacts {
            subject: UserId::from_uuid(uid(SUBJECT)),
            home: platform(),
            source_kind: PlatformSourceKind::Direct,
            source_company: None,
            iat: now().unix_timestamp() - 60,
            nbf: now().unix_timestamp() - 60,
            exp: now().unix_timestamp() + 60,
            binding: bound.then(|| PlatformBindingFacts {
                version: 1,
                family_id: uid(FAMILY),
                home: platform(),
                kind: PlatformSourceKind::Direct,
            }),
        },
        current: Some(PlatformCurrentFacts {
            subject: UserId::from_uuid(uid(SUBJECT)),
            home: platform(),
            roles: Some(vec!["SUPER_ADMIN".into()]),
            active: Some(true),
            account_fenced: Some(false),
        }),
        family: bound.then(|| PlatformFamilyFacts {
            id: uid(FAMILY),
            user_id: UserId::from_uuid(uid(SUBJECT)),
            org_id: Some(platform()),
            protocol: "LEGACY_COMPANY".into(),
            created_at: now() - Duration::hours(1),
            revoked_at: None,
            account_security_generation: None,
            auth_time: None,
            assurance: None,
        }),
        absolute_family_ttl: Duration::days(1),
        now: now(),
        feature: feature(action_name),
        target: target(row["canonical_target"].as_str().unwrap()),
        use_kind: operation(row["canonical_use"].as_str().unwrap()),
        call_site: PlatformCallSite::DirectPlatformOwner,
    }
}
fn roles(input: &mut PlatformPolicyInput, values: &[String]) {
    input.current.as_mut().unwrap().roles = Some(values.to_vec());
}
fn list_strings(v: &Value) -> Vec<String> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().into())
        .collect()
}
fn result_name(d: PlatformPolicyDecision) -> &'static str {
    match d {
        PlatformPolicyDecision::Allow { policy_identity } => {
            assert_eq!(policy_identity, POLICY_ID);
            "Allow"
        }
        PlatformPolicyDecision::Deny { .. } => "Deny",
        PlatformPolicyDecision::UpgradeRequired => "UpgradeRequired",
    }
}
fn expect_reason(d: PlatformPolicyDecision, reason: PlatformPolicyDenyReason) {
    // No Debug/PartialEq derive is required by the frozen public API.
    let actual = match d {
        PlatformPolicyDecision::Deny { reason } => reason,
        _ => panic!("expected denial"),
    };
    assert!(matches!(
        (actual, reason),
        (
            PlatformPolicyDenyReason::InvalidMaterial,
            PlatformPolicyDenyReason::InvalidMaterial
        ) | (
            PlatformPolicyDenyReason::PolicyDenied,
            PlatformPolicyDenyReason::PolicyDenied
        ) | (
            PlatformPolicyDenyReason::EvaluatorUnavailable,
            PlatformPolicyDenyReason::EvaluatorUnavailable
        )
    ));
}
fn compiled() -> PlatformPolicy {
    PlatformPolicy::compile_current().expect("frozen embedded policy must strictly compile")
}
fn source_kind(s: &str) -> PlatformSourceKind {
    match s {
        "direct" => PlatformSourceKind::Direct,
        "platform_view_as" => PlatformSourceKind::ViewAs,
        "platform_tenant_context" => PlatformSourceKind::WritableContext,
        _ => panic!("not a typed source kind: {s}"),
    }
}
fn session_case(row: &Value) -> PlatformPolicyInput {
    let mut i = base(row["action"].as_str().unwrap(), row["binding"] == "bound");
    let source = row["source_kind"].as_str().unwrap();
    i.credential.source_kind = source_kind(source);
    i.credential.source_company = (source != "direct").then(|| org(COMPANY));
    if let Some(b) = &mut i.credential.binding {
        b.kind = source_kind(source);
    }
    i.target = target(row["target_kind"].as_str().unwrap());
    i.use_kind = operation(row["use_kind"].as_str().unwrap());
    i.call_site = match row["call_site"].as_str().unwrap() {
        "platform_route" => PlatformCallSite::DirectPlatformOwner,
        "derived_source" => PlatformCallSite::DerivedPlatformSourceCheck,
        _ => panic!("unknown callsite"),
    };
    match row["target_relation"].as_str() {
        None | Some("same_as_signed_company") => (),
        Some("different_valid_company") => i.target = PlatformTarget::Company(org(OTHER_COMPANY)),
        Some("nil_company") => i.target = PlatformTarget::Company(OrgId::from_uuid(Uuid::nil())),
        Some("platform_sentinel_company") => i.target = PlatformTarget::Company(platform()),
        _ => panic!("unknown target relation"),
    }
    if row["id"] == "ordinary-company-super-admin" {
        i.credential.home = org(COMPANY);
        i.current.as_mut().unwrap().home = org(COMPANY);
        i.credential.binding.as_mut().unwrap().home = org(COMPANY);
        i.family.as_mut().unwrap().org_id = Some(org(COMPANY));
    }
    i
}

#[test]
fn finite_platform_oracle_rosters_and_every_role_subset_are_complete() {
    assert_eq!(
        hex::encode(Sha256::digest(include_bytes!("truth-table.json"))),
        "366da33371b94527d7c3a20688c89425e1ca8da5961d1d06c0961822464f873c"
    );
    let o = oracle();
    assert_eq!(Role::ALL.map(Role::as_str), ROLES);
    assert_eq!(list_strings(&o["known_roles"]), ROLES);
    let expected = o["features"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["action"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        PlatformFeature::ALL.map(action).as_slice(),
        expected.as_slice()
    );
    assert_eq!(expected.len(), 8);
    assert_eq!(o["role_matrix"].as_array().unwrap().len(), 64);
    let mut masks = BTreeSet::new();
    for row in o["role_matrix"].as_array().unwrap() {
        let rs = list_strings(&row["current_roles"]);
        let mut mask = 0usize;
        for r in &rs {
            let bit = 1 << ROLES.iter().position(|v| v == r).unwrap();
            assert_eq!(mask & bit, 0);
            mask |= bit;
        }
        assert!(masks.insert(mask));
        for mode in ["bound_direct", "historical_direct_canonical_use"] {
            let keys = row[mode]
                .as_object()
                .unwrap()
                .keys()
                .map(String::as_str)
                .collect::<BTreeSet<_>>();
            assert_eq!(keys, expected.iter().copied().collect());
        }
    }
    assert_eq!(masks, (0..64).collect());
    assert_eq!(o["role_matrix_decision_cells"], 1024);
    assert_eq!(o["single_fault_cases"].as_array().unwrap().len(), 44);
    assert_eq!(
        o["single_fault_cases"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["applies_to_actions"].as_array().unwrap().len())
            .sum::<usize>(),
        341
    );
    assert_eq!(
        o["session_and_callsite_cases"].as_array().unwrap().len(),
        70
    );
}

#[test]
fn finite_platform_all_1024_frozen_role_feature_session_cells() {
    let policy = compiled();
    let o = oracle();
    let mut count = 0;
    for row in o["role_matrix"].as_array().unwrap() {
        for (mode, bound) in [
            ("bound_direct", true),
            ("historical_direct_canonical_use", false),
        ] {
            for feature in o["features"].as_array().unwrap() {
                let a = feature["action"].as_str().unwrap();
                let mut input = base(a, bound);
                roles(&mut input, &list_strings(&row["current_roles"]));
                assert_eq!(
                    result_name(policy.evaluate_current(&input)),
                    row[mode][a].as_str().unwrap(),
                    "{} {mode} {a}",
                    row["id"]
                );
                count += 1;
            }
        }
    }
    assert_eq!(count, 1024);
}

#[test]
fn finite_platform_all_typed_session_callsite_rows_repeat_eight_role_controls() {
    let policy = compiled();
    let o = oracle();
    let mut count = 0;
    let mut raw_only = Vec::new();
    let mut controls = ROLES
        .iter()
        .map(|r| vec![(*r).to_owned()])
        .collect::<Vec<_>>();
    controls.push(vec![]);
    controls.push(vec!["SUPER_ADMIN".into(), "ADMIN".into()]);
    for row in o["session_and_callsite_cases"].as_array().unwrap() {
        if row["source_kind"] == "group_admin" {
            raw_only.push(row["id"].as_str().unwrap());
            continue;
        }
        for rs in &controls {
            let mut input = session_case(row);
            roles(&mut input, rs);
            let expected = if rs.iter().any(|r| r == "SUPER_ADMIN") {
                row["expected"].as_str().unwrap()
            } else {
                "Deny"
            };
            assert_eq!(
                result_name(policy.evaluate_current(&input)),
                expected,
                "{} {rs:?}",
                row["id"]
            );
            count += 1;
        }
    }
    assert_eq!(raw_only, ["group-derived-not-platform-source"]);
    assert_eq!(count, 552);
}

// Facts represent verified/current material. Invalid signature and raw native/Group
// credential shapes cannot be manufactured into this type. See coverage.json.
fn mutate_fact(id: &str, i: &mut PlatformPolicyInput) -> bool {
    match id {
        "subject_missing" | "current_auth_material_unavailable" => i.current = None,
        "subject_inactive" => i.current.as_mut().unwrap().active = Some(false),
        "subject_id_nil" => i.current.as_mut().unwrap().subject = UserId::from_uuid(Uuid::nil()),
        "subject_home_missing" => i.current.as_mut().unwrap().home = OrgId::from_uuid(Uuid::nil()),
        "subject_home_ordinary_company" => {
            i.current.as_mut().unwrap().home = org(COMPANY);
            i.credential.home = org(COMPANY);
        }
        "subject_home_platform_claim_only" => i.current.as_mut().unwrap().home = org(COMPANY),
        "account_fence_present" => i.current.as_mut().unwrap().account_fenced = Some(true),
        "account_fence_projection_missing" | "account_fence_projection_null" => {
            i.current.as_mut().unwrap().account_fenced = None
        }
        "current_roles_missing" | "current_roles_null" => i.current.as_mut().unwrap().roles = None,
        "current_roles_unknown_even_with_super_admin" => {
            i.current.as_mut().unwrap().roles = Some(vec!["SUPER_ADMIN".into(), "UNKNOWN".into()])
        }
        "current_roles_empty" => i.current.as_mut().unwrap().roles = Some(vec![]),
        "jwt_super_admin_only_current_member" | "group_admin_grant_only" => {
            i.current.as_mut().unwrap().roles = Some(vec!["MEMBER".into()])
        }
        "credential_subject_mismatch" => {
            i.credential.subject = UserId::from_uuid(uid(OTHER_COMPANY))
        }
        "credential_home_mismatch" => i.credential.home = org(OTHER_COMPANY),
        "credential_expired_at_now" => i.credential.exp = i.now.unix_timestamp(),
        "credential_not_yet_valid" => i.credential.nbf = i.now.unix_timestamp() + 1,
        "bound_family_missing" => i.family = None,
        "bound_family_id_nil" => i.credential.binding.as_mut().unwrap().family_id = Uuid::nil(),
        "bound_family_id_other_live_family_same_subject" => {
            i.family.as_mut().unwrap().id = uid(OTHER_COMPANY)
        }
        "bound_family_subject_mismatch" => {
            i.family.as_mut().unwrap().user_id = UserId::from_uuid(uid(OTHER_COMPANY))
        }
        "bound_family_home_mismatch" => {
            i.family.as_mut().unwrap().org_id = Some(org(OTHER_COMPANY))
        }
        "bound_family_protocol_account_v1" => {
            i.family.as_mut().unwrap().protocol = "ACCOUNT_V1".into()
        }
        "bound_family_revoked" => {
            i.family.as_mut().unwrap().revoked_at = Some(i.now - Duration::seconds(1))
        }
        "bound_family_absolute_deadline_reached" => {
            i.family.as_mut().unwrap().created_at = i.now - i.absolute_family_ttl
        }
        "bound_family_deadline_missing_or_overflow" => i.absolute_family_ttl = Duration::MAX,
        "target_id_nil_or_platform_sentinel_for_company" => {
            i.target = PlatformTarget::Company(OrgId::from_uuid(Uuid::nil()))
        }
        "feature_target_kind_mismatch" => {
            i.target = match i.feature {
                PlatformFeature::PlatformAuditRead => PlatformTarget::Companies,
                _ => PlatformTarget::PlatformAudit,
            }
        }
        "target_group_id_nil" => i.target = PlatformTarget::Group(Uuid::nil()),
        _ => return false,
    }
    true
}

#[test]
fn finite_platform_frozen_representable_material_faults_fail_before_sdk() {
    let policy = compiled();
    let o = oracle();
    let mut count = 0;
    let mut unrepresented = BTreeSet::new();
    for row in o["single_fault_cases"].as_array().unwrap() {
        let id = row["id"].as_str().unwrap();
        for a in row["applies_to_actions"].as_array().unwrap() {
            let mut input = base(a.as_str().unwrap(), true);
            if !mutate_fact(id, &mut input) {
                unrepresented.insert(id);
                continue;
            }
            policy.test_sdk_modes.lock().unwrap().clear();
            let decision = policy.evaluate_current(&input);
            if matches!(
                id,
                "jwt_super_admin_only_current_member" | "group_admin_grant_only"
            ) {
                expect_reason(decision, PlatformPolicyDenyReason::PolicyDenied);
                assert_eq!(*policy.test_sdk_modes.lock().unwrap(), ["bound"]);
            } else {
                expect_reason(decision, PlatformPolicyDenyReason::InvalidMaterial);
                assert!(
                    policy.test_sdk_modes.lock().unwrap().is_empty(),
                    "{id} must fail before SDK"
                );
            }
            count += 1;
        }
    }
    assert_eq!(count, 245);
    assert_eq!(
        unrepresented,
        [
            "credential_signature_invalid",
            "credential_protocol_account_v1",
            "credential_claim_present_malformed_no_absence_fallback",
            "unknown_feature",
            "unsupported_policy_version",
            "schema_or_policy_digest_mismatch",
            "linked_sdk_identity_mismatch",
            "strict_schema_or_policy_compile_error_or_warning",
            "cedar_entities_request_error",
            "cedar_evaluation_diagnostic_error",
            "cedar_evaluation_panic",
            "cedar_no_permit"
        ]
        .into_iter()
        .collect()
    );
}

#[test]
fn finite_platform_additional_family_binding_lifetime_and_source_faults() {
    let cases: [FactMutation; 20] = [
        ("binding_version", |i| {
            i.credential.binding.as_mut().unwrap().version = 2
        }),
        ("binding_home", |i| {
            i.credential.binding.as_mut().unwrap().home = org(COMPANY)
        }),
        ("binding_kind", |i| {
            i.credential.binding.as_mut().unwrap().kind = PlatformSourceKind::ViewAs
        }),
        ("family_home_missing", |i| {
            i.family.as_mut().unwrap().org_id = None
        }),
        ("family_native_generation", |i| {
            i.family.as_mut().unwrap().account_security_generation = Some(1)
        }),
        ("family_native_auth_time", |i| {
            i.family.as_mut().unwrap().auth_time = Some(i.now)
        }),
        ("family_native_assurance", |i| {
            i.family.as_mut().unwrap().assurance = Some("password".into())
        }),
        ("family_unknown_protocol", |i| {
            i.family.as_mut().unwrap().protocol = "UNKNOWN".into()
        }),
        ("current_active_missing", |i| {
            i.current.as_mut().unwrap().active = None
        }),
        ("credential_subject_nil", |i| {
            i.credential.subject = UserId::from_uuid(Uuid::nil())
        }),
        ("credential_home_nil", |i| {
            i.credential.home = OrgId::from_uuid(Uuid::nil())
        }),
        ("issued_future", |i| {
            i.credential.iat = i.now.unix_timestamp() + 1
        }),
        ("issued_after_expiry", |i| {
            i.credential.iat = i.credential.exp + 1
        }),
        ("nbf_after_expiry", |i| {
            i.credential.nbf = i.credential.exp + 1
        }),
        ("ttl_zero", |i| i.absolute_family_ttl = Duration::ZERO),
        ("ttl_negative", |i| {
            i.absolute_family_ttl = Duration::seconds(-1)
        }),
        ("source_company_direct", |i| {
            i.credential.source_company = Some(org(COMPANY))
        }),
        ("current_unknown_lowercase", |i| {
            i.current.as_mut().unwrap().roles = Some(vec!["super_admin".into()])
        }),
        ("current_role_whitespace", |i| {
            i.current.as_mut().unwrap().roles = Some(vec!["SUPER_ADMIN ".into()])
        }),
        ("orphan_family_without_binding", |i| {
            i.credential.binding = None
        }),
    ];
    let policy = compiled();
    for (id, mutate) in cases {
        let mut i = base("tenant_list", true);
        mutate(&mut i);
        expect_reason(
            policy.evaluate_current(&i),
            PlatformPolicyDenyReason::InvalidMaterial,
        );
        assert_eq!(
            result_name(policy.evaluate_current(&base("tenant_list", true))),
            "Allow",
            "positive after {id}"
        );
    }
    // Distinct nil and sentinel Company targets; frozen combined row tests nil above.
    for a in [
        "tenant_suspend",
        "tenant_remove",
        "tenant_health_read",
        "tenant_manage",
    ] {
        let mut i = base(a, true);
        i.target = PlatformTarget::Company(platform());
        expect_reason(
            policy.evaluate_current(&i),
            PlatformPolicyDenyReason::InvalidMaterial,
        );
    }
    let mut i = base("tenant_list", true);
    i.now += Duration::seconds(59);
    assert_eq!(result_name(policy.evaluate_current(&i)), "Allow");
    i.now += Duration::seconds(1);
    expect_reason(
        policy.evaluate_current(&i),
        PlatformPolicyDenyReason::InvalidMaterial,
    );
    let mut i = base("tenant_list", true);
    i.family.as_mut().unwrap().created_at =
        i.now - i.absolute_family_ttl + Duration::nanoseconds(1);
    assert_eq!(result_name(policy.evaluate_current(&i)), "Allow");
    i.now += Duration::nanoseconds(1);
    expect_reason(
        policy.evaluate_current(&i),
        PlatformPolicyDenyReason::InvalidMaterial,
    );
}

fn raw_parts(
    action_name: &str,
    target_kind: &str,
    use_kind: &str,
    source: &str,
    callsite: &str,
    bound: bool,
    rs: &[String],
) -> (Value, Value) {
    let key = match target_kind {
        "company" => COMPANY,
        "group" => GROUP,
        v => v,
    };
    let entities = json!([
        {"uid":{"type":"ConsoleLegacyPlatformV1::Subject","id":SUBJECT},"attrs":{"home_org":platform().to_string(),"roles":rs,"active":true,"account_fenced":false},"parents":[]},
        {"uid":{"type":"ConsoleLegacyPlatformV1::Resource","id":key},"attrs":{"platform_scope":"legacy_platform","kind":target_kind,"key":key},"parents":[]}
    ]);
    let context = json!({"policy_version":POLICY_ID,"session_mode":if bound {"bound"} else {"historical_unbound"},"source_kind":source,"use_kind":use_kind,"call_site":callsite,"source_company":if source == "direct" {""} else {COMPANY}});
    let _ = action_name;
    (entities, context)
}
fn raw_sdk(
    bundle: &CompiledBundle,
    action_name: &str,
    entities: Value,
    context: Value,
) -> Result<(Decision, usize), String> {
    let subject =
        EntityUid::from_str(&format!("ConsoleLegacyPlatformV1::Subject::\"{SUBJECT}\"")).unwrap();
    let action = EntityUid::from_str(&format!(
        "ConsoleLegacyPlatformV1::Action::\"{action_name}\""
    ))
    .unwrap();
    let key = entities[1]["uid"]["id"].as_str().unwrap();
    let resource =
        EntityUid::from_str(&format!("ConsoleLegacyPlatformV1::Resource::\"{key}\"")).unwrap();
    let entities =
        Entities::from_json_value(entities, Some(&bundle.schema)).map_err(|e| e.to_string())?;
    let context = Context::from_json_value(context, Some((&bundle.schema, &action)))
        .map_err(|e| e.to_string())?;
    let request = Request::new(subject, action, resource, context, Some(&bundle.schema))
        .map_err(|e| e.to_string())?;
    let response = Authorizer::new().is_authorized(&request, &bundle.policies, &entities);
    Ok((response.decision(), response.diagnostics().errors().count()))
}

#[test]
fn finite_platform_linked_sdk_exact_sources_and_compiled_identity() {
    fn require_send_sync<T: Send + Sync>() {}
    require_send_sync::<PlatformPolicy>();
    assert_eq!(cedar_policy::get_sdk_version().to_string(), "4.12.0");
    assert_eq!(
        SCHEMA_SOURCE,
        include_str!("legacy-platform-v1.cedarschema")
    );
    assert_eq!(POLICY_SOURCE, include_str!("legacy-platform-v1.cedar"));
    assert_eq!(
        hex::encode(Sha256::digest(SCHEMA_SOURCE.as_bytes())),
        "c7a83bf86301f3bfd27444c6e43ff70dfbabfa997e4749c383c07cba0097bff2"
    );
    assert_eq!(
        hex::encode(Sha256::digest(POLICY_SOURCE.as_bytes())),
        "80838954a2ccf18008632a7f9a082a60a7429e2519636c75d5caad9c48068136"
    );
    let p = compiled();
    let key = &p.bundle.key;
    assert_eq!(key.org_id, platform());
    assert_eq!(key.policy_version, 1);
    assert_eq!(key.schema_version, SCHEMA_ID);
    assert_eq!(key.cedar_sdk_version, "4.12.0");
    assert_eq!(key.cedar_language_version, "4.5");
    let mut digest = Sha256::new();
    digest.update(SCHEMA_SOURCE.as_bytes());
    digest.update(POLICY_SOURCE.as_bytes());
    assert_eq!(key.bundle_digest, hex::encode(digest.finalize()));
    assert_eq!(
        result_name(p.evaluate_current(&base("tenant_list", false))),
        "Allow"
    );
}

#[test]
fn finite_platform_same_strict_compiler_rejects_errors_and_warning_only_policy() {
    let good = compile_bundle_from_sources(platform(), 1, SCHEMA_ID, SCHEMA_SOURCE, POLICY_SOURCE);
    assert!(good.is_ok());
    let malformed_schema = "entity Subject = {";
    let undeclared_attr = POLICY_SOURCE.replace("principal.active", "principal.nonexistent_attr");
    let unknown_action = POLICY_SOURCE.replace("tenant_create", "unknown_future_action");
    for (schema, policy) in [
        (malformed_schema, POLICY_SOURCE),
        (SCHEMA_SOURCE, "permit("),
        (SCHEMA_SOURCE, undeclared_attr.as_str()),
        (SCHEMA_SOURCE, unknown_action.as_str()),
    ] {
        assert!(compile_bundle_from_sources(platform(), 1, SCHEMA_ID, schema, policy).is_err());
    }
    let warning = "permit(principal, action, resource) when { false };";
    let schema = Schema::from_str(SCHEMA_SOURCE).unwrap();
    let policies = PolicySet::from_str(warning).unwrap();
    let validation = Validator::new(schema).validate(&policies, ValidationMode::Strict);
    assert_eq!(
        validation.validation_errors().count(),
        0,
        "must exercise warning-only refusal"
    );
    assert!(
        validation.validation_warnings().count() > 0,
        "must produce a real SDK warning"
    );
    assert!(compile_bundle_from_sources(platform(), 1, SCHEMA_ID, SCHEMA_SOURCE, warning).is_err());
}

#[test]
fn finite_platform_private_eligibility_is_distinct_from_public_upgrade() {
    let policy = compiled();
    let o = oracle();
    let mut count = 0;
    for row in o["classification_cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r.get("action").is_some())
    {
        let rs = list_strings(&row["current_roles"]);
        let mut input = session_case(row);
        roles(&mut input, &rs);
        let a = row["action"].as_str().unwrap();
        let k = row["target_kind"].as_str().unwrap();
        let u = row["use_kind"].as_str().unwrap();
        policy.test_sdk_modes.lock().unwrap().clear();
        assert_eq!(
            result_name(policy.evaluate_current(&input)),
            row["expected_public"].as_str().unwrap()
        );
        if rs.is_empty() || rs.iter().any(|r| !ROLES.contains(&r.as_str())) {
            assert!(policy.test_sdk_modes.lock().unwrap().is_empty());
        } else {
            assert_eq!(
                *policy.test_sdk_modes.lock().unwrap(),
                ["historical_unbound"]
            );
        }
        if row["expected_private_cedar"] != "NotEvaluated" {
            let (e, c) = raw_parts(a, k, u, "direct", "platform_route", false, &rs);
            let (decision, errors) = raw_sdk(&policy.bundle, a, e, c).unwrap();
            assert_eq!(errors, 0);
            assert_eq!(
                decision,
                if row["expected_private_cedar"] == "PermitEligibility" {
                    Decision::Allow
                } else {
                    Decision::Deny
                }
            );
        }
        count += 1;
    }
    assert_eq!(count, 20);
    for (a, bound) in [
        ("tenant_list", false),
        ("tenant_create", true),
        ("tenant_manage", true),
    ] {
        policy.test_sdk_modes.lock().unwrap().clear();
        assert_eq!(
            result_name(policy.evaluate_current(&base(a, bound))),
            "Allow"
        );
        assert_eq!(
            *policy.test_sdk_modes.lock().unwrap(),
            [if bound { "bound" } else { "historical_unbound" }]
        );
    }
}

#[test]
fn finite_platform_invalid_historical_material_never_upgrades() {
    let policy = compiled();
    let row = oracle()["session_and_callsite_cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "writable-old-source-effect")
        .unwrap()
        .clone();
    assert_eq!(
        result_name(policy.evaluate_current(&session_case(&row))),
        "UpgradeRequired"
    );
    for fault in [
        "inactive",
        "fenced",
        "expired",
        "version",
        "revoked",
        "other_family",
        "wrong_company",
        "wrong_callsite",
    ] {
        let mut i = session_case(&row);
        match fault {
            "inactive" => i.current.as_mut().unwrap().active = Some(false),
            "fenced" => i.current.as_mut().unwrap().account_fenced = Some(true),
            "expired" => i.credential.exp = i.now.unix_timestamp(),
            "version" | "revoked" | "other_family" => {
                // Presence never falls back to historical absence.
                let mut b = base("tenant_manage", true);
                let mut binding = b.credential.binding.take().unwrap();
                binding.kind = PlatformSourceKind::WritableContext;
                i.credential.binding = Some(binding);
                i.family = b.family.take();
                match fault {
                    "version" => i.credential.binding.as_mut().unwrap().version = 0,
                    "revoked" => i.family.as_mut().unwrap().revoked_at = Some(i.now),
                    "other_family" => i.family.as_mut().unwrap().id = uid(OTHER_COMPANY),
                    _ => unreachable!(),
                }
            }
            "wrong_company" => i.target = PlatformTarget::Company(org(OTHER_COMPANY)),
            "wrong_callsite" => i.call_site = PlatformCallSite::DirectPlatformOwner,
            _ => unreachable!(),
        }
        policy.test_sdk_modes.lock().unwrap().clear();
        expect_reason(
            policy.evaluate_current(&i),
            PlatformPolicyDenyReason::InvalidMaterial,
        );
        assert!(policy.test_sdk_modes.lock().unwrap().is_empty(), "{fault}");
    }
    for fault in ["no_permit", "diagnostic", "panic"] {
        let mut p = compiled();
        match fault {
            "no_permit" => p.bundle.policies = PolicySet::new(),
            "diagnostic" => p.bundle.policies = PolicySet::from_str("permit(principal, action, resource); permit(principal, action, resource) when { principal.missing_attribute };").unwrap(),
            "panic" => p.test_panic_at_authorizer = true,
            _ => unreachable!(),
        }
        expect_reason(
            p.evaluate_current(&session_case(&row)),
            if fault == "no_permit" {
                PlatformPolicyDenyReason::PolicyDenied
            } else {
                PlatformPolicyDenyReason::EvaluatorUnavailable
            },
        );
        assert_eq!(*p.test_sdk_modes.lock().unwrap(), ["historical_unbound"]);
    }
}

#[test]
fn finite_platform_corrupted_compiled_identity_refuses_before_sdk_for_every_action() {
    for a in PlatformFeature::ALL.map(action) {
        for fault in ["scope", "version", "schema", "digest", "sdk", "language"] {
            let mut p = compiled();
            match fault {
                "scope" => p.bundle.key.org_id = org(COMPANY),
                "version" => p.bundle.key.policy_version = 2,
                "schema" => p.bundle.key.schema_version.push_str("-corrupt"),
                "digest" => p.bundle.key.bundle_digest.push('0'),
                "sdk" => p.bundle.key.cedar_sdk_version = "4.11.2".into(),
                "language" => p.bundle.key.cedar_language_version = "4.4".into(),
                _ => unreachable!(),
            }
            expect_reason(
                p.evaluate_current(&base(a, true)),
                PlatformPolicyDenyReason::InvalidMaterial,
            );
            assert!(p.test_sdk_modes.lock().unwrap().is_empty());
        }
    }
}

#[test]
fn finite_platform_real_sdk_faults_and_panic_never_allow_or_upgrade() {
    for a in PlatformFeature::ALL.map(action) {
        for bound in [true, false] {
            let positive = compiled();
            let input = base(a, bound);
            assert_ne!(result_name(positive.evaluate_current(&input)), "Deny");
            for fault in ["entities", "request", "diagnostic", "panic", "no_permit"] {
                let mut p = compiled();
                match fault {
                    "entities" => p.bundle.schema = Schema::from_str(&SCHEMA_SOURCE.replace("\"active\": Bool", "\"active\": String")).unwrap(),
                    "request" => p.bundle.schema = Schema::from_str(&SCHEMA_SOURCE.replace(&format!("action \"{a}\""), "action \"unknown_replacement\"")).unwrap(),
                    "diagnostic" => p.bundle.policies = PolicySet::from_str("permit(principal, action, resource); permit(principal, action, resource) when { principal.missing_attribute };").unwrap(),
                    "panic" => p.test_panic_at_authorizer = true,
                    "no_permit" => p.bundle.policies = PolicySet::new(),
                    _ => unreachable!(),
                }
                expect_reason(
                    p.evaluate_current(&input),
                    if fault == "no_permit" {
                        PlatformPolicyDenyReason::PolicyDenied
                    } else {
                        PlatformPolicyDenyReason::EvaluatorUnavailable
                    },
                );
                if matches!(fault, "entities" | "request") {
                    assert!(p.test_sdk_modes.lock().unwrap().is_empty());
                } else {
                    assert_eq!(
                        *p.test_sdk_modes.lock().unwrap(),
                        [if bound { "bound" } else { "historical_unbound" }]
                    );
                }
            }
        }
    }
    // Establish actual SDK Allow+diagnostic positive control: simply checking
    // response.decision() would incorrectly expose a permit here.
    let mut p = compiled();
    p.bundle.policies = PolicySet::from_str("permit(principal, action, resource); permit(principal, action, resource) when { principal.missing_attribute };").unwrap();
    let (e, c) = raw_parts(
        "tenant_create",
        "companies",
        "effect",
        "direct",
        "platform_route",
        false,
        &["SUPER_ADMIN".into()],
    );
    let (decision, errors) = raw_sdk(&p.bundle, "tenant_create", e, c).unwrap();
    assert_eq!(decision, Decision::Allow);
    assert!(errors > 0);
}

#[test]
fn finite_platform_raw_schema_and_closed_source_controls() {
    let p = compiled();
    let o = oracle();
    let mut controls = ROLES
        .iter()
        .map(|r| vec![(*r).to_owned()])
        .collect::<Vec<_>>();
    controls.push(vec![]);
    controls.push(vec!["SUPER_ADMIN".into(), "ADMIN".into()]);
    // Typed PlatformSourceKind intentionally cannot represent Group. This proves
    // raw Cedar denial only; genuine Auth Group rejection is a separate owner test.
    let r = o["session_and_callsite_cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "group-derived-not-platform-source")
        .unwrap();
    for rs in controls {
        let (e, c) = raw_parts(
            "group_manage",
            "group",
            "read",
            r["source_kind"].as_str().unwrap(),
            "derived_source",
            true,
            &rs,
        );
        let (decision, errors) = raw_sdk(&p.bundle, "group_manage", e, c).unwrap();
        assert_eq!(decision, Decision::Deny);
        assert_eq!(errors, 0);
    }
    for (field, value) in [
        ("policy_version", "unknown"),
        ("session_mode", "native_account"),
        ("source_kind", "unknown"),
        ("use_kind", "unknown"),
        ("call_site", "unknown"),
    ] {
        let (e, mut c) = raw_parts(
            "tenant_list",
            "companies",
            "read",
            "direct",
            "platform_route",
            true,
            &["SUPER_ADMIN".into()],
        );
        c[field] = json!(value);
        let (decision, errors) = raw_sdk(&p.bundle, "tenant_list", e, c).unwrap();
        assert_eq!(decision, Decision::Deny);
        assert_eq!(errors, 0);
    }
    let (e, c) = raw_parts(
        "unknown_action",
        "companies",
        "read",
        "direct",
        "platform_route",
        true,
        &["SUPER_ADMIN".into()],
    );
    assert!(raw_sdk(&p.bundle, "unknown_action", e, c).is_err());
    let (mut e, c) = raw_parts(
        "tenant_list",
        "companies",
        "read",
        "direct",
        "platform_route",
        true,
        &["SUPER_ADMIN".into()],
    );
    e[0]["attrs"]["invented_authority"] = json!(true);
    assert!(raw_sdk(&p.bundle, "tenant_list", e, c).is_err());
    let (e, mut c) = raw_parts(
        "tenant_list",
        "companies",
        "read",
        "direct",
        "platform_route",
        true,
        &["SUPER_ADMIN".into()],
    );
    c.as_object_mut().unwrap().remove("policy_version");
    assert!(raw_sdk(&p.bundle, "tenant_list", e, c).is_err());
}

#[test]
fn finite_platform_bound_precision_and_historical_signed_time_compatibility() {
    let p = compiled();
    let mut same_second = base("tenant_list", true);
    same_second.now = now() + Duration::milliseconds(750);
    same_second.family.as_mut().unwrap().created_at = now() + Duration::milliseconds(500);
    same_second.credential.iat = now().unix_timestamp();
    same_second.credential.nbf = same_second.credential.iat;
    assert!(
        OffsetDateTime::from_unix_timestamp(same_second.credential.iat).unwrap()
            < same_second.family.as_ref().unwrap().created_at
    );
    assert_eq!(
        same_second
            .family
            .as_ref()
            .unwrap()
            .created_at
            .unix_timestamp(),
        same_second.credential.iat
    );
    assert_eq!(result_name(p.evaluate_current(&same_second)), "Allow");
    same_second.credential.iat -= 1;
    expect_reason(
        p.evaluate_current(&same_second),
        PlatformPolicyDenyReason::InvalidMaterial,
    );

    let mut future_family = base("tenant_list", true);
    future_family.family.as_mut().unwrap().created_at =
        future_family.now + Duration::microseconds(1);
    future_family.credential.iat = future_family.now.unix_timestamp();
    future_family.credential.nbf = future_family.credential.iat;
    expect_reason(
        p.evaluate_current(&future_family),
        PlatformPolicyDenyReason::InvalidMaterial,
    );

    let faults: [FactMutation; 5] = [
        ("nbf_equals_exp", |i| i.credential.nbf = i.credential.exp),
        ("nbf_precedes_iat", |i| {
            i.credential.nbf = i.credential.iat - 1
        }),
        ("iat_unrepresentable", |i| i.credential.iat = i64::MIN),
        ("nbf_unrepresentable", |i| i.credential.nbf = i64::MAX),
        ("exp_unrepresentable", |i| i.credential.exp = i64::MAX),
    ];
    for (name, mutate) in faults {
        let mut i = base("tenant_list", true);
        mutate(&mut i);
        p.test_sdk_modes.lock().unwrap().clear();
        expect_reason(
            p.evaluate_current(&i),
            PlatformPolicyDenyReason::InvalidMaterial,
        );
        assert!(p.test_sdk_modes.lock().unwrap().is_empty(), "{name}");
    }
    // These historical facts represent already verified old claims. The existing
    // verifier admits future iat with nbf0; only original exp is rechecked here.
    // Bound-only time ordering cannot silently narrow the absence exception.
    for (a, expected) in [
        ("tenant_list", "Allow"),
        ("tenant_create", "UpgradeRequired"),
    ] {
        let mut old = base(a, false);
        old.credential.iat = old.now.unix_timestamp() + 20;
        old.credential.nbf = 0;
        let original = (old.credential.iat, old.credential.nbf, old.credential.exp);
        p.test_sdk_modes.lock().unwrap().clear();
        assert_eq!(result_name(p.evaluate_current(&old)), expected);
        assert_eq!(*p.test_sdk_modes.lock().unwrap(), ["historical_unbound"]);
        assert_eq!(
            (old.credential.iat, old.credential.nbf, old.credential.exp),
            original
        );
        old.credential.exp = old.now.unix_timestamp();
        expect_reason(
            p.evaluate_current(&old),
            PlatformPolicyDenyReason::InvalidMaterial,
        );
        let mut bound = base(a, true);
        bound.credential.iat = original.0;
        bound.credential.nbf = original.1;
        expect_reason(
            p.evaluate_current(&bound),
            PlatformPolicyDenyReason::InvalidMaterial,
        );
    }
}

#[test]
fn finite_platform_operation_shapes_refuse_before_sdk_and_preserve_valid_contexts() {
    // Explicit independent closed operation table from round5 DESIGN, not
    // production valid_material or the tenant permission matrix. Each tuple is
    // source, action, target kind, use. Boundness changes only public completion.
    let admitted = BTreeSet::from([
        ("direct", "tenant_create", "companies", "effect"),
        ("direct", "tenant_list", "companies", "read"),
        ("direct", "tenant_suspend", "company", "effect"),
        ("direct", "tenant_remove", "company", "effect"),
        ("direct", "tenant_health_read", "company", "read"),
        ("direct", "tenant_health_read", "company", "mint"),
        ("direct", "tenant_health_read", "operations", "read"),
        ("direct", "tenant_health_read", "operations", "effect"),
        ("direct", "tenant_manage", "company", "mint"),
        ("direct", "tenant_manage", "operations", "effect"),
        ("direct", "group_manage", "groups", "read"),
        ("direct", "group_manage", "groups", "effect"),
        ("direct", "group_manage", "group", "read"),
        ("direct", "group_manage", "group", "effect"),
        ("direct", "group_manage", "company", "read"),
        ("direct", "group_manage", "company", "effect"),
        ("direct", "platform_audit_read", "platform_audit", "read"),
        ("platform_view_as", "tenant_health_read", "company", "read"),
        (
            "platform_tenant_context",
            "tenant_manage",
            "company",
            "read",
        ),
        (
            "platform_tenant_context",
            "tenant_manage",
            "company",
            "effect",
        ),
    ]);
    assert_eq!(admitted.len(), 20);
    let p = compiled();
    let mut denied = 0;
    let mut allowed = 0;
    let mut upgrade = 0;
    for bound in [true, false] {
        for source in ["direct", "platform_view_as", "platform_tenant_context"] {
            for a in PlatformFeature::ALL.map(action) {
                for k in [
                    "companies",
                    "company",
                    "groups",
                    "group",
                    "operations",
                    "platform_audit",
                ] {
                    for u in ["read", "effect", "mint"] {
                        let row = json!({"action":a,"binding":if bound {"bound"} else {"absent"},
                            "source_kind":source,"target_kind":k,"use_kind":u,
                            "call_site":if source=="direct" {"platform_route"} else {"derived_source"}});
                        let input = session_case(&row);
                        p.test_sdk_modes.lock().unwrap().clear();
                        let decision = p.evaluate_current(&input);
                        if admitted.contains(&(source, a, k, u)) {
                            let expected = if !bound && u != "read" {
                                upgrade += 1;
                                "UpgradeRequired"
                            } else {
                                allowed += 1;
                                "Allow"
                            };
                            assert_eq!(
                                result_name(decision),
                                expected,
                                "{source} {a} {k} {u} bound={bound}"
                            );
                            assert_eq!(
                                *p.test_sdk_modes.lock().unwrap(),
                                [if bound { "bound" } else { "historical_unbound" }]
                            );
                        } else {
                            expect_reason(decision, PlatformPolicyDenyReason::InvalidMaterial);
                            assert!(
                                p.test_sdk_modes.lock().unwrap().is_empty(),
                                "invalid operation reached SDK: {source} {a} {k} {u} bound={bound}"
                            );
                            denied += 1;
                        }
                    }
                }
            }
        }
    }
    assert_eq!((denied, allowed, upgrade), (824, 29, 11));
}
