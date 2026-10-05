// Generated navigation-finalizer verification; all historical Group diagnostics
// remain frozen. Each leaf requires a root-leased marked disposable cluster.
const NAVIGATION_FINALIZER: &str =
    include_str!("../../../../ops/postgres-finalize-native-group-process-navigation-v1.sql");
const NAVIGATION_APP_STATE: &str =
    include_str!("../../src/native_group_process_navigation_v1_custody_state.sql");
const NAVIGATION_CORRECTED83: [&str; 2] = [
    "3f5972d2e5c1d7277e71b4f716b79a405d152ca5bbab25b614dbc1fc67c5fd7f",
    "304e176d646edf62e8767a9f986879abc74753a3b2a8812c1a8cec8447a20b78",
];

fn navigation_finalizer_pins() -> Value {
    let mut sources = group_pins();
    for (name, source, expected) in [
        (
            "navigation_finalizer",
            NAVIGATION_FINALIZER,
            "4971a73e957da8dfe4f202aea3693d3d0ad0636b92a0a2785264c5d4f9d9fc70",
        ),
        (
            "navigation_state",
            NAVIGATION_APP_STATE,
            "6a7a721c434486582ffba11d79a818088c3f3a41b93248b4bb1504ecd484dc39",
        ),
    ] {
        assert_eq!(
            digest(source),
            expected,
            "unreviewed navigation input: {name}"
        );
        assert!(
            sources
                .as_object_mut()
                .unwrap()
                .insert(name.into(), json!(expected))
                .is_none()
        );
    }
    let correction = navigation_correction();
    assert_eq!(
        NAVIGATION_FINALIZER
            .matches(correction.corrected.as_str())
            .count(),
        1
    );
    assert_eq!(
        digest(&correction.corrected),
        "83164dd8ac21cc30bbb7e6ed07c37fa184b148c4377fde22414740483cdb2f90"
    );
    sources
}

async fn navigation_classification(connection: &mut PgConnection) -> (String, Option<String>) {
    sqlx::raw_sql(CLASSIFIER_SESSION)
        .execute(&mut *connection)
        .await
        .unwrap();
    let body = NAVIGATION_APP_STATE
        .strip_suffix(" AS state;\n")
        .expect("pinned navigation classifier terminator");
    let query = format!("{body} AS state,(SELECT variant FROM matching_phase) AS variant");
    sqlx::query_as(sqlx::AssertSqlSafe(query))
        .fetch_one(connection)
        .await
        .unwrap()
}

fn navigation_phase(corrected: bool, variant: usize) -> (String, Option<String>) {
    assert!(variant < 2);
    (
        if corrected {
            "native_group_process_navigation.finalized"
        } else {
            "native_group_process_navigation.head_revision_required"
        }
        .into(),
        Some(if variant == 0 { "plain" } else { "observer" }.into()),
    )
}

// The complete catalog deliberately excludes MVCC tuple bookkeeping. Retaining
// xmin/ctid independently detects redundant CREATE OR REPLACE on a replay.
async fn navigation_tuple(connection: &mut PgConnection) -> (String, String) {
    sqlx::query_as(
        "SELECT xmin::text,ctid::text FROM pg_catalog.pg_proc WHERE oid=$1::regprocedure",
    )
    .bind(NAVIGATION_IDENTITY)
    .fetch_one(connection)
    .await
    .unwrap()
}

struct NavigationRuntimeBaseline {
    target: Target,
    capture: Capture,
    catalog: (String, Value),
    rows: BTreeMap<String, String>,
    ledger: Value,
    tuple: (String, String),
}

async fn navigation_snapshot(
    connection: &mut PgConnection,
    corrected: bool,
    variant: usize,
) -> NavigationRuntimeBaseline {
    let frozen = target(connection).await;
    assert_eq!(
        navigation_classification(connection).await,
        navigation_phase(corrected, variant)
    );
    let captured = group_capture(connection, true).await;
    assert_eq!(
        captured.sha256,
        if corrected {
            NAVIGATION_CORRECTED83[variant]
        } else {
            INSTALLED83[variant]
        }
    );
    NavigationRuntimeBaseline {
        target: frozen,
        capture: captured,
        catalog: catalog(connection).await,
        rows: rows(connection).await,
        ledger: applied_ledger(connection).await,
        tuple: navigation_tuple(connection).await,
    }
}

async fn navigation_restored(
    pool: &PgPool,
    baseline: &NavigationRuntimeBaseline,
    corrected: bool,
    variant: usize,
) {
    let mut fresh = direct(pool).await;
    assert_eq!(target(&mut fresh).await, baseline.target);
    let mut tx = sqlx::Connection::begin(&mut fresh).await.unwrap();
    let actual = navigation_snapshot(tx.as_mut(), corrected, variant).await;
    assert_eq!(actual.target, baseline.target);
    assert!(
        actual.capture == baseline.capture,
        "fresh readback changed exact full83"
    );
    assert!(
        actual.catalog == baseline.catalog,
        "fresh readback changed complete raw catalog"
    );
    assert!(
        actual.rows == baseline.rows,
        "fresh readback changed any durable row"
    );
    assert!(
        actual.ledger == baseline.ledger,
        "fresh readback changed complete231 ledger"
    );
    assert_eq!(
        actual.tuple, baseline.tuple,
        "fresh readback changed navigation tuple identity"
    );
    tx.rollback().await.unwrap();
    fresh.close().await.unwrap();
}

