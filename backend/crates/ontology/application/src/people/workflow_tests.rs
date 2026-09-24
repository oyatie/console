//! Pure orchestration fault tests, not database/authentication proof.
//! Scripted scopes exercise orchestration; production uses the retained owner.
use super::*;
use crate::people::{DirectoryRejectionV1, NativeDirectoryCommandV1};
use std::{
    future::ready,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    task::{Context, Poll, Waker},
};
use time::Duration;
fn id(n: u128) -> Uuid {
    Uuid::from_u128(n)
}
fn actor() -> AccountId {
    AccountId::from_uuid(id(1)).unwrap()
}
fn company() -> OrgId {
    OrgId::from_uuid(id(2))
}
fn locator() -> DirectoryRequestRef {
    DirectoryRequestRef::new(company(), id(3)).unwrap()
}
fn expected() -> DirectoryExpectationsV1 {
    DirectoryExpectationsV1 {
        company_epoch: 7,
        object_type_id: id(5),
        action_type_id: id(6),
        action_revision: 1,
        schema_revision: 1,
        legal_name_property_id: id(7),
        employee_number_property_id: id(8),
    }
}
fn submission() -> DirectorySubmission {
    DirectorySubmission::new(
        locator(),
        expected(),
        DirectoryRegistrationInput::new("김하늘", "K-1").unwrap(),
    )
    .unwrap()
}
fn accepted(actor: AccountId, command_id: Uuid) -> AcceptedDirectoryRequestV1 {
    AcceptedDirectoryRequestV1::new(
        actor,
        NativeDirectoryCommandV1::new(
            command_id,
            company(),
            id(4),
            expected(),
            submission().input().clone(),
        )
        .unwrap(),
        id(9),
        OffsetDateTime::from_unix_timestamp(1_790_000_000).unwrap(),
    )
    .unwrap()
}
fn run<F: Future>(f: F) -> F::Output {
    let mut f = Box::pin(f);
    match f.as_mut().poll(&mut Context::from_waker(Waker::noop())) {
        Poll::Ready(v) => v,
        Poll::Pending => panic!("pure test unexpectedly pending"),
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Fault {
    None,
    Company,
    Action,
    Mode,
    ReceiptActor,
    ReceiptCommand,
    CurrentEpoch,
    ReplayOld,
    WrongDetail,
    DuplicateList,
    CancelCreatesEffect,
    ExecuteCancels,
    FinishUnavailable,
    FinishUnconfirmed,
    ExecuteReplayOld,
    ExecuteRevisionConflict,
    ExecuteExpired,
    StatusExpired,
    StatusRefreshExpired,
    StatusStaleExpiryClock,
    StatusExpiredPending,
    StatusHistoricalCommitted,
    StatusMissingProof,
    StatusTerminalProof,
    ListOverLimit,
    ListDescending,
    ListAtCursor,
    ListNextForeign,
    ListNextPartial,
    ListFullContinuation,
}
struct Authority {
    company: OrgId,
    action: DirectoryAction,
    expected: DirectoryExpectationsV1,
    observed_at: OffsetDateTime,
}
impl DirectoryAuthority for Authority {
    fn company(&self) -> OrgId {
        self.company
    }
    fn actor(&self) -> AccountId {
        actor()
    }
    fn action(&self) -> DirectoryAction {
        self.action
    }
    fn observed_at(&self) -> OffsetDateTime {
        self.observed_at
    }
    fn creation_expectations(&self) -> Option<DirectoryExpectationsV1> {
        (self.action == DirectoryAction::Create).then_some(self.expected)
    }
}
struct Policy {
    calls: AtomicUsize,
    deny_at: usize,
}
impl DirectoryDecisionPort<Authority> for Policy {
    fn permits(&self, a: &Authority, r: DirectoryAccess) -> Result<bool, DirectoryWorkflowError> {
        assert_eq!(r.company, a.company);
        assert_eq!(r.actor, a.actor());
        assert_eq!(r.action, a.action);
        Ok(self.calls.fetch_add(1, Ordering::SeqCst) + 1 != self.deny_at)
    }
}
struct Credentials(u8);
struct Store {
    fault: Fault,
    events: Arc<Mutex<Vec<&'static str>>>,
}
struct Scope {
    fault: Fault,
    events: Arc<Mutex<Vec<&'static str>>>,
    authority: Authority,
    kind: DirectoryScopeKind,
    form_expected: DirectoryExpectationsV1,
    finished: bool,
}
impl Drop for Scope {
    fn drop(&mut self) {
        self.events.lock().unwrap().push(if self.finished {
            "finished"
        } else {
            "discarded"
        });
    }
}
impl DirectoryWorkflowStore for Store {
    type Credentials = Credentials;
    type Authority = Authority;
    type FormProof = u8;
    type Scope<'a>
        = Scope
    where
        Self: 'a;
    fn lock<'a>(
        &'a self,
        c: &'a Credentials,
        r: DirectoryScopeRequest<'a>,
    ) -> impl Future<Output = Result<Scope, DirectoryWorkflowError>> + Send {
        assert_eq!(c.0, 42);
        self.events.lock().unwrap().push("lock");
        let mut e = expected();
        if matches!(
            self.fault,
            Fault::CurrentEpoch
                | Fault::ReplayOld
                | Fault::ExecuteReplayOld
                | Fault::ExecuteRevisionConflict
                | Fault::StatusHistoricalCommitted
        ) {
            e.company_epoch += 1;
        }
        ready(Ok(Scope {
            fault: self.fault,
            events: self.events.clone(),
            authority: Authority {
                company: if self.fault == Fault::Company {
                    OrgId::from_uuid(id(99))
                } else {
                    r.company()
                },
                action: if self.fault == Fault::Action {
                    DirectoryAction::Read
                } else {
                    r.kind().action()
                },
                expected: e,
                observed_at: if matches!(
                    self.fault,
                    Fault::ExecuteExpired
                        | Fault::StatusExpired
                        | Fault::StatusExpiredPending
                        | Fault::StatusHistoricalCommitted
                ) {
                    accepted(actor(), id(3)).execution_not_after() + Duration::seconds(1)
                } else {
                    accepted(actor(), id(3)).accepted_at() + Duration::seconds(1)
                },
            },
            kind: if self.fault == Fault::Mode {
                DirectoryScopeKind::Status(id(3))
            } else {
                r.kind()
            },
            form_expected: match &r {
                DirectoryScopeRequest::ValidationForm(_, expected) => *expected,
                _ => e,
            },
            finished: false,
        }))
    }
}
impl Scope {
    fn event(&self, name: &'static str) {
        self.events.lock().unwrap().push(name);
    }
    fn accepted(&self) -> AcceptedDirectoryRequestV1 {
        accepted(
            if self.fault == Fault::ReceiptActor {
                AccountId::from_uuid(id(99)).unwrap()
            } else {
                actor()
            },
            if self.fault == Fault::ReceiptCommand {
                id(99)
            } else {
                id(3)
            },
        )
    }
    fn terminal(&self, outcome: DirectoryTerminalOutcomeV1) -> DirectoryTerminalV1 {
        let a = self.accepted();
        let t = if outcome == DirectoryTerminalOutcomeV1::Expired {
            a.execution_not_after()
        } else {
            a.accepted_at() + Duration::seconds(1)
        };
        DirectoryTerminalV1::new(a, outcome, t).unwrap()
    }
    fn record(&self) -> DirectoryRecord {
        DirectoryRecord {
            employee_id: if self.fault == Fault::WrongDetail {
                id(99)
            } else {
                id(4)
            },
            person_id: id(44),
            legal_name: "김하늘".into(),
            employee_number: "K-1".into(),
            person_version: 2,
            registered_at: self.accepted().accepted_at(),
        }
    }
}
impl DirectoryWorkflowScope for Scope {
    type Authority = Authority;
    type FormProof = u8;
    fn authority(&self) -> &Authority {
        &self.authority
    }
    fn kind(&self) -> DirectoryScopeKind {
        self.kind
    }
    fn list(
        &mut self,
    ) -> impl Future<Output = Result<DirectoryPage, DirectoryWorkflowError>> + Send {
        self.event("list");
        let mut records = vec![self.record()];
        match self.fault {
            Fault::DuplicateList => records.push(self.record()),
            Fault::ListOverLimit => {
                records = (0..26)
                    .map(|i| {
                        let mut r = self.record();
                        r.employee_id = id(4 + i);
                        r
                    })
                    .collect();
            }
            Fault::ListDescending => {
                let mut r = self.record();
                r.employee_id = id(8);
                records.insert(0, r);
            }
            _ => {}
        }
        let next_after = match self.fault {
            Fault::ListNextForeign => Some(id(99)),
            Fault::ListNextPartial | Fault::ListFullContinuation => Some(id(4)),
            _ => None,
        };
        ready(Ok(DirectoryPage {
            records,
            next_after,
        }))
    }

    fn detail(
        &mut self,
    ) -> impl Future<Output = Result<Option<DirectoryRecord>, DirectoryWorkflowError>> + Send {
        self.event("detail");
        ready(Ok(Some(self.record())))
    }
    fn form(
        &mut self,
    ) -> impl Future<Output = Result<DirectoryForm<u8>, DirectoryWorkflowError>> + Send {
        self.event("form");
        ready(Ok(DirectoryForm {
            locator: locator(),
            expected: self.form_expected,
            proof: 9,
        }))
    }
    fn preflight(&mut self) -> impl Future<Output = Result<(), DirectoryWorkflowError>> + Send {
        self.event("preflight");
        ready(Ok(()))
    }
    fn prepare(
        &mut self,
        _: &TraceContext,
    ) -> impl Future<Output = Result<DirectoryAcceptance, DirectoryWorkflowError>> + Send {
        self.event("prepare");
        ready(Ok(DirectoryAcceptance {
            inserted: self.fault != Fault::ReplayOld,
            status: DirectoryStatus::Pending(self.accepted()),
        }))
    }
    fn execute(
        &mut self,
        _: &TraceContext,
    ) -> impl Future<Output = Result<DirectoryExecution, DirectoryWorkflowError>> + Send {
        self.event("execute");
        let outcome = match self.fault {
            Fault::ExecuteCancels => DirectoryTerminalOutcomeV1::Cancelled,
            Fault::ExecuteRevisionConflict => {
                DirectoryTerminalOutcomeV1::Rejected(DirectoryRejectionV1::RevisionConflict)
            }
            Fault::ExecuteExpired => DirectoryTerminalOutcomeV1::Expired,
            _ => DirectoryTerminalOutcomeV1::Committed,
        };
        ready(Ok(DirectoryExecution {
            inserted: self.fault != Fault::ExecuteReplayOld,
            terminal: self.terminal(outcome),
        }))
    }

    fn cancel(
        &mut self,
        _: &TraceContext,
    ) -> impl Future<Output = Result<DirectoryExecution, DirectoryWorkflowError>> + Send {
        self.event("cancel");
        ready(Ok(DirectoryExecution {
            inserted: true,
            terminal: self.terminal(if self.fault == Fault::CancelCreatesEffect {
                DirectoryTerminalOutcomeV1::Committed
            } else {
                DirectoryTerminalOutcomeV1::Cancelled
            }),
        }))
    }
    fn status(
        &mut self,
        _: &TraceContext,
    ) -> impl Future<Output = Result<DirectoryRecovery<u8>, DirectoryWorkflowError>> + Send {
        self.event("status");
        let status = match self.fault {
            Fault::StatusRefreshExpired | Fault::StatusStaleExpiryClock => {
                let accepted = self.accepted();
                let terminal_at = accepted.execution_not_after() + Duration::seconds(2);
                assert!(terminal_at > self.authority.observed_at);
                let terminal = DirectoryTerminalV1::new(
                    accepted,
                    DirectoryTerminalOutcomeV1::Expired,
                    terminal_at,
                )
                .unwrap();
                if self.fault == Fault::StatusRefreshExpired {
                    self.authority.observed_at = terminal_at + Duration::microseconds(1);
                    self.event("refresh_authority");
                }
                DirectoryStatus::Terminal(terminal)
            }
            Fault::StatusExpired => {
                DirectoryStatus::Terminal(self.terminal(DirectoryTerminalOutcomeV1::Expired))
            }
            Fault::StatusHistoricalCommitted | Fault::StatusTerminalProof => {
                DirectoryStatus::Terminal(self.terminal(DirectoryTerminalOutcomeV1::Committed))
            }
            _ => DirectoryStatus::Pending(self.accepted()),
        };
        let proof = if self.fault == Fault::StatusMissingProof {
            None
        } else if self.fault == Fault::StatusTerminalProof
            || matches!(&status, DirectoryStatus::Pending(_))
        {
            Some(9)
        } else {
            None
        };
        ready(Ok(DirectoryRecovery { status, proof }))
    }
    fn finish<P: DirectoryDecisionPort<Authority> + ?Sized>(
        mut self,
        p: &P,
        proof: Option<&u8>,
    ) -> impl Future<Output = Result<(), DirectoryWorkflowError>> + Send {
        self.event("finish");
        let pending_status = matches!(self.kind, DirectoryScopeKind::Status(_))
            && !matches!(
                self.fault,
                Fault::StatusExpired
                    | Fault::StatusRefreshExpired
                    | Fault::StatusStaleExpiryClock
                    | Fault::StatusHistoricalCommitted
                    | Fault::StatusTerminalProof
            );
        assert_eq!(
            proof.copied(),
            (matches!(
                self.kind,
                DirectoryScopeKind::Form(_) | DirectoryScopeKind::ValidationForm(_)
            ) || pending_status)
                .then_some(9)
        );
        let result = match self.fault {
            Fault::FinishUnavailable => Err(DirectoryWorkflowError::Unavailable),
            Fault::FinishUnconfirmed => Err(DirectoryWorkflowError::Unconfirmed),
            _ => p
                .permits(
                    &self.authority,
                    DirectoryAccess {
                        company: self.authority.company,
                        actor: actor(),
                        action: self.authority.action,
                        resource: self.kind.resource(),
                    },
                )
                .and_then(|allow| {
                    if allow {
                        Ok(())
                    } else {
                        Err(DirectoryWorkflowError::NotFound)
                    }
                }),
        };
        self.finished = result.is_ok();
        ready(result)
    }
}
fn fixture(fault: Fault, deny_at: usize) -> (Store, Policy, Credentials) {
    (
        Store {
            fault,
            events: Arc::new(Mutex::new(vec![])),
        },
        Policy {
            calls: AtomicUsize::new(0),
            deny_at,
        },
        Credentials(42),
    )
}
fn events(store: &Store) -> Vec<&'static str> {
    store.events.lock().unwrap().clone()
}

