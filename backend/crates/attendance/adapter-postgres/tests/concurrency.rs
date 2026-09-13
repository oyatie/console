#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]
//! Deterministic two-connection regressions for attendance advisory locks.
//!
//! Each contender uses a max-one `console_rt` pool with a unique PostgreSQL
//! `application_name`. A control transaction holds the exact material used by
//! production until PostgreSQL witnesses both contender sessions waiting on
//! that same lock identity and reports the control backend in
//! `pg_blocking_pids`. Deadlines fail a test; they are not synchronization.

use console_attendance_adapter_postgres::{AttendanceStoreError, PgAttendanceStore};
use console_attendance_application::{AmendClose, AssignSubstitute, CallerScope, CloseMonth};
use console_attendance_domain::SubstitutionWindow;
use console_kernel_core::{BranchId, OrgId, UserId};
use console_platform_request_context::scope_org;
use console_platform_test_support::{runtime_role_pool, seed_branch, seed_user};
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Postgres, Transaction};
use time::{Date, Month};
use uuid::Uuid;

const CLOSE_MONTH: &str = "2026-07";
const SUBSTITUTION_KEY: &str = "attendance-substitution-race-key-0001";
const LOCK_WITNESS_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(10);

mod local_faults {
    //! Local owner-transaction conformance, not design30 command-receipt,
    //! Account-policy, replicated-durability or production-capacity proof.
    use super::*;
    use console_platform_test_support::{TestDatabaseLogin, login_test_database_url};
    use std::time::Duration;
    use tokio::task::JoinSet;

    enum StopPoint {
        OwnerLock,
        BeforeAuditInsert,
    }

    async fn restricted_pool(owner: &PgPool, name: &'static str) -> PgPool {
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .after_connect(move |connection, _| {
                Box::pin(async move {
                    sqlx::query("SELECT set_config('application_name',$1,false)")
                        .bind(name)
                        .execute(connection)
                        .await?;
                    Ok(())
                })
            })
            .connect(&login_test_database_url(owner, TestDatabaseLogin::Business))
            .await
            .expect("real console_rt LOGIN; no owner URL or SET ROLE");
        let identity: (String, String, bool, bool, bool, bool, bool) = sqlx::query_as(
            "SELECT session_user::text,current_user::text,rolsuper,rolbypassrls,rolcreaterole,rolcreatedb,rolreplication FROM pg_roles WHERE rolname=current_user",
        ).fetch_one(&pool).await.unwrap();
        assert_eq!(identity.0, "console_rt");
        assert_eq!(identity.1, "console_rt");
        assert!(!identity.2 && !identity.3 && !identity.4 && !identity.5 && !identity.6);
        pool
    }

    fn close_command(branch: BranchId) -> CloseMonth {
        CloseMonth {
            month: CLOSE_MONTH.to_owned(),
            branch_scope: Some(*branch.as_uuid()),
            attest: true,
        }
    }

    async fn effect_counts(owner: &PgPool, branch: BranchId) -> (i64, i64) {
        let effects: i64 = sqlx::query_scalar("SELECT count(*) FROM attendance_month_closes WHERE org_id=$1 AND branch_id=$2 AND month=DATE '2026-07-01'")
            .bind(OrgId::knl().as_uuid()).bind(branch.as_uuid()).fetch_one(owner).await.unwrap();
        let audits: i64 = sqlx::query_scalar("SELECT count(*) FROM audit_events WHERE org_id=$1 AND action='attendance.close.confirm'")
            .bind(OrgId::knl().as_uuid()).fetch_one(owner).await.unwrap();
        (effects, audits)
    }

    async fn install_audit_barrier(owner: &PgPool) {
        // Disposable observer-owned fault fixture. It only pauses the genuine
        // audit insert, after close_month staged its real business row in the
        // same with_audits transaction. It manufactures no effects or receipts.
        sqlx::query("CREATE FUNCTION console_test_pause_close_audit() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN PERFORM pg_advisory_xact_lock(hashtextextended('console-test-close-audit-barrier',0)); RETURN NEW; END $$")
            .execute(owner).await.unwrap();
        sqlx::query("CREATE TRIGGER console_test_pause_close_audit BEFORE INSERT ON audit_events FOR EACH ROW WHEN (NEW.action='attendance.close.confirm') EXECUTE FUNCTION console_test_pause_close_audit()")
            .execute(owner).await.unwrap();
    }

