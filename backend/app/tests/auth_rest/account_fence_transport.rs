//! Test-only next-boundary candidate. Real startup, pool and legacy token owners.
//! Does not establish Account-v1 sessions, concurrent cutover or stream revocation.
use super::*;
use futures::FutureExt;

fn transport_pairs(pool: &PgPool) -> Vec<(&'static str, String)> {
    let mut pairs = account_transport_config_pairs();
    let transports = account_transport_urls(pool);
    pairs.retain(|(key, _)| !transports.iter().any(|(transport, _)| key == transport));
    pairs.extend(transports);
    pairs
}

async fn assert_projection(pool: &PgPool, subject: UserId, expected: bool) {
    let auth = console_platform_test_support::login_test_pool(pool, TestDatabaseLogin::Auth).await;
    let found: bool = sqlx::query_scalar("SELECT public.account_legacy_fenced_v1($1)")
        .bind(subject.as_uuid())
        .fetch_one(&auth)
        .await
        .expect("real admitted projection prerequisite");
    assert_eq!(found, expected);
    auth.close().await;
}

#[sqlx::test(migrations = false)]
async fn account_fence_transport_same_database_real_startup_control(pool: PgPool) {
    prepare_http_database(&pool).await;
    let subject = UserId::from_uuid(Uuid::new_v4());
    insert_account_fence(&pool, subject, "ACTIVE").await;
    assert_projection(&pool, subject, true).await;
    let config =
        AppConfig::from_pairs(transport_pairs(&pool)).expect("real transport configuration");
    let state = AppState::from_config(config).await;
    assert!(
        state.is_ok(),
        "same-database restricted transport must start"
    );
}

#[sqlx::test(migrations = false)]
async fn account_fence_transport_other_database_cannot_silently_report_absence(pool: PgPool) {
    prepare_http_database(&pool).await;
    let subject = UserId::from_uuid(Uuid::new_v4());
    insert_account_fence(&pool, subject, "ACTIVE").await;
    assert_projection(&pool, subject, true).await;
    // A valid business startup is required before the negative configuration.
    assert!(
        AppState::from_config(AppConfig::from_pairs(transport_pairs(&pool)).unwrap())
            .await
            .is_ok()
    );
    let unique = Uuid::new_v4().simple().to_string();
    let database = format!("_sqlx_test_{unique}{}", &unique[..20]);
    let outcome = std::panic::AssertUnwindSafe(async {
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "CREATE DATABASE \"{database}\""
        )))
        .execute(&pool)
        .await
        .unwrap();
        let other = sqlx::postgres::PgPoolOptions::new()
            .max_connections(4)
            .connect_with(pool.connect_options().as_ref().clone().database(&database))
            .await
            .unwrap();
        // Complete real migrations and routine admission in BOTH databases:
        // missing schema/function must not manufacture the expected rejection.
        prepare_http_database(&other).await;
        assert_projection(&other, subject, false).await;
        assert!(
            AppState::from_config(AppConfig::from_pairs(transport_pairs(&other)).unwrap())
                .await
                .is_ok()
        );
        let mut pairs = transport_pairs(&pool);
        for (key, value) in &mut pairs {
            if *key == "AUTH_DATABASE_URL" {
                *value = login_test_database_url(&other, TestDatabaseLogin::Auth);
            }
        }
        // Static endpoint refusal and actual startup binding are both acceptable;
        // this test mandates behavior, not a particular lock/identity mechanism.
        let error = match AppConfig::from_pairs(pairs) {
            Err(error) => error,
            Ok(config) => match AppState::from_config(config).await {
                Err(error) => error,
                Ok(_) => {
                    panic!("AUTH_DATABASE_BINDING: another valid database hid the persisted fence")
                }
            },
        };
        let message = error.to_string();
        assert!(
            message.contains("AUTH_DATABASE_URL"),
            "binding diagnostic must identify auth transport"
        );
        let url = Url::parse(&login_test_database_url(&other, TestDatabaseLogin::Auth)).unwrap();
        assert!(
            !message.contains(url.password().unwrap()),
            "binding errors cannot expose credentials"
        );
        other.close().await;
    })
    .catch_unwind()
    .await;
    // Pattern reused from account_custody_lifecycle: delete only the uniquely
    // owned disposable DB even when a prerequisite or assertion panics.
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "DROP DATABASE IF EXISTS \"{database}\" WITH (FORCE)"
    )))
    .execute(&pool)
    .await
    .unwrap();
    if let Err(panic) = outcome {
        std::panic::resume_unwind(panic);
    }
}

