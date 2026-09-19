//! PLATFORM "view as" (read-only impersonation) tests.
//!
//! Proves the security-critical invariants of the troubleshooting impersonation:
//!   (a) a view_as token READS the TARGET tenant's rows — RLS is armed to the
//!       acting org and a role-appropriate read returns that tenant's data;
//!   (b) a view_as token CANNOT mutate — POST/PATCH/PUT/DELETE to ANY tenant route
//!       returns 403 `view_as_read_only`, blocked by the blanket method gate
//!       BEFORE any handler runs;
//!   (c) a NON-platform (tenant) token cannot START — 403;
//!   (d) cross-tenant isolation — a view_as token pinned to org A cannot read org
//!       B's rows (RLS makes them invisible);
//!   (e) START and EXIT write the audit events with the REAL operator id;
//!   (f) the minted token's TTL is short (≤30 min).
//!
//! Everything DB-backed runs against a pool whose connections `SET ROLE console_rt`,
//! so RLS is exercised as the production runtime role (NOBYPASSRLS, FORCE RLS) —
//! a superuser pool would mask a broken read path (rls-verify-as-runtime-role).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use axum::body::{Body, to_bytes};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use console_kernel_core::{OrgId, UserId};
use console_platform_auth::{AccessTokenInput, JwtIssuer, JwtSettings, JwtVerifier};
use console_platform_db::with_org_conn;
use console_platform_provisioning::PlatformProvisioner;
use console_platform_request_context::{current_org, with_request_context};
use console_platform_rest::{
    PLATFORM_TENANT_CONTEXT_EXIT_PATH, PLATFORM_TENANT_CONTEXT_START_PATH,
    PLATFORM_VIEW_AS_EXIT_PATH, PLATFORM_VIEW_AS_START_PATH, PlatformRestState,
    VIEW_AS_READ_ONLY_CODE, router, with_view_as_read_only_gate,
};
use http::{Request, StatusCode, header};
use p256::ecdsa::SigningKey;
use p256::elliptic_curve::rand_core::OsRng;
use p256::pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding};
use serde_json::Value;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use time::{Duration, OffsetDateTime};
use tower::ServiceExt;
use uuid::Uuid;

const TEST_ISSUER: &str = "console-platform-auth";
const TEST_AUDIENCE: &str = "console-api";

/// A tenant probe route the read-only gate is exercised against. The GET handler
/// counts the users RLS lets the request see (so a view_as token armed to the
/// acting org reads exactly that tenant), and the mutating handlers exist only to
/// be PROVEN unreachable under a view_as token.
const PROBE_USERS_PATH: &str = "/api/v1/view-as-probe/users";

struct Harness {
    private_pem: String,
    public_pem: String,
    /// The runtime-role pool every router uses (each connection is `console_rt`).
    rt_pool: PgPool,
}

impl Harness {
    async fn new(owner_pool: &PgPool) -> Self {
        let signing_key = SigningKey::random(&mut OsRng);
        let private_pem = signing_key
            .to_pkcs8_pem(LineEnding::LF)
            .unwrap()
            .to_string();
        let public_pem = signing_key
            .verifying_key()
            .to_public_key_pem(LineEnding::LF)
            .unwrap();
        Self {
            private_pem,
            public_pem,
            rt_pool: runtime_role_pool(owner_pool).await,
        }
    }

    fn jwt_settings(&self) -> JwtSettings {
        JwtSettings {
            issuer: TEST_ISSUER.to_owned(),
            audience: TEST_AUDIENCE.to_owned(),
            access_token_ttl: Duration::minutes(15),
        }
    }

    fn verifier(&self) -> JwtVerifier {
        JwtVerifier::from_es256_public_pem(self.jwt_settings(), self.public_pem.as_bytes()).unwrap()
    }

    fn issuer(&self) -> JwtIssuer {
        JwtIssuer::from_es256_pem(
            self.jwt_settings(),
            self.private_pem.as_bytes(),
            self.public_pem.as_bytes(),
        )
        .unwrap()
    }

    /// The PLATFORM router (START + EXIT + orgs), with the view-as issuer wired so
    /// START can mint impersonation tokens.
    async fn platform_service(&self) -> Router {
        let auth_database = console_platform_test_support::login_test_pool(
            &self.rt_pool,
            console_platform_test_support::TestDatabaseLogin::Auth,
        )
        .await;
        router(
            PlatformRestState::new(
                self.rt_pool.clone(),
                Some(console_platform_auth::SessionVerification::new(
                    self.verifier(),
                    auth_database.clone(),
                )),
                PlatformProvisioner::new(Duration::minutes(15)),
            )
            .with_view_as_issuer(Some(self.issuer())),
        )
    }

    /// A TENANT router that mirrors production: the per-request tenant org
    /// middleware arms `app.current_org`, and the blanket view-as read-only gate
    /// wraps the whole thing (exactly as the app composition root applies it).
    async fn tenant_service(&self) -> Router {
        let auth_database = console_platform_test_support::login_test_pool(
            &self.rt_pool,
            console_platform_test_support::TestDatabaseLogin::Auth,
        )
        .await;
        let inner = Router::new()
            .route(
                PROBE_USERS_PATH,
                get(probe_count_users)
                    .post(probe_mutation)
                    .patch(probe_mutation)
                    .put(probe_mutation)
                    .delete(probe_mutation),
            )
            .with_state(self.rt_pool.clone());
        let inner = with_request_context(
            inner,
            Some(console_platform_auth::SessionVerification::new(
                self.verifier(),
                auth_database.clone(),
            )),
            self.rt_pool.clone(),
        );
        with_view_as_read_only_gate(inner, Some(self.verifier()))
    }

    /// Mint an ordinary (non-view_as) token for `user`/`org`.
    fn token(&self, user_id: UserId, org_id: OrgId, platform: bool, role: &str) -> String {
        self.issuer()
            .issue_access_token(AccessTokenInput {
                subject: user_id,
                org_id,
                roles: vec![role.to_owned()],
                branches: vec![],
                platform,
                view_as: false,
                read_only: false,
                display_name: None,
                feature_grants: Vec::new(),
                authz_subject_version: 0,
                authz_policy_version: 0,
                session_generation: 0,
                issued_at: OffsetDateTime::now_utc(),
            })
            .unwrap()
    }
}

/// A pool whose every connection runs `SET ROLE console_rt` (NOBYPASSRLS, FORCE RLS).
async fn runtime_role_pool(owner_pool: &PgPool) -> PgPool {
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
        .unwrap()
}

// ---------------------------------------------------------------------------
// Probe handlers (tenant tier)
// ---------------------------------------------------------------------------

/// GET probe: count the users RLS lets THIS request see. A view_as token armed to
/// the acting org sees exactly that tenant's users; a different org's users are
/// invisible. Proves RLS scoping + cross-tenant isolation through the real
/// tenant middleware.
async fn probe_count_users(State(pool): State<PgPool>) -> impl IntoResponse {
    let org = current_org().expect("tenant middleware must have armed the org");
    let count = with_org_conn::<_, i64, console_platform_db::DbError>(&pool, org, |tx| {
        Box::pin(async move {
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
                .fetch_one(tx.as_mut())
                .await
                .map_err(console_platform_db::DbError::Sqlx)
        })
    })
    .await
    .unwrap_or(-1);
    Json(serde_json::json!({ "count": count, "org": org.as_uuid().to_string() })).into_response()
}

