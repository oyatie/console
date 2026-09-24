// External prerequisite-bound proposal. Include inside native_policy_startup_tests.
// Requires independently reviewed guarded FINALIZER artifact, not declared SOURCE.
// Missing artifact is a prerequisite failure, never semantic RED or fake installer.
mod native_people_directory_finalizer_tests {
    use super::*;
    use sqlx::{Postgres, Transaction};
    const FINALIZER: &str =
        include_str!("../../../../ops/postgres-finalize-native-people-directory.sql");
    const CLASSIFIER: &str =
        include_str!("../../../../ops/postgres-native-people-directory-custody-state.sql");
    const CAPTURE: &str =
        include_str!("../../../../ops/postgres-capture-native-people-directory-custody.sql");
    const OBSERVER: &str = include_str!("../../../../ops/postgres-install-durability-observer.sql");
    const STAGED: [&str; 2] = [
        "eaff3623d29f22768cb8056f27dd97cb9827a71194dedc59b289744fee7fdb0f",
        "d5cad51f05a3cd9bf8abd6a4a9a5f154ff6e82df512966d333e75bba52f22761",
    ];
    const ACTIVE: [&str; 2] = [
        "e9891784422768abcb07f731adb1295c17b40c026236c1c4ea5b9993f6ddba9b",
        "bf87ef1475ec983c4e1bd286337687ead135b76fe70e28f79fe8cd430a1c95bc",
    ];
    fn reviewed_capture() {
        assert_eq!(
            hex::encode(Sha256::digest(CLASSIFIER.as_bytes())),
            "20c96bc2a9264d5ed4b86cb948cbe0574450243509263469c45af955d5aff3dd"
        );
        assert_eq!(
            hex::encode(Sha256::digest(CAPTURE.as_bytes())),
            "bc8a1f87f57cd676ca1a3deae12263b1cca4a290c71a12e9a1749d189e090ca7"
        );
        assert_eq!(
            hex::encode(Sha256::digest(OBSERVER.as_bytes())),
            "ffe7038b43de0207d6ae3e3ce0487498edc4d061c21e4249abacc91abaacdbbc"
        );
        // Finalizer source pin is bound by the exact independent implementation
        // receipt before mount. Do not infer it from the target database.
    }
    // Fixture prerequisite only: the real packaged operator separately owns
    // same-transaction ledger locking and actual TLS negative controls.
    static CURRENT_MIGRATOR: sqlx::migrate::Migrator =
        sqlx::migrate!("../crates/platform/db/migrations");
    const APPLIED_LEDGER: &str = include_str!("../../../../ops/account-custody-migrations.sha384");
    async fn assert_actual_applied_ledger(pool: &PgPool) {
        assert_eq!(
            hex::encode(Sha256::digest(APPLIED_LEDGER.as_bytes())),
            "25e02488cdaf864f6d15ee21d62df98eb263bb82de1e2a283470ca659d160325",
            "reviewed packaged migration ledger drift"
        );
        let expected: Vec<(i64, bool, Vec<u8>)> = APPLIED_LEDGER
            .lines()
            .enumerate()
            .map(|(index, line)| {
                let (version, checksum) = line.split_once('\t').unwrap();
                let version = version.parse::<i64>().unwrap();
                assert_eq!(version, index as i64 + 1);
                let checksum = hex::decode(checksum).unwrap();
                assert_eq!(checksum.len(), 48);
                (version, true, checksum)
            })
            .collect();
        assert_eq!(expected.len(), 230);
        assert_eq!(CURRENT_MIGRATOR.iter().count(), expected.len());
        for (migration, wanted) in CURRENT_MIGRATOR.iter().zip(&expected) {
            assert_eq!(migration.version, wanted.0);
            assert_eq!(migration.checksum.as_ref(), wanted.2.as_slice());
            assert_eq!(
                sha2::Sha384::digest(migration.sql.as_str().as_bytes()).as_slice(),
                wanted.2.as_slice()
            );
        }
        let mut tx = pool.begin().await.unwrap();
        sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
            .execute(tx.as_mut())
            .await
            .unwrap();
        let correct_owner: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='public' AND c.relname='_sqlx_migrations' AND c.relkind='r' AND NOT c.relispartition AND pg_catalog.pg_get_userbyid(c.relowner)='console_app')")
            .fetch_one(tx.as_mut()).await.unwrap();
        assert!(
            correct_owner,
            "actual migration ledger owner/shape prerequisite"
        );
        let actual: Vec<(i64, bool, Vec<u8>)> = sqlx::query_as(
            "SELECT version,success,checksum FROM public._sqlx_migrations ORDER BY version",
        )
        .fetch_all(tx.as_mut())
        .await
        .unwrap();
        assert_eq!(
            actual, expected,
            "actual full1..230 migration ledger prerequisite"
        );
        tx.rollback().await.unwrap();
    }
    async fn captured(tx: &mut Transaction<'_, Postgres>) -> (Value, String, Option<bool>) {
        sqlx::raw_sql(CLASSIFIER_SESSION)
            .execute(tx.as_mut())
            .await
            .unwrap();
        sqlx::query_as(sqlx::AssertSqlSafe(CAPTURE))
            .fetch_one(tx.as_mut())
            .await
            .unwrap()
    }
    async fn metadata(pool: &PgPool) -> (Value, String, Option<bool>) {
        let mut tx = pool.begin().await.unwrap();
        sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
            .execute(tx.as_mut())
            .await
            .unwrap();
        let snapshot = captured(&mut tx).await;
        tx.rollback().await.unwrap();
        snapshot
    }
    async fn install(tx: &mut Transaction<'_, Postgres>) -> Result<(), sqlx::Error> {
        // Caller chooses isolation before any query so isolation-refusal tests
        // exercise the actual owner rather than a test helper resetting it.
        sqlx::raw_sql("SET LOCAL search_path=pg_catalog,pg_temp; SET LOCAL lock_timeout='1s'; SET LOCAL statement_timeout='120s'").execute(tx.as_mut()).await?;
        sqlx::raw_sql(FINALIZER).execute(tx.as_mut()).await?;
        sqlx::raw_sql("SET CONSTRAINTS ALL IMMEDIATE")
            .execute(tx.as_mut())
            .await?;
        Ok(())
    }
    async fn predecessor_rows(pool: &PgPool) -> BTreeMap<String, String> {
        let mut rows = all_rows(pool).await;
        // JSONB subtraction is over exact newly added fields only; PostgreSQL
        // serialization preserves all old numeric/timestamp/value bytes.
        for (table, fields) in [
            ("person_revisions", "ARRAY['actor_kind','actor_account_id']"),
            (
                "employee_person_bindings",
                "ARRAY['actor_kind','actor_account_id']",
            ),
            (
                "ont_action_command_receipts",
                "ARRAY['actor_kind','actor_account_id']",
            ),
            ("employees", "ARRAY['source_kind','native_command_id']"),
        ] {
            assert!(rows.contains_key(table));
            let query = format!(
                "SELECT coalesce(jsonb_agg(r.j ORDER BY r.j::text COLLATE \"C\"),'[]'::jsonb)::text FROM (SELECT to_jsonb(t)-{fields} AS j FROM public.{table} t) r"
            );
            rows.insert(
                table.to_owned(),
                sqlx::query_scalar(sqlx::AssertSqlSafe(query))
                    .fetch_one(pool)
                    .await
                    .unwrap(),
            );
        }
        rows
    }
    async fn expanded_legacy_shape(pool: &PgPool) {
        let correct:bool=sqlx::query_scalar("SELECT NOT EXISTS(SELECT 1 FROM public.person_revisions WHERE actor_kind IS DISTINCT FROM 'USER' OR actor_account_id IS NOT NULL OR actor_id IS NULL) AND NOT EXISTS(SELECT 1 FROM public.employee_person_bindings WHERE actor_kind IS DISTINCT FROM 'USER' OR actor_account_id IS NOT NULL OR actor_id IS NULL) AND NOT EXISTS(SELECT 1 FROM public.ont_action_command_receipts WHERE actor_kind IS DISTINCT FROM 'USER' OR actor_account_id IS NOT NULL OR actor_id IS NULL) AND NOT EXISTS(SELECT 1 FROM public.employees WHERE source_kind IS DISTINCT FROM 'LEGACY' OR native_command_id IS NOT NULL OR source_filename IS NULL OR source_sheet IS NULL OR source_row IS NULL) AND NOT EXISTS(SELECT 1 FROM public.native_people_inputs_v1) AND NOT EXISTS(SELECT 1 FROM public.native_people_terminals_v1)")
            .fetch_one(pool).await.unwrap();
        assert!(
            correct,
            "expansion fabricated or misattributed business history"
        );
    }
    async fn assert_all_relation_locks(tx: &mut Transaction<'_, Postgres>, snapshot: &Value) {
        let tables = snapshot["tables"].as_array().unwrap();
        assert_eq!(tables.len(), 73);
        let expected: BTreeSet<String> = tables
            .iter()
            .map(|t| t["name"].as_str().unwrap().to_owned())
            .collect();
        assert_eq!(expected.len(), 73);
        let actual:Vec<String>=sqlx::query_scalar("SELECT DISTINCT c.relname::text FROM pg_catalog.pg_locks l JOIN pg_catalog.pg_class c ON c.oid=l.relation JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace WHERE l.pid=pg_backend_pid() AND l.granted AND l.mode='AccessExclusiveLock' AND n.nspname='public' AND c.relkind IN ('r','p')")
            .fetch_all(tx.as_mut()).await.unwrap();
        let actual: BTreeSet<_> = actual.into_iter().collect();
        assert!(
            expected.is_subset(&actual),
            "finalizer omitted required owning relation locks"
        );
    }
    pub(super) async fn configured_native_directory_fixture(
        pool: &PgPool,
    ) -> (Fixture, SigningKey, AppState) {
        reviewed_capture();
        let (mut app, key, held) =
            native_people_codec2_install_probe::configured_successor_fixture(pool).await;
        let config = account_browser_config(pool, app._artifacts.root.clone(), &key);
        let result = AssertUnwindSafe(async {
            assert_actual_applied_ledger(pool).await;
            assert_eq!(metadata(pool).await.1, STAGED[0]);
            assert_eq!(ready_status(&held).await, StatusCode::OK);
            let before = all_rows(pool).await;
            let mut tx = pool.begin().await.unwrap();
            install(&mut tx).await.unwrap();
            let active = captured(&mut tx).await;
            assert_eq!(active.1, ACTIVE[0]);
            assert_eq!(active.2, Some(true));
            assert_all_relation_locks(&mut tx, &active.0).await;
            tx.commit().await.unwrap();
            assert!(predecessor_rows(pool).await == before);
            expanded_legacy_shape(pool).await;
        })
        .catch_unwind()
        .await;
        close_states(&[held], result).await;
        let fresh = AppState::from_config(config)
            .await
            .expect("actual freshly verified directory startup");
        let positive = AssertUnwindSafe(async {
            assert_eq!(ready_status(&fresh).await, StatusCode::OK);
        })
        .catch_unwind()
        .await;
        if positive.is_err() {
            close_states(&[fresh], positive).await;
            unreachable!();
        }
        app.service = build_router(fresh.clone());
        (app, key, fresh)
    }
    #[sqlx::test(migrations = false)]
    async fn guarded_directory_activation_rolls_back_then_restarts_and_replays_without_effects(
        pool: PgPool,
    ) {
        reviewed_capture();
        let (app, key, held) =
            native_people_codec2_install_probe::configured_successor_fixture(&pool).await;
        let config = account_browser_config(&pool, app._artifacts.root.clone(), &key);
        let mut fresh_cleanup = None;
        let outcome = AssertUnwindSafe(async {
            assert_actual_applied_ledger(&pool).await;
            let old = metadata(&pool).await;
            assert_eq!(old.1, STAGED[0]);
            assert_eq!(old.2, Some(true));
            let before = all_rows(&pool).await;
            assert_eq!(ready_status(&held).await, StatusCode::OK);
            let mut tx = pool.begin().await.unwrap();
            install(&mut tx).await.unwrap();
            let active = captured(&mut tx).await;
            assert_eq!(active.1, ACTIVE[0]);
            assert_eq!(active.2, Some(true));
            assert_all_relation_locks(&mut tx, &active.0).await;
            let failure = sqlx::query("SELECT 1/0")
                .execute(tx.as_mut())
                .await
                .unwrap_err();
            assert_eq!(
                failure.as_database_error().unwrap().code().as_deref(),
                Some("22012")
            );
            tx.rollback().await.unwrap();
            assert_eq!(metadata(&pool).await, old);
            assert!(all_rows(&pool).await == before);
            assert_eq!(ready_status(&held).await, StatusCode::OK);
            let mut tx = pool.begin().await.unwrap();
            install(&mut tx).await.unwrap();
            tx.commit().await.unwrap();
            assert!(predecessor_rows(&pool).await == before);
            expanded_legacy_shape(&pool).await;
            let installed = metadata(&pool).await;
            assert_eq!(installed.1, ACTIVE[0]);
            assert_eq!(installed.2, Some(true));
            assert_eq!(
                ready_status(&held).await,
                StatusCode::SERVICE_UNAVAILABLE,
                "held predecessor cannot promote itself"
            );
            let fresh = AppState::from_config(config.clone()).await.unwrap();
            fresh_cleanup = Some(fresh.clone());
            assert_eq!(ready_status(&fresh).await, StatusCode::OK);
            let committed = all_rows(&pool).await;
            let mut tx = pool.begin().await.unwrap();
            install(&mut tx).await.unwrap();
            tx.commit().await.unwrap();
            assert_eq!(metadata(&pool).await, installed);
            assert!(all_rows(&pool).await == committed);
            assert_eq!(ready_status(&held).await, StatusCode::SERVICE_UNAVAILABLE);
            assert_eq!(ready_status(&fresh).await, StatusCode::OK);
            assert_eq!(metadata(&pool).await, installed);
            assert!(all_rows(&pool).await == committed);
        })
        .catch_unwind()
        .await;
        if let Some(fresh) = fresh_cleanup {
            close_states(&[fresh], Ok(())).await;
        }
        close_states(&[held], outcome).await;
    }
    #[sqlx::test(migrations = false)]
    async fn guarded_directory_observer_pair_activates_and_repeats_atomically(pool: PgPool) {
        reviewed_capture();
        native_people_codec2_install_probe::prepare_successor_ready_database(&pool).await;
        assert_actual_applied_ledger(&pool).await;
        let original = metadata(&pool).await;
        assert_eq!(original.1, STAGED[0]);
        let rows = all_rows(&pool).await;
        let mut tx = pool.begin().await.unwrap();
        let outcome=AssertUnwindSafe(async {
            let absent:bool=sqlx::query_scalar("SELECT NOT EXISTS(SELECT 1 FROM pg_catalog.pg_roles WHERE rolname='console_durability_observer')").fetch_one(tx.as_mut()).await.unwrap();assert!(absent,"isolated observer transaction requires absent role");
            sqlx::raw_sql(OBSERVER).execute(tx.as_mut()).await.unwrap();
            assert_eq!(captured(&mut tx).await.1,STAGED[1]);
            install(&mut tx).await.unwrap();let first=captured(&mut tx).await;assert_eq!(first.1,ACTIVE[1]);assert_eq!(first.2,Some(true));
            assert_all_relation_locks(&mut tx,&first.0).await;install(&mut tx).await.unwrap();assert_eq!(captured(&mut tx).await,first);
        }).catch_unwind().await;
        tx.rollback().await.unwrap();
        assert_eq!(metadata(&pool).await, original);
        assert!(all_rows(&pool).await == rows);
        let absent:bool=sqlx::query_scalar("SELECT NOT EXISTS(SELECT 1 FROM pg_catalog.pg_roles WHERE rolname='console_durability_observer')").fetch_one(&pool).await.unwrap();
        assert!(absent);
        if let Err(panic) = outcome {
            std::panic::resume_unwind(panic);
        }
    }

    fn exact_error(error: &sqlx::Error, message: &str) {
        let database = error
            .as_database_error()
            .expect("owner refusal must be a database result");
        assert_eq!(database.code().as_deref(), Some("P0001"));
        assert_eq!(database.message(), message);
    }
    #[sqlx::test(migrations = false)]
    async fn guarded_directory_finalizer_refuses_runtime_identity_isolation_and_partial_shapes(
        pool: PgPool,
    ) {
        reviewed_capture();
        native_people_codec2_install_probe::prepare_successor_ready_database(&pool).await;
        assert_actual_applied_ledger(&pool).await;
        let baseline = metadata(&pool).await;
        assert_eq!(baseline.1, STAGED[0]);
        let rows = all_rows(&pool).await;
        let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
        let outcome=AssertUnwindSafe(async {
            let mut tx=runtime.begin().await.unwrap();
            let error=install(&mut tx).await.unwrap_err();exact_error(&error,"native_people_directory.operator_identity_mismatch");tx.rollback().await.unwrap();
            assert_eq!(metadata(&pool).await,baseline);assert!(all_rows(&pool).await==rows);
            let mut tx=pool.begin().await.unwrap();
            sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ").execute(tx.as_mut()).await.unwrap();
            let error=install(&mut tx).await.unwrap_err();exact_error(&error,"native_people_directory.unsupported_isolation");tx.rollback().await.unwrap();
            assert_eq!(metadata(&pool).await,baseline);assert!(all_rows(&pool).await==rows);
            for fault in [
                "ALTER TABLE public.native_people_inputs_v1 DROP CONSTRAINT native_people_inputs_staged_closed_v1",
                "CREATE FUNCTION public.native_people_unreviewed_finalizer_probe_v1() RETURNS integer LANGUAGE sql IMMUTABLE AS 'SELECT 1'",
                "CREATE POLICY native_people_owner_v1 ON public.employees TO console_account_owner USING(true)",
            ] {
                let mut tx=pool.begin().await.unwrap();
                sqlx::raw_sql(sqlx::AssertSqlSafe(fault)).execute(tx.as_mut()).await.unwrap();
                let checked=AssertUnwindSafe(async {
                    assert_ne!(captured(&mut tx).await,baseline,"fault missing from actual complete capture");
                    let error=install(&mut tx).await.unwrap_err();exact_error(&error,"native_people_directory.profile_mismatch");
                }).catch_unwind().await;
                tx.rollback().await.unwrap();assert_eq!(metadata(&pool).await,baseline);assert!(all_rows(&pool).await==rows);
                if let Err(panic)=checked {std::panic::resume_unwind(panic);}
            }
            // Positive control after all negative rollbacks exercises the same
            // actual owner, not merely a still-matching predecessor classifier.
            let mut tx=pool.begin().await.unwrap();install(&mut tx).await.unwrap();assert_eq!(captured(&mut tx).await.1,ACTIVE[0]);tx.rollback().await.unwrap();
            assert_eq!(metadata(&pool).await,baseline);assert!(all_rows(&pool).await==rows);
        }).catch_unwind().await;
        runtime.close().await;
        if let Err(panic) = outcome {
            std::panic::resume_unwind(panic);
        }
    }
    #[sqlx::test(migrations = false)]
    async fn guarded_directory_finalizer_rejects_active_guard_and_acl_drift_without_repair(
        pool: PgPool,
    ) {
        reviewed_capture();
        native_people_codec2_install_probe::prepare_successor_ready_database(&pool).await;
        assert_actual_applied_ledger(&pool).await;
        let mut tx = pool.begin().await.unwrap();
        install(&mut tx).await.unwrap();
        tx.commit().await.unwrap();
        let baseline = metadata(&pool).await;
        assert_eq!(baseline.1, ACTIVE[0]);
        let rows = all_rows(&pool).await;
        for fault in [
            "ALTER TABLE public.native_people_inputs_v1 DISABLE TRIGGER native_people_input_closure_v1",
            "GRANT SELECT(xmin) ON public.native_people_inputs_v1 TO console_rt",
            "CREATE FUNCTION public.native_people_unreviewed_finalizer_probe_v1() RETURNS integer LANGUAGE sql IMMUTABLE AS 'SELECT 1'",
        ] {
            let mut tx = pool.begin().await.unwrap();
            sqlx::raw_sql(sqlx::AssertSqlSafe(fault))
                .execute(tx.as_mut())
                .await
                .unwrap();
            let checked = AssertUnwindSafe(async {
                assert_ne!(
                    captured(&mut tx).await,
                    baseline,
                    "fault missing from actual complete capture"
                );
                let error = install(&mut tx).await.unwrap_err();
                exact_error(&error, "native_people_directory.profile_mismatch");
            })
            .catch_unwind()
            .await;
            tx.rollback().await.unwrap();
            assert_eq!(metadata(&pool).await, baseline);
            assert!(all_rows(&pool).await == rows);
            if let Err(panic) = checked {
                std::panic::resume_unwind(panic);
            }
        }
        let mut tx = pool.begin().await.unwrap();
        install(&mut tx).await.unwrap();
        tx.commit().await.unwrap();
        assert_eq!(metadata(&pool).await, baseline);
        assert!(all_rows(&pool).await == rows);
    }

    async fn legacy_person_execute(
        runtime: &PgPool,
        command: &console_ontology_canonical_adapter_postgres::person::PersonCommand,
    ) -> Result<
        console_ontology_canonical_domain::CommandReceipt,
        console_ontology_canonical_adapter_postgres::person::PersonError,
    > {
        let mut tx = runtime.begin().await?;
        sqlx::query("SELECT set_config('app.current_org',$1,true)")
            .bind(command.org_id.as_uuid().to_string())
            .execute(tx.as_mut())
            .await?;
        let receipt =
            console_ontology_canonical_adapter_postgres::person::write_in_tx(&mut tx, command)
                .await?;
        tx.commit().await?;
        Ok(receipt)
    }
    #[sqlx::test(migrations = false)]
    async fn populated_legacy_person_history_replays_after_actual_guarded_directory_expansion(
        pool: PgPool,
    ) {
        use console_ontology_canonical_adapter_postgres::person::{
            PersonCommand, PersonError, PersonQuery,
        };
        use console_ontology_canonical_domain::CommandId;
        reviewed_capture();
        prepare_http_database_staging(&pool).await;
        // Explicit isolated historical fixture, using the existing canonical
        // owner's seed contract BEFORE Account/Company finalizers. Never wired
        // to a browser/runtime environment or native Account actor substitute.
        let org_uuid = Uuid::new_v4();
        let actor = console_platform_test_support::seed_org_and_super_admin(
            &pool,
            org_uuid,
            "legacy-directory-upgrade",
        )
        .await;
        let employee:Uuid=sqlx::query_scalar("INSERT INTO public.employees(org_id,company,name,source_filename,source_sheet,source_row,source_key) VALUES($1,'Historical fixture','김이력','historical-fixture.xlsx','People',1,'legacy-directory-upgrade') RETURNING id")
            .bind(org_uuid).fetch_one(&pool).await.unwrap();
        let command = PersonCommand {
            org_id: OrgId::from_uuid(org_uuid),
            command_id: CommandId::from_uuid(Uuid::new_v4()),
            actor_id: actor,
            query: PersonQuery::Create {
                employee_id: Some(employee),
                attributes: json!({"legal_name":"김이력"}),
            },
            action_key: "revise".to_owned(),
            object_type_id: Uuid::nil(),
        };
        let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
        let outcome=AssertUnwindSafe(async {
            let receipt=legacy_person_execute(&runtime,&command).await.unwrap();
            let populated=all_rows(&pool).await;
            for table in ["employees","persons","person_revisions","employee_person_bindings","ont_action_command_receipts"] {
                assert_ne!(populated[table],"[]","empty fixture cannot prove populated expansion: {table}");
            }
            native_people_codec2_install_probe::prepare_successor_ready_database(&pool).await;
        assert_actual_applied_ledger(&pool).await;
            let before=all_rows(&pool).await;
            for table in ["employees","persons","person_revisions","employee_person_bindings","ont_action_command_receipts"] {
                assert!(before[table]==populated[table],"bootstrap changed acknowledged canonical history");
            }
            assert_eq!(legacy_person_execute(&runtime,&command).await.unwrap(),receipt);
            assert!(all_rows(&pool).await==before,"predecessor replay changed history");
            let original=metadata(&pool).await;assert_eq!(original.1,STAGED[0]);
            let mut tx=pool.begin().await.unwrap();install(&mut tx).await.unwrap();
            assert_eq!(captured(&mut tx).await.1,ACTIVE[0]);
            let failure=sqlx::query("SELECT 1/0").execute(tx.as_mut()).await.unwrap_err();assert_eq!(failure.as_database_error().unwrap().code().as_deref(),Some("22012"));
            tx.rollback().await.unwrap();assert!(all_rows(&pool).await==before);assert_eq!(metadata(&pool).await,original);
            let mut tx=pool.begin().await.unwrap();install(&mut tx).await.unwrap();tx.commit().await.unwrap();
            assert!(predecessor_rows(&pool).await==before);expanded_legacy_shape(&pool).await;
            let expanded=all_rows(&pool).await;assert_eq!(metadata(&pool).await.1,ACTIVE[0]);
            assert_eq!(legacy_person_execute(&runtime,&command).await.unwrap(),receipt);
            assert!(all_rows(&pool).await==expanded,"post-expansion replay resealed or changed history");
            let mut changed=command.clone();changed.action_key="different_accepted_wrapper".to_owned();
            assert!(matches!(legacy_person_execute(&runtime,&changed).await,Err(PersonError::DigestConflict(id)) if id==*command.command_id.as_uuid()));
            assert!(all_rows(&pool).await==expanded,"conflicting replay changed history");
            let mut fresh=command.clone();fresh.command_id=CommandId::from_uuid(Uuid::new_v4());
            fresh.query=PersonQuery::Create {employee_id:None,attributes:json!({"legal_name":"박계속"})};
            legacy_person_execute(&runtime,&fresh).await.unwrap();
            let after_new=all_rows(&pool).await;
            for (table,old) in &expanded {
                let added=added_rows(old,&after_new[table]).expect("fresh legacy write changed old row bytes");
                let expected=usize::from(matches!(table.as_str(),"persons"|"person_revisions"|"ont_action_command_receipts"));
                assert_eq!(added.len(),expected,"unexpected legacy effect table {table}");
            }
            let mut tx=pool.begin().await.unwrap();install(&mut tx).await.unwrap();tx.commit().await.unwrap();
            assert!(all_rows(&pool).await==after_new,"finalizer replay changed populated history");assert_eq!(metadata(&pool).await.1,ACTIVE[0]);
        }).catch_unwind().await;
        runtime.close().await;
        if let Err(panic) = outcome {
            std::panic::resume_unwind(panic);
        }
    }

    async fn unrelated_policies(pool: &PgPool) -> String {
        sqlx::query_scalar("SELECT coalesce(jsonb_agg(jsonb_build_array(polname,polcmd,polpermissive,polroles::text,pg_get_expr(polqual,polrelid),pg_get_expr(polwithcheck,polrelid)) ORDER BY polname),'[]'::jsonb)::text FROM pg_catalog.pg_policy WHERE polrelid='public.leave_requests'::regclass")
            .fetch_one(pool).await.unwrap()
    }
    #[sqlx::test(migrations = false)]
    async fn guarded_directory_finalizer_rejects_native_policy_outside73_even_when_fullhash_matches(
        pool: PgPool,
    ) {
        let (app, key, state) = configured_native_directory_fixture(&pool).await;
        let config = account_browser_config(&pool, app._artifacts.root.clone(), &key);
        let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
        let outcome=AssertUnwindSafe(async {
            let original=metadata(&pool).await;assert_eq!(original.1,ACTIVE[0]);
            assert!(!original.0["tables"].as_array().unwrap().iter().any(|t|t["name"]=="leave_requests"),"corruption target must lie outside actual73capture");
            assert_eq!(classified(&runtime,CLASSIFIER).await,"native_people_directory.finalized");assert_eq!(ready_status(&state).await,StatusCode::OK);
            let policies=unrelated_policies(&pool).await;let rows=all_rows(&pool).await;
            sqlx::raw_sql("CREATE POLICY native_people_owner_v1 ON public.leave_requests TO console_account_owner USING(true)").execute(&pool).await.unwrap();
            let checked=AssertUnwindSafe(async {
                let witness:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_policy WHERE polrelid='public.leave_requests'::regclass AND polname='native_people_owner_v1')").fetch_one(&pool).await.unwrap();assert!(witness);
                let hostile=unrelated_policies(&pool).await;assert_ne!(hostile,policies,"independent target metadata witness missing");
                assert_eq!(metadata(&pool).await,original,"this case must expose an out-of-capture marker, not a changed73hash");
                assert_eq!(classified(&runtime,CLASSIFIER).await,"native_people_directory.profile_mismatch");
                assert_eq!(ready_status(&state).await,StatusCode::SERVICE_UNAVAILABLE);
                let mut tx=pool.begin().await.unwrap();let error=install(&mut tx).await.unwrap_err();exact_error(&error,"native_people_directory.profile_mismatch");tx.rollback().await.unwrap();
                assert_eq!(unrelated_policies(&pool).await,hostile,"finalizer repaired or removed unexplained marker");assert!(all_rows(&pool).await==rows);
                startup_refused(config.clone(),"native_people_directory.profile_mismatch").await;
            }).catch_unwind().await;
            sqlx::raw_sql("DROP POLICY native_people_owner_v1 ON public.leave_requests").execute(&pool).await.unwrap();
            assert_eq!(unrelated_policies(&pool).await,policies);assert_eq!(metadata(&pool).await,original);assert!(all_rows(&pool).await==rows);
            assert_eq!(classified(&runtime,CLASSIFIER).await,"native_people_directory.finalized");assert_eq!(ready_status(&state).await,StatusCode::OK);
            let mut tx=pool.begin().await.unwrap();install(&mut tx).await.unwrap();tx.commit().await.unwrap();
            let fresh=AppState::from_config(config.clone()).await.unwrap();
            let positive=AssertUnwindSafe(async {assert_eq!(ready_status(&fresh).await,StatusCode::OK);}).catch_unwind().await;close_states(&[fresh],positive).await;
            assert_eq!(unrelated_policies(&pool).await,policies);assert_eq!(metadata(&pool).await,original);assert!(all_rows(&pool).await==rows);
            if let Err(panic)=checked {std::panic::resume_unwind(panic);}
        }).catch_unwind().await;
        runtime.close().await;
        close_states(&[state], outcome).await;
    }
}
