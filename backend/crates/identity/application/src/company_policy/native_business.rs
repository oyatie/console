//! Checked material from the retained native Company owner, never client proof.
use super::business::NativeBusinessOperationV1;
use super::{AccountId, ActionRef, CompanyPolicyError, PropertyRef, company_id, nonnil};
use console_kernel_core::OrgId;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

/// Decode the single registered business clause; retained SQL still owns custody
/// and the assignment's effective interval, which is distinct from clause dates.
pub fn decode_native_payroll_read_clause(
    company: OrgId,
    registered: &str,
) -> Result<(ActionRef, Vec<PropertyRef>), CompanyPolicyError> {
    use super::{Object, WireClause, canonical_revision, canonical_time, canonical_uuid};
    use CompanyPolicyError::MaterialUnavailable;
    company_id(company)?;
    if registered.len() > 32768 {
        return Err(MaterialUnavailable);
    }
    let mut clauses: Vec<Object<WireClause>> =
        serde_json::from_str(registered).map_err(|_| MaterialUnavailable)?;
    if clauses.len() != 1 {
        return Err(MaterialUnavailable);
    }
    let Object(clause) = clauses.pop().ok_or(MaterialUnavailable)?;
    let Object(action) = clause.action;
    let manifest: String = super::business::MANIFEST
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    if clause.kind != "COMPANY_CAPABILITY_CLAUSE_V1"
        || clause.resource.0.kind != "COMPANY"
        || canonical_uuid(&clause.resource.0.org_id)? != *company.as_uuid()
        || canonical_uuid(&action.org_id)? != *company.as_uuid()
        || canonical_revision(&action.registration_revision)? != 1
        || action.manifest_digest != manifest
        || clause.delegable
        || clause.valid_until.is_some()
        || clause.fields.len() != 18
    {
        return Err(MaterialUnavailable);
    }
    canonical_time(&clause.valid_from)?;
    let action = ActionRef::new(
        company,
        canonical_uuid(&action.action_type_id)?,
        canonical_uuid(&action.object_type_id)?,
        1,
        super::business::MANIFEST,
    )?;
    let mut fields = Vec::with_capacity(18);
    for Object(field) in clause.fields {
        let field = PropertyRef::new(
            OrgId::from_uuid(canonical_uuid(&field.org_id)?),
            canonical_uuid(&field.object_type_id)?,
            canonical_uuid(&field.property_id)?,
            canonical_revision(&field.schema_revision)?,
        )?;
        if field.org_id() != company
            || field.object_type_id() != action.object_type_id()
            || field.schema_revision() != 1
        {
            return Err(MaterialUnavailable);
        }
        fields.push(field);
    }
    fields.sort();
    if !super::sorted_unique(&fields) {
        return Err(MaterialUnavailable);
    }
    Ok((action, fields))
}

pub struct NativePolicySourceBinding {
    pub account: AccountId,
    pub session_id: Uuid,
    pub account_security_generation: i64,
    pub source_xid: u64,
    pub source_backend_pid: i32,
    pub observed_at: OffsetDateTime,
}

#[derive(Clone)]
pub struct NativeBootstrapProjectionRow {
    pub actor_account_id: Uuid,
    pub session_id: Uuid,
    pub account_security_generation: i64,
    pub designation_system_identifier: String,
    pub designation_database_name: String,
    pub designation_database_oid: i64,
    pub designation_revision: i64,
    pub designation_receipt_id: Uuid,
    pub org_id: Uuid,
    pub current_group_id: Uuid,
    pub group_revision: i64,
    pub group_incarnation: Uuid,
    pub membership_id: Uuid,
    pub membership_revision: i64,
    pub membership_incarnation: Uuid,
    pub company_epoch: i64,
    pub current_policy_receipt_id: Option<Uuid>,
    pub origin_account_id: Uuid,
    pub origin_command_id: Uuid,
    pub origin_receipt_id: Uuid,
    pub administrative_account_id: Uuid,
    pub company_actor_admission_receipt_id: Uuid,
    pub birth_assignment_id: Uuid,
    pub birth_role_id: Uuid,
    pub observed_at: OffsetDateTime,
    pub source_xid: String,
    pub source_backend_pid: i32,
}

pub struct CurrentNativeBootstrapAuthority(NativeBootstrapProjectionRow);

impl CurrentNativeBootstrapAuthority {
    pub fn from_retained_projection(
        source: &NativePolicySourceBinding,
        row: NativeBootstrapProjectionRow,
    ) -> Result<Self, CompanyPolicyError> {
        use CompanyPolicyError::MaterialUnavailable;
        source.check(
            row.actor_account_id,
            row.session_id,
            row.account_security_generation,
            &row.source_xid,
            row.source_backend_pid,
            row.observed_at,
        )?;
        company_id(OrgId::from_uuid(row.org_id))?;
        for id in [
            row.designation_receipt_id,
            row.current_group_id,
            row.group_incarnation,
            row.membership_id,
            row.membership_incarnation,
            row.origin_account_id,
            row.origin_command_id,
            row.origin_receipt_id,
            row.administrative_account_id,
            row.company_actor_admission_receipt_id,
            row.birth_assignment_id,
            row.birth_role_id,
        ] {
            nonnil(id)?;
        }
        positive_decimal(&row.designation_system_identifier)?;
        if row.designation_database_name.is_empty()
            || row.designation_database_name.len() > 63
            || row.designation_database_name.contains('\0')
            || !(1..=i64::from(u32::MAX)).contains(&row.designation_database_oid)
            || row.designation_revision < 1
            || row.group_revision < 1
            || row.membership_revision < 1
            || row.company_epoch < 1
            || (row.company_epoch == 1) != row.current_policy_receipt_id.is_none()
            || row.current_policy_receipt_id.is_some_and(|id| id.is_nil())
            || row.company_actor_admission_receipt_id != row.origin_receipt_id
        {
            return Err(MaterialUnavailable);
        }
        Ok(Self(row))
    }
    pub fn source(&self) -> &NativeBootstrapProjectionRow {
        &self.0
    }
}