    async fn cancelled_close(owner: PgPool, point: StopPoint) {
        scope_org(OrgId::knl(), async move {
            let branch = seed_branch(&owner, "fault-close", "operations").await;
            let actor = seed_user(&owner, "Fault fixture closer", "ADMIN", branch).await;
            let caller = branch_caller(actor, branch);
            let pool = restricted_pool(&owner, "console-close-cancel-test").await;
            let contender = session(&pool).await;
            let material = match point {
                StopPoint::OwnerLock => format!("attendance-close-v1|{}|2026-07-01", OrgId::knl().as_uuid()),
                StopPoint::BeforeAuditInsert => {
                    install_audit_barrier(&owner).await;
                    "console-test-close-audit-barrier".to_owned()
                }
            };
            let mut gate = hold_exact_advisory_gate(&owner, &material).await;
            let gate_session = transaction_session(&mut gate).await;
            assert_ne!(gate_session.backend_pid, contender.backend_pid);
            let mut tasks = JoinSet::new();
            let store = PgAttendanceStore::new(pool.clone());
            let first_caller = caller.clone();
            tasks.spawn(async move { scope_org(OrgId::knl(), store.close_month(&first_caller, close_command(branch))).await });
            wait_for_exact_gate_waiter(&owner, &contender, &gate_session).await;
            if matches!(point, StopPoint::BeforeAuditInsert) {
                let in_audit_insert: bool = sqlx::query_scalar("SELECT query ILIKE '%INSERT INTO audit_events%' AND xact_start IS NOT NULL FROM pg_stat_activity WHERE pid=$1 AND application_name=$2")
                    .bind(contender.backend_pid).bind(&contender.application_name).fetch_one(&owner).await.unwrap();
                assert!(in_audit_insert, "fault must be inside actual audit insertion, not before business staging");
            }
            assert_eq!(effect_counts(&owner, branch).await, (0, 0), "independent observer cannot see uncommitted work");
            tasks.abort_all();
            let result = tokio::time::timeout(Duration::from_secs(5), tasks.join_next()).await
                .expect("owned task cancellation must terminate").expect("one owned task");
            assert!(result.expect_err("cancelled owner call must not return success").is_cancelled());
            assert!(tasks.is_empty());
            // Dropping the Rust future is not a database rollback witness.
            // Release the server barrier, drain SQLx's pending rollback, then
            // check fresh reads and real LOGIN pool reuse independently.
            gate.rollback().await.unwrap();
            tokio::time::timeout(Duration::from_secs(5), async {
                let mut connection = pool.acquire().await.unwrap();
                let identity: (String, String) = sqlx::query_as("SELECT session_user::text,current_user::text")
                    .fetch_one(&mut *connection).await.unwrap();
                assert_eq!(identity, ("console_rt".to_owned(), "console_rt".to_owned()));
            }).await.expect("pool must drain cancelled transaction and remain usable");
            assert_eq!(effect_counts(&owner, branch).await, (0, 0), "cancelled work leaves neither business row nor audit");
            let store = PgAttendanceStore::new(pool.clone());
            let completed = store.close_month(&caller, close_command(branch)).await.unwrap();
            assert_eq!(completed.branch_id, Some(*branch.as_uuid()));
            assert_eq!(effect_counts(&owner, branch).await, (1, 1), "valid retry commits one effect and one audit");
            pool.close().await;
        }).await;
    }

    #[sqlx::test(migrations = "../../platform/db/migrations")]
    async fn cancelled_exact_owner_lock_wait_has_no_effect_and_login_pool_retries(owner: PgPool) {
        cancelled_close(owner, StopPoint::OwnerLock).await;
    }

    #[sqlx::test(migrations = "../../platform/db/migrations")]
    async fn cancelled_staged_close_before_audit_commit_rolls_back_effect_and_audit(owner: PgPool) {
        cancelled_close(owner, StopPoint::BeforeAuditInsert).await;
    }

