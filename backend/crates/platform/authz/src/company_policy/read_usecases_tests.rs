// Proposed ordinary child module of company_policy::tests. Reuses input_row()
// and real CompanyPolicy. Scripted scopes prove application orchestration only.
use super::{CompanyPolicy, input_row};
use console_identity_application::company_policy::{
    CompanyContextCandidates, CompanyPolicyDecision, CompanyPolicyDecisionPort, CompanyPolicyError,
    CompanyPolicyRequest, CompanyPolicyScope, CompanyPolicyStore, CurrentCompanyAuthority,
    InitialCompanyAction, discover_company_context, discover_company_contexts,
    read_company_identity, read_company_policy,
};
use console_kernel_core::OrgId;
use std::{
    collections::VecDeque,
    future::{Future, poll_fn},
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Event {
    Enumerate(u64),
    Lock(OrgId),
    Decision(OrgId, InitialCompanyAction, usize),
    Finish(OrgId),
    Finished(OrgId, Result<(), CompanyPolicyError>),
    Release(OrgId, bool),
    Check(u64),
}
#[derive(Default)]
struct Trace {
    events: Vec<Event>,
    active: Option<OrgId>,
}
struct ScopePlan {
    requested: OrgId,
    material: Option<CurrentCompanyAuthority>,
    finish: Result<(), CompanyPolicyError>,
    pending_finish: bool,
}
struct Store {
    trace: Arc<Mutex<Trace>>,
    scopes: Mutex<VecDeque<ScopePlan>>,
    enumerations: Mutex<VecDeque<CompanyContextCandidates>>,
    checks: Mutex<VecDeque<Result<(), CompanyPolicyError>>>,
}
struct Scope<'a> {
    store: &'a Store,
    plan: ScopePlan,
    committed: bool,
}
struct Credentials;
impl Store {
    fn new(scopes: Vec<ScopePlan>) -> Self {
        Self {
            trace: Arc::new(Mutex::new(Trace::default())),
            scopes: Mutex::new(scopes.into()),
            enumerations: Mutex::new(VecDeque::new()),
            checks: Mutex::new(VecDeque::new()),
        }
    }
    fn discovery(
        self,
        rounds: Vec<(u64, Vec<OrgId>)>,
        checks: Vec<Result<(), CompanyPolicyError>>,
    ) -> Self {
        *self.enumerations.lock().unwrap() = rounds
            .into_iter()
            .map(|(generation, companies)| {
                CompanyContextCandidates::new(generation, companies).unwrap()
            })
            .collect();
        *self.checks.lock().unwrap() = checks.into();
        self
    }
    fn events(&self) -> Vec<Event> {
        self.trace.lock().unwrap().events.clone()
    }
    fn drained(&self) {
        assert!(self.scopes.lock().unwrap().is_empty());
        assert!(self.enumerations.lock().unwrap().is_empty());
        assert!(self.checks.lock().unwrap().is_empty());
        assert!(self.trace.lock().unwrap().active.is_none());
    }
}
impl CompanyPolicyScope for Scope<'_> {
    fn authority(&self) -> Option<&CurrentCompanyAuthority> {
        self.plan.material.as_ref()
    }
    async fn finish(mut self) -> Result<(), CompanyPolicyError> {
        self.store
            .trace
            .lock()
            .unwrap()
            .events
            .push(Event::Finish(self.plan.requested));
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
        self.committed = self.plan.finish.is_ok();
        self.store
            .trace
            .lock()
            .unwrap()
            .events
            .push(Event::Finished(self.plan.requested, self.plan.finish));
        self.plan.finish
    }
}
impl Drop for Scope<'_> {
    fn drop(&mut self) {
        let mut t = self.store.trace.lock().unwrap();
        assert_eq!(t.active, Some(self.plan.requested));
        t.active = None;
        t.events
            .push(Event::Release(self.plan.requested, self.committed));
    }
}
impl CompanyPolicyStore for Store {
    type Credentials = Credentials;
    type Scope<'a>
        = Scope<'a>
    where
        Self: 'a;
    async fn lock_current<'a>(
        &'a self,
        _: &'a Credentials,
        company: OrgId,
    ) -> Result<Scope<'a>, CompanyPolicyError> {
        let plan = self
            .scopes
            .lock()
            .unwrap()
            .pop_front()
            .expect("unexpected additional Company scope");
        assert_eq!(plan.requested, company);
        let mut t = self.trace.lock().unwrap();
        assert!(t.active.is_none(), "Company scopes overlap");
        t.active = Some(company);
        t.events.push(Event::Lock(company));
        Ok(Scope {
            store: self,
            plan,
            committed: false,
        })
    }
    async fn enumerate_company_candidates(
        &self,
        _: &Credentials,
    ) -> Result<CompanyContextCandidates, CompanyPolicyError> {
        let next = self
            .enumerations
            .lock()
            .unwrap()
            .pop_front()
            .expect("unexpected discovery retry");
        let mut t = self.trace.lock().unwrap();
        assert!(t.active.is_none(), "enumeration retained a Company scope");
        t.events.push(Event::Enumerate(next.generation()));
        Ok(next)
    }
    async fn check_context_generation(
        &self,
        _: &Credentials,
        generation: u64,
    ) -> Result<(), CompanyPolicyError> {
        let mut t = self.trace.lock().unwrap();
        assert!(
            t.active.is_none(),
            "generation check preceded Company scope finish/drop"
        );
        t.events.push(Event::Check(generation));
        drop(t);
        self.checks
            .lock()
            .unwrap()
            .pop_front()
            .expect("unexpected generation check")
    }
}
struct Policy {
    real: CompanyPolicy,
    trace: Arc<Mutex<Trace>>,
    overrides: Vec<(
        OrgId,
        InitialCompanyAction,
        Result<CompanyPolicyDecision, CompanyPolicyError>,
    )>,
}
impl Policy {
    fn new(store: &Store) -> Self {
        Self {
            real: CompanyPolicy::new().unwrap(),
            trace: store.trace.clone(),
            overrides: vec![],
        }
    }
    fn override_decision(
        mut self,
        company: OrgId,
        action: InitialCompanyAction,
        result: Result<CompanyPolicyDecision, CompanyPolicyError>,
    ) -> Self {
        self.overrides.push((company, action, result));
        self
    }
}
impl CompanyPolicyDecisionPort for Policy {
    fn decide_native_bootstrap(
        &self,
        authority: &console_identity_application::company_policy::CurrentNativeBootstrapAuthority,
        request: &console_identity_application::company_policy::NativeBootstrapRequestV1,
    ) -> Result<CompanyPolicyDecision, CompanyPolicyError> {
        self.real.decide_native_bootstrap(authority, request)
    }

