//! Actual configured absolute-family lifetime and independently progressing source.
use super::*;
use console_platform_auth::guard_legacy_subject_in_tx;

pub(in super::super::super) async fn configured_family_state(
    pool: &PgPool,
    f: &Fixture,
    ttl: i64,
) -> AppState {
    let mut pairs = vec![
        (
            "CONSOLE_DATABASE_DURABILITY",
            r#"{"mode":"local_development"}"#.to_owned(),
        ),
        ("CONSOLE_APP_ROLE", AppRole::Api.to_string()),
        ("CONSOLE_HTTP_ADDR", "127.0.0.1:0".to_owned()),
        ("CONSOLE_JWT_ISSUER", TEST_ISSUER.to_owned()),
        ("CONSOLE_JWT_AUDIENCE", TEST_AUDIENCE.to_owned()),
        (
            "CONSOLE_JWT_PRIVATE_KEY_PEM",
            f.key.to_pkcs8_pem(LineEnding::LF).unwrap().to_string(),
        ),
        (
            "CONSOLE_JWT_PUBLIC_KEY_PEM",
            f.key
                .verifying_key()
                .to_public_key_pem(LineEnding::LF)
                .unwrap(),
        ),
        ("CONSOLE_WEBAUTHN_RP_ID", "example.com".to_owned()),
        ("CONSOLE_WEBAUTHN_RP_ORIGIN", TEST_ORIGIN.to_owned()),
        ("CONSOLE_WEBAUTHN_RP_NAME", "Console".to_owned()),
        ("CONSOLE_REFRESH_FAMILY_ABSOLUTE_TTL_SECS", ttl.to_string()),
        ("CONSOLE_REQUEST_TIMEOUT_SECS", "30".to_owned()),
    ];
    pairs.extend(account_transport_urls(pool));
    let config = AppConfig::from_pairs(pairs).unwrap();
    assert!(
        config
            .auth_rest
            .as_ref()
            .unwrap()
            .refresh_family_absolute_ttl
            == Duration::seconds(ttl)
            && config.request_timeout == std::time::Duration::from_secs(30)
    );
    AppState::from_config(config).await.unwrap()
}
#[sqlx::test(migrations = false)]
async fn configured_list_absolute_family_expiry_during_source_wait_denies_then_recovers(
    pool: PgPool,
) {
    let mut f = fixture(&pool).await;
    let auth = login_test_pool(&pool, TestDatabaseLogin::Auth).await;
    let narrow = configured_family_state(&pool, &f, 10).await;
    let normal = f.router.clone();
    f.router = build_router(narrow.clone());
    // Build/qualify the narrow App before issuance; stored family time comes from real DB.
    let (_, access, created) = issue_bound(&pool, &f, &auth).await;
    let deadline = created + Duration::seconds(10);
    http_token(&pool, &f, &access, StatusCode::OK).await;
    let claims = f.verifier.verify_access_token(&access).unwrap();
    assert!(
        OffsetDateTime::from_unix_timestamp(claims.exp).unwrap() > deadline + Duration::seconds(30),
        "signed token must outlive the configured family deadline"
    );
    let before = all_rows(&pool).await;
    let mut observer = pool.acquire().await.unwrap();
    let mut blocker = auth.begin().await.unwrap();
    let pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *blocker)
        .await
        .unwrap();
    guard_legacy_subject_in_tx(&mut blocker, OrgId::platform(), *f.actor.as_uuid())
        .await
        .unwrap();
    let reader = request_task(&f.router, &access, Method::GET);
    let waiter = role_wait(
        &pool,
        pid,
        "console_rt",
        "auth_legacy_platform_source_material_v1",
    )
    .await;
    let admitted_before_deadline = db_now(&pool).await < deadline;
    let pending = !reader.is_finished();
    let passed = elapsed(&pool, deadline).await;
    blocker.rollback().await.unwrap();
    let response = finish(reader).await;
    let pids: Vec<_> = std::iter::once(pid).chain(waiter).collect();
    let clean = clean_pids(&mut observer, &pids).await;
    let after = all_rows(&pool).await;
    assert_http_result(&f, &access, response.unwrap(), StatusCode::UNAUTHORIZED).await;
    assert!(
        waited_cleanly(pid, waiter, pending, clean) && admitted_before_deadline && passed,
        "real owner wait starts before and ends after configured absolute family deadline"
    );
    assert!(
        before == after,
        "family expiry leaves every public table byte unchanged"
    );
    // Same family/token succeeds under original server policy; historical absence
    // succeeds under narrow policy, distinguishing expiry from broken projection.
    http_token(&pool, &f, &f.access, StatusCode::OK).await;
    f.router = normal;
    http_token(&pool, &f, &access, StatusCode::OK).await;
    narrow.shutdown_realtime().await;
    auth.close().await;
    close(f).await;
}