    #[sqlx::test(migrations = "../../platform/db/migrations")]
    async fn suppressed_postcommit_response_reconciles_real_close_without_second_effect(
        owner: PgPool,
    ) {
        scope_org(OrgId::knl(), async move {
            let branch = seed_branch(&owner, "response-loss", "operations").await;
            let actor = seed_user(&owner, "Response fixture closer", "ADMIN", branch).await;
            let caller = branch_caller(actor, branch);
            let pool = restricted_pool(&owner, "console-close-response-test").await;
            let store = PgAttendanceStore::new(pool.clone());
            // Deliberately suppress a completed application's response. This
            // is not a network cut during COMMIT or a replica recovery test.
            drop(
                store
                    .close_month(&caller, close_command(branch))
                    .await
                    .unwrap(),
            );
            pool.close().await;
            let reader = restricted_pool(&owner, "console-close-reconcile-test").await;
            let restored = PgAttendanceStore::new(reader.clone());
            let rows = restored
                .list_closes(
                    &caller,
                    Some(*branch.as_uuid()),
                    Date::from_calendar_date(2026, Month::July, 1).unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(
                rows.len(),
                1,
                "fresh permitted owner read resolves the durable effect"
            );
            assert_eq!(rows[0].attested_by, *actor.as_uuid());
            assert_eq!(effect_counts(&owner, branch).await, (1, 1));
            let duplicate = restored.close_month(&caller, close_command(branch)).await;
            assert!(
                matches!(duplicate, Err(AttendanceStoreError::CloseBlocked)),
                "existing close owner refuses a duplicate; it has no design30 replay receipt yet"
            );
            assert_eq!(effect_counts(&owner, branch).await, (1, 1));
            reader.close().await;
        })
        .await;
    }
}


#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn concurrent_branch_month_closes_commit_one_snapshot_and_one_audit(owner_pool: PgPool) {
    scope_org(OrgId::knl(), async move {
        let branch = seed_branch(&owner_pool, "attendance-close-race", "operations").await;
        let actor = seed_user(&owner_pool, "Attendance Closer", "ADMIN", branch).await;
        let caller = branch_caller(actor, branch);
        let first_pool = one_connection_runtime_pool(&owner_pool, "attendance-close-first").await;
        let second_pool = one_connection_runtime_pool(&owner_pool, "attendance-close-second").await;
        let first = contender_session(&first_pool).await;
        let second = contender_session(&second_pool).await;
        let command = CloseMonth {
            month: CLOSE_MONTH.to_owned(),
            branch_scope: Some(*branch.as_uuid()),
            attest: true,
        };
        let month = Date::from_calendar_date(2026, Month::July, 1).unwrap();
        let mut gate = hold_exact_advisory_gate(
            &owner_pool,
            &format!("attendance-close-v1|{}|{month}", OrgId::knl().as_uuid()),
        )
        .await;
        let gate_session = transaction_session(&mut gate).await;
        assert_distinct_sessions(&gate_session, &first, &second);

        let first_task = spawn_close(PgAttendanceStore::new(first_pool), caller.clone(), command.clone());
        let second_task = spawn_close(PgAttendanceStore::new(second_pool), caller, command);
        wait_for_exact_gate_waiter(&owner_pool, &first, &gate_session).await;
        wait_for_exact_gate_waiter(&owner_pool, &second, &gate_session).await;
        gate.commit().await.unwrap();

        let (first_result, second_result) = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            async { tokio::join!(first_task, second_task) },
        )
        .await
        .expect("month-close contenders must finish promptly once the exact production lock is released");
        let first_result = first_result.expect("first close task must not panic");
        let second_result = second_result.expect("second close task must not panic");
        let winner = match (first_result, second_result) {
            (Ok(winner), Err(AttendanceStoreError::CloseBlocked))
            | (Err(AttendanceStoreError::CloseBlocked), Ok(winner)) => winner,
            (left, right) => panic!(
                "expected exactly one close and one CloseBlocked outcome, got {left:?} / {right:?}"
            ),
        };
        assert_eq!(winner.branch_id, Some(*branch.as_uuid()));

        let close_count: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM attendance_month_closes WHERE org_id=$1 AND month=DATE '2026-07-01' AND branch_id=$2",
        )
        .bind(*OrgId::knl().as_uuid())
        .bind(*branch.as_uuid())
        .fetch_one(&owner_pool)
        .await
        .unwrap();
        let audit_count: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM audit_events WHERE org_id=$1 AND action='attendance.close.confirm'",
        )
        .bind(*OrgId::knl().as_uuid())
        .fetch_one(&owner_pool)
        .await
        .unwrap();
        assert_eq!(close_count, 1, "one branch close snapshot is durable");
        assert_eq!(audit_count, 1, "only the committed close emits an audit");
    })
    .await;
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn concurrent_identical_substitutions_replay_once_and_changed_immutable_payload_conflicts(
    owner_pool: PgPool,
) {
    scope_org(OrgId::knl(), async move {
        let branch = seed_branch(&owner_pool, "attendance-substitution-race", "operations").await;
        let provisioner = seed_user(
            &owner_pool,
            "Attendance Employee Provisioner",
            "SUPER_ADMIN",
            branch,
        )
        .await;
        let actor = seed_user(&owner_pool, "Attendance Scheduler", "ADMIN", branch).await;
        let covered = seed_employee(&owner_pool, branch, provisioner, "Covered employee").await;
        let worker = seed_employee(&owner_pool, branch, provisioner, "Eligible worker").await;
        seed_approved_leave(&owner_pool, branch, provisioner, actor, covered).await;
        let caller = branch_caller(actor, branch);
        let command = substitution_command(covered, worker, branch);
        let first_pool = one_connection_runtime_pool(&owner_pool, "attendance-substitution-first").await;
        let second_pool = one_connection_runtime_pool(&owner_pool, "attendance-substitution-second").await;
        let first = contender_session(&first_pool).await;
        let second = contender_session(&second_pool).await;
        let mut gate = hold_exact_advisory_gate(
            &owner_pool,
            &format!(
                "attendance-idempotency-v1|{}|{}|{SUBSTITUTION_KEY}",
                OrgId::knl().as_uuid(),
                SUBSTITUTION_KEY.len(),
            ),
        )
        .await;
        let gate_session = transaction_session(&mut gate).await;
        assert_distinct_sessions(&gate_session, &first, &second);

        let first_task = spawn_substitution(PgAttendanceStore::new(first_pool), caller.clone(), command.clone());
        let second_task = spawn_substitution(PgAttendanceStore::new(second_pool), caller.clone(), command.clone());
        wait_for_exact_gate_waiter(&owner_pool, &first, &gate_session).await;
        wait_for_exact_gate_waiter(&owner_pool, &second, &gate_session).await;
        gate.commit().await.unwrap();

        let (first_result, second_result) = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            async { tokio::join!(first_task, second_task) },
        )
        .await
        .expect("substitution contenders must finish promptly once the exact production lock is released");
        let first_result = first_result
            .expect("first substitution task must not panic")
            .expect("the first identical substitution must succeed");
        let second_result = second_result
            .expect("second substitution task must not panic")
            .expect("the second identical substitution must replay");
        assert_eq!(first_result.id, second_result.id, "identical calls replay one substitution");

        let substitution_count: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM attendance_substitutions WHERE org_id=$1 AND idempotency_key=$2",
        )
        .bind(*OrgId::knl().as_uuid())
        .bind(SUBSTITUTION_KEY)
        .fetch_one(&owner_pool)
        .await
        .unwrap();
        let audit_count: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM audit_events WHERE org_id=$1 AND action='attendance.substitution.assign'",
        )
        .bind(*OrgId::knl().as_uuid())
        .fetch_one(&owner_pool)
        .await
        .unwrap();
        assert_eq!(substitution_count, 1, "one substitution row is durable");
        assert_eq!(audit_count, 1, "only the new assignment emits an audit");

        let verification_store = PgAttendanceStore::new(
            one_connection_runtime_pool(&owner_pool, "attendance-substitution-verify").await,
        );
        let mut changed = command;
        changed.role = "Changed immutable role".to_owned();
        let mismatch = verification_store
            .assign_substitute(&caller, changed)
            .await
            .expect_err("reusing an idempotency key with a changed immutable request must conflict");
        assert!(matches!(mismatch, AttendanceStoreError::Conflict));

        let substitutions_after: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM attendance_substitutions WHERE org_id=$1 AND idempotency_key=$2",
        )
        .bind(*OrgId::knl().as_uuid())
        .bind(SUBSTITUTION_KEY)
        .fetch_one(&owner_pool)
        .await
        .unwrap();
        let audits_after: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM audit_events WHERE org_id=$1 AND action='attendance.substitution.assign'",
        )
        .bind(*OrgId::knl().as_uuid())
        .fetch_one(&owner_pool)
        .await
        .unwrap();
        assert_eq!(substitutions_after, 1, "mismatch cannot create another substitution");
        assert_eq!(audits_after, 1, "mismatch cannot create another audit");
    })
    .await;
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn close_amendment_uses_runtime_role_without_updating_immutable_close_and_replays_idempotently(
    owner_pool: PgPool,
) {
    scope_org(OrgId::knl(), async move {
        let branch = seed_branch(&owner_pool, "attendance-amendment", "operations").await;
        let actor = seed_user(&owner_pool, "Attendance Amendment Manager", "ADMIN", branch).await;
        let caller = branch_caller(actor, branch);
        let store = PgAttendanceStore::new(runtime_role_pool(&owner_pool).await);
        let close = store
            .close_month(
                &caller,
                CloseMonth {
                    month: CLOSE_MONTH.to_owned(),
                    branch_scope: Some(*branch.as_uuid()),
                    attest: true,
                },
            )
            .await
            .expect("runtime role must create the immutable branch close snapshot");
        let command = AmendClose {
            close_id: close.id,
            reason: "approved correction".to_owned(),
            detail: "manager corrected the source evidence after close".to_owned(),
            reference: Some("ATTENDANCE-31".to_owned()),
            idempotency_key: "attendance-close-amendment-0001".to_owned(),
        };

        let amendment = store
            .amend_close(&caller, command.clone())
            .await
            .expect("runtime role must append an amendment without updating the immutable close");
        let replay = store
            .amend_close(&caller, command.clone())
            .await
            .expect("an exact amendment replay must return the original immutable amendment");
        assert_eq!(replay.id, amendment.id, "exact replay returns the one amendment");

        let amendment_count: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM attendance_close_amendments WHERE org_id=$1 AND close_id=$2",
        )
        .bind(*OrgId::knl().as_uuid())
        .bind(close.id)
        .fetch_one(&owner_pool)
        .await
        .unwrap();
        let audit_count: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM audit_events WHERE org_id=$1 AND action='attendance.close.amend' AND target_id=$2",
        )
        .bind(*OrgId::knl().as_uuid())
        .bind(close.id.to_string())
        .fetch_one(&owner_pool)
        .await
        .unwrap();
        assert_eq!(amendment_count, 1, "one immutable amendment is durable");
        assert_eq!(audit_count, 1, "only the initial append emits an audit");

        let mut changed = command;
        changed.detail = "changed immutable amendment detail".to_owned();
        let conflict = store
            .amend_close(&caller, changed)
            .await
            .expect_err("changing a replay payload must conflict");
        assert!(matches!(conflict, AttendanceStoreError::Conflict));

        let amendments_after: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM attendance_close_amendments WHERE org_id=$1 AND close_id=$2",
        )
        .bind(*OrgId::knl().as_uuid())
        .bind(close.id)
        .fetch_one(&owner_pool)
        .await
        .unwrap();
        let audits_after: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM audit_events WHERE org_id=$1 AND action='attendance.close.amend' AND target_id=$2",
        )
        .bind(*OrgId::knl().as_uuid())
        .bind(close.id.to_string())
        .fetch_one(&owner_pool)
        .await
        .unwrap();
        assert_eq!(amendments_after, 1, "conflicting replay cannot append another amendment");
        assert_eq!(audits_after, 1, "conflicting replay cannot emit another audit");
    })
    .await;
}

