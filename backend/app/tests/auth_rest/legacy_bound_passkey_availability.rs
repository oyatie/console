//! Child of the reviewed legacy_bound_passkey_reads module.
use super::*;

#[sqlx::test(migrations = false)]
async fn absent_auth_pool_keeps_pure_invalid_token_refusal_and_valid_unavailability(pool: PgPool) {
    use console_platform_auth_rest::{AuthRestConfig, AuthRestState};
    let f = fixture(&pool).await;
    let business = login_test_pool(&pool, TestDatabaseLogin::Business).await;
    let key = &f.legacy.signing_key;
    let state = AuthRestState::new(
        business.clone(),
        AuthRestConfig {
            rp_id: "example.com".to_owned(),
            rp_origin: TEST_ORIGIN.to_owned(),
            rp_name: "Console".to_owned(),
            ceremony_ttl: Duration::minutes(5),
            jwt_issuer: TEST_ISSUER.to_owned(),
            jwt_audience: TEST_AUDIENCE.to_owned(),
            jwt_private_key_pem: key.to_pkcs8_pem(LineEnding::LF).unwrap().to_string(),
            jwt_public_key_pem: key
                .verifying_key()
                .to_public_key_pem(LineEnding::LF)
                .unwrap(),
            refresh_token_ttl: Duration::days(30),
            refresh_family_absolute_ttl: Duration::days(1),
            cookie_secure: false,
        },
    )
    .unwrap();
    assert!(state.auth_database().is_none());
    let absent = console_platform_auth_rest::router(state);
    let disabled = console_platform_auth_rest::router(AuthRestState::disabled(business.clone()));
    let before = all_rows(&pool).await;
    let mut invalid_secrets = f.secrets();
    invalid_secrets.push("invalid-reader-token".to_owned());
    let invalid = response_json(
        get_legacy_raw(&absent, PATH, "invalid-reader-token").await,
        StatusCode::UNAUTHORIZED,
        &invalid_secrets,
        true,
    )
    .await;
    assert!(invalid == error_body("unauthorized", "invalid bearer token"));
    let body = response_json(
        get_legacy_raw(&absent, PATH, &f.access_a).await,
        StatusCode::SERVICE_UNAVAILABLE,
        &f.secrets(),
        true,
    )
    .await;
    assert!(body == error_body("service_unavailable", "session verification unavailable"));
    // No verifier exists when services are unconfigured, so preserve the exact
    // preexisting generic503 contract even for an invalid presented credential.
    let mut secrets = f.secrets();
    secrets.push("invalid-reader-token".to_owned());
    let body = response_json(
        get_legacy_raw(&disabled, PATH, "invalid-reader-token").await,
        StatusCode::SERVICE_UNAVAILABLE,
        &secrets,
        true,
    )
    .await;
    assert!(
        body == error_body(
            "service_unavailable",
            "auth REST is mounted but auth services are not configured"
        )
    );
    let recovered = response_json(
        get_legacy_raw(&f.legacy.router, PATH, &f.access_a).await,
        StatusCode::OK,
        &f.secrets(),
        true,
    )
    .await;
    assert!(recovered == f.expected);
    assert!(before == all_rows(&pool).await);
    business.close().await;
    f.auth.close().await;
}
