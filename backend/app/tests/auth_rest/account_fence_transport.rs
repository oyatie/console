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

use console_platform_auth::{RefreshTokenStore, RefreshTokenUseError};
use sha2::{Digest, Sha256};

async fn refresh_complete_snapshot(pool: &PgPool) -> Value {
    // Whole isolated-fixture rosters catch wrong-subject and misattributed writes.
    // Preserve terminal metadata too; never print this credential-bearing value.
    sqlx::query_scalar(
        r#"SELECT jsonb_build_object(
            'families', COALESCE((SELECT jsonb_agg(to_jsonb(f) ORDER BY f.id)
                FROM public.auth_refresh_token_families f),'[]'::jsonb),
            'tokens', COALESCE((SELECT jsonb_agg(to_jsonb(t) ||
                jsonb_build_object('token_hash', encode(t.token_hash,'hex')) ORDER BY t.id)
                FROM public.auth_refresh_tokens t),'[]'::jsonb),
            'audit', COALESCE((SELECT jsonb_agg(to_jsonb(a) ORDER BY a.id)
                FROM public.audit_events a),'[]'::jsonb))"#,
    )
    .fetch_one(pool)
    .await
    .expect("complete refresh and audit readback must succeed")
}

fn assert_exact_refresh_delta(
    before: &Value,
    after: &Value,
    subject: UserId,
    presented: &str,
    replacement: &str,
) {
    assert!(!presented.is_empty() && !replacement.is_empty());
    assert!(
        presented != replacement,
        "rotation must replace token bytes"
    );
    let before_tokens = before["tokens"].as_array().unwrap();
    let after_tokens = after["tokens"].as_array().unwrap();
    let presented_hash = hex::encode(Sha256::digest(presented.as_bytes()));
    let replacement_hash = hex::encode(Sha256::digest(replacement.as_bytes()));
    let original = before_tokens
        .iter()
        .find(|row| row["token_hash"] == presented_hash)
        .expect("positive token must correlate to an existing stored hash");
    assert!(original["user_id"] == json!(subject));
    assert!(original["used_at"].is_null() && original["replaced_by"].is_null());
    assert!(original["revoked_at"].is_null());
    assert_eq!(after_tokens.len(), before_tokens.len() + 1);
    let added: Vec<_> = after_tokens
        .iter()
        .filter(|row| !before_tokens.iter().any(|old| old["id"] == row["id"]))
        .collect();
    assert_eq!(added.len(), 1, "exactly one replacement identity");
    let replacement_row = added[0];
    assert!(replacement_row["token_hash"] == replacement_hash);
    assert!(replacement_row["user_id"] == original["user_id"]);
    assert!(replacement_row["family_id"] == original["family_id"]);
    assert!(replacement_row["org_id"] == original["org_id"]);
    assert!(replacement_row["used_at"].is_null());
    assert!(replacement_row["replaced_by"].is_null());
    assert!(replacement_row["revoked_at"].is_null());
    assert!(replacement_row["reuse_detected_at"].is_null());
    assert!(
        before["families"] == after["families"],
        "family roster preserved"
    );
    for old in before_tokens {
        let actual = after_tokens
            .iter()
            .find(|row| row["id"] == old["id"])
            .expect("every previous token identity must remain");
        let mut expected = old.clone();
        if old["id"] == original["id"] {
            expected["used_at"] = replacement_row["issued_at"].clone();
            expected["replaced_by"] = replacement_row["id"].clone();
        }
        assert!(
            &expected == actual,
            "only presented token consumption may change"
        );
    }
    let before_audit = before["audit"].as_array().unwrap();
    let after_audit = after["audit"].as_array().unwrap();
    assert_eq!(after_audit.len(), before_audit.len() + 1);
    for old in before_audit {
        assert!(
            after_audit.contains(old),
            "existing audit material must remain exact"
        );
    }
    let added_audit: Vec<_> = after_audit
        .iter()
        .filter(|row| !before_audit.iter().any(|old| old["id"] == row["id"]))
        .collect();
    assert_eq!(
        added_audit.len(),
        1,
        "rotation emits exactly one audit identity"
    );
    let audit = added_audit[0];
    assert!(audit["action"] == "auth.refresh");
    assert!(audit["actor"] == json!(subject));
    assert!(audit["org_id"] == original["org_id"]);
    assert!(audit["target_type"] == "auth_refresh_token_family");
    assert!(audit["target_id"] == original["family_id"]);
    assert!(audit["after_snap"]["used_token_id"] == original["id"]);
    assert!(audit["after_snap"]["replacement_token_id"] == replacement_row["id"]);
}

