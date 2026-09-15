#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! One sequential lifecycle test: the observer role is cluster-global. Run this
//! target alone in a marked disposable cluster, against one SQLx app database.
//! Calls the real shared installer; the refusal controls deliberately introduce
//! hostile catalog drift only inside transactions which are then rolled back.
use console_platform_test_support::{TestDatabaseLogin, login_test_pool};
use serde_json::Value;
use sqlx::{Connection, PgConnection, PgPool};

const OBSERVER_BODY_SHA256: &str =
    "855b83fbb3581ae7b052fd9863e0565acf8608800ac7890fc56300adbad9b01b";

fn installer_sql() -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../ops/postgres-install-durability-observer.sql");
    std::fs::read_to_string(path)
        .expect("DURABILITY_OBSERVER_INSTALLER_PREREQUISITE: real shared SQL file missing")
}

async fn execute(connection: &mut PgConnection, statement: &str) {
    sqlx::raw_sql(sqlx::AssertSqlSafe(statement))
        .execute(connection)
        .await
        .expect("execute explicit disposable catalog control");
}

async fn install(connection: &mut PgConnection, sql: &str) -> Result<(), sqlx::Error> {
    sqlx::raw_sql(sqlx::AssertSqlSafe(sql))
        .execute(connection)
        .await
        .map(|_| ())
}

fn assert_refused(error: sqlx::Error, scenario: &str) {
    let database = error
        .as_database_error()
        .expect("installer refusal must be a database diagnostic");
    assert_eq!(database.code().as_deref(), Some("P0001"), "{scenario}");
    assert!(
        database.message().starts_with("durability_observer."),
        "DURABILITY_OBSERVER_UNCLASSIFIED_REFUSAL: {scenario}"
    );
}

async fn admit_disposable_operator(connection: &mut PgConnection, owner: &PgPool) {
    let identity: (String, String, String, bool) = sqlx::query_as(
        "SELECT session_user::text,current_user::text,current_database(), \
         current_setting('console.sqlx_test_bootstrap',true)='buck-sqlx-superuser-v1' \
         AND (SELECT rolsuper FROM pg_catalog.pg_roles WHERE rolname=current_user) \
         AND (SELECT pg_get_userbyid(datdba)=current_user FROM pg_catalog.pg_database \
              WHERE datname=current_database())",
    )
    .fetch_one(&mut *connection)
    .await
    .expect("inspect marked disposable installer operator");
    assert_eq!(identity.0, "console_buck_admin");
    assert_eq!(identity.1, "console_buck_admin");
    assert!(identity.3, "marked SQLx admin-owned database required");
    let suffix = identity
        .2
        .strip_prefix("_sqlx_test_")
        .expect("SQLx database");
    assert!(
        suffix.len() == 52
            && suffix
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
    );
    assert_eq!(
        owner.connect_options().get_database(),
        Some(identity.2.as_str())
    );
    // Set outer timeouts before issuing the installer's atomic DO statement.
    execute(
        connection,
        "SET statement_timeout='15s'; SET lock_timeout='2s'",
    )
    .await;
}

