// Test-only candidate for inclusion as auth_rest/account_fence_projection.rs.
// Parent auth_rest.rs supplies existing real database/finalizer helpers.
use super::*;

async fn real_auth_pool(owner: &PgPool) -> PgPool {
    let url = login_test_database_url(owner, TestDatabaseLogin::Auth);
    let connection = sqlx::postgres::PgPoolOptions::new()
        .max_connections(2)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(&url)
        .await;
    assert!(
        connection.is_ok(),
        "FENCE_AUTH_LOGIN_PREREQUISITE: real restricted LOGIN must connect"
    );
    connection.unwrap()
}

async fn fenced(pool: &PgPool, account: Uuid) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT public.account_legacy_fenced_v1($1)")
        .bind(account)
        .fetch_one(pool)
        .await
}

fn assert_privilege_denial(result: Result<sqlx::postgres::PgQueryResult, sqlx::Error>) {
    let error = result.expect_err("restricted identity must not gain raw table authority");
    assert_eq!(
        error.as_database_error().and_then(|e| e.code()).as_deref(),
        Some("42501")
    );
}

#[sqlx::test(migrations = false)]
async fn account_fence_projection_real_login_is_restricted_and_database_bound(pool: PgPool) {
    prepare_http_database(&pool).await;
    let auth = real_auth_pool(&pool).await;
    let identity: (String, String, String) =
        sqlx::query_as("SELECT session_user::text,current_user::text,current_database()")
            .fetch_one(&auth)
            .await
            .unwrap();
    let expected_database: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        identity,
        (
            "console_auth_rt".into(),
            "console_auth_rt".into(),
            expected_database
        )
    );
    let flags: (bool, bool, bool, bool, bool, bool, bool) = sqlx::query_as(
        "SELECT rolcanlogin,rolsuper,rolbypassrls,rolcreatedb,rolcreaterole,rolreplication,rolinherit FROM pg_catalog.pg_roles WHERE rolname=session_user"
    ).fetch_one(&auth).await.unwrap();
    assert_eq!(flags, (true, false, false, false, false, false, false));
    let edges: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM pg_catalog.pg_auth_members m JOIN pg_catalog.pg_roles r ON r.oid=m.member OR r.oid=m.roleid WHERE r.rolname='console_auth_rt'"
    ).fetch_one(&pool).await.unwrap();
    assert_eq!(
        edges, 0,
        "dedicated fence role has no incoming/outgoing role memberships"
    );
    for role in [
        "console_app",
        "console_rt",
        "console_account_owner",
        "console_terms_owner",
    ] {
        let assumable: bool = sqlx::query_scalar("SELECT pg_catalog.pg_has_role('console_auth_rt',$1,'SET') OR pg_catalog.pg_has_role($1,'console_auth_rt','SET')")
            .bind(role).fetch_one(&pool).await.unwrap();
        assert!(
            !assumable,
            "role assumption must not bridge fence and other custody"
        );
    }
    assert_privilege_denial(
        sqlx::query("SELECT id FROM public.users LIMIT 1")
            .execute(&auth)
            .await,
    );
    assert_privilege_denial(
        sqlx::query("SELECT account_id FROM public.account_security LIMIT 1")
            .execute(&auth)
            .await,
    );
}

#[sqlx::test(migrations = false)]
async fn account_fence_projection_presence_is_state_independent_and_not_rls_absence(pool: PgPool) {
    prepare_http_database(&pool).await;
    let auth = real_auth_pool(&pool).await;
    let rls: (bool, bool) = sqlx::query_as(
        "SELECT relrowsecurity,relforcerowsecurity FROM pg_catalog.pg_class WHERE oid='public.account_security'::regclass"
    ).fetch_one(&pool).await.unwrap();
    assert_eq!(
        rls,
        (false, false),
        "frozen dormant table has no RLS; a future change needs reviewed visible-row proof"
    );
    assert!(!fenced(&auth, Uuid::new_v4()).await.unwrap());
    for state in [
        "PENDING_ENROLLMENT",
        "ACTIVE",
        "SECURITY_SUSPENDED",
        "RECOVERY_REQUIRED",
    ] {
        let subject = UserId::new();
        insert_account_fence(&pool, subject, state).await;
        assert!(
            fenced(&auth, *subject.as_uuid()).await.unwrap(),
            "all existing security rows permanently fence"
        );
        sqlx::query("UPDATE public.account_security SET security_state='ACTIVE',revision=revision+1 WHERE account_id=$1")
            .bind(subject.as_uuid()).execute(&pool).await.unwrap();
        assert!(fenced(&auth, *subject.as_uuid()).await.unwrap());
    }
    // Temp-name shadowing must not convert a real fence into false absence.
    // This isolated adversarial connection creates the shadow as administrator,
    // then assumes auth. It does not require granting TEMP to the auth LOGIN.
    // All ordinary positive/negative calls above use the real auth credential.
    let mut conn = pool.begin().await.unwrap();
    sqlx::query("CREATE TEMP TABLE account_security(account_id uuid)")
        .execute(&mut *conn)
        .await
        .unwrap();
    sqlx::query("SET LOCAL ROLE console_auth_rt")
        .execute(&mut *conn)
        .await
        .unwrap();
    let subject = UserId::new();
    insert_account_fence(&pool, subject, "ACTIVE").await;
    let value: bool = sqlx::query_scalar("SELECT public.account_legacy_fenced_v1($1)")
        .bind(subject.as_uuid())
        .fetch_one(&mut *conn)
        .await
        .unwrap();
    assert!(value);
    conn.rollback().await.unwrap();
}

