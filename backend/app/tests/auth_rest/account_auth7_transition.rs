//! Additive Auth7 operator transition tests. These execute the actual fixed SQL
//! artifacts under the existing marked disposable operator; data/fault fixtures
//! never install an owner, grant, resolver, custody constraint or serving profile.
//! Missing future SQL is a prerequisite, not an admissible product RED.

use super::{account_custody_finalizer_sql, prepare_http_migration_config};
use console_kernel_core::OrgId;
use console_platform_test_support::{TestDatabaseLogin, login_test_pool};
use serde_json::Value;
use sqlx::{Connection, PgConnection, PgPool};
use std::collections::BTreeSet;
use uuid::Uuid;

const AUTH7: &[&str] = &[
    "auth_webauthn_credentials",
    "auth_webauthn_ceremonies",
    "auth_refresh_token_families",
    "auth_refresh_tokens",
    "auth_bootstrap_credentials",
    "auth_webauthn_ceremony_bindings",
    "auth_device_login_handoffs",
];
const ROOT6: &[&str] = &[
    "accounts",
    "account_security",
    "account_security_events",
    "account_terms_acceptances",
    "account_terms_head",
    "account_terms_release_receipts",
];
const ROOT_STATE: &str = include_str!("../../src/account_custody_state.sql");

fn id(n: u128) -> Uuid {
    Uuid::from_u128(0x9ac70000000040008000000000000000 + n)
}

fn artifact(relative: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative);
    let metadata = std::fs::symlink_metadata(&path).expect(
        "AUTH7_ARTIFACT_PREREQUISITE: missing actual production SQL; deeper tests unreached",
    );
    assert!(
        metadata.file_type().is_file(),
        "actual regular production SQL required"
    );
    std::fs::read_to_string(path).expect("AUTH7_ARTIFACT_PREREQUISITE: unreadable production SQL")
}

fn auth7_sql() -> String {
    artifact("ops/postgres-finalize-account-credentials.sql")
}

fn auth7_state_sql() -> String {
    artifact("backend/app/src/account_credential_custody_state.sql")
}

async fn operator(connection: &mut PgConnection) {
    let actual: (String, String, bool) = sqlx::query_as(
        "SELECT session_user::text,current_user::text,
         current_setting('console.sqlx_test_bootstrap',true)='buck-sqlx-superuser-v1'
         AND (SELECT rolsuper FROM pg_catalog.pg_roles WHERE rolname=current_user)",
    )
    .fetch_one(&mut *connection)
    .await
    .unwrap();
    assert_eq!(
        actual,
        (
            "console_buck_admin".into(),
            "console_buck_admin".into(),
            true
        )
    );
    sqlx::raw_sql("SET LOCAL search_path=pg_catalog,pg_temp; SET LOCAL statement_timeout='60s'; SET LOCAL lock_timeout='5s'; SET LOCAL jit=off")
        .execute(&mut *connection).await.unwrap();
    let jit: String = sqlx::query_scalar("SELECT current_setting('jit')")
        .fetch_one(connection)
        .await
        .unwrap();
    assert_eq!(
        jit, "off",
        "match the production operator catalogue context"
    );
}

async fn root_state(connection: &mut PgConnection) -> String {
    sqlx::query_scalar(ROOT_STATE)
        .fetch_one(connection)
        .await
        .unwrap()
}

async fn complete_profile(connection: &mut PgConnection) {
    assert_eq!(root_state(connection).await, "account_custody.finalized");
    let state: String = sqlx::query_scalar(sqlx::AssertSqlSafe(auth7_state_sql()))
        .fetch_one(connection)
        .await
        .unwrap();
    assert_eq!(state, "account_credentials.finalized");
}

async fn composed(connection: &mut PgConnection) -> Result<(), sqlx::Error> {
    operator(connection).await;
    // Read both before execution. An absent Auth7 artifact cannot be mistaken
    // for a successful root-only composition, nor cause partial fixture DDL.
    let root = account_custody_finalizer_sql();
    let credentials = auth7_sql();
    sqlx::raw_sql(sqlx::AssertSqlSafe(root))
        .execute(&mut *connection)
        .await?;
    sqlx::raw_sql(sqlx::AssertSqlSafe(credentials))
        .execute(connection)
        .await?;
    Ok(())
}

async fn rows(connection: &mut PgConnection) -> Value {
    let mut result = serde_json::Map::new();
    for table in AUTH7
        .iter()
        .chain(ROOT6)
        .chain(["users", "audit_events"].iter())
    {
        // Fixed compile-time relation roster only, never caller/catalog SQL.
        let sql = format!(
            "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text),'[]'::jsonb) FROM public.{table} t"
        );
        let value: Value = sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
            .fetch_one(&mut *connection)
            .await
            .unwrap();
        result.insert((*table).into(), value);
    }
    Value::Object(result)
}

async fn metadata(connection: &mut PgConnection) -> Value {
    let tables: Vec<_> = AUTH7
        .iter()
        .chain(ROOT6)
        .chain(["users", "audit_events"].iter())
        .copied()
        .collect();
    sqlx::query_scalar(r#"SELECT jsonb_build_object(
      'relations',(SELECT jsonb_agg(jsonb_build_object('name',c.relname,'oid',c.oid,'owner',c.relowner,'acl',c.relacl,
        'rls',c.relrowsecurity,'force',c.relforcerowsecurity,
        'columns',(SELECT jsonb_agg(to_jsonb(a) ORDER BY a.attnum) FROM pg_catalog.pg_attribute a WHERE a.attrelid=c.oid),
        'defaults',(SELECT COALESCE(jsonb_agg(to_jsonb(d) ORDER BY d.oid),'[]'::jsonb) FROM pg_catalog.pg_attrdef d WHERE d.adrelid=c.oid),
        'constraints',(SELECT COALESCE(jsonb_agg(to_jsonb(k) ORDER BY k.oid),'[]'::jsonb) FROM pg_catalog.pg_constraint k WHERE k.conrelid=c.oid),
        'triggers',(SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY t.oid),'[]'::jsonb) FROM pg_catalog.pg_trigger t WHERE t.tgrelid=c.oid),
        'indexes',(SELECT COALESCE(jsonb_agg(to_jsonb(i) ORDER BY i.indexrelid),'[]'::jsonb) FROM pg_catalog.pg_index i WHERE i.indrelid=c.oid),
        'policies',(SELECT COALESCE(jsonb_agg(to_jsonb(p) ORDER BY p.oid),'[]'::jsonb) FROM pg_catalog.pg_policy p WHERE p.polrelid=c.oid)) ORDER BY c.relname)
        FROM pg_catalog.pg_class c WHERE c.relnamespace='public'::regnamespace AND c.relname=ANY($1)),
      'functions',(SELECT COALESCE(jsonb_agg(to_jsonb(p) ORDER BY p.oid),'[]'::jsonb) FROM pg_catalog.pg_proc p WHERE p.pronamespace='public'::regnamespace),
      'roles',(SELECT jsonb_agg(to_jsonb(r) ORDER BY r.oid) FROM pg_catalog.pg_roles r),
      'memberships',(SELECT COALESCE(jsonb_agg(to_jsonb(m) ORDER BY m.oid),'[]'::jsonb) FROM pg_catalog.pg_auth_members m),
      'ledger',(SELECT jsonb_agg(to_jsonb(m) ORDER BY version) FROM public._sqlx_migrations m))"#)
        .bind(tables).fetch_one(connection).await.unwrap()
}

