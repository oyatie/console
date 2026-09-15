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
    let verdict = projection_custody_verdict(&pool).await;
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

// Append to independently reviewed account_fence_projection.rs after review.
// All mutations affect only an individual SQLx disposable database.

async fn projection_catalog(pool: &PgPool) -> Value {
    sqlx::query_scalar(
        "SELECT jsonb_build_object('functions',(SELECT COALESCE(jsonb_agg(jsonb_build_object('row',to_jsonb(p),'xmin',p.xmin::text) ORDER BY p.oid),'[]'::jsonb) FROM pg_catalog.pg_proc p WHERE p.pronamespace='public'::regnamespace AND p.proname='account_legacy_fenced_v1'),'relations',(SELECT jsonb_agg(jsonb_build_object('name',c.relname,'xmin',c.xmin::text,'owner',c.relowner,'acl',c.relacl,'rls',c.relrowsecurity,'force',c.relforcerowsecurity) ORDER BY c.relname) FROM pg_catalog.pg_class c WHERE c.relnamespace='public'::regnamespace AND c.relname IN ('accounts','account_security','account_security_events','account_terms_acceptances','account_terms_head','account_terms_release_receipts')))"
    ).fetch_one(pool).await.unwrap()
}

#[sqlx::test(migrations = false)]
async fn account_fence_projection_finalizer_replay_is_exactly_unchanged(pool: PgPool) {
    prepare_http_database(&pool).await;
    let auth = real_auth_pool(&pool).await;
    assert!(!fenced(&auth, Uuid::new_v4()).await.unwrap());
    let before = projection_catalog(&pool).await;
    finalize_account_custody(&pool).await;
    assert_eq!(
        projection_catalog(&pool).await,
        before,
        "replay validates existing function without CREATE OR REPLACE or privilege rewrite"
    );
    let subject = UserId::new();
    insert_account_fence(&pool, subject, "ACTIVE").await;
    assert!(fenced(&auth, *subject.as_uuid()).await.unwrap());
    finalize_account_custody(&pool).await;
    assert_eq!(projection_catalog(&pool).await, before);
    assert!(
        fenced(&auth, *subject.as_uuid()).await.unwrap(),
        "replay cannot remove a permanent fence"
    );
}

async fn assert_projection_drift_is_not_repaired(
    pool: &PgPool,
    mutation: &'static str,
    body_only: bool,
) {
    prepare_http_database(pool).await;
    let auth = real_auth_pool(pool).await;
    let subject = UserId::new();
    insert_account_fence(pool, subject, "ACTIVE").await;
    assert!(
        fenced(&auth, *subject.as_uuid()).await.unwrap(),
        "positive control must reach installed function"
    );
    let valid = projection_catalog(pool).await;
    sqlx::raw_sql(sqlx::AssertSqlSafe(mutation))
        .execute(pool)
        .await
        .unwrap();
    let tampered = projection_catalog(pool).await;
    assert_ne!(
        tampered, valid,
        "fault injection must actually alter function metadata"
    );
    if body_only {
        let without_body_and_xmin = |mut value: Value| {
            for function in value["functions"].as_array_mut().unwrap() {
                function.as_object_mut().unwrap().remove("xmin");
                function["row"].as_object_mut().unwrap().remove("prosrc");
            }
            value
        };
        assert_eq!(
            without_body_and_xmin(valid.clone()),
            without_body_and_xmin(tampered.clone()),
            "body fault must preserve language, volatility, config, ACL and every other catalog attribute"
        );
    }
    let error = sqlx::raw_sql(sqlx::AssertSqlSafe(account_custody_finalizer_sql()))
        .execute(pool)
        .await
        .expect_err("operator must reject drift, never silently repair it");
    assert_eq!(
        error.as_database_error().and_then(|e| e.code()).as_deref(),
        Some("P0001")
    );
    assert!(
        error.to_string().contains("account_fence_projection"),
        "specific function drift refusal, not an unrelated prerequisite: {error}"
    );
    assert_eq!(
        projection_catalog(pool).await,
        tampered,
        "refusal must leave all function and table metadata unchanged"
    );
    let present: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM public.account_security WHERE account_id=$1)",
    )
    .bind(subject.as_uuid())
    .fetch_one(pool)
    .await
    .unwrap();
    assert!(present, "operator rejection preserves permanent fence data");
}

