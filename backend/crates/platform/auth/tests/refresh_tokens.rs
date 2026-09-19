#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use console_kernel_core::OrgId;
use console_platform_auth::{RefreshTokenStore, RefreshTokenUseError};
use console_platform_test_support::{
    TestDatabaseLogin, login_test_pool, prepare_account_test_database,
};
use sqlx::{PgPool, Row};
use time::{Duration, OffsetDateTime};

// Transport comparison for existing, unmigrated legacy subjects. Account-wide
// security audit and ACCOUNT_V1 semantics require their separate acceptance tests.
async fn auth_role_pool(owner_pool: &PgPool) -> PgPool {
    login_test_pool(owner_pool, TestDatabaseLogin::Auth).await
}

/// Regression for the task #26 deferred site: `insert_audit_in_tx` inserted the
/// refresh/logout audit rows WITHOUT the `org_id` column (NULL). The FORCE-RLS
/// `audit_events` WITH CHECK permits NULL, so the write succeeded — but a
/// tenant-scoped `/api/audit` read (`USING (org_id = app.current_org)`) could
/// then never see these events.
///
/// RED (before the fix): the `auth.refresh` row lands with NULL org, so the
/// KNL-armed read below returns 0. GREEN: the row carries KNL and is visible.
#[sqlx::test(migrations = false)]
async fn rotate_audit_row_is_visible_to_tenant_scoped_read_as_runtime_role(pool: PgPool) {
    prepare_account_test_database(&pool).await;
    let user_id = seed_user(&pool).await;
    let auth = auth_role_pool(&pool).await;
    let rt = login_test_pool(&pool, TestDatabaseLogin::Business).await;
    let store = RefreshTokenStore;
    let now = OffsetDateTime::now_utc();
    let ttl = Duration::days(30);
    let absolute_ttl = Duration::days(30);

    // Credential effects use the real auth login; Company audit observation below
    // remains on the distinct business login for this unmigrated legacy subject.
    let first = store
        .issue_family(&auth, &auth, user_id, OrgId::knl(), now, ttl)
        .await
        .unwrap();
    store
        .rotate(
            &auth,
            &auth,
            first.token.as_str(),
            now + Duration::minutes(1),
            ttl,
            absolute_ttl,
        )
        .await
        .unwrap();

    // As `console_rt`, armed to KNL: the rotate ('auth.refresh') row must be visible.
    let mut tx = rt.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.current_org', $1, true)")
        .bind(OrgId::knl().as_uuid().to_string())
        .execute(tx.as_mut())
        .await
        .unwrap();
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM audit_events WHERE action = 'auth.refresh'")
            .fetch_one(tx.as_mut())
            .await
            .unwrap();
    tx.commit().await.unwrap();
    assert_eq!(
        count, 1,
        "rotate audit row must carry the tenant org so a tenant-scoped read sees it"
    );
}

