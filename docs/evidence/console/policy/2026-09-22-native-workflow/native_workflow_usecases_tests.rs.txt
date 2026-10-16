// Proposed ordinary child of native_business_policy_unit_tests.rs.
// Reuses real CompanyPolicy and its existing strict source fixtures. Scripted
// scopes test orchestration/withholding only, never SQL/Auth/browser acceptance.
use super::{CompanyPolicy, at, binding, bootstrap_row, digest, id};
use console_identity_application::company_policy::{
    AccountId, CompanyPolicyDecision, CompanyPolicyDecisionPort, CompanyPolicyError,
    CompanyPolicyRequest, CurrentCompanyAuthority, CurrentNativeBootstrapAuthority,
    CurrentPayrollReadAuthority, NativeBootstrapRequestV1,
    business::{NativeBusinessOperationV1, NativeCompanyBusinessCommandV1},
    workflow::*,
};
use console_kernel_core::{OrgId, TraceContext};
use std::{
    collections::VecDeque,
    future::{Future, poll_fn},
    pin::pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};
use time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Form,
    Accept,
    Execute,
    Status,
}
fn mode(request: &NativePolicyScopeRequest<'_>) -> Mode {
    match request {
        NativePolicyScopeRequest::Form(_) => Mode::Form,
        NativePolicyScopeRequest::Accept(_) => Mode::Accept,
        NativePolicyScopeRequest::Execute(_) => Mode::Execute,
        NativePolicyScopeRequest::Status(_) => Mode::Status,
    }
}
fn input() -> NativeCompanyBusinessCommandV1 {
    NativeCompanyBusinessCommandV1::install(id(1000), OrgId::from_uuid(id(11)), 1).unwrap()
}
fn selector() -> NativePolicyCommandRef {
    NativePolicyCommandRef::from_command(&input())
}
fn accepted() -> NativePolicyAcceptedView {
    NativePolicyAcceptedView {
        input: input(),
        intake_receipt_id: id(1001),
        accepted_at: at(),
        execution_not_after: at() + Duration::days(7),
    }
}
fn terminal() -> NativePolicyTerminalView {
    NativePolicyTerminalView {
        accepted: accepted(),
        receipt_id: id(1002),
        executed_at: at(),
        epoch_before: 1,
        epoch_after: 2,
        outcome: NativePolicyOutcome::Committed(NativePolicyEffect::Installed {
            object_type_id: id(1003),
        }),
    }
}
fn pending() -> NativePolicyStatus {
    NativePolicyStatus::AcceptedPending(accepted())
}
struct Plan {
    mode: Mode,
    status: NativePolicyStatus,
    inserted: bool,
    operation_error: Option<NativePolicyWorkflowError>,
    finish_error: Option<NativePolicyWorkflowError>,
    pending_finish: bool,
}
impl Plan {
    fn new(mode: Mode) -> Self {
        Self {
            mode,
            status: pending(),
            inserted: true,
            operation_error: None,
            finish_error: None,
            pending_finish: false,
        }
    }
}
#[derive(Default)]
struct History {
    events: Vec<String>,
    active: bool,
    input_bytes: Vec<Vec<u8>>,
}
struct Store {
    history: Arc<Mutex<History>>,
    plans: Mutex<VecDeque<Plan>>,
    expected_selector: NativePolicyCommandRef,
    source_company: OrgId,
}
struct Scope<'a> {
    store: &'a Store,
    plan: Plan,
    authority: CurrentNativeBootstrapAuthority,
    request: NativeBootstrapRequestV1,
    committed: bool,
}
impl Store {
    fn new(plans: Vec<Plan>) -> Self {
        Self {
            history: Arc::new(Mutex::new(History::default())),
            plans: Mutex::new(plans.into()),
            expected_selector: selector(),
            source_company: OrgId::from_uuid(id(11)),
        }
    }
    fn events(&self) -> Vec<String> {
        self.history.lock().unwrap().events.clone()
    }
    fn record(&self, event: impl Into<String>) {
        self.history.lock().unwrap().events.push(event.into());
    }
    fn drained(&self) {
        assert!(self.plans.lock().unwrap().is_empty());
        assert!(!self.history.lock().unwrap().active);
    }
}
impl NativePolicyWorkflowStore for Store {
    type Credentials = ();
    type FormProof = u64;
    type Scope<'a>
        = Scope<'a>
    where
        Self: 'a;
    async fn lock<'a>(
        &'a self,
        _: &'a (),
        request: NativePolicyScopeRequest<'a>,
    ) -> Result<Scope<'a>, NativePolicyWorkflowError> {
        let plan = self
            .plans
            .lock()
            .unwrap()
            .pop_front()
            .expect("unplanned transaction");
        assert_eq!(plan.mode, mode(&request));
        let selected = request.selector();
        assert_eq!(selected, self.expected_selector);
        let mut h = self.history.lock().unwrap();
        assert!(!h.active, "A scope must finish/drop before B starts");
        h.active = true;
        h.events.push(format!("open:{:?}", plan.mode));
        let recipient = if let NativePolicyScopeRequest::Accept(command) = &request {
            h.input_bytes.push(command.encode(binding().account));
            command
                .recipient_account_id()
                .unwrap_or(AccountId::from_uuid(id(34)).unwrap())
        } else {
            AccountId::from_uuid(id(34)).unwrap()
        };
        drop(h);
        let mut row = bootstrap_row();
        row.org_id = *self.source_company.as_uuid();
        row.company_epoch = 1;
        row.current_policy_receipt_id = None;
        let authority =
            CurrentNativeBootstrapAuthority::from_retained_projection(&binding(), row).unwrap();
        let requested = NativeBootstrapRequestV1::new(
            selected.company(),
            id(12),
            recipient,
            selected.operation(),
            digest(),
        )
        .unwrap();
        Ok(Scope {
            store: self,
            plan,
            authority,
            request: requested,
            committed: false,
        })
    }
}
impl Scope<'_> {
    fn operation(&self, mode: Mode) -> Result<(), NativePolicyWorkflowError> {
        assert_eq!(self.plan.mode, mode);
        self.store.record(format!("operation:{mode:?}"));
        if let Some(error) = self.plan.operation_error {
            Err(error)
        } else {
            Ok(())
        }
    }
}
impl NativePolicyWorkflowScope for Scope<'_> {
    type FormProof = u64;
    fn authority(&self) -> &CurrentNativeBootstrapAuthority {
        &self.authority
    }
    async fn form(&mut self) -> Result<NativePolicyForm<u64>, NativePolicyWorkflowError> {
        self.operation(Mode::Form)?;
        Ok(NativePolicyForm {
            view: NativePolicyFormView {
                selector: selector(),
                company_epoch: 1,
                administrative_account_id: AccountId::from_uuid(id(34)).unwrap(),
                installed_object_type_id: None,
                assignment: None,
            },
            proof: 77,
        })
    }
    async fn accept(
        &mut self,
        _: &TraceContext,
    ) -> Result<NativePolicyAcceptance, NativePolicyWorkflowError> {
        self.operation(Mode::Accept)?;
        Ok(NativePolicyAcceptance {
            inserted: self.plan.inserted,
            status: self.plan.status.clone(),
        })
    }
    async fn execute(
        &mut self,
        _: &TraceContext,
    ) -> Result<NativePolicyExecution, NativePolicyWorkflowError> {
        self.operation(Mode::Execute)?;
        Ok(NativePolicyExecution {
            inserted: self.plan.inserted,
            terminal: terminal(),
        })
    }
    async fn status(&mut self) -> Result<NativePolicyStatus, NativePolicyWorkflowError> {
        self.operation(Mode::Status)?;
        Ok(self.plan.status.clone())
    }
    async fn finish<P: CompanyPolicyDecisionPort + ?Sized>(
        mut self,
        policy: &P,
        proof: Option<&u64>,
    ) -> Result<(), NativePolicyWorkflowError> {
        self.store.record(format!("finish:{:?}", self.plan.mode));
        assert_eq!(
            proof.copied(),
            if self.plan.mode == Mode::Form {
                Some(77)
            } else {
                None
            }
        );
        if self.plan.pending_finish {
            let mut first = true;
            poll_fn(|cx| {
                if first {
                    first = false;
                    cx.waker().wake_by_ref();
                    Poll::Pending
                } else {
                    Poll::Ready(())
                }
            })
            .await;
        }
        match policy.decide_native_bootstrap(&self.authority, &self.request) {
            Ok(CompanyPolicyDecision::Allow) => {}
            Ok(CompanyPolicyDecision::Deny) => return Err(NativePolicyWorkflowError::NotFound),
            Err(_) => return Err(NativePolicyWorkflowError::Unavailable),
        }
        if let Some(error) = self.plan.finish_error {
            return Err(error);
        }
        self.committed = true;
        self.store.record(format!("committed:{:?}", self.plan.mode));
        Ok(())
    }
}
impl Drop for Scope<'_> {
    fn drop(&mut self) {
        let mut h = self.store.history.lock().unwrap();
        assert!(h.active);
        h.active = false;
        h.events
            .push(format!("release:{:?}:{}", self.plan.mode, self.committed));
    }
}
struct Policy {
    real: CompanyPolicy,
    history: Arc<Mutex<History>>,
    overrides: Mutex<VecDeque<Result<CompanyPolicyDecision, CompanyPolicyError>>>,
}
impl Policy {
    fn new(store: &Store) -> Self {
        Self {
            real: CompanyPolicy::new().unwrap(),
            history: store.history.clone(),
            overrides: Mutex::new(VecDeque::new()),
        }
    }
}
impl CompanyPolicyDecisionPort for Policy {
    fn decide_native_bootstrap(
        &self,
        a: &CurrentNativeBootstrapAuthority,
        r: &NativeBootstrapRequestV1,
    ) -> Result<CompanyPolicyDecision, CompanyPolicyError> {
        self.history.lock().unwrap().events.push("decision".into());
        let actual = self.real.decide_native_bootstrap(a, r)?;
        assert_eq!(
            actual,
            CompanyPolicyDecision::Allow,
            "positive control must reach real policy"
        );
        self.overrides
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or(Ok(actual))
    }
    fn decide_native_payroll_collection(
        &self,
        a: &CurrentPayrollReadAuthority,
    ) -> Result<CompanyPolicyDecision, CompanyPolicyError> {
        self.real.decide_native_payroll_collection(a)
    }
    fn decide(
        &self,
        a: &CurrentCompanyAuthority,
        r: &CompanyPolicyRequest,
    ) -> Result<CompanyPolicyDecision, CompanyPolicyError> {
        self.real.decide(a, r)
    }
}
fn ready<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    let mut cx = Context::from_waker(Waker::noop());
    match future.as_mut().poll(&mut cx) {
        Poll::Ready(v) => v,
        Poll::Pending => panic!("unexpected pending"),
    }
}
fn history(mode: Mode) -> Vec<String> {
    vec![
        format!("open:{mode:?}"),
        "decision".into(),
        format!("operation:{mode:?}"),
        format!("finish:{mode:?}"),
        "decision".into(),
        format!("committed:{mode:?}"),
        format!("release:{mode:?}:true"),
    ]
}