#[sqlx::test(migrations = false)]
async fn account_fence_projection_finalizer_refuses_body_drift(pool: PgPool) {
    assert_projection_drift_is_not_repaired(&pool,
        "DO $fault$ DECLARE definition text; original_body text; replacement text; BEGIN SELECT pg_catalog.pg_get_functiondef(p.oid),p.prosrc,CASE l.lanname WHEN 'plpgsql' THEN 'BEGIN RETURN false; END;' WHEN 'sql' THEN 'SELECT false' ELSE NULL END INTO definition,original_body,replacement FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_language l ON l.oid=p.prolang WHERE p.oid='public.account_legacy_fenced_v1(uuid)'::regprocedure; IF replacement IS NULL THEN RAISE EXCEPTION 'unsupported fixture language'; END IF; EXECUTE pg_catalog.replace(definition,original_body,replacement); END $fault$", true).await;
}

#[sqlx::test(migrations = false)]
async fn account_fence_projection_finalizer_refuses_owner_drift(pool: PgPool) {
    assert_projection_drift_is_not_repaired(
        &pool,
        "ALTER FUNCTION public.account_legacy_fenced_v1(uuid) OWNER TO console_terms_owner",
        false,
    )
    .await;
}

#[sqlx::test(migrations = false)]
async fn account_fence_projection_finalizer_refuses_invoker_drift(pool: PgPool) {
    assert_projection_drift_is_not_repaired(
        &pool,
        "ALTER FUNCTION public.account_legacy_fenced_v1(uuid) SECURITY INVOKER",
        false,
    )
    .await;
}

#[sqlx::test(migrations = false)]
async fn account_fence_projection_finalizer_refuses_search_path_drift(pool: PgPool) {
    assert_projection_drift_is_not_repaired(
        &pool,
        "ALTER FUNCTION public.account_legacy_fenced_v1(uuid) SET search_path=public,pg_catalog",
        false,
    )
    .await;
}

#[sqlx::test(migrations = false)]
async fn account_fence_projection_finalizer_refuses_row_security_drift(pool: PgPool) {
    assert_projection_drift_is_not_repaired(
        &pool,
        "ALTER FUNCTION public.account_legacy_fenced_v1(uuid) SET row_security=on",
        false,
    )
    .await;
}

#[sqlx::test(migrations = false)]
async fn account_fence_projection_finalizer_refuses_default_argument_drift(pool: PgPool) {
    assert_projection_drift_is_not_repaired(&pool,
        r"DO $fault$ DECLARE definition text; BEGIN SELECT pg_catalog.pg_get_functiondef('public.account_legacy_fenced_v1(uuid)'::regprocedure) INTO definition; EXECUTE pg_catalog.regexp_replace(definition, 'uuid\)', 'uuid DEFAULT NULL)'); END $fault$", false).await;
}

#[sqlx::test(migrations = false)]
async fn account_fence_projection_finalizer_refuses_overload_drift(pool: PgPool) {
    assert_projection_drift_is_not_repaired(&pool,
        "CREATE FUNCTION public.account_legacy_fenced_v1(text) RETURNS boolean LANGUAGE sql AS 'SELECT false'", false).await;
}

#[sqlx::test(migrations = false)]
async fn account_fence_projection_finalizer_refuses_public_execute_drift(pool: PgPool) {
    assert_projection_drift_is_not_repaired(
        &pool,
        "GRANT EXECUTE ON FUNCTION public.account_legacy_fenced_v1(uuid) TO PUBLIC",
        false,
    )
    .await;
}

