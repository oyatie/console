//! Native directory application use cases and retained owner ports.
//! Exact current authority material belongs to the real adapter/Cedar boundary;
//! HTTP identifiers and this interface cannot manufacture authenticated facts.
use super::{
    AcceptedDirectoryRequestV1, DirectoryExpectationsV1, DirectoryRegistrationInput,
    DirectoryTerminalOutcomeV1, DirectoryTerminalV1,
};
use console_kernel_core::{AccountId, OrgId, TraceContext};
use std::future::Future;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectoryWorkflowError {
    InvalidInput,
    AuthenticationInvalid,
    CsrfInvalid,
    NotFound,
    Conflict,
    Capacity,
    Unavailable,
    Unconfirmed,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectoryAction {
    Read,
    Create,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectoryResource {
    Navigation,
    Collection,
    Entry(Uuid),
    Request(Uuid),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirectoryAccess {
    pub company: OrgId,
    pub actor: AccountId,
    pub action: DirectoryAction,
    pub resource: DirectoryResource,
}

/// Implement only for checked People-specific material from the SAME retained
/// authenticated SQL source. Must not wrap the deployment-operator bootstrap.
/// Concrete material also retains exact session/generation/xid/backend/time,
/// current assignment/role/catalog/registered field identities for Cedar.
pub trait DirectoryAuthority: Send + Sync {
    fn company(&self) -> OrgId;
    fn actor(&self) -> AccountId;
    fn action(&self) -> DirectoryAction;
    /// Fresh retained-source database time, never a browser clock.
    fn observed_at(&self) -> OffsetDateTime;
    /// Some only for a checked current Create action and its exact2properties.
    fn creation_expectations(&self) -> Option<DirectoryExpectationsV1>;
}
pub trait DirectoryDecisionPort<A: DirectoryAuthority>: Send + Sync {
    /// Evaluate actual current Cedar material against this exact resource/action.
    /// False is a healthy nondisclosing denial; errors never become Allow.
    fn permits(
        &self,
        authority: &A,
        request: DirectoryAccess,
    ) -> Result<bool, DirectoryWorkflowError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirectoryRequestRef {
    company: OrgId,
    command_id: Uuid,
}
impl DirectoryRequestRef {
    pub fn new(company: OrgId, command_id: Uuid) -> Result<Self, DirectoryWorkflowError> {
        company_id(company)?;
        if command_id.is_nil() {
            return Err(DirectoryWorkflowError::InvalidInput);
        }
        Ok(Self {
            company,
            command_id,
        })
    }
    pub const fn company(self) -> OrgId {
        self.company
    }
    pub const fn command_id(self) -> Uuid {
        self.command_id
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectorySubmission {
    locator: DirectoryRequestRef,
    expected: DirectoryExpectationsV1,
    input: DirectoryRegistrationInput,
}
impl DirectorySubmission {
    pub fn new(
        locator: DirectoryRequestRef,
        expected: DirectoryExpectationsV1,
        input: DirectoryRegistrationInput,
    ) -> Result<Self, DirectoryWorkflowError> {
        // Validate expectations before allocation; preflight needs no employee ID.
        expected
            .validate()
            .map_err(|_| DirectoryWorkflowError::InvalidInput)?;
        Ok(Self {
            locator,
            expected,
            input,
        })
    }
    pub const fn locator(&self) -> DirectoryRequestRef {
        self.locator
    }
    pub const fn expected(&self) -> DirectoryExpectationsV1 {
        self.expected
    }
    pub fn input(&self) -> &DirectoryRegistrationInput {
        &self.input
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectoryPageQuery {
    after: Option<Uuid>,
    limit: u16,
    employee_number: Option<String>,
}
impl DirectoryPageQuery {
    pub fn new(after: Option<Uuid>, limit: Option<u16>) -> Result<Self, DirectoryWorkflowError> {
        Self::with_number(after, limit, None)
    }
    pub fn with_number(
        after: Option<Uuid>,
        limit: Option<u16>,
        employee_number: Option<String>,
    ) -> Result<Self, DirectoryWorkflowError> {
        let limit = limit.unwrap_or(25);
        if !(1..=100).contains(&limit) || after.is_some_and(|id| id.is_nil()) {
            return Err(DirectoryWorkflowError::InvalidInput);
        }
        if employee_number.as_deref().is_some_and(|number| {
            DirectoryRegistrationInput::normalized_employee_number(number) != Ok(number)
        }) {
            return Err(DirectoryWorkflowError::InvalidInput);
        }
        Ok(Self {
            after,
            limit,
            employee_number,
        })
    }
    pub const fn after(&self) -> Option<Uuid> {
        self.after
    }
    pub const fn limit(&self) -> u16 {
        self.limit
    }
    pub fn employee_number(&self) -> Option<&str> {
        self.employee_number.as_deref()
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectoryRecord {
    pub employee_id: Uuid,
    pub person_id: Uuid,
    pub legal_name: Option<String>,
    pub employee_number: Option<String>,
    pub person_version: u64,
    pub registered_at: OffsetDateTime,
}
pub struct DirectoryPage {
    pub records: Vec<DirectoryRecord>,
    pub next_after: Option<Uuid>,
}
pub struct DirectoryForm<F> {
    pub locator: DirectoryRequestRef,
    pub expected: DirectoryExpectationsV1,
    pub proof: F,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirectoryStatus {
    NotVisible,
    Pending(AcceptedDirectoryRequestV1),
    Terminal(DirectoryTerminalV1),
}
impl DirectoryStatus {
    fn accepted(&self) -> Option<&AcceptedDirectoryRequestV1> {
        match self {
            Self::NotVisible => None,
            Self::Pending(a) => Some(a),
            Self::Terminal(t) => Some(t.accepted()),
        }
    }
}
pub struct DirectoryRecovery<F> {
    pub status: DirectoryStatus,
    /// Present exactly for an actionable Pending request in this same scope.
    pub proof: Option<F>,
}

pub struct DirectoryAcceptance {
    pub inserted: bool,
    pub status: DirectoryStatus,
}
pub struct DirectoryExecution {
    pub inserted: bool,
    pub terminal: DirectoryTerminalV1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirectoryScopeKind {
    Navigation(DirectoryAction),
    List(DirectoryPageQuery),
    Detail(Uuid),
    Form(Uuid),
    ValidationForm(Uuid),
    Preflight(Uuid),
    Prepare(Uuid),
    Execute(Uuid),
    Cancel(Uuid),
    Status(Uuid),
}
impl DirectoryScopeKind {
    fn action(&self) -> DirectoryAction {
        match self {
            Self::Navigation(action) => *action,
            Self::List(_) | Self::Detail(_) => DirectoryAction::Read,
            _ => DirectoryAction::Create,
        }
    }
    fn resource(&self) -> DirectoryResource {
        match self {
            Self::Navigation(_) => DirectoryResource::Navigation,
            Self::List(_) => DirectoryResource::Collection,
            Self::Detail(id) => DirectoryResource::Entry(*id),
            Self::Form(id)
            | Self::ValidationForm(id)
            | Self::Preflight(id)
            | Self::Prepare(id)
            | Self::Execute(id)
            | Self::Cancel(id)
            | Self::Status(id) => DirectoryResource::Request(*id),
        }
    }
}
pub enum DirectoryScopeRequest<'a> {
    Navigation(OrgId, DirectoryAction),
    List(OrgId, DirectoryPageQuery),
    Detail(OrgId, Uuid),
    Form(DirectoryRequestRef),
    ValidationForm(DirectoryRequestRef, DirectoryExpectationsV1),
    Preflight(&'a DirectorySubmission),
    Prepare(&'a DirectorySubmission),
    Execute(DirectoryRequestRef),
    Cancel(DirectoryRequestRef),
    Status(DirectoryRequestRef),
}
impl DirectoryScopeRequest<'_> {
    pub fn company(&self) -> OrgId {
        match self {
            Self::Navigation(c, _) | Self::List(c, _) | Self::Detail(c, _) => *c,
            Self::Form(r) | Self::Execute(r) | Self::Cancel(r) | Self::Status(r) => r.company(),
            Self::ValidationForm(r, _) => r.company(),
            Self::Preflight(s) | Self::Prepare(s) => s.locator().company(),
        }
    }
    pub fn kind(&self) -> DirectoryScopeKind {
        match self {
            Self::Navigation(_, action) => DirectoryScopeKind::Navigation(*action),
            Self::List(_, q) => DirectoryScopeKind::List(q.clone()),
            Self::Detail(_, id) => DirectoryScopeKind::Detail(*id),
            Self::Form(r) => DirectoryScopeKind::Form(r.command_id()),
            Self::ValidationForm(r, _) => DirectoryScopeKind::ValidationForm(r.command_id()),
            Self::Preflight(s) => DirectoryScopeKind::Preflight(s.locator().command_id()),
            Self::Prepare(s) => DirectoryScopeKind::Prepare(s.locator().command_id()),
            Self::Execute(r) => DirectoryScopeKind::Execute(r.command_id()),
            Self::Cancel(r) => DirectoryScopeKind::Cancel(r.command_id()),
            Self::Status(r) => DirectoryScopeKind::Status(r.command_id()),
        }
    }
}

/// Admission to issue a fresh form or new Pending recovery proof. Entry adapters provide the
/// real limiter; there is deliberately no unmetered production implementation.
/// The owning transaction calls this only after current authorization and the
/// fresh-form or Pending decision, then rechecks current authority/deadline after await.
pub trait DirectoryProofAdmission: Send {
    fn admit(&mut self) -> impl Future<Output = Result<(), DirectoryWorkflowError>> + Send;
}

pub trait DirectoryWorkflowScope: Send {
    type Authority: DirectoryAuthority;
    type FormProof: Send + Sync;
    fn authority(&self) -> &Self::Authority;
    fn kind(&self) -> DirectoryScopeKind;
    // Methods reject wrong scope mode before SQL. No method opens a second
    // transaction or returns an independently committed provisional result.
    fn list(
        &mut self,
    ) -> impl Future<Output = Result<DirectoryPage, DirectoryWorkflowError>> + Send;
    fn detail(
        &mut self,
    ) -> impl Future<Output = Result<Option<DirectoryRecord>, DirectoryWorkflowError>> + Send;
    fn form(
        &mut self,
    ) -> impl Future<Output = Result<DirectoryForm<Self::FormProof>, DirectoryWorkflowError>> + Send;
    /// No intake/allocation/audit/form issuance; pure owner feasibility check.
    fn preflight(&mut self) -> impl Future<Output = Result<(), DirectoryWorkflowError>> + Send;
    /// Same-input replay before capacity check, allocation once, intake+audit.
    fn prepare(
        &mut self,
        trace: &TraceContext,
    ) -> impl Future<Output = Result<DirectoryAcceptance, DirectoryWorkflowError>> + Send;
    /// Complete canonical employee/Person/binding/receipt/terminal/audit owner.
    fn execute(
        &mut self,
        trace: &TraceContext,
    ) -> impl Future<Output = Result<DirectoryExecution, DirectoryWorkflowError>> + Send;
    fn cancel(
        &mut self,
        trace: &TraceContext,
    ) -> impl Future<Output = Result<DirectoryExecution, DirectoryWorkflowError>> + Send;
    /// Materialize Expired when an authorized pending request reaches deadline.
    /// Otherwise issue an existing Auth session/Account/generation CSRF proof for
    /// Pending in this SAME transaction. Admission must run immediately before
    /// issuing that proof, only for Pending; no proof is issued on admission error.
    /// Terminal/NotVisible do not consume proof-admission capacity. This proof is not command-bound or
    /// one-use: route, owner scope and accepted actor bind the actual operation.
    /// Existing terminals get no new proof or date. After the status decision
    /// and any proof issuance, refresh the checked authority observation from
    /// this retained transaction before returning; do not reuse lock-time time.
    /// Pending deadline is checked against that refreshed observation.
    /// NotVisible does not establish that COMMIT failed.
    fn status<A: DirectoryProofAdmission + ?Sized>(
        &mut self,
        trace: &TraceContext,
        admission: &mut A,
    ) -> impl Future<Output = Result<DirectoryRecovery<Self::FormProof>, DirectoryWorkflowError>> + Send;
    /// Consume scope: drain deferred closure, fresh time/current credentials,
    /// same source identity and Cedar recheck, then COMMIT. Original proof only
    /// for form return and actionable Pending recovery. ValidationForm retains
    /// the submitted proof; it never mints a replacement. Pending recovery must
    /// remain before deadline at finalization. Existing terminal disclosure and
    /// expiry materialization do NOT enforce the old before-deadline predicate.
    /// Unknown COMMIT =>Unconfirmed; no provisional result.
    /// Drop or cancelled future rolls back before dispatch; after dispatch the
    /// adapter must preserve unknown-outcome recovery, never claim rollback.
    fn finish<P: DirectoryDecisionPort<Self::Authority> + ?Sized>(
        self,
        policy: &P,
        proof: Option<&Self::FormProof>,
    ) -> impl Future<Output = Result<(), DirectoryWorkflowError>> + Send;
}
pub trait DirectoryWorkflowStore {
    type Credentials: Sync;
    type Authority: DirectoryAuthority;
    type FormProof: Send + Sync;
    type Scope<'a>: DirectoryWorkflowScope<Authority = Self::Authority, FormProof = Self::FormProof>
        + Send
    where
        Self: 'a;
    /// Exact original Auth-owned credentials retained; same transaction/source
    /// owns Group-first locks, CSRF validation by mode and all later effects.
    fn lock<'a>(
        &'a self,
        credentials: &'a Self::Credentials,
        request: DirectoryScopeRequest<'a>,
    ) -> impl Future<Output = Result<Self::Scope<'a>, DirectoryWorkflowError>> + Send;
}

fn company_id(company: OrgId) -> Result<(), DirectoryWorkflowError> {
    if company.as_uuid().is_nil() || company == OrgId::platform() {
        Err(DirectoryWorkflowError::InvalidInput)
    } else {
        Ok(())
    }
}
fn authorize<S: DirectoryWorkflowScope, P: DirectoryDecisionPort<S::Authority> + ?Sized>(
    scope: &S,
    policy: &P,
    company: OrgId,
    kind: DirectoryScopeKind,
) -> Result<AccountId, DirectoryWorkflowError> {
    let a = scope.authority();
    if a.company() != company || a.action() != kind.action() || scope.kind() != kind {
        return Err(DirectoryWorkflowError::Unavailable);
    }
    let access = DirectoryAccess {
        company,
        actor: a.actor(),
        action: kind.action(),
        resource: kind.resource(),
    };
    if !policy.permits(a, access)? {
        return Err(DirectoryWorkflowError::NotFound);
    }
    Ok(a.actor())
}
fn accepted_matches(
    a: &AcceptedDirectoryRequestV1,
    r: DirectoryRequestRef,
    actor: AccountId,
) -> Result<(), DirectoryWorkflowError> {
    if a.actor() != actor
        || a.command().company() != r.company()
        || a.command().command_id() != r.command_id()
    {
        Err(DirectoryWorkflowError::Unavailable)
    } else {
        Ok(())
    }
}
fn current_submission<A: DirectoryAuthority>(
    authority: &A,
    s: &DirectorySubmission,
) -> Result<(), DirectoryWorkflowError> {
    match authority.creation_expectations() {
        Some(e) if e == s.expected() => Ok(()),
        Some(_) => Err(DirectoryWorkflowError::Conflict),
        None => Err(DirectoryWorkflowError::Unavailable),
    }
}

/// Point-in-time navigation admission only. No rows, proof, intake or read audit.
/// Following the destination always invokes its own current authorization again.
pub async fn directory_navigation<
    S: DirectoryWorkflowStore,
    P: DirectoryDecisionPort<S::Authority> + ?Sized,
>(
    store: &S,
    policy: &P,
    credentials: &S::Credentials,
    company: OrgId,
    action: DirectoryAction,
) -> Result<(), DirectoryWorkflowError> {
    company_id(company)?;
    let scope = store
        .lock(
            credentials,
            DirectoryScopeRequest::Navigation(company, action),
        )
        .await?;
    authorize(
        &scope,
        policy,
        company,
        DirectoryScopeKind::Navigation(action),
    )?;
    scope.finish(policy, None).await
}

pub async fn directory_list<
    S: DirectoryWorkflowStore,
    P: DirectoryDecisionPort<S::Authority> + ?Sized,
>(
    store: &S,
    policy: &P,
    credentials: &S::Credentials,
    company: OrgId,
    query: DirectoryPageQuery,
) -> Result<DirectoryPage, DirectoryWorkflowError> {
    company_id(company)?;
    let mut scope = store
        .lock(
            credentials,
            DirectoryScopeRequest::List(company, query.clone()),
        )
        .await?;
    authorize(
        &scope,
        policy,
        company,
        DirectoryScopeKind::List(query.clone()),
    )?;
    let page = scope.list().await?;
    if page.records.len() > usize::from(query.limit())
        || page.records.iter().any(|r| {
            r.employee_id.is_nil()
                || r.person_id.is_nil()
                || r.person_version == 0
                || r.person_version > i64::MAX as u64
                || query.after().is_some_and(|after| r.employee_id <= after)
                || query
                    .employee_number()
                    .is_some_and(|number| r.employee_number.as_deref() != Some(number))
        })
        || page
            .records
            .windows(2)
            .any(|r| r[0].employee_id >= r[1].employee_id)
        || page.next_after.is_some_and(|id| {
            page.records.last().map(|r| r.employee_id) != Some(id)
                || page.records.len() != usize::from(query.limit())
        })
    {
        return Err(DirectoryWorkflowError::Unavailable);
    }
    scope.finish(policy, None).await?;
    Ok(page)
}
pub async fn directory_detail<
    S: DirectoryWorkflowStore,
    P: DirectoryDecisionPort<S::Authority> + ?Sized,
>(
    store: &S,
    policy: &P,
    credentials: &S::Credentials,
    company: OrgId,
    entry: Uuid,
) -> Result<Option<DirectoryRecord>, DirectoryWorkflowError> {
    company_id(company)?;
    if entry.is_nil() {
        return Err(DirectoryWorkflowError::InvalidInput);
    }
    let mut scope = store
        .lock(credentials, DirectoryScopeRequest::Detail(company, entry))
        .await?;
    authorize(&scope, policy, company, DirectoryScopeKind::Detail(entry))?;
    let record = scope.detail().await?;
    if record.as_ref().is_some_and(|r| {
        r.employee_id != entry
            || r.person_id.is_nil()
            || r.person_version == 0
            || r.person_version > i64::MAX as u64
    }) {
        return Err(DirectoryWorkflowError::Unavailable);
    }
    scope.finish(policy, None).await?;
    Ok(record)
}
pub async fn directory_form<
    S: DirectoryWorkflowStore,
    P: DirectoryDecisionPort<S::Authority> + ?Sized,
    A: DirectoryProofAdmission + ?Sized,
>(
    store: &S,
    policy: &P,
    credentials: &S::Credentials,
    locator: DirectoryRequestRef,
    original_expectations: Option<DirectoryExpectationsV1>,
    admission: &mut A,
) -> Result<DirectoryForm<S::FormProof>, DirectoryWorkflowError> {
    let request = match original_expectations {
        Some(expected) => {
            expected
                .validate()
                .map_err(|_| DirectoryWorkflowError::InvalidInput)?;
            DirectoryScopeRequest::ValidationForm(locator, expected)
        }
        None => DirectoryScopeRequest::Form(locator),
    };
    let kind = request.kind();
    let mut scope = store.lock(credentials, request).await?;
    authorize(&scope, policy, locator.company(), kind)?;
    if original_expectations.is_none() {
        admission.admit().await?;
    }
    let form = scope.form().await?;
    if form.locator != locator
        || form.expected.validate().is_err()
        || original_expectations.or_else(|| scope.authority().creation_expectations())
            != Some(form.expected)
    {
        return Err(DirectoryWorkflowError::Unavailable);
    }
    scope.finish(policy, Some(&form.proof)).await?;
    Ok(form)
}
pub async fn directory_preflight<
    S: DirectoryWorkflowStore,
    P: DirectoryDecisionPort<S::Authority> + ?Sized,
>(
    store: &S,
    policy: &P,
    credentials: &S::Credentials,
    input: &DirectorySubmission,
) -> Result<(), DirectoryWorkflowError> {
    let r = input.locator();
    let mut scope = store
        .lock(credentials, DirectoryScopeRequest::Preflight(input))
        .await?;
    authorize(
        &scope,
        policy,
        r.company(),
        DirectoryScopeKind::Preflight(r.command_id()),
    )?;
    current_submission(scope.authority(), input)?;
    scope.preflight().await?;
    scope.finish(policy, None).await
}
pub async fn directory_prepare<
    S: DirectoryWorkflowStore,
    P: DirectoryDecisionPort<S::Authority> + ?Sized,
>(
    store: &S,
    policy: &P,
    credentials: &S::Credentials,
    input: &DirectorySubmission,
    trace: &TraceContext,
) -> Result<DirectoryAcceptance, DirectoryWorkflowError> {
    let r = input.locator();
    let mut scope = store
        .lock(credentials, DirectoryScopeRequest::Prepare(input))
        .await?;
    let actor = authorize(
        &scope,
        policy,
        r.company(),
        DirectoryScopeKind::Prepare(r.command_id()),
    )?;
    // The owner checks current expectations only for a new input. An authorized
    // replay after an epoch advance must reopen original accepted/terminal bytes.
    let result = scope.prepare(trace).await?;
    let accepted = result
        .status
        .accepted()
        .ok_or(DirectoryWorkflowError::Unavailable)?;
    accepted_matches(accepted, r, actor)?;
    if accepted.command().expected() != input.expected()
        || accepted.command().input() != input.input()
    {
        return Err(DirectoryWorkflowError::Unavailable);
    }
    if result.inserted {
        if !matches!(&result.status, DirectoryStatus::Pending(_)) {
            return Err(DirectoryWorkflowError::Unavailable);
        }
        current_submission(scope.authority(), input)?;
    }
    scope.finish(policy, None).await?;
    Ok(result)
}
pub async fn directory_execute<
    S: DirectoryWorkflowStore,
    P: DirectoryDecisionPort<S::Authority> + ?Sized,
>(
    store: &S,
    policy: &P,
    credentials: &S::Credentials,
    locator: DirectoryRequestRef,
    trace: &TraceContext,
) -> Result<DirectoryExecution, DirectoryWorkflowError> {
    let mut scope = store
        .lock(credentials, DirectoryScopeRequest::Execute(locator))
        .await?;
    let actor = authorize(
        &scope,
        policy,
        locator.company(),
        DirectoryScopeKind::Execute(locator.command_id()),
    )?;
    let result = scope.execute(trace).await?;
    if result.inserted && result.terminal.outcome() == DirectoryTerminalOutcomeV1::Cancelled {
        return Err(DirectoryWorkflowError::Unavailable);
    }
    accepted_matches(result.terminal.accepted(), locator, actor)?;
    if result.inserted
        && result.terminal.outcome() == DirectoryTerminalOutcomeV1::Committed
        && scope.authority().creation_expectations()
            != Some(result.terminal.accepted().command().expected())
    {
        return Err(DirectoryWorkflowError::Unavailable);
    }
    scope.finish(policy, None).await?;
    Ok(result)
}
pub async fn directory_cancel<
    S: DirectoryWorkflowStore,
    P: DirectoryDecisionPort<S::Authority> + ?Sized,
>(
    store: &S,
    policy: &P,
    credentials: &S::Credentials,
    locator: DirectoryRequestRef,
    trace: &TraceContext,
) -> Result<DirectoryExecution, DirectoryWorkflowError> {
    let mut scope = store
        .lock(credentials, DirectoryScopeRequest::Cancel(locator))
        .await?;
    let actor = authorize(
        &scope,
        policy,
        locator.company(),
        DirectoryScopeKind::Cancel(locator.command_id()),
    )?;
    let result = scope.cancel(trace).await?;
    if result.inserted
        && !matches!(
            result.terminal.outcome(),
            DirectoryTerminalOutcomeV1::Cancelled | DirectoryTerminalOutcomeV1::Expired
        )
    {
        return Err(DirectoryWorkflowError::Unavailable);
    }
    accepted_matches(result.terminal.accepted(), locator, actor)?;
    scope.finish(policy, None).await?;
    Ok(result)
}
pub async fn directory_status<
    S: DirectoryWorkflowStore,
    P: DirectoryDecisionPort<S::Authority> + ?Sized,
    A: DirectoryProofAdmission + ?Sized,
>(
    store: &S,
    policy: &P,
    credentials: &S::Credentials,
    locator: DirectoryRequestRef,
    trace: &TraceContext,
    admission: &mut A,
) -> Result<DirectoryRecovery<S::FormProof>, DirectoryWorkflowError> {
    let mut scope = store
        .lock(credentials, DirectoryScopeRequest::Status(locator))
        .await?;
    let actor = authorize(
        &scope,
        policy,
        locator.company(),
        DirectoryScopeKind::Status(locator.command_id()),
    )?;
    let recovery = scope.status(trace, admission).await?;
    if let Some(accepted) = recovery.status.accepted() {
        accepted_matches(accepted, locator, actor)?;
        if accepted.accepted_at() > scope.authority().observed_at() {
            return Err(DirectoryWorkflowError::Unavailable);
        }
    }
    match &recovery.status {
        DirectoryStatus::Pending(accepted) => {
            if recovery.proof.is_none()
                || accepted.execution_not_after() <= scope.authority().observed_at()
            {
                return Err(DirectoryWorkflowError::Unavailable);
            }
        }
        DirectoryStatus::Terminal(terminal) => {
            if recovery.proof.is_some() || terminal.terminal_at() > scope.authority().observed_at()
            {
                return Err(DirectoryWorkflowError::Unavailable);
            }
        }
        DirectoryStatus::NotVisible => {
            if recovery.proof.is_some() {
                return Err(DirectoryWorkflowError::Unavailable);
            }
        }
    }
    scope.finish(policy, recovery.proof.as_ref()).await?;
    Ok(recovery)
}

#[cfg(test)]
#[path = "workflow_tests.rs"]
mod tests;