fn preserved_columns(before: &Value, after: &Value, table: &str) {
    let original = before[table].as_array().unwrap();
    assert!(
        !original.is_empty(),
        "populated preservation fixture required for {table}"
    );
    let keys: BTreeSet<_> = original[0].as_object().unwrap().keys().collect();
    // JSON object insertion order is not a stored value. Normalize both sides
    // recursively; keep row multiplicity, array order and every value intact.
    let canonical = |mut row: Value| {
        row.sort_all_objects();
        row.to_string()
    };
    let mut expected: Vec<_> = original.iter().map(|row| canonical(row.clone())).collect();
    let mut actual: Vec<_> = after[table]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            let object = row.as_object().unwrap();
            assert!(
                keys.iter().all(|key| object.contains_key(*key)),
                "retained column removed"
            );
            let projection: serde_json::Map<String, Value> = keys
                .iter()
                .map(|key| (key.to_string(), object[*key].clone()))
                .collect();
            canonical(Value::Object(projection))
        })
        .collect();
    expected.sort();
    actual.sort();
    assert_eq!(
        actual, expected,
        "every original row, column and credential byte must survive: {table}"
    );
}

// Data-only synthetic material. Never usable cryptographic keys or sessions.
// Each of the seven tables is populated with live/null and consumed/revoked
// variants, including replacement links, before any custody SQL is applied.
async fn seed_legacy(connection: &mut PgConnection) {
    let org = *OrgId::knl().as_uuid();
    for offset in [0_u128, 100] {
        sqlx::query("INSERT INTO public.users(id,display_name,roles,org_id,created_at) VALUES($1,'AUTH7 TEST ONLY',ARRAY['MECHANIC']::text[],$2,'2026-09-14T01:02:03.123456Z')")
            .bind(id(offset+1)).bind(org).execute(&mut *connection).await.unwrap();
        sqlx::query("INSERT INTO public.auth_webauthn_credentials(id,user_id,org_id,credential_id,passkey_json,last_used_at) VALUES($1,$2,$3,$4,jsonb_build_object('TEST_ONLY_counter',17,'opaque','retained'),$5::timestamptz)")
            .bind(id(offset+11)).bind(id(offset+1)).bind(org).bind(format!("AUTH7_TEST_ONLY_{offset}"))
            .bind(if offset==0 {Some("2026-09-14T01:00:00Z")} else {None}).execute(&mut *connection).await.unwrap();
        sqlx::query("INSERT INTO public.auth_webauthn_ceremonies(id,user_id,ceremony_kind,challenge_json,state_json,created_at,expires_at,consumed_at) VALUES($1,$2,'authentication','{\"TEST_ONLY\":true}','{\"retained\":17}','2026-09-14T00:00:00Z','2026-09-15T00:00:00Z',$3::timestamptz)")
            .bind(id(offset+12)).bind(id(offset+1)).bind(if offset==0 {Some("2026-09-14T00:30:00Z")} else {None})
            .execute(&mut *connection).await.unwrap();
        sqlx::query("INSERT INTO public.auth_refresh_token_families(id,user_id,org_id,created_at,revoked_at,revoked_reason) VALUES($1,$2,$3,'2026-09-14T00:00:00Z',$4::timestamptz,$5)")
            .bind(id(offset+13)).bind(id(offset+1)).bind(org)
            .bind(if offset==0 {Some("2026-09-14T01:00:00Z")} else {None})
            .bind(if offset==0 {Some("TEST_ONLY retained revocation")} else {None}).execute(&mut *connection).await.unwrap();
        sqlx::query("INSERT INTO public.auth_refresh_tokens(id,family_id,user_id,org_id,token_hash,issued_at,expires_at,used_at,revoked_at,reuse_detected_at) VALUES($1,$2,$3,$4,$5,'2026-09-14T00:00:00Z','2026-09-15T00:00:00Z',$6::timestamptz,$6::timestamptz,$6::timestamptz)")
            .bind(id(offset+14)).bind(id(offset+13)).bind(id(offset+1)).bind(org).bind(vec![if offset==0 {0x71_u8} else {0x81};32])
            .bind(if offset==0 {Some("2026-09-14T01:00:00Z")} else {None}).execute(&mut *connection).await.unwrap();
        sqlx::query("INSERT INTO public.auth_bootstrap_credentials(id,user_id,org_id,token_hash,issued_at,expires_at,consumed_at,revoked_reason) VALUES($1,$2,$3,$4,'2026-09-14T00:00:00Z','2026-09-15T00:00:00Z',$5::timestamptz,$6)")
            .bind(id(offset+15)).bind(id(offset+1)).bind(org).bind(vec![if offset==0 {0x72_u8} else {0x82};32])
            .bind(if offset==0 {Some("2026-09-14T01:00:00Z")} else {None})
            .bind(if offset==0 {Some("TEST_ONLY retained bootstrap")} else {None}).execute(&mut *connection).await.unwrap();
        sqlx::query("INSERT INTO public.auth_webauthn_ceremony_bindings(ceremony_id,action_kind,object_id,reason_key,replay_attempt) VALUES($1,'POLL_VOTE',$2,'operations_passkey_poll_vote',$3)")
            .bind(id(offset+12)).bind(id(offset+16)).bind(if offset==0 {Some(7_i32)} else {None})
            .execute(&mut *connection).await.unwrap();
        sqlx::query("INSERT INTO public.auth_device_login_handoffs(id,poll_token_hash,approve_token_hash,issued_at,expires_at,target_user_id,target_org_id,approved_at,approved_user_id,approved_org_id,approved_passkey_id,consumed_at) VALUES($1,$2,$3,'2026-09-14T00:00:00Z','2026-09-15T00:00:00Z',$4,$5,$6::timestamptz,$7,$8,$9,$6::timestamptz)")
            .bind(id(offset+17)).bind(vec![if offset==0 {0x73_u8} else {0x83};32]).bind(vec![if offset==0 {0x74_u8} else {0x84};32])
            .bind(id(offset+1)).bind(org).bind(if offset==0 {Some("2026-09-14T00:30:00Z")} else {None})
            .bind(if offset==0 {Some(id(1))} else {None}).bind(if offset==0 {Some(org)} else {None})
            .bind(if offset==0 {Some(id(11))} else {None}).execute(&mut *connection).await.unwrap();
    }
    sqlx::query("INSERT INTO public.auth_refresh_tokens(id,family_id,user_id,org_id,token_hash,issued_at,expires_at) VALUES($1,$2,$3,$4,$5,'2026-09-14T00:30:00Z','2026-09-15T00:00:00Z')")
        .bind(id(18)).bind(id(13)).bind(id(1)).bind(org).bind(vec![0x75_u8;32]).execute(&mut *connection).await.unwrap();
    sqlx::query("UPDATE public.auth_refresh_tokens SET replaced_by=$1 WHERE id=$2")
        .bind(id(18))
        .bind(id(14))
        .execute(connection)
        .await
        .unwrap();
}

/// These retained transition oracles require actual numbered226, before native
/// catalogs. Current-serving fixtures retain the full migration/Apalis path.
async fn prepare_historical226_database(pool: &PgPool) {
    let config = prepare_http_migration_config(pool).await;
    let mut owner = PgConnection::connect(config.database_url.as_deref().unwrap())
        .await
        .expect("authenticate actual historical migration owner");
    let identity: (String, String, String, bool) = sqlx::query_as(
        "SELECT session_user::text,current_user::text, \
         (SELECT pg_get_userbyid(datdba) FROM pg_catalog.pg_database WHERE datname=current_database()), \
         rolcanlogin AND rolinherit AND NOT rolsuper AND rolbypassrls \
         AND NOT rolcreatedb AND NOT rolcreaterole AND NOT rolreplication \
         FROM pg_catalog.pg_roles WHERE rolname=current_user",
    )
    .fetch_one(&mut owner)
    .await
    .unwrap();
    assert_eq!(
        identity,
        (
            "console_app".to_owned(),
            "console_app".to_owned(),
            "console_app".to_owned(),
            true
        ),
        "historical migrations must execute as the real restricted migration owner"
    );
    sqlx::raw_sql("SET SESSION lock_timeout='5s'; SET SESSION statement_timeout='60s'")
        .execute(&mut owner)
        .await
        .unwrap();
    sqlx::migrate!("../crates/platform/db/migrations")
        .run_to(226, &mut owner)
        .await
        .expect("execute real numbered migrations through226 only");
    let historical: bool = sqlx::query_scalar(
        "SELECT count(*)=226 AND min(version)=1 AND max(version)=226 AND bool_and(success) \
         AND pg_catalog.to_regclass('public.company_actors') IS NULL \
         AND pg_catalog.to_regclass('public.account_context_candidates') IS NULL \
         FROM public._sqlx_migrations",
    )
    .fetch_one(&mut owner)
    .await
    .unwrap();
    assert!(
        historical,
        "exact historical226 ledger and absent native catalogs required"
    );
    owner.close().await.unwrap();
}

