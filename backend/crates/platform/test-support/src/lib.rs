//! Integration-test-only helpers shared by REST-crate test suites.
//!
//! `#[sqlx::test]` hands tests a pool connected as the migration/owner role,
//! which has BYPASSRLS. Building the router straight off that pool means the
//! request path never actually exercises row-level security, so a broken
//! policy can pass green. Route requests through [`runtime_role_pool`]
//! instead; keep seeding on the original owner pool.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use console_kernel_core::{BranchId, OrgId, UserId};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

/// A pool cloned from `owner_pool`'s connection settings, with every
/// connection switched to the low-privilege `console_rt` role via `SET ROLE`.
/// Build routers/stores from this pool so RLS applies to test requests.
pub async fn runtime_role_pool(owner_pool: &PgPool) -> PgPool {
    let options = owner_pool.connect_options().as_ref().clone();
    PgPoolOptions::new()
        .max_connections(4)
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                sqlx::query("SET ROLE console_rt").execute(conn).await?;
                Ok(())
            })
        })
        .connect_with(options)
        .await
        .expect("connect console_rt-role test pool")
}

/// Run a batch of static GRANT statements on `owner_pool` so the `console_rt`
/// runtime role can reach base tables the `#[sqlx::test]` superuser owns but
/// hasn't granted by default. `grants` must be static literals — no
/// interpolation. Living here (an unscanned crate) keeps the mutating-SQL
/// scanners off the caller's REST test file.
pub async fn grant_console_rt(owner_pool: &PgPool, grants: &[&'static str]) {
    for grant in grants {
        sqlx::query(*grant).execute(owner_pool).await.unwrap();
    }
}

/// Seed an `organizations` row for `org` plus a single `SUPER_ADMIN` user under
/// it (slug derived from the org UUID), returning the user id. Seeds as the
/// migration owner during setup, before the `console_rt` role switch.
pub async fn seed_org_and_super_admin(owner_pool: &PgPool, org: uuid::Uuid, tag: &str) -> UserId {
    let slug = format!("org-{}", &org.simple().to_string()[..12]);
    sqlx::query("INSERT INTO organizations (id, slug, name) VALUES ($1, $2, $3) ON CONFLICT (id) DO NOTHING")
        .bind(org)
        .bind(slug)
        .bind(format!("Org {tag}"))
        // rls-arming: ok test fixture seeds RLS tables as owner during setup, before the console_rt role switch
        .execute(owner_pool)
        .await
        .unwrap();
    let user_id = UserId::new();
    sqlx::query("INSERT INTO users (id, display_name, roles, org_id) VALUES ($1, $2, $3, $4)")
        .bind(*user_id.as_uuid())
        .bind(format!("Admin {tag}"))
        .bind(["SUPER_ADMIN"].as_slice())
        .bind(org)
        // rls-arming: ok test fixture seeds RLS tables as owner during setup, before the console_rt role switch
        .execute(owner_pool)
        .await
        .unwrap();
    user_id
}

/// Seed an `organizations` row with row-security disabled for the insert; slug
/// derived from `tag`. Owner-pool setup helper for docs REST tests.
pub async fn seed_org_rls_off(owner_pool: &PgPool, org: uuid::Uuid, tag: &str) {
    let mut tx = owner_pool.begin().await.unwrap();
    sqlx::query("SET LOCAL row_security = off")
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO organizations (id, slug, name) VALUES ($1, $2, $3) ON CONFLICT (id) DO NOTHING",
    )
    .bind(org)
    .bind(format!("org-{}", tag.to_lowercase()))
    .bind(format!("Org {tag}"))
    // rls-arming: ok test fixture seeds RLS tables as owner during setup, before the console_rt role switch
    .execute(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();
}

/// Seed an active `ADMIN` user under `org` with row-security disabled for the
/// insert, returning its id. Owner-pool setup helper for docs REST tests.
pub async fn seed_admin_user_rls_off(owner_pool: &PgPool, org: uuid::Uuid) -> UserId {
    let mut tx = owner_pool.begin().await.unwrap();
    sqlx::query("SET LOCAL row_security = off")
        .execute(&mut *tx)
        .await
        .unwrap();
    let user_id: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO users (display_name, roles, org_id, is_active) VALUES ($1, $2, $3, true) RETURNING id",
    )
    .bind(format!("User {}", uuid::Uuid::new_v4()))
    .bind(vec!["ADMIN".to_string()])
    .bind(org)
    // rls-arming: ok test fixture seeds RLS tables as owner during setup, before the console_rt role switch
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();
    UserId::from_uuid(user_id)
}