async fn refresh_issuance_snapshot(pool: &PgPool, subject: UserId) -> Value {
    // Include every existing token/family identity and immutable issuance field.
    // Terminal refusal may mark used/reuse/revocation metadata without issuing
    // a replacement; those fields are deliberately outside this oracle. No hash or token
    // bytes enter assertion output; equality is checked with a static message.
    sqlx::query_scalar(r#"SELECT jsonb_build_object(
        'families', COALESCE((SELECT jsonb_agg(jsonb_build_array(id,user_id,org_id,created_at) ORDER BY id)
            FROM auth_refresh_token_families WHERE user_id=$1),'[]'::jsonb),
        'tokens', COALESCE((SELECT jsonb_agg(jsonb_build_array(id,family_id,user_id,org_id,
            encode(token_hash,'hex'),issued_at,expires_at,replaced_by) ORDER BY id)
            FROM auth_refresh_tokens WHERE user_id=$1),'[]'::jsonb))"#)
        .bind(subject.as_uuid()).fetch_one(pool).await.unwrap()
}

async fn refresh_response(fixture: &LegacyFenceFixture, cookie: bool) -> http::Response<Body> {
    if cookie {
        post_cookie_mode(
            fixture.router.clone(),
            "/api/v1/auth/token/refresh",
            Some(&fixture.cookie_refresh),
            json!({}),
        )
        .await
    } else {
        post_raw(
            fixture.router.clone(),
            "/api/v1/auth/token/refresh",
            None,
            json!({"refresh_token":fixture.body_refresh}),
        )
        .await
    }
}

#[sqlx::test(migrations = false)]
async fn account_fence_refresh_snapshot_detects_real_rotation_both_transports(pool: PgPool) {
    let fixture = legacy_fence_fixture(&pool).await;
    for cookie in [false, true] {
        let before = refresh_issuance_snapshot(&pool, fixture.subject).await;
        let response = refresh_response(&fixture, cookie).await;
        assert_eq!(response.status(), StatusCode::OK);
        let after = refresh_issuance_snapshot(&pool, fixture.subject).await;
        assert!(
            before != after,
            "oracle must detect actual successful token rotation"
        );
        assert_eq!(
            after["tokens"].as_array().unwrap().len(),
            before["tokens"].as_array().unwrap().len() + 1
        );
        assert!(
            before["families"] == after["families"],
            "rotation retains family identity"
        );
    }
}

async fn assert_fenced_refresh_has_no_issuance(pool: &PgPool, cookie: bool) {
    let fixture = legacy_fence_fixture(pool).await;
    let before = refresh_issuance_snapshot(pool, fixture.subject).await;
    assert!(
        !before["tokens"].as_array().unwrap().is_empty(),
        "real tokens must exist"
    );
    insert_account_fence(pool, fixture.subject, "ACTIVE").await;
    assert_projection(pool, fixture.subject, true).await;
    let response = refresh_response(&fixture, cookie).await;
    let after = refresh_issuance_snapshot(pool, fixture.subject).await;
    // Assert storage first: a401 obtained only AFTER committing a replacement
    // must fail independently of an apparently safe HTTP refusal.
    assert!(
        before == after,
        "FENCE_PREWRITE: refusal created, removed, rewrote or replaced refresh issuance material"
    );
    assert_legacy_denials([response], &fixture.known_secrets()).await;
    assert_legacy_reads(&fixture.router, fixture.control, &fixture.control_access).await;
}

#[sqlx::test(migrations = false)]
async fn account_fence_refresh_body_refuses_before_issuance(pool: PgPool) {
    assert_fenced_refresh_has_no_issuance(&pool, false).await;
}

#[sqlx::test(migrations = false)]
async fn account_fence_refresh_cookie_refuses_before_issuance(pool: PgPool) {
    assert_fenced_refresh_has_no_issuance(&pool, true).await;
}

