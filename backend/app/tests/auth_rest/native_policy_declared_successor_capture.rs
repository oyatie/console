// Additive proposal child module in native_policy_startup_tests.
// Prerequisite capture only. Exact declared artifacts are pinned before database
// mutation; mounted Buck inputs still need independent review before execution.
mod native_policy_declared_successor_capture {
    use super::*;
    use sqlx::{Postgres, Transaction};

    const SOURCE: &str =
        include_str!("../../../../ops/postgres-native-company-policy-v2-owner.sql");
    const CAPTURE: &str =
        include_str!("../../../../ops/postgres-capture-native-company-policy-v2-custody.sql");
    const OLD_CAPTURE: &str =
        include_str!("../../../../ops/postgres-capture-native-company-policy-custody.sql");
    const OBSERVER: &str = include_str!("../../../../ops/postgres-install-durability-observer.sql");
    const PREDECESSORS: [&str; 2] = [
        "3755792f52a4f78236a70e509f4f1546588049daa53f7545d8a101c74684d843",
        "9bde6410d51f8b665e5ca4400820e6d8a549a6c19b7b4d84feb26ef32c4bd604",
    ];

    async fn capture(
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
            .expect("declared metadata capture must execute; SQL failure is not a profile")
    }
    fn digest(source: &str) -> String {
        hex::encode(Sha256::digest(source.as_bytes()))
    }