async fn control_refresh_token(pool: &PgPool, fixture: &LegacyFenceFixture) -> String {
    // Seed through the actual current owner, then prove HTTP rotation succeeds.
    // This also calibrates the complete snapshot against a real one-effect write.
    let business =
        console_platform_test_support::login_test_pool(pool, TestDatabaseLogin::Business).await;
    let issued = RefreshTokenStore
        .issue_family(
            &business,
            *fixture.control.as_uuid(),
            OrgId::knl(),
            OffsetDateTime::now_utc(),
            Duration::days(30),
        )
        .await
        .expect("real control-subject token issuance prerequisite");
    business.close().await;
    assert_projection(pool, fixture.subject, false).await;
    assert_projection(pool, fixture.control, false).await;
    let before = refresh_complete_snapshot(pool).await;
    let response = post_raw(
        fixture.router.clone(),
        "/api/v1/auth/token/refresh",
        None,
        json!({"refresh_token": issued.token.as_str()}),
    )
    .await;
    let body: TokenPairResponse = response.into_json(StatusCode::OK).await;
    let replacement = body
        .refresh_token
        .expect("body-mode positive returns refresh token");
    assert_exact_refresh_delta(
        &before,
        &refresh_complete_snapshot(pool).await,
        fixture.control,
        issued.token.as_str(),
        &replacement,
    );
    assert_legacy_reads(&fixture.router, fixture.control, &body.access_token).await;
    replacement
}

#[sqlx::test(migrations = false)]
async fn account_fence_refresh_canonical_writer_refuses_fenced_subject_and_rotates_control(
    pool: PgPool,
) {
    let fixture = legacy_fence_fixture(&pool).await;
    let control = control_refresh_token(&pool, &fixture).await;
    let business =
        console_platform_test_support::login_test_pool(&pool, TestDatabaseLogin::Business).await;
    let auth = console_platform_test_support::login_test_pool(&pool, TestDatabaseLogin::Auth).await;
    insert_account_fence(&pool, fixture.subject, "ACTIVE").await;
    for (subject, expected) in [(fixture.subject, true), (fixture.control, false)] {
        let fenced: bool = sqlx::query_scalar("SELECT public.account_legacy_fenced_v1($1)")
            .bind(subject.as_uuid())
            .fetch_one(&auth)
            .await
            .expect("direct restricted projection prerequisite");
        assert_eq!(fenced, expected);
    }
    let before_control = refresh_complete_snapshot(&pool).await;
    let rotated = RefreshTokenStore
        .rotate(
            &business,
            &auth,
            &control,
            OffsetDateTime::now_utc(),
            Duration::days(30),
            Duration::days(30),
        )
        .await
        .expect("unfenced canonical writer must still rotate after another subject is fenced");
    assert!(rotated.user_id == *fixture.control.as_uuid());
    assert_exact_refresh_delta(
        &before_control,
        &refresh_complete_snapshot(&pool).await,
        fixture.control,
        &control,
        rotated.token.as_str(),
    );
    for _ in 0..2 {
        let before = refresh_complete_snapshot(&pool).await;
        let denied = RefreshTokenStore
            .rotate(
                &business,
                &auth,
                &fixture.body_refresh,
                OffsetDateTime::now_utc(),
                Duration::days(30),
                Duration::days(30),
            )
            .await;
        assert!(
            before == refresh_complete_snapshot(&pool).await,
            "FENCE_CANONICAL_PREWRITE: direct owner changed refresh or audit material"
        );
        assert!(
            matches!(denied, Err(RefreshTokenUseError::InvalidToken)),
            "canonical owner must refuse a committed fence with existing InvalidToken"
        );
    }
    auth.close().await;
    business.close().await;
}

