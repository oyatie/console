//! Root-only transition regression candidate. Real historical/current operator
//! SQL and LOGIN roles; privileged data/fault fixtures confer no Account authority.
use super::{account_custody_finalizer_sql, prepare_http_database, prepare_http_database_staging};
use console_kernel_core::OrgId;
use console_platform_test_support::{TestDatabaseLogin, login_test_pool};
use futures::FutureExt;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::{Connection, PgConnection, PgPool, postgres::PgQueryResult};
use std::{collections::BTreeMap, time::Duration};
use uuid::Uuid;

const TABLES: &[&str] = &[
    "accounts",
    "account_security",
    "account_security_events",
    "account_terms_acceptances",
    "account_terms_head",
    "account_terms_release_receipts",
    "auth_webauthn_credentials",
    "auth_webauthn_ceremonies",
    "auth_refresh_token_families",
    "auth_refresh_tokens",
    "auth_bootstrap_credentials",
    "auth_webauthn_ceremony_bindings",
    "auth_device_login_handoffs",
    "users",
    "audit_events",
];
const LEGACY: &[&str] = &[
    "auth_webauthn_credentials",
    "auth_webauthn_ceremonies",
    "auth_refresh_token_families",
    "auth_refresh_tokens",
    "auth_bootstrap_credentials",
    "auth_webauthn_ceremony_bindings",
    "auth_device_login_handoffs",
];
const BRIDGE: &str = "00_account_legacy_user_root_v1";

fn id(n: u128) -> Uuid {
    Uuid::from_u128(0x9ab00000000040008000000000000000 + n)
}

async fn rows(connection: &mut PgConnection) -> Value {
    let mut result = serde_json::Map::new();
    for table in TABLES {
        let sql = format!(
            "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text),'[]'::jsonb) FROM public.{table} t"
        );
        let value: Value = sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
            .fetch_one(&mut *connection)
            .await
            .unwrap();
        result.insert((*table).to_owned(), value);
    }
    Value::Object(result)
}

async fn metadata(connection: &mut PgConnection, tables: &[&str]) -> Value {
    sqlx::query_scalar(r#"SELECT jsonb_build_object(
      'relations',(SELECT jsonb_agg(jsonb_build_object('name',c.relname,'oid',c.oid,'owner',c.relowner,'acl',c.relacl,
        'rls',c.relrowsecurity,'force',c.relforcerowsecurity,
        'columns',(SELECT jsonb_agg(to_jsonb(a) ORDER BY a.attnum) FROM pg_attribute a WHERE a.attrelid=c.oid),
        'constraints',(SELECT COALESCE(jsonb_agg(to_jsonb(k) ORDER BY k.oid),'[]'::jsonb) FROM pg_constraint k WHERE k.conrelid=c.oid),
        'triggers',(SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY t.oid),'[]'::jsonb) FROM pg_trigger t WHERE t.tgrelid=c.oid),
        'policies',(SELECT COALESCE(jsonb_agg(to_jsonb(p) ORDER BY p.oid),'[]'::jsonb) FROM pg_policy p WHERE p.polrelid=c.oid)) ORDER BY c.relname)
        FROM pg_class c WHERE c.relnamespace='public'::regnamespace AND c.relname=ANY($1)),
      'functions',(SELECT COALESCE(jsonb_agg(to_jsonb(p) ORDER BY p.oid),'[]'::jsonb) FROM pg_proc p
        WHERE p.pronamespace='public'::regnamespace AND p.proname LIKE 'account\_%' ESCAPE '\'),
      'ledger',(SELECT jsonb_agg(to_jsonb(m) ORDER BY version) FROM public._sqlx_migrations m))"#)
        .bind(tables).fetch_one(connection).await.unwrap()
}

pub(super) async fn expected_roots_after_backfill(pool: &PgPool) -> Value {
    let mut roots: BTreeMap<String, Value> = BTreeMap::new();
    let existing: Vec<Value> = sqlx::query_scalar("SELECT to_jsonb(a) FROM public.accounts a")
        .fetch_all(pool)
        .await
        .unwrap();
    for row in existing {
        assert!(
            roots
                .insert(row["id"].as_str().unwrap().to_owned(), row)
                .is_none()
        );
    }
    let users: Vec<Value> = sqlx::query_scalar(
        "SELECT jsonb_build_object('id',id,'created_at',created_at) FROM public.users",
    )
    .fetch_all(pool)
    .await
    .unwrap();
    for row in users {
        roots
            .entry(row["id"].as_str().unwrap().to_owned())
            .or_insert(row);
    }
    let mut values: Vec<_> = roots.into_values().collect();
    // Match the complete-row ordering used by rows(), independent of insertion order.
    values.sort_by_key(Value::to_string);
    Value::Array(values)
}

pub(super) async fn assert_exact_roots(pool: &PgPool, expected: &Value) {
    let actual: Vec<Value> = sqlx::query_scalar("SELECT to_jsonb(a) FROM public.accounts a")
        .fetch_all(pool)
        .await
        .unwrap();
    let mut actual = actual;
    actual.sort_by_key(Value::to_string);
    assert_eq!(
        &Value::Array(actual),
        expected,
        "exact retained plus copied Account roots"
    );
}

async fn transition(connection: &mut PgConnection) -> Result<(), sqlx::Error> {
    sqlx::raw_sql("SET LOCAL statement_timeout='60s'")
        .execute(&mut *connection)
        .await?;
    sqlx::raw_sql(sqlx::AssertSqlSafe(account_custody_finalizer_sql()))
        .execute(connection)
        .await?;
    Ok(())
}

async fn historical(pool: &PgPool) {
    prepare_http_database_staging(pool).await;
    let sql = include_str!("fixtures/account-custody-root-input-d66e2112.sql");
    assert_eq!(
        hex::encode(Sha256::digest(sql)),
        "9726edda826cb2c7496b73744f5189adaf14f0cee2a445e41c7202df09943496"
    );
    let mut tx = pool.begin().await.unwrap();
    sqlx::raw_sql(sql).execute(&mut *tx).await.unwrap();
    tx.commit().await.unwrap();
}

async fn company(connection: &mut PgConnection) {
    sqlx::query("SELECT set_config('app.current_org',$1,true)")
        .bind(OrgId::knl().to_string())
        .execute(connection)
        .await
        .unwrap();
}

async fn insert_user(
    connection: &mut PgConnection,
    user: Uuid,
) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query("INSERT INTO public.users(id,display_name,roles,org_id,created_at) VALUES($1,'ROOT TRANSITION TEST ONLY',ARRAY['MECHANIC']::text[],$2,'2026-09-14T01:02:03.123456Z')")
        .bind(user).bind(OrgId::knl().as_uuid()).execute(connection).await
}

