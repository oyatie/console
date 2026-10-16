//! Test-only historical numbered-schema producer. No product downgrade mode.
//! Ordinary current fixtures continue to use console_app::run_migrations.
use super::*;
use sha2::{Digest, Sha256, Sha384};
use sqlx::{
    Connection, PgConnection,
    migrate::{Migrate, Migration, Migrator},
};
use std::collections::BTreeMap;

const TARGET: i64 = 228;
const ROSTER: &str = include_str!("preextension/historical228-roster.json");
const ROSTER_SHA256: &str = "489f5f5e8ace5c0049e2f9b4affbaf35c74d6db487df6ed9992d6f812523d742";
static MIGRATIONS: Migrator = sqlx::migrate!("../crates/platform/db/migrations");

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedMigration {
    version: i64,
    filename: String,
    sha256: String,
    sha384: String,
    description: String,
    no_tx: bool,
}

fn expected() -> Vec<ExpectedMigration> {
    assert!(
        hex::encode(Sha256::digest(ROSTER.as_bytes())) == ROSTER_SHA256,
        "historical228.immutable_roster_changed"
    );
    let roster: Vec<ExpectedMigration> = serde_json::from_str(ROSTER).unwrap();
    assert_eq!(roster.len(), TARGET as usize);
    for (index, item) in roster.iter().enumerate() {
        assert_eq!(item.version, index as i64 + 1);
        assert!(
            item.filename.starts_with(&format!("{:04}_", item.version))
                && item.filename.ends_with(".sql")
                && !item.filename.contains('/')
                && item.sha256.len() == 64
                && item.sha384.len() == 96
        );
    }
    roster
}

fn source_files() -> Result<BTreeMap<String, Vec<u8>>, &'static str> {
    let directory =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../crates/platform/db/migrations");
    let mut files = BTreeMap::new();
    for entry in std::fs::read_dir(directory).map_err(|_| "historical228.source_directory")? {
        let entry = entry.map_err(|_| "historical228.source_entry")?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| "historical228.source_name")?;
        if !name.ends_with(".sql") {
            continue;
        }
        if !entry
            .file_type()
            .map_err(|_| "historical228.source_type")?
            .is_file()
        {
            return Err("historical228.nonregular_source");
        }
        let bytes = std::fs::read(entry.path()).map_err(|_| "historical228.source_read")?;
        if files.insert(name, bytes).is_some() {
            return Err("historical228.duplicate_source");
        }
    }
    Ok(files)
}

fn source_prefix_matches(
    expected: &[ExpectedMigration],
    files: &BTreeMap<String, Vec<u8>>,
    embedded: &[Migration],
) -> bool {
    let mut prefix = BTreeMap::new();
    for (name, bytes) in files {
        let Some(version) = name
            .split_once('_')
            .and_then(|(v, _)| v.parse::<i64>().ok())
        else {
            return false;
        };
        if version <= 0 {
            return false;
        }
        if version <= TARGET && prefix.insert(version, (name, bytes)).is_some() {
            return false;
        }
    }
    let embedded: Vec<_> = embedded.iter().filter(|m| m.version <= TARGET).collect();
    prefix.len() == TARGET as usize
        && embedded.len() == expected.len()
        && expected.iter().enumerate().all(|(index, wanted)| {
            let Some((name, bytes)) = prefix.get(&wanted.version) else {
                return false;
            };
            let migration = embedded[index];
            wanted.version == index as i64 + 1
                && **name == wanted.filename
                && hex::encode(Sha256::digest(bytes)) == wanted.sha256
                && hex::encode(Sha384::digest(bytes)) == wanted.sha384
                && migration.version == wanted.version
                && migration.migration_type == sqlx::migrate::MigrationType::Simple
                && migration.description == wanted.description
                && migration.no_tx == wanted.no_tx
                && hex::encode(Sha256::digest(migration.sql.as_str().as_bytes())) == wanted.sha256
                && hex::encode(Sha384::digest(migration.sql.as_str().as_bytes())) == wanted.sha384
                && hex::encode(migration.checksum.as_ref()) == wanted.sha384
        })
}

fn ledger_prefix_matches(expected: &[ExpectedMigration], rows: &[(i64, bool, Vec<u8>)]) -> bool {
    rows.len() <= TARGET as usize
        && rows
            .iter()
            .enumerate()
            .all(|(index, (version, success, checksum))| {
                *success
                    && *version == index as i64 + 1
                    && hex::encode(checksum) == expected[index].sha384
            })
}

