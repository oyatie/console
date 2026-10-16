//! Session-local TEMP permission must not change persistent custody admission.
//! Independent retained serializer proves which metadata component changed.
use super::{
    account_transport_urls, finalize_serving_account_custody, prepare_http_database_staging,
};
use axum::body::Body;
use console_app::{AppConfig, AppState, DatabaseDependency, build_router};
use console_platform_test_support::{TestDatabaseLogin, login_test_pool};
use http::{Request, StatusCode};
use serde_json::Value;
use sqlx::PgPool;
use tower::ServiceExt;

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

async fn metadata(pool: &PgPool) -> Value {
    let mut tx = pool.begin().await.unwrap();
    sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY; SET LOCAL search_path=pg_catalog,pg_temp")
        .execute(&mut *tx).await.unwrap();
    let (snapshot, _): (Value, String) = sqlx::query_as(include_str!(
        "fixtures/account-native-extension-metadata.sql"
    ))
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();
    snapshot
}

#[sqlx::test(migrations = false)]
async fn temporary_create_is_session_local_but_persistent_grants_still_close_readiness(
    pool: PgPool,
) {
    prepare_http_database_staging(&pool).await;
    finalize_serving_account_custody(&pool).await;
    let login = login_test_pool(&pool, TestDatabaseLogin::Business).await;
    let options = login.connect_options().as_ref().clone();
    login.close().await;
    let runtime = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .idle_timeout(None)
        .max_lifetime(None)
        .connect_with(options)
        .await
        .unwrap();
    let mut pairs = account_transport_urls(&pool);
    pairs.extend([
        (
            "CONSOLE_DATABASE_DURABILITY",
            r#"{"mode":"local_development"}"#.to_owned(),
        ),
        ("CONSOLE_APP_ROLE", "worker".to_owned()),
        ("CONSOLE_HTTP_ADDR", "127.0.0.1:0".to_owned()),
    ]);
    let state = AppState::new(
        AppConfig::from_pairs(pairs).unwrap(),
        DatabaseDependency::Postgres(runtime.clone()),
    )
    .unwrap();
    let before = metadata(&runtime).await;
    assert_eq!(ready(&state).await, StatusCode::OK);

    let other = login_test_pool(&pool, TestDatabaseLogin::Business).await;
    let mut other_connection = other.acquire().await.unwrap();
    sqlx::raw_sql("CREATE TEMP TABLE custody_temp_control(payload text)")
        .execute(&mut *other_connection)
        .await
        .unwrap();
    let other_temp: i64 = sqlx::query_scalar("SELECT pg_catalog.pg_my_temp_schema()::bigint")
        .fetch_one(&mut *other_connection)
        .await
        .unwrap();
    assert!(other_temp > 0);
    assert!(
        metadata(&runtime).await == before,
        "another session changed persistent custody metadata"
    );
    assert_eq!(ready(&state).await, StatusCode::OK);

    sqlx::raw_sql("CREATE TEMP TABLE custody_temp_control(payload text)")
        .execute(&runtime)
        .await
        .unwrap();
    let (pid, own_temp, temp_name, has_toast): (i32, i64, String, bool) = sqlx::query_as(
        "SELECT pg_catalog.pg_backend_pid(), n.oid::bigint, n.nspname::text, c.reltoastrelid<>0 FROM pg_catalog.pg_namespace n JOIN pg_catalog.pg_class c ON c.relnamespace=n.oid WHERE n.oid=pg_catalog.pg_my_temp_schema() AND c.relname='custody_temp_control'")
        .fetch_one(&runtime).await.unwrap();
    assert!(own_temp > 0 && own_temp != other_temp && has_toast);
    let mut after = metadata(&runtime).await;
    let effective = after["schema_create"]
        .as_array()
        .expect("actual TEMP-derived CREATE entries");
    assert_eq!(effective.len(), 3);
    for owner in [
        "console_account_owner",
        "console_credential_owner",
        "console_terms_owner",
    ] {
        assert!(effective.contains(&serde_json::json!([owner, temp_name])));
    }
    after["schema_create"] = before["schema_create"].clone();
    assert!(
        after == before,
        "TEMP changed metadata beyond session-local effective CREATE"
    );
    assert_eq!(ready(&state).await, StatusCode::OK);

    sqlx::raw_sql("CREATE SCHEMA custody_create_control")
        .execute(&pool)
        .await
        .unwrap();
    for (grant, revoke) in [
        (
            "GRANT CREATE ON SCHEMA custody_create_control TO console_account_owner",
            "REVOKE CREATE ON SCHEMA custody_create_control FROM console_account_owner",
        ),
        (
            "GRANT CREATE ON SCHEMA custody_create_control TO console_credential_owner",
            "REVOKE CREATE ON SCHEMA custody_create_control FROM console_credential_owner",
        ),
        (
            "GRANT CREATE ON SCHEMA custody_create_control TO console_terms_owner",
            "REVOKE CREATE ON SCHEMA custody_create_control FROM console_terms_owner",
        ),
        (
            "GRANT CREATE ON SCHEMA custody_create_control TO PUBLIC",
            "REVOKE CREATE ON SCHEMA custody_create_control FROM PUBLIC",
        ),
    ] {
        assert_eq!(ready(&state).await, StatusCode::OK);
        sqlx::raw_sql(grant).execute(&pool).await.unwrap();
        let effective: bool = sqlx::query_scalar("SELECT bool_or(has_schema_privilege(oid,'custody_create_control','CREATE')) FROM pg_catalog.pg_roles WHERE rolname IN ('console_account_owner','console_credential_owner','console_terms_owner')")
            .fetch_one(&runtime).await.unwrap();
        assert!(effective, "persistent CREATE fault was not injected");
        assert_eq!(ready(&state).await, StatusCode::SERVICE_UNAVAILABLE);
        sqlx::raw_sql(revoke).execute(&pool).await.unwrap();
        assert_eq!(ready(&state).await, StatusCode::OK);
    }
    assert_eq!(
        sqlx::query_scalar::<_, i32>("SELECT pg_catalog.pg_backend_pid()")
            .fetch_one(&runtime)
            .await
            .unwrap(),
        pid,
        "readiness did not reuse the session"
    );
    state.shutdown_realtime().await;
    drop(other_connection);
    other.close().await;
    runtime.close().await;
}
