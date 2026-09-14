//! LC01/03/04/06/08: actual migration and shared production SQL boundaries.
//! No fixture-created Account tables, fake migration ledger or substitute finalizer.
use super::{
    account_custody_finalizer_sql, finalize_account_custody, prepare_http_database_staging,
};
use console_platform_test_support::{TestDatabaseLogin, login_test_pool};
use futures::FutureExt;
use serde_json::Value;
use sqlx::{Connection, PgConnection, PgPool};
use std::time::Duration;

const TABLES: [&str; 6] = [
    "accounts",
    "account_security",
    "account_security_events",
    "account_terms_acceptances",
    "account_terms_head",
    "account_terms_release_receipts",
];

async fn snapshot(pool: &PgPool) -> Value {
    let mut value: Value = sqlx::query_scalar(r#"
        SELECT jsonb_agg(jsonb_build_object(
            'name', wanted.name, 'oid', c.oid, 'kind', c.relkind,
            'owner', pg_get_userbyid(c.relowner), 'acl', c.relacl::text,
            'rls', c.relrowsecurity, 'force_rls', c.relforcerowsecurity,
            'persistence', c.relpersistence, 'partition', c.relispartition,
            'columns', (SELECT jsonb_agg(jsonb_build_array(a.attnum,a.attname,format_type(a.atttypid,a.atttypmod),a.attnotnull,a.attisdropped,a.attidentity,a.attgenerated,a.attacl::text,pg_get_expr(d.adbin,d.adrelid)) ORDER BY a.attnum)
                FROM pg_attribute a LEFT JOIN pg_attrdef d ON d.adrelid=a.attrelid AND d.adnum=a.attnum WHERE a.attrelid=c.oid AND a.attnum>0),
            'constraints', (SELECT jsonb_agg(jsonb_build_array(k.conname,k.contype,k.convalidated,k.condeferrable,k.condeferred,pg_get_constraintdef(k.oid)) ORDER BY k.conname) FROM pg_constraint k WHERE k.conrelid=c.oid),
            'triggers', (SELECT jsonb_agg(jsonb_build_array(t.tgname,t.tgenabled,pg_get_triggerdef(t.oid)) ORDER BY t.tgname) FROM pg_trigger t WHERE t.tgrelid=c.oid AND NOT t.tgisinternal),
            'rules', (SELECT jsonb_agg(pg_get_ruledef(r.oid) ORDER BY r.rulename) FROM pg_rewrite r WHERE r.ev_class=c.oid),
            'policies', (SELECT jsonb_agg(to_jsonb(p) ORDER BY p.polname) FROM pg_policy p WHERE p.polrelid=c.oid),
            'inheritance', (SELECT jsonb_agg(jsonb_build_array(i.inhrelid,i.inhparent,i.inhseqno) ORDER BY i.inhrelid,i.inhparent) FROM pg_inherits i WHERE i.inhrelid=c.oid OR i.inhparent=c.oid)
        ) ORDER BY wanted.name)
        FROM unnest($1::text[]) wanted(name)
        LEFT JOIN pg_namespace ns ON ns.nspname='public'
        LEFT JOIN pg_class c ON c.relnamespace=ns.oid AND c.relname=wanted.name
    "#).bind(TABLES.to_vec()).fetch_one(pool).await.unwrap();
    assert!(
        complete_observation(&value),
        "incomplete custody observation"
    );
    for row in value.as_array_mut().unwrap() {
        if !row["oid"].is_null() {
            let table = row["name"].as_str().unwrap();
            assert!(TABLES.contains(&table));
            let data: Value = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
                "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text),'[]'::jsonb) FROM public.{table} t"
            ))).fetch_one(pool).await.unwrap();
            row["rows"] = data;
        }
    }
    value
}

fn complete_observation(value: &Value) -> bool {
    let Some(rows) = value.as_array() else {
        return false;
    };
    rows.len() == TABLES.len()
        && TABLES.iter().all(|name| {
            rows.iter()
                .filter(|row| row.get("name").and_then(Value::as_str) == Some(name))
                .count()
                == 1
        })
}

