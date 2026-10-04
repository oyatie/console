// Additional private child of validation_form_workflow's reviewed scripted
// harness. Existing V4 tests/fixtures remain unchanged. These tests prove
// application preflight ordering, not real Auth, limiter accounting or SQL.
mod form_target_preflight {
    use super::*;

    struct TargetStore {
        inner: Store,
        suspended: bool,
    }
    impl TargetStore {
        fn new(suspended: bool) -> Self {
            let mut inner = Store::new(false);
            inner.ordinary = true;
            Self { inner, suspended }
        }
    }
    impl GroupProcessStore for TargetStore {
        type Credentials = ();
        type FormProof = Proof;
        type Scope<'a> = Scope<'a>;
        async fn resolve_current_incarnation(
            &self,
            credentials: &(),
            group: GroupId,
        ) -> Result<GroupIncarnation, GroupProcessError> {
            self.inner
                .resolve_current_incarnation(credentials, group)
                .await
        }
        async fn resolve_original_locator(
            &self,
            credentials: &(),
            requested: GroupProcessRouteSelectorV1,
        ) -> Result<GroupProcessLocator, GroupProcessError> {
            self.inner
                .resolve_original_locator(credentials, requested)
                .await
        }
        async fn lock<'a>(
            &'a self,
            credentials: &'a (),
            request: GroupProcessScopeRequest<'a>,
        ) -> Result<Scope<'a>, GroupProcessError> {
            assert_eq!(request.mode(), GroupProcessMaterialModeV1::Form);
            assert_eq!(request.group(), group());
            assert_eq!(request.incarnation(), incarnation());
            assert_eq!(request.locator(), None);
            let mut scope = self.inner.lock(credentials, request).await?;
            if self.suspended {
                // The actual protected Form source is the independently
                // declared suspended postimage, not a stale Landing projection.
                let mut row = fixture("suspend_valid", FINAL_US).projected;
                row.mode = GroupProcessMaterialModeV1::Form;
                row.original_status = None;
                let retained_request = GroupProcessScopeRequest::Form {
                    group: group(),
                    incarnation: incarnation(),
                };
                scope.authority = CurrentGroupProcessAuthority::from_retained_projection(
                    binding(FINAL_US),
                    &retained_request,
                    GroupProcessRetainedProjectionV1::Current(row),
                )?;
                assert_eq!(
                    scope
                        .authority
                        .current_view()?
                        .head
                        .unwrap()
                        .reference
                        .state(),
                    ProcessStateV1::Suspended,
                );
            }
            Ok(scope)
        }
        async fn enumerate_navigation_candidates(
            &self,
            credentials: &(),
        ) -> Result<GroupProcessNavigationCandidatesV1, GroupProcessError> {
            self.inner
                .enumerate_navigation_candidates(credentials)
                .await
        }
        async fn recheck_navigation_candidates(
            &self,
            credentials: &(),
            original: &GroupProcessNavigationSnapshotV1,
        ) -> Result<(), GroupProcessError> {
            self.inner
                .recheck_navigation_candidates(credentials, original)
                .await
        }
    }
    fn denied_before_proof(events: Vec<&'static str>) {
        assert_eq!(events, ["resolve-current", "lock-form", "release"]);
    }
    fn successful_form(events: Vec<&'static str>) {
        assert_eq!(
            events,
            [
                "resolve-current",
                "lock-form",
                "mint-form",
                "finish-proof",
                "finish-confirmed",
                "release",
            ]
        );
    }

    fn authorized_once(policy: &Policy) {
        assert_eq!(*policy.events.lock().unwrap(), ["authorize-read-form"]);
    }
    fn deny_policy() -> Policy {
        Policy {
            events: Mutex::new(Vec::new()),
            decision: Ok(GroupProcessDecision::Deny),
        }
    }

    #[test]
    fn group_form_target_adopt_concurrent_winner_is_refused_before_form_proof() {
        let policy = Policy::allow();
        let mut empty = Store::new(true);
        empty.ordinary = true;
        let form = ready(group_process_form_for(
            &empty,
            &policy,
            &(),
            group(),
            GroupProcessFormTargetV1::Adopt,
        ))
        .unwrap();
        assert!(form.view.head.is_none());
        successful_form(empty.events());
        authorized_once(&policy);

        let winner = TargetStore::new(false);
        let winner_policy = Policy::allow();
        let result = ready(group_process_form_for(
            &winner,
            &winner_policy,
            &(),
            group(),
            GroupProcessFormTargetV1::Adopt,
        ));
        assert!(matches!(result, Err(GroupProcessError::NotFound)));
        denied_before_proof(winner.inner.events());
        authorized_once(&winner_policy);

        let mut denied = Store::new(true);
        denied.ordinary = true;
        let policy = deny_policy();
        let result = ready(group_process_form_for(
            &denied,
            &policy,
            &(),
            group(),
            GroupProcessFormTargetV1::Adopt,
        ));
        assert!(matches!(result, Err(GroupProcessError::NotFound)));
        denied_before_proof(denied.events());
        authorized_once(&policy);
    }
    #[test]
    fn group_form_target_replace_wrong_actual_process_is_refused_before_form_proof() {
        let exact = TargetStore::new(false);
        let exact_policy = Policy::allow();
        let form = ready(group_process_form_for(
            &exact,
            &exact_policy,
            &(),
            group(),
            GroupProcessFormTargetV1::Replace { process_id: id(5) },
        ))
        .unwrap();
        assert_eq!(form.process_id, id(5));
        successful_form(exact.inner.events());
        authorized_once(&exact_policy);

        let changed = TargetStore::new(false);
        let changed_policy = Policy::allow();
        let result = ready(group_process_form_for(
            &changed,
            &changed_policy,
            &(),
            group(),
            GroupProcessFormTargetV1::Replace { process_id: id(93) },
        ));
        assert!(matches!(result, Err(GroupProcessError::NotFound)));
        denied_before_proof(changed.inner.events());
        authorized_once(&changed_policy);

        let denied = TargetStore::new(false);
        let policy = deny_policy();
        let result = ready(group_process_form_for(
            &denied,
            &policy,
            &(),
            group(),
            GroupProcessFormTargetV1::Replace { process_id: id(5) },
        ));
        assert!(matches!(result, Err(GroupProcessError::NotFound)));
        denied_before_proof(denied.inner.events());
        authorized_once(&policy);
    }
    #[test]
    fn group_form_target_suspend_already_suspended_source_is_refused_before_form_proof() {
        let active = TargetStore::new(false);
        let active_policy = Policy::allow();
        let form = ready(group_process_form_for(
            &active,
            &active_policy,
            &(),
            group(),
            GroupProcessFormTargetV1::Suspend { process_id: id(5) },
        ))
        .unwrap();
        assert_eq!(form.process_id, id(5));
        assert!(
            form.view
                .allowed_actions
                .contains(&GroupProcessActionV1::Suspend)
        );
        successful_form(active.inner.events());
        authorized_once(&active_policy);

        let suspended = TargetStore::new(true);
        let suspended_policy = Policy::allow();
        let result = ready(group_process_form_for(
            &suspended,
            &suspended_policy,
            &(),
            group(),
            GroupProcessFormTargetV1::Suspend { process_id: id(5) },
        ));
        assert!(matches!(result, Err(GroupProcessError::NotFound)));
        denied_before_proof(suspended.inner.events());
        authorized_once(&suspended_policy);

        let denied = TargetStore::new(false);
        let policy = deny_policy();
        let result = ready(group_process_form_for(
            &denied,
            &policy,
            &(),
            group(),
            GroupProcessFormTargetV1::Suspend { process_id: id(5) },
        ));
        assert!(matches!(result, Err(GroupProcessError::NotFound)));
        denied_before_proof(denied.inner.events());
        authorized_once(&policy);
    }
}
