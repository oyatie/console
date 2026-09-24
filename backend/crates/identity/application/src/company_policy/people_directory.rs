//! Checked current People source. SQL owns admission/custody; no HTTP authority.
use super::native_business::{NativePolicySourceBinding, exact_micros};
use super::people_business::{DirectoryActionV1, MANIFEST};
use super::{
    AccountId, ActionRef, CompanyPolicyError, Object, PropertyRef, WireAction, WireClause,
    canonical_revision, canonical_time, canonical_uuid, company_id, nonnil,
};
use console_kernel_core::OrgId;
use serde::Deserialize;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

fn property_keys(kind: DirectoryActionV1) -> &'static [&'static str] {
    match kind {
        DirectoryActionV1::Read => &[
            "person.directory_registered_at",
            "person.employee_id",
            "person.employee_number",
            "person.legal_name",
            "person.person_id",
            "person.person_version",
        ],
        DirectoryActionV1::Create => &["person.employee_number", "person.legal_name"],
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativePeopleDirectoryResource {
    Collection,
    Entry(Uuid),
    Request(Uuid),
}
pub struct NativePeopleDirectoryRequestV1 {
    company: OrgId,
    actor: AccountId,
    action: DirectoryActionV1,
    resource: NativePeopleDirectoryResource,
}
impl NativePeopleDirectoryRequestV1 {
    pub fn new(
        company: OrgId,
        actor: AccountId,
        action: DirectoryActionV1,
        resource: NativePeopleDirectoryResource,
    ) -> Result<Self, CompanyPolicyError> {
        company_id(company)?;
        match (action, resource) {
            (DirectoryActionV1::Read, NativePeopleDirectoryResource::Collection) => {}
            (DirectoryActionV1::Read, NativePeopleDirectoryResource::Entry(id))
            | (DirectoryActionV1::Create, NativePeopleDirectoryResource::Request(id)) => {
                nonnil(id)?
            }
            _ => return Err(CompanyPolicyError::MaterialUnavailable),
        }
        Ok(Self {
            company,
            actor,
            action,
            resource,
        })
    }
    pub const fn company(&self) -> OrgId {
        self.company
    }
    pub const fn actor(&self) -> AccountId {
        self.actor
    }
    pub const fn action(&self) -> DirectoryActionV1 {
        self.action
    }
    pub const fn resource(&self) -> NativePeopleDirectoryResource {
        self.resource
    }
}

/// Exact SQL outputs plus independently authenticated retained-source coordinates.
/// ACTIVE/nondelegable comes from the checked named SQL owner's contract. This is
/// never a general grant DTO or a means to bypass that owner/custody verification.
#[derive(Clone)]
pub struct NativePeopleDirectoryProjectionRow {
    pub account_id: Uuid,
    pub session_id: Uuid,
    pub account_security_generation: i64,
    pub org_id: Uuid,
    pub company_epoch: i64,
    pub current_policy_receipt_id: Uuid,
    pub assignment_id: Uuid,
    pub assignment_revision: i64,
    pub role_id: Uuid,
    pub role_revision: i64,
    pub registered_clauses: String,
    pub assignment_valid_from: OffsetDateTime,
    pub assignment_valid_until: OffsetDateTime,
    pub action_reference: String,
    pub named_properties: String,
    pub action: DirectoryActionV1,
    pub observed_at: OffsetDateTime,
    pub source_xid: String,
    pub source_backend_pid: i32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamedPeopleDirectoryProperty {
    key: String,
    property: PropertyRef,
}
impl NamedPeopleDirectoryProperty {
    pub fn key(&self) -> &str {
        &self.key
    }
    pub const fn property(&self) -> &PropertyRef {
        &self.property
    }
}
pub struct CurrentPeopleDirectoryAuthority {
    source: NativePeopleDirectoryProjectionRow,
    observed_at: OffsetDateTime,
    action_reference: ActionRef,
    named_properties: Vec<NamedPeopleDirectoryProperty>,
}
impl CurrentPeopleDirectoryAuthority {
    pub fn from_retained_projection(
        binding: &NativePolicySourceBinding,
        row: NativePeopleDirectoryProjectionRow,
    ) -> Result<Self, CompanyPolicyError> {
        use CompanyPolicyError::MaterialUnavailable;
        binding.check(
            row.account_id,
            row.session_id,
            row.account_security_generation,
            &row.source_xid,
            row.source_backend_pid,
            row.observed_at,
        )?;
        let company = OrgId::from_uuid(row.org_id);
        company_id(company)?;
        for id in [
            row.current_policy_receipt_id,
            row.assignment_id,
            row.role_id,
        ] {
            nonnil(id)?;
        }
        exact_micros(row.assignment_valid_from)?;
        exact_micros(row.assignment_valid_until)?;
        if row.company_epoch < 2
            || row.assignment_revision < 1
            || row.role_revision != 1
            || row.assignment_valid_from >= row.assignment_valid_until
            || row.assignment_valid_until - row.assignment_valid_from > Duration::days(30)
        {
            return Err(MaterialUnavailable);
        }
        let (action_reference, named_properties) = decode_native_people_directory_registration(
            company,
            row.action,
            &row.registered_clauses,
            &row.action_reference,
            &row.named_properties,
        )?;
        Ok(Self {
            source: row,
            observed_at: binding.observed_at,
            action_reference,
            named_properties,
        })
    }
    pub fn source(&self) -> &NativePeopleDirectoryProjectionRow {
        &self.source
    }
    pub const fn observed_at(&self) -> OffsetDateTime {
        self.observed_at
    }
    pub const fn action_reference(&self) -> &ActionRef {
        &self.action_reference
    }
    pub fn named_properties(&self) -> &[NamedPeopleDirectoryProperty] {
        &self.named_properties
    }
    pub fn property(&self, key: &str) -> Option<&PropertyRef> {
        self.named_properties
            .iter()
            .find(|p| p.key == key)
            .map(|p| &p.property)
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireNamedProperty {
    key: String,
    org_id: String,
    object_type_id: String,
    property_id: String,
    schema_revision: String,
}
fn action_ref(company: OrgId, a: WireAction) -> Result<ActionRef, CompanyPolicyError> {
    if canonical_uuid(&a.org_id)? != *company.as_uuid()
        || canonical_revision(&a.registration_revision)? != 1
        || a.manifest_digest
            != MANIFEST
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
    {
        return Err(CompanyPolicyError::MaterialUnavailable);
    }
    ActionRef::new(
        company,
        canonical_uuid(&a.action_type_id)?,
        canonical_uuid(&a.object_type_id)?,
        1,
        MANIFEST,
    )
}

/// Decode raw text before any serde_json::Value roundtrip: duplicate keys remain
/// observable. Compare exact clause refs to independently named catalog refs.
pub fn decode_native_people_directory_registration(
    company: OrgId,
    kind: DirectoryActionV1,
    registered: &str,
    reference: &str,
    named: &str,
) -> Result<(ActionRef, Vec<NamedPeopleDirectoryProperty>), CompanyPolicyError> {
    use CompanyPolicyError::MaterialUnavailable;
    company_id(company)?;
    if registered.len() > 32768 || reference.len() > 4096 || named.len() > 16384 {
        return Err(MaterialUnavailable);
    }
    let Object(a): Object<WireAction> =
        serde_json::from_str(reference).map_err(|_| MaterialUnavailable)?;
    let action = action_ref(company, a)?;
    let names: Vec<Object<WireNamedProperty>> =
        serde_json::from_str(named).map_err(|_| MaterialUnavailable)?;
    if names.len() != property_keys(kind).len() {
        return Err(MaterialUnavailable);
    }
    let mut named_properties = Vec::with_capacity(names.len());
    for Object(p) in names {
        let property = PropertyRef::new(
            OrgId::from_uuid(canonical_uuid(&p.org_id)?),
            canonical_uuid(&p.object_type_id)?,
            canonical_uuid(&p.property_id)?,
            canonical_revision(&p.schema_revision)?,
        )?;
        if property.org_id() != company
            || property.object_type_id() != action.object_type_id()
            || property.schema_revision() != 1
        {
            return Err(MaterialUnavailable);
        }
        named_properties.push(NamedPeopleDirectoryProperty {
            key: p.key,
            property,
        });
    }
    named_properties.sort_by(|a, b| a.key.cmp(&b.key));
    if named_properties
        .iter()
        .map(|p| p.key.as_str())
        .ne(property_keys(kind).iter().copied())
    {
        return Err(MaterialUnavailable);
    }
    let mut named_refs: Vec<_> = named_properties
        .iter()
        .map(|p| p.property.clone())
        .collect();
    named_refs.sort();
    if !super::sorted_unique(&named_refs) {
        return Err(MaterialUnavailable);
    }
    let mut clauses: Vec<Object<WireClause>> =
        serde_json::from_str(registered).map_err(|_| MaterialUnavailable)?;
    if clauses.len() != 1 {
        return Err(MaterialUnavailable);
    }
    let Object(c) = clauses.pop().ok_or(MaterialUnavailable)?;
    if c.kind != "COMPANY_CAPABILITY_CLAUSE_V1"
        || c.resource.0.kind != "COMPANY"
        || canonical_uuid(&c.resource.0.org_id)? != *company.as_uuid()
        || c.delegable
        || c.valid_until.is_some()
        || c.fields.len() != named_refs.len()
        || action_ref(company, c.action.0)? != action
    {
        return Err(MaterialUnavailable);
    }
    canonical_time(&c.valid_from)?;
    let mut fields = Vec::with_capacity(c.fields.len());
    for Object(f) in c.fields {
        fields.push(PropertyRef::new(
            OrgId::from_uuid(canonical_uuid(&f.org_id)?),
            canonical_uuid(&f.object_type_id)?,
            canonical_uuid(&f.property_id)?,
            canonical_revision(&f.schema_revision)?,
        )?);
    }
    fields.sort();
    if fields != named_refs {
        return Err(MaterialUnavailable);
    }
    Ok((action, named_properties))
}

#[cfg(test)]
#[path = "people_directory_tests.rs"]
mod tests;
