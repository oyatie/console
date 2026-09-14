//! Proposed concrete owner-call fixture source. Not a dummy owner implementation.
//! Exact interface prerequisites: owner-interface-refinement.md. Do not claim this compiles today.
use console_platform_authz::Principal;
use console_ontology_application::action30::RegisteredActionRequest;
use sqlx::PgPool;
use console_ontology_adapter_postgres::action30 as ontology;
use console_ontology_adapter_postgres::group_action30 as group;
use console_payroll_adapter_postgres::action30 as payroll;
use console_governance_adapter_postgres::action30 as governance;
use console_attendance_adapter_postgres::action30 as attendance;
use console_workflow_adapter_postgres::action30 as workflow;
use console_docs_adapter_postgres::action30 as docs;
use console_leave_adapter_postgres::action30 as leave;

// Every function calls a fixed real proposed command entry in its responsible owner.
// The caller receives actual typed results, never preloaded expected JSON.

// Before: EDITABLE account-custodied draft at exact head/epoch; source_note TEXT field editable
// Durable: Append one acknowledged content revision; source_note becomes 계약 근거 메모; rotate continuation; no domain wage effect
// Refusal: Stale expected_revision_id refuses and preserves current head
pub async fn draft_save(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<ontology::DraftSave>) -> Result<ontology::DraftAck, Box<dyn std::error::Error + Send + Sync>> {
    Ok(ontology::draft_save(pool, principal, request).await?)
}

// Before: EDITABLE draft plus retained permitted older revision with selected source_note
// Durable: Append new revision copying only selected exact older field; retain both historical revisions
// Refusal: Hidden source field or different Company revision refuses
pub async fn draft_restore(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<ontology::DraftRestore>) -> Result<ontology::DraftAck, Box<dyn std::error::Error + Send + Sync>> {
    Ok(ontology::draft_restore(pool, principal, request).await?)
}

// Before: Original READY attempt; owner receipt proves definitive no effect
// Durable: Attempt becomes CANCELLED exactly once; no domain write; preserve submitted content
// Refusal: Lost/unknown owner outcome must not become CANCELLED
pub async fn attempt_cancel(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<ontology::AttemptCancel>) -> Result<ontology::ResultObservation, Box<dyn std::error::Error + Send + Sync>> {
    Ok(ontology::attempt_cancel(pool, principal, request).await?)
}

// Before: Registered nonproduction profile and scope; no command receipt or allocated run
// Durable: One STAGED payable=false run with exact period/pay date/profile/scope and one receipt
// Refusal: Same command id with changed period refuses without second run
pub async fn payroll_create_run(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<payroll::RunCreate>) -> Result<payroll::RunResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(payroll::payroll_create_run(pool, principal, request).await?)
}

// Before: Current same-Company Employee+Employment; source generation1; real account authority
// Durable: Append one immutable monthly wage3000000/209; increment wage source guard; no money result
// Refusal: Different-Company employment refuses atomically
pub async fn payroll_create_contract_wage(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<payroll::WageCreate>) -> Result<payroll::WageResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(payroll::payroll_create_contract_wage(pool, principal, request).await?)
}

// Before: Immutable base wage3000000 and current correction head; permitted original actor
// Durable: Append one correction3100000/209; advance exact head and source guard; preserve original wage
// Refusal: Mismatched base Company refuses; no correction/head change
pub async fn payroll_correct_contract_wage(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<payroll::WageCorrect>) -> Result<payroll::WageResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(payroll::payroll_correct_contract_wage(pool, principal, request).await?)
}

// Before: STAGED run with full one-member population and source fixtures resolved by owners
// Durable: Private BUILDING completes/seals/activates one full native input under original command; no money row
// Refusal: Changed population/source guard at finalization refuses activation and preserves acknowledged intent
pub async fn payroll_prepare_inputs(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<payroll::NativePrepare>) -> Result<payroll::RunResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(payroll::payroll_prepare_inputs(pool, principal, request).await?)
}

