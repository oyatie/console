//! Proposed sequential test-composition source; not compiled/executed evidence.
//! Auth contexts come only from actual owner credentials/resolvers. Source/profile/
//! scope/evidence choices require actual owner bootstrap; no fake verification.
use console_ontology_application::action30::*;
use console_payroll_adapter_postgres::action30 as payroll;
use console_governance_adapter_postgres::action30 as governance;
use console_platform_request_context::{account::AuthenticatedCompanyContext, worker::AuthenticatedWorkerContext};
use sqlx::PgPool;
use std::time::Duration;
use crate::binder_sequence::{bind_and_seal, completed, ProducerStop};

#[derive(Clone, serde::Deserialize)]
pub struct NativeUserChoices {
    pub create_selection: UntrustedActionTargetSelection,
    pub create: payroll::RunCreate, // unsealed present choices; no expected future run
    pub source_refs: Vec<ExactEvidence>,
    pub preparation_evidence: Vec<ExactEvidence>,
    pub review_evidence: Vec<ExactEvidence>,
    pub review_decision: governance::ReviewDecision,
    pub review_reason: String,
}
pub struct ActualNativeOwnerResults {
    pub created: payroll::RunResult,
    pub prepared: payroll::RunResult,
    pub closed: payroll::RunResult,
    pub calculated: payroll::RunResult,
    pub review: payroll::ReviewResult,
    pub decisions: Vec<governance::ReviewResult>,
    pub completion: payroll::CompletionRef,
}

pub async fn complete_native(
    pool: &PgPool, auth: &AuthenticatedCompanyContext, worker: &AuthenticatedWorkerContext,
    attempt: &AttemptRef, initial: OwnerExecution<payroll::RunResult>,
) -> Result<payroll::RunResult, ProducerStop> {
    let mut pending = match initial {
        OwnerExecution::Completed(result) => return completed_native_result(result, attempt),
        OwnerExecution::Observation(value @ ResultObservation::Pending { .. }) => value,
        OwnerExecution::Observation(value) => return Err(ProducerStop::Observation(value)),
    };
    let original_subject = pending.subject().expect("Pending always has subject").clone();
    match &original_subject {
        ReconciliationSubject::CompanyCommand { command } if command.org_id==attempt.org_id && command.command_id==attempt.command_id => {},
        ReconciliationSubject::Attempt { attempt: observed } if observed==attempt => {},
        _ => return Err(ProducerStop::InvalidActualResult("pending subject not original attempt command")),
    }
    // Both iteration and wall limits bound fixture progression. Each owner
    // operation also uses its approved claim/chunk/statement/lock budgets.
    // Timeout is an inconclusive same-subject stop; never rollback or replacement.
    let progression = async {
        for _ in 0..64 {
            payroll::advance_native_preparation_once(pool, worker, &original_subject).await?;
            match payroll::observe_native_command(pool, auth, &original_subject).await? {
                OwnerExecution::Completed(result) => return completed_native_result(result, attempt),
                OwnerExecution::Observation(value @ ResultObservation::Pending { .. }) => {
                    if value.subject() != Some(&original_subject) {
                        return Err(ProducerStop::InvalidActualResult("changed original subject"));
                    }
                    pending = value;
                }
                OwnerExecution::Observation(value) => return Err(ProducerStop::Observation(value)),
            }
        }
        Err(ProducerStop::Observation(pending.clone()))
    };
    match tokio::time::timeout(Duration::from_secs(60), progression).await {
        Ok(result) => result,
        Err(_) => Err(ProducerStop::Observation(ResultObservation::unknown_after_interrupted_wait(original_subject))),
    }
}