#[test]
fn workflow_form_is_authorized_and_exact_proof_withheld_until_finish() {
    let s = Store::new(vec![Plan::new(Mode::Form)]);
    let p = Policy::new(&s);
    let form = ready(native_policy_form(&s, &p, &(), selector())).unwrap();
    assert_eq!(form.proof, 77);
    assert_eq!(form.view.company_epoch, 1);
    assert_eq!(form.view.selector, selector());
    assert_eq!(s.events(), history(Mode::Form));
    s.drained();
}
#[test]
fn workflow_denied_or_unavailable_policy_never_reads_or_mutates() {
    for outcome in [
        Ok(CompanyPolicyDecision::Deny),
        Err(CompanyPolicyError::EvaluatorUnavailable),
    ] {
        for mode in [Mode::Form, Mode::Accept, Mode::Execute, Mode::Status] {
            let s = Store::new(vec![Plan::new(mode)]);
            let p = Policy::new(&s);
            p.overrides.lock().unwrap().push_back(outcome);
            let input = input();
            let trace = TraceContext::generate();
            let result = match mode {
                Mode::Form => ready(native_policy_form(&s, &p, &(), selector())).map(|_| ()),
                Mode::Accept => {
                    ready(accept_native_policy_command(&s, &p, &(), &input, &trace)).map(|_| ())
                }
                Mode::Execute => ready(execute_native_policy_command(
                    &s,
                    &p,
                    &(),
                    selector(),
                    &trace,
                ))
                .map(|_| ()),
                Mode::Status => {
                    ready(native_policy_command_status(&s, &p, &(), selector())).map(|_| ())
                }
            };
            assert_eq!(
                result,
                Err(if outcome == Ok(CompanyPolicyDecision::Deny) {
                    NativePolicyWorkflowError::NotFound
                } else {
                    NativePolicyWorkflowError::Unavailable
                })
            );
            assert_eq!(
                s.events(),
                vec![
                    format!("open:{mode:?}"),
                    "decision".into(),
                    format!("release:{mode:?}:false")
                ]
            );
            s.drained();
        }
    }
}
#[test]
fn workflow_submit_confirms_a_before_b_with_exact_original_input() {
    let s = Store::new(vec![Plan::new(Mode::Accept), Plan::new(Mode::Execute)]);
    let p = Policy::new(&s);
    let result = ready(submit_native_policy_command(
        &s,
        &p,
        &(),
        &input(),
        &TraceContext::generate(),
    ))
    .unwrap();
    assert!(result.inserted);
    assert_eq!(result.terminal, terminal());
    assert_eq!(
        s.history.lock().unwrap().input_bytes,
        vec![input().encode(binding().account)]
    );
    let mut expected = history(Mode::Accept);
    expected.extend(history(Mode::Execute));
    assert_eq!(s.events(), expected);
    s.drained();
}
#[test]
fn workflow_failed_or_unknown_a_never_starts_b() {
    for error in [
        NativePolicyWorkflowError::CsrfInvalid,
        NativePolicyWorkflowError::Conflict,
        NativePolicyWorkflowError::Capacity,
        NativePolicyWorkflowError::Unavailable,
        NativePolicyWorkflowError::Unconfirmed,
    ] {
        let mut a = Plan::new(Mode::Accept);
        a.finish_error = Some(error);
        let s = Store::new(vec![a]);
        let p = Policy::new(&s);
        assert!(
            matches!(ready(submit_native_policy_command(&s,&p,&(),&input(),&TraceContext::generate())),Err(e) if e==error)
        );
        assert_eq!(
            s.events(),
            [
                "open:Accept",
                "decision",
                "operation:Accept",
                "finish:Accept",
                "decision",
                "release:Accept:false"
            ]
        );
        s.drained();
    }
}
#[test]
fn workflow_replayed_final_a_skips_b_and_preserves_terminal() {
    for outcome in [
        terminal().outcome,
        NativePolicyOutcome::Rejected(NativePolicyRejection::RevisionConflict),
    ] {
        let mut final_result = terminal();
        final_result.outcome = outcome;
        if matches!(final_result.outcome, NativePolicyOutcome::Rejected(_)) {
            final_result.epoch_after = 1;
        }
        let mut a = Plan::new(Mode::Accept);
        a.inserted = false;
        a.status = NativePolicyStatus::Terminal(final_result.clone());
        let s = Store::new(vec![a]);
        let p = Policy::new(&s);
        let actual = ready(submit_native_policy_command(
            &s,
            &p,
            &(),
            &input(),
            &TraceContext::generate(),
        ))
        .unwrap();
        assert!(!actual.inserted);
        assert_eq!(actual.terminal, final_result);
        assert_eq!(s.events(), history(Mode::Accept));
        s.drained();
    }
}
#[test]
fn workflow_status_is_read_only_for_missing_pending_expired_and_final() {
    for status in [
        NativePolicyStatus::NotVisible,
        pending(),
        NativePolicyStatus::AcceptedExpired(accepted()),
        NativePolicyStatus::Terminal(terminal()),
    ] {
        let mut plan = Plan::new(Mode::Status);
        plan.status = status.clone();
        let s = Store::new(vec![plan]);
        let p = Policy::new(&s);
        assert_eq!(
            ready(native_policy_command_status(&s, &p, &(), selector())).unwrap(),
            status
        );
        assert!(s.history.lock().unwrap().input_bytes.is_empty());
        assert_eq!(s.events(), history(Mode::Status));
        s.drained();
    }
}
#[test]
fn workflow_unknown_b_returns_unknown_and_recovery_reuses_selector_only() {
    let mut b = Plan::new(Mode::Execute);
    b.finish_error = Some(NativePolicyWorkflowError::Unconfirmed);
    let s = Store::new(vec![Plan::new(Mode::Accept), b]);
    let p = Policy::new(&s);
    assert!(matches!(
        ready(submit_native_policy_command(
            &s,
            &p,
            &(),
            &input(),
            &TraceContext::generate()
        )),
        Err(NativePolicyWorkflowError::Unconfirmed)
    ));
    s.drained();
    // Separate later authenticated status, followed by explicit same-ID retry.
    let mut b = Plan::new(Mode::Execute);
    b.inserted = false;
    let s = Store::new(vec![Plan::new(Mode::Status), b]);
    let p = Policy::new(&s);
    assert_eq!(
        ready(native_policy_command_status(&s, &p, &(), selector())).unwrap(),
        pending()
    );
    let result = ready(execute_native_policy_command(
        &s,
        &p,
        &(),
        selector(),
        &TraceContext::generate(),
    ))
    .unwrap();
    assert!(!result.inserted);
    assert_eq!(result.terminal, terminal());
    assert!(s.history.lock().unwrap().input_bytes.is_empty());
    s.drained();
}
#[test]
fn workflow_final_denial_withholds_provisional_terminal() {
    let s = Store::new(vec![Plan::new(Mode::Execute)]);
    let p = Policy::new(&s);
    *p.overrides.lock().unwrap() = vec![
        Ok(CompanyPolicyDecision::Allow),
        Ok(CompanyPolicyDecision::Deny),
    ]
    .into();
    assert!(matches!(
        ready(execute_native_policy_command(
            &s,
            &p,
            &(),
            selector(),
            &TraceContext::generate()
        )),
        Err(NativePolicyWorkflowError::NotFound)
    ));
    assert_eq!(
        s.events(),
        [
            "open:Execute",
            "decision",
            "operation:Execute",
            "finish:Execute",
            "decision",
            "release:Execute:false"
        ]
    );
    s.drained();
}
#[test]
fn workflow_cancellation_during_a_finish_releases_without_b_or_response() {
    let mut a = Plan::new(Mode::Accept);
    a.pending_finish = true;
    let s = Store::new(vec![a]);
    let p = Policy::new(&s);
    let input = input();
    let trace = TraceContext::generate();
    {
        let future = submit_native_policy_command(&s, &p, &(), &input, &trace);
        let mut future = pin!(future);
        let mut cx = Context::from_waker(Waker::noop());
        assert!(future.as_mut().poll(&mut cx).is_pending());
        assert_eq!(
            s.events(),
            [
                "open:Accept",
                "decision",
                "operation:Accept",
                "finish:Accept"
            ]
        );
    }
    assert_eq!(s.events().last().unwrap(), "release:Accept:false");
    s.drained();
}