async fn installed(pool: &PgPool) {
    prepare_historical226_database(pool).await;
    let mut tx = pool.begin().await.unwrap();
    composed(&mut tx)
        .await
        .expect("actual composed root+Auth7 install");
    complete_profile(&mut tx).await;
    tx.commit().await.unwrap();
}

#[sqlx::test(migrations = false)]
async fn composed_populated_transition_preserves_rows_and_both_replays(pool: PgPool) {
    prepare_historical226_database(&pool).await;
    let mut connection = pool.acquire().await.unwrap();
    seed_legacy(&mut connection).await;
    let before = rows(&mut connection).await;
    let expected_roots = super::account_root_transition::expected_roots_after_backfill(&pool).await;
    drop(connection);
    let mut tx = pool.begin().await.unwrap();
    composed(&mut tx)
        .await
        .expect("actual populated Auth7 transition");
    complete_profile(&mut tx).await;
    let after = rows(&mut tx).await;
    for table in AUTH7 {
        preserved_columns(&before, &after, table);
    }
    assert_eq!(before["users"], after["users"]);
    assert_eq!(before["audit_events"], after["audit_events"]);
    for table in &ROOT6[1..] {
        assert_eq!(
            before[*table], after[*table],
            "no synthetic native identity/security rows"
        );
    }
    tx.commit().await.unwrap();
    super::account_root_transition::assert_exact_roots(&pool, &expected_roots).await;
    let mut tx = pool.begin().await.unwrap();
    operator(&mut tx).await;
    let catalog = metadata(&mut tx).await;
    let data = rows(&mut tx).await;
    sqlx::raw_sql(sqlx::AssertSqlSafe(account_custody_finalizer_sql()))
        .execute(&mut *tx)
        .await
        .unwrap();
    assert_eq!(
        metadata(&mut tx).await,
        catalog,
        "root replay must certify exact Auth7 extension without rewriting"
    );
    assert_eq!(rows(&mut tx).await, data);
    sqlx::raw_sql(sqlx::AssertSqlSafe(auth7_sql()))
        .execute(&mut *tx)
        .await
        .unwrap();
    assert_eq!(
        metadata(&mut tx).await,
        catalog,
        "Auth7 replay must not repair or recreate catalogs"
    );
    assert_eq!(rows(&mut tx).await, data);
    complete_profile(&mut tx).await;
    tx.commit().await.unwrap();
}

#[sqlx::test(migrations = false)]
async fn composed_legacy_profile_refusal_rolls_back_root_and_credentials(pool: PgPool) {
    prepare_historical226_database(&pool).await;
    let mut connection = pool.acquire().await.unwrap();
    seed_legacy(&mut connection).await;
    sqlx::raw_sql("ALTER TABLE public.auth_webauthn_credentials ADD CONSTRAINT auth7_test_unreviewed_input CHECK (true)")
        .execute(&mut *connection).await.unwrap();
    let catalog = metadata(&mut connection).await;
    let data = rows(&mut connection).await;
    drop(connection);
    let credentials = auth7_sql(); // Absent artifact is prerequisite, never desired refusal.
    let mut tx = pool.begin().await.unwrap();
    operator(&mut tx).await;
    sqlx::raw_sql(sqlx::AssertSqlSafe(account_custody_finalizer_sql()))
        .execute(&mut *tx)
        .await
        .unwrap();
    assert_eq!(
        root_state(&mut tx).await,
        "account_custody.finalized",
        "root must have executed successfully inside the failing composition"
    );
    let error = sqlx::raw_sql(sqlx::AssertSqlSafe(credentials))
        .execute(&mut *tx)
        .await
        .expect_err("unreviewed legacy Auth7 input must refuse");
    let db = error
        .as_database_error()
        .expect("actual PostgreSQL rejection");
    assert_eq!(db.code().as_deref(), Some("P0001"));
    assert_eq!(db.message(), "account_credentials.legacy_profile_mismatch");
    tx.rollback().await.unwrap();
    let mut connection = pool.acquire().await.unwrap();
    assert_eq!(
        metadata(&mut connection).await,
        catalog,
        "failed Auth7 must roll back prior root DDL too"
    );
    assert_eq!(
        rows(&mut connection).await,
        data,
        "failed Auth7 must not leave roots or credential effects"
    );
}

#[sqlx::test(migrations = false)]
async fn credential_owner_separates_all_seven_relations_without_identity_escalation(pool: PgPool) {
    installed(&pool).await;
    let owners: Vec<(String, String)> = sqlx::query_as("SELECT c.relname::text,r.rolname::text FROM pg_catalog.pg_class c JOIN pg_catalog.pg_roles r ON r.oid=c.relowner WHERE c.relnamespace='public'::regnamespace AND c.relname=ANY($1)")
        .bind(AUTH7).fetch_all(&pool).await.unwrap();
    assert_eq!(owners.len(), 7);
    let owner_names: BTreeSet<_> = owners.iter().map(|(_, role)| role.as_str()).collect();
    assert_eq!(
        owner_names.len(),
        1,
        "all seven tables have one credential owner"
    );
    let owner = *owner_names.first().unwrap();
    assert!(
        ![
            "console_account_owner",
            "console_terms_owner",
            "console_app",
            "console_rt",
            "console_auth_rt"
        ]
        .contains(&owner)
    );
    let restricted: bool = sqlx::query_scalar("SELECT NOT rolcanlogin AND NOT rolsuper AND NOT rolbypassrls AND NOT rolcreatedb AND NOT rolcreaterole AND NOT rolreplication FROM pg_catalog.pg_roles WHERE rolname=$1")
        .bind(owner).fetch_one(&pool).await.unwrap();
    assert!(
        restricted,
        "credential owner must have no administrative capability"
    );
    for role in [owner, "console_auth_rt", "console_rt"] {
        let root_rights: Vec<(String, bool, bool, bool)> = sqlx::query_as("SELECT c.relname::text,pg_catalog.has_table_privilege(r.oid,c.oid,'SELECT,INSERT,UPDATE,DELETE,TRUNCATE,REFERENCES,TRIGGER'),pg_catalog.has_any_column_privilege(r.oid,c.oid,'SELECT,INSERT,UPDATE,REFERENCES'),pg_catalog.pg_has_role(r.oid,c.relowner,'MEMBER') OR pg_catalog.pg_has_role(r.oid,c.relowner,'SET') OR pg_catalog.pg_has_role(r.oid,c.relowner,'USAGE') FROM pg_catalog.pg_class c CROSS JOIN pg_catalog.pg_roles r WHERE r.rolname=$1 AND c.relnamespace='public'::regnamespace AND c.relname=ANY($2)")
            .bind(role).bind(ROOT6).fetch_all(&pool).await.unwrap();
        assert_eq!(root_rights.len(), 6);
        assert!(
            root_rights
                .iter()
                .all(|(_, table, column, member)| !table && !column && !member),
            "credential composition cannot widen raw Account/security rights: {role} {root_rights:?}"
        );
    }
    let auth = login_test_pool(&pool, TestDatabaseLogin::Auth).await;
    let fenced: bool = sqlx::query_scalar("SELECT public.account_legacy_fenced_v1($1)")
        .bind(id(900))
        .fetch_one(&auth)
        .await
        .unwrap();
    assert!(
        !fenced,
        "retained Auth projection is callable through actual LOGIN"
    );
    auth.close().await;
    // This is owner/ACL storage proof; retained protocol and purge suites must
    // separately prove that converted real callers still perform their work.
}

