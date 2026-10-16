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

async fn assert_backfilled_empty_security(pool: &PgPool, expected_roots: &Value) {
    super::account_root_transition::assert_exact_roots(pool, expected_roots).await;
    for table in TABLES.into_iter().filter(|table| *table != "accounts") {
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
    let expected_roots = super::account_root_transition::expected_roots_after_backfill(&pool).await;
    finalize_account_custody(&pool).await;
    assert_owners(&pool, true).await;
    assert_backfilled_empty_security(&pool, &expected_roots).await;
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
    let mut connection = pool.begin().await.unwrap();
    sqlx::raw_sql("GRANT console_account_owner TO console_rt WITH ADMIN FALSE, INHERIT FALSE, SET TRUE; SAVEPOINT before_finalizer")
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
    connection.rollback().await.unwrap();
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
    let expected_roots = super::account_root_transition::expected_roots_after_backfill(&pool).await;
    finalize_account_custody(&pool).await;
    assert_backfilled_empty_security(&pool, &expected_roots).await;
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

// Aborting a Rust task alone does not prove that its PostgreSQL operation
// stopped. These actors own unpooled connections; join their cancellation and
// observe backend termination before releasing fixture databases.
async fn stop_finalizer<T>(
    pool: &PgPool,
    task: &mut tokio::task::JoinHandle<T>,
    pid: i32,
    joined: bool,
) {
    if !joined {
        task.abort();
        let _ = task.await;
    }
    let _: bool = sqlx::query_scalar(
        "SELECT CASE WHEN EXISTS (SELECT 1 FROM pg_stat_activity WHERE pid=$1) \
         THEN pg_terminate_backend($1, 5000) ELSE true END",
    )
    .bind(pid)
    .fetch_one(pool)
    .await
    .expect("finalizer backend termination could not be observed");
    // The backend can disappear between the catalog observation and signal;
    // absence, rather than the signal result, is the cleanup oracle.
    let absent: bool =
        sqlx::query_scalar("SELECT NOT EXISTS (SELECT 1 FROM pg_stat_activity WHERE pid=$1)")
            .bind(pid)
            .fetch_one(pool)
            .await
            .expect("finalizer backend absence could not be checked");
    assert!(absent, "finalizer backend did not terminate");
}

#[sqlx::test(migrations = false)]
async fn concurrent_finalizers_have_complete_request_outcomes_and_one_final_state(pool: PgPool) {
    staged(&pool).await;
    let expected_roots = super::account_root_transition::expected_roots_after_backfill(&pool).await;
    let sql = account_custody_finalizer_sql();
    let mut first = pool.begin().await.unwrap();
    sqlx::query("SET LOCAL jit=off")
        .execute(&mut *first)
        .await
        .unwrap();
    let mut second = PgConnection::connect_with(&pool.connect_options())
        .await
        .unwrap();
    sqlx::query("SET SESSION jit=off")
        .execute(&mut second)
        .await
        .unwrap();
    let first_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *first)
        .await
        .unwrap();
    let second_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut second)
        .await
        .unwrap();
    sqlx::raw_sql("LOCK TABLE public.organizations, public.users, public.accounts, public.account_security, public.account_security_events, public.account_terms_acceptances, public.account_terms_head, public.account_terms_release_receipts IN ACCESS EXCLUSIVE MODE")
        .execute(&mut *first)
        .await
        .unwrap();
    let second_sql = sql.clone();
    let mut second_task = tokio::spawn(async move {
        sqlx::raw_sql(sqlx::AssertSqlSafe(second_sql.as_str()))
            .execute(&mut second)
            .await
    });
    let outcome = std::panic::AssertUnwindSafe(async {
        wait_for_blocker(&pool, second_pid, first_pid).await;
        sqlx::raw_sql(sqlx::AssertSqlSafe(sql.as_str()))
            .execute(&mut *first)
            .await
            .unwrap();
        first.commit().await.unwrap();
        tokio::time::timeout(Duration::from_secs(65), &mut second_task).await
    })
    .catch_unwind()
    .await;
    let second_outcome = match outcome {
        Ok(Ok(joined)) => joined.unwrap(),
        Ok(Err(error)) => {
            stop_finalizer(&pool, &mut second_task, second_pid, false).await;
            panic!("second finalizer timed out: {error}");
        }
        Err(panic) => {
            stop_finalizer(&pool, &mut second_task, second_pid, false).await;
            std::panic::resume_unwind(panic);
        }
    };
    let requests = [("first", true), ("second", second_outcome.is_ok())];
    assert!(
        complete_successful_requests(&["first", "second"], &requests),
        "finalizer request accounting failed: second={second_outcome:?}"
    );
    assert_owners(&pool, true).await;
    assert_backfilled_empty_security(&pool, &expected_roots).await;
    let before = snapshot(&pool).await;
    finalize_account_custody(&pool).await;
    assert_eq!(before, snapshot(&pool).await);
}

