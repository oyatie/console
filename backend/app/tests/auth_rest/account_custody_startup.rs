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
