#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Isolated Auth7 purpose-separation target. The pinned auth_rest target is unchanged.
//! Setup and HTTP/SoftPasskey helpers below are borrowed from auth_rest at19818c07.

use axum::body::{Body, to_bytes};
use console_app::{AppConfig, AppRole, AppState, build_router, run_migrations};
use console_kernel_core::{BranchId, OrgId, UserId};
use console_platform_provisioning::BootstrapCredentialStore;
use console_platform_test_support::{TestDatabaseLogin, login_test_database_url};
use http::{Request, StatusCode, header};
use p256::ecdsa::SigningKey;
use p256::elliptic_curve::rand_core::OsRng;
use p256::pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::PgPool;
use time::{Duration, OffsetDateTime};
use tower::ServiceExt;
use url::Url;
use uuid::Uuid;
use webauthn_authenticator_rs::prelude::WebauthnAuthenticator;
use webauthn_authenticator_rs::softpasskey::SoftPasskey;
use webauthn_rs::prelude::{CreationChallengeResponse, RequestChallengeResponse};

const TEST_ISSUER: &str = "console-platform-auth";
const TEST_AUDIENCE: &str = "console-api";
const TEST_ORIGIN: &str = "https://auth.example.com";

#[derive(Debug, Deserialize)]
struct RegisterStartResponse {
    ceremony_id: Uuid,
    challenge: CreationChallengeResponse,
}

#[derive(Debug, Deserialize)]
struct RegisterFinishResponse {
    credential_id: String,
}

#[derive(Debug, Deserialize)]
struct OtpRedeemResponse {
    access_token: String,
}

#[derive(Debug, Deserialize)]
struct PrivacyConsentStatusResponse {
    policy_version: String,
    accepted: bool,
    #[serde(with = "time::serde::rfc3339::option")]
    accepted_at: Option<OffsetDateTime>,
}

async fn prepare_http_database(pool: &PgPool) {
    // Same actual migrator and canonical composed finalizer as the admitted
    // serving fixture. Missing Auth7 SQL is a prerequisite, never a fallback.
    let config = AppConfig::from_pairs([
        ("CONSOLE_APP_ROLE", AppRole::Migrate.to_string()),
        (
            "DATABASE_URL",
            console_platform_test_support::prepare_test_migration_owner_url(pool).await,
        ),
    ])
    .expect("production migration configuration");
    run_migrations(&config)
        .await
        .expect("complete production schema before HTTP fixtures");
    console_platform_test_support::finalize_serving_account_custody(pool).await;
}
async fn app_state(
    pool: PgPool,
    private_key_pem: String,
    public_key_pem: String,
) -> Result<AppState, console_app::AppError> {
    let mut pairs = vec![
        (
            "CONSOLE_DATABASE_DURABILITY",
            r#"{"mode":"local_development"}"#.to_owned(),
        ),
        ("CONSOLE_APP_ROLE", AppRole::Api.to_string()),
        ("CONSOLE_HTTP_ADDR", "127.0.0.1:0".to_owned()),
        ("CONSOLE_JWT_ISSUER", TEST_ISSUER.to_owned()),
        ("CONSOLE_JWT_AUDIENCE", TEST_AUDIENCE.to_owned()),
        ("CONSOLE_JWT_PRIVATE_KEY_PEM", private_key_pem),
        ("CONSOLE_JWT_PUBLIC_KEY_PEM", public_key_pem),
        ("CONSOLE_WEBAUTHN_RP_ID", "example.com".to_owned()),
        ("CONSOLE_WEBAUTHN_RP_ORIGIN", TEST_ORIGIN.to_owned()),
        ("CONSOLE_WEBAUTHN_RP_NAME", "Console".to_owned()),
    ];
    pairs.extend(account_transport_urls(&pool));
    let config = AppConfig::from_pairs(pairs)?;

    AppState::from_config(config).await
}

fn account_transport_urls(owner_pool: &PgPool) -> Vec<(&'static str, String)> {
    [
        ("DATABASE_URL", TestDatabaseLogin::Business),
        ("AUTH_DATABASE_URL", TestDatabaseLogin::Auth),
        (
            "LEAVE_COMMAND_DATABASE_URL",
            TestDatabaseLogin::LeaveCommand,
        ),
        (
            "ONTOLOGY_COMMAND_DATABASE_URL",
            TestDatabaseLogin::OntologyCommand,
        ),
        (
            "PLATFORM_FORCE_COMMAND_DATABASE_URL",
            TestDatabaseLogin::PlatformForceCommand,
        ),
    ]
    .into_iter()
    .map(|(key, login)| (key, login_test_database_url(owner_pool, login)))
    .collect()
}