async fn navigation_original_fixture(pool: &PgPool, variant: usize) -> NavigationRuntimeBaseline {
    navigation_finalizer_pins();
    let oracle = group_oracle();
    let original = predecessor(pool, variant).await;
    let closed = group_closed_fixture(pool, &original, variant).await;
    let mut admin = direct(pool).await;
    let mut install = begin_protocol(&mut admin, &closed.target).await;
    let outcome = AssertUnwindSafe(async {
        group_install_replay(install.as_mut(), &closed, variant, &oracle).await;
        navigation_snapshot(install.as_mut(), false, variant).await
    })
    .catch_unwind()
    .await;
    let baseline = match outcome {
        Ok(baseline) => {
            install.commit().await.unwrap();
            baseline
        }
        Err(panic) => {
            install.rollback().await.unwrap();
            admin.close().await.unwrap();
            serving_restored(pool, variant, &closed).await;
            std::panic::resume_unwind(panic)
        }
    };
    admin.close().await.unwrap();
    navigation_restored(pool, &baseline, false, variant).await;
    baseline
}

async fn navigation_finalize(connection: &mut PgConnection) -> Duration {
    bounds(connection).await;
    let start = Instant::now();
    sqlx::raw_sql(NAVIGATION_FINALIZER)
        .execute(connection)
        .await
        .unwrap();
    let elapsed = start.elapsed();
    assert!(
        elapsed <= Duration::from_secs(60),
        "navigation correction exceeded reviewed fixture bound"
    );
    elapsed
}

// Deliberately does not reset settings: entry-refusal tests must retain the
// injected setting rather than inadvertently erase it through bounds().
async fn navigation_refused(connection: &mut PgConnection, message: &str) {
    let error = sqlx::raw_sql(NAVIGATION_FINALIZER)
        .execute(connection)
        .await
        .expect_err("generated navigation finalizer must refuse this concrete fault");
    let database = error
        .as_database_error()
        .expect("actual PostgreSQL refusal required");
    assert_eq!(database.code().as_deref(), Some("P0001"));
    assert_eq!(database.message(), message);
}

async fn navigation_transition_history(pool: PgPool, variant: usize) {
    let sources = navigation_finalizer_pins();
    let oracle = group_oracle();
    let correction = navigation_correction();
    let original = navigation_original_fixture(&pool, variant).await;
    let mut admin = direct(&pool).await;
    let mut tx = begin_protocol(&mut admin, &original.target).await;
    let outcome = AssertUnwindSafe(async {
        let exclusive_before: i64 = sqlx::query_scalar("SELECT count(*) FROM pg_catalog.pg_locks l JOIN pg_catalog.pg_class c ON c.oid=l.relation JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace WHERE l.pid=pg_backend_pid() AND l.locktype='relation' AND l.mode='AccessExclusiveLock' AND l.granted AND n.nspname='public'")
            .fetch_one(tx.as_mut()).await.unwrap();
        assert_eq!(exclusive_before, 0, "fresh correction entry must acquire its own83 locks");
        let install_elapsed = navigation_finalize(tx.as_mut()).await;
        let corrected = navigation_snapshot(tx.as_mut(), true, variant).await;
        assert_ne!(corrected.tuple, original.tuple, "actual correction must replace the navigation tuple");
        navigation_catalog_delta(&original.catalog.1, &corrected.catalog.1, &correction);
        assert!(corrected.rows == original.rows, "correction changed durable rows");
        assert!(corrected.ledger == original.ledger);
        let locks = group_locks(tx.as_mut(), &oracle).await;
        let replay_elapsed = navigation_finalize(tx.as_mut()).await;
        let replayed = navigation_snapshot(tx.as_mut(), true, variant).await;
        assert!(replayed.capture == corrected.capture);
        assert!(replayed.catalog == corrected.catalog);
        assert!(replayed.rows == corrected.rows);
        assert!(replayed.ledger == corrected.ledger);
        assert_eq!(replayed.tuple, corrected.tuple, "same-transaction replay performed redundant DDL");
        assert_eq!(group_locks(tx.as_mut(), &oracle).await, locks);
        // Positive control proves the tuple oracle catches a no-byte-change DDL
        // fault. Raw correction is solely this rolled-back oracle corruption,
        // never an installation or browser fixture path.
        sqlx::raw_sql("SAVEPOINT replay_ddl_oracle_fault").execute(tx.as_mut()).await.unwrap();
        navigation_apply(tx.as_mut(), &correction).await;
        assert!(catalog(tx.as_mut()).await == corrected.catalog);
        assert_ne!(navigation_tuple(tx.as_mut()).await, corrected.tuple, "replay DDL oracle missed injected replacement");
        sqlx::raw_sql("ROLLBACK TO SAVEPOINT replay_ddl_oracle_fault; RELEASE SAVEPOINT replay_ddl_oracle_fault")
            .execute(tx.as_mut()).await.unwrap();
        assert_eq!(navigation_tuple(tx.as_mut()).await, corrected.tuple);
        assert!(group_capture(tx.as_mut(), true).await == corrected.capture);
        assert!(rows(tx.as_mut()).await == corrected.rows);
        assert!(applied_ledger(tx.as_mut()).await == corrected.ledger);
        assert_eq!(group_locks(tx.as_mut(), &oracle).await, locks);
        (corrected, json!({"install_ms":install_elapsed.as_millis(),"same_transaction_replay_ms":replay_elapsed.as_millis(),"locks":locks,"replay_ddl_oracle_positive_control":true}))
    }).catch_unwind().await;
    let (corrected, mut packet) = match outcome {
        Ok(value) => {
            tx.commit().await.unwrap();
            value
        }
        Err(panic) => {
            tx.rollback().await.unwrap();
            admin.close().await.unwrap();
            navigation_restored(&pool, &original, false, variant).await;
            std::panic::resume_unwind(panic)
        }
    };
    admin.close().await.unwrap();
    navigation_restored(&pool, &corrected, true, variant).await;
    let mut replay = direct(&pool).await;
    let mut fresh_tx = begin_protocol(&mut replay, &corrected.target).await;
    let replay_outcome = AssertUnwindSafe(async {
        navigation_finalize(fresh_tx.as_mut()).await;
        let after = navigation_snapshot(fresh_tx.as_mut(), true, variant).await;
        assert!(after.capture == corrected.capture);
        assert!(after.catalog == corrected.catalog);
        assert!(after.rows == corrected.rows);
        assert!(after.ledger == corrected.ledger);
        assert_eq!(
            after.tuple, corrected.tuple,
            "new-connection replay performed redundant DDL"
        );
        group_locks(fresh_tx.as_mut(), &oracle).await;
    })
    .catch_unwind()
    .await;
    if replay_outcome.is_ok() {
        fresh_tx.commit().await.unwrap();
    } else {
        fresh_tx.rollback().await.unwrap();
    }
    replay.close().await.unwrap();
    navigation_restored(&pool, &corrected, true, variant).await;
    if let Err(panic) = replay_outcome {
        std::panic::resume_unwind(panic);
    }
    packet["sources"] = sources;
    packet["variant"] = json!(if variant == 0 { "plain" } else { "observer" });
    packet["before83"] = json!(original.capture.sha256);
    packet["corrected83"] = json!(corrected.capture.sha256);
    packet["fresh_postcommit_replay_and_readback"] = json!(true);
    packet["only_eleven_navigation_local_sites_changed"] = json!(true);
    packet["business_ui_evidence"] = json!(false);
    packet["production_qualified"] = json!(false);
    writeln!(
        &mut std::io::stderr().lock(),
        "GROUP_NAVIGATION_FINALIZER_HISTORY {packet}"
    )
    .unwrap();
}

