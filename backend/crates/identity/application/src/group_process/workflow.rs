//! Real owner transactions in dependency order: durable A before B, with
//! original-byte recovery and no provisional result escaping consuming finish.
use super::*;
use console_kernel_core::TraceContext;
use uuid::Uuid;

pub async fn read_group_process_landing<
    S: GroupProcessStore,
    P: GroupProcessDecisionPort + ?Sized,
>(
    store: &S,
    policy: &P,
    credentials: &S::Credentials,
    group: GroupId,
) -> Result<GroupProcessLandingViewV1, GroupProcessError> {
    let incarnation = store
        .resolve_current_incarnation(credentials, group)
        .await?;
    read_group_process_landing_bound(store, policy, credentials, group, incarnation).await
}

async fn read_group_process_landing_bound<
    S: GroupProcessStore,
    P: GroupProcessDecisionPort + ?Sized,
>(
    store: &S,
    policy: &P,
    credentials: &S::Credentials,
    group: GroupId,
    incarnation: GroupIncarnation,
) -> Result<GroupProcessLandingViewV1, GroupProcessError> {
    let mut scope = store
        .lock(
            credentials,
            GroupProcessScopeRequest::Landing { group, incarnation },
        )
        .await?;
    scope
        .authority()
        .require_method(GroupProcessMaterialModeV1::Landing)?;
    scope.authority().authorize(policy)?;
    let view = scope.landing().await?;
    let context = match &view {
        GroupProcessLandingViewV1::Empty(value) => &value.context,
        GroupProcessLandingViewV1::Current(value) => &value.context,
    };
    check_context(scope.authority(), context)?;
    let expected = scope.authority().current_view()?;
    match &view {
        GroupProcessLandingViewV1::Empty(value)
            if expected.head.is_some() || value.allowed_actions != expected.allowed_actions =>
        {
            return Err(GroupProcessError::Unavailable);
        }
        GroupProcessLandingViewV1::Current(value) if value != &expected => {
            return Err(GroupProcessError::Unavailable);
        }
        _ => {}
    }
    match &view {
        GroupProcessLandingViewV1::Empty(_)
            if scope.authority().bootstrap_stage()
                != Some(GroupProcessBootstrapStageV1::EmptyLanding) =>
        {
            return Err(GroupProcessError::Unavailable);
        }
        GroupProcessLandingViewV1::Current(value)
            if value.head.is_none()
                || scope.authority().bootstrap_stage()
                    != Some(GroupProcessBootstrapStageV1::None) =>
        {
            return Err(GroupProcessError::Unavailable);
        }
        _ => {}
    }
    scope.finish(policy, None).await?;
    Ok(view)
}

pub async fn read_current_group_process<
    S: GroupProcessStore,
    P: GroupProcessDecisionPort + ?Sized,
>(
    store: &S,
    policy: &P,
    credentials: &S::Credentials,
    group: GroupId,
) -> Result<GroupProcessCurrentView, GroupProcessError> {
    let incarnation = store
        .resolve_current_incarnation(credentials, group)
        .await?;
    let mut scope = store
        .lock(
            credentials,
            GroupProcessScopeRequest::Current { group, incarnation },
        )
        .await?;
    scope
        .authority()
        .require_method(GroupProcessMaterialModeV1::Current)?;
    scope.authority().authorize(policy)?;
    let view = scope.current().await?;
    check_current_view(scope.authority(), &view)?;
    if view.head.is_none() {
        return Err(GroupProcessError::Unavailable);
    }
    scope.finish(policy, None).await?;
    Ok(view)
}