/// Seed the automation + policy fixtures the ontology acting-read test asserts
/// on: a workflow definition bound to the `workorder` type key, plus a catalog
/// Cedar policy attached to `object_type_id` as an object policy. Owner-pool
/// setup helper; the mutating SQL lives here so it stays off the scanned test.
pub async fn seed_bound_workflow_and_policy(
    owner_pool: &PgPool,
    org: uuid::Uuid,
    object_type_id: uuid::Uuid,
) {
    sqlx::query(
        r#"
        INSERT INTO workflow_definitions (org_id, workflow_key, display_name, object_type, status)
        VALUES ($1, 'wf.wo.review', 'WO Review', 'workorder', 'ACTIVE')
        "#,
    )
    .bind(org)
    // rls-arming: ok test fixture seeds RLS tables as owner during setup, before the console_rt role switch
    .execute(owner_pool)
    .await
    .unwrap();

    let cedar_id: uuid::Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO cedar_policy_catalog_entries
            (org_id, stable_key, title, natural_language_rule, effect, status, source,
             principal, action, resource, conditions, validation_status, generated_policy_text)
        VALUES ($1, 'pbac.wo_edit', 'WO Edit', 'authored in test', 'permit', 'draft', 'no_code_draft',
                '{}'::jsonb, '{}'::jsonb, '{}'::jsonb, '[]'::jsonb, 'valid', 'permit(principal,action,resource);')
        RETURNING id
        "#,
    )
    .bind(org)
    // rls-arming: ok test fixture seeds RLS tables as owner during setup, before the console_rt role switch
    .fetch_one(owner_pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO ont_object_policies (org_id, object_type_id, cedar_policy_id, effect) VALUES ($1, $2, $3, 'permit')",
    )
    .bind(org)
    .bind(object_type_id)
    .bind(cedar_id)
    // rls-arming: ok test fixture seeds RLS tables as owner during setup, before the console_rt role switch
    .execute(owner_pool)
    .await
    .unwrap();
}

/// Attach ONE unconditional enforced `view` permit to an object type.
///
/// Object-policy visibility is deny-by-default and unconditional: with nothing
/// attached, every single-row read, every action against an existing instance
/// and every lifecycle transition on that type is a 404. A suite whose subject
/// is command receipts or lifecycle configuration — not row visibility — needs
/// this to reach its subject at all, and it is the fixture equivalent of the org
/// having authored one permit through the audited attach route.
///
/// Purely additive, and it can never hide anything: a `forbid` still wins, and a
/// type with no permit stays invisible. Never call it from a suite that is
/// asserting visibility, which must author its own policy.
///
/// Owner-pool setup helper — `console_rt` holds no INSERT on either table
/// (0150:117-118, and 0205 took back the `ont_object_policies` INSERT 0154
/// granted).
pub async fn attach_enforced_view_permit(
    owner_pool: &PgPool,
    org: uuid::Uuid,
    object_type_id: uuid::Uuid,
    resource_type: &str,
) {
    let cedar_id: uuid::Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO cedar_policy_catalog_entries
            (org_id, stable_key, title, natural_language_rule, effect, status, source,
             principal, action, resource, conditions, policy_version, schema_version,
             bundle_digest, validation_status, normalized_row)
        VALUES ($1,
                'object_policy.' || $2 || '.' || replace(gen_random_uuid()::text, '-', ''),
                'Test view permit', 'permit viewing every row of this type',
                'permit', 'enforced', 'no_code_draft',
                '{}'::jsonb, '{}'::jsonb, '{}'::jsonb, '[]'::jsonb, 1,
                'ontology-runtime-filter-v1',
                'sha256:0000000000000000000000000000000000000000000000000000000000000000',
                'valid',
                jsonb_build_object('effect', 'permit', 'action', 'view',
                                   'resource_type', $2::text,
                                   'conditions', '[]'::jsonb))
        RETURNING id
        "#,
    )
    .bind(org)
    .bind(resource_type)
    // rls-arming: ok test fixture seeds RLS tables as owner during setup, before the console_rt role switch
    .fetch_one(owner_pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO ont_object_policies (org_id, object_type_id, cedar_policy_id, effect) VALUES ($1, $2, $3, 'permit')",
    )
    .bind(org)
    .bind(object_type_id)
    .bind(cedar_id)
    // rls-arming: ok test fixture seeds RLS tables as owner during setup, before the console_rt role switch
    .execute(owner_pool)
    .await
    .unwrap();
}