#[sqlx::test(migrations = false)]
async fn account_fence_projection_operator_refuses_serving_and_migration_logins(pool: PgPool) {
    prepare_http_database(&pool).await;
    let auth = real_auth_pool(&pool).await;
    assert!(!fenced(&auth, Uuid::new_v4()).await.unwrap());
    let before = projection_catalog(&pool).await;
    let mut urls = account_transport_urls(&pool)
        .into_iter()
        .map(|(_, url)| url)
        .collect::<Vec<_>>();
    let mut migration = Url::parse(
        &std::env::var("CONSOLE_APALIS_OWNER_DATABASE_URL")
            .expect("real migration credential supplied by existing harness"),
    )
    .unwrap();
    let database: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(&pool)
        .await
        .unwrap();
    migration.set_path(&database);
    assert_eq!(migration.username(), "console_app");
    urls.push(migration.to_string());
    for url in urls {
        let caller = PgPool::connect(&url)
            .await
            .expect("actual nonoperator LOGIN connects");
        let error = sqlx::raw_sql(sqlx::AssertSqlSafe(account_custody_finalizer_sql()))
            .execute(&caller)
            .await
            .expect_err("serving/migration identity cannot install privileged projection");
        assert_eq!(
            error.as_database_error().and_then(|e| e.code()).as_deref(),
            Some("P0001")
        );
        assert!(
            error
                .to_string()
                .contains("account_custody.operator_identity_mismatch")
        );
        assert_eq!(projection_catalog(&pool).await, before);
        caller.close().await;
    }
    let migration = PgPool::connect(migration.as_str()).await.unwrap();
    let error = fenced(&migration, Uuid::new_v4())
        .await
        .expect_err("migration LOGIN cannot execute runtime projection");
    assert_eq!(
        error.as_database_error().and_then(|e| e.code()).as_deref(),
        Some("42501")
    );
}

// Test fixture correction: honor canonical query's documented caller context.
// Replace the frozen execute-only test's direct query_scalar include_str read
// with projection_custody_verdict(&pool).await; assertion stays unchanged.
async fn projection_custody_verdict(pool: &PgPool) -> String {
    let mut transaction = pool.begin().await.unwrap();
    sqlx::query("SET LOCAL search_path=pg_catalog,pg_temp")
        .execute(&mut *transaction)
        .await
        .unwrap();
    let verdict = sqlx::query_scalar(sqlx::AssertSqlSafe(include_str!(
        "../../src/account_custody_state.sql"
    )))
    .fetch_one(&mut *transaction)
    .await
    .unwrap();
    transaction.rollback().await.unwrap();
    verdict
}

// Additive tests; append to the reviewed projection module after review.

#[sqlx::test(migrations = false)]
async fn account_fence_projection_finalizer_refuses_duplicate_owner_acl_without_auth(pool: PgPool) {
    assert_projection_drift_is_not_repaired(
        &pool,
        "UPDATE pg_catalog.pg_proc SET proacl=ARRAY['console_account_owner=X/console_account_owner'::aclitem,'console_account_owner=X/console_account_owner'::aclitem] WHERE oid='public.account_legacy_fenced_v1(uuid)'::regprocedure",
        false,
    ).await;
}