async fn insert_root(
    connection: &mut PgConnection,
    root: Uuid,
) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query("INSERT INTO public.accounts(id,created_at) VALUES($1,'2026-09-14T04:05:06Z')")
        .bind(root)
        .execute(connection)
        .await
}

fn db_error<T>(
    result: Result<T, sqlx::Error>,
    code: &str,
    message: Option<&str>,
    constraint: Option<&str>,
) {
    let error = result.err().expect("required refusal did not occur");
    let db = error.as_database_error().expect("actual PostgreSQL error");
    assert_eq!(db.code().as_deref(), Some(code));
    if let Some(message) = message {
        assert_eq!(db.message(), message);
    }
    if let Some(constraint) = constraint {
        assert_eq!(db.constraint(), Some(constraint));
    }
}

async fn exact_user_root(connection: &mut PgConnection, user: Uuid) {
    let exact: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM public.users u JOIN public.accounts a ON a.id=u.id WHERE u.id=$1 AND a.created_at=u.created_at) AND NOT EXISTS(SELECT 1 FROM public.account_security WHERE account_id=$1)")
        .bind(user).fetch_one(connection).await.unwrap();
    assert!(
        exact,
        "actual insert must create exact unusable legacy root, without security"
    );
}

async fn require_bridge(connection: &mut PgConnection) {
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM pg_trigger WHERE tgrelid='public.users'::regclass AND tgname=$1 AND NOT tgisinternal AND tgenabled='A' AND tgtype=5")
        .bind(BRIDGE).fetch_one(connection).await.unwrap();
    assert_eq!(
        count, 1,
        "ROOT_BRIDGE_NOT_INSTALLED: later behavioral assertions unreached"
    );
}

async fn seed_legacy(connection: &mut PgConnection) {
    insert_user(connection, id(1)).await.unwrap();
    let org = *OrgId::knl().as_uuid();
    sqlx::query("INSERT INTO auth_webauthn_credentials(id,user_id,org_id,credential_id,passkey_json,last_used_at) VALUES($1,$2,$3,'TEST_ONLY retained key',jsonb_build_object('TEST_ONLY_counter',17),'2026-09-14T01:00:00Z')")
        .bind(id(11)).bind(id(1)).bind(org).execute(&mut *connection).await.unwrap();
    sqlx::query("INSERT INTO auth_webauthn_ceremonies(id,user_id,ceremony_kind,challenge_json,state_json,created_at,expires_at,consumed_at) VALUES($1,$2,'authentication','{\"TEST_ONLY\":true}','{\"retained\":17}','2026-09-14T00:00:00Z','2026-09-14T01:00:00Z','2026-09-14T00:30:00Z')")
        .bind(id(12)).bind(id(1)).execute(&mut *connection).await.unwrap();
    sqlx::query("INSERT INTO auth_refresh_token_families(id,user_id,org_id,created_at,revoked_at,revoked_reason) VALUES($1,$2,$3,'2026-09-14T00:00:00Z','2026-09-14T01:00:00Z','TEST_ONLY retained revocation')")
        .bind(id(13)).bind(id(1)).bind(org).execute(&mut *connection).await.unwrap();
    sqlx::query("INSERT INTO auth_refresh_tokens(id,family_id,user_id,org_id,token_hash,issued_at,expires_at,used_at,revoked_at,reuse_detected_at) VALUES($1,$2,$3,$4,decode(repeat('71',32),'hex'),'2026-09-14T00:00:00Z','2026-09-15T00:00:00Z','2026-09-14T01:00:00Z','2026-09-14T02:00:00Z','2026-09-14T03:00:00Z')")
        .bind(id(14)).bind(id(13)).bind(id(1)).bind(org).execute(&mut *connection).await.unwrap();
    sqlx::query("INSERT INTO auth_bootstrap_credentials(id,user_id,org_id,token_hash,issued_at,expires_at,consumed_at,revoked_reason) VALUES($1,$2,$3,decode(repeat('72',32),'hex'),'2026-09-14T00:00:00Z','2026-09-15T00:00:00Z','2026-09-14T01:00:00Z','TEST_ONLY retained bootstrap')")
        .bind(id(15)).bind(id(1)).bind(org).execute(&mut *connection).await.unwrap();
    sqlx::query("INSERT INTO auth_webauthn_ceremony_bindings(ceremony_id,action_kind,object_id,reason_key,replay_attempt) VALUES($1,'POLL_VOTE',$2,'operations_passkey_poll_vote',7)")
        .bind(id(12)).bind(id(16)).execute(&mut *connection).await.unwrap();
    sqlx::query("INSERT INTO auth_device_login_handoffs(id,poll_token_hash,approve_token_hash,issued_at,expires_at,target_user_id,target_org_id,approved_at,approved_user_id,approved_org_id,approved_passkey_id,consumed_at) VALUES($1,decode(repeat('73',32),'hex'),decode(repeat('74',32),'hex'),'2026-09-14T00:00:00Z','2026-09-14T01:00:00Z',$2,$3,'2026-09-14T00:30:00Z',$2,$3,$4,'2026-09-14T00:45:00Z')")
        .bind(id(17)).bind(id(1)).bind(org).bind(id(11)).execute(connection).await.unwrap();
}