/// Seed a region + branch under `OrgId::knl()`. `region_name`/`branch_name`
/// are used as label prefixes; a random UUID suffix keeps names unique
/// across concurrent test runs.
pub async fn seed_branch(pool: &PgPool, region_name: &str, branch_name: &str) -> BranchId {
    let region_id: uuid::Uuid =
        sqlx::query_scalar("INSERT INTO regions (name, org_id) VALUES ($1, $2) RETURNING id")
            .bind(format!("{region_name}-{}", uuid::Uuid::new_v4()))
            .bind(*OrgId::knl().as_uuid())
            // rls-arming: ok test fixture seeds RLS tables as owner during setup, before the console_rt role switch
            .fetch_one(pool)
            .await
            .unwrap();
    let branch_id: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO branches (region_id, name, org_id) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(region_id)
    .bind(format!("{branch_name}-{}", uuid::Uuid::new_v4()))
    .bind(*OrgId::knl().as_uuid())
    // rls-arming: ok test fixture seeds RLS tables as owner during setup, before the console_rt role switch
    .fetch_one(pool)
    .await
    .unwrap();
    BranchId::from_uuid(branch_id)
}

/// Seed a user with `role`, assigned to `branch`, under `OrgId::knl()`.
pub async fn seed_user(pool: &PgPool, name: &str, role: &str, branch: BranchId) -> UserId {
    let id = UserId::new();
    sqlx::query("INSERT INTO users (id, display_name, roles, org_id) VALUES ($1, $2, $3, $4)")
        .bind(*id.as_uuid())
        .bind(name)
        .bind(Vec::from([role]))
        .bind(*OrgId::knl().as_uuid())
        // rls-arming: ok test fixture seeds RLS tables as owner during setup, before the console_rt role switch
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO user_branches (user_id, branch_id, org_id) VALUES ($1, $2, $3)")
        .bind(*id.as_uuid())
        .bind(*branch.as_uuid())
        .bind(*OrgId::knl().as_uuid())
        // rls-arming: ok test fixture seeds RLS tables as owner during setup, before the console_rt role switch
        .execute(pool)
        .await
        .unwrap();
    id
}

/// Real database login identities used only by the disposable integration harness.
/// These are infrastructure principals, not Console user roles or job titles.
#[derive(Clone, Copy)]
pub enum TestDatabaseLogin {
    Business,
    Auth,
    LeaveCommand,
    OntologyCommand,
    PlatformForceCommand,
}

impl TestDatabaseLogin {
    fn binding(self) -> (&'static str, &'static str) {
        match self {
            Self::Business => ("CONSOLE_APALIS_RUNTIME_DATABASE_URL", "console_rt"),
            Self::Auth => ("CONSOLE_TEST_AUTH_DATABASE_URL", "console_auth_rt"),
            Self::LeaveCommand => (
                "CONSOLE_TEST_LEAVE_COMMAND_DATABASE_URL",
                "console_leave_cmd",
            ),
            Self::OntologyCommand => (
                "CONSOLE_TEST_ONTOLOGY_COMMAND_DATABASE_URL",
                "console_ontology_cmd",
            ),
            Self::PlatformForceCommand => (
                "CONSOLE_TEST_PLATFORM_FORCE_COMMAND_DATABASE_URL",
                "console_platform_force_cmd",
            ),
        }
    }
}

/// Point a harness-provisioned LOGIN URL at this sqlx test's actual database.
/// Never derive a runtime connection from the owner URL or assume a role.
/// Missing external credentials are a harness error, not an authorization result.
pub fn login_test_database_url(owner_pool: &PgPool, login: TestDatabaseLogin) -> String {
    let (key, role) = login.binding();
    let value = std::env::var(key).unwrap_or_else(|_| panic!("missing disposable transport {key}"));
    let mut url = url::Url::parse(&value).expect("valid disposable PostgreSQL URL");
    assert!(matches!(url.scheme(), "postgres" | "postgresql"));
    assert_eq!(
        url.username(),
        role,
        "test transport must name its real login"
    );
    assert!(
        url.password().is_some_and(|value| !value.is_empty()),
        "test transport requires a password"
    );
    assert!(
        url.query().is_none(),
        "test transport cannot contain identity/role overrides"
    );
    let options = owner_pool.connect_options();
    let database = options.get_database().expect("sqlx test database name");
    url.set_path(database);
    url.to_string()
}

