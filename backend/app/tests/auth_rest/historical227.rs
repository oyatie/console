//! Test-only historical numbered-schema producer. No product downgrade mode.
//! Ordinary current fixtures continue to use console_app::run_migrations.
use super::*;
use sha2::{Digest, Sha256, Sha384};
use sqlx::{
    Connection, PgConnection,
    migrate::{Migrate, Migration, Migrator},
};
use std::collections::BTreeMap;

const TARGET: i64 = 227;
const ROSTER: &str = include_str!("fixtures/historical-migrations227-9de5c767.json");
const ROSTER_SHA256: &str = "caa734fedcfb956a0b651adc95d7502be61f31b99c567dd855ea454daf51dfce";
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
        "historical227.immutable_roster_changed"
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
    for entry in std::fs::read_dir(directory).map_err(|_| "historical227.source_directory")? {
        let entry = entry.map_err(|_| "historical227.source_entry")?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| "historical227.source_name")?;
        if !name.ends_with(".sql") {
            continue;
        }
        if !entry
            .file_type()
            .map_err(|_| "historical227.source_type")?
            .is_file()
        {
            return Err("historical227.nonregular_source");
        }
        let bytes = std::fs::read(entry.path()).map_err(|_| "historical227.source_read")?;
        if files.insert(name, bytes).is_some() {
            return Err("historical227.duplicate_source");
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
            .map_err(|_| "historical227.ledger_presence")?;
    if !present {
        return Ok(None);
    }
    sqlx::query_as("SELECT version,success,checksum FROM public._sqlx_migrations ORDER BY version")
        .fetch_all(connection)
        .await
        .map(Some)
        .map_err(|_| "historical227.ledger_read")
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
        .map_err(|_| "historical227.owner_budgets")?;
    let valid: bool = sqlx::query_scalar(
        "SELECT session_user='console_app' AND current_user='console_app' AND current_database()=$1 AND d.oid::bigint=$2 AND pg_catalog.pg_get_userbyid(d.datdba)='console_app' AND r.rolcanlogin AND r.rolinherit AND NOT r.rolsuper AND r.rolbypassrls AND NOT r.rolcreatedb AND NOT r.rolcreaterole AND NOT r.rolreplication AND current_setting('lock_timeout')::interval=interval '5 seconds' AND current_setting('statement_timeout')::interval=interval '60 seconds' FROM pg_catalog.pg_roles r CROSS JOIN pg_catalog.pg_database d WHERE r.rolname=current_user AND d.datname=current_database()"
    ).bind(database).bind(oid).fetch_one(connection).await.map_err(|_| "historical227.owner_identity")?;
    if valid {
        Ok(())
    } else {
        Err("historical227.owner_or_target_mismatch")
    }
}

async fn produce(url: &str, database: &str, oid: i64) -> Result<(), &'static str> {
    let expected = expected();
    let embedded: Vec<_> = MIGRATIONS.iter().cloned().collect();
    if !source_prefix_matches(&expected, &source_files()?, &embedded) {
        return Err("historical227.filesystem_or_embedded_prefix_drift");
    }
    let mut connection = PgConnection::connect(url)
        .await
        .map_err(|_| "historical227.owner_connection")?;
    let outcome = async {
        validate_owner(&mut connection, database, oid).await?;
        // Reuse SQLx's own lock API/key. PostgreSQL session advisory locks are
        // reentrant: run_to takes/releases its nested acquisition, while this
        // outer acquisition protects prefix preflight and postflight too.
        connection.lock().await.map_err(|_| "historical227.migration_lock")?;
        match ledger(&mut connection).await? {
            Some(rows) if !ledger_prefix_matches(&expected, &rows) => {
                return Err("historical227.dirty_newer_or_foreign_ledger");
            }
            None => {
                let empty: bool = sqlx::query_scalar("SELECT NOT EXISTS(SELECT 1 FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname !~ '^pg_' AND n.nspname<>'information_schema')")
                    .fetch_one(&mut connection).await.map_err(|_| "historical227.freshness_read")?;
                if !empty { return Err("historical227.unledgered_existing_database"); }
            }
            Some(_) => {}
        }
        MIGRATIONS.run_to(TARGET, &mut connection).await.map_err(|_| "historical227.sqlx_run_failed")?;
        // Reuse the public existing helper on the same validated checkout. This
        // is current vendor queue setup; only numbered migrations are pinned227.
        console_platform_jobs::migrate_and_reconcile_apalis_postgres(&mut connection)
            .await.map_err(|_| "historical227.apalis_setup_failed")?;
        validate_owner(&mut connection, database, oid).await?;
        let rows = ledger(&mut connection).await?.ok_or("historical227.missing_produced_ledger")?;
        if rows.len() != TARGET as usize || !ledger_prefix_matches(&expected, &rows) {
            return Err("historical227.produced_ledger_mismatch");
        }
        connection.unlock().await.map_err(|_| "historical227.migration_unlock")?;
        Ok(())
    }.await;
    // Close the physical connection on every result; SQLx early errors can
    // retain session advisory locks. Never return it to a reusable pool.
    let closed = connection
        .close()
        .await
        .map_err(|_| "historical227.owner_close");
    outcome.and(closed)
}