// Before: Current ATTENDANCE_CLOSED run and exact supplied run/input revision
// Durable: Run reopened with monotonic revision; historical input/close refs remain
// Refusal: Stale close/input head cannot reopen newer state
pub async fn payroll_reopen_inputs(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<payroll::RunReopen>) -> Result<payroll::RunResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(payroll::payroll_reopen_inputs(pool, principal, request).await?)
}

// Before: Resolved activated input and exact same membership close basis; attest=true
// Durable: ATTENDANCE_CLOSED payable=false; retain exact close basis/manifest and receipt
// Refusal: Blocked member or incomplete close member set prevents close
pub async fn payroll_close_attendance(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<payroll::CloseAttendance>) -> Result<payroll::RunResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(payroll::payroll_close_attendance(pool, principal, request).await?)
}

// Before: ATTENDANCE_CLOSED run and current exact resolved native input
// Durable: CALCULATED payable=false; exactly one outcome per member, existing money row only for SUCCESS
// Refusal: Semantic digest substituted for input custody digest refuses
pub async fn payroll_calculate_run(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<payroll::RunCalculate>) -> Result<payroll::RunResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(payroll::payroll_calculate_run(pool, principal, request).await?)
}

// Before: CALCULATED run with exact batch and current registered review derivation
// Durable: REVIEWING with one complete immutable cycle/member/basis set and receipt
// Refusal: Incomplete basis membership prevents cycle completion
pub async fn payroll_open_review_cycle(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<payroll::ReviewOpen>) -> Result<payroll::ReviewResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(payroll::payroll_open_review_cycle(pool, principal, request).await?)
}

// Before: Current open basis and evidence contract; original submitter authorized
// Durable: One governance request bound to exact basis; submitted evidence protected atomically
// Refusal: Wrong basis digest refuses without request/evidence mutation
pub async fn review_submit(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<governance::ReviewSubmit>) -> Result<governance::ReviewResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(governance::review_submit(pool, principal, request).await?)
}

// Before: Pending request; independent actual reviewer Human and applicable credential current
// Durable: Append APPROVE decision for exact basis/request; completion only when all obligations met
// Refusal: Same Human through another Account cannot approve own request
pub async fn review_decide(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<governance::ReviewDecide>) -> Result<governance::ReviewResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(governance::review_decide(pool, principal, request).await?)
}

// Before: Current exact approved decision and authorized withdrawal actor
// Durable: Append decision withdrawal; invalidate dependent completion without rewriting old decision
// Refusal: Different request/decision pair refuses
pub async fn review_withdraw(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<governance::ReviewWithdraw>) -> Result<governance::ReviewResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(governance::review_withdraw(pool, principal, request).await?)
}

// Before: Review cycle plus actual committed correction receipts affecting listed bases
// Durable: New cycle revision replaces exact affected bases; old obligations/decisions retained and dependencies invalidated
// Refusal: Forged or uncommitted correction receipt refuses
pub async fn payroll_revise_review_cycle(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<payroll::ReviewRevise>) -> Result<payroll::ReviewResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(payroll::payroll_revise_review_cycle(pool, principal, request).await?)
}

// Before: Current active cycle with supplied membership digest/revision
// Durable: Cycle cancellation recorded once; no payable transition; old basis evidence retained
// Refusal: Stale cycle revision refuses
pub async fn payroll_cancel_review_cycle(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<payroll::ReviewCancel>) -> Result<payroll::RunResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(payroll::payroll_cancel_review_cycle(pool, principal, request).await?)
}

// Before: APPROVED nonpayable run with current exact review cycle and correction evidence
// Durable: SUPERSEDED payable=false; dependent publications marked correction-pending; prior approval retained
// Refusal: Hidden/unrelated affected basis cannot be supplied
pub async fn payroll_supersede_approved_draft(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<payroll::ApprovedSupersede>) -> Result<payroll::SupersessionResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(payroll::payroll_supersede_approved_draft(pool, principal, request).await?)
}