fn complete_successful_requests(expected: &[&str], observed: &[(&str, bool)]) -> bool {
    expected.len() == observed.len()
        && expected.iter().all(|id| {
            observed
                .iter()
                .filter(|(actual, success)| actual == id && *success)
                .count()
                == 1
        })
}

#[test]
fn request_accounting_rejects_omission_duplicate_and_failure() {
    let expected = ["first", "second"];
    assert!(complete_successful_requests(
        &expected,
        &[("first", true), ("second", true)]
    ));
    assert!(!complete_successful_requests(&expected, &[("first", true)]));
    assert!(!complete_successful_requests(
        &expected,
        &[("first", true), ("first", true)]
    ));
    assert!(!complete_successful_requests(
        &expected,
        &[("first", true), ("second", false)]
    ));
    assert!(!complete_successful_requests(&expected, &[]));
}

async fn assert_empty(pool: &PgPool) {
    for table in TABLES {
        let count: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "SELECT count(*) FROM public.{table}"
        )))
        .fetch_one(pool)
        .await
        .unwrap();
        assert_eq!(count, 0, "unexpected seeded {table}");
    }
}

async fn assert_owners(pool: &PgPool, finalized: bool) {
    let observed = snapshot(pool).await;
    for row in observed.as_array().unwrap() {
        let name = row["name"].as_str().unwrap();
        let owner = if !finalized {
            "console_app"
        } else if matches!(
            name,
            "account_terms_head" | "account_terms_release_receipts"
        ) {
            "console_terms_owner"
        } else {
            "console_account_owner"
        };
        assert_eq!(row["owner"], owner, "wrong owner for {name}");
    }
}

async fn finalizer(pool: &PgPool) -> Result<(), sqlx::Error> {
    let sql = account_custody_finalizer_sql();
    tokio::time::timeout(
        Duration::from_secs(70),
        sqlx::raw_sql(sqlx::AssertSqlSafe(sql.as_str())).execute(pool),
    )
    .await
    .expect("bounded finalizer did not return")
    .map(|_| ())
}

async fn assert_refusal(pool: &PgPool, expected: &str, before: Value) {
    let error = finalizer(pool).await.expect_err("hostile catalog accepted");
    let db = error
        .as_database_error()
        .expect("refusal must be PostgreSQL diagnostic, not transport failure");
    assert_eq!(db.code().as_deref(), Some("P0001"));
    assert!(db.message().contains(expected), "wrong refusal: {db}");
    assert_eq!(
        snapshot(pool).await,
        before,
        "finalizer changed refused catalog"
    );
}

async fn staged(pool: &PgPool) {
    // Read the real owner source first: absent source is a prerequisite, never
    // a successful hostile refusal or a forged fixture implementation.
    let _ = account_custody_finalizer_sql();
    prepare_http_database_staging(pool).await;
    assert_owners(pool, false).await;
    assert_empty(pool).await;
}