#[sqlx::test(migrations = false)]
async fn blocked_finalizer_times_out_without_partial_transfer_and_retries(pool: PgPool) {
    staged(&pool).await;
    let before = snapshot(&pool).await;
    let mut blocker = pool.begin().await.unwrap();
    sqlx::raw_sql("LOCK TABLE public.accounts IN ACCESS EXCLUSIVE MODE")
        .execute(&mut *blocker)
        .await
        .unwrap();
    let result = finalizer(&pool).await;
    blocker.rollback().await.unwrap();
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
    let mut lock = pool.begin().await.unwrap();
    let lock_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *lock)
        .await
        .unwrap();
    sqlx::raw_sql("LOCK TABLE public.accounts IN ACCESS EXCLUSIVE MODE")
        .execute(&mut *lock)
        .await
        .unwrap();
    let mut actor = PgConnection::connect_with(&pool.connect_options()).await.unwrap();
    let actor_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut actor)
        .await
        .unwrap();
    let sql = account_custody_finalizer_sql();
    let mut transfer = tokio::spawn(async move {
        sqlx::raw_sql(sqlx::AssertSqlSafe(sql.as_str()))
            .execute(&mut actor)
            .await
    });
    let mut transfer_joined = false;
    let concurrent_outcome = std::panic::AssertUnwindSafe(async {
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
        lock.commit().await.unwrap();
        let result = tokio::time::timeout(Duration::from_secs(70), &mut transfer).await;
        transfer_joined = result.is_ok();
        result
    };
    tokio::join!(
        controller,
        tokio::time::timeout(
            Duration::from_secs(70),
            console_app::run_migrations(&config)
        )
    )
    }).catch_unwind().await;
    let (transfer_result, migration_result) = match concurrent_outcome {
        Ok((Ok(joined), migration)) => (joined.unwrap(), migration),
        Ok((Err(error), _)) => {
            stop_finalizer(&pool, &mut transfer, actor_pid, transfer_joined).await;
            panic!("finalizer timed out: {error}");
        }
        Err(panic) => {
            stop_finalizer(&pool, &mut transfer, actor_pid, transfer_joined).await;
            std::panic::resume_unwind(panic);
        }
    };
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
    let expected_roots = super::account_root_transition::expected_roots_after_backfill(&pool).await;
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
    assert_backfilled_empty_security(&pool, &expected_roots).await;
}

// Exercise the shared wrapper transaction, not an extracted historical SQL body.
async fn helper_actor(pool: &PgPool) -> (PgPool, i32) {
    let options = pool
        .connect_options()
        .as_ref()
        .clone()
        .options([("default_transaction_isolation", "repeatable read")]);
    let actor = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .unwrap();
    let (pid, isolation): (i32, String) =
        sqlx::query_as("SELECT pg_backend_pid(), current_setting('transaction_isolation')")
            .fetch_one(&actor)
            .await
            .unwrap();
    assert_eq!(
        isolation, "repeatable read",
        "positive control for explicit helper isolation"
    );
    (actor, pid)
}

async fn require_helper_read_committed(pool: &PgPool) {
    sqlx::raw_sql(
        r#"
        CREATE FUNCTION public.lc_helper_isolation() RETURNS event_trigger LANGUAGE plpgsql AS $$
        BEGIN
            IF current_setting('transaction_isolation') <> 'read committed' THEN
                RAISE EXCEPTION 'LC_HELPER_REQUIRES_READ_COMMITTED';
            END IF;
        END $$;
        CREATE EVENT TRIGGER lc_helper_isolation ON ddl_command_start
        WHEN TAG IN ('ALTER TABLE') EXECUTE FUNCTION public.lc_helper_isolation();
    "#,
    )
    .execute(pool)
    .await
    .unwrap();
}

