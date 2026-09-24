//! Finite initial Company grants evaluated by the existing strict Cedar engine.
use cedar_policy::{Authorizer, Context, Decision, Entities, EntityUid, Request};
use console_identity_application::company_policy::{
    CompanyPolicyDecision, CompanyPolicyDecisionPort, CompanyPolicyError, CompanyPolicyRequest,
    CurrentCompanyAuthority, CurrentNativeBootstrapAuthority, CurrentPayrollReadAuthority,
    CurrentPeopleDirectoryAuthority, InitialCompanyAction, NativeBootstrapRequestV1,
    NativePeopleDirectoryRequestV1, PropertyRef,
};
use console_kernel_core::OrgId;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{panic::AssertUnwindSafe, str::FromStr};

use crate::cedar_pbac::engine::{
    CEDAR_LANGUAGE_VERSION, CEDAR_SDK_VERSION, compile_bundle_from_sources,
};

const SCHEMA_ID: &str = "native-company-authorization-2026-09-19.1";
const SCHEMA: &str = include_str!("company_policy/native-company-authorization.cedarschema");
mod native_business;
mod people_directory;

pub struct CompanyPolicy {
    #[cfg(test)]
    sdk_calls: std::sync::atomic::AtomicUsize,
    #[cfg(test)]
    panic_at_authorizer: bool,
}

impl CompanyPolicy {
    pub fn new() -> Result<Self, CompanyPolicyError> {
        with_cedar_stack(|| {
            sdk_identity()?;
            native_business::validate_bundles()?;
            people_directory::validate_bundle()?;
            // Validate the fixed schema at composition, before any request.
            compile_bundle_from_sources(
                OrgId::platform(),
                1,
                SCHEMA_ID,
                SCHEMA,
                "forbid(principal, action, resource);",
            )
            .map_err(|_| CompanyPolicyError::EvaluatorUnavailable)?;
            Ok(Self {
                #[cfg(test)]
                sdk_calls: std::sync::atomic::AtomicUsize::new(0),
                #[cfg(test)]
                panic_at_authorizer: false,
            })
        })
    }

    fn evaluate(
        &self,
        authority: &CurrentCompanyAuthority,
        request: &CompanyPolicyRequest,
    ) -> Result<CompanyPolicyDecision, CompanyPolicyError> {
        use CompanyPolicyError::EvaluatorUnavailable;
        sdk_identity()?;
        let Some(selected) = authority
            .clauses()
            .iter()
            .find(|clause| !clause.delegable() && clause.action() == request.action())
        else {
            return Ok(CompanyPolicyDecision::Deny);
        };
        let account = authority.account().as_uuid().to_string();
        let company = authority.company().to_string();
        let mut source = String::new();
        for (index, clause) in authority
            .clauses()
            .iter()
            .enumerate()
            .filter(|(_, c)| !c.delegable())
        {
            let object_type = clause.action().object_type_id().to_string();
            let object = match clause.action_kind() {
                InitialCompanyAction::Discover | InitialCompanyAction::ReadIdentity => {
                    *authority.company().as_uuid()
                }
                _ => authority.assignment_id(),
            };
            let resource = format!("{company}/{object_type}/{object}");
            let fields: Vec<_> = clause
                .fields()
                .iter()
                .map(|field| field_key(field, clause.action().manifest_digest()))
                .collect();
            // Strict Cedar cannot infer the type of an empty set literal.
            let field_condition = if fields.is_empty() {
                "context.requested_fields.isEmpty()".to_owned()
            } else {
                format!(
                    "{}.containsAll(context.requested_fields)",
                    serde_json::to_string(&fields).map_err(|_| EvaluatorUnavailable)?
                )
            };
            // SDK-local policy IDs are source ordered; retain the logical
            // assignment/revision/clause identity explicitly in the source.
            source.push_str(&format!(
                "// {}/{}/{}\npermit(principal == Account::{}, action == Action::{}, resource == CompanyResource::{}) when {{ resource.org_id == {} && resource.object_type_id == {} && {} }};\n",
                authority.assignment_id(),authority.assignment_revision(),index+1,
                quote(&account)?,quote(clause.action_kind().as_str())?,quote(&resource)?,
                quote(&company)?,quote(&object_type)?,field_condition));
        }
        let bundle = compile_bundle_from_sources(
            authority.company(),
            authority.epoch(),
            SCHEMA_ID,
            SCHEMA,
            &source,
        )
        .map_err(|_| EvaluatorUnavailable)?;
        let mut digest = Sha256::new();
        digest.update(SCHEMA);
        digest.update(&source);
        if bundle.key.org_id != authority.company()
            || bundle.key.policy_version != authority.epoch()
            || bundle.key.schema_version != SCHEMA_ID
            || bundle.key.cedar_sdk_version != CEDAR_SDK_VERSION
            || bundle.key.cedar_language_version != CEDAR_LANGUAGE_VERSION
            || bundle.key.bundle_digest != hex::encode(digest.finalize())
        {
            return Err(EvaluatorUnavailable);
        }
        let requested_company = request.requested_company().to_string();
        let requested_type = request.object_type_id().to_string();
        let resource = format!(
            "{requested_company}/{requested_type}/{}",
            request.object_id()
        );
        let principal_uid = EntityUid::from_str(&format!("Account::{}", quote(&account)?))
            .map_err(|_| EvaluatorUnavailable)?;
        let action_uid = EntityUid::from_str(&format!(
            "Action::{}",
            quote(selected.action_kind().as_str())?
        ))
        .map_err(|_| EvaluatorUnavailable)?;
        let resource_uid = EntityUid::from_str(&format!("CompanyResource::{}", quote(&resource)?))
            .map_err(|_| EvaluatorUnavailable)?;
        let entities = Entities::from_json_value(json!([
            {"uid":{"type":"Account","id":account},"attrs":{},"parents":[]},
            {"uid":{"type":"CompanyResource","id":resource},"attrs":{"org_id":requested_company,"object_type_id":requested_type},"parents":[]}
        ]),Some(&bundle.schema)).map_err(|_| EvaluatorUnavailable)?;
        let fields: Vec<_> = request
            .requested_properties()
            .iter()
            .map(|field| field_key(field, request.action().manifest_digest()))
            .collect();
        let context = Context::from_json_value(
            json!({"requested_fields":fields}),
            Some((&bundle.schema, &action_uid)),
        )
        .map_err(|_| EvaluatorUnavailable)?;
        let request = Request::new(
            principal_uid,
            action_uid,
            resource_uid,
            context,
            Some(&bundle.schema),
        )
        .map_err(|_| EvaluatorUnavailable)?;
        self.authorize(&request, &bundle, &entities)
    }