pub async fn produce_native_and_review(
    pool: &PgPool, submitter: &AuthenticatedCompanyContext,
    independent_reviewer: &AuthenticatedCompanyContext,
    worker: &AuthenticatedWorkerContext, choices: NativeUserChoices,
) -> Result<ActualNativeOwnerResults, ProducerStop> {
    let create_attempt = bind_and_seal(pool, submitter, &choices.create_selection, &choices.create).await?;
    let created = completed(payroll::payroll_create_run(pool, submitter, &create_attempt).await?)?;
    if created.state != payroll::RunState::Staged || created.payable {
        return Err(ProducerStop::InvalidActualResult("create result"));
    }
    // Selection names an actual object from an actual preceding receipt. It is
    // still untrusted; each binder resolves fresh registered target authority.
    let run_selection = UntrustedActionTargetSelection::existing_run(created.run_id);
    let prepare_control = payroll::read_preparation_control(pool, submitter, created.run_id).await?;
    let prepare_input = payroll::NativePrepare {
        expected: prepare_control.expected, // ALL six RunExpected fields
        expected_membership_revision: prepare_control.expected_membership_revision,
        scope_basis_ref: prepare_control.scope_basis_ref,
        profile_ref: prepare_control.profile_ref,
        source_refs: choices.source_refs, evidence_refs: choices.preparation_evidence,
    };
    let prepare_attempt = bind_and_seal(pool, submitter, &run_selection, &prepare_input).await?;
    let prepared = complete_native(pool, submitter, worker, &prepare_attempt,
        payroll::payroll_prepare_inputs(pool, submitter, &prepare_attempt).await?).await?;
    if prepared.state != payroll::RunState::Staged || prepared.input.is_none() {
        return Err(ProducerStop::InvalidActualResult("preparation not closeable"));
    }
    // Complete exact close basis is an actual current owner read, never guessed
    // from RunResult (which has no close_basis or review_cycle_id).
    let close_control = payroll::read_attendance_close_control(pool, submitter, created.run_id).await?;
    let close_input = payroll::CloseAttendance {
        expected: close_control.expected, close_basis: close_control.close_basis, attest: true,
    };
    let close_attempt = bind_and_seal(pool, submitter, &run_selection, &close_input).await?;
    let closed = completed(payroll::payroll_close_attendance(pool, submitter, &close_attempt).await?)?;
    if closed.state != payroll::RunState::AttendanceClosed {
        return Err(ProducerStop::InvalidActualResult("attendance not closed"));
    }
    let calculate_input = payroll::RunCalculate {
        expected: payroll::read_run_control(pool, submitter, created.run_id).await?,
    };
    let calculate_attempt = bind_and_seal(pool, submitter, &run_selection, &calculate_input).await?;
    let calculated = complete_native(pool, submitter, worker, &calculate_attempt,
        payroll::payroll_calculate_run(pool, submitter, &calculate_attempt).await?).await?;
    if calculated.state != payroll::RunState::Calculated || calculated.payable || calculated.batch.is_none() {
        return Err(ProducerStop::InvalidActualResult("calculation result"));
    }
    let open_input = payroll::ReviewOpen {
        expected: payroll::read_run_control(pool, submitter, created.run_id).await?,
    };
    let open_attempt = bind_and_seal(pool, submitter, &run_selection, &open_input).await?;
    let review = completed(payroll::payroll_open_review_cycle(pool, submitter, &open_attempt).await?)?;
    // Owner reads the bounded COMPLETE basis set for exact cycle membership;
    // incomplete pagination/overbudget is a refusal, never a truncated success.
    let bases = payroll::read_complete_review_bases(pool, submitter, &review.cycle).await?;
    let mut decisions = Vec::new();
    for basis in bases {
        let basis_selection = UntrustedActionTargetSelection::existing_review_basis(basis.basis_id);
        let submit_input = governance::ReviewSubmit {
            basis: basis.clone(), attest: true, submitted_evidence: choices.review_evidence.clone(),
        };
        let submit_attempt = bind_and_seal(pool, submitter, &basis_selection, &submit_input).await?;
        let submitted = completed(governance::review_submit(pool, submitter, &submit_attempt).await?)?;
        let request = submitted.request.ok_or(ProducerStop::InvalidActualResult("review request missing"))?;
        // Decision is constructed NOW from actual returned request, then sealed
        // as the independent reviewer's original command. Human/credential/
        // task independence is the actual governance guard, not UUID inequality.
        let decision_selection = UntrustedActionTargetSelection::existing_review_basis(basis.basis_id);
        let decision_input = governance::ReviewDecide {
            request, basis, decision: choices.review_decision.clone(), reason: choices.review_reason.clone(),
        };
        let decision_attempt = bind_and_seal(pool, independent_reviewer, &decision_selection, &decision_input).await?;
        let decided = completed(governance::review_decide(pool, independent_reviewer, &decision_attempt).await?)?;
        if decided.decision_id.is_none() {
            return Err(ProducerStop::InvalidActualResult("decision missing"));
        }
        decisions.push(decided);
    }
    let completion = payroll::read_review_completion(pool, submitter, &review.cycle).await?;
    Ok(ActualNativeOwnerResults { created, prepared, closed, calculated, review, decisions, completion })
}

fn completed_native_result(result: payroll::RunResult, attempt: &AttemptRef) -> Result<payroll::RunResult,ProducerStop> {
    match &result.receipt {
        ReceiptRef::Company { command, .. } if command.org_id==attempt.org_id && command.command_id==attempt.command_id => Ok(result),
        _ => Err(ProducerStop::InvalidActualResult("terminal receipt not original attempt command")),
    }
}