async fn helper_anchor_wait(pool: &PgPool, actor: i32, peer: i32, holder: i32) {
    tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            let observed: bool = sqlx::query_scalar(r#"
                SELECT EXISTS(SELECT 1 FROM pg_stat_activity a WHERE a.pid=$1
                  AND a.query LIKE '%pg_catalog.pg_authid%'
                  AND a.query LIKE '%console_account_owner%'
                  AND a.wait_event_type='Lock'
                  AND ($3=ANY(pg_blocking_pids(a.pid))
                    OR ($2=ANY(pg_blocking_pids(a.pid)) AND $3=ANY(pg_blocking_pids($2)))))
                AND EXISTS(SELECT 1 FROM pg_locks WHERE pid=$1
                  AND relation='pg_catalog.pg_authid'::regclass AND database=0
                  AND mode='RowShareLock' AND granted)
            "#).bind(actor).bind(peer).bind(holder).fetch_one(pool).await.unwrap();
            if observed { break; }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }).await.expect("actual helper must wait on shared Account owner anchor (directly or behind its known peer)");
}

async fn custody_profiles(pool: &PgPool) -> (String, String) {
    let mut tx = pool.begin().await.unwrap();
    sqlx::raw_sql(include_str!("../../src/account_custody_session.sql"))
        .execute(tx.as_mut())
        .await
        .unwrap();
    let root = sqlx::query_scalar(include_str!("../../src/account_custody_state.sql"))
        .fetch_one(tx.as_mut())
        .await
        .unwrap();
    let credentials = sqlx::query_scalar(include_str!(
        "../../src/account_credential_custody_state.sql"
    ))
    .fetch_one(tx.as_mut())
    .await
    .unwrap();
    tx.rollback().await.unwrap();
    (root, credentials)
}

async fn cross_database_helpers(pool: PgPool, both_serving: bool) {
    staged(&pool).await;
    let unique = uuid::Uuid::new_v4().simple().to_string();
    let database = format!("_sqlx_test_{unique}{}", &unique[..20]);
    let outcome = std::panic::AssertUnwindSafe(async {
        sqlx::query(sqlx::AssertSqlSafe(format!("CREATE DATABASE \"{database}\"")))
            .execute(&pool).await.unwrap();
        let other = sqlx::postgres::PgPoolOptions::new().max_connections(4)
            .connect_with(pool.connect_options().as_ref().clone().database(&database))
            .await.unwrap();
        prepare_http_database_staging(&other).await;
        require_helper_read_committed(&pool).await;
        require_helper_read_committed(&other).await;
        let before = snapshot(&pool).await;
        let other_before = snapshot(&other).await;
        let credential_role_before: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM pg_roles WHERE rolname='console_credential_owner')"
        ).fetch_one(&pool).await.unwrap();
        println!("HELPER_FIRST_CREDENTIAL_ROLE_CREATION={}", !credential_role_before);
        let (root_actor, root_pid) = helper_actor(&pool).await;
        let (serving_actor, serving_pid) = helper_actor(&other).await;
        let mut holder = pool.begin().await.unwrap();
        let holder_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut *holder).await.unwrap();
        let _: String = sqlx::query_scalar(
            "SELECT oid::text FROM pg_catalog.pg_authid WHERE rolname='console_account_owner' FOR UPDATE"
        ).fetch_one(&mut *holder).await.unwrap();
        let root_connection = root_actor.clone();
        let serving_connection = serving_actor.clone();
        let mut root_task = tokio::spawn(async move {
            if both_serving { console_platform_test_support::finalize_serving_account_custody(&root_connection).await; }
            else { console_platform_test_support::finalize_account_custody(&root_connection).await; }
        });
        let mut serving_task = tokio::spawn(async move {
            console_platform_test_support::finalize_serving_account_custody(&serving_connection).await;
        });
        let mut root_joined = false;
        let mut serving_joined = false;
        let concurrent = std::panic::AssertUnwindSafe(async {
            let observation_start = std::time::Instant::now();
            tokio::time::timeout(Duration::from_secs(4), async {
                tokio::join!(
                    helper_anchor_wait(&pool, root_pid, serving_pid, holder_pid),
                    helper_anchor_wait(&pool, serving_pid, root_pid, holder_pid));
                assert!(!root_task.is_finished() && !serving_task.is_finished());
                let (root_snapshot, serving_snapshot) = tokio::join!(snapshot(&pool), snapshot(&other));
                assert_eq!(before, root_snapshot);
                assert_eq!(other_before, serving_snapshot);
            }).await.expect("observe both blocked helpers and unpublished state within lock budget");
            println!("HELPER_ANCHOR_OBSERVATION_MS={}", observation_start.elapsed().as_millis());
            holder.rollback().await.unwrap();
            let root_result = tokio::time::timeout(Duration::from_secs(65), &mut root_task).await;
            root_joined = root_result.is_ok();
            let serving_result = tokio::time::timeout(Duration::from_secs(65), &mut serving_task).await;
            serving_joined = serving_result.is_ok();
            let requests = [("root", root_result.unwrap().is_ok()),
                ("serving", serving_result.unwrap().is_ok())];
            assert!(complete_successful_requests(&["root", "serving"], &requests));
            assert_owners(&pool, true).await;
            assert_owners(&other, true).await;
            assert_eq!(custody_profiles(&pool).await.0, if both_serving { "account_custody.native_finalized" } else { "account_custody.finalized" });
            if both_serving {
                assert_eq!(custody_profiles(&pool).await.1, "account_credentials.native_finalized");
            }
            assert_eq!(custody_profiles(&other).await,
                ("account_custody.native_finalized".to_owned(), "account_credentials.native_finalized".to_owned()));
            let credential_role_after: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM pg_roles WHERE rolname='console_credential_owner')"
            ).fetch_one(&pool).await.unwrap();
            assert!(credential_role_after);
        }).catch_unwind().await;
        stop_finalizer(&pool, &mut root_task, root_pid, root_joined).await;
        stop_finalizer(&pool, &mut serving_task, serving_pid, serving_joined).await;
        root_actor.close().await;
        serving_actor.close().await;
        other.close().await;
        if let Err(panic) = concurrent { std::panic::resume_unwind(panic); }
    }).catch_unwind().await;
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

