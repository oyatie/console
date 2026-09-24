//! Business People policy, separate from catalog/bootstrap and Payroll policies.
use super::*;
use console_identity_application::company_policy::{
    CurrentPeopleDirectoryAuthority, NativePeopleDirectoryRequestV1, NativePeopleDirectoryResource,
};
const ID: &str = "native-people-directory-business-v1";
const SCHEMA: &str = include_str!("native-people-directory-business-v1.cedarschema");
const POLICY: &str = include_str!("native-people-directory-business-v1.cedar");
pub(super) fn validate_bundle() -> Result<(), CompanyPolicyError> {
    super::native_business::compile(OrgId::platform(), 1, ID, SCHEMA, POLICY).map(|_| ())
}
impl CompanyPolicy {
    pub(super) fn evaluate_native_people_directory(
        &self,
        authority: &CurrentPeopleDirectoryAuthority,
        requested: &NativePeopleDirectoryRequestV1,
    ) -> Result<CompanyPolicyDecision, CompanyPolicyError> {
        let row = authority.source();
        let registered = authority.action_reference();
        let bundle = super::native_business::compile(
            OrgId::from_uuid(row.org_id),
            row.company_epoch as u64,
            ID,
            SCHEMA,
            POLICY,
        )?;
        let account = row.account_id.to_string();
        let company = row.org_id.to_string();
        let object_type = registered.object_type_id().to_string();
        let action_id = registered.action_type_id().to_string();
        let fields: Vec<_> = authority
            .named_properties()
            .iter()
            .map(|p| field_key(p.property(), registered.manifest_digest()))
            .collect();
        let (kind, resource_id) = match requested.resource() {
            NativePeopleDirectoryResource::Collection => ("collection", "collection".to_owned()),
            NativePeopleDirectoryResource::Entry(id) => ("entry", id.to_string()),
            NativePeopleDirectoryResource::Request(id) => ("request", id.to_string()),
        };
        // Resource UID includes requested kind/id; business grant applies to its
        // Company collection. Actual accepted-request actor checks remain owner
        // responsibilities before receipt reads; this is not an ownership proof.
        let resource = format!("{company}/{object_type}/{kind}/{resource_id}");
        let micros = |t: time::OffsetDateTime| {
            i64::try_from(t.unix_timestamp_nanos() / 1000)
                .map_err(|_| CompanyPolicyError::MaterialUnavailable)
        };
        // ACTIVE/nondelegable values are guarantees of the exact retained SQL
        // source, not client attributes. Missing/inactive sources yield no row.
        let entities = json!([
            {"uid":{"type":"Account","id":account},"parents":[],"attrs":{
                "account_id":account,"session_id":row.session_id.to_string(),
                "security_generation":row.account_security_generation.to_string(),"state":"ACTIVE"}},
            {"uid":{"type":"PeopleDirectory","id":resource},"parents":[],"attrs":{
                "org_id":company,"object_type_id":object_type,"registered_action_id":action_id,
                "registration_revision":registered.registration_revision().to_string(),"manifest_digest":hex::encode(registered.manifest_digest()),
                "granted_action":row.action.as_str(),"recipient_account_id":account,
                "assignment_id":row.assignment_id.to_string(),"assignment_revision":row.assignment_revision.to_string(),"assignment_state":"ACTIVE",
                "valid_from_us":micros(row.assignment_valid_from)?,"valid_until_us":micros(row.assignment_valid_until)?,
                "role_id":row.role_id.to_string(),"role_revision":row.role_revision.to_string(),"role_state":"ACTIVE",
                "company_epoch":row.company_epoch.to_string(),"current_policy_receipt_id":row.current_policy_receipt_id.to_string(),
                "delegable":false,"allowed_fields":fields}}
        ]);
        let context = json!({"requested_org_id":requested.company().to_string(),"requested_actor_id":requested.actor().as_uuid().to_string(),
            "object_type_id":object_type,"registered_action_id":action_id,"requested_fields":fields,
            "observed_at_us":micros(authority.observed_at())?,"resource_kind":kind,"resource_id":resource_id});
        self.evaluate_native_request(
            &bundle,
            (
                &account,
                requested.action().as_str(),
                "PeopleDirectory",
                &resource,
            ),
            entities,
            context,
        )
    }
}