#[sqlx::test(migrations = false)]
async fn fresh_actual_migration_finalizes_empty_catalog_and_is_idempotent(pool: PgPool) {
    staged(&pool).await;
    let historical: Vec<(i64, Vec<u8>, bool)> = sqlx::query_as(
        "SELECT version,checksum,success FROM _sqlx_migrations WHERE version<=225 ORDER BY version",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(historical.len(), 225);
    assert!(historical.iter().all(|r| r.2));
    finalize_account_custody(&pool).await;
    assert_owners(&pool, true).await;
    assert_empty(&pool).await;
    let before = snapshot(&pool).await;
    finalize_account_custody(&pool).await;
    assert_eq!(snapshot(&pool).await, before);
    let after: Vec<(i64, Vec<u8>, bool)> = sqlx::query_as(
        "SELECT version,checksum,success FROM _sqlx_migrations WHERE version<=225 ORDER BY version",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(after, historical);
}

macro_rules! hostile {
    ($name:ident,$mutation:expr,$reason:expr) => {
        #[sqlx::test(migrations = false)]
        async fn $name(pool: PgPool) {
            staged(&pool).await;
            sqlx::raw_sql($mutation).execute(&pool).await.unwrap();
            let before = snapshot(&pool).await;
            assert_refusal(&pool, $reason, before).await;
        }
    };
}

hostile!(
    missing_relation_refuses,
    "DROP TABLE public.account_terms_head",
    "account_custody.catalog_missing"
);
hostile!(
    wrong_column_shape_refuses,
    "ALTER TABLE public.account_security ALTER COLUMN context_generation DROP NOT NULL",
    "account_custody.catalog_shape_mismatch"
);
hostile!(
    wrong_owner_refuses,
    "ALTER TABLE public.accounts OWNER TO console_rt",
    "account_custody.owner_mismatch"
);
hostile!(
    mixed_owners_refuse,
    "ALTER TABLE public.accounts OWNER TO console_account_owner",
    "account_custody.owner_mismatch"
);
hostile!(
    unexpected_table_privilege_refuses,
    "GRANT SELECT ON public.accounts TO console_rt",
    "account_custody.unexpected_privilege"
);
hostile!(
    unexpected_column_privilege_refuses,
    "GRANT SELECT(id) ON public.accounts TO console_rt",
    "account_custody.unexpected_privilege"
);
hostile!(
    nonempty_staging_refuses,
    "INSERT INTO public.accounts(id,created_at) VALUES ('00000000-0000-4000-8000-000000000001','2026-09-14T00:00:00Z')",
    "account_custody.nonempty_staging"
);
hostile!(
    unvalidated_tuple_fk_refuses,
    r#"DO $$ DECLARE key_name text; BEGIN SELECT conname INTO STRICT key_name FROM pg_constraint WHERE conrelid='public.account_terms_head'::regclass AND contype='f'; EXECUTE format('ALTER TABLE public.account_terms_head DROP CONSTRAINT %I',key_name); ALTER TABLE public.account_terms_head ADD CONSTRAINT lc_unvalidated FOREIGN KEY(release_receipt_ref,revision,manifest_sha256) REFERENCES public.account_terms_release_receipts(id,revision,manifest_sha256) ON DELETE RESTRICT NOT VALID; END $$;"#,
    "account_custody.catalog_shape_mismatch"
);
hostile!(
    unexpected_table_trigger_refuses,
    r#"CREATE FUNCTION public.lc_trigger() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RETURN NEW; END $$; CREATE TRIGGER lc_trigger BEFORE INSERT ON public.accounts FOR EACH ROW EXECUTE FUNCTION public.lc_trigger();"#,
    "account_custody.catalog_shape_mismatch"
);
hostile!(
    unexpected_rule_refuses,
    "CREATE RULE lc_rule AS ON DELETE TO public.accounts DO INSTEAD NOTHING",
    "account_custody.catalog_shape_mismatch"
);
hostile!(
    unexpected_inheritance_refuses,
    "CREATE TABLE public.lc_child() INHERITS(public.accounts)",
    "account_custody.catalog_shape_mismatch"
);

#[sqlx::test(migrations = false)]
async fn owner_membership_refuses_and_fixture_cleans_cluster_edge(pool: PgPool) {
    staged(&pool).await;
    let sql = account_custody_finalizer_sql();
    let before = snapshot(&pool).await;
    let mut connection = pool.acquire().await.unwrap();
    sqlx::raw_sql("BEGIN; GRANT console_account_owner TO console_rt WITH ADMIN FALSE, INHERIT FALSE, SET TRUE; SAVEPOINT before_finalizer")
        .execute(&mut *connection).await.unwrap();
    let result = tokio::time::timeout(
        Duration::from_secs(70),
        sqlx::raw_sql(sqlx::AssertSqlSafe(sql.as_str())).execute(&mut *connection),
    )
    .await
    .expect("bounded finalizer did not return");
    // Clear the expected failed statement without publishing the hostile edge.
    sqlx::raw_sql("ROLLBACK TO SAVEPOINT before_finalizer")
        .execute(&mut *connection)
        .await
        .unwrap();
    let edge_preserved: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_auth_members WHERE roleid='console_account_owner'::regrole AND member='console_rt'::regrole AND NOT admin_option AND NOT inherit_option AND set_option)").fetch_one(&mut *connection).await.unwrap();
    sqlx::raw_sql("ROLLBACK")
        .execute(&mut *connection)
        .await
        .unwrap();
    let after = snapshot(&pool).await;
    assert!(edge_preserved, "finalizer repaired a refused membership");
    let error = result.expect_err("owner membership accepted");
    let db = error.as_database_error().unwrap();
    assert_eq!(db.code().as_deref(), Some("P0001"));
    assert!(
        db.message()
            .contains("account_custody.owner_topology_mismatch")
    );
    assert_eq!(before, after);
}

#[sqlx::test(migrations = false)]
async fn runtime_refusals_have_real_login_and_exact_table_controls(pool: PgPool) {
    staged(&pool).await;
    finalize_account_custody(&pool).await;
    let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
    let expected_db: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(&pool)
        .await
        .unwrap();
    for table in TABLES {
        let mut connection = runtime.acquire().await.unwrap();
        let positive: (String, String, i32) =
            sqlx::query_as("SELECT current_user::text,current_database(),1")
                .fetch_one(&mut *connection)
                .await
                .unwrap();
        assert_eq!(positive, ("console_rt".to_owned(), expected_db.clone(), 1));
        let error = sqlx::query(sqlx::AssertSqlSafe(format!(
            "SELECT * FROM public.{table} LIMIT 0"
        )))
        .execute(&mut *connection)
        .await
        .expect_err("runtime custody read succeeded");
        let db = error
            .as_database_error()
            .expect("connection failure cannot prove privilege refusal");
        assert_eq!(db.code().as_deref(), Some("42501"));
        assert_eq!(db.message(), format!("permission denied for table {table}"));
    }
    runtime.close().await;
}

#[sqlx::test(migrations = false)]
async fn snapshot_checker_detects_missing_null_duplicate_and_omitted_transfer(pool: PgPool) {
    staged(&pool).await;
    let nominal = snapshot(&pool).await;
    assert!(complete_observation(&nominal));
    let mut missing = nominal.clone();
    missing.as_array_mut().unwrap().pop();
    assert!(!complete_observation(&missing));
    assert!(!complete_observation(&Value::Null));
    assert!(!complete_observation(&serde_json::json!([])));
    let mut duplicate = nominal.clone();
    duplicate[0] = duplicate[1].clone();
    assert!(!complete_observation(&duplicate));
    let mut tx = pool.begin().await.unwrap();
    sqlx::raw_sql("ALTER TABLE public.accounts OWNER TO console_account_owner")
        .execute(&mut *tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let partial = snapshot(&pool).await;
    assert_ne!(nominal, partial, "observer lost an actual ownership change");
    assert_refusal(&pool, "account_custody.owner_mismatch", partial).await;
}

#[sqlx::test(migrations = false)]
async fn populated_225_upgrade_preserves_legacy_rows_and_checksums(pool: PgPool) {
    let _ = account_custody_finalizer_sql();
    let config = super::prepare_http_migration_config(&pool).await;
    let mut owner = PgConnection::connect(config.database_url.as_deref().unwrap())
        .await
        .unwrap();
    let identity: (String, String) = sqlx::query_as("SELECT session_user::text,current_user::text")
        .fetch_one(&mut owner)
        .await
        .unwrap();
    assert_eq!(
        identity,
        ("console_app".to_owned(), "console_app".to_owned())
    );
    sqlx::migrate!("../crates/platform/db/migrations")
        .run_to(225, &mut owner)
        .await
        .unwrap();
    let branch = super::seed_branch(&pool, "lc-existing", "lc-existing").await;
    let old_row: Value = sqlx::query_scalar("SELECT to_jsonb(b) FROM branches b WHERE id=$1")
        .bind(branch.as_uuid())
        .fetch_one(&pool)
        .await
        .unwrap();
    let old_checksums: Vec<(i64, Vec<u8>)> =
        sqlx::query_as("SELECT version,checksum FROM _sqlx_migrations ORDER BY version")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(old_checksums.len(), 225);
    drop(owner);
    console_app::run_migrations(&config).await.unwrap();
    assert_owners(&pool, false).await;
    finalize_account_custody(&pool).await;
    assert_empty(&pool).await;
    let after_row: Value = sqlx::query_scalar("SELECT to_jsonb(b) FROM branches b WHERE id=$1")
        .bind(branch.as_uuid())
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(old_row, after_row);
    let after_checksums: Vec<(i64, Vec<u8>)> = sqlx::query_as(
        "SELECT version,checksum FROM _sqlx_migrations WHERE version<=225 ORDER BY version",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(old_checksums, after_checksums);
}

#[sqlx::test(migrations = false)]
async fn failure_after_actual_third_transfer_rolls_back_every_owner(pool: PgPool) {
    staged(&pool).await;
    // Privileged fault injection, not a replacement finalizer or fixture owner.
    // Observe actual transactional catalog changes from the DDL event context.
    sqlx::raw_sql(r#"
        CREATE FUNCTION public.lc_third_owner_fault() RETURNS event_trigger LANGUAGE plpgsql AS $$
        DECLARE transferred integer;
        BEGIN
            SELECT count(*) INTO transferred FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace
            WHERE n.nspname='public' AND c.relname IN ('accounts','account_security','account_security_events','account_terms_acceptances','account_terms_head','account_terms_release_receipts')
            AND pg_get_userbyid(c.relowner) IN ('console_account_owner','console_terms_owner');
            IF transferred=3 THEN RAISE EXCEPTION 'LC_FAULT_AFTER_THREE_ACTUAL_TRANSFERS'; END IF;
        END $$;
        CREATE EVENT TRIGGER lc_third_owner_fault ON ddl_command_end WHEN TAG IN ('ALTER TABLE') EXECUTE FUNCTION public.lc_third_owner_fault();
    "#).execute(&pool).await.unwrap();
    let before = snapshot(&pool).await;
    let outcome = finalizer(&pool).await;
    let after = snapshot(&pool).await;
    sqlx::raw_sql(
        "DROP EVENT TRIGGER lc_third_owner_fault; DROP FUNCTION public.lc_third_owner_fault()",
    )
    .execute(&pool)
    .await
    .unwrap();
    let error = outcome.expect_err("third transfer fault was not reached");
    let db = error.as_database_error().unwrap();
    assert_eq!(db.code().as_deref(), Some("P0001"));
    assert!(
        db.message()
            .contains("LC_FAULT_AFTER_THREE_ACTUAL_TRANSFERS"),
        "wrong failure {db}"
    );
    assert_eq!(before, after);
    assert_owners(&pool, false).await;
    finalize_account_custody(&pool).await;
    assert_owners(&pool, true).await;
}

async fn wait_for_blocker(pool: &PgPool, blocked: i32, blocker: i32) {
    tokio::time::timeout(Duration::from_secs(4), async {
        loop {
            let seen: bool = sqlx::query_scalar("SELECT $2=ANY(pg_blocking_pids($1))")
                .bind(blocked)
                .bind(blocker)
                .fetch_one(pool)
                .await
                .unwrap();
            if seen {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("actual PostgreSQL lock dependency not observed");
}

#[sqlx::test(migrations = false)]
async fn concurrent_finalizers_have_complete_request_outcomes_and_one_final_state(pool: PgPool) {
    staged(&pool).await;
    let sql = account_custody_finalizer_sql();
    let mut first = pool.acquire().await.unwrap();
    let mut second = pool.acquire().await.unwrap();
    let first_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *first)
        .await
        .unwrap();
    let second_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *second)
        .await
        .unwrap();
    sqlx::raw_sql("BEGIN; LOCK TABLE public.accounts IN ACCESS EXCLUSIVE MODE")
        .execute(&mut *first)
        .await
        .unwrap();
    let second_sql = sql.clone();
    let second_task = tokio::spawn(async move {
        sqlx::raw_sql(sqlx::AssertSqlSafe(second_sql.as_str()))
            .execute(&mut *second)
            .await
    });
    wait_for_blocker(&pool, second_pid, first_pid).await;
    sqlx::raw_sql(sqlx::AssertSqlSafe(sql.as_str()))
        .execute(&mut *first)
        .await
        .unwrap();
    sqlx::raw_sql("COMMIT").execute(&mut *first).await.unwrap();
    let second_outcome = tokio::time::timeout(Duration::from_secs(65), second_task)
        .await
        .unwrap()
        .unwrap();
    let requests = [("first", true), ("second", second_outcome.is_ok())];
    assert!(complete_successful_requests(
        &["first", "second"],
        &requests
    ));
    assert_owners(&pool, true).await;
    assert_empty(&pool).await;
    let before = snapshot(&pool).await;
    finalize_account_custody(&pool).await;
    assert_eq!(before, snapshot(&pool).await);
}

#[sqlx::test(migrations = false)]
async fn blocked_finalizer_times_out_without_partial_transfer_and_retries(pool: PgPool) {
    staged(&pool).await;
    let before = snapshot(&pool).await;
    let mut blocker = pool.acquire().await.unwrap();
    sqlx::raw_sql("BEGIN; LOCK TABLE public.accounts IN ACCESS EXCLUSIVE MODE")
        .execute(&mut *blocker)
        .await
        .unwrap();
    let result = finalizer(&pool).await;
    sqlx::raw_sql("ROLLBACK")
        .execute(&mut *blocker)
        .await
        .unwrap();
    let error = result.expect_err("held relation lock should time out");
    assert_eq!(
        error.as_database_error().unwrap().code().as_deref(),
        Some("55P03")
    );
    assert_eq!(snapshot(&pool).await, before);
    finalize_account_custody(&pool).await;
    assert_owners(&pool, true).await;
}

#[sqlx::test(migrations = false)]
async fn another_database_crosses_0165_while_finalization_is_in_flight(pool: PgPool) {
    staged(&pool).await;
    let unique = uuid::Uuid::new_v4().simple().to_string();
    let database = format!("_sqlx_test_{unique}{}", &unique[..20]);
    let outcome = std::panic::AssertUnwindSafe(async {
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "CREATE DATABASE \"{database}\""
    )))
    .execute(&pool)
    .await
    .unwrap();
    let options = pool.connect_options().as_ref().clone().database(&database);
    let other = sqlx::postgres::PgPoolOptions::new()
        .max_connections(4)
        .connect_with(options)
        .await
        .unwrap();
    let config = super::prepare_http_migration_config(&other).await;
    let mut owner = PgConnection::connect(config.database_url.as_deref().unwrap())
        .await
        .unwrap();
    sqlx::migrate!("../crates/platform/db/migrations")
        .run_to(164, &mut owner)
        .await
        .unwrap();
    drop(owner);
    let mut lock = pool.acquire().await.unwrap();
    let lock_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *lock)
        .await
        .unwrap();
    sqlx::raw_sql("BEGIN; LOCK TABLE public.accounts IN ACCESS EXCLUSIVE MODE")
        .execute(&mut *lock)
        .await
        .unwrap();
    let mut actor = pool.acquire().await.unwrap();
    let actor_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *actor)
        .await
        .unwrap();
    let sql = account_custody_finalizer_sql();
    let transfer = tokio::spawn(async move {
        sqlx::raw_sql(sqlx::AssertSqlSafe(sql.as_str()))
            .execute(&mut *actor)
            .await
    });
    wait_for_blocker(&pool, actor_pid, lock_pid).await;
    // run_migrations borrows a non-Send SQLx acquisition future; poll both
    // real operations on this runtime instead of substituting migration SQL.
    let controller = async {
        tokio::time::timeout(Duration::from_secs(4), async {
            loop {
                let crossed: bool = sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM _sqlx_migrations WHERE version=165 AND success)",
                )
                .fetch_one(&other)
                .await
                .unwrap();
                if crossed {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("second actual migrator did not cross historical0165 while finalizer was held");
        sqlx::raw_sql("COMMIT").execute(&mut *lock).await.unwrap();
        tokio::time::timeout(Duration::from_secs(70), transfer)
            .await
            .unwrap()
            .unwrap()
    };
    let (transfer_result, migration_result) = tokio::join!(
        controller,
        tokio::time::timeout(
            Duration::from_secs(70),
            console_app::run_migrations(&config)
        )
    );
    let migration_result = migration_result.unwrap();
    assert!(
        transfer_result.is_ok(),
        "finalization failed: {transfer_result:?}"
    );
    assert!(
        migration_result.is_ok(),
        "other actual migrator failed: {migration_result:?}"
    );
    let memberships:Vec<(String,bool,bool,bool)>=sqlx::query_as("SELECT r.rolname,m.admin_option,m.inherit_option,m.set_option FROM pg_auth_members m JOIN pg_roles r ON r.oid=m.roleid WHERE m.member='console_app'::regrole ORDER BY r.rolname").fetch_all(&pool).await.unwrap();
    assert_eq!(
        memberships,
        vec![
            ("console_leave_definer".to_owned(), false, true, true),
            ("console_ontology_writer".to_owned(), false, true, true)
        ]
    );
    assert_owners(&pool, true).await;
    assert_owners(&other, false).await;
    finalize_account_custody(&other).await;
    assert_owners(&other, true).await;
    other.close().await;
    }).catch_unwind().await;
    // The owned database name is known before creation; attempt cleanup even
    // when a prerequisite, assertion or migration panics inside the scenario.
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "DROP DATABASE IF EXISTS \"{database}\" WITH (FORCE)"
    )))
    .execute(&pool)
    .await
    .unwrap();
    if let Err(panic) = outcome {
        std::panic::resume_unwind(panic);
    }
}