#[sqlx::test(migrations = false)]
async fn helper_finalizers_share_anchor_across_databases(pool: PgPool) {
    cross_database_helpers(pool, false).await;
}

#[sqlx::test(migrations = false)]
async fn helper_finalizers_serialize_credential_creation_across_databases(pool: PgPool) {
    cross_database_helpers(pool, true).await;
}

#[sqlx::test(migrations = false)]
async fn helper_finalizers_release_anchor_after_failure_and_retry(pool: PgPool) {
    staged(&pool).await;
    require_helper_read_committed(&pool).await;
    for serving in [false, true] {
        let cold = !credential_role_committed(&pool).await;
        println!("HELPER_FAILURE_REQUIRES_RETAINED_ANCHOR={cold};serving={serving}");
        let before = snapshot(&pool).await;
        let profiles_before = custody_profiles(&pool).await;
        let (actor, pid) = helper_actor(&pool).await;
        let mut holder = pool.begin().await.unwrap();
        let holder_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut *holder)
            .await
            .unwrap();
        sqlx::raw_sql("LOCK TABLE public.accounts IN ACCESS EXCLUSIVE MODE")
            .execute(&mut *holder)
            .await
            .unwrap();
        let connection = actor.clone();
        let mut task = tokio::spawn(async move {
            if serving {
                console_platform_test_support::finalize_serving_account_custody(&connection).await;
            } else {
                console_platform_test_support::finalize_account_custody(&connection).await;
            }
        });
        let mut joined = false;
        let outcome = std::panic::AssertUnwindSafe(async {
            wait_for_blocker(&pool, pid, holder_pid).await;
            // Reviewed savepoint contract: retain only while the shared role is
            // absent. Warm local-work failure must not pin the cluster anchor.
            assert_anchor_retention(&pool, cold).await;
            let result = tokio::time::timeout(Duration::from_secs(8), &mut task).await;
            joined = result.is_ok();
            let panic = result.unwrap().expect_err("actual helper must time out on held business relation").into_panic();
            let message = panic.downcast_ref::<String>().map(String::as_str)
                .or_else(|| panic.downcast_ref::<&str>().copied()).unwrap_or("");
            assert!(message.contains("55P03"), "wrong helper failure: {message}");
            holder.rollback().await.unwrap();
            // Acquiring the pooled backend flushes SQLx's queued rollback.
            let _: i32 = sqlx::query_scalar("SELECT 1::int").fetch_one(&actor).await.unwrap();
            let mut retry = pool.begin().await.unwrap();
            let _: String = sqlx::query_scalar(
                "SELECT oid::text FROM pg_catalog.pg_authid WHERE rolname='console_account_owner' FOR UPDATE NOWAIT"
            ).fetch_one(&mut *retry).await.unwrap();
            retry.rollback().await.unwrap();
            assert_eq!(before, snapshot(&pool).await);
            assert_eq!(profiles_before, custody_profiles(&pool).await);
            if serving { console_platform_test_support::finalize_serving_account_custody(&actor).await; }
            else { console_platform_test_support::finalize_account_custody(&actor).await; }
            assert_owners(&pool, true).await;
        }).catch_unwind().await;
        stop_finalizer(&pool, &mut task, pid, joined).await;
        actor.close().await;
        if let Err(panic) = outcome {
            std::panic::resume_unwind(panic);
        }
    }
    assert_eq!(
        custody_profiles(&pool).await,
        (
            "account_custody.native_finalized".to_owned(),
            "account_credentials.native_finalized".to_owned()
        )
    );
}

