//! Actual Account-only topology writers, HTTP commands, and PostgreSQL lock witnesses.
//! Missing owner key privileges are prerequisites, not proof of a race defect.

use super::*;
use effects::*;
use std::sync::Arc;
use support::*;
use tokio::task::JoinSet;

#[path = "native_removal_races/effects.rs"]
mod effects;
#[path = "native_removal_races/support.rs"]
mod support;

#[sqlx::test(migrations = false)]
async fn absent_company_status_and_assignment_are_known_not_found(owner: PgPool) {
    console_platform_test_support::prepare_account_test_database(&owner).await;
    // This contract probe deliberately precedes the removal-key prerequisite.
    // Both published mutation operations declare 404 for an absent Company.
    let fixture = Fixture::new(&owner, "absent").await;
    let worker = Arc::clone(&fixture);
    finish(
        fixture,
        tokio::spawn(async move {
            let missing = Uuid::new_v4();
            assert!(!org_exists(&worker.owner, missing).await);
            let before = census(&worker.owner).await;
            let mut failures = Vec::new();
            for kind in [Kind::Reactivate, Kind::Assign] {
                let reply = request(&worker, kind, missing).await;
                if reply.status != StatusCode::NOT_FOUND
                    || reply.body["error"]["code"] != "not_found"
                {
                    failures.push(format!(
                        "{kind:?}: expected known404/not_found, got {} / {}",
                        reply.status, reply.body["error"]["code"]
                    ));
                }
                if census(&worker.owner).await != before {
                    failures.push(format!("{kind:?}: absent-target request changed rows"));
                }
            }
            assert!(failures.is_empty(), "{}", failures.join("\n"));
        }),
    )
    .await;
}

#[sqlx::test(migrations = false)]
async fn reactivation_before_force_removal_is_rechecked_after_the_lock_wait(owner: PgPool) {
    console_platform_test_support::prepare_account_test_database(&owner).await;
    require_removal_keys(&owner).await;
    let fixture = Fixture::new(&owner, "status-first").await;
    let worker = Arc::clone(&fixture);
    finish(
        fixture,
        tokio::spawn(async move {
            let before = census(&worker.owner).await;
            let (blocker, blocker_pid) = lock_row(&worker, HeldRow::Company).await;
            let mut tasks = JoinSet::new();
            start(&mut tasks, &worker, Kind::Reactivate);
            let status_pid = waiting(&worker, &mut tasks, blocker_pid, Kind::Reactivate).await;
            start(&mut tasks, &worker, Kind::Remove(true));
            waiting(&worker, &mut tasks, status_pid, Kind::Remove(true)).await;
            blocker.rollback().await.unwrap();
            let replies = drain(&mut tasks).await;
            expect(&replies[&Kind::Reactivate], StatusCode::OK, None);
            expect(
                &replies[&Kind::Remove(true)],
                StatusCode::CONFLICT,
                Some("tenant_active"),
            );
            assert_transition(&worker, &before, Transition::Reactivate).await;
        }),
    )
    .await;
}

#[sqlx::test(migrations = false)]
async fn force_removal_before_reactivation_preserves_absence_and_foreign_state(owner: PgPool) {
    console_platform_test_support::prepare_account_test_database(&owner).await;
    require_removal_keys(&owner).await;
    let fixture = Fixture::new(&owner, "remove-status").await;
    let worker = Arc::clone(&fixture);
    finish(
        fixture,
        tokio::spawn(async move {
            let before = census(&worker.owner).await;
            let (blocker, blocker_pid) = lock_row(&worker, HeldRow::Company).await;
            let mut tasks = JoinSet::new();
            start(&mut tasks, &worker, Kind::Remove(true));
            let removal_pid = waiting(&worker, &mut tasks, blocker_pid, Kind::Remove(true)).await;
            start(&mut tasks, &worker, Kind::Reactivate);
            waiting(&worker, &mut tasks, removal_pid, Kind::Reactivate).await;
            blocker.rollback().await.unwrap();
            let replies = drain(&mut tasks).await;
            expect(&replies[&Kind::Remove(true)], StatusCode::NO_CONTENT, None);
            expect(
                &replies[&Kind::Reactivate],
                StatusCode::NOT_FOUND,
                Some("not_found"),
            );
            assert_removed(&worker, &before, true).await;
        }),
    )
    .await;
}

#[sqlx::test(migrations = false)]
async fn claimed_membership_refuses_removal_without_waiting_or_losing_assignment(owner: PgPool) {
    console_platform_test_support::prepare_account_test_database(&owner).await;
    require_removal_keys(&owner).await;
    for force in [false, true] {
        let fixture = Fixture::new(
            &owner,
            if force {
                "member-force"
            } else {
                "member-plain"
            },
        )
        .await;
        let worker = Arc::clone(&fixture);
        finish(
            fixture,
            tokio::spawn(async move {
                let before = census(&worker.owner).await;
                // The real assignment owns its membership UPDATE before its FK
                // check waits for this destination Group row. No fake writer runs.
                let (blocker, blocker_pid) = lock_row(&worker, HeldRow::DestinationGroup).await;
                let mut tasks = JoinSet::new();
                start(&mut tasks, &worker, Kind::Assign);
                let assignment_pid = waiting(&worker, &mut tasks, blocker_pid, Kind::Assign).await;
                membership_is_locked(&worker).await;
                start(&mut tasks, &worker, Kind::Remove(force));
                let refused = next(&mut tasks).await;
                assert_eq!(refused.kind, Kind::Remove(force));
                expect(&refused, StatusCode::CONFLICT, Some("conflict"));
                assert!(
                    census(&worker.owner).await == before,
                    "refused removal changed committed rows"
                );
                assert!(still_waiting(&worker.owner, assignment_pid, blocker_pid).await);
                blocker.rollback().await.unwrap();
                let assigned = next(&mut tasks).await;
                assert_eq!(assigned.kind, Kind::Assign);
                expect(&assigned, StatusCode::OK, None);
                assert!(tasks.is_empty());
                assert_transition(&worker, &before, Transition::Assign).await;
            }),
        )
        .await;
    }
}

#[sqlx::test(migrations = false)]
async fn removal_retains_membership_against_late_assignment_through_its_commit(owner: PgPool) {
    console_platform_test_support::prepare_account_test_database(&owner).await;
    require_removal_keys(&owner).await;
    for force in [false, true] {
        let fixture = Fixture::new(
            &owner,
            if force {
                "remove-force"
            } else {
                "remove-plain"
            },
        )
        .await;
        let worker = Arc::clone(&fixture);
        finish(
            fixture,
            tokio::spawn(async move {
                let before = census(&worker.owner).await;
                // Purge's existing user lock follows topology admission. Blocking
                // it exposes whether the remover already retained membership.
                let (blocker, blocker_pid) = lock_row(&worker, HeldRow::TargetUser).await;
                let mut tasks = JoinSet::new();
                start(&mut tasks, &worker, Kind::Remove(force));
                let removal_pid =
                    waiting(&worker, &mut tasks, blocker_pid, Kind::Remove(force)).await;
                start(&mut tasks, &worker, Kind::Assign);
                waiting(&worker, &mut tasks, removal_pid, Kind::Assign).await;
                blocker.rollback().await.unwrap();
                let replies = drain(&mut tasks).await;
                expect(&replies[&Kind::Remove(force)], StatusCode::NO_CONTENT, None);
                expect(
                    &replies[&Kind::Assign],
                    StatusCode::NOT_FOUND,
                    Some("not_found"),
                );
                assert_removed(&worker, &before, force).await;
            }),
        )
        .await;
    }
}
