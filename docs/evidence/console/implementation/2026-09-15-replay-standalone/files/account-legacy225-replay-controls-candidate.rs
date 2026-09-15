#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Populated actual numbered-migration compatibility, starting at225.
//! No test-created Account schema, backfill, cutover authority or HTTP handlers.
//! Captured historical auth-owner output retains real key/family/token/ceremony/audit rows. Fixture
//! ownership is privileged setup, not evidence of serving-role authorization.
use console_app::{AppConfig, AppRole, run_migrations};
use console_platform_auth::RefreshTokenStore;
use serde_json::{Value, json};
use sqlx::PgPool;
use time::{Duration, OffsetDateTime};
use url::Url;
use uuid::Uuid;

struct LegacyFixture {
    subjects: Vec<Uuid>,
    audit_ids: Vec<Uuid>,
    revoked_token: String,
    credential_count: i64,
    snapshot: Value,
    old_checksums: Vec<(i64, Vec<u8>)>,
    migrate_config: AppConfig,
}

async fn old_migration_login(admin: &PgPool) -> (PgPool, AppConfig) {
    let mut connection = admin.acquire().await.unwrap();
    let (session, current, database, marked, empty): (String, String, String, bool, bool) =
        sqlx::query_as(r#"SELECT session_user::text,current_user::text,current_database(),
            current_setting('console.sqlx_test_bootstrap',true)='buck-sqlx-superuser-v1'
            AND (SELECT rolsuper FROM pg_catalog.pg_roles WHERE rolname=current_user)
            AND (SELECT pg_get_userbyid(datdba)=current_user FROM pg_catalog.pg_database WHERE datname=current_database()),
            NOT EXISTS(SELECT 1 FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
                WHERE n.nspname !~ '^pg_' AND n.nspname<>'information_schema')"#)
        .fetch_one(&mut *connection).await.unwrap();
    assert_eq!(
        (session.as_str(), current.as_str()),
        ("console_buck_admin", "console_buck_admin")
    );
    assert!(
        marked && empty,
        "requires existing marked empty disposable SQLx database"
    );
    let suffix = database.strip_prefix("_sqlx_test_").unwrap();
    assert!(
        suffix.len() == 52
            && suffix
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
    );
    let binding = std::env::var("CONSOLE_APALIS_OWNER_DATABASE_URL")
        .expect("migration-owner transport prerequisite absent");
    let mut url = Url::parse(&binding).unwrap();
    assert!(matches!(url.scheme(), "postgres" | "postgresql"));
    assert!(url.username() == "console_app" && url.password().is_some_and(|v| !v.is_empty()));
    assert!(url.query().is_none() && url.fragment().is_none());
    let options = admin.connect_options();
    assert!(
        url.host_str() == Some(options.get_host())
            && url.port().unwrap_or(5432) == options.get_port()
    );
    url.set_path(&database);
    // Same empty-container provisioning boundary as auth_rest::prepare_http_database.
    // No table/role grants or migration bypass are installed by this test.
    sqlx::raw_sql("DO $owner$ BEGIN EXECUTE format('ALTER DATABASE %I OWNER TO console_app',current_database()); END $owner$;")
        .execute(&mut *connection).await.unwrap();
    drop(connection);
    let migrator = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(url.as_str())
        .await
        .expect("actual migration LOGIN connection prerequisite");
    let identity: (String, String) = sqlx::query_as("SELECT session_user::text,current_user::text")
        .fetch_one(&migrator)
        .await
        .unwrap();
    assert_eq!(
        identity,
        ("console_app".to_owned(), "console_app".to_owned())
    );
    let config = AppConfig::from_pairs([
        ("CONSOLE_APP_ROLE", AppRole::Migrate.to_string()),
        ("DATABASE_URL", url.to_string()),
    ])
    .unwrap();
    (migrator, config)
}

async fn material_snapshot(pool: &PgPool, subjects: &[Uuid], audit_ids: &[Uuid]) -> Value {
    // Explicit old fields permit additive columns/Company-ownership removal.
    // Live credentials may gain revocations at cutover; old protected material
    // and pre-existing revocations below must remain unchanged.
    let keys: Vec<Value> = sqlx::query_scalar(
        r#"SELECT jsonb_build_object(
        'id',id,'user_id',user_id,'credential_id',credential_id,'passkey_json',passkey_json,
        'created_at',created_at,'last_used_at',last_used_at)
        FROM auth_webauthn_credentials WHERE user_id=ANY($1) ORDER BY id"#,
    )
    .bind(subjects)
    .fetch_all(pool)
    .await
    .unwrap();
    let families: Vec<Value> = sqlx::query_scalar(
        r#"SELECT jsonb_build_object(
        'id',id,'user_id',user_id,'created_at',created_at)
        FROM auth_refresh_token_families WHERE user_id=ANY($1) ORDER BY id"#,
    )
    .bind(subjects)
    .fetch_all(pool)
    .await
    .unwrap();
    let tokens: Vec<Value> = sqlx::query_scalar(
        r#"SELECT jsonb_build_object(
        'id',id,'family_id',family_id,'user_id',user_id,'token_hash',encode(token_hash,'hex'),
        'issued_at',issued_at,'expires_at',expires_at,'used_at',used_at,'replaced_by',replaced_by)
        FROM auth_refresh_tokens WHERE user_id=ANY($1) ORDER BY id"#,
    )
    .bind(subjects)
    .fetch_all(pool)
    .await
    .unwrap();
    let ceremonies:Vec<Value>=sqlx::query_scalar(r#"SELECT jsonb_build_object(
        'id',id,'user_id',user_id,'ceremony_kind',ceremony_kind,
        'challenge_json',challenge_json,'state_json',state_json,'expires_at',expires_at,'created_at',created_at)
        FROM auth_webauthn_ceremonies WHERE user_id=ANY($1) ORDER BY id"#)
        .bind(subjects).fetch_all(pool).await.unwrap();
    let audits:Vec<Value>=sqlx::query_scalar(r#"SELECT jsonb_build_object(
        'id',id,'actor',actor,'action',action,'target_type',target_type,'target_id',target_id,
        'branch_id',branch_id,'before_snap',before_snap,'after_snap',after_snap,
        'trace_id',trace_id,'span_id',span_id,'occurred_at',occurred_at,'created_at',created_at,'org_id',org_id)
        FROM audit_events WHERE id=ANY($1) ORDER BY id"#)
        .bind(audit_ids).fetch_all(pool).await.unwrap();
    json!({"keys":keys,"families":families,"tokens":tokens,"ceremonies":ceremonies,"audits":audits})
}