async fn credential_role_committed(pool: &PgPool) -> bool {
    sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_roles WHERE rolname='console_credential_owner')",
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

// The probe always rolls back, including a failed NOWAIT transaction. It never
// creates, changes or removes either shared role.
async fn assert_anchor_retention(pool: &PgPool, retained: bool) {
    let mut probe = pool.begin().await.unwrap();
    let result = sqlx::query_scalar::<_, String>(
        "SELECT oid::text FROM pg_catalog.pg_authid WHERE rolname='console_account_owner' FOR UPDATE NOWAIT",
    ).fetch_one(&mut *probe).await;
    probe.rollback().await.unwrap();
    if retained {
        let error =
            result.expect_err("cold helper must retain the anchor through outer completion");
        assert_eq!(
            error.as_database_error().unwrap().code().as_deref(),
            Some("55P03")
        );
    } else {
        assert!(
            !result
                .expect("warm helper must release anchor before database-local work")
                .is_empty()
        );
    }
}

async fn helper_local_progress(pool: &PgPool, actor: i32, holder: i32) {
    wait_for_blocker(pool, actor, holder).await;
    let local_lock: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(SELECT 1 FROM pg_locks WHERE pid=$1
          AND database=(SELECT oid FROM pg_database WHERE datname=current_database())
          AND relation='public.organizations'::regclass
          AND mode='AccessExclusiveLock' AND granted)
        AND $2=ANY(pg_blocking_pids($1))
    "#,
    )
    .bind(actor)
    .bind(holder)
    .fetch_one(pool)
    .await
    .unwrap();
    assert!(
        local_lock,
        "actual helper must enter its own database's historical finalizer while a peer is blocked"
    );
}

