// Tests-first child of the retained Manager capture module. Actual generated
// source is prerequisite; these tests do not implement custody or bless hashes.
mod native_company_information_manager_serving_custody_tests {
    use super::*;
    use std::time::Duration;

    const FINALIZER: &str = include_str!(
        "../../../../ops/postgres-finalize-company-information-manager-current-policy-v1.sql"
    );
    const STATE: &str = include_str!(
        "../../../../ops/postgres-company-information-manager-current-policy-v1-custody-state.sql"
    );
    const APP_STATE: &str =
        include_str!("../../src/company_information_manager_current_policy_v1_custody_state.sql");
    const FINALIZED: &str = "company_information_manager_current_policy_v1.finalized";
    const INSTALL_REQUIRED: &str = "company_information_manager_current_policy_v1.install_required";
    const ABSENT: &str = "company_information_manager_current_policy_v1.absent";
    const MISMATCH: &str = "company_information_manager_current_policy_v1.profile_mismatch";
    // Proposed finite observations from the independent actual V7 capture;
    // interpretation is limited to the original PolicyV1 composition.
    const EMPTY: [&str; 2] = [
        "20cac016d6cab101839b5689ecae0372dfb834d40184e445cd9ba73eabb85964",
        "9c528d7d3bc708611b49cca19a4eab84c85b36da67959910a6b30d6b79dfd6f4",
    ];
    const SUCCESSOR: [&str; 2] = [
        "c68aa085e01610c25db30ddc9c10ef5c49a73199173f17471dbbbc0e324c10ed",
        "eeef509f4e443a6ead4b8be28a5591f32b5ba33a097d2b4389b0482bfe806812",
    ];
    const LOCKS: [&str; 61] = [
        "account_context_candidates",
        "account_security",
        "account_security_events",
        "account_terms_acceptances",
        "account_terms_head",
        "account_terms_release_receipts",
        "accounts",
        "audit_events",
        "auth_bootstrap_credentials",
        "auth_device_login_handoffs",
        "auth_refresh_token_families",
        "auth_refresh_tokens",
        "auth_webauthn_ceremonies",
        "auth_webauthn_ceremony_bindings",
        "auth_webauthn_credentials",
        "cedar_policy_catalog_entries",
        "company_actors",
        "company_authority_heads",
        "company_enrollment_effect_bindings",
        "company_enrollment_receipts",
        "company_enrollment_request_events",
        "company_enrollment_requests",
        "deployment_operator_head",
        "deployment_operator_receipts",
        "group_authority_heads",
        "group_membership_revisions",
        "group_memberships",
        "group_role_grants",
        "groups",
        "native_company_action_refs",
        "native_company_catalog_installs",
        "native_company_object_refs",
        "native_company_policy_inputs_v1",
        "native_company_policy_receipts_v1",
        "native_company_property_refs",
        "ont_action_types",
        "ont_analytics",
        "ont_builtin_catalog_allowlist",
        "ont_builtin_catalog_installs",
        "ont_link_types",
        "ont_object_policies",
        "ont_object_type_key_revisions",
        "ont_object_types",
        "ont_property_defs",
        "organizations",
        "platform_force_removal_effect_bindings",
        "platform_force_removal_receipts",
        "platform_legacy_catalog_effect_bindings",
        "platform_legacy_membership_effect_bindings",
        "platform_legacy_topology_effect_bindings",
        "platform_legacy_topology_receipts",
        "platform_legacy_user_birth_witnesses",
        "policy_assignment_revisions",
        "policy_capability_clause_fields",
        "policy_capability_clauses",
        "policy_role_conditions",
        "policy_role_permissions",
        "policy_role_revisions",
        "policy_roles",
        "user_role_assignments",
        "users",
    ];

    fn reviewed_sources() {
        manager_source_pins(); // Existing V4/capture/PolicyV1 bytes remain pinned.
        assert_eq!(STATE, APP_STATE, "ops/app serving classifier bytes differ");
        for (source, expected) in [
            (
                FINALIZER,
                "4916eaf30459b7d28f17b4fbb4793f2159b82e5f9bf5b8102ec6b0abde9fc30d",
            ),
            (
                STATE,
                "2ad10750d99ded7ff02acd5ddb10214eefa4063d7d0a14c2ca83e1db9e263853",
            ),
            (
                APP_STATE,
                "2ad10750d99ded7ff02acd5ddb10214eefa4063d7d0a14c2ca83e1db9e263853",
            ),
        ] {
            assert_eq!(
                digest(source),
                expected,
                "STOP: accepted source postimage drift"
            );
        }
        let receipt: Value = pinned_external_receipt(
            "CONSOLE_MANAGER_SERVING_SOURCE_RECEIPT",
            "fd4c4f9e0c4e3f7aff02ee0406562a2662c61dccfc7cb93ed5bc14f1394ae8c2",
        );
        assert_eq!(receipt["base"], "8ba2357ad624d025c87e4c6a1ea4a47bf3caa65a");
        assert_eq!(
            receipt["source_sha256"],
            "408d35ef0a607e31d5e543fe28c2632589a88b5bb63978cf5e41842d44a14421"
        );
        assert_eq!(
            receipt["patch_sha256"],
            "201bdf7efb7c96a8c68cb3225128bfe13b5035090f46c15aabfc30fe2a1fa282"
        );
        assert_eq!(receipt["runtime_execution"], false);
        assert_eq!(receipt["installed_profile"], false);
        assert_eq!(receipt["generated_canonical_writes"], false);
        let outputs = receipt["outputs"].as_array().unwrap();
        assert_eq!(outputs.len(), 3);
        for (entry, name, source) in [
            (
                &outputs[0],
                "ops/postgres-finalize-company-information-manager-current-policy-v1.sql",
                FINALIZER,
            ),
            (
                &outputs[1],
                "ops/postgres-company-information-manager-current-policy-v1-custody-state.sql",
                STATE,
            ),
            (
                &outputs[2],
                "backend/app/src/company_information_manager_current_policy_v1_custody_state.sql",
                APP_STATE,
            ),
        ] {
            assert_eq!(entry["path"], name);
            assert_eq!(entry["sha256"], digest(source));
            assert_eq!(entry["bytes"].as_u64(), Some(source.len() as u64));
        }
        let commit = pinned_external_receipt(
            "CONSOLE_MANAGER_SERVING_COMMIT_RECEIPT",
            "78f32392ccf2bb7317bdd3ddb8c201d72f27b835195dc1c4d569065e42bd96aa",
        );
        assert_eq!(commit["parent"], receipt["base"]);
        assert_eq!(commit["head"], "b2333303a7d43f6ec4dcf1f092dd9dd0e990cca1");
        assert_eq!(commit["tree"], "d96fab1f81f9fc67195c5b40f52bebfa11023655");
        assert_eq!(commit["clean"], true);
        assert_eq!(commit["uninstalled"], true);
        assert_eq!(commit["hosted_integration"], false);
        assert!(LOCKS.windows(2).all(|w| w[0] < w[1]));
        // These are the actual source-review/commit receipts. Their false flags
        // remain false; only execution below may establish isolated DB facts.
    }

