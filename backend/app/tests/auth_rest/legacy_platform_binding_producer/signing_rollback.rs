use super::*;

// Additive include INSIDE legacy_platform_binding_producer. Reuses its genuine
// direct_fixture/exact_bound_pair. No production failpoint or issuer API change.
use std::collections::BTreeSet;

// Public, intentionally unusable test key: well-formed PKCS8 EC/P256 envelope,
// ECPrivateKey version1 with zero scalar. It can never sign a genuine session.
const ZERO_SCALAR_PKCS8: &str = "-----BEGIN PRIVATE KEY-----\nMEECAQAwEwYHKoZIzj0CAQYIKoZIzj0DAQcEJzAlAgEBBCAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==\n-----END PRIVATE KEY-----\n";

async fn signing_failure_router(pool: &PgPool, f: &DirectFixture) -> (axum::Router, String) {
    use console_platform_auth::{AccessTokenInput, AuthError, JwtIssuer};
    let public = f
        .key
        .verifying_key()
        .to_public_key_pem(LineEnding::LF)
        .unwrap();
    let settings = JwtSettings {
        issuer: TEST_ISSUER.into(),
        audience: TEST_AUDIENCE.into(),
        access_token_ttl: Duration::minutes(15),
    };
    // Independent prerequisites: parser accepts the EC envelope, real ES256
    // signing rejects it. A constructor failure is NOT the rollback boundary.
    let issuer =
        JwtIssuer::from_es256_pem(settings, ZERO_SCALAR_PKCS8.as_bytes(), public.as_bytes())
            .expect("test-only EC envelope must initialize actual issuer");
    let error = issuer
        .issue_access_token(AccessTokenInput {
            subject: f.subject,
            org_id: OrgId::platform(),
            roles: vec!["SUPER_ADMIN".into()],
            branches: vec![],
            platform: true,
            view_as: false,
            read_only: false,
            display_name: None,
            feature_grants: vec![],
            authz_subject_version: 0,
            authz_policy_version: 0,
            session_generation: 0,
            issued_at: OffsetDateTime::now_utc(),
        })
        .expect_err("zero scalar unexpectedly signed ES256");
    assert!(
        matches!(&error,AuthError::Jwt(jwt) if matches!(jwt.kind(),jsonwebtoken::errors::ErrorKind::InvalidEcdsaKey)),
        "signing fixture failed outside actual P256 encoder"
    );
    let expected_error = error.to_string();
    let router = build_router(
        app_state(pool.clone(), ZERO_SCALAR_PKCS8.to_owned(), public)
            .await
            .expect("real app startup must accept parser fixture before rollback probe"),
    );
    (router, expected_error)
}

async fn install_refresh_insert_witness(pool: &PgPool, subject: UserId) {
    // Observational fixture only. No production schema/guard is changed or
    // disabled. Sequence survives transaction rollback; trigger returns NEW.
    sqlx::raw_sql(
        r#"
      CREATE SEQUENCE public.platform_signing_insert_seen;
      CREATE FUNCTION public.platform_signing_insert_seen() RETURNS trigger
      LANGUAGE plpgsql SECURITY DEFINER SET search_path=pg_catalog,pg_temp AS $$
      BEGIN
        IF NEW.user_id=TG_ARGV[0]::uuid AND NEW.org_id=TG_ARGV[1]::uuid THEN
          PERFORM nextval('public.platform_signing_insert_seen'::regclass);
        END IF;
        RETURN NEW;
      END $$;
    "#,
    )
    .execute(pool)
    .await
    .unwrap();
    let trigger = format!(
        "CREATE TRIGGER platform_signing_insert_seen AFTER INSERT ON public.auth_refresh_tokens FOR EACH ROW EXECUTE FUNCTION public.platform_signing_insert_seen('{}','{}')",
        subject,
        OrgId::platform()
    );
    sqlx::raw_sql(&trigger).execute(pool).await.unwrap();
}

async fn assert_one_refresh_insert_witness(pool: &PgPool) {
    let seen: (bool, i64) =
        sqlx::query_as("SELECT is_called,last_value FROM public.platform_signing_insert_seen")
            .fetch_one(pool)
            .await
            .unwrap();
    assert!(
        seen == (true, 1),
        "actual refresh token insert must precede signing failure; absent schema/auth/startup/witness is not rollback evidence"
    );
}

async fn remove_refresh_insert_witness(pool: &PgPool) {
    sqlx::raw_sql("DROP TRIGGER platform_signing_insert_seen ON public.auth_refresh_tokens;DROP FUNCTION public.platform_signing_insert_seen();DROP SEQUENCE public.platform_signing_insert_seen;")
        .execute(pool).await.unwrap();
}

async fn assert_exact_signing_failure(response: http::Response<Body>, expected: &str) {
    assert!(response.status() == StatusCode::INTERNAL_SERVER_ERROR);
    assert!(
        !response.headers().contains_key(header::SET_COOKIE),
        "failed signing issued credential cookie"
    );
    let body: Value = response.into_json(StatusCode::INTERNAL_SERVER_ERROR).await;
    assert!(
        body == json!({"error":{"code":"internal","message":expected}}),
        "failure did not reach selected encoder"
    );
}