async fn ledger(
    connection: &mut PgConnection,
) -> Result<Option<Vec<(i64, bool, Vec<u8>)>>, &'static str> {
    let present: bool =
        sqlx::query_scalar("SELECT pg_catalog.to_regclass('public._sqlx_migrations') IS NOT NULL")
            .fetch_one(&mut *connection)
            .await
            .map_err(|_| "historical228.ledger_presence")?;
    if !present {
        return Ok(None);
    }
    sqlx::query_as("SELECT version,success,checksum FROM public._sqlx_migrations ORDER BY version")
        .fetch_all(connection)
        .await
        .map(Some)
        .map_err(|_| "historical228.ledger_read")
}

// This is the same bounded actual-LOGIN identity contract as the existing226
// fixture, with explicit physical connection budget readback and target binding.
// It is not a claim that private production validators were called.
async fn validate_owner(
    connection: &mut PgConnection,
    database: &str,
    oid: i64,
) -> Result<(), &'static str> {
    sqlx::raw_sql("SET SESSION search_path=public,pg_temp; SET SESSION lock_timeout='5s'; SET SESSION statement_timeout='60s'")
        .execute(&mut *connection)
        .await
        .map_err(|_| "historical228.owner_budgets")?;
    let valid: bool = sqlx::query_scalar(
        "SELECT session_user='console_app' AND current_user='console_app' AND current_database()=$1 AND d.oid::bigint=$2 AND pg_catalog.pg_get_userbyid(d.datdba)='console_app' AND r.rolcanlogin AND r.rolinherit AND NOT r.rolsuper AND r.rolbypassrls AND NOT r.rolcreatedb AND NOT r.rolcreaterole AND NOT r.rolreplication AND current_setting('lock_timeout')::interval=interval '5 seconds' AND current_setting('statement_timeout')::interval=interval '60 seconds' FROM pg_catalog.pg_roles r CROSS JOIN pg_catalog.pg_database d WHERE r.rolname=current_user AND d.datname=current_database()"
    ).bind(database).bind(oid).fetch_one(connection).await.map_err(|_| "historical228.owner_identity")?;
    if valid {
        Ok(())
    } else {
        Err("historical228.owner_or_target_mismatch")
    }
}

async fn produce(url: &str, database: &str, oid: i64) -> Result<(), &'static str> {
    let expected = expected();
    let embedded: Vec<_> = MIGRATIONS.iter().cloned().collect();
    if !source_prefix_matches(&expected, &source_files()?, &embedded) {
        return Err("historical228.filesystem_or_embedded_prefix_drift");
    }
    let mut connection = PgConnection::connect(url)
        .await
        .map_err(|_| "historical228.owner_connection")?;
    let outcome = async {
        validate_owner(&mut connection, database, oid).await?;
        // Reuse SQLx's own lock API/key. PostgreSQL session advisory locks are
        // reentrant: run_to takes/releases its nested acquisition, while this
        // outer acquisition protects prefix preflight and postflight too.
        connection.lock().await.map_err(|_| "historical228.migration_lock")?;
        match ledger(&mut connection).await? {
            Some(rows) if !ledger_prefix_matches(&expected, &rows) => {
                return Err("historical228.dirty_newer_or_foreign_ledger");
            }
            None => {
                let empty: bool = sqlx::query_scalar("SELECT NOT EXISTS(SELECT 1 FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname !~ '^pg_' AND n.nspname<>'information_schema')")
                    .fetch_one(&mut connection).await.map_err(|_| "historical228.freshness_read")?;
                if !empty { return Err("historical228.unledgered_existing_database"); }
            }
            Some(_) => {}
        }
        MIGRATIONS.run_to(TARGET, &mut connection).await.map_err(|_| "historical228.sqlx_run_failed")?;
        // Reuse the public existing helper on the same validated checkout. This
        // is current vendor queue setup; only numbered migrations are pinned228.
        console_platform_jobs::migrate_and_reconcile_apalis_postgres(&mut connection)
            .await.map_err(|_| "historical228.apalis_setup_failed")?;
        validate_owner(&mut connection, database, oid).await?;
        let rows = ledger(&mut connection).await?.ok_or("historical228.missing_produced_ledger")?;
        if rows.len() != TARGET as usize || !ledger_prefix_matches(&expected, &rows) {
            return Err("historical228.produced_ledger_mismatch");
        }
        connection.unlock().await.map_err(|_| "historical228.migration_unlock")?;
        Ok(())
    }.await;
    // Close the physical connection on every result; SQLx early errors can
    // retain session advisory locks. Never return it to a reusable pool.
    let closed = connection
        .close()
        .await
        .map_err(|_| "historical228.owner_close");
    outcome.and(closed)
}

pub(super) async fn prepare_historical228_database(pool: &PgPool) -> String {
    let config = prepare_http_migration_config(pool).await;
    let database = pool.connect_options().get_database().unwrap().to_owned();
    let oid: i64 = sqlx::query_scalar(
        "SELECT oid::bigint FROM pg_catalog.pg_database WHERE datname=current_database()",
    )
    .fetch_one(pool)
    .await
    .unwrap();
    produce(config.database_url.as_deref().unwrap(), &database, oid)
        .await
        .expect("historical228 genuine SQLx fixture producer");
    config.database_url.unwrap()
}