pub struct NativeBootstrapRequestV1 {
    company: OrgId,
    group: Uuid,
    recipient: AccountId,
    operation: NativeBusinessOperationV1,
    manifest_digest: [u8; 32],
}

impl NativeBootstrapRequestV1 {
    pub fn new(
        company: OrgId,
        group: Uuid,
        recipient: AccountId,
        operation: NativeBusinessOperationV1,
        manifest_digest: [u8; 32],
    ) -> Result<Self, CompanyPolicyError> {
        company_id(company)?;
        nonnil(group)?;
        Ok(Self {
            company,
            group,
            recipient,
            operation,
            manifest_digest,
        })
    }
    pub const fn company(&self) -> OrgId {
        self.company
    }
    pub const fn group(&self) -> Uuid {
        self.group
    }
    pub const fn recipient(&self) -> AccountId {
        self.recipient
    }
    pub const fn operation(&self) -> NativeBusinessOperationV1 {
        self.operation
    }
    pub const fn manifest_digest(&self) -> &[u8; 32] {
        &self.manifest_digest
    }
}

#[derive(Clone)]
pub struct NativePayrollReadProjectionRow {
    pub account_id: Uuid,
    pub session_id: Uuid,
    pub account_security_generation: i64,
    pub org_id: Uuid,
    pub company_epoch: i64,
    pub current_policy_receipt_id: Uuid,
    pub assignment_id: Uuid,
    pub assignment_revision: i64,
    pub assignment_state: String,
    pub role_id: Uuid,
    pub role_revision: i64,
    pub role_state: String,
    pub valid_from: OffsetDateTime,
    pub valid_until: OffsetDateTime,
    pub delegable: bool,
    pub action: ActionRef,
    pub properties: Vec<PropertyRef>,
    pub observed_at: OffsetDateTime,
    pub source_xid: String,
    pub source_backend_pid: i32,
}

pub struct CurrentPayrollReadAuthority {
    source: NativePayrollReadProjectionRow,
    observed_at: OffsetDateTime,
}

impl CurrentPayrollReadAuthority {
    pub fn from_retained_projection(
        source: &NativePolicySourceBinding,
        mut row: NativePayrollReadProjectionRow,
    ) -> Result<Self, CompanyPolicyError> {
        use CompanyPolicyError::MaterialUnavailable;
        source.check(
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
        exact_micros(row.valid_from)?;
        exact_micros(row.valid_until)?;
        if row.company_epoch < 2
            || row.assignment_revision < 1
            || row.role_revision != 1
            || row.assignment_state != "ACTIVE"
            || row.role_state != "ACTIVE"
            || row.delegable
            || row.valid_from >= row.valid_until
            || row.valid_until - row.valid_from > Duration::days(30)
            || row.action.org_id() != company
            || row.action.registration_revision() != 1
            || row.action.manifest_digest() != &super::business::MANIFEST
            || row.properties.len() != 18
        {
            return Err(MaterialUnavailable);
        }
        row.properties.sort();
        if !super::sorted_unique(&row.properties)
            || row.properties.iter().any(|p| {
                p.org_id() != company
                    || p.object_type_id() != row.action.object_type_id()
                    || p.schema_revision() != 1
            })
        {
            return Err(MaterialUnavailable);
        }
        Ok(Self {
            source: row,
            observed_at: source.observed_at,
        })
    }
    pub fn source(&self) -> &NativePayrollReadProjectionRow {
        &self.source
    }
    pub const fn observed_at(&self) -> OffsetDateTime {
        self.observed_at
    }
}

impl NativePolicySourceBinding {
    fn check(
        &self,
        account: Uuid,
        session: Uuid,
        generation: i64,
        xid: &str,
        pid: i32,
        observed_at: OffsetDateTime,
    ) -> Result<(), CompanyPolicyError> {
        use CompanyPolicyError::MaterialUnavailable;
        nonnil(self.session_id)?;
        exact_micros(self.observed_at)?;
        exact_micros(observed_at)?;
        if self.account_security_generation < 1
            || self.source_xid == 0
            || self.source_backend_pid < 1
            || account != *self.account.as_uuid()
            || session != self.session_id
            || generation != self.account_security_generation
            || positive_decimal(xid)? != self.source_xid
            || pid != self.source_backend_pid
            || observed_at > self.observed_at
        {
            return Err(MaterialUnavailable);
        }
        Ok(())
    }
}

fn positive_decimal(value: &str) -> Result<u64, CompanyPolicyError> {
    let parsed = value
        .parse::<u64>()
        .map_err(|_| CompanyPolicyError::MaterialUnavailable)?;
    if parsed == 0 || parsed.to_string() != value {
        return Err(CompanyPolicyError::MaterialUnavailable);
    }
    Ok(parsed)
}

fn exact_micros(value: OffsetDateTime) -> Result<i64, CompanyPolicyError> {
    let nanos = value.unix_timestamp_nanos();
    if nanos % 1_000 != 0 {
        return Err(CompanyPolicyError::MaterialUnavailable);
    }
    i64::try_from(nanos / 1_000).map_err(|_| CompanyPolicyError::MaterialUnavailable)
}
