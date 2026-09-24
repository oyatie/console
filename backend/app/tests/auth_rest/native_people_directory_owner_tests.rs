// Actual owning adapter tests, included under native_policy_startup_tests.
// Explicit isolated fixtures; not browser qualification or runtime seed data.
// Requires actual guarded Directory activation and verified fresh AppState.
mod native_people_directory_owner {
    use super::*;
    use console_identity_application::company_policy::{
        AccountId,
        people_business::{DirectoryActionV1, NativePeoplePolicyCommandV1},
        workflow::{
            NativePolicyCommand, NativePolicyCommandRef, NativePolicyOutcome, native_policy_form,
            submit_native_policy_command,
        },
    };
    use console_ontology_application::people::{workflow::*, *};
    use console_ontology_canonical_adapter_postgres::native_directory::{
        DirectoryCedarDecision, NativeDirectoryAuthority, PgNativeDirectoryStore,
    };
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    #[derive(Default)]
    struct TestAdmission {
        calls: usize,
        error: Option<DirectoryWorkflowError>,
    }
    impl DirectoryProofAdmission for TestAdmission {
        async fn admit(&mut self) -> Result<(), DirectoryWorkflowError> {
            self.calls += 1;
            match self.error {
                Some(error) => Err(error),
                None => Ok(()),
            }
        }
    }
    struct DirectoryFixture {
        app: Fixture,
        state: AppState,
        runtime: PgPool,
        policy_store: PgOrgStore,
        store: PgNativeDirectoryStore,
        policy: Arc<CompanyPolicy>,
        decision: DirectoryCedarDecision,
        cookies: Cookies,
        created: Committed,
        verifier: JwtVerifier,
        ttl: time::Duration,
    }
    impl DirectoryFixture {
        async fn new(pool: &PgPool) -> Self {
            let (app, key, state) =
                native_people_directory_finalizer_tests::configured_native_directory_fixture(pool)
                    .await;
            let (app, cookies, created) = create_owned_company(pool, app).await;
            let config = account_browser_config(pool, app._artifacts.root.clone(), &key);
            let (verifier, issuer, ttl) = bindings(&config);
            let runtime = login_test_pool(pool, TestDatabaseLogin::Business).await;
            let role:(String,bool,bool)=sqlx::query_as("SELECT current_user::text,rolsuper,rolbypassrls FROM pg_roles WHERE rolname=current_user").fetch_one(&runtime).await.unwrap();
            assert_eq!(role, ("console_rt".into(), false, false));
            let policy_store = PgOrgStore::new(runtime.clone()).with_native_account_policy(
                verifier.clone(),
                issuer.clone(),
                ttl,
            );
            let store = PgNativeDirectoryStore::new(runtime.clone(), verifier.clone(), issuer, ttl)
                .unwrap();
            let policy = Arc::new(CompanyPolicy::new().unwrap());
            let decision = DirectoryCedarDecision::new(policy.clone());
            Self {
                app,
                state,
                runtime,
                policy_store,
                store,
                policy,
                decision,
                cookies,
                created,
                verifier,
                ttl,
            }
        }
        fn company(&self) -> OrgId {
            OrgId::from_uuid(self.created.company)
        }
        async fn policy_command(&self, input: NativePeoplePolicyCommandV1) {
            let input = NativePolicyCommand::People(input);
            let read = read_credentials(&self.cookies);
            let form = native_policy_form(
                &self.policy_store,
                self.policy.as_ref(),
                &read,
                NativePolicyCommandRef::from_command(&input),
            )
            .await
            .unwrap();
            let credentials = AccountEnrollmentCredentials::for_mutation(
                &self.cookies.0[ACCESS],
                form.proof.as_str(),
            )
            .unwrap();
            let terminal = submit_native_policy_command(
                &self.policy_store,
                self.policy.as_ref(),
                &credentials,
                &input,
                &TraceContext::generate(),
            )
            .await
            .unwrap()
            .terminal;
            assert!(matches!(
                terminal.outcome,
                NativePolicyOutcome::Committed(_)
            ));
        }
        async fn configure(&self, read: bool, create: bool) {
            self.policy_command(
                NativePeoplePolicyCommandV1::install(Uuid::new_v4(), self.company(), 1).unwrap(),
            )
            .await;
            let now: time::OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
                .fetch_one(&self.runtime)
                .await
                .unwrap();
            let until =
                time::OffsetDateTime::from_unix_timestamp((now.unix_timestamp() / 60 + 1440) * 60)
                    .unwrap();
            let mut epoch = 2;
            for (action, enabled) in [
                (DirectoryActionV1::Read, read),
                (DirectoryActionV1::Create, create),
            ] {
                if enabled {
                    self.policy_command(
                        NativePeoplePolicyCommandV1::grant(
                            Uuid::new_v4(),
                            self.company(),
                            epoch,
                            action,
                            AccountId::from_uuid(self.created.administrator).unwrap(),
                            None,
                            until,
                        )
                        .unwrap(),
                    )
                    .await;
                    epoch += 1;
                }
            }
        }
        async fn input(&self, number: &str) -> (DirectorySubmission, AccountEnrollmentCredentials) {
            let locator = DirectoryRequestRef::new(self.company(), Uuid::new_v4()).unwrap();
            let form = directory_form(
                &self.store,
                &self.decision,
                &read_credentials(&self.cookies),
                locator,
                None,
            )
            .await
            .unwrap();
            let credentials = AccountEnrollmentCredentials::for_mutation(
                &self.cookies.0[ACCESS],
                form.proof.as_str(),
            )
            .unwrap();
            (
                DirectorySubmission::new(
                    locator,
                    form.expected,
                    DirectoryRegistrationInput::new("김하늘", number).unwrap(),
                )
                .unwrap(),
                credentials,
            )
        }
        async fn close(self, outcome: Result<(), Box<dyn std::any::Any + Send>>) {
            self.runtime.close().await;
            close_states(&[self.state], outcome).await;
        }
    }
    fn exact_delta(
        before: &BTreeMap<String, String>,
        after: &BTreeMap<String, String>,
        expected: &[(&str, usize)],
    ) {
        assert!(before.keys().eq(after.keys()));
        let expected: BTreeMap<_, _> = expected.iter().copied().collect();
        for key in expected.keys() {
            assert!(before.contains_key(*key));
        }
        for (table, old) in before {
            let added = added_rows(old, &after[table])
                .unwrap_or_else(|| panic!("history changed in {table}"));
            assert_eq!(
                added.len(),
                expected.get(table.as_str()).copied().unwrap_or(0),
                "unexpected effects in {table}"
            );
        }
    }
    #[sqlx::test(migrations = false)]
    async fn native_directory_owner_prepares_commits_reopens_and_replays_exact_effects(
        pool: PgPool,
    ) {
        let f = DirectoryFixture::new(&pool).await;
        let outcome=AssertUnwindSafe(async {
            f.configure(true,true).await;let (input,credentials)=f.input("K-1").await;
            let before=all_rows(&pool).await;directory_preflight(&f.store,&f.decision,&credentials,&input).await.unwrap();assert_eq!(all_rows(&pool).await,before);
            let accepted=directory_prepare(&f.store,&f.decision,&credentials,&input,&TraceContext::generate()).await.unwrap();assert!(accepted.inserted);let DirectoryStatus::Pending(accepted)=accepted.status else{panic!("pending intake required")};
            let prepared=all_rows(&pool).await;exact_delta(&before,&prepared,&[("native_people_inputs_v1",1),("audit_events",1)]);
            let status=directory_status(&f.store,&f.decision,&read_credentials(&f.cookies),input.locator(),&TraceContext::generate(),&mut TestAdmission::default()).await.unwrap();assert_eq!(status.status,DirectoryStatus::Pending(accepted.clone()));let proof=status.proof.unwrap();assert_eq!(all_rows(&pool).await,prepared);
            let execute_credentials=AccountEnrollmentCredentials::for_mutation(&f.cookies.0[ACCESS],proof.as_str()).unwrap();
            let result=directory_execute(&f.store,&f.decision,&execute_credentials,input.locator(),&TraceContext::generate()).await.unwrap();assert!(result.inserted);assert_eq!(result.terminal.outcome(),DirectoryTerminalOutcomeV1::Committed);
            let committed=all_rows(&pool).await;exact_delta(&prepared,&committed,&[("employees",1),("persons",1),("person_revisions",1),("employee_person_bindings",1),("ont_action_command_receipts",1),("native_people_terminals_v1",1),("audit_events",1)]);
            let again=directory_execute(&f.store,&f.decision,&execute_credentials,input.locator(),&TraceContext::generate()).await.unwrap();assert!(!again.inserted);assert_eq!(again.terminal,result.terminal);assert_eq!(all_rows(&pool).await,committed);
            let reopened=directory_status(&f.store,&f.decision,&read_credentials(&f.cookies),input.locator(),&TraceContext::generate(),&mut TestAdmission::default()).await.unwrap();assert!(reopened.proof.is_none());assert_eq!(reopened.status,DirectoryStatus::Terminal(result.terminal.clone()));
            let person=directory_detail(&f.store,&f.decision,&read_credentials(&f.cookies),f.company(),accepted.command().employee_id()).await.unwrap().unwrap();assert_eq!(person.person_id,accepted.command().employee_id());assert_eq!(person.legal_name.as_deref(),Some("김하늘"));assert_eq!(person.employee_number.as_deref(),Some("K-1"));
            let after=all_rows(&pool).await;exact_delta(&committed,&after,&[("audit_events",1)]);
            let row:(String,bool,bool,bool)=sqlx::query_as("SELECT employment_status,source_kind='NATIVE_DIRECTORY' AND source_filename IS NULL AND source_sheet IS NULL AND source_row IS NULL,identity_review_required AND identity_resolution_confidence='low',NOT EXISTS(SELECT 1 FROM employee_employment_profiles p WHERE p.org_id=e.org_id AND p.employee_id=e.id) FROM employees e WHERE org_id=$1 AND id=$2").bind(f.created.company).bind(accepted.command().employee_id()).fetch_one(&pool).await.unwrap();assert_eq!(row,("UNKNOWN".into(),true,true,true));
        }).catch_unwind().await;
        f.close(outcome).await;
    }
    struct FinalDeny {
        real: DirectoryCedarDecision,
        calls: AtomicUsize,
    }
    impl DirectoryDecisionPort<NativeDirectoryAuthority> for FinalDeny {
        fn permits(
            &self,
            a: &NativeDirectoryAuthority,
            r: DirectoryAccess,
        ) -> Result<bool, DirectoryWorkflowError> {
            let allowed = self.real.permits(a, r)?;
            assert!(allowed, "positive control must reach actual Cedar");
            Ok(self.calls.fetch_add(1, Ordering::SeqCst) == 0)
        }
    }
    #[sqlx::test(migrations = false)]
    async fn native_directory_owner_rejects_missing_grant_csrf_and_final_denial_without_effects(
        pool: PgPool,
    ) {
        let f = DirectoryFixture::new(&pool).await;
        let outcome = AssertUnwindSafe(async {
            let before = all_rows(&pool).await;
            assert!(matches!(
                directory_list(
                    &f.store,
                    &f.decision,
                    &read_credentials(&f.cookies),
                    f.company(),
                    DirectoryPageQuery::new(None, None).unwrap()
                )
                .await,
                Err(DirectoryWorkflowError::NotFound)
            ));
            assert_eq!(all_rows(&pool).await, before);
            f.configure(false, true).await;
            let (input, credentials) = f.input("K-2").await;
            let before = all_rows(&pool).await;
            assert!(matches!(
                directory_list(
                    &f.store,
                    &f.decision,
                    &read_credentials(&f.cookies),
                    f.company(),
                    DirectoryPageQuery::new(None, None).unwrap()
                )
                .await,
                Err(DirectoryWorkflowError::NotFound)
            ));
            assert_eq!(all_rows(&pool).await, before);
            assert!(matches!(
                directory_prepare(
                    &f.store,
                    &f.decision,
                    &read_credentials(&f.cookies),
                    &input,
                    &TraceContext::generate()
                )
                .await,
                Err(DirectoryWorkflowError::CsrfInvalid)
            ));
            assert_eq!(all_rows(&pool).await, before);
            let denied = FinalDeny {
                real: DirectoryCedarDecision::new(f.policy.clone()),
                calls: AtomicUsize::new(0),
            };
            assert!(matches!(
                directory_prepare(
                    &f.store,
                    &denied,
                    &credentials,
                    &input,
                    &TraceContext::generate()
                )
                .await,
                Err(DirectoryWorkflowError::NotFound)
            ));
            assert_eq!(denied.calls.load(Ordering::SeqCst), 2);
            assert_eq!(all_rows(&pool).await, before);
            assert!(
                directory_prepare(
                    &f.store,
                    &f.decision,
                    &credentials,
                    &input,
                    &TraceContext::generate()
                )
                .await
                .unwrap()
                .inserted
            );
        })
        .catch_unwind()
        .await;
        f.close(outcome).await;
    }
    #[sqlx::test(migrations = false)]
    async fn native_directory_terminal_closure_refuses_omitted_canonical_effects_and_audit(
        pool: PgPool,
    ) {
        let f = DirectoryFixture::new(&pool).await;
        let outcome=AssertUnwindSafe(async {
            f.configure(true,true).await;let (input,credentials)=f.input("K-3").await;
            directory_prepare(&f.store,&f.decision,&credentials,&input,&TraceContext::generate()).await.unwrap();let before=all_rows(&pool).await;
            let mut tx=f.runtime.begin().await.unwrap();let (actor,family)=credentials.session_ids_in_tx(&mut tx,&f.verifier,f.ttl).await.unwrap();
            let admitted:i64=sqlx::query_scalar("SELECT count(*) FROM identity_company_people_projection_v1($1,$2,$3,'people.directory.create')").bind(actor).bind(family).bind(f.created.company).fetch_one(tx.as_mut()).await.unwrap();assert_eq!(admitted,1);credentials.validate_mutation_in_tx(&mut tx,&f.verifier,f.ttl).await.unwrap();
            let opened:(bool,String)=sqlx::query_as("SELECT n.inserted,(n.terminal).outcome FROM native_people_terminal_open_v1($1,$2,$3,$4,'EXECUTE') n").bind(actor).bind(family).bind(f.created.company).bind(input.locator().command_id()).fetch_one(tx.as_mut()).await.unwrap();assert_eq!(opened,(true,"COMMITTED".into()));
            // Named owner constraint isolates the omitted-frame failure from FK
            // execution order. Never count missing schema/permission as semantic.
            let error=sqlx::query("SET CONSTRAINTS native_people_terminal_closure_v1 IMMEDIATE").execute(tx.as_mut()).await.unwrap_err();let db=error.as_database_error().unwrap();assert_eq!(db.code().as_deref(),Some("P0001"));assert_eq!(db.message(),"people.directory.audit_closure_invalid");tx.rollback().await.unwrap();assert_eq!(all_rows(&pool).await,before);
            let result=directory_execute(&f.store,&f.decision,&credentials,input.locator(),&TraceContext::generate()).await.unwrap();assert_eq!(result.terminal.outcome(),DirectoryTerminalOutcomeV1::Committed);
        }).catch_unwind().await;
        f.close(outcome).await;
    }

