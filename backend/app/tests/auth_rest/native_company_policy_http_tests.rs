// Include inside existing auth_rest company_setup module. Entire fixture uses
// the real router, signing key, Account/designee owners and mounted Company use case.
use console_identity_application::company_policy::{
    CompanyPolicyDecision, CompanyPolicyDecisionPort, CompanyPolicyError, CompanyPolicyRequest,
    CurrentCompanyAuthority, InitialCompanyAction,
};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU8, Ordering},
};

struct ObservedCompanyPolicy {
    real: console_platform_authz::company_policy::CompanyPolicy,
    mode: AtomicU8,
    calls: Mutex<Vec<(InitialCompanyAction, CompanyPolicyRequest)>>,
}
impl ObservedCompanyPolicy {
    fn new() -> Self {
        Self {
            real: console_platform_authz::company_policy::CompanyPolicy::new().unwrap(),
            mode: AtomicU8::new(0),
            calls: Mutex::new(Vec::new()),
        }
    }
    fn select(&self, mode: u8) {
        self.calls.lock().unwrap().clear();
        self.mode.store(mode, Ordering::SeqCst);
    }
    fn expect_calls(&self, company: Uuid, expected: &[(InitialCompanyAction, usize)]) {
        let calls = self.calls.lock().unwrap();
        assert_eq!(
            calls.len(),
            expected.len(),
            "actual owning decision port skipped/duplicated"
        );
        for ((kind, request), (expected_kind, fields)) in calls.iter().zip(expected) {
            assert_eq!(kind, expected_kind);
            assert_eq!(*request.requested_company().as_uuid(), company);
            assert_eq!(request.requested_properties().len(), *fields);
            assert_eq!(request.action().org_id(), request.requested_company());
            assert!(
                request
                    .requested_properties()
                    .iter()
                    .all(|f| f.org_id() == request.requested_company()
                        && f.object_type_id() == request.object_type_id())
            );
        }
    }
}
impl CompanyPolicyDecisionPort for ObservedCompanyPolicy {
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
        authority: &CurrentCompanyAuthority,
        request: &CompanyPolicyRequest,
    ) -> Result<CompanyPolicyDecision, CompanyPolicyError> {
        let clause = authority
            .clauses()
            .iter()
            .find(|c| !c.delegable() && c.action() == request.action())
            .expect("owner supplied unknown request action");
        self.calls
            .lock()
            .unwrap()
            .push((clause.action_kind(), request.clone()));
        match self.mode.load(Ordering::SeqCst) {
            0 => self.real.decide(authority, request),
            1 => Ok(CompanyPolicyDecision::Deny),
            2 => Err(CompanyPolicyError::EvaluatorUnavailable),
            3 if matches!(clause.action_kind(), InitialCompanyAction::ReadPolicy) => {
                Ok(CompanyPolicyDecision::Deny)
            }
            3 => self.real.decide(authority, request),
            _ => panic!("invalid test mode"),
        }
    }
}
async fn policy_fixture(
    pool: &PgPool,
) -> (
    Fixture,
    Cookies,
    Attempt,
    PgPool,
    Arc<ObservedCompanyPolicy>,
    SigningKey,
) {
    prepare_http_database(pool).await;
    seed_terms(pool).await;
    let artifacts = Artifacts::new();
    let key = SigningKey::random(&mut OsRng);
    let policy = Arc::new(ObservedCompanyPolicy::new());
    let state = state_with_key(pool, artifacts.root.clone(), &key)
        .await
        .with_company_policy_decider(policy.clone());
    let app = Fixture {
        service: build_router(state),
        _artifacts: artifacts,
        pool: pool.clone(),
    };
    let (operator, cookies) = enrolled(&app).await;
    let startup = startup(pool).await;
    let input = designation(pool, operator.account).await;
    let receipt = designate(&startup, &input).await.unwrap();
    assert_eq!(receipt.1, 1);
    assert!(!receipt.2);
    assert_no_company_identity(pool, operator.account).await;
    (app, cookies, operator, startup, policy, key)
}