#[sqlx::test(migrations = false)]
async fn account_fence_projection_installation_failure_rolls_back_custody_and_created_function(
    pool: PgPool,
) {
    prepare_http_database_staging(&pool).await;
    let auth = real_auth_pool(&pool).await;
    let before = projection_catalog(&pool).await;
    assert_eq!(
        before["functions"],
        json!([]),
        "fresh staging must not have the projection"
    );
    let pending = projection_custody_verdict(&pool).await;
    assert_eq!(pending, "account_custody.pending");
    // PostgreSQL's real ddl_command_end trigger aborts after the newly created
    // function's owner transfer, before the subsequent EXECUTE grants. The
    // operator still owns the entire custody+function installation statement.
    sqlx::raw_sql(
        "CREATE FUNCTION public.fence_fixture_fail_after_owner() RETURNS event_trigger LANGUAGE plpgsql AS $trigger_body$ BEGIN IF EXISTS(SELECT 1 FROM pg_catalog.pg_event_trigger_ddl_commands() command WHERE command.classid='pg_catalog.pg_proc'::regclass AND command.objid=pg_catalog.to_regprocedure('public.account_legacy_fenced_v1(uuid)')) THEN RAISE EXCEPTION 'fence_fixture.after_create_before_grant'; END IF; END; $trigger_body$; CREATE EVENT TRIGGER fence_fixture_post_create ON ddl_command_end WHEN TAG IN ('ALTER FUNCTION') EXECUTE FUNCTION public.fence_fixture_fail_after_owner();"
    ).execute(&pool).await.unwrap();
    let error = sqlx::raw_sql(sqlx::AssertSqlSafe(account_custody_finalizer_sql()))
        .execute(&pool)
        .await
        .expect_err("injected post-CREATE DDL failure must abort installation");
    assert_eq!(
        error.as_database_error().and_then(|e| e.code()).as_deref(),
        Some("P0001")
    );
    assert!(
        error
            .to_string()
            .contains("fence_fixture.after_create_before_grant"),
        "fault must be reached after CREATE, not an earlier prerequisite: {error}"
    );
    assert_eq!(
        projection_catalog(&pool).await,
        before,
        "all six ownership changes and new function creation must roll back atomically"
    );
    let pending = projection_custody_verdict(&pool).await;
    assert_eq!(pending, "account_custody.pending");
    sqlx::raw_sql("DROP EVENT TRIGGER fence_fixture_post_create; DROP FUNCTION public.fence_fixture_fail_after_owner();")
        .execute(&pool).await.unwrap();
    finalize_account_custody(&pool).await;
    assert!(
        !fenced(&auth, Uuid::new_v4()).await.unwrap(),
        "same installation succeeds once only the fault is removed"
    );
    let subject = UserId::new();
    insert_account_fence(&pool, subject, "ACTIVE").await;
    assert!(fenced(&auth, *subject.as_uuid()).await.unwrap());
}

// Additive exact collective custody-v2 profile tests. These grant only the
// existing NOLOGIN owners read access; all serving denial assertions remain.

#[sqlx::test(migrations = false)]
async fn account_fence_projection_prepared_profile_has_only_two_exact_owner_reads(pool: PgPool) {
    prepare_http_database(&pool).await;
    let auth = real_auth_pool(&pool).await;
    let acl: Vec<(String, String, String, String, bool)> = sqlx::query_as(
        "SELECT c.relname::text,pg_catalog.pg_get_userbyid(a.grantor)::text,pg_catalog.pg_get_userbyid(a.grantee)::text,a.privilege_type,a.is_grantable FROM pg_catalog.pg_class c CROSS JOIN LATERAL pg_catalog.aclexplode(c.relacl) a WHERE c.relnamespace='public'::regnamespace AND c.relname IN ('accounts','account_security','account_security_events','account_terms_acceptances','account_terms_head','account_terms_release_receipts') ORDER BY c.relname,a.grantee,a.privilege_type"
    ).fetch_all(&pool).await.unwrap();
    assert_eq!(
        acl,
        vec![
            (
                "account_security".into(),
                "console_account_owner".into(),
                "console_account_owner".into(),
                "SELECT".into(),
                false
            ),
            (
                "accounts".into(),
                "console_account_owner".into(),
                "console_account_owner".into(),
                "SELECT".into(),
                false
            ),
        ]
    );
    assert_eq!(
        projection_custody_verdict(&pool).await,
        "account_custody.finalized"
    );
    let subject = UserId::new();
    insert_account_fence(&pool, subject, "ACTIVE").await;
    assert!(
        fenced(&auth, *subject.as_uuid()).await.unwrap(),
        "real FK insert and definer read require both owner reads"
    );
    assert_privilege_denial(
        sqlx::query("SELECT account_id FROM public.account_security")
            .execute(&auth)
            .await,
    );
    let before = projection_catalog(&pool).await;
    finalize_account_custody(&pool).await;
    assert_eq!(
        projection_catalog(&pool).await,
        before,
        "prepared replay must not rewrite ACL or function metadata"
    );
}

