//! Current People source held in the caller's original owner transaction.
use super::*;
use console_identity_application::company_policy::people_business::DirectoryActionV1;
use console_identity_application::company_policy::{
    CompanyPolicyDecision, CurrentPeopleDirectoryAuthority, NativePeopleDirectoryProjectionRow,
    NativePeopleDirectoryRequestV1, NativePeopleDirectoryResource, NativePolicySourceBinding,
};
use console_platform_auth::account::AccountLiveSession;

pub struct NativeDirectoryAuthority {
    pub(super) current: CurrentPeopleDirectoryAuthority,
    actor: AccountId,
}
impl DirectoryAuthority for NativeDirectoryAuthority {
    fn company(&self) -> OrgId {
        OrgId::from_uuid(self.current.source().org_id)
    }
    fn actor(&self) -> AccountId {
        self.actor
    }
    fn action(&self) -> DirectoryAction {
        match self.current.source().action {
            DirectoryActionV1::Read => DirectoryAction::Read,
            DirectoryActionV1::Create => DirectoryAction::Create,
        }
    }
    fn observed_at(&self) -> OffsetDateTime {
        self.current.observed_at()
    }
    fn creation_expectations(&self) -> Option<DirectoryExpectationsV1> {
        if self.action() != DirectoryAction::Create {
            return None;
        }
        let action = self.current.action_reference();
        let name = self.current.property("person.legal_name")?;
        let number = self.current.property("person.employee_number")?;
        Some(DirectoryExpectationsV1 {
            company_epoch: self.current.source().company_epoch as u64,
            object_type_id: action.object_type_id(),
            action_type_id: action.action_type_id(),
            action_revision: action.registration_revision(),
            schema_revision: name.schema_revision(),
            legal_name_property_id: name.property_id(),
            employee_number_property_id: number.property_id(),
        })
    }
}
pub struct DirectoryCedarDecision {
    policy: Arc<dyn CompanyPolicyDecisionPort>,
}
impl DirectoryCedarDecision {
    pub fn new(policy: Arc<dyn CompanyPolicyDecisionPort>) -> Self {
        Self { policy }
    }
}
impl DirectoryDecisionPort<NativeDirectoryAuthority> for DirectoryCedarDecision {
    fn permits(
        &self,
        authority: &NativeDirectoryAuthority,
        request: DirectoryAccess,
    ) -> Result<bool, Error> {
        let action = identity_action(request.action);
        let resource = match request.resource {
            DirectoryResource::Navigation => NativePeopleDirectoryResource::Navigation,
            DirectoryResource::Collection => NativePeopleDirectoryResource::Collection,
            DirectoryResource::Entry(id) => NativePeopleDirectoryResource::Entry(id),
            DirectoryResource::Request(id) => NativePeopleDirectoryResource::Request(id),
        };
        let request =
            NativePeopleDirectoryRequestV1::new(request.company, request.actor, action, resource)
                .map_err(|_| Error::Unavailable)?;
        self.policy
            .decide_native_people_directory(&authority.current, &request)
            .map(|d| d == CompanyPolicyDecision::Allow)
            .map_err(|_| Error::Unavailable)
    }
}
pub(super) fn identity_action(action: DirectoryAction) -> DirectoryActionV1 {
    match action {
        DirectoryAction::Read => DirectoryActionV1::Read,
        DirectoryAction::Create => DirectoryActionV1::Create,
    }
}
pub(super) fn action(kind: &DirectoryScopeKind) -> DirectoryAction {
    match kind {
        DirectoryScopeKind::Navigation(action) => *action,
        DirectoryScopeKind::List(_) | DirectoryScopeKind::Detail(_) => DirectoryAction::Read,
        _ => DirectoryAction::Create,
    }
}
pub(super) fn resource(kind: &DirectoryScopeKind) -> DirectoryResource {
    match kind {
        DirectoryScopeKind::Navigation(_) => DirectoryResource::Navigation,
        DirectoryScopeKind::List(_) => DirectoryResource::Collection,
        DirectoryScopeKind::Detail(id) => DirectoryResource::Entry(*id),
        DirectoryScopeKind::Form(id)
        | DirectoryScopeKind::ValidationForm(id)
        | DirectoryScopeKind::Preflight(id)
        | DirectoryScopeKind::Prepare(id)
        | DirectoryScopeKind::Execute(id)
        | DirectoryScopeKind::Cancel(id)
        | DirectoryScopeKind::Status(id) => DirectoryResource::Request(*id),
    }
}
pub(super) struct Source {
    pub authority: NativeDirectoryAuthority,
    pub binding: NativePolicySourceBinding,
    pub row: SqlSource,
}
#[derive(sqlx::FromRow)]
pub(super) struct SqlSource {
    company_epoch: i64,
    current_policy_receipt_id: Uuid,
    assignment_id: Uuid,
    assignment_revision: i64,
    role_id: Uuid,
    role_revision: i64,
    registered_clauses: String,
    assignment_valid_from: OffsetDateTime,
    assignment_valid_until: OffsetDateTime,
    action_reference: String,
    named_properties: String,
    source_xid: String,
    source_backend_pid: i32,
    observed_at: OffsetDateTime,
}
pub(super) async fn live_session(
    tx: &mut Transaction<'_, Postgres>,
    store: &PgNativeDirectoryStore,
    credentials: &AccountEnrollmentCredentials,
    kind: &DirectoryScopeKind,
) -> Result<AccountLiveSession, Error> {
    match kind {
        DirectoryScopeKind::ValidationForm(_)
        | DirectoryScopeKind::Preflight(_)
        | DirectoryScopeKind::Prepare(_)
        | DirectoryScopeKind::Execute(_)
        | DirectoryScopeKind::Cancel(_) => {
            credentials
                .validate_mutation_in_tx(tx, &store.verifier, store.absolute_ttl)
                .await
        }
        _ => {
            credentials
                .read_session_in_tx(tx, &store.verifier, store.absolute_ttl)
                .await
        }
    }
    .map_err(auth_error)
}
pub(super) async fn current(
    tx: &mut Transaction<'_, Postgres>,
    store: &PgNativeDirectoryStore,
    credentials: &AccountEnrollmentCredentials,
    kind: DirectoryScopeKind,
    company: OrgId,
    actor: Uuid,
    family: Uuid,
) -> Result<Source, Error> {
    let mut rows=sqlx::query_as::<_,SqlSource>("SELECT company_epoch,current_policy_receipt_id,assignment_id,assignment_revision,role_id,role_revision,registered_clauses::text,assignment_valid_from,assignment_valid_until,action_reference::text,named_properties::text,pg_current_xact_id()::text AS source_xid,pg_backend_pid() AS source_backend_pid,clock_timestamp() AS observed_at FROM public.identity_company_people_projection_v1($1,$2,$3,$4) LIMIT 2")
      .bind(actor).bind(family).bind(*company.as_uuid()).bind(identity_action(action(&kind)).as_str()).fetch_all(tx.as_mut()).await.map_err(sql_error)?;
    // Current Auth is required even when the source is absent. This follows the
    // canonical Group-first query's locks; signed IDs alone are not a session.
    let session = live_session(tx, store, credentials, &kind).await?;
    if session.account_id != actor || session.session_id != family || rows.len() > 1 {
        return Err(Error::Unavailable);
    }
    let row = rows.pop().ok_or(Error::NotFound)?;
    let (xid, pid, observed_at): (String, i32, OffsetDateTime) =
        sqlx::query_as("SELECT pg_current_xact_id()::text,pg_backend_pid(),clock_timestamp()")
            .fetch_one(tx.as_mut())
            .await
            .map_err(sql_error)?;
    if session.expires_at <= observed_at
        || session.family_expires_at <= observed_at
        || session.auth_time > observed_at
    {
        return Err(Error::AuthenticationInvalid);
    }
    let binding = NativePolicySourceBinding {
        account: AccountId::from_uuid(actor).map_err(|_| Error::Unavailable)?,
        session_id: family,
        account_security_generation: session.security_generation,
        source_xid: xid_number(&xid)?,
        source_backend_pid: pid,
        observed_at,
    };
    let projection = NativePeopleDirectoryProjectionRow {
        account_id: actor,
        session_id: family,
        account_security_generation: session.security_generation,
        org_id: *company.as_uuid(),
        company_epoch: row.company_epoch,
        current_policy_receipt_id: row.current_policy_receipt_id,
        assignment_id: row.assignment_id,
        assignment_revision: row.assignment_revision,
        role_id: row.role_id,
        role_revision: row.role_revision,
        registered_clauses: row.registered_clauses.clone(),
        assignment_valid_from: row.assignment_valid_from,
        assignment_valid_until: row.assignment_valid_until,
        action_reference: row.action_reference.clone(),
        named_properties: row.named_properties.clone(),
        action: identity_action(action(&kind)),
        observed_at: row.observed_at,
        source_xid: row.source_xid.clone(),
        source_backend_pid: row.source_backend_pid,
    };
    let current = CurrentPeopleDirectoryAuthority::from_retained_projection(&binding, projection)
        .map_err(|_| Error::Unavailable)?;
    Ok(Source {
        authority: NativeDirectoryAuthority {
            current,
            actor: binding.account,
        },
        binding,
        row,
    })
}
pub(super) fn same(before: &Source, after: &Source) -> Result<(), Error> {
    let a = &before.row;
    let b = &after.row;
    if a.company_epoch != b.company_epoch
        || a.current_policy_receipt_id != b.current_policy_receipt_id
        || a.assignment_id != b.assignment_id
        || a.assignment_revision != b.assignment_revision
        || a.role_id != b.role_id
        || a.role_revision != b.role_revision
        || a.registered_clauses != b.registered_clauses
        || a.assignment_valid_from != b.assignment_valid_from
        || a.assignment_valid_until != b.assignment_valid_until
        || a.action_reference != b.action_reference
        || a.named_properties != b.named_properties
        || a.source_xid != b.source_xid
        || a.source_backend_pid != b.source_backend_pid
        || b.observed_at < a.observed_at
        || before.binding.account != after.binding.account
        || before.binding.session_id != after.binding.session_id
        || before.binding.account_security_generation != after.binding.account_security_generation
        || before.binding.source_xid != after.binding.source_xid
        || before.binding.source_backend_pid != after.binding.source_backend_pid
        || after.binding.observed_at < before.binding.observed_at
        || before.authority.company() != after.authority.company()
        || before.authority.action() != after.authority.action()
    {
        Err(Error::Unavailable)
    } else {
        Ok(())
    }
}
