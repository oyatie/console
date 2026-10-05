// Generated Group custody against the actual marked disposable PostgreSQL owner.
// This metadata fixture contains inherited historical rows, never UI-created
// business evidence. Existing diagnostics, browser semantics and items are frozen.
mod native_group_process_finalizer_tests {
    use super::*;
    use sqlx::Connection as _;
    use std::io::Write as _;

    const GROUP_FINALIZER: &str =
        include_str!("../../../../ops/postgres-finalize-native-group-process-v1.sql");
    const GROUP_STATE: &str =
        include_str!("../../../../ops/postgres-native-group-process-v1-custody-state.sql");
    const GROUP_APP_STATE: &str =
        include_str!("../../src/native_group_process_v1_custody_state.sql");
    const GROUP_OWNER: &str =
        include_str!("../../../../ops/postgres-native-group-process-v1-owner.sql");
    const GROUP_CAPTURE: &str =
        include_str!("../../../../ops/postgres-capture-native-group-process-v1-custody.sql");
    const GROUP_ORACLE: &str =
        include_str!("fixtures/native-group-full83-diagnostic-oracle-v1.json");
    const BEFORE83: [&str; 2] = [
        "e14842248916f3d79770fba90adb18a6eb947ec4be052f04103b87acae1dd651",
        "342aedf98ddcc8646cb50576abdf0cacbdd3a0cc5adf56e3931b678b91250a99",
    ];
    const INSTALLED83: [&str; 2] = [
        "ec5c2d1523e69520ac32f3d253c1c222d52e1b01bba12040328502ab319d6862",
        "cda9967f7b8b267e5294551314c80fd9fc1795f962450b7b0f3d6353f11043a6",
    ];

    fn group_pins() -> Value {
        let mut result = pins();
        assert_eq!(GROUP_STATE, GROUP_APP_STATE);
        for (name, source, expected) in [
            (
                "group_finalizer",
                GROUP_FINALIZER,
                "2f7d9463df35c8f5ffa679ffacff869b8724e2fe042e2bdeadd5498a4e0e8f65",
            ),
            (
                "group_state",
                GROUP_STATE,
                "c44e239958fd5e716ddaec66459aba45df783fedf63b1d4d7939425fd018491d",
            ),
            (
                "group_owner",
                GROUP_OWNER,
                "cbf641175a7bf589bd46fc21dc775fe2fab8b1a8ab7e46b04dee3c32422038f9",
            ),
            (
                "group_capture",
                GROUP_CAPTURE,
                "3406bac381896fab4e9a1c079110d3dc770a9d89b0b564e7744034379f60cd3b",
            ),
            (
                "group_oracle",
                GROUP_ORACLE,
                "16f95a8bdc1f38bae675a8fce908ce54a959fdcd50f7a524362ed1cdc0c7a909",
            ),
        ] {
            assert_eq!(
                digest(source),
                expected,
                "unreviewed Group fixture source: {name}"
            );
            assert!(
                result
                    .as_object_mut()
                    .unwrap()
                    .insert(name.into(), json!(expected))
                    .is_none()
            );
        }
        assert_eq!(GROUP_FINALIZER.matches(GROUP_OWNER).count(), 1);
        result
    }

    fn group_oracle() -> Value {
        let oracle: Value = serde_json::from_str(GROUP_ORACLE).unwrap();
        assert_eq!(oracle["source_owner_sha256"], digest(GROUP_OWNER));
        assert_eq!(oracle["source_order"].as_array().unwrap().len(), 14);
        assert_eq!(oracle["routines"].as_array().unwrap().len(), 41);
        assert!(
            oracle["phase_pairs"].is_null(),
            "frozen diagnostic never becomes installed authority"
        );
        let old: BTreeSet<_> = oracle["historical76_relations"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap())
            .collect();
        assert_eq!(old, RELATIONS76.into_iter().collect());
        oracle
    }

    fn group_names(oracle: &Value) -> Vec<String> {
        let names: BTreeSet<_> = oracle["relation_roster"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap().to_owned())
            .collect();
        assert_eq!(names.len(), 7);
        names.into_iter().collect()
    }

