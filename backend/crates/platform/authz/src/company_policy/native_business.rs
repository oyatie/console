//! Finite native maintenance and payroll-use policies; no inherited role grants.
use super::*;
use crate::cedar_pbac::engine::CompiledBundle;
use console_identity_application::company_policy::{
    CurrentNativeBootstrapAuthority, CurrentPayrollReadAuthority, NativeBootstrapRequestV1,
    business::NativeBusinessOperationV1,
};

const BOOT_ID: &str = "native-business-bootstrap-v1";
const BOOT_SCHEMA: &str = include_str!("native-business-bootstrap-v1.cedarschema");
const BOOT_POLICY: &str = include_str!("native-business-bootstrap-v1.cedar");
const READ_ID: &str = "native-payroll-collection-read-v1";
const READ_SCHEMA: &str = include_str!("native-payroll-collection-read-v1.cedarschema");
const READ_POLICY: &str = include_str!("native-payroll-collection-read-v1.cedar");
const MANIFEST: &str = "07781514029d5f8f7e96221d6504387324c8c0f2513ded2b214d0bd683ce3ddd";

pub(super) fn validate_bundles() -> Result<(), CompanyPolicyError> {
    for (id, schema, policy) in [
        (BOOT_ID, BOOT_SCHEMA, BOOT_POLICY),
        (READ_ID, READ_SCHEMA, READ_POLICY),
    ] {
        compile(OrgId::platform(), 1, id, schema, policy)?;
    }
    Ok(())
}

fn compile(
    company: OrgId,
    epoch: u64,
    id: &str,
    schema: &str,
    policy: &str,
) -> Result<CompiledBundle, CompanyPolicyError> {
    use CompanyPolicyError::EvaluatorUnavailable;
    sdk_identity()?;
    let bundle = compile_bundle_from_sources(company, epoch, id, schema, policy)
        .map_err(|_| EvaluatorUnavailable)?;
    let mut digest = Sha256::new();
    digest.update(schema);
    digest.update(policy);
    if bundle.key.org_id != company
        || bundle.key.policy_version != epoch
        || bundle.key.schema_version != id
        || bundle.key.cedar_sdk_version != CEDAR_SDK_VERSION
        || bundle.key.cedar_language_version != CEDAR_LANGUAGE_VERSION
        || bundle.key.bundle_digest != hex::encode(digest.finalize())
    {
        return Err(EvaluatorUnavailable);
    }
    Ok(bundle)
}

impl CompanyPolicy {
    pub(super) fn evaluate_native_bootstrap(
        &self,
        authority: &CurrentNativeBootstrapAuthority,
        requested: &NativeBootstrapRequestV1,
    ) -> Result<CompanyPolicyDecision, CompanyPolicyError> {
        let row = authority.source();
        let bundle = compile(
            OrgId::from_uuid(row.org_id),
            row.company_epoch as u64,
            BOOT_ID,
            BOOT_SCHEMA,
            BOOT_POLICY,
        )?;
        let account = row.actor_account_id.to_string();
        let company = row.org_id.to_string();
        let operation = match requested.operation() {
            NativeBusinessOperationV1::Install => "InstallPayrollReadCatalogV1",
            NativeBusinessOperationV1::Grant => "GrantPayrollReadV1",
            NativeBusinessOperationV1::Revoke => "RevokePayrollReadV1",
        };
        // ACTIVE values come only from the canonical current-source contract;
        // HTTP selectors never supply entity attributes or source state.
        let entities = json!([
            {"uid":{"type":"Account","id":account},"parents":[],"attrs":{
                "account_id":account,"session_id":row.session_id.to_string(),
                "security_generation":row.account_security_generation.to_string(),"state":"ACTIVE",
                "deployment_system_identifier":row.designation_system_identifier,
                "deployment_database_name":row.designation_database_name,
                "deployment_database_oid":row.designation_database_oid.to_string(),
                "designation_account_id":account,"designation_revision":row.designation_revision.to_string(),
                "designation_receipt_id":row.designation_receipt_id.to_string(),"designation_state":"ACTIVE"}},
            {"uid":{"type":"NativeCompany","id":company},"parents":[],"attrs":{
                "org_id":company,"state":"ACTIVE",
                "deployment_system_identifier":row.designation_system_identifier,
                "deployment_database_name":row.designation_database_name,
                "deployment_database_oid":row.designation_database_oid.to_string(),
                "current_group_id":row.current_group_id.to_string(),"group_state":"ACTIVE",
                "group_revision":row.group_revision.to_string(),"group_incarnation":row.group_incarnation.to_string(),
                "membership_id":row.membership_id.to_string(),"membership_state":"ACTIVE",
                "membership_revision":row.membership_revision.to_string(),"membership_incarnation":row.membership_incarnation.to_string(),
                "company_epoch":row.company_epoch.to_string(),"origin_account_id":row.origin_account_id.to_string(),
                "origin_command_id":row.origin_command_id.to_string(),"origin_receipt_id":row.origin_receipt_id.to_string(),
                "administrative_account_id":row.administrative_account_id.to_string(),
                "company_actor_admission_receipt_id":row.company_actor_admission_receipt_id.to_string(),
                "birth_assignment_id":row.birth_assignment_id.to_string(),"birth_role_id":row.birth_role_id.to_string(),
                "manifest_digest":MANIFEST}}
        ]);
        let context = json!({"requested_org_id":requested.company().to_string(),
            "requested_group_id":requested.group().to_string(),
            "recipient_account_id":requested.recipient().as_uuid().to_string(),
            "manifest_digest":hex::encode(requested.manifest_digest())});
        self.evaluate_native_request(
            &bundle,
            (&account, operation, "NativeCompany", &company),
            entities,
            context,
        )
    }

