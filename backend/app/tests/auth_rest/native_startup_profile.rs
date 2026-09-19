// Included directly inside account_browser, outside the retained designation
// selector. Tests the real generated serving classifiers, not a copied hash map.
mod native_startup_profile {
    use super::*;
    use console_platform_test_support::login_test_pool;
    use sqlx::{Acquire, PgConnection, Postgres, Transaction};

    async fn classify(connection: &mut PgConnection) -> (String, String) {
        sqlx::raw_sql(include_str!("../../src/account_custody_session.sql"))
            .execute(&mut *connection)
            .await
            .unwrap();
        let root = sqlx::query_scalar(include_str!("../../src/account_custody_state.sql"))
            .fetch_one(&mut *connection)
            .await
            .expect("root serving classifier must execute successfully");
        let credentials = sqlx::query_scalar(include_str!(
            "../../src/account_credential_custody_state.sql"
        ))
        .fetch_one(&mut *connection)
        .await
        .expect("credential serving classifier must execute successfully");
        (root, credentials)
    }

    fn finalized((root, credentials): (String, String)) {
        assert_eq!(root, "account_custody.native_finalized");
        assert_eq!(credentials, "account_credentials.native_finalized");
    }

    async fn runtime_positive(runtime: &PgPool) {
        let mut tx = runtime.begin().await.unwrap();
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
            .execute(&mut *tx)
            .await
            .unwrap();
        let identity: (String, String) =
            sqlx::query_as("SELECT session_user::text,current_user::text")
                .fetch_one(&mut *tx)
                .await
                .unwrap();
        assert_eq!(identity, ("console_rt".into(), "console_rt".into()));
        finalized(classify(&mut tx).await);
        tx.commit().await.unwrap();
    }

    async fn reject_fault(
        tx: &mut Transaction<'_, Postgres>,
        name: &str,
        mutation: &'static str,
        witness: &'static str,
    ) {
        let mut fault = tx.begin().await.unwrap();
        sqlx::raw_sql(sqlx::AssertSqlSafe(mutation))
            .execute(&mut *fault)
            .await
            .expect("declared metadata fault must execute");
        let changed: bool = sqlx::query_scalar(sqlx::AssertSqlSafe(witness))
            .fetch_one(&mut *fault)
            .await
            .expect("independent direct catalog witness must execute");
        assert!(changed, "fault injection did not take effect: {name}");
        // Separate LOGINs cannot see uncommitted catalog faults. Evaluate the
        // identical generated read-only SQL on this administrator transaction;
        // runtime_positive separately verifies genuine serving LOGIN admission.
        let (root, credentials) = classify(&mut fault).await;
        assert_eq!(
            root, "account_native.profile_mismatch",
            "root accepted {name}"
        );
        assert_eq!(
            credentials, "account_native.profile_mismatch",
            "credentials accepted {name}"
        );
        fault.rollback().await.unwrap();
        finalized(classify(&mut *tx).await);
    }