/// A mutation handler that MUST be unreachable under a view_as token. If the
/// read-only gate ever failed open, this would return 200 and the test would
/// catch it.
async fn probe_mutation() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({ "mutated": true }))).into_response()
}

// ---------------------------------------------------------------------------
// HTTP helpers
// ---------------------------------------------------------------------------

async fn request(
    service: &Router,
    method: &str,
    path: &str,
    token: &str,
    body: Option<String>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method(method)
        .uri(path)
        .header(header::AUTHORIZATION, format!("Bearer {token}"));
    if body.is_some() {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
    }
    let request = builder
        .body(body.map_or(Body::empty(), Body::from))
        .unwrap();
    let response = service.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes)
            .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into_owned()))
    };
    (status, json)
}

/// Call START on the platform service and return (status, body).
async fn start_view_as(
    platform: &Router,
    token: &str,
    org_id: Uuid,
    role: &str,
) -> (StatusCode, Value) {
    let body = format!(r#"{{"org_id":"{org_id}","role":"{role}"}}"#);
    request(
        platform,
        "POST",
        PLATFORM_VIEW_AS_START_PATH,
        token,
        Some(body),
    )
    .await
}

/// Call writable TENANT CONTEXT START on the platform service.
async fn start_tenant_context(platform: &Router, token: &str, org_id: Uuid) -> (StatusCode, Value) {
    let body = format!(r#"{{"org_id":"{org_id}"}}"#);
    request(
        platform,
        "POST",
        PLATFORM_TENANT_CONTEXT_START_PATH,
        token,
        Some(body),
    )
    .await
}

// ---------------------------------------------------------------------------
// Seeding (owner pool, RLS off)
// ---------------------------------------------------------------------------

/// Seed the platform sentinel org + a platform-admin user (the operator). The
/// audit actor FK references this user.
async fn seed_platform_admin(owner_pool: &PgPool) -> UserId {
    sqlx::query(
        "INSERT INTO organizations (id, slug, name, status) VALUES ($1, 'platform', 'Platform', 'ARCHIVED') ON CONFLICT (id) DO NOTHING",
    )
    .bind(*OrgId::platform().as_uuid())
    .execute(owner_pool)
    .await
    .unwrap();
    let id = UserId::new();
    sqlx::query("INSERT INTO users (id, display_name, roles, org_id) VALUES ($1, $2, $3, $4)")
        .bind(*id.as_uuid())
        .bind("Platform Admin")
        .bind(vec!["SUPER_ADMIN".to_owned()])
        .bind(*OrgId::platform().as_uuid())
        .execute(owner_pool)
        .await
        .unwrap();
    id
}

/// Seed an ACTIVE tenant org with `user_count` users, returning its id. Owner
/// pool with RLS off so the WITH CHECK + org constraints accept the inserts.
async fn seed_tenant(owner_pool: &PgPool, slug: &str, user_count: usize) -> Uuid {
    let mut tx = owner_pool.begin().await.unwrap();
    sqlx::query("SET LOCAL row_security = off")
        .execute(&mut *tx)
        .await
        .unwrap();
    let org_id: Uuid = sqlx::query_scalar(
        "INSERT INTO organizations (slug, name, status) VALUES ($1, $2, 'ACTIVE') RETURNING id",
    )
    .bind(slug)
    .bind(format!("Tenant {slug}"))
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    for i in 0..user_count {
        sqlx::query("INSERT INTO users (display_name, roles, org_id) VALUES ($1, $2, $3)")
            .bind(format!("User {i}"))
            .bind(vec!["MECHANIC".to_owned()])
            .bind(org_id)
            .execute(&mut *tx)
            .await
            .unwrap();
    }
    tx.commit().await.unwrap();
    org_id
}

/// Set a tenant's status (e.g. to SUSPENDED) via the owner pool, RLS off.
async fn set_org_status(owner_pool: &PgPool, org_id: Uuid, status: &str) {
    let mut tx = owner_pool.begin().await.unwrap();
    sqlx::query("SET LOCAL row_security = off")
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::query("UPDATE organizations SET status = $2 WHERE id = $1")
        .bind(org_id)
        .bind(status)
        .execute(&mut *tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();
}

/// Give a tenant a non-zero per-org policy revision (owner pool, RLS off), so a
/// minted token's sourced `authz_policy_version` is provably non-zero rather than
/// the absent-row baseline. Mirrors the `policy_versions` shape (0065).
async fn set_policy_version(owner_pool: &PgPool, org_id: Uuid, version: i64) {
    let mut tx = owner_pool.begin().await.unwrap();
    sqlx::query("SET LOCAL row_security = off")
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::query(
        r#"
        INSERT INTO policy_versions (org_id, version, updated_at)
        VALUES ($1, $2, now())
        ON CONFLICT (org_id) DO UPDATE SET version = EXCLUDED.version, updated_at = now()
        "#,
    )
    .bind(org_id)
    .bind(version)
    .execute(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();
}

/// Count audit rows for one action AND one actor (owner pool, RLS off).
async fn audit_count(owner_pool: &PgPool, action: &str, actor: UserId) -> i64 {
    let mut tx = owner_pool.begin().await.unwrap();
    sqlx::query("SET LOCAL row_security = off")
        .execute(&mut *tx)
        .await
        .unwrap();
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM audit_events WHERE action = $1 AND actor = $2")
            .bind(action)
            .bind(*actor.as_uuid())
            .fetch_one(&mut *tx)
            .await
            .unwrap();
    tx.commit().await.unwrap();
    count
}

// ===========================================================================
// (c) A non-platform token cannot START — 403.
// ===========================================================================
#[sqlx::test(migrations = false)]
async fn tenant_token_cannot_start_view_as(owner_pool: PgPool) {
    console_platform_test_support::prepare_account_test_database(&owner_pool).await;
    let harness = Harness::new(&owner_pool).await;
    let _ = seed_platform_admin(&owner_pool).await;
    let target = seed_tenant(&owner_pool, "acme", 3).await;
    let platform = harness.platform_service().await;

    // A TENANT token (platform = false) must be rejected by the platform extractor.
    let tenant_user = UserId::new();
    let tenant_token = harness.token(tenant_user, OrgId::from_uuid(target), false, "SUPER_ADMIN");
    let (status, _body) = start_view_as(&platform, &tenant_token, target, "ADMIN").await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "a tenant token must not be able to START view-as",
    );
}

// ===========================================================================
// (a)+(e)+(f) A platform operator STARTs; the token is a short-lived read-only
// view_as token pinned to the target org/role; START is audited with the real
// operator id.
// ===========================================================================
#[sqlx::test(migrations = false)]
async fn start_mints_short_lived_read_only_token_and_audits(owner_pool: PgPool) {
    console_platform_test_support::prepare_account_test_database(&owner_pool).await;
    let harness = Harness::new(&owner_pool).await;
    let operator = seed_platform_admin(&owner_pool).await;
    let target = seed_tenant(&owner_pool, "acme", 4).await;
    let platform = harness.platform_service().await;
    let platform_token = harness.token(operator, OrgId::platform(), true, "SUPER_ADMIN");

    let (status, body) = start_view_as(&platform, &platform_token, target, "ADMIN").await;
    assert_eq!(status, StatusCode::OK, "{body:?}");

    let access_token = body["access_token"].as_str().unwrap();
    assert_eq!(body["acting_org_id"].as_str().unwrap(), target.to_string());
    assert_eq!(body["acting_role"].as_str().unwrap(), "ADMIN");

    // The token is a TENANT token (platform false) pinned to the target org/role
    // with the read-only flags, and its TTL is short (≤30 min). Verify it with the
    // real verifier (which also proves the START path signed a valid token).
    let claims = harness
        .verifier()
        .verify_access_token(access_token)
        .unwrap();
    assert!(
        !claims.platform,
        "view_as token must NOT be a platform token"
    );
    assert!(claims.view_as, "view_as flag must be set");
    assert!(claims.read_only, "read_only flag must be set");
    assert_eq!(claims.org, target.to_string());
    assert_eq!(claims.roles, vec!["ADMIN".to_owned()]);
    // sub is the REAL operator id, never spoofed from the body.
    assert_eq!(claims.sub, operator.as_uuid().to_string());
    assert!(
        claims.exp - claims.iat <= 30 * 60 && claims.exp - claims.iat > 0,
        "view_as token TTL must be a short positive window (≤30m), got {}s",
        claims.exp - claims.iat,
    );

    // START is audited with the real operator id and org_id = NULL (platform tier).
    assert_eq!(
        audit_count(&owner_pool, "platform.view_as.start", operator).await,
        1,
        "START must write exactly one audit row with the operator actor",
    );
}

// ===========================================================================
// (a) The view_as token READS the target tenant's rows (RLS armed to acting org).
// ===========================================================================
#[sqlx::test(migrations = false)]
async fn view_as_token_reads_target_tenant_rows(owner_pool: PgPool) {
    console_platform_test_support::prepare_account_test_database(&owner_pool).await;
    let harness = Harness::new(&owner_pool).await;
    let operator = seed_platform_admin(&owner_pool).await;
    let target = seed_tenant(&owner_pool, "acme", 5).await;
    let _other = seed_tenant(&owner_pool, "globex", 9).await;
    let platform = harness.platform_service().await;
    let tenant = harness.tenant_service().await;
    let platform_token = harness.token(operator, OrgId::platform(), true, "SUPER_ADMIN");

    let (_s, body) = start_view_as(&platform, &platform_token, target, "ADMIN").await;
    let view_as_token = body["access_token"].as_str().unwrap();

    // A GET with the view_as token reads EXACTLY the target tenant's users (5),
    // not the other tenant's (9) — RLS is armed to the acting org.
    let (status, read) = request(&tenant, "GET", PROBE_USERS_PATH, view_as_token, None).await;
    assert_eq!(status, StatusCode::OK, "{read:?}");
    assert_eq!(
        read["count"].as_i64().unwrap(),
        5,
        "must see the target tenant's 5 users"
    );
    assert_eq!(read["org"].as_str().unwrap(), target.to_string());
}

// ===========================================================================
// (b) The view_as token CANNOT mutate: every non-GET/HEAD method → 403
// view_as_read_only, BEFORE any handler runs.
// ===========================================================================
#[sqlx::test(migrations = false)]
async fn view_as_token_cannot_mutate_any_method(owner_pool: PgPool) {
    console_platform_test_support::prepare_account_test_database(&owner_pool).await;
    let harness = Harness::new(&owner_pool).await;
    let operator = seed_platform_admin(&owner_pool).await;
    let target = seed_tenant(&owner_pool, "acme", 2).await;
    let platform = harness.platform_service().await;
    let tenant = harness.tenant_service().await;
    let platform_token = harness.token(operator, OrgId::platform(), true, "SUPER_ADMIN");

    let (_s, body) = start_view_as(&platform, &platform_token, target, "SUPER_ADMIN").await;
    let view_as_token = body["access_token"].as_str().unwrap();

    // Every mutating method is blocked by the blanket gate with 403 + the code.
    for method in ["POST", "PATCH", "PUT", "DELETE"] {
        let payload = (method != "DELETE").then(|| "{}".to_owned());
        let (status, resp) =
            request(&tenant, method, PROBE_USERS_PATH, view_as_token, payload).await;
        assert_eq!(
            status,
            StatusCode::FORBIDDEN,
            "{method} under a view_as token must be 403, got {status}: {resp:?}",
        );
        assert_eq!(
            resp["error"]["code"].as_str().unwrap(),
            VIEW_AS_READ_ONLY_CODE,
            "{method} must be rejected with the read-only code",
        );
        assert_ne!(
            resp["mutated"],
            Value::Bool(true),
            "the mutation handler must NEVER run under a view_as token ({method})",
        );
    }

    // A GET with the SAME token still works — the gate blocks only unsafe methods.
    let (status, _read) = request(&tenant, "GET", PROBE_USERS_PATH, view_as_token, None).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "GET must still be allowed under view_as"
    );
}

// ===========================================================================
// An ORDINARY tenant token (NOT view_as) can still mutate the probe route — the
// gate must not block normal traffic.
// ===========================================================================
#[sqlx::test(migrations = false)]
async fn ordinary_tenant_token_can_still_mutate(owner_pool: PgPool) {
    console_platform_test_support::prepare_account_test_database(&owner_pool).await;
    let harness = Harness::new(&owner_pool).await;
    let _operator = seed_platform_admin(&owner_pool).await;
    let target = seed_tenant(&owner_pool, "acme", 1).await;
    let tenant = harness.tenant_service().await;

    // An ordinary SUPER_ADMIN tenant token (view_as = false) passes the gate.
    let user = UserId::new();
    sqlx::query("INSERT INTO users (id, display_name, roles, org_id) VALUES ($1, $2, $3, $4)")
        .bind(user.as_uuid())
        .bind("Ordinary tenant mutation control")
        .bind(vec!["SUPER_ADMIN".to_owned()])
        .bind(target)
        .execute(&owner_pool)
        .await
        .unwrap();
    let persisted: (Uuid, bool, Vec<String>) =
        sqlx::query_as("SELECT org_id, is_active, roles FROM users WHERE id=$1")
            .bind(user.as_uuid())
            .fetch_one(&owner_pool)
            .await
            .unwrap();
    assert_eq!(persisted, (target, true, vec!["SUPER_ADMIN".to_owned()]));
    let normal = harness.token(user, OrgId::from_uuid(target), false, "SUPER_ADMIN");
    let (status, resp) = request(
        &tenant,
        "POST",
        PROBE_USERS_PATH,
        &normal,
        Some("{}".to_owned()),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "an ordinary tenant token must NOT be blocked by the view-as gate: {resp:?}",
    );
    assert_eq!(resp["mutated"], Value::Bool(true));
}

// ===========================================================================
// (d) Cross-tenant isolation: a view_as token pinned to org A cannot read org B.
// The token's org claim arms RLS to A; B's rows are invisible.
// ===========================================================================
#[sqlx::test(migrations = false)]
async fn view_as_token_cannot_read_a_different_org(owner_pool: PgPool) {
    console_platform_test_support::prepare_account_test_database(&owner_pool).await;
    let harness = Harness::new(&owner_pool).await;
    let operator = seed_platform_admin(&owner_pool).await;
    let org_a = seed_tenant(&owner_pool, "acme", 3).await;
    let org_b = seed_tenant(&owner_pool, "globex", 11).await;
    let platform = harness.platform_service().await;
    let tenant = harness.tenant_service().await;
    let platform_token = harness.token(operator, OrgId::platform(), true, "SUPER_ADMIN");

    // Start a view_as session pinned to org A.
    let (_s, body) = start_view_as(&platform, &platform_token, org_a, "ADMIN").await;
    let view_as_token = body["access_token"].as_str().unwrap();

    // The probe reads under the token's armed org (A) only: it sees A's 3 users,
    // and there is NO path for this token to read B's 11 — the org is baked into
    // the verified token, not request-controlled.
    let (status, read) = request(&tenant, "GET", PROBE_USERS_PATH, view_as_token, None).await;
    assert_eq!(status, StatusCode::OK, "{read:?}");
    assert_eq!(
        read["count"].as_i64().unwrap(),
        3,
        "sees ONLY org A's users"
    );
    assert_eq!(read["org"].as_str().unwrap(), org_a.to_string());
    assert_ne!(
        read["org"].as_str().unwrap(),
        org_b.to_string(),
        "the token can never be armed to org B",
    );
}

// ===========================================================================
// START refuses a non-ACTIVE tenant (409) — impersonation is scoped to live
// tenants.
// ===========================================================================
#[sqlx::test(migrations = false)]
async fn start_refuses_suspended_tenant(owner_pool: PgPool) {
    console_platform_test_support::prepare_account_test_database(&owner_pool).await;
    let harness = Harness::new(&owner_pool).await;
    let operator = seed_platform_admin(&owner_pool).await;
    let target = seed_tenant(&owner_pool, "acme", 2).await;
    set_org_status(&owner_pool, target, "SUSPENDED").await;
    let platform = harness.platform_service().await;
    let platform_token = harness.token(operator, OrgId::platform(), true, "SUPER_ADMIN");

    let (status, _body) = start_view_as(&platform, &platform_token, target, "ADMIN").await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "view-as must refuse a suspended tenant with 409",
    );
}

// ===========================================================================
// START rejects an unknown role code (422).
// ===========================================================================
#[sqlx::test(migrations = false)]
async fn start_rejects_unknown_role(owner_pool: PgPool) {
    console_platform_test_support::prepare_account_test_database(&owner_pool).await;
    let harness = Harness::new(&owner_pool).await;
    let operator = seed_platform_admin(&owner_pool).await;
    let target = seed_tenant(&owner_pool, "acme", 1).await;
    let platform = harness.platform_service().await;
    let platform_token = harness.token(operator, OrgId::platform(), true, "SUPER_ADMIN");

    let (status, _body) = start_view_as(&platform, &platform_token, target, "WIZARD").await;
    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "an unknown role code must be rejected 422",
    );
}

