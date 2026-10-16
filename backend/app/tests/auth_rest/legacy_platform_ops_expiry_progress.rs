//! Actual configured family lifetime and unrelated operator progress through ops.
use super::super::super::legacy_platform_list_credentials::{
    configured_family_state, elapsed, other_operator, role_wait,
};
use super::legacy_platform_ops_family_histories::assert_ops_response;
use super::legacy_platform_ops_transport::ops_task;
use super::*;
use console_platform_auth::guard_legacy_subject_in_tx;

#[sqlx::test(migrations = false)]
async fn configured_ops_absolute_family_expiry_during_source_wait_denies_then_recovers(
    pool: PgPool,
) {
    let (mut f, expected) = populated_fixture(&pool).await;
    let auth = login_test_pool(&pool, TestDatabaseLogin::Auth).await;
    let narrow = configured_family_state(&pool, &f, 10).await;
    let normal = f.router.clone();
    f.router = build_router(narrow.clone());
    // Build/qualify the narrow App before issuance; stored family time comes from real DB.
    let (_, access, created) = issue_bound(&pool, &f, &auth).await;
    let deadline = created + Duration::seconds(10);
    mounted_token(&pool, &f, &access, StatusCode::OK, &expected).await;
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
    let reader = ops_task(&f.router, &access, Method::GET);
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
    assert_ops_response(
        &f,
        &access,
        response.unwrap(),
        StatusCode::UNAUTHORIZED,
        &expected,
    )
    .await;
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
    mounted_token(&pool, &f, &f.access, StatusCode::OK, &expected).await;
    f.router = normal;
    mounted_token(&pool, &f, &access, StatusCode::OK, &expected).await;
    narrow.shutdown_realtime().await;
    auth.close().await;
    close_ops(f).await;
}

#[sqlx::test(migrations = false)]
async fn unrelated_operator_completes_while_one_ops_source_is_blocked(pool: PgPool) {
    let (f, expected) = populated_fixture(&pool).await;
    let auth = login_test_pool(&pool, TestDatabaseLogin::Auth).await;
    let (other, other_access) = other_operator(&pool, &f).await;
    assert!(other != f.actor);
    mounted_token(&pool, &f, &f.access, StatusCode::OK, &expected).await;
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
    let reader = ops_task(&f.router, &f.access, Method::GET);
    let waiter = role_wait(
        &pool,
        pid,
        "console_rt",
        "auth_legacy_platform_source_material_v1",
    )
    .await;
    let other_start = db_now(&pool).await;
    let control = finish(ops_task(&f.router, &other_access, Method::GET)).await;
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
    assert_ops_response(
        &f,
        &other_access,
        control.unwrap(),
        StatusCode::OK,
        &expected,
    )
    .await;
    assert_ops_response(&f, &f.access, response.unwrap(), StatusCode::OK, &expected).await;
    assert!(
        waited_cleanly(pid, waiter, pending, clean) && still_waiting,
        "other source must complete while first owner remains actually blocked"
    );
    assert!(
        health_read_delta(
            &before,
            &during,
            other,
            expected["tenants"].as_array().unwrap().len(),
            other_start,
            other_end
        ),
        "independent source completes one exact owned read while blocker remains"
    );
    assert!(
        health_read_delta(
            &during,
            &after,
            f.actor,
            expected["tenants"].as_array().unwrap().len(),
            start,
            end
        ),
        "released source resumes once without affecting completed independent work"
    );
    mounted_token(&pool, &f, &f.access, StatusCode::OK, &expected).await;
    auth.close().await;
    close_ops(f).await;
}