// Before: EDITABLE exact draft/schema/head/epoch; actual gate policy and validity interval
// Durable: Reserve request identity then seal command; fill binding after fingerprint; no domain execution
// Refusal: Future gate decision/binding digest cannot enter original command identity
pub async fn governance_execution_gate_request(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<governance::GateRequest>) -> Result<governance::GateResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(governance::governance_execution_gate_request(pool, principal, request).await?)
}

// Before: Pending exact gate request; independent current decision actor
// Durable: Append PERMIT for exact request command binding; never autoexecute gated command
// Refusal: SELF or expired policy authority refuses without decision/consumption
pub async fn governance_execution_gate_decide(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<governance::GateDecide>) -> Result<governance::GateResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(governance::governance_execution_gate_decide(pool, principal, request).await?)
}

// Before: Approved completion and actual immutable result for historical subject; allowed projection
// Durable: One NONPAYABLE_REVIEW publication per exact target; slot currentness CURRENT; protected source proof
// Refusal: One forbidden target in<=2000 prevents entire Company release; no partial rows
pub async fn payroll_review_release(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<payroll::PublicationRelease>) -> Result<payroll::PublicationResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(payroll::payroll_review_release(pool, principal, request).await?)
}

// Before: Current publication predecessor and allowed replacement projection
// Durable: New immutable projected content; advance exact slot generation; retain predecessor
// Refusal: Stale predecessor slot generation refuses
pub async fn payroll_review_replace_projection(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<payroll::PublicationReplace>) -> Result<payroll::PublicationResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(payroll::payroll_review_replace_projection(pool, principal, request).await?)
}

// Before: Own historical publication readable via actual Person/Employment binding
// Durable: One OPEN request with exact item/narrative content and existing configured handler task or typed unassigned exception
// Refusal: Unrelated historical subject is undiscoverable and cannot create request
pub async fn payroll_review_request(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<payroll::CaseRequest>) -> Result<payroll::CaseResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(payroll::payroll_review_request(pool, principal, request).await?)
}

// Before: Own OPEN request at exact content/control revision
// Durable: Append one issue revision preserving prior acknowledged items and narrative
// Refusal: Stale issue revision refuses without losing current draft text
pub async fn payroll_review_request_append(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<payroll::CaseAppend>) -> Result<payroll::CaseResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(payroll::payroll_review_request_append(pool, principal, request).await?)
}

// Before: Own current open request and exact content/control revision
// Durable: WITHDRAWN once, retain submitted content/task causal evidence
// Refusal: Another Account without own-subject authority cannot withdraw
pub async fn payroll_review_request_withdraw(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<payroll::CaseWithdraw>) -> Result<payroll::CaseResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(payroll::payroll_review_request_withdraw(pool, principal, request).await?)
}

// Before: Current handler task/assignment and case issue revision; one exact item
// Durable: Immutable ANSWERED response; exact issue dispositions; case RESOLVED when all handled
// Refusal: Stale assignment or omitted issue disposition refuses; no invented task
pub async fn payroll_review_respond(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<payroll::CaseRespond>) -> Result<payroll::CaseResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(payroll::payroll_review_respond(pool, principal, request).await?)
}

// Before: Current task assignment1/control1; nominated eligible other Account; explicit interval
// Durable: Actual transfer-plan content REQUESTED with exact target selection/responsibility and receipt
// Refusal: Transfer cannot grant recipient authority to execute domain action
pub async fn work_transfer_submit(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<workflow::TransferSubmit>) -> Result<workflow::TransferResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(workflow::work_transfer_submit(pool, principal, request).await?)
}

// Before: Authorized exact transfer plan and current target task assignment/control
// Durable: One TRANSFERRED target; advance assignment epoch; append old/new responsibility history
// Refusal: Stale assignment/capacity conflict produces no unauthorized assignment
pub async fn work_transfer_execute(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<workflow::TransferExecute>) -> Result<workflow::TransferResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(workflow::work_transfer_execute(pool, principal, request).await?)
}

