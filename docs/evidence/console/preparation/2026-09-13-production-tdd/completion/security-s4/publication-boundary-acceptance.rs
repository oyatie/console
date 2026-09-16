use crate::{
    browser_auth_producer::login,
    expanded_custody_producer::{ExpandedCustodyFixture, Result},
    native_fixture::build_native_deployment,
};
use console_platform_auth::terms_publication as publisher;
use serde_json::{Value, json};
use sqlx::PgPool;
async fn snapshot(pool: &PgPool) -> Result<Value> {
    Ok(sqlx::query_scalar("SELECT jsonb_build_object('head',(SELECT jsonb_agg(to_jsonb(h)) FROM account_terms_head h),'receipts',(SELECT jsonb_agg(to_jsonb(r) ORDER BY revision) FROM account_terms_release_receipts r))").fetch_one(pool).await?)
}
async fn publish(
    f: &ExpandedCustodyFixture,
    expected: i64,
    manifest: &[u8],
) -> Result<publisher::PublicationReceipt> {
    let a = f.approve_manifest(expected, manifest)?;
    let operator = f.authenticate(publisher::Environment::TestOnly).await?;
    Ok(publisher::publish_terms(
        &operator,
        publisher::OperatorApprovalRef {
            kind: publisher::ApprovalKind::OperatorReleaseApproval,
            approval_sha256: a.approval_digest,
        },
    )
    .await?)
}
#[sqlx::test(migrations = false)]
async fn publication_eight_registered_kinds_and_exact_wrapper_bytes_are_real_positive_controls(
    pool: PgPool,
) -> Result {
    super::prepare_http_database(&pool).await;
    let kinds = (0..8)
        .map(|n| format!("test.account.kind{n}"))
        .collect::<Vec<_>>();
    let f = ExpandedCustodyFixture::new(
        "eight",
        pool.connect_options().get_database().ok_or("db")?,
        &kinds,
    )?;
    let mut exact = f.manifest.clone();
    assert!(exact.len() < 16384);
    exact.resize(16384, b' ');
    assert_eq!(
        serde_json::from_slice::<Value>(&exact)?,
        serde_json::from_slice::<Value>(&f.manifest)?
    );
    let receipt = publish(&f, 0, &exact).await?;
    assert_eq!(receipt.revision, 1);
    let before = snapshot(&pool).await?;
    let mut over = exact.clone();
    over.push(b' ');
    assert!(
        matches!(publish(&f,1,&over).await,Err(e) if e.downcast_ref::<publisher::PublicationError>().is_some_and(|e|matches!(e,publisher::PublicationError::ManifestTooLarge)))
    );
    assert_eq!(snapshot(&pool).await?, before);
    let mut too_many: Value = serde_json::from_slice(&f.manifest)?;
    too_many["items"].as_array_mut().unwrap().push(json!({"terms_kind":"test.account.kind8","title":"TEST_ONLY ninth","locale":"en-US","content_sha256":f.content_digest,"required":true}));
    assert!(
        publish(&f, 1, &serde_json::to_vec(&too_many)?)
            .await
            .is_err()
    );
    assert_eq!(snapshot(&pool).await?, before);
    Ok(())
}
#[sqlx::test(migrations = false)]
async fn publisher_unknown_operator_challenge_replay_missing_file_and_public_json_refuse(
    pool: PgPool,
) -> Result {
    let mut d = build_native_deployment(pool, "publisher-public-refusal", 1).await?;
    let f = ExpandedCustodyFixture::new(
        "operator-negatives",
        d.readback.connect_options().get_database().ok_or("db")?,
        &["test.account.service".into()],
    )?;
    let custody =
        publisher::load_deployment_custody(&f.config_path, publisher::Environment::TestOnly)
            .await?;
    assert!(
        publisher::begin_operator_authentication(&custody, "unknown.operator")
            .await
            .is_err()
    );
    let challenge = publisher::begin_operator_authentication(&custody, "test.operator").await?;
    let replay = challenge.clone();
    let proof = f.sign_operator_challenge(challenge.signing_bytes())?;
    publisher::finish_operator_authentication(custody.clone(), challenge, &proof).await?;
    assert!(matches!(
        publisher::finish_operator_authentication(custody, replay, &proof).await,
        Err(publisher::PublicationError::OperatorChallengeConsumed)
    ));
    let head: i64 = sqlx::query_scalar("SELECT revision FROM account_terms_head WHERE id=1")
        .fetch_one(&d.readback)
        .await?;
    let a = f.approve(head)?;
    let before = snapshot(&d.readback).await?;
    let browser = login(
        &d.runtime.client,
        &d.runtime.origin,
        &mut d.accounts.submitter,
    )
    .await?;
    for path in [
        "/api/v2/accounts/terms/publish",
        "/api/v2/platform/account-terms/publish",
    ] {
        let r = browser
            .request(
                &d.runtime.client,
                &d.runtime.origin,
                reqwest::Method::POST,
                path,
                Some(serde_json::from_slice(&a.approval_bytes)?),
            )
            .await?;
        assert!(matches!(r.status().as_u16(), 403 | 404 | 405));
    }
    assert_eq!(snapshot(&d.readback).await?, before);
    std::fs::remove_file(&a.custody_path)?;
    let operator = f.authenticate(publisher::Environment::TestOnly).await?;
    assert!(matches!(
        publisher::publish_terms(
            &operator,
            publisher::OperatorApprovalRef {
                kind: publisher::ApprovalKind::OperatorReleaseApproval,
                approval_sha256: a.approval_digest
            }
        )
        .await,
        Err(publisher::PublicationError::CustodyUnavailable)
    ));
    assert_eq!(snapshot(&d.readback).await?, before);
    Ok(())
}
#[sqlx::test(migrations = false)]
async fn newer_publication_with_old_artifact_configuration_closes_enrollment_preserves_existing_account(
    pool: PgPool,
) -> Result {
    let mut d = build_native_deployment(pool, "mixed-terms-release", 1).await?;
    let browser = login(
        &d.runtime.client,
        &d.runtime.origin,
        &mut d.accounts.submitter,
    )
    .await?;
    let accepted:Vec<Value>=sqlx::query_scalar("SELECT to_jsonb(t) FROM account_terms_acceptances t WHERE account_id=$1 ORDER BY terms_kind,terms_version").bind(d.accounts.submitter.account_id).fetch_all(&d.readback).await?;
    assert!(!accepted.is_empty());
    let head: i64 = sqlx::query_scalar("SELECT revision FROM account_terms_head WHERE id=1")
        .fetch_one(&d.readback)
        .await?;
    let f = ExpandedCustodyFixture::new(
        "new-artifacts-separate-root",
        d.readback.connect_options().get_database().ok_or("db")?,
        &["test.account.service".into()],
    )?;
    let new = publish(&f, head, &f.manifest).await?;
    assert!(
        !d.runtime
            .custody
            .directory
            .path()
            .join("objects")
            .join(&new.manifest_sha256)
            .exists()
    );
    let response = d
        .runtime
        .client
        .get(d.runtime.origin.join("/api/v2/auth/terms")?)
        .send()
        .await?;
    assert_eq!(response.status(), 503);
    assert_eq!(
        response.json::<Value>().await?["error"]["code"],
        "authority_unavailable"
    );
    let account_count: i64 = sqlx::query_scalar("SELECT count(*) FROM accounts")
        .fetch_one(&d.readback)
        .await?;
    let response = d
        .runtime
        .client
        .post(d.runtime.origin.join("/api/v2/auth/registration/start")?)
        .header(
            reqwest::header::ORIGIN,
            d.runtime.origin.as_str().trim_end_matches('/'),
        )
        .json(&json!({"terms_version":new.manifest_sha256}))
        .send()
        .await?;
    assert_eq!(response.status(), 503);
    assert_eq!(
        response.json::<Value>().await?["error"]["code"],
        "authority_unavailable"
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM accounts")
            .fetch_one(&d.readback)
            .await?,
        account_count
    );
    let me = browser
        .request(
            &d.runtime.client,
            &d.runtime.origin,
            reqwest::Method::GET,
            "/api/v2/accounts/me",
            None,
        )
        .await?;
    assert_eq!(me.status(), 200);
    let refresh = browser
        .request(
            &d.runtime.client,
            &d.runtime.origin,
            reqwest::Method::POST,
            "/api/v2/auth/token/refresh",
            Some(json!({})),
        )
        .await?;
    assert_eq!(refresh.status(), 200);
    let current = login(
        &d.runtime.client,
        &d.runtime.origin,
        &mut d.accounts.submitter,
    )
    .await?;
    let logout = current
        .request(
            &d.runtime.client,
            &d.runtime.origin,
            reqwest::Method::POST,
            "/api/v2/auth/logout",
            Some(json!({})),
        )
        .await?;
    assert_eq!(logout.status(), 200);
    let after:Vec<Value>=sqlx::query_scalar("SELECT to_jsonb(t) FROM account_terms_acceptances t WHERE account_id=$1 ORDER BY terms_kind,terms_version").bind(d.accounts.submitter.account_id).fetch_all(&d.readback).await?;
    assert_eq!(after, accepted);
    Ok(())
}