    async fn group_classification(connection: &mut PgConnection) -> (String, Option<String>) {
        sqlx::raw_sql(CLASSIFIER_SESSION)
            .execute(&mut *connection)
            .await
            .unwrap();
        // Preserve the generated classifier in full; only expose its already
        // computed same-variant result beside state, as the finalizer does.
        let body = GROUP_STATE
            .strip_suffix(" AS state;\n")
            .expect("exact classifier terminator");
        let query = format!("{body} AS state,(SELECT variant FROM matching_phase) AS variant");
        sqlx::query_as(sqlx::AssertSqlSafe(query))
            .fetch_one(connection)
            .await
            .unwrap()
    }

    async fn group_capture(connection: &mut PgConnection, expected_rights: bool) -> Capture {
        sqlx::raw_sql(CLASSIFIER_SESSION)
            .execute(&mut *connection)
            .await
            .unwrap();
        let body = GROUP_CAPTURE
            .strip_suffix(";\n")
            .expect("exact full83 capture terminator");
        let query = format!(
            "SELECT snapshot::text,snapshot_sha256,native_group_process_startup_rights_valid FROM ({body}) full_capture"
        );
        let (text, sha256, rights): (String, String, Option<bool>) =
            sqlx::query_as(sqlx::AssertSqlSafe(query))
                .fetch_one(connection)
                .await
                .unwrap();
        assert_eq!(
            digest(&text),
            sha256,
            "raw PostgreSQL UTF-8 snapshot digest differs"
        );
        assert_eq!(
            rights,
            Some(expected_rights),
            "NULL or wrong complete83 rights verdict"
        );
        let snapshot: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(snapshot["tables"].as_array().unwrap().len(), 83);
        assert_eq!(
            snapshot["deployment_operator_boundary"]["startup_final_rights_valid"], false,
            "serialized historical18 predicate remains raw FALSE"
        );
        Capture {
            text,
            sha256,
            rights: expected_rights,
            snapshot,
        }
    }

    fn assert_absent(snapshot: &Value) {
        for key in [
            "native_group_process_relation_namespace",
            "native_group_process_schema_namespace",
            "native_group_process_type_namespace",
            "native_group_process_routine_namespace",
        ] {
            assert!(snapshot.as_object().unwrap().contains_key(key));
            assert!(
                snapshot[key].is_null(),
                "whole reserved namespace must be absent: {key}"
            );
        }
    }

    fn unchanged_group_rows(
        before: &BTreeMap<String, String>,
        after: &BTreeMap<String, String>,
        oracle: &Value,
    ) {
        let mut existing = after.clone();
        for name in group_names(oracle) {
            let identity = serde_json::to_string(&("public", name)).unwrap();
            assert!(!before.contains_key(&identity));
            assert_eq!(
                existing.remove(&identity).as_deref(),
                Some("[]"),
                "new Group tables must remain business-empty"
            );
        }
        assert!(
            existing == *before,
            "Group finalizer changed an existing durable business/history row"
        );
        assert_eq!(after.len(), before.len() + 7);
    }