// ===========================================================================
// (e) EXIT is platform-gated and audits `platform.view_as.stop` with the real
// operator id; a tenant token cannot EXIT.
// ===========================================================================
#[sqlx::test(migrations = false)]
async fn exit_audits_stop_with_operator_and_rejects_tenant_token(owner_pool: PgPool) {
    console_platform_test_support::prepare_account_test_database(&owner_pool).await;
    let harness = Harness::new(&owner_pool).await;
    let operator = seed_platform_admin(&owner_pool).await;
    let target = seed_tenant(&owner_pool, "acme", 1).await;
    let platform = harness.platform_service().await;
    let platform_token = harness.token(operator, OrgId::platform(), true, "SUPER_ADMIN");

    // EXIT with the platform token succeeds and audits the stop.
    let (status, _body) = request(
        &platform,
        "POST",
        PLATFORM_VIEW_AS_EXIT_PATH,
        &platform_token,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        audit_count(&owner_pool, "platform.view_as.stop", operator).await,
        1,
        "EXIT must write exactly one stop audit row with the operator actor",
    );

    // A TENANT token cannot EXIT (platform extractor → 403).
    let tenant_user = UserId::new();
    let tenant_token = harness.token(tenant_user, OrgId::from_uuid(target), false, "SUPER_ADMIN");
    let (status, _body) = request(
        &platform,
        "POST",
        PLATFORM_VIEW_AS_EXIT_PATH,
        &tenant_token,
        None,
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "a tenant token cannot EXIT view-as"
    );
}