#[sqlx::test(migrations = false)]
async fn account_fence_transport_startup_consumes_configured_auth_credential(pool: PgPool) {
    prepare_http_database(&pool).await;
    let subject = UserId::from_uuid(Uuid::new_v4());
    insert_account_fence(&pool, subject, "ACTIVE").await;
    // Prove the real restricted auth credential reads the installed projection.
    // A missing role, routine or database cannot manufacture this refusal.
    assert_projection(&pool, subject, true).await;
    let mut pairs = transport_pairs(&pool);
    let original_auth_url = pairs
        .iter()
        .find(|(key, _)| *key == "AUTH_DATABASE_URL")
        .map(|(_, value)| value.as_str())
        .expect("configured original auth URL");
    let direct_auth = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(std::time::Duration::from_secs(3))
        .connect(original_auth_url)
        .await
        .expect("original configured auth credential must authenticate directly");
    direct_auth.close().await;
    let original = AppConfig::from_pairs(pairs.clone()).expect("valid configured transports");
    assert!(
        AppState::from_config(original).await.is_ok(),
        "valid business and original auth transports must start"
    );

    // Change only the password at the same target and exact auth username.
    // URL identity syntax, business credentials and database state stay valid.
    let wrong_password = format!("TEST_ONLY_wrong_auth_{}", Uuid::new_v4().simple());
    for (_, configured_url) in account_transport_urls(&pool) {
        let configured = Url::parse(&configured_url).unwrap();
        assert!(
            configured.password() != Some(wrong_password.as_str()),
            "negative credential must differ from every provisioned transport"
        );
    }
    let auth_pair = pairs
        .iter_mut()
        .find(|(key, _)| *key == "AUTH_DATABASE_URL")
        .expect("one configured auth transport");
    let mut auth_url = Url::parse(&auth_pair.1).unwrap();
    auth_url.set_password(Some(&wrong_password)).unwrap();
    auth_pair.1 = auth_url.to_string();
    let rejected_url = auth_pair.1.clone();
    let config = AppConfig::from_pairs(pairs)
        .expect("distinct wrong credential remains syntactically valid configuration");
    // Require real password authentication, not trust-authenticated fixtures or
    // a stopped database masquerading as an invalid credential refusal.
    let wrong_login = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(std::time::Duration::from_secs(3))
        .connect(&rejected_url)
        .await;
    let direct_error = match wrong_login {
        Err(error) => error,
        Ok(unexpected) => {
            unexpected.close().await;
            panic!("AUTH_FIXTURE_PASSWORD: invalid credential authenticated directly");
        }
    };
    assert!(
        direct_error
            .as_database_error()
            .and_then(|error| error.code())
            .as_deref()
            == Some("28P01"),
        "wrong auth credential must fail with PostgreSQL invalid_password"
    );
    let error = match AppState::from_config(config).await {
        Err(error) => error,
        Ok(_) => panic!("AUTH_POOL_CREDENTIAL: startup ignored the invalid auth credential"),
    };
    let message = error.to_string();
    assert!(
        message.contains("AUTH_DATABASE_URL"),
        "startup refusal must identify the auth transport"
    );
    assert!(
        !message.contains(&wrong_password) && !message.contains(&rejected_url),
        "startup refusal must not expose auth credential or connection URL"
    );
}

