// Group-only browser prerequisite. Included inside the reviewed private Group
// finalizer module; no private Baseline/Target or second protocol is exposed.
pub(in super::super::super::super) async fn prepare_native_group_browser_database(pool: &PgPool) {
    let sources = group_pins();
    let oracle = group_oracle();
    let original = predecessor(pool, 0).await;
    let closed = group_closed_fixture(pool, &original, 0).await;
    let artifacts = Artifacts::new();
    let key = SigningKey::random(&mut OsRng);
    let config = account_browser_config(pool, artifacts.root.clone(), &key);
    let mut states = Vec::<AppState>::new();
    let mut admin = direct(pool).await;
    let outcome = AssertUnwindSafe(async {
        // Use configured login pools and real verified profiles. The temporary
        // fixture origin is never reused by the subsequent TLS browser state.
        for role in [console_app::AppRole::Api, console_app::AppRole::Worker] {
            let mut held_config = config.clone();
            held_config.role = role;
            let held = AppState::from_config(held_config)
                .await
                .expect("real closed Group predecessor must support API and Worker startup");
            states.push(held);
            assert_eq!(ready_status(states.last().unwrap()).await, StatusCode::OK);
        }
        let mut install = begin_protocol(&mut admin, &closed.target).await;
        let history = group_install_replay(install.as_mut(), &closed, 0, &oracle).await;
        let installed_capture = group_capture(install.as_mut(), true).await;
        let installed_catalog = catalog(install.as_mut()).await;
        let installed_rows = rows(install.as_mut()).await;
        install.commit().await.unwrap();

        // A reader bound to Bridge must not silently become a Group reader.
        for held in &states {
            assert_eq!(
                ready_status(held).await,
                StatusCode::SERVICE_UNAVAILABLE,
                "held Bridge profile must fence after committed Group successor"
            );
        }
        // Request shutdown and drop held states before constructing fresh roles.
        // Listener completion and pool readmission still need runtime evidence.
        for held in &states {
            held.shutdown_realtime().await;
        }
        states.clear();
        for role in [console_app::AppRole::Api, console_app::AppRole::Worker] {
            let mut fresh_config = config.clone();
            fresh_config.role = role;
            let fresh = AppState::from_config(fresh_config)
                .await
                .expect("genuine committed Group profile must compose API and Worker startup");
            states.push(fresh);
            assert_eq!(ready_status(states.last().unwrap()).await, StatusCode::OK);
            states.last().unwrap().shutdown_realtime().await;
            states.pop();
        }

        // Fresh postcommit full readback, including startup's actual no-effects.
        assert_eq!(target(&mut admin).await, closed.target);
        let mut readback = sqlx::Connection::begin(&mut admin).await.unwrap();
        assert_eq!(
            group_classification(readback.as_mut()).await,
            (
                "native_group_process.finalized".into(),
                Some("plain".into())
            )
        );
        assert!(
            group_capture(readback.as_mut(), true).await == installed_capture,
            "configured startup/readiness changed exact installed83 capture"
        );
        assert!(
            catalog(readback.as_mut()).await == installed_catalog,
            "configured startup/readiness changed complete raw catalog"
        );
        assert!(
            rows(readback.as_mut()).await == installed_rows,
            "configured startup/readiness changed any durable row"
        );
        unchanged_group_rows(&closed.rows, &installed_rows, &oracle);
        assert!(applied_ledger(readback.as_mut()).await == closed.ledger);
        readback.rollback().await.unwrap();
        let mut output = std::io::stderr().lock();
        writeln!(&mut output, "GROUP_BROWSER_PREREQUISITE {}", json!({
            "sources":sources,"variant":"plain","installation":history,
            "target":{"database":closed.target.database,"database_oid":closed.target.database_oid,
                "system_identifier":closed.target.system_identifier},
            "held_api_worker_before":200,"held_api_worker_after":503,
            "fresh_api_worker_after":200,"temporary_states_shutdown_requested_and_dropped":true,
            "postcommit_catalog_rows_ledger_unchanged":true,"new_group_tables":7,
            "new_group_tables_empty":true,"historical_prerequisite_owner_effects":true,
            "business_empty":false,"group_browser_accepted":false,"production_qualified":false
        })).expect("Group browser prerequisite evidence write failed");
        output
            .flush()
            .expect("Group browser prerequisite evidence flush failed");
    })
    .catch_unwind()
    .await;
    // Request state shutdown and close the actual direct connection on panic.
    admin.close().await.unwrap();
    close_states(&states, outcome).await;
    drop(states);
    drop(artifacts);
}