#[sqlx::test(migrations = false)]
async fn navigation_generated_plain_install_same_and_new_connection_replay(pool: PgPool) {
    navigation_transition_history(pool, 0).await;
}

#[sqlx::test(migrations = false)]
async fn navigation_generated_observer_install_same_and_new_connection_replay(pool: PgPool) {
    navigation_transition_history(pool, 1).await;
}

#[sqlx::test(migrations = false)]
async fn navigation_generated_precommit_error_restores_original_catalog_rows_ledger(pool: PgPool) {
    let original = navigation_original_fixture(&pool, 0).await;
    let oracle = group_oracle();
    let correction = navigation_correction();
    let mut admin = direct(&pool).await;
    let mut tx = begin_protocol(&mut admin, &original.target).await;
    let outcome = AssertUnwindSafe(async {
        navigation_finalize(tx.as_mut()).await;
        let corrected = navigation_snapshot(tx.as_mut(), true, 0).await;
        navigation_catalog_delta(&original.catalog.1, &corrected.catalog.1, &correction);
        assert!(corrected.rows == original.rows);
        assert!(corrected.ledger == original.ledger);
        group_locks(tx.as_mut(), &oracle).await;
        let error = sqlx::query("SELECT 1/0")
            .execute(tx.as_mut())
            .await
            .expect_err("actual precommit fault must execute");
        assert_eq!(
            error.as_database_error().unwrap().code().as_deref(),
            Some("22012")
        );
    })
    .catch_unwind()
    .await;
    tx.rollback().await.unwrap();
    admin.close().await.unwrap();
    navigation_restored(&pool, &original, false, 0).await;
    if let Err(panic) = outcome {
        std::panic::resume_unwind(panic);
    }
}