fn branch_caller(actor: UserId, branch: BranchId) -> CallerScope {
    CallerScope {
        org_id: *OrgId::knl().as_uuid(),
        user_id: *actor.as_uuid(),
        branch_ids: vec![*branch.as_uuid()],
        org_wide: false,
    }
}

struct DatabaseSession {
    backend_pid: i32,
    application_name: String,
}

fn assert_distinct_sessions(
    gate: &DatabaseSession,
    first_contender: &DatabaseSession,
    second_contender: &DatabaseSession,
) {
    assert_ne!(
        gate.backend_pid, first_contender.backend_pid,
        "gate and first contender must use distinct PostgreSQL backends"
    );
    assert_ne!(
        gate.backend_pid, second_contender.backend_pid,
        "gate and second contender must use distinct PostgreSQL backends"
    );
    assert_ne!(
        first_contender.backend_pid, second_contender.backend_pid,
        "the contenders must use distinct PostgreSQL backends"
    );
    assert_ne!(
        gate.application_name, first_contender.application_name,
        "gate and first contender must have distinct application names"
    );
    assert_ne!(
        gate.application_name, second_contender.application_name,
        "gate and second contender must have distinct application names"
    );
    assert_ne!(
        first_contender.application_name, second_contender.application_name,
        "the contenders must have distinct application names"
    );
}

