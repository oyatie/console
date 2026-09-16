use crate::context_owner_fixture::ContextFixture;
use crate::operator_custody_producer::Result;
use console_identity_adapter_postgres::account13 as identity;
use http::StatusCode;
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

async fn actors(pool: &PgPool) -> Result<Vec<Value>> {
    Ok(
        sqlx::query_scalar("SELECT to_jsonb(a) FROM company_actors a ORDER BY org_id,account_id")
            .fetch_all(pool)
            .await?,
    )
}
fn error(status: StatusCode, body: &Value, expected: StatusCode, code: &str) {
    assert_eq!(status, expected);
    assert_eq!(body["error"]["code"], code);
    assert_eq!(
        body.as_object().unwrap().keys().collect::<Vec<_>>(),
        vec!["error"]
    );
    assert_eq!(body["error"].as_object().unwrap().len(), 2);
}
#[sqlx::test(migrations = false)]
async fn ctx_maximum_populated_page_one_is_complete_projected_and_read_only(
    pool: PgPool,
) -> Result<()> {
    let f = ContextFixture::create(&pool, 32, 8).await?;
    let before = actors(&pool).await?;
    assert_eq!(f.scan().await?, f.expected);
    assert_eq!(f.expected.get(&f.label_denied), Some(&None));
    assert!(!f.expected.contains_key(&f.undiscoverable));
    assert_eq!(actors(&pool).await?, before);
    Ok(())
}

#[sqlx::test(migrations = false)]
async fn ctx_revocation_and_denied_to_allowed_invalidate_whole_scan_cursor(
    pool: PgPool,
) -> Result<()> {
    let mut f = ContextFixture::create(&pool, 2, 3).await?;
    let path = "/api/v2/accounts/me/contexts?page_size=1";
    let (status, first, _) = f
        .page(&f.accounts.submitter.account_access_token, path)
        .await?;
    assert_eq!(status, StatusCode::OK);
    let cursor = first["next_cursor"].as_str().unwrap();
    let old = format!("{path}&cursor={cursor}");
    let index = f.companies.len() - 1;
    let org = f.companies[index];
    let expected = identity::read_company_policy_control(&f.owner_pool, &f.operator, org).await?;
    // Restore a formerly denied candidate outside the first page using the same
    // ordinary grant transaction. No manual generation increment stands in for it.
    identity::restore_company_grant(
        &f.owner_pool,
        &f.operator,
        identity::RestoreCompanyGrant {
            command_id: Uuid::new_v4(),
            org_id: org,
            grant_id: f.grants[index].grant_id,
            expected,
            reason: "TEST_ONLY denied-to-allowed transition".into(),
        },
    )
    .await?;
    let (status, body, _) = f
        .page(&f.accounts.submitter.account_access_token, &old)
        .await?;
    error(status, &body, StatusCode::CONFLICT, "stale_cursor");
    f.expected
        .insert(f.undiscoverable.clone(), Some("visible-01-02".into()));
    // The fixture's permanent negative sentinel must no longer suppress this
    // now-authorized context in the scan oracle.
    f.undiscoverable = "no-longer-denied".into();
    assert_eq!(f.scan().await?, f.expected);
    let (_, page, _) = f
        .page(&f.accounts.submitter.account_access_token, path)
        .await?;
    let old = format!("{path}&cursor={}", page["next_cursor"].as_str().unwrap());
    let org = f.companies[1];
    let expected = identity::read_company_policy_control(&f.owner_pool, &f.operator, org).await?;
    identity::revoke_company_grant(
        &f.owner_pool,
        &f.operator,
        identity::RevokeCompanyGrant {
            command_id: Uuid::new_v4(),
            org_id: org,
            grant_id: f.grants[1].grant_id,
            expected,
            reason: "TEST_ONLY allowed-to-denied transition".into(),
        },
    )
    .await?;
    let (status, body, _) = f
        .page(&f.accounts.submitter.account_access_token, &old)
        .await?;
    error(status, &body, StatusCode::CONFLICT, "stale_cursor");
    f.expected.remove(&format!("org_id:{org}"));
    assert_eq!(f.scan().await?, f.expected);
    Ok(())
}