#[sqlx::test(migrations = false)]
async fn populated_upgrade_preserves_rows_roots_and_legacy_acl(pool: PgPool) {
    historical(&pool).await;
    let mut connection = pool.acquire().await.unwrap();
    seed_legacy(&mut connection).await;
    drop(connection);
    super::insert_account_fence(
        &pool,
        console_kernel_core::UserId::from_uuid(id(90)),
        "RECOVERY_REQUIRED",
    )
    .await;
    super::account_browser::seed_terms(&pool).await;
    // Privileged retained-data fixture only. Execute real native tuple FKs,
    // restoring the exact dormant parent ACL before certifying the old profile.
    let mut fixture = pool.begin().await.unwrap();
    let acl_before: (Option<String>,Value) = sqlx::query_as("SELECT c.relacl::text,(SELECT jsonb_agg(jsonb_build_array(attnum,attacl::text) ORDER BY attnum) FROM pg_attribute WHERE attrelid=c.oid AND attnum>0) FROM pg_class c WHERE c.oid='public.account_security_events'::regclass")
        .fetch_one(&mut *fixture).await.unwrap();
    sqlx::raw_sql(
        "GRANT SELECT,UPDATE ON TABLE public.account_security_events TO console_account_owner",
    )
    .execute(&mut *fixture)
    .await
    .unwrap();
    let evidence = json!({"fixture_only":true});
    let payload = json!({"kind":"TERMS_ACCEPTED","account_id":id(90).to_string(),"before_generation":"1","after_generation":"1","credential_id":null,"session_id":null,"evidence":evidence.clone()});
    sqlx::query("INSERT INTO account_security_events(id,account_id,kind,occurred_at,actor_account_id,session_id,evidence_ref,payload) VALUES($1,$2,'TERMS_ACCEPTED',now(),$2,NULL,$3,$4)")
        .bind(id(91)).bind(id(90)).bind(&evidence).bind(payload).execute(&mut *fixture).await.unwrap();
    // Kind/content are the exact existing BW31 privacy fixture; current head
    // supplies the real release tuple. This is retained test data, not consent.
    sqlx::query("INSERT INTO account_terms_acceptances(account_id,terms_kind,terms_version,content_sha256,accepted_at,security_event_id,terms_release_receipt_id,terms_release_revision,terms_manifest_sha256) SELECT $1,'test.account.privacy',encode(h.manifest_sha256,'hex'),decode('4254fa64b821906ac2995f0829590ca151540929b05594a023afc99d228c23d2','hex'),now(),$2,h.release_receipt_ref,h.revision,h.manifest_sha256 FROM public.account_terms_head h WHERE h.id=1")
        .bind(id(90)).bind(id(91)).execute(&mut *fixture).await.unwrap();
    sqlx::raw_sql(
        "REVOKE SELECT,UPDATE ON TABLE public.account_security_events FROM console_account_owner",
    )
    .execute(&mut *fixture)
    .await
    .unwrap();
    let acl_after: (Option<String>,Value) = sqlx::query_as("SELECT c.relacl::text,(SELECT jsonb_agg(jsonb_build_array(attnum,attacl::text) ORDER BY attnum) FROM pg_attribute WHERE attrelid=c.oid AND attnum>0) FROM pg_class c WHERE c.oid='public.account_security_events'::regclass")
        .fetch_one(&mut *fixture).await.unwrap();
    assert_eq!(
        acl_after, acl_before,
        "fixture-only parent grants must restore exact table and column ACLs"
    );
    sqlx::raw_sql(include_str!(
        "fixtures/account-custody-root-input-d66e2112.sql"
    ))
    .execute(&mut *fixture)
    .await
    .expect("exact historical operator profile restored before retention snapshot");
    fixture.commit().await.unwrap();
    let expected_roots = expected_roots_after_backfill(&pool).await;
    let mut tx = pool.begin().await.unwrap();
    let before = rows(&mut tx).await;
    assert!(
        LEGACY
            .iter()
            .all(|name| !before[*name].as_array().unwrap().is_empty())
    );
    for table in ["account_security_events", "account_terms_acceptances"] {
        assert_eq!(
            before[table].as_array().unwrap().len(),
            1,
            "real linked retained fixture must be populated"
        );
    }
    let old_meta = metadata(&mut tx, LEGACY).await;
    transition(&mut tx).await.unwrap();
    let mut expected = before.clone();
    let mut expected_array = expected_roots.as_array().unwrap().clone();
    // Compare logical row sets below; PostgreSQL jsonb's text key ordering is distinct.
    expected_array.sort_by_key(Value::to_string);
    expected["accounts"] = Value::Array(expected_array);
    let mut after = rows(&mut tx).await;
    after["accounts"]
        .as_array_mut()
        .unwrap()
        .sort_by_key(Value::to_string);
    assert_eq!(
        after, expected,
        "only exact users-derived roots may be added"
    );
    let after_meta = metadata(&mut tx, LEGACY).await;
    assert_eq!(
        before_legacy_metadata(&old_meta),
        before_legacy_metadata(&after_meta)
    );
    for original in old_meta["functions"].as_array().unwrap() {
        assert_eq!(
            after_meta["functions"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|row| *row == original)
                .count(),
            1,
            "existing Account owner routine must remain byte-exact"
        );
    }
    tx.commit().await.unwrap();
    assert_exact_roots(&pool, &expected_roots).await;
    let mut tx = pool.begin().await.unwrap();
    let stable_rows = rows(&mut tx).await;
    let stable_meta = metadata(&mut tx, TABLES).await;
    for _ in 0..2 {
        transition(&mut tx).await.unwrap();
        assert_eq!(rows(&mut tx).await, stable_rows);
        assert_eq!(metadata(&mut tx, TABLES).await, stable_meta);
    }
    sqlx::query("UPDATE public.users SET created_at=created_at+interval '1 day' WHERE id=$1")
        .bind(id(1))
        .execute(&mut *tx)
        .await
        .unwrap();
    let changed_users = rows(&mut tx).await;
    transition(&mut tx).await.unwrap();
    assert_eq!(
        rows(&mut tx).await,
        changed_users,
        "replay cannot rewrite roots from later user metadata"
    );
    tx.commit().await.unwrap();
    assert_exact_roots(&pool, &expected_roots).await;
}

fn before_legacy_metadata(value: &Value) -> Value {
    json!({"relations":value["relations"],"ledger":value["ledger"]})
}

#[sqlx::test(migrations = false)]
async fn initial_overlap_refuses_atomically_without_association(pool: PgPool) {
    historical(&pool).await;
    for (equal_time, security) in [(true, false), (false, true)] {
        let mut tx = pool.begin().await.unwrap();
        insert_user(&mut tx, id(20)).await.unwrap();
        if equal_time {
            sqlx::query(
                "INSERT INTO accounts(id,created_at) SELECT id,created_at FROM users WHERE id=$1",
            )
            .bind(id(20))
            .execute(&mut *tx)
            .await
            .unwrap();
        } else {
            insert_root(&mut tx, id(20)).await.unwrap();
        }
        if security {
            sqlx::query("INSERT INTO account_security VALUES($1,'RECOVERY_REQUIRED',7,9,now(),11)")
                .bind(id(20))
                .execute(&mut *tx)
                .await
                .unwrap();
        }
        let before = rows(&mut tx).await;
        let meta = metadata(&mut tx, TABLES).await;
        sqlx::raw_sql("SAVEPOINT attempted_transition")
            .execute(&mut *tx)
            .await
            .unwrap();
        let result = transition(&mut tx).await;
        sqlx::raw_sql("ROLLBACK TO SAVEPOINT attempted_transition")
            .execute(&mut *tx)
            .await
            .unwrap();
        db_error(
            result,
            "P0001",
            Some("account_root_transition.overlap_requires_admission"),
            None,
        );
        assert_eq!(rows(&mut tx).await, before);
        assert_eq!(metadata(&mut tx, TABLES).await, meta);
        tx.rollback().await.unwrap();
    }
}