async fn one_connection_runtime_pool(owner_pool: &PgPool, application_name: &str) -> PgPool {
    let application_name = application_name.to_owned();
    PgPoolOptions::new()
        .max_connections(1)
        .after_connect(move |connection, _| {
            let application_name = application_name.clone();
            Box::pin(async move {
                sqlx::query("SELECT set_config('application_name', $1, false)")
                    .bind(application_name)
                    .execute(&mut *connection)
                    .await?;
                sqlx::query("SET ROLE console_rt")
                    .execute(&mut *connection)
                    .await?;
                Ok(())
            })
        })
        .connect_with(owner_pool.connect_options().as_ref().clone())
        .await
        .expect("connect max-one console_rt contender pool")
}

async fn contender_session(pool: &PgPool) -> DatabaseSession {
    sqlx::query("SELECT set_config('app.current_org', $1, false)")
        .bind(OrgId::knl().as_uuid().to_string())
        .execute(pool)
        .await
        .unwrap();
    session(pool).await
}

async fn transaction_session(tx: &mut Transaction<'_, Postgres>) -> DatabaseSession {
    let (backend_pid, application_name): (i32, String) =
        sqlx::query_as("SELECT pg_backend_pid(), current_setting('application_name')")
            .fetch_one(tx.as_mut())
            .await
            .unwrap();
    DatabaseSession {
        backend_pid,
        application_name,
    }
}

