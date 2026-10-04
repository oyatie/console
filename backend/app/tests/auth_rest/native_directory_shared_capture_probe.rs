// Test-only classifier discriminator. No serving switch or alternate installer.
pub(super) mod native_directory_shared_capture_probe {
    use super::*;

    const ORIGINAL: &str =
        include_str!("../../../../ops/postgres-native-people-directory-row-lock-custody-state.sql");
    const PROPOSED: &str = include_str!("fixtures/native-directory-row-lock-shared-capture-v1.sql");
    const PROVENANCE: &str = include_str!("../../src/company_provenance_v1_custody_state.sql");
    const POLICY_V2: &str = include_str!("../../src/native_company_policy_v2_custody_state.sql");
    type Witness = (String, String, String, Option<bool>);

    async fn read_transaction(pool: &PgPool, diagnostic: bool) -> Transaction<'_, Postgres> {
        let mut tx = pool.begin().await.unwrap();
        sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
            .execute(tx.as_mut())
            .await
            .unwrap();
        sqlx::raw_sql(CLASSIFIER_SESSION)
            .execute(tx.as_mut())
            .await
            .unwrap();
        if diagnostic {
            // DIAGNOSTIC ONLY: obtains original bytes; never qualifies serving.
            sqlx::raw_sql("SET LOCAL statement_timeout='30s'")
                .execute(tx.as_mut())
                .await
                .unwrap();
        }
        let identity: (String, String, bool, bool) = sqlx::query_as(
            "SELECT session_user::text,current_user::text,rolsuper,rolbypassrls \
             FROM pg_catalog.pg_roles WHERE rolname=current_user",
        )
        .fetch_one(tx.as_mut())
        .await
        .unwrap();
        assert_eq!(
            identity,
            ("console_rt".into(), "console_rt".into(), false, false)
        );
        let settings: (String, String, String, String, String) = sqlx::query_as(
            "SELECT current_setting('transaction_isolation'),current_setting('transaction_read_only'),\
             current_setting('search_path'),current_setting('jit'),current_setting('statement_timeout')"
        ).fetch_one(tx.as_mut()).await.unwrap();
        assert_eq!(
            (
                settings.0.as_str(),
                settings.1.as_str(),
                settings.3.as_str()
            ),
            ("repeatable read", "on", "off")
        );
        assert_eq!(
            settings.2.split(',').map(str::trim).collect::<Vec<_>>(),
            ["pg_catalog", "pg_temp"]
        );
        assert_eq!(settings.4, if diagnostic { "30s" } else { "3s" });
        tx
    }

    fn with_witness(source: &str) -> String {
        // Only the terminal SELECT adds diagnostics; all state rules remain exact.
        let prefix = source
            .strip_suffix("END AS state;\n")
            .expect("reviewed classifier terminal SELECT changed");
        format!(
            "{prefix}END AS state,\n\
            (SELECT snapshot::text FROM full_capture) AS snapshot_bytes,\n\
            (SELECT snapshot_sha256 FROM full_capture) AS snapshot_sha256,\n\
            (SELECT native_directory_startup_rights_valid FROM full_capture) AS rights_valid;\n"
        )
    }

    async fn witness(tx: &mut Transaction<'_, Postgres>, source: &str, label: &str) -> Witness {
        let started = std::time::Instant::now();
        let result = sqlx::query_as(sqlx::AssertSqlSafe(with_witness(source)))
            .fetch_one(tx.as_mut())
            .await;
        eprintln!(
            "DIRECTORY_DIAGNOSTIC_ONLY statement={label} budget=30s elapsed={:?}",
            started.elapsed()
        );
        result.unwrap_or_else(|error| panic!("diagnostic statement={label}: {error:?}"))
    }

    async fn named_three_second_probe(
        pool: &PgPool,
        source: &str,
        label: &str,
        expected: &str,
        required: bool,
    ) {
        let mut tx = read_transaction(pool, false).await;
        let started = std::time::Instant::now();
        let result: Result<String, sqlx::Error> = sqlx::query_scalar(sqlx::AssertSqlSafe(source))
            .fetch_one(tx.as_mut())
            .await;
        tx.rollback().await.unwrap();
        match result {
            Ok(state) => {
                eprintln!(
                    "DIRECTORY_3S_PROBE statement={label} state={state} elapsed={:?}",
                    started.elapsed()
                );
                assert_eq!(state, expected, "wrong verdict from {label}");
            }
            Err(error) => {
                let code = error.as_database_error().and_then(|db| db.code());
                eprintln!(
                    "DIRECTORY_3S_PROBE statement={label} error={code:?} elapsed={:?}",
                    started.elapsed()
                );
                assert_eq!(
                    code.as_deref(),
                    Some("57014"),
                    "unrelated statement failure from {label}: {error:?}"
                );
                assert!(
                    !required,
                    "required unchanged-3s statement={label} timed out"
                );
                // Observed original timeout is diagnosis, never a passing budget claim.
            }
        }
    }

    #[sqlx::test(migrations = false)]
    async fn staged_policy_v2_shared_capture_matches_and_actual_startup_meets_three_seconds(
        pool: PgPool,
    ) {
        assert_eq!(
            hex::encode(Sha256::digest(ORIGINAL.as_bytes())),
            "781cc446ce26525cbad6cd281c66239d1a7366eb9369d914068f39886cad3d6e"
        );
        assert_eq!(
            hex::encode(Sha256::digest(PROPOSED.as_bytes())),
            "3e27bde257a6cd7c4363bfc9e43b7f1a66e38c7efdcd0580aba043381425dfae"
        );
        assert_eq!(
            CLASSIFIER_SESSION,
            "SET LOCAL search_path=pg_catalog,pg_temp;\nSET LOCAL statement_timeout='3s';\nSET LOCAL jit=off;\n"
        );
        // Genuine migrations and existing Account/Company/policy v1/v2 finalizers.
        // This helper's own v2/3s validation is a prerequisite, never bypassed.
        native_people_codec2_install_probe::prepare_successor_ready_database(&pool).await;
        assert_actual_applied_ledger(&pool).await;
        let before = all_rows(&pool).await;
        let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
        let artifacts = Artifacts::new();
        let key = SigningKey::random(&mut OsRng);
        let config = account_browser_config(&pool, artifacts.root.clone(), &key);
        let mut states = Vec::new();
        let outcome = AssertUnwindSafe(async {
            let mut tx = read_transaction(&runtime, true).await;
            let original = witness(&mut tx, ORIGINAL, "original-row-lock").await;
            let proposed = witness(&mut tx, PROPOSED, "shared-row-lock").await;
            assert!(
                original == proposed,
                "old/shared verdict or complete snapshot/digest/rights differ"
            );
            assert_eq!(original.0, "native_people_directory.staged_closed");
            assert_eq!(original.2, STAGED[0], "genuine staged fixture changed");
            assert_eq!(original.3, Some(true));
            assert_eq!(
                hex::encode(Sha256::digest(original.1.as_bytes())),
                original.2,
                "snapshot bytes do not match their digest"
            );
            tx.rollback().await.unwrap();

            // Label each statement on the actual staged serving path independently.
            for (source, label, expected) in [
                (
                    PROVENANCE,
                    "company-provenance",
                    "company_provenance.absent",
                ),
                (
                    ORIGINAL,
                    "original-row-lock",
                    "native_people_directory.staged_closed",
                ),
                (POLICY_V2, "policy-v2", "native_company_policy_v2.finalized"),
            ] {
                named_three_second_probe(&runtime, source, label, expected, false).await;
            }
            named_three_second_probe(
                &runtime,
                PROPOSED,
                "shared-row-lock",
                "native_people_directory.staged_closed",
                true,
            )
            .await;
            assert!(
                before == all_rows(&pool).await,
                "read-only probes changed durable rows"
            );

            // Real owning composition still uses canonical serving code and its 3s.
            let state = AppState::from_config(config)
                .await
                .expect("ACTUAL_APPSTATE_STARTUP_WITH_CANONICAL_3S_CUSTODY");
            states.push(state.clone());
            assert_eq!(
                ready_status(&state).await,
                StatusCode::OK,
                "actual owner readiness must satisfy canonical 3s custody"
            );
        })
        .catch_unwind()
        .await;
        let unchanged = before == all_rows(&pool).await;
        runtime.close().await;
        for state in states {
            state.shutdown_realtime().await;
        }
        assert!(unchanged, "classification/startup changed durable rows");
        if let Err(panic) = outcome {
            std::panic::resume_unwind(panic);
        }
    }
}