pub(super) async fn prepare_historical227_database(pool: &PgPool) -> String {
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
        .expect("historical227 genuine SQLx fixture producer");
    config.database_url.unwrap()
}

#[test]
fn historical227_prefix_rejects_source_and_embedded_drift_without_rejecting_later_suffix() {
    let expected = expected();
    let files = source_files().unwrap();
    let embedded: Vec<_> = MIGRATIONS.iter().cloned().collect();
    assert!(source_prefix_matches(&expected, &files, &embedded));
    let mut suffix = files.clone();
    suffix.insert("9999_unexecuted_future.sql".into(), b"SELECT 1;".to_vec());
    assert!(source_prefix_matches(&expected, &suffix, &embedded));
    let name = &expected[0].filename;
    let mut altered = files.clone();
    altered.get_mut(name).unwrap().push(b' ');
    assert!(!source_prefix_matches(&expected, &altered, &embedded));
    let mut missing = files.clone();
    missing.remove(name);
    assert!(!source_prefix_matches(&expected, &missing, &embedded));
    let mut renamed = files.clone();
    let bytes = renamed.remove(name).unwrap();
    renamed.insert("0001_renamed.sql".into(), bytes.clone());
    assert!(!source_prefix_matches(&expected, &renamed, &embedded));
    let mut duplicate = files.clone();
    duplicate.insert("0001_duplicate.sql".into(), bytes);
    assert!(!source_prefix_matches(&expected, &duplicate, &embedded));
    let mut stale = embedded.clone();
    stale[0].checksum = std::borrow::Cow::Owned(vec![0; 48]);
    assert!(!source_prefix_matches(&expected, &files, &stale));
    let mut missing_embedded = embedded.clone();
    missing_embedded.remove(0);
    assert!(!source_prefix_matches(&expected, &files, &missing_embedded));
    let mut duplicate_embedded = embedded.clone();
    duplicate_embedded.insert(0, embedded[0].clone());
    assert!(!source_prefix_matches(
        &expected,
        &files,
        &duplicate_embedded
    ));
}

#[test]
fn historical227_ledger_requires_exact_successful_immutable_prefix() {
    let expected = expected();
    let rows: Vec<_> = expected
        .iter()
        .map(|m| (m.version, true, hex::decode(&m.sha384).unwrap()))
        .collect();
    assert!(ledger_prefix_matches(&expected, &[]));
    assert!(ledger_prefix_matches(&expected, &rows[..226]));
    assert!(ledger_prefix_matches(&expected, &rows));
    let mut dirty = rows.clone();
    dirty[0].1 = false;
    assert!(!ledger_prefix_matches(&expected, &dirty));
    let mut newer = rows.clone();
    newer.push((228, true, vec![0; 48]));
    assert!(!ledger_prefix_matches(&expected, &newer));
    let mut failed_newer = rows.clone();
    failed_newer.push((228, false, vec![0; 48]));
    assert!(!ledger_prefix_matches(&expected, &failed_newer));
    let mut missing = rows.clone();
    missing.remove(0);
    assert!(!ledger_prefix_matches(&expected, &missing));
    let mut bad_checksum = rows.clone();
    bad_checksum[0].2[0] ^= 1;
    assert!(!ledger_prefix_matches(&expected, &bad_checksum));
    let mut duplicate = rows.clone();
    duplicate[1] = rows[0].clone();
    assert!(!ledger_prefix_matches(&expected, &duplicate));
}