    fn authorize(
        &self,
        request: &Request,
        bundle: &crate::cedar_pbac::engine::CompiledBundle,
        entities: &Entities,
    ) -> Result<CompanyPolicyDecision, CompanyPolicyError> {
        use CompanyPolicyError::EvaluatorUnavailable;
        #[cfg(test)]
        {
            self.sdk_calls
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            #[allow(clippy::panic)]
            if self.panic_at_authorizer {
                panic!("test authorizer boundary");
            }
        }
        let response = Authorizer::new().is_authorized(request, &bundle.policies, entities);
        if response.diagnostics().errors().next().is_some() {
            return Err(EvaluatorUnavailable);
        }
        Ok(if response.decision() == Decision::Allow {
            CompanyPolicyDecision::Allow
        } else {
            CompanyPolicyDecision::Deny
        })
    }
}

impl CompanyPolicyDecisionPort for CompanyPolicy {
    fn decide_native_people_directory(
        &self,
        authority: &CurrentPeopleDirectoryAuthority,
        request: &NativePeopleDirectoryRequestV1,
    ) -> Result<CompanyPolicyDecision, CompanyPolicyError> {
        with_cedar_stack(|| self.evaluate_native_people_directory(authority, request))
    }

    fn decide_native_bootstrap(
        &self,
        authority: &CurrentNativeBootstrapAuthority,
        request: &NativeBootstrapRequestV1,
    ) -> Result<CompanyPolicyDecision, CompanyPolicyError> {
        with_cedar_stack(|| self.evaluate_native_bootstrap(authority, request))
    }

    fn decide_native_payroll_collection(
        &self,
        authority: &CurrentPayrollReadAuthority,
    ) -> Result<CompanyPolicyDecision, CompanyPolicyError> {
        with_cedar_stack(|| self.evaluate_native_payroll(authority))
    }

    fn decide(
        &self,
        authority: &CurrentCompanyAuthority,
        request: &CompanyPolicyRequest,
    ) -> Result<CompanyPolicyDecision, CompanyPolicyError> {
        with_cedar_stack(|| self.evaluate(authority, request))
    }
}

// Deep HTTP frames can exhaust Cedar's remaining-stack budget. Reserve one
// bounded, same-thread segment per entry; Cedar's own recursion guard remains.
fn with_cedar_stack<T>(
    evaluate: impl FnOnce() -> Result<T, CompanyPolicyError>,
) -> Result<T, CompanyPolicyError> {
    const STACK_BYTES: usize = 2 * 1024 * 1024;
    std::panic::catch_unwind(AssertUnwindSafe(|| {
        stacker::maybe_grow(STACK_BYTES, STACK_BYTES, evaluate)
    }))
    .map_err(|_| CompanyPolicyError::EvaluatorUnavailable)?
}

fn sdk_identity() -> Result<(), CompanyPolicyError> {
    if CEDAR_LANGUAGE_VERSION != "4.5"
        || cedar_policy::get_sdk_version().to_string() != CEDAR_SDK_VERSION
    {
        return Err(CompanyPolicyError::EvaluatorUnavailable);
    }
    Ok(())
}
fn quote(value: &str) -> Result<String, CompanyPolicyError> {
    serde_json::to_string(value).map_err(|_| CompanyPolicyError::EvaluatorUnavailable)
}
fn field_key(field: &PropertyRef, digest: &[u8; 32]) -> String {
    format!(
        "{}/{}/{}/{}/{}",
        field.org_id(),
        field.object_type_id(),
        field.property_id(),
        field.schema_revision(),
        hex::encode(digest)
    )
}

#[cfg(test)]
#[path = "native_company_policy_unit_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "native_business_policy_unit_tests.rs"]
mod native_business_tests;

#[cfg(test)]
#[path = "native_people_directory_policy_tests.rs"]
mod native_people_directory_tests;
