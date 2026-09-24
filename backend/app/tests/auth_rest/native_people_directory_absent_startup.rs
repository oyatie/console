// External additive proposal: include inside native_policy_startup_tests.
// Genuine historical229 producer; no current-schema subtraction or fake ledger.
mod native_people_directory_absent_startup {
    use super::*;
    use sqlx::{Connection, PgConnection};

    // Reuse reviewed legacy-receipt-migration-helper-proposal-v1.rs construction;
    // only the App executable's embedded source path differs.
    static RECEIPT_CURRENT_MIGRATOR: sqlx::migrate::Migrator =
        sqlx::migrate!("../crates/platform/db/migrations");
    static HISTORICAL_RECEIPT_MIGRATOR: std::sync::LazyLock<sqlx::migrate::Migrator> =
        std::sync::LazyLock::new(|| {
            let migrations: Vec<_> = RECEIPT_CURRENT_MIGRATOR
                .iter()
                .filter(|m| m.version <= 229)
                .cloned()
                .collect();
            assert_eq!(migrations.len(), 229);
            for (index, migration) in migrations.iter().enumerate() {
                assert_eq!(migration.version, index as i64 + 1);
            }
            sqlx::migrate::Migrator::with_migrations(migrations)
        });
    const LEDGER: &str = include_str!("../../../../ops/account-custody-migrations.sha384");
    const CAPTURE: &str =
        include_str!("../../../../ops/postgres-capture-native-people-directory-custody.sql");
    const PREDECESSOR: &str =
        include_str!("../../../../ops/postgres-native-company-policy-v2-custody-state.sql");
    const ACCOUNT: &str = include_str!("../../../../ops/postgres-finalize-account-custody.sql");
    const CREDENTIAL: &str =
        include_str!("../../../../ops/postgres-finalize-account-credentials.sql");
    const COMPANY: &str = include_str!("../../../../ops/postgres-finalize-company-enrollment.sql");
    const POLICY_V2: &str =
        include_str!("../../../../ops/postgres-finalize-native-company-policy-v2.sql");
    const MISMATCH: &str = "native_people_directory.profile_mismatch";