    const PLAIN_FAULTS: &[(&str, &str, &str)] = &[
        (
            "builtin_public_execute",
            r##"GRANT EXECUTE ON FUNCTION pg_catalog.pg_control_system() TO PUBLIC;"##,
            r##"SELECT bool_and(value = 'true'::jsonb) AND count(*)=2 FROM jsonb_each((SELECT jsonb_build_object('explicit_public_grant',(EXISTS(SELECT 1 FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a WHERE p.oid='pg_catalog.pg_control_system()'::regprocedure AND a.grantee=0 AND a.privilege_type='EXECUTE' AND NOT a.is_grantable)),'startup_effective_execute',(has_function_privilege('console_auth_startup','pg_catalog.pg_control_system()'::regprocedure,'EXECUTE')))))"##,
        ),
        (
            "startup_indirect_table_grant",
            r##"CREATE ROLE native_snapshot_probe_parent NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS;
GRANT SELECT ON TABLE public.deployment_operator_receipts TO native_snapshot_probe_parent;
GRANT native_snapshot_probe_parent TO console_auth_startup WITH ADMIN FALSE, INHERIT TRUE, SET FALSE;"##,
            r##"SELECT bool_and(value = 'true'::jsonb) AND count(*)=3 FROM jsonb_each((SELECT jsonb_build_object('startup_effective_select',(has_table_privilege('console_auth_startup','public.deployment_operator_receipts','SELECT')),'no_direct_startup_grant',(NOT EXISTS(SELECT 1 FROM pg_class c CROSS JOIN LATERAL aclexplode(c.relacl) a WHERE c.oid='public.deployment_operator_receipts'::regclass AND a.grantee='console_auth_startup'::regrole)),'exact_inheritance_edge',(EXISTS(SELECT 1 FROM pg_auth_members m WHERE roleid='native_snapshot_probe_parent'::regrole AND member='console_auth_startup'::regrole AND NOT admin_option AND inherit_option AND NOT set_option)))))"##,
        ),
        (
            "partial_builtin_only",
            r##"CREATE ROLE console_durability_observer NOLOGIN NOSUPERUSER NOBYPASSRLS
 INHERIT NOCREATEDB NOCREATEROLE NOREPLICATION CONNECTION LIMIT -1;
GRANT EXECUTE ON FUNCTION pg_catalog.pg_control_system() TO console_durability_observer;"##,
            r##"SELECT bool_and(value = 'true'::jsonb) AND count(*)=2 FROM jsonb_each((SELECT jsonb_build_object('no_local_routine',(to_regprocedure('public.console_durability_observation_v1(name,oid)') IS NULL),'explicit_builtin_grant',(EXISTS(SELECT 1 FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a WHERE p.oid='pg_catalog.pg_control_system()'::regprocedure AND a.grantee='console_durability_observer'::regrole AND a.privilege_type='EXECUTE' AND NOT a.is_grantable)))))"##,
        ),
        (
            "partial_named_routine_only",
            r##"CREATE FUNCTION public.console_durability_observation_v1() RETURNS boolean LANGUAGE sql AS 'SELECT true';"##,
            r##"SELECT bool_and(value = 'true'::jsonb) AND count(*)=3 FROM jsonb_each((SELECT jsonb_build_object('observer_role_absent',(NOT EXISTS(SELECT 1 FROM pg_roles WHERE rolname='console_durability_observer')),'wrong_signature_present',(EXISTS(SELECT 1 FROM pg_proc WHERE oid='public.console_durability_observation_v1()'::regprocedure AND pronargs=0 AND NOT prosecdef)),'required_signature_absent',(to_regprocedure('public.console_durability_observation_v1(name,oid)') IS NULL))))"##,
        ),
    ];