#[sqlx::test(migrations = false)]
async fn account_fence_refresh_auth_only_outage_preserves_token_and_audit_then_recovers(
    pool: PgPool,
) {
    let fixture = legacy_fence_fixture(&pool).await;
    let control = control_refresh_token(&pool, &fixture).await;
    let auth_url = login_test_database_url(&pool, TestDatabaseLogin::Auth);
    let auth_password = Url::parse(&auth_url)
        .unwrap()
        .password()
        .unwrap()
        .to_owned();
    let original_role: Value = sqlx::query_scalar(
        "SELECT to_jsonb(r) FROM pg_catalog.pg_roles r WHERE rolname='console_auth_rt'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(original_role["rolcanlogin"], true);
    let before = refresh_complete_snapshot(&pool).await;
    // The effective body token is the real fixture's unspent replacement.
    // Prove its hash, subject and live family before introducing any fault.
    let live_token: bool = sqlx::query_scalar(
        "SELECT t.user_id=$2 AND t.used_at IS NULL AND t.revoked_at IS NULL \
         AND t.replaced_by IS NULL AND t.expires_at>now() AND f.revoked_at IS NULL \
         FROM public.auth_refresh_tokens t JOIN public.auth_refresh_token_families f \
         ON f.id=t.family_id WHERE t.token_hash=$1",
    )
    .bind(Sha256::digest(fixture.body_refresh.as_bytes()).to_vec())
    .bind(fixture.subject.as_uuid())
    .fetch_one(&pool)
    .await
    .expect("effective token must exist before auth outage");
    assert!(
        live_token,
        "effective token must be unused and live before auth outage"
    );
    let outcome = std::panic::AssertUnwindSafe(async {
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
            "real router must retain authenticated auth transport"
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
            "auth fault requires exclusive fixture custody"
        );
        // Root-owned disposable cluster only; no passwords, grants or schema change.
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
            "bounded termination must stop actual retained auth backends"
        );
        let remaining: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM pg_catalog.pg_stat_activity \
             WHERE datname=current_database() AND usename='console_auth_rt'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(remaining, 0, "terminated auth backends must be gone");
        let direct = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .acquire_timeout(std::time::Duration::from_secs(3))
            .connect(&auth_url)
            .await;
        let auth_error = match direct {
            Err(error) => error,
            Ok(unexpected) => {
                unexpected.close().await;
                panic!("REFRESH_OUTAGE_FAULT: disabled auth LOGIN still authenticated");
            }
        };
        assert!(
            auth_error
                .as_database_error()
                .and_then(|error| error.code())
                .as_deref()
                == Some("28000"),
            "fault witness must be PostgreSQL NOLOGIN, not generic infrastructure failure"
        );
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
                let mut tx = unaffected.begin().await.unwrap();
                sqlx::query("SELECT set_config('app.current_org', $1, true)")
                    .bind(OrgId::knl().as_uuid().to_string())
                    .execute(&mut *tx)
                    .await
                    .unwrap();
                let available: i64 = sqlx::query_scalar(
                    "SELECT count(*) FROM public.auth_refresh_tokens WHERE user_id=$1 \
                     AND used_at IS NULL AND revoked_at IS NULL",
                )
                .bind(fixture.subject.as_uuid())
                .fetch_one(&mut *tx)
                .await
                .unwrap();
                assert!(
                    available >= 2,
                    "business can still read real unused subject tokens"
                );
                tx.rollback().await.unwrap();
            }
            unaffected.close().await;
        }
        let response = refresh_response(&fixture, false).await;
        assert!(
            before == refresh_complete_snapshot(&pool).await,
            "FENCE_OUTAGE_PREWRITE: indeterminate auth consumed token or changed audit"
        );
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert!(
            set_cookie_values(&response).is_empty(),
            "storage error must not issue cookies"
        );
        let secrets = [
            fixture.access.as_str(),
            fixture.body_refresh.as_str(),
            fixture.cookie_refresh.as_str(),
            control.as_str(),
            auth_url.as_str(),
            auth_password.as_str(),
        ];
        for value in response.headers().values() {
            let text = value.to_str().expect("valid response header");
            assert!(
                secrets.iter().all(|secret| !text.contains(secret)),
                "storage failure headers cannot echo known credentials"
            );
        }
        let bytes = to_bytes(response.into_body(), 64 * 1024).await.unwrap();
        let text = std::str::from_utf8(&bytes).unwrap();
        assert!(
            secrets.iter().all(|secret| !text.contains(secret)),
            "storage failure body cannot echo known credentials"
        );
        let body: Value = serde_json::from_slice(&bytes).unwrap();
        assert!(
            body == json!({"error":{"code":"internal","message":"refresh token storage error"}}),
            "existing constant Storage error only; no raw driver detail or token fields"
        );
    })
    .catch_unwind()
    .await;
    // Outside caught unwinding: restore the proven original LOGIN flag and
    // compare every public role attribute. Driver owns abort/kill cleanup.
    let restore = sqlx::query("ALTER ROLE console_auth_rt LOGIN")
        .execute(&pool)
        .await;
    let restored: Result<Value, _> = sqlx::query_scalar(
        "SELECT to_jsonb(r) FROM pg_catalog.pg_roles r WHERE rolname='console_auth_rt'",
    )
    .fetch_one(&pool)
    .await;
    assert!(
        restore.is_ok() && restored.as_ref().ok() == Some(&original_role),
        "REFRESH_OUTAGE_CLEANUP: original auth role must be restored exactly"
    );
    if let Err(panic) = outcome {
        std::panic::resume_unwind(panic);
    }
    assert_projection(&pool, fixture.subject, false).await;
    let recovered = refresh_response(&fixture, false).await;
    let body: TokenPairResponse = recovered.into_json(StatusCode::OK).await;
    let replacement = body
        .refresh_token
        .expect("same body token must recover after restoration");
    assert_exact_refresh_delta(
        &before,
        &refresh_complete_snapshot(&pool).await,
        fixture.subject,
        &fixture.body_refresh,
        &replacement,
    );
    assert_legacy_reads(&fixture.router, fixture.subject, &body.access_token).await;
}