#[sqlx::test(migrations = false)]
async fn company_real_policy_port_governs_api_workspace_and_populated_discovery(pool: PgPool) {
    use InitialCompanyAction::{Discover, ReadIdentity, ReadPolicy};
    let (app, operator_cookies, _operator, startup, policy, _key) = policy_fixture(&pool).await;
    let (admin, cookies) = enrolled(&app).await;
    let command = Uuid::new_v4();
    let input = enrollment(command, admin.account);
    let created = committed(
        &submit(&app, &operator_cookies, &input).await,
        StatusCode::CREATED,
        command,
        admin.account,
        false,
    );
    let (other, other_cookies) = enrolled(&app).await;
    let other_command = Uuid::new_v4();
    let mut other_input = enrollment(other_command, other.account);
    other_input["name"] = json!("다른 회사 비공개");
    let other_company = committed(
        &submit(&app, &operator_cookies, &other_input).await,
        StatusCode::CREATED,
        other_command,
        other.account,
        false,
    );
    policy.select(0);
    initial_ceiling(&app, &other_cookies, &other_company).await;
    policy.expect_calls(other_company.company, &[(ReadPolicy, 8)]);
    policy.select(0);
    request(
        &app,
        "GET",
        &format!("/api/v2/companies/{}/policy", other_company.company),
        &cookies,
        None,
        &[],
    )
    .await
    .error(StatusCode::NOT_FOUND, "company_not_found");
    policy.expect_calls(other_company.company, &[]);
    native_entry_html(
        &document(
            &app,
            &format!("/companies/{}", other_company.company),
            &cookies,
        )
        .await,
        StatusCode::NOT_FOUND,
    );
    let before = all_rows(&pool).await;
    policy.select(0);
    initial_ceiling(&app, &cookies, &created).await;
    policy.expect_calls(created.company, &[(ReadPolicy, 8)]);
    policy.select(0);
    native_entry_html(
        &document(&app, &format!("/companies/{}", created.company), &cookies).await,
        StatusCode::OK,
    );
    policy.expect_calls(created.company, &[(ReadIdentity, 2), (ReadPolicy, 0)]);
    policy.select(0);
    let page = document(&app, "/account", &cookies).await;
    let html = native_entry_html(&page, StatusCode::OK);
    assert!(
        html.contains(&format!("href=\"/companies/{}\"", created.company))
            && html.contains("data-context-state=\"populated\"")
    );
    policy.expect_calls(created.company, &[(Discover, 2)]);
    assert!(
        !html.contains("다른 회사 비공개")
            && !html.contains(&format!("href=\"/companies/{}\"", other_company.company))
    );
    assert!(before == all_rows(&pool).await);

    policy.select(1);
    request(
        &app,
        "GET",
        &format!("/api/v2/companies/{}/policy", created.company),
        &cookies,
        None,
        &[],
    )
    .await
    .error(StatusCode::NOT_FOUND, "company_not_found");
    policy.expect_calls(created.company, &[(ReadPolicy, 8)]);
    policy.select(1);
    native_entry_html(
        &document(&app, &format!("/companies/{}", created.company), &cookies).await,
        StatusCode::NOT_FOUND,
    );
    policy.expect_calls(created.company, &[(ReadIdentity, 2)]);
    policy.select(1);
    let page = document(&app, "/account", &cookies).await;
    let html = native_entry_html(&page, StatusCode::OK);
    assert!(
        !html.contains(&format!("href=\"/companies/{}\"", created.company))
            && html.contains("data-context-state=\"empty\"")
    );
    policy.expect_calls(created.company, &[(Discover, 2)]);
    assert!(
        before == all_rows(&pool).await,
        "denied-port control wrote durable policy or history"
    );

    // A denied policy-navigation action must not hide an independently allowed identity view.
    policy.select(3);
    let page = document(&app, &format!("/companies/{}", created.company), &cookies).await;
    let html = native_entry_html(&page, StatusCode::OK);
    assert!(!html.contains(&format!("href=\"/companies/{}/policy\"", created.company)));
    policy.expect_calls(created.company, &[(ReadIdentity, 2), (ReadPolicy, 0)]);
    assert!(before == all_rows(&pool).await);

    policy.select(2);
    request(
        &app,
        "GET",
        &format!("/api/v2/companies/{}/policy", created.company),
        &cookies,
        None,
        &[],
    )
    .await
    .error(
        StatusCode::SERVICE_UNAVAILABLE,
        "company_policy_unavailable",
    );
    policy.expect_calls(created.company, &[(ReadPolicy, 8)]);
    policy.select(2);
    native_entry_html(
        &document(&app, &format!("/companies/{}", created.company), &cookies).await,
        StatusCode::SERVICE_UNAVAILABLE,
    );
    policy.expect_calls(created.company, &[(ReadIdentity, 2)]);
    policy.select(2);
    let page = document(&app, "/account", &cookies).await;
    let html = native_entry_html(&page, StatusCode::SERVICE_UNAVAILABLE);
    assert!(
        html.contains("data-context-state=\"unavailable\"")
            && !html.contains("data-context-state=\"empty\"")
    );
    assert!(!html.contains(&format!("href=\"/companies/{}\"", created.company)));
    policy.expect_calls(created.company, &[(Discover, 2)]);
    assert!(before == all_rows(&pool).await);

    policy.select(0);
    initial_ceiling(&app, &cookies, &created).await;
    let denied = document(
        &app,
        &format!("/companies/{}", created.company),
        &operator_cookies,
    )
    .await;
    native_entry_html(&denied, StatusCode::NOT_FOUND);
    assert!(
        before == all_rows(&pool).await,
        "reopening or denied operator read changed durable effects"
    );
    startup.close().await;
}