    #[sqlx::test(migrations = false)]
    async fn capture_declared_dual_codec_metadata_from_both_exact_predecessors(pool: PgPool) {
        let marked: bool = sqlx::query_scalar(
            "SELECT session_user=current_user AND session_user='console_buck_admin' AND starts_with(current_database(),'_sqlx_test_') AND current_setting('console.sqlx_test_bootstrap',true)='buck-sqlx-superuser-v1' AND (SELECT rolsuper FROM pg_catalog.pg_roles WHERE rolname=current_user) AND to_regclass('public._sqlx_migrations') IS NULL")
            .fetch_one(&pool).await.unwrap();
        assert!(
            marked,
            "requires marked fresh disposable SQLx operator database"
        );
        for (name, source, expected) in [
            (
                "source",
                SOURCE,
                "f2050f21ef8151339289b2f013abdd543e90f8803fb6f6d4b9f004abf4409602",
            ),
            (
                "capture",
                CAPTURE,
                "7534375fbae287ccf5e5015815e788ef0d7a379eed623af9a72f5db0d89614b4",
            ),
            (
                "old capture",
                OLD_CAPTURE,
                "d5d2c45c691383ddde9ac213d9fe2674db0750f3ceabdc733eaaf2a570d937e3",
            ),
            (
                "observer",
                OBSERVER,
                "ffe7038b43de0207d6ae3e3ce0487498edc4d061c21e4249abacc91abaacdbbc",
            ),
        ] {
            assert_eq!(digest(source), expected, "unreviewed declared {name} bytes");
        }
        // Genuine current numbered migrations, Account/Company finalizers and
        // exact legacy policy installer/classifier pins. No business row seeds
        // beyond the existing fixture's reviewed terms prerequisite.
        prepare_policy_ready_database(&pool).await;
        let before = all_rows(&pool).await;
        let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
        let login: (String, String, bool, bool) = sqlx::query_as(
            "SELECT session_user::text,current_user::text,rolsuper,rolbypassrls FROM pg_roles WHERE rolname=current_user")
            .fetch_one(&runtime).await.unwrap();
        assert_eq!(
            login,
            ("console_rt".into(), "console_rt".into(), false, false)
        );
        assert_eq!(
            policy_classified(&runtime).await,
            "native_company_policy.finalized"
        );
        let mut baseline_tx = pool.begin().await.unwrap();
        let baseline_metadata = capture(&mut baseline_tx, OLD_CAPTURE).await;
        baseline_tx.rollback().await.unwrap();
        assert_eq!(baseline_metadata.1, PREDECESSORS[0]);
        assert_eq!(baseline_metadata.2, Some(true));
        // Runtime is deliberately idle while capture transaction holds DDL locks.
        let mut captures = Vec::new();
        let result = AssertUnwindSafe(async {
            for (variant, predecessor) in PREDECESSORS.iter().enumerate() {
                let mut tx = pool.begin().await.unwrap();
                let result = AssertUnwindSafe(async {
                    sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL READ COMMITTED; SET LOCAL search_path=pg_catalog,pg_temp; SET LOCAL lock_timeout='1s'; SET LOCAL statement_timeout='120s'")
                        .execute(tx.as_mut()).await.unwrap();
                    let observer_absent: bool = sqlx::query_scalar(
                        "SELECT NOT EXISTS(SELECT 1 FROM pg_catalog.pg_roles WHERE rolname='console_durability_observer') AND NOT EXISTS(SELECT 1 FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace WHERE n.nspname='public' AND p.proname='console_durability_observation_v1')")
                        .fetch_one(tx.as_mut()).await.unwrap();
                    assert!(observer_absent,"plain prerequisite/previous rollback must have no observer role or routine");
                    let plain = capture(&mut tx,OLD_CAPTURE).await;
                    assert_eq!(plain,baseline_metadata,"plain metadata changed before source execution");
                    if variant==1 {
                        // Actual unchanged optional observer installer; rollback
                        // includes its cluster role, memberships and native ACL.
                        sqlx::raw_sql(OBSERVER).execute(tx.as_mut()).await
                            .expect("actual optional observer installer prerequisite");
                    }
                    let old = capture(&mut tx,OLD_CAPTURE).await;
                    assert_eq!(&old.1,predecessor,"exact predecessor required before candidate source");
                    assert_eq!(old.2,Some(true));
                    // This is declared-source capture on a disposable DB, not a
                    // second shipping installer or finalizer/profile acceptance.
                    // Read-only classifier settings deliberately bound capture to
                    // 3 seconds. Restore the declared DDL budget for source install.
                    sqlx::raw_sql("SET LOCAL statement_timeout='120s'")
                        .execute(tx.as_mut()).await.unwrap();
                    sqlx::raw_sql(SOURCE).execute(tx.as_mut()).await
                        .expect("exact reviewed declared successor source must install for capture");
                    sqlx::raw_sql("SET CONSTRAINTS ALL IMMEDIATE").execute(tx.as_mut()).await.unwrap();
                    let observed = capture(&mut tx,CAPTURE).await;
                    assert_eq!(observed.2,Some(true),"effective startup rights failed");
                    assert_eq!(observed.1.len(),64);
                    assert!(observed.1.bytes().all(|b|b.is_ascii_digit() || (b'a'..=b'f').contains(&b)));
                    assert!(!PREDECESSORS.contains(&observed.1.as_str()));
                    assert_ne!(observed.0,old.0,"successor capture omitted all metadata changes");
                    let again = capture(&mut tx,CAPTURE).await;
                    assert_eq!(observed,again,"identical capture is unstable inside retained transaction");
                    json!({"variant":if variant==0 {"plain"} else {"observer"},
                        "predecessor_sha256":old.1,"successor_sha256":observed.1,
                        "startup_rights_valid":observed.2,"snapshot":observed.0,
                        "source_sha256":digest(SOURCE),"capture_sha256":digest(CAPTURE),
                        "old_capture_sha256":digest(OLD_CAPTURE),"observer_source_sha256":digest(OBSERVER)})
                }).catch_unwind().await;
                tx.rollback().await.expect("capture transaction and optional observer must roll back");
                assert!(before==all_rows(&pool).await,"capture left persistent business rows");
                let absent: bool = sqlx::query_scalar(
                    "SELECT NOT EXISTS(SELECT 1 FROM pg_catalog.pg_roles WHERE rolname='console_durability_observer') AND NOT EXISTS(SELECT 1 FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace WHERE n.nspname='public' AND p.proname IN ('console_durability_observation_v1','native_company_policy_codec_v2'))")
                    .fetch_one(&pool).await.unwrap();
                assert!(absent,"capture rollback left new role/routine");
                assert_eq!(policy_classified(&runtime).await,"native_company_policy.finalized");
                let mut restored_tx = pool.begin().await.unwrap();
                let restored_metadata = capture(&mut restored_tx, OLD_CAPTURE).await;
                restored_tx.rollback().await.unwrap();
                assert_eq!(restored_metadata,baseline_metadata,"rollback changed original metadata or startup rights");
                match result { Ok(value)=>captures.push(value),Err(panic)=>std::panic::resume_unwind(panic) }
            }
            assert_eq!(captures.len(),2);
            assert_ne!(captures[0]["successor_sha256"],captures[1]["successor_sha256"]);
            for value in &captures { eprintln!("NATIVE_POLICY_DECLARED_CAPTURE {value}"); }
            eprintln!("NATIVE_POLICY_DECLARED_CAPTURE_COMPLETE variants=2 rollback=verified acceptance=not_claimed");
        }).catch_unwind().await;
        runtime.close().await;
        if let Err(panic) = result {
            std::panic::resume_unwind(panic);
        }
    }
}