async fn seed_legacy(pool: &PgPool) -> LegacyFixture {
    use sha2::Digest as _;

    // Immutable TEST_ONLY output of the real historical auth owner at schema225.
    // Producer1bb508a2 + capture-only028828ac; never run new auth code on old schema.
    let bytes = include_bytes!("fixtures/account-legacy225-owner-output-v1.json");
    require_legacy225_digest(bytes);
    let saved: Value = serde_json::from_slice(bytes).expect("reviewed historical fixture JSON");
    assert!(saved["kind"] == "TEST_ONLY_HISTORICAL225_OWNER_OUTPUT");
    assert!(saved["format_version"] == 1);
    assert!(saved["producer_base_sha"] == "1bb508a28e43fef188d52f88e11fcb5b0d0c4dbf");
    assert!(saved["capture"]["migration_version"] == 225);
    assert!(saved["credential_count"] == 2);
    let subjects: Vec<Uuid> = serde_json::from_value(saved["subjects"].clone()).unwrap();
    let audit_ids: Vec<Uuid> = serde_json::from_value(saved["audit_ids"].clone()).unwrap();
    assert_eq!(subjects.len(), 2);
    assert_eq!(audit_ids.len(), 7);
    assert_eq!(
        subjects
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        2
    );
    assert_eq!(
        audit_ids
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        7
    );
    let tables = &saved["capture"]["tables"];
    assert_eq!(tables.as_object().unwrap().len(), 10);
    for (name, count) in [
        ("groups", 2),
        ("organizations", 2),
        ("group_memberships", 2),
        ("users", 2),
        ("auth_webauthn_credentials", 2),
        ("auth_webauthn_ceremonies", 4),
        ("auth_webauthn_ceremony_bindings", 0),
        ("auth_refresh_token_families", 2),
        ("auth_refresh_tokens", 4),
        ("audit_events", 7),
    ] {
        assert_eq!(
            tables[name].as_array().unwrap().len(),
            count,
            "historical roster differs"
        );
    }
    let revoked_token = saved["revoked_token"].as_str().unwrap().to_owned();
    let revoked_hash = sha2::Sha256::digest(revoked_token.as_bytes()).to_vec();
    assert!(saved["revoked_token_sha256"] == hex::encode(&revoked_hash));

    let (migrator, migrate_config) = old_migration_login(pool).await;
    sqlx::migrate!("../crates/platform/db/migrations")
        .run_to(225, &migrator)
        .await
        .expect("actual immutable migrations through225 must apply before seeding");
    let version: i64 =
        sqlx::query_scalar("SELECT max(version) FROM _sqlx_migrations WHERE success")
            .fetch_one(&migrator)
            .await
            .unwrap();
    assert_eq!(version, 225);
    let old_checksums: Vec<(i64, Vec<u8>)> = sqlx::query_as(
        "SELECT version,checksum FROM _sqlx_migrations WHERE version<=225 ORDER BY version",
    )
    .fetch_all(&migrator)
    .await
    .unwrap();
    let checksums: Vec<Value> = old_checksums
        .iter()
        .map(|(version, checksum)| json!([version, hex::encode(checksum)]))
        .collect();
    assert!(
        saved["capture"]["migration_checksums"] == json!(checksums),
        "historical migration checksum differs"
    );
    migrator.close().await;

    let mut tx = pool.begin().await.unwrap();
    // Fixed native table types retain bytea, timestamps, arrays and JSON exactly.
    // All four tokens share one INSERT so replaced_by self-FKs resolve normally.
    for (name, statement) in [
        (
            "groups",
            "INSERT INTO public.groups SELECT * FROM jsonb_populate_recordset(NULL::public.groups, $1::jsonb)",
        ),
        (
            "organizations",
            "INSERT INTO public.organizations SELECT * FROM jsonb_populate_recordset(NULL::public.organizations, $1::jsonb)",
        ),
        (
            "users",
            "INSERT INTO public.users SELECT * FROM jsonb_populate_recordset(NULL::public.users, $1::jsonb)",
        ),
        (
            "auth_webauthn_credentials",
            "INSERT INTO public.auth_webauthn_credentials SELECT * FROM jsonb_populate_recordset(NULL::public.auth_webauthn_credentials, $1::jsonb)",
        ),
        (
            "auth_webauthn_ceremonies",
            "INSERT INTO public.auth_webauthn_ceremonies SELECT * FROM jsonb_populate_recordset(NULL::public.auth_webauthn_ceremonies, $1::jsonb)",
        ),
        (
            "auth_webauthn_ceremony_bindings",
            "INSERT INTO public.auth_webauthn_ceremony_bindings SELECT * FROM jsonb_populate_recordset(NULL::public.auth_webauthn_ceremony_bindings, $1::jsonb)",
        ),
        (
            "auth_refresh_token_families",
            "INSERT INTO public.auth_refresh_token_families SELECT * FROM jsonb_populate_recordset(NULL::public.auth_refresh_token_families, $1::jsonb)",
        ),
        (
            "auth_refresh_tokens",
            "INSERT INTO public.auth_refresh_tokens SELECT * FROM jsonb_populate_recordset(NULL::public.auth_refresh_tokens, $1::jsonb)",
        ),
        (
            "audit_events",
            "INSERT INTO public.audit_events SELECT * FROM jsonb_populate_recordset(NULL::public.audit_events, $1::jsonb)",
        ),
    ] {
        let affected = sqlx::query(statement)
            .bind(&tables[name])
            .execute(&mut *tx)
            .await
            .expect("native historical replay must preserve active constraints")
            .rows_affected();
        assert_eq!(
            affected,
            tables[name].as_array().unwrap().len() as u64,
            "historical insert count differs"
        );
    }
    // Migration225 creates these rows on organization INSERT. Restore only the
    // original timestamp, after proving the trigger generated exactly our keys.
    let memberships: Vec<(Uuid, Uuid)> = sqlx::query_as(
        "SELECT group_id,org_id FROM public.group_memberships WHERE org_id IN \
         (SELECT org_id FROM public.users WHERE id=ANY($1)) ORDER BY group_id,org_id",
    )
    .bind(&subjects)
    .fetch_all(&mut *tx)
    .await
    .unwrap();
    let expected_memberships: Vec<(Uuid, Uuid)> = tables["group_memberships"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            (
                serde_json::from_value(row["group_id"].clone()).unwrap(),
                serde_json::from_value(row["org_id"].clone()).unwrap(),
            )
        })
        .collect();
    assert!(
        memberships == expected_memberships,
        "historical membership keys differ"
    );
    let affected = sqlx::query(
        "UPDATE public.group_memberships AS actual SET created_at=captured.created_at \
         FROM jsonb_populate_recordset(NULL::public.group_memberships, $1::jsonb) AS captured \
         WHERE actual.group_id=captured.group_id AND actual.org_id=captured.org_id",
    )
    .bind(&tables["group_memberships"])
    .execute(&mut *tx)
    .await
    .unwrap()
    .rows_affected();
    assert_eq!(affected, 2);

    let restored: Value = sqlx::query_scalar(LEGACY225_ROWS)
        .bind(&subjects)
        .bind(&audit_ids)
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    require_legacy225_rows(&restored, tables, &saved["capture"]["migration_checksums"]);
    let credential_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM auth_webauthn_credentials WHERE user_id=ANY($1)")
            .bind(&subjects)
            .fetch_one(&mut *tx)
            .await
            .unwrap();
    assert_eq!(
        credential_count, 2,
        "two real independently generated historical credentials"
    );
    let correlation: (i64, i64, i64) = sqlx::query_as(r#"SELECT
        (SELECT count(*) FROM auth_refresh_tokens WHERE user_id=ANY($1)),
        (SELECT count(*) FROM auth_refresh_tokens WHERE user_id=ANY($1) AND used_at IS NOT NULL AND replaced_by IS NOT NULL),
        (SELECT count(*) FROM auth_refresh_token_families WHERE user_id=ANY($1) AND revoked_at IS NOT NULL)"#)
        .bind(&subjects).fetch_one(&mut *tx).await.unwrap();
    assert_eq!(correlation, (4, 2, 1));
    let revoked: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM public.auth_refresh_tokens t \
         JOIN public.auth_refresh_token_families f ON f.id=t.family_id \
         WHERE t.token_hash=$1 AND t.user_id=ANY($2::uuid[]) \
         AND t.revoked_at IS NOT NULL AND f.revoked_at IS NOT NULL)",
    )
    .bind(&revoked_hash)
    .bind(&subjects)
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    assert!(
        revoked,
        "historical raw token must correlate to a revoked token and family"
    );
    tx.commit().await.unwrap();
    // Full native rows were checked before commit. Preserve the original
    // independent material projection before invoking any current migration.
    let snapshot = material_snapshot(pool, &subjects, &audit_ids).await;
    assert!(
        snapshot == saved["material_snapshot"],
        "historical material differs; no sensitive dump"
    );
    LegacyFixture {
        subjects,
        audit_ids,
        revoked_token,
        credential_count,
        snapshot,
        old_checksums,
        migrate_config,
    }
}