#[sqlx::test(migrations = false)]
async fn bridge_insert_conflict_noop_and_rollback_are_exact(pool: PgPool) {
    prepare_http_database(&pool).await;
    let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
    let mut tx = runtime.begin().await.unwrap();
    company(&mut tx).await;
    insert_user(&mut tx, id(21)).await.unwrap();
    tx.commit().await.unwrap();
    let mut owner = pool.acquire().await.unwrap();
    exact_user_root(&mut owner, id(21)).await;
    let before = rows(&mut owner).await;
    let mut tx = runtime.begin().await.unwrap();
    company(&mut tx).await;
    let count = sqlx::query("INSERT INTO users(id,display_name,org_id,created_at) VALUES($1,'different',$2,'1999-01-01T00:00:00Z') ON CONFLICT(id) DO NOTHING")
        .bind(id(21)).bind(OrgId::knl().as_uuid()).execute(&mut *tx).await.unwrap().rows_affected();
    assert_eq!(count, 0);
    tx.commit().await.unwrap();
    assert_eq!(rows(&mut owner).await, before);
    let mut tx = runtime.begin().await.unwrap();
    company(&mut tx).await;
    insert_user(&mut tx, id(22)).await.unwrap();
    tx.rollback().await.unwrap();
    assert_eq!(rows(&mut owner).await, before);
    runtime.close().await;
}

#[sqlx::test(migrations = false)]
async fn bridge_collision_and_retained_identity_reinsertion_refuse(pool: PgPool) {
    prepare_http_database(&pool).await;
    let mut owner = pool.acquire().await.unwrap();
    require_bridge(&mut owner).await;
    insert_root(&mut owner, id(23)).await.unwrap();
    let before = rows(&mut owner).await;
    let mut tx = pool.begin().await.unwrap();
    let result = insert_user(&mut tx, id(23)).await;
    tx.rollback().await.unwrap();
    db_error(result, "23505", None, Some("accounts_pkey"));
    assert_eq!(rows(&mut owner).await, before);
    let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
    let mut tx = runtime.begin().await.unwrap();
    company(&mut tx).await;
    insert_user(&mut tx, id(24)).await.unwrap();
    tx.commit().await.unwrap();
    exact_user_root(&mut owner, id(24)).await;
    let mut tx = runtime.begin().await.unwrap();
    company(&mut tx).await;
    assert_eq!(
        sqlx::query("DELETE FROM users WHERE id=$1")
            .bind(id(24))
            .execute(&mut *tx)
            .await
            .unwrap()
            .rows_affected(),
        1
    );
    tx.commit().await.unwrap();
    let retained = rows(&mut owner).await;
    let mut tx = runtime.begin().await.unwrap();
    company(&mut tx).await;
    let result = insert_user(&mut tx, id(24)).await;
    tx.rollback().await.unwrap();
    db_error(result, "23505", None, Some("accounts_pkey"));
    assert_eq!(rows(&mut owner).await, retained);
    runtime.close().await;
}

async fn blocker(pool: &PgPool, waiting: i32, holding: i32) {
    tokio::time::timeout(Duration::from_secs(4), async {
        loop {
            if sqlx::query_scalar::<_, bool>("SELECT $2=ANY(pg_blocking_pids($1))")
                .bind(waiting)
                .bind(holding)
                .fetch_one(pool)
                .await
                .unwrap()
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("required PostgreSQL blocker not observed");
}

async fn stop<T>(pool: &PgPool, task: &mut tokio::task::JoinHandle<T>, pid: i32) {
    task.abort();
    let _ = task.await;
    let _: bool = sqlx::query_scalar("SELECT CASE WHEN EXISTS(SELECT 1 FROM pg_stat_activity WHERE pid=$1) THEN pg_terminate_backend($1,5000) ELSE true END")
        .bind(pid).fetch_one(pool).await.unwrap();
    assert!(
        sqlx::query_scalar::<_, bool>(
            "SELECT NOT EXISTS(SELECT 1 FROM pg_stat_activity WHERE pid=$1)"
        )
        .bind(pid)
        .fetch_one(pool)
        .await
        .unwrap()
    );
}

#[sqlx::test(migrations = false)]
async fn concurrent_root_allocators_have_one_winner(pool: PgPool) {
    historical(&pool).await;
    // A pre-transition users writer must finish before the operator can freeze
    // the users snapshot. Observe the real relation-lock dependency.
    let mut writer = pool.begin().await.unwrap();
    insert_user(&mut writer, id(29)).await.unwrap();
    let writer_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *writer)
        .await
        .unwrap();
    let mut operator = PgConnection::connect_with(&pool.connect_options())
        .await
        .unwrap();
    let operator_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut operator)
        .await
        .unwrap();
    let mut operator_task = tokio::spawn(async move {
        let mut tx = operator.begin().await?;
        transition(&mut tx).await?;
        tx.commit().await
    });
    let outcome = std::panic::AssertUnwindSafe(async {
        blocker(&pool, operator_pid, writer_pid).await;
        writer.commit().await.unwrap();
        tokio::time::timeout(Duration::from_secs(65), &mut operator_task)
            .await
            .unwrap()
            .unwrap()
    })
    .catch_unwind()
    .await;
    match outcome {
        Ok(result) => result.unwrap(),
        Err(panic) => {
            stop(&pool, &mut operator_task, operator_pid).await;
            std::panic::resume_unwind(panic);
        }
    }
    let mut owner = pool.acquire().await.unwrap();
    require_bridge(&mut owner).await;
    exact_user_root(&mut owner, id(29)).await;
    drop(owner);
    for (n, user_first, commit_first) in [(30, false, true), (31, true, true), (32, false, false)] {
        let mut first = pool.begin().await.unwrap();
        let first_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut *first)
            .await
            .unwrap();
        if user_first {
            insert_user(&mut first, id(n)).await.unwrap();
        } else {
            insert_root(&mut first, id(n)).await.unwrap();
        }
        let mut second = PgConnection::connect_with(&pool.connect_options())
            .await
            .unwrap();
        let second_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut second)
            .await
            .unwrap();
        let mut task = tokio::spawn(async move {
            let mut tx = second.begin().await?;
            sqlx::raw_sql("SET LOCAL statement_timeout='10s'")
                .execute(&mut *tx)
                .await?;
            let result = if user_first {
                insert_root(&mut tx, id(n)).await
            } else {
                insert_user(&mut tx, id(n)).await
            };
            match result {
                Ok(_) => tx.commit().await,
                Err(error) => {
                    tx.rollback().await?;
                    Err(error)
                }
            }
        });
        let outcome = std::panic::AssertUnwindSafe(async {
            blocker(&pool, second_pid, first_pid).await;
            if commit_first {
                first.commit().await.unwrap();
            } else {
                first.rollback().await.unwrap();
            }
            tokio::time::timeout(Duration::from_secs(12), &mut task)
                .await
                .unwrap()
                .unwrap()
        })
        .catch_unwind()
        .await;
        let result = match outcome {
            Ok(result) => result,
            Err(panic) => {
                stop(&pool, &mut task, second_pid).await;
                std::panic::resume_unwind(panic);
            }
        };
        if commit_first {
            db_error(result, "23505", None, Some("accounts_pkey"));
        } else {
            result.unwrap();
        }
        let mut owner = pool.acquire().await.unwrap();
        if user_first || !commit_first {
            exact_user_root(&mut owner, id(n)).await;
        } else {
            assert_eq!(
                sqlx::query_scalar::<_, i64>("SELECT count(*) FROM users WHERE id=$1")
                    .bind(id(n))
                    .fetch_one(&mut *owner)
                    .await
                    .unwrap(),
                0
            );
        }
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT count(*) FROM accounts WHERE id=$1")
                .bind(id(n))
                .fetch_one(&mut *owner)
                .await
                .unwrap(),
            1
        );
    }
}

