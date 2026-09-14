//! Actual provisioner membership writers + real PostgreSQL contention witness.
//! Detached Company may remain discoverable through independent direct grants;
//! this test requires generation/incarnation invalidation, not false disappearance.
use crate::{context_owner_fixture::ContextFixture, operator_custody_producer::Result};
use console_identity_adapter_postgres::account13 as identity;
use console_platform_provisioning::{PlatformProvisioner, account13 as provisioning};
use console_platform_request_context::account as context;
use sqlx::PgPool;
use uuid::Uuid;
#[sqlx::test(migrations = false)]
async fn ctx_known_undiscoverable_target_and_unknown_target_have_same_public404(
    pool: PgPool,
) -> Result<()> {
    use axum::response::IntoResponse;
    let f = ContextFixture::create(&pool, 2, 3).await?;
    let token = &f.accounts.submitter.account_access_token;
    context::switch_company_context(&f.auth_pool, &f.verifier, token, f.companies[1], &f.serving)
        .await?;
    let before: i64 = sqlx::query_scalar("SELECT count(*) FROM company_actors WHERE account_id=$1")
        .bind(f.accounts.submitter.account_id)
        .fetch_one(&pool)
        .await?;
    let mut observations = Vec::new();
    for target in [*f.companies.last().unwrap(), Uuid::new_v4()] {
        let error =
            context::switch_company_context(&f.auth_pool, &f.verifier, token, target, &f.serving)
                .await
                .expect_err("known/unknown undiscoverable target refuses");
        // Exact ordinary public error projection used by active-context handler.
        let response =
            console_platform_auth_rest::account_context_error_response(error).into_response();
        assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
        let body = axum::body::to_bytes(response.into_body(), 4096).await?;
        observations.push(serde_json::from_slice::<serde_json::Value>(&body)?);
    }
    assert_eq!(observations[0], observations[1]);
    assert_eq!(observations[0]["error"]["code"], "not_found");
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM company_actors WHERE account_id=$1")
            .bind(f.accounts.submitter.account_id)
            .fetch_one(&pool)
            .await?,
        before
    );
    Ok(())
}
#[sqlx::test(migrations = false)]
async fn ctx_other_account_actual_allowed_a_does_not_gain_b_from_first_account(
    pool: PgPool,
) -> Result<()> {
    let f = ContextFixture::create(&pool, 2, 3).await?;
    let a = f.companies[1];
    let b = f.companies[2];
    let schema = identity::read_company_policy_schema(&f.owner_pool, &f.operator, a).await?;
    let expected = identity::read_company_policy_control(&f.owner_pool, &f.operator, a).await?;
    identity::apply_company_grant_plan(
        &f.owner_pool,
        &f.operator,
        identity::ApplyCompanyGrantPlan {
            command_id: Uuid::new_v4(),
            org_id: a,
            expected,
            plan: identity::CompanyGrantPlan {
                account_id: f.accounts.reviewer.account_id,
                scope: identity::PolicyScope::Company,
                actions: vec![schema.action("context.discover")?],
                field_projection: schema.projection("context.identity_and_label")?,
                valid_from: None,
                valid_to: None,
                reason: "TEST_ONLY Y actual A-only permission".into(),
            },
        },
    )
    .await?;
    let token = &f.accounts.reviewer.account_access_token;
    let selected =
        context::switch_company_context(&f.auth_pool, &f.verifier, token, a, &f.serving).await?;
    let auth =
        context::resolve_company_context(&f.auth_pool, &f.verifier, token, &selected, &f.serving)
            .await?;
    assert_eq!(
        identity::read_company_context(&f.owner_pool, &auth, a)
            .await?
            .context_ref
            .org_id,
        a
    );
    assert!(
        context::switch_company_context(&f.auth_pool, &f.verifier, token, b, &f.serving)
            .await
            .is_err()
    );
    Ok(())
}
#[sqlx::test(migrations = false)]
async fn ctx_detach_reattach_witnesses_atomic_candidate_fence_and_fresh_membership_incarnation(
    pool: PgPool,
) -> Result<()> {
    let f = ContextFixture::create(&pool, 2, 3).await?;
    let org = f.companies[1];
    let group = f.groups[0];
    let account = f.accounts.submitter.account_id;
    let old: Uuid = sqlx::query_scalar(
        "SELECT membership_incarnation FROM group_memberships WHERE group_id=$1 AND org_id=$2",
    )
    .bind(group)
    .bind(org)
    .fetch_one(&pool)
    .await?;
    let sourced:i64=sqlx::query_scalar("SELECT count(*) FROM account_context_candidates WHERE account_id=$1 AND context_id=$2 AND incarnation=$3 AND state='CURRENT'").bind(account).bind(org).bind(old).fetch_one(&pool).await?;
    assert!(
        sourced > 0,
        "actual Group expansion source fixture required"
    );
    let (_, first, _) = f
        .page(
            &f.accounts.submitter.account_access_token,
            "/api/v2/accounts/me/contexts?page_size=1",
        )
        .await?;
    let cursor = first["next_cursor"]
        .as_str()
        .ok_or("populated cursor")?
        .to_string();
    let generation: i64 =
        sqlx::query_scalar("SELECT context_generation FROM account_security WHERE account_id=$1")
            .bind(account)
            .fetch_one(&pool)
            .await?;
    let p = PlatformProvisioner::new(time::Duration::minutes(5));
    let expected = p
        .read_account_group_control(&f.owner_pool, &f.operator, group)
        .await?;
    let mut barrier = pool.begin().await?;
    let blocker: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *barrier)
        .await?;
    sqlx::query("SELECT account_id FROM account_security WHERE account_id=$1 FOR UPDATE")
        .bind(account)
        .fetch_one(&mut *barrier)
        .await?;
    let removing = p.remove_account_company_from_group(
        &f.owner_pool,
        &f.operator,
        provisioning::CompanyMembershipChange {
            command_id: Uuid::new_v4(),
            group_id: group,
            org_id: org,
            expected: expected.clone(),
        },
    );
    tokio::pin!(removing);
    let witness = async {
        for _ in 0..200 {
            let pids:Vec<i32>=sqlx::query_scalar("SELECT pid FROM pg_stat_activity WHERE datname=current_database() AND $1=ANY(pg_blocking_pids(pid))").bind(blocker).fetch_all(&pool).await?;
            if pids.len() == 1 {
                return Ok::<_, Box<dyn std::error::Error + Send + Sync>>(pids[0]);
            }
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
        Err("no actual topology writer Account-guard contention".into())
    };
    let pid = tokio::select! {r=&mut removing=>{r?;panic!("topology writer bypassed affected Account guard")},p=witness=>p?};
    assert_ne!(pid, blocker);
    assert_eq!(
        sqlx::query_scalar::<_, Uuid>(
            "SELECT membership_incarnation FROM group_memberships WHERE group_id=$1 AND org_id=$2"
        )
        .bind(group)
        .bind(org)
        .fetch_one(&pool)
        .await?,
        old
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT context_generation FROM account_security WHERE account_id=$1"
        )
        .bind(account)
        .fetch_one(&pool)
        .await?,
        generation
    );
    barrier.commit().await?;
    removing.await?;
    let (status, body, _) = f
        .page(
            &f.accounts.submitter.account_access_token,
            &format!("/api/v2/accounts/me/contexts?page_size=1&cursor={cursor}"),
        )
        .await?;
    assert_eq!(status, http::StatusCode::CONFLICT);
    assert_eq!(body["error"]["code"], "stale_cursor");
    assert!(
        sqlx::query_scalar::<_, i64>(
            "SELECT context_generation FROM account_security WHERE account_id=$1"
        )
        .bind(account)
        .fetch_one(&pool)
        .await?
            > generation
    );
    assert_eq!(sqlx::query_scalar::<_,i64>("SELECT count(*) FROM account_context_candidates WHERE account_id=$1 AND context_id=$2 AND incarnation=$3 AND state='CURRENT'").bind(account).bind(org).bind(old).fetch_one(&pool).await?,0);
    let expected = p
        .read_account_group_control(&f.owner_pool, &f.operator, group)
        .await?;
    p.assign_account_company_to_group(
        &f.owner_pool,
        &f.operator,
        provisioning::CompanyMembershipChange {
            command_id: Uuid::new_v4(),
            group_id: group,
            org_id: org,
            expected,
        },
    )
    .await?;
    let new: Uuid = sqlx::query_scalar(
        "SELECT membership_incarnation FROM group_memberships WHERE group_id=$1 AND org_id=$2",
    )
    .bind(group)
    .bind(org)
    .fetch_one(&pool)
    .await?;
    assert_ne!(new, old);
    let current:i64=sqlx::query_scalar("SELECT count(*) FROM account_context_candidates WHERE account_id=$1 AND context_id=$2 AND incarnation=$3 AND state='CURRENT'").bind(account).bind(org).bind(new).fetch_one(&pool).await?;
    assert!(current > 0);
    assert!(f.scan().await?.contains_key(&format!("org_id:{org}")));
    Ok(())
}