async fn apply_current_and_require_account_catalog(pool: &PgPool, fixture: &LegacyFixture) {
    run_migrations(&fixture.migrate_config)
        .await
        .expect("actual current production migration entrypoint");
    let checksums: Vec<(i64, Vec<u8>)> = sqlx::query_as(
        "SELECT version,checksum FROM _sqlx_migrations WHERE version<=225 ORDER BY version",
    )
    .fetch_all(pool)
    .await
    .unwrap();
    assert!(
        checksums == fixture.old_checksums,
        "historical migration checksums must remain unchanged"
    );
    let missing: Vec<String> = sqlx::query_scalar(
        r#"SELECT name FROM unnest(ARRAY[
        'accounts','account_security','company_actors']) AS required(name)
        WHERE to_regclass('public.'||name) IS NULL ORDER BY name"#,
    )
    .fetch_all(pool)
    .await
    .unwrap();
    assert!(
        missing.is_empty(),
        "ACCOUNT_CATALOG_PREREQUISITE: legacy fixture seeded with{} subjects/{} credentials/{} audit rows and current migrator completed; preservation assertions not reached; missing={missing:?}",
        fixture.subjects.len(),
        fixture.credential_count,
        fixture.audit_ids.len()
    );
}

#[sqlx::test(migrations = false)]
async fn populated_225_to_current_preserves_credential_bytes_token_correlations_and_original_audits(
    pool: PgPool,
) {
    let fixture = seed_legacy(&pool).await;
    apply_current_and_require_account_catalog(&pool, &fixture).await;
    let after = material_snapshot(&pool, &fixture.subjects, &fixture.audit_ids).await;
    assert!(
        after == fixture.snapshot,
        "original credential/state/token/audit material changed; sensitive bytes intentionally not printed"
    );
    let preserved: i64 = sqlx::query_scalar("SELECT count(*) FROM accounts WHERE id=ANY($1)")
        .bind(&fixture.subjects)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        preserved,
        fixture.subjects.len() as i64,
        "ACCOUNT_MAPPING_PREREQUISITE: real identity mapping/backfill has not preserved every legacy users.id; no fake Account rows inserted by test"
    );
}