    const OBSERVER_FAULTS: &[(&str, &str, &str)] = &[
        (
            "observer_body",
            r##"DO $probe$
DECLARE original text; definition text;
BEGIN
 SELECT prosrc,pg_get_functiondef(oid) INTO STRICT original,definition
 FROM pg_proc WHERE oid='public.console_durability_observation_v1(name,oid)'::regprocedure;
 IF encode(sha256(convert_to(original,'UTF8')),'hex')<>'855b83fbb3581ae7b052fd9863e0565acf8608800ac7890fc56300adbad9b01b'
    OR length(original)=0 OR length(definition)-length(replace(definition,original,''))<>length(original) THEN
   RAISE EXCEPTION 'mutation_control.body_source_mismatch';
 END IF;
 EXECUTE replace(definition,original,original || E'\n-- metadata_probe_observer_body_v1\n');
END $probe$;"##,
            r##"SELECT bool_and(value = 'true'::jsonb) AND count(*)=2 FROM jsonb_each((SELECT jsonb_build_object('exact_mutated_body',((SELECT encode(sha256(convert_to(prosrc,'UTF8')),'hex')='3990f0f3e3955edd74f5ed8bf08ad98013767a84fcb43d5315493f088858aa81' FROM pg_proc WHERE oid='public.console_durability_observation_v1(name,oid)'::regprocedure)),'security_definer_preserved',((SELECT prosecdef FROM pg_proc WHERE oid='public.console_durability_observation_v1(name,oid)'::regprocedure)))))"##,
        ),
        (
            "observer_security_invoker",
            r##"ALTER FUNCTION public.console_durability_observation_v1(name,oid) SECURITY INVOKER;"##,
            r##"SELECT bool_and(value = 'true'::jsonb) AND count(*)=2 FROM jsonb_each((SELECT jsonb_build_object('security_invoker',((SELECT NOT prosecdef FROM pg_proc WHERE oid='public.console_durability_observation_v1(name,oid)'::regprocedure)),'body_preserved',((SELECT encode(sha256(convert_to(prosrc,'UTF8')),'hex')='855b83fbb3581ae7b052fd9863e0565acf8608800ac7890fc56300adbad9b01b' FROM pg_proc WHERE oid='public.console_durability_observation_v1(name,oid)'::regprocedure)))))"##,
        ),
        (
            "observer_login",
            r##"ALTER ROLE console_durability_observer LOGIN;"##,
            r##"SELECT bool_and(value = 'true'::jsonb) AND count(*)=2 FROM jsonb_each((SELECT jsonb_build_object('login_enabled',((SELECT rolcanlogin FROM pg_roles WHERE rolname='console_durability_observer')),'not_superuser',((SELECT NOT rolsuper FROM pg_roles WHERE rolname='console_durability_observer')))))"##,
        ),
        (
            "observer_startup_grant_option",
            r##"GRANT EXECUTE ON FUNCTION public.console_durability_observation_v1(name,oid) TO console_auth_startup WITH GRANT OPTION;"##,
            r##"SELECT bool_and(value = 'true'::jsonb) AND count(*)=3 FROM jsonb_each((SELECT jsonb_build_object('startup_execute',(has_function_privilege('console_auth_startup','public.console_durability_observation_v1(name,oid)'::regprocedure,'EXECUTE')),'startup_grant_option',(has_function_privilege('console_auth_startup','public.console_durability_observation_v1(name,oid)'::regprocedure,'EXECUTE WITH GRANT OPTION')),'direct_grant_option',(EXISTS(SELECT 1 FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a WHERE p.oid='public.console_durability_observation_v1(name,oid)'::regprocedure AND a.grantee='console_auth_startup'::regrole AND a.privilege_type='EXECUTE' AND a.is_grantable)))))"##,
        ),
        (
            "observer_global_setting",
            r##"ALTER ROLE console_durability_observer SET statement_timeout='137ms';"##,
            r##"SELECT bool_and(value = 'true'::jsonb) AND count(*)=2 FROM jsonb_each((SELECT jsonb_build_object('exact_global_setting',((SELECT rolconfig=ARRAY['statement_timeout=137ms']::text[] FROM pg_roles WHERE rolname='console_durability_observer')),'global_catalog_setting',(EXISTS(SELECT 1 FROM pg_db_role_setting WHERE setrole='console_durability_observer'::regrole AND setdatabase=0 AND setconfig=ARRAY['statement_timeout=137ms']::text[])))))"##,
        ),
        (
            "observer_database_setting",
            r##"DO $probe$ BEGIN EXECUTE format('ALTER ROLE console_durability_observer IN DATABASE %I SET lock_timeout=%L',current_database(),'139ms'); END $probe$;"##,
            r##"SELECT bool_and(value = 'true'::jsonb) AND count(*)=2 FROM jsonb_each((SELECT jsonb_build_object('global_settings_absent',((SELECT rolconfig IS NULL FROM pg_roles WHERE rolname='console_durability_observer')),'exact_database_setting',(EXISTS(SELECT 1 FROM pg_db_role_setting WHERE setrole='console_durability_observer'::regrole AND setdatabase=(SELECT oid FROM pg_database WHERE datname=current_database()) AND setconfig=ARRAY['lock_timeout=139ms']::text[])))))"##,
        ),
        (
            "observer_membership_set_option",
            r##"GRANT pg_read_all_stats TO console_durability_observer WITH SET TRUE;"##,
            r##"SELECT bool_and(value = 'true'::jsonb) AND count(*)=1 FROM jsonb_each((SELECT jsonb_build_object('exact_changed_membership',(EXISTS(SELECT 1 FROM pg_auth_members WHERE roleid='pg_read_all_stats'::regrole AND member='console_durability_observer'::regrole AND NOT admin_option AND inherit_option AND set_option)))))"##,
        ),
        (
            "observer_native_grant_removed",
            r##"REVOKE EXECUTE ON FUNCTION pg_catalog.pg_control_system() FROM console_durability_observer;"##,
            r##"SELECT bool_and(value = 'true'::jsonb) AND count(*)=3 FROM jsonb_each((SELECT jsonb_build_object('observer_cannot_execute',(NOT has_function_privilege('console_durability_observer','pg_catalog.pg_control_system()'::regprocedure,'EXECUTE')),'account_owner_still_executes',(has_function_privilege('console_account_owner','pg_catalog.pg_control_system()'::regprocedure,'EXECUTE')),'startup_still_denied',(NOT has_function_privilege('console_auth_startup','pg_catalog.pg_control_system()'::regprocedure,'EXECUTE')))))"##,
        ),
        (
            "partial_drop_observer_routine",
            r##"DROP FUNCTION public.console_durability_observation_v1(name,oid);"##,
            r##"SELECT bool_and(value = 'true'::jsonb) AND count(*)=2 FROM jsonb_each((SELECT jsonb_build_object('local_routine_absent',(NOT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace WHERE n.nspname='public' AND p.proname='console_durability_observation_v1')),'builtin_grant_survives',(EXISTS(SELECT 1 FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a WHERE p.oid='pg_catalog.pg_control_system()'::regprocedure AND a.grantee='console_durability_observer'::regrole)))))"##,
        ),
    ];

