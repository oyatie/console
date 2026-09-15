//! LC05: actual production startup and HTTP readiness. The temp-catalog
//! regression explicitly constructs a state to control readiness session reuse.
//! Requires ordinary migration and production finalization helpers; missing
//! lifecycle prerequisites cannot be counted as startup refusal evidence.
use super::{account_transport_urls, finalize_account_custody, prepare_http_database_staging};
use axum::body::Body;
use console_app::{AppConfig, AppRole, AppState, build_router};
use console_platform_test_support::{TestDatabaseLogin, login_test_pool};
use http::{Request, StatusCode};
use sqlx::PgPool;
use tower::ServiceExt;

fn config(pool: &PgPool, role: AppRole) -> AppConfig {
    let mut pairs = vec![
        ("CONSOLE_APP_ROLE", role.to_string()),
        ("CONSOLE_HTTP_ADDR", "127.0.0.1:0".to_owned()),
    ];
    pairs.extend(account_transport_urls(pool));
    AppConfig::from_pairs(pairs).expect("valid real runtime configuration")
}

async fn ready(state: &AppState) -> StatusCode {
    build_router(state.clone())
        .oneshot(
            Request::builder()
                .uri("/readyz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
        .status()
}

async fn pending_then_finalized(pool: PgPool, role: AppRole) {
    prepare_http_database_staging(&pool).await;
    // A failed login is not custody refusal. Authenticate separately first.
    let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
    assert_eq!(
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&runtime)
            .await
            .unwrap(),
        1
    );
    let attempt = AppState::from_config(config(&pool, role)).await;
    if let Ok(state) = &attempt {
        state.shutdown_realtime().await;
    }
    let error = attempt
        .err()
        .expect("pending custody must refuse actual startup");
    assert!(
        error.to_string().contains("account_custody"),
        "specific custody refusal, not unrelated setup failure: {error}"
    );
    finalize_account_custody(&pool).await;
    let state = AppState::from_config(config(&pool, role))
        .await
        .expect("same transport starts after actual finalization");
    assert_eq!(ready(&state).await, StatusCode::OK);
    state.shutdown_realtime().await;
    runtime.close().await;
}

#[sqlx::test(migrations = false)]
async fn custody_api_startup_requires_actual_finalization(pool: PgPool) {
    pending_then_finalized(pool, AppRole::Api).await;
}

#[sqlx::test(migrations = false)]
async fn custody_worker_startup_requires_actual_finalization(pool: PgPool) {
    pending_then_finalized(pool, AppRole::Worker).await;
}

async fn drift_after_start(pool: PgPool, role: AppRole, mutation: &'static str) {
    prepare_http_database_staging(&pool).await;
    finalize_account_custody(&pool).await;
    let state = AppState::from_config(config(&pool, role))
        .await
        .expect("finalized positive startup");
    assert_eq!(ready(&state).await, StatusCode::OK);
    // Explicit hostile administrative mutation in this disposable database only.
    sqlx::raw_sql(mutation).execute(&pool).await.unwrap();
    let changed: bool = sqlx::query_scalar("SELECT pg_get_userbyid(c.relowner) = 'console_app' OR has_column_privilege('console_rt', c.oid, 'id', 'SELECT') FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='public' AND c.relname='accounts'")
        .fetch_one(&pool).await.unwrap();
    assert!(changed, "fault injection must alter actual custody");
    assert_eq!(ready(&state).await, StatusCode::SERVICE_UNAVAILABLE);
    // Readiness is detection, not proof that already running consumers stopped.
    state.shutdown_realtime().await;
}

#[sqlx::test(migrations = false)]
async fn custody_api_readyz_detects_owner_drift(pool: PgPool) {
    drift_after_start(
        pool,
        AppRole::Api,
        "ALTER TABLE public.accounts OWNER TO console_app",
    )
    .await;
}
#[sqlx::test(migrations = false)]
async fn custody_worker_readyz_detects_owner_drift(pool: PgPool) {
    drift_after_start(
        pool,
        AppRole::Worker,
        "ALTER TABLE public.accounts OWNER TO console_app",
    )
    .await;
}
#[sqlx::test(migrations = false)]
async fn custody_api_readyz_detects_column_grant_drift(pool: PgPool) {
    drift_after_start(
        pool,
        AppRole::Api,
        "GRANT SELECT(id) ON public.accounts TO console_rt",
    )
    .await;
}
#[sqlx::test(migrations = false)]
async fn custody_worker_readyz_detects_column_grant_drift(pool: PgPool) {
    drift_after_start(
        pool,
        AppRole::Worker,
        "GRANT SELECT(id) ON public.accounts TO console_rt",
    )
    .await;
}

#[tokio::test]
async fn custody_genuine_no_database_configuration_keeps_existing_behavior() {
    for role in [AppRole::Api, AppRole::Worker] {
        let config = AppConfig::from_pairs([
            ("CONSOLE_APP_ROLE", role.to_string()),
            ("CONSOLE_HTTP_ADDR", "127.0.0.1:0".to_owned()),
        ])
        .unwrap();
        let state = AppState::from_config(config)
            .await
            .expect("genuine no database mode");
        assert_eq!(ready(&state).await, StatusCode::OK);
        state.shutdown_realtime().await;
    }
}

async fn partial_catalog(pool: PgPool, role: AppRole) {
    prepare_http_database_staging(&pool).await;
    finalize_account_custody(&pool).await;
    let healthy = AppState::from_config(config(&pool, role)).await.unwrap();
    assert_eq!(ready(&healthy).await, StatusCode::OK);
    healthy.shutdown_realtime().await;
    sqlx::raw_sql("ALTER TABLE public.accounts RENAME TO accounts_custody_hidden")
        .execute(&pool)
        .await
        .unwrap();
    let missing: bool = sqlx::query_scalar("SELECT to_regclass('public.accounts') IS NULL AND to_regclass('public.accounts_custody_hidden') IS NOT NULL")
        .fetch_one(&pool).await.unwrap();
    assert!(missing);
    let attempt = AppState::from_config(config(&pool, role)).await;
    if let Ok(state) = &attempt {
        state.shutdown_realtime().await;
    }
    let error = attempt.err().expect("partial catalog must refuse startup");
    assert!(
        error.to_string().contains("account_custody"),
        "not an unrelated database/setup failure: {error}"
    );
}

#[sqlx::test(migrations = false)]
async fn custody_api_startup_refuses_partial_catalog(pool: PgPool) {
    partial_catalog(pool, AppRole::Api).await;
}
#[sqlx::test(migrations = false)]
async fn custody_worker_startup_refuses_partial_catalog(pool: PgPool) {
    partial_catalog(pool, AppRole::Worker).await;
}

#[tokio::test]
async fn custody_configured_database_failure_does_not_become_no_database_mode() {
    let config = AppConfig::from_pairs([
        ("CONSOLE_APP_ROLE", "worker"),
        ("CONSOLE_HTTP_ADDR", "127.0.0.1:0"),
        (
            "DATABASE_URL",
            "postgresql://console_rt:synthetic@127.0.0.1:1/unreachable",
        ),
    ])
    .unwrap();
    let outcome = AppState::from_config(config).await;
    assert!(
        matches!(outcome, Err(console_app::AppError::Database(_))),
        "configured transport failure must remain a database error, not ready no-DB state"
    );
}

/// Readiness must inspect real catalogs on a reused runtime session. Unlike the
/// startup cases above, constructing AppState here deliberately controls the
/// single connection consumed by the actual HTTP readiness adapter.
#[sqlx::test(migrations = false)]
async fn custody_readyz_ignores_session_temporary_pg_class_forgery(pool: PgPool) {
    prepare_http_database_staging(&pool).await;
    finalize_account_custody(&pool).await;
    let login = login_test_pool(&pool, TestDatabaseLogin::Business).await;
    let options = login.connect_options().as_ref().clone();
    login.close().await;
    let runtime = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .idle_timeout(None)
        .max_lifetime(None)
        .connect_with(options)
        .await
        .expect("genuine runtime connection with no additional privileges");
    let identity: (String, String, i32) =
        sqlx::query_as("SELECT session_user::text,current_user::text,pg_catalog.pg_backend_pid()")
            .fetch_one(&runtime)
            .await
            .unwrap();
    assert_eq!(identity.0, "console_rt");
    assert_eq!(identity.1, "console_rt");
    // Copy public catalog metadata only. No private Account rows or changed
    // privileges are needed for implicit pg_temp relation-name precedence.
    sqlx::raw_sql("CREATE TEMP TABLE pg_class AS SELECT * FROM pg_catalog.pg_class; RESET ALL")
        .execute(&runtime)
        .await
        .unwrap();
    let state = AppState::new(
        config(&pool, AppRole::Worker),
        console_app::DatabaseDependency::Postgres(runtime.clone()),
    )
    .expect("controlled runtime pool for actual readiness reuse");
    assert_eq!(ready(&state).await, StatusCode::OK);
    sqlx::raw_sql("ALTER TABLE public.accounts OWNER TO console_app")
        .execute(&pool)
        .await
        .unwrap();
    let real_owner: String = sqlx::query_scalar(
        "SELECT pg_catalog.pg_get_userbyid(c.relowner) FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='public' AND c.relname='accounts'",
    ).fetch_one(&pool).await.unwrap();
    let stale_owner: String = sqlx::query_scalar(
        "SELECT pg_catalog.pg_get_userbyid(c.relowner) FROM pg_temp.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='public' AND c.relname='accounts'",
    ).fetch_one(&runtime).await.unwrap();
    assert_eq!(real_owner, "console_app", "real owner drift must exist");
    assert_eq!(
        stale_owner, "console_account_owner",
        "session forgery must still present finalized ownership"
    );
    let status = ready(&state).await;
    let reused_pid: i32 = sqlx::query_scalar("SELECT pg_catalog.pg_backend_pid()")
        .fetch_one(&runtime)
        .await
        .unwrap();
    state.shutdown_realtime().await;
    runtime.close().await;
    assert_eq!(
        reused_pid, identity.2,
        "test must reuse the same forged runtime session"
    );
    assert_eq!(
        status,
        StatusCode::SERVICE_UNAVAILABLE,
        "temporary pg_class must not hide actual custody drift"
    );
}

mod current_profile_admission {
    use super::*;
    use console_kernel_core::UserId;
    use futures::FutureExt;
    use serde_json::{Value, json};
    use sha2::{Digest, Sha256};
    use uuid::Uuid;

    async fn retained_ready(router: &axum::Router) -> StatusCode {
        router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/readyz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap()
            .status()
    }

    async fn seed_snapshot_rows(pool: &PgPool) {
        super::super::insert_account_fence(pool, UserId::new(), "ACTIVE").await;
        // Data-only retained bytes: neither an approved publication nor a helper
        // that implements the future publisher. No head/FK grant is fabricated.
        let approval = b"TEST_ONLY custody startup receipt; never publication authority";
        sqlx::query("INSERT INTO public.account_terms_release_receipts(id,previous_revision,revision,manifest_sha256,approved_release_ref,approval_bytes,recorded_at) VALUES($1,NULL,1,$2,$3,$4,now())")
            .bind(Uuid::new_v4()).bind(Sha256::digest(b"TEST_ONLY custody manifest").to_vec())
            .bind(json!({"kind":"OPERATOR_RELEASE_APPROVAL","approval_sha256":hex::encode(Sha256::digest(approval))}))
            .bind(approval.as_slice()).execute(pool).await.expect("actual data-only receipt storage");
    }

    async fn complete_state(pool: &PgPool) -> Value {
        // Logical catalog rows are exact; committed fault/restore legitimately
        // advances transaction IDs. Never print retained approval/Account data.
        sqlx::query_scalar(r#"SELECT jsonb_build_object(
          'rows',jsonb_build_object(
            'accounts',COALESCE((SELECT jsonb_agg(to_jsonb(a) ORDER BY a.id) FROM public.accounts a),'[]'::jsonb),
            'security',COALESCE((SELECT jsonb_agg(to_jsonb(s) ORDER BY s.account_id) FROM public.account_security s),'[]'::jsonb),
            'events',COALESCE((SELECT jsonb_agg(to_jsonb(e) ORDER BY e.id) FROM public.account_security_events e),'[]'::jsonb),
            'acceptances',COALESCE((SELECT jsonb_agg(to_jsonb(a) ORDER BY a.account_id,a.terms_kind,a.terms_version) FROM public.account_terms_acceptances a),'[]'::jsonb),
            'head',COALESCE((SELECT jsonb_agg(to_jsonb(h) ORDER BY h.id) FROM public.account_terms_head h),'[]'::jsonb),
            'receipts',COALESCE((SELECT jsonb_agg(to_jsonb(r) ORDER BY r.id) FROM public.account_terms_release_receipts r),'[]'::jsonb),
            'audit',COALESCE((SELECT jsonb_agg(to_jsonb(a) ORDER BY a.id) FROM public.audit_events a),'[]'::jsonb)),
          'functions',(SELECT COALESCE(jsonb_agg(to_jsonb(p) ORDER BY p.oid),'[]'::jsonb)
            FROM pg_catalog.pg_proc p WHERE p.pronamespace='public'::regnamespace
              AND p.proname IN ('account_legacy_fenced_v1','account_terms_receipts_immutable_v1')),
          'triggers',(SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY t.oid),'[]'::jsonb)
            FROM pg_catalog.pg_trigger t JOIN pg_catalog.pg_class c ON c.oid=t.tgrelid
            WHERE c.relnamespace='public'::regnamespace AND c.relname IN
              ('accounts','account_security','account_security_events','account_terms_acceptances','account_terms_head','account_terms_release_receipts')),
          'relations',(SELECT jsonb_agg(jsonb_build_object('name',c.relname,'oid',c.oid,'owner',c.relowner,'acl',c.relacl,
            'columns',(SELECT jsonb_agg(to_jsonb(a) ORDER BY a.attnum) FROM pg_catalog.pg_attribute a WHERE a.attrelid=c.oid),
            'constraints',(SELECT COALESCE(jsonb_agg(to_jsonb(k) ORDER BY k.oid),'[]'::jsonb) FROM pg_catalog.pg_constraint k WHERE k.conrelid=c.oid)) ORDER BY c.relname)
            FROM pg_catalog.pg_class c WHERE c.relnamespace='public'::regnamespace AND c.relname IN
              ('accounts','account_security','account_security_events','account_terms_acceptances','account_terms_head','account_terms_release_receipts')))
        "#).fetch_one(pool).await.expect("complete custody row/routine metadata snapshot")
    }

    async fn historical_upgrade(pool: PgPool, role: AppRole) {
        prepare_http_database_staging(&pool).await;
        let historical = include_str!("fixtures/account-custody-projection-v2-69e3ca9c.sql");
        assert_eq!(
            hex::encode(Sha256::digest(historical.as_bytes())),
            "eea1ef091ebdacf3fe537bee293a0e0eedf077ab0033bb87f37d617fe6e75c5c",
            "historical operator SQL must equal the immutable69e3ca9c Git blob"
        );
        sqlx::raw_sql(historical)
            .execute(&pool)
            .await
            .expect("execute actual historical projection finalizer");
        seed_snapshot_rows(&pool).await;
        let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
        assert_eq!(
            sqlx::query_scalar::<_, i32>("SELECT 1")
                .fetch_one(&runtime)
                .await
                .unwrap(),
            1
        );
        let before = complete_state(&pool).await;
        assert_eq!(before["functions"].as_array().unwrap().len(), 1);
        assert!(before["functions"][0]["proname"] == "account_legacy_fenced_v1");
        assert_eq!(before["rows"]["receipts"].as_array().unwrap().len(), 1);
        let attempt = AppState::from_config(config(&pool, role)).await;
        if let Ok(state) = &attempt {
            state.shutdown_realtime().await;
        }
        assert!(
            before == complete_state(&pool).await,
            "startup admission must not rewrite historical rows or metadata"
        );
        assert!(
            matches!(attempt, Err(console_app::AppError::Config(ref code)) if code == "account_custody.upgrade_required"),
            "HISTORICAL_PROFILE_ADMISSION: exact old finalized profile must refuse current startup with upgrade_required"
        );

        let expected_roots =
            super::super::account_root_transition::expected_roots_after_backfill(&pool).await;
        // Same real operator API, no runtime mode and no test-built verifier.
        finalize_account_custody(&pool).await;
        let upgraded = complete_state(&pool).await;
        let mut expected_rows = before["rows"].clone();
        let mut roots = expected_roots.as_array().unwrap().clone();
        roots.sort_by_key(|row| row["id"].as_str().unwrap().to_owned());
        expected_rows["accounts"] = Value::Array(roots);
        assert!(
            expected_rows == upgraded["rows"],
            "operator upgrade preserves every old row and adds only exact missing legacy roots"
        );
        for original in before["functions"].as_array().unwrap() {
            assert!(
                upgraded["functions"].as_array().unwrap().contains(original),
                "upgrade preserves the existing projection routine exactly"
            );
        }
        assert_eq!(upgraded["functions"].as_array().unwrap().len(), 2);
        let state = AppState::from_config(config(&pool, role))
            .await
            .expect("same transport starts after real operator upgrade");
        let router = build_router(state.clone());
        let ready = retained_ready(&router).await;
        state.shutdown_realtime().await;
        runtime.close().await;
        assert_eq!(ready, StatusCode::OK);
        assert!(
            upgraded == complete_state(&pool).await,
            "successful startup/readiness remains read-only after upgrade"
        );
    }

    async fn routine_drift_recovers(pool: PgPool, role: AppRole) {
        prepare_http_database_staging(&pool).await;
        finalize_account_custody(&pool).await;
        seed_snapshot_rows(&pool).await;
        let state = AppState::from_config(config(&pool, role))
            .await
            .expect("positive startup through actual current finalizer");
        let router = build_router(state.clone());
        assert_eq!(retained_ready(&router).await, StatusCode::OK);
        // Projection body is FIRST on the current baseline: old readiness
        // wrongly reports200. A missing future guard must not mask that RED.
        for (signature, owner, hostile_body) in [
            (
                "public.account_legacy_fenced_v1(uuid)",
                "console_account_owner",
                "BEGIN RETURN false; END;",
            ),
            (
                "public.account_terms_receipts_immutable_v1()",
                "console_terms_owner",
                "BEGIN RETURN NULL; END;",
            ),
        ] {
            for kind in ["body", "owner", "public_execute", "search_path"] {
                let before = complete_state(&pool).await;
                let routine: Value = sqlx::query_scalar("SELECT to_jsonb(p) FROM pg_catalog.pg_proc p WHERE p.oid=pg_catalog.to_regprocedure($1)")
                    .bind(signature).fetch_one(&pool).await.expect("real installed routine prerequisite; never create a fixture replacement");
                let definition: String = sqlx::query_scalar(
                    "SELECT pg_catalog.pg_get_functiondef(pg_catalog.to_regprocedure($1))",
                )
                .bind(signature)
                .fetch_one(&pool)
                .await
                .unwrap();
                let (mutation, restoration) = match kind {
                    "body" => {
                        let body = routine["prosrc"].as_str().unwrap();
                        assert_eq!(
                            definition.matches(body).count(),
                            1,
                            "body fault changes only the observed actual routine body"
                        );
                        (
                            definition.replacen(body, hostile_body, 1),
                            definition.clone(),
                        )
                    }
                    "owner" => (
                        format!("ALTER FUNCTION {signature} OWNER TO console_app"),
                        format!("ALTER FUNCTION {signature} OWNER TO {owner}"),
                    ),
                    "public_execute" => (
                        format!("GRANT EXECUTE ON FUNCTION {signature} TO PUBLIC"),
                        format!("REVOKE EXECUTE ON FUNCTION {signature} FROM PUBLIC"),
                    ),
                    "search_path" => (
                        format!("ALTER FUNCTION {signature} SET search_path=public"),
                        definition.clone(),
                    ),
                    _ => unreachable!(),
                };
                let outcome = std::panic::AssertUnwindSafe(async {
                    sqlx::raw_sql(sqlx::AssertSqlSafe(mutation)).execute(&pool).await.expect("committed actual routine metadata fault");
                    let tampered = complete_state(&pool).await;
                    assert!(before["functions"] != tampered["functions"], "fault must alter actual routine metadata");
                    assert!(before["rows"] == tampered["rows"], "routine-only fault must preserve all rows and approval bytes");
                    assert_eq!(retained_ready(&router).await, StatusCode::SERVICE_UNAVAILABLE,
                        "ROUTINE_READINESS: actual projection or guard metadata drift must fail retained readiness");
                    assert!(tampered == complete_state(&pool).await, "readiness must detect drift without repairing metadata or rows");
                }).catch_unwind().await;

                // Always restore the exact logical catalog state, including on
                // baseline RED; no finalizer is allowed to silently repair drift.
                let restore = sqlx::raw_sql(sqlx::AssertSqlSafe(restoration))
                    .execute(&pool)
                    .await;
                let restored = complete_state(&pool).await;
                let recovered = retained_ready(&router).await;
                if !restore.is_ok() || restored != before || recovered != StatusCode::OK {
                    state.shutdown_realtime().await;
                    panic!(
                        "ROUTINE_READINESS_CLEANUP: exact metadata/rows and retained readiness must recover"
                    );
                }
                if let Err(panic) = outcome {
                    state.shutdown_realtime().await;
                    std::panic::resume_unwind(panic);
                }
            }
        }
        // Readiness is detection only, never proof that consumers were drained.
        state.shutdown_realtime().await;
    }

    #[sqlx::test(migrations = false)]
    async fn api_historical_projection_requires_upgrade_then_starts(pool: PgPool) {
        historical_upgrade(pool, AppRole::Api).await;
    }

    #[sqlx::test(migrations = false)]
    async fn worker_historical_projection_requires_upgrade_then_starts(pool: PgPool) {
        historical_upgrade(pool, AppRole::Worker).await;
    }

    #[sqlx::test(migrations = false)]
    async fn api_projection_and_guard_drift_readiness_recovers(pool: PgPool) {
        routine_drift_recovers(pool, AppRole::Api).await;
    }

    #[sqlx::test(migrations = false)]
    async fn worker_projection_and_guard_drift_readiness_recovers(pool: PgPool) {
        routine_drift_recovers(pool, AppRole::Worker).await;
    }
}
