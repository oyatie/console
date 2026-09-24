// Child of native_policy_startup_tests. Captures only; no new serving profile.
mod native_people_directory_declared_capture {
    use super::*;
    use sqlx::{Acquire, Postgres, Transaction};

    const SOURCE: &str = include_str!("../../../../ops/postgres-native-people-directory-owner.sql");
    const CAPTURE: &str =
        include_str!("../../../../ops/postgres-capture-native-people-directory-custody.sql");
    const CLOSED: &str =
        include_str!("../../../../ops/postgres-capture-native-people-directory-staged-custody.sql");
    const MIGRATION: &str = include_str!(
        "../../../crates/platform/db/migrations/0230_native_people_directory_storage.sql"
    );
    const OLD_CAPTURE: &str =
        include_str!("../../../../ops/postgres-capture-native-company-policy-v2-custody.sql");
    const OBSERVER: &str = include_str!("../../../../ops/postgres-install-durability-observer.sql");
    const ACCOUNT: &str = include_str!("../../../../ops/postgres-finalize-account-custody.sql");
    const CREDENTIAL: &str =
        include_str!("../../../../ops/postgres-finalize-account-credentials.sql");
    const COMPANY: &str = include_str!("../../../../ops/postgres-finalize-company-enrollment.sql");
    const POLICY_V2: &str =
        include_str!("../../../../ops/postgres-finalize-native-company-policy-v2.sql");
    const PREDECESSORS: [&str; 2] = [
        "e94251d48fdee392d7632709b19d80cc04d53ea6742e0f2d6538d68391ab8bf2",
        "f132054a56641dcb0cc5875d6485846df2091e631d0d902a18356d2202343fd9",
    ];
    fn digest(source: &str) -> String {
        hex::encode(Sha256::digest(source.as_bytes()))
    }
    fn reviewed_sources() {
        for (name, source, expected) in [
            (
                "source",
                SOURCE,
                "86ab1703f6a236952764f048aed3dd879daa44daa9c0abfe1a1c7a8501efb92a",
            ),
            (
                "capture",
                CAPTURE,
                "bc8a1f87f57cd676ca1a3deae12263b1cca4a290c71a12e9a1749d189e090ca7",
            ),
            (
                "closed capture",
                CLOSED,
                "890ebc034fe9836f45e26c13d05352085db753f17e4a72b1ec05b6ecbe0a0e36",
            ),
            (
                "migration",
                MIGRATION,
                "a48173dbbd28d2c8e1940d2dcf337b6b74ca07bc5b3a8cbf194b28a4b78eebed",
            ),
            (
                "old capture",
                OLD_CAPTURE,
                "7534375fbae287ccf5e5015815e788ef0d7a379eed623af9a72f5db0d89614b4",
            ),
            (
                "observer",
                OBSERVER,
                "ffe7038b43de0207d6ae3e3ce0487498edc4d061c21e4249abacc91abaacdbbc",
            ),
            (
                "Account",
                ACCOUNT,
                "fb532ef3b82f6d03b58d6e164a26567683039444098cf1a0a98d33e27e5dd4f8",
            ),
            (
                "credential",
                CREDENTIAL,
                "2f960163c9bd8832cdd3a058a6ae69d7503cf1009c476ef292af9443036624d9",
            ),
            (
                "Company",
                COMPANY,
                "bc35b52d5692e474a3c890dde73112f56e7b85a241ab390075d58b5d5b43a92f",
            ),
            (
                "policy v1",
                POLICY_INSTALLER,
                "4e7fc41b1d2ed6c2155d44d43347c18815ed9e70611eb9285f21e1c590bf996a",
            ),
            (
                "policy v2",
                POLICY_V2,
                "ec945607e209b93843116ae2b2a20772797dce38ff7884fb96081f09651f7d8e",
            ),
        ] {
            assert_eq!(digest(source), expected, "unreviewed {name} bytes");
        }
    }
    async fn closed(tx: &mut Transaction<'_, Postgres>) -> (Value, String) {
        sqlx::raw_sql(CLASSIFIER_SESSION)
            .execute(tx.as_mut())
            .await
            .unwrap();
        sqlx::query_as(sqlx::AssertSqlSafe(CLOSED))
            .fetch_one(tx.as_mut())
            .await
            .expect("actual closed extension capture must execute; SQL error is not a profile")
    }
    async fn full(
        tx: &mut Transaction<'_, Postgres>,
        source: &'static str,
    ) -> (Value, String, Option<bool>) {
        sqlx::raw_sql(CLASSIFIER_SESSION)
            .execute(tx.as_mut())
            .await
            .unwrap();
        sqlx::query_as(sqlx::AssertSqlSafe(source))
            .fetch_one(tx.as_mut())
            .await
            .expect("actual complete capture must execute; SQL error is not a profile")
    }
    async fn execute(tx: &mut Transaction<'_, Postgres>, source: &'static str) {
        sqlx::raw_sql("SET LOCAL search_path=pg_catalog,pg_temp; SET LOCAL statement_timeout='120s'; SET LOCAL lock_timeout='5s'; SET LOCAL jit=off")
            .execute(tx.as_mut()).await.unwrap();
        sqlx::raw_sql(sqlx::AssertSqlSafe(source))
            .execute(tx.as_mut())
            .await
            .expect("unchanged production bootstrap or reviewed capture source must execute");
        sqlx::raw_sql("SET CONSTRAINTS ALL IMMEDIATE")
            .execute(tx.as_mut())
            .await
            .unwrap();
    }
    async fn observer_absent(tx: &mut Transaction<'_, Postgres>) {
        let absent: bool = sqlx::query_scalar("SELECT NOT EXISTS(SELECT 1 FROM pg_catalog.pg_roles WHERE rolname='console_durability_observer') AND NOT EXISTS(SELECT 1 FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace WHERE n.nspname='public' AND p.proname='console_durability_observation_v1')")
            .fetch_one(tx.as_mut()).await.unwrap();
        assert!(
            absent,
            "dedicated capture cluster requires genuinely absent observer"
        );
    }
    async fn capture_detects_added_namespace(
        tx: &mut Transaction<'_, Postgres>,
        original: &(Value, String, Option<bool>),
        extension: &(Value, String),
    ) {
        for mutation in [
            "CREATE TABLE public.native_people_capture_corruption_probe_v1 (unexpected integer)",
            "CREATE FUNCTION public.native_people_capture_corruption_probe_v1() RETURNS integer LANGUAGE sql IMMUTABLE AS 'SELECT 1'",
        ] {
            let mut probe = tx.begin().await.unwrap();
            execute(&mut probe, mutation).await;
            let changed_full = full(&mut probe, CAPTURE).await;
            let changed_closed = closed(&mut probe).await;
            assert_ne!(
                changed_full.0, original.0,
                "complete capture omitted actual unexpected metadata"
            );
            assert_ne!(changed_full.1, original.1);
            assert_ne!(
                changed_closed.0, extension.0,
                "extension capture omitted native namespace drift"
            );
            assert_ne!(changed_closed.1, extension.1);
            probe.rollback().await.unwrap();
            assert_eq!(
                &full(tx, CAPTURE).await,
                original,
                "corruption rollback changed complete baseline"
            );
            assert_eq!(
                &closed(tx).await,
                extension,
                "corruption rollback changed extension baseline"
            );
        }
    }