    fn actual_group_metadata(before: &Value, after: &Value, captured: &Value, oracle: &Value) {
        let old = routine_map(before);
        let new = routine_map(after);
        assert_eq!(new.len(), old.len() + 41);
        assert!(
            old.iter()
                .all(|(identity, value)| new.get(identity) == Some(value)),
            "Group finalizer changed an existing routine"
        );
        for expected in oracle["routines"].as_array().unwrap() {
            let identity = format!(
                "public.{}({})",
                expected["name"].as_str().unwrap(),
                expected["type_arguments"].as_str().unwrap()
            );
            let routine = &new[&identity];
            assert_eq!(routine["owner"], "console_account_owner");
            for field in [
                "name",
                "language",
                "identity_arguments",
                "result",
                "source_sha256",
            ] {
                assert_eq!(
                    routine[field], expected[field],
                    "declared Group metadata: {identity}/{field}"
                );
            }
            assert_eq!(
                routine["raw"]["prosrc"], expected["body"],
                "exact declared Group body: {identity}"
            );
            for (field, expected_field) in [
                ("provolatile", "volatility"),
                ("proparallel", "parallel"),
                ("proisstrict", "strict"),
                ("prosecdef", "security_definer"),
                ("proretset", "returns_set"),
                ("proargnames", "argnames"),
                ("proargmodes", "argmodes"),
                ("proconfig", "config"),
            ] {
                assert_eq!(
                    routine["raw"][field], expected[expected_field],
                    "declared Group ABI: {identity}/{field}"
                );
            }
            let runtime = expected["runtime_execute"].as_bool().unwrap();
            let mut acl = vec![json!([
                "console_account_owner",
                "console_account_owner",
                "EXECUTE",
                false
            ])];
            if runtime {
                acl.push(json!([
                    "console_account_owner",
                    "console_rt",
                    "EXECUTE",
                    false
                ]));
            }
            assert_eq!(routine["acl"], json!(acl));
        }
        for key in [
            "roles",
            "memberships",
            "database",
            "database_role_settings",
            "schemas",
            "default_acls",
            "event_triggers",
        ] {
            assert_eq!(
                before[key], after[key],
                "Group finalizer changed unrelated catalog: {key}"
            );
        }
        let expected: BTreeSet<_> = group_names(oracle).into_iter().collect();
        let relation_namespace = captured["native_group_process_relation_namespace"]
            .as_array()
            .unwrap();
        assert_eq!(relation_namespace.len(), 30);
        assert!(relation_namespace.iter().all(|row| row[0] == "public"
            && row[3] == "console_account_owner"
            && (row[2] == "r" || row[2] == "i")));
        assert_eq!(
            relation_namespace
                .iter()
                .filter(|row| row[2] == "r")
                .map(|row| row[1].as_str().unwrap().to_owned())
                .collect::<BTreeSet<_>>(),
            expected
        );
        assert!(captured["native_group_process_schema_namespace"].is_null());
        let types = captured["native_group_process_type_namespace"]
            .as_array()
            .unwrap();
        assert_eq!(types.len(), 7);
        assert_eq!(
            types
                .iter()
                .map(|row| {
                    assert_eq!(row[0], "public");
                    assert_eq!(row[2], "c");
                    assert_eq!(row[3], "console_account_owner");
                    row[1].as_str().unwrap().to_owned()
                })
                .collect::<BTreeSet<_>>(),
            expected
        );
        assert_eq!(
            captured["native_group_process_routine_namespace"]
                .as_array()
                .unwrap()
                .len(),
            41
        );
    }

    async fn group_locks(connection: &mut PgConnection, oracle: &Value) -> Value {
        let mut expected: Vec<String> = RELATIONS76.into_iter().map(str::to_owned).collect();
        expected.extend(group_names(oracle));
        expected.sort();
        assert_eq!(expected.len(), 83);
        assert_eq!(expected.iter().collect::<BTreeSet<_>>().len(), 83);
        let actual: Vec<String> = sqlx::query_scalar(
            "SELECT c.relname::text FROM pg_catalog.pg_locks l JOIN pg_catalog.pg_class c ON c.oid=l.relation \
             JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace WHERE l.pid=pg_backend_pid() \
             AND l.locktype='relation' AND l.mode='AccessExclusiveLock' AND l.granted AND n.nspname='public' \
             AND c.relkind='r' AND NOT c.relispartition ORDER BY c.relname COLLATE \"C\"")
            .fetch_all(&mut *connection).await.unwrap();
        assert_eq!(
            actual, expected,
            "actual retained public AccessExclusiveLock set must be exact83"
        );
        let share: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM pg_catalog.pg_locks WHERE pid=pg_backend_pid() AND locktype='relation' \
             AND relation='public._sqlx_migrations'::regclass AND mode='ShareLock' AND granted")
            .fetch_one(&mut *connection).await.unwrap();
        assert_eq!(share, 1);
        let (pid, xid): (i32, i64) =
            sqlx::query_as("SELECT pg_backend_pid(),txid_current()::bigint")
                .fetch_one(connection)
                .await
                .unwrap();
        // A sorted retained set is not evidence of transient inner lock order.
        json!({"relation_names":actual, "ledger_share":share, "backend_pid":pid, "transaction_id":xid})
    }