    fn pinned_external_receipt(key: &str, expected: &str) -> Value {
        let path = std::path::PathBuf::from(
            std::env::var_os(key)
                .unwrap_or_else(|| panic!("STOP: immutable root probe must provide {key}")),
        );
        assert!(
            path.is_absolute()
                && std::fs::symlink_metadata(&path)
                    .unwrap()
                    .file_type()
                    .is_file()
        );
        let bytes = std::fs::read(path).unwrap();
        assert_eq!(
            hex::encode(Sha256::digest(&bytes)),
            expected,
            "STOP: actual receipt drift"
        );
        serde_json::from_slice(&bytes).unwrap()
    }

    fn maintenance_lease() -> std::fs::File {
        assert_eq!(
            std::env::var("CONSOLE_MANAGER_DEDICATED_CLUSTER").as_deref(),
            Ok("root-owned-disposable"),
            "STOP: root-owned dedicated cluster required"
        );
        let path = std::path::PathBuf::from(
            std::env::var_os("CONSOLE_MANAGER_MAINTENANCE_LEASE")
                .expect("STOP: immutable root probe must declare the owned cluster lease"),
        );
        assert!(
            path.is_absolute()
                && std::fs::symlink_metadata(&path)
                    .unwrap()
                    .file_type()
                    .is_file()
        );
        let lease = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .unwrap();
        assert!(lease.metadata().unwrap().is_file());
        lease
            .try_lock()
            .expect("STOP: maintenance lease is occupied");
        lease
    }

    async fn setup(pool: &PgPool) -> std::fs::File {
        let lease = maintenance_lease();
        reviewed_sources();
        let mut tx = pool.begin().await.unwrap();
        marked(tx.as_mut(), true).await;
        tx.rollback().await.unwrap();
        prepare_policy_ready_database(pool).await;
        lease
    }