    #[sqlx::test(migrations = false)]
    async fn finalized_profiles_refuse_capability_and_observer_drift(pool: PgPool) {
        let app = fixture(&pool).await;
        let (account, _) = enrolled(&app).await;
        assert_committed(&pool, &account).await;
        let retained = snapshot(&pool, account.account).await;
        let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
        runtime_positive(&runtime).await;
        let mut tx = pool.begin().await.unwrap();
        let disposable_admin: bool = sqlx::query_scalar("SELECT session_user=current_user AND session_user='console_buck_admin' AND current_database() LIKE '_sqlx_test_%' AND current_setting('console.sqlx_test_bootstrap',true)='buck-sqlx-superuser-v1' AND (SELECT rolsuper FROM pg_catalog.pg_roles WHERE rolname=current_user)")
            .fetch_one(&mut *tx).await.unwrap();
        assert!(
            disposable_admin,
            "requires marked disposable operator transaction"
        );
        let initially_absent: bool = sqlx::query_scalar("SELECT NOT EXISTS(SELECT 1 FROM pg_catalog.pg_roles WHERE rolname IN ('console_durability_observer','native_snapshot_probe_parent')) AND NOT EXISTS(SELECT 1 FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace WHERE n.nspname='public' AND p.proname='console_durability_observation_v1')")
            .fetch_one(&mut *tx).await.unwrap();
        assert!(
            initially_absent,
            "observer role/function prerequisite must be genuinely absent"
        );
        finalized(classify(&mut tx).await);
        // A global role with no local function/native grant is deliberately
        // invisible to the local profile. This does not claim a second DB's
        // complete observer installation was exercised.
        let mut role_only = tx.begin().await.unwrap();
        sqlx::raw_sql("CREATE ROLE console_durability_observer NOLOGIN NOSUPERUSER NOBYPASSRLS INHERIT NOCREATEDB NOCREATEROLE NOREPLICATION CONNECTION LIMIT -1")
            .execute(&mut *role_only).await.unwrap();
        let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_roles WHERE rolname='console_durability_observer' AND NOT rolcanlogin)")
            .fetch_one(&mut *role_only).await.unwrap();
        assert!(exists);
        finalized(classify(&mut role_only).await);
        role_only.rollback().await.unwrap();
        let absent: bool = sqlx::query_scalar("SELECT NOT EXISTS(SELECT 1 FROM pg_catalog.pg_roles WHERE rolname='console_durability_observer')")
            .fetch_one(&mut *tx).await.unwrap();
        assert!(
            absent,
            "profile equality alone cannot prove global role cleanup"
        );
        let mut executed = 0;
        for &(name, mutation, witness) in PLAIN_FAULTS {
            reject_fault(&mut tx, name, mutation, witness).await;
            assert!(
                snapshot(&pool, account.account).await == retained,
                "plain metadata test changed enrolled Account history"
            );
            executed += 1;
        }
        let observer_sql = include_str!("../../../../ops/postgres-install-durability-observer.sql");
        let mut observer = tx.begin().await.unwrap();
        sqlx::raw_sql(observer_sql)
            .execute(&mut *observer)
            .await
            .expect("unchanged real observer installer");
        finalized(classify(&mut observer).await);
        // Existing installer certification must still replay with the additional
        // Account-owner pg_control_system grant. No substitute positive grants.
        sqlx::raw_sql(observer_sql)
            .execute(&mut *observer)
            .await
            .expect("unchanged observer installer replay after228");
        finalized(classify(&mut observer).await);
        for &(name, mutation, witness) in OBSERVER_FAULTS {
            reject_fault(&mut observer, name, mutation, witness).await;
            assert!(
                snapshot(&pool, account.account).await == retained,
                "observer metadata test changed enrolled Account history"
            );
            executed += 1;
        }
        assert_eq!(
            executed, 13,
            "every declared metadata mutation must execute"
        );
        observer.rollback().await.unwrap();
        let restored: bool = sqlx::query_scalar("SELECT NOT EXISTS(SELECT 1 FROM pg_catalog.pg_roles WHERE rolname IN ('console_durability_observer','native_snapshot_probe_parent')) AND NOT EXISTS(SELECT 1 FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace WHERE n.nspname='public' AND p.proname='console_durability_observation_v1')")
            .fetch_one(&mut *tx).await.unwrap();
        assert!(
            restored,
            "optional installation and probe roles must be rolled back"
        );
        finalized(classify(&mut tx).await);
        tx.rollback().await.unwrap();
        assert!(
            snapshot(&pool, account.account).await == retained,
            "metadata regressions preserve enrolled Account history"
        );
        runtime_positive(&runtime).await;
        runtime.close().await;
    }
}