#[sqlx::test(migrations = false)]
async fn ctx_foreign_query_and_future_cursor_binding_refuse_uniformly(pool: PgPool) -> Result<()> {
    let f = ContextFixture::create(&pool, 2, 2).await?;
    let path = "/api/v2/accounts/me/contexts?page_size=1";
    let (_, first, _) = f
        .page(&f.accounts.submitter.account_access_token, path)
        .await?;
    let cursor = first["next_cursor"].as_str().unwrap();
    let old = format!("{path}&cursor={cursor}");
    let (status, foreign, _) = f
        .page(&f.accounts.reviewer.account_access_token, &old)
        .await?;
    error(status, &foreign, StatusCode::CONFLICT, "stale_cursor");
    let (status, changed, _) = f
        .page(
            &f.accounts.submitter.account_access_token,
            &format!("/api/v2/accounts/me/contexts?page_size=2&cursor={cursor}"),
        )
        .await?;
    error(status, &changed, StatusCode::CONFLICT, "stale_cursor");
    // Adversarial persisted cursor state tests exact equality, not an alternative
    // context authority constructor. Cursor format/hash is an explicit owner
    // codec registration; must be bound before this query is admitted.
    let affected=sqlx::query("UPDATE account_context_cursors SET context_generation=context_generation+1 WHERE account_id=$1")
        .bind(f.accounts.submitter.account_id).execute(&pool).await?.rows_affected();
    assert_eq!(affected, 1);
    let (status, future, _) = f
        .page(&f.accounts.submitter.account_access_token, &old)
        .await?;
    error(status, &future, StatusCode::CONFLICT, "stale_cursor");
    assert_eq!(
        foreign, future,
        "foreign ownership must not be disclosed in errors"
    );
    Ok(())
}

#[sqlx::test(migrations = false)]
async fn ctx_empty_is_success_and_unavailable_is_not_empty(pool: PgPool) -> Result<()> {
    let f = ContextFixture::create(&pool, 2, 2).await?;
    let (status, empty, _) = f
        .page(
            &f.accounts.reviewer.account_access_token,
            "/api/v2/accounts/me/contexts?page_size=1",
        )
        .await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(empty, json!({"contexts":[]}));
    // A real SQL authority lookup failure in this disposable database. The
    // projector may not swallow it and publish the earlier empty result.
    let mut blocker = pool.begin().await?;
    sqlx::query("LOCK TABLE account_context_candidates IN ACCESS EXCLUSIVE MODE")
        .execute(&mut *blocker)
        .await?;
    let result = f
        .page(
            &f.accounts.reviewer.account_access_token,
            "/api/v2/accounts/me/contexts?page_size=1",
        )
        .await;
    blocker.rollback().await?;
    let (status, body, _) = result?;
    error(
        status,
        &body,
        StatusCode::SERVICE_UNAVAILABLE,
        "navigation_unavailable",
    );
    let (status, empty, _) = f
        .page(
            &f.accounts.reviewer.account_access_token,
            "/api/v2/accounts/me/contexts?page_size=1",
        )
        .await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(empty, json!({"contexts":[]}));
    Ok(())
}