#[sqlx::test(migrations = false)]
async fn root_profile_rejects_each_auth7_account_fk_and_ri_trigger_drift(pool: PgPool) {
    installed(&pool).await;
    let mut connection = pool.acquire().await.unwrap();
    let keys: Vec<(String, String, Vec<String>)> = sqlx::query_as("SELECT c.conrelid::regclass::text,c.conname::text,ARRAY(SELECT a.attname::text FROM unnest(c.conkey) WITH ORDINALITY k(num,ord) JOIN pg_catalog.pg_attribute a ON a.attrelid=c.conrelid AND a.attnum=k.num ORDER BY k.ord) FROM pg_catalog.pg_constraint c JOIN pg_catalog.pg_class s ON s.oid=c.conrelid WHERE c.contype='f' AND c.confrelid='public.accounts'::regclass AND s.relnamespace='public'::regnamespace AND s.relname=ANY($1) ORDER BY c.oid")
        .bind(AUTH7).fetch_all(&mut *connection).await.unwrap();
    let expected: BTreeSet<(String, Vec<String>)> = [
        ("auth_webauthn_credentials", "user_id"),
        ("auth_webauthn_ceremonies", "user_id"),
        ("auth_refresh_token_families", "user_id"),
        ("auth_refresh_tokens", "user_id"),
        ("auth_bootstrap_credentials", "user_id"),
        ("auth_device_login_handoffs", "target_user_id"),
        ("auth_device_login_handoffs", "approved_user_id"),
    ]
    .into_iter()
    .map(|(table, column)| (format!("public.{table}"), vec![column.to_owned()]))
    .collect();
    assert_eq!(keys.len(), 7, "exact approved Account FK extension roster");
    let actual: BTreeSet<_> = keys
        .iter()
        .map(|(table, _, cols)| {
            (
                if table.starts_with("public.") {
                    table.clone()
                } else {
                    format!("public.{table}")
                },
                cols.clone(),
            )
        })
        .collect();
    assert_eq!(actual, expected);
    let triggers: Vec<(String, String)> = sqlx::query_as("SELECT t.tgrelid::regclass::text,t.tgname::text FROM pg_catalog.pg_trigger t JOIN pg_catalog.pg_constraint c ON c.oid=t.tgconstraint JOIN pg_catalog.pg_class s ON s.oid=c.conrelid WHERE c.contype='f' AND c.confrelid='public.accounts'::regclass AND s.relnamespace='public'::regnamespace AND s.relname=ANY($1) AND t.tgrelid='public.accounts'::regclass ORDER BY t.oid")
        .bind(AUTH7).fetch_all(&mut *connection).await.unwrap();
    assert_eq!(
        triggers.len(),
        14,
        "two incoming Account RI triggers per FK"
    );
    drop(connection);
    // All identifiers come from the already checked finite FK/trigger roster;
    // quote even catalog identifiers rather than treating metadata as SQL text.
    let quote = |name: &str| format!("\"{}\"", name.replace('"', "\"\""));
    let relation = |name: &str| {
        format!(
            "public.{}",
            quote(name.strip_prefix("public.").unwrap_or(name))
        )
    };
    let mut faults: Vec<String> = keys
        .iter()
        .flat_map(|(table, key, _)| {
            [
                format!(
                    "ALTER TABLE {} ALTER CONSTRAINT {} DEFERRABLE INITIALLY DEFERRED",
                    relation(table),
                    quote(key)
                ),
                format!(
                    "ALTER TABLE {} DROP CONSTRAINT {}",
                    relation(table),
                    quote(key)
                ),
            ]
        })
        .collect();
    faults.extend(triggers.iter().map(|(table, trigger)| {
        format!(
            "ALTER TABLE {} DISABLE TRIGGER {}",
            relation(table),
            quote(trigger)
        )
    }));
    assert_eq!(faults.len(), 28);
    for fault in faults {
        let mut tx = pool.begin().await.unwrap();
        operator(&mut tx).await;
        complete_profile(&mut tx).await;
        sqlx::raw_sql(sqlx::AssertSqlSafe(fault.clone()))
            .execute(&mut *tx)
            .await
            .unwrap();
        assert_ne!(
            root_state(&mut tx).await,
            "account_custody.finalized",
            "root may project out only complete certified Auth7 RI extensions: {fault}"
        );
        let catalog = metadata(&mut tx).await;
        sqlx::raw_sql("SAVEPOINT auth7_root_replay")
            .execute(&mut *tx)
            .await
            .unwrap();
        sqlx::raw_sql(sqlx::AssertSqlSafe(account_custody_finalizer_sql()))
            .execute(&mut *tx)
            .await
            .expect_err("root replay must not repair partial Auth7 extensions");
        sqlx::raw_sql("ROLLBACK TO SAVEPOINT auth7_root_replay")
            .execute(&mut *tx)
            .await
            .unwrap();
        assert_eq!(metadata(&mut tx).await, catalog);
        tx.rollback().await.unwrap();
    }
}

#[sqlx::test(migrations = false)]
async fn auth7_replay_refuses_effective_column_acl_drift_without_repair(pool: PgPool) {
    installed(&pool).await;
    for table in AUTH7 {
        let mut tx = pool.begin().await.unwrap();
        operator(&mut tx).await;
        complete_profile(&mut tx).await;
        let column = if *table == "auth_webauthn_ceremony_bindings" {
            "ceremony_id"
        } else {
            "id"
        };
        sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
            "GRANT SELECT ({column}) ON public.{table} TO PUBLIC"
        )))
        .execute(&mut *tx)
        .await
        .unwrap();
        let effective: bool = sqlx::query_scalar(
            "SELECT pg_catalog.has_any_column_privilege('console_rt',$1::text,'SELECT')",
        )
        .bind(format!("public.{table}"))
        .fetch_one(&mut *tx)
        .await
        .unwrap();
        assert!(
            effective,
            "fault must create an actual inherited PUBLIC column privilege"
        );
        let state: String = sqlx::query_scalar(sqlx::AssertSqlSafe(auth7_state_sql()))
            .fetch_one(&mut *tx)
            .await
            .unwrap();
        assert_ne!(
            state, "account_credentials.finalized",
            "Auth7 profile must see column/PUBLIC drift"
        );
        let catalog = metadata(&mut tx).await;
        let data = rows(&mut tx).await;
        sqlx::raw_sql("SAVEPOINT auth7_acl_replay")
            .execute(&mut *tx)
            .await
            .unwrap();
        sqlx::raw_sql(sqlx::AssertSqlSafe(auth7_sql()))
            .execute(&mut *tx)
            .await
            .expect_err("Auth7 replay must refuse rather than repair drift");
        sqlx::raw_sql("ROLLBACK TO SAVEPOINT auth7_acl_replay")
            .execute(&mut *tx)
            .await
            .unwrap();
        assert_eq!(metadata(&mut tx).await, catalog);
        assert_eq!(rows(&mut tx).await, data);
        tx.rollback().await.unwrap();
    }
}

// Account-only removal controls. The hostile Actor rows below are deliberately
// injected disposable data, not canonical Company births or control evidence.
// Production finalizers and real LOGINs supply every privilege and owner port.
mod native_removal_owner_boundary {
    use super::super::account_browser::deployment_operator_designation::company_setup::all_rows;
    use super::*;
    use console_kernel_core::{AuditAction, AuditEvent, TraceContext};
    use console_platform_provisioning::{PlatformProvisioner, ProvisioningError};
    use futures::FutureExt;
    use std::panic::AssertUnwindSafe;

    type ColumnAcl = (String, String, String, String, bool);

    async fn column_acl(connection: &mut PgConnection, table: &str) -> Vec<ColumnAcl> {
        sqlx::query_as("SELECT a.attname::text,pg_catalog.pg_get_userbyid(x.grantor)::text,pg_catalog.pg_get_userbyid(x.grantee)::text,x.privilege_type,x.is_grantable FROM pg_catalog.pg_attribute a CROSS JOIN LATERAL pg_catalog.aclexplode(a.attacl) x WHERE a.attrelid=pg_catalog.to_regclass($1) AND a.attnum>0 AND NOT a.attisdropped ORDER BY 1,2,3,4,5")
            .bind(table).fetch_all(connection).await.unwrap()
    }

