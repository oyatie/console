//! Genuine UI-issued credential capture; secret transport bytes stay in memory.
use super::*;
use axum::http::{HeaderValue, header};
use console_identity_adapter_postgres::PgOrgStore;
use console_platform_auth::{
    JwtIssuer, JwtSettings, JwtVerifier, account::AccountEnrollmentCredentials,
};
use console_platform_test_support::{TestDatabaseLogin, login_test_pool};
use std::sync::{Arc, Mutex};

pub(super) type CapturedCookies = Arc<Mutex<Vec<HeaderValue>>>;

pub(super) fn capture_router(router: axum::Router, captured: CapturedCookies) -> axum::Router {
    router.layer(axum::middleware::from_fn(
        move |request: axum::extract::Request, next: axum::middleware::Next| {
            let captured = captured.clone();
            async move {
                if request.method() == axum::http::Method::GET
                    && let Some(cookie) = request.headers().get(header::COOKIE)
                    && cookie.to_str().is_ok_and(|value| {
                        value
                            .split(';')
                            .any(|part| part.trim().starts_with("__Host-console_account_session="))
                    })
                {
                    let mut values = captured.lock().unwrap();
                    if !values.contains(cookie) {
                        assert!(
                            values.len() < 8,
                            "genuine browser cookie capture exceeded finite bound"
                        );
                        values.push(cookie.clone());
                    }
                }
                next.run(request).await
            }
        },
    ))
}

pub(super) fn bindings(config: &AppConfig) -> (JwtVerifier, JwtIssuer, time::Duration) {
    let auth = config.auth_rest.as_ref().unwrap();
    let settings = JwtSettings {
        issuer: auth.jwt_issuer.clone(),
        audience: auth.jwt_audience.clone(),
        access_token_ttl: time::Duration::minutes(15),
    };
    let verifier =
        JwtVerifier::from_es256_public_pem(settings.clone(), auth.jwt_public_key_pem.as_bytes())
            .unwrap();
    let issuer = JwtIssuer::from_es256_pem(
        settings,
        auth.jwt_private_key_pem.as_bytes(),
        auth.jwt_public_key_pem.as_bytes(),
    )
    .unwrap();
    (verifier, issuer, auth.refresh_family_absolute_ttl)
}

pub(super) async fn runtime(pool: &PgPool) -> PgPool {
    let runtime = login_test_pool(pool, TestDatabaseLogin::Business).await;
    let actual: (String, String, bool, bool, bool) = sqlx::query_as(
        "SELECT session_user::text,current_user::text,r.rolsuper,r.rolbypassrls,r.rolcanlogin FROM pg_catalog.pg_roles r WHERE r.rolname=session_user",
    ).fetch_one(&runtime).await.unwrap();
    assert_eq!(
        actual,
        (
            "console_rt".to_owned(),
            "console_rt".to_owned(),
            false,
            false,
            true
        ),
        "genuine restricted runtime LOGIN prerequisite"
    );
    runtime
}

pub(super) async fn credentials(
    runtime: &PgPool,
    config: &AppConfig,
    captured: &CapturedCookies,
    expected: Uuid,
) -> AccountEnrollmentCredentials {
    let values = captured.lock().unwrap().clone();
    assert!(
        !values.is_empty(),
        "real browser credential capture prerequisite"
    );
    let (verifier, _, ttl) = bindings(config);
    for cookie in values.iter().rev() {
        let access: Vec<_> = cookie
            .to_str()
            .unwrap()
            .split(';')
            .filter_map(|part| {
                let (name, value) = part.trim().split_once('=')?;
                (name == "__Host-console_account_session").then_some(value)
            })
            .collect();
        assert_eq!(access.len(), 1, "ambiguous captured session transport");
        let credentials = AccountEnrollmentCredentials::for_read(access[0]).unwrap();
        let mut tx = runtime.begin().await.unwrap();
        sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL READ COMMITTED")
            .execute(tx.as_mut())
            .await
            .unwrap();
        let namespace = credentials.session_ids_in_tx(&mut tx, &verifier, ttl).await;
        if !namespace.is_ok_and(|(account, _)| account == expected) {
            tx.rollback().await.unwrap();
            continue;
        }
        let session = credentials
            .read_session_in_tx(&mut tx, &verifier, ttl)
            .await
            .expect("genuine UI-issued current Account/family prerequisite");
        assert_eq!(session.account_id, expected);
        assert!(!session.session_id.is_nil() && session.security_generation == 1);
        tx.commit().await.unwrap();
        return credentials;
    }
    panic!("no genuine UI-issued credential for expected Account");
}

pub(super) fn store(runtime: PgPool, config: &AppConfig) -> PgOrgStore {
    let (verifier, issuer, ttl) = bindings(config);
    PgOrgStore::new(runtime).with_native_account_policy(verifier, issuer, ttl)
}

pub(super) fn pin_profile() {
    for (source, digest) in [
        (
            include_str!("../../../../ops/postgres-finalize-native-company-policy.sql"),
            "4e7fc41b1d2ed6c2155d44d43347c18815ed9e70611eb9285f21e1c590bf996a",
        ),
        (
            include_str!("../../../../ops/postgres-native-company-policy-custody-state.sql"),
            "072794defc065f8730eafeab66a111bdd2439c49b8c1d7c3052ca83a7edf1479",
        ),
    ] {
        assert_eq!(
            hex::encode(Sha256::digest(source.as_bytes())),
            digest,
            "current-policy fixture source mismatch is not semantic RED"
        );
    }
    assert_eq!(
        include_str!("../../../../ops/postgres-native-company-policy-custody-state.sql"),
        include_str!("../../src/native_company_policy_custody_state.sql")
    );
}