// Before: Current active transfer plan
// Durable: STOPPED for remaining pending targets; completed transfers remain historical facts
// Refusal: Stop cannot undo an already committed transfer by rewriting history
pub async fn work_transfer_stop(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<workflow::TransferStop>) -> Result<workflow::TransferResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(workflow::work_transfer_stop(pool, principal, request).await?)
}

// Before: Actual Group authority and one current member incarnation; exact child run intent
// Durable: One PREPARING Group operation with immutable selected slot set; no Company domain effect
// Refusal: Different membership incarnation refuses slot admission
pub async fn group_operation_create(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<group::GroupCreate>) -> Result<group::GroupResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(group::group_operation_create(pool, principal, request).await?)
}

// Before: Prepared exact child attempt per slot; original actor confirms fixed intent
// Durable: ACTIVE Group dispatch; children retain Company-local commands/receipts; no Group-wide atomicity claim
// Refusal: Missing/changed prepared attempt prevents activation
pub async fn group_operation_activate(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<group::GroupActivate>) -> Result<group::GroupResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(group::group_operation_activate(pool, principal, request).await?)
}

// Before: Current paused original operation with unknown/pending child outcomes
// Durable: Reconcile old child receipts first; resume only unchanged original pending intent
// Refusal: Resume cannot replace failed child intent or duplicate committed child effect
pub async fn group_operation_resume(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<group::GroupResume>) -> Result<group::GroupResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(group::group_operation_resume(pool, principal, request).await?)
}

// Before: Current active Group operation
// Durable: STOPPED for undispatched children; actual completed/unknown outcomes preserved
// Refusal: Unknown child cannot be reported definitive no effect
pub async fn group_operation_stop(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<group::GroupStop>) -> Result<group::GroupResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(group::group_operation_stop(pool, principal, request).await?)
}

// Before: Current verified calendar-paid-time qualification and exact seven weekday plans
// Durable: Retain immutable SOURCE19 calendar bytes; source24 custody; increment CALENDAR guard
// Refusal: Duplicate weekday or changed derived minutes refuses
pub async fn calendar_publish(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<attendance::CalendarPayload>) -> Result<attendance::SourceEffectResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(attendance::attendance_source_api.commit_calendar(pool, principal, request).await?)
}

// Before: Current Employment and retained calendar covering Sep2026; no overlapping active assignment
// Durable: Append ACTIVE assignment Sep1..Oct1; advance CALENDAR family guard
// Refusal: Cross-Employment/overlapping active interval refuses
pub async fn calendar_assign(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<attendance::CalendarAssignmentPayload>) -> Result<attendance::SourceEffectResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(attendance::attendance_source_api.commit_assignment(pool, principal, request).await?)
}

// Before: Actual event/day packages and boundary witnesses for requested subject/period
// Durable: Retain exact BLOCKED coverage with typed incomplete-attendance blockers and source24 custody
// Refusal: Omitted owner-enumerated event or changed package digest refuses
pub async fn attendance_resolve_coverage(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<attendance::CoveragePayload>) -> Result<attendance::SourceEffectResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(attendance::attendance_source_api.commit_coverage(pool, principal, request).await?)
}

// Before: Actual operational close plus exhaustive one-member resolved coverage artifact
// Durable: Immutable close basis with exact member count/digest; no payroll calculation
// Refusal: Missing/extra coverage member refuses
pub async fn attendance_create_close_basis(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<attendance::ClosePayload>) -> Result<attendance::SourceEffectResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(attendance::attendance_source_api.commit_close_basis(pool, principal, request).await?)
}

// Before: Actual issuer-trust profile and signed synthetic credential evidence; subject distinct from approver
// Durable: Unverified QUALIFICATION admission retained with source24 command custody
// Refusal: Client status VERIFIED or unknown issuer evidence cannot create verified qualification
pub async fn source_propose_qualification(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<payroll::AdmissionPayload>) -> Result<payroll::SourceEffectResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(payroll::payroll_source_api.commit_proposal(pool, principal, request).await?)
}