#[sqlx::test(migrations = false)]
async fn platform_direct_login_signing_failure_rolls_back_real_family_and_exact_assertion_retries(
    pool: PgPool,
) {
    let mut f = direct_fixture(&pool, true, false).await;
    let (bad, error) = signing_failure_router(&pool, &f).await;
    let start: LoginStartResponse = post_json(
        bad.clone(),
        "/api/v1/auth/passkey/login/start",
        None,
        json!({}),
        StatusCode::OK,
    )
    .await;
    let assertion = f
        .authenticator
        .do_authentication(
            Url::parse(TEST_ORIGIN).unwrap(),
            inject_allow_credential(start.challenge, &f.credential),
        )
        .unwrap();
    let input = json!({"ceremony_id":start.ceremony_id,"credential":assertion});
    install_refresh_insert_witness(&pool, f.subject).await;
    let before = all_rows(&pool).await;
    let response = post_raw(
        bad,
        "/api/v1/auth/passkey/login/finish",
        None,
        input.clone(),
    )
    .await;
    assert_exact_signing_failure(response, &error).await;
    assert_one_refresh_insert_witness(&pool).await;
    assert!(
        before == all_rows(&pool).await,
        "signing failure committed family/token/ceremony/credential/audit effects"
    );
    remove_refresh_insert_witness(&pool).await;
    // Same stored ceremony and exact authenticator assertion succeed after the
    // ordinary configured signer is restored; no synthetic new credential.
    let recovered: TokenPairResponse = post_json(
        f.router.clone(),
        "/api/v1/auth/passkey/login/finish",
        None,
        input,
        StatusCode::OK,
    )
    .await;
    exact_bound_pair(
        &pool,
        &f,
        &recovered.access_token,
        recovered.refresh_token.as_deref().unwrap(),
    )
    .await;
}

#[sqlx::test(migrations = false)]
async fn platform_direct_refresh_signing_failure_rolls_back_rotation_and_original_token_retries(
    pool: PgPool,
) {
    let f = direct_fixture(&pool, true, false).await;
    let (bad, error) = signing_failure_router(&pool, &f).await;
    let original = f.login.refresh_token.as_deref().unwrap();
    let family = exact_bound_pair(&pool, &f, &f.login.access_token, original).await;
    install_refresh_insert_witness(&pool, f.subject).await;
    let before = all_rows(&pool).await;
    let start = OffsetDateTime::now_utc();
    let response = post_raw(
        bad,
        "/api/v1/auth/token/refresh",
        None,
        json!({"refresh_token":original}),
    )
    .await;
    let end = OffsetDateTime::now_utc();
    assert_exact_signing_failure(response, &error).await;
    assert_one_refresh_insert_witness(&pool).await;
    let after = all_rows(&pool).await;
    assert!(before.keys().eq(after.keys()));
    for (table, rows) in &before {
        if table != "auth_rate_limit" {
            assert!(
                rows == &after[table],
                "failed signing committed durable effect in {table}"
            );
        }
    }
    // post_raw sends no trusted client IP/device. Refresh has exactly one
    // intentional global attempt outside the owner transaction; prove it.
    let old: Vec<Value> = serde_json::from_str(&before["auth_rate_limit"]).unwrap();
    let new: Vec<Value> = serde_json::from_str(&after["auth_rate_limit"]).unwrap();
    assert!(
        !old.iter().any(|r| r["endpoint"] == "refresh"),
        "fixture unexpectedly refreshed before selected request"
    );
    assert!(new.len() == old.len() + 1 && old.iter().all(|r| new.contains(r)));
    let additions: Vec<_> = new.iter().filter(|r| !old.contains(r)).collect();
    assert!(additions.len() == 1);
    let row = additions[0];
    assert!(
        row.as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>()
            == ["attempts", "client_key", "endpoint", "window_start"]
                .into_iter()
                .collect()
    );
    assert!(row["client_key"] == "global" && row["endpoint"] == "refresh" && row["attempts"] == 1);
    let window = OffsetDateTime::parse(
        row["window_start"].as_str().unwrap(),
        &time::format_description::well_known::Rfc3339,
    )
    .unwrap()
    .unix_timestamp();
    assert!(
        window % 60 == 0
            && window >= start.unix_timestamp().div_euclid(60) * 60
            && window <= end.unix_timestamp().div_euclid(60) * 60
    );
    remove_refresh_insert_witness(&pool).await;
    let recovered = rotate(&f, original).await;
    assert!(
        exact_bound_pair(
            &pool,
            &f,
            &recovered.access_token,
            recovered.refresh_token.as_deref().unwrap()
        )
        .await
            == family
    );
    assert!(recovered.refresh_token.as_deref() != Some(original));
}
