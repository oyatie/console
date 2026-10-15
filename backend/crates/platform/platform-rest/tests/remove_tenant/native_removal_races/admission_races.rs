//! Operator edits below are adversarial fixtures, not product mutation journeys.

use super::*;

async fn current_group_lock(f: &Fixture) -> (sqlx::Transaction<'static, sqlx::Postgres>, i32) {
    let mut tx = f.owner.begin().await.unwrap();
    let pid = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(tx.as_mut())
        .await
        .unwrap();
    let group: Uuid = sqlx::query_scalar("SELECT g.id FROM public.groups g JOIN public.organizations o ON o.group_id=g.id WHERE o.id=$1 FOR UPDATE OF g")
        .bind(f.target).fetch_one(tx.as_mut()).await.unwrap();
    assert_ne!(group, f.destination);
    (tx, pid)
}

async fn customer(f: &Fixture) -> (Uuid, Uuid) {
    // Existing disposable legacy data helper, never a successful business API.
    seed_real_data(&f.owner, f.target).await;
    sqlx::query_as("SELECT id,branch_id FROM public.registry_customers WHERE org_id=$1")
        .bind(f.target)
        .fetch_one(&f.owner)
        .await
        .unwrap()
}

fn same_rows(mut expected: Census, actual: &Census) {
    for rows in expected.values_mut() {
        rows.sort_by_key(Value::to_string);
    }
    assert_eq!(&expected, actual, "unexpected fixture or business effects");
}

#[derive(Clone, Copy)]
enum Change {
    Rename,
    Insert,
    Delete,
}

async fn adversarial_change(
    f: &Fixture,
    before: &Census,
    change: Change,
    child: Option<(Uuid, Uuid)>,
) -> Census {
    let slug = format!("renamed-{}", f.target.simple());
    let name = "입장 대기 중 변경된 시험 회사";
    let mut tx = f.owner.begin().await.unwrap();
    let changed = sqlx::query("UPDATE public.organizations SET slug=$2,name=$3 WHERE id=$1")
        .bind(f.target)
        .bind(&slug)
        .bind(name)
        .execute(tx.as_mut())
        .await
        .unwrap();
    assert_eq!(changed.rows_affected(), 1);
    let mut expected = before.clone();
    let organization = expected
        .get_mut("organizations")
        .unwrap()
        .iter_mut()
        .find(|row| row["id"] == f.target.to_string())
        .unwrap();
    organization["slug"] = serde_json::json!(slug);
    organization["name"] = serde_json::json!(name);
    match change {
        Change::Rename => assert!(child.is_none()),
        Change::Insert => {
            let (_, branch) = child.unwrap();
            let id = Uuid::new_v4();
            let inserted: Value = sqlx::query_scalar("INSERT INTO public.registry_customers(id,branch_id,name,org_id) VALUES($1,$2,'ADVERSARIAL RECEIPT FIXTURE',$3) RETURNING to_jsonb(registry_customers)")
                .bind(id).bind(branch).bind(f.target).fetch_one(tx.as_mut()).await.unwrap();
            assert_eq!(inserted["id"], id.to_string());
            assert_eq!(inserted["org_id"], f.target.to_string());
            assert_eq!(inserted["branch_id"], branch.to_string());
            expected
                .get_mut("registry_customers")
                .unwrap()
                .push(inserted);
        }
        Change::Delete => {
            let (id, _) = child.unwrap();
            let deleted =
                sqlx::query("DELETE FROM public.registry_customers WHERE id=$1 AND org_id=$2")
                    .bind(id)
                    .bind(f.target)
                    .execute(tx.as_mut())
                    .await
                    .unwrap();
            assert_eq!(deleted.rows_affected(), 1);
            let customers = expected.get_mut("registry_customers").unwrap();
            let count = customers.len();
            customers.retain(|row| row["id"] != id.to_string());
            assert_eq!(customers.len() + 1, count);
        }
    }
    tx.commit().await.unwrap();
    let admitted = census(&f.owner).await;
    same_rows(expected, &admitted);
    assert_ne!(before, &admitted, "fixture mutation must actually commit");
    admitted
}