#[test]
fn directory_workflow_refuses_wrong_source_or_policy_before_effect_method() {
    for fault in [Fault::Company, Fault::Action, Fault::Mode] {
        let (s, p, c) = fixture(fault, 0);
        assert!(matches!(
            run(directory_prepare(
                &s,
                &p,
                &c,
                &submission(),
                &TraceContext::generate()
            )),
            Err(DirectoryWorkflowError::Unavailable)
        ));
        assert_eq!(events(&s), vec!["lock", "discarded"]);
        assert_eq!(p.calls.load(Ordering::SeqCst), 0);
    }
    let (s, p, c) = fixture(Fault::None, 1);
    assert!(matches!(
        run(directory_prepare(
            &s,
            &p,
            &c,
            &submission(),
            &TraceContext::generate()
        )),
        Err(DirectoryWorkflowError::NotFound)
    ));
    assert_eq!(events(&s), vec!["lock", "discarded"]);
}
#[test]
fn directory_workflow_releases_no_provisional_acceptance_on_finish_failure() {
    for (fault, deny, error) in [
        (Fault::None, 2, DirectoryWorkflowError::NotFound),
        (
            Fault::FinishUnavailable,
            0,
            DirectoryWorkflowError::Unavailable,
        ),
        (
            Fault::FinishUnconfirmed,
            0,
            DirectoryWorkflowError::Unconfirmed,
        ),
    ] {
        let (s, p, c) = fixture(fault, deny);
        assert!(
            matches!(run(directory_prepare(&s,&p,&c,&submission(),&TraceContext::generate())),Err(e)if e==error)
        );
        assert_eq!(events(&s), vec!["lock", "prepare", "finish", "discarded"]);
    }
    // This pure scope cannot model physical post-dispatch COMMIT. The real adapter
    // must classify that uncertainty and reconcile; discarded here is Drop only.
}
#[test]
fn directory_workflow_rejects_receipt_identity_drift_and_new_stale_prepare() {
    for fault in [
        Fault::ReceiptActor,
        Fault::ReceiptCommand,
        Fault::CurrentEpoch,
    ] {
        let (s, p, c) = fixture(fault, 0);
        assert!(
            run(directory_prepare(
                &s,
                &p,
                &c,
                &submission(),
                &TraceContext::generate()
            ))
            .is_err()
        );
        assert_eq!(events(&s), vec!["lock", "prepare", "discarded"]);
    }
    let (s, p, c) = fixture(Fault::ReplayOld, 0);
    let r = run(directory_prepare(
        &s,
        &p,
        &c,
        &submission(),
        &TraceContext::generate(),
    ))
    .unwrap();
    assert!(!r.inserted);
    assert_eq!(
        r.status
            .accepted()
            .unwrap()
            .command()
            .expected()
            .company_epoch,
        7
    );
    assert_eq!(events(&s), vec!["lock", "prepare", "finish", "finished"]);
}
#[test]
fn directory_workflow_binds_mutation_mode_and_original_request() {
    for fault in [
        Fault::ReceiptActor,
        Fault::ReceiptCommand,
        Fault::ExecuteCancels,
    ] {
        let (s, p, c) = fixture(fault, 0);
        assert!(
            run(directory_execute(
                &s,
                &p,
                &c,
                locator(),
                &TraceContext::generate()
            ))
            .is_err()
        );
        assert_eq!(events(&s), vec!["lock", "execute", "discarded"]);
    }
    let (s, p, c) = fixture(Fault::CancelCreatesEffect, 0);
    assert!(
        run(directory_cancel(
            &s,
            &p,
            &c,
            locator(),
            &TraceContext::generate()
        ))
        .is_err()
    );
    assert_eq!(events(&s), vec!["lock", "cancel", "discarded"]);
    let (s, p, c) = fixture(Fault::ReceiptActor, 0);
    assert!(
        run(directory_status(
            &s,
            &p,
            &c,
            locator(),
            &TraceContext::generate()
        ))
        .is_err()
    );
    assert_eq!(events(&s), vec!["lock", "status", "discarded"]);
}
#[test]
fn directory_workflow_read_shape_and_final_authorization_bound_disclosure() {
    let (s, p, c) = fixture(Fault::WrongDetail, 0);
    assert!(run(directory_detail(&s, &p, &c, company(), id(4))).is_err());
    assert_eq!(events(&s), vec!["lock", "detail", "discarded"]);
    let (s, p, c) = fixture(Fault::DuplicateList, 0);
    assert!(
        run(directory_list(
            &s,
            &p,
            &c,
            company(),
            DirectoryPageQuery::new(None, None).unwrap()
        ))
        .is_err()
    );
    assert_eq!(events(&s), vec!["lock", "list", "discarded"]);
    let (s, p, c) = fixture(Fault::None, 2);
    assert!(run(directory_detail(&s, &p, &c, company(), id(4))).is_err());
    assert_eq!(events(&s), vec!["lock", "detail", "finish", "discarded"]);
    let (s, p, c) = fixture(Fault::None, 0);
    let r = run(directory_detail(&s, &p, &c, company(), id(4)))
        .unwrap()
        .unwrap();
    assert_eq!(r.person_id, id(44));
    assert_eq!(events(&s), vec!["lock", "detail", "finish", "finished"]);
}
#[test]
fn directory_workflow_form_proof_and_pure_preflight_follow_distinct_modes() {
    for validation in [false, true] {
        let (s, p, c) = fixture(Fault::None, 0);
        assert_eq!(
            run(directory_form(
                &s,
                &p,
                &c,
                locator(),
                validation.then_some(expected())
            ))
            .unwrap()
            .proof,
            9
        );
        assert_eq!(events(&s), vec!["lock", "form", "finish", "finished"]);
    }
    let (s, p, c) = fixture(Fault::CurrentEpoch, 0);
    let original = expected();
    let form = run(directory_form(&s, &p, &c, locator(), Some(original))).unwrap();
    assert_eq!(form.expected, original);
    assert_eq!(form.proof, 9);
    assert_eq!(events(&s), vec!["lock", "form", "finish", "finished"]);
    let (s, p, c) = fixture(Fault::None, 0);
    run(directory_preflight(&s, &p, &c, &submission())).unwrap();
    assert_eq!(events(&s), vec!["lock", "preflight", "finish", "finished"]);
    let (s, p, c) = fixture(Fault::CurrentEpoch, 0);
    assert_eq!(
        run(directory_preflight(&s, &p, &c, &submission())),
        Err(DirectoryWorkflowError::Conflict)
    );
    assert_eq!(events(&s), vec!["lock", "discarded"]);
    assert!(DirectoryRequestRef::new(OrgId::platform(), id(3)).is_err());
    assert!(DirectoryRequestRef::new(company(), Uuid::nil()).is_err());
    for limit in [0, 101, u16::MAX] {
        assert!(DirectoryPageQuery::new(None, Some(limit)).is_err());
    }
    assert!(DirectoryPageQuery::new(Some(Uuid::nil()), None).is_err());
    assert_eq!(DirectoryPageQuery::new(None, None).unwrap().limit(), 25);
}