// Bounded catalog census: every role and membership, role settings, default
// privileges, all public function definitions, native observer dependencies,
// relation/column ACLs, namespaces/databases and shared dependency rows.
// Preserve raw ACL representations and tuple xmins; do not print credentials.
// These are catalog-preservation checks, not a claim of cross-database census.
async fn snapshot(connection: &mut PgConnection) -> Value {
    sqlx::query_scalar(
        r#"
        SELECT jsonb_build_object(
          'roles', (SELECT jsonb_agg((to_jsonb(r)-'rolpassword') ||
                     jsonb_build_object('xmin',r.xmin::text,'has_password',r.rolpassword IS NOT NULL,
                       'password_sha256',encode(sha256(convert_to(r.rolpassword,'UTF8')),'hex'))
                     ORDER BY r.oid) FROM pg_catalog.pg_authid r),
          'memberships', (SELECT COALESCE(jsonb_agg(to_jsonb(m) ||
                     jsonb_build_object('xmin',m.xmin::text) ORDER BY m.roleid,m.member,m.grantor),'[]')
                     FROM pg_catalog.pg_auth_members m),
          'role_settings', (SELECT COALESCE(jsonb_agg(to_jsonb(s) ||
                     jsonb_build_object('xmin',s.xmin::text) ORDER BY s.setdatabase,s.setrole),'[]')
                     FROM pg_catalog.pg_db_role_setting s),
          'functions', (SELECT jsonb_agg(to_jsonb(p) ||
                     jsonb_build_object('xmin',p.xmin::text) ORDER BY p.oid)
                     FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace
                     WHERE n.nspname='public' OR p.proname IN
                     ('pg_control_system','pg_postmaster_start_time')),
          'default_acls', (SELECT COALESCE(jsonb_agg(to_jsonb(a) ||
                     jsonb_build_object('xmin',a.xmin::text) ORDER BY a.oid),'[]')
                     FROM pg_catalog.pg_default_acl a),
          'relations', (SELECT jsonb_agg(jsonb_build_array(c.oid,c.relowner,c.relacl)
                     ORDER BY c.oid) FROM pg_catalog.pg_class c),
          'column_acls', (SELECT COALESCE(jsonb_agg(jsonb_build_array(a.attrelid,a.attnum,a.attacl,a.xmin::text)
                     ORDER BY a.attrelid,a.attnum),'[]') FROM pg_catalog.pg_attribute a WHERE a.attacl IS NOT NULL),
          'schemas', (SELECT jsonb_agg(to_jsonb(n) || jsonb_build_object('xmin',n.xmin::text)
                     ORDER BY n.oid) FROM pg_catalog.pg_namespace n),
          'databases', (SELECT jsonb_agg(jsonb_build_array(d.oid,d.datdba,d.datacl)
                     ORDER BY d.oid) FROM pg_catalog.pg_database d),
          'shared_dependencies', (SELECT COALESCE(jsonb_agg(to_jsonb(d) ||
                     jsonb_build_object('xmin',d.xmin::text)
                     ORDER BY d.dbid,d.classid,d.objid,d.objsubid,d.refclassid,d.refobjid,d.deptype),'[]')
                     FROM pg_catalog.pg_shdepend d)
        )
        "#,
    )
    .fetch_one(connection)
    .await
    .expect("read exact catalog snapshot")
}

async fn assert_installed(connection: &mut PgConnection) {
    let role_valid: bool = sqlx::query_scalar(
        r#"
        SELECT NOT r.rolcanlogin AND NOT r.rolsuper AND NOT r.rolbypassrls
          AND r.rolinherit AND NOT r.rolcreatedb AND NOT r.rolcreaterole AND NOT r.rolreplication
          AND r.rolconnlimit=-1 AND r.rolvaliduntil IS NULL AND r.rolpassword IS NULL
          AND NOT EXISTS(SELECT 1 FROM pg_catalog.pg_db_role_setting s WHERE s.setrole=r.oid)
          AND (SELECT count(*)=1 AND bool_and(m.roleid='pg_read_all_stats'::regrole
               AND NOT m.admin_option AND m.inherit_option AND NOT m.set_option)
               FROM pg_catalog.pg_auth_members m WHERE m.member=r.oid)
          AND NOT EXISTS(SELECT 1 FROM pg_catalog.pg_auth_members m WHERE m.roleid=r.oid)
          AND has_function_privilege(r.oid,'pg_catalog.pg_control_system()','EXECUTE')
        FROM pg_catalog.pg_authid r WHERE r.rolname='console_durability_observer'
        "#,
    )
    .fetch_one(&mut *connection)
    .await
    .expect("installed observer role must exist");
    assert!(role_valid, "DURABILITY_OBSERVER_INVALID_ROLE_PROFILE");

    let function_valid: bool = sqlx::query_scalar(
        r#"
        SELECT count(*)=1 AND bool_and(
          p.proowner='console_durability_observer'::regrole
          AND p.prolang=(SELECT oid FROM pg_catalog.pg_language WHERE lanname='sql')
          AND p.prokind='f' AND p.provolatile='v' AND p.proparallel='u'
          AND p.prosecdef AND NOT p.proleakproof AND NOT p.proisstrict AND p.proretset
          AND p.prorettype='record'::regtype AND p.pronargs=2 AND p.pronargdefaults=0
          AND p.proargtypes='19 26'::oidvector AND p.proargdefaults IS NULL
          AND p.prosqlbody IS NULL AND p.prosupport=0 AND p.protrftypes IS NULL
          AND p.proallargtypes=ARRAY[19,26,25,1184,16,23,1184,25,26,25,23,1184,25,25,25,16,25,25,25,3220,3220]::oid[]
          AND p.proargmodes=ARRAY['i','i','t','t','t','t','t','t','t','t','t','t','t','t','t','t','t','t','t','t','t']::"char"[]
          AND p.proargnames=ARRAY['expected_slot','expected_replication_role','primary_system_id',
             'primary_started_at','primary_in_recovery','observer_pid','observer_started_at','slot_name',
             'replication_role_oid','replication_role_name','sender_pid','sender_started_at',
             'application_name','sender_state','sender_client_addr','sender_ssl','sender_client_dn',
             'sender_client_serial','sender_issuer_dn','flush_lsn','replay_lsn']::text[]
          AND p.proconfig=ARRAY['search_path=pg_catalog']::text[]
          AND encode(sha256(convert_to(p.prosrc,'UTF8')),'hex')=$1
          AND p.proacl IS NOT NULL AND cardinality(p.proacl)=2
          AND (SELECT count(*)=2 AND count(DISTINCT a.grantee)=2 AND bool_and(
               a.grantor=p.proowner AND a.privilege_type='EXECUTE' AND NOT a.is_grantable
               AND a.grantee IN (p.proowner,'console_rt'::regrole))
               FROM pg_catalog.aclexplode(p.proacl) a)
        ) FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace
        WHERE n.nspname='public' AND p.proname='console_durability_observation_v1'
        "#,
    )
    .bind(OBSERVER_BODY_SHA256)
    .fetch_one(&mut *connection)
    .await
    .unwrap();
    assert!(
        function_valid,
        "DURABILITY_OBSERVER_INVALID_FUNCTION_PROFILE"
    );

    let native_acl_valid: bool = sqlx::query_scalar(
        r#"
        SELECT count(*)=1 AND bool_and(a.privilege_type='EXECUTE' AND NOT a.is_grantable)
        FROM pg_catalog.pg_proc p CROSS JOIN LATERAL pg_catalog.aclexplode(p.proacl) a
        WHERE p.oid='pg_catalog.pg_control_system()'::regprocedure
          AND a.grantee='console_durability_observer'::regrole
        "#,
    )
    .fetch_one(connection)
    .await
    .unwrap();
    assert!(native_acl_valid, "DURABILITY_OBSERVER_INVALID_NATIVE_GRANT");
}