#[sqlx::test(migrations = false)]
async fn account_fence_projection_dormant_finalized_v1_is_upgradeable_without_data_change(
    pool: PgPool,
) {
    prepare_http_database_staging(&pool).await;
    let auth = real_auth_pool(&pool).await;
    // Execute the actual frozen historical operator SQL, not a fixture that
    // imitates owner transfer. This proves SQL-level compatibility only, not
    // historical deployment transport or a production rollback procedure.
    use sha2::Digest as _;
    let historical = include_str!("fixtures/account-custody-dormant-v1-7af6dfd4.sql");
    assert_eq!(
        format!("{:x}", sha2::Sha256::digest(historical.as_bytes())),
        "84e356b88be8762726c26df03a4990a19e98c0fcda3d48a73389d1d2a087559a",
        "exact historical7af6dfd4 operator SQL blob must remain immutable"
    );
    sqlx::raw_sql(historical).execute(&pool).await.unwrap();
    let before = projection_catalog(&pool).await;
    assert_eq!(before["functions"], json!([]));
    assert!(
        before["relations"]
            .as_array()
            .unwrap()
            .iter()
            .all(|r| r["acl"] == json!([]))
    );
    assert_eq!(
        projection_custody_verdict(&pool).await,
        "account_custody.finalized"
    );
    let data_before: Vec<i64> = sqlx::query_scalar("SELECT (SELECT count(*) FROM public.accounts) UNION ALL SELECT count(*) FROM public.account_security")
        .fetch_all(&pool).await.unwrap();
    finalize_account_custody(&pool).await;
    assert!(!fenced(&auth, Uuid::new_v4()).await.unwrap());
    let data_after: Vec<i64> = sqlx::query_scalar("SELECT (SELECT count(*) FROM public.accounts) UNION ALL SELECT count(*) FROM public.account_security")
        .fetch_all(&pool).await.unwrap();
    assert_eq!(data_before, data_after);
    assert_eq!(
        projection_custody_verdict(&pool).await,
        "account_custody.finalized"
    );
}

async fn assert_prepared_profile_drift_is_not_repaired(
    pool: &PgPool,
    mutation: &'static str,
    expected_error: &'static str,
) {
    prepare_http_database(pool).await;
    let auth = real_auth_pool(pool).await;
    let subject = UserId::new();
    insert_account_fence(pool, subject, "ACTIVE").await;
    assert!(fenced(&auth, *subject.as_uuid()).await.unwrap());
    let valid = projection_catalog(pool).await;
    sqlx::raw_sql(sqlx::AssertSqlSafe(mutation))
        .execute(pool)
        .await
        .unwrap();
    let tampered = projection_catalog(pool).await;
    assert_ne!(
        tampered, valid,
        "actual profile mutation must reach PostgreSQL catalog"
    );
    let error = sqlx::raw_sql(sqlx::AssertSqlSafe(account_custody_finalizer_sql()))
        .execute(pool)
        .await
        .expect_err("profile drift must refuse, never repair into an accepted profile");
    assert_eq!(
        error.as_database_error().and_then(|e| e.code()).as_deref(),
        Some("P0001")
    );
    assert!(
        error.to_string().contains(expected_error),
        "specific reached profile refusal: {error}"
    );
    assert_eq!(
        projection_catalog(pool).await,
        tampered,
        "failed finalization cannot rewrite any metadata"
    );
    let present: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM public.account_security WHERE account_id=$1)",
    )
    .bind(subject.as_uuid())
    .fetch_one(pool)
    .await
    .unwrap();
    assert!(present);
}