    async fn group_run(connection: &mut PgConnection) -> Duration {
        bounds(connection).await;
        let start = Instant::now();
        sqlx::raw_sql(GROUP_FINALIZER)
            .execute(connection)
            .await
            .unwrap();
        let elapsed = start.elapsed();
        assert!(
            elapsed <= Duration::from_secs(60),
            "reviewed Group fixture finalizer envelope exceeded"
        );
        elapsed
    }

    async fn group_refused(connection: &mut PgConnection, expected: &str) {
        bounds(connection).await;
        let failure = sqlx::raw_sql(GROUP_FINALIZER)
            .execute(connection)
            .await
            .expect_err("exact Group finalizer must refuse this concrete corruption");
        let database = failure
            .as_database_error()
            .expect("actual PostgreSQL refusal, not infrastructure failure");
        assert_eq!(database.code().as_deref(), Some("P0001"));
        assert_eq!(database.message(), expected);
    }

    async fn group_install_replay(
        connection: &mut PgConnection,
        before: &Baseline,
        variant: usize,
        oracle: &Value,
    ) -> Value {
        let variant_name = if variant == 0 { "plain" } else { "observer" };
        assert_eq!(
            group_classification(connection).await,
            (
                "native_group_process.install_required".into(),
                Some(variant_name.into())
            )
        );
        let before83 = group_capture(connection, false).await;
        assert_eq!(before83.sha256, BEFORE83[variant]);
        assert_absent(&before83.snapshot);
        let before_catalog = catalog(connection).await;
        assert!(
            before_catalog == before.catalog,
            "exact frozen closed predecessor catalog differs"
        );
        let install_elapsed = group_run(connection).await;
        assert_eq!(
            group_classification(connection).await,
            (
                "native_group_process.finalized".into(),
                Some(variant_name.into())
            )
        );
        let installed83 = group_capture(connection, true).await;
        assert_eq!(installed83.sha256, INSTALLED83[variant]);
        assert_ne!(before83.sha256, installed83.sha256);
        let installed_catalog = catalog(connection).await;
        actual_group_metadata(
            &before.catalog.1,
            &installed_catalog.1,
            &installed83.snapshot,
            oracle,
        );
        let installed_rows = rows(connection).await;
        unchanged_group_rows(&before.rows, &installed_rows, oracle);
        assert!(applied_ledger(connection).await == before.ledger);
        let locks = group_locks(connection, oracle).await;
        let replay_elapsed = group_run(connection).await;
        assert_eq!(
            group_classification(connection).await,
            (
                "native_group_process.finalized".into(),
                Some(variant_name.into())
            )
        );
        assert!(
            group_capture(connection, true).await == installed83,
            "same-transaction replay changed exact full83 capture"
        );
        assert!(
            catalog(connection).await == installed_catalog,
            "replay changed complete raw catalog"
        );
        assert!(
            rows(connection).await == installed_rows,
            "replay changed any durable row"
        );
        assert!(
            applied_ledger(connection).await == before.ledger,
            "replay changed complete231 ledger"
        );
        assert_eq!(
            group_locks(connection, oracle).await,
            locks,
            "replay changed locks/backend/transaction"
        );
        json!({"variant":variant_name, "before83":before83.sha256, "installed83":installed83.sha256,
            "before_rights":false, "installed_rights":true, "historical18_serialized_flag":false,
            "install_elapsed_ms":install_elapsed.as_millis(), "replay_elapsed_ms":replay_elapsed.as_millis(),
            "retained_locks":locks, "existing_rows_unchanged":true, "new_group_tables":7,
            "new_group_tables_empty":true, "ledger_records":231, "ledger_unchanged":true,
            "closed_catalog_sha256":digest(&before.catalog.0), "installed_catalog_sha256":digest(&installed_catalog.0)})
    }

    async fn group_closed_fixture(pool: &PgPool, before: &Baseline, variant: usize) -> Baseline {
        let mut admin = direct(pool).await;
        let mut tx = begin_protocol(&mut admin, &before.target).await;
        run_finalizer(tx.as_mut()).await;
        retained_locks(tx.as_mut()).await;
        assert!(rows(tx.as_mut()).await == before.rows);
        assert!(applied_ledger(tx.as_mut()).await == before.ledger);
        tx.commit().await.unwrap();
        admin.close().await.unwrap();
        let closed = serving_snapshot(pool, variant).await;
        closed_delta(&before.catalog.1, &closed.catalog.1);
        assert!(closed.rows == before.rows);
        assert!(closed.ledger == before.ledger);
        closed
    }