#[sqlx::test(migrations = false)]
async fn populated_225_to_current_preserves_preexisting_family_and_token_revocations(pool: PgPool) {
    let fixture = seed_legacy(&pool).await;
    let before:Vec<(Uuid,OffsetDateTime,Option<String>)>=sqlx::query_as(
        "SELECT id,revoked_at,revoked_reason FROM auth_refresh_token_families WHERE user_id=ANY($1) AND revoked_at IS NOT NULL ORDER BY id")
        .bind(&fixture.subjects).fetch_all(&pool).await.unwrap();
    assert_eq!(before.len(), 1);
    let old_tokens:Vec<(Uuid,OffsetDateTime)>=sqlx::query_as(
        "SELECT id,revoked_at FROM auth_refresh_tokens WHERE user_id=ANY($1) AND revoked_at IS NOT NULL ORDER BY id")
        .bind(&fixture.subjects).fetch_all(&pool).await.unwrap();
    assert!(
        !old_tokens.is_empty(),
        "real logout must have revoked its family's tokens"
    );
    apply_current_and_require_account_catalog(&pool, &fixture).await;
    for (id, at, reason) in before {
        let actual: (Option<OffsetDateTime>, Option<String>) = sqlx::query_as(
            "SELECT revoked_at,revoked_reason FROM auth_refresh_token_families WHERE id=$1",
        )
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            actual,
            (Some(at), reason),
            "preexisting revocation cannot be removed or rewritten"
        );
    }
    for (id, at) in old_tokens {
        let actual: Option<OffsetDateTime> =
            sqlx::query_scalar("SELECT revoked_at FROM auth_refresh_tokens WHERE id=$1")
                .bind(id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(actual, Some(at));
    }
    // Existing legacy store must not remint from a preserved revoked family.
    // This is not the missing global Account cutover/fresh-session owner.
    let result = RefreshTokenStore
        .rotate(
            &pool,
            &fixture.revoked_token,
            OffsetDateTime::now_utc(),
            Duration::hours(2),
            Duration::days(1),
        )
        .await;
    assert!(
        matches!(
            result,
            Err(console_platform_auth::RefreshTokenUseError::FamilyRevoked)
        ),
        "revoked-family refusal must be exact; storage/missing-token failures are not a valid denial oracle"
    );
    let after = material_snapshot(&pool, &fixture.subjects, &fixture.audit_ids).await;
    assert!(
        after == fixture.snapshot,
        "refused old refresh changed retained material; no sensitive dump"
    );
}