/// Authenticate as the actual restricted login, checking both session and current
/// user. A migration-owner session followed by SET ROLE cannot satisfy this.
pub async fn login_test_pool(owner_pool: &PgPool, login: TestDatabaseLogin) -> PgPool {
    let (_, role) = login.binding();
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&login_test_database_url(owner_pool, login))
        .await
        .expect(
            "connect actual restricted test login; missing role/topology is not a crypto failure",
        );
    let identity: (String, String, bool, bool, bool, bool, bool) = sqlx::query_as(
        "SELECT session_user::text, current_user::text, rolsuper, rolbypassrls, rolcreaterole, rolcreatedb, rolreplication FROM pg_catalog.pg_roles WHERE rolname = current_user",
    )
    // rls-arming: ok pg_catalog.pg_roles is cluster-global login metadata; this restricted-session identity check reads no Company rows
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(identity.0, role);
    assert_eq!(identity.1, role);
    assert!(
        !identity.2 && !identity.3 && !identity.4 && !identity.5 && !identity.6,
        "test execution login must not carry administrative capabilities"
    );
    pool
}

/// Read the existing operator finalizer; never emulate its grants or projection.
pub fn account_custody_finalizer_sql() -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../ops/postgres-finalize-account-custody.sql");
    std::fs::read_to_string(path)
        .expect("ACCOUNT_CUSTODY_FINALIZER_PREREQUISITE: production SQL file missing")
}

pub async fn finalize_account_custody(pool: &PgPool) {
    let mut tx = pool.begin().await.expect("begin disposable finalization");
    sqlx::query("SET LOCAL statement_timeout = '60s'")
        .execute(tx.as_mut())
        .await
        .expect("bound disposable finalization statement");
    let identity: (String, String, bool) = sqlx::query_as(
        "SELECT session_user::text,current_user::text,current_setting('console.sqlx_test_bootstrap',true)='buck-sqlx-superuser-v1' AND (SELECT rolsuper FROM pg_roles WHERE rolname=current_user)",
    ).fetch_one(tx.as_mut()).await.expect("inspect disposable finalization administrator");
    assert_eq!(
        identity,
        (
            "console_buck_admin".to_owned(),
            "console_buck_admin".to_owned(),
            true
        )
    );
    sqlx::raw_sql(sqlx::AssertSqlSafe(account_custody_finalizer_sql()))
        .execute(tx.as_mut())
        .await
        .expect("actual production Account custody finalizer");
    tx.commit().await.expect("commit disposable finalization");
}

/// Read the fixed production Auth7 SQL; no fixture grants or owner emulation.
pub fn account_credential_custody_finalizer_sql() -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../ops/postgres-finalize-account-credentials.sql");
    std::fs::read_to_string(path)
        .expect("AUTH7_ARTIFACT_PREREQUISITE: production credential SQL file missing")
}

/// Actual root+credential SQL in one marked disposable operator transaction.
pub async fn finalize_serving_account_custody(pool: &PgPool) {
    // Missing SQL is a prerequisite; never fall back to root-only behavior.
    let credentials = account_credential_custody_finalizer_sql();
    let mut tx = pool.begin().await.expect("begin disposable finalization");
    sqlx::query("SET LOCAL statement_timeout = '60s'")
        .execute(tx.as_mut())
        .await
        .expect("bound disposable finalization statement");
    sqlx::raw_sql("SET LOCAL lock_timeout='5s'; SET LOCAL search_path=pg_catalog,pg_temp")
        .execute(tx.as_mut())
        .await
        .expect("bound composed operator catalog and lock context");
    let identity: (String, String, bool) = sqlx::query_as(
        "SELECT session_user::text,current_user::text,current_setting('console.sqlx_test_bootstrap',true)='buck-sqlx-superuser-v1' AND (SELECT rolsuper FROM pg_roles WHERE rolname=current_user)",
    ).fetch_one(tx.as_mut()).await.expect("inspect disposable finalization administrator");
    assert_eq!(
        identity,
        (
            "console_buck_admin".to_owned(),
            "console_buck_admin".to_owned(),
            true
        )
    );
    sqlx::raw_sql(sqlx::AssertSqlSafe(account_custody_finalizer_sql()))
        .execute(tx.as_mut())
        .await
        .expect("actual production Account custody finalizer");
    sqlx::raw_sql(sqlx::AssertSqlSafe(credentials))
        .execute(tx.as_mut())
        .await
        .expect("actual production Account credential custody finalizer");
    tx.commit().await.expect("commit disposable finalization");
}