pub(in super::super::super) async fn other_operator(
    pool: &PgPool,
    f: &Fixture,
) -> (UserId, String) {
    let actor = UserId::new();
    sqlx::query("INSERT INTO public.users(id,display_name,roles,org_id) VALUES($1,'독립 플랫폼 운영 담당자',ARRAY['SUPER_ADMIN']::text[],$2)").bind(actor.as_uuid()).bind(OrgId::platform().as_uuid()).execute(pool).await.unwrap();
    let issued = BootstrapCredentialStore
        .issue_for_zero_credential_user(
            &f.business,
            *actor.as_uuid(),
            OrgId::platform(),
            db_now(pool).await,
            Duration::hours(1),
        )
        .await
        .unwrap();
    let redeemed: OtpRedeemResponse = post_json(
        f.router.clone(),
        "/api/v1/auth/otp/redeem",
        None,
        json!({"otp":issued.token.as_str()}),
        StatusCode::OK,
    )
    .await;
    let mut authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let credential = enroll_passkey(&f.router, &mut authenticator, &redeemed.access_token).await;
    let mut login = usernameless_login(&f.router, &mut authenticator, &credential).await;
    // Historical-shape successor: preserve all verified claims and original
    // expiry; only remove the newly produced additive binding in test memory.
    login.access_token = crate::legacy_platform_binding_producer::historical_access(
        &f.key,
        &f.verifier,
        &login.access_token,
    );
    let verified = f.verifier.verify_access_token(&login.access_token).unwrap();
    assert!(
        verified.sub == actor.to_string()
            && verified.org == OrgId::platform().to_string()
            && verified.platform
            && verified.legacy_session.is_none(),
        "separate genuine operator login"
    );
    (actor, login.access_token)
}
#[sqlx::test(migrations = false)]
async fn unrelated_operator_completes_while_one_list_source_is_blocked(pool: PgPool) {
    let f = fixture(&pool).await;
    let auth = login_test_pool(&pool, TestDatabaseLogin::Auth).await;
    let (other, other_access) = other_operator(&pool, &f).await;
    assert!(other != f.actor);
    positive(&pool, &f, &f.router).await;
    let mut observer = pool.acquire().await.unwrap();
    let mut blocker = auth.begin().await.unwrap();
    let pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *blocker)
        .await
        .unwrap();
    guard_legacy_subject_in_tx(&mut blocker, OrgId::platform(), *f.actor.as_uuid())
        .await
        .unwrap();
    let before = all_rows(&pool).await;
    let start = db_now(&pool).await;
    let reader = request_task(&f.router, &f.access, Method::GET);
    let waiter = role_wait(
        &pool,
        pid,
        "console_rt",
        "auth_legacy_platform_source_material_v1",
    )
    .await;
    let other_start = db_now(&pool).await;
    let control = finish(request_task(&f.router, &other_access, Method::GET)).await;
    let other_end = db_now(&pool).await;
    let during = all_rows(&pool).await;
    let still_waiting = if let Some(waiter) = waiter {
        sqlx::query_scalar::<_,bool>("SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_stat_activity WHERE pid=$1 AND wait_event_type='Lock' AND $2=ANY(pg_catalog.pg_blocking_pids(pid)) AND strpos(query,'auth_legacy_platform_source_material_v1')>0)").bind(waiter).bind(pid).fetch_one(&mut *observer).await.unwrap()
    } else {
        false
    };
    let pending = !reader.is_finished();
    blocker.rollback().await.unwrap();
    let response = finish(reader).await;
    let pids: Vec<_> = std::iter::once(pid).chain(waiter).collect();
    let clean = clean_pids(&mut observer, &pids).await;
    let end = db_now(&pool).await;
    let after = all_rows(&pool).await;
    assert_http_result(&f, &other_access, control.unwrap(), StatusCode::OK).await;
    assert_http_result(&f, &f.access, response.unwrap(), StatusCode::OK).await;
    assert!(
        waited_cleanly(pid, waiter, pending, clean) && still_waiting,
        "other source must complete while first owner remains actually blocked"
    );
    assert!(
        exact_read_delta(
            &before,
            &during,
            other,
            f.expected.as_array().unwrap().len(),
            other_start,
            other_end
        ),
        "independent source completes one exact owned read while blocker remains"
    );
    assert!(
        exact_read_delta(
            &during,
            &after,
            f.actor,
            f.expected.as_array().unwrap().len(),
            start,
            end
        ),
        "released source resumes once without affecting completed independent work"
    );
    positive(&pool, &f, &f.router).await;
    auth.close().await;
    close(f).await;
}