#[sqlx::test(migrations = false)]
async fn navigation_generated_actual_prefix_wait_cancellation_restores_original(pool: PgPool) {
    let original = navigation_original_fixture(&pool, 0).await;
    // All fallible connection creation precedes the blocker and spawned task.
    let mut observer = direct(&pool).await;
    let mut blocker = direct(&pool).await;
    let mut installer = direct(&pool).await;
    let blocker_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut blocker)
        .await
        .unwrap();
    let frozen = target(&mut installer).await;
    let installer_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut installer)
        .await
        .unwrap();
    let mut blocking = sqlx::Connection::begin(&mut blocker).await.unwrap();
    bounds(blocking.as_mut()).await;
    sqlx::raw_sql("LOCK TABLE ONLY public.account_security IN ACCESS SHARE MODE")
        .execute(blocking.as_mut())
        .await
        .unwrap();
    let mut task = tokio::spawn(async move {
        let mut tx = begin_protocol(&mut installer, &frozen).await;
        let result = sqlx::raw_sql(NAVIGATION_FINALIZER)
            .execute(tx.as_mut())
            .await;
        tx.rollback().await.unwrap();
        installer.close().await.unwrap();
        result
    });
    let cancellation_deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    let outcome = tokio::time::timeout_at(cancellation_deadline, AssertUnwindSafe(async {
        let wait = wait_for_blocker(&mut observer, installer_pid, blocker_pid).await;
        let locks: Vec<(String, String, bool)> = sqlx::query_as("SELECT c.relname::text,l.mode::text,l.granted FROM pg_catalog.pg_locks l JOIN pg_catalog.pg_class c ON c.oid=l.relation JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace WHERE l.pid=$1 AND n.nspname='public' AND l.mode='AccessExclusiveLock' ORDER BY c.relname COLLATE \"C\"")
            .bind(installer_pid).fetch_all(&mut observer).await.unwrap();
        assert_eq!(locks, vec![("account_context_candidates".into(), "AccessExclusiveLock".into(), true), ("account_security".into(), "AccessExclusiveLock".into(), false)]);
        assert!(wait < Duration::from_millis(750));
        let cancelled: bool = sqlx::query_scalar("SELECT pg_catalog.pg_cancel_backend($1)")
            .bind(installer_pid).fetch_one(&mut observer).await.unwrap();
        assert!(cancelled);
    }).catch_unwind()).await;
    // Defer every failure until the blocker, task and observer are cleaned up.
    let mut timed_out = false;
    let joined = tokio::time::timeout_at(cancellation_deadline, &mut task).await;
    // Forced cancellation and the entire census share the existing 5s cleanup
    // envelope; a stalled observer query cannot delay owned-task termination.
    let cleanup_deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    let mut forced_cancel: Option<Result<Result<bool, sqlx::Error>, tokio::time::error::Elapsed>> =
        None;
    let result = match joined {
        Ok(joined) => joined,
        Err(_) => {
            timed_out = true;
            forced_cancel = Some(
                tokio::time::timeout_at(
                    cleanup_deadline,
                    sqlx::query_scalar("SELECT pg_catalog.pg_cancel_backend($1)")
                        .bind(installer_pid)
                        .fetch_one(&mut observer),
                )
                .await,
            );
            task.abort();
            task.await
        }
    };
    let blocker_rollback = tokio::time::timeout_at(cleanup_deadline, blocking.rollback()).await;
    let blocker_close = tokio::time::timeout_at(cleanup_deadline, blocker.close()).await;
    let cleanup = tokio::time::timeout_at(
        cleanup_deadline,
        AssertUnwindSafe(async {
            loop {
                let (backends, locks): (i64, i64) = sqlx::query_as(
                "SELECT (SELECT count(*) FROM pg_catalog.pg_stat_activity WHERE pid IN ($1,$2)), \
                 (SELECT count(*) FROM pg_catalog.pg_locks WHERE pid IN ($1,$2))")
                .bind(installer_pid).bind(blocker_pid).fetch_one(&mut observer).await.unwrap();
                if backends == 0 && locks == 0 {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .catch_unwind(),
    )
    .await;
    // Bound graceful shutdown too: its owning future drops the observer socket
    // if a buffered Terminate/TLS shutdown cannot complete by this deadline.
    let observer_close = tokio::time::timeout_at(cleanup_deadline, observer.close()).await;
    navigation_restored(&pool, &original, false, 0).await;
    match cleanup {
        Ok(Ok(())) => {}
        Ok(Err(panic)) => std::panic::resume_unwind(panic),
        Err(_) => panic!(
            "owned cancellation backend/lock cleanup exceeded its existing 5s budget after task termination, observer close and restoration readback"
        ),
    }
    blocker_rollback
        .expect("owned blocker rollback exceeded the cleanup deadline")
        .expect("owned blocker rollback failed after cancellation cleanup");
    blocker_close
        .expect("owned blocker close exceeded the cleanup deadline")
        .expect("owned blocker close failed after cancellation cleanup");
    observer_close
        .expect("owned observer close exceeded the cleanup deadline")
        .expect("owned observer close failed after cancellation cleanup");
    if let Some(cancel) = forced_cancel {
        cancel
            .expect("forced cancellation transport exceeded the cleanup deadline")
            .expect("forced cancellation transport failed after cleanup");
    }
    assert!(
        !timed_out,
        "bounded cancellation did not finish; task was aborted and joined, owned backends and locks absent, original state restored"
    );
    match outcome {
        Ok(Ok(())) => {}
        Ok(Err(panic)) => std::panic::resume_unwind(panic),
        Err(_) => panic!(
            "owned cancellation observation exceeded its5s deadline; task terminated and cleanup/readback completed"
        ),
    }
    assert_eq!(
        result
            .expect("owned installer task failed after cleanup")
            .unwrap_err()
            .as_database_error()
            .unwrap()
            .code()
            .as_deref(),
        Some("57014")
    );
}

#[sqlx::test(migrations = false)]
async fn navigation_generated_refuses_faults_without_repair_on_install_and_replay(pool: PgPool) {
    let original = navigation_original_fixture(&pool, 0).await;
    for replay in [false, true] {
        let mut admin = direct(&pool).await;
        let mut tx = begin_protocol(&mut admin, &original.target).await;
        let outcome = AssertUnwindSafe(async {
            if replay { navigation_finalize(tx.as_mut()).await; }
            let healthy = navigation_snapshot(tx.as_mut(), replay, 0).await;
            for fault in ["ledger_checksum", "ledger_success", "ledger_missing", "ledger_owner", "navigation_source", "navigation_owner", "public_execute", "reserved_schema"] {
                sqlx::raw_sql("SAVEPOINT navigation_injected_fault").execute(tx.as_mut()).await.unwrap();
                let message = match fault {
                    "ledger_checksum" | "ledger_success" | "ledger_missing" => {
                        let statement = match fault {
                            "ledger_checksum" => "UPDATE public._sqlx_migrations SET checksum=decode(repeat('00',48),'hex') WHERE version=231",
                            "ledger_success" => "UPDATE public._sqlx_migrations SET success=false WHERE version=231",
                            _ => "DELETE FROM public._sqlx_migrations WHERE version=231",
                        };
                        assert_eq!(sqlx::query(statement).execute(tx.as_mut()).await.unwrap().rows_affected(), 1);
                        "native_group_process_navigation.migration_ledger_mismatch"
                    }
                    "ledger_owner" => {
                        sqlx::raw_sql("ALTER TABLE public._sqlx_migrations OWNER TO console_account_owner").execute(tx.as_mut()).await.unwrap();
                        let owner: String = sqlx::query_scalar("SELECT pg_get_userbyid(relowner) FROM pg_catalog.pg_class WHERE oid='public._sqlx_migrations'::regclass").fetch_one(tx.as_mut()).await.unwrap();
                        assert_eq!(owner, "console_account_owner");
                        "native_group_process_navigation.migration_ledger_lock_missing"
                    }
                    "navigation_source" => {
                        let definition: String = sqlx::query_scalar("SELECT pg_catalog.pg_get_functiondef($1::regprocedure)").bind(NAVIGATION_IDENTITY).fetch_one(tx.as_mut()).await.unwrap();
                        assert_eq!(definition.matches("$function$").count(), 2);
                        let (body, suffix) = definition.rsplit_once("$function$").unwrap();
                        let changed = format!("{body}\n-- explicit navigation source corruption\n$function${suffix}");
                        sqlx::raw_sql(sqlx::AssertSqlSafe(changed)).execute(tx.as_mut()).await.unwrap();
                        "native_group_process_navigation.profile_mismatch"
                    }
                    "navigation_owner" => {
                        sqlx::raw_sql("ALTER FUNCTION public.identity_native_group_process_navigation_candidates_v1(uuid,uuid,bytea) OWNER TO console_app").execute(tx.as_mut()).await.unwrap();
                        let owner: String = sqlx::query_scalar("SELECT pg_get_userbyid(proowner) FROM pg_catalog.pg_proc WHERE oid=$1::regprocedure").bind(NAVIGATION_IDENTITY).fetch_one(tx.as_mut()).await.unwrap();
                        assert_eq!(owner, "console_app");
                        "native_group_process_navigation.profile_mismatch"
                    }
                    "public_execute" => {
                        sqlx::raw_sql("GRANT EXECUTE ON FUNCTION public.identity_native_group_process_navigation_candidates_v1(uuid,uuid,bytea) TO PUBLIC").execute(tx.as_mut()).await.unwrap();
                        let granted: bool = sqlx::query_scalar("SELECT has_function_privilege('console_auth_startup',$1::regprocedure,'EXECUTE')").bind(NAVIGATION_IDENTITY).fetch_one(tx.as_mut()).await.unwrap();
                        assert!(granted, "ACL positive control must actually disclose the forbidden executor");
                        "native_group_process_navigation.profile_mismatch"
                    }
                    "reserved_schema" => {
                        sqlx::raw_sql("CREATE SCHEMA identity_native_group_process_unreviewed").execute(tx.as_mut()).await.unwrap();
                        let present: bool = sqlx::query_scalar("SELECT to_regnamespace('identity_native_group_process_unreviewed') IS NOT NULL").fetch_one(tx.as_mut()).await.unwrap();
                        assert!(present);
                        "native_group_process_navigation.profile_mismatch"
                    }
                    _ => unreachable!(),
                };
                let faulty_catalog = catalog(tx.as_mut()).await;
                let faulty_rows = rows(tx.as_mut()).await;
                let faulty_tuple = navigation_tuple(tx.as_mut()).await;
                assert!(faulty_catalog != healthy.catalog || faulty_rows != healthy.rows, "injected {fault} did not change the actual database");
                let faulty_phase = navigation_classification(tx.as_mut()).await;
                if message == "native_group_process_navigation.profile_mismatch" { assert_eq!(faulty_phase, ("native_group_process_navigation.profile_mismatch".into(), None)); }
                bounds(tx.as_mut()).await;
                sqlx::raw_sql("SAVEPOINT navigation_finalizer_attempt").execute(tx.as_mut()).await.unwrap();
                navigation_refused(tx.as_mut(), message).await;
                // Recover only the failed invocation, preserving the injected
                // fault for exact no-repair/no-effect readback.
                sqlx::raw_sql("ROLLBACK TO SAVEPOINT navigation_finalizer_attempt; RELEASE SAVEPOINT navigation_finalizer_attempt").execute(tx.as_mut()).await.unwrap();
                assert!(catalog(tx.as_mut()).await == faulty_catalog, "refusal repaired or changed {fault} metadata");
                assert!(rows(tx.as_mut()).await == faulty_rows, "refusal repaired or changed {fault} rows");
                assert_eq!(navigation_tuple(tx.as_mut()).await, faulty_tuple);
                assert_eq!(navigation_classification(tx.as_mut()).await, faulty_phase);
                sqlx::raw_sql("ROLLBACK TO SAVEPOINT navigation_injected_fault; RELEASE SAVEPOINT navigation_injected_fault").execute(tx.as_mut()).await.unwrap();
                assert!(catalog(tx.as_mut()).await == healthy.catalog);
                assert!(rows(tx.as_mut()).await == healthy.rows);
                assert!(applied_ledger(tx.as_mut()).await == healthy.ledger);
                assert!(group_capture(tx.as_mut(), true).await == healthy.capture);
                assert_eq!(navigation_tuple(tx.as_mut()).await, healthy.tuple);
                assert_eq!(navigation_classification(tx.as_mut()).await, navigation_phase(replay, 0));
            }
            // Both install and replay positives follow the complete refusal
            // census. A permanently failing finalizer cannot pass negatives.
            navigation_finalize(tx.as_mut()).await;
            let corrected = navigation_snapshot(tx.as_mut(), true, 0).await;
            navigation_finalize(tx.as_mut()).await;
            assert_eq!(navigation_tuple(tx.as_mut()).await, corrected.tuple);
            assert!(rows(tx.as_mut()).await == original.rows);
            assert!(applied_ledger(tx.as_mut()).await == original.ledger);
        }).catch_unwind().await;
        tx.rollback().await.unwrap();
        admin.close().await.unwrap();
        navigation_restored(&pool, &original, false, 0).await;
        if let Err(panic) = outcome {
            std::panic::resume_unwind(panic);
        }
    }
}

#[sqlx::test(migrations = false)]
async fn navigation_generated_entry_target_operator_settings_and_ledger_lock_refuse(pool: PgPool) {
    let original = navigation_original_fixture(&pool, 0).await;
    for field in ["database", "database_oid", "system_identifier"] {
        let mut wrong = original.target.clone();
        match field {
            "database" => wrong.database.push_str("_wrong"),
            "database_oid" => wrong.database_oid += 1,
            "system_identifier" => wrong.system_identifier.push('0'),
            _ => unreachable!(),
        }
        let mut descriptor = direct(&pool).await;
        let refusal = AssertUnwindSafe(async {
            // Invoke the actual caller entry. A predicate-only check would pass
            // even if begin_protocol stopped enforcing its expected target.
            let tx = begin_protocol(&mut descriptor, &wrong).await;
            tx.rollback().await.unwrap();
        })
        .catch_unwind()
        .await;
        let actual = match refusal {
            Ok(()) => None,
            Err(panic) => panic.downcast_ref::<String>().cloned().or_else(|| {
                panic
                    .downcast_ref::<&str>()
                    .map(|message| (*message).to_owned())
            }),
        };
        let cleanup = AssertUnwindSafe(async {
            no_prior_user_locks(&mut descriptor).await;
        })
        .catch_unwind()
        .await;
        let closed = descriptor.close().await;
        navigation_restored(&pool, &original, false, 0).await;
        if let Err(panic) = cleanup {
            std::panic::resume_unwind(panic);
        }
        closed.expect("wrong-target caller connection must close before reporting refusal");
        assert_eq!(
            actual.as_deref(),
            Some("caller target mismatch before BEGIN"),
            "actual caller entry must refuse wrong {field} at its pre-BEGIN guard"
        );
    }
    for (fault, message) in [
        (
            "SET LOCAL lock_timeout='0'",
            "native_group_process_navigation.entry_bounds_mismatch",
        ),
        (
            "SET LOCAL lock_timeout='1001ms'",
            "native_group_process_navigation.entry_bounds_mismatch",
        ),
        (
            "SET LOCAL statement_timeout='0'",
            "native_group_process_navigation.entry_bounds_mismatch",
        ),
        (
            "SET LOCAL statement_timeout='60001ms'",
            "native_group_process_navigation.entry_bounds_mismatch",
        ),
        (
            "SET LOCAL idle_in_transaction_session_timeout='0'",
            "native_group_process_navigation.entry_bounds_mismatch",
        ),
        (
            "SET LOCAL idle_in_transaction_session_timeout='30001ms'",
            "native_group_process_navigation.entry_bounds_mismatch",
        ),
        (
            "SET LOCAL transaction_timeout='0'",
            "native_group_process_navigation.entry_bounds_mismatch",
        ),
        (
            "SET LOCAL transaction_timeout='120001ms'",
            "native_group_process_navigation.entry_bounds_mismatch",
        ),
        (
            "SET LOCAL search_path=public,pg_catalog",
            "native_group_process_navigation.entry_settings_mismatch",
        ),
        (
            "SET LOCAL jit=on",
            "native_group_process_navigation.entry_settings_mismatch",
        ),
        (
            "SET LOCAL console.sqlx_test_bootstrap='wrong'",
            "native_group_process_navigation.operator_identity_mismatch",
        ),
        (
            "SET LOCAL ROLE console_account_owner",
            "native_group_process_navigation.operator_identity_mismatch",
        ),
    ] {
        let mut admin = direct(&pool).await;
        let mut tx = begin_protocol(&mut admin, &original.target).await;
        let outcome = AssertUnwindSafe(async {
            sqlx::raw_sql(fault).execute(tx.as_mut()).await.unwrap();
            navigation_refused(tx.as_mut(), message).await;
        })
        .catch_unwind()
        .await;
        tx.rollback().await.unwrap();
        admin.close().await.unwrap();
        navigation_restored(&pool, &original, false, 0).await;
        if let Err(panic) = outcome {
            std::panic::resume_unwind(panic);
        }
    }
    for isolation in ["REPEATABLE READ", "SERIALIZABLE"] {
        let mut admin = direct(&pool).await;
        assert_eq!(target(&mut admin).await, original.target);
        no_prior_user_locks(&mut admin).await;
        let mut tx = sqlx::Connection::begin(&mut admin).await.unwrap();
        let outcome = AssertUnwindSafe(async {
            sqlx::raw_sql(sqlx::AssertSqlSafe(format!("SET TRANSACTION ISOLATION LEVEL {isolation}"))).execute(tx.as_mut()).await.unwrap();
            sqlx::raw_sql(ENTRY_BOUNDS).execute(tx.as_mut()).await.unwrap();
            sqlx::raw_sql("LOCK TABLE ONLY public._sqlx_migrations IN SHARE MODE").execute(tx.as_mut()).await.unwrap();
            applied_ledger(tx.as_mut()).await;
            let role_oids: Vec<String> = sqlx::query_scalar("SELECT oid::text FROM pg_catalog.pg_authid WHERE rolname='console_account_owner' FOR UPDATE").fetch_all(tx.as_mut()).await.unwrap();
            assert_eq!(role_oids.len(), 1);
            assert!(role_oids[0].parse::<u32>().unwrap() > 0);
            navigation_refused(tx.as_mut(), "native_group_process_navigation.entry_settings_mismatch").await;
        }).catch_unwind().await;
        tx.rollback().await.unwrap();
        admin.close().await.unwrap();
        navigation_restored(&pool, &original, false, 0).await;
        if let Err(panic) = outcome {
            std::panic::resume_unwind(panic);
        }
    }
    let mut admin = direct(&pool).await;
    assert_eq!(target(&mut admin).await, original.target);
    no_prior_user_locks(&mut admin).await;
    let mut tx = sqlx::Connection::begin(&mut admin).await.unwrap();
    let outcome = AssertUnwindSafe(async {
        bounds(tx.as_mut()).await;
        applied_ledger(tx.as_mut()).await;
        let role_oids: Vec<String> = sqlx::query_scalar("SELECT oid::text FROM pg_catalog.pg_authid WHERE rolname='console_account_owner' FOR UPDATE").fetch_all(tx.as_mut()).await.unwrap();
        assert_eq!(role_oids.len(), 1);
        assert!(role_oids[0].parse::<u32>().unwrap() > 0);
        let ledger_share: i64 = sqlx::query_scalar("SELECT count(*) FROM pg_catalog.pg_locks WHERE pid=pg_backend_pid() AND locktype='relation' AND relation='public._sqlx_migrations'::regclass AND mode='ShareLock' AND granted").fetch_one(tx.as_mut()).await.unwrap();
        assert_eq!(ledger_share, 0, "missing-ShareLock control must actually omit that lease");
        navigation_refused(tx.as_mut(), "native_group_process_navigation.migration_ledger_lock_missing").await;
    }).catch_unwind().await;
    tx.rollback().await.unwrap();
    admin.close().await.unwrap();
    navigation_restored(&pool, &original, false, 0).await;
    if let Err(panic) = outcome {
        std::panic::resume_unwind(panic);
    }
    // A final positive install followed by rollback validates the same artifact
    // after all entry controls and keeps the original fixture unchanged.
    let mut positive = direct(&pool).await;
    let mut tx = begin_protocol(&mut positive, &original.target).await;
    let outcome = AssertUnwindSafe(async {
        navigation_finalize(tx.as_mut()).await;
        navigation_snapshot(tx.as_mut(), true, 0).await;
    })
    .catch_unwind()
    .await;
    tx.rollback().await.unwrap();
    positive.close().await.unwrap();
    navigation_restored(&pool, &original, false, 0).await;
    if let Err(panic) = outcome {
        std::panic::resume_unwind(panic);
    }
}

// External test-only experiment: corrected metadata fixtures contain historical
// business rows. Scalar timings/plans retain the exact frozen SELECT projection;
// parity alone exposes the existing full83 CTE. No serving claim is made.
async fn navigation_materialized_classifier_readonly_experiment(pool: PgPool, variant: usize) {
    let sources = navigation_finalizer_pins();
    let marker = "), snapshots AS (\n";
    let hinted_marker = "), snapshots AS MATERIALIZED (\n";
    assert_eq!(NAVIGATION_APP_STATE.matches(marker).count(), 1);
    let hinted = NAVIGATION_APP_STATE.replacen(marker, hinted_marker, 1);
    assert_eq!(hinted.matches(hinted_marker).count(), 1);
    assert_eq!(
        hinted.replacen(hinted_marker, marker, 1),
        NAVIGATION_APP_STATE
    );
    assert_eq!(
        digest(&hinted),
        "b7b46b958ed86fe032dda3202152995d0b8873beaa748d5e1f078b11396c498e"
    );

    // Owned task boundary avoids nesting the large frozen fixture in this leaf.
    // Join immediately; neither fixture work nor its panic becomes detached.
    let fixture_pool = pool.clone();
    let mut fixture = tokio::task::JoinSet::new();
    fixture.spawn(async move { navigation_transition_history(fixture_pool, variant).await });
    match fixture
        .join_next()
        .await
        .expect("owned navigation fixture task missing")
    {
        Ok(()) => {}
        Err(error) if error.is_panic() => std::panic::resume_unwind(error.into_panic()),
        Err(_) => panic!("owned navigation fixture task unexpectedly cancelled"),
    }
    assert!(fixture.is_empty());

    let mut admin = direct(&pool).await;
    let baseline_outcome = AssertUnwindSafe(async {
        sqlx::raw_sql("BEGIN ISOLATION LEVEL REPEATABLE READ, READ ONLY")
            .execute(&mut admin)
            .await
            .unwrap();
        navigation_snapshot(&mut admin, true, variant).await
    })
    .catch_unwind()
    .await;
    let baseline_rollback = sqlx::raw_sql("ROLLBACK").execute(&mut admin).await;
    let baseline_close = admin.close().await;
    assert!(
        baseline_rollback.is_ok(),
        "experiment baseline rollback failed"
    );
    assert!(
        baseline_close.is_ok(),
        "experiment baseline connection close failed"
    );
    let baseline = match baseline_outcome {
        Ok(baseline) => baseline,
        Err(panic) => std::panic::resume_unwind(panic),
    };
    navigation_restored(&pool, &baseline, true, variant).await;

    let runtime_url = console_platform_test_support::login_test_database_url(
        &pool,
        console_platform_test_support::TestDatabaseLogin::Business,
    );
    let mut runtime = PgConnection::connect(&runtime_url)
        .await
        .unwrap_or_else(|_| panic!("classifier experiment Business LOGIN connection failed"));
    drop(runtime_url);
    let expected_phase = navigation_phase(true, variant);
    let mut identity = Value::Null;
    let mut parity = Vec::<Value>::new();
    let mut samples = Vec::<Value>::new();
    let mut plans = Vec::<Value>::new();
    let comparison = AssertUnwindSafe(async {
        sqlx::raw_sql("BEGIN ISOLATION LEVEL REPEATABLE READ, READ ONLY")
            .execute(&mut runtime).await
            .unwrap_or_else(|_| panic!("classifier experiment read-only transaction failed"));
        sqlx::raw_sql(CLASSIFIER_SESSION).execute(&mut runtime).await
            .unwrap_or_else(|_| panic!("classifier experiment fixed session failed"));
        identity = sqlx::query_scalar(
            "SELECT jsonb_build_object(\
                'database',current_database(),'database_oid',(SELECT oid::bigint FROM pg_catalog.pg_database WHERE datname=current_database()),\
                'backend_pid',pg_backend_pid(),'session_user',session_user,'current_user',current_user,\
                'role',current_setting('role'),'server_version',current_setting('server_version'),\
                'server_version_num',current_setting('server_version_num'),'version',version(),\
                'transaction_isolation',current_setting('transaction_isolation'),\
                'transaction_read_only',current_setting('transaction_read_only'),\
                'statement_timeout',current_setting('statement_timeout'),'jit',current_setting('jit'),\
                'search_path',current_setting('search_path'),'row_security',current_setting('row_security'),\
                'session_role_superuser',(SELECT rolsuper FROM pg_catalog.pg_roles WHERE rolname=session_user),\
                'session_role_bypassrls',(SELECT rolbypassrls FROM pg_catalog.pg_roles WHERE rolname=session_user))"
        ).fetch_one(&mut runtime).await
            .unwrap_or_else(|_| panic!("classifier experiment safe login metadata failed"));
        assert_eq!(identity["database"], json!(baseline.target.database));
        assert_eq!(identity["database_oid"], json!(baseline.target.database_oid));
        assert_eq!(identity["session_user"], "console_rt");
        assert_eq!(identity["current_user"], "console_rt");
        assert_eq!(identity["role"], "none");
        assert_eq!(identity["session_role_superuser"], false);
        assert_eq!(identity["transaction_isolation"], "repeatable read");
        assert_eq!(identity["transaction_read_only"], "on");
        assert_eq!(identity["statement_timeout"], "3s");
        assert_eq!(identity["jit"], "off");
        assert_eq!(identity["search_path"], "pg_catalog, pg_temp");

        // Do not serialize a second snapshot or time this changed projection.
        // Each restricted-login snapshot must equal the owner-known exact bytes.
        for (label, source) in [("original", NAVIGATION_APP_STATE), ("materialized", hinted.as_str())] {
            let body = source.strip_suffix(" AS state;\n")
                .expect("pinned navigation classifier terminator");
            let projected = format!("{body} AS state,(SELECT variant FROM matching_phase) AS variant,(SELECT snapshot::text FROM full83) AS snapshot_text,(SELECT snapshot_sha256 FROM full83) AS snapshot_sha256");
            let (state, matched_variant, text, sha256): (String, Option<String>, String, String) =
                sqlx::query_as(sqlx::AssertSqlSafe(projected)).fetch_one(&mut runtime).await
                    .unwrap_or_else(|_| panic!("classifier experiment full83 parity projection failed"));
            assert_eq!((state.clone(), matched_variant.clone()), expected_phase);
            assert!(text == baseline.capture.text, "restricted-login exact full83 snapshot changed");
            assert_eq!(digest(&text), sha256, "raw PostgreSQL UTF-8 snapshot digest differs");
            assert_eq!(sha256, baseline.capture.sha256);
            assert_eq!(sha256, NAVIGATION_CORRECTED83[variant]);
            parity.push(json!({"query":label,"state":state,"variant":matched_variant,
                "full83_sha256":sha256,"exact_owner_snapshot_bytes_equal":true}));
        }

        // Four paired rounds balance first/second order without changing either
        // scalar statement. Statement timeout remains the frozen three seconds.
        for round in 0..4 {
            let order = if round % 2 == 0 {
                [("original", NAVIGATION_APP_STATE), ("materialized", hinted.as_str())]
            } else {
                [("materialized", hinted.as_str()), ("original", NAVIGATION_APP_STATE)]
            };
            for (position, (label, source)) in order.into_iter().enumerate() {
                let started = Instant::now();
                let result: Result<String, sqlx::Error> = sqlx::query_scalar(sqlx::AssertSqlSafe(source))
                    .fetch_one(&mut runtime).await;
                samples.push(json!({"round":round + 1,"position":position + 1,"query":label,
                    "status":if result.is_ok() {"ok"} else {"query_error"},
                    "state":result.as_ref().ok(),
                    "elapsed_micros":u64::try_from(started.elapsed().as_micros()).unwrap()}));
                let state = result.unwrap_or_else(|_| panic!("classifier experiment exact scalar sample failed"));
                assert_eq!(state, expected_phase.0);
            }
        }
        let plan_order = if variant == 0 {
            [("original", NAVIGATION_APP_STATE), ("materialized", hinted.as_str())]
        } else {
            [("materialized", hinted.as_str()), ("original", NAVIGATION_APP_STATE)]
        };
        for (position, (label, source)) in plan_order.into_iter().enumerate() {
            let plan: Value = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
                "EXPLAIN (ANALYZE, FORMAT JSON) {source}"
            ))).fetch_one(&mut runtime).await
                .unwrap_or_else(|_| panic!("classifier experiment exact scalar plan failed"));
            plans.push(json!({"position":position + 1,"query":label,"plan":plan}));
        }
    }).catch_unwind().await;

    // Always await both cleanup operations and exact fresh effect readback before
    // asserting cleanup or rethrowing a semantic failure. Never export rows/text.
    let runtime_rollback = sqlx::raw_sql("ROLLBACK").execute(&mut runtime).await;
    let runtime_close = runtime.close().await;
    let restored = AssertUnwindSafe(navigation_restored(&pool, &baseline, true, variant))
        .catch_unwind()
        .await;
    let packet = json!({"sources":sources,"variant":expected_phase.1,
        "original_source_sha256":digest(NAVIGATION_APP_STATE),"materialized_source_sha256":digest(&hinted),
        "known_corrected83":NAVIGATION_CORRECTED83[variant],"login_and_session":identity,
        "owner_target_system_identifier":baseline.target.system_identifier,
        "parity":parity,"scalar_samples":samples,"exact_scalar_plans":plans,
        "comparison_succeeded":comparison.is_ok(),"runtime_rollback_succeeded":runtime_rollback.is_ok(),
        "runtime_close_succeeded":runtime_close.is_ok(),"fresh_full_effect_readback_succeeded":restored.is_ok(),
        "historical_business_rows_present_in_fixture":true,"business_rows_exported":false,
        "performance_qualified":false,"serving_qualified":false,"production_qualified":false});
    eprintln!("GROUP_NAVIGATION_CLASSIFIER_READONLY_EXPERIMENT {packet}");
    assert!(
        runtime_rollback.is_ok(),
        "classifier experiment rollback failed"
    );
    assert!(
        runtime_close.is_ok(),
        "classifier experiment connection close failed"
    );
    if let Err(panic) = comparison {
        std::panic::resume_unwind(panic);
    }
    if let Err(panic) = restored {
        std::panic::resume_unwind(panic);
    }
}

#[sqlx::test(migrations = false)]
async fn navigation_plain_materialized_classifier_readonly_parity_and_alternating_samples(
    pool: PgPool,
) {
    navigation_materialized_classifier_readonly_experiment(pool, 0).await;
}

#[sqlx::test(migrations = false)]
async fn navigation_observer_materialized_classifier_readonly_parity_and_alternating_samples(
    pool: PgPool,
) {
    navigation_materialized_classifier_readonly_experiment(pool, 1).await;
}
