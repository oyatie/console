#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Populated actual numbered-migration compatibility, starting at225.
//! No test-created Account schema, backfill, cutover authority or HTTP handlers.
//! Legacy auth stores produce real key/family/token/ceremony/audit rows. Fixture
//! ownership is privileged setup, not evidence of serving-role authorization.
use console_app::{AppConfig, AppRole, run_migrations};
use console_kernel_core::OrgId;
use console_platform_auth::{
    PasskeyRegistrationStart, PasskeyService, RefreshTokenStore, WebauthnSettings,
};
use console_platform_test_support::seed_org_and_super_admin;
use serde_json::{Value, json};
use sqlx::PgPool;
use time::{Duration, OffsetDateTime};
use url::Url;
use uuid::Uuid;
use webauthn_authenticator_rs::{prelude::WebauthnAuthenticator, softpasskey::SoftPasskey};

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
    let old_checksums = sqlx::query_as(
        "SELECT version,checksum FROM _sqlx_migrations WHERE version<=225 ORDER BY version",
    )
    .fetch_all(&migrator)
    .await
    .unwrap();
    migrator.close().await;
    let service = PasskeyService::new(WebauthnSettings {
        rp_id: "example.com".to_owned(),
        rp_origin: Url::parse("https://auth.example.com").unwrap(),
        rp_name: "Console".to_owned(),
        extra_allowed_origins: vec![],
        ceremony_ttl: Duration::minutes(5),
    })
    .unwrap();
    let orgs = [OrgId::new(), OrgId::new()];
    let mut subjects = Vec::new();
    let mut revoked_token = None;
    for (index, org) in orgs.into_iter().enumerate() {
        let subject =
            seed_org_and_super_admin(pool, *org.as_uuid(), &format!("migration225-{index}")).await;
        subjects.push(*subject.as_uuid());
        let registration = service
            .start_registration(
                pool,
                org,
                PasskeyRegistrationStart {
                    user_id: *subject.as_uuid(),
                    username: format!("migration-{index}"),
                    display_name: format!("TEST_ONLY migration {index}"),
                },
            )
            .await
            .unwrap();
        let mut authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
        let credential = authenticator
            .do_registration(
                Url::parse("https://auth.example.com").unwrap(),
                registration.challenge,
            )
            .unwrap();
        let stored = service
            .finish_registration(pool, org, registration.ceremony_id, credential)
            .await
            .unwrap();
        assert_eq!(stored.user_id, *subject.as_uuid());
        // Retain a second real outstanding ceremony. No fabricated state JSON.
        service
            .start_registration(
                pool,
                org,
                PasskeyRegistrationStart {
                    user_id: *subject.as_uuid(),
                    username: format!("pending-{index}"),
                    display_name: "TEST_ONLY pending".to_owned(),
                },
            )
            .await
            .unwrap();
        let now = OffsetDateTime::now_utc();
        let issue = RefreshTokenStore
            .issue_family(pool, *subject.as_uuid(), org, now, Duration::hours(2))
            .await
            .unwrap();
        let rotated = RefreshTokenStore
            .rotate(
                pool,
                issue.token.as_str(),
                now + Duration::seconds(1),
                Duration::hours(2),
                Duration::days(1),
            )
            .await
            .unwrap();
        assert_eq!(rotated.family_id, issue.family_id);
        assert_ne!(rotated.token_id, issue.token_id);
        if index == 1 {
            RefreshTokenStore
                .revoke_family_for_logout(pool, rotated.token.as_str(), now + Duration::seconds(2))
                .await
                .unwrap();
            revoked_token = Some(rotated.token.as_str().to_owned());
        }
    }
    let audit_ids: Vec<Uuid> =
        sqlx::query_scalar("SELECT id FROM audit_events WHERE actor=ANY($1) ORDER BY id")
            .bind(&subjects)
            .fetch_all(pool)
            .await
            .unwrap();
    assert!(
        !audit_ids.is_empty(),
        "real auth-owner operations must produce attributed audits"
    );
    let credential_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM auth_webauthn_credentials WHERE user_id=ANY($1)")
            .bind(&subjects)
            .fetch_one(pool)
            .await
            .unwrap();
    assert_eq!(
        credential_count, 2,
        "two real independently generated credentials"
    );
    let correlation:(i64,i64,i64)=sqlx::query_as(r#"SELECT
        (SELECT count(*) FROM auth_refresh_tokens WHERE user_id=ANY($1)),
        (SELECT count(*) FROM auth_refresh_tokens WHERE user_id=ANY($1) AND used_at IS NOT NULL AND replaced_by IS NOT NULL),
        (SELECT count(*) FROM auth_refresh_token_families WHERE user_id=ANY($1) AND revoked_at IS NOT NULL)"#)
        .bind(&subjects).fetch_one(pool).await.unwrap();
    assert_eq!(correlation, (4, 2, 1));
    let snapshot = material_snapshot(pool, &subjects, &audit_ids).await;
    LegacyFixture {
        subjects,
        audit_ids,
        revoked_token: revoked_token.unwrap(),
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