#[sqlx::test(migrations = false)]
async fn historical227_real_sqlx_prefix_and_replay_preserve_exact_ledger(pool: PgPool) {
    let owner_url = prepare_historical227_database(&pool).await;
    let database = pool.connect_options().get_database().unwrap().to_owned();
    let oid: i64 = sqlx::query_scalar(
        "SELECT oid::bigint FROM pg_catalog.pg_database WHERE datname=current_database()",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let snapshot =
        "SELECT jsonb_agg(to_jsonb(m) ORDER BY version)::text FROM public._sqlx_migrations m";
    let before: String = sqlx::query_scalar(snapshot).fetch_one(&pool).await.unwrap();
    produce(&owner_url, &database, oid).await.unwrap();
    let after: String = sqlx::query_scalar(snapshot).fetch_one(&pool).await.unwrap();
    assert!(
        before == after,
        "historical227 replay changed actual SQLx ledger bytes"
    );
}

#[sqlx::test(migrations = false)]
async fn historical227_real_sqlx_failure_keeps_ledger_and_fault_without_repair(pool: PgPool) {
    let config = prepare_http_migration_config(&pool).await;
    let url = config.database_url.as_deref().unwrap();
    let database = pool.connect_options().get_database().unwrap().to_owned();
    let oid: i64 = sqlx::query_scalar(
        "SELECT oid::bigint FROM pg_catalog.pg_database WHERE datname=current_database()",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let mut owner = PgConnection::connect(url).await.unwrap();
    validate_owner(&mut owner, &database, oid).await.unwrap();
    // Genuine historical prefix, then a negative conflicting-index fault. Never
    // insert/update a SQLx ledger row. Real no-transaction migration84 must fail.
    MIGRATIONS.run_to(83, &mut owner).await.unwrap();
    sqlx::query("CREATE INDEX employees_org_directory_order_idx ON public.employees(id)")
        .execute(&mut owner)
        .await
        .unwrap();
    owner.close().await.unwrap();
    let snapshot =
        "SELECT jsonb_agg(to_jsonb(m) ORDER BY version)::text FROM public._sqlx_migrations m";
    let before: String = sqlx::query_scalar(snapshot).fetch_one(&pool).await.unwrap();
    let index_sql =
        "SELECT pg_catalog.pg_get_indexdef('public.employees_org_directory_order_idx'::regclass)";
    let original_index: String = sqlx::query_scalar(index_sql)
        .fetch_one(&pool)
        .await
        .unwrap();
    for _ in 0..2 {
        assert_eq!(
            produce(url, &database, oid).await,
            Err("historical227.sqlx_run_failed")
        );
        let after: String = sqlx::query_scalar(snapshot).fetch_one(&pool).await.unwrap();
        let retained_index: String = sqlx::query_scalar(index_sql)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!(
            before == after && original_index == retained_index,
            "failed historical migration/retry must preserve genuine ledger and negative fault"
        );
    }
}

// This opt-in entry is an existing integration-test executable operation, not
// a new migration binary. Ordinary test selection does not discover it.
#[cfg(feature = "test-historical227-producer")]
#[tokio::test]
async fn historical227_operator_owned_database_producer() {
    assert!(
        std::env::vars_os()
            .all(|(name, value)| !name.to_string_lossy().starts_with("PG") || value.is_empty()),
        "historical227 ambient PostgreSQL overrides forbidden"
    );
    let raw = std::env::var("CONSOLE_HISTORICAL227_DATABASE_URL")
        .expect("explicit private owned-fixture URL");
    let url = Url::parse(&raw).expect("historical227 fixture URL shape");
    let database = std::env::var("CONSOLE_HISTORICAL227_EXPECTED_DATABASE")
        .expect("independent database descriptor");
    let oid: i64 = std::env::var("CONSOLE_HISTORICAL227_EXPECTED_DATABASE_OID")
        .expect("independent database OID")
        .parse()
        .unwrap();
    assert!(
        matches!(url.scheme(), "postgres" | "postgresql")
            && url.username() == "console_app"
            && url.password().is_some_and(|p| !p.is_empty())
            && url.host_str() == Some("localhost")
            && url.port().is_some()
            && url.fragment().is_none()
            && oid > 0
            && database.starts_with("wrapper_native_")
            && database.len() <= 63
            && database
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
            && url.path() == format!("/{database}"),
        "historical227 owned TLS target required"
    );
    let query: Vec<_> = url.query_pairs().collect();
    assert!(
        query.len() == 2
            && query
                .iter()
                .filter(|(k, v)| k == "sslmode" && v == "verify-full")
                .count()
                == 1
            && query
                .iter()
                .filter(
                    |(k, v)| k == "sslrootcert" && std::path::Path::new(v.as_ref()).is_absolute()
                )
                .count()
                == 1,
        "historical227 only explicit verify-full and private CA parameters"
    );
    produce(&raw, &database, oid)
        .await
        .expect("historical227 owned producer");
    println!("HISTORICAL227_PRODUCER_PASS");
}