pub async fn group_process_form<S: GroupProcessStore, P: GroupProcessDecisionPort + ?Sized>(
    store: &S,
    policy: &P,
    credentials: &S::Credentials,
    group: GroupId,
) -> Result<GroupProcessForm<S::FormProof>, GroupProcessError> {
    group_process_form_bound(store, policy, credentials, group, None).await
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroupProcessFormTargetV1 {
    Adopt,
    Replace { process_id: Uuid },
    Suspend { process_id: Uuid },
}
impl GroupProcessFormTargetV1 {
    /// Target selection is a precondition, never a grant. Check the same rule
    /// during preliminary admission and again before retained proof issuance.
    pub fn require(
        self,
        current: Option<Uuid>,
        actions: &[GroupProcessActionV1],
    ) -> Result<(), GroupProcessError> {
        let action = match self {
            Self::Adopt if current.is_none() => GroupProcessActionV1::Adopt,
            Self::Replace { process_id } if current == Some(process_id) && !process_id.is_nil() => {
                GroupProcessActionV1::Adopt
            }
            Self::Suspend { process_id } if current == Some(process_id) && !process_id.is_nil() => {
                GroupProcessActionV1::Suspend
            }
            _ => return Err(GroupProcessError::NotFound),
        };
        if actions.contains(&action) {
            Ok(())
        } else {
            Err(GroupProcessError::NotFound)
        }
    }
}

pub async fn group_process_form_for<S: GroupProcessStore, P: GroupProcessDecisionPort + ?Sized>(
    store: &S,
    policy: &P,
    credentials: &S::Credentials,
    group: GroupId,
    target: GroupProcessFormTargetV1,
) -> Result<GroupProcessForm<S::FormProof>, GroupProcessError> {
    group_process_form_bound(store, policy, credentials, group, Some(target)).await
}

async fn group_process_form_bound<S: GroupProcessStore, P: GroupProcessDecisionPort + ?Sized>(
    store: &S,
    policy: &P,
    credentials: &S::Credentials,
    group: GroupId,
    target: Option<GroupProcessFormTargetV1>,
) -> Result<GroupProcessForm<S::FormProof>, GroupProcessError> {
    let incarnation = store
        .resolve_current_incarnation(credentials, group)
        .await?;
    let mut scope = store
        .lock(
            credentials,
            GroupProcessScopeRequest::Form { group, incarnation },
        )
        .await?;
    scope
        .authority()
        .require_method(GroupProcessMaterialModeV1::Form)?;
    scope.authority().authorize(policy)?;
    if let Some(target) = target {
        let view = scope.authority().current_view()?;
        target.require(
            view.head.as_ref().map(|head| head.reference.process_id()),
            &view.allowed_actions,
        )?;
    }
    let form = scope.form().await?;
    check_current_view(scope.authority(), &form.view)?;
    nonnil(form.command_id)?;
    nonnil(form.process_id)?;
    if form
        .view
        .head
        .as_ref()
        .is_some_and(|head| head.reference.process_id() != form.process_id)
    {
        return Err(GroupProcessError::Unavailable);
    }
    scope.finish(policy, Some(&form.proof)).await?;
    Ok(form)
}

/// Redisplay invalid input with its original selectors and submitted proof.
/// Current source still governs disclosure; no replacement command is minted.
pub async fn group_process_validation_form<
    S: GroupProcessStore,
    P: GroupProcessDecisionPort + ?Sized,
>(
    store: &S,
    policy: &P,
    credentials: &S::Credentials,
    original: GroupProcessLocator,
    process_id: Uuid,
) -> Result<GroupProcessForm<S::FormProof>, GroupProcessError> {
    nonnil(original.command_id())?;
    nonnil(process_id)?;
    let mut scope = store
        .lock(
            credentials,
            GroupProcessScopeRequest::ValidationForm {
                original,
                process_id,
            },
        )
        .await?;
    scope
        .authority()
        .require_method(GroupProcessMaterialModeV1::Form)?;
    scope.authority().authorize(policy)?;
    let form = scope.form().await?;
    check_current_view(scope.authority(), &form.view)?;
    if form.command_id != original.command_id() || form.process_id != process_id {
        return Err(GroupProcessError::Unavailable);
    }
    scope.finish(policy, Some(&form.proof)).await?;
    Ok(form)
}

pub async fn prepare_group_process<S: GroupProcessStore, P: GroupProcessDecisionPort + ?Sized>(
    store: &S,
    policy: &P,
    credentials: &S::Credentials,
    input: &GroupProcessCommandV1,
    trace: &TraceContext,
) -> Result<GroupProcessAcceptance, GroupProcessError> {
    let mut scope = store
        .lock(credentials, GroupProcessScopeRequest::Accept(input))
        .await?;
    scope
        .authority()
        .require_method(GroupProcessMaterialModeV1::Accept)?;
    scope.authority().authorize(policy)?;
    let actor = scope.authority().account();
    let accepted = scope.accept(trace).await?;
    let original = accepted
        .status
        .accepted()
        .ok_or(GroupProcessError::Unavailable)?;
    check_original(actor, GroupProcessLocator::from_command(input), original)?;
    if &original.input != input || original.input_bytes != input.encode(actor)? {
        return Err(GroupProcessError::Conflict);
    }
    if accepted.inserted && !matches!(accepted.status, GroupProcessStatus::AcceptedPending(_)) {
        return Err(GroupProcessError::Unavailable);
    }
    scope.finish(policy, None).await?;
    Ok(accepted)
}

pub async fn execute_group_process<S: GroupProcessStore, P: GroupProcessDecisionPort + ?Sized>(
    store: &S,
    policy: &P,
    credentials: &S::Credentials,
    original: GroupProcessLocator,
    trace: &TraceContext,
) -> Result<GroupProcessExecution, GroupProcessError> {
    let mut scope = store
        .lock(credentials, GroupProcessScopeRequest::Execute(original))
        .await?;
    scope
        .authority()
        .require_method(GroupProcessMaterialModeV1::Execute)?;
    scope.authority().authorize(policy)?;
    let actor = scope.authority().account();
    let execution = scope.execute(trace).await?;
    check_original(actor, original, &execution.terminal.accepted)?;
    check_execution(scope.authority(), &execution)?;
    scope.finish(policy, None).await?;
    Ok(execution)
}

/// The confirmed A locator survives any unconfirmed B commit. No recovery path
/// substitutes a new command or newly observed expected revisions.
pub async fn submit_group_process<S: GroupProcessStore, P: GroupProcessDecisionPort + ?Sized>(
    store: &S,
    policy: &P,
    credentials: &S::Credentials,
    input: &GroupProcessCommandV1,
    trace: &TraceContext,
) -> Result<GroupProcessExecution, GroupProcessError> {
    let accepted = prepare_group_process(store, policy, credentials, input, trace).await?;
    if let GroupProcessStatus::Terminal(terminal) = accepted.status {
        // A replay was already authorized and finalized against original bytes;
        // it does not invent another effect transaction or rewrite its result.
        return Ok(GroupProcessExecution {
            inserted: false,
            terminal,
        });
    }
    execute_group_process(
        store,
        policy,
        credentials,
        GroupProcessLocator::from_command(input),
        trace,
    )
    .await
}

pub async fn group_process_status<S: GroupProcessStore, P: GroupProcessDecisionPort + ?Sized>(
    store: &S,
    policy: &P,
    credentials: &S::Credentials,
    requested: GroupProcessRouteSelectorV1,
) -> Result<GroupProcessStatus, GroupProcessError> {
    let original = resolve_original(store, credentials, requested).await?;
    let mut scope = store
        .lock(credentials, GroupProcessScopeRequest::Status(original))
        .await?;
    scope
        .authority()
        .require_method(GroupProcessMaterialModeV1::Status)?;
    scope.authority().authorize(policy)?;
    let status = scope.status().await?;
    let accepted = status.accepted().ok_or(GroupProcessError::NotFound)?;
    check_original(scope.authority().account(), original, accepted)?;
    scope.finish(policy, None).await?;
    Ok(status)
}

/// HTTP performs preliminary own-status admission and the one existing Auth
/// limiter admission before this retained own-form scope. This use case does
/// not mint IDs, execute pending work or require current designation.
pub async fn group_process_retry_form<
    S: GroupProcessStore,
    P: GroupProcessDecisionPort + ?Sized,
>(
    store: &S,
    policy: &P,
    credentials: &S::Credentials,
    requested: GroupProcessRouteSelectorV1,
) -> Result<GroupProcessOwnRetryFormV1<S::FormProof>, GroupProcessError> {
    let original = resolve_original(store, credentials, requested).await?;
    let mut scope = store
        .lock(
            credentials,
            GroupProcessScopeRequest::OwnRetryForm(original),
        )
        .await?;
    scope
        .authority()
        .require_method(GroupProcessMaterialModeV1::OwnRetryForm)?;
    scope.authority().authorize(policy)?;
    let form = scope.retry_form().await?;
    if form.original != original {
        return Err(GroupProcessError::Unavailable);
    }
    scope.finish(policy, Some(&form.proof)).await?;
    Ok(form)
}

pub async fn retry_group_process<S: GroupProcessStore, P: GroupProcessDecisionPort + ?Sized>(
    store: &S,
    policy: &P,
    credentials: &S::Credentials,
    requested: GroupProcessRouteSelectorV1,
    trace: &TraceContext,
) -> Result<GroupProcessExecution, GroupProcessError> {
    let original = resolve_original(store, credentials, requested).await?;
    let mut scope = store
        .lock(
            credentials,
            GroupProcessScopeRequest::RetryResolve(original),
        )
        .await?;
    scope
        .authority()
        .require_method(GroupProcessMaterialModeV1::RetryResolve)?;
    // Terminal is read-own with the captured endpoint proof; pending is a
    // separately reconstructed full current mutation. Status never chooses it.
    scope.authority().authorize(policy)?;
    let actor = scope.authority().account();
    let resolution = scope.retry_resolve().await?;
    let result = match resolution {
        GroupProcessRetryResolution::Terminal(terminal) => {
            if scope.authority().finish_purpose() != GroupProcessFinishPurposeV1::TerminalReplay {
                return Err(GroupProcessError::Unavailable);
            }
            check_original(actor, original, &terminal.accepted)?;
            let GroupProcessStatus::Terminal(locked) = scope
                .authority()
                .original_status()
                .ok_or(GroupProcessError::Unavailable)?
            else {
                return Err(GroupProcessError::Unavailable);
            };
            if locked != &terminal {
                return Err(GroupProcessError::Unavailable);
            }
            GroupProcessExecution {
                inserted: false,
                terminal,
            }
        }
        GroupProcessRetryResolution::Pending(accepted) => {
            if scope.authority().finish_purpose() != GroupProcessFinishPurposeV1::Execute {
                return Err(GroupProcessError::Unavailable);
            }
            check_original(actor, original, &accepted)?;
            let execution = scope.execute(trace).await?;
            if execution.terminal.accepted != accepted {
                return Err(GroupProcessError::Unavailable);
            }
            check_execution(scope.authority(), &execution)?;
            execution
        }
        GroupProcessRetryResolution::NotVisible => return Err(GroupProcessError::NotFound),
    };
    scope.finish(policy, None).await?;
    Ok(result)
}

pub async fn discover_group_process_navigation<
    S: GroupProcessStore,
    P: GroupProcessDecisionPort + ?Sized,
>(
    store: &S,
    policy: &P,
    credentials: &S::Credentials,
) -> Result<Vec<GroupProcessNavigationViewV1>, GroupProcessError> {
    for attempt in 0..2 {
        // Enumeration finishes before any Group-first retained scope starts.
        let hints = store.enumerate_navigation_candidates(credentials).await?;
        let mut views = Vec::with_capacity(hints.candidates().len());
        for candidate in hints.candidates() {
            let landing = read_group_process_landing_bound(
                store,
                policy,
                credentials,
                candidate.group(),
                candidate.incarnation(),
            )
            .await;
            let context = match landing {
                Ok(GroupProcessLandingViewV1::Empty(view)) => view.context,
                Ok(GroupProcessLandingViewV1::Current(view)) => view.context,
                Err(GroupProcessError::NotFound) => continue,
                Err(error) => return Err(error),
            };
            views.push(GroupProcessNavigationViewV1 {
                group: context.group,
                incarnation: context.incarnation,
                label: context.label,
                revision: context.revision,
            });
        }
        match store
            .recheck_navigation_candidates(credentials, hints.snapshot())
            .await
        {
            Ok(()) => return Ok(views),
            Err(GroupProcessError::Conflict) if attempt == 0 => continue,
            Err(GroupProcessError::Conflict) => return Err(GroupProcessError::Unavailable),
            Err(error) => return Err(error),
        }
    }
    Err(GroupProcessError::Unavailable)
}

fn check_current_view(
    authority: &CurrentGroupProcessAuthority,
    view: &GroupProcessCurrentView,
) -> Result<(), GroupProcessError> {
    if view != &authority.current_view()? {
        return Err(GroupProcessError::Unavailable);
    }
    Ok(())
}

fn check_context(
    authority: &CurrentGroupProcessAuthority,
    context: &GroupProcessContextV1,
) -> Result<(), GroupProcessError> {
    let current = authority
        .current_material()
        .ok_or(GroupProcessError::Unavailable)?;
    if current.context() != context {
        return Err(GroupProcessError::Unavailable);
    }
    Ok(())
}
fn check_original(
    actor: AccountId,
    locator: GroupProcessLocator,
    accepted: &GroupProcessAcceptedV1,
) -> Result<(), GroupProcessError> {
    if accepted.actor != actor || accepted.locator() != locator {
        return Err(GroupProcessError::Unavailable);
    }
    let (stored_actor, input) = GroupProcessCommandV1::decode(&accepted.input_bytes)?;
    if stored_actor != actor
        || input != accepted.input
        || input.encode(actor)? != accepted.input_bytes
    {
        return Err(GroupProcessError::Unavailable);
    }
    nonnil(accepted.intake_receipt)?;
    time_from_us(accepted.accepted_at_us)?;
    Ok(())
}
pub(super) fn check_execution(
    authority: &CurrentGroupProcessAuthority,
    execution: &GroupProcessExecution,
) -> Result<(), GroupProcessError> {
    let current = authority
        .current_material()
        .ok_or(GroupProcessError::Unavailable)?;
    let result = &execution.terminal.result;
    let binding = authority.binding();
    if result.actor() != binding.account()
        || result.session() != binding.session()
        || result.security_generation() != binding.security_generation()
        || result.effect_xid8() != binding.xid8()
        || result.effect_pid() != binding.backend_pid()
        || result.designation_receipt() != current.designation_receipt()
        || result.designation_revision() != current.designation_revision()
        || result.observed_group_revision() != current.context().revision
        || result.policy_before()
            != authority
                .policy_head()
                .ok_or(GroupProcessError::Unavailable)?
        || result.before_head() != current.head().map(|head| &head.reference)
        || result.evaluated_bundle() != authority.evaluated_bundle()
        || result.encode() != execution.terminal.result_bytes
    {
        return Err(GroupProcessError::Unavailable);
    }
    let plan = classify_group_process_transition_at_v1(authority, result.executed_at_us())?;
    match plan {
        GroupProcessTransitionPlanV1::Refuse(code) if result.terminal_code() != code => {
            return Err(GroupProcessError::Unavailable);
        }
        GroupProcessTransitionPlanV1::FirstAdopt {
            content_version,
            head_revision,
        }
        | GroupProcessTransitionPlanV1::Replace {
            content_version,
            head_revision,
        }
        | GroupProcessTransitionPlanV1::Suspend {
            content_version,
            head_revision,
        } => {
            let expected_code = match plan {
                GroupProcessTransitionPlanV1::FirstAdopt { .. } => ProcessTerminalCodeV1::Adopted,
                GroupProcessTransitionPlanV1::Replace { .. } => ProcessTerminalCodeV1::Replaced,
                GroupProcessTransitionPlanV1::Suspend { .. } => ProcessTerminalCodeV1::Suspended,
                _ => return Err(GroupProcessError::Unavailable),
            };
            let after = result.after_head().ok_or(GroupProcessError::Unavailable)?;
            if result.terminal_code() != expected_code
                || after.content_version() != content_version
                || after.head_revision() != head_revision
            {
                return Err(GroupProcessError::Unavailable);
            }
        }
        _ => {}
    }
    Ok(())
}

async fn resolve_original<S: GroupProcessStore>(
    store: &S,
    credentials: &S::Credentials,
    requested: GroupProcessRouteSelectorV1,
) -> Result<GroupProcessLocator, GroupProcessError> {
    let original = store
        .resolve_original_locator(credentials, requested)
        .await?;
    if original.group() != requested.group() || original.command_id() != requested.command_id() {
        return Err(GroupProcessError::Unavailable);
    }
    Ok(original)
}

/// Parse a route UUID without accepting alternate text encodings. Selectors
/// never establish Account or Group authority.
pub fn canonical_group_process_uuid(raw: &str) -> Result<Uuid, GroupProcessError> {
    let value = Uuid::parse_str(raw).map_err(|_| GroupProcessError::InvalidInput)?;
    if value.is_nil() || value.hyphenated().to_string() != raw {
        return Err(GroupProcessError::InvalidInput);
    }
    Ok(value)
}
