//! Current-authorized policy workflows over retained owning transactions.
//! Results remain provisional until final authentication and commit succeed.
mod command;
pub use command::NativePolicyCommand;

use super::business::{NativeBusinessOperationV1, PolicyAssignmentExpectationV1};
use super::org_unit_business::NativeOrgUnitActionV1;
use super::people_business::{DirectoryActionV1, NativePeoplePolicyCommandV1};
use super::{
    AccountId, CompanyPolicyDecision, CompanyPolicyDecisionPort, CurrentNativeBootstrapAuthority,
    NativeBootstrapRequestV1,
};
use console_kernel_core::{OrgId, TraceContext};
use std::future::Future;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativePolicyWorkflowError {
    InvalidInput,
    AuthenticationInvalid,
    CsrfInvalid,
    NotFound,
    Conflict,
    Capacity,
    Unavailable,
    /// Commit was not confirmed. Reopen the original command before retrying.
    Unconfirmed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NativePolicyFamily {
    Payroll,
    People,
    OrgUnit,
    CompanyInformation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NativePolicyAction {
    People(DirectoryActionV1),
    OrgUnit(NativeOrgUnitActionV1),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativePolicyCommandRef {
    company: OrgId,
    command_id: Uuid,
    operation: NativeBusinessOperationV1,
    family: NativePolicyFamily,
    action: Option<NativePolicyAction>,
}

impl NativePolicyCommandRef {
    /// A syntactically valid locator selects a resource; it grants no authority.
    pub fn new(
        company: OrgId,
        command_id: Uuid,
        operation: NativeBusinessOperationV1,
    ) -> Result<Self, NativePolicyWorkflowError> {
        if super::company_id(company).is_err() || command_id.is_nil() {
            return Err(NativePolicyWorkflowError::InvalidInput);
        }
        Ok(Self {
            company,
            command_id,
            operation,
            family: NativePolicyFamily::Payroll,
            action: None,
        })
    }

    /// Only recovery may omit a People grant/revoke action. The retained owner
    /// must resolve it from the original command before any operation proceeds.
    pub fn new_people(
        company: OrgId,
        command_id: Uuid,
        operation: NativeBusinessOperationV1,
        action: Option<DirectoryActionV1>,
    ) -> Result<Self, NativePolicyWorkflowError> {
        let mut selected = Self::new(company, command_id, operation)?;
        if operation == NativeBusinessOperationV1::Install && action.is_some() {
            return Err(NativePolicyWorkflowError::InvalidInput);
        }
        selected.family = NativePolicyFamily::People;
        selected.action = action.map(NativePolicyAction::People);
        Ok(selected)
    }

    pub fn from_command(command: impl Into<NativePolicyCommand>) -> Self {
        match command.into() {
            NativePolicyCommand::Payroll(command) => Self {
                company: command.company(),
                command_id: command.command_id(),
                operation: command.operation(),
                family: NativePolicyFamily::Payroll,
                action: None,
            },
            NativePolicyCommand::People(command) => Self::from_people_command(&command),
            NativePolicyCommand::CompanyInformation(command) => Self {
                company: command.company(),
                command_id: command.command_id(),
                operation: command.operation(),
                family: NativePolicyFamily::CompanyInformation,
                action: None,
            },
            NativePolicyCommand::OrgUnit(command) => Self {
                company: command.company(),
                command_id: command.command_id(),
                operation: command.operation(),
                family: NativePolicyFamily::OrgUnit,
                action: command.action().map(NativePolicyAction::OrgUnit),
            },
        }
    }

    pub fn from_people_command(command: &NativePeoplePolicyCommandV1) -> Self {
        Self {
            company: command.company(),
            command_id: command.command_id(),
            operation: command.operation(),
            family: NativePolicyFamily::People,
            action: command.action().map(NativePolicyAction::People),
        }
    }

    pub const fn codec_version(&self) -> i16 {
        match self.family {
            NativePolicyFamily::Payroll => 1,
            NativePolicyFamily::People => 2,
            NativePolicyFamily::OrgUnit => 3,
            NativePolicyFamily::CompanyInformation => 4,
        }
    }

    pub const fn manifest_digest(&self) -> &'static [u8; 32] {
        match self.family {
            NativePolicyFamily::Payroll => &super::business::MANIFEST,
            NativePolicyFamily::People => &super::people_business::MANIFEST,
            NativePolicyFamily::OrgUnit => &super::org_unit_business::MANIFEST,
            NativePolicyFamily::CompanyInformation => &super::company_information::MANIFEST,
        }
    }

    pub const fn directory_action(&self) -> Option<DirectoryActionV1> {
        match self.action {
            Some(NativePolicyAction::People(action)) => Some(action),
            None | Some(NativePolicyAction::OrgUnit(_)) => None,
        }
    }

    fn action_resolved(&self) -> bool {
        matches!(
            self.family,
            NativePolicyFamily::Payroll | NativePolicyFamily::CompanyInformation
        ) || self.operation == NativeBusinessOperationV1::Install
            || self.action.is_some()
    }

    /// Match a requested locator to locked original material. This is identity
    /// validation only; it does not confer any authorization.
    pub fn resolve(self, original: Self) -> Result<Self, NativePolicyWorkflowError> {
        if self.company != original.company
            || self.command_id != original.command_id
            || self.operation != original.operation
            || self.family != original.family
            || self
                .action
                .is_some_and(|action| Some(action) != original.action)
            || !original.action_resolved()
        {
            return Err(NativePolicyWorkflowError::Unavailable);
        }
        Ok(original)
    }

    pub const fn company(&self) -> OrgId {
        self.company
    }
    pub const fn command_id(&self) -> Uuid {
        self.command_id
    }
    pub const fn operation(&self) -> NativeBusinessOperationV1 {
        self.operation
    }
}

// Codec availability is not store activation. Keep OrgUnit and Company-information
// closed for every store until their owning transactions are independently admitted.
fn ensure_active_policy_family(
    selector: NativePolicyCommandRef,
) -> Result<(), NativePolicyWorkflowError> {
    match selector.family {
        NativePolicyFamily::Payroll | NativePolicyFamily::People => Ok(()),
        NativePolicyFamily::OrgUnit | NativePolicyFamily::CompanyInformation => {
            Err(NativePolicyWorkflowError::Unavailable)
        }
    }
}

pub enum NativePolicyScopeRequest<'a> {
    Current(NativePolicyCommandRef),
    Form(NativePolicyCommandRef),
    ValidationForm(NativePolicyCommandRef),
    Accept(&'a NativePolicyCommand),
    Execute(NativePolicyCommandRef),
    Status(NativePolicyCommandRef),
}

impl NativePolicyScopeRequest<'_> {
    pub fn selector(&self) -> NativePolicyCommandRef {
        match self {
            Self::Current(selector)
            | Self::Form(selector)
            | Self::ValidationForm(selector)
            | Self::Execute(selector)
            | Self::Status(selector) => *selector,
            Self::Accept(command) => NativePolicyCommandRef::from_command(*command),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativePolicyAssignmentState {
    Active,
    Revoked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativePolicyAssignmentView {
    pub expectation: PolicyAssignmentExpectationV1,
    pub role_id: Uuid,
    pub state: NativePolicyAssignmentState,
    pub valid_from: OffsetDateTime,
    pub valid_until: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativePolicyFormView {
    pub selector: NativePolicyCommandRef,
    pub group_id: Uuid,
    pub company_epoch: u64,
    pub acting_account_id: AccountId,
    pub administrative_account_id: AccountId,
    pub installed_object_type_id: Option<Uuid>,
    pub assignment: Option<NativePolicyAssignmentView>,
}

/// The Auth-owned proof is neither cloned, logged nor serialized here.
pub struct NativePolicyForm<F> {
    pub view: NativePolicyFormView,
    pub proof: F,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativePolicyAcceptedView {
    pub input: NativePolicyCommand,
    pub intake_receipt_id: Uuid,
    pub accepted_at: OffsetDateTime,
    pub execution_not_after: OffsetDateTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativePolicyRejection {
    IntakeExpired,
    RevisionConflict,
    GrantExpiryInvalid,
    RecipientIneligible,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativePolicyEffect {
    Installed {
        object_type_id: Uuid,
    },
    Granted {
        recipient: AccountId,
        assignment: NativePolicyAssignmentView,
        assignment_revision_before: Option<u64>,
    },
    Revoked {
        recipient: AccountId,
        assignment: NativePolicyAssignmentView,
        assignment_revision_before: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativePolicyOutcome {
    Committed(NativePolicyEffect),
    Rejected(NativePolicyRejection),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativePolicyTerminalView {
    pub accepted: NativePolicyAcceptedView,
    pub receipt_id: Uuid,
    pub executed_at: OffsetDateTime,
    pub epoch_before: u64,
    pub epoch_after: u64,
    pub outcome: NativePolicyOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativePolicyStatus {
    /// No currently disclosable result. This does not prove noncommit.
    NotVisible,
    AcceptedPending(NativePolicyAcceptedView),
    AcceptedExpired(NativePolicyAcceptedView),
    Terminal(NativePolicyTerminalView),
}

impl NativePolicyStatus {
    fn input(&self) -> Option<&NativePolicyCommand> {
        match self {
            Self::NotVisible => None,
            Self::AcceptedPending(accepted) | Self::AcceptedExpired(accepted) => {
                Some(&accepted.input)
            }
            Self::Terminal(terminal) => Some(&terminal.accepted.input),
        }
    }
}

pub struct NativePolicyAcceptance {
    pub inserted: bool,
    pub status: NativePolicyStatus,
}

pub struct NativePolicyExecution {
    pub inserted: bool,
    pub terminal: NativePolicyTerminalView,
}

pub trait NativePolicyWorkflowScope: Send {
    type FormProof: Send + Sync;
    fn authority(&self) -> &CurrentNativeBootstrapAuthority;
    /// The exact family/action resolved under the retained owner's locks.
    fn selector(&self) -> NativePolicyCommandRef;
    /// Each operation must reject the wrong scope mode before invoking SQL.
    fn current(
        &mut self,
    ) -> impl Future<Output = Result<NativePolicyFormView, NativePolicyWorkflowError>> + Send;
    fn form(
        &mut self,
    ) -> impl Future<Output = Result<NativePolicyForm<Self::FormProof>, NativePolicyWorkflowError>> + Send;
    fn accept(
        &mut self,
        trace: &TraceContext,
    ) -> impl Future<Output = Result<NativePolicyAcceptance, NativePolicyWorkflowError>> + Send;
    fn execute(
        &mut self,
        trace: &TraceContext,
    ) -> impl Future<Output = Result<NativePolicyExecution, NativePolicyWorkflowError>> + Send;
    fn status(
        &mut self,
    ) -> impl Future<Output = Result<NativePolicyStatus, NativePolicyWorkflowError>> + Send;
    /// Drain closure, recheck original credentials/current sources/Cedar/time,
    /// and commit the retained transaction. Recheck the exact Form proof only
    /// in Form mode; never mint a replacement during acceptance or execution.
    /// Dropping this scope rolls back, including when its future is cancelled.
    fn finish<P: CompanyPolicyDecisionPort + ?Sized>(
        self,
        policy: &P,
        proof: Option<&Self::FormProof>,
    ) -> impl Future<Output = Result<(), NativePolicyWorkflowError>> + Send;
}

pub trait NativePolicyWorkflowStore {
    type Credentials: Sync;
    type FormProof: Send + Sync;
    type Scope<'a>: NativePolicyWorkflowScope<FormProof = Self::FormProof> + Send
    where
        Self: 'a;
    /// Acquire the owner's Group-first locks and verify the actual Auth/source
    /// binding. Keep the transaction and original request until finalization.
    fn lock<'a>(
        &'a self,
        credentials: &'a Self::Credentials,
        request: NativePolicyScopeRequest<'a>,
    ) -> impl Future<Output = Result<Self::Scope<'a>, NativePolicyWorkflowError>> + Send;
}

fn authorize<P: CompanyPolicyDecisionPort + ?Sized>(
    policy: &P,
    authority: &CurrentNativeBootstrapAuthority,
    selector: NativePolicyCommandRef,
    recipient: Option<AccountId>,
) -> Result<(), NativePolicyWorkflowError> {
    use NativePolicyWorkflowError::{NotFound, Unavailable};
    let source = authority.source();
    if source.org_id != *selector.company().as_uuid() {
        return Err(Unavailable);
    }
    let recipient = match recipient {
        Some(recipient) => recipient,
        None => AccountId::from_uuid(source.administrative_account_id).map_err(|_| Unavailable)?,
    };
    let request = NativeBootstrapRequestV1::new(
        selector.company(),
        source.current_group_id,
        recipient,
        selector.operation(),
        *selector.manifest_digest(),
    )
    .map_err(|_| Unavailable)?;
    match policy
        .decide_native_bootstrap(authority, &request)
        .map_err(|_| Unavailable)?
    {
        CompanyPolicyDecision::Allow => Ok(()),
        CompanyPolicyDecision::Deny => Err(NotFound),
    }
}

/// Read current authorized context without creating an unused mutation proof.
pub async fn native_policy_current<
    S: NativePolicyWorkflowStore,
    P: CompanyPolicyDecisionPort + ?Sized,
>(
    store: &S,
    policy: &P,
    credentials: &S::Credentials,
    selector: NativePolicyCommandRef,
) -> Result<NativePolicyFormView, NativePolicyWorkflowError> {
    ensure_active_policy_family(selector)?;
    if !selector.action_resolved() {
        return Err(NativePolicyWorkflowError::InvalidInput);
    }
    let mut scope = store
        .lock(credentials, NativePolicyScopeRequest::Current(selector))
        .await?;
    let selector = selector.resolve(scope.selector())?;
    authorize(policy, scope.authority(), selector, None)?;
    let view = scope.current().await?;
    if view.selector != selector {
        return Err(NativePolicyWorkflowError::Unavailable);
    }
    scope.finish(policy, None).await?;
    Ok(view)
}

pub async fn native_policy_form<
    S: NativePolicyWorkflowStore,
    P: CompanyPolicyDecisionPort + ?Sized,
>(
    store: &S,
    policy: &P,
    credentials: &S::Credentials,
    selector: NativePolicyCommandRef,
) -> Result<NativePolicyForm<S::FormProof>, NativePolicyWorkflowError> {
    read_form(
        store,
        policy,
        credentials,
        NativePolicyScopeRequest::Form(selector),
    )
    .await
}

/// Redisplay invalid input only after validating the original mutation proof.
pub async fn native_policy_validation_form<
    S: NativePolicyWorkflowStore,
    P: CompanyPolicyDecisionPort + ?Sized,
>(
    store: &S,
    policy: &P,
    credentials: &S::Credentials,
    selector: NativePolicyCommandRef,
) -> Result<NativePolicyForm<S::FormProof>, NativePolicyWorkflowError> {
    read_form(
        store,
        policy,
        credentials,
        NativePolicyScopeRequest::ValidationForm(selector),
    )
    .await
}

async fn read_form<S: NativePolicyWorkflowStore, P: CompanyPolicyDecisionPort + ?Sized>(
    store: &S,
    policy: &P,
    credentials: &S::Credentials,
    request: NativePolicyScopeRequest<'_>,
) -> Result<NativePolicyForm<S::FormProof>, NativePolicyWorkflowError> {
    let selector = request.selector();
    ensure_active_policy_family(selector)?;
    if !selector.action_resolved() {
        return Err(NativePolicyWorkflowError::InvalidInput);
    }
    let mut scope = store.lock(credentials, request).await?;
    let selector = selector.resolve(scope.selector())?;
    authorize(policy, scope.authority(), selector, None)?;
    let form = scope.form().await?;
    if form.view.selector != selector {
        return Err(NativePolicyWorkflowError::Unavailable);
    }
    scope.finish(policy, Some(&form.proof)).await?;
    Ok(form)
}

pub async fn accept_native_policy_command<
    S: NativePolicyWorkflowStore,
    P: CompanyPolicyDecisionPort + ?Sized,
>(
    store: &S,
    policy: &P,
    credentials: &S::Credentials,
    input: impl Into<NativePolicyCommand>,
    trace: &TraceContext,
) -> Result<NativePolicyAcceptance, NativePolicyWorkflowError> {
    let input = input.into();
    let selector = NativePolicyCommandRef::from_command(&input);
    ensure_active_policy_family(selector)?;
    let mut scope = store
        .lock(credentials, NativePolicyScopeRequest::Accept(&input))
        .await?;
    let selector = selector.resolve(scope.selector())?;
    authorize(
        policy,
        scope.authority(),
        selector,
        input.recipient_account_id(),
    )?;
    let accepted = scope.accept(trace).await?;
    if accepted.status.input() != Some(&input) {
        return Err(NativePolicyWorkflowError::Unavailable);
    }
    scope.finish(policy, None).await?;
    Ok(accepted)
}

pub async fn execute_native_policy_command<
    S: NativePolicyWorkflowStore,
    P: CompanyPolicyDecisionPort + ?Sized,
>(
    store: &S,
    policy: &P,
    credentials: &S::Credentials,
    selector: NativePolicyCommandRef,
    trace: &TraceContext,
) -> Result<NativePolicyExecution, NativePolicyWorkflowError> {
    ensure_active_policy_family(selector)?;
    let mut scope = store
        .lock(credentials, NativePolicyScopeRequest::Execute(selector))
        .await?;
    let selector = selector.resolve(scope.selector())?;
    authorize(policy, scope.authority(), selector, None)?;
    let executed = scope.execute(trace).await?;
    if NativePolicyCommandRef::from_command(&executed.terminal.accepted.input) != selector {
        return Err(NativePolicyWorkflowError::Unavailable);
    }
    scope.finish(policy, None).await?;
    Ok(executed)
}

pub async fn native_policy_command_status<
    S: NativePolicyWorkflowStore,
    P: CompanyPolicyDecisionPort + ?Sized,
>(
    store: &S,
    policy: &P,
    credentials: &S::Credentials,
    selector: NativePolicyCommandRef,
) -> Result<NativePolicyStatus, NativePolicyWorkflowError> {
    ensure_active_policy_family(selector)?;
    let mut scope = store
        .lock(credentials, NativePolicyScopeRequest::Status(selector))
        .await?;
    let selector = selector.resolve(scope.selector())?;
    authorize(policy, scope.authority(), selector, None)?;
    let status = scope.status().await?;
    if status
        .input()
        .is_some_and(|input| NativePolicyCommandRef::from_command(input) != selector)
    {
        return Err(NativePolicyWorkflowError::Unavailable);
    }
    scope.finish(policy, None).await?;
    Ok(status)
}

pub async fn submit_native_policy_command<
    S: NativePolicyWorkflowStore,
    P: CompanyPolicyDecisionPort + ?Sized,
>(
    store: &S,
    policy: &P,
    credentials: &S::Credentials,
    input: impl Into<NativePolicyCommand>,
    trace: &TraceContext,
) -> Result<NativePolicyExecution, NativePolicyWorkflowError> {
    let input = input.into();
    let accepted = accept_native_policy_command(store, policy, credentials, &input, trace).await?;
    match accepted.status {
        NativePolicyStatus::Terminal(terminal) => Ok(NativePolicyExecution {
            inserted: false,
            terminal,
        }),
        NativePolicyStatus::AcceptedPending(_) | NativePolicyStatus::AcceptedExpired(_) => {
            // A has committed and released its scope. B accepts only the same
            // locator and reloads original input; no replacement bytes or retry.
            execute_native_policy_command(
                store,
                policy,
                credentials,
                NativePolicyCommandRef::from_command(&input),
                trace,
            )
            .await
        }
        NativePolicyStatus::NotVisible => Err(NativePolicyWorkflowError::Unavailable),
    }
}