    async fn narrow_owner_acl(connection: &mut PgConnection) {
        assert_eq!(
            column_acl(connection, "public.company_actors").await,
            vec![
                (
                    "org_id".into(),
                    "console_account_owner".into(),
                    "console_account_owner".into(),
                    "SELECT".into(),
                    false
                ),
                (
                    "org_id".into(),
                    "console_account_owner".into(),
                    "console_account_owner".into(),
                    "UPDATE".into(),
                    false
                ),
            ],
            "NATIVE_REMOVAL_ACTOR_KEY_ACL: only the owner's exact RI key read/lock columns"
        );
        assert_eq!(
            column_acl(connection, "public.auth_webauthn_credentials").await,
            vec![(
                "org_id".into(),
                "console_credential_owner".into(),
                "console_credential_owner".into(),
                "UPDATE".into(),
                false
            )],
            "NATIVE_REMOVAL_CREDENTIAL_KEY_ACL: retain SELECT/DELETE without table-wide UPDATE"
        );
        let narrow: bool = sqlx::query_scalar("SELECT NOT pg_catalog.has_table_privilege('console_account_owner','public.company_actors','SELECT,INSERT,UPDATE,DELETE,TRUNCATE') AND NOT pg_catalog.has_any_column_privilege('console_account_owner','public.company_actors','INSERT') AND NOT pg_catalog.has_table_privilege('console_credential_owner','public.auth_webauthn_credentials','UPDATE')")
            .fetch_one(&mut *connection).await.unwrap();
        assert!(narrow, "key locking must not create a dormant writer");
        for role in [
            "console_app",
            "console_rt",
            "console_auth_rt",
            "console_platform_force_cmd",
            "console_credential_owner",
            "console_terms_owner",
        ] {
            let denied: bool = sqlx::query_scalar("SELECT NOT pg_catalog.has_table_privilege($1,'public.company_actors','SELECT,INSERT,UPDATE,DELETE,TRUNCATE,REFERENCES,TRIGGER') AND NOT pg_catalog.has_any_column_privilege($1,'public.company_actors','SELECT,INSERT,UPDATE,REFERENCES')")
                .bind(role).fetch_one(&mut *connection).await.unwrap();
            assert!(denied, "Actor custody cannot grant generic access: {role}");
        }
    }