fn frozen_sources() -> BTreeMap<&'static str, &'static str> {
    let mut sources = BTreeMap::new();
    let sql = include_str!("preextension/postgres-finalize-account-custody.sql");
    assert_eq!(
        hex::encode(Sha256::digest(sql.as_bytes())),
        "fb532ef3b82f6d03b58d6e164a26567683039444098cf1a0a98d33e27e5dd4f8",
        "historical preextension source drift"
    );
    sources.insert("postgres-finalize-account-custody.sql", sql);
    let sql = include_str!("preextension/postgres-finalize-account-credentials.sql");
    assert_eq!(
        hex::encode(Sha256::digest(sql.as_bytes())),
        "2f960163c9bd8832cdd3a058a6ae69d7503cf1009c476ef292af9443036624d9",
        "historical preextension source drift"
    );
    sources.insert("postgres-finalize-account-credentials.sql", sql);
    let sql = include_str!("preextension/postgres-verify-account-native.sql");
    assert_eq!(
        hex::encode(Sha256::digest(sql.as_bytes())),
        "c4e30ee9560b4d443acad1813a63667cbab9a32aadfbd893542f4b054bf6b308",
        "historical preextension source drift"
    );
    sources.insert("postgres-verify-account-native.sql", sql);
    let sql = include_str!("preextension/account_custody_state.sql");
    assert_eq!(
        hex::encode(Sha256::digest(sql.as_bytes())),
        "5fb0a2608c5904fe0c1d58723290b9603afb9af08eef19c410d023a484230865",
        "historical preextension source drift"
    );
    sources.insert("account_custody_state.sql", sql);
    let sql = include_str!("preextension/account_credential_custody_state.sql");
    assert_eq!(
        hex::encode(Sha256::digest(sql.as_bytes())),
        "0bc6fe6579414ebed546d076c0ec1b7f1cc5ba30f4fe5e135159f4e75c0416a2",
        "historical preextension source drift"
    );
    sources.insert("account_credential_custody_state.sql", sql);
    let sql = include_str!("preextension/postgres-company-enrollment-input.sql");
    assert_eq!(
        hex::encode(Sha256::digest(sql.as_bytes())),
        "cbc685bff861fec809b930e57673cce5426a771a236323471955510de0631290",
        "historical preextension source drift"
    );
    sources.insert("postgres-company-enrollment-input.sql", sql);
    let sql = include_str!("preextension/postgres-company-enrollment-schema.sql");
    assert_eq!(
        hex::encode(Sha256::digest(sql.as_bytes())),
        "7e1dbee080cb3f09d0133ac9a30857ee5101a076e1cec171ec06849116e672bb",
        "historical preextension source drift"
    );
    sources.insert("postgres-company-enrollment-schema.sql", sql);
    let sql = include_str!("preextension/postgres-company-enrollment-intake.sql");
    assert_eq!(
        hex::encode(Sha256::digest(sql.as_bytes())),
        "2d8e5bc991e35ca1c8195d5ee9f0b5c2f367a96e4d77cf613945f9d533cc7532",
        "historical preextension source drift"
    );
    sources.insert("postgres-company-enrollment-intake.sql", sql);
    let sql = include_str!("preextension/postgres-company-enrollment-guards.sql");
    assert_eq!(
        hex::encode(Sha256::digest(sql.as_bytes())),
        "e7ad2c8343fa596073ad82842b8eae9e2a67eb0aba8903daf86b853e59a4575a",
        "historical preextension source drift"
    );
    sources.insert("postgres-company-enrollment-guards.sql", sql);
    sources
}