// ===========================================================================
// A platform operator can enter a short-lived WRITABLE tenant context pinned to
// exactly one org. The token is not view_as/read_only, so ordinary tenant
// mutations remain reachable while RLS is armed to the selected tenant.
// ===========================================================================
#[sqlx::test(migrations = false)]
async fn tenant_context_mints_writable_super_admin_token_and_audits(owner_pool: PgPool) {
    console_platform_test_support::prepare_account_test_database(&owner_pool).await;
    let harness = Harness::new(&owner_pool).await;
    let operator = seed_platform_admin(&owner_pool).await;
    let target = seed_tenant(&owner_pool, "acme", 2).await;
    let platform = harness.platform_service().await;
    let tenant = harness.tenant_service().await;
    let platform_token = harness.token(operator, OrgId::platform(), true, "SUPER_ADMIN");

    let (status, body) = start_tenant_context(&platform, &platform_token, target).await;
    assert_eq!(status, StatusCode::OK, "{body:?}");

    let access_token = body["access_token"].as_str().unwrap();
    assert_eq!(body["acting_org_id"].as_str().unwrap(), target.to_string());
    assert_eq!(body["acting_role"].as_str().unwrap(), "SUPER_ADMIN");

    let claims = harness
        .verifier()
        .verify_access_token(access_token)
        .unwrap();
    assert!(
        !claims.platform,
        "tenant management context must be a tenant token"
    );
    assert!(
        !claims.view_as,
        "tenant management context is not read-only view-as"
    );
    assert!(
        !claims.read_only,
        "tenant management context must allow mutations"
    );
    assert_eq!(claims.org, target.to_string());
    assert_eq!(claims.roles, vec!["SUPER_ADMIN".to_owned()]);
    assert_eq!(claims.sub, operator.as_uuid().to_string());
    assert!(
        claims.exp - claims.iat <= 30 * 60 && claims.exp - claims.iat > 0,
        "tenant context token TTL must be a short positive window (≤30m), got {}s",
        claims.exp - claims.iat,
    );

    // Because view_as=false, the blanket read-only gate must not block ordinary
    // tenant mutations. The per-request tenant middleware still arms RLS to the
    // token's selected org.
    let (status, mutation) = request(
        &tenant,
        "POST",
        PROBE_USERS_PATH,
        access_token,
        Some("{}".to_owned()),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{mutation:?}");
    assert_eq!(mutation["mutated"], Value::Bool(true));

    assert_eq!(
        audit_count(&owner_pool, "platform.tenant_context.start", operator).await,
        1,
        "tenant context START must write exactly one audit row with the operator actor",
    );
}

// ===========================================================================
// Tenant context EXIT is platform-gated and audited; tenant tokens cannot call
// the platform EXIT endpoint.
// ===========================================================================
#[sqlx::test(migrations = false)]
async fn tenant_context_exit_audits_and_rejects_tenant_token(owner_pool: PgPool) {
    console_platform_test_support::prepare_account_test_database(&owner_pool).await;
    let harness = Harness::new(&owner_pool).await;
    let operator = seed_platform_admin(&owner_pool).await;
    let target = seed_tenant(&owner_pool, "acme", 1).await;
    let platform = harness.platform_service().await;
    let platform_token = harness.token(operator, OrgId::platform(), true, "SUPER_ADMIN");

    let (status, _body) = request(
        &platform,
        "POST",
        PLATFORM_TENANT_CONTEXT_EXIT_PATH,
        &platform_token,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        audit_count(&owner_pool, "platform.tenant_context.stop", operator).await,
        1,
        "tenant context EXIT must write exactly one stop audit row",
    );

    let tenant_user = UserId::new();
    let tenant_token = harness.token(tenant_user, OrgId::from_uuid(target), false, "SUPER_ADMIN");
    let (status, _body) = request(
        &platform,
        "POST",
        PLATFORM_TENANT_CONTEXT_EXIT_PATH,
        &tenant_token,
        None,
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "a tenant token cannot EXIT tenant context"
    );
}

// ===========================================================================
// Cedar/PBAC freshness hardening: BOTH platform mints (read-only view-as and
// writable tenant-context) now stamp REAL subject freshness for the token's own
// (target org, operator) instead of a hardcoded 0, sourced under console_rt RLS
// through the real HTTP handlers. The target org's policy_version is real; the
// operator has no `users` row in the target tenant, so subject/session are the
// absent-row 0 baseline (the true DB-current value the guard also reads).
// ===========================================================================
#[sqlx::test(migrations = false)]
async fn start_mints_carry_real_policy_freshness(owner_pool: PgPool) {
    console_platform_test_support::prepare_account_test_database(&owner_pool).await;
    let harness = Harness::new(&owner_pool).await;
    let operator = seed_platform_admin(&owner_pool).await;
    let target = seed_tenant(&owner_pool, "acme", 1).await;
    // Target tenant has a real custom-policy revision (version 7).
    set_policy_version(&owner_pool, target, 7).await;
    // A second target with NO policy_versions row proves the absent baseline.
    let fresh_target = seed_tenant(&owner_pool, "globex", 1).await;
    let platform = harness.platform_service().await;
    let platform_token = harness.token(operator, OrgId::platform(), true, "SUPER_ADMIN");

    // (1) read-only view-as START carries the real policy_version.
    let (status, body) = start_view_as(&platform, &platform_token, target, "ADMIN").await;
    assert_eq!(status, StatusCode::OK, "{body:?}");
    let claims = harness
        .verifier()
        .verify_access_token(body["access_token"].as_str().unwrap())
        .unwrap();
    assert_eq!(
        claims.authz_policy_version, 7,
        "view-as token must carry the target org's real policy_version, not 0"
    );
    assert_eq!(
        claims.authz_subject_version, 0,
        "operator has no subject row in the target tenant → absent 0 baseline"
    );
    assert_eq!(claims.session_generation, 0);

    // (2) writable tenant-context START carries the same real policy_version.
    let (status, body) = start_tenant_context(&platform, &platform_token, target).await;
    assert_eq!(status, StatusCode::OK, "{body:?}");
    let claims = harness
        .verifier()
        .verify_access_token(body["access_token"].as_str().unwrap())
        .unwrap();
    assert_eq!(
        claims.authz_policy_version, 7,
        "tenant-context token must carry the target org's real policy_version, not 0"
    );
    assert_eq!(claims.authz_subject_version, 0);
    assert_eq!(claims.session_generation, 0);

    // (3) a target with no policy revision still yields the safe 0 baseline — the
    // fix sources the TRUE DB-current value, it does not invent a non-zero.
    let (status, body) = start_tenant_context(&platform, &platform_token, fresh_target).await;
    assert_eq!(status, StatusCode::OK, "{body:?}");
    let claims = harness
        .verifier()
        .verify_access_token(body["access_token"].as_str().unwrap())
        .unwrap();
    assert_eq!(
        claims.authz_policy_version, 0,
        "an org with no policy_versions row reads the absent 0 baseline"
    );
    assert_eq!(claims.authz_subject_version, 0);
    assert_eq!(claims.session_generation, 0);
}

// Additive current-authority regressions. Uses this file's existing key/issuer,
// organization and platform context owners. No new production interface.
mod live_authority_regressions {
    use super::*;
    use console_kernel_core::{
        AccessScope, AccessScopeLevel, BranchId, BranchScope, ErrorKind, KernelError, ScopeNodeId,
    };
    use console_platform_auth::SessionVerification;
    use console_platform_authz::Role;
    use console_platform_request_context::{
        RequestContextError, resolve_platform_principal, resolve_principal,
        resolve_principal_from_bearer_token,
    };
    use console_platform_test_support::{TestDatabaseLogin, login_test_pool};
    use std::collections::BTreeSet;

    async fn bindings(owner: &PgPool, harness: &Harness) -> (SessionVerification, PgPool) {
        let auth = login_test_pool(owner, TestDatabaseLogin::Auth).await;
        let business = login_test_pool(owner, TestDatabaseLogin::Business).await;
        for (pool, expected) in [(&auth, "console_auth_rt"), (&business, "console_rt")] {
            let identity: (String, String) =
                sqlx::query_as("SELECT session_user::text,current_user::text")
                    .fetch_one(pool)
                    .await
                    .unwrap();
            assert_eq!(identity, (expected.into(), expected.into()));
        }
        (SessionVerification::new(harness.verifier(), auth), business)
    }

    async fn tenant_subject(owner: &PgPool) -> (OrgId, UserId) {
        let org = seed_tenant(owner, "live-authority", 1).await;
        let subject: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE org_id=$1")
            .bind(org)
            .fetch_one(owner)
            .await
            .unwrap();
        (OrgId::from_uuid(org), UserId::from_uuid(subject))
    }

    async fn set_roles(owner: &PgPool, subject: UserId, roles: &[&str]) {
        let result = sqlx::query("UPDATE users SET roles=$2 WHERE id=$1")
            .bind(subject.as_uuid())
            .bind(roles)
            .execute(owner)
            .await
            .unwrap();
        assert_eq!(result.rows_affected(), 1);
        let persisted: Vec<String> = sqlx::query_scalar("SELECT roles FROM users WHERE id=$1")
            .bind(subject.as_uuid())
            .fetch_one(owner)
            .await
            .unwrap();
        assert_eq!(persisted, roles);
    }

    async fn set_active(owner: &PgPool, subject: UserId, active: bool) {
        let changed = sqlx::query("UPDATE users SET is_active=$2 WHERE id=$1")
            .bind(subject.as_uuid())
            .bind(active)
            .execute(owner)
            .await
            .unwrap();
        assert_eq!(changed.rows_affected(), 1);
        let persisted: bool = sqlx::query_scalar("SELECT is_active FROM users WHERE id=$1")
            .bind(subject.as_uuid())
            .fetch_one(owner)
            .await
            .unwrap();
        assert_eq!(persisted, active);
    }

    fn headers(token: &str) -> http::HeaderMap {
        let mut headers = http::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            format!("Bearer {token}").parse().unwrap(),
        );
        headers
    }

    fn input(subject: UserId, org: OrgId, roles: &[&str]) -> AccessTokenInput {
        AccessTokenInput {
            subject,
            org_id: org,
            roles: roles.iter().map(|r| (*r).into()).collect(),
            branches: Vec::new(),
            platform: false,
            view_as: false,
            read_only: false,
            display_name: None,
            feature_grants: Vec::new(),
            authz_subject_version: 0,
            authz_policy_version: 0,
            session_generation: 0,
            issued_at: OffsetDateTime::now_utc(),
        }
    }

    fn is_public_refusal<T>(result: &Result<T, RequestContextError>) -> bool {
        match result {
            Err(
                RequestContextError::InvalidToken
                | RequestContextError::LegacySessionRejected
                | RequestContextError::InvalidClaim(_)
                | RequestContextError::WrongTokenTier,
            ) => true,
            Err(RequestContextError::AccessScope(error)) => error.kind == ErrorKind::Forbidden,
            _ => false,
        }
    }

    fn assert_refused<T>(result: Result<T, RequestContextError>) {
        assert!(
            result.is_err(),
            "current invalid identity must not resolve a principal"
        );
        assert!(
            is_public_refusal(&result),
            "refusal must map to public401/403; internal500/503 is not denial"
        );
    }

    // Checker controls are separate from the application's behavior. Constructed
    // errors test this private predicate only; they never replace a runtime owner.
    fn assert_refusal_oracle_controls() {
        for error in [
            RequestContextError::InvalidToken,
            RequestContextError::LegacySessionRejected,
            RequestContextError::InvalidClaim("control"),
            RequestContextError::WrongTokenTier,
            RequestContextError::AccessScope(KernelError::forbidden("control")),
        ] {
            assert!(
                is_public_refusal::<()>(&Err(error)),
                "legitimate public denial control"
            );
        }
        assert!(
            !is_public_refusal(&Ok(())),
            "successful resolution is not refusal"
        );
        for error in [
            RequestContextError::VerifierUnavailable,
            RequestContextError::SessionVerificationUnavailable,
            RequestContextError::BranchScope("control".into()),
            RequestContextError::EffectivePolicy("control".into()),
            RequestContextError::AccessScope(KernelError::internal("control")),
            RequestContextError::AccessScope(KernelError::validation("control")),
            RequestContextError::AccessScope(KernelError::not_found("control")),
        ] {
            assert!(
                !is_public_refusal::<()>(&Err(error)),
                "internal/unavailable control must not satisfy denial"
            );
        }
    }

    #[sqlx::test(migrations = false)]
    async fn live_authority_ordinary_unchanged_positive_both_entry_points(owner: PgPool) {
        assert_refusal_oracle_controls();
        console_platform_test_support::prepare_account_test_database(&owner).await;
        let harness = Harness::new(&owner).await;
        let (org, subject) = tenant_subject(&owner).await;
        let (session, business) = bindings(&owner, &harness).await;
        let token = harness.token(subject, org, false, "MECHANIC");
        let via_headers = resolve_principal(&session, &business, &headers(&token))
            .await
            .unwrap();
        let via_token = resolve_principal_from_bearer_token(&session, &business, &token)
            .await
            .unwrap();
        for principal in [via_headers, via_token] {
            assert_eq!(principal.user_id, subject);
            assert_eq!(principal.org_id, org);
            assert_eq!(principal.roles, BTreeSet::from([Role::Mechanic]));
            assert_eq!(principal.branch_scope, BranchScope::none());
            assert_eq!(principal.access_scope, AccessScope::legacy_org(org));
        }
    }

    #[sqlx::test(migrations = false)]
    async fn live_authority_ordinary_same_bearer_observes_committed_downgrade(owner: PgPool) {
        console_platform_test_support::prepare_account_test_database(&owner).await;
        let harness = Harness::new(&owner).await;
        let (org, subject) = tenant_subject(&owner).await;
        set_roles(&owner, subject, &["SUPER_ADMIN"]).await;
        let (session, business) = bindings(&owner, &harness).await;
        let token = harness.token(subject, org, false, "SUPER_ADMIN");
        let before = resolve_principal(&session, &business, &headers(&token))
            .await
            .unwrap();
        assert_eq!(before.roles, BTreeSet::from([Role::SuperAdmin]));
        assert_eq!(before.branch_scope, BranchScope::All);
        set_roles(&owner, subject, &["MEMBER"]).await;
        let after = resolve_principal(&session, &business, &headers(&token))
            .await
            .unwrap();
        assert_eq!(
            after.roles,
            BTreeSet::from([Role::Member]),
            "LIVE_ROLE_DOWNGRADE"
        );
        assert_eq!(after.branch_scope, BranchScope::none());
    }

    #[sqlx::test(migrations = false)]
    async fn live_authority_partial_removal_retains_unaffected_current_role(owner: PgPool) {
        console_platform_test_support::prepare_account_test_database(&owner).await;
        let harness = Harness::new(&owner).await;
        let (org, subject) = tenant_subject(&owner).await;
        set_roles(&owner, subject, &["SUPER_ADMIN", "MECHANIC"]).await;
        let (session, business) = bindings(&owner, &harness).await;
        let token = harness
            .issuer()
            .issue_access_token(input(subject, org, &["SUPER_ADMIN", "MECHANIC"]))
            .unwrap();
        let before = resolve_principal_from_bearer_token(&session, &business, &token)
            .await
            .unwrap();
        assert_eq!(
            before.roles,
            BTreeSet::from([Role::SuperAdmin, Role::Mechanic])
        );
        set_roles(&owner, subject, &["MECHANIC"]).await;
        let after = resolve_principal_from_bearer_token(&session, &business, &token)
            .await
            .unwrap();
        assert_eq!(
            after.roles,
            BTreeSet::from([Role::Mechanic]),
            "LIVE_PARTIAL_REMOVAL"
        );
        assert_eq!(after.branch_scope, BranchScope::none());
    }

    #[sqlx::test(migrations = false)]
    async fn live_authority_role_addition_applies_without_widening_signed_scope(owner: PgPool) {
        console_platform_test_support::prepare_account_test_database(&owner).await;
        let harness = Harness::new(&owner).await;
        let (org, subject) = tenant_subject(&owner).await;
        let (session, business) = bindings(&owner, &harness).await;
        // Pure signed-scope projection is exercised here. This UUID is not a
        // claim that a data-bearing branch exists or that resource access passes.
        let branch = BranchId::new();
        let scope = AccessScope::new(
            AccessScopeLevel::Branch,
            ScopeNodeId::from_uuid(*branch.as_uuid()),
        );
        let token = harness
            .issuer()
            .issue_scoped_access_token(input(subject, org, &["MECHANIC"]), scope, Vec::new())
            .unwrap();
        let before = resolve_principal_from_bearer_token(&session, &business, &token)
            .await
            .unwrap();
        assert_eq!(before.roles, BTreeSet::from([Role::Mechanic]));
        assert_eq!(before.branch_scope, BranchScope::none());
        set_roles(&owner, subject, &["SUPER_ADMIN", "MECHANIC"]).await;
        let after = resolve_principal_from_bearer_token(&session, &business, &token)
            .await
            .unwrap();
        assert_eq!(
            after.roles,
            BTreeSet::from([Role::SuperAdmin, Role::Mechanic]),
            "LIVE_ROLE_ADDITION"
        );
        assert_eq!(after.access_scope, scope);
        assert_eq!(after.branch_scope, BranchScope::single(branch));
    }

    #[sqlx::test(migrations = false)]
    async fn live_authority_inactive_target_refused_after_success(owner: PgPool) {
        console_platform_test_support::prepare_account_test_database(&owner).await;
        let harness = Harness::new(&owner).await;
        let (org, subject) = tenant_subject(&owner).await;
        set_roles(&owner, subject, &["SUPER_ADMIN"]).await;
        let (session, business) = bindings(&owner, &harness).await;
        let token = harness.token(subject, org, false, "SUPER_ADMIN");
        resolve_principal_from_bearer_token(&session, &business, &token)
            .await
            .unwrap();
        set_active(&owner, subject, false).await;
        assert_refused(resolve_principal_from_bearer_token(&session, &business, &token).await);
    }

    #[sqlx::test(migrations = false)]
    async fn live_authority_missing_target_is_not_platform_delegation(owner: PgPool) {
        console_platform_test_support::prepare_account_test_database(&owner).await;
        let harness = Harness::new(&owner).await;
        let (org, control) = tenant_subject(&owner).await;
        let (session, business) = bindings(&owner, &harness).await;
        let valid = harness.token(control, org, false, "MECHANIC");
        resolve_principal_from_bearer_token(&session, &business, &valid)
            .await
            .unwrap();
        let missing = UserId::new();
        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM users WHERE id=$1")
            .bind(missing.as_uuid())
            .fetch_one(&owner)
            .await
            .unwrap();
        assert_eq!(count, 0);
        let token = harness.token(missing, org, false, "SUPER_ADMIN");
        assert_refused(resolve_principal_from_bearer_token(&session, &business, &token).await);
    }

    #[sqlx::test(migrations = false)]
    async fn live_authority_empty_current_roles_refused_after_success(owner: PgPool) {
        console_platform_test_support::prepare_account_test_database(&owner).await;
        let harness = Harness::new(&owner).await;
        let (org, subject) = tenant_subject(&owner).await;
        let (session, business) = bindings(&owner, &harness).await;
        let token = harness.token(subject, org, false, "MECHANIC");
        resolve_principal_from_bearer_token(&session, &business, &token)
            .await
            .unwrap();
        set_roles(&owner, subject, &[]).await;
        assert_refused(resolve_principal_from_bearer_token(&session, &business, &token).await);
    }

    #[sqlx::test(migrations = false)]
    async fn live_authority_projection_permission_failure_is_unavailable_and_recovers(
        owner: PgPool,
    ) {
        console_platform_test_support::prepare_account_test_database(&owner).await;
        let harness = Harness::new(&owner).await;
        let (org, subject) = tenant_subject(&owner).await;
        let (session, business) = bindings(&owner, &harness).await;
        let token = harness.token(subject, org, false, "MECHANIC");
        resolve_principal_from_bearer_token(&session, &business, &token)
            .await
            .unwrap();
        let admitted: bool = sqlx::query_scalar("SELECT pg_catalog.has_function_privilege(current_user,'public.auth_legacy_session_context_v1(uuid,uuid)','EXECUTE')")
            .fetch_one(session.auth_pool()).await.unwrap();
        assert!(admitted, "real Auth LOGIN initially has function execution");
        let before: String = sqlx::query_scalar("SELECT proacl::text FROM pg_catalog.pg_proc WHERE oid='public.auth_legacy_session_context_v1(uuid,uuid)'::regprocedure")
            .fetch_one(&owner).await.unwrap();
        sqlx::query("REVOKE EXECUTE ON FUNCTION public.auth_legacy_session_context_v1(uuid,uuid) FROM console_auth_rt")
            .execute(&owner).await.unwrap();
        // Keep failure injection/verification in a joined task so even a panic
        // is caught before the owner restores the privilege. No detached work.
        let fault_session = session.clone();
        let fault_business = business.clone();
        let fault_token = token.clone();
        let outcome = tokio::spawn(async move {
            let admitted: bool = sqlx::query_scalar("SELECT pg_catalog.has_function_privilege(current_user,'public.auth_legacy_session_context_v1(uuid,uuid)','EXECUTE')")
                .fetch_one(fault_session.auth_pool()).await.unwrap();
            assert!(!admitted, "real Auth LOGIN lost exactly this function execution");
            resolve_principal_from_bearer_token(&fault_session, &fault_business, &fault_token).await
        }).await;
        // Restore exact function privilege before unwrapping a caught task panic
        // or asserting the expected RED, so baseline failure leaves no ACL change.
        sqlx::query("GRANT EXECUTE ON FUNCTION public.auth_legacy_session_context_v1(uuid,uuid) TO console_auth_rt")
            .execute(&owner).await.unwrap();
        let restored: String = sqlx::query_scalar("SELECT proacl::text FROM pg_catalog.pg_proc WHERE oid='public.auth_legacy_session_context_v1(uuid,uuid)'::regprocedure")
            .fetch_one(&owner).await.unwrap();
        assert_eq!(restored, before, "fault injection restores the exact ACL");
        let admitted: bool = sqlx::query_scalar("SELECT pg_catalog.has_function_privilege(current_user,'public.auth_legacy_session_context_v1(uuid,uuid)','EXECUTE')")
            .fetch_one(session.auth_pool()).await.unwrap();
        assert!(admitted, "real Auth LOGIN execution is restored");
        let outcome = outcome.expect("fault-path task must not panic");
        let recovered = resolve_principal_from_bearer_token(&session, &business, &token)
            .await
            .unwrap();
        assert_eq!(recovered.roles, BTreeSet::from([Role::Mechanic]));
        assert!(
            matches!(
                outcome,
                Err(RequestContextError::SessionVerificationUnavailable)
            ),
            "LIVE_PROJECTION_REQUIRED"
        );
    }

    #[sqlx::test(migrations = false)]
    async fn live_authority_platform_delegations_preserve_acting_roles_then_revoke(owner: PgPool) {
        console_platform_test_support::prepare_account_test_database(&owner).await;
        let harness = Harness::new(&owner).await;
        let operator = seed_platform_admin(&owner).await;
        let target = seed_tenant(&owner, "delegated-target", 1).await;
        // Current platform authority is home identity, not SUPER_ADMIN role.
        set_roles(&owner, operator, &["MEMBER"]).await;
        let platform = harness.platform_service().await;
        let platform_token = harness.token(operator, OrgId::platform(), true, "MEMBER");
        let (view_status, view_body) =
            start_view_as(&platform, &platform_token, target, "ADMIN").await;
        assert_eq!(view_status, StatusCode::OK);
        let (write_status, write_body) =
            start_tenant_context(&platform, &platform_token, target).await;
        assert_eq!(write_status, StatusCode::OK);
        let view_token = view_body["access_token"].as_str().unwrap();
        let write_token = write_body["access_token"].as_str().unwrap();
        let (session, business) = bindings(&owner, &harness).await;
        let target_rows: i64 =
            sqlx::query_scalar("SELECT count(*) FROM users WHERE id=$1 AND org_id=$2")
                .bind(operator.as_uuid())
                .bind(target)
                .fetch_one(&owner)
                .await
                .unwrap();
        assert_eq!(target_rows, 0);
        let view = resolve_principal_from_bearer_token(&session, &business, view_token)
            .await
            .unwrap();
        let write = resolve_principal_from_bearer_token(&session, &business, write_token)
            .await
            .unwrap();
        assert_eq!(view.roles, BTreeSet::from([Role::Admin]));
        assert_eq!(view.branch_scope, BranchScope::none());
        assert_eq!(write.roles, BTreeSet::from([Role::SuperAdmin]));
        assert_eq!(write.branch_scope, BranchScope::All);
        for principal in [view, write] {
            assert_eq!(principal.user_id, operator);
            assert_eq!(principal.org_id, OrgId::from_uuid(target));
            assert_eq!(
                principal.access_scope,
                AccessScope::legacy_org(OrgId::from_uuid(target))
            );
        }
        set_active(&owner, operator, false).await;
        // Evaluate both before asserting: neither delegated branch is hidden
        // behind the first expected baseline failure.
        let view_after = resolve_principal_from_bearer_token(&session, &business, view_token).await;
        let write_after =
            resolve_principal_from_bearer_token(&session, &business, write_token).await;
        assert_refused(view_after);
        assert_refused(write_after);
    }

    #[sqlx::test(migrations = false)]
    async fn live_authority_platform_tier_requires_current_active_home(owner: PgPool) {
        console_platform_test_support::prepare_account_test_database(&owner).await;
        let harness = Harness::new(&owner).await;
        let operator = seed_platform_admin(&owner).await;
        let (session, _business) = bindings(&owner, &harness).await;
        let token = harness.token(operator, OrgId::platform(), true, "SUPER_ADMIN");
        let before = resolve_platform_principal(&session, &headers(&token))
            .await
            .unwrap();
        assert_eq!(before.user_id, operator);
        // Explicit new guard: historical JWT validation accepts any UUID org
        // even on a platform token; the current platform resolver ignored it.
        let tenant_org = OrgId::from_uuid(seed_tenant(&owner, "platform-org-mismatch", 1).await);
        let wrong_org_token = harness.token(operator, tenant_org, true, "SUPER_ADMIN");
        let signed = harness
            .verifier()
            .verify_access_token(&wrong_org_token)
            .unwrap();
        assert!(signed.platform);
        assert_eq!(signed.org, tenant_org.to_string());
        let wrong_org_outcome =
            resolve_platform_principal(&session, &headers(&wrong_org_token)).await;
        set_active(&owner, operator, false).await;
        let after = resolve_platform_principal(&session, &headers(&token)).await;
        assert!(after.is_err(), "LIVE_PLATFORM_HOME_REQUIRED");
        assert_refused(after);
        assert_refused(wrong_org_outcome);
    }

    #[sqlx::test(migrations = false)]
    async fn live_authority_view_as_flag_does_not_prove_platform_home(owner: PgPool) {
        console_platform_test_support::prepare_account_test_database(&owner).await;
        let harness = Harness::new(&owner).await;
        let (org, subject) = tenant_subject(&owner).await;
        set_roles(&owner, subject, &["SUPER_ADMIN"]).await;
        let (session, business) = bindings(&owner, &harness).await;
        let valid = harness.token(subject, org, false, "SUPER_ADMIN");
        resolve_principal_from_bearer_token(&session, &business, &valid)
            .await
            .unwrap();
        // Signed adversarial shape; this is not a claim the ordinary production
        // issuer currently exposes caller-selected view_as.
        let mut claims = input(subject, org, &["SUPER_ADMIN"]);
        claims.view_as = true;
        claims.read_only = true;
        let token = harness.issuer().issue_access_token(claims).unwrap();
        assert_refused(resolve_principal_from_bearer_token(&session, &business, &token).await);
    }

    #[sqlx::test(migrations = false)]
    async fn live_authority_unmarked_platform_context_requires_writable_super_admin_shape(
        owner: PgPool,
    ) {
        console_platform_test_support::prepare_account_test_database(&owner).await;
        let harness = Harness::new(&owner).await;
        let operator = seed_platform_admin(&owner).await;
        let target = OrgId::from_uuid(seed_tenant(&owner, "shape-target", 1).await);
        let (session, business) = bindings(&owner, &harness).await;
        let valid = harness.token(operator, target, false, "SUPER_ADMIN");
        resolve_principal_from_bearer_token(&session, &business, &valid)
            .await
            .unwrap();
        let token = harness.token(operator, target, false, "ADMIN");
        let wrong_single = resolve_principal_from_bearer_token(&session, &business, &token).await;
        let mut malformed_shapes = Vec::new();
        for role_codes in [
            Vec::<&str>::new(),
            vec!["SUPER_ADMIN", "ADMIN"],
            vec!["SUPER_ADMIN", "SUPER_ADMIN"],
        ] {
            for read_only_view in [false, true] {
                let mut claims = input(operator, target, &role_codes);
                claims.view_as = read_only_view;
                claims.read_only = read_only_view;
                let malformed = harness.issuer().issue_access_token(claims).unwrap();
                let signed = harness.verifier().verify_access_token(&malformed).unwrap();
                assert_eq!(signed.roles.len(), role_codes.len());
                malformed_shapes.push(
                    resolve_principal_from_bearer_token(&session, &business, &malformed).await,
                );
            }
        }
        // Evaluate every malformed request before asserting, so baseline wrong
        // ADMIN cannot prevent execution of the empty/multiple-role controls.
        assert_refused(wrong_single);
        for outcome in malformed_shapes {
            assert_refused(outcome);
        }
    }

    #[sqlx::test(migrations = false)]
    async fn live_authority_missing_platform_home_refuses_tier_and_view_as(owner: PgPool) {
        console_platform_test_support::prepare_account_test_database(&owner).await;
        let harness = Harness::new(&owner).await;
        let operator = seed_platform_admin(&owner).await;
        let target = OrgId::from_uuid(seed_tenant(&owner, "missing-home-target", 1).await);
        let (session, business) = bindings(&owner, &harness).await;
        let positive = harness.token(operator, OrgId::platform(), true, "SUPER_ADMIN");
        resolve_platform_principal(&session, &headers(&positive))
            .await
            .unwrap();
        let missing = UserId::new();
        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM users WHERE id=$1")
            .bind(missing.as_uuid())
            .fetch_one(&owner)
            .await
            .unwrap();
        assert_eq!(count, 0);
        let platform_token = harness.token(missing, OrgId::platform(), true, "SUPER_ADMIN");
        let mut claims = input(missing, target, &["SUPER_ADMIN"]);
        claims.view_as = true;
        claims.read_only = true;
        let view_token = harness.issuer().issue_access_token(claims).unwrap();
        let platform = resolve_platform_principal(&session, &headers(&platform_token)).await;
        let view = resolve_principal_from_bearer_token(&session, &business, &view_token).await;
        assert!(
            platform.is_err(),
            "missing platform home must not resolve platform authority"
        );
        assert_refused(platform);
        assert_refused(view);
    }

    #[sqlx::test(migrations = false)]
    async fn live_authority_platform_delegation_rejects_inconsistent_read_only_flags(
        owner: PgPool,
    ) {
        console_platform_test_support::prepare_account_test_database(&owner).await;
        let harness = Harness::new(&owner).await;
        let operator = seed_platform_admin(&owner).await;
        let target = OrgId::from_uuid(seed_tenant(&owner, "flag-target", 1).await);
        let (session, business) = bindings(&owner, &harness).await;
        let valid = harness.token(operator, target, false, "SUPER_ADMIN");
        resolve_principal_from_bearer_token(&session, &business, &valid)
            .await
            .unwrap();
        let mut outcomes = Vec::new();
        for (view_as, read_only) in [(true, false), (false, true)] {
            let mut claims = input(operator, target, &["SUPER_ADMIN"]);
            claims.view_as = view_as;
            claims.read_only = read_only;
            let token = harness.issuer().issue_access_token(claims).unwrap();
            outcomes.push(resolve_principal_from_bearer_token(&session, &business, &token).await);
        }
        for outcome in outcomes {
            assert_refused(outcome);
        }
    }
}
