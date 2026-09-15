//! Test-side producer against the explicit proposed real-owner interfaces.
//! This module belongs in the test composition root; it is not an adapter-to-adapter runtime dependency.
//! Missing interfaces/current bootstrap are prerequisites, not an executable product RED claim.
use console_ontology_application::action30::RegisteredActionRequest;
use console_payroll_adapter_postgres::action30 as payroll;
use console_governance_adapter_postgres::action30 as governance;
use console_platform_authz::Principal;
use sqlx::PgPool;

// All request identities below come from actual existing direct admission or
// draft/attempt owner protocols. Supplying this plain struct grants nothing:
// every invoked command owner must load and compare the actual stored custody.
pub struct NativeOwnerRecipe {
    pub create: RegisteredActionRequest<payroll::RunCreate>,
    pub prepare: RegisteredActionRequest<payroll::NativePrepare>,
    pub close: RegisteredActionRequest<payroll::CloseAttendance>,
    pub calculate: RegisteredActionRequest<payroll::RunCalculate>,
    pub open_review: RegisteredActionRequest<payroll::ReviewOpen>,
}

pub struct ActualNativeOwnerResults {
    pub created: payroll::RunResult,
    pub prepared: payroll::RunResult,
    pub closed: payroll::RunResult,
    pub calculated: payroll::RunResult,
    pub review: payroll::ReviewResult,
}

// This is intentionally a direct sequence of actual command entries; no fixture
// owner, callback, verified flag, SQL effect insertion, or expected result path.
// Each recipe's identity must be freshly admitted by the real command binder
// AFTER rebinding its input from the preceding actual result. A previously sealed
// identity cannot be reused with these changed expected fields. This sequencing
// requirement is enforced by the private current-context issuer in each command.
pub async fn produce_native_through_review_open(
    pool: &PgPool,
    principal: &Principal,
    recipe: NativeOwnerRecipe,
) -> Result<ActualNativeOwnerResults, Box<dyn std::error::Error + Send + Sync>> {
    let created = payroll::payroll_create_run(pool, principal, recipe.create).await?;
    assert_eq!(created.state, payroll::RunState::Staged);
    assert!(!created.payable);
    // These exact expectations are already captured from the preceding real
    // owner in the draft binder recipe. Refuse an operator-invented stale chain.
    assert_eq!(recipe.prepare.input.expected.run_id, created.run_id);
    assert_eq!(recipe.prepare.input.expected.run_revision, created.run_revision);
    let prepared = payroll::payroll_prepare_inputs(pool, principal, recipe.prepare).await?;
    assert!(prepared.input.is_some());
    assert_eq!(recipe.close.input.expected.run_id, prepared.run_id);
    assert_eq!(recipe.close.input.expected.run_revision, prepared.run_revision);
    let closed = payroll::payroll_close_attendance(pool, principal, recipe.close).await?;
    assert_eq!(closed.state, payroll::RunState::AttendanceClosed);
    assert_eq!(recipe.calculate.input.expected.run_revision, closed.run_revision);
    let calculated = payroll::payroll_calculate_run(pool, principal, recipe.calculate).await?;
    assert_eq!(calculated.state, payroll::RunState::Calculated);
    assert!(!calculated.payable);
    assert!(calculated.batch.is_some());
    assert_eq!(recipe.open_review.input.expected.run_revision, calculated.run_revision);
    let review = payroll::payroll_open_review_cycle(pool, principal, recipe.open_review).await?;
    Ok(ActualNativeOwnerResults {created,prepared,closed,calculated,review})
}

// Submit/decide identities must be issued after the actual basis/request exists,
// with actual independent Human/credential authority; this function cannot issue it.
pub async fn submit_then_decide_actual_basis(
    pool: &PgPool,
    submitter: &Principal,
    independent_reviewer: &Principal,
    submit: RegisteredActionRequest<governance::ReviewSubmit>,
    decide: RegisteredActionRequest<governance::ReviewDecide>,
) -> Result<governance::ReviewResult, Box<dyn std::error::Error + Send + Sync>> {
    let submitted = governance::review_submit(pool, submitter, submit).await?;
    let actual_request = submitted.request.as_ref().ok_or("actual request missing")?;
    assert_eq!(&decide.input.request, actual_request);
    // Private owner guards must check Human independence, not Principal UUID inequality.
    let decision = governance::review_decide(pool, independent_reviewer, decide).await?;
    assert!(decision.decision_id.is_some());
    Ok(decision)
}