    pub(super) fn evaluate_native_payroll(
        &self,
        authority: &CurrentPayrollReadAuthority,
    ) -> Result<CompanyPolicyDecision, CompanyPolicyError> {
        let row = authority.source();
        let bundle = compile(
            OrgId::from_uuid(row.org_id),
            row.company_epoch as u64,
            READ_ID,
            READ_SCHEMA,
            READ_POLICY,
        )?;
        let account = row.account_id.to_string();
        let company = row.org_id.to_string();
        let object_type = row.action.object_type_id().to_string();
        let action_id = row.action.action_type_id().to_string();
        let resource = format!("{company}/{object_type}/collection");
        let fields: Vec<_> = row
            .properties
            .iter()
            .map(|p| field_key(p, row.action.manifest_digest()))
            .collect();
        let micros = |time: time::OffsetDateTime| {
            i64::try_from(time.unix_timestamp_nanos() / 1_000)
                .map_err(|_| CompanyPolicyError::MaterialUnavailable)
        };
        let entities = json!([
            {"uid":{"type":"Account","id":account},"parents":[],"attrs":{"account_id":account,"state":"ACTIVE"}},
            {"uid":{"type":"PayrollCollection","id":resource},"parents":[],"attrs":{
                "org_id":company,"object_type_id":object_type,"registered_action_id":action_id,
                "registration_revision":row.action.registration_revision().to_string(),
                "manifest_digest":hex::encode(row.action.manifest_digest()),"recipient_account_id":account,
                "assignment_id":row.assignment_id.to_string(),"assignment_revision":row.assignment_revision.to_string(),
                "assignment_state":row.assignment_state,"valid_from_us":micros(row.valid_from)?,"valid_until_us":micros(row.valid_until)?,
                "role_id":row.role_id.to_string(),"role_revision":row.role_revision.to_string(),"role_state":row.role_state,
                "company_epoch":row.company_epoch.to_string(),"delegable":row.delegable,"allowed_fields":fields}}
        ]);
        let context = json!({"requested_org_id":company,"object_type_id":object_type,
            "registered_action_id":action_id,"requested_fields":fields,"observed_at_us":micros(authority.observed_at())?});
        self.evaluate_native_request(
            &bundle,
            (
                &account,
                "payroll.collection.read",
                "PayrollCollection",
                &resource,
            ),
            entities,
            context,
        )
    }

    fn evaluate_native_request(
        &self,
        bundle: &CompiledBundle,
        selectors: (&str, &str, &str, &str),
        entities: serde_json::Value,
        context: serde_json::Value,
    ) -> Result<CompanyPolicyDecision, CompanyPolicyError> {
        use CompanyPolicyError::EvaluatorUnavailable;
        let (account, action, resource_type, resource) = selectors;
        let principal = EntityUid::from_str(&format!("Account::{}", quote(account)?))
            .map_err(|_| EvaluatorUnavailable)?;
        let action = EntityUid::from_str(&format!("Action::{}", quote(action)?))
            .map_err(|_| EvaluatorUnavailable)?;
        let resource = EntityUid::from_str(&format!("{resource_type}::{}", quote(resource)?))
            .map_err(|_| EvaluatorUnavailable)?;
        let entities = Entities::from_json_value(entities, Some(&bundle.schema))
            .map_err(|_| EvaluatorUnavailable)?;
        let context = Context::from_json_value(context, Some((&bundle.schema, &action)))
            .map_err(|_| EvaluatorUnavailable)?;
        let request = Request::new(principal, action, resource, context, Some(&bundle.schema))
            .map_err(|_| EvaluatorUnavailable)?;
        self.authorize(&request, bundle, &entities)
    }
}