// Shared guards make the rollback controls exercise the actual replay checks.
fn require_legacy225_digest(bytes: &[u8]) {
    use sha2::Digest as _;

    assert!(
        hex::encode(sha2::Sha256::digest(bytes))
            == "bc0c8a562d54866f892706a0a0b6a6723d8874922f58635f3db6f78685889ce4",
        "historical fixture digest differs"
    );
}

fn require_legacy225_rows(restored: &Value, tables: &Value, checksums: &Value) {
    assert!(
        restored["tables"] == *tables,
        "complete historical rows differ; no sensitive dump"
    );
    assert!(restored["migration_version"] == 225);
    assert!(restored["migration_checksums"] == *checksums);
    assert!(restored["unused_fixture_edges"] == json!({"user_branches":0,"group_role_grants":0}));
}

const LEGACY225_ROWS: &str = r#"WITH fixture_subjects AS (
    SELECT * FROM public.users WHERE id = ANY($1::uuid[])
), fixture_orgs AS (
    SELECT * FROM public.organizations WHERE id IN (SELECT org_id FROM fixture_subjects)
), fixture_ceremonies AS (
    SELECT * FROM public.auth_webauthn_ceremonies WHERE user_id = ANY($1::uuid[])
)
SELECT jsonb_build_object(
    'database', current_database(),
    'server_version_num', current_setting('server_version_num'),
    'migration_version', (SELECT max(version) FROM public._sqlx_migrations WHERE success),
    'migration_checksums', (SELECT jsonb_agg(jsonb_build_array(version, encode(checksum,'hex')) ORDER BY version)
        FROM public._sqlx_migrations WHERE version <= 225),
    'tables', jsonb_build_object(
        'groups', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY t.id),'[]'::jsonb)
            FROM public.groups t WHERE id IN (SELECT group_id FROM fixture_orgs)),
        'organizations', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY t.id),'[]'::jsonb) FROM fixture_orgs t),
        'group_memberships', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY t.group_id,t.org_id),'[]'::jsonb)
            FROM public.group_memberships t WHERE org_id IN (SELECT id FROM fixture_orgs)),
        'users', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY t.id),'[]'::jsonb) FROM fixture_subjects t),
        'auth_webauthn_credentials', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY t.id),'[]'::jsonb)
            FROM public.auth_webauthn_credentials t WHERE user_id = ANY($1::uuid[])),
        'auth_webauthn_ceremonies', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY t.id),'[]'::jsonb) FROM fixture_ceremonies t),
        'auth_webauthn_ceremony_bindings', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY t.ceremony_id),'[]'::jsonb)
            FROM public.auth_webauthn_ceremony_bindings t WHERE ceremony_id IN (SELECT id FROM fixture_ceremonies)),
        'auth_refresh_token_families', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY t.id),'[]'::jsonb)
            FROM public.auth_refresh_token_families t WHERE user_id = ANY($1::uuid[])),
        'auth_refresh_tokens', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY t.id),'[]'::jsonb)
            FROM public.auth_refresh_tokens t WHERE user_id = ANY($1::uuid[])),
        'audit_events', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY t.id),'[]'::jsonb)
            FROM public.audit_events t WHERE id = ANY($2::uuid[]))
    ),
    'unused_fixture_edges', jsonb_build_object(
        'user_branches', (SELECT count(*) FROM public.user_branches WHERE user_id=ANY($1::uuid[])),
        'group_role_grants', (SELECT count(*) FROM public.group_role_grants WHERE user_id=ANY($1::uuid[]))
    )
)"#;

