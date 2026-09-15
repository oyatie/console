//! Additive Auth7 operator transition tests. These execute the actual fixed SQL
//! artifacts under the existing marked disposable operator; data/fault fixtures
//! never install an owner, grant, resolver, custody constraint or serving profile.
//! Missing future SQL is a prerequisite, not an admissible product RED.

use super::{account_custody_finalizer_sql, prepare_http_database_staging};
use console_kernel_core::OrgId;
use console_platform_test_support::{TestDatabaseLogin, login_test_pool};
use serde_json::Value;
use sqlx::{PgConnection, PgPool};
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
    sqlx::raw_sql("SET LOCAL search_path=pg_catalog,pg_temp; SET LOCAL statement_timeout='60s'; SET LOCAL lock_timeout='5s'")
        .execute(connection).await.unwrap();
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
    let mut expected: Vec<_> = original.iter().map(Value::to_string).collect();
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
            Value::Object(projection).to_string()
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

async fn installed(pool: &PgPool) {
    prepare_http_database_staging(pool).await;
    let mut tx = pool.begin().await.unwrap();
    composed(&mut tx)
        .await
        .expect("actual composed root+Auth7 install");
    complete_profile(&mut tx).await;
    tx.commit().await.unwrap();
}

#[sqlx::test(migrations = false)]
async fn composed_populated_transition_preserves_rows_and_both_replays(pool: PgPool) {
    prepare_http_database_staging(&pool).await;
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
    prepare_http_database_staging(&pool).await;
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
