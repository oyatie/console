// Separate navigation-successor fixture. Frozen Group installer/browser fixture
// and diagnostics remain untouched; only the pinned generated finalizer writes.
pub(in super::super::super::super) async fn prepare_native_group_navigation_browser_database(
    pool: &PgPool,
) {
    prepare_native_group_browser_database(pool).await;
    let receipt = correct_native_group_browser_database(pool).await;
    assert_eq!(receipt["confirmed"], true);
}

pub(in super::super::super::super) async fn correct_native_group_browser_database(
    pool: &PgPool,
) -> Value {
    let sources = navigation_finalizer_pins();
    let oracle = group_oracle();
    let correction = navigation_correction();
    let artifacts = Artifacts::new();
    let key = SigningKey::random(&mut OsRng);
    let config = account_browser_config(pool, artifacts.root.clone(), &key);
    let mut states = Vec::<AppState>::new();
    let mut admin = direct(pool).await;
    let mut read = sqlx::Connection::begin(&mut admin).await.unwrap();
    let original = navigation_snapshot(read.as_mut(), false, 0).await;
    read.rollback().await.unwrap();
    let outcome = AssertUnwindSafe(async {
        for role in [console_app::AppRole::Api, console_app::AppRole::Worker] {
            let mut held_config = config.clone();
            held_config.role = role;
            let held = AppState::from_config(held_config)
                .await
                .expect("actual original Group navigation startup prerequisite");
            states.push(held);
            assert_eq!(ready_status(states.last().unwrap()).await, StatusCode::OK);
        }
        navigation_restored(pool, &original, false, 0).await;
        let mut install = begin_protocol(&mut admin, &original.target).await;
        let attempt = AssertUnwindSafe(async {
            let elapsed = navigation_finalize(install.as_mut()).await;
            let candidate = navigation_snapshot(install.as_mut(), true, 0).await;
            navigation_catalog_delta(&original.catalog.1, &candidate.catalog.1, &correction);
            assert!(
                candidate.rows == original.rows,
                "correction changed a UI-created or historical row"
            );
            assert!(
                candidate.ledger == original.ledger,
                "correction changed historical231 ledger"
            );
            let locks = group_locks(install.as_mut(), &oracle).await;
            navigation_finalize(install.as_mut()).await;
            let replay = navigation_snapshot(install.as_mut(), true, 0).await;
            assert!(
                replay.capture == candidate.capture
                    && replay.catalog == candidate.catalog
                    && replay.rows == candidate.rows
                    && replay.ledger == candidate.ledger
            );
            assert_eq!(
                replay.tuple, candidate.tuple,
                "same-transaction replay repeated DDL"
            );
            assert_eq!(group_locks(install.as_mut(), &oracle).await, locks);
            (candidate, elapsed)
        })
        .catch_unwind()
        .await;
        let (candidate, elapsed) = match attempt {
            Ok(value) => {
                install.commit().await.unwrap();
                value
            }
            Err(panic) => {
                install
                    .rollback()
                    .await
                    .expect("failed fixture correction rollback");
                navigation_restored(pool, &original, false, 0).await;
                std::panic::resume_unwind(panic)
            }
        };
        navigation_restored(pool, &candidate, true, 0).await;
        for held in &states {
            assert_eq!(
                ready_status(held).await,
                StatusCode::SERVICE_UNAVAILABLE,
                "original Group runtime must remain fenced after successor commit"
            );
            held.shutdown_realtime().await;
        }
        states.clear();
        let mut fresh_readiness_millis = Vec::new();
        for role in [console_app::AppRole::Api, console_app::AppRole::Worker] {
            let mut fresh_config = config.clone();
            fresh_config.role = role;
            let fresh = AppState::from_config(fresh_config)
                .await
                .expect("actual corrected Group startup prerequisite");
            states.push(fresh);
            let readiness_started = std::time::Instant::now();
            assert_eq!(ready_status(states.last().unwrap()).await, StatusCode::OK);
            fresh_readiness_millis.push(json!({"role":role.to_string(),
                "elapsed_millis":u64::try_from(readiness_started.elapsed().as_millis()).unwrap()}));
            states.last().unwrap().shutdown_realtime().await;
            states.pop();
        }
        navigation_restored(pool, &candidate, true, 0).await;
        let mut replay_connection = direct(pool).await;
        let mut replay = begin_protocol(&mut replay_connection, &candidate.target).await;
        let replay_attempt = AssertUnwindSafe(async {
            navigation_finalize(replay.as_mut()).await;
            let current = navigation_snapshot(replay.as_mut(), true, 0).await;
            assert!(
                current.capture == candidate.capture
                    && current.catalog == candidate.catalog
                    && current.rows == candidate.rows
                    && current.ledger == candidate.ledger
            );
            assert_eq!(
                current.tuple, candidate.tuple,
                "new-connection replay repeated DDL"
            );
            group_locks(replay.as_mut(), &oracle).await;
        })
        .catch_unwind()
        .await;
        match replay_attempt {
            Ok(()) => replay.commit().await.unwrap(),
            Err(panic) => {
                replay
                    .rollback()
                    .await
                    .expect("new-connection replay rollback");
                replay_connection.close().await.unwrap();
                navigation_restored(pool, &candidate, true, 0).await;
                std::panic::resume_unwind(panic)
            }
        }
        replay_connection.close().await.unwrap();
        navigation_restored(pool, &candidate, true, 0).await;
        json!({"confirmed":true,"sources":sources,"variant":"plain",
            "held_api_worker_before":200,"held_api_worker_after":503,"fresh_api_worker_after":200,
            "full83":NAVIGATION_CORRECTED83[0],"ledger_records":231,
            "catalog_only_eleven_site_correction":true,"all_durable_rows_unchanged":true,
            "same_transaction_replay_unchanged":true,"new_connection_replay_unchanged":true,
            "fresh_postcommit_readback":true,"retained_exact83_locks":true,
            "correction_elapsed_millis":u64::try_from(elapsed.as_millis()).unwrap(),
            "fresh_readiness_millis":fresh_readiness_millis,
            "metadata":navigation_browser_metadata(&candidate),
            "browser_accepted":false,"production_qualified":false})
    })
    .catch_unwind()
    .await;
    admin.close().await.unwrap();
    // A confirmed successor is never resealed or reverted after commit. Failed
    // assertions preserve its state for evidence and still close owned states.
    for state in &states {
        state.shutdown_realtime().await;
    }
    drop(states);
    drop(artifacts);
    let mut receipt = match outcome {
        Ok(receipt) => receipt,
        Err(panic) => std::panic::resume_unwind(panic),
    };
    // Test-only planning comparison on the exact corrected disposable target.
    // Original fixture cleanup/readback is complete; this adds no serving claim.
    let marker = "), snapshots AS (\n";
    let hinted_marker = "), snapshots AS MATERIALIZED (\n";
    assert_eq!(NAVIGATION_APP_STATE.matches(marker).count(), 1);
    let hinted = NAVIGATION_APP_STATE.replacen(marker, hinted_marker, 1);
    assert_eq!(
        hinted.replacen(hinted_marker, marker, 1),
        NAVIGATION_APP_STATE
    );
    let runtime_url = console_platform_test_support::login_test_database_url(
        pool,
        console_platform_test_support::TestDatabaseLogin::Business,
    );
    let mut runtime = PgConnection::connect(&runtime_url)
        .await
        .unwrap_or_else(|_| panic!("classifier comparison Business LOGIN connection failed"));
    let mut read = sqlx::Connection::begin(&mut runtime)
        .await
        .unwrap_or_else(|_| panic!("classifier comparison read transaction failed"));
    let comparison = AssertUnwindSafe(async {
        sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
            .execute(read.as_mut())
            .await
            .unwrap_or_else(|_| panic!("classifier comparison read-only snapshot failed"));
        sqlx::raw_sql(CLASSIFIER_SESSION)
            .execute(read.as_mut())
            .await
            .unwrap_or_else(|_| panic!("classifier comparison fixed session failed"));
        let original_started = std::time::Instant::now();
        let original_state: String = sqlx::query_scalar(NAVIGATION_APP_STATE)
            .fetch_one(read.as_mut())
            .await
            .unwrap_or_else(|_| panic!("original classifier read-only comparison failed"));
        let original_elapsed_millis =
            u64::try_from(original_started.elapsed().as_millis()).unwrap();
        let hinted_started = std::time::Instant::now();
        let hinted_state: String = sqlx::query_scalar(sqlx::AssertSqlSafe(hinted.as_str()))
            .fetch_one(read.as_mut())
            .await
            .unwrap_or_else(|_| panic!("materialized classifier read-only comparison failed"));
        let hinted_elapsed_millis = u64::try_from(hinted_started.elapsed().as_millis()).unwrap();
        let original_plan: Value = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "EXPLAIN (ANALYZE, FORMAT JSON) {NAVIGATION_APP_STATE}"
        )))
        .fetch_one(read.as_mut())
        .await
        .unwrap_or_else(|_| panic!("original classifier read-only plan failed"));
        let hinted_plan: Value = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "EXPLAIN (ANALYZE, FORMAT JSON) {hinted}"
        )))
        .fetch_one(read.as_mut())
        .await
        .unwrap_or_else(|_| panic!("materialized classifier read-only plan failed"));
        let metrics = json!({"original_elapsed_millis":original_elapsed_millis,
            "materialized_elapsed_millis":hinted_elapsed_millis,
            "original_source_sha256":digest(NAVIGATION_APP_STATE),
            "materialized_source_sha256":digest(&hinted),
            "original_state":original_state,"materialized_state":hinted_state,
            "read_only":true,"statement_timeout_millis":3000,
            "order":"original_then_materialized","performance_qualified":false,
            "original_planning_millis":original_plan[0]["Planning Time"],
            "original_execution_millis":original_plan[0]["Execution Time"],
            "materialized_planning_millis":hinted_plan[0]["Planning Time"],
            "materialized_execution_millis":hinted_plan[0]["Execution Time"]});
        eprintln!(
            "GROUP_CLASSIFIER_MATERIALIZED_DIAGNOSTIC {}",
            json!({
            "metrics":metrics,"original_explain_analyze":original_plan,
            "materialized_explain_analyze":hinted_plan})
        );
        assert_eq!(original_state, hinted_state);
        assert_eq!(original_state, "native_group_process_navigation.finalized");
        metrics
    })
    .catch_unwind()
    .await;
    let rollback = read.rollback().await;
    let close = runtime.close().await;
    assert!(
        rollback.is_ok() && close.is_ok(),
        "classifier comparison cleanup failed"
    );
    let metrics = match comparison {
        Ok(metrics) => metrics,
        Err(panic) => std::panic::resume_unwind(panic),
    };
    receipt["classifier_materialized_experiment"] = metrics;
    receipt
}

fn navigation_browser_metadata(candidate: &NavigationRuntimeBaseline) -> Value {
    json!({"database":candidate.target.database,"database_oid":candidate.target.database_oid,
        "system_identifier":candidate.target.system_identifier,"capture_sha256":candidate.capture.sha256,
        "catalog_sha256":digest(&candidate.catalog.0),"ledger_sha256":digest(&candidate.ledger.to_string()),
        "navigation_tuple":candidate.tuple})
}

// Read-only, independently opened post-probe metadata evidence. No private
// Target/Baseline, SQL authority, row contents or credentials cross this boundary.
pub(in super::super::super::super) async fn native_group_navigation_browser_metadata(
    pool: &PgPool,
) -> Value {
    navigation_finalizer_pins();
    let mut connection = direct(pool).await;
    let mut tx = sqlx::Connection::begin(&mut connection).await.unwrap();
    sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
        .execute(tx.as_mut())
        .await
        .unwrap();
    let candidate = navigation_snapshot(tx.as_mut(), true, 0).await;
    let result = navigation_browser_metadata(&candidate);
    tx.rollback().await.unwrap();
    connection.close().await.unwrap();
    result
}