#[sqlx::test(migrations = false)]
async fn native_fk_enforces_default_and_explicit_constraint_timing(pool: PgPool) {
    prepare_http_database(&pool).await;
    let mut owner = pool.acquire().await.unwrap();
    require_bridge(&mut owner).await;
    let fk: (bool,bool,bool,String,String) = sqlx::query_as("SELECT convalidated,condeferrable,condeferred,confdeltype::text,confupdtype::text FROM pg_constraint WHERE conrelid='users'::regclass AND conname='users_account_root_v1' AND confrelid='accounts'::regclass AND conkey=ARRAY[1]::smallint[] AND confkey=ARRAY[1]::smallint[]")
        .fetch_one(&mut *owner).await.expect("actual exact native root FK");
    assert_eq!(fk, (true, true, true, "r".to_owned(), "r".to_owned()));
    let order: bool = sqlx::query_scalar("SELECT count(*)=1 AND bool_and(t.tgname::text COLLATE \"C\">$1 COLLATE \"C\") FROM pg_trigger t JOIN pg_proc p ON p.oid=t.tgfoid WHERE t.tgrelid='users'::regclass AND t.tgconstraint=(SELECT oid FROM pg_constraint WHERE conrelid='users'::regclass AND conname='users_account_root_v1') AND p.proname='RI_FKey_check_ins'")
        .bind(BRIDGE).fetch_one(&mut *owner).await.unwrap();
    assert!(order);
    for (n, before, after) in [(40, false, false), (41, true, false), (42, false, true)] {
        let mut tx = pool.begin().await.unwrap();
        if before {
            sqlx::raw_sql("SET CONSTRAINTS ALL IMMEDIATE")
                .execute(&mut *tx)
                .await
                .unwrap();
        }
        insert_user(&mut tx, id(n)).await.unwrap();
        exact_user_root(&mut tx, id(n)).await;
        if after {
            sqlx::raw_sql("SET CONSTRAINTS ALL IMMEDIATE")
                .execute(&mut *tx)
                .await
                .unwrap();
        }
        tx.commit().await.unwrap();
        exact_user_root(&mut owner, id(n)).await;
    }
    insert_user(&mut owner, id(43)).await.unwrap();
    exact_user_root(&mut owner, id(43)).await;
    let before = rows(&mut owner).await;
    let mut tx = pool.begin().await.unwrap();
    sqlx::raw_sql("ALTER TABLE users DISABLE TRIGGER \"00_account_legacy_user_root_v1\"")
        .execute(&mut *tx)
        .await
        .unwrap();
    insert_user(&mut tx, id(44)).await.unwrap();
    let result = sqlx::raw_sql("SET CONSTRAINTS users_account_root_v1 IMMEDIATE")
        .execute(&mut *tx)
        .await;
    tx.rollback().await.unwrap();
    db_error(result, "23503", None, Some("users_account_root_v1"));
    assert_eq!(rows(&mut owner).await, before);
    require_bridge(&mut owner).await;
}