#[sqlx::test(migrations = false)]
async fn ctx_cursor_has_five_minute_lifetime_and_expires_at_real_authoritative_time(
    pool: PgPool,
) -> Result<()> {
    let f = ContextFixture::create(&pool, 2, 2).await?;
    let (_, first, _) = f
        .page(
            &f.accounts.submitter.account_access_token,
            "/api/v2/accounts/me/contexts?page_size=1",
        )
        .await?;
    let lifetime:f64=sqlx::query_scalar("SELECT extract(epoch FROM expires_at-clock_timestamp())::float8 FROM account_context_cursors WHERE account_id=$1")
        .bind(f.accounts.submitter.account_id).fetch_one(&pool).await?;
    assert!(lifetime > 0.0 && lifetime <= 300.0);
    // Deliberately a real-clock acceptance. A future faster clock adapter must be
    // injected into the actual owner, not substituted into the expected response.
    tokio::time::sleep(std::time::Duration::from_secs(301)).await;
    let (status, body, _) = f
        .page(
            &f.accounts.submitter.account_access_token,
            &format!(
                "/api/v2/accounts/me/contexts?page_size=1&cursor={}",
                first["next_cursor"].as_str().unwrap()
            ),
        )
        .await?;
    error(status, &body, StatusCode::CONFLICT, "stale_cursor");
    Ok(())
}

#[sqlx::test(migrations = false)]
async fn ctx_future_allow_outside_page_one_invalidates_cursor_without_epoch_change(
    pool: PgPool,
) -> Result<()> {
    let mut f = ContextFixture::create(&pool, 2, 3).await?;
    let org = f.companies[5];
    let now: time::OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
        .fetch_one(&pool)
        .await?;
    let transition = now + time::Duration::seconds(10);
    let schema = identity::read_company_policy_schema(&f.owner_pool, &f.operator, org).await?;
    let expected = identity::read_company_policy_control(&f.owner_pool, &f.operator, org).await?;
    identity::apply_company_grant_plan(
        &f.owner_pool,
        &f.operator,
        identity::ApplyCompanyGrantPlan {
            command_id: Uuid::new_v4(),
            org_id: org,
            expected,
            plan: identity::CompanyGrantPlan {
                account_id: f.accounts.submitter.account_id,
                scope: identity::PolicyScope::Company,
                actions: vec![schema.action("context.discover")?],
                field_projection: schema.projection("context.identity_and_label")?,
                valid_from: Some(transition),
                valid_to: None,
                reason: "TEST_ONLY future denied-to-allowed".into(),
            },
        },
    )
    .await?;
    let (_, page, _) = f
        .page(
            &f.accounts.submitter.account_access_token,
            "/api/v2/accounts/me/contexts?page_size=1",
        )
        .await?;
    let cursor = page["next_cursor"]
        .as_str()
        .ok_or("future fixture must start before transition")?;
    let (stable,generation):(time::OffsetDateTime,i64)=sqlx::query_as("SELECT decision_stable_until,context_generation FROM account_context_cursors WHERE account_id=$1")
        .bind(f.accounts.submitter.account_id).fetch_one(&pool).await?;
    assert!(
        stable <= transition,
        "whole scan must include future denied candidate"
    );
    let captured_now: time::OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
        .fetch_one(&pool)
        .await?;
    assert!(
        captured_now < transition,
        "temporal fixture setup missed its boundary; not a behavioral failure"
    );
    tokio::time::sleep(std::time::Duration::from_secs_f64(
        (transition - captured_now).as_seconds_f64() + 0.05,
    ))
    .await;
    let current: i64 =
        sqlx::query_scalar("SELECT context_generation FROM account_security WHERE account_id=$1")
            .bind(f.accounts.submitter.account_id)
            .fetch_one(&pool)
            .await?;
    assert_eq!(
        generation, current,
        "no writer or generation change should explain invalidation"
    );
    let (status, body, _) = f
        .page(
            &f.accounts.submitter.account_access_token,
            &format!("/api/v2/accounts/me/contexts?page_size=1&cursor={cursor}"),
        )
        .await?;
    error(status, &body, StatusCode::CONFLICT, "stale_cursor");
    f.expected
        .insert(f.undiscoverable.clone(), Some("visible-01-02".into()));
    f.undiscoverable = "no-longer-denied".into();
    assert_eq!(f.scan().await?, f.expected);
    Ok(())
}