// Before: Unverified exact qualification admission/digest/head; genuine independent reviewer/issuer chain
// Durable: Append verification transition and current source guard; admission bytes immutable
// Refusal: Synthetic UUID distinction without actual Human/issuer evidence refuses
pub async fn source_verify_qualification(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<payroll::VerifyPayload>) -> Result<payroll::SourceEffectResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(payroll::payroll_source_api.commit_verification(pool, principal, request).await?)
}

// Before: Current verified exact qualification head and authorized actor
// Durable: Append revocation, advance source guard; dependent current eligibility invalidated
// Refusal: Wrong family/head or stale revision refuses
pub async fn source_revoke_qualification(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<payroll::RevokePayload>) -> Result<payroll::SourceEffectResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(payroll::payroll_source_api.commit_revocation(pool, principal, request).await?)
}

// Before: Current exact employment coverage/wage/attendance/profile/qualified source facts
// Durable: Unverified eligibility admission with four distinct insurance and seven distinct obligation facts
// Refusal: Duplicate scheme/family or subject mismatch refuses before admission
pub async fn eligibility_propose(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<payroll::AdmissionPayload>) -> Result<payroll::SourceEffectResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(payroll::payroll_source_api.commit_proposal(pool, principal, request).await?)
}

// Before: Current unverified exact eligibility admission and independent actual qualified reviewer
// Durable: Append VERIFIED eligibility head; retain original typed facts/custody
// Refusal: Missing referenced legal/source fact cannot be replaced by attest=true
pub async fn eligibility_verify(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<payroll::VerifyPayload>) -> Result<payroll::SourceEffectResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(payroll::payroll_source_api.commit_verification(pool, principal, request).await?)
}

// Before: Current verified eligibility head
// Durable: Append revocation and guard bump; leave historical admission/evidence intact
// Refusal: Wrong Company head refuses
pub async fn eligibility_revoke(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<payroll::RevokePayload>) -> Result<payroll::SourceEffectResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(payroll::payroll_source_api.commit_revocation(pool, principal, request).await?)
}

// Before: Verified table qualification, actual retained artifact, eligibility and exact row selection
// Durable: Unverified TAX_EVIDENCE admission with zero-valued synthetic table cells and exact locators
// Refusal: Wrong income band/locator/artifact digest refuses; zero never inferred from missing data
pub async fn payroll_admit_tax_evidence(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<payroll::AdmissionPayload>) -> Result<payroll::SourceEffectResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(payroll::payroll_source_api.commit_proposal(pool, principal, request).await?)
}

// Before: Unverified tax evidence and independent current qualified reviewer
// Durable: Append verification bound to exact input/digest/head; retain immutable source24 custody
// Refusal: Unverified table qualification or same-Human reviewer refuses
pub async fn payroll_verify_tax_evidence(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<payroll::VerifyPayload>) -> Result<payroll::SourceEffectResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(payroll::payroll_source_api.commit_verification(pool, principal, request).await?)
}

// Before: Current verified exact tax-evidence head
// Durable: Append revoke transition and TAX_EVIDENCE guard bump; invalidate dependent current source selection
// Refusal: Stale/different family head refuses
pub async fn payroll_revoke_tax_evidence(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<payroll::RevokePayload>) -> Result<payroll::SourceEffectResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(payroll::payroll_source_api.commit_revocation(pool, principal, request).await?)
}

// Before: Current exact action registration/schema/target; Account has editing authority
// Durable: One EDITABLE Account-custodied draft for exact intent slot; no domain side effect
// Refusal: GET/preflight cannot create draft; existing intent slot cannot fork invisibly
pub async fn draft_start(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<ontology::DraftStart>) -> Result<ontology::DraftAck, Box<dyn std::error::Error + Send + Sync>> {
    Ok(ontology::draft_start(pool, principal, request).await?)
}