pub(super) async fn prepare_company_preextension_database(pool: &PgPool) {
    prepare_historical228_database(pool).await;
    let sources = frozen_sources();
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SET TRANSACTION ISOLATION LEVEL READ COMMITTED")
        .execute(tx.as_mut())
        .await
        .unwrap();
    sqlx::raw_sql("SET LOCAL statement_timeout='60s'; SET LOCAL jit=off; SET LOCAL lock_timeout='5s'; SET LOCAL search_path=pg_catalog,pg_temp")
        .execute(tx.as_mut()).await.unwrap();
    let identity:(String,String,bool)=sqlx::query_as("SELECT session_user::text,current_user::text,current_setting('console.sqlx_test_bootstrap',true)='buck-sqlx-superuser-v1' AND (SELECT rolsuper FROM pg_roles WHERE rolname=current_user)")
        .fetch_one(tx.as_mut()).await.unwrap();
    assert_eq!(
        identity,
        (
            "console_buck_admin".into(),
            "console_buck_admin".into(),
            true
        )
    );
    // Exact shared-role coordination used by the current real finalizer helper.
    sqlx::query("SAVEPOINT account_initial_role")
        .execute(tx.as_mut())
        .await
        .unwrap();
    let _:String=sqlx::query_scalar("SELECT oid::text FROM pg_catalog.pg_authid WHERE rolname='console_account_owner' FOR UPDATE").fetch_one(tx.as_mut()).await.unwrap();
    let committed: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_roles WHERE rolname='console_credential_owner')",
    )
    .fetch_one(tx.as_mut())
    .await
    .unwrap();
    if committed {
        sqlx::query("ROLLBACK TO SAVEPOINT account_initial_role")
            .execute(tx.as_mut())
            .await
            .unwrap();
    }
    sqlx::query("RELEASE SAVEPOINT account_initial_role")
        .execute(tx.as_mut())
        .await
        .unwrap();
    for name in [
        "postgres-finalize-account-custody.sql",
        "postgres-finalize-account-credentials.sql",
    ] {
        sqlx::raw_sql(sources[name])
            .execute(tx.as_mut())
            .await
            .unwrap();
    }
    tx.commit().await.unwrap();
    let mut connection = pool.acquire().await.unwrap();
    sqlx::raw_sql("SET search_path=pg_catalog,pg_temp; SET statement_timeout='60s'; SET lock_timeout='5s'; SET jit=off").execute(&mut *connection).await.unwrap();
    for (file, expected) in [
        (
            "account_custody_state.sql",
            "account_custody.native_finalized",
        ),
        (
            "account_credential_custody_state.sql",
            "account_credentials.native_finalized",
        ),
    ] {
        let actual: String = sqlx::query_scalar(sources[file])
            .fetch_one(&mut *connection)
            .await
            .unwrap();
        assert_eq!(
            actual, expected,
            "exact frozen finalized predecessor classifier"
        );
    }
    let mut tx = connection.begin().await.unwrap();
    for name in [
        "postgres-company-enrollment-input.sql",
        "postgres-company-enrollment-schema.sql",
        "postgres-company-enrollment-intake.sql",
        "postgres-company-enrollment-guards.sql",
    ] {
        sqlx::raw_sql(sources[name])
            .execute(tx.as_mut())
            .await
            .unwrap();
    }
    tx.commit().await.unwrap();
    for file in [
        "account_custody_state.sql",
        "account_credential_custody_state.sql",
    ] {
        let actual: String = sqlx::query_scalar(sources[file])
            .fetch_one(&mut *connection)
            .await
            .unwrap();
        assert_eq!(
            actual, "account_native.profile_mismatch",
            "frozen private intake is not complete serving profile"
        );
    }
    let absent:bool=sqlx::query_scalar("SELECT to_regclass('public.company_enrollment_effect_bindings') IS NULL AND to_regclass('public.group_authority_heads') IS NULL AND to_regclass('public.group_membership_revisions') IS NULL AND NOT EXISTS(SELECT 1 FROM pg_attribute WHERE attrelid IN ('public.organizations'::regclass,'public.groups'::regclass) AND attname='origin_account_id' AND NOT attisdropped)")
        .fetch_one(&mut *connection).await.unwrap();
    assert!(
        absent,
        "historical fixture accidentally installed future topology"
    );
}

#[test]
fn company_preextension_source_locks_reject_drift_and_keep_exact_228_prefix() {
    assert_eq!(frozen_sources().len(), 9);
    let expected = expected();
    let files = source_files().unwrap();
    let embedded: Vec<_> = MIGRATIONS.iter().cloned().collect();
    assert!(source_prefix_matches(&expected, &files, &embedded));
    let mut changed = files.clone();
    changed.get_mut(&expected[227].filename).unwrap().push(b' ');
    assert!(!source_prefix_matches(&expected, &changed, &embedded));
    let mut changed = files.clone();
    changed.remove(&expected[0].filename);
    assert!(!source_prefix_matches(&expected, &changed, &embedded));
    let mut changed = embedded.clone();
    changed[227].checksum = std::borrow::Cow::Owned(vec![0; 48]);
    assert!(!source_prefix_matches(&expected, &files, &changed));
    let rows: Vec<_> = expected
        .iter()
        .map(|m| (m.version, true, hex::decode(&m.sha384).unwrap()))
        .collect();
    assert!(ledger_prefix_matches(&expected, &rows));
    let mut changed = rows.clone();
    changed[227].1 = false;
    assert!(!ledger_prefix_matches(&expected, &changed));
    let mut changed = rows.clone();
    changed[0].2[0] ^= 1;
    assert!(!ledger_prefix_matches(&expected, &changed));
    let mut changed = rows.clone();
    changed.push((229, true, vec![0; 48]));
    assert!(!ledger_prefix_matches(&expected, &changed));
}