    async fn boundary_metadata(connection: &mut PgConnection) -> Value {
        let existing = metadata(connection).await;
        let relations: Value = sqlx::query_scalar(
            r#"SELECT jsonb_agg(jsonb_build_object(
                'relation',to_jsonb(c),
                'columns',(SELECT COALESCE(jsonb_agg(to_jsonb(a) ORDER BY a.attnum),'[]'::jsonb)
                    FROM pg_catalog.pg_attribute a WHERE a.attrelid=c.oid),
                'defaults',(SELECT COALESCE(jsonb_agg(to_jsonb(d) ORDER BY d.oid),'[]'::jsonb)
                    FROM pg_catalog.pg_attrdef d WHERE d.adrelid=c.oid),
                'indexes',(SELECT COALESCE(jsonb_agg(jsonb_build_object(
                    'index',to_jsonb(i),'relation',to_jsonb(ic),
                    'definition',pg_catalog.pg_get_indexdef(i.indexrelid)) ORDER BY i.indexrelid),'[]'::jsonb)
                    FROM pg_catalog.pg_index i JOIN pg_catalog.pg_class ic ON ic.oid=i.indexrelid
                    WHERE i.indrelid=c.oid),
                'constraints',(SELECT COALESCE(jsonb_agg(to_jsonb(k) ORDER BY k.oid),'[]'::jsonb)
                    FROM pg_catalog.pg_constraint k WHERE k.conrelid=c.oid),
                'triggers',(SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY t.oid),'[]'::jsonb)
                    FROM pg_catalog.pg_trigger t WHERE t.tgrelid=c.oid),
                'policies',(SELECT COALESCE(jsonb_agg(to_jsonb(p) ORDER BY p.oid),'[]'::jsonb)
                    FROM pg_catalog.pg_policy p WHERE p.polrelid=c.oid)
            ) ORDER BY c.relname COLLATE "C")
            FROM pg_catalog.pg_class c WHERE c.oid=ANY(ARRAY[
                'public.company_actors'::regclass,'public.organizations'::regclass,
                'public.groups'::regclass,'public.group_memberships'::regclass])"#,
        )
        .fetch_one(connection)
        .await
        .unwrap();
        assert_eq!(relations.as_array().unwrap().len(), 4);
        serde_json::json!({"account_auth7": existing, "removal_relations": relations})
    }

    async fn hostile_actor_fixture(pool: &PgPool) -> (Uuid, Uuid, Uuid) {
        super::super::prepare_http_database(pool).await;
        let (company, foreign, account) = (id(70_001), id(70_002), id(70_003));
        for (org, slug) in [(company, "removal-hostile"), (foreign, "removal-foreign")] {
            sqlx::query("INSERT INTO public.organizations(id,slug,name,status) VALUES($1,$2,'REMOVAL TEST ONLY','ARCHIVED')")
                .bind(org).bind(slug).execute(pool).await.unwrap();
        }
        sqlx::query("INSERT INTO public.users(id,display_name,roles,org_id) VALUES($1,'REMOVAL TEST ONLY',ARRAY['SUPER_ADMIN']::text[],$2)")
            .bind(account).bind(company).execute(pool).await.unwrap();
        for org in [company, foreign] {
            // Real dormant constraints accept this shape. The referenced event
            // is intentionally absent: malformed retained rows must also block
            // generic removal, not become evidence of authorized native birth.
            sqlx::query("INSERT INTO public.company_actors(org_id,account_id,admission_receipt_id,entitlement_ref,created_at) VALUES($1,$2,$3,jsonb_build_object('kind','ACCOUNT_SECURITY_EVENT','account_id',$2::uuid::text,'event_id',$3::uuid::text),now())")
                .bind(org).bind(account).bind(id(70_004)).execute(pool).await.unwrap();
        }
        sqlx::query("INSERT INTO public.auth_webauthn_credentials(id,user_id,org_id,credential_id,passkey_json) VALUES($1,$2,$3,'REMOVAL_TEST_ONLY','{\"TEST_ONLY\":true}')")
            .bind(id(70_005)).bind(account).bind(company).execute(pool).await.unwrap();
        let runtime = login_test_pool(pool, TestDatabaseLogin::Business).await;
        for org in [company, foreign] {
            // Synthetic historical data through the existing audit writer;
            // this fixture action does not assert a product/API operation.
            let event = AuditEvent::new(
                None,
                AuditAction::new("audit.native_removal_fixture").unwrap(),
                "TEST_ONLY_REMOVAL_AUDIT",
                org.to_string(),
                TraceContext::generate(),
                time::OffsetDateTime::now_utc(),
            )
            .with_org(OrgId::from_uuid(org))
            .with_snapshots(
                None,
                Some(serde_json::json!({"fixture":"adversarial-data-only","company":org})),
            );
            let mut tx = runtime.begin().await.unwrap();
            let _: String =
                sqlx::query_scalar("SELECT pg_catalog.set_config('app.current_org',$1,true)")
                    .bind(org.to_string())
                    .fetch_one(tx.as_mut())
                    .await
                    .unwrap();
            console_platform_db::insert_audit_event(&mut tx, &event)
                .await
                .unwrap();
            tx.commit().await.unwrap();
            let actual: (Uuid, Uuid, Option<Uuid>, String, String, String, Option<Value>) = sqlx::query_as("SELECT id,org_id,actor,action,target_type,target_id,after_snap FROM public.audit_events WHERE id=$1")
                .bind(event.id.as_uuid()).fetch_one(pool).await.unwrap();
            assert_eq!(
                actual,
                (
                    *event.id.as_uuid(),
                    org,
                    None,
                    "audit.native_removal_fixture".into(),
                    "TEST_ONLY_REMOVAL_AUDIT".into(),
                    org.to_string(),
                    event.after.clone()
                ),
                "the actual writer must commit each exact scoped fixture audit before removers run"
            );
        }
        runtime.close().await;
        (company, foreign, account)
    }

    async fn assume_nologin_owner(connection: &mut PgConnection, owner: &str, company: Uuid) {
        let statement = match owner {
            "console_account_owner" => "SET LOCAL ROLE console_account_owner",
            "console_credential_owner" => "SET LOCAL ROLE console_credential_owner",
            _ => panic!("finite NOLOGIN owner roster only"),
        };
        sqlx::raw_sql(statement)
            .execute(&mut *connection)
            .await
            .unwrap();
        sqlx::raw_sql("SET LOCAL row_security=on")
            .execute(&mut *connection)
            .await
            .unwrap();
        let actual: (String, String, bool, bool, bool) = sqlx::query_as("SELECT session_user::text,current_user::text,rolcanlogin,rolsuper,rolbypassrls FROM pg_catalog.pg_roles WHERE rolname=current_user")
            .fetch_one(&mut *connection).await.unwrap();
        assert_eq!(
            actual,
            (
                "console_buck_admin".into(),
                owner.into(),
                false,
                false,
                false
            ),
            "NOLOGIN owner SET ROLE is distinct from a restricted LOGIN"
        );
        let _: String =
            sqlx::query_scalar("SELECT pg_catalog.set_config('app.current_org',$1,true)")
                .bind(company.to_string())
                .fetch_one(connection)
                .await
                .unwrap();
    }

    #[sqlx::test(migrations = false)]
    async fn native_removal_owner_key_locks_are_narrow_and_real_logins_remain_denied(pool: PgPool) {
        let (company, foreign, _) = hostile_actor_fixture(&pool).await;
        let before = all_rows(&pool).await;
        let mut connection = pool.acquire().await.unwrap();
        narrow_owner_acl(&mut connection).await;
        drop(connection);
        for (owner, table) in [
            ("console_account_owner", "company_actors"),
            ("console_credential_owner", "auth_webauthn_credentials"),
        ] {
            let mut tx = pool.begin().await.unwrap();
            assume_nologin_owner(&mut tx, owner, company).await;
            let statement =
                format!("SELECT org_id FROM public.{table} WHERE org_id=$1 FOR KEY SHARE");
            let locked: Vec<Uuid> = sqlx::query_scalar(sqlx::AssertSqlSafe(statement))
                .bind(company)
                .fetch_all(tx.as_mut())
                .await
                .expect("NATIVE_REMOVAL_OWNER_RI_KEY_LOCK");
            assert_eq!(
                locked,
                vec![company],
                "owner lock must reach a real RLS-visible row"
            );
            if owner == "console_account_owner" {
                let hidden: i64 = sqlx::query_scalar(
                    "SELECT count(*) FROM public.company_actors WHERE org_id=$1",
                )
                .bind(foreign)
                .fetch_one(tx.as_mut())
                .await
                .unwrap();
                assert_eq!(
                    hidden, 0,
                    "selected Company scope must not reveal foreign Actors"
                );
            }
            tx.rollback().await.unwrap();
        }
        for login in [
            TestDatabaseLogin::Business,
            TestDatabaseLogin::Auth,
            TestDatabaseLogin::PlatformForceCommand,
        ] {
            let runtime = login_test_pool(&pool, login).await;
            let actual: (String, String, bool, bool) = sqlx::query_as("SELECT session_user::text,current_user::text,rolsuper,rolbypassrls FROM pg_catalog.pg_roles WHERE rolname=current_user")
                .fetch_one(&runtime).await.unwrap();
            assert_eq!(actual.0, actual.1, "authenticate the real restricted LOGIN");
            assert_eq!((actual.2, actual.3), (false, false));
            for statement in [
                "SELECT org_id FROM public.company_actors",
                "UPDATE public.company_actors SET org_id=org_id WHERE false",
                "DELETE FROM public.company_actors WHERE false",
                "TRUNCATE public.company_actors",
            ] {
                let error = sqlx::raw_sql(statement)
                    .execute(&runtime)
                    .await
                    .expect_err("no generic/runtime Actor-table capability");
                assert_eq!(
                    error.as_database_error().unwrap().code().as_deref(),
                    Some("42501")
                );
            }
            runtime.close().await;
        }
        assert!(
            all_rows(&pool).await == before,
            "key locks and denied runtime probes must preserve every public row"
        );
    }

    #[sqlx::test(migrations = false)]
    async fn native_removal_dormant_actor_statement_guard_rejects_real_noop_and_zero_rows(
        pool: PgPool,
    ) {
        use sqlx::Acquire;
        let (company, foreign, _) = hostile_actor_fixture(&pool).await;
        let before = all_rows(&pool).await;
        let guarded: bool = sqlx::query_scalar("SELECT COALESCE(bit_or(t.tgtype::integer & 56)=56 AND bool_and(t.tgenabled='A' AND (t.tgtype::integer & 1)=0 AND (t.tgtype::integer & 2)=2),false) FROM pg_catalog.pg_trigger t WHERE t.tgrelid='public.company_actors'::regclass AND NOT t.tgisinternal AND (t.tgtype::integer & 56)<>0")
            .fetch_one(&pool).await.unwrap();
        assert!(
            guarded,
            "NATIVE_REMOVAL_DORMANT_ACTOR_GUARD: ALWAYS BEFORE STATEMENT UPDATE/DELETE/TRUNCATE before key grants"
        );
        let mut tx = pool.begin().await.unwrap();
        assume_nologin_owner(&mut tx, "console_account_owner", company).await;
        let visible: Vec<Uuid> = sqlx::query_scalar(
            "SELECT org_id FROM public.company_actors WHERE org_id=$1 FOR KEY SHARE",
        )
        .bind(company)
        .fetch_all(tx.as_mut())
        .await
        .unwrap();
        assert_eq!(
            visible,
            vec![company],
            "mutation controls cannot pass against an invisible/empty projection"
        );
        for statement in [
            "UPDATE public.company_actors SET org_id=$2 WHERE org_id=$1",
            "UPDATE public.company_actors SET org_id=org_id WHERE org_id=$1 AND $2::uuid<>org_id",
            "UPDATE public.company_actors SET org_id=$2 WHERE org_id=$1 AND false",
        ] {
            let mut attempt = tx.begin().await.unwrap();
            let error = sqlx::query(statement)
                .bind(company)
                .bind(foreign)
                .execute(attempt.as_mut())
                .await
                .expect_err("dormant owner mutation must reach the statement refusal");
            assert_eq!(
                error.as_database_error().unwrap().code().as_deref(),
                Some("P0001"),
                "missing UPDATE privilege or an RLS refusal cannot stand in for the immutable guard"
            );
            attempt.rollback().await.unwrap();
        }
        tx.rollback().await.unwrap();
        assert!(
            all_rows(&pool).await == before,
            "all real/no-op/zero-row attempts must preserve every public row"
        );
    }

    fn exact_native_refusal(result: Result<String, sqlx::Error>) {
        let error =
            result.expect_err("native Actor custody must refuse before any destructive work");
        let database = error.as_database_error().unwrap();
        assert_eq!(database.code().as_deref(), Some("P0001"));
        assert_eq!(
            database.message(),
            "auth_legacy_purge.native_actors_present"
        );
    }

    #[sqlx::test(migrations = false)]
    async fn native_removal_hostile_actor_refusal_preserves_credentials_accounts_audit_and_foreign_company(
        pool: PgPool,
    ) {
        let (company, foreign, account) = hostile_actor_fixture(&pool).await;
        let audit_scopes: Vec<(Uuid, String)> = sqlx::query_as("SELECT org_id,target_id FROM public.audit_events WHERE action='audit.native_removal_fixture' AND target_type='TEST_ONLY_REMOVAL_AUDIT' ORDER BY org_id")
            .fetch_all(&pool).await.unwrap();
        assert_eq!(
            audit_scopes,
            vec![
                (company, company.to_string()),
                (foreign, foreign.to_string())
            ],
            "target and foreign audit baselines must both be populated before removers run"
        );
        let before = all_rows(&pool).await;
        let business = login_test_pool(&pool, TestDatabaseLogin::Business).await;
        let command = login_test_pool(&pool, TestDatabaseLogin::PlatformForceCommand).await;
        let normal = sqlx::query_scalar("SELECT public.platform_remove_organization($1)")
            .bind(company)
            .fetch_one(&business)
            .await;
        let force = sqlx::query_scalar("SELECT public.platform_force_remove_organization_command($1,$2,$3::char(32),$4::char(16),$5)")
            .bind(company).bind(account).bind("1".repeat(32)).bind("2".repeat(16)).bind(time::OffsetDateTime::now_utc()).fetch_one(&command).await;
        let provisioner = PlatformProvisioner::new(time::Duration::minutes(15));
        let normal_mapped = provisioner
            .remove_tenant(&business, None, company, time::OffsetDateTime::now_utc())
            .await;
        let force_mapped = provisioner
            .force_remove_tenant(
                &command,
                Some(console_kernel_core::UserId::from_uuid(account)),
                company,
                time::OffsetDateTime::now_utc(),
            )
            .await;
        business.close().await;
        command.close().await;
        assert!(
            all_rows(&pool).await == before,
            "both removal paths must retain all credentials, Account roots, Actors, audit and foreign Company rows"
        );
        exact_native_refusal(normal);
        exact_native_refusal(force);
        assert!(
            matches!(normal_mapped, Err(ProvisioningError::Conflict(_))),
            "known native refusal must enter the existing Conflict boundary"
        );
        assert!(
            matches!(force_mapped, Err(ProvisioningError::Conflict(_))),
            "force removal must classify the same exact native refusal"
        );
    }

    #[sqlx::test(migrations = false)]
    async fn native_removal_composed_finalizer_replay_and_late_failure_preserve_exact_state(
        pool: PgPool,
    ) {
        super::super::prepare_http_database_staging(&pool).await;
        let mut connection = pool.acquire().await.unwrap();
        seed_legacy(&mut connection).await;
        let staged = boundary_metadata(&mut connection).await;
        drop(connection);
        let rows_before = all_rows(&pool).await;
        let mut tx = pool.begin().await.unwrap();
        let result = AssertUnwindSafe(async {
            operator(&mut tx).await;
            console_platform_test_support::finalize_serving_account_custody_on_connection(
                tx.as_mut(),
            )
            .await
            .expect("actual Account+credential finalizer must install the correction");
            narrow_owner_acl(&mut tx).await;
            let corrected = boundary_metadata(&mut tx).await;
            operator(&mut tx).await;
            console_platform_test_support::finalize_serving_account_custody_on_connection(
                tx.as_mut(),
            )
            .await
            .expect("actual corrected finalizer replay");
            assert!(
                boundary_metadata(&mut tx).await == corrected,
                "replay must not repair/recreate metadata"
            );
            let error = sqlx::query("SELECT 1/0")
                .execute(tx.as_mut())
                .await
                .unwrap_err();
            assert_eq!(
                error.as_database_error().unwrap().code().as_deref(),
                Some("22012")
            );
        })
        .catch_unwind()
        .await;
        tx.rollback().await.unwrap();
        let mut connection = pool.acquire().await.unwrap();
        assert!(
            boundary_metadata(&mut connection).await == staged,
            "late failure must roll back grants, guards, functions and profile metadata"
        );
        drop(connection);
        assert!(
            all_rows(&pool).await == rows_before,
            "late failure must roll back every populated public row and ledger effect"
        );
        if let Err(panic) = result {
            std::panic::resume_unwind(panic);
        }
        super::super::finalize_serving_account_custody(&pool).await;
        let mut connection = pool.acquire().await.unwrap();
        narrow_owner_acl(&mut connection).await;
        let committed = boundary_metadata(&mut connection).await;
        drop(connection);
        let committed_rows = all_rows(&pool).await;
        super::super::finalize_serving_account_custody(&pool).await;
        let mut connection = pool.acquire().await.unwrap();
        assert!(
            boundary_metadata(&mut connection).await == committed,
            "committed replay must be read-only"
        );
        drop(connection);
        assert!(
            all_rows(&pool).await == committed_rows,
            "committed replay must preserve every public row"
        );
    }

    // These tests use the new standalone production finalizer directly. They
    // do not replay historical root SQL onto the corrected successor profile.
    fn removal_transition_artifacts() -> (String, String, String) {
        // Resolve every required artifact before migrations or fault injection.
        // Missing source is a prerequisite failure, never the intended RED.
        (
            artifact("ops/postgres-finalize-account-legacy-removal-v1.sql"),
            artifact("backend/app/src/account_legacy_removal_v1_custody_state.sql"),
            artifact("ops/postgres-capture-account-legacy-removal-v1-custody.sql"),
        )
    }

    async fn removal_transition_state(connection: &mut PgConnection, state: &str) -> String {
        sqlx::query_scalar(sqlx::AssertSqlSafe(state.to_owned()))
            .fetch_one(connection)
            .await
            .unwrap()
    }

    async fn removal_transition_metadata(connection: &mut PgConnection, capture: &str) -> Value {
        let (snapshot, digest): (Value, String) =
            sqlx::query_as(sqlx::AssertSqlSafe(capture.to_owned()))
                .fetch_one(&mut *connection)
                .await
                .unwrap();
        assert_eq!(snapshot["tables"].as_array().unwrap().len(), 21);
        assert_eq!(digest.len(), 64);
        serde_json::json!({
            "full21": snapshot,
            "full21_sha256": digest,
            "raw_boundary": boundary_metadata(connection).await,
        })
    }

    async fn removal_transition_predecessor(pool: &PgPool, state: &str) {
        super::super::prepare_http_database_staging(pool).await;
        let mut connection = pool.acquire().await.unwrap();
        seed_legacy(&mut connection).await;
        drop(connection);
        // Fixed frozen artifacts produce the real complete Account predecessor.
        // The serving helper will later gain the correction and is not used here.
        let root = artifact("ops/postgres-finalize-account-custody.sql");
        let credentials = auth7_sql();
        let mut tx = pool.begin().await.unwrap();
        operator(&mut tx).await;
        sqlx::raw_sql(sqlx::AssertSqlSafe(root))
            .execute(tx.as_mut())
            .await
            .unwrap();
        sqlx::raw_sql(sqlx::AssertSqlSafe(credentials))
            .execute(tx.as_mut())
            .await
            .unwrap();
        assert_eq!(
            root_state(&mut tx).await,
            "account_custody.native_finalized"
        );
        let credential_state: String = sqlx::query_scalar(sqlx::AssertSqlSafe(auth7_state_sql()))
            .fetch_one(tx.as_mut())
            .await
            .unwrap();
        assert_eq!(credential_state, "account_credentials.native_finalized");
        let absent: bool = sqlx::query_scalar(
            "SELECT to_regprocedure('public.platform_legacy_removal_admit_v1(uuid)') IS NULL
             AND to_regprocedure('public.account_legacy_actor_mutation_guard_v1()') IS NULL",
        )
        .fetch_one(tx.as_mut())
        .await
        .unwrap();
        assert!(absent, "unmodified complete Account predecessor required");
        assert_eq!(
            removal_transition_state(&mut tx, state).await,
            "account_legacy_removal_v1.install_required"
        );
        tx.commit().await.unwrap();
    }

    #[sqlx::test(migrations = false)]
    async fn native_removal_plain_transition_commits_and_standalone_replay_is_read_only(
        pool: PgPool,
    ) {
        let (finalizer, state, capture) = removal_transition_artifacts();
        removal_transition_predecessor(&pool, &state).await;
        let before = all_rows(&pool).await;
        let mut tx = pool.begin().await.unwrap();
        operator(&mut tx).await;
        assert_eq!(
            removal_transition_state(&mut tx, &state).await,
            "account_legacy_removal_v1.install_required"
        );
        sqlx::raw_sql(sqlx::AssertSqlSafe(finalizer.clone()))
            .execute(tx.as_mut())
            .await
            .expect("actual standalone Account correction must upgrade the exact predecessor");
        assert_eq!(
            removal_transition_state(&mut tx, &state).await,
            "account_legacy_removal_v1.finalized"
        );
        narrow_owner_acl(&mut tx).await;
        tx.commit().await.unwrap();
        assert!(
            all_rows(&pool).await == before,
            "committed plain correction must preserve every public row and migration ledger"
        );
        let mut tx = pool.begin().await.unwrap();
        operator(&mut tx).await;
        assert_eq!(
            removal_transition_state(&mut tx, &state).await,
            "account_legacy_removal_v1.finalized",
            "a new transaction must observe the committed successor"
        );
        let corrected = removal_transition_metadata(&mut tx, &capture).await;
        sqlx::raw_sql(sqlx::AssertSqlSafe(finalizer))
            .execute(tx.as_mut())
            .await
            .expect("standalone successor replay must certify without rewriting");
        assert!(
            removal_transition_metadata(&mut tx, &capture).await == corrected,
            "committed replay must preserve the full21 snapshot and exact raw metadata"
        );
        assert_eq!(
            removal_transition_state(&mut tx, &state).await,
            "account_legacy_removal_v1.finalized"
        );
        tx.commit().await.unwrap();
        assert!(
            all_rows(&pool).await == before,
            "replay must preserve every public row"
        );
    }

    #[sqlx::test(migrations = false)]
    async fn native_removal_observer_transition_and_replay_are_rollback_only(pool: PgPool) {
        let (finalizer, state, capture) = removal_transition_artifacts();
        let observer = artifact("ops/postgres-install-durability-observer.sql");
        removal_transition_predecessor(&pool, &state).await;
        let mut connection = pool.acquire().await.unwrap();
        let absent: bool = sqlx::query_scalar(
            "SELECT NOT EXISTS(SELECT 1 FROM pg_catalog.pg_roles
               WHERE rolname='console_durability_observer')
             AND to_regprocedure('public.console_durability_observation_v1(name,oid)') IS NULL",
        )
        .fetch_one(&mut *connection)
        .await
        .unwrap();
        assert!(
            absent,
            "OBSERVER_PREREQUISITE: this rollback-only fixture requires absent global observer custody"
        );
        let baseline = removal_transition_metadata(&mut connection, &capture).await;
        drop(connection);
        let all_before = all_rows(&pool).await;
        let mut tx = pool.begin().await.unwrap();
        let result = AssertUnwindSafe(async {
            operator(&mut tx).await;
            sqlx::raw_sql(sqlx::AssertSqlSafe(observer))
                .execute(tx.as_mut())
                .await
                .expect("actual optional observer installer");
            let present: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_roles
                   WHERE rolname='console_durability_observer' AND NOT rolcanlogin)
                 AND to_regprocedure('public.console_durability_observation_v1(name,oid)') IS NOT NULL
                 AND NOT EXISTS(SELECT 1 FROM pg_catalog.pg_proc p
                   CROSS JOIN LATERAL pg_catalog.aclexplode(COALESCE(p.proacl,
                     pg_catalog.acldefault('f',p.proowner))) a
                   WHERE p.oid='pg_catalog.pg_control_system()'::regprocedure
                     AND a.grantee=0 AND a.privilege_type='EXECUTE')",
            )
            .fetch_one(tx.as_mut())
            .await
            .unwrap();
            assert!(present, "actual observer role, function and native PUBLIC denial required");
            assert_eq!(
                removal_transition_state(&mut tx, &state).await,
                "account_legacy_removal_v1.install_required",
                "the exact observer predecessor must be recognized"
            );
            let retained = rows(&mut tx).await;
            sqlx::raw_sql(sqlx::AssertSqlSafe(finalizer.clone()))
                .execute(tx.as_mut())
                .await
                .expect("actual observer predecessor correction");
            assert_eq!(
                removal_transition_state(&mut tx, &state).await,
                "account_legacy_removal_v1.finalized"
            );
            narrow_owner_acl(&mut tx).await;
            assert!(rows(&mut tx).await == retained, "observer upgrade must retain Account/Auth7/users/audit rows");
            let corrected = removal_transition_metadata(&mut tx, &capture).await;
            sqlx::raw_sql(sqlx::AssertSqlSafe(finalizer))
                .execute(tx.as_mut())
                .await
                .expect("observer correction replay within the rollback-only transaction");
            assert!(removal_transition_metadata(&mut tx, &capture).await == corrected);
            assert!(rows(&mut tx).await == retained);
        })
        .catch_unwind()
        .await;
        tx.rollback().await.unwrap();
        let mut connection = pool.acquire().await.unwrap();
        assert!(
            removal_transition_metadata(&mut connection, &capture).await == baseline,
            "rollback must restore full21/raw metadata, observer roles/memberships and pg_control_system ACL"
        );
        drop(connection);
        assert!(
            all_rows(&pool).await == all_before,
            "observer rollback must preserve every public row and ledger"
        );
        if let Err(panic) = result {
            std::panic::resume_unwind(panic);
        }
    }

    #[sqlx::test(migrations = false)]
    async fn native_removal_wrong_partial_and_drifted_profiles_refuse_without_repair(pool: PgPool) {
        use sqlx::Acquire;
        let (finalizer, state, capture) = removal_transition_artifacts();
        removal_transition_predecessor(&pool, &state).await;
        let all_before = all_rows(&pool).await;
        for (successor, fault) in [
            (
                false,
                "ALTER TABLE public.auth_webauthn_credentials ADD CONSTRAINT removal_test_unreviewed_input CHECK (true)",
            ),
            (
                true,
                "ALTER TABLE public.company_actors DISABLE TRIGGER account_legacy_actor_mutation_guard_v1",
            ),
            (
                true,
                "GRANT EXECUTE ON FUNCTION public.platform_legacy_removal_membership_claim_v1(uuid,uuid) TO PUBLIC",
            ),
        ] {
            let mut tx = pool.begin().await.unwrap();
            operator(&mut tx).await;
            assert_eq!(
                removal_transition_state(&mut tx, &state).await,
                "account_legacy_removal_v1.install_required"
            );
            if successor {
                sqlx::raw_sql(sqlx::AssertSqlSafe(finalizer.clone()))
                    .execute(tx.as_mut())
                    .await
                    .unwrap();
                assert_eq!(
                    removal_transition_state(&mut tx, &state).await,
                    "account_legacy_removal_v1.finalized"
                );
            }
            sqlx::raw_sql(sqlx::AssertSqlSafe(fault.to_owned()))
                .execute(tx.as_mut())
                .await
                .unwrap();
            assert_eq!(
                removal_transition_state(&mut tx, &state).await,
                "account_legacy_removal_v1.profile_mismatch",
                "the intentional wrong/partial metadata must be observed before the refusal"
            );
            let faulted = removal_transition_metadata(&mut tx, &capture).await;
            let retained = rows(&mut tx).await;
            let mut attempt = tx.begin().await.unwrap();
            let error = sqlx::raw_sql(sqlx::AssertSqlSafe(finalizer.clone()))
                .execute(attempt.as_mut())
                .await
                .expect_err("unknown/partial metadata must refuse rather than repair");
            let db = error
                .as_database_error()
                .expect("actual PostgreSQL refusal");
            assert_eq!(db.code().as_deref(), Some("P0001"));
            assert_eq!(
                db.message(),
                "account_legacy_removal_v1.predecessor_mismatch"
            );
            attempt.rollback().await.unwrap();
            assert!(
                removal_transition_metadata(&mut tx, &capture).await == faulted,
                "refusal must preserve the exact deliberately faulted metadata"
            );
            assert!(rows(&mut tx).await == retained);
            assert_eq!(
                removal_transition_state(&mut tx, &state).await,
                "account_legacy_removal_v1.profile_mismatch"
            );
            tx.rollback().await.unwrap();
            assert!(
                all_rows(&pool).await == all_before,
                "each refusal/outer rollback must preserve every public row and ledger"
            );
        }
    }
}