#[test]
fn directory_new_effect_requires_current_expectations_but_historical_or_rejected_outcomes_survive()
{
    let (s, p, c) = fixture(Fault::CurrentEpoch, 0);
    assert!(matches!(
        run(directory_execute(
            &s,
            &p,
            &c,
            locator(),
            &TraceContext::generate()
        )),
        Err(DirectoryWorkflowError::Unavailable)
    ));
    assert_eq!(events(&s), vec!["lock", "execute", "discarded"]);
    for fault in [
        Fault::None,
        Fault::ExecuteReplayOld,
        Fault::ExecuteRevisionConflict,
        Fault::ExecuteExpired,
    ] {
        let (s, p, c) = fixture(fault, 0);
        let result = run(directory_execute(
            &s,
            &p,
            &c,
            locator(),
            &TraceContext::generate(),
        ))
        .unwrap();
        assert_eq!(result.inserted, fault != Fault::ExecuteReplayOld);
        assert_eq!(
            result
                .terminal
                .accepted()
                .command()
                .expected()
                .company_epoch,
            7
        );
        let expected = match fault {
            Fault::ExecuteRevisionConflict => {
                DirectoryTerminalOutcomeV1::Rejected(DirectoryRejectionV1::RevisionConflict)
            }
            Fault::ExecuteExpired => DirectoryTerminalOutcomeV1::Expired,
            _ => DirectoryTerminalOutcomeV1::Committed,
        };
        assert_eq!(result.terminal.outcome(), expected);
        assert_eq!(events(&s), vec!["lock", "execute", "finish", "finished"]);
    }
}
#[test]
fn directory_recovery_proof_and_expiry_are_bound_to_one_scope() {
    for fault in [
        Fault::StatusMissingProof,
        Fault::StatusTerminalProof,
        Fault::StatusExpiredPending,
    ] {
        let (s, p, c) = fixture(fault, 0);
        assert!(matches!(
            run(directory_status(
                &s,
                &p,
                &c,
                locator(),
                &TraceContext::generate()
            )),
            Err(DirectoryWorkflowError::Unavailable)
        ));
        assert_eq!(events(&s), vec!["lock", "status", "discarded"]);
    }
    for fault in [
        Fault::None,
        Fault::StatusExpired,
        Fault::StatusHistoricalCommitted,
    ] {
        let (s, p, c) = fixture(fault, 0);
        let result = run(directory_status(
            &s,
            &p,
            &c,
            locator(),
            &TraceContext::generate(),
        ))
        .unwrap();
        assert_eq!(
            result.proof,
            if fault == Fault::None { Some(9) } else { None }
        );
        match (&result.status, fault) {
            (DirectoryStatus::Pending(_), Fault::None) => {}
            (DirectoryStatus::Terminal(t), Fault::StatusExpired) => {
                assert_eq!(t.outcome(), DirectoryTerminalOutcomeV1::Expired)
            }
            (DirectoryStatus::Terminal(t), Fault::StatusHistoricalCommitted) => {
                assert_eq!(t.outcome(), DirectoryTerminalOutcomeV1::Committed);
                assert_eq!(t.accepted().command().expected().company_epoch, 7);
            }
            _ => panic!("unexpected recovery branch"),
        }
        assert_eq!(events(&s), vec!["lock", "status", "finish", "finished"]);
    }
    let (s, p, c) = fixture(Fault::None, 2);
    assert!(matches!(
        run(directory_status(
            &s,
            &p,
            &c,
            locator(),
            &TraceContext::generate()
        )),
        Err(DirectoryWorkflowError::NotFound)
    ));
    assert_eq!(events(&s), vec!["lock", "status", "finish", "discarded"]);
}
#[test]
fn directory_collection_rejects_bounds_order_cursor_and_next_link_corruption() {
    for fault in [
        Fault::ListOverLimit,
        Fault::ListDescending,
        Fault::ListAtCursor,
        Fault::ListNextForeign,
        Fault::ListNextPartial,
    ] {
        let (s, p, c) = fixture(fault, 0);
        let q = DirectoryPageQuery::new(
            if fault == Fault::ListAtCursor {
                Some(id(4))
            } else {
                None
            },
            None,
        )
        .unwrap();
        assert!(matches!(
            run(directory_list(&s, &p, &c, company(), q)),
            Err(DirectoryWorkflowError::Unavailable)
        ));
        assert_eq!(events(&s), vec!["lock", "list", "discarded"]);
    }
    let (s, p, c) = fixture(Fault::None, 0);
    let result = run(directory_list(
        &s,
        &p,
        &c,
        company(),
        DirectoryPageQuery::new(Some(id(3)), None).unwrap(),
    ))
    .unwrap();
    assert_eq!(result.records[0].employee_id, id(4));
    assert_eq!(events(&s), vec!["lock", "list", "finish", "finished"]);
}