#[sqlx::test(migrations = false)]
async fn company_current_registry_drift_refuses_before_cedar_and_recovers(pool: PgPool) {
    let (app, operator_cookies, _operator, startup, policy, _key) = policy_fixture(&pool).await;
    let (admin, cookies) = enrolled(&app).await;
    let command = Uuid::new_v4();
    let input = enrollment(command, admin.account);
    let created = committed(
        &submit(&app, &operator_cookies, &input).await,
        StatusCode::CREATED,
        command,
        admin.account,
        false,
    );
    policy.select(0);
    initial_ceiling(&app, &cookies, &created).await;
    let(action,title):(Uuid,String)=sqlx::query_as("SELECT a.id,a.title FROM public.ont_action_types a JOIN public.native_company_action_refs r ON r.org_id=a.org_id AND r.action_type_id=a.id WHERE r.org_id=$1 AND r.action_key='company.identity.read'")
        .bind(created.company).fetch_one(&pool).await.unwrap();
    // Actual source-content corruption in this disposable DB; no grant/receipt
    // or successful native identity is seeded and no guard is disabled.
    assert_eq!(
        sqlx::query("UPDATE public.ont_action_types SET title=$1 WHERE id=$2 AND org_id=$3")
            .bind(format!("{title} [fault]"))
            .bind(action)
            .bind(created.company)
            .execute(&pool)
            .await
            .unwrap()
            .rows_affected(),
        1
    );
    let damaged = all_rows(&pool).await;
    policy.select(0);
    request(
        &app,
        "GET",
        &format!("/api/v2/companies/{}/policy", created.company),
        &cookies,
        None,
        &[],
    )
    .await
    .error(
        StatusCode::SERVICE_UNAVAILABLE,
        "company_policy_unavailable",
    );
    assert!(
        policy.calls.lock().unwrap().is_empty(),
        "unvalidated registry material reached Cedar"
    );
    let page = document(&app, "/account", &cookies).await;
    let html = native_entry_html(&page, StatusCode::SERVICE_UNAVAILABLE);
    assert!(
        html.contains("data-context-state=\"unavailable\"")
            && !html.contains(&format!("href=\"/companies/{}\"", created.company))
    );
    assert!(
        damaged == all_rows(&pool).await,
        "failed current-source projection changed durable effects"
    );
    assert_eq!(
        sqlx::query("UPDATE public.ont_action_types SET title=$1 WHERE id=$2 AND org_id=$3")
            .bind(title)
            .bind(action)
            .bind(created.company)
            .execute(&pool)
            .await
            .unwrap()
            .rows_affected(),
        1
    );
    let repaired = all_rows(&pool).await;
    policy.select(0);
    initial_ceiling(&app, &cookies, &created).await;
    assert!(
        repaired == all_rows(&pool).await,
        "recovered projection repeated a write"
    );
    startup.close().await;
}

