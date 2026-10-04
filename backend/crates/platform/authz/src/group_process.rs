//! Strict Account/Group Cedar evaluation from retained, checked owner material.
//!
//! Compiled source identity is separate from the actual business policy head.
//! Own receipts therefore need neither a current designation nor a live process.
use std::{panic::AssertUnwindSafe, str::FromStr};

use cedar_policy::{
    Authorizer, Context, Decision, Entities, EntityUid, PolicySet, Request, Schema,
};
use console_identity_application::group_process::{
    CurrentGroupProcessAuthority, EvaluatedPolicyBundleV1, GroupId, GroupIncarnation,
    GroupProcessActionV1, GroupProcessDecision, GroupProcessDecisionPort, GroupProcessError,
    GroupProcessPolicyRequestV1,
};
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::cedar_pbac::engine::{
    CEDAR_LANGUAGE_VERSION, CEDAR_SDK_VERSION, validate_strict_sources,
};

const SCHEMA_ID: &str = "native-group-process-v1";
const SCHEMA: &str = include_str!("group_process/process-v1.cedarschema");
const POLICY: &str = include_str!("group_process/process-v1.cedar");

/// Source-only compiled identity. It never caches a decision, carries a scope
/// mode or proof, or substitutes an OrgId for the actual Group/incarnation.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CompiledGroupProcessBundleKey {
    group: GroupId,
    incarnation: GroupIncarnation,
    schema_id: String,
    schema_version: u64,
    schema_digest: [u8; 32],
    policy_version: u64,
    policy_digest: [u8; 32],
    codec_contract_digest: [u8; 32],
    action_manifest_version: u64,
    action_manifest_digest: [u8; 32],
    cedar_sdk_version: String,
    cedar_language_version: String,
}

impl CompiledGroupProcessBundleKey {
    fn checked(
        group: GroupId,
        incarnation: GroupIncarnation,
        source: &EvaluatedPolicyBundleV1,
    ) -> Result<Self, GroupProcessError> {
        sdk_identity()?;
        let schema_digest = digest(SCHEMA);
        let policy_digest = digest(POLICY);
        if source.schema_id() != SCHEMA_ID
            || source.schema_digest() != &schema_digest
            || source.policy_digest() != &policy_digest
            || source.registration_manifest_version() != 1
            || source.cedar_sdk_version() != CEDAR_SDK_VERSION
            || source.cedar_language_version() != CEDAR_LANGUAGE_VERSION
        {
            return Err(GroupProcessError::Unavailable);
        }
        // Codec and actual action-manifest identities come from the verified
        // protected source. Proposal hashes and generated UUIDs are not custody.
        Ok(Self {
            group,
            incarnation,
            schema_id: source.schema_id().to_owned(),
            schema_version: 1,
            schema_digest,
            policy_version: 1,
            policy_digest,
            codec_contract_digest: *source.codec_contract_digest(),
            action_manifest_version: source.registration_manifest_version(),
            action_manifest_digest: *source.registration_manifest_digest(),
            cedar_sdk_version: source.cedar_sdk_version().to_owned(),
            cedar_language_version: source.cedar_language_version().to_owned(),
        })
    }

    pub const fn group(&self) -> GroupId {
        self.group
    }

    pub const fn incarnation(&self) -> GroupIncarnation {
        self.incarnation
    }

    pub fn schema_id(&self) -> &str {
        &self.schema_id
    }

    pub const fn schema_version(&self) -> u64 {
        self.schema_version
    }

    pub const fn schema_digest(&self) -> &[u8; 32] {
        &self.schema_digest
    }

    pub const fn policy_version(&self) -> u64 {
        self.policy_version
    }

    pub const fn policy_digest(&self) -> &[u8; 32] {
        &self.policy_digest
    }

    pub const fn codec_contract_digest(&self) -> &[u8; 32] {
        &self.codec_contract_digest
    }

    pub const fn action_manifest_version(&self) -> u64 {
        self.action_manifest_version
    }

    pub const fn action_manifest_digest(&self) -> &[u8; 32] {
        &self.action_manifest_digest
    }

    pub fn cedar_sdk_version(&self) -> &str {
        &self.cedar_sdk_version
    }

    pub fn cedar_language_version(&self) -> &str {
        &self.cedar_language_version
    }
}

struct CompiledGroupProcessBundle {
    key: CompiledGroupProcessBundleKey,
    schema: Schema,
    policies: PolicySet,
}

impl CompiledGroupProcessBundle {
    fn compile(authority: &CurrentGroupProcessAuthority) -> Result<Self, GroupProcessError> {
        let key = CompiledGroupProcessBundleKey::checked(
            authority.group(),
            authority.incarnation(),
            authority.evaluated_bundle(),
        )?;
        let (schema, policies) = strict_sources()?;
        Ok(Self {
            key,
            schema,
            policies,
        })
    }
}

/// The Group owner supplies new current material for every decision and again
/// at consuming finish. This evaluator holds no authority or decision cache.
pub struct GroupProcessPolicy;

impl GroupProcessPolicy {
    pub fn new() -> Result<Self, GroupProcessError> {
        with_cedar_stack(|| {
            sdk_identity()?;
            strict_sources()?;
            Ok(Self)
        })
    }

