//! Additive histories through the real public Manager Current owner.
//! Runs only after inherited owner/reopen/negative oracles succeed.
//! No product hook, raw credential copy, business DML or replacement source.
use super::super::manager_policy::ObservedRead;
use super::{all_rows, credentials};
use console_app::AppConfig;
use console_identity_adapter_postgres::PgOrgStore;
use console_identity_application::company_policy::{
    CurrentCompanyAuthority,
    workflow::{NativePolicyCommandRef, NativePolicyFormView, native_policy_current},
};
use console_platform_auth::account::{
    AccountEnrollmentCredentials, ensure_account_session_fresh_in_tx,
};
use futures::FutureExt;
use sqlx::{Connection, PgConnection, PgPool};
use std::{collections::BTreeMap, time::Duration};
use time::OffsetDateTime;
use uuid::Uuid;

async fn positive(
    store: &PgOrgStore,
    credential: &AccountEnrollmentCredentials,
    authority: &CurrentCompanyAuthority,
    selector: NativePolicyCommandRef,
    expected: &NativePolicyFormView,
) {
    let policy = ObservedRead::new(authority.clone(), None);
    let result = native_policy_current(store, &policy, credential, selector).await;
    assert!(
        result.is_ok(),
        "STOP: adjacent actual Manager owner positive prerequisite"
    );
    assert_eq!(result.unwrap(), *expected);
    policy.count(2);
}

#[derive(Clone, Copy)]
enum WaitEnd {
    Release,
    Drop,
    Expire(OffsetDateTime),
}