#[sqlx::test(migrations = false)]
async fn account_fence_refresh_effective_cookie_subject_controls_conflicting_body(pool: PgPool) {
    let fixture = legacy_fence_fixture(&pool).await;
    let control = control_refresh_token(&pool, &fixture).await;
    insert_account_fence(&pool, fixture.subject, "ACTIVE").await;
    assert_projection(&pool, fixture.subject, true).await;
    assert_projection(&pool, fixture.control, false).await;
    let before = refresh_complete_snapshot(&pool).await;
    let denied = post_cookie_mode(
        fixture.router.clone(),
        "/api/v1/auth/token/refresh",
        Some(&fixture.cookie_refresh),
        json!({"refresh_token": control}),
    )
    .await;
    assert!(
        before == refresh_complete_snapshot(&pool).await,
        "FENCE_COOKIE_SUBJECT: fenced cookie must preserve both subjects despite valid body"
    );
    assert_legacy_denials(
        [denied],
        &[
            &fixture.access,
            &fixture.body_refresh,
            &fixture.cookie_refresh,
            &control,
        ],
    )
    .await;
    let accepted = post_cookie_mode(
        fixture.router.clone(),
        "/api/v1/auth/token/refresh",
        Some(&control),
        json!({"refresh_token": fixture.body_refresh}),
    )
    .await;
    assert_eq!(accepted.status(), StatusCode::OK);
    let cookie =
        console_refresh_set_cookie(&accepted).expect("cookie mode returns rotating cookie");
    assert!(cookie.contains("HttpOnly") && cookie.contains("SameSite=Strict"));
    let replacement = cookie_token(&cookie).to_owned();
    let body: TokenPairResponse = accepted.into_json(StatusCode::OK).await;
    assert!(
        body.refresh_token.is_none(),
        "cookie mode must not return token in JSON"
    );
    assert_exact_refresh_delta(
        &before,
        &refresh_complete_snapshot(&pool).await,
        fixture.control,
        &control,
        &replacement,
    );
    assert_legacy_reads(&fixture.router, fixture.control, &body.access_token).await;
}