#[test]
fn workflow_impossible_acceptance_not_visible_is_unavailable_and_never_starts_b() {
    for inserted in [false, true] {
        let mut a = Plan::new(Mode::Accept);
        a.status = NativePolicyStatus::NotVisible;
        a.inserted = inserted;
        let s = Store::new(vec![a]);
        let p = Policy::new(&s);
        assert!(matches!(
            ready(submit_native_policy_command(
                &s,
                &p,
                &(),
                &input(),
                &TraceContext::generate()
            )),
            Err(NativePolicyWorkflowError::Unavailable)
        ));
        assert!(!s.events().iter().any(|event| event.contains("Execute")));
        s.drained();
    }
}

#[test]
fn workflow_foreign_source_company_fails_before_any_scope_operation() {
    for mode in [Mode::Form, Mode::Accept, Mode::Execute, Mode::Status] {
        let mut s = Store::new(vec![Plan::new(mode)]);
        s.source_company = OrgId::from_uuid(id(9000));
        let p = CompanyPolicy::new().unwrap();
        let input = input();
        let trace = TraceContext::generate();
        let actual = match mode {
            Mode::Form => ready(native_policy_form(&s, &p, &(), selector())).map(|_| ()),
            Mode::Accept => {
                ready(accept_native_policy_command(&s, &p, &(), &input, &trace)).map(|_| ())
            }
            Mode::Execute => ready(execute_native_policy_command(
                &s,
                &p,
                &(),
                selector(),
                &trace,
            ))
            .map(|_| ()),
            Mode::Status => {
                ready(native_policy_command_status(&s, &p, &(), selector())).map(|_| ())
            }
        };
        assert_eq!(actual, Err(NativePolicyWorkflowError::Unavailable));
        assert_eq!(
            s.events(),
            vec![format!("open:{mode:?}"), format!("release:{mode:?}:false")]
        );
        s.drained();
    }
}