async fn waited(
    pool: &PgPool,
    runtime: &PgPool,
    config: &AppConfig,
    credential: &AccountEnrollmentCredentials,
    recovery: &AccountEnrollmentCredentials,
    authority: &CurrentCompanyAuthority,
    selector: NativePolicyCommandRef,
    group: Uuid,
    expected: &NativePolicyFormView,
    before: &BTreeMap<String, String>,
    on_group: bool,
    end: WaitEnd,
) {
    // Direct owned operator sockets provide observation/locks only; the actual
    // owner always runs through the previously qualified genuine console_rt LOGIN.
    let mut blocker = tokio::time::timeout(
        Duration::from_secs(3),
        PgConnection::connect_with(pool.connect_options().as_ref()),
    )
    .await
    .unwrap()
    .unwrap();
    let mut observer = tokio::time::timeout(
        Duration::from_secs(3),
        PgConnection::connect_with(pool.connect_options().as_ref()),
    )
    .await
    .unwrap()
    .unwrap();
    let mut checker = tokio::time::timeout(
        Duration::from_secs(3),
        PgConnection::connect_with(pool.connect_options().as_ref()),
    )
    .await
    .unwrap()
    .unwrap();
    let blocker_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut blocker)
        .await
        .unwrap();
    let observer_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut observer)
        .await
        .unwrap();
    let checker_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut checker)
        .await
        .unwrap();
    let mut waiter = None;
    let policy = ObservedRead::new(authority.clone(), None);
    let outcome = std::panic::AssertUnwindSafe(async {
        let store = credentials::store(runtime.clone(), config);
        sqlx::raw_sql("BEGIN ISOLATION LEVEL READ COMMITTED; SET LOCAL statement_timeout='5s'; SET LOCAL lock_timeout='500ms'; SET LOCAL transaction_timeout='10s'")
            .execute(&mut blocker).await.unwrap();
        let target = if on_group { group } else { *authority.account().as_uuid() };
        let query = if on_group {
            "SELECT group_id FROM public.group_authority_heads WHERE group_id=$1 FOR UPDATE"
        } else {
            "SELECT account_id FROM public.account_security WHERE account_id=$1 FOR UPDATE"
        };
        let locked: Uuid = sqlx::query_scalar(sqlx::AssertSqlSafe(query))
            .bind(target).fetch_one(&mut blocker).await.unwrap();
        assert_eq!(locked, target, "STOP: actual UI-created lock target required");
        let (cancel, cancelled) = tokio::sync::oneshot::channel();
        let (dropped, drop_observed) = tokio::sync::oneshot::channel();
        let invocation = async {
            let result = {
                let future = native_policy_current(&store, &policy, credential, selector);
                tokio::pin!(future);
                if matches!(end, WaitEnd::Drop) {
                    tokio::select! { result=&mut future => Some(result), _=cancelled => None }
                } else {
                    Some(future.await)
                }
            }; // Actual owner future has dropped before the acknowledgement below.
            if result.is_none() { let _ = dropped.send(()); }
            result
        };
        let control = async {
            let observation_deadline = tokio::time::Instant::now() + Duration::from_millis(350);
            loop {
                let pids: Vec<i32> = sqlx::query_scalar("SELECT pid FROM pg_catalog.pg_stat_activity WHERE datname=current_database() AND usename='console_rt' AND wait_event_type='Lock' AND $1=ANY(pg_catalog.pg_blocking_pids(pid)) AND strpos(query,'identity_company_information_manager_current_v1')>0 ORDER BY pid")
                    .bind(blocker_pid).fetch_all(&mut observer).await.unwrap();
                if !pids.is_empty() {
                    assert_eq!(pids.len(), 1, "STOP: ambiguous actual Manager source waiter");
                    waiter = Some(pids[0]);
                    break;
                }
                assert!(tokio::time::Instant::now() < observation_deadline,
                        "STOP: actual Manager source wait not reached");
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
            policy.count(0);
            sqlx::raw_sql("BEGIN ISOLATION LEVEL READ COMMITTED; SET LOCAL statement_timeout='1s'")
                .execute(&mut checker).await.unwrap();
            if on_group {
                let free: Uuid = sqlx::query_scalar("SELECT account_id FROM public.account_security WHERE account_id=$1 FOR UPDATE NOWAIT")
                    .bind(*authority.account().as_uuid()).fetch_one(&mut checker).await
                    .expect("STOP: Account lock acquired before actual Group wait");
                assert_eq!(free, *authority.account().as_uuid());
            } else {
                let error = sqlx::query_scalar::<_, Uuid>("SELECT group_id FROM public.group_authority_heads WHERE group_id=$1 FOR UPDATE NOWAIT")
                    .bind(group).fetch_one(&mut checker).await.unwrap_err();
                assert_eq!(error.as_database_error().unwrap().code().as_deref(), Some("55P03"),
                           "STOP: Account wait omitted retained Group guard");
            }
            sqlx::raw_sql("ROLLBACK").execute(&mut checker).await.unwrap();
            if matches!(end, WaitEnd::Drop) {
                cancel.send(()).unwrap();
                drop_observed.await.expect("STOP: actual owner future drop was not acknowledged");
            } else if let WaitEnd::Expire(deadline) = end {
                let started: OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
                    .fetch_one(&mut observer).await.unwrap();
                assert!(started < deadline, "STOP: actual Manager wait reached only after access expiry");
                loop {
                    let at: OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
                        .fetch_one(&mut observer).await.unwrap();
                    if at >= deadline { break; }
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
            }
            sqlx::raw_sql("ROLLBACK").execute(&mut blocker).await.unwrap();
        };
        let (result, ()) = tokio::time::timeout(Duration::from_secs(5),
            async { tokio::join!(invocation, control) }).await
            .expect("STOP: bounded Manager history completion");
        match end {
            WaitEnd::Release => {
                let result = result.expect("ordinary wait was cancelled");
                assert!(result.is_ok(), "actual released Manager wait refused a healthy owner");
                assert_eq!(result.unwrap(), *expected);
                policy.count(2);
            }
            WaitEnd::Drop => {
                assert!(result.is_none(), "cancelled actual owner released a result");
                policy.count(0);
            }
            WaitEnd::Expire(_) => {
                assert!(matches!(result, Some(Err(console_identity_application::company_policy::workflow::NativePolicyWorkflowError::AuthenticationInvalid))),
                        "actual post-wait access expiry released a view or failed before Auth");
                policy.count(0);
            }
        }
    }).catch_unwind().await;

    // Attempt every cleanup before reporting the outcome. Dropped SQLx work may
    // queue rollback; actual transaction/row-lock readback must converge first.
    let cleanup_deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    let rollback_blocker = tokio::time::timeout_at(
        cleanup_deadline,
        sqlx::raw_sql("ROLLBACK").execute(&mut blocker),
    )
    .await;
    let rollback_checker = tokio::time::timeout_at(
        cleanup_deadline,
        sqlx::raw_sql("ROLLBACK").execute(&mut checker),
    )
    .await;
    let released = tokio::time::timeout_at(cleanup_deadline, std::panic::AssertUnwindSafe(async {
        loop {
            let clean: bool = sqlx::query_scalar("SELECT NOT EXISTS(SELECT 1 FROM pg_catalog.pg_stat_activity WHERE pid=$1 AND (xact_start IS NOT NULL OR wait_event_type='Lock')) AND NOT EXISTS(SELECT 1 FROM pg_catalog.pg_locks WHERE pid=$1 AND locktype IN ('relation','transactionid','tuple'))")
                .bind(waiter.unwrap_or(-1)).fetch_one(&mut observer).await.unwrap();
            if clean { break; }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }).catch_unwind()).await;
    let (blocker_close, checker_close, observer_close) = tokio::join!(
        tokio::time::timeout_at(cleanup_deadline, blocker.close()),
        tokio::time::timeout_at(cleanup_deadline, checker.close()),
        tokio::time::timeout_at(cleanup_deadline, observer.close()),
    );
    let absent = tokio::time::timeout_at(
        cleanup_deadline,
        std::panic::AssertUnwindSafe(async {
            loop {
                let present: bool = sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_stat_activity WHERE pid=ANY($1))",
                )
                .bind(vec![blocker_pid, checker_pid, observer_pid])
                .fetch_one(pool)
                .await
                .unwrap();
                if !present {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .catch_unwind(),
    )
    .await;
    assert!(
        matches!(rollback_blocker, Ok(Ok(_)))
            && matches!(rollback_checker, Ok(Ok(_)))
            && matches!(released, Ok(Ok(())))
            && matches!(blocker_close, Ok(Ok(())))
            && matches!(checker_close, Ok(Ok(())))
            && matches!(observer_close, Ok(Ok(())))
            && matches!(absent, Ok(Ok(()))),
        "STOP: owned Manager history cleanup not proven"
    );
    assert!(
        *before == all_rows(pool).await,
        "Manager wait/drop changed complete public census"
    );
    if let Err(panic) = outcome {
        std::panic::resume_unwind(panic);
    }
    positive(
        &credentials::store(runtime.clone(), config),
        recovery,
        authority,
        selector,
        expected,
    )
    .await;
    assert!(
        *before == all_rows(pool).await,
        "adjacent Manager recovery changed any public table"
    );
}

pub(super) async fn verify(
    pool: &PgPool,
    runtime: &PgPool,
    config: &AppConfig,
    credential: &AccountEnrollmentCredentials,
    authority: &CurrentCompanyAuthority,
    selector: NativePolicyCommandRef,
    group: Uuid,
    expected: &NativePolicyFormView,
    before: &BTreeMap<String, String>,
) {
    let (verifier, _, ttl) = credentials::bindings(config);
    let mut tx = runtime.begin().await.unwrap();
    sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL READ COMMITTED")
        .execute(tx.as_mut())
        .await
        .unwrap();
    let session = credential
        .read_session_in_tx(&mut tx, &verifier, ttl)
        .await
        .unwrap();
    assert_eq!(session.account_id, *authority.account().as_uuid());
    ensure_account_session_fresh_in_tx(&mut tx, &session)
        .await
        .unwrap();
    tx.rollback().await.unwrap();
    let store = credentials::store(runtime.clone(), config);
    positive(&store, credential, authority, selector, expected).await;
    // Four named histories: real Group/Account waits with release or future drop.
    // Expiry/final-source/COMMIT acknowledgement cases remain separate HOLDs.
    for on_group in [true, false] {
        for end in [WaitEnd::Release, WaitEnd::Drop] {
            waited(
                pool, runtime, config, credential, credential, authority, selector, group,
                expected, before, on_group, end,
            )
            .await;
        }
    }
}

// Reuse the exact observed-wait/cleanup path. Recovery uses the unaltered browser
// credential; the genuine newly issued short access is only the negative input.
pub(super) async fn expire(
    pool: &PgPool,
    runtime: &PgPool,
    config: &AppConfig,
    credential: &AccountEnrollmentCredentials,
    original: &AccountEnrollmentCredentials,
    authority: &CurrentCompanyAuthority,
    selector: NativePolicyCommandRef,
    group: Uuid,
    expected: &NativePolicyFormView,
    before: &BTreeMap<String, String>,
    deadline: OffsetDateTime,
    on_group: bool,
) {
    waited(
        pool,
        runtime,
        config,
        credential,
        original,
        authority,
        selector,
        group,
        expected,
        before,
        on_group,
        WaitEnd::Expire(deadline),
    )
    .await;
}