async fn session(pool: &PgPool) -> DatabaseSession {
    let (backend_pid, application_name): (i32, String) =
        sqlx::query_as("SELECT pg_backend_pid(), current_setting('application_name')")
            .fetch_one(pool)
            .await
            .unwrap();
    DatabaseSession {
        backend_pid,
        application_name,
    }
}

async fn hold_exact_advisory_gate<'a>(
    owner_pool: &'a PgPool,
    material: &str,
) -> Transaction<'a, Postgres> {
    let mut gate = owner_pool.begin().await.unwrap();
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
        .bind(material)
        .execute(gate.as_mut())
        .await
        .unwrap();
    gate
}

async fn wait_for_exact_gate_waiter(
    observer_pool: &PgPool,
    contender: &DatabaseSession,
    gate: &DatabaseSession,
) {
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        let waiting: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM pg_stat_activity contender_activity JOIN pg_stat_activity gate_activity ON gate_activity.pid=$3 AND gate_activity.application_name=$4 JOIN pg_locks contender_lock ON contender_lock.pid=contender_activity.pid AND contender_lock.locktype='advisory' AND NOT contender_lock.granted JOIN pg_locks gate_lock ON gate_lock.pid=gate_activity.pid AND gate_lock.locktype='advisory' AND gate_lock.granted AND gate_lock.database IS NOT DISTINCT FROM contender_lock.database AND gate_lock.classid=contender_lock.classid AND gate_lock.objid=contender_lock.objid AND gate_lock.objsubid=contender_lock.objsubid WHERE contender_activity.pid=$1 AND contender_activity.application_name=$2 AND contender_activity.wait_event_type='Lock' AND contender_activity.wait_event='advisory' AND $3 = ANY(pg_blocking_pids(contender_activity.pid)))",
        )
        .bind(contender.backend_pid)
        .bind(&contender.application_name)
        .bind(gate.backend_pid)
        .bind(&gate.application_name)
        .fetch_one(observer_pool)
        .await
        .unwrap();
        if waiting {
            return;
        }
        if tokio::time::Instant::now() >= deadline {
            let blockers: Vec<i32> = sqlx::query_scalar("SELECT pg_blocking_pids($1)")
                .bind(contender.backend_pid)
                .fetch_one(observer_pool)
                .await
                .unwrap();
            panic!(
                "contender {} ({}) never waited on the exact advisory lock held by gate {} ({}); blockers: {:?}",
                contender.backend_pid,
                contender.application_name,
                gate.backend_pid,
                gate.application_name,
                blockers,
            );
        }
        tokio::time::sleep(LOCK_WITNESS_POLL_INTERVAL).await;
    }
}