    #[sqlx::test(migrations = false)]
    async fn native_directory_status_admits_only_pending_before_proof_and_preserves_effects(
        pool: PgPool,
    ) {
        let f = DirectoryFixture::new(&pool).await;
        let outcome = AssertUnwindSafe(async {
            let unknown = DirectoryRequestRef::new(f.company(), Uuid::new_v4()).unwrap();
            let before = all_rows(&pool).await;
            let mut admission = TestAdmission::default();
            assert!(matches!(directory_status(&f.store,&f.decision,&read_credentials(&f.cookies),unknown,&TraceContext::generate(),&mut admission).await,Err(DirectoryWorkflowError::NotFound)));
            assert_eq!(admission.calls,0);
            assert_eq!(all_rows(&pool).await,before);
            f.configure(true,true).await;
            let (input,credentials)=f.input("K-ADMISSION").await;
            let before = all_rows(&pool).await;
            let missing=directory_status(&f.store,&f.decision,&read_credentials(&f.cookies),unknown,&TraceContext::generate(),&mut admission).await.unwrap();
            assert_eq!(missing.status,DirectoryStatus::NotVisible);
            assert!(missing.proof.is_none());
            assert_eq!(admission.calls,0);
            assert_eq!(all_rows(&pool).await,before);
            directory_prepare(&f.store,&f.decision,&credentials,&input,&TraceContext::generate()).await.unwrap();
            let prepared=all_rows(&pool).await;
            for error in [DirectoryWorkflowError::Capacity,DirectoryWorkflowError::Unavailable] {
                let mut denied=TestAdmission{calls:0,error:Some(error)};
                assert!(matches!(directory_status(&f.store,&f.decision,&read_credentials(&f.cookies),input.locator(),&TraceContext::generate(),&mut denied).await,Err(actual) if actual==error));
                assert_eq!(denied.calls,1);
                assert_eq!(all_rows(&pool).await,prepared);
            }
            let pending=directory_status(&f.store,&f.decision,&read_credentials(&f.cookies),input.locator(),&TraceContext::generate(),&mut admission).await.unwrap();
            assert!(matches!(pending.status,DirectoryStatus::Pending(_)));
            assert_eq!(admission.calls,1);
            assert_eq!(all_rows(&pool).await,prepared);
            let proof=pending.proof.unwrap();
            let credentials=AccountEnrollmentCredentials::for_mutation(&f.cookies.0[ACCESS],proof.as_str()).unwrap();
            let cancelled=directory_cancel(&f.store,&f.decision,&credentials,input.locator(),&TraceContext::generate()).await.unwrap();
            let before=all_rows(&pool).await;
            let mut must_not_admit=TestAdmission{calls:0,error:Some(DirectoryWorkflowError::Capacity)};
            let terminal=directory_status(&f.store,&f.decision,&read_credentials(&f.cookies),input.locator(),&TraceContext::generate(),&mut must_not_admit).await.unwrap();
            assert_eq!(terminal.status,DirectoryStatus::Terminal(cancelled.terminal));
            assert!(terminal.proof.is_none());
            assert_eq!(must_not_admit.calls,0);
            assert_eq!(all_rows(&pool).await,before);
        }).catch_unwind().await;
        f.close(outcome).await;
    }
}