#[sqlx::test(migrations = false)]
async fn account_fence_projection_execute_only_and_no_custody_business_grants(pool: PgPool) {
    prepare_http_database(&pool).await;
    let auth = real_auth_pool(&pool).await;
    let function: (String, bool, bool, String, Vec<String>) = sqlx::query_as(
        "SELECT pg_catalog.pg_get_userbyid(p.proowner),p.prosecdef,p.proisstrict,p.prorettype::regtype::text,p.proconfig FROM pg_catalog.pg_proc p WHERE p.oid='public.account_legacy_fenced_v1(uuid)'::regprocedure"
    ).fetch_one(&pool).await.unwrap();
    assert_eq!(function.0, "console_account_owner");
    assert!(function.1);
    assert!(
        !function.2,
        "NULL must be explicitly rejected, not return SQL NULL through STRICT"
    );
    assert_eq!(function.3, "boolean");
    assert!(
        function.4.iter().any(
            |v| v == "search_path=pg_catalog, pg_temp" || v == "search_path=pg_catalog,pg_temp"
        )
    );
    assert!(
        function.4.iter().any(|v| v == "row_security=off"),
        "unexpected FORCE RLS must error instead of hiding a present fence"
    );
    let public_execute: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_proc p CROSS JOIN LATERAL pg_catalog.aclexplode(COALESCE(p.proacl,pg_catalog.acldefault('f',p.proowner))) a WHERE p.oid='public.account_legacy_fenced_v1(uuid)'::regprocedure AND a.grantee=0 AND a.privilege_type='EXECUTE')"
    ).fetch_one(&pool).await.unwrap();
    assert!(!public_execute);
    let unexpected_execute: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_proc p CROSS JOIN LATERAL pg_catalog.aclexplode(COALESCE(p.proacl,pg_catalog.acldefault('f',p.proowner))) a LEFT JOIN pg_catalog.pg_roles r ON r.oid=a.grantee WHERE p.oid='public.account_legacy_fenced_v1(uuid)'::regprocedure AND (a.grantee=0 OR r.rolname NOT IN ('console_account_owner','console_auth_rt') OR (r.rolname='console_auth_rt' AND a.is_grantable)))"
    ).fetch_one(&pool).await.unwrap();
    assert!(
        !unexpected_execute,
        "no unexpected grantee or delegation authority"
    );
    let auth_schema_create: bool = sqlx::query_scalar(
        "SELECT pg_catalog.has_schema_privilege('console_auth_rt','public','CREATE')",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(!auth_schema_create);
    for login in [
        TestDatabaseLogin::Business,
        TestDatabaseLogin::LeaveCommand,
        TestDatabaseLogin::OntologyCommand,
        TestDatabaseLogin::PlatformForceCommand,
    ] {
        let other = PgPool::connect(&login_test_database_url(&pool, login))
            .await
            .unwrap();
        let error = fenced(&other, Uuid::new_v4())
            .await
            .expect_err("only auth LOGIN can call projection");
        assert_eq!(
            error.as_database_error().and_then(|e| e.code()).as_deref(),
            Some("42501")
        );
    }
    for statement in [
        "SELECT account_id FROM public.account_security LIMIT 1",
        "SELECT security_state FROM public.account_security LIMIT 1",
        "UPDATE public.account_security SET revision=revision WHERE false",
        "DELETE FROM public.account_security WHERE false",
        "SELECT id FROM public.users LIMIT 1",
        "SELECT id FROM public.auth_webauthn_credentials LIMIT 1",
    ] {
        assert_privilege_denial(
            sqlx::query(sqlx::AssertSqlSafe(statement))
                .execute(&auth)
                .await,
        );
    }
    // Installing the narrow projection must retain the frozen table custody verdict.
    let verdict: String = sqlx::query_scalar(sqlx::AssertSqlSafe(include_str!(
        "../../src/account_custody_state.sql"
    )))
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(verdict, "account_custody.finalized");
}

#[sqlx::test(migrations = false)]
async fn account_fence_projection_null_and_database_failure_never_return_false(pool: PgPool) {
    prepare_http_database(&pool).await;
    let auth = real_auth_pool(&pool).await;
    let error =
        sqlx::query_scalar::<_, Option<bool>>("SELECT public.account_legacy_fenced_v1(NULL::uuid)")
            .fetch_one(&auth)
            .await
            .expect_err("NULL identity must not yield absent or null projection");
    assert_eq!(
        error.as_database_error().and_then(|e| e.code()).as_deref(),
        Some("22004")
    );
    assert!(
        !fenced(&auth, Uuid::new_v4()).await.unwrap(),
        "positive absence query reaches real routine"
    );
    let subject = UserId::new();
    insert_account_fence(&pool, subject, "ACTIVE").await;
    assert!(fenced(&auth, *subject.as_uuid()).await.unwrap());
    // Fault injection only in this disposable database: unseen RLS drift must
    // not silently map a real existing security row to an unmigrated account.
    sqlx::query("ALTER TABLE public.account_security ENABLE ROW LEVEL SECURITY")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("ALTER TABLE public.account_security FORCE ROW LEVEL SECURITY")
        .execute(&pool)
        .await
        .unwrap();
    let error = fenced(&auth, *subject.as_uuid())
        .await
        .expect_err("RLS-hidden fence must error, never false");
    assert_eq!(
        error.as_database_error().and_then(|e| e.code()).as_deref(),
        Some("42501")
    );
    auth.close().await;
    assert!(matches!(
        fenced(&auth, Uuid::new_v4()).await,
        Err(sqlx::Error::PoolClosed)
    ));
}