// Before: Complete valid current draft with exact head/epoch/schema and applicable current authority
// Durable: Actual READY attempt and reserved domain command; immutable original normalized input/pins; no domain execution
// Refusal: Incomplete/hidden/oversize input refuses while preserving acknowledged draft
pub async fn draft_seal(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<ontology::DraftSeal>) -> Result<ontology::SealResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(ontology::draft_seal(pool, principal, request).await?)
}

// Before: Actual READY original attempt; current authority and complete source/evidence protections
// Durable: Original owner executes one exact sealed domain command; returns public reconciliation subject
// Refusal: Wrapper command must not replace original actor/input or publish duplicate OWNER_SUBMISSION
pub async fn attempt_execute(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<ontology::AttemptExecute>) -> Result<ontology::ResultObservation, Box<dyn std::error::Error + Send + Sync>> {
    Ok(ontology::attempt_execute(pool, principal, request).await?)
}

// Before: Own EDITABLE draft at exact head/epoch
// Durable: Discard state recorded; acknowledged history retained according to policy
// Refusal: Discard cannot erase already sealed submission history
pub async fn draft_discard(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<ontology::DraftDiscard>) -> Result<ontology::ResultObservation, Box<dyn std::error::Error + Send + Sync>> {
    Ok(ontology::draft_discard(pool, principal, request).await?)
}

// Before: Own EDITABLE draft; target schema and registered mapping current
// Durable: Append explicitly mapped new draft revision; retain old schema/content and stable field ids
// Refusal: Unmapped complete field or stale old schema refuses without silent loss
pub async fn draft_upgrade_schema(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<ontology::DraftUpgrade>) -> Result<ontology::DraftAck, Box<dyn std::error::Error + Send + Sync>> {
    Ok(ontology::draft_upgrade_schema(pool, principal, request).await?)
}

// Before: Retention worker with actual current policy and exact control revision; expiry due
// Durable: Expire allowed editable draft and close editing continuation; retain required evidence/pins
// Refusal: Client-supplied time/policy alone cannot expire held or sealed evidence
pub async fn draft_expire(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<ontology::DraftExpire>) -> Result<ontology::ResultObservation, Box<dyn std::error::Error + Send + Sync>> {
    Ok(ontology::draft_expire(pool, principal, request).await?)
}

// Before: Exact source draft/head/schema/editing token with complete source19 input
// Durable: Actual immutable Preparation24 input unit/digest; no future attempt/command/receipt; source ordinal rules unchanged
// Refusal: A fake prepared_submission_unit or cross-draft input cannot substitute for stored preparation
pub async fn source_prepare_submission(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<ontology::SourcePreparationRequest>) -> Result<ontology::PreparationResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(ontology::source_prepare_submission(pool, principal, request).await?)
}

// Before: Current docs upload policy/target authority and literal upload.csv expected bytes/digest
// Durable: One actual upload ticket with scoped issuer-controlled version target; no VERIFIED attestation yet
// Refusal: Caller-chosen storage capability or oversized body refuses
pub async fn artifact_request_upload(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<docs::UploadRequest>) -> Result<docs::UploadTicket, Box<dyn std::error::Error + Send + Sync>> {
    Ok(docs::artifact_request_upload(pool, principal, request).await?)
}

// Before: Actual worker job for exact ticket/unit/location revision and uploaded immutable bytes
// Durable: Actual attestation for observed bytes/version; preserve immutable location evidence
// Refusal: Ordinary actor or mismatched observed version/digest cannot mark verified
pub async fn artifact_verify_version(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<docs::VerifyUpload>) -> Result<docs::AttestationResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(docs::artifact_verify_version(pool, principal, request).await?)
}

// Before: Current leave request and authorized current manager; consultation branch
// Durable: TIME_CHANGE_CONSULT with Sep15 proposal, retained source witness/receipt, no invented charge approval
// Refusal: Current request revision mismatch or absent consulted authority refuses
pub async fn leave_decide(pool: &PgPool, principal: &Principal, request: RegisteredActionRequest<leave::CurrentLeaveDecision>) -> Result<leave::LeaveResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(leave::leave_decide(pool, principal, request).await?)
}
