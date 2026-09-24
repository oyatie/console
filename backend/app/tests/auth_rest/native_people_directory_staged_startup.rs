// Include inside native_policy_startup_tests after independent review.
// Prerequisite-blocked until actual all230/original bootstrap succeeds.
// No future classifier symbol or fabricated fingerprint is required to compile.
mod native_people_directory_staged_startup {
    use super::*;

    const CAPTURE: &str =
        include_str!("../../../../ops/postgres-capture-native-people-directory-custody.sql");
    const PREDECESSOR: &str =
        include_str!("../../../../ops/postgres-native-company-policy-v2-custody-state.sql");
    const MISMATCH: &str = "native_people_directory.profile_mismatch";

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
        let value = sqlx::query_as(sqlx::AssertSqlSafe(CAPTURE))
            .fetch_one(tx.as_mut())
            .await
            .expect("actual metadata capture must work before any admission inference");
        tx.rollback().await.unwrap();
        value
    }
    async fn operator_change(pool: &PgPool, source: &'static str) {
        let mut tx = pool.begin().await.unwrap();
        sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL READ COMMITTED; SET LOCAL search_path=pg_catalog,pg_temp; SET LOCAL lock_timeout='1s'; SET LOCAL statement_timeout='30s'")
            .execute(tx.as_mut()).await.unwrap();
        let marked: bool = sqlx::query_scalar("SELECT session_user=current_user AND session_user='console_buck_admin' AND starts_with(current_database(),'_sqlx_test_') AND current_setting('console.sqlx_test_bootstrap',true)='buck-sqlx-superuser-v1' AND (SELECT rolsuper FROM pg_catalog.pg_roles WHERE rolname=current_user)")
            .fetch_one(tx.as_mut()).await.unwrap();
        assert!(
            marked,
            "negative metadata injection requires marked disposable operator database"
        );
        sqlx::raw_sql(sqlx::AssertSqlSafe(source))
            .execute(tx.as_mut())
            .await
            .unwrap();
        tx.commit().await.unwrap();
    }

    #[sqlx::test(migrations = false)]
    async fn directory_closed_metadata_is_required_by_actual_startup_even_when_old_profile_matches(
        pool: PgPool,
    ) {
        assert_eq!(
            hex::encode(Sha256::digest(CAPTURE.as_bytes())),
            "bc8a1f87f57cd676ca1a3deae12263b1cca4a290c71a12e9a1749d189e090ca7",
            "reviewed capture source drift"
        );
        assert_eq!(
            hex::encode(Sha256::digest(PREDECESSOR.as_bytes())),
            "e507d75f446ad8d3e0befe9321d94731a1a2cc9b3e0e9c0306c1459a098a53cc",
            "historical classifier source drift"
        );
        // Existing fixture invokes actual current production migrations and all
        // original Account/credential/Company/policy finalizers. No new fixture
        // grants, shortened migration set or synthetic profile is permitted.
        let (app, key, held) =
            native_people_codec2_install_probe::configured_successor_fixture(&pool).await;
        let config = account_browser_config(&pool, app._artifacts.root.clone(), &key);
        let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
        let outcome = AssertUnwindSafe(async {
            let login: (String,String,bool,bool) = sqlx::query_as("SELECT session_user::text,current_user::text,rolsuper,rolbypassrls FROM pg_catalog.pg_roles WHERE rolname=current_user")
                .fetch_one(&runtime).await.unwrap();
            assert_eq!(login,("console_rt".into(),"console_rt".into(),false,false));
            assert_eq!(classified(&runtime,PREDECESSOR).await,"native_company_policy_v2.finalized");
            assert_eq!(ready_status(&held).await,StatusCode::OK);
            let baseline=metadata(&pool).await;
            let rows=all_rows(&pool).await;
            for (name, mutation, witness, restore) in [
                ("input_closed_gate",
                 "ALTER TABLE public.native_people_inputs_v1 DROP CONSTRAINT native_people_inputs_staged_closed_v1",
                 "SELECT NOT EXISTS(SELECT 1 FROM pg_catalog.pg_constraint WHERE conrelid='public.native_people_inputs_v1'::regclass AND conname='native_people_inputs_staged_closed_v1')",
                 "ALTER TABLE public.native_people_inputs_v1 ADD CONSTRAINT native_people_inputs_staged_closed_v1 CHECK(false)"),
                ("terminal_closed_gate",
                 "ALTER TABLE public.native_people_terminals_v1 DROP CONSTRAINT native_people_terminals_staged_closed_v1",
                 "SELECT NOT EXISTS(SELECT 1 FROM pg_catalog.pg_constraint WHERE conrelid='public.native_people_terminals_v1'::regclass AND conname='native_people_terminals_staged_closed_v1')",
                 "ALTER TABLE public.native_people_terminals_v1 ADD CONSTRAINT native_people_terminals_staged_closed_v1 CHECK(false)"),
                ("runtime_input_select",
                 "GRANT SELECT ON public.native_people_inputs_v1 TO console_rt",
                 "SELECT has_table_privilege('console_rt','public.native_people_inputs_v1','SELECT')",
                 "REVOKE SELECT ON public.native_people_inputs_v1 FROM console_rt"),
                ("unknown_native_relation",
                 "CREATE TABLE public.native_people_startup_probe_v1 (unexpected integer)",
                 "SELECT to_regclass('public.native_people_startup_probe_v1') IS NOT NULL",
                 "DROP TABLE public.native_people_startup_probe_v1"),
                ("unknown_native_routine",
                 "CREATE FUNCTION public.native_people_startup_probe_v1() RETURNS integer LANGUAGE sql IMMUTABLE AS 'SELECT 1'",
                 "SELECT to_regprocedure('public.native_people_startup_probe_v1()') IS NOT NULL",
                 "DROP FUNCTION public.native_people_startup_probe_v1()"),
            ] {
                operator_change(&pool, mutation).await;
                let checked = AssertUnwindSafe(async {
                    let visible:bool=sqlx::query_scalar(sqlx::AssertSqlSafe(witness)).fetch_one(&pool).await.unwrap();
                    assert!(visible,"fault did not reach actual metadata: {name}");
                    assert_ne!(metadata(&pool).await,baseline,"capture omitted actual fault {name}");
                    assert_eq!(classified(&runtime,PREDECESSOR).await,"native_company_policy_v2.finalized","old classifier must still be a real unsafe fallback positive control");
                    startup_refused(config.clone(),MISMATCH).await;
                    assert_eq!(ready_status(&held).await,StatusCode::SERVICE_UNAVAILABLE);
                }).catch_unwind().await;
                // Each negative is independently reversible. Restore before
                // propagating panic, then prove complete metadata/row equality.
                operator_change(&pool,restore).await;
                assert_eq!(metadata(&pool).await,baseline,"negative restore changed metadata: {name}");
                assert!(all_rows(&pool).await==rows,"negative startup touched durable rows");
                assert_eq!(ready_status(&held).await,StatusCode::OK);
                let fresh=AppState::from_config(config.clone()).await.unwrap();
                let positive=AssertUnwindSafe(async {assert_eq!(ready_status(&fresh).await,StatusCode::OK);}).catch_unwind().await;
                close_states(&[fresh],positive).await;
                assert_eq!(metadata(&pool).await,baseline,"restored fresh startup changed metadata");
                assert!(all_rows(&pool).await==rows,"restored fresh startup changed durable rows");
                if let Err(panic)=checked {std::panic::resume_unwind(panic);}
            }
        }).catch_unwind().await;
        // Dedicated observer pool closes before close_states may resume panic.
        runtime.close().await;
        close_states(&[held], outcome).await;
    }
}