    async fn group_transition_history(pool: PgPool, variant: usize) {
        let pins = group_pins();
        let oracle = group_oracle();
        let original = predecessor(&pool, variant).await;
        let mut admin = direct(&pool).await;
        let mut inherited = begin_protocol(&mut admin, &original.target).await;
        let inherited_result = AssertUnwindSafe(async {
            // Complete namespace absence on an older unclosed substrate permits
            // fallback, and must not permit Group installation.
            assert_eq!(
                group_classification(inherited.as_mut()).await,
                ("native_group_process.absent".into(), None)
            );
            let prior83 = group_capture(inherited.as_mut(), false).await;
            assert_absent(&prior83.snapshot);
            sqlx::raw_sql("SAVEPOINT generic_absence")
                .execute(inherited.as_mut())
                .await
                .unwrap();
            group_refused(inherited.as_mut(), "native_group_process.profile_mismatch").await;
            sqlx::raw_sql(
                "ROLLBACK TO SAVEPOINT generic_absence; RELEASE SAVEPOINT generic_absence",
            )
            .execute(inherited.as_mut())
            .await
            .unwrap();
            assert!(group_capture(inherited.as_mut(), false).await == prior83);
            assert!(catalog(inherited.as_mut()).await == original.catalog);
            assert!(rows(inherited.as_mut()).await == original.rows);
            assert!(applied_ledger(inherited.as_mut()).await == original.ledger);
            run_finalizer(inherited.as_mut()).await;
            let observed76 = retained_locks(inherited.as_mut()).await;
            let closed = Baseline {
                target: original.target.clone(),
                rows: rows(inherited.as_mut()).await,
                catalog: catalog(inherited.as_mut()).await,
                ledger: applied_ledger(inherited.as_mut()).await,
                denied: added_three_denied_rights(inherited.as_mut()).await,
            };
            closed_delta(&original.catalog.1, &closed.catalog.1);
            let mut packet =
                group_install_replay(inherited.as_mut(), &closed, variant, &oracle).await;
            packet["entry_history"] = json!("existing_org_finalizer_retained76");
            packet["observed_before_group_locks"] = observed76;
            packet["fresh_group_entry_qualified"] = json!(false);
            packet["transient_inner76_or_order_proven"] = json!(false);
            packet
        })
        .catch_unwind()
        .await;
        inherited
            .rollback()
            .await
            .expect("entire inherited closed+Group installation/replay must roll back");
        admin.close().await.unwrap();
        restored(&pool, &original, variant).await;
        let mut inherited_packet = match inherited_result {
            Ok(value) => value,
            Err(panic) => std::panic::resume_unwind(panic),
        };
        inherited_packet["complete_catalog_rows_ledger_rollback_verified"] = json!(true);

        // The second history commits only the accepted historical closed
        // prerequisite, then starts the genuine Group protocol on a new direct
        // connection. begin_protocol proves no prior user relation locks before
        // BEGIN and freezes actual target, ledger SHARE and owner-row FOR UPDATE.
        let closed = group_closed_fixture(&pool, &original, variant).await;
        let mut fresh = direct(&pool).await;
        let mut tx = begin_protocol(&mut fresh, &closed.target).await;
        let fresh_result = AssertUnwindSafe(async {
            let exclusive_before: i64 = sqlx::query_scalar(
                "SELECT count(*) FROM pg_catalog.pg_locks l JOIN pg_catalog.pg_class c ON c.oid=l.relation \
                 JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace WHERE l.pid=pg_backend_pid() \
                 AND l.locktype='relation' AND l.mode='AccessExclusiveLock' AND l.granted AND n.nspname='public'")
                .fetch_one(tx.as_mut()).await.unwrap();
            assert_eq!(exclusive_before, 0, "fresh Group finalizer must acquire its own relation locks");
            let mut packet = group_install_replay(tx.as_mut(), &closed, variant, &oracle).await;
            packet["entry_history"] = json!("fresh_direct_zero_prior_user_locks");
            packet["fresh_group_entry_executed"] = json!(true);
            packet["exclusive_before_group"] = json!(exclusive_before);
            packet["transient_inner76_or_order_proven"] = json!(false);
            packet
        }).catch_unwind().await;
        tx.rollback()
            .await
            .expect("entire fresh Group installation/replay must roll back");
        fresh.close().await.unwrap();
        serving_restored(&pool, variant, &closed).await;
        let mut fresh_packet = match fresh_result {
            Ok(value) => value,
            Err(panic) => std::panic::resume_unwind(panic),
        };
        fresh_packet["complete_catalog_rows_ledger_rollback_verified"] = json!(true);
        let mut output = std::io::stderr().lock();
        writeln!(&mut output, "GROUP_FINALIZER_HISTORY {}", json!({"sources":pins,
            "target":{"database":closed.target.database,"database_oid":closed.target.database_oid,"system_identifier":closed.target.system_identifier},
            "histories":[inherited_packet,fresh_packet],"maintenance_lease":"root_owned_dedicated_disposable_cluster",
            "actual_owner_row_retained_by_caller":true,"historical_prerequisite_owner_effects":true,
            "business_empty":false,"serving_startup_accepted":false,"group_browser_accepted":false,"production_qualified":false}))
            .expect("Group finalizer evidence write failed");
        output
            .flush()
            .expect("Group finalizer evidence flush failed");
    }

