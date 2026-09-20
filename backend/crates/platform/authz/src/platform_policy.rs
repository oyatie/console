//! Finite Platform decisions over verified, currently guarded owner facts.
//! These inputs are facts, never credentials or reusable database grants.
use cedar_policy::{Authorizer, Context, Decision, Entities, EntityUid, Request};
use console_kernel_core::{OrgId, UserId};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{panic::AssertUnwindSafe, str::FromStr};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::{
    PlatformFeature, Role,
    cedar_pbac::engine::{CompiledBundle, compile_bundle_from_sources},
};

const POLICY_ID: &str = "legacy-platform-authority-v1";
const SCHEMA_ID: &str = "legacy-platform-subject-resource-v1";
const SCHEMA_SOURCE: &str = include_str!("platform_policy/legacy-platform-v1.cedarschema");
const POLICY_SOURCE: &str = include_str!("platform_policy/legacy-platform-v1.cedar");

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PlatformSourceKind {
    Direct,
    ViewAs,
    WritableContext,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PlatformTarget {
    Companies,
    Company(OrgId),
    Groups,
    Group(Uuid),
    Operations,
    PlatformAudit,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PlatformUse {
    ReadProjection,
    Effect,
    ContextMint,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PlatformCallSite {
    DirectPlatformOwner,
    DerivedPlatformSourceCheck,
}

pub struct PlatformCredentialFacts {
    pub subject: UserId,
    pub home: OrgId,
    pub source_kind: PlatformSourceKind,
    pub source_company: Option<OrgId>,
    pub iat: i64,
    pub nbf: i64,
    pub exp: i64,
    pub binding: Option<PlatformBindingFacts>,
}
pub struct PlatformBindingFacts {
    pub version: u8,
    pub family_id: Uuid,
    pub home: OrgId,
    pub kind: PlatformSourceKind,
}
pub struct PlatformCurrentFacts {
    pub subject: UserId,
    pub home: OrgId,
    pub roles: Option<Vec<String>>,
    pub active: Option<bool>,
    pub account_fenced: Option<bool>,
}
pub struct PlatformFamilyFacts {
    pub id: Uuid,
    pub user_id: UserId,
    pub org_id: Option<OrgId>,
    pub protocol: String,
    pub created_at: OffsetDateTime,
    pub revoked_at: Option<OffsetDateTime>,
    pub account_security_generation: Option<i64>,
    pub auth_time: Option<OffsetDateTime>,
    pub assurance: Option<String>,
}
pub struct PlatformPolicyInput {
    pub credential: PlatformCredentialFacts,
    pub current: Option<PlatformCurrentFacts>,
    pub family: Option<PlatformFamilyFacts>,
    pub absolute_family_ttl: Duration,
    pub now: OffsetDateTime,
    pub feature: PlatformFeature,
    pub target: PlatformTarget,
    pub use_kind: PlatformUse,
    pub call_site: PlatformCallSite,
}
pub enum PlatformPolicyDecision {
    Allow { policy_identity: &'static str },
    Deny { reason: PlatformPolicyDenyReason },
    UpgradeRequired,
}
pub enum PlatformPolicyDenyReason {
    InvalidMaterial,
    PolicyDenied,
    EvaluatorUnavailable,
}
#[derive(Debug)]
pub struct PlatformPolicyCompileError;

pub struct PlatformPolicy {
    bundle: CompiledBundle,
    #[cfg(test)]
    test_sdk_modes: std::sync::Mutex<Vec<&'static str>>,
    #[cfg(test)]
    test_panic_at_authorizer: bool,
}

impl PlatformPolicy {
    pub fn compile_current() -> Result<Self, PlatformPolicyCompileError> {
        std::panic::catch_unwind(|| {
            let bundle = compile_bundle_from_sources(
                OrgId::platform(),
                1,
                SCHEMA_ID,
                SCHEMA_SOURCE,
                POLICY_SOURCE,
            )
            .map_err(|_| PlatformPolicyCompileError)?;
            let policy = Self {
                bundle,
                #[cfg(test)]
                test_sdk_modes: std::sync::Mutex::new(Vec::new()),
                #[cfg(test)]
                test_panic_at_authorizer: false,
            };
            if !policy.valid_identity() {
                return Err(PlatformPolicyCompileError);
            }
            Ok(policy)
        })
        .map_err(|_| PlatformPolicyCompileError)?
    }

    fn valid_identity(&self) -> bool {
        let key = &self.bundle.key;
        let mut digest = Sha256::new();
        digest.update(SCHEMA_SOURCE.as_bytes());
        digest.update(POLICY_SOURCE.as_bytes());
        key.org_id == OrgId::platform()
            && key.policy_version == 1
            && key.schema_version == SCHEMA_ID
            && key.cedar_sdk_version == "4.12.0"
            && key.cedar_language_version == "4.5"
            && cedar_policy::get_sdk_version().to_string() == "4.12.0"
            && key.bundle_digest == hex::encode(digest.finalize())
    }

    pub fn evaluate_current(&self, input: &PlatformPolicyInput) -> PlatformPolicyDecision {
        use PlatformPolicyDecision::{Allow, Deny, UpgradeRequired};
        use PlatformPolicyDenyReason::{EvaluatorUnavailable, InvalidMaterial, PolicyDenied};
        match std::panic::catch_unwind(AssertUnwindSafe(|| {
            if !self.valid_identity() || !valid_material(input) {
                return Deny {
                    reason: InvalidMaterial,
                };
            }
            match self.evaluate_sdk(input) {
                Ok(true)
                    if input.credential.binding.is_none()
                        && input.use_kind != PlatformUse::ReadProjection =>
                {
                    UpgradeRequired
                }
                Ok(true) => Allow {
                    policy_identity: POLICY_ID,
                },
                Ok(false) => Deny {
                    reason: PolicyDenied,
                },
                Err(()) => Deny {
                    reason: EvaluatorUnavailable,
                },
            }
        })) {
            Ok(decision) => decision,
            Err(_) => Deny {
                reason: EvaluatorUnavailable,
            },
        }
    }

    fn evaluate_sdk(&self, input: &PlatformPolicyInput) -> Result<bool, ()> {
        let current = input.current.as_ref().ok_or(())?;
        let (kind, key) = match input.target {
            PlatformTarget::Companies => ("companies", "companies".to_owned()),
            PlatformTarget::Company(id) => ("company", id.to_string()),
            PlatformTarget::Groups => ("groups", "groups".to_owned()),
            PlatformTarget::Group(id) => ("group", id.to_string()),
            PlatformTarget::Operations => ("operations", "operations".to_owned()),
            PlatformTarget::PlatformAudit => ("platform_audit", "platform_audit".to_owned()),
        };
        let action = match input.feature {
            PlatformFeature::TenantCreate => "tenant_create",
            PlatformFeature::TenantList => "tenant_list",
            PlatformFeature::TenantSuspend => "tenant_suspend",
            PlatformFeature::TenantRemove => "tenant_remove",
            PlatformFeature::TenantHealthRead => "tenant_health_read",
            PlatformFeature::TenantManage => "tenant_manage",
            PlatformFeature::GroupManage => "group_manage",
            PlatformFeature::PlatformAuditRead => "platform_audit_read",
        };
        let subject_uid = EntityUid::from_str(&format!(
            "ConsoleLegacyPlatformV1::Subject::\"{}\"",
            input.credential.subject
        ))
        .map_err(|_| ())?;
        let action_uid =
            EntityUid::from_str(&format!("ConsoleLegacyPlatformV1::Action::\"{action}\""))
                .map_err(|_| ())?;
        let resource_uid =
            EntityUid::from_str(&format!("ConsoleLegacyPlatformV1::Resource::\"{key}\""))
                .map_err(|_| ())?;
        let entities = Entities::from_json_value(json!([
            {"uid": {"type":"ConsoleLegacyPlatformV1::Subject", "id":input.credential.subject.to_string()},
             "attrs":{"home_org":current.home.to_string(),"roles":current.roles,"active":current.active,"account_fenced":current.account_fenced},"parents":[]},
            {"uid":{"type":"ConsoleLegacyPlatformV1::Resource","id":key},
             "attrs":{"platform_scope":"legacy_platform","kind":kind,"key":key},"parents":[]}
        ]), Some(&self.bundle.schema)).map_err(|_| ())?;
        let session_mode = if input.credential.binding.is_some() {
            "bound"
        } else {
            "historical_unbound"
        };
        let context = Context::from_json_value(json!({
            "policy_version": POLICY_ID, "session_mode": session_mode,
            "source_kind": match input.credential.source_kind { PlatformSourceKind::Direct => "direct", PlatformSourceKind::ViewAs => "platform_view_as", PlatformSourceKind::WritableContext => "platform_tenant_context" },
            "use_kind": match input.use_kind { PlatformUse::ReadProjection => "read", PlatformUse::Effect => "effect", PlatformUse::ContextMint => "mint" },
            "call_site": match input.call_site { PlatformCallSite::DirectPlatformOwner => "platform_route", PlatformCallSite::DerivedPlatformSourceCheck => "derived_source" },
            "source_company": input.credential.source_company.map(|id| id.to_string()).unwrap_or_default()
        }), Some((&self.bundle.schema, &action_uid))).map_err(|_| ())?;
        let request = Request::new(
            subject_uid,
            action_uid,
            resource_uid,
            context,
            Some(&self.bundle.schema),
        )
        .map_err(|_| ())?;
        #[cfg(test)]
        {
            self.test_sdk_modes
                .lock()
                .map_err(|_| ())?
                .push(session_mode);
            #[allow(clippy::panic)]
            if self.test_panic_at_authorizer {
                panic!("test authorizer boundary");
            }
        }
        let response = Authorizer::new().is_authorized(&request, &self.bundle.policies, &entities);
        if response.diagnostics().errors().next().is_some() {
            return Err(());
        }
        Ok(response.decision() == Decision::Allow)
    }
}

fn valid_company(id: OrgId) -> bool {
    !id.as_uuid().is_nil() && id != OrgId::platform()
}

fn valid_material(input: &PlatformPolicyInput) -> bool {
    let c = &input.credential;
    let Some(current) = &input.current else {
        return false;
    };
    let Some(roles) = &current.roles else {
        return false;
    };
    if c.subject.as_uuid().is_nil()
        || c.home != OrgId::platform()
        || current.subject != c.subject
        || current.home != c.home
        || current.active != Some(true)
        || current.account_fenced != Some(false)
        || roles.is_empty()
        || roles
            .iter()
            .any(|role| !Role::ALL.iter().any(|known| known.as_str() == role))
        || OffsetDateTime::from_unix_timestamp(c.exp).map_or(true, |exp| exp <= input.now)
    {
        return false;
    }
    match (c.source_kind, c.source_company, input.call_site) {
        (PlatformSourceKind::Direct, None, PlatformCallSite::DirectPlatformOwner) => (),
        (
            PlatformSourceKind::ViewAs | PlatformSourceKind::WritableContext,
            Some(company),
            PlatformCallSite::DerivedPlatformSourceCheck,
        ) if valid_company(company) && input.target == PlatformTarget::Company(company) => (),
        _ => return false,
    }
    match input.target {
        PlatformTarget::Company(id) if !valid_company(id) => return false,
        PlatformTarget::Group(id) if id.is_nil() => return false,
        _ => (),
    }
    // Validate the finite operation shape before asking Cedar for role policy.
    let operation_valid = matches!(
        (c.source_kind, input.feature, input.target, input.use_kind),
        (
            PlatformSourceKind::Direct,
            PlatformFeature::TenantCreate,
            PlatformTarget::Companies,
            PlatformUse::Effect,
        ) | (
            PlatformSourceKind::Direct,
            PlatformFeature::TenantList,
            PlatformTarget::Companies,
            PlatformUse::ReadProjection,
        ) | (
            PlatformSourceKind::Direct,
            PlatformFeature::TenantSuspend | PlatformFeature::TenantRemove,
            PlatformTarget::Company(_),
            PlatformUse::Effect,
        ) | (
            PlatformSourceKind::Direct,
            PlatformFeature::TenantHealthRead,
            PlatformTarget::Company(_),
            PlatformUse::ReadProjection | PlatformUse::ContextMint,
        ) | (
            PlatformSourceKind::Direct,
            PlatformFeature::TenantHealthRead,
            PlatformTarget::Operations,
            PlatformUse::ReadProjection | PlatformUse::Effect,
        ) | (
            PlatformSourceKind::Direct,
            PlatformFeature::TenantManage,
            PlatformTarget::Company(_),
            PlatformUse::ContextMint,
        ) | (
            PlatformSourceKind::Direct,
            PlatformFeature::TenantManage,
            PlatformTarget::Operations,
            PlatformUse::Effect,
        ) | (
            PlatformSourceKind::Direct,
            PlatformFeature::GroupManage,
            PlatformTarget::Groups | PlatformTarget::Group(_) | PlatformTarget::Company(_),
            PlatformUse::ReadProjection | PlatformUse::Effect,
        ) | (
            PlatformSourceKind::Direct,
            PlatformFeature::PlatformAuditRead,
            PlatformTarget::PlatformAudit,
            PlatformUse::ReadProjection,
        ) | (
            PlatformSourceKind::ViewAs,
            PlatformFeature::TenantHealthRead,
            PlatformTarget::Company(_),
            PlatformUse::ReadProjection,
        ) | (
            PlatformSourceKind::WritableContext,
            PlatformFeature::TenantManage,
            PlatformTarget::Company(_),
            PlatformUse::ReadProjection | PlatformUse::Effect,
        )
    );
    if !operation_valid {
        return false;
    }
    match (&c.binding, &input.family) {
        (None, None) => true,
        (Some(b), Some(f)) => {
            let Ok(iat) = OffsetDateTime::from_unix_timestamp(c.iat) else {
                return false;
            };
            let Ok(nbf) = OffsetDateTime::from_unix_timestamp(c.nbf) else {
                return false;
            };
            b.version == 1
                && !b.family_id.is_nil()
                && b.home == c.home
                && b.kind == c.source_kind
                && f.id == b.family_id
                && f.user_id == c.subject
                && f.org_id == Some(c.home)
                && f.protocol == "LEGACY_COMPANY"
                && f.revoked_at.is_none()
                && f.account_security_generation.is_none()
                && f.auth_time.is_none()
                && f.assurance.is_none()
                && f.created_at <= input.now
                && f.created_at.unix_timestamp() <= c.iat
                && input.absolute_family_ttl > Duration::ZERO
                && f.created_at
                    .checked_add(input.absolute_family_ttl)
                    .is_some_and(|deadline| deadline > input.now)
                && iat <= nbf
                && c.nbf < c.exp
                && iat <= input.now
                && nbf <= input.now
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests;