fn spawn_close(
    store: PgAttendanceStore,
    caller: CallerScope,
    command: CloseMonth,
) -> tokio::task::JoinHandle<
    Result<console_attendance_application::MonthCloseRead, AttendanceStoreError>,
> {
    tokio::spawn(async move { scope_org(OrgId::knl(), store.close_month(&caller, command)).await })
}

fn spawn_substitution(
    store: PgAttendanceStore,
    caller: CallerScope,
    command: AssignSubstitute,
) -> tokio::task::JoinHandle<
    Result<console_attendance_application::AttendanceSubstitutionRead, AttendanceStoreError>,
> {
    tokio::spawn(
        async move { scope_org(OrgId::knl(), store.assign_substitute(&caller, command)).await },
    )
}

fn substitution_command(
    covered_employee_id: Uuid,
    worker_employee_id: Uuid,
    branch: BranchId,
) -> AssignSubstitute {
    AssignSubstitute {
        window: SubstitutionWindow::new(
            Date::from_calendar_date(2026, Month::July, 2).unwrap(),
            480,
            960,
        )
        .unwrap(),
        branch_id: Some(*branch.as_uuid()),
        site: "Seoul depot".to_owned(),
        role: "Forklift operator".to_owned(),
        covered_employee_id,
        reason_kind: "APPROVED_LEAVE".to_owned(),
        reason_detail: Some("approved cover".to_owned()),
        worker_employee_id: Some(worker_employee_id),
        worker_name: "untrusted client snapshot".to_owned(),
        worker_type: "untrusted client snapshot".to_owned(),
        worker_rate: Some("untrusted client rate".to_owned()),
        exception_id: None,
        idempotency_key: SUBSTITUTION_KEY.to_owned(),
    }
}

async fn seed_approved_leave(
    pool: &PgPool,
    branch: BranchId,
    requester: UserId,
    decider: UserId,
    employee: Uuid,
) {
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SET LOCAL ROLE console_leave_definer")
        .execute(tx.as_mut())
        .await
        .unwrap();
    sqlx::query("SELECT set_config('app.current_org', $1, true)")
        .bind(OrgId::knl().as_uuid().to_string())
        .execute(tx.as_mut())
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO leave_requests (id,org_id,branch_id,requester_user_id,subject_employee_id,leave_type,days,start_date,end_date,status,decided_by,decided_at,charge_state,charge_review_reasons,charge_units) VALUES ($1,$2,$3,$4,$5,'annual',1,DATE '2026-07-02',DATE '2026-07-02','approved',$6,now(),'legacy_unverified',ARRAY[]::text[],1)",
    )
    .bind(Uuid::new_v4())
    .bind(*OrgId::knl().as_uuid())
    .bind(*branch.as_uuid())
    .bind(*requester.as_uuid())
    .bind(employee)
    .bind(*decider.as_uuid())
    .execute(tx.as_mut())
    .await
    .unwrap();
    tx.commit().await.unwrap();
}

async fn seed_employee(pool: &PgPool, branch: BranchId, actor: UserId, name: &str) -> Uuid {
    let employee = Uuid::new_v4();
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SET LOCAL ROLE console_leave_cmd")
        .execute(tx.as_mut())
        .await
        .unwrap();
    sqlx::query(
        "SELECT * FROM leave_api.create_employee($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,0::numeric,$12,$13,$14,$15,$16)",
    )
    .bind(*OrgId::knl().as_uuid())
    .bind(employee)
    .bind(format!("ATT-RACE-{employee}"))
    .bind(name)
    .bind("attendance-race")
    .bind("REGULAR")
    .bind("+821012345678")
    .bind("Operations")
    .bind("Forklift operator")
    .bind("Seoul depot")
    .bind(*branch.as_uuid())
    .bind(format!("attendance-race-{employee}"))
    .bind("a".repeat(64))
    .bind(*actor.as_uuid())
    .bind("0123456789abcdef0123456789abcdef")
    .bind("0123456789abcdef")
    .fetch_one(tx.as_mut())
    .await
    .unwrap();
    tx.commit().await.unwrap();
    employee
}