#[sqlx::test(migrations = false)]
async fn account_fence_transport_retains_auth_pool_and_readyz_tracks_auth_only_outage(
    pool: PgPool,
) {
    prepare_http_database(&pool).await;
    let subject = UserId::from_uuid(Uuid::new_v4());
    insert_account_fence(&pool, subject, "ACTIVE").await;
    assert_projection(&pool, subject, true).await;
    let auth_url = login_test_database_url(&pool, TestDatabaseLogin::Auth);
    tokio::time::timeout(std::time::Duration::from_secs(3), async {
        loop {
            let before_start: i64 = sqlx::query_scalar(
                "SELECT count(*) FROM pg_catalog.pg_stat_activity \
                 WHERE datname=current_database() AND usename='console_auth_rt'",
            )
            .fetch_one(&pool)
            .await
            .unwrap();
            if before_start == 0 {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        }
    })
    .await
    .expect("direct projection control must close its auth pool");
    let original_role: Value = sqlx::query_scalar(
        "SELECT to_jsonb(r) FROM pg_catalog.pg_roles r WHERE rolname='console_auth_rt'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(original_role["rolcanlogin"], true);
    let state = AppState::from_config(
        AppConfig::from_pairs(transport_pairs(&pool)).expect("valid configured transports"),
    )
    .await
    .expect("real configured startup must succeed before outage");
    let router = build_router(state.clone());
    let ready = |router: axum::Router| async move {
        router
            .oneshot(
                Request::builder()
                    .uri("/readyz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap()
            .status()
    };
    let outcome = std::panic::AssertUnwindSafe(async {
        assert_eq!(ready(router.clone()).await, StatusCode::OK);
        let retained: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM pg_catalog.pg_stat_activity \
             WHERE datname=current_database() AND usename='console_auth_rt' \
             AND backend_type='client backend'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(
            retained > 0,
            "AUTH_POOL_RETAINED: real startup must retain an authenticated auth backend"
        );
        let other_database_auth: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM pg_catalog.pg_stat_activity \
             WHERE datname<>current_database() AND usename='console_auth_rt'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            other_database_auth, 0,
            "auth role fault requires exclusive fixture custody"
        );
        // Cluster-global role fault is permitted only inside this exclusive
        // disposable SQLx harness. No credential, membership or grant changes.
        // The outer cleanup restores LOGIN even when any assertion panics.
        sqlx::query("ALTER ROLE console_auth_rt NOLOGIN")
            .execute(&pool)
            .await
            .unwrap();
        let can_login: bool = sqlx::query_scalar(
            "SELECT rolcanlogin FROM pg_catalog.pg_roles WHERE rolname='console_auth_rt'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(!can_login, "auth-only login fault must be installed");
        let terminated: Vec<(i32, bool)> = sqlx::query_as(
            "SELECT pid, pg_terminate_backend(pid, 5000) FROM pg_catalog.pg_stat_activity \
             WHERE datname=current_database() AND usename='console_auth_rt' \
             AND backend_type='client backend'",
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        assert!(
            !terminated.is_empty() && terminated.iter().all(|(_, stopped)| *stopped),
            "terminate only the retained auth backends in this disposable database"
        );
        let remaining: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM pg_catalog.pg_stat_activity \
             WHERE datname=current_database() AND usename='console_auth_rt'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            remaining, 0,
            "terminated auth backends must actually be gone"
        );
        let direct = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .acquire_timeout(std::time::Duration::from_secs(3))
            .connect(&auth_url)
            .await;
        let auth_error = match direct {
            Err(error) => error,
            Ok(unexpected) => {
                unexpected.close().await;
                panic!("AUTH_OUTAGE_FAULT: disabled auth LOGIN still authenticated");
            }
        };
        assert!(
            auth_error
                .as_database_error()
                .and_then(|error| error.code())
                .as_deref()
                == Some("28000"),
            "outage must be PostgreSQL NOLOGIN refusal, not database/network failure"
        );
        // Every other serving role must still authenticate and execute work.
        // This rules out the existing runtime/command health dependencies as
        // the reason for a503 from the actual readyz handler.
        for login in [
            TestDatabaseLogin::Business,
            TestDatabaseLogin::LeaveCommand,
            TestDatabaseLogin::OntologyCommand,
            TestDatabaseLogin::PlatformForceCommand,
        ] {
            let unaffected = console_platform_test_support::login_test_pool(&pool, login).await;
            assert_eq!(
                sqlx::query_scalar::<_, i32>("SELECT 1")
                    .fetch_one(&unaffected)
                    .await
                    .unwrap(),
                1
            );
            if matches!(login, TestDatabaseLogin::Business) {
                // Same production custody SQL and transaction settings: NOLOGIN
                // must not cause an unrelated catalog custody failure.
                let mut tx = unaffected.begin().await.unwrap();
                sqlx::query("SET TRANSACTION READ ONLY")
                    .execute(&mut *tx)
                    .await
                    .unwrap();
                sqlx::raw_sql(
                    "SET LOCAL search_path=pg_catalog,pg_temp; SET LOCAL statement_timeout='3s'",
                )
                .execute(&mut *tx)
                .await
                .unwrap();
                let verdict: String =
                    sqlx::query_scalar(include_str!("../../src/account_custody_state.sql"))
                        .fetch_one(&mut *tx)
                        .await
                        .unwrap();
                tx.commit().await.unwrap();
                assert_eq!(verdict, "account_custody.finalized");
            }
            unaffected.close().await;
        }
        assert_eq!(
            ready(router.clone()).await,
            StatusCode::SERVICE_UNAVAILABLE,
            "AUTH_READINESS: healthy business cannot hide unavailable retained auth transport"
        );
    })
    .catch_unwind()
    .await;
    // This fixed restoration is outside the unwinding body. LOGIN was proved
    // true before the fault; all other role attributes must remain identical.
    let restore = sqlx::query("ALTER ROLE console_auth_rt LOGIN")
        .execute(&pool)
        .await;
    let restored: Result<Value, _> = sqlx::query_scalar(
        "SELECT to_jsonb(r) FROM pg_catalog.pg_roles r WHERE rolname='console_auth_rt'",
    )
    .fetch_one(&pool)
    .await;
    if restore.is_err() || restored.as_ref().ok() != Some(&original_role) {
        state.shutdown_realtime().await;
        panic!("AUTH_OUTAGE_CLEANUP: exact original auth role was not restored");
    }
    if let Err(panic) = outcome {
        state.shutdown_realtime().await;
        std::panic::resume_unwind(panic);
    }
    // The same retained AppState/router must recover without reconstructing it.
    let recovered = ready(router).await;
    state.shutdown_realtime().await;
    assert_eq!(
        recovered,
        StatusCode::OK,
        "restored auth transport must recover readiness"
    );
}
