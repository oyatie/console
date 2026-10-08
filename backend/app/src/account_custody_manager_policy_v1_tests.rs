// Schema/custody-only tests. No Account, Company or user business truth is created.
use super::VerifiedCustodyProfile;

#[cfg(not(feature = "test-postgres"))]
#[test]
fn historical_profile_capabilities_remain_exact() {
    for (profile, policy, people, directory, group, provenance) in [
        (
            VerifiedCustodyProfile::NativeAccount,
            false,
            false,
            false,
            false,
            false,
        ),
        (
            VerifiedCustodyProfile::CompanyEnrollment,
            false,
            false,
            false,
            false,
            false,
        ),
        (
            VerifiedCustodyProfile::NativeCompanyPolicy,
            true,
            false,
            false,
            false,
            false,
        ),
        (
            VerifiedCustodyProfile::NativeCompanyPolicyV2,
            true,
            true,
            false,
            false,
            false,
        ),
        (
            VerifiedCustodyProfile::NativePeopleDirectory,
            true,
            true,
            false,
            false,
            false,
        ),
        (
            VerifiedCustodyProfile::NativePeopleDirectoryRowLock,
            true,
            true,
            true,
            false,
            true,
        ),
        (
            VerifiedCustodyProfile::NativeOrgBridgeCompatible,
            true,
            true,
            true,
            false,
            true,
        ),
        (
            VerifiedCustodyProfile::NativeGroupProcess,
            true,
            true,
            true,
            true,
            true,
        ),
        (
            VerifiedCustodyProfile::NativeGroupProcessNavigation,
            true,
            true,
            true,
            true,
            true,
        ),
    ] {
        assert_eq!(profile.supports_policy(), policy, "{profile:?}");
        assert_eq!(profile.supports_people(), people, "{profile:?}");
        assert_eq!(
            profile.supports_native_directory(),
            directory,
            "{profile:?}"
        );
        assert_eq!(
            profile.supports_native_group_process(),
            group,
            "{profile:?}"
        );
        assert_eq!(
            profile.requires_current_company_provenance(),
            provenance,
            "{profile:?}"
        );
    }
}

#[cfg(feature = "test-postgres")]
mod actual_profile {
    use crate::AppError;
    use console_platform_test_support::{TestDatabaseLogin, login_test_pool};
    use futures::FutureExt;
    use sha2::{Digest, Sha256};
    use sqlx::{PgPool, Postgres, Transaction};
    use std::panic::AssertUnwindSafe;
    use uuid::Uuid;

    const COMPANY: &str = include_str!("../../../ops/postgres-finalize-company-enrollment.sql");
    const POLICY: &str = include_str!("../../../ops/postgres-finalize-native-company-policy.sql");
    const FINALIZER: &str = include_str!(
        "../../../ops/postgres-finalize-company-information-manager-current-policy-v1.sql"
    );
    const STATE: &str =
        include_str!("company_information_manager_current_policy_v1_custody_state.sql");
    const PREDECESSOR: &str = include_str!("native_company_policy_custody_state.sql");
    const SESSION: &str = include_str!("account_custody_session.sql");
    const LEDGER: &str = include_str!("../../../ops/account-custody-migrations.sha384");
    static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../crates/platform/db/migrations");

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