#[sqlx::test(migrations = false)]
async fn receipts_observe_rename_and_child_changes_that_commit_before_admission(owner: PgPool) {
    console_platform_test_support::prepare_account_test_database(&owner).await;
    require_removal_keys(&owner).await;
    for (tag, force, change) in [
        ("receipt-normal", false, Change::Rename),
        ("receipt-insert", true, Change::Insert),
        ("receipt-delete", true, Change::Delete),
    ] {
        let fixture = Fixture::new(&owner, tag).await;
        let worker = Arc::clone(&fixture);
        finish(
            fixture,
            tokio::spawn(async move {
                if !force {
                    // Empty ordinary removal still permits ACTIVE; only force needs ARCHIVED.
                    expect(
                        &request(&worker, Kind::Reactivate, worker.target).await,
                        StatusCode::OK,
                        None,
                    );
                }
                let child = if force {
                    Some(customer(&worker).await)
                } else {
                    None
                };
                let before = census(&worker.owner).await;
                let (blocker, blocker_pid) = current_group_lock(&worker).await;
                let mut tasks = JoinSet::new();
                start(&mut tasks, &worker, Kind::Remove(force));
                let remover = waiting(&worker, &mut tasks, blocker_pid, Kind::Remove(force)).await;
                let admitted = adversarial_change(&worker, &before, change, child).await;
                assert!(still_waiting(&worker.owner, remover, blocker_pid).await);
                blocker.rollback().await.unwrap();
                let removed = next(&mut tasks).await;
                expect(&removed, StatusCode::NO_CONTENT, None);
                assert!(tasks.is_empty());
                assert_removed(&worker, &admitted, force).await;
            }),
        )
        .await;
    }
}

#[sqlx::test(migrations = false)]
async fn force_receipt_is_the_observed_snapshot_not_a_physical_delete_tally(owner: PgPool) {
    console_platform_test_support::prepare_account_test_database(&owner).await;
    require_removal_keys(&owner).await;
    let fixture = Fixture::new(&owner, "receipt-observed").await;
    let worker = Arc::clone(&fixture);
    finish(
        fixture,
        tokio::spawn(async move {
            let (id, _) = customer(&worker).await;
            let observed = census(&worker.owner).await;
            assert_eq!(
                count_in_org(&worker.owner, "registry_customers", worker.target).await,
                1
            );
            let (blocker, blocker_pid) = lock_row(&worker, HeldRow::TargetUser).await;
            let mut tasks = JoinSet::new();
            start(&mut tasks, &worker, Kind::Remove(true));
            // The real wrapper reads its receipt before this owning purge user lock.
            let remover = waiting(&worker, &mut tasks, blocker_pid, Kind::Remove(true)).await;
            let deleted =
                sqlx::query("DELETE FROM public.registry_customers WHERE id=$1 AND org_id=$2")
                    .bind(id)
                    .bind(worker.target)
                    .execute(&worker.owner)
                    .await
                    .unwrap();
            assert_eq!(deleted.rows_affected(), 1);
            let physical = census(&worker.owner).await;
            let mut expected = observed.clone();
            expected
                .get_mut("registry_customers")
                .unwrap()
                .retain(|row| row["id"] != id.to_string());
            same_rows(expected, &physical);
            assert_eq!(
                count_in_org(&worker.owner, "registry_customers", worker.target).await,
                0
            );
            assert!(still_waiting(&worker.owner, remover, blocker_pid).await);
            blocker.rollback().await.unwrap();
            let removed = next(&mut tasks).await;
            expect(&removed, StatusCode::NO_CONTENT, None);
            assert!(tasks.is_empty());
            // All target rows disappear either way; the receipt must retain count1.
            assert_removed(&worker, &observed, true).await;
        }),
    )
    .await;
}

#[sqlx::test(migrations = false)]
async fn changed_group_after_admission_wait_is_refused_without_chasing_the_new_group(
    owner: PgPool,
) {
    console_platform_test_support::prepare_account_test_database(&owner).await;
    require_removal_keys(&owner).await;
    for (force, tag) in [(false, "changed-normal"), (true, "changed-force")] {
        let fixture = Fixture::new(&owner, tag).await;
        let worker = Arc::clone(&fixture);
        finish(
            fixture,
            tokio::spawn(async move {
                if !force {
                    expect(
                        &request(&worker, Kind::Reactivate, worker.target).await,
                        StatusCode::OK,
                        None,
                    );
                }
                let before = census(&worker.owner).await;
                let (old_group, blocker_pid) = current_group_lock(&worker).await;
                let mut tasks = JoinSet::new();
                start(&mut tasks, &worker, Kind::Remove(force));
                let remover = waiting(&worker, &mut tasks, blocker_pid, Kind::Remove(force)).await;
                expect(
                    &request(&worker, Kind::Assign, worker.target).await,
                    StatusCode::OK,
                    None,
                );
                assert_transition(&worker, &before, Transition::Assign).await;
                let assigned = census(&worker.owner).await;
                assert!(still_waiting(&worker.owner, remover, blocker_pid).await);
                // Keep the new Group unavailable: a wait or timeout is not 409.
                let (new_group, _) = lock_row(&worker, HeldRow::DestinationGroup).await;
                old_group.rollback().await.unwrap();
                let refused = next(&mut tasks).await;
                expect(&refused, StatusCode::CONFLICT, Some("conflict"));
                assert!(tasks.is_empty());
                assert_eq!(census(&worker.owner).await, assigned);
                new_group.rollback().await.unwrap();
            }),
        )
        .await;
    }
}
