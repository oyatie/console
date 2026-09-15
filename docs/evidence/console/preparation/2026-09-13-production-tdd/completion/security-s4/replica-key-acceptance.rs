//! CTX03 actual two-process startup and closed all-key replacement sequence.
use crate::{
    browser_auth_producer::{login, login_at},
    native_fixture::{TestResult, build_native_deployment},
    real_process_producer::Process,
};
use console_platform_auth::account_deployment as deployment;
use serde_json::{Value, json};
use sqlx::PgPool;
async fn retained(pool: &PgPool, account: uuid::Uuid) -> TestResult<Value> {
    Ok(sqlx::query_scalar("SELECT jsonb_build_object('account',(SELECT to_jsonb(a) FROM accounts a WHERE id=$1),'human',(SELECT COALESCE(jsonb_agg(to_jsonb(h) ORDER BY to_jsonb(h)::text),'[]') FROM account_human_bindings h WHERE account_id=$1),'consent',(SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY t.terms_kind,t.terms_version),'[]') FROM account_terms_acceptances t WHERE account_id=$1))").bind(account).fetch_one(pool).await?)
}
#[sqlx::test(migrations = false)]
async fn replica_key_identity_requires_homogeneity_and_replacement_revokes_old_credentials(
    pool: PgPool,
) -> TestResult {
    let mut d = build_native_deployment(pool, "replica-key", 1).await?;
    let before = retained(&d.readback, d.accounts.submitter.account_id).await?;
    let old = login(
        &d.runtime.client,
        &d.runtime.origin,
        &mut d.accounts.submitter,
    )
    .await?;
    // Real replica config copies custody/build/database/public origin. The child
    // binds port0 and reports its actual listener in a pinned startup event.
    let mut first_config = d.runtime.config.replica_config()?;
    let mut first = Process::launch(
        &first_config.binary_path,
        &first_config.binary_sha256,
        &first_config.environment_pairs()?,
    )
    .await?;
    assert_eq!(first.ready(&d.runtime.client).await?, 200);
    let keys = deployment::SigningKeyMaterial::generate_es256_test_only()?;
    let mut second_config = d.runtime.config.replica_config()?;
    second_config.replace_account_keys(&keys)?;
    let mut wrong = Process::launch(
        &second_config.binary_path,
        &second_config.binary_sha256,
        &second_config.environment_pairs()?,
    )
    .await?;
    assert_eq!(
        wrong
            .ready_status(&d.runtime.client, reqwest::StatusCode::SERVICE_UNAVAILABLE)
            .await?,
        503
    );
    let decision =
        deployment::read_process_admission(&d.runtime.config, wrong.child.id().ok_or("child pid")?)
            .await?;
    assert_eq!(decision.binary_sha256, wrong.binary_sha256);
    assert!(matches!(
        decision.refusal,
        deployment::AdmissionRefusal::AccountKeyIdentityMismatch { .. }
    ));
    assert_eq!(decision.presented_key_identity, keys.public_identity());
    assert_eq!(
        decision.required_key_identity,
        d.runtime.config.account_key_identity()
    );
    assert_ne!(
        decision.presented_key_identity,
        decision.required_key_identity
    );
    let rejected = old
        .request(
            &d.runtime.client,
            &wrong.origin,
            reqwest::Method::GET,
            "/api/v2/accounts/me",
            None,
        )
        .await?;
    assert!(!rejected.status().is_success());
    wrong.stop().await?;
    first.stop().await?;
    // Ordinary deployment owner authenticates operator, witnesses every registered
    // replica drain, revokes all families under AS1 and persists new homogeneous key
    // identity before service can resume. Test passes no Boolean drained/revoked.
    let plan =
        deployment::begin_key_replacement(&d.runtime.config, &d.operator, keys.public_identity())
            .await?;
    let drained = deployment::stop_and_drain_account_admission(&d.runtime.config, &plan).await?;
    let revoked = deployment::revoke_all_account_families(&d.runtime.config, &drained).await?;
    let installed =
        deployment::install_replacement_key_identity(&d.runtime.config, &revoked, &keys).await?;
    let config = installed.replica_config()?;
    let mut fresh = Process::launch(
        &config.binary_path,
        &config.binary_sha256,
        &config.environment_pairs()?,
    )
    .await?;
    assert_eq!(fresh.ready(&d.runtime.client).await?, 200);
    for (method, path, body) in [
        (reqwest::Method::GET, "/api/v2/accounts/me", None),
        (
            reqwest::Method::POST,
            "/api/v2/auth/token/refresh",
            Some(json!({})),
        ),
        (
            reqwest::Method::POST,
            "/api/v2/auth/logout",
            Some(json!({})),
        ),
    ] {
        let r = old
            .request(&d.runtime.client, &fresh.origin, method, path, body)
            .await?;
        assert!(matches!(r.status().as_u16(), 401 | 403));
        assert!(!r.headers().contains_key(reqwest::header::SET_COOKIE));
    }
    let current = login_at(
        &d.runtime.client,
        &fresh.origin,
        &d.runtime.origin,
        &mut d.accounts.submitter,
    )
    .await?;
    let me = current
        .request(
            &d.runtime.client,
            &fresh.origin,
            reqwest::Method::GET,
            "/api/v2/accounts/me",
            None,
        )
        .await?;
    assert_eq!(me.status(), 200);
    assert_eq!(
        retained(&d.readback, d.accounts.submitter.account_id).await?,
        before
    );
    fresh.stop().await?;
    Ok(())
}