/// Reuse the app fixture's exact empty-database and owner-transport admission.
pub async fn prepare_test_migration_owner_url(pool: &PgPool) -> String {
    let mut connection = pool.acquire().await.expect("disposable admin connection");
    let identity: (String, String, String, bool, bool) = sqlx::query_as(
        r#"
        SELECT session_user::text, current_user::text, current_database(),
            current_setting('console.sqlx_test_bootstrap', true) = 'buck-sqlx-superuser-v1'
            AND (SELECT rolsuper FROM pg_catalog.pg_roles WHERE rolname = current_user)
            AND (SELECT pg_get_userbyid(datdba) = current_user
                 FROM pg_catalog.pg_database WHERE datname = current_database()),
            NOT EXISTS (
                SELECT 1 FROM pg_catalog.pg_class c
                JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
                WHERE n.nspname !~ '^pg_' AND n.nspname <> 'information_schema'
            )
        "#,
    )
    .fetch_one(&mut *connection)
    .await
    .expect("inspect empty disposable HTTP test database");
    assert_eq!(identity.0, "console_buck_admin");
    assert_eq!(identity.1, "console_buck_admin");
    assert!(
        identity.3 && identity.4,
        "requires marked empty SQLx database"
    );
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

    let owner_binding = std::env::var("CONSOLE_APALIS_OWNER_DATABASE_URL")
        .expect("missing disposable migration-owner transport");
    let mut owner_url = url::Url::parse(&owner_binding).expect("valid migration-owner URL");
    assert!(matches!(owner_url.scheme(), "postgres" | "postgresql"));
    assert_eq!(owner_url.username(), "console_app");
    assert!(owner_url.password().is_some_and(|p| !p.is_empty()));
    assert!(owner_url.query().is_none() && owner_url.fragment().is_none());
    let options = pool.connect_options();
    assert_eq!(Some(identity.2.as_str()), options.get_database());
    assert_eq!(owner_url.host_str(), Some(options.get_host()));
    assert_eq!(owner_url.port().unwrap_or(5432), options.get_port());
    owner_url.set_path(&identity.2);

    // Provision only the empty database container. Product tables, grants and
    // queue schema are created by the existing production migration boundary.
    sqlx::raw_sql(
        "DO $owner$ BEGIN          EXECUTE format('ALTER DATABASE %I OWNER TO console_app', current_database());          END $owner$;",
    )
    .execute(&mut *connection)
    .await
    .expect("assign empty test database to its real migration owner");
    drop(connection);
    owner_url.to_string()
}

/// Prepare the real numbered schema and Account projection for standalone tests.
/// The caller supplies only an empty, marked, exclusively owned SQLx database.
/// App fixtures retain their production migration/Apalis entry point separately.
pub async fn prepare_account_test_database(pool: &PgPool) {
    assert!(
        std::env::var_os("PGOPTIONS").is_none_or(|options| options.is_empty()),
        "standalone migration must not inherit PostgreSQL startup options"
    );
    let owner_url = prepare_test_migration_owner_url(pool).await;
    let owner = PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(std::time::Duration::from_secs(10))
        .connect(&owner_url)
        .await
        .expect("authenticate disposable migration owner");
    let mut connection = owner.acquire().await.expect("migration owner connection");
    let identity: (String, String, String, bool) = sqlx::query_as(
        "SELECT session_user::text, current_user::text, \
         (SELECT pg_get_userbyid(datdba) FROM pg_catalog.pg_database WHERE datname=current_database()), \
         rolcanlogin AND rolinherit AND NOT rolsuper AND rolbypassrls \
         AND NOT rolcreatedb AND NOT rolcreaterole AND NOT rolreplication \
         FROM pg_catalog.pg_roles WHERE rolname=current_user",
    )
    .fetch_one(&mut *connection)
    .await
    .expect("verify direct disposable migration-owner identity");
    assert_eq!(
        identity,
        (
            "console_app".to_owned(),
            "console_app".to_owned(),
            "console_app".to_owned(),
            true
        )
    );
    sqlx::raw_sql("SET SESSION lock_timeout = '5s'; SET SESSION statement_timeout = '60s';")
        .execute(&mut *connection)
        .await
        .expect("bound standalone numbered migrations");
    sqlx::migrate!("../db/migrations")
        .run(&mut *connection)
        .await
        .expect("apply actual numbered migrations as console_app");
    drop(connection);
    owner.close().await;
    finalize_serving_account_custody(pool).await;
    let auth = login_test_pool(pool, TestDatabaseLogin::Auth).await;
    let fenced: bool = sqlx::query_scalar("SELECT public.account_legacy_fenced_v1($1)")
        .bind(uuid::Uuid::new_v4())
        .fetch_one(&auth)
        .await
        .expect("real finalized projection must be readable by the auth login");
    assert!(!fenced, "fresh unmigrated fixture subject must be unfenced");
    auth.close().await;
}