#[sqlx::test(migrations = false)]
async fn root_guard_preserves_native_keyshare_and_user_identity(pool: PgPool) {
    prepare_http_database(&pool).await;
    let mut owner = pool.acquire().await.unwrap();
    require_bridge(&mut owner).await;
    insert_user(&mut owner, id(50)).await.unwrap();
    insert_root(&mut owner, id(51)).await.unwrap();
    insert_root(&mut owner, id(255)).await.unwrap();
    let guard: (String, i16, String) = sqlx::query_as("SELECT tgname::text,tgtype,tgattr::text FROM pg_trigger WHERE tgrelid='users'::regclass AND tgname='00_account_legacy_user_id_immutable_v1' AND tgenabled='A'").fetch_one(&mut *owner).await.expect("actual all-update identity guard");
    assert_eq!(
        guard,
        (
            "00_account_legacy_user_id_immutable_v1".to_owned(),
            17,
            "".to_owned()
        )
    );
    let mut tx = pool.begin().await.unwrap();
    sqlx::raw_sql("SET LOCAL ROLE console_account_owner")
        .execute(&mut *tx)
        .await
        .unwrap();
    let selected: Uuid = sqlx::query_scalar("SELECT id FROM accounts WHERE id=$1 FOR KEY SHARE")
        .bind(id(50))
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    assert_eq!(selected, id(50));
    let result = sqlx::query("UPDATE accounts SET id=id WHERE id=$1")
        .bind(id(50))
        .execute(&mut *tx)
        .await;
    tx.rollback().await.unwrap();
    db_error(result, "P0001", Some("account_roots.immutable"), None);
    sqlx::query("INSERT INTO account_security VALUES($1,'PENDING_ENROLLMENT',1,1,now(),1)")
        .bind(id(50))
        .execute(&mut *owner)
        .await
        .expect("actual Account FK must keep key-share privilege");
    let before = rows(&mut owner).await;
    for replica in [false, true] {
        for statement in [
            "UPDATE accounts SET created_at=now()",
            "UPDATE accounts SET id=id WHERE false",
            "DELETE FROM accounts WHERE false",
            "DELETE FROM accounts",
            "TRUNCATE accounts CASCADE",
        ] {
            let mut tx = pool.begin().await.unwrap();
            if replica {
                sqlx::raw_sql("SET LOCAL session_replication_role=replica")
                    .execute(&mut *tx)
                    .await
                    .unwrap();
            }
            let result = sqlx::raw_sql(statement).execute(&mut *tx).await;
            tx.rollback().await.unwrap();
            db_error(result, "P0001", Some("account_roots.immutable"), None);
            assert_eq!(rows(&mut owner).await, before);
        }
    }
    let mut tx = pool.begin().await.unwrap();
    sqlx::raw_sql("SET CONSTRAINTS ALL IMMEDIATE")
        .execute(&mut *tx)
        .await
        .unwrap();
    assert_eq!(
        sqlx::query("UPDATE users SET id=id WHERE id=$1")
            .bind(id(50))
            .execute(&mut *tx)
            .await
            .unwrap()
            .rows_affected(),
        1
    );
    tx.commit().await.unwrap();
    for (hidden, replica) in [(false, false), (true, false), (false, true)] {
        let mut tx = pool.begin().await.unwrap();
        if replica {
            sqlx::raw_sql("SET LOCAL session_replication_role=replica")
                .execute(&mut *tx)
                .await
                .unwrap();
        }
        sqlx::raw_sql("SET CONSTRAINTS ALL IMMEDIATE")
            .execute(&mut *tx)
            .await
            .unwrap();
        assert_eq!(
            sqlx::query("UPDATE users SET id=id WHERE id=$1")
                .bind(id(50))
                .execute(&mut *tx)
                .await
                .unwrap()
                .rows_affected(),
            1,
            "no-op identity update remains valid in each replication mode"
        );
        if hidden {
            sqlx::raw_sql("CREATE FUNCTION public.root_test_rewrite_id() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN NEW.id='9ab00000-0000-4000-8000-0000000000ff'; RETURN NEW; END $$; CREATE TRIGGER root_test_rewrite_id BEFORE UPDATE OF display_name ON users FOR EACH ROW EXECUTE FUNCTION public.root_test_rewrite_id()")
            .execute(&mut *tx).await.unwrap();
        }
        let result = if hidden {
            sqlx::query("UPDATE users SET display_name='hidden identity rewrite' WHERE id=$1")
                .bind(id(50))
                .execute(&mut *tx)
                .await
        } else {
            sqlx::query("UPDATE users SET id=$2 WHERE id=$1")
                .bind(id(50))
                .bind(id(51))
                .execute(&mut *tx)
                .await
        };
        tx.rollback().await.unwrap();
        db_error(
            result,
            "P0001",
            Some("account_legacy_user_id.immutable"),
            None,
        );
        assert_eq!(rows(&mut owner).await, before);
    }
}

#[sqlx::test(migrations = false)]
async fn root_roles_and_bridge_privileges_are_exact(pool: PgPool) {
    prepare_http_database(&pool).await;
    let mut owner = pool.acquire().await.unwrap();
    require_bridge(&mut owner).await;
    let privileges: Vec<(String,String,bool)> = sqlx::query_as("SELECT a.attname::text,x.privilege_type,x.is_grantable FROM pg_attribute a CROSS JOIN LATERAL aclexplode(a.attacl) x WHERE a.attrelid='accounts'::regclass AND x.grantee='console_account_owner'::regrole AND x.grantor='console_account_owner'::regrole ORDER BY a.attname,x.privilege_type")
        .fetch_all(&mut *owner).await.unwrap();
    assert_eq!(
        privileges,
        vec![
            ("created_at".into(), "INSERT".into(), false),
            ("id".into(), "INSERT".into(), false),
            ("id".into(), "UPDATE".into(), false)
        ]
    );
    let table_acl: Vec<(String,String,bool)> = sqlx::query_as("SELECT r.rolname::text,x.privilege_type,x.is_grantable FROM pg_class c CROSS JOIN LATERAL aclexplode(c.relacl) x JOIN pg_roles r ON r.oid=x.grantee WHERE c.oid='accounts'::regclass ORDER BY r.rolname,x.privilege_type").fetch_all(&mut *owner).await.unwrap();
    assert_eq!(
        table_acl,
        vec![(
            "console_account_owner".to_owned(),
            "SELECT".to_owned(),
            false
        )]
    );
    let owner_exact: bool = sqlx::query_scalar("SELECT NOT rolcanlogin AND NOT rolsuper AND NOT rolbypassrls AND NOT rolinherit AND NOT rolcreatedb AND NOT rolcreaterole AND NOT rolreplication AND NOT EXISTS(SELECT 1 FROM pg_auth_members m WHERE m.roleid=r.oid OR m.member=r.oid) FROM pg_roles r WHERE rolname='console_account_owner'").fetch_one(&mut *owner).await.unwrap();
    assert!(owner_exact);
    for login in [
        TestDatabaseLogin::Business,
        TestDatabaseLogin::Auth,
        TestDatabaseLogin::LeaveCommand,
        TestDatabaseLogin::OntologyCommand,
        TestDatabaseLogin::PlatformForceCommand,
    ] {
        let runtime = login_test_pool(&pool, login).await;
        assert_eq!(
            sqlx::query_scalar::<_, i32>("SELECT 1")
                .fetch_one(&runtime)
                .await
                .unwrap(),
            1
        );
        for statement in [
            "SELECT * FROM accounts LIMIT 0",
            "INSERT INTO accounts(id,created_at) VALUES('9ab00000-0000-4000-8000-000000000060',now())",
            "UPDATE accounts SET id=id WHERE false",
            "DELETE FROM accounts WHERE false",
            "TRUNCATE accounts",
            "SELECT public.account_legacy_user_root_v1()",
        ] {
            let mut tx = runtime.begin().await.unwrap();
            company(&mut tx).await;
            let result = sqlx::raw_sql(statement).execute(&mut *tx).await;
            tx.rollback().await.unwrap();
            db_error(result, "42501", None, None);
        }
        let escalation: bool = sqlx::query_scalar("SELECT pg_has_role(current_user,'console_account_owner','MEMBER') OR pg_has_role(current_user,'console_account_owner','SET') OR pg_has_role(current_user,'console_account_owner','USAGE')")
            .fetch_one(&runtime).await.unwrap();
        assert!(!escalation);
        runtime.close().await;
    }
    let auth = login_test_pool(&pool, TestDatabaseLogin::Auth).await;
    insert_user(&mut owner, id(61)).await.unwrap();
    assert!(
        !sqlx::query_scalar::<_, bool>("SELECT account_legacy_fenced_v1($1)")
            .bind(id(61))
            .fetch_one(&auth)
            .await
            .unwrap()
    );
    sqlx::query("INSERT INTO account_security VALUES($1,'PENDING_ENROLLMENT',1,1,now(),1)")
        .bind(id(61))
        .execute(&mut *owner)
        .await
        .unwrap();
    assert!(
        sqlx::query_scalar::<_, bool>("SELECT account_legacy_fenced_v1($1)")
            .bind(id(61))
            .fetch_one(&auth)
            .await
            .unwrap()
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM account_terms_current_v1()")
            .fetch_one(&auth)
            .await
            .unwrap(),
        0
    );
    auth.close().await;
}

