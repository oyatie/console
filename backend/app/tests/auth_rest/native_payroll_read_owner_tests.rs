// Private proposed owner tests; not browser/populated Payroll acceptance.
// Include in native_policy_startup_tests after independent review.
mod native_payroll_read_owner_tests {
    use super::*;
    use console_identity_application::company_policy::{
        AccountId,
        business::PolicyAssignmentExpectationV1,
        workflow::{NativePolicyEffect, NativePolicyTerminalView},
    };
    use console_payroll_adapter_postgres::PgNativePayrollRunsReadPort;
    use console_payroll_application::read::{
        ListPayrollRuns, PayrollRunsReadError, PayrollRunsReadPort, PayrollRunsReadResult,
        list_payroll_runs,
    };
    use std::sync::Arc;

    struct FixtureRead {
        app: Fixture,
        state: AppState,
        runtime: PgPool,
        store: PgOrgStore,
        verifier: JwtVerifier,
        ttl: time::Duration,
        cookies: Cookies,
        created: Committed,
        policy: Arc<CompanyPolicy>,
    }
    impl FixtureRead {
        async fn new(pool: &PgPool) -> Self {
            let (app, key, state) = configured_fixture(pool, true).await;
            let (app, cookies, created) = create_owned_company(pool, app).await;
            let config = account_browser_config(pool, app._artifacts.root.clone(), &key);
            let (verifier, issuer, ttl) = bindings(&config);
            let runtime = login_test_pool(pool, TestDatabaseLogin::Business).await;
            let login: (String, String, bool, bool) = sqlx::query_as(
                "SELECT session_user::text,current_user::text,rolsuper,rolbypassrls FROM pg_roles WHERE rolname=current_user"
            ).fetch_one(&runtime).await.unwrap();
            assert_eq!(
                login,
                ("console_rt".into(), "console_rt".into(), false, false)
            );
            let store = PgOrgStore::new(runtime.clone()).with_native_account_policy(
                verifier.clone(),
                issuer,
                ttl,
            );
            Self {
                app,
                state,
                runtime,
                store,
                verifier,
                ttl,
                cookies,
                created,
                policy: Arc::new(CompanyPolicy::new().unwrap()),
            }
        }
        fn company(&self) -> OrgId {
            OrgId::from_uuid(self.created.company)
        }
        fn reader(&self) -> PgNativePayrollRunsReadPort {
            PgNativePayrollRunsReadPort::new(
                self.runtime.clone(),
                self.verifier.clone(),
                self.ttl,
                self.policy.clone(),
                read_credentials(&self.cookies),
                self.company(),
            )
        }
        async fn read(&self) -> Result<PayrollRunsReadResult, PayrollRunsReadError> {
            list_payroll_runs(
                &mut self.reader(),
                ListPayrollRuns {
                    limit: None,
                    offset: None,
                },
            )
            .await
        }
        async fn command(
            &self,
            command: NativeCompanyBusinessCommandV1,
        ) -> NativePolicyTerminalView {
            let form = native_policy_form(
                &self.store,
                self.policy.as_ref(),
                &read_credentials(&self.cookies),
                NativePolicyCommandRef::from_command(&command),
            )
            .await
            .unwrap();
            let credentials = AccountEnrollmentCredentials::for_mutation(
                &self.cookies.0[ACCESS],
                form.proof.as_str(),
            )
            .unwrap();
            submit_native_policy_command(
                &self.store,
                self.policy.as_ref(),
                &credentials,
                &command,
                &TraceContext::generate(),
            )
            .await
            .unwrap()
            .terminal
        }
        async fn install(&self) {
            let terminal = self
                .command(
                    NativeCompanyBusinessCommandV1::install(Uuid::new_v4(), self.company(), 1)
                        .unwrap(),
                )
                .await;
            assert!(matches!(
                terminal.outcome,
                NativePolicyOutcome::Committed(NativePolicyEffect::Installed { .. })
            ));
        }
        async fn grant(&self, pool: &PgPool) -> PolicyAssignmentExpectationV1 {
            let now: time::OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
                .fetch_one(pool)
                .await
                .unwrap();
            let until =
                time::OffsetDateTime::from_unix_timestamp((now.unix_timestamp() / 60 + 1440) * 60)
                    .unwrap();
            self.grant_until(until).await
        }
        async fn grant_until(&self, until: time::OffsetDateTime) -> PolicyAssignmentExpectationV1 {
            let recipient = AccountId::from_uuid(self.created.administrator).unwrap();
            match self
                .command(
                    NativeCompanyBusinessCommandV1::grant(
                        Uuid::new_v4(),
                        self.company(),
                        2,
                        recipient,
                        None,
                        until,
                    )
                    .unwrap(),
                )
                .await
                .outcome
            {
                NativePolicyOutcome::Committed(NativePolicyEffect::Granted {
                    assignment,
                    recipient: actual,
                    ..
                }) => {
                    assert_eq!(actual, recipient);
                    assignment.expectation
                }
                _ => panic!("actual native grant did not commit"),
            }
        }
        async fn close(self, result: Result<(), Box<dyn std::any::Any + Send>>) {
            self.runtime.close().await;
            close_states(&[self.state], result).await;
        }
    }
    async fn successful_empty_read(pool: &PgPool, f: &FixtureRead) {
        let before = all_rows(pool).await;
        let started: time::OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
            .fetch_one(pool)
            .await
            .unwrap();
        let result = f.read().await.unwrap();
        let finished: time::OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
            .fetch_one(pool)
            .await
            .unwrap();
        assert!(result.page.items.is_empty());
        assert_eq!(
            (result.page.total, result.page.limit, result.page.offset),
            (0, 100, 0)
        );
        let context = result.company.unwrap();
        assert_eq!(context.id, f.company());
        let identity = context.identity.unwrap();
        let row: (String, String) =
            sqlx::query_as("SELECT name,slug FROM public.organizations WHERE id=$1")
                .bind(f.created.company)
                .fetch_one(pool)
                .await
                .unwrap();
        assert_eq!((identity.name, identity.slug), row);
        let after = all_rows(pool).await;
        assert!(before.keys().eq(after.keys()));
        for (table, value) in &before {
            if table != "audit_events" {
                assert!(value == &after[table], "read changed {table}");
            }
        }
        let audits = added_rows(&before["audit_events"], &after["audit_events"]).unwrap();
        assert_eq!(audits.len(), 1);
        let audit = &audits[0];
        assert_eq!(audit["actor"], json!(f.created.administrator));
        assert_eq!(audit["org_id"], json!(f.created.company));
        assert_eq!(audit["action"], "payroll_run.list_read");
        assert_eq!(audit["target_type"], "payroll_draft_run");
        assert_eq!(audit["target_id"], "query");
        let nulls: bool = sqlx::query_scalar("SELECT before_snap IS NULL AND after_snap IS NULL AND branch_id IS NULL FROM public.audit_events WHERE id=$1")
            .bind(Uuid::parse_str(audit["id"].as_str().unwrap()).unwrap()).fetch_one(pool).await.unwrap();
        assert!(nulls);
        TraceContext::new(
            audit["trace_id"].as_str().unwrap(),
            audit["span_id"].as_str().unwrap(),
        )
        .unwrap();
        assert_ne!(audit["trace_id"], "0".repeat(32));
        assert_ne!(audit["span_id"], "0".repeat(16));
        let timed: bool = sqlx::query_scalar(
            "SELECT occurred_at BETWEEN $1 AND $2 FROM public.audit_events WHERE id=$3",
        )
        .bind(started)
        .bind(finished)
        .bind(Uuid::parse_str(audit["id"].as_str().unwrap()).unwrap())
        .fetch_one(pool)
        .await
        .unwrap();
        assert!(timed);
    }
    async fn denied_without_writes(pool: &PgPool, f: &FixtureRead) {
        let before = all_rows(pool).await;
        match f.read().await {
            Err(PayrollRunsReadError::Authorization(error)) => {
                assert_eq!(error.kind, console_kernel_core::ErrorKind::NotFound)
            }
            _ => panic!("healthy missing/revoked Payroll scope must deny without a page"),
        }
        assert!(before == all_rows(pool).await);
    }
    #[sqlx::test(migrations = false)]
    async fn genuine_grant_read_revoke_and_revoked_session_preserve_read_ownership(pool: PgPool) {
        let f = FixtureRead::new(&pool).await;
        let result = AssertUnwindSafe(async {
            denied_without_writes(&pool, &f).await;
            f.install().await;
            denied_without_writes(&pool, &f).await;
            let assignment = f.grant(&pool).await;
            successful_empty_read(&pool, &f).await;
            successful_empty_read(&pool, &f).await;
            let revoked = f
                .command(
                    NativeCompanyBusinessCommandV1::revoke(
                        Uuid::new_v4(),
                        f.company(),
                        3,
                        assignment,
                    )
                    .unwrap(),
                )
                .await;
            assert!(matches!(
                revoked.outcome,
                NativePolicyOutcome::Committed(NativePolicyEffect::Revoked { .. })
            ));
            denied_without_writes(&pool, &f).await;
            let csrf = proof(&f.app, &f.cookies).await;
            let response = request(
                &f.app,
                "POST",
                "/api/v2/auth/logout",
                &f.cookies,
                Some(json!({})),
                &[("X-Console-CSRF", &csrf)],
            )
            .await;
            assert_eq!(
                response.json(StatusCode::OK),
                json!({"outcome":"COMMITTED"})
            );
            let before = all_rows(&pool).await;
            assert!(matches!(
                f.read().await,
                Err(PayrollRunsReadError::AuthenticationInvalid)
            ));
            assert!(before == all_rows(&pool).await);
        })
        .catch_unwind()
        .await;
        f.close(result).await;
    }
    #[sqlx::test(migrations = false)]
    async fn native_read_audit_and_deferred_finalization_failure_withhold_result_and_recover(
        pool: PgPool,
    ) {
        let f = FixtureRead::new(&pool).await;
        let result = AssertUnwindSafe(async {
            f.install().await; f.grant(&pool).await;
            successful_empty_read(&pool, &f).await;
            sqlx::raw_sql("CREATE SEQUENCE public.test_native_payroll_audit_witness; GRANT USAGE,SELECT ON SEQUENCE public.test_native_payroll_audit_witness TO console_rt; CREATE FUNCTION public.test_native_payroll_read_refusal() RETURNS trigger LANGUAGE plpgsql SET search_path=pg_catalog,pg_temp AS $body$ BEGIN IF NEW.action='payroll_run.list_read' THEN PERFORM nextval('public.test_native_payroll_audit_witness'); RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='private-read-fault'; END IF; RETURN NEW; END $body$")
                .execute(&pool).await.unwrap();
            for deferred in [false, true] {
                let sql = if deferred {
                    "CREATE CONSTRAINT TRIGGER test_native_payroll_read_refusal AFTER INSERT ON public.audit_events DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.test_native_payroll_read_refusal()"
                } else {
                    "CREATE TRIGGER test_native_payroll_read_refusal BEFORE INSERT ON public.audit_events FOR EACH ROW EXECUTE FUNCTION public.test_native_payroll_read_refusal()"
                };
                sqlx::raw_sql(sql).execute(&pool).await.unwrap();
                let before = all_rows(&pool).await;
                let fired_before: i64 = sqlx::query_scalar("SELECT CASE WHEN is_called THEN last_value ELSE 0 END FROM public.test_native_payroll_audit_witness").fetch_one(&pool).await.unwrap();
                let refused = f.read().await;
                let fired_after: i64 = sqlx::query_scalar("SELECT last_value FROM public.test_native_payroll_audit_witness").fetch_one(&pool).await.unwrap();
                assert_eq!(fired_after, fired_before + 1, "actual immediate/deferred audit trigger must fire");
                assert!(matches!(refused, Err(PayrollRunsReadError::Unavailable)), "audit failure released page or lost error classification");
                assert!(before == all_rows(&pool).await, "failed read left durable effects");
                sqlx::raw_sql("DROP TRIGGER test_native_payroll_read_refusal ON public.audit_events").execute(&pool).await.unwrap();
                successful_empty_read(&pool, &f).await;
            }
        }).catch_unwind().await;
        // Restore all injected DDL even after an assertion or query panic.
        let cleanup = sqlx::raw_sql("DROP TRIGGER IF EXISTS test_native_payroll_read_refusal ON public.audit_events; DROP FUNCTION IF EXISTS public.test_native_payroll_read_refusal(); DROP SEQUENCE IF EXISTS public.test_native_payroll_audit_witness;")
            .execute(&pool).await;
        f.close(result).await;
        cleanup.expect("restore actual audit fault objects");
    }