#[sqlx::test(migrations = false)]
async fn historical225_fixture_corrupt_digest_refuses_before_replay_write(pool: PgPool) {
    let fixture = seed_legacy(&pool).await;
    let before: Value = sqlx::query_scalar(LEGACY225_ROWS)
        .bind(&fixture.subjects)
        .bind(&fixture.audit_ids)
        .fetch_one(&pool)
        .await
        .unwrap();
    let mut bytes = include_bytes!("fixtures/account-legacy225-owner-output-v1.json").to_vec();
    // Change only trailing JSON whitespace so malformed JSON cannot explain refusal.
    assert_eq!(bytes.pop(), Some(b'\n'));
    bytes.push(b' ');
    let rejected = std::panic::catch_unwind(|| require_legacy225_digest(&bytes))
        .expect_err("corrupt fixture digest was accepted");
    let message = rejected
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| rejected.downcast_ref::<&str>().copied());
    assert_eq!(message, Some("historical fixture digest differs"));
    let after: Value = sqlx::query_scalar(LEGACY225_ROWS)
        .bind(&fixture.subjects)
        .bind(&fixture.audit_ids)
        .fetch_one(&pool)
        .await
        .unwrap();
    require_legacy225_rows(&after, &before["tables"], &before["migration_checksums"]);
}