    #[sqlx::test(migrations = false)]
    async fn group_plain_finalizer_transitions_replays_and_rolls_back(pool: PgPool) {
        group_transition_history(pool, 0).await;
    }

    #[sqlx::test(migrations = false)]
    async fn group_observer_finalizer_transitions_replays_and_rolls_back(pool: PgPool) {
        group_transition_history(pool, 1).await;
    }

    #[sqlx::test(migrations = false)]
    async fn group_finalizer_refuses_ledger_namespace_and_installed_body_corruption(pool: PgPool) {
        let pins = group_pins();
        let oracle = group_oracle();
        let original = predecessor(&pool, 0).await;
        let closed = group_closed_fixture(&pool, &original, 0).await;
        let mut admin = direct(&pool).await;
        let mut tx = begin_protocol(&mut admin, &closed.target).await;
        let outcome = AssertUnwindSafe(async {
            for corruption in ["ledger_checksum", "reserved_schema"] {
                assert_eq!(group_classification(tx.as_mut()).await,
                    ("native_group_process.install_required".into(), Some("plain".into())));
                let before83 = group_capture(tx.as_mut(), false).await;
                assert_eq!(before83.sha256, BEFORE83[0]);
                sqlx::raw_sql("SAVEPOINT group_fault").execute(tx.as_mut()).await.unwrap();
                let expected = if corruption == "ledger_checksum" {
                    let changed = sqlx::query("UPDATE public._sqlx_migrations SET checksum=decode(repeat('00',48),'hex') WHERE version=231")
                        .execute(tx.as_mut()).await.unwrap();
                    assert_eq!(changed.rows_affected(), 1);
                    let checksum: String = sqlx::query_scalar("SELECT encode(checksum,'hex') FROM public._sqlx_migrations WHERE version=231")
                        .fetch_one(tx.as_mut()).await.unwrap();
                    assert_eq!(checksum, "00".repeat(48));
                    "native_group_process.migration_ledger_mismatch"
                } else {
                    sqlx::raw_sql("CREATE SCHEMA identity_native_group_process_unreviewed")
                        .execute(tx.as_mut()).await.unwrap();
                    let capture = group_capture(tx.as_mut(), false).await;
                    assert_ne!(capture.sha256, BEFORE83[0]);
                    assert_eq!(capture.snapshot["native_group_process_schema_namespace"].as_array().unwrap().len(), 1);
                    assert_eq!(group_classification(tx.as_mut()).await, ("native_group_process.profile_mismatch".into(), None));
                    "native_group_process.profile_mismatch"
                };
                group_refused(tx.as_mut(), expected).await;
                sqlx::raw_sql("ROLLBACK TO SAVEPOINT group_fault; RELEASE SAVEPOINT group_fault")
                    .execute(tx.as_mut()).await.unwrap();
                assert!(group_capture(tx.as_mut(), false).await == before83, "fault rollback changed exact predecessor83");
                assert!(catalog(tx.as_mut()).await == closed.catalog, "fault/refusal rollback changed complete raw catalog");
                assert!(rows(tx.as_mut()).await == closed.rows, "fault/refusal rollback changed any durable row");
                assert!(applied_ledger(tx.as_mut()).await == closed.ledger);
            }
            // Positive installation and replay must follow the refusal controls;
            // a permanently broken finalizer cannot satisfy only negative tests.
            let installed = group_install_replay(tx.as_mut(), &closed, 0, &oracle).await;
            let installed_catalog = catalog(tx.as_mut()).await;
            let installed_rows = rows(tx.as_mut()).await;
            let installed_capture = group_capture(tx.as_mut(), true).await;
            let installed_locks = group_locks(tx.as_mut(), &oracle).await;
            sqlx::raw_sql("SAVEPOINT installed_source_fault").execute(tx.as_mut()).await.unwrap();
            let definition: String = sqlx::query_scalar(
                "SELECT pg_catalog.pg_get_functiondef('public.native_group_process_uuid_v1(bytea,integer)'::regprocedure)")
                .fetch_one(tx.as_mut()).await.unwrap();
            assert_eq!(definition.matches("$function$").count(), 2);
            let (body, suffix) = definition.rsplit_once("$function$").unwrap();
            let changed = format!("{body}\n-- explicit Group body corruption control\n$function${suffix}");
            assert_ne!(changed, definition);
            sqlx::raw_sql(sqlx::AssertSqlSafe(changed)).execute(tx.as_mut()).await.unwrap();
            let corrupted = group_capture(tx.as_mut(), true).await;
            assert_ne!(corrupted.sha256, INSTALLED83[0], "concrete source body change must alter capture");
            assert_eq!(group_classification(tx.as_mut()).await, ("native_group_process.profile_mismatch".into(), None));
            group_refused(tx.as_mut(), "native_group_process.profile_mismatch").await;
            sqlx::raw_sql("ROLLBACK TO SAVEPOINT installed_source_fault; RELEASE SAVEPOINT installed_source_fault")
                .execute(tx.as_mut()).await.unwrap();
            assert!(catalog(tx.as_mut()).await == installed_catalog, "source refusal silently repaired/changed metadata");
            assert!(rows(tx.as_mut()).await == installed_rows);
            assert!(group_capture(tx.as_mut(), true).await == installed_capture);
            assert_eq!(group_locks(tx.as_mut(), &oracle).await, installed_locks);
            assert!(applied_ledger(tx.as_mut()).await == closed.ledger);
            group_run(tx.as_mut()).await;
            assert!(catalog(tx.as_mut()).await == installed_catalog, "post-recovery replay changed metadata");
            assert!(rows(tx.as_mut()).await == installed_rows);
            json!({"sources":pins,"variant":"plain","controls":["ledger_checksum","reserved_schema","installed_body"],
                "positive_install_replay":installed,"savepoint_recovery_verified":true,"ledger_records":231,
                "business_empty":false,"serving_startup_accepted":false,"group_browser_accepted":false,"production_qualified":false})
        }).catch_unwind().await;
        tx.rollback()
            .await
            .expect("all Group source and fault injections must roll back");
        admin.close().await.unwrap();
        serving_restored(&pool, 0, &closed).await;
        let mut packet = match outcome {
            Ok(value) => value,
            Err(panic) => std::panic::resume_unwind(panic),
        };
        packet["complete_catalog_rows_ledger_rollback_verified"] = json!(true);
        let mut output = std::io::stderr().lock();
        writeln!(&mut output, "GROUP_FINALIZER_REFUSAL_HISTORY {packet}")
            .expect("refusal evidence write failed");
        output.flush().expect("refusal evidence flush failed");
    }
    include!("native_group_navigation_correction_tests.rs");
    include!("native_group_navigation_finalizer_tests.rs");

    #[cfg(feature = "test-browser")]
    include!("native_group_process_browser_fixture.rs");
}
