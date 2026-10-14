#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
//! Frozen reference Cedar syntax/typechecking only; no current-source constructor,
//! Company owner, installed policy, business data or workflow acceptance claim.
use console_kernel_core::{KernelError, OrgId};
use console_platform_authz::cedar_pbac::engine;
use sha2::{Digest, Sha256};

const BUNDLE: &str = "native-company-hr-appointment-bootstrap-v1";
const SCHEMA: &str = include_str!("fixtures/native_hr_bootstrap.cedarschema");
const POLICY: &str = include_str!("fixtures/native_hr_bootstrap.cedar");
const SCHEMA_SHA: &str = "89b67aa1d8df01fa6e4e0e9ae99885609d0e607b53213563de5bb2f30004ce14";
const POLICY_SHA: &str = "bfe91fdcb3b31a67ebb3b7d514669e5fcdc0faa0e2f92d473bd7eec4c18cf0c9";

fn compile(schema: &str, policy: &str) -> Result<engine::CompiledBundle, KernelError> {
    engine::compile_bundle_from_sources(OrgId::knl(), 7, BUNDLE, schema, policy)
}

fn replace_exact(source: &str, before: &str, after: &str, count: usize) -> String {
    assert_eq!(
        source.matches(before).count(),
        count,
        "control mutation drift"
    );
    source.replace(before, after)
}

fn rejected(schema: &str, policy: &str, phase: &str) {
    let error = match compile(schema, policy) {
        Err(error) => error,
        Ok(_) => panic!("reference corruption passed the actual strict Cedar compiler"),
    };
    assert!(
        error.to_string().contains(phase),
        "reference control failed at an unrelated compiler phase: {error}"
    );
}

#[test]
fn native_hr_reference_assets_strictly_compile_with_exact_bundle_identity() {
    assert_eq!(hex::encode(Sha256::digest(SCHEMA.as_bytes())), SCHEMA_SHA);
    assert_eq!(hex::encode(Sha256::digest(POLICY.as_bytes())), POLICY_SHA);
    let actual_sdk = cedar_policy::get_sdk_version().to_string();
    assert_eq!(actual_sdk, "4.13.0");
    let bundle = compile(SCHEMA, POLICY).expect("adopted reference must strictly typecheck");
    let mut digest = Sha256::new();
    digest.update(SCHEMA.as_bytes());
    digest.update(POLICY.as_bytes());
    assert_eq!(bundle.key.org_id, OrgId::knl());
    assert_eq!(bundle.key.policy_version, 7);
    assert_eq!(bundle.key.schema_version, BUNDLE);
    assert_eq!(bundle.key.bundle_digest, hex::encode(digest.finalize()));
    assert_eq!(bundle.key.cedar_sdk_version, actual_sdk);
    assert_eq!(bundle.key.cedar_language_version, "4.5");
}

#[test]
fn native_hr_reference_malformed_schema_and_policy_refuse_at_parse_boundary() {
    compile(SCHEMA, POLICY).expect("clean control must compile first");
    rejected("entity Account = {", POLICY, "cedar schema parse failed:");
    rejected(SCHEMA, "permit(", "cedar policy parse failed:");
}

#[test]
fn native_hr_reference_undeclared_source_context_and_action_refuse_strict_validation() {
    compile(SCHEMA, POLICY).expect("clean control must compile first");
    for corrupted in [
        replace_exact(
            POLICY,
            "principal.state == \"ACTIVE\"",
            "principal.unreviewed_state == \"ACTIVE\"",
            2,
        ),
        replace_exact(
            POLICY,
            "resource.company_id == context.requested_company_id",
            "resource.company_id == context.unreviewed_company_id",
            2,
        ),
        replace_exact(
            POLICY,
            "action == Action::\"hr.policy.read\"",
            "action == Action::\"hr.policy.unreviewed\"",
            1,
        ),
    ] {
        rejected(SCHEMA, &corrupted, "cedar bundle failed strict validation:");
    }
}

#[test]
fn native_hr_reference_declared_attribute_type_corruption_refuses_strict_validation() {
    compile(SCHEMA, POLICY).expect("clean control must compile first");
    let schema = replace_exact(
        SCHEMA,
        "\"root_read_policy_allowed\": Bool",
        "\"root_read_policy_allowed\": String",
        1,
    );
    rejected(&schema, POLICY, "cedar bundle failed strict validation:");
}