    fn decide_native_payroll_collection(
        &self,
        authority: &console_identity_application::company_policy::CurrentPayrollReadAuthority,
    ) -> Result<CompanyPolicyDecision, CompanyPolicyError> {
        self.real.decide_native_payroll_collection(authority)
    }

    fn decide(
        &self,
        a: &CurrentCompanyAuthority,
        r: &CompanyPolicyRequest,
    ) -> Result<CompanyPolicyDecision, CompanyPolicyError> {
        let clause = a
            .clauses()
            .iter()
            .find(|c| !c.delegable() && c.action() == r.action())
            .unwrap();
        let kind = clause.action_kind();
        assert_eq!(r.requested_company(), a.company());
        assert_eq!(r.object_type_id(), clause.action().object_type_id());
        assert_eq!(
            r.object_id(),
            if matches!(
                kind,
                InitialCompanyAction::Discover | InitialCompanyAction::ReadIdentity
            ) {
                *a.company().as_uuid()
            } else {
                a.assignment_id()
            }
        );
        assert!(
            r.requested_properties() == clause.fields()
                || (kind == InitialCompanyAction::ReadPolicy
                    && r.requested_properties().is_empty())
        );
        let mut t = self.trace.lock().unwrap();
        assert_eq!(t.active, Some(a.company()));
        t.events.push(Event::Decision(
            a.company(),
            kind,
            r.requested_properties().len(),
        ));
        drop(t);
        if let Some((_, _, result)) = self
            .overrides
            .iter()
            .find(|(company, action, _)| *company == a.company() && *action == kind)
        {
            return *result;
        }
        self.real.decide(a, r)
    }
}
fn company(id: u128) -> OrgId {
    OrgId::from_uuid(Uuid::from_u128(id))
}
fn material(id: u128, generation: i64, name: &str) -> CurrentCompanyAuthority {
    let (account, original, at, mut row) = input_row();
    let requested = company(id);
    row.registered_clauses = row
        .registered_clauses
        .replace(&original.to_string(), &requested.to_string());
    row.context_generation = generation;
    row.company_name = name.into();
    row.company_slug = format!("company-{id}");
    CurrentCompanyAuthority::from_initial_projection(account, requested, at, row).unwrap()
}
fn plan(id: u128, generation: i64, name: &str) -> ScopePlan {
    ScopePlan {
        requested: company(id),
        material: Some(material(id, generation, name)),
        finish: Ok(()),
        pending_finish: false,
    }
}
fn poll_once<F: Future>(future: Pin<&mut F>) -> Poll<F::Output> {
    future.poll(&mut Context::from_waker(Waker::noop()))
}
fn ready<F: Future>(future: F) -> F::Output {
    let mut pinned = Box::pin(future);
    match poll_once(pinned.as_mut()) {
        Poll::Ready(v) => v,
        Poll::Pending => panic!("fixture unexpectedly waited outside selected finish"),
    }
}
fn completed(company: OrgId, decisions: &[(InitialCompanyAction, usize)]) -> Vec<Event> {
    let mut events = vec![Event::Lock(company)];
    events.extend(
        decisions
            .iter()
            .map(|(kind, n)| Event::Decision(company, *kind, *n)),
    );
    events.extend([
        Event::Finish(company),
        Event::Finished(company, Ok(())),
        Event::Release(company, true),
    ]);
    events
}