async fn seed_branch(pool: &PgPool, region_name: &str, branch_name: &str) -> BranchId {
    let region_id: uuid::Uuid =
        sqlx::query_scalar("INSERT INTO regions (name, org_id) VALUES ($1, $2) RETURNING id")
            .bind(region_name)
            .bind(*OrgId::knl().as_uuid())
            .fetch_one(pool)
            .await
            .unwrap();
    let branch_id: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO branches (region_id, name, org_id) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(region_id)
    .bind(branch_name)
    .bind(*OrgId::knl().as_uuid())
    .fetch_one(pool)
    .await
    .unwrap();
    BranchId::from_uuid(branch_id)
}

async fn seed_user_with_branch(
    pool: &PgPool,
    display_name: &str,
    phone: &str,
    role: &str,
    branch_id: BranchId,
) -> UserId {
    let user_id = UserId::new();
    sqlx::query(
        "INSERT INTO users (id, display_name, phone, roles, org_id) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(*user_id.as_uuid())
    .bind(display_name)
    .bind(phone)
    .bind(Vec::from([role]))
    .bind(*OrgId::knl().as_uuid())
    .execute(pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO user_branches (user_id, branch_id, org_id) VALUES ($1, $2, $3)")
        .bind(*user_id.as_uuid())
        .bind(*branch_id.as_uuid())
        .bind(*OrgId::knl().as_uuid())
        .execute(pool)
        .await
        .unwrap();
    user_id
}

async fn seed_mobile_step_up_poll(pool: &PgPool, actor: UserId) -> (Uuid, Uuid) {
    let poll_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO collaboration_polls (
            id, org_id, target_scope_type, title, question, status, anonymity,
            allow_multiple, created_by, updated_by
        )
        VALUES ($1, $2, 'ORG', 'Mobile step-up poll', 'Select one', 'OPEN', 'NAMED', false, $3, $3)
        "#,
    )
    .bind(poll_id)
    .bind(*OrgId::knl().as_uuid())
    .bind(*actor.as_uuid())
    .execute(pool)
    .await
    .unwrap();
    let option_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO collaboration_poll_options (org_id, poll_id, label, position)
        VALUES ($1, $2, 'Approve', 0)
        RETURNING id
        "#,
    )
    .bind(*OrgId::knl().as_uuid())
    .bind(poll_id)
    .fetch_one(pool)
    .await
    .unwrap();
    sqlx::query(
        r#"
        INSERT INTO collaboration_poll_options (org_id, poll_id, label, position)
        VALUES ($1, $2, 'Reject', 1)
        "#,
    )
    .bind(*OrgId::knl().as_uuid())
    .bind(poll_id)
    .execute(pool)
    .await
    .unwrap();
    (poll_id, option_id)
}

async fn assert_poll_vote_count(pool: &PgPool, poll_id: Uuid, expected: i64) {
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM collaboration_poll_votes WHERE poll_id = $1")
            .bind(poll_id)
            .fetch_one(pool)
            .await
            .unwrap();
    assert_eq!(count, expected, "unexpected vote count for poll {poll_id}");
}