#[test]
fn directory_status_uses_refreshed_retained_observation_after_expiry_materialization() {
    let (s, p, c) = fixture(Fault::StatusRefreshExpired, 0);
    let result = run(directory_status(
        &s,
        &p,
        &c,
        locator(),
        &TraceContext::generate(),
    ))
    .unwrap();
    let DirectoryStatus::Terminal(terminal) = result.status else {
        panic!("expiry terminal required")
    };
    assert_eq!(terminal.outcome(), DirectoryTerminalOutcomeV1::Expired);
    assert_eq!(
        terminal.terminal_at(),
        terminal.accepted().execution_not_after() + Duration::seconds(2)
    );
    assert!(result.proof.is_none());
    assert_eq!(
        events(&s),
        vec!["lock", "status", "refresh_authority", "finish", "finished"]
    );
    let (s, p, c) = fixture(Fault::StatusStaleExpiryClock, 0);
    assert!(matches!(
        run(directory_status(
            &s,
            &p,
            &c,
            locator(),
            &TraceContext::generate()
        )),
        Err(DirectoryWorkflowError::Unavailable)
    ));
    assert_eq!(events(&s), vec!["lock", "status", "discarded"]);
}

#[test]
fn directory_collection_accepts_full_page_with_last_record_continuation() {
    let (s, p, c) = fixture(Fault::ListFullContinuation, 0);
    let result = run(directory_list(
        &s,
        &p,
        &c,
        company(),
        DirectoryPageQuery::new(Some(id(3)), Some(1)).unwrap(),
    ))
    .unwrap();
    assert_eq!(result.records.len(), 1);
    assert_eq!(result.records[0].employee_id, id(4));
    assert_eq!(result.next_after, Some(id(4)));
    assert_eq!(events(&s), vec!["lock", "list", "finish", "finished"]);
}