struct KillChild(std::process::Child);
impl Drop for KillChild {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[sqlx::test(migrations = false)]
async fn committed_finalization_survives_unobserved_cli_response_and_retry(pool: PgPool) {
    use std::io::Write;
    staged(&pool).await;
    let mut url =
        url::Url::parse(&std::env::var("DATABASE_URL").expect("actual disposable admin transport"))
            .unwrap();
    let db: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(&pool)
        .await
        .unwrap();
    url.set_path(&db);
    assert_eq!(url.username(), "console_buck_admin");
    assert_eq!(url.host_str(), Some(pool.connect_options().get_host()));
    assert_eq!(
        url.port().unwrap_or(5432),
        pool.connect_options().get_port()
    );
    let mut child = KillChild(
        std::process::Command::new("psql")
            .args([
                "-X",
                "-w",
                "--set",
                "ON_ERROR_STOP=1",
                "--host",
                url.host_str().unwrap(),
                "--port",
                &url.port().unwrap_or(5432).to_string(),
                "--username",
                url.username(),
                "--dbname",
                &db,
            ])
            .env(
                "PGPASSWORD",
                url.password().expect("private admin password"),
            )
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("PSQL_PROCESS_PREREQUISITE: installed psql executable required"),
    );
    // Parent never reads stdout: bounded pipe backpressure retains the CLI
    // before the observer can receive its completion. Independent DB readback
    // below, not a reported process result, establishes the actual commit.
    let input = format!(
        "BEGIN;\n{}\nCOMMIT;\nSELECT repeat('x',8192) FROM generate_series(1,1024);\n",
        account_custody_finalizer_sql()
    );
    child
        .0
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    tokio::time::timeout(Duration::from_secs(65),async {
        loop {
            let finalized:bool=sqlx::query_scalar("SELECT pg_get_userbyid(relowner)='console_account_owner' FROM pg_class WHERE oid='public.accounts'::regclass").fetch_one(&pool).await.unwrap();
            if finalized {break;}
            assert!(child.0.try_wait().unwrap().is_none(),"CLI exited before committed custody readback");
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }).await.unwrap();
    assert_owners(&pool, true).await;
    assert!(
        child.0.try_wait().unwrap().is_none(),
        "no actual unobserved CLI was retained"
    );
    child.0.kill().unwrap();
    child.0.wait().unwrap();
    let before = snapshot(&pool).await;
    finalize_account_custody(&pool).await;
    assert_eq!(before, snapshot(&pool).await);
    assert_empty(&pool).await;
}
