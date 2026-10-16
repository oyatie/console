// Proposed private child of postimage_contract_tests. Reuses its independently
// declared current-source fixtures; no business data is stored or resealed.
// This scripted scope proves application orchestration, not real Auth, proof
// validation, limiter accounting, SQL custody, SHA execution or browser UX.
mod validation_form_workflow {
    use super::*;
    use std::{
        future::Future,
        sync::{Arc, Mutex},
        task::{Context, Poll, Wake, Waker},
    };

    const ORIGINAL_COMMAND: u128 = 71;
    const ORIGINAL_PROCESS: u128 = 72;

    struct NoopWake;
    impl Wake for NoopWake {
        fn wake(self: Arc<Self>) {}
    }
    fn ready<F: Future>(future: F) -> F::Output {
        let waker = Waker::from(Arc::new(NoopWake));
        let mut cx = Context::from_waker(&waker);
        let mut future = std::pin::pin!(future);
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(value) => value,
            Poll::Pending => panic!("scripted immediate scope unexpectedly awaited I/O"),
        }
    }

    // This marker is an opaque port object, never a real CSRF token or claim of
    // Auth validity. Identity equality catches replacing the proof en route.
    struct Proof(Arc<()>);
    #[derive(Clone, Copy)]
    enum ResponseFault {
        None,
        Command,
        Process,
        NilCommand,
        NilProcess,
        View(fn(&mut GroupProcessCurrentView)),
    }
    struct Store {
        events: Mutex<Vec<&'static str>>,
        retained_proof: Arc<()>,
        empty: bool,
        ordinary: bool,
        fault: ResponseFault,
        lock_error: Option<GroupProcessError>,
        form_error: Option<GroupProcessError>,
        finish_error: Option<GroupProcessError>,
        source_incarnation: GroupIncarnation,
        pending_finish: bool,
    }
    struct Scope<'a> {
        store: &'a Store,
        authority: CurrentGroupProcessAuthority,
        original_command: Uuid,
        original_process: Uuid,
        ordinary: bool,
    }
    impl Store {
        fn new(empty: bool) -> Self {
            Self {
                events: Mutex::new(Vec::new()),
                retained_proof: Arc::new(()),
                empty,
                ordinary: false,
                fault: ResponseFault::None,
                lock_error: None,
                form_error: None,
                finish_error: None,
                source_incarnation: incarnation(),
                pending_finish: false,
            }
        }
        fn record(&self, event: &'static str) {
            self.events.lock().unwrap().push(event);
        }
        fn events(&self) -> Vec<&'static str> {
            self.events.lock().unwrap().clone()
        }
    }
    fn original() -> GroupProcessLocator {
        GroupProcessLocator::new(group(), incarnation(), id(ORIGINAL_COMMAND)).unwrap()
    }
    fn source_row(store: &Store) -> GroupProcessCurrentProjectionV1 {
        let mut row = fixture("first_valid", FINAL_US).projected;
        row.mode = GroupProcessMaterialModeV1::Form;
        row.original_status = None;
        row.incarnation = store.source_incarnation;
        if store.empty {
            row.policy_head = PolicyHeadReferenceV1::Absent;
            row.head = None;
            row.version = None;
            row.history.clear();
        }
        row
    }
    fn expected_view(store: &Store) -> GroupProcessCurrentView {
        let request = GroupProcessScopeRequest::Form {
            group: group(),
            incarnation: incarnation(),
        };
        CurrentGroupProcessAuthority::from_retained_projection(
            binding(FINAL_US),
            &request,
            GroupProcessRetainedProjectionV1::Current(source_row(store)),
        )
        .unwrap()
        .current_view()
        .unwrap()
    }
    impl GroupProcessStore for Store {
        type Credentials = ();
        type FormProof = Proof;
        type Scope<'a> = Scope<'a>;

        async fn resolve_current_incarnation(
            &self,
            _: &(),
            requested: GroupId,
        ) -> Result<GroupIncarnation, GroupProcessError> {
            self.record("resolve-current");
            assert!(self.ordinary, "validation substituted current incarnation");
            assert_eq!(requested, group());
            Ok(incarnation())
        }
        async fn resolve_original_locator(
            &self,
            _: &(),
            _: GroupProcessRouteSelectorV1,
        ) -> Result<GroupProcessLocator, GroupProcessError> {
            panic!("validation redisplay looked up historical command/receipt")
        }
        async fn lock<'a>(
            &'a self,
            _: &'a (),
            request: GroupProcessScopeRequest<'a>,
        ) -> Result<Scope<'a>, GroupProcessError> {
            let (original_command, original_process, ordinary) = match &request {
                GroupProcessScopeRequest::ValidationForm {
                    original: selected,
                    process_id,
                } => {
                    self.record("lock-validation");
                    assert!(!self.ordinary);
                    assert_eq!(*selected, original());
                    assert_eq!(*process_id, id(ORIGINAL_PROCESS));
                    assert_eq!(request.mode(), GroupProcessMaterialModeV1::Form);
                    assert_eq!(request.group(), group());
                    assert_eq!(request.incarnation(), incarnation());
                    assert_eq!(request.locator(), None);
                    (selected.command_id(), *process_id, false)
                }
                GroupProcessScopeRequest::Form {
                    group: selected_group,
                    incarnation: selected_incarnation,
                } => {
                    self.record("lock-form");
                    assert!(self.ordinary);
                    assert_eq!(*selected_group, group());
                    assert_eq!(*selected_incarnation, incarnation());
                    (id(81), if self.empty { id(82) } else { id(5) }, true)
                }
                _ => panic!("validation reached a different owner method"),
            };
            if let Some(error) = self.lock_error {
                return Err(error);
            }
            let authority = CurrentGroupProcessAuthority::from_retained_projection(
                binding(FINAL_US),
                &request,
                GroupProcessRetainedProjectionV1::Current(source_row(self)),
            )?;
            Ok(Scope {
                store: self,
                authority,
                original_command,
                original_process,
                ordinary,
            })
        }
        async fn enumerate_navigation_candidates(
            &self,
            _: &(),
        ) -> Result<GroupProcessNavigationCandidatesV1, GroupProcessError> {
            panic!("validation performed navigation enumeration")
        }
        async fn recheck_navigation_candidates(
            &self,
            _: &(),
            _: &GroupProcessNavigationSnapshotV1,
        ) -> Result<(), GroupProcessError> {
            panic!("validation performed navigation recheck")
        }
    }
    impl Drop for Scope<'_> {
        fn drop(&mut self) {
            self.store.record("release");
        }
    }
    impl GroupProcessScope for Scope<'_> {
        type FormProof = Proof;
        fn authority(&self) -> &CurrentGroupProcessAuthority {
            &self.authority
        }
        async fn form(&mut self) -> Result<GroupProcessForm<Proof>, GroupProcessError> {
            self.store.record(if self.ordinary {
                "mint-form"
            } else {
                "retain-form"
            });
            if let Some(error) = self.store.form_error {
                return Err(error);
            }
            let mut view = self.authority.current_view()?;
            let mut command_id = self.original_command;
            let mut process_id = self.original_process;
            match self.store.fault {
                ResponseFault::None => {}
                ResponseFault::Command => command_id = id(91),
                ResponseFault::Process => process_id = id(92),
                ResponseFault::NilCommand => command_id = Uuid::nil(),
                ResponseFault::NilProcess => process_id = Uuid::nil(),
                ResponseFault::View(corrupt) => corrupt(&mut view),
            }
            Ok(GroupProcessForm {
                view,
                command_id,
                process_id,
                proof: Proof(Arc::clone(&self.store.retained_proof)),
            })
        }
        async fn finish<P: GroupProcessDecisionPort + ?Sized>(
            self,
            _: &P,
            proof: Option<&Proof>,
        ) -> Result<(), GroupProcessError> {
            self.store.record("finish-proof");
            self.authority.check_finish_proof(proof.is_some())?;
            let proof = proof.ok_or(GroupProcessError::Unavailable)?;
            assert!(
                Arc::ptr_eq(&proof.0, &self.store.retained_proof),
                "finalizer received a substitute proof"
            );
            if self.store.pending_finish {
                std::future::pending::<()>().await;
            }
            if let Some(error) = self.store.finish_error {
                return Err(error);
            }
            self.store.record("finish-confirmed");
            Ok(())
        }
        async fn current(&mut self) -> Result<GroupProcessCurrentView, GroupProcessError> {
            panic!("validation called current instead of retained form")
        }
        async fn landing(&mut self) -> Result<GroupProcessLandingViewV1, GroupProcessError> {
            panic!("validation performed preliminary landing read")
        }
        async fn status(&mut self) -> Result<GroupProcessStatus, GroupProcessError> {
            panic!("validation looked up historical receipt")
        }
        async fn retry_form(
            &mut self,
        ) -> Result<GroupProcessOwnRetryFormV1<Proof>, GroupProcessError> {
            panic!("validation used retry-form ownership")
        }
        async fn accept(
            &mut self,
            _: &console_kernel_core::TraceContext,
        ) -> Result<GroupProcessAcceptance, GroupProcessError> {
            panic!("validation admitted a business command")
        }
        async fn execute(
            &mut self,
            _: &console_kernel_core::TraceContext,
        ) -> Result<GroupProcessExecution, GroupProcessError> {
            panic!("validation executed business work")
        }
        async fn retry_resolve(
            &mut self,
        ) -> Result<GroupProcessRetryResolution, GroupProcessError> {
            panic!("validation retried existing work")
        }
    }
    struct Policy {
        events: Mutex<Vec<&'static str>>,
        decision: Result<GroupProcessDecision, GroupProcessError>,
    }
    impl Policy {
        fn allow() -> Self {
            Self {
                events: Mutex::new(Vec::new()),
                decision: Ok(GroupProcessDecision::Allow),
            }
        }
    }
    impl GroupProcessDecisionPort for Policy {
        fn decide(
            &self,
            authority: &CurrentGroupProcessAuthority,
            request: &GroupProcessPolicyRequestV1,
        ) -> Result<GroupProcessDecision, GroupProcessError> {
            self.events.lock().unwrap().push("authorize-read-form");
            assert_eq!(authority.mode(), GroupProcessMaterialModeV1::Form);
            assert_eq!(
                authority.finish_purpose(),
                GroupProcessFinishPurposeV1::CurrentForm
            );
            assert_eq!(request.action(), GroupProcessActionV1::Read);
            assert_eq!(
                request.resource_kind(),
                GroupProcessResourceKindV1::ProcessForm
            );
            assert_eq!(request, authority.policy_request());
            self.decision
        }
    }
    fn validation(
        store: &Store,
        policy: &Policy,
    ) -> Result<GroupProcessForm<Proof>, GroupProcessError> {
        ready(group_process_validation_form(
            store,
            policy,
            &(),
            original(),
            id(ORIGINAL_PROCESS),
        ))
    }
    fn unavailable<T>(result: Result<T, GroupProcessError>) {
        assert!(matches!(result, Err(GroupProcessError::Unavailable)));
    }

    #[test]
    fn group_validation_form_empty_preserves_original_ids_full_view_and_proof_finish() {
        let store = Store::new(true);
        let policy = Policy::allow();
        let expected = expected_view(&store);
        let form = validation(&store, &policy).unwrap();
        assert_eq!(form.view, expected);
        assert_eq!(
            (form.command_id, form.process_id),
            (id(ORIGINAL_COMMAND), id(ORIGINAL_PROCESS))
        );
        assert!(Arc::ptr_eq(&form.proof.0, &store.retained_proof));
        assert_eq!(
            store.events(),
            [
                "lock-validation",
                "retain-form",
                "finish-proof",
                "finish-confirmed",
                "release"
            ]
        );
        assert_eq!(*policy.events.lock().unwrap(), ["authorize-read-form"]);
    }
    #[test]
    fn group_validation_form_current_winner_does_not_replace_original_draft_process() {
        let store = Store::new(false);
        let policy = Policy::allow();
        let expected = expected_view(&store);
        assert_eq!(
            expected.head.as_ref().unwrap().reference.process_id(),
            id(5)
        );
        let form = validation(&store, &policy).unwrap();
        assert_eq!(form.view, expected);
        assert_eq!(
            (form.command_id, form.process_id),
            (id(ORIGINAL_COMMAND), id(ORIGINAL_PROCESS))
        );
        assert_ne!(
            form.process_id,
            form.view.head.as_ref().unwrap().reference.process_id()
        );
        assert_eq!(
            store.events(),
            [
                "lock-validation",
                "retain-form",
                "finish-proof",
                "finish-confirmed",
                "release"
            ]
        );
    }
    #[test]
    fn group_validation_form_rejects_each_whole_view_substitution_before_finish() {
        let corruptions: [(&str, fn(&mut GroupProcessCurrentView)); 21] = [
            ("Group", |v| {
                v.context.group = GroupId::from_uuid(id(93)).unwrap()
            }),
            ("incarnation", |v| {
                v.context.incarnation = GroupIncarnation::from_uuid(id(94)).unwrap()
            }),
            ("label", |v| v.context.label.push_str(" substituted")),
            ("revision", |v| v.context.revision += 1),
            ("acting account", |v| v.context.account = actor(93)),
            ("observed time", |v| v.observed_at_us += 1),
            ("policy", |v| v.policy_head = PolicyHeadReferenceV1::Absent),
            ("head", |v| v.head = None),
            ("head causing receipt", |v| {
                v.head.as_mut().unwrap().causing_receipt = id(93)
            }),
            ("content", |v| v.content = None),
            ("content replacement", |v| {
                v.content = Some(content("다른 제출의 절차"))
            }),
            ("history omission", |v| v.history.clear()),
            ("history actor", |v| v.history[0].actor = actor(93)),
            ("history command", |v| v.history[0].command_id = id(93)),
            ("history time", |v| v.history[0].occurred_at_us += 1),
            ("history reason", |v| {
                v.history[0].reason = Some("변경된 이유".into())
            }),
            ("history receipt", |v| {
                v.history[0].head.causing_receipt = id(93)
            }),
            ("history duplicate", |v| {
                v.history.push(v.history[0].clone())
            }),
            ("actions omission", |v| v.allowed_actions.clear()),
            ("actions reordered", |v| v.allowed_actions.reverse()),
            ("actions duplicate", |v| {
                v.allowed_actions.push(GroupProcessActionV1::Read)
            }),
        ];
        for (label, corrupt) in corruptions {
            let mut store = Store::new(false);
            store.fault = ResponseFault::View(corrupt);
            let result = validation(&store, &Policy::allow());
            assert!(
                matches!(result, Err(GroupProcessError::Unavailable)),
                "accepted altered {label}"
            );
            assert_eq!(
                store.events(),
                ["lock-validation", "retain-form", "release"],
                "altered {label} reached finalization"
            );
        }
    }
    #[test]
    fn group_validation_form_rejects_substituted_or_nil_command_and_process() {
        for fault in [
            ResponseFault::Command,
            ResponseFault::Process,
            ResponseFault::NilCommand,
            ResponseFault::NilProcess,
        ] {
            let mut store = Store::new(false);
            store.fault = fault;
            unavailable(validation(&store, &Policy::allow()));
            assert_eq!(
                store.events(),
                ["lock-validation", "retain-form", "release"]
            );
        }
    }
    #[test]
    fn group_validation_form_nil_process_is_rejected_before_owner_work() {
        let store = Store::new(true);
        let policy = Policy::allow();
        unavailable(ready(group_process_validation_form(
            &store,
            &policy,
            &(),
            original(),
            Uuid::nil(),
        )));
        assert!(store.events().is_empty());
        assert!(policy.events.lock().unwrap().is_empty());
        assert_eq!(
            GroupProcessLocator::new(group(), incarnation(), Uuid::nil()),
            Err(GroupProcessError::InvalidInput)
        );
    }
    #[test]
    fn group_validation_form_lock_auth_source_and_authorization_failures_release_no_form() {
        for error in [
            GroupProcessError::AuthenticationInvalid,
            GroupProcessError::CsrfInvalid,
            GroupProcessError::NotFound,
            GroupProcessError::Unavailable,
        ] {
            let mut store = Store::new(true);
            store.lock_error = Some(error);
            assert!(matches!(validation(&store, &Policy::allow()), Err(actual) if actual == error));
            assert_eq!(store.events(), ["lock-validation"]);
        }
        for (decision, expected) in [
            (Ok(GroupProcessDecision::Deny), GroupProcessError::NotFound),
            (
                Err(GroupProcessError::Unavailable),
                GroupProcessError::Unavailable,
            ),
        ] {
            let store = Store::new(true);
            let policy = Policy {
                events: Mutex::new(Vec::new()),
                decision,
            };
            assert!(matches!(validation(&store, &policy), Err(actual) if actual == expected));
            assert_eq!(store.events(), ["lock-validation", "release"]);
            assert_eq!(*policy.events.lock().unwrap(), ["authorize-read-form"]);
        }
    }
    #[test]
    fn group_validation_form_retained_proof_and_finish_failure_release_no_provisional_form() {
        for error in [
            GroupProcessError::AuthenticationInvalid,
            GroupProcessError::CsrfInvalid,
            GroupProcessError::NotFound,
            GroupProcessError::Unavailable,
            GroupProcessError::Unconfirmed,
        ] {
            let mut store = Store::new(false);
            store.form_error = Some(error);
            assert!(matches!(validation(&store, &Policy::allow()), Err(actual) if actual == error));
            assert_eq!(
                store.events(),
                ["lock-validation", "retain-form", "release"]
            );
            store.events.lock().unwrap().clear();
            store.form_error = None;
            store.finish_error = Some(error);
            assert!(matches!(validation(&store, &Policy::allow()), Err(actual) if actual == error));
            assert_eq!(
                store.events(),
                ["lock-validation", "retain-form", "finish-proof", "release"]
            );
        }
    }
    #[test]
    fn group_validation_form_posted_incarnation_is_not_resolved_or_substituted() {
        for empty in [true, false] {
            let mut store = Store::new(empty);
            store.source_incarnation = GroupIncarnation::from_uuid(id(44)).unwrap();
            assert!(matches!(
                validation(&store, &Policy::allow()),
                Err(GroupProcessError::NotFound)
            ));
            assert_eq!(store.events(), ["lock-validation"]);
        }
    }
    #[test]
    fn group_validation_form_cancellation_during_finish_keeps_form_provisional_and_releases_scope()
    {
        let mut store = Store::new(false);
        store.pending_finish = true;
        let policy = Policy::allow();
        let waker = Waker::from(Arc::new(NoopWake));
        let mut cx = Context::from_waker(&waker);
        let mut future = Box::pin(group_process_validation_form(
            &store,
            &policy,
            &(),
            original(),
            id(ORIGINAL_PROCESS),
        ));
        assert!(matches!(future.as_mut().poll(&mut cx), Poll::Pending));
        assert_eq!(
            store.events(),
            ["lock-validation", "retain-form", "finish-proof"]
        );
        drop(future);
        assert_eq!(
            store.events(),
            ["lock-validation", "retain-form", "finish-proof", "release"]
        );
    }
    #[test]
    fn group_ordinary_form_still_resolves_mints_and_requires_current_head_process() {
        for empty in [true, false] {
            let mut store = Store::new(empty);
            store.ordinary = true;
            let form = ready(group_process_form(&store, &Policy::allow(), &(), group())).unwrap();
            assert_eq!(form.view, expected_view(&store));
            assert_eq!(form.command_id, id(81));
            assert_eq!(form.process_id, if empty { id(82) } else { id(5) });
            assert_eq!(
                store.events(),
                [
                    "resolve-current",
                    "lock-form",
                    "mint-form",
                    "finish-proof",
                    "finish-confirmed",
                    "release"
                ]
            );
        }
        let mut store = Store::new(false);
        store.ordinary = true;
        store.fault = ResponseFault::Process;
        unavailable(ready(group_process_form(
            &store,
            &Policy::allow(),
            &(),
            group(),
        )));
        assert_eq!(
            store.events(),
            ["resolve-current", "lock-form", "mint-form", "release"]
        );
    }

    // Separate exact Form target preflight tests; existing tests unchanged.
    include!("form_target_workflow_tests.rs");
}