async fn post_raw(
    service: axum::Router,
    uri: &str,
    bearer: Option<&str>,
    body: Value,
) -> http::Response<Body> {
    let mut builder = Request::builder()
        .uri(uri)
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(token) = bearer {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    service
        .oneshot(builder.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap()
}

async fn post_json<T>(
    service: axum::Router,
    uri: &str,
    bearer: Option<&str>,
    body: Value,
    expected: StatusCode,
) -> T
where
    T: for<'de> Deserialize<'de>,
{
    let response = post_raw(service, uri, bearer, body).await;
    assert_eq!(response.status(), expected);
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn post_cookie_mode(
    service: axum::Router,
    uri: &str,
    cookie: Option<&str>,
    body: Value,
) -> http::Response<Body> {
    let mut builder = Request::builder()
        .uri(uri)
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-auth-transport", "cookie");
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, format!("console_refresh={cookie}"));
    }
    service
        .oneshot(builder.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap()
}

async fn body_json(response: http::Response<Body>) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn admin_session_via_otp(service: &axum::Router, pool: &PgPool, user_id: UserId) -> String {
    let issue = BootstrapCredentialStore
        .issue_for_zero_credential_user(
            pool,
            *user_id.as_uuid(),
            OrgId::knl(),
            OffsetDateTime::now_utc(),
            Duration::hours(24),
        )
        .await
        .unwrap();
    let redeem: OtpRedeemResponse = post_json(
        service.clone(),
        "/api/v1/auth/otp/redeem",
        None,
        json!({ "otp": issue.token.as_str() }),
        StatusCode::OK,
    )
    .await;
    redeem.access_token
}

async fn enroll_passkey(
    service: &axum::Router,
    authenticator: &mut WebauthnAuthenticator<SoftPasskey>,
    access_token: &str,
) -> String {
    accept_required_privacy_consent(service, access_token).await;
    let registration: RegisterStartResponse = post_json(
        service.clone(),
        "/api/v1/auth/passkey/register/start",
        Some(access_token),
        json!({ "username": "new.user", "display_name": "New User" }),
        StatusCode::OK,
    )
    .await;
    let credential = authenticator
        .do_registration(Url::parse(TEST_ORIGIN).unwrap(), registration.challenge)
        .unwrap();
    let finish: RegisterFinishResponse = post_json(
        service.clone(),
        "/api/v1/auth/passkey/register/finish",
        Some(access_token),
        json!({ "ceremony_id": registration.ceremony_id, "credential": credential }),
        StatusCode::CREATED,
    )
    .await;
    finish.credential_id
}

async fn accept_required_privacy_consent(
    service: &axum::Router,
    access_token: &str,
) -> PrivacyConsentStatusResponse {
    let status: PrivacyConsentStatusResponse = post_json(
        service.clone(),
        "/api/v1/auth/privacy-consent/status",
        Some(access_token),
        json!({}),
        StatusCode::OK,
    )
    .await;
    if status.accepted {
        return status;
    }

    let accepted: PrivacyConsentStatusResponse = post_json(
        service.clone(),
        "/api/v1/auth/privacy-consent/accept",
        Some(access_token),
        json!({
            "policy_version": status.policy_version,
            "privacy_collection": true,
            "terms_of_service": true
        }),
        StatusCode::OK,
    )
    .await;
    assert!(accepted.accepted);
    assert!(
        accepted.accepted_at.is_some(),
        "accepted consent must record an audit timestamp"
    );
    accepted
}

fn inject_allow_credential(
    challenge: RequestChallengeResponse,
    credential_id: &str,
) -> RequestChallengeResponse {
    let mut value = serde_json::to_value(&challenge).unwrap();
    let allow = value
        .get_mut("publicKey")
        .and_then(|pk| pk.get_mut("allowCredentials"))
        .and_then(Value::as_array_mut)
        .expect("discoverable challenge must have an allowCredentials array");
    allow.push(json!({ "type": "public-key", "id": credential_id }));
    serde_json::from_value(value).unwrap()
}

async fn start_mobile_step_up_assertion(
    service: &axum::Router,
    authenticator: &mut WebauthnAuthenticator<SoftPasskey>,
    credential_id: &str,
    access_token: &str,
    binding: Value,
) -> Value {
    let start = post_json::<Value>(
        service.clone(),
        "/api/v1/auth/passkey/step-up/start",
        Some(access_token),
        json!({ "binding": binding.clone() }),
        StatusCode::OK,
    )
    .await;
    assert_eq!(start["binding"], binding);

    let ceremony_id = start["ceremony_id"]
        .as_str()
        .expect("step-up start must return ceremony id")
        .to_owned();
    let challenge: RequestChallengeResponse =
        serde_json::from_value(start["challenge"].clone()).unwrap();
    let challenge = inject_allow_credential(challenge, credential_id);
    let assertion = authenticator
        .do_authentication(Url::parse(TEST_ORIGIN).unwrap(), challenge)
        .unwrap();

    json!({
        "binding": binding,
        "assertion": {
            "ceremony_id": ceremony_id,
            "credential": assertion
        }
    })
}

async fn assert_persisted_mobile_step_up_binding(pool: &PgPool, step_up: &Value) {
    let ceremony_id = Uuid::parse_str(
        step_up["assertion"]["ceremony_id"]
            .as_str()
            .expect("step-up assertion must carry ceremony id"),
    )
    .unwrap();
    let row: (String, Uuid, String, Option<i32>) = sqlx::query_as(
        r#"
        SELECT action_kind, object_id, reason_key, replay_attempt
        FROM auth_webauthn_ceremony_bindings
        WHERE ceremony_id = $1
        "#,
    )
    .bind(ceremony_id)
    .fetch_one(pool)
    .await
    .unwrap();
    assert_eq!(row.0, step_up["binding"]["action_kind"]);
    assert_eq!(row.1.to_string(), step_up["binding"]["object_id"]);
    assert_eq!(row.2, step_up["binding"]["reason_key"]);
    assert_eq!(
        row.3,
        step_up["binding"]["replay_attempt"]
            .as_i64()
            .map(|value| value as i32)
    );
}
// Auth7 purpose separation uses unchanged HTTP contracts and the existing real
// SoftPasskey harness, so the assertions can run before owner API changes.
struct Auth7MobilePurposeFixture {
    router: axum::Router,
    subject: UserId,
    access: String,
    authenticator: WebauthnAuthenticator<SoftPasskey>,
    credential_id: String,
}

async fn auth7_mobile_purpose_fixture(pool: &PgPool) -> Auth7MobilePurposeFixture {
    prepare_http_database(pool).await;
    let key = SigningKey::random(&mut OsRng);
    let private = key.to_pkcs8_pem(LineEnding::LF).unwrap();
    let public = key
        .verifying_key()
        .to_public_key_pem(LineEnding::LF)
        .unwrap();
    let branch = seed_branch(pool, "Purpose Region", "Purpose Branch").await;
    let subject =
        seed_user_with_branch(pool, "Purpose Admin", "010-4900-0001", "ADMIN", branch).await;
    let router = build_router(
        app_state(pool.clone(), private.to_string(), public)
            .await
            .unwrap(),
    );
    let access = admin_session_via_otp(&router, pool, subject).await;
    let mut authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let credential_id = enroll_passkey(&router, &mut authenticator, &access).await;
    let mut fixture = Auth7MobilePurposeFixture {
        router,
        subject,
        access,
        authenticator,
        credential_id,
    };
    // Establish a real positive before any denial: a missing owner, bad fixture,
    // invalid signature, or blanket denial cannot satisfy these tests.
    let (proof, poll, option) = auth7_mobile_purpose_proof(pool, &mut fixture).await;
    auth7_use_correct_mobile_purpose(pool, &fixture, &proof, poll, option).await;
    fixture
}

async fn auth7_mobile_purpose_proof(
    pool: &PgPool,
    fixture: &mut Auth7MobilePurposeFixture,
) -> (Value, Uuid, Uuid) {
    let (poll, option) = seed_mobile_step_up_poll(pool, fixture.subject).await;
    let proof = start_mobile_step_up_assertion(
        &fixture.router,
        &mut fixture.authenticator,
        &fixture.credential_id,
        &fixture.access,
        json!({
            "action_kind": "POLL_VOTE",
            "object_id": poll,
            "reason_key": "operations_passkey_poll_vote",
            "replay_attempt": null
        }),
    )
    .await;
    assert_persisted_mobile_step_up_binding(pool, &proof).await;
    let consumed: Option<OffsetDateTime> =
        sqlx::query_scalar("SELECT consumed_at FROM public.auth_webauthn_ceremonies WHERE id=$1")
            .bind(Uuid::parse_str(proof["assertion"]["ceremony_id"].as_str().unwrap()).unwrap())
            .fetch_one(pool)
            .await
            .unwrap();
    assert!(
        consumed.is_none(),
        "fresh actual mobile ceremony must be unconsumed"
    );
    (proof, poll, option)
}

async fn auth7_use_correct_mobile_purpose(
    pool: &PgPool,
    fixture: &Auth7MobilePurposeFixture,
    proof: &Value,
    poll: Uuid,
    option: Uuid,
) {
    let voted = post_json::<Value>(
        fixture.router.clone(),
        &format!("/api/v1/mobile/collaboration/polls/{poll}/vote"),
        Some(&fixture.access),
        json!({"selected_option_ids": [option], "step_up": proof}),
        StatusCode::OK,
    )
    .await;
    assert_eq!(voted["my_vote"]["submitted"], true);
    assert_poll_vote_count(pool, poll, 1).await;
    let consumed: Option<OffsetDateTime> =
        sqlx::query_scalar("SELECT consumed_at FROM public.auth_webauthn_ceremonies WHERE id=$1")
            .bind(Uuid::parse_str(proof["assertion"]["ceremony_id"].as_str().unwrap()).unwrap())
            .fetch_one(pool)
            .await
            .unwrap();
    assert!(
        consumed.is_some(),
        "correct mobile operation must consume this exact proof"
    );
}

async fn auth7_purpose_state(pool: &PgPool) -> Value {
    sqlx::query_scalar(r#"SELECT jsonb_build_object(
      'users',(SELECT COALESCE(jsonb_agg(to_jsonb(x) ORDER BY x.id),'[]'::jsonb) FROM public.users x),
      'accounts',(SELECT COALESCE(jsonb_agg(to_jsonb(x) ORDER BY x.id),'[]'::jsonb) FROM public.accounts x),
      'security',(SELECT COALESCE(jsonb_agg(to_jsonb(x) ORDER BY x.account_id),'[]'::jsonb) FROM public.account_security x),
      'keys',(SELECT COALESCE(jsonb_agg(to_jsonb(x) ORDER BY x.id),'[]'::jsonb) FROM public.auth_webauthn_credentials x),
      'ceremonies',(SELECT COALESCE(jsonb_agg(to_jsonb(x) ORDER BY x.id),'[]'::jsonb) FROM public.auth_webauthn_ceremonies x),
      'bindings',(SELECT COALESCE(jsonb_agg(to_jsonb(x) ORDER BY x.ceremony_id),'[]'::jsonb) FROM public.auth_webauthn_ceremony_bindings x),
      'families',(SELECT COALESCE(jsonb_agg(to_jsonb(x) ORDER BY x.id),'[]'::jsonb) FROM public.auth_refresh_token_families x),
      'tokens',(SELECT COALESCE(jsonb_agg(to_jsonb(x) ORDER BY x.id),'[]'::jsonb) FROM public.auth_refresh_tokens x),
      'bootstrap',(SELECT COALESCE(jsonb_agg(to_jsonb(x) ORDER BY x.id),'[]'::jsonb) FROM public.auth_bootstrap_credentials x),
      'handoffs',(SELECT COALESCE(jsonb_agg(to_jsonb(x) ORDER BY x.id),'[]'::jsonb) FROM public.auth_device_login_handoffs x),
      'audits',(SELECT COALESCE(jsonb_agg(to_jsonb(x) ORDER BY x.id),'[]'::jsonb) FROM public.audit_events x))"#)
        .fetch_one(pool).await.unwrap()
}

#[sqlx::test(migrations = false)]
async fn auth7_mobile_bound_proof_cannot_be_reused_for_ordinary_login(pool: PgPool) {
    let mut fixture = auth7_mobile_purpose_fixture(&pool).await;
    for cookie in [false, true] {
        let (proof, poll, option) = auth7_mobile_purpose_proof(&pool, &mut fixture).await;
        let before = auth7_purpose_state(&pool).await;
        let body = proof["assertion"].clone();
        let response = if cookie {
            post_cookie_mode(
                fixture.router.clone(),
                "/api/v1/auth/passkey/login/finish",
                None,
                body,
            )
            .await
        } else {
            post_raw(
                fixture.router.clone(),
                "/api/v1/auth/passkey/login/finish",
                None,
                body,
            )
            .await
        };
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "AUTH7_PURPOSE: a mobile-bound proof must not authenticate ordinary login (cookie={cookie})"
        );
        assert!(!response.headers().contains_key(header::SET_COOKIE));
        let response = body_json(response).await;
        assert!(response.get("access_token").is_none() && response.get("refresh_token").is_none());
        assert_eq!(
            before,
            auth7_purpose_state(&pool).await,
            "purpose refusal must preserve exact ceremony, credential, token and audit state"
        );
        assert_poll_vote_count(&pool, poll, 0).await;
        // Reuse the identical signed assertion through its intended operation.
        // No new proof or repaired ceremony may hide an accidental consume.
        auth7_use_correct_mobile_purpose(&pool, &fixture, &proof, poll, option).await;
    }
}

#[sqlx::test(migrations = false)]
async fn auth7_mobile_bound_proof_cannot_be_reused_for_generic_step_up(pool: PgPool) {
    let mut fixture = auth7_mobile_purpose_fixture(&pool).await;
    let (proof, poll, option) = auth7_mobile_purpose_proof(&pool, &mut fixture).await;
    let before = auth7_purpose_state(&pool).await;
    let response = post_raw(
        fixture.router.clone(),
        "/api/v1/auth/passkey/register/start",
        Some(&fixture.access),
        json!({"step_up": proof["assertion"]}),
    )
    .await;
    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "AUTH7_PURPOSE: a mobile-bound proof must not authorize generic registration step-up"
    );
    assert!(!response.headers().contains_key(header::SET_COOKIE));
    let response = body_json(response).await;
    assert!(response.get("ceremony_id").is_none() && response.get("challenge").is_none());
    assert_eq!(
        before,
        auth7_purpose_state(&pool).await,
        "purpose refusal must not consume the proof, issue a registration challenge or change credentials/audits"
    );
    assert_poll_vote_count(&pool, poll, 0).await;
    auth7_use_correct_mobile_purpose(&pool, &fixture, &proof, poll, option).await;
}