#[sqlx::test(migrations = false)]
async fn company_current_read_expiry_after_real_company_guard_wait_discards_projection(
    pool: PgPool,
) {
    let (app, operator_cookies, _operator, startup, policy, key) = policy_fixture(&pool).await;
    let (admin, cookies) = enrolled(&app).await;
    let command = Uuid::new_v4();
    let input = enrollment(command, admin.account);
    let created = committed(
        &submit(&app, &operator_cookies, &input).await,
        StatusCode::CREATED,
        command,
        admin.account,
        false,
    );
    policy.select(0);
    initial_ceiling(&app, &cookies, &created).await;
    let before = all_rows(&pool).await;
    let mut held = pool.begin().await.unwrap();
    let blocker: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(held.as_mut())
        .await
        .unwrap();
    let epoch: i64 = sqlx::query_scalar(
        "SELECT epoch FROM public.company_authority_heads WHERE org_id=$1 FOR UPDATE",
    )
    .bind(created.company)
    .fetch_one(held.as_mut())
    .await
    .unwrap();
    assert_eq!(epoch, 1);
    // Same narrower signed-expiry derivative used by accepted signed credential
    // tests; Account/family/claims otherwise come from actual enrollment.
    let expires = tokio::time::timeout(std::time::Duration::from_secs(2), async {
        loop {
            let at: OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
                .fetch_one(&pool)
                .await
                .unwrap();
            if (250_000_000..=350_000_000).contains(&at.nanosecond()) {
                break at.unix_timestamp() + 1;
            }
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap();
    let mut claims = signed_claims(&cookies.0[ACCESS], &key).unwrap();
    claims["exp"] = json!(expires);
    let mut short = cookies.clone();
    short
        .0
        .insert(ACCESS.to_owned(), sign_proof_claims(&claims, &key));
    let target = format!("/api/v2/companies/{}/policy", created.company);
    let (response, witnessed) = tokio::join!(
        request(&app, "GET", &target, &short, None, &[]),
        async {
            let observed=tokio::time::timeout(std::time::Duration::from_secs(2),async {
                loop {
                    let blocked:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_stat_activity a WHERE a.datname=current_database() AND a.usename='console_rt' AND $1=ANY(pg_blocking_pids(a.pid)) AND a.query LIKE '%identity_company_projection_v1%' AND extract(epoch FROM clock_timestamp()) < $2::double precision)")
                        .bind(blocker).bind(expires as f64).fetch_one(&pool).await.unwrap();
                    if blocked {break}
                    tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                }
            }).await.is_ok();
            if observed {
                sqlx::query("SELECT pg_sleep(GREATEST(0.0,$1::double precision-extract(epoch FROM clock_timestamp()))+0.025)")
                    .bind(expires as f64).execute(&pool).await.unwrap();
            }
            held.rollback().await.unwrap();
            observed
        }
    );
    assert!(
        witnessed,
        "actual Company current-source lock wait not reached; unrelated auth/schema failure is not expiry evidence"
    );
    response.error(StatusCode::UNAUTHORIZED, "authentication_invalid");
    assert!(
        before == all_rows(&pool).await,
        "expired current read emitted a durable effect"
    );
    // Original authentic longer-lived cookie remains valid and reopens source.
    initial_ceiling(&app, &cookies, &created).await;
    assert!(before == all_rows(&pool).await);
    startup.close().await;
}

#[sqlx::test(migrations = false)]
async fn company_same_account_two_births_populate_generation_three_without_replay_effect(
    pool: PgPool,
) {
    let (app, operator_cookies, _operator, startup, policy, _key) = policy_fixture(&pool).await;
    let (admin, cookies) = enrolled(&app).await;
    let first_command = Uuid::new_v4();
    let second_command = Uuid::new_v4();
    let first_input = enrollment(first_command, admin.account);
    let second_input = enrollment(second_command, admin.account);
    let first = committed(
        &submit(&app, &operator_cookies, &first_input).await,
        StatusCode::CREATED,
        first_command,
        admin.account,
        false,
    );
    let second = committed(
        &submit(&app, &operator_cookies, &second_input).await,
        StatusCode::CREATED,
        second_command,
        admin.account,
        false,
    );
    let generation: i64 = sqlx::query_scalar(
        "SELECT context_generation FROM public.account_security WHERE account_id=$1",
    )
    .bind(admin.account)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(generation, 3);
    let before = all_rows(&pool).await;
    for replay in [false, true] {
        if replay {
            let prior = committed(
                &submit(&app, &operator_cookies, &first_input).await,
                StatusCode::OK,
                first_command,
                admin.account,
                true,
            );
            assert_eq!(prior.company, first.company);
            assert_eq!(prior.receipt, first.receipt);
            assert!(
                before == all_rows(&pool).await,
                "replay advanced context or rewrote birth history"
            );
        }
        policy.select(0);
        let page = document(&app, "/account", &cookies).await;
        let html = native_entry_html(&page, StatusCode::OK);
        assert!(html.contains("data-context-state=\"populated\""));
        for company in [first.company, second.company] {
            assert!(html.contains(&format!("href=\"/companies/{company}\"")));
        }
        let calls = policy.calls.lock().unwrap();
        assert_eq!(calls.len(), 2);
        let mut expected = vec![first.company, second.company];
        expected.sort();
        for ((kind, request), company) in calls.iter().zip(expected) {
            assert!(matches!(kind, InitialCompanyAction::Discover));
            assert_eq!(*request.requested_company().as_uuid(), company);
            assert_eq!(request.requested_properties().len(), 2);
            assert_eq!(request.action().org_id(), request.requested_company());
            assert!(
                request
                    .requested_properties()
                    .iter()
                    .all(|p| p.org_id() == request.requested_company()
                        && p.object_type_id() == request.object_type_id())
            );
        }
        drop(calls);
        assert!(
            before == all_rows(&pool).await,
            "discovery changed Account generation or Company effects"
        );
    }
    startup.close().await;
}

#[sqlx::test(migrations = false)]
async fn company_actual_projection_retains_every_mutable_registry_source_until_commit(
    pool: PgPool,
) {
    let (app, operator_cookies, _operator, startup, policy, key) = policy_fixture(&pool).await;
    let (admin, cookies) = enrolled(&app).await;
    let command = Uuid::new_v4();
    let input = enrollment(command, admin.account);
    let created = committed(
        &submit(&app, &operator_cookies, &input).await,
        StatusCode::CREATED,
        command,
        admin.account,
        false,
    );
    policy.select(0);
    initial_ceiling(&app, &cookies, &created).await;
    let claims = signed_claims(&cookies.0[ACCESS], &key).unwrap();
    let family: Uuid = claims["sid"].as_str().unwrap().parse().unwrap();
    // Exact minimum owner-only privileges for locking existing immutable IDs.
    for table in ["ont_action_types", "ont_property_defs"] {
        let columns:Vec<String>=sqlx::query_scalar("SELECT a.attname::text FROM pg_attribute a JOIN pg_class c ON c.oid=a.attrelid WHERE c.relnamespace='public'::regnamespace AND c.relname=$1 AND a.attnum>0 AND NOT a.attisdropped AND has_column_privilege('console_ontology_writer',c.oid,a.attnum,'UPDATE') ORDER BY a.attnum")
            .bind(table).fetch_all(&pool).await.unwrap();
        assert_eq!(columns, vec!["id"]);
        let broad:bool=sqlx::query_scalar("SELECT has_table_privilege('console_ontology_writer',c.oid,'UPDATE') OR has_table_privilege('console_rt',c.oid,'UPDATE') OR has_any_column_privilege('console_rt',c.oid,'UPDATE') OR has_any_column_privilege('console_account_owner',c.oid,'UPDATE') FROM pg_class c WHERE c.relnamespace='public'::regnamespace AND c.relname=$1")
            .bind(table).fetch_one(&pool).await.unwrap();
        assert!(!broad);
    }
    for role in ["console_rt", "console_auth_rt", "console_ontology_cmd"] {
        let allowed:bool=sqlx::query_scalar("SELECT has_function_privilege($1,'ontology_api.lock_native_company_catalog_current_v1(uuid)','EXECUTE')")
            .bind(role).fetch_one(&pool).await.unwrap();
        assert!(!allowed, "private source helper exposed to {role}");
    }
    let grants:Vec<(String,bool)>=sqlx::query_as("SELECT CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee)::text END,a.is_grantable FROM pg_proc p CROSS JOIN LATERAL aclexplode(COALESCE(p.proacl,acldefault('f',p.proowner))) a WHERE p.oid='ontology_api.lock_native_company_catalog_current_v1(uuid)'::regprocedure AND a.privilege_type='EXECUTE' ORDER BY 1")
        .fetch_all(&pool).await.unwrap();
    assert_eq!(
        grants,
        vec![
            ("console_account_owner".into(), false),
            ("console_ontology_writer".into(), false)
        ]
    );
    // console_app retains its established migration-only inherited owner rights;
    // no direct grant is added and it is not a serving identity.
    // All2keys/2objects/10properties/5actions from actual committed native maps.
    let sources:Vec<(String,Uuid)>=sqlx::query_as("SELECT 'key'::text,k.validator_id FROM public.ont_object_type_key_revisions k JOIN public.ont_object_types o ON o.org_id=k.org_id AND o.stable_key=k.stable_key JOIN public.native_company_object_refs r ON r.org_id=o.org_id AND r.object_type_id=o.id WHERE r.org_id=$1 UNION ALL SELECT 'object',object_type_id FROM public.native_company_object_refs WHERE org_id=$1 UNION ALL SELECT 'property',property_id FROM public.native_company_property_refs WHERE org_id=$1 UNION ALL SELECT 'action',action_type_id FROM public.native_company_action_refs WHERE org_id=$1 ORDER BY 1,2")
        .bind(created.company).fetch_all(&pool).await.unwrap();
    assert_eq!(sources.len(), 19);
    let runtime =
        console_platform_test_support::login_test_pool(&pool, TestDatabaseLogin::Business).await;
    let roles: (String, String) = sqlx::query_as("SELECT session_user::text,current_user::text")
        .fetch_one(&runtime)
        .await
        .unwrap();
    assert_eq!(roles, ("console_rt".into(), "console_rt".into()));
    let before = all_rows(&pool).await;
    for (kind, id) in sources {
        let mut retained = runtime.begin().await.unwrap();
        let reader: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(retained.as_mut())
            .await
            .unwrap();
        let rows: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM public.identity_company_projection_v1($1,$2,$3)",
        )
        .bind(admin.account)
        .bind(family)
        .bind(created.company)
        .fetch_one(retained.as_mut())
        .await
        .unwrap();
        assert_eq!(rows, 1);
        let mut writer = pool.begin().await.unwrap();
        sqlx::query("SET LOCAL lock_timeout='5s'")
            .execute(writer.as_mut())
            .await
            .unwrap();
        let writer_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(writer.as_mut())
            .await
            .unwrap();
        // Closed static SQL; no caller-generated identifier or arbitrary SQL.
        let update = match kind.as_str() {
            "key" => {
                "UPDATE public.ont_object_type_key_revisions SET revision=revision WHERE org_id=$1 AND validator_id=$2"
            }
            "object" => "UPDATE public.ont_object_types SET title=title WHERE org_id=$1 AND id=$2",
            "property" => {
                "UPDATE public.ont_property_defs SET title=title WHERE org_id=$1 AND id=$2"
            }
            "action" => "UPDATE public.ont_action_types SET title=title WHERE org_id=$1 AND id=$2",
            _ => panic!("unknown actual source kind"),
        };
        let (effect, witness) = tokio::join!(
            sqlx::query(update)
                .bind(created.company)
                .bind(id)
                .execute(writer.as_mut()),
            async {
                let observed=tokio::time::timeout(std::time::Duration::from_secs(2),async {
                    loop {
                        let blocked:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_stat_activity a WHERE a.datname=current_database() AND a.pid=$1 AND $2=ANY(pg_blocking_pids(a.pid)))")
                            .bind(writer_pid).bind(reader).fetch_one(&pool).await.unwrap();
                        if blocked {break}
                        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                    }
                }).await.is_ok();
                retained.commit().await.unwrap();
                observed
            }
        );
        let rows = effect.unwrap().rows_affected();
        writer.rollback().await.unwrap();
        assert!(
            witness,
            "{kind}/{id} was mutable after projection returned; source lock not retained"
        );
        assert_eq!(rows, 1);
        assert!(
            before == all_rows(&pool).await,
            "source-lock probe or read changed durable state"
        );
    }
    runtime.close().await;
    startup.close().await;
}