#[test]
fn workspace_and_policy_reads_use_exact_fields_and_complete_retained_scope() {
    use InitialCompanyAction::{ReadIdentity, ReadPolicy};
    for deny_navigation in [false, true] {
        let store = Store::new(vec![plan(1, 2, "회사 이름")]);
        let mut policy = Policy::new(&store);
        if deny_navigation {
            policy =
                policy.override_decision(company(1), ReadPolicy, Ok(CompanyPolicyDecision::Deny));
        }
        let view = ready(read_company_identity(
            &store,
            &policy as &dyn CompanyPolicyDecisionPort,
            &Credentials,
            company(1),
        ))
        .unwrap();
        assert_eq!(view.org_id, company(1));
        assert_eq!(view.name, "회사 이름");
        assert_eq!(view.slug, "company-1");
        assert_eq!(view.show_policy_navigation, !deny_navigation);
        assert_eq!(
            store.events(),
            completed(company(1), &[(ReadIdentity, 2), (ReadPolicy, 0)])
        );
        store.drained();
    }
    let authority = material(1, 2, "not a policy-readable Company name");
    let store = Store::new(vec![plan(1, 2, authority.name())]);
    let policy = Policy::new(&store);
    let view = ready(read_company_policy(
        &store,
        &policy,
        &Credentials,
        company(1),
    ))
    .unwrap();
    let ceiling = view.initial_ceiling;
    assert_eq!(ceiling.org_id, company(1));
    assert_eq!(ceiling.account_id, authority.account());
    assert_eq!(
        ceiling.action_keys,
        authority.clauses()[..5]
            .iter()
            .map(|c| c.action_kind().as_str())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        ceiling.action_refs,
        authority.clauses()[..5]
            .iter()
            .map(|c| c.action().clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        ceiling.delegable_action_keys,
        vec!["context.discover", "company.identity.read"]
    );
    assert_eq!(
        ceiling.delegable_action_refs,
        authority.clauses()[5..]
            .iter()
            .map(|c| c.action().clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        ceiling.company_property_keys,
        vec!["company.name", "company.slug"]
    );
    assert_eq!(
        ceiling.delegable_company_property_keys,
        ceiling.company_property_keys
    );
    assert_eq!(
        ceiling.company_property_refs,
        authority.clauses()[0].fields()
    );
    assert_eq!(
        ceiling.delegable_company_property_refs,
        ceiling.company_property_refs
    );
    assert!(!ceiling.future_registrations && !ceiling.group_control);
    assert_eq!(store.events(), completed(company(1), &[(ReadPolicy, 8)]));
    store.drained();
}

#[test]
fn pending_finish_failures_discard_all_authorized_view_types() {
    use CompanyPolicyError::{
        AuthenticationInvalid, Conflict, EvaluatorUnavailable, MaterialUnavailable,
    };
    use InitialCompanyAction::{Discover, ReadIdentity, ReadPolicy};
    for kind in 0..3 {
        for error in [
            AuthenticationInvalid,
            Conflict,
            MaterialUnavailable,
            EvaluatorUnavailable,
        ] {
            let mut p = plan(1, 2, "must not escape unfinished scope");
            p.finish = Err(error);
            p.pending_finish = true;
            let store = Store::new(vec![p]);
            let policy = Policy::new(&store);
            let mut future = Box::pin(async {
                match kind {
                    0 => read_company_identity(&store, &policy, &Credentials, company(1))
                        .await
                        .map(|_| ()),
                    1 => read_company_policy(&store, &policy, &Credentials, company(1))
                        .await
                        .map(|_| ()),
                    _ => discover_company_context(&store, &policy, &Credentials, company(1))
                        .await
                        .map(|_| ()),
                }
            });
            assert!(
                matches!(poll_once(future.as_mut()), Poll::Pending),
                "view returned before pending finish"
            );
            let decisions = match kind {
                0 => vec![(ReadIdentity, 2), (ReadPolicy, 0)],
                1 => vec![(ReadPolicy, 8)],
                _ => vec![(Discover, 2)],
            };
            let mut expected = vec![Event::Lock(company(1))];
            expected.extend(
                decisions
                    .into_iter()
                    .map(|(a, n)| Event::Decision(company(1), a, n)),
            );
            expected.push(Event::Finish(company(1)));
            assert_eq!(store.events(), expected);
            assert_eq!(store.trace.lock().unwrap().active, Some(company(1)));
            assert!(matches!(poll_once(future.as_mut()),Poll::Ready(Err(actual)) if actual==error));
            drop(future);
            expected.extend([
                Event::Finished(company(1), Err(error)),
                Event::Release(company(1), false),
            ]);
            assert_eq!(store.events(), expected);
            store.drained();
        }
    }
}

#[test]
fn discovery_restarts_after_generation_conflict_once_and_discards_old_labels() {
    use CompanyPolicyError::{Conflict, MaterialUnavailable};
    use InitialCompanyAction::Discover;
    let store = Store::new(vec![
        plan(1, 2, "old projection"),
        plan(1, 3, "fresh first"),
        plan(2, 3, "fresh second"),
    ])
    .discovery(
        vec![(2, vec![company(1)]), (3, vec![company(1), company(2)])],
        vec![Err(Conflict), Ok(())],
    );
    let policy = Policy::new(&store);
    let views = ready(discover_company_contexts(&store, &policy, &Credentials)).unwrap();
    assert_eq!(
        views
            .iter()
            .map(|v| (v.org_id, v.name.as_str(), v.slug.as_str()))
            .collect::<Vec<_>>(),
        vec![
            (company(1), "fresh first", "company-1"),
            (company(2), "fresh second", "company-2")
        ]
    );
    let mut expected = vec![Event::Enumerate(2)];
    expected.extend(completed(company(1), &[(Discover, 2)]));
    expected.extend([Event::Check(2), Event::Enumerate(3)]);
    expected.extend(completed(company(1), &[(Discover, 2)]));
    expected.extend(completed(company(2), &[(Discover, 2)]));
    expected.push(Event::Check(3));
    assert_eq!(store.events(), expected);
    store.drained();
    let store = Store::new(vec![
        plan(1, 2, "discard first"),
        plan(1, 2, "discard second"),
    ])
    .discovery(
        vec![(2, vec![company(1)]), (2, vec![company(1)])],
        vec![Err(Conflict), Err(Conflict)],
    );
    let policy = Policy::new(&store);
    assert!(matches!(
        ready(discover_company_contexts(&store, &policy, &Credentials)),
        Err(MaterialUnavailable)
    ));
    assert_eq!(
        store
            .events()
            .iter()
            .filter(|e| matches!(e, Event::Enumerate(_)))
            .count(),
        2
    );
    assert_eq!(
        store
            .events()
            .iter()
            .filter(|e| matches!(e, Event::Check(_)))
            .count(),
        2
    );
    store.drained();
}

#[test]
fn discovery_omits_healthy_absence_or_denial_but_never_returns_faulted_partial_results() {
    use CompanyPolicyError::{AuthenticationInvalid, EvaluatorUnavailable};
    use InitialCompanyAction::Discover;
    for mode in 0..4 {
        let mut second = plan(2, 3, "restricted second");
        if mode == 0 {
            second.material = None;
        }
        if mode == 3 {
            second.finish = Err(AuthenticationInvalid);
        }
        let healthy = mode < 2;
        let store = Store::new(vec![plan(1, 3, "first allowed"), second]).discovery(
            vec![(3, vec![company(1), company(2)])],
            if healthy { vec![Ok(())] } else { vec![] },
        );
        let mut policy = Policy::new(&store);
        if mode == 1 {
            policy =
                policy.override_decision(company(2), Discover, Ok(CompanyPolicyDecision::Deny));
        }
        if mode == 2 {
            policy = policy.override_decision(company(2), Discover, Err(EvaluatorUnavailable));
        }
        let result = ready(discover_company_contexts(&store, &policy, &Credentials));
        if healthy {
            let views = result.unwrap();
            assert_eq!(views.len(), 1);
            assert_eq!(views[0].org_id, company(1));
            assert_eq!(views[0].name, "first allowed");
        } else {
            assert!(
                matches!(result,Err(e) if e==if mode==2 {EvaluatorUnavailable}else{AuthenticationInvalid})
            );
        }
        let events = store.events();
        assert_eq!(
            &events[..7],
            &[
                Event::Enumerate(3),
                Event::Lock(company(1)),
                Event::Decision(company(1), Discover, 2),
                Event::Finish(company(1)),
                Event::Finished(company(1), Ok(())),
                Event::Release(company(1), true),
                Event::Lock(company(2))
            ]
        );
        if healthy {
            assert!(events.contains(&Event::Finished(company(2), Ok(()))));
            assert_eq!(events.last(), Some(&Event::Check(3)));
        } else {
            assert!(!events.iter().any(|e| matches!(e, Event::Check(_))));
        }
        if mode == 0 {
            assert!(
                !events
                    .iter()
                    .any(|e| matches!(e,Event::Decision(c,_,_) if *c==company(2)))
            );
        }
        store.drained();
    }
    let store = Store::new(vec![]).discovery(vec![(1, vec![])], vec![Ok(())]);
    let policy = Policy::new(&store);
    assert!(
        ready(discover_company_contexts(&store, &policy, &Credentials))
            .unwrap()
            .is_empty()
    );
    assert_eq!(store.events(), vec![Event::Enumerate(1), Event::Check(1)]);
    store.drained();
}

#[test]
fn wrong_company_material_never_reaches_policy_and_absence_requires_fresh_finish() {
    use CompanyPolicyError::{AuthenticationInvalid, MaterialUnavailable};
    let mut wrong = plan(1, 2, "foreign material");
    wrong.material = Some(material(2, 2, "foreign secret"));
    let store = Store::new(vec![wrong]);
    let policy = Policy::new(&store);
    assert!(matches!(
        ready(read_company_identity(
            &store,
            &policy,
            &Credentials,
            company(1)
        )),
        Err(MaterialUnavailable)
    ));
    assert!(
        !store
            .events()
            .iter()
            .any(|e| matches!(e, Event::Decision(_, _, _)))
    );
    store.drained();
    let mut absent = plan(1, 2, "absent");
    absent.material = None;
    absent.finish = Err(AuthenticationInvalid);
    let store = Store::new(vec![absent]);
    let policy = Policy::new(&store);
    assert!(
        matches!(
            ready(read_company_identity(
                &store,
                &policy,
                &Credentials,
                company(1)
            )),
            Err(AuthenticationInvalid)
        ),
        "stale authentication became healthy absence"
    );
    assert_eq!(
        store.events(),
        vec![
            Event::Lock(company(1)),
            Event::Finish(company(1)),
            Event::Finished(company(1), Err(AuthenticationInvalid)),
            Event::Release(company(1), false)
        ]
    );
    store.drained();
}

#[test]
fn company_context_candidates_validate_bounds_without_inventing_generation_provenance() {
    use CompanyPolicyError::MaterialUnavailable;
    let full: Vec<_> = (1..=256).map(company).collect();
    // The constructor checks representation only. Either generation boundary
    // accepts either collection boundary; only the Account owner proves origin.
    for generation in [1, 257] {
        for companies in [Vec::new(), full.clone()] {
            let candidates = CompanyContextCandidates::new(generation, companies.clone()).unwrap();
            assert_eq!(candidates.generation(), generation);
            assert_eq!(candidates.companies(), companies.as_slice());
        }
    }
    for (name, generation, companies) in [
        ("zero generation", 0, vec![]),
        ("overflow generation", 258, vec![]),
        ("nil Company", 1, vec![OrgId::from_uuid(Uuid::nil())]),
        ("platform sentinel", 1, vec![OrgId::platform()]),
        ("duplicate Company", 3, vec![company(1), company(1)]),
        ("unsorted Company", 3, vec![company(2), company(1)]),
        ("257 Companies", 257, (1..=257).map(company).collect()),
    ] {
        assert!(
            matches!(
                CompanyContextCandidates::new(generation, companies),
                Err(MaterialUnavailable)
            ),
            "accepted {name}"
        );
    }
}

#[test]
fn pure_company_identity_projection_requires_exact_independent_field_decision() {
    use console_identity_application::company_policy::project_company_identity;
    for decision in [Ok(CompanyPolicyDecision::Allow), Ok(CompanyPolicyDecision::Deny),
        Err(CompanyPolicyError::EvaluatorUnavailable)] {
        let store = Store::new(vec![]);
        // Pure decision fixture: no real transaction or database authority claim.
        store.trace.lock().unwrap().active = Some(company(1));
        let policy = Policy::new(&store).override_decision(company(1), InitialCompanyAction::ReadIdentity, decision);
        let authority = material(1, 2, "허가된 회사 <이름>");
        let actual = project_company_identity(&policy, &authority);
        match decision {
            Ok(CompanyPolicyDecision::Allow) => {
                let view = actual.unwrap();
                assert_eq!(view.org_id, company(1));
                assert_eq!(view.name, "허가된 회사 <이름>");
                assert_eq!(view.slug, "company-1");
            }
            Ok(CompanyPolicyDecision::Deny) => assert_eq!(actual, Err(CompanyPolicyError::NotFound)),
            Err(error) => assert_eq!(actual, Err(error)),
        }
        assert_eq!(store.events(), [Event::Decision(company(1), InitialCompanyAction::ReadIdentity, 2)]);
    }
}