async fn require_replay_tamper_rejected_and_rolled_back(pool: PgPool, mutation: &str) {
    let fixture = seed_legacy(&pool).await;
    let before: Value = sqlx::query_scalar(LEGACY225_ROWS)
        .bind(&fixture.subjects)
        .bind(&fixture.audit_ids)
        .fetch_one(&pool)
        .await
        .unwrap();
    let mut tx = pool.begin().await.unwrap();
    let affected = sqlx::query(mutation)
        .bind(&fixture.subjects)
        .execute(&mut *tx)
        .await
        .expect("bounded tamper must reach the row oracle")
        .rows_affected();
    assert_eq!(affected, 1, "tamper must change exactly one historical row");
    let changed: Value = sqlx::query_scalar(LEGACY225_ROWS)
        .bind(&fixture.subjects)
        .bind(&fixture.audit_ids)
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    let rejected = std::panic::catch_unwind(|| {
        require_legacy225_rows(&changed, &before["tables"], &before["migration_checksums"]);
    })
    .expect_err("tampered historical rows were accepted");
    let message = rejected
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| rejected.downcast_ref::<&str>().copied());
    assert_eq!(
        message,
        Some("complete historical rows differ; no sensitive dump")
    );
    tx.rollback().await.unwrap();
    let after: Value = sqlx::query_scalar(LEGACY225_ROWS)
        .bind(&fixture.subjects)
        .bind(&fixture.audit_ids)
        .fetch_one(&pool)
        .await
        .unwrap();
    require_legacy225_rows(&after, &before["tables"], &before["migration_checksums"]);
    assert!(
        material_snapshot(&pool, &fixture.subjects, &fixture.audit_ids).await == fixture.snapshot,
        "rollback must restore original historical material; no sensitive dump"
    );
}

#[sqlx::test(migrations = false)]
async fn historical225_fixture_omitted_row_rejects_and_rolls_back(pool: PgPool) {
    require_replay_tamper_rejected_and_rolled_back(
        pool,
        "DELETE FROM public.auth_webauthn_ceremonies WHERE id=(SELECT id \
         FROM public.auth_webauthn_ceremonies WHERE user_id=ANY($1) ORDER BY id LIMIT 1)",
    )
    .await;
}

#[sqlx::test(migrations = false)]
async fn historical225_fixture_token_correlation_rejects_and_rolls_back(pool: PgPool) {
    require_replay_tamper_rejected_and_rolled_back(
        pool,
        "UPDATE public.auth_refresh_tokens AS original SET replaced_by=(SELECT other.id \
         FROM public.auth_refresh_tokens other WHERE other.user_id=ANY($1) \
         AND other.family_id<>original.family_id ORDER BY other.id LIMIT 1) \
         WHERE original.id=(SELECT id FROM public.auth_refresh_tokens \
         WHERE user_id=ANY($1) AND replaced_by IS NOT NULL ORDER BY id LIMIT 1)",
    )
    .await;
}

#[sqlx::test(migrations = false)]
async fn historical225_fixture_membership_key_rejects_and_rolls_back(pool: PgPool) {
    require_replay_tamper_rejected_and_rolled_back(
        pool,
        "UPDATE public.group_memberships AS actual SET group_id=(SELECT other.group_id \
         FROM public.organizations other WHERE other.id<>actual.org_id \
         AND other.id IN(SELECT org_id FROM public.users WHERE id=ANY($1)) \
         ORDER BY other.id LIMIT 1) WHERE actual.org_id=(SELECT org_id \
         FROM public.users WHERE id=ANY($1) ORDER BY org_id LIMIT 1)",
    )
    .await;
}