async fn seed_user(pool: &PgPool) -> uuid::Uuid {
    sqlx::query_scalar(
        "INSERT INTO users (display_name, roles, org_id) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind("Refresh User")
    .bind(Vec::<String>::from(["MECHANIC".to_owned()]))
    .bind(*OrgId::knl().as_uuid())
    .fetch_one(pool)
    .await
    .unwrap()
}

#[sqlx::test(migrations = false)]
async fn refresh_token_reuse_revokes_the_whole_family(pool: PgPool) {
    prepare_account_test_database(&pool).await;
    let user_id = seed_user(&pool).await;
    let auth = auth_role_pool(&pool).await;
    let store = RefreshTokenStore;
    let now = OffsetDateTime::now_utc();
    let ttl = Duration::days(30);
    // Wide absolute cap so this reuse test is unaffected by the family TTL ceiling.
    let absolute_ttl = Duration::days(30);

    let first = store
        .issue_family(&auth, &auth, user_id, OrgId::knl(), now, ttl)
        .await
        .unwrap();
    let second = store
        .rotate(
            &auth,
            &auth,
            first.token.as_str(),
            now + Duration::minutes(1),
            ttl,
            absolute_ttl,
        )
        .await
        .unwrap();

    let reuse = store
        .rotate(
            &auth,
            &auth,
            first.token.as_str(),
            now + Duration::minutes(2),
            ttl,
            absolute_ttl,
        )
        .await
        .unwrap_err();
    assert_eq!(reuse, RefreshTokenUseError::ReuseDetected);

    let after_reuse = store
        .rotate(
            &auth,
            &auth,
            second.token.as_str(),
            now + Duration::minutes(3),
            ttl,
            absolute_ttl,
        )
        .await
        .unwrap_err();
    assert_eq!(after_reuse, RefreshTokenUseError::FamilyRevoked);

    let family = sqlx::query(
        "SELECT revoked_at, revoked_reason FROM auth_refresh_token_families WHERE id = $1",
    )
    .bind(first.family_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let revoked_at: Option<OffsetDateTime> = family.try_get("revoked_at").unwrap();
    let revoked_reason: Option<String> = family.try_get("revoked_reason").unwrap();
    assert!(revoked_at.is_some());
    assert_eq!(revoked_reason.as_deref(), Some("reuse_detected"));

    let token_rows: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM auth_refresh_tokens WHERE family_id = $1")
            .bind(first.family_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(token_rows, 2);

    let revoked_rows: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM auth_refresh_tokens WHERE family_id = $1 AND revoked_at IS NOT NULL",
    )
    .bind(first.family_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(revoked_rows, 2);
}

/// A rotation past the family's absolute TTL (measured from creation) is rejected
/// and revokes the family, even when the presented token is otherwise valid,
/// unused, and not individually expired. This is the NIST AAL2 absolute
/// session-lifetime cap.
#[sqlx::test(migrations = false)]
async fn rotation_past_family_absolute_ttl_revokes_the_family(pool: PgPool) {
    prepare_account_test_database(&pool).await;
    let user_id = seed_user(&pool).await;
    let auth = auth_role_pool(&pool).await;
    let store = RefreshTokenStore;
    let now = OffsetDateTime::now_utc();
    // Per-token TTL is generous so the rejection can ONLY come from the absolute
    // family cap, not from individual-token expiry.
    let ttl = Duration::days(30);
    let absolute_ttl = Duration::hours(24);

    let first = store
        .issue_family(&auth, &auth, user_id, OrgId::knl(), now, ttl)
        .await
        .unwrap();

    // A rotation comfortably within the cap still succeeds.
    let second = store
        .rotate(
            &auth,
            &auth,
            first.token.as_str(),
            now + Duration::hours(1),
            ttl,
            absolute_ttl,
        )
        .await
        .unwrap();

    // One second past the absolute ceiling: rejected as FamilyRevoked.
    let expired = store
        .rotate(
            &auth,
            &auth,
            second.token.as_str(),
            now + absolute_ttl + Duration::seconds(1),
            ttl,
            absolute_ttl,
        )
        .await
        .unwrap_err();
    assert_eq!(expired, RefreshTokenUseError::FamilyRevoked);

    // The family is revoked with the absolute-TTL reason.
    let family = sqlx::query(
        "SELECT revoked_at, revoked_reason FROM auth_refresh_token_families WHERE id = $1",
    )
    .bind(first.family_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let revoked_at: Option<OffsetDateTime> = family.try_get("revoked_at").unwrap();
    let revoked_reason: Option<String> = family.try_get("revoked_reason").unwrap();
    assert!(revoked_at.is_some());
    assert_eq!(revoked_reason.as_deref(), Some("absolute_ttl_exceeded"));

    // Every token in the family is now revoked, so no sibling can rotate either.
    let live_tokens: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM auth_refresh_tokens WHERE family_id = $1 AND revoked_at IS NULL",
    )
    .bind(first.family_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(live_tokens, 0);
}

// Privileged observation only. Compare whole rows without printing credentials
// on failure; neither runtime gains read access through this test helper.
async fn refusal_snapshot(pool: &PgPool) -> serde_json::Value {
    sqlx::query_scalar(
        "SELECT jsonb_build_object(
          'roots',(SELECT jsonb_agg(to_jsonb(r) ORDER BY id) FROM accounts r),
          'security',(SELECT jsonb_agg(to_jsonb(r) ORDER BY account_id) FROM account_security r),
          'events',(SELECT jsonb_agg(to_jsonb(r) ORDER BY id) FROM account_security_events r),
          'keys',(SELECT jsonb_agg(to_jsonb(r) ORDER BY id) FROM auth_webauthn_credentials r),
          'families',(SELECT jsonb_agg(to_jsonb(r) ORDER BY id) FROM auth_refresh_token_families r),
          'tokens',(SELECT jsonb_agg(to_jsonb(r) ORDER BY id) FROM auth_refresh_tokens r),
          'users',(SELECT jsonb_agg(to_jsonb(r) ORDER BY id) FROM users r),
          'audit',(SELECT jsonb_agg(to_jsonb(r) ORDER BY id) FROM audit_events r))",
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn refusal_pair(
    auth: &PgPool,
    token: &str,
    now: OffsetDateTime,
    expected: RefreshTokenUseError,
) {
    let rotate = match RefreshTokenStore
        .rotate(
            auth,
            auth,
            token,
            now,
            Duration::days(30),
            Duration::days(30),
        )
        .await
    {
        Err(error) => error,
        Ok(_) => panic!("operation must refuse"),
    };
    let logout = match RefreshTokenStore
        .revoke_family_for_logout(auth, token, now)
        .await
    {
        Err(error) => error,
        Ok(_) => panic!("operation must refuse"),
    };
    assert_eq!((rotate, logout), (expected, expected));
}

#[sqlx::test(migrations = false)]
async fn revoked_family_refusal_survives_subject_deactivation_without_effects(pool: PgPool) {
    prepare_account_test_database(&pool).await;
    let user = seed_user(&pool).await;
    let auth = auth_role_pool(&pool).await;
    let now = OffsetDateTime::now_utc();
    let dead = RefreshTokenStore
        .issue_family(&auth, &auth, user, OrgId::knl(), now, Duration::days(30))
        .await
        .unwrap();
    let live = RefreshTokenStore
        .issue_family(&auth, &auth, user, OrgId::knl(), now, Duration::days(30))
        .await
        .unwrap();
    RefreshTokenStore
        .revoke_family_for_logout(&auth, dead.token.as_str(), now)
        .await
        .unwrap();
    // Deliberately retain one live family while denying the subject: the early
    // observation may refuse a dead family but must never authorize a live one.
    assert_eq!(
        sqlx::query("UPDATE users SET is_active=false WHERE id=$1")
            .bind(user)
            .execute(&pool)
            .await
            .unwrap()
            .rows_affected(),
        1
    );
    let before = refusal_snapshot(&pool).await;
    refusal_pair(
        &auth,
        dead.token.as_str(),
        now,
        RefreshTokenUseError::FamilyRevoked,
    )
    .await;
    refusal_pair(
        &auth,
        live.token.as_str(),
        now,
        RefreshTokenUseError::InvalidToken,
    )
    .await;
    refusal_pair(
        &auth,
        "unknown-token",
        now,
        RefreshTokenUseError::InvalidToken,
    )
    .await;
    assert!(
        before == refusal_snapshot(&pool).await,
        "refusals must have no persistent effects"
    );
}

#[sqlx::test(migrations = false)]
async fn revoked_family_refusal_survives_guard_outage_with_live_control_and_recovery(pool: PgPool) {
    prepare_account_test_database(&pool).await;
    let user = seed_user(&pool).await;
    let auth = auth_role_pool(&pool).await;
    let now = OffsetDateTime::now_utc();
    let dead = RefreshTokenStore
        .issue_family(&auth, &auth, user, OrgId::knl(), now, Duration::days(30))
        .await
        .unwrap();
    let live = RefreshTokenStore
        .issue_family(&auth, &auth, user, OrgId::knl(), now, Duration::days(30))
        .await
        .unwrap();
    RefreshTokenStore
        .revoke_family_for_logout(&auth, dead.token.as_str(), now)
        .await
        .unwrap();
    sqlx::query("REVOKE EXECUTE ON FUNCTION public.account_company_deactivation_guard_v1(uuid,uuid) FROM console_auth_rt")
        .execute(&pool).await.unwrap();
    let before = refusal_snapshot(&pool).await;
    // Positive injection control: the same known-live token must reach the
    // unavailable guard through both public callers and fail closed.
    refusal_pair(
        &auth,
        live.token.as_str(),
        now,
        RefreshTokenUseError::Storage,
    )
    .await;
    refusal_pair(
        &auth,
        dead.token.as_str(),
        now,
        RefreshTokenUseError::FamilyRevoked,
    )
    .await;
    assert!(
        before == refusal_snapshot(&pool).await,
        "outage/refusals must preserve all rows"
    );
    sqlx::query("GRANT EXECUTE ON FUNCTION public.account_company_deactivation_guard_v1(uuid,uuid) TO console_auth_rt")
        .execute(&pool).await.unwrap();
    let recovered = RefreshTokenStore
        .rotate(
            &auth,
            &auth,
            live.token.as_str(),
            now,
            Duration::days(30),
            Duration::days(30),
        )
        .await
        .unwrap();
    assert_eq!(recovered.family_id, live.family_id);
    RefreshTokenStore
        .revoke_family_for_logout(&auth, recovered.token.as_str(), now)
        .await
        .unwrap();
    let after = refusal_snapshot(&pool).await;
    assert!(
        before != after,
        "recovered live operations must have effects"
    );
    let logout_events: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM audit_events WHERE action='auth.logout' AND target_id=$1",
    )
    .bind(live.family_id.to_string())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(logout_events, 1);
}