    async fn operator_tx(pool: &PgPool) -> Transaction<'static, Postgres> {
        let mut tx = pool.begin().await.unwrap();
        sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL READ COMMITTED; SET LOCAL search_path=pg_catalog,pg_temp; SET LOCAL jit=off; SET LOCAL statement_timeout='60s'; SET LOCAL lock_timeout='1s'; SET LOCAL idle_in_transaction_session_timeout='30s'; SET LOCAL transaction_timeout='120s'")
            .execute(tx.as_mut()).await.unwrap();
        marked(tx.as_mut(), false).await;
        sqlx::raw_sql("LOCK TABLE ONLY public._sqlx_migrations IN SHARE MODE")
            .execute(tx.as_mut())
            .await
            .unwrap();
        applied_ledger(tx.as_mut()).await;
        let owner_rows: Vec<String> = sqlx::query_scalar("SELECT oid::text FROM pg_catalog.pg_authid WHERE rolname='console_account_owner' FOR UPDATE")
            .fetch_all(tx.as_mut()).await.unwrap();
        assert_eq!(
            owner_rows.len(),
            1,
            "actual retained account-owner row lock"
        );
        assert!(owner_rows[0].parse::<u32>().unwrap() > 0);
        tx
    }

    async fn install(tx: &mut Transaction<'_, Postgres>) -> Result<(), sqlx::Error> {
        // Distinct entry statement: the finalizer cannot bound its own statement.
        sqlx::raw_sql("SET LOCAL search_path=pg_catalog,pg_temp; SET LOCAL jit=off; SET LOCAL statement_timeout='60s'; SET LOCAL lock_timeout='1s'")
            .execute(tx.as_mut()).await?;
        sqlx::raw_sql(FINALIZER).execute(tx.as_mut()).await?;
        sqlx::raw_sql("SET CONSTRAINTS ALL IMMEDIATE")
            .execute(tx.as_mut())
            .await?;
        Ok(())
    }

    fn procedural_refusal(error: sqlx::Error) {
        let db = error
            .as_database_error()
            .expect("SQL source refusal must be a database error");
        assert_eq!(
            db.code().as_deref(),
            Some("P0001"),
            "unrelated permission/decode/SQL failure is not custody refusal"
        );
        assert!(
            matches!(
                db.message(),
                "company_information_manager_current_policy_v1.operator_identity_mismatch"
                    | "company_information_manager_current_policy_v1.entry_settings_mismatch"
                    | "company_information_manager_current_policy_v1.migration_ledger_mismatch"
                    | "company_information_manager_current_policy_v1.profile_mismatch"
                    | "company_information_manager_current_policy_v1.predecessor_mismatch"
                    | "company_information_manager_current_policy_v1.relation_locks_missing"
            ),
            "unrelated procedural exception cannot qualify refusal"
        );
    }

    async fn state(connection: &mut PgConnection) -> String {
        sqlx::raw_sql(CLASSIFIER_SESSION)
            .execute(&mut *connection)
            .await
            .unwrap();
        let ops: String = sqlx::query_scalar(STATE)
            .fetch_one(&mut *connection)
            .await
            .unwrap();
        let app: String = sqlx::query_scalar(APP_STATE)
            .fetch_one(connection)
            .await
            .unwrap();
        assert_eq!(ops, app);
        ops
    }

    async fn exact_locks(tx: &mut Transaction<'_, Postgres>) {
        let actual: Vec<String> = sqlx::query_scalar(
            "SELECT DISTINCT c.relname::text COLLATE \"C\" FROM pg_catalog.pg_locks l \
             JOIN pg_catalog.pg_class c ON c.oid=l.relation \
             JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace \
             WHERE l.pid=pg_backend_pid() AND l.granted AND l.mode='AccessExclusiveLock' \
               AND n.nspname='public' AND c.relkind IN ('r','p') ORDER BY c.relname::text COLLATE \"C\""
        ).fetch_all(tx.as_mut()).await.unwrap();
        assert_eq!(actual, LOCKS.map(str::to_owned));
        let ledger_share: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_locks WHERE pid=pg_backend_pid() AND locktype='relation' AND relation='public._sqlx_migrations'::regclass AND mode='ShareLock' AND granted)")
            .fetch_one(tx.as_mut()).await.unwrap();
        assert!(ledger_share, "actual retained migration-ledger SHARE lock");
    }

    async fn routine_versions(
        connection: &mut PgConnection,
    ) -> Vec<(String, String, String, String, String)> {
        sqlx::query_as(
            "SELECT n.nspname::text,p.proname::text,p.oid::text,p.xmin::text,p.cmin::text \
             FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace \
             WHERE starts_with(p.proname,'identity_company_information_') \
             ORDER BY n.nspname COLLATE \"C\",p.proname COLLATE \"C\",p.oid",
        )
        .fetch_all(connection)
        .await
        .unwrap()
    }

    async fn runtime_login(pool: &PgPool) -> PgPool {
        let runtime = login_test_pool(pool, TestDatabaseLogin::Business).await;
        let actual: (String, String, bool, bool, bool) = sqlx::query_as(
            "SELECT session_user::text,current_user::text,rolsuper,rolbypassrls,rolcanlogin \
             FROM pg_catalog.pg_roles WHERE rolname=session_user",
        )
        .fetch_one(&runtime)
        .await
        .unwrap();
        assert_eq!(
            actual,
            ("console_rt".into(), "console_rt".into(), false, false, true)
        );
        runtime
    }

    async fn runtime_source_guard_calibration(runtime: &PgPool) {
        let mut tx = runtime.begin().await.unwrap();
        sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL READ COMMITTED; SET LOCAL statement_timeout='10s'; SET LOCAL lock_timeout='1s'")
            .execute(tx.as_mut()).await.unwrap();
        let rights: (bool, bool, bool, bool) = sqlx::query_as("SELECT has_function_privilege(current_user,'public.identity_company_information_manager_current_v1(uuid,uuid,uuid,uuid,uuid)','EXECUTE'),has_function_privilege(current_user,'public.identity_company_information_group_lock_v1(uuid,uuid)','EXECUTE'),has_function_privilege(current_user,'public.identity_company_information_selected_lock_v1(uuid,uuid)','EXECUTE'),has_function_privilege(current_user,'public.identity_company_information_root_material_v1(uuid,uuid)','EXECUTE')")
            .fetch_one(tx.as_mut()).await.unwrap();
        assert_eq!(rights, (true, false, false, false));
        // Nil-command refusal enters the actual installed routine before any
        // Account/Company truth is needed. It is not a healthy-manager history.
        sqlx::raw_sql("SAVEPOINT manager_runtime_parameter")
            .execute(tx.as_mut())
            .await
            .unwrap();
        let error = sqlx::query(
            "SELECT * FROM public.identity_company_information_manager_current_v1($1,$2,$3,$4,$5)",
        )
        .bind(Uuid::nil())
        .bind(Uuid::nil())
        .bind(Uuid::nil())
        .bind(Uuid::nil())
        .bind(None::<Uuid>)
        .fetch_all(tx.as_mut())
        .await
        .unwrap_err();
        let db = error
            .as_database_error()
            .expect("actual source guard must reach PostgreSQL");
        assert_eq!(db.code().as_deref(), Some("P0001"));
        assert_eq!(db.message(), "company_information.material_unavailable");
        sqlx::raw_sql("ROLLBACK TO SAVEPOINT manager_runtime_parameter; RELEASE SAVEPOINT manager_runtime_parameter")
            .execute(tx.as_mut()).await.unwrap();
        for source in [
            "SELECT * FROM public.identity_company_information_group_lock_v1($1,$2)",
            "SELECT * FROM public.identity_company_information_selected_lock_v1($1,$2)",
            "SELECT * FROM public.identity_company_information_root_material_v1($1,$2)",
        ] {
            sqlx::raw_sql("SAVEPOINT manager_runtime_helper")
                .execute(tx.as_mut())
                .await
                .unwrap();
            let error = sqlx::query(sqlx::AssertSqlSafe(source))
                .bind(Uuid::nil())
                .bind(Uuid::nil())
                .fetch_all(tx.as_mut())
                .await
                .unwrap_err();
            assert_eq!(
                error.as_database_error().unwrap().code().as_deref(),
                Some("42501")
            );
            sqlx::raw_sql("ROLLBACK TO SAVEPOINT manager_runtime_helper; RELEASE SAVEPOINT manager_runtime_helper")
                .execute(tx.as_mut()).await.unwrap();
        }
        tx.rollback().await.unwrap();
    }

    #[sqlx::test(migrations = false)]
    async fn manager_policy_v1_install_replay_and_complete_rollback_both_variants(pool: PgPool) {
        let _lease = setup(&pool).await;
        let mut baseline_tx = operator_tx(&pool).await;
        manager_observer_absent(baseline_tx.as_mut()).await;
        let baseline = policy_capture(baseline_tx.as_mut(), QUERY).await;
        let baseline_catalog = catalog(baseline_tx.as_mut()).await;
        let baseline_rows = rows(baseline_tx.as_mut()).await;
        let ledger = applied_ledger(baseline_tx.as_mut()).await;
        baseline_tx.rollback().await.unwrap();
        for variant in 0..2 {
            let mut tx = operator_tx(&pool).await;
            let outcome = AssertUnwindSafe(async {
                let _: String = sqlx::query_scalar("SELECT oid::text FROM pg_catalog.pg_authid WHERE rolname='console_account_owner' FOR UPDATE")
                    .fetch_one(tx.as_mut()).await.unwrap();
                if variant == 1 { execute(&mut tx, OBSERVER).await; }
                assert!(manager_namespace(tx.as_mut()).await.is_null());
                let predecessor = policy_capture(tx.as_mut(), OLD_QUERY).await;
                assert_eq!(predecessor.sha256, PREDECESSORS[variant]);
                let empty = policy_capture(tx.as_mut(), QUERY).await;
                assert_eq!(empty.sha256, EMPTY[variant]);
                assert!(empty.rights && predecessor.rights);
                assert_eq!(state(tx.as_mut()).await, INSTALL_REQUIRED);
                let before_catalog = catalog(tx.as_mut()).await;
                let before_rows = rows(tx.as_mut()).await;
                install(&mut tx).await.expect("actual production Manager finalizer installation");
                exact_locks(&mut tx).await;
                let installed = policy_capture(tx.as_mut(), QUERY).await;
                assert_eq!(installed.sha256, SUCCESSOR[variant]);
                assert!(installed.rights);
                assert_eq!(state(tx.as_mut()).await, FINALIZED);
                assert_eq!(manager_namespace(tx.as_mut()).await, expected_namespace());
                let installed_catalog = catalog(tx.as_mut()).await;
                let installed_versions = routine_versions(tx.as_mut()).await;
                assert_eq!(installed_versions.len(), 4);
                manager_catalog_delta(&before_catalog.1, &installed_catalog.1);
                assert!(rows(tx.as_mut()).await == before_rows, "STOP: complete nonsystem-table row census differs");
                assert_eq!(applied_ledger(tx.as_mut()).await, ledger);
                install(&mut tx).await.expect("exact finalized replay");
                exact_locks(&mut tx).await;
                assert_eq!(policy_capture(tx.as_mut(), QUERY).await, installed);
                assert_eq!(catalog(tx.as_mut()).await, installed_catalog,
                           "replay replaced source/owner/ACL or other catalog bytes");
                assert_eq!(routine_versions(tx.as_mut()).await, installed_versions,
                           "same-transaction replay repeated catalog DDL despite equal bodies");
                assert!(rows(tx.as_mut()).await == before_rows, "STOP: complete nonsystem-table row census differs");
                assert_eq!(applied_ledger(tx.as_mut()).await, ledger);
            }).catch_unwind().await;
            tx.rollback()
                .await
                .expect("complete source/observer rollback");
            let mut restored = operator_tx(&pool).await;
            assert_eq!(policy_capture(restored.as_mut(), QUERY).await, baseline);
            assert_eq!(catalog(restored.as_mut()).await, baseline_catalog);
            assert!(
                rows(restored.as_mut()).await == baseline_rows,
                "STOP: complete nonsystem-table row census differs"
            );
            assert_eq!(applied_ledger(restored.as_mut()).await, ledger);
            manager_observer_absent(restored.as_mut()).await;
            assert!(manager_namespace(restored.as_mut()).await.is_null());
            restored.rollback().await.unwrap();
            if let Err(panic) = outcome {
                std::panic::resume_unwind(panic);
            }
        }
    }

    // Each negative enters actual classifier and finalizer. Faults are isolated
    // savepoints and never change an existing test, guard or historical source.
    const FAULTS: [(&str, &str); 15] = [
        (
            "partial",
            "DROP FUNCTION public.identity_company_information_manager_current_v1(uuid,uuid,uuid,uuid,uuid)",
        ),
        (
            "unknown_schema",
            "CREATE SCHEMA manager_serving_unknown; CREATE FUNCTION manager_serving_unknown.identity_company_information_unknown_v1() RETURNS integer LANGUAGE sql AS 'SELECT 1'",
        ),
        (
            "unknown_kind",
            "CREATE PROCEDURE public.identity_company_information_unknown_v1() LANGUAGE plpgsql AS 'BEGIN RETURN; END'",
        ),
        (
            "unknown_overload",
            "CREATE FUNCTION public.identity_company_information_manager_current_v1(text) RETURNS text LANGUAGE sql AS 'SELECT $1'",
        ),
        (
            "manager_owner",
            "ALTER FUNCTION public.identity_company_information_manager_current_v1(uuid,uuid,uuid,uuid,uuid) OWNER TO console_app",
        ),
        (
            "helper_owner",
            "ALTER FUNCTION public.identity_company_information_group_lock_v1(uuid,uuid) OWNER TO console_account_owner",
        ),
        (
            "helper_runtime_acl",
            "GRANT EXECUTE ON FUNCTION public.identity_company_information_group_lock_v1(uuid,uuid) TO console_rt",
        ),
        (
            "public_acl",
            "GRANT EXECUTE ON FUNCTION public.identity_company_information_manager_current_v1(uuid,uuid,uuid,uuid,uuid) TO PUBLIC",
        ),
        (
            "helper_grant_option",
            "GRANT EXECUTE ON FUNCTION public.identity_company_information_selected_lock_v1(uuid,uuid) TO console_account_owner WITH GRANT OPTION",
        ),
        (
            "search_path",
            "ALTER FUNCTION public.identity_company_information_root_material_v1(uuid,uuid) SET search_path=public,pg_catalog",
        ),
        (
            "invoker",
            "ALTER FUNCTION public.identity_company_information_selected_lock_v1(uuid,uuid) SECURITY INVOKER",
        ),
        (
            "body",
            "CREATE OR REPLACE FUNCTION public.identity_company_information_group_lock_v1(p_company uuid,p_group uuid) RETURNS TABLE(group_row jsonb,group_head_row jsonb) LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE SET search_path=pg_catalog,pg_temp SET row_security=on SET TimeZone='UTC' SET bytea_output='hex' SET DateStyle='ISO, YMD' SET IntervalStyle='postgres' AS 'BEGIN RETURN; END'",
        ),
        (
            "default_acl",
            "ALTER DEFAULT PRIVILEGES FOR ROLE console_account_owner IN SCHEMA public GRANT EXECUTE ON FUNCTIONS TO console_rt",
        ),
        (
            "inherited_table_acl",
            "GRANT SELECT ON public.account_security TO console_rt",
        ),
        (
            "inherited_closure_body",
            "CREATE OR REPLACE FUNCTION public.native_company_policy_assert_current_head_v1(p_org uuid) RETURNS void LANGUAGE plpgsql AS 'BEGIN RETURN; END'",
        ),
    ];

    #[sqlx::test(migrations = false)]
    async fn manager_policy_v1_classifier_and_finalizer_refuse_drift_both_variants(pool: PgPool) {
        let _lease = setup(&pool).await;
        for variant in 0..2 {
            let mut tx = operator_tx(&pool).await;
            let outcome = AssertUnwindSafe(async {
                if variant == 1 {
                    execute(&mut tx, OBSERVER).await;
                }
                // Unknown namespace must refuse a new install, not just replay.
                for fault in [
                    FAULTS[1],
                    FAULTS[2],
                    FAULTS[3],
                    (
                        "known_partial",
                        include_str!(
                            "../../../../ops/native-company-information/group-lock-v1.sql"
                        ),
                    ),
                ] {
                    let mut bad = tx.begin().await.unwrap();
                    execute(&mut bad, fault.1).await;
                    assert_eq!(state(bad.as_mut()).await, MISMATCH, "{}", fault.0);
                    let before = catalog(bad.as_mut()).await;
                    let mut attempt = bad.begin().await.unwrap();
                    procedural_refusal(install(&mut attempt).await.unwrap_err());
                    attempt.rollback().await.unwrap();
                    assert_eq!(
                        catalog(bad.as_mut()).await,
                        before,
                        "unknown namespace was repaired"
                    );
                    bad.rollback().await.unwrap();
                    assert_eq!(state(tx.as_mut()).await, INSTALL_REQUIRED);
                }
                install(&mut tx).await.unwrap();
                let accepted = policy_capture(tx.as_mut(), QUERY).await;
                let accepted_catalog = catalog(tx.as_mut()).await;
                let accepted_rows = rows(tx.as_mut()).await;
                let ledger = applied_ledger(tx.as_mut()).await;
                for (name, fault) in FAULTS {
                    let mut bad = tx.begin().await.unwrap();
                    execute(&mut bad, fault).await;
                    let faulted = policy_capture(bad.as_mut(), QUERY).await;
                    assert_ne!(
                        faulted, accepted,
                        "fault failed to alter captured metadata: {name}"
                    );
                    assert_eq!(state(bad.as_mut()).await, MISMATCH, "{name}");
                    let faulted_catalog = catalog(bad.as_mut()).await;
                    let mut attempt = bad.begin().await.unwrap();
                    procedural_refusal(install(&mut attempt).await.unwrap_err());
                    attempt.rollback().await.unwrap();
                    assert_eq!(
                        catalog(bad.as_mut()).await,
                        faulted_catalog,
                        "finalizer silently repaired or adopted drift: {name}"
                    );
                    assert!(
                        rows(bad.as_mut()).await == accepted_rows,
                        "STOP: complete nonsystem-table row census differs"
                    );
                    bad.rollback().await.unwrap();
                    assert_eq!(state(tx.as_mut()).await, FINALIZED);
                    assert_eq!(policy_capture(tx.as_mut(), QUERY).await, accepted);
                    assert_eq!(catalog(tx.as_mut()).await, accepted_catalog);
                    assert!(
                        rows(tx.as_mut()).await == accepted_rows,
                        "STOP: complete nonsystem-table row census differs"
                    );
                    assert_eq!(applied_ledger(tx.as_mut()).await, ledger);
                }
            })
            .catch_unwind()
            .await;
            tx.rollback().await.unwrap();
            if let Err(panic) = outcome {
                std::panic::resume_unwind(panic);
            }
        }
    }

    #[sqlx::test(migrations = false)]
    async fn manager_policy_v1_unsupported_predecessor_absence_and_atomic_refusal(pool: PgPool) {
        let _lease = setup(&pool).await;
        for variant in 0..2 {
            let mut tx = operator_tx(&pool).await;
            let outcome = AssertUnwindSafe(async {
                if variant == 1 {
                    execute(&mut tx, OBSERVER).await;
                }
                // Actual existing PolicyV2 finalizer, not a mocked old state.
                execute(&mut tx, POLICY_V2).await;
                assert!(manager_namespace(tx.as_mut()).await.is_null());
                assert_eq!(state(tx.as_mut()).await, ABSENT);
                let before = catalog(tx.as_mut()).await;
                let before_rows = rows(tx.as_mut()).await;
                let mut attempt = tx.begin().await.unwrap();
                procedural_refusal(install(&mut attempt).await.unwrap_err());
                attempt.rollback().await.unwrap();
                assert_eq!(catalog(tx.as_mut()).await, before);
                assert!(
                    rows(tx.as_mut()).await == before_rows,
                    "STOP: complete nonsystem-table row census differs"
                );
                // A genuine unsupported composition plus the exact declared
                // Manager source must classify mismatch, even if raw old profile
                // verification independently remains available.
                execute(&mut tx, SOURCE).await;
                assert_eq!(state(tx.as_mut()).await, MISMATCH);
                let drifted = catalog(tx.as_mut()).await;
                let mut replay = tx.begin().await.unwrap();
                procedural_refusal(install(&mut replay).await.unwrap_err());
                replay.rollback().await.unwrap();
                assert_eq!(catalog(tx.as_mut()).await, drifted);
                assert!(
                    rows(tx.as_mut()).await == before_rows,
                    "STOP: complete nonsystem-table row census differs"
                );
            })
            .catch_unwind()
            .await;
            tx.rollback().await.unwrap();
            if let Err(panic) = outcome {
                std::panic::resume_unwind(panic);
            }
        }
    }

    async fn operator_and_locks(pool: PgPool, variant: usize) {
        assert_eq!(
            std::env::var("CONSOLE_MANAGER_DEDICATED_CLUSTER").as_deref(),
            Ok("root-owned-disposable"),
            "STOP: exclusive disposable cluster required"
        );
        let _lease = setup(&pool).await;
        let mut optional_observer = operator_tx(&pool).await;
        manager_observer_absent(optional_observer.as_mut()).await;
        if variant == 1 {
            execute(&mut optional_observer, OBSERVER).await;
        }
        assert_eq!(
            policy_capture(optional_observer.as_mut(), OLD_QUERY)
                .await
                .sha256,
            PREDECESSORS[variant]
        );
        optional_observer.commit().await.unwrap();
        let mut before_tx = operator_tx(&pool).await;
        let before = catalog(before_tx.as_mut()).await;
        let before_rows = rows(before_tx.as_mut()).await;
        before_tx.rollback().await.unwrap();
        for isolation in ["repeatable read", "serializable"] {
            let mut tx = pool.begin().await.unwrap();
            let sql = format!(
                "SET TRANSACTION ISOLATION LEVEL {isolation}; SET LOCAL statement_timeout='30s'"
            );
            sqlx::raw_sql(sqlx::AssertSqlSafe(sql))
                .execute(tx.as_mut())
                .await
                .unwrap();
            procedural_refusal(install(&mut tx).await.unwrap_err());
            tx.rollback().await.unwrap();
        }
        let runtime = runtime_login(&pool).await;
        let mut unauthorized = runtime.begin().await.unwrap();
        sqlx::raw_sql(
            "SET TRANSACTION ISOLATION LEVEL READ COMMITTED; SET LOCAL statement_timeout='30s'",
        )
        .execute(unauthorized.as_mut())
        .await
        .unwrap();
        procedural_refusal(install(&mut unauthorized).await.unwrap_err());
        unauthorized.rollback().await.unwrap();
        runtime.close().await;
        // Every retained relation is tested as a real conflicting lock; the
        // expected timeout alone cannot prove where the waiter actually waited.
        for relation in LOCKS {
            let mut blocker = pool.begin().await.unwrap();
            let sql = format!("LOCK TABLE ONLY public.{relation} IN ACCESS SHARE MODE");
            sqlx::raw_sql(sqlx::AssertSqlSafe(sql))
                .execute(blocker.as_mut())
                .await
                .unwrap();
            let mut contender = operator_tx(&pool).await;
            let pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
                .fetch_one(contender.as_mut())
                .await
                .unwrap();
            let mut running = tokio::spawn(async move {
                let result = install(&mut contender).await;
                contender.rollback().await.unwrap();
                result
            });
            let observed = tokio::time::timeout(Duration::from_millis(800), async {
                loop {
                    let waiting: bool = sqlx::query_scalar(
                        "SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_locks l \
                         JOIN pg_catalog.pg_class c ON c.oid=l.relation \
                         JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace \
                         WHERE l.pid=$1 AND NOT l.granted AND l.mode='AccessExclusiveLock' \
                           AND n.nspname='public' AND c.relname=$2)",
                    )
                    .bind(pid)
                    .bind(relation)
                    .fetch_one(&pool)
                    .await
                    .unwrap();
                    if waiting {
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            })
            .await;
            // Keep the blocker until the actual finalizer times out, then release
            // it before asserting, including a faulted observation path.
            let finished = tokio::time::timeout(Duration::from_secs(5), &mut running).await;
            if finished.is_err() {
                running.abort();
                let _ = running.await;
            }
            blocker.rollback().await.unwrap();
            observed.expect("actual finite finalizer did not wait on the required relation");
            let error = finished
                .expect("unbounded finalizer lock wait")
                .expect("finalizer waiter panicked")
                .unwrap_err();
            assert_eq!(
                error.as_database_error().unwrap().code().as_deref(),
                Some("55P03")
            );
            let mut restored = operator_tx(&pool).await;
            assert_eq!(catalog(restored.as_mut()).await, before);
            assert!(
                rows(restored.as_mut()).await == before_rows,
                "STOP: complete nonsystem-table row census differs"
            );
            assert_eq!(state(restored.as_mut()).await, INSTALL_REQUIRED);
            restored.rollback().await.unwrap();
        }
        let mut positive = operator_tx(&pool).await;
        install(&mut positive).await.unwrap();
        exact_locks(&mut positive).await;
        assert_eq!(state(positive.as_mut()).await, FINALIZED);
        positive.rollback().await.unwrap();
    }

    #[sqlx::test(migrations = false)]
    async fn manager_policy_v1_plain_operator_isolation_and_real_locks_are_atomic(pool: PgPool) {
        operator_and_locks(pool, 0).await;
    }

    #[sqlx::test(migrations = false)]
    async fn manager_policy_v1_observer_operator_isolation_and_real_locks_are_atomic(pool: PgPool) {
        operator_and_locks(pool, 1).await;
    }

    async fn change(pool: &PgPool, sql: &'static str) {
        let mut tx = operator_tx(pool).await;
        execute(&mut tx, sql).await;
        tx.commit().await.unwrap();
    }

    // Exercise old early-return behavior only while its frozen profile is a
    // genuine positive. The added namespace is owned by the disposable operator,
    // never by a protected owner selected in the historical routine snapshot.
    async fn unknown_namespace_refuses_old_fallback(
        pool: &PgPool,
        runtime: &PgPool,
        config: &AppConfig,
        old_classifier: &'static str,
        old_finalized: &'static str,
        manager_baseline: &'static str,
    ) {
        assert_eq!(
            classified(runtime, old_classifier).await,
            old_finalized,
            "STOP: actual historical classifier prerequisite failed"
        );
        assert_eq!(
            classified(runtime, STATE).await,
            manager_baseline,
            "STOP: actual empty Manager namespace prerequisite failed"
        );
        let old = AppState::from_config(config.clone())
            .await
            .expect("STOP: actual historical application startup prerequisite failed");
        let check = AssertUnwindSafe(async {
            assert_eq!(ready_status(&old).await, StatusCode::OK);
        })
        .catch_unwind()
        .await;
        close_states(&[old], check).await;
        let mut before = operator_tx(pool).await;
        assert!(manager_namespace(before.as_mut()).await.is_null());
        let before_catalog = catalog(before.as_mut()).await;
        let before_rows = rows(before.as_mut()).await;
        let before_versions = routine_versions(before.as_mut()).await;
        let ledger = applied_ledger(before.as_mut()).await;
        assert!(before_versions.is_empty());
        before.rollback().await.unwrap();
        for (name, mutate, restore) in [
            (
                "foreign_schema",
                "CREATE SCHEMA manager_startup_unknown AUTHORIZATION console_buck_admin; CREATE FUNCTION manager_startup_unknown.identity_company_information_extra_v1() RETURNS integer LANGUAGE sql AS 'SELECT 1'; REVOKE ALL ON FUNCTION manager_startup_unknown.identity_company_information_extra_v1() FROM PUBLIC,console_rt",
                "DROP FUNCTION manager_startup_unknown.identity_company_information_extra_v1(); DROP SCHEMA manager_startup_unknown",
            ),
            (
                "routine_kind",
                "CREATE PROCEDURE public.identity_company_information_extra_v1() LANGUAGE plpgsql AS 'BEGIN RETURN; END'; REVOKE ALL ON PROCEDURE public.identity_company_information_extra_v1() FROM PUBLIC,console_rt",
                "DROP PROCEDURE public.identity_company_information_extra_v1()",
            ),
            (
                "overload",
                "CREATE FUNCTION public.identity_company_information_manager_current_v1(text) RETURNS text LANGUAGE sql AS 'SELECT $1'; REVOKE ALL ON FUNCTION public.identity_company_information_manager_current_v1(text) FROM PUBLIC,console_rt",
                "DROP FUNCTION public.identity_company_information_manager_current_v1(text)",
            ),
        ] {
            change(pool, mutate).await;
            let refused = AssertUnwindSafe(async {
                let mut calibrated = operator_tx(pool).await;
                let isolated: bool = sqlx::query_scalar("SELECT count(*)=1 AND bool_and(pg_catalog.pg_get_userbyid(p.proowner)='console_buck_admin' AND NOT p.prosecdef AND NOT pg_catalog.has_function_privilege('console_rt',p.oid,'EXECUTE')) FROM pg_catalog.pg_proc p WHERE starts_with(p.proname,'identity_company_information_')")
                    .fetch_one(calibrated.as_mut()).await.unwrap();
                assert!(isolated, "STOP: operator-owned unknown routine calibration failed");
                assert!(rows(calibrated.as_mut()).await == before_rows,
                        "STOP: complete nonsystem-table row census differs");
                assert_eq!(applied_ledger(calibrated.as_mut()).await, ledger);
                calibrated.rollback().await.unwrap();
                assert_eq!(classified(runtime, STATE).await, MISMATCH, "{name}");
                assert_eq!(classified(runtime, old_classifier).await, old_finalized,
                           "STOP: old fallback must remain an actual exact positive: {name}");
                startup_refused(config.clone(), MISMATCH).await;
            }).catch_unwind().await;
            // Restore even when the missing new dispatch produces the intended
            // refusal failure. A failed old positive remains STOP, never RED.
            change(pool, restore).await;
            let mut restored = operator_tx(pool).await;
            assert_eq!(catalog(restored.as_mut()).await, before_catalog);
            assert!(
                rows(restored.as_mut()).await == before_rows,
                "STOP: complete nonsystem-table row census differs"
            );
            assert_eq!(routine_versions(restored.as_mut()).await, before_versions);
            assert_eq!(applied_ledger(restored.as_mut()).await, ledger);
            assert!(manager_namespace(restored.as_mut()).await.is_null());
            restored.rollback().await.unwrap();
            assert_eq!(classified(runtime, STATE).await, manager_baseline);
            assert_eq!(classified(runtime, old_classifier).await, old_finalized);
            let adjacent = AppState::from_config(config.clone())
                .await
                .expect("STOP: restored historical application prerequisite failed");
            let check = AssertUnwindSafe(async {
                assert_eq!(ready_status(&adjacent).await, StatusCode::OK);
            })
            .catch_unwind()
            .await;
            close_states(&[adjacent], check).await;
            if let Err(panic) = refused {
                std::panic::resume_unwind(panic);
            }
        }
    }

    // Each startup leaf requires its own ROOT-OWNED disposable cluster. This
    // guard prevents an observer role/pg_control_system ACL change on a shared
    // test cluster. Root's command/probe pins resource custody and exact sources.
    async fn startup(pool: PgPool, variant: usize) {
        let _lease = setup(&pool).await;
        let mut tx = operator_tx(&pool).await;
        manager_observer_absent(tx.as_mut()).await;
        if variant == 1 {
            execute(&mut tx, OBSERVER).await;
        }
        assert_eq!(
            policy_capture(tx.as_mut(), OLD_QUERY).await.sha256,
            PREDECESSORS[variant]
        );
        assert_eq!(
            policy_capture(tx.as_mut(), QUERY).await.sha256,
            EMPTY[variant]
        );
        assert_eq!(state(tx.as_mut()).await, INSTALL_REQUIRED);
        tx.commit().await.unwrap();
        let runtime = runtime_login(&pool).await;
        let artifacts = Artifacts::new();
        let key = SigningKey::random(&mut OsRng);
        let config = account_browser_config(&pool, artifacts.root.clone(), &key);
        let outcome = AssertUnwindSafe(async {
            unknown_namespace_refuses_old_fallback(
                &pool,
                &runtime,
                &config,
                APP_POLICY_CLASSIFIER,
                "native_company_policy.finalized",
                INSTALL_REQUIRED,
            )
            .await;
            // The complete source changes protected-owner routine snapshots.
            // Only the new classifier/profile can qualify installed Manager.
            let mut installing = operator_tx(&pool).await;
            let before_rows = rows(installing.as_mut()).await;
            let ledger = applied_ledger(installing.as_mut()).await;
            install(&mut installing)
                .await
                .expect("STOP: accepted Manager SQL installation prerequisite failed");
            exact_locks(&mut installing).await;
            let accepted = policy_capture(installing.as_mut(), QUERY).await;
            assert_eq!(accepted.sha256, SUCCESSOR[variant]);
            assert!(accepted.rights);
            let accepted_catalog = catalog(installing.as_mut()).await;
            let accepted_versions = routine_versions(installing.as_mut()).await;
            assert!(
                rows(installing.as_mut()).await == before_rows,
                "STOP: complete nonsystem-table row census differs"
            );
            assert_eq!(applied_ledger(installing.as_mut()).await, ledger);
            installing.commit().await.unwrap();
            assert_eq!(classified(&runtime, STATE).await, FINALIZED);
            runtime_source_guard_calibration(&runtime).await;
            let mut guarded = operator_tx(&pool).await;
            assert!(
                rows(guarded.as_mut()).await == before_rows,
                "STOP: complete nonsystem-table row census differs"
            );
            assert_eq!(catalog(guarded.as_mut()).await, accepted_catalog);
            assert_eq!(applied_ledger(guarded.as_mut()).await, ledger);
            guarded.rollback().await.unwrap();
            let fresh = match AppState::from_config(config.clone()).await {
                Ok(state) => state,
                Err(AppError::Config(_)) => panic!("MANAGER_CUSTODY_STARTUP_UNAVAILABLE"),
                Err(_) => panic!("STOP: unrelated restricted LOGIN startup failure"),
            };
            let check = AssertUnwindSafe(async {
                assert_eq!(ready_status(&fresh).await, StatusCode::OK);
            })
            .catch_unwind()
            .await;
            close_states(&[fresh], check).await;
            // Exact replay commits without routine replacement or row changes.
            let mut replay = operator_tx(&pool).await;
            install(&mut replay).await.unwrap();
            exact_locks(&mut replay).await;
            assert_eq!(policy_capture(replay.as_mut(), QUERY).await, accepted);
            assert_eq!(applied_ledger(replay.as_mut()).await, ledger);
            assert_eq!(catalog(replay.as_mut()).await, accepted_catalog);
            assert_eq!(
                routine_versions(replay.as_mut()).await,
                accepted_versions,
                "postcommit replay replaced an equal-source routine tuple"
            );
            assert!(
                rows(replay.as_mut()).await == before_rows,
                "STOP: complete nonsystem-table row census differs"
            );
            replay.commit().await.unwrap();
        })
        .catch_unwind()
        .await;
        runtime.close().await;
        if let Err(panic) = outcome {
            std::panic::resume_unwind(panic);
        }
        // Source/observer remain only in this disposable fixture. Root destroys
        // its owned cluster; no role drop, old hash reseal or shared repair.
    }

    #[sqlx::test(migrations = false)]
    async fn manager_policy_v1_plain_real_login_startup_refuses_old_fallback(pool: PgPool) {
        startup(pool, 0).await;
    }

    #[sqlx::test(migrations = false)]
    async fn manager_policy_v1_observer_real_login_startup_refuses_old_fallback(pool: PgPool) {
        startup(pool, 1).await;
    }

    // Prepare the supported frozen PolicyV2 before adding any Manager routine.
    // Installing PolicyV2 on top of Manager would fail an unrelated predecessor
    // gate and cannot be evidence of the application refusal being tested.
    async fn policy_v2_unsupported_startup(pool: PgPool, variant: usize) {
        let _lease = setup(&pool).await;
        let mut tx = operator_tx(&pool).await;
        manager_observer_absent(tx.as_mut()).await;
        if variant == 1 {
            execute(&mut tx, OBSERVER).await;
        }
        assert_eq!(
            policy_capture(tx.as_mut(), OLD_QUERY).await.sha256,
            PREDECESSORS[variant]
        );
        execute(&mut tx, POLICY_V2).await;
        assert!(manager_namespace(tx.as_mut()).await.is_null());
        assert_eq!(state(tx.as_mut()).await, ABSENT);
        let before_rows = rows(tx.as_mut()).await;
        let ledger = applied_ledger(tx.as_mut()).await;
        tx.commit().await.unwrap();
        let runtime = runtime_login(&pool).await;
        let artifacts = Artifacts::new();
        let key = SigningKey::random(&mut OsRng);
        let config = account_browser_config(&pool, artifacts.root.clone(), &key);
        let predecessor = include_str!("../../src/native_company_policy_v2_custody_state.sql");
        let outcome = AssertUnwindSafe(async {
            assert_eq!(
                classified(&runtime, predecessor).await,
                "native_company_policy_v2.finalized",
                "STOP: supported frozen PolicyV2 prerequisite failed"
            );
            let old = AppState::from_config(config.clone())
                .await
                .expect("STOP: supported frozen PolicyV2 application prerequisite failed");
            let check = AssertUnwindSafe(async {
                assert_eq!(ready_status(&old).await, StatusCode::OK);
            })
            .catch_unwind()
            .await;
            close_states(&[old], check).await;
            let mut unsupported = operator_tx(&pool).await;
            execute(&mut unsupported, SOURCE).await;
            let faulted_catalog = catalog(unsupported.as_mut()).await;
            assert_eq!(state(unsupported.as_mut()).await, MISMATCH);
            let mut refused = unsupported.begin().await.unwrap();
            procedural_refusal(install(&mut refused).await.unwrap_err());
            refused.rollback().await.unwrap();
            assert_eq!(catalog(unsupported.as_mut()).await, faulted_catalog);
            assert!(
                rows(unsupported.as_mut()).await == before_rows,
                "STOP: complete nonsystem-table row census differs"
            );
            assert_eq!(applied_ledger(unsupported.as_mut()).await, ledger);
            unsupported.commit().await.unwrap();
            assert_eq!(classified(&runtime, STATE).await, MISMATCH);
            runtime_source_guard_calibration(&runtime).await;
            // Full source is unsupported here; no frozen-profile finalized
            // assertion is made after its protected-owner snapshot changes.
            startup_refused(config, MISMATCH).await;
            let mut after = operator_tx(&pool).await;
            assert_eq!(catalog(after.as_mut()).await, faulted_catalog);
            assert!(
                rows(after.as_mut()).await == before_rows,
                "STOP: complete nonsystem-table row census differs"
            );
            assert_eq!(applied_ledger(after.as_mut()).await, ledger);
            after.rollback().await.unwrap();
        })
        .catch_unwind()
        .await;
        runtime.close().await;
        if let Err(panic) = outcome {
            std::panic::resume_unwind(panic);
        }
    }

    #[sqlx::test(migrations = false)]
    async fn manager_plain_source_refuses_actual_policy_v2_unsupported_composition(pool: PgPool) {
        policy_v2_unsupported_startup(pool, 0).await;
    }

    #[sqlx::test(migrations = false)]
    async fn manager_observer_source_refuses_actual_policy_v2_unsupported_composition(
        pool: PgPool,
    ) {
        policy_v2_unsupported_startup(pool, 1).await;
    }

    #[cfg(feature = "test-browser")]
    async fn navigation_precedence(pool: PgPool, variant: usize) {
        let _lease = maintenance_lease();
        reviewed_sources();
        let mut marker = pool.begin().await.unwrap();
        marked(marker.as_mut(), true).await;
        marker.rollback().await.unwrap();
        // Actual Group/Navigation sources only; no browser-driven or SQL-seeded
        // Account/Company business truth is introduced by this metadata fixture.
        prepare_native_group_navigation_browser_database(&pool).await;
        let mut tx = operator_tx(&pool).await;
        manager_observer_absent(tx.as_mut()).await;
        if variant == 1 {
            execute(&mut tx, OBSERVER).await;
        }
        let before_rows = rows(tx.as_mut()).await;
        let ledger = applied_ledger(tx.as_mut()).await;
        assert_eq!(state(tx.as_mut()).await, ABSENT);
        tx.commit().await.unwrap();
        let runtime = runtime_login(&pool).await;
        let artifacts = Artifacts::new();
        let key = SigningKey::random(&mut OsRng);
        let config = account_browser_config(&pool, artifacts.root.clone(), &key);
        let navigation =
            include_str!("../../src/native_group_process_navigation_serving_v1_custody_state.sql");
        let outcome = AssertUnwindSafe(async {
            unknown_namespace_refuses_old_fallback(
                &pool,
                &runtime,
                &config,
                navigation,
                "native_group_process_navigation.finalized",
                ABSENT,
            )
            .await;
            // Separate unsupported full-source composition. Adding protected-
            // owner routines also invalidates the old Navigation snapshot.
            let mut unsupported = operator_tx(&pool).await;
            execute(&mut unsupported, SOURCE).await;
            let faulted_catalog = catalog(unsupported.as_mut()).await;
            assert_eq!(state(unsupported.as_mut()).await, MISMATCH);
            let mut refused = unsupported.begin().await.unwrap();
            procedural_refusal(install(&mut refused).await.unwrap_err());
            refused.rollback().await.unwrap();
            assert_eq!(catalog(unsupported.as_mut()).await, faulted_catalog);
            assert!(
                rows(unsupported.as_mut()).await == before_rows,
                "STOP: complete nonsystem-table row census differs"
            );
            assert_eq!(applied_ledger(unsupported.as_mut()).await, ledger);
            unsupported.commit().await.unwrap();
            assert_eq!(classified(&runtime, STATE).await, MISMATCH);
            runtime_source_guard_calibration(&runtime).await;
            startup_refused(config, MISMATCH).await;
            let mut after = operator_tx(&pool).await;
            assert_eq!(catalog(after.as_mut()).await, faulted_catalog);
            assert!(
                rows(after.as_mut()).await == before_rows,
                "STOP: complete nonsystem-table row census differs"
            );
            assert_eq!(applied_ledger(after.as_mut()).await, ledger);
            after.rollback().await.unwrap();
        })
        .catch_unwind()
        .await;
        runtime.close().await;
        if let Err(panic) = outcome {
            std::panic::resume_unwind(panic);
        }
    }

    #[cfg(feature = "test-browser")]
    #[sqlx::test(migrations = false)]
    async fn manager_plain_namespace_refuses_actual_navigation_early_return(pool: PgPool) {
        navigation_precedence(pool, 0).await;
    }

    #[cfg(feature = "test-browser")]
    #[sqlx::test(migrations = false)]
    async fn manager_observer_namespace_refuses_actual_navigation_early_return(pool: PgPool) {
        navigation_precedence(pool, 1).await;
    }
}