// Both helpers below execute the unchanged historical SQL. Hold their local
// accounts relations so completed progress observations cannot be timing luck.
async fn warm_helper_progress(pool: PgPool, first_serving: bool) {
    staged(&pool).await;
    // Warm topology only through the actual owning serving finalizer.
    console_platform_test_support::finalize_serving_account_custody(&pool).await;
    assert!(credential_role_committed(&pool).await);
    assert_eq!(
        custody_profiles(&pool).await,
        (
            "account_custody.native_finalized".to_owned(),
            "account_credentials.native_finalized".to_owned()
        )
    );
    let unique = uuid::Uuid::new_v4().simple().to_string();
    let database = format!("_sqlx_test_{unique}{}", &unique[..20]);
    let outcome = std::panic::AssertUnwindSafe(async {
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "CREATE DATABASE \"{database}\""
        )))
        .execute(&pool)
        .await
        .unwrap();
        let other = sqlx::postgres::PgPoolOptions::new()
            .max_connections(4)
            .connect_with(pool.connect_options().as_ref().clone().database(&database))
            .await
            .unwrap();
        prepare_http_database_staging(&other).await;
        require_helper_read_committed(&pool).await;
        require_helper_read_committed(&other).await;
        let (first_actor, first_pid) = helper_actor(&pool).await;
        let (second_actor, second_pid) = helper_actor(&other).await;
        let mut first_holder = pool.begin().await.unwrap();
        let first_holder_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut *first_holder)
            .await
            .unwrap();
        sqlx::raw_sql("LOCK TABLE public.accounts IN ACCESS EXCLUSIVE MODE")
            .execute(&mut *first_holder)
            .await
            .unwrap();
        let mut second_holder = other.begin().await.unwrap();
        let second_holder_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut *second_holder)
            .await
            .unwrap();
        sqlx::raw_sql("LOCK TABLE public.accounts IN ACCESS EXCLUSIVE MODE")
            .execute(&mut *second_holder)
            .await
            .unwrap();
        let first_connection = first_actor.clone();
        let second_connection = second_actor.clone();
        let mut first_task = tokio::spawn(async move {
            if first_serving {
                console_platform_test_support::finalize_serving_account_custody(&first_connection)
                    .await;
            } else {
                console_platform_test_support::finalize_account_custody(&first_connection).await;
            }
        });
        let mut second_task = tokio::spawn(async move {
            console_platform_test_support::finalize_serving_account_custody(&second_connection)
                .await;
        });
        let mut first_joined = false;
        let mut second_joined = false;
        let concurrent = std::panic::AssertUnwindSafe(async {
            let started = std::time::Instant::now();
            tokio::time::timeout(Duration::from_secs(4), async {
                tokio::join!(
                    helper_local_progress(&pool, first_pid, first_holder_pid),
                    helper_local_progress(&other, second_pid, second_holder_pid)
                );
                assert!(!first_task.is_finished() && !second_task.is_finished());
                assert_anchor_retention(&pool, false).await;
            })
            .await
            .expect("both warm helpers must progress within unchanged lock deadline");
            println!(
                "HELPER_WARM_LOCAL_PROGRESS_MS={}",
                started.elapsed().as_millis()
            );
            first_holder.rollback().await.unwrap();
            second_holder.rollback().await.unwrap();
            let first_result = tokio::time::timeout(Duration::from_secs(65), &mut first_task).await;
            first_joined = first_result.is_ok();
            let second_result =
                tokio::time::timeout(Duration::from_secs(65), &mut second_task).await;
            second_joined = second_result.is_ok();
            assert!(complete_successful_requests(
                &["first", "second"],
                &[
                    ("first", first_result.unwrap().is_ok()),
                    ("second", second_result.unwrap().is_ok())
                ]
            ));
            let expected = (
                "account_custody.native_finalized".to_owned(),
                "account_credentials.native_finalized".to_owned(),
            );
            assert_eq!(custody_profiles(&pool).await, expected);
            assert_eq!(custody_profiles(&other).await, expected);
            assert_owners(&pool, true).await;
            assert_owners(&other, true).await;
            assert_anchor_retention(&pool, false).await;
        })
        .catch_unwind()
        .await;
        stop_finalizer(&pool, &mut first_task, first_pid, first_joined).await;
        stop_finalizer(&pool, &mut second_task, second_pid, second_joined).await;
        first_actor.close().await;
        second_actor.close().await;
        other.close().await;
        if let Err(panic) = concurrent {
            std::panic::resume_unwind(panic);
        }
    })
    .catch_unwind()
    .await;
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

#[sqlx::test(migrations = false)]
async fn helper_finalizers_warm_root_allows_peer_database_progress(pool: PgPool) {
    warm_helper_progress(pool, false).await;
}

#[sqlx::test(migrations = false)]
async fn helper_finalizers_warm_serving_allows_peer_database_progress(pool: PgPool) {
    warm_helper_progress(pool, true).await;
}