    #[sqlx::test(migrations = false)]
    async fn native_read_cannot_borrow_another_account_or_company_grant(pool: PgPool) {
        let f = FixtureRead::new(&pool).await;
        let result = AssertUnwindSafe(async {
            f.install().await;
            f.grant(&pool).await;
            successful_empty_read(&pool, &f).await;
            let (_, other_cookies) = enrolled(&f.app).await;
            let command = Uuid::new_v4();
            let input = enrollment(command, f.created.administrator);
            let csrf = proof(&f.app, &f.cookies).await;
            let response = submit(&f.app, &f.cookies, &csrf, &input).await;
            let other = committed(
                &response,
                StatusCode::CREATED,
                command,
                f.created.administrator,
                false,
            );
            durable(&pool, &other, &input, f.created.administrator).await;
            for (cookies, company) in [
                (&other_cookies, f.company()),
                (&f.cookies, OrgId::from_uuid(other.company)),
            ] {
                let before = all_rows(&pool).await;
                let mut reader = PgNativePayrollRunsReadPort::new(
                    f.runtime.clone(),
                    f.verifier.clone(),
                    f.ttl,
                    f.policy.clone(),
                    read_credentials(cookies),
                    company,
                );
                match list_payroll_runs(
                    &mut reader,
                    ListPayrollRuns {
                        limit: None,
                        offset: None,
                    },
                )
                .await
                {
                    Err(PayrollRunsReadError::Authorization(error)) => {
                        assert_eq!(error.kind, console_kernel_core::ErrorKind::NotFound)
                    }
                    _ => panic!("foreign Account/Company released or misclassified a Payroll view"),
                }
                assert!(before == all_rows(&pool).await);
            }
            successful_empty_read(&pool, &f).await;
        })
        .catch_unwind()
        .await;
        f.close(result).await;
    }
    async fn company_lock(pool: &PgPool, company: Uuid) -> bool {
        let mut tx = pool.begin().await.unwrap();
        let result = sqlx::query(
            "SELECT org_id FROM public.company_authority_heads WHERE org_id=$1 FOR UPDATE NOWAIT",
        )
        .bind(company)
        .fetch_one(tx.as_mut())
        .await;
        tx.rollback().await.unwrap();
        match result {
            Ok(_) => true,
            Err(sqlx::Error::Database(error)) if error.code().as_deref() == Some("55P03") => false,
            Err(error) => panic!("Company lock probe failed: {error}"),
        }
    }
    async fn released_company_lock(pool: &PgPool, company: Uuid) {
        tokio::time::timeout(std::time::Duration::from_secs(3), async {
            while !company_lock(pool, company).await {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("cancelled/dropped reader retained its Company lock");
    }
    #[sqlx::test(migrations = false)]
    async fn native_read_retains_then_releases_owner_locks_on_drop_and_cancellation(pool: PgPool) {
        let f = FixtureRead::new(&pool).await;
        let result = AssertUnwindSafe(async {
            f.install().await; f.grant(&pool).await;
            successful_empty_read(&pool, &f).await;
            let before = all_rows(&pool).await;
            let mut reader = f.reader(); reader.authorize().await.unwrap();
            assert!(!company_lock(&pool, f.created.company).await, "authorization did not retain Company owner lock");
            assert!(before == all_rows(&pool).await);
            drop(reader); released_company_lock(&pool, f.created.company).await;
            assert!(before == all_rows(&pool).await);

            let mut reader = f.reader(); reader.authorize().await.unwrap();
            let mut blocker = pool.begin().await.unwrap();
            sqlx::query("LOCK TABLE public.payroll_draft_runs IN ACCESS EXCLUSIVE MODE").execute(blocker.as_mut()).await.unwrap();
            let holder: i32 = sqlx::query_scalar("SELECT pg_backend_pid()").fetch_one(blocker.as_mut()).await.unwrap();
            // Cancel only the pending future: the reader remains alive.
            let mut work = reader.read_page(ListPayrollRuns { limit: None, offset: None });
            let observed = tokio::select! {
                _ = &mut work => false,
                observed = tokio::time::timeout(std::time::Duration::from_millis(800), async {
                    loop {
                        let waiting: i64 = sqlx::query_scalar("SELECT count(*) FROM pg_catalog.pg_stat_activity a WHERE a.datname=current_database() AND a.usename='console_rt' AND a.query LIKE '%COUNT(*) FROM payroll_draft_runs%' AND $1=ANY(pg_catalog.pg_blocking_pids(a.pid))")
                            .bind(holder).fetch_one(&pool).await.unwrap();
                        if waiting == 1 { break !company_lock(&pool, f.created.company).await; }
                        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                    }
                }) => matches!(observed, Ok(true)),
            };
            drop(work);
            blocker.rollback().await.unwrap();
            released_company_lock(&pool, f.created.company).await;
            assert!(observed, "read never reached the actual blocked Payroll query");
            assert!(matches!(reader.read_page(ListPayrollRuns { limit: None, offset: None }).await, Err(PayrollRunsReadError::Unavailable)));
            drop(reader);
            assert!(before == all_rows(&pool).await, "cancelled read persisted an audit or other effect");
            successful_empty_read(&pool, &f).await;
        }).catch_unwind().await;
        f.close(result).await;
    }

    #[sqlx::test(migrations = false)]
    async fn native_read_assignment_expiry_after_authorize_withholds_page_and_audit(pool: PgPool) {
        let f = FixtureRead::new(&pool).await;
        let result = AssertUnwindSafe(async {
            f.install().await;
            let now: time::OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
                .fetch_one(&pool)
                .await
                .unwrap();
            // A real, whole-minute grant, never an edited assignment/receipt.
            // Leave enough time for the witnessed initial read on a loaded runner.
            let deadline = (now.unix_timestamp() / 60 + 1) * 60;
            let deadline = if deadline - now.unix_timestamp() >= 5 {
                deadline
            } else {
                deadline + 60
            };
            let expires = time::OffsetDateTime::from_unix_timestamp(deadline).unwrap();
            f.grant_until(expires).await;
            successful_empty_read(&pool, &f).await;
            // Wait outside the reader transaction: console_rt has real 30s idle
            // and 45s transaction deadlines, which must remain enabled.
            tokio::time::timeout(std::time::Duration::from_secs(66), async {
                loop {
                    let now: time::OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
                        .fetch_one(&pool)
                        .await
                        .unwrap();
                    if expires - now <= time::Duration::seconds(10) {
                        break;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                }
            })
            .await
            .expect("database clock did not reach bounded admission window");
            let before = all_rows(&pool).await;
            let mut reader = f.reader();
            reader.authorize().await.unwrap();
            let observed: time::OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
                .fetch_one(&pool)
                .await
                .unwrap();
            assert!(
                expires - observed > time::Duration::seconds(2)
                    && expires - observed <= time::Duration::seconds(10),
                "positive retained-authorization prerequisite missed bounded expiry window"
            );
            assert!(!company_lock(&pool, f.created.company).await);
            tokio::time::timeout(std::time::Duration::from_secs(66), async {
                loop {
                    let observed: time::OffsetDateTime =
                        sqlx::query_scalar("SELECT clock_timestamp()")
                            .fetch_one(&pool)
                            .await
                            .unwrap();
                    if observed >= expires {
                        break;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                }
            })
            .await
            .expect("database clock did not cross actual granted interval");
            match reader
                .read_page(ListPayrollRuns {
                    limit: None,
                    offset: None,
                })
                .await
            {
                Err(PayrollRunsReadError::Authorization(error)) => {
                    assert_eq!(error.kind, console_kernel_core::ErrorKind::NotFound)
                }
                _ => panic!("expired final Payroll authority released or misclassified a page"),
            }
            drop(reader);
            released_company_lock(&pool, f.created.company).await;
            assert!(
                before == all_rows(&pool).await,
                "expired retained read left an audit or other effect"
            );
            denied_without_writes(&pool, &f).await;
        })
        .catch_unwind()
        .await;
        f.close(result).await;
    }

    use console_identity_application::company_policy::{
        CompanyPolicyDecision, CompanyPolicyDecisionPort, CompanyPolicyError, CompanyPolicyRequest,
        CurrentCompanyAuthority, CurrentNativeBootstrapAuthority, CurrentPayrollReadAuthority,
        InitialCompanyAction, NativeBootstrapRequestV1,
    };
    use std::sync::Mutex;

    // Test-only named faults delegate to actual Cedar first and never invent Allow.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Fault {
        InitialIdentityDeny,
        FinalIdentityDeny,
        InitialIdentityFailure,
        FinalIdentityFailure,
        FinalPayrollDeny,
    }
    #[derive(Default)]
    struct DecisionTrace {
        events: Vec<&'static str>,
        payroll: usize,
        identity: usize,
        faults: usize,
        binding: Option<(String, i32, Uuid, i64, i64)>,
        observed: Option<time::OffsetDateTime>,
    }
    struct FaultPolicy {
        real: Arc<CompanyPolicy>,
        company: OrgId,
        actor: Uuid,
        fault: Fault,
        trace: Mutex<DecisionTrace>,
    }
    impl CompanyPolicyDecisionPort for FaultPolicy {
        fn decide_native_bootstrap(
            &self,
            _: &CurrentNativeBootstrapAuthority,
            _: &NativeBootstrapRequestV1,
        ) -> Result<CompanyPolicyDecision, CompanyPolicyError> {
            panic!("Payroll reader attempted bootstrap")
        }
        fn decide_native_payroll_collection(
            &self,
            a: &CurrentPayrollReadAuthority,
        ) -> Result<CompanyPolicyDecision, CompanyPolicyError> {
            let row = a.source();
            assert_eq!(row.org_id, *self.company.as_uuid());
            assert_eq!(row.account_id, self.actor);
            assert_eq!(
                self.real.decide_native_payroll_collection(a)?,
                CompanyPolicyDecision::Allow
            );
            let mut t = self.trace.lock().unwrap();
            let binding = (
                row.source_xid.clone(),
                row.source_backend_pid,
                row.assignment_id,
                row.assignment_revision,
                row.account_security_generation,
            );
            if let Some(prior) = &t.binding {
                assert_eq!(prior, &binding);
            }
            if let Some(prior) = t.observed {
                assert!(
                    a.observed_at() > prior,
                    "final source must carry fresh DB time"
                );
            }
            t.binding = Some(binding);
            t.observed = Some(a.observed_at());
            t.payroll += 1;
            t.events.push("payroll");
            if self.fault == Fault::FinalPayrollDeny && t.payroll == 2 {
                t.faults += 1;
                return Ok(CompanyPolicyDecision::Deny);
            }
            Ok(CompanyPolicyDecision::Allow)
        }
        fn decide(
            &self,
            a: &CurrentCompanyAuthority,
            r: &CompanyPolicyRequest,
        ) -> Result<CompanyPolicyDecision, CompanyPolicyError> {
            assert_eq!(a.company(), self.company);
            assert_eq!(*a.account().as_uuid(), self.actor);
            let clause = a
                .clauses()
                .iter()
                .find(|c| !c.delegable() && c.action_kind() == InitialCompanyAction::ReadIdentity)
                .unwrap();
            assert_eq!(r.requested_company(), self.company);
            assert_eq!(r.object_id(), *self.company.as_uuid());
            assert_eq!(r.object_type_id(), clause.action().object_type_id());
            assert_eq!(r.action(), clause.action());
            assert_eq!(r.requested_properties(), clause.fields());
            assert_eq!(r.requested_properties().len(), 2);
            assert_eq!(self.real.decide(a, r)?, CompanyPolicyDecision::Allow);
            let mut t = self.trace.lock().unwrap();
            t.identity += 1;
            t.events.push("identity");
            let applies = matches!(
                (self.fault, t.identity),
                (
                    Fault::InitialIdentityDeny | Fault::InitialIdentityFailure,
                    1
                ) | (Fault::FinalIdentityDeny | Fault::FinalIdentityFailure, 2)
            );
            if applies {
                t.faults += 1;
                return if matches!(
                    self.fault,
                    Fault::InitialIdentityFailure | Fault::FinalIdentityFailure
                ) {
                    Err(CompanyPolicyError::EvaluatorUnavailable)
                } else {
                    Ok(CompanyPolicyDecision::Deny)
                };
            }
            Ok(CompanyPolicyDecision::Allow)
        }
    }
    #[sqlx::test(migrations = false)]
    async fn native_read_named_policy_faults_never_substitute_identity_or_release_failed_reads(
        pool: PgPool,
    ) {
        let f = FixtureRead::new(&pool).await;
        let result = AssertUnwindSafe(async {
            f.install().await; f.grant(&pool).await;
            successful_empty_read(&pool, &f).await;
            // This sequence survives rollback; it witnesses the real audit insert.
            sqlx::raw_sql("CREATE SEQUENCE public.test_native_read_final_witness; GRANT USAGE,SELECT ON SEQUENCE public.test_native_read_final_witness TO console_rt; CREATE FUNCTION public.test_native_read_final_witness_fn() RETURNS trigger LANGUAGE plpgsql SET search_path=pg_catalog,pg_temp AS $body$ BEGIN IF NEW.action='payroll_run.list_read' THEN PERFORM nextval('public.test_native_read_final_witness'); PERFORM pg_advisory_xact_lock(719238461::bigint); END IF; RETURN NEW; END $body$; CREATE TRIGGER test_native_read_final_witness BEFORE INSERT ON public.audit_events FOR EACH ROW EXECUTE FUNCTION public.test_native_read_final_witness_fn()")
                .execute(&pool).await.unwrap();
            for fault in [Fault::InitialIdentityDeny, Fault::FinalIdentityDeny,
                Fault::InitialIdentityFailure, Fault::FinalIdentityFailure, Fault::FinalPayrollDeny] {
                let policy = Arc::new(FaultPolicy { real: f.policy.clone(), company: f.company(),
                    actor: f.created.administrator, fault, trace: Mutex::new(DecisionTrace::default()) });
                let mut reader = PgNativePayrollRunsReadPort::new(f.runtime.clone(), f.verifier.clone(), f.ttl,
                    policy.clone(), read_credentials(&f.cookies), f.company());
                let before = all_rows(&pool).await;
                let fired_before: i64 = sqlx::query_scalar("SELECT CASE WHEN is_called THEN last_value ELSE 0 END FROM public.test_native_read_final_witness").fetch_one(&pool).await.unwrap();
                let admission = reader.authorize().await;
                assert_eq!(policy.trace.lock().unwrap().events, ["payroll", "identity"]);
                if fault == Fault::InitialIdentityFailure {
                    assert!(matches!(admission, Err(PayrollRunsReadError::Unavailable)));
                } else {
                    admission.unwrap();
                    // Hold the actual audit INSERT before final decisions; counts
                    // observed after return alone cannot prove execution order.
                    let mut audit_latch = pool.begin().await.unwrap();
                    sqlx::query("SELECT pg_advisory_xact_lock(719238461::bigint)").execute(audit_latch.as_mut()).await.unwrap();
                    let holder: i32 = sqlx::query_scalar("SELECT pg_backend_pid()").fetch_one(audit_latch.as_mut()).await.unwrap();
                    let observe = async {
                        let witness = tokio::time::timeout(std::time::Duration::from_millis(800), async {
                            loop {
                                let waiting: i64 = sqlx::query_scalar("SELECT count(*) FROM pg_catalog.pg_stat_activity a WHERE a.datname=current_database() AND a.usename='console_rt' AND a.query ILIKE '%INSERT INTO audit_events%' AND $1=ANY(pg_catalog.pg_blocking_pids(a.pid))")
                                    .bind(holder).fetch_one(&pool).await.unwrap();
                                if waiting == 1 { break policy.trace.lock().unwrap().events.clone(); }
                                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                            }
                        }).await;
                        audit_latch.rollback().await.unwrap();
                        witness.expect("no actual audit INSERT wait observed")
                    };
                    let (result, events_at_audit) = tokio::join!(
                        reader.read_page(ListPayrollRuns { limit: None, offset: None }), observe);
                    assert_eq!(events_at_audit, ["payroll", "identity"], "final decision preceded audit insert");
                    match fault {
                        Fault::InitialIdentityDeny | Fault::FinalIdentityDeny => {
                            let result = result.unwrap(); assert!(result.page.items.is_empty());
                            let context = result.company.unwrap(); assert_eq!(context.id, f.company());
                            assert!(context.identity.is_none(), "initial or final denial leaked identity pair");
                        }
                        Fault::FinalIdentityFailure => assert!(matches!(result, Err(PayrollRunsReadError::Unavailable))),
                        Fault::FinalPayrollDeny => match result {
                            Err(PayrollRunsReadError::Authorization(e)) => assert_eq!(e.kind, console_kernel_core::ErrorKind::NotFound),
                            _ => panic!("final Payroll Deny released or misclassified result"),
                        },
                        Fault::InitialIdentityFailure => unreachable!(),
                    }
                }
                drop(reader); released_company_lock(&pool, f.created.company).await;
                let t = policy.trace.lock().unwrap();
                assert_eq!(t.faults, 1, "targeted fault never fired or fired repeatedly");
                let expected: &[&str] = match fault {
                    Fault::InitialIdentityFailure => &["payroll", "identity"],
                    Fault::FinalPayrollDeny => &["payroll", "identity", "payroll"],
                    _ => &["payroll", "identity", "payroll", "identity"],
                };
                assert_eq!(t.events, expected); drop(t);
                let fired_after: i64 = sqlx::query_scalar("SELECT CASE WHEN is_called THEN last_value ELSE 0 END FROM public.test_native_read_final_witness").fetch_one(&pool).await.unwrap();
                assert_eq!(fired_after - fired_before, if fault == Fault::InitialIdentityFailure { 0 } else { 1 });
                let after = all_rows(&pool).await;
                if matches!(fault, Fault::InitialIdentityDeny | Fault::FinalIdentityDeny) {
                    assert!(before.keys().eq(after.keys()));
                    for (table, value) in &before { if table != "audit_events" { assert!(value == &after[table], "changed {table}"); } }
                    let audits = added_rows(&before["audit_events"], &after["audit_events"]).unwrap();
                    assert_eq!(audits.len(), 1); assert_eq!(audits[0]["action"], "payroll_run.list_read");
                    assert_eq!(audits[0]["org_id"], json!(f.created.company));
                    assert_eq!(audits[0]["actor"], json!(f.created.administrator));
                } else { assert!(before == after, "failed read persisted effects"); }
                successful_empty_read(&pool, &f).await;
            }
        }).catch_unwind().await;
        let cleanup = sqlx::raw_sql("DROP TRIGGER IF EXISTS test_native_read_final_witness ON public.audit_events; DROP FUNCTION IF EXISTS public.test_native_read_final_witness_fn(); DROP SEQUENCE IF EXISTS public.test_native_read_final_witness;")
            .execute(&pool).await;
        f.close(result).await;
        cleanup.expect("restore final fault witness objects");
    }

    #[sqlx::test(migrations = false)]
    async fn native_reader_rejects_reuse_and_out_of_order_calls_without_effects(pool: PgPool) {
        let f = FixtureRead::new(&pool).await;
        let result = AssertUnwindSafe(async {
            f.install().await;
            f.grant(&pool).await;
            let before = all_rows(&pool).await;
            let query = ListPayrollRuns {
                limit: None,
                offset: None,
            };
            let mut reader = f.reader();
            assert!(matches!(
                reader.read_page(query).await,
                Err(PayrollRunsReadError::Unavailable)
            ));
            assert!(matches!(
                reader.authorize().await,
                Err(PayrollRunsReadError::Unavailable)
            ));
            drop(reader);
            assert!(before == all_rows(&pool).await);
            let mut reader = f.reader();
            reader.authorize().await.unwrap();
            assert!(matches!(
                reader.authorize().await,
                Err(PayrollRunsReadError::Unavailable)
            ));
            assert!(matches!(
                reader.read_page(query).await,
                Err(PayrollRunsReadError::Unavailable)
            ));
            drop(reader);
            released_company_lock(&pool, f.created.company).await;
            assert!(before == all_rows(&pool).await);
            let mut reader = f.reader();
            list_payroll_runs(&mut reader, query).await.unwrap();
            let committed = all_rows(&pool).await;
            assert!(matches!(
                reader.read_page(query).await,
                Err(PayrollRunsReadError::Unavailable)
            ));
            assert!(matches!(
                reader.authorize().await,
                Err(PayrollRunsReadError::Unavailable)
            ));
            assert!(committed == all_rows(&pool).await);
            successful_empty_read(&pool, &f).await;
        })
        .catch_unwind()
        .await;
        f.close(result).await;
    }
}