#[sqlx::test(migrations = false)]
async fn profile_drift_refuses_without_repair(pool: PgPool) {
    prepare_http_database(&pool).await;
    require_bridge(&mut pool.acquire().await.unwrap()).await;
    let query = include_str!("../../src/account_custody_state.sql");
    for (mutation, expected_error) in [
        (
            "ALTER TABLE users DISABLE TRIGGER \"00_account_legacy_user_root_v1\"",
            "account_root_transition.profile_mismatch",
        ),
        (
            "ALTER TRIGGER \"00_account_legacy_user_root_v1\" ON users RENAME TO zz_root_bridge",
            "account_root_transition.profile_mismatch",
        ),
        (
            "ALTER FUNCTION account_legacy_user_root_v1() SECURITY INVOKER",
            "account_root_transition.profile_mismatch",
        ),
        (
            "ALTER FUNCTION account_legacy_user_root_v1() SET search_path=public",
            "account_root_transition.profile_mismatch",
        ),
        (
            "ALTER FUNCTION account_legacy_user_root_v1() OWNER TO console_app",
            "account_root_transition.profile_mismatch",
        ),
        (
            "CREATE OR REPLACE FUNCTION public.account_legacy_user_root_v1() RETURNS trigger LANGUAGE plpgsql VOLATILE SECURITY DEFINER SET search_path=pg_catalog,pg_temp AS $$ BEGIN RETURN NEW; END $$",
            "account_root_transition.profile_mismatch",
        ),
        (
            "CREATE OR REPLACE TRIGGER \"00_account_legacy_user_root_v1\" AFTER INSERT ON public.users FOR EACH ROW EXECUTE FUNCTION public.account_roots_immutable_v1(); ALTER TABLE public.users ENABLE ALWAYS TRIGGER \"00_account_legacy_user_root_v1\"",
            "account_root_transition.profile_mismatch",
        ),
        (
            "ALTER TABLE accounts DISABLE TRIGGER account_roots_immutable_v1",
            "account_root_transition.profile_mismatch",
        ),
        (
            "ALTER TABLE users DISABLE TRIGGER \"00_account_legacy_user_id_immutable_v1\"",
            "account_root_transition.profile_mismatch",
        ),
        (
            "GRANT EXECUTE ON FUNCTION account_legacy_user_root_v1() TO console_rt",
            "account_root_transition.profile_mismatch",
        ),
        (
            "GRANT INSERT(id) ON accounts TO console_rt",
            "account_root_transition.profile_mismatch",
        ),
        (
            "GRANT UPDATE(created_at) ON accounts TO console_account_owner",
            "account_custody.unexpected_privilege",
        ),
        (
            "ALTER TABLE users ALTER CONSTRAINT users_account_root_v1 NOT DEFERRABLE",
            "account_root_transition.profile_mismatch",
        ),
        (
            "ALTER TABLE users DROP CONSTRAINT users_account_root_v1",
            "account_root_transition.profile_mismatch",
        ),
        (
            "ALTER TABLE users DROP CONSTRAINT users_account_root_v1; ALTER TABLE users ADD CONSTRAINT users_account_root_v1 FOREIGN KEY(id) REFERENCES accounts(id) ON DELETE RESTRICT ON UPDATE RESTRICT DEFERRABLE INITIALLY DEFERRED NOT VALID",
            "account_root_transition.profile_mismatch",
        ),
    ] {
        let mut tx = pool.begin().await.unwrap();
        sqlx::raw_sql("SET LOCAL search_path=pg_catalog,pg_temp")
            .execute(&mut *tx)
            .await
            .unwrap();
        let positive: String = sqlx::query_scalar(query).fetch_one(&mut *tx).await.unwrap();
        assert_eq!(positive, "account_custody.finalized");
        // These fixed mutation statements are test faults, never request inputs.
        sqlx::raw_sql("SET LOCAL search_path=public,pg_catalog,pg_temp")
            .execute(&mut *tx)
            .await
            .unwrap();
        let body_only = mutation.starts_with("CREATE OR REPLACE FUNCTION");
        let rebind_only = mutation.starts_with("CREATE OR REPLACE TRIGGER");
        let before_body: Option<Value> = if body_only {
            Some(sqlx::query_scalar("SELECT to_jsonb(p)-'prosrc' FROM pg_proc p WHERE p.oid='public.account_legacy_user_root_v1()'::regprocedure").fetch_one(&mut *tx).await.unwrap())
        } else {
            None
        };
        let before_binding: Option<Value> = if rebind_only {
            Some(sqlx::query_scalar("SELECT to_jsonb(t)-'tgfoid' FROM pg_trigger t WHERE t.tgrelid='public.users'::regclass AND t.tgname='00_account_legacy_user_root_v1'").fetch_one(&mut *tx).await.unwrap())
        } else {
            None
        };
        sqlx::raw_sql(mutation).execute(&mut *tx).await.unwrap();
        if let Some(before) = before_body {
            let after: Value = sqlx::query_scalar("SELECT to_jsonb(p)-'prosrc' FROM pg_proc p WHERE p.oid='public.account_legacy_user_root_v1()'::regprocedure").fetch_one(&mut *tx).await.unwrap();
            assert_eq!(
                after, before,
                "body-only fault preserves SECDEF and all other routine metadata"
            );
        }
        if let Some(before) = before_binding {
            let after: Value = sqlx::query_scalar("SELECT to_jsonb(t)-'tgfoid' FROM pg_trigger t WHERE t.tgrelid='public.users'::regclass AND t.tgname='00_account_legacy_user_root_v1'").fetch_one(&mut *tx).await.unwrap();
            assert_eq!(
                after, before,
                "rebind fault preserves every other trigger property"
            );
        }
        sqlx::raw_sql("SET LOCAL search_path=pg_catalog,pg_temp")
            .execute(&mut *tx)
            .await
            .unwrap();
        let before = metadata(&mut tx, TABLES).await;
        let state: String = sqlx::query_scalar(query).fetch_one(&mut *tx).await.unwrap();
        assert_eq!(
            state, expected_error,
            "each metadata drift has its fixed compatible refusal"
        );
        sqlx::raw_sql("SAVEPOINT attempted_transition")
            .execute(&mut *tx)
            .await
            .unwrap();
        let result = transition(&mut tx).await;
        sqlx::raw_sql("ROLLBACK TO SAVEPOINT attempted_transition")
            .execute(&mut *tx)
            .await
            .unwrap();
        db_error(result, "P0001", Some(expected_error), None);
        assert_eq!(metadata(&mut tx, TABLES).await, before);
        tx.rollback().await.unwrap();
    }
    // Privileged data-only corruption retains the exact validated metadata.
    // Serving checks remain metadata-only; only direct operator replay inspects roots.
    let mut tx = pool.begin().await.unwrap();
    insert_user(&mut tx, id(81)).await.unwrap();
    sqlx::raw_sql("SET CONSTRAINTS ALL IMMEDIATE; ALTER TABLE accounts DISABLE TRIGGER account_roots_immutable_v1; SET LOCAL session_replication_role=replica").execute(&mut *tx).await.unwrap();
    sqlx::query("DELETE FROM accounts WHERE id=$1")
        .bind(id(81))
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::raw_sql("SET LOCAL session_replication_role=origin; ALTER TABLE accounts ENABLE ALWAYS TRIGGER account_roots_immutable_v1; SET LOCAL search_path=pg_catalog,pg_temp").execute(&mut *tx).await.unwrap();
    let state: String = sqlx::query_scalar(query).fetch_one(&mut *tx).await.unwrap();
    assert_eq!(
        state, "account_custody.finalized",
        "serving profile must remain metadata-only"
    );
    let missing = rows(&mut tx).await;
    sqlx::raw_sql("SAVEPOINT missing_root_replay")
        .execute(&mut *tx)
        .await
        .unwrap();
    let result = transition(&mut tx).await;
    sqlx::raw_sql("ROLLBACK TO SAVEPOINT missing_root_replay")
        .execute(&mut *tx)
        .await
        .unwrap();
    db_error(
        result,
        "P0001",
        Some("account_root_transition.missing_root"),
        None,
    );
    assert_eq!(
        rows(&mut tx).await,
        missing,
        "operator must refuse without root repair"
    );
    tx.rollback().await.unwrap();
    // Real serving admission sees committed drift. A separate pool timeout on
    // an uncommitted DDL fixture would not prove metadata-drift detection.
    let mut pairs = vec![("CONSOLE_APP_ROLE", super::AppRole::Worker.to_string())];
    pairs.push((
        "CONSOLE_DATABASE_DURABILITY",
        r#"{"mode":"local_development"}"#.to_owned(),
    ));
    pairs.extend(super::account_transport_urls(&pool));
    let config = super::AppConfig::from_pairs(pairs).unwrap();
    let good = super::AppState::from_config(config.clone()).await.unwrap();
    good.shutdown_realtime().await;
    sqlx::raw_sql("ALTER TABLE users DISABLE TRIGGER \"00_account_legacy_user_root_v1\"")
        .execute(&pool)
        .await
        .unwrap();
    let outcome = std::panic::AssertUnwindSafe(super::AppState::from_config(config.clone()))
        .catch_unwind()
        .await;
    sqlx::raw_sql("ALTER TABLE users ENABLE ALWAYS TRIGGER \"00_account_legacy_user_root_v1\"")
        .execute(&pool)
        .await
        .unwrap();
    let attempt = outcome.unwrap();
    if let Ok(state) = &attempt {
        state.shutdown_realtime().await;
    }
    assert!(
        matches!(attempt, Err(console_app::AppError::Config(ref code)) if code == "account_root_transition.profile_mismatch")
    );
    let restored = super::AppState::from_config(config).await.unwrap();
    restored.shutdown_realtime().await;
}