    fn evaluate(
        &self,
        authority: &CurrentGroupProcessAuthority,
        requested: &GroupProcessPolicyRequestV1,
    ) -> Result<GroupProcessDecision, GroupProcessError> {
        use GroupProcessError::Unavailable;
        let bundle = CompiledGroupProcessBundle::compile(authority)?;
        if requested != authority.policy_request() {
            return Ok(GroupProcessDecision::Deny);
        }
        let group = bundle.key.group().as_uuid().to_string();
        let incarnation = bundle.key.incarnation().as_uuid().to_string();
        let account = authority.account().as_uuid().to_string();
        let resource = format!("{group}/{incarnation}");
        let kind = authority.resource_kind().as_str();
        let manifest = hex::encode(bundle.key.action_manifest_digest());
        let allowed_fields: Vec<_> = authority
            .allowed_fields(requested.action())
            .iter()
            .map(|field| field.as_str())
            .collect();
        let requested_fields: Vec<_> = requested
            .requested_fields()
            .iter()
            .map(|field| field.as_str())
            .collect();
        let mut principal_attrs =
            json!({"account_id": account, "state": authority.account_state()});
        let mut resource_attrs = json!({
            "group_id": group, "incarnation": incarnation, "resource_kind": kind,
            "manifest_digest": manifest, "allowed_fields": allowed_fields
        });
        let mut context = json!({
            "group_id": group, "incarnation": incarnation, "resource_kind": kind,
            "manifest_digest": manifest, "requested_fields": requested_fields
        });
        match (authority.current_material(), authority.receipt_actor()) {
            (Some(current), _) => {
                let policy = authority.policy_head().ok_or(Unavailable)?.state_str();
                let stage = authority.bootstrap_stage().ok_or(Unavailable)?.as_str();
                let deployment = current.deployment();
                let group_deployment = current.group_deployment();
                principal_attrs["designation_account_id"] =
                    json!(current.designation_account().as_uuid().to_string());
                principal_attrs["designation_state"] = json!(current.designation_state());
                principal_attrs["system_identifier"] = json!(deployment.system_identifier());
                principal_attrs["database_name"] = json!(deployment.database_name());
                principal_attrs["database_oid"] = json!(deployment.database_oid().to_string());
                resource_attrs["group_state"] = json!(current.group_state());
                resource_attrs["system_identifier"] = json!(group_deployment.system_identifier());
                resource_attrs["database_name"] = json!(group_deployment.database_name());
                resource_attrs["database_oid"] = json!(group_deployment.database_oid().to_string());
                resource_attrs["policy_state"] = json!(policy);
                resource_attrs["bootstrap_stage"] = json!(stage);
                context["policy_state"] = json!(policy);
                context["bootstrap_stage"] = json!(stage);
            }
            (None, Some(actor)) => {
                if requested.action() != GroupProcessActionV1::ReadOwnReceipt {
                    return Ok(GroupProcessDecision::Deny);
                }
                // Own receipt material genuinely omits every current-only
                // attribute. No invented designation or policy-head sentinel.
                resource_attrs["receipt_actor"] = json!(actor.as_uuid().to_string());
            }
            _ => return Err(Unavailable),
        }
        let entities = Entities::from_json_value(
            json!([
                {"uid":{"type":"Account","id":account},"attrs":principal_attrs,"parents":[]},
                {"uid":{"type":"GroupIdentityResource","id":resource},"attrs":resource_attrs,"parents":[]}
            ]),
            Some(&bundle.schema),
        )
        .map_err(|_| Unavailable)?;
        let principal = uid("Account", &account)?;
        let action = uid("Action", requested.action().as_str())?;
        let resource = uid("GroupIdentityResource", &resource)?;
        let context = Context::from_json_value(context, Some((&bundle.schema, &action)))
            .map_err(|_| Unavailable)?;
        let request = Request::new(principal, action, resource, context, Some(&bundle.schema))
            .map_err(|_| Unavailable)?;
        let response = Authorizer::new().is_authorized(&request, &bundle.policies, &entities);
        if response.diagnostics().errors().next().is_some() {
            return Err(Unavailable);
        }
        Ok(match response.decision() {
            Decision::Allow => GroupProcessDecision::Allow,
            Decision::Deny => GroupProcessDecision::Deny,
        })
    }
}

impl GroupProcessDecisionPort for GroupProcessPolicy {
    fn decide(
        &self,
        authority: &CurrentGroupProcessAuthority,
        request: &GroupProcessPolicyRequestV1,
    ) -> Result<GroupProcessDecision, GroupProcessError> {
        with_cedar_stack(|| self.evaluate(authority, request))
    }
}

fn strict_sources() -> Result<(Schema, PolicySet), GroupProcessError> {
    let schema = Schema::from_json_str(SCHEMA).map_err(|_| GroupProcessError::Unavailable)?;
    let policies = PolicySet::from_str(POLICY).map_err(|_| GroupProcessError::Unavailable)?;
    validate_strict_sources(&schema, &policies).map_err(|_| GroupProcessError::Unavailable)?;
    Ok((schema, policies))
}

fn sdk_identity() -> Result<(), GroupProcessError> {
    if CEDAR_LANGUAGE_VERSION != "4.5"
        || cedar_policy::get_sdk_version().to_string() != CEDAR_SDK_VERSION
    {
        return Err(GroupProcessError::Unavailable);
    }
    Ok(())
}

fn uid(kind: &str, value: &str) -> Result<EntityUid, GroupProcessError> {
    let value = serde_json::to_string(value).map_err(|_| GroupProcessError::Unavailable)?;
    EntityUid::from_str(&format!("{kind}::{value}")).map_err(|_| GroupProcessError::Unavailable)
}

fn digest(source: &str) -> [u8; 32] {
    Sha256::digest(source.as_bytes()).into()
}

fn with_cedar_stack<T>(
    evaluate: impl FnOnce() -> Result<T, GroupProcessError>,
) -> Result<T, GroupProcessError> {
    const STACK_BYTES: usize = 2 * 1024 * 1024;
    std::panic::catch_unwind(AssertUnwindSafe(|| {
        stacker::maybe_grow(STACK_BYTES, STACK_BYTES, evaluate)
    }))
    .map_err(|_| GroupProcessError::Unavailable)?
}
