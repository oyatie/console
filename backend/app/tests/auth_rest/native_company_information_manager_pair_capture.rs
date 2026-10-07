// Additive prerequisite observation only; never a serving profile or business RED.
mod native_company_information_manager_pair_capture {
    use super::*;
    use std::io::Write as _;

    const SOURCE: &str =
        include_str!("../../../../ops/postgres-company-information-manager-current-v1-owner.sql");
    const QUERY: &str = include_str!(
        "../../../../ops/postgres-capture-company-information-manager-current-v1-custody.sql"
    );
    const OLD_QUERY: &str =
        include_str!("../../../../ops/postgres-capture-native-company-policy-custody.sql");
    const PREDECESSORS: [&str; 2] = [
        "3755792f52a4f78236a70e509f4f1546588049daa53f7545d8a101c74684d843",
        "9bde6410d51f8b665e5ca4400820e6d8a549a6c19b7b4d84feb26ef32c4bd604",
    ];
    const NAMESPACE_KEY: &str = "company_information_manager_current_routine_namespace";

    include!("native_company_information_manager_capture_support.rs");
    include!("native_company_information_manager_capture_oracles.rs");
    include!("native_company_information_manager_capture_controls.rs");

    #[sqlx::test(migrations = false)]
    async fn captures_actual_manager_plain_and_observer_policy_v1_metadata_then_rolls_back(
        pool: PgPool,
    ) {
        let pins = manager_source_pins();
        let mut initial = pool.begin().await.unwrap();
        marked(initial.as_mut(), true).await;
        initial.rollback().await.unwrap();
        // Actual Policy V1 prerequisite. No Accounts, Companies, business grants
        // or business commands are created. Historical migrations and the
        // existing fixture's TEST_ONLY terms publication remain unchanged.
        prepare_policy_ready_database(&pool).await;
        let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
        let outcome = AssertUnwindSafe(async {
            let login: (String, String, bool, bool) = sqlx::query_as(
                "SELECT session_user::text,current_user::text,rolsuper,rolbypassrls FROM pg_catalog.pg_roles WHERE rolname=current_user"
            ).fetch_one(&runtime).await.unwrap();
            assert_eq!(login, ("console_rt".into(), "console_rt".into(), false, false));
            assert_eq!(policy_classified(&runtime).await, "native_company_policy.finalized");
            let mut initial = pool.begin().await.unwrap();
            marked(initial.as_mut(), false).await;
            let ledger = applied_ledger(initial.as_mut()).await;
            let baseline_rows = rows(initial.as_mut()).await;
            let baseline_catalog = catalog(initial.as_mut()).await;
            assert!(manager_namespace(initial.as_mut()).await.is_null());
            manager_observer_absent(initial.as_mut()).await;
            let baseline = policy_capture(initial.as_mut(), OLD_QUERY).await;
            assert_eq!(baseline.sha256, PREDECESSORS[0]);
            assert!(baseline.rights, "original Policy V1 startup rights invalid");
            let database_identity: Value = sqlx::query_scalar(
                "SELECT jsonb_build_object('database',current_database(),'session_user',session_user,'current_user',current_user,'fixture_marker',current_setting('console.sqlx_test_bootstrap',true),'system_identifier',(pg_catalog.pg_control_system()).system_identifier::text,'server_version_num',current_setting('server_version_num'),'fsync',current_setting('fsync'),'synchronous_commit',current_setting('synchronous_commit'),'full_page_writes',current_setting('full_page_writes'))"
            ).fetch_one(initial.as_mut()).await.unwrap();
            initial.rollback().await.unwrap();
            let mut packets = Vec::new();
            for (variant, predecessor) in PREDECESSORS.iter().enumerate() {
                let mut tx = pool.begin().await.unwrap();
                let attempt = AssertUnwindSafe(async {
                    sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL READ COMMITTED; SET LOCAL search_path=pg_catalog,pg_temp; SET LOCAL lock_timeout='1s'; SET LOCAL statement_timeout='120s'")
                        .execute(tx.as_mut()).await.unwrap();
                    marked(tx.as_mut(), false).await;
                    // Coordinate cluster-global optional observer role/ACL work.
                    let _: String = sqlx::query_scalar("SELECT oid::text FROM pg_catalog.pg_authid WHERE rolname='console_account_owner' FOR UPDATE")
                        .fetch_one(tx.as_mut()).await.unwrap();
                    manager_observer_absent(tx.as_mut()).await;
                    assert!(policy_capture(tx.as_mut(), OLD_QUERY).await == baseline);
                    if variant == 1 { execute(&mut tx, OBSERVER).await; }
                    let old = policy_capture(tx.as_mut(), OLD_QUERY).await;
                    assert_eq!(&old.sha256, predecessor, "original raw V1 fingerprint changed");
                    assert!(manager_namespace(tx.as_mut()).await.is_null());
                    let absent_extended = policy_capture(tx.as_mut(), QUERY).await;
                    assert!(old.rights && absent_extended.rights, "predecessor startup rights invalid");
                    absent_extension(&old, &absent_extended);
                    let before_rows = rows(tx.as_mut()).await;
                    let before_catalog = catalog(tx.as_mut()).await;
                    assert_eq!(applied_ledger(tx.as_mut()).await, ledger);
                    // Exact source creates + owner/ACL block install together in
                    // this retained disposable transaction; ACL remains last.
                    execute(&mut tx, SOURCE).await;
                    let successor = policy_capture(tx.as_mut(), QUERY).await;
                    assert!(successor.rights, "new source changed required startup rights");
                    assert!(successor != absent_extended, "capture omitted source metadata");
                    assert!(policy_capture(tx.as_mut(), QUERY).await == successor);
                    assert_eq!(successor.snapshot[NAMESPACE_KEY], expected_namespace());
                    assert_eq!(manager_namespace(tx.as_mut()).await, expected_namespace());
                    let after_catalog = catalog(tx.as_mut()).await;
                    manager_catalog_delta(&before_catalog.1, &after_catalog.1);
                    assert!(catalog(tx.as_mut()).await == after_catalog);
                    assert!(rows(tx.as_mut()).await == before_rows);
                    assert_eq!(applied_ledger(tx.as_mut()).await, ledger);
                    let controls = manager_controls(&mut tx, &successor, &after_catalog, &before_rows, &ledger).await;
                    json!({"schema":"console.company_information_manager_pair_capture.v1",
                        "variant":if variant==0 {"plain"} else {"observer"},
                        "sources":pins,"database_identity":database_identity,"applied_ledger":ledger,
                        "predecessor":old.record(),"absent_source_extended":absent_extended.record(),
                        "successor":successor.record(),"namespace":expected_namespace(),
                        "raw_catalog_before_text":before_catalog.0,"raw_catalog_after_text":after_catalog.0,
                        "raw_catalog_before_sha256":digest(&before_catalog.0),"raw_catalog_after_sha256":digest(&after_catalog.0),
                        "business_rows":row_summary(&before_rows),"metadata_controls":controls,
                        "business_history_unchanged":true,"successor_hash_is_observed_not_accepted":true,
                        "business_empty":false,"workflow_accepted":false,"production_qualified":false})
                }).catch_unwind().await;
                tx.rollback().await.expect("source and observer metadata must roll back");
                let mut restored = pool.begin().await.unwrap();
                assert!(rows(restored.as_mut()).await == baseline_rows);
                assert!(catalog(restored.as_mut()).await == baseline_catalog);
                assert_eq!(applied_ledger(restored.as_mut()).await, ledger);
                assert!(manager_namespace(restored.as_mut()).await.is_null());
                manager_observer_absent(restored.as_mut()).await;
                assert!(policy_capture(restored.as_mut(), OLD_QUERY).await == baseline);
                restored.rollback().await.unwrap();
                assert_eq!(policy_classified(&runtime).await, "native_company_policy.finalized");
                match attempt {
                    Ok(mut packet) => {
                        packet["complete_rollback_verified"] = json!(true);
                        packets.push(packet);
                    }
                    Err(panic) => std::panic::resume_unwind(panic),
                }
            }
            assert_eq!(packets.len(), 2);
            assert_ne!(packets[0]["successor"]["snapshot_sha256"], packets[1]["successor"]["snapshot_sha256"]);
            // Publish only after every control and both complete rollbacks pass.
            // Original PostgreSQL text/hash pairs remain separately preserved.
            let mut output = std::io::stderr().lock();
            for packet in packets {
                writeln!(&mut output, "COMPANY_INFORMATION_MANAGER_PAIR_CAPTURE {packet}")
                    .expect("paired Manager capture publication failed");
            }
            writeln!(&mut output, "COMPANY_INFORMATION_MANAGER_PAIR_CAPTURE_COMPLETE variants=2 controls=30 ledger231=verified historical230=verified rows=unchanged rollback=verified acceptance=not_claimed")
                .expect("paired Manager capture completion publication failed");
            output.flush().expect("paired Manager capture flush failed");
        }).catch_unwind().await;
        runtime.close().await;
        if let Err(panic) = outcome {
            std::panic::resume_unwind(panic);
        }
    }
}
