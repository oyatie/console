// Include inside existing deployment_operator_designation::company_setup.
// Pure owner parser in a disposable incomplete profile: no serving enrollment.
mod input_parser {
    use super::*;
    use console_platform_test_support::{TestDatabaseLogin, login_test_pool};

    const INSTALL: &str = include_str!("../../../../ops/postgres-company-enrollment-input.sql");
    const ROOT_STATE: &str = include_str!("../../src/account_custody_state.sql");
    const CREDENTIAL_STATE: &str = include_str!("../../src/account_credential_custody_state.sql");
    const GOLDEN: &str = include_str!("fixtures/company-enrollment-input-golden.sql");
    const MALFORMED: &str = include_str!("fixtures/company-enrollment-input-malformed.sql");
    const BOUNDARY: &str = include_str!("fixtures/company-enrollment-input-boundary.sql");

    async fn assert_profiles(connection: &mut sqlx::PgConnection, root: &str, credentials: &str) {
        sqlx::raw_sql("SET search_path=pg_catalog,pg_temp")
            .execute(&mut *connection)
            .await
            .unwrap();
        for (source, expected) in [(ROOT_STATE, root), (CREDENTIAL_STATE, credentials)] {
            let actual: String = sqlx::query_scalar(source)
                .fetch_one(&mut *connection)
                .await
                .unwrap();
            assert_eq!(
                actual, expected,
                "exact existing classifier, without rewritten fingerprint"
            );
        }
    }

    async fn denied_parser(pool: &PgPool, expected_login: &str) {
        let identity:(String,String,bool,bool)=sqlx::query_as("SELECT session_user::text,current_user::text,(SELECT rolsuper FROM pg_roles WHERE rolname=current_user),(SELECT rolbypassrls FROM pg_roles WHERE rolname=current_user)")
            .fetch_one(pool).await.unwrap();
        assert_eq!(
            identity,
            (
                expected_login.to_owned(),
                expected_login.to_owned(),
                false,
                false
            )
        );
        let result =
            sqlx::query("SELECT * FROM public.company_enrollment_decode_input_v1(NULL::bytea)")
                .fetch_all(pool)
                .await;
        let error = result.expect_err("real serving login must not EXECUTE private decoder");
        assert_eq!(
            error.as_database_error().and_then(|e| e.code()).as_deref(),
            Some("42501")
        );
    }

    #[sqlx::test(migrations = false)]
    async fn company_input_parser_executes_710_cases_only_in_private_mismatched_profile(
        pool: PgPool,
    ) {
        let (_app, _account, _cookies, startup, _designation) = designated(&pool).await;
        let before = all_rows(&pool).await;
        let mut admin = pool.acquire().await.unwrap();
        let identity:(String,String,bool)=sqlx::query_as("SELECT session_user::text,current_user::text,current_setting('console.sqlx_test_bootstrap',true)='buck-sqlx-superuser-v1' AND (SELECT rolsuper FROM pg_roles WHERE rolname=current_user)")
            .fetch_one(&mut *admin).await.unwrap();
        assert_eq!(
            identity,
            (
                "console_buck_admin".into(),
                "console_buck_admin".into(),
                true
            )
        );
        assert_profiles(
            &mut admin,
            "account_custody.native_finalized",
            "account_credentials.native_finalized",
        )
        .await;
        let absent:bool=sqlx::query_scalar("SELECT NOT EXISTS(SELECT 1 FROM pg_proc WHERE pronamespace='public'::regnamespace AND proname='company_enrollment_decode_input_v1')")
            .fetch_one(&mut *admin).await.unwrap();
        assert!(
            absent,
            "pure helper must not be installed by current finalization"
        );
        // Root emits this one exact helper from its single generator owner.
        // The generated resource itself contains no transaction/finalizer hook.
        sqlx::raw_sql("BEGIN; SET LOCAL statement_timeout='10s'; SET LOCAL lock_timeout='1s'")
            .execute(&mut *admin)
            .await
            .unwrap();
        sqlx::raw_sql(INSTALL).execute(&mut *admin).await.unwrap();
        sqlx::raw_sql("COMMIT").execute(&mut *admin).await.unwrap();
        assert_profiles(
            &mut admin,
            "account_native.profile_mismatch",
            "account_native.profile_mismatch",
        )
        .await;
        let helper_exact:bool=sqlx::query_scalar(r#"
          SELECT count(*)=1 AND bool_and(
            p.proargtypes='17'::oidvector AND p.pronargs=1 AND p.pronargdefaults=0
            AND p.proallargtypes=ARRAY[17,21,2950,2950,2950,2950,25,25,17]::oid[]
            AND p.proargmodes=ARRAY['i','t','t','t','t','t','t','t','t']::"char"[]
            AND p.proargnames=ARRAY['p_input','codec_version','account_id','command_id','group_id','administrative_account_id','company_slug','company_name','input_digest']::text[]
            AND p.prokind='f' AND NOT p.prosecdef AND NOT p.proisstrict AND NOT p.proleakproof
            AND p.provolatile='i' AND p.proparallel='u' AND p.proretset
            AND p.prorettype='record'::regtype AND p.proowner='console_account_owner'::regrole
            AND p.prolang=(SELECT oid FROM pg_language WHERE lanname='plpgsql')
            AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp']::text[]
            AND (SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(a.grantor),
                CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
                a.privilege_type,a.is_grantable) ORDER BY a.grantee,a.privilege_type)
                FROM aclexplode(p.proacl) a)
                = '[["console_account_owner","console_account_owner","EXECUTE",false]]'::jsonb)
          FROM pg_proc p WHERE p.pronamespace='public'::regnamespace
            AND p.proname='company_enrollment_decode_input_v1'
        "#).fetch_one(&mut *admin).await.unwrap();
        assert!(
            helper_exact,
            "one exact private immutable parser and owner ACL required"
        );
        // Original DO oracles, all synthetic data; only psql client directives
        // removed and golden rejected-count assertion added. Each uses its own
        // bounded READ ONLY transaction ending ROLLBACK.
        for (source, expected_calls) in [(GOLDEN, 557usize), (MALFORMED, 24), (BOUNDARY, 129)] {
            sqlx::raw_sql(source).execute(&mut *admin).await.unwrap();
            eprintln!("company_input_parser_probe approved_calls={expected_calls}");
        }
        assert_profiles(
            &mut admin,
            "account_native.profile_mismatch",
            "account_native.profile_mismatch",
        )
        .await;
        for (login, name) in [
            (TestDatabaseLogin::Business, "console_rt"),
            (TestDatabaseLogin::Auth, "console_auth_rt"),
            (TestDatabaseLogin::LeaveCommand, "console_leave_cmd"),
            (TestDatabaseLogin::OntologyCommand, "console_ontology_cmd"),
            (
                TestDatabaseLogin::PlatformForceCommand,
                "console_platform_force_cmd",
            ),
        ] {
            let runtime = login_test_pool(&pool, login).await;
            denied_parser(&runtime, name).await;
            runtime.close().await;
        }
        denied_parser(&startup, "console_auth_startup").await;
        startup.close().await;
        drop(admin);
        assert!(
            before == all_rows(&pool).await,
            "pure decoder changed durable business/identity history"
        );
        eprintln!(
            "company_input_parser_probe parser_calls=710 serving_login_denials=6 sqlx_leaves=1"
        );
        // SQLx discards this database. Do not remove the helper and pretend this
        // incomplete candidate was a serving-profile upgrade or downgrade.
    }
}
