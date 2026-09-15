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