    #[sqlx::test(migrations = false)]
    async fn all230_bootstrap_preserves_closed_extension_and_captures_declared_directory_both_variants(
        pool: PgPool,
    ) {
        reviewed_sources();
        let marked: bool = sqlx::query_scalar("SELECT session_user=current_user AND session_user='console_buck_admin' AND starts_with(current_database(),'_sqlx_test_') AND current_setting('console.sqlx_test_bootstrap',true)='buck-sqlx-superuser-v1' AND (SELECT rolsuper FROM pg_catalog.pg_roles WHERE rolname=current_user) AND to_regclass('public._sqlx_migrations') IS NULL")
            .fetch_one(&pool).await.unwrap();
        assert!(
            marked,
            "requires marked fresh disposable SQLx operator database"
        );
        // Real production migration entry, including Apalis. No alternate schema,
        // truncated migration set, synthesized ledger or business fixtures.
        prepare_http_database_staging(&pool).await;
        let ledger: (i64, bool, Vec<u8>) = sqlx::query_as(
            "SELECT version, success, checksum FROM public._sqlx_migrations WHERE version=230",
        )
        .fetch_one(&pool)
        .await
        .expect("actual current migration230 prerequisite");
        assert_eq!((ledger.0, ledger.1), (230, true));
        use sha2::Sha384;
        assert_eq!(ledger.2, Sha384::digest(MIGRATION.as_bytes()).to_vec());
        let before = all_rows(&pool).await;
        let mut baseline_tx = pool.begin().await.unwrap();
        observer_absent(&mut baseline_tx).await;
        let baseline_closed = closed(&mut baseline_tx).await;
        let baseline_full = full(&mut baseline_tx, CAPTURE).await;
        assert_eq!(
            baseline_closed.0["protocol"],
            "native_people_directory_closed230_v1"
        );
        assert_eq!(baseline_closed.0["tables"].as_array().unwrap().len(), 7);
        baseline_tx.rollback().await.unwrap();
        let mut captures = Vec::new();
        for (variant, predecessor) in PREDECESSORS.iter().enumerate() {
            let mut tx = pool.begin().await.unwrap();
            let result = AssertUnwindSafe(async {
                sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL READ COMMITTED; SET LOCAL lock_timeout='5s'; SET LOCAL statement_timeout='120s'; SET LOCAL search_path=pg_catalog,pg_temp")
                    .execute(tx.as_mut()).await.unwrap();
                observer_absent(&mut tx).await;
                assert_eq!(closed(&mut tx).await, baseline_closed);
                // Dedicated serialized capture coordinates actual cluster role DDL.
                // It does not grant serving capabilities or emulate an installer.
                let _: String = sqlx::query_scalar("SELECT oid::text FROM pg_catalog.pg_authid WHERE rolname='console_account_owner' FOR UPDATE")
                    .fetch_one(tx.as_mut()).await.unwrap();
                let mut stages = vec![json!({"stage":"all230", "closed_sha256":baseline_closed.1})];
                if variant == 1 {
                    execute(&mut tx, OBSERVER).await;
                    assert_eq!(closed(&mut tx).await, baseline_closed, "observer altered closed extension");
                    stages.push(json!({"stage":"observer", "closed_sha256":baseline_closed.1}));
                }
                for (name, source) in [("account", ACCOUNT), ("credentials", CREDENTIAL), ("company", COMPANY), ("policy_v1", POLICY_INSTALLER), ("policy_v2", POLICY_V2)] {
                    execute(&mut tx, source).await;
                    let observed = closed(&mut tx).await;
                    assert_eq!(observed, baseline_closed, "closed extension changed during original {name} bootstrap");
                    stages.push(json!({"stage":name, "closed_sha256":observed.1}));
                }
                let old = full(&mut tx, OLD_CAPTURE).await;
                assert_eq!(&old.1, predecessor, "all230 changed actual frozen policyv2 predecessor");
                assert_eq!(old.2, Some(true));
                let staged = full(&mut tx, CAPTURE).await;
                assert_eq!(staged.2, Some(true), "staged effective startup rights failed");
                assert_eq!(full(&mut tx, CAPTURE).await, staged);
                capture_detects_added_namespace(&mut tx, &staged, &baseline_closed).await;
                execute(&mut tx, SOURCE).await;
                let active = full(&mut tx, CAPTURE).await;
                let active_extension = closed(&mut tx).await;
                assert_eq!(active.2, Some(true), "activated effective startup rights failed");
                assert_ne!(active.0, staged.0, "complete capture omitted activation");
                assert_ne!(active.1, staged.1);
                assert_ne!(active_extension, baseline_closed, "closed capture failed to detect activation");
                assert_eq!(full(&mut tx, CAPTURE).await, active, "actual activated capture is unstable");
                capture_detects_added_namespace(&mut tx, &active, &active_extension).await;
                json!({"variant":if variant==0 {"plain"} else {"observer"},
                    "stages":stages,"predecessor_sha256":old.1,
                    "closed_sha256":baseline_closed.1,"closed_snapshot":baseline_closed.0,
                    "staged_sha256":staged.1,"staged_snapshot":staged.0,
                    "active_sha256":active.1,"active_snapshot":active.0,
                    "source_sha256":digest(SOURCE),"capture_sha256":digest(CAPTURE),
                    "closed_capture_sha256":digest(CLOSED),"migration_sha256":digest(MIGRATION),
                    "startup_rights_valid":active.2,"corruption_controls":4})
            }).catch_unwind().await;
            tx.rollback()
                .await
                .expect("all bootstrap/source/observer DDL must roll back");
            assert_eq!(
                all_rows(&pool).await,
                before,
                "capture changed persistent business rows or migration ledger"
            );
            let mut restored = pool.begin().await.unwrap();
            observer_absent(&mut restored).await;
            assert_eq!(
                closed(&mut restored).await,
                baseline_closed,
                "rollback changed closed extension metadata"
            );
            assert_eq!(
                full(&mut restored, CAPTURE).await,
                baseline_full,
                "rollback changed complete metadata"
            );
            restored.rollback().await.unwrap();
            match result {
                Ok(value) => captures.push(value),
                Err(panic) => std::panic::resume_unwind(panic),
            }
        }
        assert_eq!(captures.len(), 2);
        assert_ne!(captures[0]["staged_sha256"], captures[1]["staged_sha256"]);
        assert_ne!(captures[0]["active_sha256"], captures[1]["active_sha256"]);
        for value in &captures {
            eprintln!("NATIVE_PEOPLE_DECLARED_CAPTURE {value}");
        }
        eprintln!(
            "NATIVE_PEOPLE_DECLARED_CAPTURE_COMPLETE variants=2 closed_parity=verified rollback=verified corruption_controls=8 acceptance=not_claimed"
        );
    }
}