#[test]
fn workflow_grant_uses_original_recipient_and_actual_cedar_denies_mismatch() {
    for recipient in [id(34), id(9001)] {
        let command = NativeCompanyBusinessCommandV1::grant(
            id(1000),
            OrgId::from_uuid(id(11)),
            1,
            AccountId::from_uuid(recipient).unwrap(),
            None,
            at() + Duration::days(1),
        )
        .unwrap();
        let mut accepted_value = accepted();
        accepted_value.input = command.clone();
        let mut a = Plan::new(Mode::Accept);
        a.status = NativePolicyStatus::AcceptedPending(accepted_value);
        let mut s = Store::new(vec![a]);
        s.expected_selector = NativePolicyCommandRef::from_command(&command);
        let p = CompanyPolicy::new().unwrap();
        let actual = ready(accept_native_policy_command(
            &s,
            &p,
            &(),
            &command,
            &TraceContext::generate(),
        ));
        assert_eq!(
            s.history.lock().unwrap().input_bytes,
            vec![command.encode(binding().account)]
        );
        if recipient == id(34) {
            assert!(actual.is_ok());
            assert_eq!(
                s.events(),
                [
                    "open:Accept",
                    "operation:Accept",
                    "finish:Accept",
                    "committed:Accept",
                    "release:Accept:true"
                ]
            );
        } else {
            assert!(matches!(actual, Err(NativePolicyWorkflowError::NotFound)));
            assert_eq!(s.events(), ["open:Accept", "release:Accept:false"]);
        }
        s.drained();
    }
}

#[test]
fn workflow_form_finish_failure_withholds_view_and_exact_proof() {
    for error in [
        NativePolicyWorkflowError::CsrfInvalid,
        NativePolicyWorkflowError::NotFound,
        NativePolicyWorkflowError::Unavailable,
        NativePolicyWorkflowError::Unconfirmed,
    ] {
        let mut plan = Plan::new(Mode::Form);
        plan.finish_error = Some(error);
        let s = Store::new(vec![plan]);
        let p = Policy::new(&s);
        assert_eq!(
            ready(native_policy_form(&s, &p, &(), selector())).map(|_| ()),
            Err(error)
        );
        assert_eq!(
            s.events(),
            [
                "open:Form",
                "decision",
                "operation:Form",
                "finish:Form",
                "decision",
                "release:Form:false"
            ]
        );
        s.drained();
    }
}