// Each mutation is independently restored. A savepoint placed AFTER the drift
// proves refused installation leaves the hostile state unchanged, rather than
// repairing it. The enclosing rollback then restores the known baseline.
async fn drift_refused(connection: &mut PgConnection, sql: &str, name: &str, mutation: &str) {
    let baseline = snapshot(connection).await;
    let mut tx = connection.begin().await.unwrap();
    execute(tx.as_mut(), mutation).await;
    let drift = snapshot(tx.as_mut()).await;
    assert!(
        baseline != drift,
        "control must create actual catalog drift: {name}"
    );
    execute(tx.as_mut(), "SAVEPOINT installer_attempt").await;
    let result = install(tx.as_mut(), sql).await;
    match result {
        Err(error) => assert_refused(error, name),
        Ok(()) => panic!("DURABILITY_OBSERVER_ACCEPTED_DRIFT: {name}"),
    }
    execute(tx.as_mut(), "ROLLBACK TO SAVEPOINT installer_attempt").await;
    assert!(
        snapshot(tx.as_mut()).await == drift,
        "DURABILITY_OBSERVER_REPAIRED_REFUSED_DRIFT: {name}"
    );
    tx.rollback().await.unwrap();
    assert!(
        snapshot(connection).await == baseline,
        "DURABILITY_OBSERVER_DRIFT_ROLLBACK_CHANGED_CATALOG: {name}"
    );
    println!("durability-observer-subcase: {name} PASS");
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn real_observer_installer_is_atomic_replay_exact_and_drift_refusing(owner: PgPool) {
    let sql = installer_sql();
    assert!(
        !sql.trim().is_empty(),
        "real shared installer must not be empty"
    );
    let mut connection = owner.acquire().await.unwrap();
    admit_disposable_operator(&mut connection, &owner).await;
    let absent: bool = sqlx::query_scalar(
        "SELECT NOT EXISTS(SELECT 1 FROM pg_catalog.pg_roles WHERE rolname='console_durability_observer') \
         AND NOT EXISTS(SELECT 1 FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_namespace n \
         ON n.oid=p.pronamespace WHERE n.nspname='public' AND p.proname='console_durability_observation_v1')",
    ).fetch_one(&mut *connection).await.unwrap();
    assert!(
        absent,
        "dedicated fresh cluster/appDB required; no second-database role reuse"
    );
    let original = snapshot(&mut connection).await;

    for (name, mutation) in [
        (
            "preexisting_unbound_role",
            "CREATE ROLE console_durability_observer NOLOGIN NOSUPERUSER NOBYPASSRLS INHERIT NOCREATEDB NOCREATEROLE NOREPLICATION",
        ),
        (
            "preexisting_login_role",
            "CREATE ROLE console_durability_observer LOGIN",
        ),
        (
            "preexisting_wrong_function",
            "CREATE FUNCTION public.console_durability_observation_v1(name,oid) RETURNS integer LANGUAGE sql AS 'SELECT 1'",
        ),
        (
            "unexpected_default_function_grantee",
            "ALTER DEFAULT PRIVILEGES FOR ROLE console_buck_admin GRANT EXECUTE ON FUNCTIONS TO console_auth_rt",
        ),
    ] {
        drift_refused(&mut connection, &sql, name, mutation).await;
    }

    let mut tx = connection.begin().await.unwrap();
    install(tx.as_mut(), &sql)
        .await
        .expect("real initial installation before deliberate failure");
    assert_installed(tx.as_mut()).await;
    let failure = sqlx::query("SELECT 1/0")
        .execute(tx.as_mut())
        .await
        .unwrap_err();
    assert_eq!(
        failure.as_database_error().unwrap().code().as_deref(),
        Some("22012")
    );
    tx.rollback().await.unwrap();
    assert!(
        snapshot(&mut connection).await == original,
        "DURABILITY_OBSERVER_PARTIAL_INSTALL_AFTER_ROLLBACK"
    );
    println!("durability-observer-subcase: full_outer_rollback PASS");

    let mut tx = connection.begin().await.unwrap();
    install(tx.as_mut(), &sql)
        .await
        .expect("actual absent-state installation");
    assert_installed(tx.as_mut()).await;
    tx.commit().await.unwrap();
    let installed = snapshot(&mut connection).await;
    println!("durability-observer-subcase: initial_install PASS");

    let mut tx = connection.begin().await.unwrap();
    install(tx.as_mut(), &sql)
        .await
        .expect("exact valid installer replay");
    tx.commit().await.unwrap();
    assert!(
        snapshot(&mut connection).await == installed,
        "DURABILITY_OBSERVER_REPLAY_WROTE_CATALOG"
    );
    println!("durability-observer-subcase: exact_replay PASS");

    let runtime = login_test_pool(&owner, TestDatabaseLogin::Business).await;
    let mut runtime_connection = runtime.acquire().await.unwrap();
    let privileges: (bool, bool, bool, bool, bool) = sqlx::query_as(
        "SELECT has_function_privilege(current_user,'public.console_durability_observation_v1(name,oid)','EXECUTE'), \
         has_function_privilege(current_user,'pg_catalog.pg_control_system()','EXECUTE'), \
         pg_has_role(current_user,'console_durability_observer','MEMBER'), \
         pg_has_role(current_user,'pg_read_all_stats','MEMBER'), \
         pg_has_role(current_user,'pg_monitor','MEMBER')",
    ).fetch_one(&mut *runtime_connection).await.unwrap();
    assert_eq!(privileges, (true, false, false, false, false));
    let rows: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM public.console_durability_observation_v1('console_observer_no_such_slot'::name,0::oid)",
    ).fetch_one(&mut *runtime_connection).await.expect("actual Business LOGIN calls native observer");
    assert_eq!(
        rows, 0,
        "nonexistent slot never yields fabricated observation"
    );
    let denied = sqlx::query("SELECT * FROM pg_catalog.pg_control_system()")
        .execute(&mut *runtime_connection)
        .await
        .unwrap_err();
    assert_eq!(
        denied.as_database_error().unwrap().code().as_deref(),
        Some("42501")
    );
    assert_refused(
        install(&mut runtime_connection, &sql).await.unwrap_err(),
        "business_installer_denied",
    );
    drop(runtime_connection);
    runtime.close().await;
    assert!(
        snapshot(&mut connection).await == installed,
        "restricted caller changed catalog"
    );
    println!("durability-observer-subcase: actual_business_login_capability PASS");

    for (name, mutation) in [
        ("role_login", "ALTER ROLE console_durability_observer LOGIN"),
        (
            "role_superuser",
            "ALTER ROLE console_durability_observer SUPERUSER",
        ),
        (
            "role_bypassrls",
            "ALTER ROLE console_durability_observer BYPASSRLS",
        ),
        (
            "role_noinherit",
            "ALTER ROLE console_durability_observer NOINHERIT",
        ),
        (
            "role_createdb",
            "ALTER ROLE console_durability_observer CREATEDB",
        ),
        (
            "role_createrole",
            "ALTER ROLE console_durability_observer CREATEROLE",
        ),
        (
            "role_replication",
            "ALTER ROLE console_durability_observer REPLICATION",
        ),
        (
            "role_configuration",
            "ALTER ROLE console_durability_observer SET search_path='public'",
        ),
        (
            "missing_membership",
            "REVOKE pg_read_all_stats FROM console_durability_observer",
        ),
        (
            "extra_membership",
            "GRANT pg_monitor TO console_durability_observer",
        ),
        (
            "membership_admin",
            "GRANT pg_read_all_stats TO console_durability_observer WITH ADMIN OPTION",
        ),
        (
            "membership_noinherit",
            "GRANT pg_read_all_stats TO console_durability_observer WITH INHERIT FALSE",
        ),
        (
            "membership_set_enabled",
            "GRANT pg_read_all_stats TO console_durability_observer WITH SET TRUE",
        ),
        (
            "incoming_membership",
            "GRANT console_durability_observer TO console_rt",
        ),
        (
            "missing_native_grant",
            "REVOKE EXECUTE ON FUNCTION pg_catalog.pg_control_system() FROM console_durability_observer",
        ),
        (
            "native_grant_option",
            "GRANT EXECUTE ON FUNCTION pg_catalog.pg_control_system() TO console_durability_observer WITH GRANT OPTION",
        ),
        (
            "extra_direct_native_grant",
            "GRANT EXECUTE ON FUNCTION pg_catalog.pg_postmaster_start_time() TO console_durability_observer",
        ),
        (
            "missing_function",
            "DROP FUNCTION public.console_durability_observation_v1(name,oid)",
        ),
        (
            "wrong_function_owner",
            "ALTER FUNCTION public.console_durability_observation_v1(name,oid) OWNER TO console_buck_admin",
        ),
        (
            "function_invoker",
            "ALTER FUNCTION public.console_durability_observation_v1(name,oid) SECURITY INVOKER",
        ),
        (
            "function_stable",
            "ALTER FUNCTION public.console_durability_observation_v1(name,oid) STABLE",
        ),
        (
            "function_parallel",
            "ALTER FUNCTION public.console_durability_observation_v1(name,oid) PARALLEL SAFE",
        ),
        (
            "function_strict",
            "ALTER FUNCTION public.console_durability_observation_v1(name,oid) STRICT",
        ),
        (
            "function_search_path",
            "ALTER FUNCTION public.console_durability_observation_v1(name,oid) SET search_path=public,pg_catalog",
        ),
        (
            "function_extra_config",
            "ALTER FUNCTION public.console_durability_observation_v1(name,oid) SET row_security=off",
        ),
        (
            "function_public_execute",
            "GRANT EXECUTE ON FUNCTION public.console_durability_observation_v1(name,oid) TO PUBLIC",
        ),
        (
            "function_auth_execute",
            "GRANT EXECUTE ON FUNCTION public.console_durability_observation_v1(name,oid) TO console_auth_rt",
        ),
        (
            "function_runtime_grant_option",
            "GRANT EXECUTE ON FUNCTION public.console_durability_observation_v1(name,oid) TO console_rt WITH GRANT OPTION",
        ),
        (
            "function_runtime_denied",
            "REVOKE EXECUTE ON FUNCTION public.console_durability_observation_v1(name,oid) FROM console_rt",
        ),
        (
            "function_overload",
            "CREATE FUNCTION public.console_durability_observation_v1(text,oid) RETURNS integer LANGUAGE sql AS 'SELECT 1'",
        ),
        (
            "function_body",
            "DO $drift$ BEGIN EXECUTE replace(pg_get_functiondef('public.console_durability_observation_v1(name,oid)'::regprocedure),'slots.active','NOT slots.active'); END $drift$",
        ),
    ] {
        drift_refused(&mut connection, &sql, name, mutation).await;
    }
    assert_installed(&mut connection).await;
    assert!(
        snapshot(&mut connection).await == installed,
        "catalog drift escaped test rollback"
    );
}