#[sqlx::test(migrations = false)]
async fn account_fence_projection_refuses_half_profile_without_accounts_read(pool: PgPool) {
    assert_prepared_profile_drift_is_not_repaired(
        &pool,
        "REVOKE SELECT ON public.accounts FROM console_account_owner",
        "account_custody.unexpected_privilege",
    )
    .await;
}

#[sqlx::test(migrations = false)]
async fn account_fence_projection_refuses_half_profile_without_security_read(pool: PgPool) {
    assert_prepared_profile_drift_is_not_repaired(
        &pool,
        "REVOKE SELECT ON public.account_security FROM console_account_owner",
        "account_custody.unexpected_privilege",
    )
    .await;
}

#[sqlx::test(migrations = false)]
async fn account_fence_projection_refuses_owner_write(pool: PgPool) {
    assert_prepared_profile_drift_is_not_repaired(
        &pool,
        "GRANT UPDATE ON public.account_security TO console_account_owner",
        "account_custody.unexpected_privilege",
    )
    .await;
}

#[sqlx::test(migrations = false)]
async fn account_fence_projection_refuses_owner_grant_option(pool: PgPool) {
    assert_prepared_profile_drift_is_not_repaired(
        &pool,
        "GRANT SELECT ON public.account_security TO console_account_owner WITH GRANT OPTION",
        "account_custody.unexpected_privilege",
    )
    .await;
}

#[sqlx::test(migrations = false)]
async fn account_fence_projection_refuses_owner_wrong_grantor(pool: PgPool) {
    assert_prepared_profile_drift_is_not_repaired(&pool, "UPDATE pg_catalog.pg_class SET relacl=ARRAY['console_account_owner=r/console_terms_owner'::aclitem] WHERE oid='public.account_security'::regclass", "account_custody.unexpected_privilege").await;
}

#[sqlx::test(migrations = false)]
async fn account_fence_projection_refuses_security_events_read(pool: PgPool) {
    assert_prepared_profile_drift_is_not_repaired(
        &pool,
        "GRANT SELECT ON public.account_security_events TO console_account_owner",
        "account_custody.unexpected_privilege",
    )
    .await;
}

#[sqlx::test(migrations = false)]
async fn account_fence_projection_refuses_terms_acceptances_read(pool: PgPool) {
    assert_prepared_profile_drift_is_not_repaired(
        &pool,
        "GRANT SELECT ON public.account_terms_acceptances TO console_account_owner",
        "account_custody.unexpected_privilege",
    )
    .await;
}

#[sqlx::test(migrations = false)]
async fn account_fence_projection_refuses_terms_head_read(pool: PgPool) {
    assert_prepared_profile_drift_is_not_repaired(
        &pool,
        "GRANT SELECT ON public.account_terms_head TO console_terms_owner",
        "account_custody.unexpected_privilege",
    )
    .await;
}

#[sqlx::test(migrations = false)]
async fn account_fence_projection_refuses_terms_receipts_read(pool: PgPool) {
    assert_prepared_profile_drift_is_not_repaired(
        &pool,
        "GRANT SELECT ON public.account_terms_release_receipts TO console_terms_owner",
        "account_custody.unexpected_privilege",
    )
    .await;
}

#[sqlx::test(migrations = false)]
async fn account_fence_projection_refuses_function_with_all_owner_reads_removed(pool: PgPool) {
    assert_prepared_profile_drift_is_not_repaired(
        &pool,
        "REVOKE SELECT ON public.accounts,public.account_security FROM console_account_owner",
        "account_fence_projection.profile_mismatch",
    )
    .await;
}

#[sqlx::test(migrations = false)]
async fn account_fence_projection_refuses_prepared_acl_with_function_deleted(pool: PgPool) {
    assert_prepared_profile_drift_is_not_repaired(
        &pool,
        "DROP FUNCTION public.account_legacy_fenced_v1(uuid)",
        "account_fence_projection.profile_mismatch",
    )
    .await;
}