    fn source_pins() {
        for (source, expected) in [
            (
                COMPANY,
                "bc35b52d5692e474a3c890dde73112f56e7b85a241ab390075d58b5d5b43a92f",
            ),
            (
                POLICY,
                "4e7fc41b1d2ed6c2155d44d43347c18815ed9e70611eb9285f21e1c590bf996a",
            ),
            (
                FINALIZER,
                "4916eaf30459b7d28f17b4fbb4793f2159b82e5f9bf5b8102ec6b0abde9fc30d",
            ),
            (
                STATE,
                "2ad10750d99ded7ff02acd5ddb10214eefa4063d7d0a14c2ca83e1db9e263853",
            ),
        ] {
            assert_eq!(
                hex::encode(Sha256::digest(source.as_bytes())),
                expected,
                "STOP: exact accepted source prerequisite drift"
            );
        }
        assert!(LOCKS.windows(2).all(|w| w[0] < w[1]));
        for (key, expected) in [
            (
                "CONSOLE_MANAGER_SERVING_SOURCE_RECEIPT",
                "fd4c4f9e0c4e3f7aff02ee0406562a2662c61dccfc7cb93ed5bc14f1394ae8c2",
            ),
            (
                "CONSOLE_MANAGER_SERVING_COMMIT_RECEIPT",
                "78f32392ccf2bb7317bdd3ddb8c201d72f27b835195dc1c4d569065e42bd96aa",
            ),
        ] {
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
                hex::encode(Sha256::digest(bytes)),
                expected,
                "STOP: actual source receipt drift"
            );
        }
    }

    fn maintenance_lease() -> std::fs::File {
        assert_eq!(
            std::env::var("CONSOLE_MANAGER_DEDICATED_CLUSTER").as_deref(),
            Ok("root-owned-disposable"),
            "STOP: root-owned dedicated cluster required"
        );
        let path = std::path::PathBuf::from(
            std::env::var_os("CONSOLE_MANAGER_MAINTENANCE_LEASE")
                .expect("STOP: root probe must declare this cluster's maintenance lease"),
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
            .open(&path)
            .unwrap();
        assert!(lease.metadata().unwrap().is_file());
        lease
            .try_lock()
            .expect("STOP: cluster maintenance lease is occupied");
        // Actual OS lock stays held by this File through completion/drop. Root's
        // separate admission binds the path and authenticates the owned cluster.
        lease
    }

    async fn operator_tx(pool: &PgPool) -> Transaction<'static, Postgres> {
        let mut tx = pool.begin().await.unwrap();
        sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL READ COMMITTED; SET LOCAL search_path=pg_catalog,pg_temp; SET LOCAL jit=off; SET LOCAL statement_timeout='60s'; SET LOCAL lock_timeout='1s'; SET LOCAL idle_in_transaction_session_timeout='30s'; SET LOCAL transaction_timeout='120s'")
            .execute(tx.as_mut()).await.unwrap();
        let marked: bool = sqlx::query_scalar("SELECT session_user=current_user AND current_user='console_buck_admin' AND starts_with(current_database(),'_sqlx_test_') AND current_setting('console.sqlx_test_bootstrap',true)='buck-sqlx-superuser-v1' AND (SELECT rolsuper FROM pg_catalog.pg_roles WHERE rolname=current_user)")
            .fetch_one(tx.as_mut()).await.unwrap();
        assert!(marked, "STOP: genuine marked disposable operator required");
        sqlx::raw_sql("LOCK TABLE ONLY public._sqlx_migrations IN SHARE MODE")
            .execute(tx.as_mut())
            .await
            .unwrap();
        assert_eq!(
            hex::encode(Sha256::digest(LEDGER.as_bytes())),
            "42079d3f1b8077e163960adc65f35f1959c22a67bf42acf43d6b816721ba1357"
        );
        let records: Vec<_> = LEDGER.split_inclusive('\n').collect();
        assert_eq!(records.len(), 231);
        assert_eq!(
            hex::encode(Sha256::digest(records[..230].concat().as_bytes())),
            "25e02488cdaf864f6d15ee21d62df98eb263bb82de1e2a283470ca659d160325"
        );
        let expected: Vec<_> = MIGRATOR
            .iter()
            .map(|migration| (migration.version, true, migration.checksum.to_vec()))
            .collect();
        assert_eq!(expected.len(), 231);
        let applied: Vec<(i64, bool, Vec<u8>)> = sqlx::query_as(
            "SELECT version,success,checksum FROM public._sqlx_migrations ORDER BY version",
        )
        .fetch_all(tx.as_mut())
        .await
        .unwrap();
        assert_eq!(
            applied, expected,
            "STOP: complete applied migration ledger differs"
        );
        let role: Vec<String> = sqlx::query_scalar("SELECT oid::text FROM pg_catalog.pg_authid WHERE rolname='console_account_owner' FOR UPDATE")
            .fetch_all(tx.as_mut()).await.unwrap();
        assert_eq!(role.len(), 1);
        assert!(role[0].parse::<u32>().unwrap() > 0);
        tx
    }

    async fn classified(runtime: &PgPool, source: &'static str) -> String {
        let mut tx = runtime.begin().await.unwrap();
        sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
            .execute(tx.as_mut())
            .await
            .unwrap();
        sqlx::raw_sql(SESSION).execute(tx.as_mut()).await.unwrap();
        let state: String = sqlx::query_scalar(sqlx::AssertSqlSafe(source))
            .fetch_one(tx.as_mut())
            .await
            .expect("STOP: actual classifier prerequisite SQL fault");
        tx.commit().await.unwrap();
        state
    }

    async fn rows(pool: &PgPool) -> std::collections::BTreeMap<String, String> {
        let tables: Vec<(String, String)> = sqlx::query_as("SELECT n.nspname::text,c.relname::text FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace WHERE c.relkind IN ('r','p') AND n.nspname NOT IN ('pg_catalog','information_schema') AND NOT starts_with(n.nspname,'pg_toast') AND NOT starts_with(n.nspname,'pg_temp_') ORDER BY n.nspname COLLATE \"C\",c.relname COLLATE \"C\"")
            .fetch_all(pool).await.unwrap();
        assert!(!tables.is_empty() && tables.len() <= 1024);
        let mut result = std::collections::BTreeMap::new();
        let mut total = 0_usize;
        for (schema, table) in tables {
            let identity = serde_json::to_string(&(schema.clone(), table.clone())).unwrap();
            let schema = schema.replace('"', "\"\"");
            let table = table.replace('"', "\"\"");
            let sql = format!(
                "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text COLLATE \"C\"),'[]'::jsonb)::text FROM \"{schema}\".\"{table}\" t"
            );
            let value: String = sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
                .fetch_one(pool)
                .await
                .unwrap();
            assert!(value.len() <= 16 * 1024 * 1024);
            let decoded: serde_json::Value = serde_json::from_str(&value).unwrap();
            assert!(
                decoded
                    .as_array()
                    .is_some_and(|items| items.iter().all(serde_json::Value::is_object))
            );
            total = total.checked_add(value.len()).unwrap();
            assert!(total <= 64 * 1024 * 1024);
            assert!(result.insert(identity, value).is_none());
        }
        result
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

    #[cfg(feature = "test-postgres")]
    #[sqlx::test(migrations = false)]
    async fn actual_manager_policy_v1_verify_returns_finite_profile_without_population(
        pool: PgPool,
    ) {
        let _lease = maintenance_lease();
        source_pins();
        console_platform_test_support::prepare_account_test_database(&pool).await;
        let mut tx = operator_tx(&pool).await;
        let absent_observer: bool = sqlx::query_scalar("SELECT NOT EXISTS(SELECT 1 FROM pg_catalog.pg_roles WHERE rolname='console_durability_observer') AND NOT EXISTS(SELECT 1 FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace WHERE n.nspname='public' AND p.proname='console_durability_observation_v1')")
            .fetch_one(tx.as_mut()).await.unwrap();
        assert!(
            absent_observer,
            "STOP: fresh plain fixture must have absent observer"
        );
        sqlx::raw_sql(COMPANY).execute(tx.as_mut()).await.unwrap();
        sqlx::raw_sql(POLICY).execute(tx.as_mut()).await.unwrap();
        tx.commit().await.unwrap();
        let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
        let outcome = AssertUnwindSafe(async {
            let identity: (String, String, bool, bool, bool) = sqlx::query_as("SELECT session_user::text,current_user::text,rolsuper,rolbypassrls,rolcanlogin FROM pg_catalog.pg_roles WHERE rolname=session_user")
                .fetch_one(&runtime).await.unwrap();
            assert_eq!(identity, ("console_rt".into(), "console_rt".into(), false, false, true));
            assert_eq!(classified(&runtime, PREDECESSOR).await, "native_company_policy.finalized");
            assert_eq!(classified(&runtime, STATE).await, "company_information_manager_current_policy_v1.install_required");
            assert_eq!(super::super::verify(&runtime).await.unwrap(), super::VerifiedCustodyProfile::NativeCompanyPolicy,
                       "STOP: retained actual PolicyV1 verifier prerequisite failed");
            let before = rows(&pool).await;
            let mut tx = operator_tx(&pool).await;
            sqlx::raw_sql(FINALIZER).execute(tx.as_mut()).await.expect("STOP: accepted Manager SQL installation prerequisite failed");
            let actual: Vec<String> = sqlx::query_scalar("SELECT DISTINCT c.relname::text COLLATE \"C\" FROM pg_catalog.pg_locks l JOIN pg_catalog.pg_class c ON c.oid=l.relation JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace WHERE l.pid=pg_backend_pid() AND l.granted AND l.mode='AccessExclusiveLock' AND n.nspname='public' AND c.relkind='r' ORDER BY c.relname::text COLLATE \"C\"")
                .fetch_all(tx.as_mut()).await.unwrap();
            assert_eq!(actual, LOCKS.map(str::to_owned));
            tx.commit().await.unwrap();
            assert!(rows(&pool).await == before, "STOP: complete nonsystem-table row census differs");
            assert_eq!(classified(&runtime, STATE).await, "company_information_manager_current_policy_v1.finalized");
            runtime_source_guard_calibration(&runtime).await;
            assert!(rows(&pool).await == before, "STOP: complete nonsystem-table row census differs");
            // Current Rust symbols only: the returned actual profile must be new
            // and narrow. Debug identity is an assertion oracle, never authority.
            let profile = match super::super::verify(&runtime).await {
                Ok(profile) => profile,
                Err(AppError::Config(code)) => panic!("MANAGER_CUSTODY_PROFILE_UNAVAILABLE: {code}"),
                Err(_) => panic!("STOP: unrelated verifier connection/source failure"),
            };
            assert_eq!(format!("{profile:?}"), "NativeCompanyInformationManagerCurrentPolicyV1",
                       "MANAGER_CUSTODY_PROFILE_MISCLASSIFIED");
            assert!(profile.supports_policy());
            assert!(!profile.supports_people());
            assert!(!profile.supports_native_directory());
            assert!(!profile.supports_native_group_process());
            assert!(!profile.requires_current_company_provenance());
            assert!(rows(&pool).await == before, "STOP: complete nonsystem-table row census differs");
        }).catch_unwind().await;
        runtime.close().await;
        if let Err(panic) = outcome {
            std::panic::resume_unwind(panic);
        }
    }
}