#[sqlx::test(migrations = false)]
async fn fault_after_backfill_rolls_back_complete_transition(pool: PgPool) {
    historical(&pool).await;
    let mut tx = pool.begin().await.unwrap();
    insert_user(&mut tx, id(70)).await.unwrap();
    tx.commit().await.unwrap();
    sqlx::raw_sql("CREATE FUNCTION public.root_test_after_backfill() RETURNS event_trigger LANGUAGE plpgsql AS $$ BEGIN IF EXISTS(SELECT 1 FROM pg_constraint WHERE conrelid='public.users'::regclass AND conname='users_account_root_v1') THEN IF NOT EXISTS(SELECT 1 FROM public.accounts a JOIN public.users u ON u.id=a.id AND u.created_at=a.created_at WHERE u.id='9ab00000-0000-4000-8000-000000000046') THEN RAISE EXCEPTION 'ROOT_TEST_WRONG_ORDER'; END IF; RAISE EXCEPTION 'ROOT_TEST_AFTER_ACTUAL_BACKFILL'; END IF; END $$; CREATE EVENT TRIGGER root_test_after_backfill ON ddl_command_end WHEN TAG IN ('ALTER TABLE') EXECUTE FUNCTION public.root_test_after_backfill()")
        .execute(&pool).await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    let before = rows(&mut tx).await;
    let meta = metadata(&mut tx, TABLES).await;
    sqlx::raw_sql("SAVEPOINT attempted_transition")
        .execute(&mut *tx)
        .await
        .unwrap();
    let result = transition(&mut tx).await;
    sqlx::raw_sql("ROLLBACK TO SAVEPOINT attempted_transition")
        .execute(&mut *tx)
        .await
        .unwrap();
    assert_eq!(rows(&mut tx).await, before);
    assert_eq!(metadata(&mut tx, TABLES).await, meta);
    tx.rollback().await.unwrap();
    sqlx::raw_sql("DROP EVENT TRIGGER root_test_after_backfill; DROP FUNCTION public.root_test_after_backfill()").execute(&pool).await.unwrap();
    db_error(
        result,
        "P0001",
        Some("ROOT_TEST_AFTER_ACTUAL_BACKFILL"),
        None,
    );
    let mut tx = pool.begin().await.unwrap();
    transition(&mut tx).await.unwrap();
    exact_user_root(&mut tx, id(70)).await;
    tx.commit().await.unwrap();
}
