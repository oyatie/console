//! Exact current source refusal and isolated corruption recovery through the ops owner.
use super::super::super::legacy_platform_list_credentials::{root_catalog, rows, same_except};
use super::*;
use futures::FutureExt;

#[sqlx::test(migrations = false)]
async fn direct_ops_current_inactive_and_account_fence_refuse_with_exact_recovery(pool: PgPool) {
    let (f, expected) = populated_fixture(&pool).await;
    owner_positive(&pool, &f, &f.access, &expected).await;
    let original = all_rows(&pool).await;
    sqlx::query("UPDATE public.users SET is_active=false WHERE id=$1")
        .bind(f.actor.as_uuid())
        .execute(&pool)
        .await
        .unwrap();
    let inactive = all_rows(&pool).await;
    let failure = std::panic::AssertUnwindSafe(async {
        assert!(same_except(&original, &inactive, &["users"]));
        let mut expected = rows(&original, "users");
        let actual = rows(&inactive, "users");
        let mut changed = 0;
        for row in &mut expected {
            if row["id"] == json!(f.actor) {
                assert!(row["is_active"] == true);
                row["is_active"] = json!(false);
                changed += 1;
            }
        }
        assert!(
            changed == 1
                && expected.len() == actual.len()
                && expected.iter().all(|r| actual.contains(r))
        );
        owner_denied(&pool, &f, &f.access, Duration::days(30), "unauthorized").await;
    })
    .catch_unwind()
    .await;
    sqlx::query("UPDATE public.users SET is_active=true WHERE id=$1")
        .bind(f.actor.as_uuid())
        .execute(&pool)
        .await
        .unwrap();
    assert!(
        original == all_rows(&pool).await,
        "inactive source restored exactly"
    );
    if let Err(p) = failure {
        std::panic::resume_unwind(p);
    }
    owner_positive(&pool, &f, &f.access, &expected).await;
    let original = all_rows(&pool).await;
    insert_account_fence(&pool, f.actor, "ACTIVE").await;
    let fenced = all_rows(&pool).await;
    let failure = std::panic::AssertUnwindSafe(async {
        assert!(same_except(&original, &fenced, &["account_security"]));
        let old = rows(&original, "account_security");
        let new = rows(&fenced, "account_security");
        assert!(new.len() == old.len() + 1 && old.iter().all(|r| new.contains(r)));
        let added: Vec<_> = new.iter().filter(|r| !old.contains(r)).collect();
        assert!(
            added.len() == 1
                && added[0]["account_id"] == json!(f.actor)
                && added[0]["security_state"] == "ACTIVE"
        );
        owner_denied(&pool, &f, &f.access, Duration::days(30), "unauthorized").await;
    })
    .catch_unwind()
    .await;
    sqlx::query("DELETE FROM public.account_security WHERE account_id=$1")
        .bind(f.actor.as_uuid())
        .execute(&pool)
        .await
        .unwrap();
    assert!(
        original == all_rows(&pool).await,
        "privileged fence fixture restored exactly"
    );
    if let Err(p) = failure {
        std::panic::resume_unwind(p);
    }
    owner_positive(&pool, &f, &f.access, &expected).await;
    close_ops(f).await;
}

#[sqlx::test(migrations = false)]
async fn direct_ops_missing_account_root_is_unavailable_and_restores_exactly(pool: PgPool) {
    let (f, expected) = populated_fixture(&pool).await;
    owner_positive(&pool, &f, &f.access, &expected).await;
    let before = all_rows(&pool).await;
    let catalog = root_catalog(&pool).await;
    let account: Value =
        sqlx::query_scalar("SELECT to_jsonb(a) FROM public.accounts a WHERE id=$1")
            .bind(f.actor.as_uuid())
            .fetch_one(&pool)
            .await
            .unwrap();
    // Privileged isolated corruption fixture, never a proposed production mutation route.
    let mut tx = pool.begin().await.unwrap();
    sqlx::raw_sql("SET CONSTRAINTS ALL IMMEDIATE; ALTER TABLE public.accounts DISABLE TRIGGER account_roots_immutable_v1; SET LOCAL session_replication_role=replica").execute(&mut *tx).await.unwrap();
    sqlx::query("DELETE FROM public.accounts WHERE id=$1")
        .bind(f.actor.as_uuid())
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::raw_sql("SET LOCAL session_replication_role=origin; ALTER TABLE public.accounts ENABLE ALWAYS TRIGGER account_roots_immutable_v1").execute(&mut *tx).await.unwrap();
    tx.commit().await.unwrap();
    let failure = std::panic::AssertUnwindSafe(async {
        assert!(
            catalog == root_catalog(&pool).await,
            "missing data must not masquerade as changed metadata"
        );
        let missing = all_rows(&pool).await;
        assert!(same_except(&before, &missing, &["accounts"]));
        let old = rows(&before, "accounts");
        let new = rows(&missing, "accounts");
        assert!(
            old.len() == new.len() + 1
                && new.iter().all(|r| old.contains(r))
                && old
                    .iter()
                    .filter(|r| !new.contains(r))
                    .all(|r| r["id"] == json!(f.actor))
        );
        owner_denied(&pool, &f, &f.access, Duration::days(30), "unavailable").await;
    })
    .catch_unwind()
    .await;
    sqlx::query(
        "INSERT INTO public.accounts SELECT (jsonb_populate_record(NULL::public.accounts,$1)).*",
    )
    .bind(account)
    .execute(&pool)
    .await
    .unwrap();
    assert!(
        before == all_rows(&pool).await && catalog == root_catalog(&pool).await,
        "Account data and exact metadata restore before recovery"
    );
    if let Err(p) = failure {
        std::panic::resume_unwind(p);
    }
    owner_positive(&pool, &f, &f.access, &expected).await;
    close_ops(f).await;
}