    async fn historical229_predecessor(pool: &PgPool) {
        let fresh:bool=sqlx::query_scalar("SELECT session_user=current_user AND session_user='console_buck_admin' AND starts_with(current_database(),'_sqlx_test_') AND current_setting('console.sqlx_test_bootstrap',true)='buck-sqlx-superuser-v1' AND (SELECT rolsuper FROM pg_catalog.pg_roles WHERE rolname=current_user) AND NOT EXISTS(SELECT 1 FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname !~ '^pg_' AND n.nspname<>'information_schema')")
            .fetch_one(pool).await.unwrap();
        assert!(fresh, "requires genuinely empty marked disposable database");
        let prefix = LEDGER.lines().take(229).collect::<Vec<_>>().join("\n") + "\n";
        assert_eq!(
            hex::encode(Sha256::digest(prefix.as_bytes())),
            "6a7e45857de4892b314243679a3558b0b6f09122d838d484803dfc0a82455b27",
            "historical1–229 source ledger drift"
        );
        let expected: Vec<(i64, bool, Vec<u8>)> = prefix
            .lines()
            .enumerate()
            .map(|(i, line)| {
                let (version, checksum) = line.split_once('\t').unwrap();
                let version = version.parse::<i64>().unwrap();
                assert_eq!(version, i as i64 + 1);
                (version, true, hex::decode(checksum).unwrap())
            })
            .collect();
        for (migration, wanted) in HISTORICAL_RECEIPT_MIGRATOR.iter().zip(&expected) {
            assert_eq!(migration.version, wanted.0);
            assert_eq!(migration.checksum.as_ref(), wanted.2.as_slice());
            assert_eq!(
                sha2::Sha384::digest(migration.sql.as_str().as_bytes()).as_slice(),
                wanted.2.as_slice()
            );
        }
        let config = prepare_http_migration_config(pool).await;
        let database = pool.connect_options().get_database().unwrap().to_owned();
        let oid: i64 = sqlx::query_scalar(
            "SELECT oid::bigint FROM pg_catalog.pg_database WHERE datname=current_database()",
        )
        .fetch_one(pool)
        .await
        .unwrap();
        let mut owner = PgConnection::connect(config.database_url.as_deref().unwrap())
            .await
            .unwrap();
        let migrated=AssertUnwindSafe(async {
            sqlx::raw_sql("SET SESSION search_path=public,pg_temp; SET SESSION lock_timeout='5s'; SET SESSION statement_timeout='120s'")
                .execute(&mut owner).await.unwrap();
            let identity:bool=sqlx::query_scalar("SELECT session_user='console_app' AND current_user='console_app' AND current_database()=$1 AND d.oid::bigint=$2 AND pg_catalog.pg_get_userbyid(d.datdba)='console_app' AND r.rolcanlogin AND r.rolinherit AND NOT r.rolsuper AND r.rolbypassrls AND NOT r.rolcreatedb AND NOT r.rolcreaterole AND NOT r.rolreplication FROM pg_catalog.pg_roles r CROSS JOIN pg_catalog.pg_database d WHERE r.rolname=current_user AND d.datname=current_database()")
                .bind(&database).bind(oid).fetch_one(&mut owner).await.unwrap();
            assert!(identity,"actual migration owner LOGIN/target mismatch");
            HISTORICAL_RECEIPT_MIGRATOR.run(&mut owner).await.expect("real SQLx229 migrations; failure is prerequisite failure");
            console_platform_jobs::migrate_and_reconcile_apalis_postgres(&mut owner).await.unwrap();
            let actual:Vec<(i64,bool,Vec<u8>)>=sqlx::query_as("SELECT version,success,checksum FROM public._sqlx_migrations ORDER BY version")
                .fetch_all(&mut owner).await.unwrap();
            assert_eq!(actual,expected,"real produced229 ledger mismatch");
        }).catch_unwind().await;
        owner.close().await.unwrap();
        if let Err(panic) = migrated {
            std::panic::resume_unwind(panic);
        }
        let mut tx = pool.begin().await.unwrap();
        sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL READ COMMITTED; SET LOCAL search_path=pg_catalog,pg_temp; SET LOCAL lock_timeout='5s'; SET LOCAL statement_timeout='120s'")
            .execute(tx.as_mut()).await.unwrap();
        for (source, pin) in [
            (
                ACCOUNT,
                "fb532ef3b82f6d03b58d6e164a26567683039444098cf1a0a98d33e27e5dd4f8",
            ),
            (
                CREDENTIAL,
                "2f960163c9bd8832cdd3a058a6ae69d7503cf1009c476ef292af9443036624d9",
            ),
            (
                COMPANY,
                "bc35b52d5692e474a3c890dde73112f56e7b85a241ab390075d58b5d5b43a92f",
            ),
            (
                POLICY_INSTALLER,
                "4e7fc41b1d2ed6c2155d44d43347c18815ed9e70611eb9285f21e1c590bf996a",
            ),
            (
                POLICY_V2,
                "ec945607e209b93843116ae2b2a20772797dce38ff7884fb96081f09651f7d8e",
            ),
        ] {
            assert_eq!(
                hex::encode(Sha256::digest(source.as_bytes())),
                pin,
                "historical production bootstrap source changed"
            );
            sqlx::raw_sql(sqlx::AssertSqlSafe(source))
                .execute(tx.as_mut())
                .await
                .expect("actual historical production finalizer prerequisite");
            sqlx::raw_sql("SET CONSTRAINTS ALL IMMEDIATE")
                .execute(tx.as_mut())
                .await
                .unwrap();
        }
        tx.commit().await.unwrap();
    }
    async fn metadata(pool: &PgPool) -> (Value, String, Option<bool>) {
        let mut tx = pool.begin().await.unwrap();
        sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
            .execute(tx.as_mut())
            .await
            .unwrap();
        sqlx::raw_sql(CLASSIFIER_SESSION)
            .execute(tx.as_mut())
            .await
            .unwrap();
        let snapshot = sqlx::query_as(sqlx::AssertSqlSafe(CAPTURE))
            .fetch_one(tx.as_mut())
            .await
            .expect("capture on genuine absent schema must execute");
        tx.rollback().await.unwrap();
        snapshot
    }
    async fn operator_change(pool: &PgPool, source: &'static str) {
        let mut tx = pool.begin().await.unwrap();
        sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL READ COMMITTED; SET LOCAL search_path=pg_catalog,pg_temp; SET LOCAL lock_timeout='1s'; SET LOCAL statement_timeout='30s'").execute(tx.as_mut()).await.unwrap();
        let marked:bool=sqlx::query_scalar("SELECT session_user=current_user AND session_user='console_buck_admin' AND starts_with(current_database(),'_sqlx_test_') AND current_setting('console.sqlx_test_bootstrap',true)='buck-sqlx-superuser-v1' AND (SELECT rolsuper FROM pg_catalog.pg_roles WHERE rolname=current_user)").fetch_one(tx.as_mut()).await.unwrap();
        assert!(marked, "metadata fault requires marked disposable operator");
        sqlx::raw_sql(sqlx::AssertSqlSafe(source))
            .execute(tx.as_mut())
            .await
            .unwrap();
        tx.commit().await.unwrap();
    }
    #[sqlx::test(migrations = false)]
    async fn genuine_pre230_reserved_namespace_drift_cannot_fall_back_to_old_profile(pool: PgPool) {
        assert_eq!(
            hex::encode(Sha256::digest(CAPTURE.as_bytes())),
            "bc8a1f87f57cd676ca1a3deae12263b1cca4a290c71a12e9a1749d189e090ca7"
        );
        assert_eq!(
            hex::encode(Sha256::digest(PREDECESSOR.as_bytes())),
            "e507d75f446ad8d3e0befe9321d94731a1a2cc9b3e0e9c0306c1459a098a53cc"
        );
        historical229_predecessor(&pool).await;
        let artifacts = Artifacts::new();
        let key = SigningKey::random(&mut OsRng);
        let config = account_browser_config(&pool, artifacts.root.clone(), &key);
        let held = AppState::from_config(config.clone())
            .await
            .expect("actual229 predecessor must start before corruption");
        let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
        let outcome=AssertUnwindSafe(async {
            let absent:bool=sqlx::query_scalar("SELECT to_regclass('public.native_people_inputs_v1') IS NULL AND to_regclass('public.native_people_terminals_v1') IS NULL AND NOT EXISTS(SELECT 1 FROM public._sqlx_migrations WHERE version>229) AND NOT EXISTS(SELECT 1 FROM pg_catalog.pg_attribute a JOIN pg_catalog.pg_class c ON c.oid=a.attrelid JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='public' AND a.attnum>0 AND NOT a.attisdropped AND ((c.relname IN ('person_revisions','employee_person_bindings','ont_action_command_receipts') AND a.attname IN ('actor_kind','actor_account_id')) OR (c.relname='employees' AND a.attname IN ('source_kind','native_command_id'))))")
                .fetch_one(&pool).await.unwrap();assert!(absent,"fixture must be genuinely prior to230");
            assert_eq!(classified(&runtime,PREDECESSOR).await,"native_company_policy_v2.finalized");
            assert_eq!(ready_status(&held).await,StatusCode::OK);
            let baseline=metadata(&pool).await;let rows=all_rows(&pool).await;
            for (name,mutation,witness,restore) in [
                ("unknown_prefixed_constraint","ALTER TABLE public.employees ADD CONSTRAINT native_people_absent_probe_v1 CHECK(true)","SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_constraint WHERE conrelid='public.employees'::regclass AND conname='native_people_absent_probe_v1')","ALTER TABLE public.employees DROP CONSTRAINT native_people_absent_probe_v1"),
                ("reserved_canonical_constraint","ALTER TABLE public.employees ADD CONSTRAINT employees_native_intake_v1 CHECK(true)","SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_constraint WHERE conrelid='public.employees'::regclass AND conname='employees_native_intake_v1')","ALTER TABLE public.employees DROP CONSTRAINT employees_native_intake_v1"),
                ("unknown_prefixed_trigger","CREATE TRIGGER native_people_absent_probe_v1 BEFORE UPDATE ON public.employees FOR EACH ROW EXECUTE FUNCTION public.canonical_person_row_immutable()","SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_trigger WHERE tgrelid='public.employees'::regclass AND tgname='native_people_absent_probe_v1')","DROP TRIGGER native_people_absent_probe_v1 ON public.employees"),
                ("reserved_native_policy","CREATE POLICY native_people_owner_v1 ON public.employees TO console_account_owner USING(true)","SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_policy WHERE polrelid='public.employees'::regclass AND polname='native_people_owner_v1')","DROP POLICY native_people_owner_v1 ON public.employees"),
                ("unknown_prefixed_index","CREATE INDEX native_people_absent_probe_v1 ON public.employees(id)","SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_index i JOIN pg_catalog.pg_class c ON c.oid=i.indexrelid WHERE i.indrelid='public.employees'::regclass AND c.relname='native_people_absent_probe_v1')","DROP INDEX public.native_people_absent_probe_v1"),
            ] {
                operator_change(&pool,mutation).await;
                let checked=AssertUnwindSafe(async {
                    let visible:bool=sqlx::query_scalar(sqlx::AssertSqlSafe(witness)).fetch_one(&pool).await.unwrap();assert!(visible,"missing actual fault {name}");
                    assert_ne!(metadata(&pool).await,baseline,"complete capture omitted {name}");
                    assert_eq!(classified(&runtime,PREDECESSOR).await,"native_company_policy_v2.finalized","original fallback must remain independently valid");
                    startup_refused(config.clone(),MISMATCH).await;
                    assert_eq!(ready_status(&held).await,StatusCode::SERVICE_UNAVAILABLE);
                }).catch_unwind().await;
                operator_change(&pool,restore).await;
                assert_eq!(metadata(&pool).await,baseline,"restoration metadata drift {name}");
                assert!(all_rows(&pool).await==rows,"corruption/startup wrote rows");
                assert_eq!(ready_status(&held).await,StatusCode::OK);
                let fresh=AppState::from_config(config.clone()).await.unwrap();
                let positive=AssertUnwindSafe(async {assert_eq!(ready_status(&fresh).await,StatusCode::OK);}).catch_unwind().await;
                close_states(&[fresh],positive).await;
                assert_eq!(metadata(&pool).await,baseline);assert!(all_rows(&pool).await==rows);
                if let Err(panic)=checked {std::panic::resume_unwind(panic);}
            }
        }).catch_unwind().await;
        runtime.close().await;
        close_states(&[held], outcome).await;
    }
}