#[sqlx::test(migrations = false)]
async fn helper_finalizers_recheck_role_after_creator_commit(pool: PgPool) {
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
        let other = sqlx::postgres::PgPoolOptions::new()
            .max_connections(4)
            .connect_with(pool.connect_options().as_ref().clone().database(&database))
            .await
            .unwrap();
        prepare_http_database_staging(&other).await;
        require_helper_read_committed(&pool).await;
        require_helper_read_committed(&other).await;
        let cold = !credential_role_committed(&pool).await;
        println!("HELPER_POST_WAIT_FIRST_CREATION={cold}");
        let (creator_actor, creator_pid) = helper_actor(&pool).await;
        let (waiter_actor, waiter_pid) = helper_actor(&other).await;
        let mut creator_holder = pool.begin().await.unwrap();
        let creator_holder_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut *creator_holder)
            .await
            .unwrap();
        sqlx::raw_sql("LOCK TABLE public.accounts IN ACCESS EXCLUSIVE MODE")
            .execute(&mut *creator_holder)
            .await
            .unwrap();
        let mut waiter_holder = other.begin().await.unwrap();
        let waiter_holder_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut *waiter_holder)
            .await
            .unwrap();
        sqlx::raw_sql("LOCK TABLE public.accounts IN ACCESS EXCLUSIVE MODE")
            .execute(&mut *waiter_holder)
            .await
            .unwrap();
        let creator_connection = creator_actor.clone();
        let mut creator_task = tokio::spawn(async move {
            console_platform_test_support::finalize_serving_account_custody(&creator_connection)
                .await;
        });
        let mut creator_joined = false;
        // Store the waiter handle as soon as it exists so every panic path
        // cancels/observes its backend before dropping the fixture database.
        let mut waiter_task = None;
        let mut waiter_joined = false;
        let concurrent = std::panic::AssertUnwindSafe(async {
            helper_local_progress(&pool, creator_pid, creator_holder_pid).await;
            assert_anchor_retention(&pool, cold).await;
            assert!(!creator_task.is_finished());
            let waiter_connection = waiter_actor.clone();
            waiter_task = Some(tokio::spawn(async move {
                console_platform_test_support::finalize_serving_account_custody(&waiter_connection)
                    .await;
            }));
            if cold {
                helper_anchor_wait(&pool, waiter_pid, creator_pid, creator_pid).await;
                assert!(
                    !credential_role_committed(&pool).await,
                    "creator has not yet published its role"
                );
                assert!(!waiter_task.as_ref().unwrap().is_finished());
                println!("HELPER_POST_WAIT_COLD_BARRIER_OBSERVED=true");
            } else {
                // Warm suite still executes complete synchronization/progress
                // assertions; only a fresh filtered run may claim cold proof.
                helper_local_progress(&other, waiter_pid, waiter_holder_pid).await;
                assert_anchor_retention(&pool, false).await;
            }
            creator_holder.rollback().await.unwrap();
            let result = tokio::time::timeout(Duration::from_secs(65), &mut creator_task).await;
            creator_joined = result.is_ok();
            assert!(result.unwrap().is_ok(), "actual creator must commit");
            assert!(credential_role_committed(&pool).await);
            helper_local_progress(&other, waiter_pid, waiter_holder_pid).await;
            assert!(!waiter_task.as_ref().unwrap().is_finished());
            // Decisive stale-snapshot/recheck oracle: an initially cold waiter
            // must now release the anchor even while its own SQL remains blocked.
            assert_anchor_retention(&pool, false).await;
            println!("HELPER_POST_WAIT_ANCHOR_RELEASED=true");
            waiter_holder.rollback().await.unwrap();
            let result =
                tokio::time::timeout(Duration::from_secs(65), waiter_task.as_mut().unwrap()).await;
            waiter_joined = result.is_ok();
            assert!(result.unwrap().is_ok(), "actual waiter must finalize");
            let expected = (
                "account_custody.native_finalized".to_owned(),
                "account_credentials.native_finalized".to_owned(),
            );
            assert_eq!(custody_profiles(&pool).await, expected);
            assert_eq!(custody_profiles(&other).await, expected);
            assert_owners(&pool, true).await;
            assert_owners(&other, true).await;
        })
        .catch_unwind()
        .await;
        stop_finalizer(&pool, &mut creator_task, creator_pid, creator_joined).await;
        if let Some(task) = waiter_task.as_mut() {
            stop_finalizer(&pool, task, waiter_pid, waiter_joined).await;
        }
        creator_actor.close().await;
        waiter_actor.close().await;
        other.close().await;
        if let Err(panic) = concurrent {
            std::panic::resume_unwind(panic);
        }
    })
    .catch_unwind()
    .await;
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
