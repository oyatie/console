#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use axum::body::{Body, to_bytes};
use axum::extract::ConnectInfo;
use console_app::{AppConfig, AppRole, AppState, build_router, run_migrations};
use console_financial_adapter_postgres::PgFinancialStore;
use console_financial_application::{
    CreatePurchaseRequestCommand, FinancialConfigSnapshot, PrepareExpenditureCommand,
    PurchaseApprovalCommand, PurchaseRequestLineInput, PurchaseSubmitCommand, PurchaseType,
};
use console_financial_domain::DepreciationMethod;
use console_kernel_core::{
    BranchId, EquipmentId, EvidenceId, OrgId, PurchaseRequestId, TraceContext, UserId, WorkOrderId,
};
use console_platform_provisioning::BootstrapCredentialStore;
use console_platform_test_support::{TestDatabaseLogin, login_test_database_url};
use http::{Request, StatusCode, header};
use p256::ecdsa::SigningKey;
use p256::elliptic_curve::rand_core::OsRng;
use p256::pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::PgPool;
use std::net::SocketAddr;
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

#[path = "auth_rest/account_custody_lifecycle.rs"]
mod account_custody_lifecycle;
#[path = "auth_rest/account_custody_startup.rs"]
mod account_custody_startup;
#[path = "auth_rest/account_fence_projection.rs"]
mod account_fence_projection;
#[path = "auth_rest/account_fence_transport.rs"]
mod account_fence_transport;
#[path = "auth_rest/account_storage.rs"]
mod account_storage;
#[path = "auth_rest/auth_target_parser.rs"]
mod auth_target_parser;
#[path = "auth_rest/publication_privileges.rs"]
mod publication_privileges;

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
struct LoginStartResponse {
    ceremony_id: Uuid,
    challenge: RequestChallengeResponse,
}

#[derive(Debug, Deserialize)]
struct TokenPairResponse {
    access_token: String,
    /// Present (body transport, mobile) or null (cookie transport, web).
    refresh_token: Option<String>,
    #[serde(default)]
    requires_passkey_setup: bool,
}

#[derive(Debug, Deserialize)]
struct DeviceLoginStartResponse {
    approve_url: String,
}

#[derive(Debug, Deserialize)]
struct OtpRedeemResponse {
    access_token: String,
    /// Present (body transport, mobile) or null (cookie transport, web).
    refresh_token: Option<String>,
    requires_passkey_setup: bool,
}

#[derive(Debug, Deserialize)]
struct AdminIssueOtpResponse {
    otp: String,
    user_id: Uuid,
}

#[derive(Debug, Deserialize)]
struct AdminCredentialResetResponse {
    otp: String,
    user_id: Uuid,
}

#[derive(Debug, Deserialize)]
struct PrivacyConsentStatusResponse {
    policy_version: String,
    accepted: bool,
    #[serde(with = "time::serde::rfc3339::option")]
    accepted_at: Option<OffsetDateTime>,
}

/// End-to-end: an admin issues a one-time code; the new user signs in for the
/// FIRST time by redeeming it (minting a session, flagged for passkey setup),
/// enrolls a passkey from that authenticated session, and then signs in again
/// usernamelessly (discoverable) with no user_id. Refresh reuse still revokes the
/// family.
#[sqlx::test(migrations = false)]
async fn otp_first_signin_then_passkey_enrollment_then_usernameless_login(pool: PgPool) {
    prepare_http_database(&pool).await;
    let signing_key = SigningKey::random(&mut OsRng);
    let private_key_pem = signing_key.to_pkcs8_pem(LineEnding::LF).unwrap();
    let public_key_pem = signing_key
        .verifying_key()
        .to_public_key_pem(LineEnding::LF)
        .unwrap();
    let branch_id = seed_branch(&pool, "Auth Region", "Auth Branch").await;
    // The admin who issues codes.
    let admin_id =
        seed_user_with_branch(&pool, "Branch Admin", "010-4000-0000", "ADMIN", branch_id).await;
    // The pre-provisioned new user who will do their first sign-in via OTP.
    let new_user_id =
        seed_user_with_branch(&pool, "New User", "010-4000-0001", "MECHANIC", branch_id).await;
    seed_equipment(&pool, branch_id, "290").await;

    let service = build_router(
        app_state(
            pool.clone(),
            private_key_pem.to_string(),
            public_key_pem.clone(),
        )
        .await
        .unwrap(),
    );

    // The admin first signs in (cold start in this test uses a directly-issued
    // OTP for the admin) and enrolls a passkey so it can call admin endpoints.
    let admin_access = admin_session_via_otp(&service, &pool, admin_id).await;

    // Admin issues a one-time code for the new user.
    let issued: AdminIssueOtpResponse = post_json(
        service.clone(),
        "/api/v1/auth/admin/otp/issue",
        Some(&admin_access),
        json!({ "user_id": new_user_id.as_uuid(), "branch_id": branch_id }),
        StatusCode::OK,
    )
    .await;
    assert_eq!(&issued.user_id, new_user_id.as_uuid());
    assert_eq!(issued.otp.chars().count(), 8, "issued OTP must be 8 chars");

    // FIRST SIGN-IN: the new user redeems the OTP -> session + setup flag.
    let redeem: OtpRedeemResponse = post_json(
        service.clone(),
        "/api/v1/auth/otp/redeem",
        None,
        json!({ "otp": issued.otp }),
        StatusCode::OK,
    )
    .await;
    assert!(
        redeem.requires_passkey_setup,
        "a zero-passkey user must be flagged for passkey setup"
    );
    assert!(
        !redeem.access_token.is_empty()
            && redeem
                .refresh_token
                .as_ref()
                .is_some_and(|token| !token.is_empty()),
        "an OTP redeem is a first sign-in: it must mint a full session (access + refresh tokens)"
    );

    // INITIAL SETTINGS: the OTP-signed-in user enrolls a passkey via the
    // authenticated register path (no bootstrap token).
    let mut authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let credential_id = enroll_passkey(&service, &mut authenticator, &redeem.access_token).await;

    // A second OTP redeem is rejected: single-use.
    let replay = post_raw(
        service.clone(),
        "/api/v1/auth/otp/redeem",
        None,
        json!({ "otp": issued.otp }),
    )
    .await;
    assert_eq!(replay.status(), StatusCode::UNAUTHORIZED);

    // USERNAMELESS SIGN-IN: no user_id, discoverable assertion -> token pair.
    let first_tokens = usernameless_login(&service, &mut authenticator, &credential_id).await;
    let work_order: Value = post_json(
        service.clone(),
        "/api/work-orders",
        Some(&first_tokens.access_token),
        json!({
            "branch_id": branch_id,
            "management_no": "#290",
            "symptom": "Hydraulic oil leak"
        }),
        StatusCode::CREATED,
    )
    .await;
    assert_eq!(work_order["status"], "RECEIVED");

    // Refresh rotation + reuse-detection still holds (mobile/body transport).
    let first_refresh = first_tokens
        .refresh_token
        .clone()
        .expect("body-transport login must return a refresh token");
    let rotated: TokenPairResponse = post_json(
        service.clone(),
        "/api/v1/auth/token/refresh",
        None,
        json!({ "refresh_token": first_refresh }),
        StatusCode::OK,
    )
    .await;
    assert_ne!(rotated.refresh_token, first_tokens.refresh_token);

    let reuse = post_raw(
        service.clone(),
        "/api/v1/auth/token/refresh",
        None,
        json!({ "refresh_token": first_refresh }),
    )
    .await;
    assert_eq!(reuse.status(), StatusCode::UNAUTHORIZED);

    assert_audit_count(&pool, "auth.otp.signin", 2).await; // admin + new user
    assert_audit_count(&pool, "auth.login", 1).await; // usernameless login
}

#[sqlx::test(migrations = false)]
async fn mobile_bound_step_up_start_gates_mobile_approval_and_poll_vote(pool: PgPool) {
    prepare_http_database(&pool).await;
    let signing_key = SigningKey::random(&mut OsRng);
    let private_key_pem = signing_key.to_pkcs8_pem(LineEnding::LF).unwrap();
    let public_key_pem = signing_key
        .verifying_key()
        .to_public_key_pem(LineEnding::LF)
        .unwrap();
    let branch_id = seed_branch(&pool, "Mobile Step-Up Region", "Mobile Step-Up Branch").await;
    let admin_id = seed_user_with_branch(
        &pool,
        "Mobile Step-Up Admin",
        "010-4100-0000",
        "ADMIN",
        branch_id,
    )
    .await;
    let executive_id = seed_user_with_branch(
        &pool,
        "Mobile Step-Up Executive",
        "010-4100-0001",
        "EXECUTIVE",
        branch_id,
    )
    .await;
    let mechanic_id = seed_user_with_branch(
        &pool,
        "Mobile Step-Up Mechanic",
        "010-4100-0002",
        "MECHANIC",
        branch_id,
    )
    .await;
    let receptionist_id = seed_user_with_branch(
        &pool,
        "Mobile Step-Up Reception",
        "010-4100-0003",
        "RECEPTIONIST",
        branch_id,
    )
    .await;
    seed_equipment(&pool, branch_id, "4100").await;
    let work_order_id = seed_mobile_step_up_work_order(
        &pool,
        branch_id,
        receptionist_id,
        mechanic_id,
        admin_id,
        executive_id,
    )
    .await;
    let (poll_id, poll_option_id) = seed_mobile_step_up_poll(&pool, admin_id).await;

    let service = build_router(
        app_state(
            pool.clone(),
            private_key_pem.to_string(),
            public_key_pem.clone(),
        )
        .await
        .unwrap(),
    );
    let admin_access = admin_session_via_otp(&service, &pool, admin_id).await;
    let mut authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let credential_id = enroll_passkey(&service, &mut authenticator, &admin_access).await;

    let approval_path = format!("/api/v1/mobile/work-orders/{work_order_id}/approve");
    let missing_approval = post_raw(
        service.clone(),
        &approval_path,
        Some(&admin_access),
        json!({ "comment": "missing step-up must not mutate" }),
    )
    .await;
    assert_eq!(missing_approval.status(), StatusCode::PRECONDITION_REQUIRED);
    assert_eq!(
        body_json(missing_approval).await["error"]["code"],
        "passkey_step_up_required"
    );
    assert_work_order_status(&pool, work_order_id, "REPORT_SUBMITTED").await;
    assert_audit_count(&pool, "work_order.approve", 0).await;

    let missing_replay_attempt_start = post_raw(
        service.clone(),
        "/api/v1/auth/passkey/step-up/start",
        Some(&admin_access),
        json!({
            "binding": {
                "action_kind": "APPROVAL_DECISION",
                "object_id": work_order_id,
                "reason_key": "operations_passkey_approval_decision"
            }
        }),
    )
    .await;
    assert_eq!(
        missing_replay_attempt_start.status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );

    let invalid_replay_attempt_start = post_raw(
        service.clone(),
        "/api/v1/auth/passkey/step-up/start",
        Some(&admin_access),
        json!({
            "binding": {
                "action_kind": "APPROVAL_DECISION",
                "object_id": work_order_id,
                "reason_key": "operations_passkey_approval_decision",
                "replay_attempt": 0
            }
        }),
    )
    .await;
    assert_eq!(
        invalid_replay_attempt_start.status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );

    let approval_binding = json!({
        "action_kind": "APPROVAL_DECISION",
        "object_id": work_order_id,
        "reason_key": "operations_passkey_approval_decision",
        "replay_attempt": null
    });
    let approval_step_up = start_mobile_step_up_assertion(
        &service,
        &mut authenticator,
        &credential_id,
        &admin_access,
        approval_binding.clone(),
    )
    .await;
    assert_persisted_mobile_step_up_binding(&pool, &approval_step_up).await;

    let executive_access = admin_session_via_otp(&service, &pool, executive_id).await;
    let mut executive_authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let executive_credential_id =
        enroll_passkey(&service, &mut executive_authenticator, &executive_access).await;
    let wrong_user_step_up = start_mobile_step_up_assertion(
        &service,
        &mut executive_authenticator,
        &executive_credential_id,
        &executive_access,
        approval_binding.clone(),
    )
    .await;

    let wrong_user_approval = post_raw(
        service.clone(),
        &approval_path,
        Some(&admin_access),
        json!({
            "comment": "wrong user step-up must not mutate",
            "step_up": wrong_user_step_up
        }),
    )
    .await;
    assert_eq!(wrong_user_approval.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        body_json(wrong_user_approval).await["error"]["code"],
        "passkey_step_up_failed"
    );
    assert_work_order_status(&pool, work_order_id, "REPORT_SUBMITTED").await;
    assert_audit_count(&pool, "work_order.approve", 0).await;

    let mismatched_approval = post_raw(
        service.clone(),
        &approval_path,
        Some(&admin_access),
        json!({
            "comment": "mismatched step-up must not mutate",
            "step_up": {
                "binding": {
                    "action_kind": "APPROVAL_DECISION",
                    "object_id": Uuid::new_v4(),
                    "reason_key": "operations_passkey_approval_decision",
                    "replay_attempt": null
                },
                "assertion": approval_step_up["assertion"].clone()
            }
        }),
    )
    .await;
    assert_eq!(mismatched_approval.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        body_json(mismatched_approval).await["error"]["code"],
        "passkey_step_up_binding_mismatch"
    );
    assert_work_order_status(&pool, work_order_id, "REPORT_SUBMITTED").await;
    assert_audit_count(&pool, "work_order.approve", 0).await;

    let approved = post_json::<Value>(
        service.clone(),
        &approval_path,
        Some(&admin_access),
        json!({
            "comment": "bound step-up approved",
            "step_up": approval_step_up
        }),
        StatusCode::OK,
    )
    .await;
    assert_eq!(approved["status"], "ADMIN_REVIEW");
    assert_audit_count(&pool, "work_order.approve", 1).await;

    let replayed_approval = post_raw(
        service.clone(),
        &approval_path,
        Some(&admin_access),
        json!({
            "comment": "replayed step-up must not mutate",
            "step_up": approval_step_up
        }),
    )
    .await;
    assert_eq!(replayed_approval.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        body_json(replayed_approval).await["error"]["code"],
        "passkey_step_up_failed"
    );
    assert_work_order_status(&pool, work_order_id, "ADMIN_REVIEW").await;
    assert_audit_count(&pool, "work_order.approve", 1).await;

    let poll_path = format!("/api/v1/mobile/collaboration/polls/{poll_id}/vote");
    let missing_poll = post_raw(
        service.clone(),
        &poll_path,
        Some(&admin_access),
        json!({ "selected_option_ids": [poll_option_id] }),
    )
    .await;
    assert_eq!(missing_poll.status(), StatusCode::PRECONDITION_REQUIRED);
    assert_eq!(
        body_json(missing_poll).await["error"]["code"],
        "passkey_step_up_required"
    );
    assert_poll_vote_count(&pool, poll_id, 0).await;
    assert_audit_count(&pool, "collaboration.poll.vote", 0).await;

    let poll_binding = json!({
        "action_kind": "POLL_VOTE",
        "object_id": poll_id,
        "reason_key": "operations_passkey_poll_vote",
        "replay_attempt": null
    });
    let poll_step_up = start_mobile_step_up_assertion(
        &service,
        &mut authenticator,
        &credential_id,
        &admin_access,
        poll_binding.clone(),
    )
    .await;
    assert_persisted_mobile_step_up_binding(&pool, &poll_step_up).await;

    let voted = post_json::<Value>(
        service.clone(),
        &poll_path,
        Some(&admin_access),
        json!({
            "selected_option_ids": [poll_option_id],
            "step_up": poll_step_up
        }),
        StatusCode::OK,
    )
    .await;
    assert_eq!(voted["my_vote"]["submitted"], true);
    assert_poll_vote_count(&pool, poll_id, 1).await;
    assert_audit_count(&pool, "collaboration.poll.vote", 1).await;

    let replayed_poll = post_raw(
        service.clone(),
        &poll_path,
        Some(&admin_access),
        json!({
            "selected_option_ids": [poll_option_id],
            "step_up": poll_step_up
        }),
    )
    .await;
    assert_eq!(replayed_poll.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        body_json(replayed_poll).await["error"]["code"],
        "passkey_step_up_failed"
    );
    assert_poll_vote_count(&pool, poll_id, 1).await;
    assert_audit_count(&pool, "collaboration.poll.vote", 1).await;

    let replay_poll_binding = json!({
        "action_kind": "POLL_VOTE",
        "object_id": poll_id,
        "reason_key": "operations_passkey_poll_vote",
        "replay_attempt": 1
    });
    let replay_poll_step_up = start_mobile_step_up_assertion(
        &service,
        &mut authenticator,
        &credential_id,
        &admin_access,
        replay_poll_binding,
    )
    .await;
    assert_persisted_mobile_step_up_binding(&pool, &replay_poll_step_up).await;
    let replay_attempt_voted = post_json::<Value>(
        service.clone(),
        &poll_path,
        Some(&admin_access),
        json!({
            "selected_option_ids": [poll_option_id],
            "step_up": replay_poll_step_up
        }),
        StatusCode::OK,
    )
    .await;
    assert_eq!(replay_attempt_voted["my_vote"]["submitted"], true);
    assert_poll_vote_count(&pool, poll_id, 1).await;
    assert_audit_count(&pool, "collaboration.poll.vote", 2).await;
}

#[sqlx::test(migrations = false)]
async fn financial_purchase_sensitive_actions_require_fresh_passkey_step_up(pool: PgPool) {
    prepare_http_database(&pool).await;
    let signing_key = SigningKey::random(&mut OsRng);
    let private_key_pem = signing_key.to_pkcs8_pem(LineEnding::LF).unwrap();
    let public_key_pem = signing_key
        .verifying_key()
        .to_public_key_pem(LineEnding::LF)
        .unwrap();
    let branch_id = seed_branch(
        &pool,
        "Financial Step-Up Region",
        "Financial Step-Up Branch",
    )
    .await;
    let requester_id = seed_user_with_branch(
        &pool,
        "Financial Step-Up Requester",
        "010-4200-0000",
        "MECHANIC",
        branch_id,
    )
    .await;
    let receptionist_id = seed_user_with_branch(
        &pool,
        "Financial Step-Up Reception",
        "010-4200-0001",
        "RECEPTIONIST",
        branch_id,
    )
    .await;
    let admin_id = seed_user_with_branch(
        &pool,
        "Financial Step-Up Admin",
        "010-4200-0002",
        "ADMIN",
        branch_id,
    )
    .await;
    let executive_id = seed_user_with_branch(
        &pool,
        "Financial Step-Up Executive",
        "010-4200-0003",
        "EXECUTIVE",
        branch_id,
    )
    .await;
    let fixture =
        seed_financial_step_up_fixture(&pool, branch_id, requester_id, receptionist_id, admin_id)
            .await;

    let service = build_router(
        app_state(
            pool.clone(),
            private_key_pem.to_string(),
            public_key_pem.clone(),
        )
        .await
        .unwrap(),
    );
    let admin_access = admin_session_via_otp(&service, &pool, admin_id).await;
    let mut admin_authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let admin_credential_id =
        enroll_passkey(&service, &mut admin_authenticator, &admin_access).await;
    let receptionist_access = admin_session_via_otp(&service, &pool, receptionist_id).await;
    let mut receptionist_authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let receptionist_credential_id = enroll_passkey(
        &service,
        &mut receptionist_authenticator,
        &receptionist_access,
    )
    .await;
    let executive_access = admin_session_via_otp(&service, &pool, executive_id).await;
    let mut executive_authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let executive_credential_id =
        enroll_passkey(&service, &mut executive_authenticator, &executive_access).await;

    let admin_approve_purchase = submitted_financial_purchase(&pool, fixture, 900_000).await;
    let admin_approve_path =
        format!("/api/v1/financial/purchase-requests/{admin_approve_purchase}/approve-admin");
    assert_financial_step_up_denied(
        service.clone(),
        &pool,
        &admin_access,
        &admin_approve_path,
        json!({}),
        admin_approve_purchase,
        "REQUEST_SUBMITTED",
        "purchase.admin.approve",
        "purchase.admin.approve",
        StatusCode::PRECONDITION_REQUIRED,
        "passkey_step_up_required",
        "missing",
    )
    .await;
    let invalid_admin_step_up = start_step_up_assertion(
        &service,
        &mut executive_authenticator,
        &executive_credential_id,
    )
    .await;
    assert_financial_step_up_denied(
        service.clone(),
        &pool,
        &admin_access,
        &admin_approve_path,
        json!({ "step_up": invalid_admin_step_up }),
        admin_approve_purchase,
        "REQUEST_SUBMITTED",
        "purchase.admin.approve",
        "purchase.admin.approve",
        StatusCode::UNAUTHORIZED,
        "passkey_step_up_failed",
        "invalid_or_expired",
    )
    .await;
    let valid_admin_step_up =
        start_step_up_assertion(&service, &mut admin_authenticator, &admin_credential_id).await;
    let admin_approved = post_json::<Value>(
        service.clone(),
        &admin_approve_path,
        Some(&admin_access),
        json!({ "step_up": valid_admin_step_up }),
        StatusCode::OK,
    )
    .await;
    assert_eq!(admin_approved["status"], "ADMIN_APPROVED");
    assert_purchase_status(&pool, admin_approve_purchase, "ADMIN_APPROVED").await;
    assert_financial_audit_count(&pool, "purchase.admin.approve", admin_approve_purchase, 1).await;

    let prepare_purchase = admin_approved_financial_purchase(&pool, fixture, 3_000_000).await;
    let prepare_path =
        format!("/api/v1/financial/purchase-requests/{prepare_purchase}/prepare-expenditure");
    assert_financial_step_up_denied(
        service.clone(),
        &pool,
        &admin_access,
        &prepare_path,
        json!({ "expenditure_no": "EXP-MISSING-001" }),
        prepare_purchase,
        "ADMIN_APPROVED",
        "purchase.expenditure.prepare",
        "purchase.expenditure.prepare",
        StatusCode::PRECONDITION_REQUIRED,
        "passkey_step_up_required",
        "missing",
    )
    .await;
    let invalid_prepare_step_up = start_step_up_assertion(
        &service,
        &mut executive_authenticator,
        &executive_credential_id,
    )
    .await;
    assert_financial_step_up_denied(
        service.clone(),
        &pool,
        &admin_access,
        &prepare_path,
        json!({ "expenditure_no": "EXP-INVALID-001", "step_up": invalid_prepare_step_up }),
        prepare_purchase,
        "ADMIN_APPROVED",
        "purchase.expenditure.prepare",
        "purchase.expenditure.prepare",
        StatusCode::UNAUTHORIZED,
        "passkey_step_up_failed",
        "invalid_or_expired",
    )
    .await;
    let valid_prepare_step_up =
        start_step_up_assertion(&service, &mut admin_authenticator, &admin_credential_id).await;
    let prepared = post_json::<Value>(
        service.clone(),
        &prepare_path,
        Some(&admin_access),
        json!({ "expenditure_no": "EXP-VALID-001", "step_up": valid_prepare_step_up }),
        StatusCode::OK,
    )
    .await;
    assert_eq!(prepared["status"], "EXECUTIVE_PENDING");
    assert_purchase_status(&pool, prepare_purchase, "EXECUTIVE_PENDING").await;
    assert_financial_audit_count(&pool, "purchase.expenditure.prepare", prepare_purchase, 1).await;

    let executive_purchase = executive_pending_financial_purchase(&pool, fixture).await;
    let executive_path =
        format!("/api/v1/financial/purchase-requests/{executive_purchase}/approve-executive");
    assert_financial_step_up_denied(
        service.clone(),
        &pool,
        &executive_access,
        &executive_path,
        json!({}),
        executive_purchase,
        "EXECUTIVE_PENDING",
        "purchase.executive.approve",
        "purchase.executive.approve",
        StatusCode::PRECONDITION_REQUIRED,
        "passkey_step_up_required",
        "missing",
    )
    .await;
    let invalid_executive_step_up =
        start_step_up_assertion(&service, &mut admin_authenticator, &admin_credential_id).await;
    assert_financial_step_up_denied(
        service.clone(),
        &pool,
        &executive_access,
        &executive_path,
        json!({ "step_up": invalid_executive_step_up }),
        executive_purchase,
        "EXECUTIVE_PENDING",
        "purchase.executive.approve",
        "purchase.executive.approve",
        StatusCode::UNAUTHORIZED,
        "passkey_step_up_failed",
        "invalid_or_expired",
    )
    .await;
    let valid_executive_step_up = start_step_up_assertion(
        &service,
        &mut executive_authenticator,
        &executive_credential_id,
    )
    .await;
    let executive_approved = post_json::<Value>(
        service.clone(),
        &executive_path,
        Some(&executive_access),
        json!({ "step_up": valid_executive_step_up }),
        StatusCode::OK,
    )
    .await;
    assert_eq!(executive_approved["status"], "READY_TO_EXECUTE");
    assert_purchase_status(&pool, executive_purchase, "READY_TO_EXECUTE").await;
    assert_financial_audit_count(&pool, "purchase.executive.approve", executive_purchase, 1).await;

    let reject_purchase = submitted_financial_purchase(&pool, fixture, 900_000).await;
    let reject_path = format!("/api/v1/financial/purchase-requests/{reject_purchase}/reject");
    assert_financial_step_up_denied(
        service.clone(),
        &pool,
        &admin_access,
        &reject_path,
        json!({ "memo": "missing proof" }),
        reject_purchase,
        "REQUEST_SUBMITTED",
        "purchase.reject",
        "purchase.reject",
        StatusCode::PRECONDITION_REQUIRED,
        "passkey_step_up_required",
        "missing",
    )
    .await;
    let invalid_reject_step_up = start_step_up_assertion(
        &service,
        &mut executive_authenticator,
        &executive_credential_id,
    )
    .await;
    assert_financial_step_up_denied(
        service.clone(),
        &pool,
        &admin_access,
        &reject_path,
        json!({ "memo": "invalid proof", "step_up": invalid_reject_step_up }),
        reject_purchase,
        "REQUEST_SUBMITTED",
        "purchase.reject",
        "purchase.reject",
        StatusCode::UNAUTHORIZED,
        "passkey_step_up_failed",
        "invalid_or_expired",
    )
    .await;
    let valid_reject_step_up =
        start_step_up_assertion(&service, &mut admin_authenticator, &admin_credential_id).await;
    let rejected = post_json::<Value>(
        service.clone(),
        &reject_path,
        Some(&admin_access),
        json!({ "memo": "valid rejection", "step_up": valid_reject_step_up }),
        StatusCode::OK,
    )
    .await;
    assert_eq!(rejected["status"], "REJECTED");
    assert_purchase_status(&pool, reject_purchase, "REJECTED").await;
    assert_financial_audit_count(&pool, "purchase.reject", reject_purchase, 1).await;

    let execute_purchase = ready_to_execute_financial_purchase(&pool, fixture).await;
    let execute_path = format!("/api/v1/financial/purchase-requests/{execute_purchase}/execute");
    assert_financial_step_up_denied(
        service.clone(),
        &pool,
        &receptionist_access,
        &execute_path,
        json!({}),
        execute_purchase,
        "READY_TO_EXECUTE",
        "purchase.execute",
        "purchase.execute",
        StatusCode::PRECONDITION_REQUIRED,
        "passkey_step_up_required",
        "missing",
    )
    .await;
    let invalid_execute_step_up =
        start_step_up_assertion(&service, &mut admin_authenticator, &admin_credential_id).await;
    assert_financial_step_up_denied(
        service.clone(),
        &pool,
        &receptionist_access,
        &execute_path,
        json!({ "step_up": invalid_execute_step_up }),
        execute_purchase,
        "READY_TO_EXECUTE",
        "purchase.execute",
        "purchase.execute",
        StatusCode::UNAUTHORIZED,
        "passkey_step_up_failed",
        "invalid_or_expired",
    )
    .await;
    let valid_execute_step_up = start_step_up_assertion(
        &service,
        &mut receptionist_authenticator,
        &receptionist_credential_id,
    )
    .await;
    let executed = post_json::<Value>(
        service.clone(),
        &execute_path,
        Some(&receptionist_access),
        json!({ "step_up": valid_execute_step_up }),
        StatusCode::OK,
    )
    .await;
    assert_eq!(executed["status"], "EXECUTED");
    assert_purchase_status(&pool, execute_purchase, "EXECUTED").await;
    assert_financial_audit_count(&pool, "purchase.execute", execute_purchase, 1).await;
}

/// Regression for desktop/phone onboarding: a zero-passkey user can refresh the
/// OTP-minted session before enrollment completes. Refresh must keep carrying the
/// setup flag, otherwise a hard reload recreates a normal session and lets the
/// user into the app without registering a passkey.
#[sqlx::test(migrations = false)]
async fn refresh_keeps_zero_passkey_user_in_setup_mode_until_enrolled(pool: PgPool) {
    prepare_http_database(&pool).await;
    let signing_key = SigningKey::random(&mut OsRng);
    let private_key_pem = signing_key.to_pkcs8_pem(LineEnding::LF).unwrap();
    let public_key_pem = signing_key
        .verifying_key()
        .to_public_key_pem(LineEnding::LF)
        .unwrap();
    let branch_id = seed_branch(&pool, "Refresh Setup Region", "Refresh Setup Branch").await;
    let user_id = seed_user_with_branch(
        &pool,
        "Refresh Setup User",
        "010-4090-0001",
        "MECHANIC",
        branch_id,
    )
    .await;
    let service = build_router(
        app_state(
            pool.clone(),
            private_key_pem.to_string(),
            public_key_pem.clone(),
        )
        .await
        .unwrap(),
    );

    let issue = BootstrapCredentialStore
        .issue_for_zero_credential_user(
            &pool,
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
    assert!(redeem.requires_passkey_setup);

    let refresh_token = redeem
        .refresh_token
        .clone()
        .expect("body transport must return refresh token");
    let refreshed: TokenPairResponse = post_json(
        service.clone(),
        "/api/v1/auth/token/refresh",
        None,
        json!({ "refresh_token": refresh_token }),
        StatusCode::OK,
    )
    .await;
    assert!(
        refreshed.requires_passkey_setup,
        "refresh before enrollment must keep the client locked on passkey setup"
    );

    let mut authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
    enroll_passkey(&service, &mut authenticator, &refreshed.access_token).await;
    let post_enrollment_refresh = refreshed
        .refresh_token
        .clone()
        .expect("rotated body refresh token must be returned");
    let refreshed_after_enrollment: TokenPairResponse = post_json(
        service.clone(),
        "/api/v1/auth/token/refresh",
        None,
        json!({ "refresh_token": post_enrollment_refresh }),
        StatusCode::OK,
    )
    .await;
    assert!(
        !refreshed_after_enrollment.requires_passkey_setup,
        "refresh after successful passkey enrollment should clear the setup flag"
    );
}

/// `/device-login/approve-session` is only for the first-enrollment QR path,
/// where the desktop handoff is pinned to the OTP user/org. Generic desktop QR
/// logins must still require a fresh WebAuthn assertion through
/// `/device-login/approve`; a normal bearer session is not enough.
#[sqlx::test(migrations = false)]
async fn approve_session_rejects_generic_desktop_handoff_without_target(pool: PgPool) {
    prepare_http_database(&pool).await;
    let signing_key = SigningKey::random(&mut OsRng);
    let private_key_pem = signing_key.to_pkcs8_pem(LineEnding::LF).unwrap();
    let public_key_pem = signing_key
        .verifying_key()
        .to_public_key_pem(LineEnding::LF)
        .unwrap();
    let branch_id = seed_branch(&pool, "Device QR Region", "Device QR Branch").await;
    let user_id = seed_user_with_branch(
        &pool,
        "Device QR User",
        "010-4090-0002",
        "MECHANIC",
        branch_id,
    )
    .await;
    let service = build_router(
        app_state(
            pool.clone(),
            private_key_pem.to_string(),
            public_key_pem.clone(),
        )
        .await
        .unwrap(),
    );

    let issue = BootstrapCredentialStore
        .issue_for_zero_credential_user(
            &pool,
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
    let mut authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
    enroll_passkey(&service, &mut authenticator, &redeem.access_token).await;

    let handoff: DeviceLoginStartResponse = post_json(
        service.clone(),
        "/api/v1/auth/device-login/start",
        None,
        json!({}),
        StatusCode::OK,
    )
    .await;
    let approve_url = Url::parse(&handoff.approve_url).unwrap();
    let approve_token = approve_url
        .fragment()
        .and_then(|fragment| {
            url::form_urlencoded::parse(fragment.as_bytes())
                .find(|(key, _)| key == "desktop_approve")
                .map(|(_, value)| value.into_owned())
        })
        .expect("desktop approve token must be in the URL fragment");

    let response = post_raw(
        service.clone(),
        "/api/v1/auth/device-login/approve-session",
        Some(&redeem.access_token),
        json!({ "approve_token": approve_token }),
    )
    .await;
    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "generic desktop QR handoffs require a fresh passkey assertion"
    );
}

/// Initial passkey enrollment is gated on separate privacy/data-collection and
/// service-terms agreements. A freshly OTP-authenticated user can read the
/// required version, cannot start enrollment until both required boxes are true,
/// and can proceed after acceptance is recorded.
#[sqlx::test(migrations = false)]
async fn first_passkey_enrollment_requires_privacy_terms(pool: PgPool) {
    prepare_http_database(&pool).await;
    let signing_key = SigningKey::random(&mut OsRng);
    let private_key_pem = signing_key.to_pkcs8_pem(LineEnding::LF).unwrap();
    let public_key_pem = signing_key
        .verifying_key()
        .to_public_key_pem(LineEnding::LF)
        .unwrap();
    let branch_id = seed_branch(&pool, "Privacy Region", "Privacy Branch").await;
    let user_id = seed_user_with_branch(
        &pool,
        "Privacy User",
        "010-4050-0001",
        "MECHANIC",
        branch_id,
    )
    .await;
    let service = build_router(
        app_state(
            pool.clone(),
            private_key_pem.to_string(),
            public_key_pem.clone(),
        )
        .await
        .unwrap(),
    );

    let issue = BootstrapCredentialStore
        .issue_for_zero_credential_user(
            &pool,
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
    assert!(redeem.requires_passkey_setup);

    let initial_status: PrivacyConsentStatusResponse = post_json(
        service.clone(),
        "/api/v1/auth/privacy-consent/status",
        Some(&redeem.access_token),
        json!({}),
        StatusCode::OK,
    )
    .await;
    assert!(!initial_status.accepted);
    assert!(initial_status.accepted_at.is_none());

    let blocked = post_raw(
        service.clone(),
        "/api/v1/auth/passkey/register/start",
        Some(&redeem.access_token),
        json!({ "username": "privacy.user", "display_name": "Privacy User" }),
    )
    .await;
    assert_eq!(blocked.status(), StatusCode::FORBIDDEN);

    let bundled_or_partial_consent = post_raw(
        service.clone(),
        "/api/v1/auth/privacy-consent/accept",
        Some(&redeem.access_token),
        json!({
            "policy_version": initial_status.policy_version,
            "privacy_collection": true,
            "terms_of_service": false
        }),
    )
    .await;
    assert_eq!(bundled_or_partial_consent.status(), StatusCode::BAD_REQUEST);

    let accepted = accept_required_privacy_consent(&service, &redeem.access_token).await;
    assert!(accepted.accepted);
    assert!(accepted.accepted_at.is_some());

    let allowed: RegisterStartResponse = post_json(
        service.clone(),
        "/api/v1/auth/passkey/register/start",
        Some(&redeem.access_token),
        json!({ "username": "privacy.user", "display_name": "Privacy User" }),
        StatusCode::OK,
    )
    .await;
    assert_ne!(allowed.ceremony_id, Uuid::nil());
}

/// The one-time code is consumed on PASSKEY REGISTRATION, not on redeem. A redeem
/// only mints a session, so a failed/incomplete enrollment never burns the code —
/// the user can re-redeem (within the TTL) until a passkey actually sticks. Once a
/// passkey is registered the code is consumed atomically and can never be reused.
#[sqlx::test(migrations = false)]
async fn otp_is_consumed_on_passkey_registration_not_on_redeem(pool: PgPool) {
    prepare_http_database(&pool).await;
    let signing_key = SigningKey::random(&mut OsRng);
    let private_key_pem = signing_key.to_pkcs8_pem(LineEnding::LF).unwrap();
    let public_key_pem = signing_key
        .verifying_key()
        .to_public_key_pem(LineEnding::LF)
        .unwrap();
    let branch_id = seed_branch(&pool, "OTP Region", "OTP Branch").await;
    let admin_id =
        seed_user_with_branch(&pool, "Issuer Admin", "010-4100-0000", "ADMIN", branch_id).await;
    let new_user_id = seed_user_with_branch(
        &pool,
        "Pending User",
        "010-4100-0001",
        "MECHANIC",
        branch_id,
    )
    .await;
    let service = build_router(
        app_state(
            pool.clone(),
            private_key_pem.to_string(),
            public_key_pem.clone(),
        )
        .await
        .unwrap(),
    );

    let admin_access = admin_session_via_otp(&service, &pool, admin_id).await;
    let issued: AdminIssueOtpResponse = post_json(
        service.clone(),
        "/api/v1/auth/admin/otp/issue",
        Some(&admin_access),
        json!({ "user_id": new_user_id.as_uuid(), "branch_id": branch_id }),
        StatusCode::OK,
    )
    .await;

    // First redeem -> session, code NOT consumed.
    let first: OtpRedeemResponse = post_json(
        service.clone(),
        "/api/v1/auth/otp/redeem",
        None,
        json!({ "otp": issued.otp }),
        StatusCode::OK,
    )
    .await;
    assert!(first.requires_passkey_setup);

    // Re-redeem BEFORE enrolling a passkey -> STILL succeeds (a failed enrollment
    // must not lock the user out of their own code).
    let second: OtpRedeemResponse = post_json(
        service.clone(),
        "/api/v1/auth/otp/redeem",
        None,
        json!({ "otp": issued.otp }),
        StatusCode::OK,
    )
    .await;
    assert!(
        second.requires_passkey_setup,
        "the code must remain redeemable until a passkey is actually registered"
    );

    // Enroll a passkey from the session -> consumes the code atomically with the
    // passkey insert.
    let mut authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
    enroll_passkey(&service, &mut authenticator, &second.access_token).await;

    // Now the code is dead: a further redeem is rejected.
    let after = post_raw(
        service.clone(),
        "/api/v1/auth/otp/redeem",
        None,
        json!({ "otp": issued.otp }),
    )
    .await;
    assert_eq!(
        after.status(),
        StatusCode::UNAUTHORIZED,
        "the code is consumed once a passkey is registered"
    );

    // DB: exactly one consumed credential for this user (consumed at enrollment).
    let consumed: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM auth_bootstrap_credentials \
         WHERE user_id = $1 AND consumed_at IS NOT NULL",
    )
    .bind(new_user_id.as_uuid())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(consumed, 1);
}

/// The admin issue-OTP endpoint is authz-gated: a non-admin session is forbidden.
#[sqlx::test(migrations = false)]
async fn admin_issue_otp_rejects_non_admin(pool: PgPool) {
    prepare_http_database(&pool).await;
    let signing_key = SigningKey::random(&mut OsRng);
    let private_key_pem = signing_key.to_pkcs8_pem(LineEnding::LF).unwrap();
    let public_key_pem = signing_key
        .verifying_key()
        .to_public_key_pem(LineEnding::LF)
        .unwrap();
    let branch_id = seed_branch(&pool, "AZ Region", "AZ Branch").await;
    let mechanic_id = seed_user_with_branch(
        &pool,
        "Plain Mechanic",
        "010-5000-0001",
        "MECHANIC",
        branch_id,
    )
    .await;
    let target_id =
        seed_user_with_branch(&pool, "Target User", "010-5000-0002", "MECHANIC", branch_id).await;

    let service = build_router(
        app_state(
            pool.clone(),
            private_key_pem.to_string(),
            public_key_pem.clone(),
        )
        .await
        .unwrap(),
    );

    // A mechanic signs in via OTP and tries to issue a code -> 403.
    let mechanic_access = admin_session_via_otp(&service, &pool, mechanic_id).await;
    let forbidden = post_raw(
        service.clone(),
        "/api/v1/auth/admin/otp/issue",
        Some(&mechanic_access),
        json!({ "user_id": target_id.as_uuid(), "branch_id": branch_id }),
    )
    .await;
    assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);

    // No bearer at all -> 401.
    let unauth = post_raw(
        service,
        "/api/v1/auth/admin/otp/issue",
        None,
        json!({ "user_id": target_id.as_uuid(), "branch_id": branch_id }),
    )
    .await;
    assert_eq!(unauth.status(), StatusCode::UNAUTHORIZED);
}

/// IDOR: a branch-A admin must NOT be able to mint a sign-in OTP for a user who
/// belongs only to branch B. Authorization is bound to the TARGET's real branch
/// scope, not the client-supplied branch_id.
#[sqlx::test(migrations = false)]
async fn admin_issue_otp_rejects_cross_branch_target(pool: PgPool) {
    prepare_http_database(&pool).await;
    let signing_key = SigningKey::random(&mut OsRng);
    let private_key_pem = signing_key.to_pkcs8_pem(LineEnding::LF).unwrap();
    let public_key_pem = signing_key
        .verifying_key()
        .to_public_key_pem(LineEnding::LF)
        .unwrap();
    let branch_a = seed_branch(&pool, "Region A", "Branch A").await;
    let branch_b = seed_branch(&pool, "Region B", "Branch B").await;
    let admin_a = seed_user_with_branch(&pool, "Admin A", "010-6000-0000", "ADMIN", branch_a).await;
    // The target belongs ONLY to branch B.
    let target_b =
        seed_user_with_branch(&pool, "User B", "010-6000-0001", "MECHANIC", branch_b).await;

    let service = build_router(
        app_state(
            pool.clone(),
            private_key_pem.to_string(),
            public_key_pem.clone(),
        )
        .await
        .unwrap(),
    );

    let admin_access = admin_session_via_otp(&service, &pool, admin_a).await;

    // Even when the admin lies and passes its own branch_a as branch_id, the
    // target's REAL scope (branch B) is what is authorized against -> 403.
    let forbidden = post_raw(
        service.clone(),
        "/api/v1/auth/admin/otp/issue",
        Some(&admin_access),
        json!({ "user_id": target_b.as_uuid(), "branch_id": branch_a }),
    )
    .await;
    assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);

    // Passing the target's real branch_b also fails — admin A has no authority there.
    let forbidden_real_branch = post_raw(
        service,
        "/api/v1/auth/admin/otp/issue",
        Some(&admin_access),
        json!({ "user_id": target_b.as_uuid(), "branch_id": branch_b }),
    )
    .await;
    assert_eq!(forbidden_real_branch.status(), StatusCode::FORBIDDEN);
}

/// IDOR: a branch admin must NOT be able to mint a sign-in OTP for a privileged
/// (SUPER_ADMIN or EXECUTIVE) target. Only a SUPER_ADMIN caller may do so.
#[sqlx::test(migrations = false)]
async fn admin_issue_otp_rejects_privileged_target(pool: PgPool) {
    prepare_http_database(&pool).await;
    let signing_key = SigningKey::random(&mut OsRng);
    let private_key_pem = signing_key.to_pkcs8_pem(LineEnding::LF).unwrap();
    let public_key_pem = signing_key
        .verifying_key()
        .to_public_key_pem(LineEnding::LF)
        .unwrap();
    let branch_id = seed_branch(&pool, "Priv Region", "Priv Branch").await;
    let admin_id =
        seed_user_with_branch(&pool, "Branch Admin", "010-6100-0000", "ADMIN", branch_id).await;
    // A SUPER_ADMIN target that also (incidentally) belongs to the admin's branch.
    let super_admin_target = seed_user_with_branch(
        &pool,
        "Super Admin Target",
        "010-6100-0001",
        "SUPER_ADMIN",
        branch_id,
    )
    .await;
    // An EXECUTIVE target in the same branch.
    let executive_target = seed_user_with_branch(
        &pool,
        "Executive Target",
        "010-6100-0002",
        "EXECUTIVE",
        branch_id,
    )
    .await;

    let service = build_router(
        app_state(
            pool.clone(),
            private_key_pem.to_string(),
            public_key_pem.clone(),
        )
        .await
        .unwrap(),
    );

    let admin_access = admin_session_via_otp(&service, &pool, admin_id).await;

    let super_admin_forbidden = post_raw(
        service.clone(),
        "/api/v1/auth/admin/otp/issue",
        Some(&admin_access),
        json!({ "user_id": super_admin_target.as_uuid(), "branch_id": branch_id }),
    )
    .await;
    assert_eq!(super_admin_forbidden.status(), StatusCode::FORBIDDEN);

    let executive_forbidden = post_raw(
        service,
        "/api/v1/auth/admin/otp/issue",
        Some(&admin_access),
        json!({ "user_id": executive_target.as_uuid(), "branch_id": branch_id }),
    )
    .await;
    assert_eq!(executive_forbidden.status(), StatusCode::FORBIDDEN);
}

/// The happy path still works: a branch admin issues a code for an in-branch
/// subordinate (a non-privileged user whose only branch is the admin's).
#[sqlx::test(migrations = false)]
async fn admin_issue_otp_allows_in_branch_subordinate(pool: PgPool) {
    prepare_http_database(&pool).await;
    let signing_key = SigningKey::random(&mut OsRng);
    let private_key_pem = signing_key.to_pkcs8_pem(LineEnding::LF).unwrap();
    let public_key_pem = signing_key
        .verifying_key()
        .to_public_key_pem(LineEnding::LF)
        .unwrap();
    let branch_id = seed_branch(&pool, "Sub Region", "Sub Branch").await;
    let admin_id =
        seed_user_with_branch(&pool, "Branch Admin", "010-6200-0000", "ADMIN", branch_id).await;
    let subordinate =
        seed_user_with_branch(&pool, "Subordinate", "010-6200-0001", "MECHANIC", branch_id).await;

    let service = build_router(
        app_state(
            pool.clone(),
            private_key_pem.to_string(),
            public_key_pem.clone(),
        )
        .await
        .unwrap(),
    );

    let admin_access = admin_session_via_otp(&service, &pool, admin_id).await;

    let issued: AdminIssueOtpResponse = post_json(
        service,
        "/api/v1/auth/admin/otp/issue",
        Some(&admin_access),
        json!({ "user_id": subordinate.as_uuid(), "branch_id": branch_id }),
        StatusCode::OK,
    )
    .await;
    assert_eq!(&issued.user_id, subordinate.as_uuid());
    assert_eq!(issued.otp.chars().count(), 8);
}

/// Lost-device recovery: a branch admin can reset an in-branch subordinate that
/// already has a passkey. The old passkey is revoked and the fresh one-time code
/// redeems so the user can enroll a replacement device.
#[sqlx::test(migrations = false)]
async fn admin_credential_reset_recovers_in_branch_subordinate(pool: PgPool) {
    prepare_http_database(&pool).await;
    let signing_key = SigningKey::random(&mut OsRng);
    let private_key_pem = signing_key.to_pkcs8_pem(LineEnding::LF).unwrap();
    let public_key_pem = signing_key
        .verifying_key()
        .to_public_key_pem(LineEnding::LF)
        .unwrap();
    let branch_id = seed_branch(&pool, "Reset Region", "Reset Branch").await;
    let admin_id =
        seed_user_with_branch(&pool, "Reset Admin", "010-6300-0000", "ADMIN", branch_id).await;
    let subordinate = seed_user_with_branch(
        &pool,
        "Locked Mechanic",
        "010-6300-0001",
        "MECHANIC",
        branch_id,
    )
    .await;
    let service = build_router(
        app_state(
            pool.clone(),
            private_key_pem.to_string(),
            public_key_pem.clone(),
        )
        .await
        .unwrap(),
    );

    let subordinate_access = admin_session_via_otp(&service, &pool, subordinate).await;
    let mut old_authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let old_credential_id =
        enroll_passkey(&service, &mut old_authenticator, &subordinate_access).await;

    let admin_access = admin_session_via_otp(&service, &pool, admin_id).await;
    let reset: AdminCredentialResetResponse = post_json(
        service.clone(),
        "/api/v1/auth/admin/credential-reset",
        Some(&admin_access),
        json!({ "user_id": subordinate.as_uuid() }),
        StatusCode::OK,
    )
    .await;
    assert_eq!(&reset.user_id, subordinate.as_uuid());
    assert_eq!(reset.otp.chars().count(), 8);

    let remaining_passkeys: i64 =
        sqlx::query_scalar("SELECT count(*) FROM auth_webauthn_credentials WHERE user_id = $1")
            .bind(subordinate.as_uuid())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(remaining_passkeys, 0);

    let login_start: LoginStartResponse = post_raw(
        service.clone(),
        "/api/v1/auth/passkey/login/start",
        None,
        json!({}),
    )
    .await
    .into_json(StatusCode::OK)
    .await;
    let challenge = inject_allow_credential(login_start.challenge, &old_credential_id);
    let assertion = old_authenticator
        .do_authentication(Url::parse(TEST_ORIGIN).unwrap(), challenge)
        .unwrap();
    let old_passkey_login = post_raw(
        service.clone(),
        "/api/v1/auth/passkey/login/finish",
        None,
        json!({ "ceremony_id": login_start.ceremony_id, "credential": assertion }),
    )
    .await;
    assert_eq!(old_passkey_login.status(), StatusCode::UNAUTHORIZED);

    let recovered: OtpRedeemResponse = post_json(
        service,
        "/api/v1/auth/otp/redeem",
        None,
        json!({ "otp": reset.otp }),
        StatusCode::OK,
    )
    .await;
    assert!(recovered.requires_passkey_setup);
}

/// The DB-backed per-IP rate limiter's cap/window/reset behavior is covered
/// deterministically in `auth-rest`'s own unit test
/// (`rate_limit_trips_at_cap_and_resets_after_window`), which drives `now` as
/// a synthetic clock instead of racing this HTTP-level test's real
/// round-trips against the wall clock's minute boundary — that race was the
/// CI flake (twelve sequential requests could straddle a minute and reset the
/// bucket before the cap tripped). This is the REAL-clock smoke: a handful of
/// requests through the actual HTTP path must behave normally, proving
/// `OffsetDateTime::now_utc()` still wires into `rate_limit` end-to-end, and
/// that per-IP buckets stay independent.
#[sqlx::test(migrations = false)]
async fn otp_redeem_rate_limit_wires_up_on_real_clock_path(pool: PgPool) {
    prepare_http_database(&pool).await;
    let signing_key = SigningKey::random(&mut OsRng);
    let private_key_pem = signing_key.to_pkcs8_pem(LineEnding::LF).unwrap();
    let public_key_pem = signing_key
        .verifying_key()
        .to_public_key_pem(LineEnding::LF)
        .unwrap();

    let service = build_router(
        app_state_with_trusted_proxy(
            pool.clone(),
            private_key_pem.to_string(),
            public_key_pem.clone(),
        )
        .await
        .unwrap(),
    );

    // The limiter buckets into a FIXED one-minute TUMBLING window -- see
    // `floor_to_window` in crates/platform/auth-rest/src/lib.rs, which floors to
    // `unix - unix.rem_euclid(60)` -- not a sliding one. Every request below
    // must therefore land inside the SAME window: if the loop straddles a minute
    // boundary the counter resets mid-loop and the eleventh request comes back
    // 401 instead of 429.
    //
    // That is not hypothetical. This test drives eleven real DB-backed requests,
    // so on a loaded runner it occupies a meaningful fraction of the window and
    // fails whenever it starts late in one. It did exactly that on run
    // 32225725163, on a pull request that changed nothing near auth.
    //
    // The sibling test named in this test's own doc comment
    // (`rate_limit_trips_at_cap_and_resets_after_window`) drives `now` directly
    // and so has no such exposure; this one exists to prove the REAL clock path
    // is wired, so it cannot inject a clock -- but it can decline to start near
    // a boundary.
    let into_window = OffsetDateTime::now_utc().unix_timestamp().rem_euclid(60);
    const WINDOW_SECS: i64 = 60;
    // Budget generously: the eleven requests each round-trip to PostgreSQL.
    const NEEDED_SECS: i64 = 30;
    if into_window > WINDOW_SECS - NEEDED_SECS {
        let wait = (WINDOW_SECS - into_window + 1) as u64;
        tokio::time::sleep(std::time::Duration::from_secs(wait)).await;
    }

    // Drive the real ingress boundary: the XFF identity is accepted only from
    // the configured trusted transport peer. The first identity exhausts its
    // own bucket while the second remains independently usable.
    for i in 0..10 {
        let response = post_raw_with_trusted_ip(
            service.clone(),
            "/api/v1/auth/otp/redeem",
            "203.0.113.7",
            json!({ "otp": "badcode1" }),
        )
        .await;
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "request {i} within the first identity's cap must not be rate limited"
        );
    }

    let exhausted = post_raw_with_trusted_ip(
        service.clone(),
        "/api/v1/auth/otp/redeem",
        "203.0.113.7",
        json!({ "otp": "badcode1" }),
    )
    .await;
    assert_eq!(
        exhausted.status(),
        StatusCode::TOO_MANY_REQUESTS,
        "the trusted ingress identity must select the first per-IP bucket"
    );

    let other_ip = post_raw_with_trusted_ip(
        service,
        "/api/v1/auth/otp/redeem",
        "203.0.113.99",
        json!({ "otp": "badcode2" }),
    )
    .await;
    assert_eq!(
        other_ip.status(),
        StatusCode::UNAUTHORIZED,
        "a second trusted ingress identity must have a separate per-IP bucket"
    );
}

/// WEB dual-transport: when `X-Auth-Transport: cookie` is present, an OTP redeem
/// sets the refresh token as an HttpOnly `console_refresh` cookie and OMITS it from
/// the JSON body, while the access token stays in the body. The cookie carries
/// the CSRF-safe attributes (HttpOnly, SameSite=Strict, Path=/api/v1/auth).
#[sqlx::test(migrations = false)]
async fn cookie_mode_redeem_sets_httponly_cookie_and_omits_body_refresh(pool: PgPool) {
    prepare_http_database(&pool).await;
    let signing_key = SigningKey::random(&mut OsRng);
    let private_key_pem = signing_key.to_pkcs8_pem(LineEnding::LF).unwrap();
    let public_key_pem = signing_key
        .verifying_key()
        .to_public_key_pem(LineEnding::LF)
        .unwrap();
    let branch_id = seed_branch(&pool, "Cookie Region", "Cookie Branch").await;
    let user_id =
        seed_user_with_branch(&pool, "Web User", "010-7000-0000", "MECHANIC", branch_id).await;
    let service = build_router(
        app_state(
            pool.clone(),
            private_key_pem.to_string(),
            public_key_pem.clone(),
        )
        .await
        .unwrap(),
    );

    let issue = BootstrapCredentialStore
        .issue_for_zero_credential_user(
            &pool,
            *user_id.as_uuid(),
            OrgId::knl(),
            OffsetDateTime::now_utc(),
            Duration::hours(24),
        )
        .await
        .unwrap();

    let response = post_cookie_mode(
        service,
        "/api/v1/auth/otp/redeem",
        None,
        json!({ "otp": issue.token.as_str() }),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);

    let set_cookie = console_refresh_set_cookie(&response)
        .expect("cookie-mode redeem must set an console_refresh cookie");
    assert!(set_cookie.contains("HttpOnly"), "{set_cookie}");
    assert!(set_cookie.contains("SameSite=Strict"), "{set_cookie}");
    assert!(set_cookie.contains("Path=/api/v1/auth"), "{set_cookie}");
    assert!(set_cookie.contains("Max-Age="), "{set_cookie}");
    // Local-dev config leaves CONSOLE_COOKIE_SECURE at its default (true) in this
    // test harness, so Secure must be present.
    assert!(set_cookie.contains("Secure"), "{set_cookie}");
    assert!(
        !cookie_token(&set_cookie).is_empty(),
        "cookie must carry the refresh token value"
    );

    let body = body_json(response).await;
    assert!(
        !body["access_token"].as_str().unwrap().is_empty(),
        "access token must always be in the body"
    );
    assert!(
        body["refresh_token"].is_null(),
        "cookie mode must NOT leak the refresh token into the JSON body, got {body}"
    );
}

/// WEB dual-transport: passkey login finish in cookie mode sets the cookie and
/// nulls the body refresh token; the cookie value then authorizes a refresh that
/// reads the token from the cookie (no body token) and rotates the cookie.
#[sqlx::test(migrations = false)]
async fn cookie_mode_login_then_refresh_reads_and_rotates_cookie(pool: PgPool) {
    prepare_http_database(&pool).await;
    let signing_key = SigningKey::random(&mut OsRng);
    let private_key_pem = signing_key.to_pkcs8_pem(LineEnding::LF).unwrap();
    let public_key_pem = signing_key
        .verifying_key()
        .to_public_key_pem(LineEnding::LF)
        .unwrap();
    let branch_id = seed_branch(&pool, "Cookie Region", "Cookie Branch").await;
    let user_id =
        seed_user_with_branch(&pool, "Web User", "010-7100-0000", "MECHANIC", branch_id).await;
    let service = build_router(
        app_state(
            pool.clone(),
            private_key_pem.to_string(),
            public_key_pem.clone(),
        )
        .await
        .unwrap(),
    );

    // First sign-in via OTP (cookie mode) then enroll a passkey so we can do a
    // real cookie-mode passkey login.
    let issue = BootstrapCredentialStore
        .issue_for_zero_credential_user(
            &pool,
            *user_id.as_uuid(),
            OrgId::knl(),
            OffsetDateTime::now_utc(),
            Duration::hours(24),
        )
        .await
        .unwrap();
    let redeem = post_cookie_mode(
        service.clone(),
        "/api/v1/auth/otp/redeem",
        None,
        json!({ "otp": issue.token.as_str() }),
    )
    .await;
    assert_eq!(redeem.status(), StatusCode::OK);
    let access_token = body_json(redeem).await["access_token"]
        .as_str()
        .unwrap()
        .to_owned();
    let mut authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let credential_id = enroll_passkey(&service, &mut authenticator, &access_token).await;

    // Cookie-mode usernameless passkey login -> cookie set, body refresh null.
    let login = cookie_mode_usernameless_login(&service, &mut authenticator, &credential_id).await;
    let login_cookie = console_refresh_set_cookie(&login)
        .expect("cookie-mode login must set an console_refresh cookie");
    let cookie_value = cookie_token(&login_cookie).to_owned();
    let login_body = body_json(login).await;
    assert!(login_body["refresh_token"].is_null());

    // Refresh reading the token from the cookie (NO body token) rotates and sets
    // a fresh cookie whose value differs from the one presented.
    let refreshed = post_cookie_mode(
        service.clone(),
        "/api/v1/auth/token/refresh",
        Some(&cookie_value),
        json!({}),
    )
    .await;
    assert_eq!(refreshed.status(), StatusCode::OK);
    let rotated_cookie = console_refresh_set_cookie(&refreshed)
        .expect("cookie-mode refresh must rotate the console_refresh cookie");
    let rotated_value = cookie_token(&rotated_cookie).to_owned();
    assert_ne!(
        rotated_value, cookie_value,
        "the refresh token cookie must rotate on use"
    );
    assert!(body_json(refreshed).await["refresh_token"].is_null());

    // Logout in cookie mode clears the cookie (Max-Age=0) and revokes the family,
    // so the rotated cookie can no longer refresh. (Reuse-detection of the old
    // pre-rotation token is covered by the body-transport end-to-end test; here we
    // logout with the LIVE rotated cookie, the one a browser would actually hold.)
    let logout = post_cookie_mode(
        service.clone(),
        "/api/v1/auth/logout",
        Some(&rotated_value),
        json!({}),
    )
    .await;
    assert_eq!(logout.status(), StatusCode::NO_CONTENT);
    let clear_cookie = console_refresh_set_cookie(&logout)
        .expect("logout must emit a clearing console_refresh cookie");
    assert!(clear_cookie.contains("Max-Age=0"), "{clear_cookie}");
    assert!(clear_cookie.contains("Path=/api/v1/auth"), "{clear_cookie}");

    let after_logout = post_cookie_mode(
        service,
        "/api/v1/auth/token/refresh",
        Some(&rotated_value),
        json!({}),
    )
    .await;
    assert_eq!(after_logout.status(), StatusCode::UNAUTHORIZED);
}

/// Browser hard navigations drop the in-memory access token and rebuild the
/// session from the HttpOnly refresh cookie on each document load. That normal
/// pattern must have a wider refresh budget than OTP/passkey credential
/// submission, while still retaining a bounded per-device refresh limiter.
#[sqlx::test(migrations = false)]
async fn cookie_mode_refresh_allows_rapid_navigation_burst_with_device_id(pool: PgPool) {
    prepare_http_database(&pool).await;
    let signing_key = SigningKey::random(&mut OsRng);
    let private_key_pem = signing_key.to_pkcs8_pem(LineEnding::LF).unwrap();
    let public_key_pem = signing_key
        .verifying_key()
        .to_public_key_pem(LineEnding::LF)
        .unwrap();
    let branch_id = seed_branch(&pool, "Refresh Nav Region", "Refresh Nav Branch").await;
    let user_id = seed_user_with_branch(
        &pool,
        "Refresh Nav User",
        "010-7500-0000",
        "SUPER_ADMIN",
        branch_id,
    )
    .await;
    let service = build_router(
        app_state(
            pool.clone(),
            private_key_pem.to_string(),
            public_key_pem.clone(),
        )
        .await
        .unwrap(),
    );

    let issue = BootstrapCredentialStore
        .issue_for_zero_credential_user(
            &pool,
            *user_id.as_uuid(),
            OrgId::knl(),
            OffsetDateTime::now_utc(),
            Duration::hours(24),
        )
        .await
        .unwrap();
    let redeem = post_cookie_mode(
        service.clone(),
        "/api/v1/auth/otp/redeem",
        None,
        json!({ "otp": issue.token.as_str() }),
    )
    .await;
    assert_eq!(redeem.status(), StatusCode::OK);
    let access_token = body_json(redeem).await["access_token"]
        .as_str()
        .unwrap()
        .to_owned();
    let mut authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let credential_id = enroll_passkey(&service, &mut authenticator, &access_token).await;

    let login = cookie_mode_usernameless_login(&service, &mut authenticator, &credential_id).await;
    let login_cookie = console_refresh_set_cookie(&login)
        .expect("cookie-mode login must set an console_refresh cookie");
    let mut cookie_value = cookie_token(&login_cookie).to_owned();

    for attempt in 1..=12 {
        let refreshed = post_cookie_mode_with_device_id(
            service.clone(),
            "/api/v1/auth/token/refresh",
            Some(&cookie_value),
            "browser-nav-device-01",
            json!({}),
        )
        .await;
        assert_eq!(
            refreshed.status(),
            StatusCode::OK,
            "refresh attempt {attempt} should stay within the normal browser navigation budget"
        );
        let rotated_cookie = console_refresh_set_cookie(&refreshed)
            .expect("cookie-mode refresh must rotate the console_refresh cookie");
        cookie_value = cookie_token(&rotated_cookie).to_owned();
        assert!(body_json(refreshed).await["refresh_token"].is_null());
    }
}

/// MOBILE (no transport header) is unchanged: refresh and logout read the token
/// from the request BODY, the response carries the refresh token in the body, and
/// NO Set-Cookie header is emitted.
#[sqlx::test(migrations = false)]
async fn body_mode_without_header_is_unchanged_and_sets_no_cookie(pool: PgPool) {
    prepare_http_database(&pool).await;
    let signing_key = SigningKey::random(&mut OsRng);
    let private_key_pem = signing_key.to_pkcs8_pem(LineEnding::LF).unwrap();
    let public_key_pem = signing_key
        .verifying_key()
        .to_public_key_pem(LineEnding::LF)
        .unwrap();
    let branch_id = seed_branch(&pool, "Mobile Region", "Mobile Branch").await;
    let user_id =
        seed_user_with_branch(&pool, "Mobile User", "010-7200-0000", "MECHANIC", branch_id).await;
    let service = build_router(
        app_state(
            pool.clone(),
            private_key_pem.to_string(),
            public_key_pem.clone(),
        )
        .await
        .unwrap(),
    );

    let issue = BootstrapCredentialStore
        .issue_for_zero_credential_user(
            &pool,
            *user_id.as_uuid(),
            OrgId::knl(),
            OffsetDateTime::now_utc(),
            Duration::hours(24),
        )
        .await
        .unwrap();

    // Redeem with NO transport header -> body carries the refresh token, no cookie.
    let redeem = post_raw(
        service.clone(),
        "/api/v1/auth/otp/redeem",
        None,
        json!({ "otp": issue.token.as_str() }),
    )
    .await;
    assert_eq!(redeem.status(), StatusCode::OK);
    assert!(
        set_cookie_values(&redeem).is_empty(),
        "mobile/body mode must not set any cookie"
    );
    let redeem_body = body_json(redeem).await;
    let refresh_token = redeem_body["refresh_token"]
        .as_str()
        .expect("body mode must return the refresh token in the JSON body")
        .to_owned();

    // Body-mode refresh rotates using the body token and still returns no cookie.
    let refreshed = post_raw(
        service.clone(),
        "/api/v1/auth/token/refresh",
        None,
        json!({ "refresh_token": refresh_token }),
    )
    .await;
    assert_eq!(refreshed.status(), StatusCode::OK);
    assert!(
        set_cookie_values(&refreshed).is_empty(),
        "mobile/body mode refresh must not set any cookie"
    );
    let rotated = body_json(refreshed).await["refresh_token"]
        .as_str()
        .expect("body-mode refresh must return the new refresh token in the body")
        .to_owned();
    assert_ne!(rotated, refresh_token);

    // Body-mode logout accepts the body token and revokes the family.
    let logout = post_raw(
        service.clone(),
        "/api/v1/auth/logout",
        None,
        json!({ "refresh_token": rotated }),
    )
    .await;
    assert_eq!(logout.status(), StatusCode::NO_CONTENT);
    assert!(
        set_cookie_values(&logout).is_empty(),
        "mobile/body mode logout must not set any cookie"
    );
}

// --- helpers ---------------------------------------------------------------

/// Cookie-mode usernameless passkey login: mirrors `usernameless_login` but sends
/// the `X-Auth-Transport: cookie` header so the response carries a Set-Cookie and
/// a null body refresh token. Returns the raw response for header + body asserts.
async fn cookie_mode_usernameless_login(
    service: &axum::Router,
    authenticator: &mut WebauthnAuthenticator<SoftPasskey>,
    credential_id: &str,
) -> http::Response<Body> {
    let start: LoginStartResponse = post_raw(
        service.clone(),
        "/api/v1/auth/passkey/login/start",
        None,
        json!({}),
    )
    .await
    .into_json(StatusCode::OK)
    .await;
    let challenge = inject_allow_credential(start.challenge, credential_id);
    let assertion = authenticator
        .do_authentication(Url::parse(TEST_ORIGIN).unwrap(), challenge)
        .unwrap();
    post_cookie_mode(
        service.clone(),
        "/api/v1/auth/passkey/login/finish",
        None,
        json!({ "ceremony_id": start.ceremony_id, "credential": assertion }),
    )
    .await
}

async fn start_step_up_assertion(
    service: &axum::Router,
    authenticator: &mut WebauthnAuthenticator<SoftPasskey>,
    credential_id: &str,
) -> Value {
    let start: LoginStartResponse = post_raw(
        service.clone(),
        "/api/v1/auth/passkey/login/start",
        None,
        json!({}),
    )
    .await
    .into_json(StatusCode::OK)
    .await;
    let challenge = inject_allow_credential(start.challenge, credential_id);
    let assertion = authenticator
        .do_authentication(Url::parse(TEST_ORIGIN).unwrap(), challenge)
        .unwrap();

    json!({
        "ceremony_id": start.ceremony_id,
        "credential": assertion
    })
}

/// Sign a user in via a directly-issued OTP (used to bootstrap an authenticated
/// session for any role in tests without a pre-existing passkey).
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

/// Enroll a passkey and return its credential id (base64url string).
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

async fn usernameless_login(
    service: &axum::Router,
    authenticator: &mut WebauthnAuthenticator<SoftPasskey>,
    credential_id: &str,
) -> TokenPairResponse {
    // login/start takes NO body and NO user_id; the server returns a discoverable
    // challenge with an EMPTY allowCredentials list.
    let start: LoginStartResponse = post_raw(
        service.clone(),
        "/api/v1/auth/passkey/login/start",
        None,
        json!({}),
    )
    .await
    .into_json(StatusCode::OK)
    .await;

    // The SoftPasskey harness cannot resolve a resident credential from an empty
    // allowCredentials (it has no resident-key store), so the test injects the
    // known credential id to emulate what a real discoverable authenticator does
    // internally. The SERVER ceremony stays fully discoverable — see the report's
    // SoftPasskey compromise note. The returned assertion still carries the
    // credential id, which is what the server resolves the user by.
    let challenge = inject_allow_credential(start.challenge, credential_id);
    let assertion = authenticator
        .do_authentication(Url::parse(TEST_ORIGIN).unwrap(), challenge)
        .unwrap();

    post_json(
        service.clone(),
        "/api/v1/auth/passkey/login/finish",
        None,
        json!({ "ceremony_id": start.ceremony_id, "credential": assertion }),
        StatusCode::OK,
    )
    .await
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

#[derive(Clone, Copy)]
struct FinancialStepUpFixture {
    branch_id: BranchId,
    requester: UserId,
    receptionist: UserId,
    admin: UserId,
    equipment: EquipmentId,
    work_order: WorkOrderId,
    statement_evidence: EvidenceId,
}

async fn seed_financial_step_up_fixture(
    pool: &PgPool,
    branch_id: BranchId,
    requester: UserId,
    receptionist: UserId,
    admin: UserId,
) -> FinancialStepUpFixture {
    let equipment = seed_financial_step_up_equipment(pool, branch_id).await;
    let work_order =
        seed_financial_step_up_work_order(pool, branch_id, receptionist, equipment).await;
    let statement_evidence = seed_financial_step_up_statement(pool, work_order, requester).await;
    FinancialStepUpFixture {
        branch_id,
        requester,
        receptionist,
        admin,
        equipment,
        work_order,
        statement_evidence,
    }
}

async fn submitted_financial_purchase(
    pool: &PgPool,
    fixture: FinancialStepUpFixture,
    amount_won: i64,
) -> PurchaseRequestId {
    let purchase_id = create_financial_purchase(pool, fixture, amount_won).await;
    let pool = pool.clone();
    console_platform_request_context::scope_org(OrgId::knl(), async move {
        PgFinancialStore::new(pool)
            .submit_purchase_request(PurchaseSubmitCommand {
                actor: fixture.receptionist,
                purchase_request_id: purchase_id,
                trace: TraceContext::generate(),
                occurred_at: OffsetDateTime::now_utc(),
            })
            .await
            .unwrap();
    })
    .await;
    purchase_id
}

async fn admin_approved_financial_purchase(
    pool: &PgPool,
    fixture: FinancialStepUpFixture,
    amount_won: i64,
) -> PurchaseRequestId {
    let purchase_id = submitted_financial_purchase(pool, fixture, amount_won).await;
    let pool = pool.clone();
    console_platform_request_context::scope_org(OrgId::knl(), async move {
        PgFinancialStore::new(pool)
            .approve_purchase_admin(PurchaseApprovalCommand {
                actor: fixture.admin,
                purchase_request_id: purchase_id,
                trace: TraceContext::generate(),
                occurred_at: OffsetDateTime::now_utc(),
            })
            .await
            .unwrap();
    })
    .await;
    purchase_id
}

async fn executive_pending_financial_purchase(
    pool: &PgPool,
    fixture: FinancialStepUpFixture,
) -> PurchaseRequestId {
    let purchase_id = admin_approved_financial_purchase(pool, fixture, 3_000_000).await;
    let pool = pool.clone();
    console_platform_request_context::scope_org(OrgId::knl(), async move {
        PgFinancialStore::new(pool)
            .prepare_expenditure(PrepareExpenditureCommand {
                actor: fixture.admin,
                purchase_request_id: purchase_id,
                expenditure_no: format!("EXP-{}", Uuid::new_v4()),
                trace: TraceContext::generate(),
                occurred_at: OffsetDateTime::now_utc(),
            })
            .await
            .unwrap();
    })
    .await;
    purchase_id
}

async fn ready_to_execute_financial_purchase(
    pool: &PgPool,
    fixture: FinancialStepUpFixture,
) -> PurchaseRequestId {
    let purchase_id = admin_approved_financial_purchase(pool, fixture, 900_000).await;
    let pool = pool.clone();
    console_platform_request_context::scope_org(OrgId::knl(), async move {
        PgFinancialStore::new(pool)
            .prepare_expenditure(PrepareExpenditureCommand {
                actor: fixture.admin,
                purchase_request_id: purchase_id,
                expenditure_no: format!("EXP-{}", Uuid::new_v4()),
                trace: TraceContext::generate(),
                occurred_at: OffsetDateTime::now_utc(),
            })
            .await
            .unwrap();
    })
    .await;
    purchase_id
}

async fn create_financial_purchase(
    pool: &PgPool,
    fixture: FinancialStepUpFixture,
    amount_won: i64,
) -> PurchaseRequestId {
    let pool = pool.clone();
    console_platform_request_context::scope_org(OrgId::knl(), async move {
        let purchase = PgFinancialStore::new(pool)
            .create_purchase_request(CreatePurchaseRequestCommand {
                actor: fixture.requester,
                branch_id: fixture.branch_id,
                equipment_id: Some(fixture.equipment),
                work_order_id: Some(fixture.work_order),
                statement_evidence_id: Some(fixture.statement_evidence),
                purchase_type: PurchaseType::LegacyManual,
                vendor_name: "Financial Step-Up Vendor".to_owned(),
                amount_won: Some(amount_won),
                lines: vec![financial_step_up_purchase_line(amount_won)],
                quote_attachment_ids: Vec::new(),
                memo: "financial step-up fixture".to_owned(),
                config: financial_step_up_config(),
                trace: TraceContext::generate(),
                occurred_at: OffsetDateTime::now_utc(),
            })
            .await
            .unwrap();
        purchase.id
    })
    .await
}

fn financial_step_up_config() -> FinancialConfigSnapshot {
    FinancialConfigSnapshot {
        depreciation_method: DepreciationMethod::StraightLine,
        useful_life_months: 60,
        residual_rate_bps: 1_000,
        declining_balance_rate_bps: 2_000,
        management_fee_rate_bps: 1_000,
        profit_rate_bps: 500,
        floor_negative_quote_residual: true,
        executive_approval_threshold_won: 2_000_000,
    }
}

fn financial_step_up_purchase_line(amount_won: i64) -> PurchaseRequestLineInput {
    PurchaseRequestLineInput {
        item: "step-up protected purchase".to_owned(),
        quantity: 1,
        unit_supply_price_won: amount_won,
        vat_won: Some(0),
    }
}

#[allow(clippy::too_many_arguments)]
async fn assert_financial_step_up_denied(
    service: axum::Router,
    pool: &PgPool,
    access_token: &str,
    path: &str,
    body: Value,
    purchase_request_id: PurchaseRequestId,
    unchanged_status: &str,
    mutation_action: &str,
    required_action: &str,
    expected_status: StatusCode,
    expected_code: &str,
    expected_failure_reason: &str,
) {
    let mutation_count_before =
        financial_audit_count(pool, mutation_action, purchase_request_id).await;
    let denial_count_before =
        financial_audit_count(pool, "purchase.step_up.denied", purchase_request_id).await;
    let rejected = post_raw(service, path, Some(access_token), body).await;
    assert_eq!(rejected.status(), expected_status);
    assert_eq!(body_json(rejected).await["error"]["code"], expected_code);
    assert_purchase_status(pool, purchase_request_id, unchanged_status).await;
    assert_financial_audit_count(
        pool,
        mutation_action,
        purchase_request_id,
        mutation_count_before,
    )
    .await;
    assert_financial_audit_count(
        pool,
        "purchase.step_up.denied",
        purchase_request_id,
        denial_count_before + 1,
    )
    .await;
    let after = latest_financial_step_up_denial_after(pool, purchase_request_id).await;
    assert_eq!(after["required_action"], required_action);
    assert_eq!(after["failure_code"], expected_code);
    assert_eq!(after["failure_reason"], expected_failure_reason);
    assert_eq!(after["step_up_verified"], false);
    assert!(after.get("credential").is_none());
    assert!(after.get("ceremony_id").is_none());
}

async fn assert_purchase_status(
    pool: &PgPool,
    purchase_request_id: PurchaseRequestId,
    expected: &str,
) {
    let status: String =
        sqlx::query_scalar("SELECT status FROM financial_purchase_requests WHERE id = $1")
            .bind(*purchase_request_id.as_uuid())
            .fetch_one(pool)
            .await
            .unwrap();
    assert_eq!(
        status, expected,
        "unexpected status for {purchase_request_id}"
    );
}

async fn assert_financial_audit_count(
    pool: &PgPool,
    action: &str,
    purchase_request_id: PurchaseRequestId,
    expected: i64,
) {
    let count = financial_audit_count(pool, action, purchase_request_id).await;
    assert_eq!(
        count, expected,
        "unexpected audit count for {action} on {purchase_request_id}"
    );
}

async fn financial_audit_count(
    pool: &PgPool,
    action: &str,
    purchase_request_id: PurchaseRequestId,
) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*)::BIGINT FROM audit_events WHERE action = $1 AND target_id = $2",
    )
    .bind(action)
    .bind(purchase_request_id.to_string())
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn latest_financial_step_up_denial_after(
    pool: &PgPool,
    purchase_request_id: PurchaseRequestId,
) -> Value {
    sqlx::query_scalar::<_, Option<Value>>(
        r#"
        SELECT after_snap
        FROM audit_events
        WHERE action = 'purchase.step_up.denied'
          AND target_id = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(purchase_request_id.to_string())
    .fetch_one(pool)
    .await
    .unwrap()
    .expect("step-up denial audit must include an after snapshot")
}

async fn seed_financial_step_up_equipment(pool: &PgPool, branch_id: BranchId) -> EquipmentId {
    let customer_id: Uuid = sqlx::query_scalar(
        "INSERT INTO registry_customers (branch_id, name, org_id) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(*branch_id.as_uuid())
    .bind("Financial Step-Up Customer")
    .bind(*OrgId::knl().as_uuid())
    .fetch_one(pool)
    .await
    .unwrap();
    let site_id: Uuid = sqlx::query_scalar(
        "INSERT INTO registry_sites (branch_id, customer_id, name, org_id) VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(*branch_id.as_uuid())
    .bind(customer_id)
    .bind("Financial Step-Up Site")
    .bind(*OrgId::knl().as_uuid())
    .fetch_one(pool)
    .await
    .unwrap();
    let equipment_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO registry_equipment (
            branch_id, customer_id, site_id, equipment_no, management_no,
            manufacturer_code, kind_code, power_code, status,
            specification, ton_text, model, vehicle_value, residual_value,
            asset_registered_on, source_sheet, source_row, org_id
        )
        VALUES ($1, $2, $3, $4, $5,
                'A', 'B', 'C', '임대', '좌식', '2.5T', 'GTS25DE',
                12000000, 9000000, DATE '2024-01-01', 'financial-step-up-test', 1, $6)
        RETURNING id
        "#,
    )
    .bind(*branch_id.as_uuid())
    .bind(customer_id)
    .bind(site_id)
    .bind("FST12-4200")
    .bind("FST-4200")
    .bind(*OrgId::knl().as_uuid())
    .fetch_one(pool)
    .await
    .unwrap();
    EquipmentId::from_uuid(equipment_id)
}

async fn seed_financial_step_up_work_order(
    pool: &PgPool,
    branch_id: BranchId,
    requested_by: UserId,
    equipment_id: EquipmentId,
) -> WorkOrderId {
    let row: (Uuid, Uuid) =
        sqlx::query_as("SELECT customer_id, site_id FROM registry_equipment WHERE id = $1")
            .bind(*equipment_id.as_uuid())
            .fetch_one(pool)
            .await
            .unwrap();
    let work_order_id = WorkOrderId::new();
    sqlx::query(
        r#"
        INSERT INTO work_orders (
            id, request_no, branch_id, equipment_id, customer_id, site_id,
            requested_by, status, symptom, org_id
        )
        VALUES ($1, '20260709-420', $2, $3, $4, $5, $6, 'RECEIVED', 'financial step-up fixture', $7)
        "#,
    )
    .bind(*work_order_id.as_uuid())
    .bind(*branch_id.as_uuid())
    .bind(*equipment_id.as_uuid())
    .bind(row.0)
    .bind(row.1)
    .bind(*requested_by.as_uuid())
    .bind(*OrgId::knl().as_uuid())
    .execute(pool)
    .await
    .unwrap();
    work_order_id
}

async fn seed_financial_step_up_statement(
    pool: &PgPool,
    work_order_id: WorkOrderId,
    uploaded_by: UserId,
) -> EvidenceId {
    let evidence_id = EvidenceId::new();
    sqlx::query(
        r#"
        INSERT INTO evidence_media (
            id, work_order_id, stage, s3_key, content_type, size_bytes,
            uploaded_by, worm_replica_status, retry_count, org_id
        )
        VALUES ($1, $2, 'REQUEST', $3, 'application/pdf', 2048, $4, 'VERIFIED', 0, $5)
        "#,
    )
    .bind(*evidence_id.as_uuid())
    .bind(*work_order_id.as_uuid())
    .bind(format!(
        "work-orders/{work_order_id}/REQUEST/{evidence_id}.pdf"
    ))
    .bind(*uploaded_by.as_uuid())
    .bind(*OrgId::knl().as_uuid())
    .execute(pool)
    .await
    .unwrap();
    evidence_id
}

/// Inject one `allowCredentials` entry into a discoverable challenge so the
/// SoftPasskey harness can locate its key. Emulates resident-credential
/// discovery; production never does this.
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

trait ResponseExt {
    async fn into_json<T: for<'de> Deserialize<'de>>(self, expected: StatusCode) -> T;
}

impl ResponseExt for http::Response<Body> {
    async fn into_json<T: for<'de> Deserialize<'de>>(self, expected: StatusCode) -> T {
        assert_eq!(self.status(), expected);
        let bytes = to_bytes(self.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }
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

async fn post_raw_with_trusted_ip(
    service: axum::Router,
    uri: &str,
    ip: &str,
    body: Value,
) -> http::Response<Body> {
    let mut request = Request::builder()
        .uri(uri)
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-forwarded-for", ip)
        .body(Body::from(body.to_string()))
        .unwrap();
    request
        .extensions_mut()
        .insert(ConnectInfo("10.0.0.3:443".parse::<SocketAddr>().unwrap()));
    service.oneshot(request).await.unwrap()
}

/// POST as a WEB client: sends `X-Auth-Transport: cookie` and, optionally, a
/// `Cookie` header carrying `console_refresh=<token>` (what a browser would replay).
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

/// Same as `post_cookie_mode`, but includes a browser `X-Device-Id` header so
/// tests exercise the per-device auth rate-limit bucket that the web console
/// sends on every request.
async fn post_cookie_mode_with_device_id(
    service: axum::Router,
    uri: &str,
    cookie: Option<&str>,
    device_id: &str,
    body: Value,
) -> http::Response<Body> {
    let mut builder = Request::builder()
        .uri(uri)
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-auth-transport", "cookie")
        .header("x-device-id", device_id);
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, format!("console_refresh={cookie}"));
    }
    service
        .oneshot(builder.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap()
}

/// Collect every `Set-Cookie` header value off a response as owned strings.
fn set_cookie_values(response: &http::Response<Body>) -> Vec<String> {
    response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .map(|value| value.to_str().unwrap().to_owned())
        .collect()
}

/// Find the `console_refresh` Set-Cookie attribute string, returning its full
/// directive (e.g. `console_refresh=abc; HttpOnly; SameSite=Strict; ...`).
fn console_refresh_set_cookie(response: &http::Response<Body>) -> Option<String> {
    set_cookie_values(response)
        .into_iter()
        .find(|value| value.starts_with("console_refresh="))
}

/// Pull the cookie's value (the substring between `console_refresh=` and the first `;`).
fn cookie_token(set_cookie: &str) -> &str {
    set_cookie
        .strip_prefix("console_refresh=")
        .and_then(|rest| rest.split(';').next())
        .unwrap()
}

async fn body_json(response: http::Response<Body>) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn app_state_with_trusted_proxy(
    pool: PgPool,
    private_key_pem: String,
    public_key_pem: String,
) -> Result<AppState, console_app::AppError> {
    let mut pairs = vec![
        ("CONSOLE_APP_ROLE", AppRole::Api.to_string()),
        ("CONSOLE_HTTP_ADDR", "127.0.0.1:0".to_owned()),
        ("CONSOLE_JWT_ISSUER", TEST_ISSUER.to_owned()),
        ("CONSOLE_JWT_AUDIENCE", TEST_AUDIENCE.to_owned()),
        ("CONSOLE_JWT_PRIVATE_KEY_PEM", private_key_pem),
        ("CONSOLE_JWT_PUBLIC_KEY_PEM", public_key_pem),
        ("CONSOLE_WEBAUTHN_RP_ID", "example.com".to_owned()),
        ("CONSOLE_WEBAUTHN_RP_ORIGIN", TEST_ORIGIN.to_owned()),
        ("CONSOLE_WEBAUTHN_RP_NAME", "Console".to_owned()),
        ("CONSOLE_TRUSTED_PROXY_COUNT", "1".to_owned()),
        ("CONSOLE_TRUSTED_PROXY_CIDRS", "10.0.0.0/8".to_owned()),
    ];
    pairs.extend(account_transport_urls(&pool));
    let config = AppConfig::from_pairs(pairs)?;

    AppState::from_config(config).await
}

async fn app_state(
    pool: PgPool,
    private_key_pem: String,
    public_key_pem: String,
) -> Result<AppState, console_app::AppError> {
    let mut pairs = vec![
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

async fn seed_equipment(pool: &PgPool, branch_id: BranchId, management_no: &str) {
    let equipment_suffix = format!("{:0>4}", management_no);
    let customer_id: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO registry_customers (branch_id, name, org_id) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(*branch_id.as_uuid())
    .bind(format!("Customer {management_no}"))
    .bind(*OrgId::knl().as_uuid())
    .fetch_one(pool)
    .await
    .unwrap();
    let site_id: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO registry_sites (branch_id, customer_id, name, org_id) VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(*branch_id.as_uuid())
    .bind(customer_id)
    .bind(format!("Site {management_no}"))
    .bind(*OrgId::knl().as_uuid())
    .fetch_one(pool)
    .await
    .unwrap();
    sqlx::query(
        r#"
        INSERT INTO registry_equipment (
            branch_id, customer_id, site_id, equipment_no, management_no,
            manufacturer_code, kind_code, power_code, status,
            specification, ton_text, model, source_sheet, source_row, org_id
        )
        VALUES ($1, $2, $3, $4, $5,
                'A', 'B', 'C', '임대', '좌식', '2.5', 'GTS25DE', 'test', 1, $6)
        "#,
    )
    .bind(*branch_id.as_uuid())
    .bind(customer_id)
    .bind(site_id)
    .bind(format!("ABC12-{equipment_suffix}"))
    .bind(management_no)
    .bind(*OrgId::knl().as_uuid())
    .execute(pool)
    .await
    .unwrap();
}

async fn seed_mobile_step_up_work_order(
    pool: &PgPool,
    branch_id: BranchId,
    receptionist: UserId,
    mechanic: UserId,
    admin: UserId,
    executive: UserId,
) -> Uuid {
    let (equipment_id, customer_id, site_id): (Uuid, Uuid, Uuid) = sqlx::query_as(
        r#"
        SELECT id, customer_id, site_id
        FROM registry_equipment
        WHERE branch_id = $1 AND org_id = $2
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(*branch_id.as_uuid())
    .bind(*OrgId::knl().as_uuid())
    .fetch_one(pool)
    .await
    .unwrap();
    let work_order_id = Uuid::new_v4();
    let submitted_at = OffsetDateTime::now_utc() - Duration::hours(1);
    sqlx::query(
        r#"
        INSERT INTO work_orders (
            id, request_no, branch_id, equipment_id, customer_id, site_id,
            requested_by, status, priority, symptom, result_type, diagnosis,
            action_taken, target_due_at, report_submitted_by, report_submitted_at,
            created_at, updated_at, org_id
        )
        VALUES (
            $1, '20260709-410', $2, $3, $4, $5, $6, 'REPORT_SUBMITTED', 'P1',
            'Mobile step-up fixture', 'COMPLETED', 'diagnosis', 'action taken',
            $7, $8, $9, $9, $9, $10
        )
        "#,
    )
    .bind(work_order_id)
    .bind(*branch_id.as_uuid())
    .bind(equipment_id)
    .bind(customer_id)
    .bind(site_id)
    .bind(*receptionist.as_uuid())
    .bind(submitted_at + Duration::days(1))
    .bind(*mechanic.as_uuid())
    .bind(submitted_at)
    .bind(*OrgId::knl().as_uuid())
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        r#"
        INSERT INTO work_order_assignments (work_order_id, mechanic_id, role, assigned_at, org_id)
        VALUES ($1, $2, 'PRIMARY', $3, $4)
        "#,
    )
    .bind(work_order_id)
    .bind(*mechanic.as_uuid())
    .bind(submitted_at - Duration::hours(1))
    .bind(*OrgId::knl().as_uuid())
    .execute(pool)
    .await
    .unwrap();
    for (step_order, role, approver_id, status, requested_at) in [
        (
            1_i16,
            "MECHANIC",
            Some(mechanic),
            "APPROVED",
            Some(submitted_at),
        ),
        (2_i16, "ADMIN", Some(admin), "PENDING", Some(submitted_at)),
        (3_i16, "EXECUTIVE", Some(executive), "NOT_STARTED", None),
    ] {
        sqlx::query(
            r#"
            INSERT INTO work_order_approval_steps (
                work_order_id, step_order, role, approver_id, status, requested_at, org_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(work_order_id)
        .bind(step_order)
        .bind(role)
        .bind(approver_id.map(|user| *user.as_uuid()))
        .bind(status)
        .bind(requested_at)
        .bind(*OrgId::knl().as_uuid())
        .execute(pool)
        .await
        .unwrap();
    }
    work_order_id
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

async fn assert_work_order_status(pool: &PgPool, work_order_id: Uuid, expected: &str) {
    let status: String = sqlx::query_scalar("SELECT status FROM work_orders WHERE id = $1")
        .bind(work_order_id)
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(status, expected);
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

async fn assert_audit_count(pool: &PgPool, action: &str, expected: i64) {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM audit_events WHERE action = $1")
        .bind(action)
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(count, expected, "unexpected audit count for {action}");
}

// AS1.2 test candidate: exercise the public production configuration parser.
// These synthetic URLs are never connected by these configuration-only tests.
fn account_transport_config_pairs() -> Vec<(&'static str, String)> {
    let signing_key = SigningKey::random(&mut OsRng);
    vec![
        ("CONSOLE_APP_ROLE", AppRole::Api.to_string()),
        ("CONSOLE_HTTP_ADDR", "127.0.0.1:0".to_owned()),
        (
            "DATABASE_URL",
            "postgresql://console_rt:runtime-fixture@localhost:5544/console".to_owned(),
        ),
        (
            "LEAVE_COMMAND_DATABASE_URL",
            "postgresql://console_leave_cmd:leave-fixture@localhost:5544/console".to_owned(),
        ),
        (
            "ONTOLOGY_COMMAND_DATABASE_URL",
            "postgresql://console_ontology_cmd:ontology-fixture@localhost:5544/console".to_owned(),
        ),
        (
            "PLATFORM_FORCE_COMMAND_DATABASE_URL",
            "postgresql://console_platform_force_cmd:force-fixture@localhost:5544/console"
                .to_owned(),
        ),
        ("CONSOLE_JWT_ISSUER", TEST_ISSUER.to_owned()),
        ("CONSOLE_JWT_AUDIENCE", TEST_AUDIENCE.to_owned()),
        (
            "CONSOLE_JWT_PRIVATE_KEY_PEM",
            signing_key
                .to_pkcs8_pem(LineEnding::LF)
                .unwrap()
                .to_string(),
        ),
        (
            "CONSOLE_JWT_PUBLIC_KEY_PEM",
            signing_key
                .verifying_key()
                .to_public_key_pem(LineEnding::LF)
                .unwrap(),
        ),
        ("CONSOLE_WEBAUTHN_RP_ID", "example.com".to_owned()),
        ("CONSOLE_WEBAUTHN_RP_ORIGIN", TEST_ORIGIN.to_owned()),
        ("CONSOLE_WEBAUTHN_RP_NAME", "Console".to_owned()),
    ]
}

fn assert_account_transport_config_rejected(auth_url: Option<&str>, secret: Option<&str>) {
    let mut pairs = account_transport_config_pairs();
    if let Some(url) = auth_url {
        pairs.push(("AUTH_DATABASE_URL", url.to_owned()));
    }
    let result = AppConfig::from_pairs(pairs);
    assert!(
        result.is_err(),
        "the API must reject invalid or missing auth-pool configuration"
    );
    let error = result.unwrap_err().to_string();
    assert!(
        error.contains("AUTH_DATABASE_URL"),
        "diagnostic must identify the rejected configuration key"
    );
    if let Some(secret) = secret {
        assert!(
            !error.contains(secret),
            "configuration errors must not echo auth credentials"
        );
    }
}

#[test]
fn account_auth_database_requires_explicit_configuration() {
    assert_account_transport_config_rejected(None, None);
    assert_account_transport_config_rejected(Some(""), None);
    assert_account_transport_config_rejected(Some("   "), None);
}

#[test]
fn account_auth_database_accepts_distinct_narrow_identity() {
    let mut pairs = account_transport_config_pairs();
    pairs.push((
        "AUTH_DATABASE_URL",
        "postgresql://console_auth_rt:auth-fixture@localhost:5544/console".to_owned(),
    ));
    let config = AppConfig::from_pairs(pairs).expect("separate configured auth transport is valid");
    assert_eq!(
        config.database_url.as_deref(),
        Some("postgresql://console_rt:runtime-fixture@localhost:5544/console")
    );
    assert!(
        config.auth_rest.is_some(),
        "control must exercise enabled authentication"
    );
}

#[test]
fn account_auth_database_rejects_other_database_identities() {
    for role in [
        "console_app",
        "console_rt",
        "console_leave_cmd",
        "console_ontology_cmd",
        "console_platform_force_cmd",
        "other_auth_role",
    ] {
        let url = format!("postgresql://{role}:auth-secret-canary@localhost:5544/console");
        assert_account_transport_config_rejected(Some(&url), Some("auth-secret-canary"));
    }
}

#[test]
fn account_auth_database_requires_nonempty_distinct_password() {
    for url in [
        "postgresql://console_auth_rt@localhost:5544/console",
        "postgresql://console_auth_rt:@localhost:5544/console",
        "postgresql://console_auth_rt:auth-fixture@localhost:5544/console?password=",
        "postgresql://console_auth_rt:runtime-fixture@localhost:5544/console",
        "postgresql://console_auth_rt:leave-fixture@localhost:5544/console",
        "postgresql://console_auth_rt:ontology-fixture@localhost:5544/console",
        "postgresql://console_auth_rt:force-fixture@localhost:5544/console",
        "postgresql://console_auth_rt:runtime%2Dfixture@localhost:5544/console",
    ] {
        assert_account_transport_config_rejected(Some(url), None);
    }
}

#[test]
fn account_auth_database_rejects_identity_overrides_without_leaking_secrets() {
    for query in [
        "options=-c%20role%3Dconsole_app",
        "options=-crole%3Dconsole_app",
        "options%5Brole%5D=console_app",
        "user=console_app",
    ] {
        let url = format!(
            "postgresql://console_auth_rt:auth-secret-canary@localhost:5544/console?{query}"
        );
        assert_account_transport_config_rejected(Some(&url), Some("auth-secret-canary"));
    }
}

// Owner pool supplies only the per-test database name; every serving URL comes
// from a distinct real LOGIN credential provisioned by the disposable harness.
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

/// SQLx supplies an empty disposable database; the production migration entry
/// owns both numbered migrations and Apalis initialization before any fixtures.
/// Runtime pools never receive this directly authenticated migration-owner URL.
async fn prepare_http_database(pool: &PgPool) {
    prepare_http_database_staging(pool).await;
    finalize_account_custody(pool).await;
}

fn account_custody_finalizer_sql() -> String {
    console_platform_test_support::account_custody_finalizer_sql()
}

async fn finalize_account_custody(pool: &PgPool) {
    console_platform_test_support::finalize_account_custody(pool).await;
}

async fn prepare_http_database_staging(pool: &PgPool) {
    let config = prepare_http_migration_config(pool).await;
    run_migrations(&config)
        .await
        .expect("complete production schema migration before HTTP fixtures");
}

async fn prepare_http_migration_config(pool: &PgPool) -> AppConfig {
    AppConfig::from_pairs([
        ("CONSOLE_APP_ROLE", AppRole::Migrate.to_string()),
        (
            "DATABASE_URL",
            console_platform_test_support::prepare_test_migration_owner_url(pool).await,
        ),
    ])
    .expect("production migration configuration")
}

// AS1.3 selected legacy-fence acceptance. These owner fixtures install only
// approved data rows into real migrations; they do not emulate production
// cutover/drain, Account enrollment, security recovery authority, or DDL.
#[sqlx::test(migrations = false)]
async fn account_fence_unmigrated_transport_positive_control(pool: PgPool) {
    let fixture = legacy_fence_fixture(&pool).await;
    assert_legacy_reads(&fixture.router, fixture.subject, &fixture.access).await;
    assert_legacy_reads(&fixture.router, fixture.control, &fixture.control_access).await;
}

#[sqlx::test(migrations = false)]
async fn account_fence_pending_enrollment_rejects_legacy_sessions(pool: PgPool) {
    assert_legacy_fence_state(&pool, "PENDING_ENROLLMENT").await;
}

#[sqlx::test(migrations = false)]
async fn account_fence_active_rejects_legacy_sessions(pool: PgPool) {
    assert_legacy_fence_state(&pool, "ACTIVE").await;
}

#[sqlx::test(migrations = false)]
async fn account_fence_security_suspended_rejects_legacy_sessions(pool: PgPool) {
    assert_legacy_fence_state(&pool, "SECURITY_SUSPENDED").await;
}

#[sqlx::test(migrations = false)]
async fn account_fence_recovery_required_rejects_legacy_sessions(pool: PgPool) {
    assert_legacy_fence_state(&pool, "RECOVERY_REQUIRED").await;
}

#[sqlx::test(migrations = false)]
async fn account_fence_recovery_to_active_does_not_restore_legacy_sessions(pool: PgPool) {
    let fixture = legacy_fence_fixture(&pool).await;
    let keys_before = fence_credential_snapshot(&pool, fixture.subject).await;
    insert_account_fence(&pool, fixture.subject, "RECOVERY_REQUIRED").await;
    // Reserve BOTH unspent refresh tokens until after ACTIVE, so a refusal's
    // revocation side effect cannot hide a state-dependent authentication fence.
    let before = legacy_read_responses(&fixture.router, &fixture.access).await;
    assert_legacy_reads(&fixture.router, fixture.control, &fixture.control_access).await;
    let updated = sqlx::query(
        "UPDATE account_security SET security_state = 'ACTIVE', \
         security_generation = security_generation + 1, revision = revision + 1, \
         updated_at = now() WHERE account_id = $1",
    )
    .bind(fixture.subject.as_uuid())
    .execute(&pool)
    .await
    .expect("seed persisted recovered state; not a production recovery command");
    assert_eq!(updated.rows_affected(), 1);
    let recovered: (String, i64, i64) = sqlx::query_as(
        "SELECT security_state, security_generation, revision FROM account_security WHERE account_id = $1",
    ).bind(fixture.subject.as_uuid()).fetch_one(&pool).await.unwrap();
    assert_eq!(recovered, ("ACTIVE".to_owned(), 2, 2));
    let after = legacy_session_responses(&fixture).await;
    assert!(
        keys_before == fence_credential_snapshot(&pool, fixture.subject).await,
        "recovered-state refusal must preserve credential ID and serialized passkey"
    );
    assert_legacy_reads(&fixture.router, fixture.control, &fixture.control_access).await;
    assert_legacy_denials(before.into_iter().chain(after), &fixture.known_secrets()).await;
}

#[sqlx::test(migrations = false)]
async fn account_fence_outstanding_employer_otp_cannot_create_a_session(pool: PgPool) {
    let fixture = legacy_fence_fixture(&pool).await;
    let branch = seed_branch(&pool, "Fence OTP Region", "Fence OTP Branch").await;
    let subject =
        seed_user_with_branch(&pool, "Fence OTP", "010-8900-0003", "MECHANIC", branch).await;
    let control =
        seed_user_with_branch(&pool, "Control OTP", "010-8900-0004", "MECHANIC", branch).await;
    let issue = BootstrapCredentialStore
        .issue_for_zero_credential_user(
            &pool,
            *subject.as_uuid(),
            OrgId::knl(),
            OffsetDateTime::now_utc(),
            Duration::hours(24),
        )
        .await
        .unwrap();
    let control_issue = BootstrapCredentialStore
        .issue_for_zero_credential_user(
            &pool,
            *control.as_uuid(),
            OrgId::knl(),
            OffsetDateTime::now_utc(),
            Duration::hours(24),
        )
        .await
        .unwrap();
    insert_account_fence(&pool, subject, "ACTIVE").await;
    let rejected = post_raw(
        fixture.router.clone(),
        "/api/v1/auth/otp/redeem",
        None,
        json!({"otp": issue.token.as_str()}),
    )
    .await;
    let accepted: OtpRedeemResponse = post_json(
        fixture.router.clone(),
        "/api/v1/auth/otp/redeem",
        None,
        json!({"otp": control_issue.token.as_str()}),
        StatusCode::OK,
    )
    .await;
    assert!(!accepted.access_token.is_empty());
    let families: i64 =
        sqlx::query_scalar("SELECT count(*) FROM auth_refresh_token_families WHERE user_id = $1")
            .bind(subject.as_uuid())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(families, 0, "fenced employer OTP cannot mint a family");
    assert_legacy_denials([rejected], &[issue.token.as_str()]).await;
}

struct LegacyFenceFixture {
    router: axum::Router,
    subject: UserId,
    access: String,
    body_refresh: String,
    cookie_refresh: String,
    control: UserId,
    control_access: String,
}

async fn legacy_fence_fixture(pool: &PgPool) -> LegacyFenceFixture {
    prepare_http_database(pool).await;
    let key = SigningKey::random(&mut OsRng);
    let private = key.to_pkcs8_pem(LineEnding::LF).unwrap();
    let public = key
        .verifying_key()
        .to_public_key_pem(LineEnding::LF)
        .unwrap();
    let branch = seed_branch(pool, "Fence Region", "Fence Branch").await;
    let subject =
        seed_user_with_branch(pool, "Fence Subject", "010-8900-0001", "MECHANIC", branch).await;
    let control =
        seed_user_with_branch(pool, "Fence Control", "010-8900-0002", "MECHANIC", branch).await;
    let router = build_router(
        app_state(pool.clone(), private.to_string(), public)
            .await
            .unwrap(),
    );
    let setup = admin_session_via_otp(&router, pool, subject).await;
    let mut authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let credential = enroll_passkey(&router, &mut authenticator, &setup).await;
    let login = usernameless_login(&router, &mut authenticator, &credential).await;
    assert_legacy_reads(&router, subject, &login.access_token).await;
    let expected_key: Uuid =
        sqlx::query_scalar("SELECT id FROM auth_webauthn_credentials WHERE user_id = $1")
            .bind(subject.as_uuid())
            .fetch_one(pool)
            .await
            .unwrap();
    let keys: Value = get_legacy_raw(&router, "/api/v1/auth/passkeys", &login.access_token)
        .await
        .into_json(StatusCode::OK)
        .await;
    assert_eq!(keys.as_array().unwrap().len(), 1);
    assert_eq!(keys[0]["id"], json!(expected_key));
    let rotated: TokenPairResponse = post_json(
        router.clone(),
        "/api/v1/auth/token/refresh",
        None,
        json!({"refresh_token": login.refresh_token.unwrap()}),
        StatusCode::OK,
    )
    .await;
    let cookie_login =
        cookie_mode_usernameless_login(&router, &mut authenticator, &credential).await;
    assert_eq!(cookie_login.status(), StatusCode::OK);
    let cookie = console_refresh_set_cookie(&cookie_login).unwrap();
    let cookie_rotation = post_cookie_mode(
        router.clone(),
        "/api/v1/auth/token/refresh",
        Some(cookie_token(&cookie)),
        json!({}),
    )
    .await;
    assert_eq!(cookie_rotation.status(), StatusCode::OK);
    let cookie_rotation = console_refresh_set_cookie(&cookie_rotation).unwrap();
    let control_access = admin_session_via_otp(&router, pool, control).await;
    assert_legacy_reads(&router, control, &control_access).await;
    LegacyFenceFixture {
        router,
        subject,
        access: rotated.access_token,
        body_refresh: rotated.refresh_token.unwrap(),
        cookie_refresh: cookie_token(&cookie_rotation).to_owned(),
        control,
        control_access,
    }
}

async fn insert_account_fence(pool: &PgPool, subject: UserId, state: &str) {
    let exists: bool = sqlx::query_scalar(
        "SELECT to_regclass('public.accounts') IS NOT NULL AND to_regclass('public.account_security') IS NOT NULL",
    ).fetch_one(pool).await.unwrap();
    assert!(
        exists,
        "Account schema prerequisite missing; legacy-fence assertions not reached"
    );
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("INSERT INTO accounts (id, created_at) VALUES ($1, now())")
        .bind(subject.as_uuid())
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO account_security (account_id, security_state, security_generation, revision, updated_at, context_generation) \
         VALUES ($1, $2, 1, 1, now(), 1)",
    ).bind(subject.as_uuid()).bind(state).execute(&mut *tx).await.unwrap();
    tx.commit().await.unwrap();
}

async fn assert_legacy_fence_state(pool: &PgPool, state: &str) {
    let fixture = legacy_fence_fixture(pool).await;
    let before = fence_credential_snapshot(pool, fixture.subject).await;
    insert_account_fence(pool, fixture.subject, state).await;
    // Same router: a cached successful old-session lookup must not survive.
    let responses = legacy_session_responses(&fixture).await;
    assert_legacy_reads(&fixture.router, fixture.control, &fixture.control_access).await;
    let active: bool = sqlx::query_scalar("SELECT is_active FROM users WHERE id = $1")
        .bind(fixture.subject.as_uuid())
        .fetch_one(pool)
        .await
        .unwrap();
    assert!(
        active,
        "Company deactivation must not manufacture this rejection"
    );
    let after = fence_credential_snapshot(pool, fixture.subject).await;
    assert!(
        before == after,
        "legacy refusal must preserve existing passkey bytes"
    );
    assert_legacy_denials(responses, &fixture.known_secrets()).await;
}

async fn get_legacy_raw(router: &axum::Router, path: &str, access: &str) -> http::Response<Body> {
    router
        .clone()
        .oneshot(
            Request::builder()
                .uri(path)
                .header(header::AUTHORIZATION, format!("Bearer {access}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn legacy_read_responses(router: &axum::Router, access: &str) -> Vec<http::Response<Body>> {
    let mut responses = Vec::new();
    for path in ["/api/v1/users/me", "/api/v1/auth/passkeys"] {
        responses.push(get_legacy_raw(router, path, access).await);
    }
    responses
}

async fn assert_legacy_reads(router: &axum::Router, subject: UserId, access: &str) {
    let me: Value = get_legacy_raw(router, "/api/v1/users/me", access)
        .await
        .into_json(StatusCode::OK)
        .await;
    assert_eq!(me["id"], json!(subject));
    let keys: Value = get_legacy_raw(router, "/api/v1/auth/passkeys", access)
        .await
        .into_json(StatusCode::OK)
        .await;
    assert!(keys.is_array());
}

async fn legacy_session_responses(fixture: &LegacyFenceFixture) -> Vec<http::Response<Body>> {
    let mut responses = legacy_read_responses(&fixture.router, &fixture.access).await;
    responses.push(
        post_raw(
            fixture.router.clone(),
            "/api/v1/auth/token/refresh",
            None,
            json!({"refresh_token": fixture.body_refresh}),
        )
        .await,
    );
    responses.push(
        post_cookie_mode(
            fixture.router.clone(),
            "/api/v1/auth/token/refresh",
            Some(&fixture.cookie_refresh),
            json!({}),
        )
        .await,
    );
    responses
}

impl LegacyFenceFixture {
    fn known_secrets(&self) -> [&str; 3] {
        [&self.access, &self.body_refresh, &self.cookie_refresh]
    }
}

async fn fence_credential_snapshot(pool: &PgPool, subject: UserId) -> Vec<(Uuid, String, String)> {
    sqlx::query_as("SELECT id, credential_id, passkey_json::text FROM auth_webauthn_credentials WHERE user_id = $1 ORDER BY id")
        .bind(subject.as_uuid()).fetch_all(pool).await.unwrap()
}

async fn assert_legacy_denials(
    responses: impl IntoIterator<Item = http::Response<Body>>,
    known_secrets: &[&str],
) {
    for (ordinal, response) in responses.into_iter().enumerate() {
        let status = response.status().as_u16();
        assert!(
            legacy_denial_is_safe(response, known_secrets).await,
            "legacy refusal must be401 with v1 error JSON, no credential cookie/token fields or known secret echoes; response {} returned HTTP {status}",
            ordinal + 1
        );
    }
}

async fn legacy_denial_is_safe(response: http::Response<Body>, secrets: &[&str]) -> bool {
    if response.status() != StatusCode::UNAUTHORIZED {
        return false;
    }
    for value in response.headers().values() {
        let Ok(text) = value.to_str() else {
            return false;
        };
        if secrets
            .iter()
            .any(|secret| !secret.is_empty() && text.contains(secret))
        {
            return false;
        }
    }
    for cookie in set_cookie_values(&response) {
        let Some((_, value)) = cookie.split(';').next().unwrap().split_once('=') else {
            return false;
        };
        if !value.is_empty() {
            return false;
        }
    }
    let Ok(body) = to_bytes(response.into_body(), 64 * 1024).await else {
        return false;
    };
    let Ok(text) = std::str::from_utf8(&body) else {
        return false;
    };
    if secrets
        .iter()
        .any(|secret| !secret.is_empty() && text.contains(secret))
    {
        return false;
    }
    let Ok(value) = serde_json::from_slice::<Value>(&body) else {
        return false;
    };
    if !value["error"]["code"].is_string() || !value["error"]["message"].is_string() {
        return false;
    }
    let mut pending = vec![&value];
    while let Some(value) = pending.pop() {
        match value {
            Value::Object(object) => {
                for (key, child) in object {
                    let key = key.replace(['_', '-'], "").to_ascii_lowercase();
                    if matches!(
                        key.as_str(),
                        "accesstoken" | "refreshtoken" | "token" | "otp"
                    ) {
                        return false;
                    }
                    pending.push(child);
                }
            }
            Value::Array(array) => pending.extend(array),
            _ => {}
        }
    }
    true
}

// Exercise the denial oracle even while Account-schema prerequisites keep the
// product fence assertions unreachable. These are oracle controls, not HTTP
// product coverage or a general information-flow proof.
#[tokio::test]
async fn account_fence_denial_oracle_rejects_credential_leaks_and_malformed_errors() {
    let error = r#"{"error":{"code":"unauthorized","message":"denied"}}"#;
    let safe = http::Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header(header::SET_COOKIE, "console_refresh=; Max-Age=0; Path=/")
        .body(Body::from(error))
        .unwrap();
    assert!(legacy_denial_is_safe(safe, &["secret-canary"]).await);
    for cookie in [
        "console_refresh=secret-canary; Max-Age=01",
        "console_refresh=other; Path=/Max-Age=0",
    ] {
        let response = http::Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .header(header::SET_COOKIE, cookie)
            .body(Body::from(error))
            .unwrap();
        assert!(!legacy_denial_is_safe(response, &["secret-canary"]).await);
    }
    for body in [
        "not JSON",
        r#"{"error":{"code":"unauthorized","message":"secret-canary"}}"#,
        r#"{"error":{"code":"unauthorized","message":"denied"},"data":{"refresh_token":"different-token"}}"#,
    ] {
        let response = http::Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::from(body))
            .unwrap();
        assert!(!legacy_denial_is_safe(response, &["secret-canary"]).await);
    }
    let header_echo = http::Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header("x-debug", "secret-canary")
        .body(Body::from(error))
        .unwrap();
    assert!(!legacy_denial_is_safe(header_echo, &["secret-canary"]).await);
}

// BW31: additive browser HTTP/crypto candidate; no resident-browser claim.
mod account_browser {
    //! Test candidate bound to BW31 5f9f3b15704deaf8e81a682e4475af489d164210.
    //! No test-created schema/roles or substitute handlers. Terms publication rows
    //! are prerequisites, not publisher-authority/CAS proof. SoftPasskey's UV flag,
    //! nonresident registration and allow-list/userHandle accommodations below are
    //! synthetic crypto fixtures, NOT browser presence, residency or discovery proof.

    use super::*;
    use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
    use sha2::{Digest, Sha256};
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::PathBuf;

    const MANIFEST_DIGEST: &str =
        "3643e74a128a2fb3facce194264bcb40487dc79548b4d3b734b1de90be76fe52";
    const ACCESS: &str = "__Host-console_account_session";
    const REFRESH: &str = "__Host-console_account_refresh";
    const ENROLLMENT: &str = "__Host-console_account_enrollment";
    const LOGIN: &str = "__Host-console_account_login";
    const RECEIPT: &str = "31313131-3131-4131-8131-313131313131";

    const MANIFEST_TEXT: &str = r##"{
  "format_version": 1,
  "fixture_only": true,
  "items": [
    {
      "terms_kind": "test.account.service",
      "title": "테스트 서비스 약관",
      "locale": "ko-KR",
      "content_sha256": "d58e7fdecdb5cf68640ae805ede9ce5613ace72bf9f92b9b9ccd0d2729646f2c",
      "required": true
    },
    {
      "terms_kind": "test.account.privacy",
      "title": "테스트 개인정보 안내",
      "locale": "ko-KR",
      "content_sha256": "4254fa64b821906ac2995f0829590ca151540929b05594a023afc99d228c23d2",
      "required": true
    }
  ]
}
"##;
    const INDEX_TEXT: &str = r##"{
  "manifest": {
    "path": "fixtures/manifest.json",
    "sha256": "3643e74a128a2fb3facce194264bcb40487dc79548b4d3b734b1de90be76fe52"
  },
  "content": [
    {
      "path": "fixtures/service.txt",
      "sha256": "d58e7fdecdb5cf68640ae805ede9ce5613ace72bf9f92b9b9ccd0d2729646f2c"
    },
    {
      "path": "fixtures/privacy.txt",
      "sha256": "4254fa64b821906ac2995f0829590ca151540929b05594a023afc99d228c23d2"
    }
  ],
  "publication": "TEST_ONLY; cannot be production content authority"
}
"##;
    const SERVICE_TEXT: &str = r##"테스트 전용 자료입니다. 실제 서비스 약관이나 법적 동의 문서가 아닙니다.
"##;
    const PRIVACY_TEXT: &str = r##"테스트 전용 자료입니다. 실제 개인정보 처리 동의나 법적 판단을 나타내지 않습니다.
"##;

    fn fixture_file(path: &str) -> &'static [u8] {
        match path {
            "fixtures/manifest.json" => MANIFEST_TEXT.as_bytes(),
            "fixtures/artifact-index.json" => INDEX_TEXT.as_bytes(),
            "fixtures/service.txt" => SERVICE_TEXT.as_bytes(),
            "fixtures/privacy.txt" => PRIVACY_TEXT.as_bytes(),
            _ => panic!("unregistered test artifact"),
        }
    }

    struct Artifacts {
        root: PathBuf,
    }
    impl Artifacts {
        fn new() -> Self {
            let root =
                std::env::temp_dir().join(format!("console-bw31-fixture-{}", Uuid::new_v4()));
            std::fs::create_dir(&root).unwrap();
            let owned = Self { root };
            std::fs::create_dir(owned.root.join("fixtures")).unwrap();
            for file in [
                "manifest.json",
                "artifact-index.json",
                "service.txt",
                "privacy.txt",
            ] {
                let path = format!("fixtures/{file}");
                std::fs::write(owned.root.join(&path), fixture_file(&path)).unwrap();
            }
            owned
        }
    }
    impl Drop for Artifacts {
        fn drop(&mut self) {
            // Only the unique directory created by this fixture; never repo paths.
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }
    struct Fixture {
        service: axum::Router,
        _artifacts: Artifacts,
        pool: PgPool,
    }
    impl std::ops::Deref for Fixture {
        type Target = axum::Router;
        fn deref(&self) -> &Self::Target {
            &self.service
        }
    }
    fn manifest() -> Value {
        serde_json::from_slice(fixture_file("fixtures/manifest.json")).unwrap()
    }

    fn exact_keys(value: &Value, keys: &[&str]) {
        let actual: BTreeSet<_> = value
            .as_object()
            .expect("object wrapper")
            .keys()
            .map(String::as_str)
            .collect();
        assert!(
            actual == keys.iter().copied().collect(),
            "unexpected wrapper keys"
        );
    }

    #[derive(Clone, Default)]
    struct Cookies(BTreeMap<String, String>);

    impl Cookies {
        fn header(&self) -> String {
            self.0
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join("; ")
        }

        fn absorb(&mut self, headers: &http::HeaderMap) {
            let mut seen = BTreeSet::new();
            for value in headers.get_all(header::SET_COOKIE) {
                let mut parts = value.to_str().unwrap().split(';').map(str::trim);
                let (name, secret) = parts.next().unwrap().split_once('=').unwrap();
                assert!(
                    [ACCESS, REFRESH, ENROLLMENT, LOGIN].contains(&name),
                    "unexpected cookie name"
                );
                assert!(seen.insert(name.to_owned()), "duplicate Set-Cookie name");
                let mut attrs = BTreeMap::new();
                for part in parts {
                    let (key, value) = part
                        .split_once('=')
                        .map_or((part, None), |(k, v)| (k, Some(v)));
                    assert!(
                        attrs
                            .insert(key.trim().to_ascii_lowercase(), value.map(str::trim))
                            .is_none(),
                        "duplicate cookie attribute"
                    );
                }
                assert_eq!(attrs.get("secure"), Some(&None));
                assert_eq!(attrs.get("httponly"), Some(&None));
                assert_eq!(attrs.get("path"), Some(&Some("/")));
                assert!(!attrs.contains_key("domain"));
                let site = attrs.get("samesite").copied().flatten().unwrap();
                assert!(site.eq_ignore_ascii_case(if name == ACCESS { "lax" } else { "strict" }));
                let age: i64 = attrs
                    .get("max-age")
                    .copied()
                    .flatten()
                    .expect("bounded cookie lifetime")
                    .parse()
                    .unwrap();
                if secret.is_empty() {
                    assert_eq!(age, 0);
                    self.0.remove(name);
                } else {
                    assert!(age > 0);
                    if name == ACCESS {
                        assert!(age <= 900);
                    }
                    if name == ENROLLMENT || name == LOGIN {
                        assert!(age <= 300);
                    }
                    self.0.insert(name.to_owned(), secret.to_owned());
                }
            }
        }
    }

    struct Response {
        status: StatusCode,
        headers: http::HeaderMap,
        bytes: Vec<u8>,
        sent_secrets: Vec<String>,
    }

    impl Response {
        fn json(&self, expected: StatusCode) -> Value {
            assert_eq!(self.status, expected, "BW31 HTTP contract not satisfied");
            serde_json::from_slice(&self.bytes).expect("JSON response, no fallback HTML")
        }

        // Bounded literal echo oracle, not tracing/covert/encoded information-flow proof.
        fn no_literal_echo(&self) {
            let mut secrets = self.sent_secrets.clone();
            for value in self.headers.get_all(header::SET_COOKIE) {
                let pair = value.to_str().unwrap().split(';').next().unwrap();
                if let Some((_, value)) = pair.split_once('=')
                    && !value.is_empty()
                {
                    secrets.push(value.to_owned());
                }
            }
            for secret in secrets.iter().filter(|s| !s.is_empty()) {
                assert!(
                    !self
                        .bytes
                        .windows(secret.len())
                        .any(|w| w == secret.as_bytes()),
                    "literal credential in response body"
                );
                for (name, value) in &self.headers {
                    if self.status.is_success() && name == header::SET_COOKIE {
                        continue;
                    }
                    assert!(
                        !value
                            .as_bytes()
                            .windows(secret.len())
                            .any(|w| w == secret.as_bytes()),
                        "literal credential in response header"
                    );
                }
            }
        }
        fn private(&self) {
            self.no_literal_echo();
            assert!(
                self.headers
                    .get(header::CACHE_CONTROL)
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .split(',')
                    .any(|s| s.trim() == "no-store")
            );
            assert_eq!(self.headers.get(header::PRAGMA).unwrap(), "no-cache");
        }

        fn error(&self, status: StatusCode, code: &str) {
            let value = self.json(status);
            exact_keys(&value, &["error"]);
            exact_keys(&value["error"], &["code", "message"]);
            assert_eq!(value["error"]["code"], code);
            assert!(
                value["error"]["message"]
                    .as_str()
                    .is_some_and(|m| !m.is_empty() && m.len() <= 256)
            );
            assert!(
                !self.headers.contains_key(header::SET_COOKIE),
                "refusal changed browser identity"
            );
            self.private();
        }
    }

    async fn request(
        router: &axum::Router,
        method: &str,
        path: &str,
        cookies: &Cookies,
        body: Option<Value>,
        extra: &[(&str, &str)],
    ) -> Response {
        let mut sent_secrets: Vec<String> = cookies.0.values().cloned().collect();
        for (name, value) in extra {
            if name.eq_ignore_ascii_case("x-console-csrf") && *value != "fetch" {
                sent_secrets.push((*value).to_owned());
            }
            if name.eq_ignore_ascii_case("authorization") {
                sent_secrets.push(value.strip_prefix("Bearer ").unwrap_or(value).to_owned());
            }
            if name.eq_ignore_ascii_case("cookie") {
                for pair in value.split(';') {
                    if let Some((_, secret)) = pair.trim().split_once('=') {
                        sent_secrets.push(secret.to_owned());
                    }
                }
            }
        }
        let mut req = Request::builder()
            .method(method)
            .uri(path)
            .header(header::ORIGIN, TEST_ORIGIN)
            .header("Sec-Fetch-Site", "same-origin");
        if !cookies.0.is_empty() {
            req = req.header(header::COOKIE, cookies.header());
        }
        for (name, value) in extra {
            req = req.header(*name, *value);
        }
        let data = if let Some(body) = body {
            req = req.header(header::CONTENT_TYPE, "application/json");
            Body::from(serde_json::to_vec(&body).unwrap())
        } else {
            Body::empty()
        };
        let mut req = req.body(data).unwrap();
        req.extensions_mut().insert(ConnectInfo(
            "127.0.0.1:41000".parse::<SocketAddr>().unwrap(),
        ));
        let response = router.clone().oneshot(req).await.unwrap();
        let (parts, body) = response.into_parts();
        Response {
            status: parts.status,
            headers: parts.headers,
            bytes: to_bytes(body, 256 * 1024).await.unwrap().to_vec(),
            sent_secrets,
        }
    }

    async fn router(pool: &PgPool, root: PathBuf) -> axum::Router {
        router_with_key(pool, root, &SigningKey::random(&mut OsRng)).await
    }

    async fn router_with_key(
        pool: &PgPool,
        root: PathBuf,
        signing_key: &SigningKey,
    ) -> axum::Router {
        build_router(state_with_key(pool, root, signing_key).await)
    }

    async fn state_with_key(pool: &PgPool, root: PathBuf, signing_key: &SigningKey) -> AppState {
        let mut pairs = vec![
            ("CONSOLE_APP_ROLE", AppRole::Api.to_string()),
            ("CONSOLE_HTTP_ADDR", "127.0.0.1:0".to_owned()),
            ("CONSOLE_JWT_ISSUER", TEST_ISSUER.to_owned()),
            ("CONSOLE_JWT_AUDIENCE", TEST_AUDIENCE.to_owned()),
            (
                "CONSOLE_JWT_PRIVATE_KEY_PEM",
                signing_key
                    .to_pkcs8_pem(LineEnding::LF)
                    .unwrap()
                    .to_string(),
            ),
            (
                "CONSOLE_JWT_PUBLIC_KEY_PEM",
                signing_key
                    .verifying_key()
                    .to_public_key_pem(LineEnding::LF)
                    .unwrap(),
            ),
            ("CONSOLE_WEBAUTHN_RP_ID", "example.com".to_owned()),
            ("CONSOLE_WEBAUTHN_RP_ORIGIN", TEST_ORIGIN.to_owned()),
            ("CONSOLE_WEBAUTHN_RP_NAME", "Console".to_owned()),
            // Trusted internal release-directory wiring, not caller-controlled HTTP.
            // The index lives at fixtures/artifact-index.json under this package root.
            // Config parsing alone is NOT proof: exact served bytes are asserted.
            (
                "CONSOLE_ACCOUNT_TERMS_ARTIFACT_ROOT",
                root.to_str().unwrap().to_owned(),
            ),
        ];
        pairs.extend(account_transport_urls(pool));
        AppState::from_config(AppConfig::from_pairs(pairs).unwrap())
            .await
            .unwrap()
    }

    /// Fixture inserts ONLY the reviewed immutable publication/head data. Runtime
    /// migration owns the actual tables, constraints, functions, roles and grants.
    pub(super) async fn seed_terms(pool: &PgPool) {
        let present: bool = sqlx::query_scalar("SELECT to_regclass('public.account_terms_head') IS NOT NULL AND to_regclass('public.account_terms_release_receipts') IS NOT NULL")
        .fetch_one(pool).await.unwrap();
        assert!(
            present,
            "BW31 terms schema prerequisite missing; enrollment assertions not reached"
        );
        let approval = serde_json::to_vec(&json!({
        "kind":"ACCOUNT_TERMS_PUBLICATION_APPROVAL", "approval_id":"32323232-3232-4232-8232-323232323232",
        "manifest_sha256":MANIFEST_DIGEST, "expected_revision":"0", "next_revision":"1", "fixture_only":true,
        "approved_by":"test_only.operator", "approved_at":"2026-09-13T00:00:00Z",
        "content_authority_refs":[hex::encode(Sha256::digest(b"TEST_ONLY content authority, never legal proof"))]
    })).unwrap();
        let mut tx = pool.begin().await.unwrap();
        sqlx::query("INSERT INTO account_terms_release_receipts (id,previous_revision,revision,manifest_sha256,approved_release_ref,approval_bytes,recorded_at) VALUES ($1,NULL,1,$2,$3,$4,now())")
        .bind(Uuid::parse_str(RECEIPT).unwrap()).bind(hex::decode(MANIFEST_DIGEST).unwrap())
        .bind(json!({"kind":"OPERATOR_RELEASE_APPROVAL","approval_sha256":hex::encode(Sha256::digest(&approval))}))
        .bind(approval).execute(&mut *tx).await.unwrap();
        sqlx::query("INSERT INTO account_terms_head (id,manifest_sha256,revision,release_receipt_ref,updated_at) VALUES (1,$1,1,$2,now())")
        .bind(hex::decode(MANIFEST_DIGEST).unwrap()).bind(Uuid::parse_str(RECEIPT).unwrap()).execute(&mut *tx).await.unwrap();
        tx.commit().await.unwrap();
    }

    // Native reference/immutable-data tests only. TEST_ONLY rows below are not
    // publication approval or an implementation of the future publisher.
    fn terms_reference_approval(revision: i64) -> Vec<u8> {
        serde_json::to_vec(&json!({
                "kind":"ACCOUNT_TERMS_PUBLICATION_APPROVAL",
                "approval_id":"32323232-3232-4232-8232-323232323232",
                "manifest_sha256":MANIFEST_DIGEST,
                "expected_revision":(revision - 1).to_string(),
                "next_revision":revision.to_string(),
                "fixture_only":true,
                "approved_by":"test_only.operator",
                "approved_at":"2026-09-13T00:00:00Z",
                "content_authority_refs":[hex::encode(Sha256::digest(b"TEST_ONLY content authority, never legal proof"))]
            })).unwrap()
    }

    pub(super) async fn terms_reference_rows(pool: &PgPool) -> Value {
        // Complete isolated-fixture rows, including unrelated Account/audit data.
        // Never print this snapshot or its retained approval bytes.
        sqlx::query_scalar(r#"SELECT jsonb_build_object(
                'receipts',COALESCE((SELECT jsonb_agg(to_jsonb(r) ORDER BY r.id) FROM public.account_terms_release_receipts r),'[]'::jsonb),
                'head',COALESCE((SELECT jsonb_agg(to_jsonb(h) ORDER BY h.id) FROM public.account_terms_head h),'[]'::jsonb),
                'acceptances',COALESCE((SELECT jsonb_agg(to_jsonb(a) ORDER BY a.account_id,a.terms_kind,a.terms_version) FROM public.account_terms_acceptances a),'[]'::jsonb),
                'accounts',COALESCE((SELECT jsonb_agg(to_jsonb(a) ORDER BY a.id) FROM public.accounts a),'[]'::jsonb),
                'security',COALESCE((SELECT jsonb_agg(to_jsonb(s) ORDER BY s.account_id) FROM public.account_security s),'[]'::jsonb),
                'events',COALESCE((SELECT jsonb_agg(to_jsonb(e) ORDER BY e.id) FROM public.account_security_events e),'[]'::jsonb),
                'audit',COALESCE((SELECT jsonb_agg(to_jsonb(a) ORDER BY a.id) FROM public.audit_events a),'[]'::jsonb))"#)
                .fetch_one(pool).await.expect("complete terms reference fixture readback")
    }

    fn terms_reference_guard_error(result: Result<sqlx::postgres::PgQueryResult, sqlx::Error>) {
        let error = match result {
            Ok(done) => {
                assert_eq!(
                    done.rows_affected(),
                    1,
                    "baseline must update the matched receipt"
                );
                panic!("TERMS_GUARD_PREWRITE: matched receipt UPDATE succeeded; affected=1");
            }
            Err(error) => error,
        };
        let database = error
            .as_database_error()
            .expect("guard must be PostgreSQL refusal");
        assert!(
            database.code().as_deref() == Some("P0001"),
            "guard SQLSTATE must be exact"
        );
        assert!(
            database.message() == "account_terms_receipts.immutable",
            "fixed immutable guard message only"
        );
    }

    #[sqlx::test(migrations = false)]
    async fn terms_reference_native_fk_preserves_exact_receipt(pool: PgPool) {
        prepare_http_database(&pool).await;
        // The current baseline must reach the actual positive head INSERT42501.
        // This existing data-only fixture stays byte-identical in both variants.
        seed_terms(&pool).await;
        let id = Uuid::parse_str(RECEIPT).unwrap();
        let digest = hex::decode(MANIFEST_DIGEST).unwrap();
        let approval = terms_reference_approval(1);
        let receipt: (Uuid, Option<i64>, i64, Vec<u8>, Vec<u8>, Value) = sqlx::query_as(
                "SELECT id,previous_revision,revision,manifest_sha256,approval_bytes,approved_release_ref FROM public.account_terms_release_receipts WHERE id=$1",
            ).bind(id).fetch_one(&pool).await.unwrap();
        assert!(receipt.0 == id && receipt.1.is_none() && receipt.2 == 1);
        assert!(
            receipt.3 == digest && receipt.4 == approval,
            "original digest and approval bytes must be exact"
        );
        assert!(
            receipt.5
                == json!({"kind":"OPERATOR_RELEASE_APPROVAL","approval_sha256":hex::encode(Sha256::digest(&approval))})
        );
        let head: (i16, Uuid, i64, Vec<u8>) = sqlx::query_as(
            "SELECT id,release_receipt_ref,revision,manifest_sha256 FROM public.account_terms_head",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(head.0 == 1 && head.1 == id && head.2 == 1 && head.3 == digest);
        let before = terms_reference_rows(&pool).await;
        assert_eq!(before["receipts"].as_array().unwrap().len(), 1);
        assert_eq!(before["head"].as_array().unwrap().len(), 1);
        assert!(before["acceptances"].as_array().unwrap().is_empty());
        // Pin the actual validated0226 composite FK, not a generic SQL error.
        let constraint: String = sqlx::query_scalar(
                "SELECT conname::text FROM pg_catalog.pg_constraint WHERE conrelid='public.account_terms_head'::regclass AND confrelid='public.account_terms_release_receipts'::regclass AND contype='f' AND convalidated AND NOT condeferrable AND NOT condeferred AND conkey=ARRAY[4,3,2]::smallint[] AND confkey=ARRAY[1,3,4]::smallint[] AND confdeltype='r'",
            ).fetch_one(&pool).await.expect("exact native receipt-reference constraint");
        let mut wrong_digest = digest.clone();
        wrong_digest[0] ^= 1;
        for (candidate_id, revision, candidate_digest) in [
            (Uuid::new_v4(), 1_i64, digest.clone()),
            (id, 2_i64, digest.clone()),
            (id, 1_i64, wrong_digest),
        ] {
            let mut tx = pool.begin().await.unwrap();
            let result = sqlx::query("UPDATE public.account_terms_head SET release_receipt_ref=$1,revision=$2,manifest_sha256=$3 WHERE id=1")
                    .bind(candidate_id).bind(revision).bind(candidate_digest).execute(&mut *tx).await;
            tx.rollback().await.unwrap();
            assert!(
                before == terms_reference_rows(&pool).await,
                "failed native FK must preserve all fixture rows and receipt bytes"
            );
            let error = result.expect_err("mismatched reference must be refused");
            let database = error
                .as_database_error()
                .expect("native FK PostgreSQL refusal");
            assert!(
                database.code().as_deref() == Some("23503"),
                "mismatch must reach native FK, not privilege/type/CHECK failure"
            );
            assert!(
                database.constraint() == Some(constraint.as_str()),
                "refusal must identify the exact head receipt-reference FK"
            );
        }
    }

    #[sqlx::test(migrations = false)]
    async fn terms_reference_receipt_guard_refuses_real_mutations(pool: PgPool) {
        prepare_http_database(&pool).await;
        let target = Uuid::parse_str(RECEIPT).unwrap();
        let next = Uuid::new_v4();
        for (id, previous, revision) in [(target, None, 1_i64), (next, Some(1_i64), 2_i64)] {
            let approval = terms_reference_approval(revision);
            sqlx::query("INSERT INTO public.account_terms_release_receipts(id,previous_revision,revision,manifest_sha256,approved_release_ref,approval_bytes,recorded_at) VALUES($1,$2,$3,$4,$5,$6,now())")
                    .bind(id).bind(previous).bind(revision).bind(hex::decode(MANIFEST_DIGEST).unwrap())
                    .bind(json!({"kind":"OPERATOR_RELEASE_APPROVAL","approval_sha256":hex::encode(Sha256::digest(&approval))}))
                    .bind(approval).execute(&pool).await.expect("data-only unreferenced receipt fixture");
        }
        let before = terms_reference_rows(&pool).await;
        assert_eq!(before["receipts"].as_array().unwrap().len(), 2);
        assert!(before["head"].as_array().unwrap().is_empty());
        assert!(before["acceptances"].as_array().unwrap().is_empty());
        let identity: (String, String, bool) = sqlx::query_as(
                "SELECT session_user::text,current_user::text,(SELECT rolsuper FROM pg_catalog.pg_roles WHERE rolname=current_user)",
            ).fetch_one(&pool).await.unwrap();
        assert!(
            identity.0 == "console_buck_admin" && identity.1 == identity.0 && identity.2,
            "real marked fixture administrator required"
        );

        // FIRST on baseline and future implementation: real privileged matched
        // no-op UPDATE. Owner-context42501 must not hide the missing guard RED.
        let mut tx = pool.begin().await.unwrap();
        let result =
            sqlx::query("UPDATE public.account_terms_release_receipts SET id=id WHERE id=$1")
                .bind(target)
                .execute(&mut *tx)
                .await;
        tx.rollback().await.unwrap();
        assert!(
            before == terms_reference_rows(&pool).await,
            "rollback preserves original receipt rows and bytes"
        );
        terms_reference_guard_error(result);

        for replacement in [target, Uuid::new_v4()] {
            let mut tx = pool.begin().await.unwrap();
            // Explicit owner-rights probe from fixture administrator, not a claim
            // to authenticate the NOLOGIN owner as a serving transport.
            sqlx::query("SET LOCAL ROLE console_terms_owner")
                .execute(&mut *tx)
                .await
                .unwrap();
            let owner: (String, String, bool, bool) = sqlx::query_as(
                    "SELECT session_user::text,current_user::text,has_column_privilege(current_user,'public.account_terms_release_receipts','id','SELECT'),has_column_privilege(current_user,'public.account_terms_release_receipts','id','UPDATE')",
                ).fetch_one(&mut *tx).await.unwrap();
            assert!(
                owner.0 == "console_buck_admin"
                    && owner.1 == "console_terms_owner"
                    && owner.2
                    && owner.3
            );
            let result =
                sqlx::query("UPDATE public.account_terms_release_receipts SET id=$2 WHERE id=$1")
                    .bind(target)
                    .bind(replacement)
                    .execute(&mut *tx)
                    .await;
            tx.rollback().await.unwrap();
            assert!(
                before == terms_reference_rows(&pool).await,
                "owner mutation refusal preserves all rows"
            );
            terms_reference_guard_error(result);
        }
        for statement in [
            "DELETE FROM public.account_terms_release_receipts WHERE id='31313131-3131-4131-8131-313131313131'::uuid",
            "TRUNCATE public.account_terms_release_receipts, public.account_terms_head, public.account_terms_acceptances",
        ] {
            let mut tx = pool.begin().await.unwrap();
            sqlx::query("SET LOCAL ROLE console_terms_owner")
                .execute(&mut *tx)
                .await
                .unwrap();
            let result = sqlx::query(statement).execute(&mut *tx).await;
            tx.rollback().await.unwrap();
            assert!(
                before == terms_reference_rows(&pool).await,
                "owner privilege refusal preserves all rows"
            );
            let error = result.expect_err("minimal owner has no DELETE or TRUNCATE privilege");
            assert!(
                error.as_database_error().and_then(|e| e.code()).as_deref() == Some("42501"),
                "owner privilege refusal cannot substitute for guard proof"
            );
        }
        for statement in [
            "DELETE FROM public.account_terms_release_receipts WHERE id='31313131-3131-4131-8131-313131313131'::uuid",
            // PostgreSQL checks referencing-table closure before BEFORE TRUNCATE.
            // All three0226 relations are explicit; no CASCADE or fixture grants.
            "TRUNCATE public.account_terms_release_receipts, public.account_terms_head, public.account_terms_acceptances",
            "UPDATE public.account_terms_release_receipts SET id=id WHERE false",
            "DELETE FROM public.account_terms_release_receipts WHERE false",
        ] {
            let mut tx = pool.begin().await.unwrap();
            let result = sqlx::query(statement).execute(&mut *tx).await;
            tx.rollback().await.unwrap();
            assert!(
                before == terms_reference_rows(&pool).await,
                "unconditional statement guard preserves all rows"
            );
            let error = result.expect_err("statement guard must refuse even zero-match mutations");
            let database = error.as_database_error().expect("guard PostgreSQL refusal");
            assert!(database.code().as_deref() == Some("P0001"));
            assert!(database.message() == "account_terms_receipts.immutable");
        }
        // ENABLE ALWAYS must retain the guard under transaction-local replica
        // mode. Hold one real connection so restoration readback is not pooled.
        let mut connection = pool.acquire().await.unwrap();
        let original: String = sqlx::query_scalar("SHOW session_replication_role")
            .fetch_one(&mut *connection)
            .await
            .unwrap();
        assert_eq!(original, "origin");
        let mut tx = sqlx::Connection::begin(&mut *connection).await.unwrap();
        sqlx::query("SET LOCAL session_replication_role=replica")
            .execute(&mut *tx)
            .await
            .unwrap();
        let result =
            sqlx::query("UPDATE public.account_terms_release_receipts SET id=id WHERE id=$1")
                .bind(target)
                .execute(&mut *tx)
                .await;
        tx.rollback().await.unwrap();
        let restored: String = sqlx::query_scalar("SHOW session_replication_role")
            .fetch_one(&mut *connection)
            .await
            .unwrap();
        assert_eq!(restored, original);
        assert!(
            before == terms_reference_rows(&pool).await,
            "replica-mode refusal preserves all rows"
        );
        terms_reference_guard_error(result);
    }

    async fn fixture(pool: &PgPool) -> Fixture {
        prepare_http_database(pool).await;
        seed_terms(pool).await;
        let artifacts = Artifacts::new();
        let service = router(pool, artifacts.root.clone()).await;
        Fixture {
            service,
            _artifacts: artifacts,
            pool: pool.clone(),
        }
    }

    struct Attempt {
        account: Uuid,
        ceremony: Uuid,
        cookies: Cookies,
        finish: Value,
        authenticator: WebauthnAuthenticator<SoftPasskey>,
    }

    async fn start(router: &Fixture) -> Attempt {
        let response = request(
            router,
            "POST",
            "/api/v2/auth/registration/start",
            &Cookies::default(),
            Some(json!({"terms_version":MANIFEST_DIGEST})),
            &[],
        )
        .await;
        let start = response.json(StatusCode::OK);
        response.private();
        exact_keys(&start, &["ceremony_id", "public_key_options"]);
        let options = &start["public_key_options"];
        exact_keys(options, &["publicKey"]);
        assert_eq!(
            options["publicKey"]["authenticatorSelection"]["residentKey"],
            "required"
        );
        assert_eq!(
            options["publicKey"]["authenticatorSelection"]["requireResidentKey"],
            true
        );
        assert_eq!(
            options["publicKey"]["authenticatorSelection"]["userVerification"],
            "required"
        );
        let account = Uuid::from_slice(
            &URL_SAFE_NO_PAD
                .decode(options["publicKey"]["user"]["id"].as_str().unwrap())
                .unwrap(),
        )
        .unwrap();
        assert!(!account.is_nil());
        let pending = snapshot(&router.pool, account).await;
        assert_eq!(pending["security"]["security_state"], "PENDING_ENROLLMENT");
        for field in ["keys", "families", "tokens", "terms"] {
            assert_eq!(
                pending[field],
                json!([]),
                "registration start persisted authentication material"
            );
        }
        assert!(
            !pending["events"]
                .as_array()
                .unwrap()
                .iter()
                .any(|e| e["kind"] == "TERMS_ACCEPTED" || e["kind"] == "ENROLLED")
        );
        assert_no_company_identity(&router.pool, account).await;
        let mut cookies = Cookies::default();
        cookies.absorb(&response.headers);
        assert!(
            cookies.0.len() == 1 && cookies.0.contains_key(ENROLLMENT),
            "start minted a session"
        );
        // Explicit CLIENT-ONLY accommodation: SoftPasskey cannot store resident keys.
        // Server challenge/state/signature inputs remain untouched. Never ship this
        // adjustment in the browser or describe this as resident-authenticator proof.
        let mut adapted = options.clone();
        adapted["publicKey"]["authenticatorSelection"]["residentKey"] = json!("discouraged");
        adapted["publicKey"]["authenticatorSelection"]["requireResidentKey"] = json!(false);
        let challenge: CreationChallengeResponse = serde_json::from_value(adapted).unwrap();
        let mut authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
        let credential = authenticator
            .do_registration(Url::parse(TEST_ORIGIN).unwrap(), challenge)
            .unwrap();
        let ceremony = Uuid::parse_str(start["ceremony_id"].as_str().unwrap()).unwrap();
        let acknowledgments: Vec<_> = manifest()["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| json!({"terms_kind":i["terms_kind"],"accepted":true}))
            .collect();
        Attempt {
            account,
            ceremony,
            cookies,
            finish: json!({"ceremony_id":ceremony,"credential":credential,"accept_terms_version":MANIFEST_DIGEST,"accept_items":acknowledgments}),
            authenticator,
        }
    }

    fn projection(value: &Value, account: Uuid) {
        exact_keys(value, &["account_id", "session", "permitted_self_actions"]);
        assert_eq!(value["account_id"], account.to_string());
        exact_keys(&value["session"], &["assurance", "expires_at"]);
        assert_eq!(value["session"]["assurance"], "PASSKEY_PRIMARY");
        let expiry = OffsetDateTime::parse(
            value["session"]["expires_at"].as_str().unwrap(),
            &time::format_description::well_known::Rfc3339,
        )
        .unwrap();
        assert!(
            expiry > OffsetDateTime::now_utc()
                && expiry <= OffsetDateTime::now_utc() + Duration::minutes(15)
        );
        assert_eq!(
            value["permitted_self_actions"],
            json!([{"action_key":"account.session.logout","registration_revision":"1"}])
        );
    }

    fn session(response: &Response, status: StatusCode, account: Uuid, cookies: &mut Cookies) {
        let value = response.json(status);
        response.private();
        exact_keys(&value, &["account"]);
        projection(&value["account"], account);
        cookies.absorb(&response.headers);
        assert!(cookies.0.contains_key(ACCESS) && cookies.0.contains_key(REFRESH));
        assert!(!cookies.0.contains_key(ENROLLMENT) && !cookies.0.contains_key(LOGIN));
        for secret in cookies.0.values() {
            assert!(
                !response
                    .bytes
                    .windows(secret.len())
                    .any(|w| w == secret.as_bytes()),
                "credential in response body"
            );
        }
    }

    async fn finish(router: &axum::Router, attempt: &Attempt) -> Response {
        request(
            router,
            "POST",
            "/api/v2/auth/registration/finish",
            &attempt.cookies,
            Some(attempt.finish.clone()),
            &[],
        )
        .await
    }

    async fn enrolled(router: &Fixture) -> (Attempt, Cookies) {
        let attempt = start(router).await;
        let response = finish(router, &attempt).await;
        let mut cookies = attempt.cookies.clone();
        session(
            &response,
            StatusCode::CREATED,
            attempt.account,
            &mut cookies,
        );
        (attempt, cookies)
    }

    /// Snapshot carries no plaintext token. Assert equality without printing rows.
    async fn snapshot(pool: &PgPool, account: Uuid) -> Value {
        sqlx::query_scalar("SELECT jsonb_build_object('security',(SELECT to_jsonb(s) FROM account_security s WHERE account_id=$1), 'keys',(SELECT coalesce(jsonb_agg(to_jsonb(k) ORDER BY id),'[]') FROM auth_webauthn_credentials k WHERE user_id=$1), 'families',(SELECT coalesce(jsonb_agg(to_jsonb(f) ORDER BY id),'[]') FROM auth_refresh_token_families f WHERE user_id=$1), 'tokens',(SELECT coalesce(jsonb_agg(to_jsonb(t) ORDER BY id),'[]') FROM auth_refresh_tokens t WHERE user_id=$1), 'terms',(SELECT coalesce(jsonb_agg(to_jsonb(a) ORDER BY terms_kind),'[]') FROM account_terms_acceptances a WHERE account_id=$1), 'ceremonies',(SELECT coalesce(jsonb_agg(to_jsonb(c) ORDER BY id),'[]') FROM auth_webauthn_ceremonies c WHERE user_id=$1), 'events',(SELECT coalesce(jsonb_agg(to_jsonb(e) ORDER BY id),'[]') FROM account_security_events e WHERE account_id=$1))")
        .bind(account).fetch_one(pool).await.unwrap()
    }

    async fn assert_committed(pool: &PgPool, attempt: &Attempt) {
        let value = snapshot(pool, attempt.account).await;
        assert_eq!(value["security"]["security_state"], "ACTIVE");
        assert_eq!(value["keys"].as_array().unwrap().len(), 1);
        assert_eq!(value["families"].as_array().unwrap().len(), 1);
        assert_eq!(value["families"][0]["protocol"], "ACCOUNT_V1");
        assert_eq!(value["terms"].as_array().unwrap().len(), 2);
        for item in manifest()["items"].as_array().unwrap() {
            let terms = value["terms"]
                .as_array()
                .unwrap()
                .iter()
                .find(|a| a["terms_kind"] == item["terms_kind"])
                .unwrap();
            assert_eq!(terms["terms_version"], MANIFEST_DIGEST);
            assert_eq!(terms["terms_release_receipt_id"], RECEIPT);
            assert_eq!(terms["terms_release_revision"], 1);
            assert_eq!(
                terms["terms_manifest_sha256"],
                format!("\\x{MANIFEST_DIGEST}")
            );
            assert_eq!(
                terms["content_sha256"],
                format!("\\x{}", item["content_sha256"].as_str().unwrap())
            );
            let event = value["events"]
                .as_array()
                .unwrap()
                .iter()
                .find(|e| e["id"] == terms["security_event_id"])
                .unwrap();
            assert_eq!(event["account_id"], attempt.account.to_string());
            assert_eq!(event["kind"], "TERMS_ACCEPTED");
            let expected = json!({"kind":"ACCOUNT_TERMS_RELEASE","receipt_id":RECEIPT,"revision":"1","manifest_sha256":MANIFEST_DIGEST});
            assert_eq!(event["evidence_ref"], expected);
            assert_eq!(event["payload"]["evidence"], expected);
            assert!(terms["accepted_at"].as_str().is_some());
        }
        let consumed: bool = sqlx::query_scalar(
        "SELECT consumed_at IS NOT NULL FROM auth_webauthn_ceremonies WHERE id=$1 AND user_id=$2",
    )
    .bind(attempt.ceremony)
    .bind(attempt.account)
    .fetch_one(pool)
    .await
    .unwrap();
        assert!(consumed);
        assert_no_company_identity(pool, attempt.account).await;
    }

    async fn assert_no_company_identity(pool: &PgPool, account: Uuid) {
        let invented: i64 = sqlx::query_scalar("SELECT (SELECT count(*) FROM users WHERE id=$1) + (SELECT count(*) FROM company_actors WHERE account_id=$1)")
        .bind(account).fetch_one(pool).await.unwrap();
        assert_eq!(invented, 0, "Account enrollment invented Company identity");
    }

    async fn proof(router: &axum::Router, cookies: &Cookies) -> String {
        let response = request(
            router,
            "GET",
            "/api/v2/auth/csrf",
            cookies,
            None,
            &[("X-Console-CSRF", "fetch")],
        )
        .await;
        let value = response.json(StatusCode::OK);
        response.private();
        exact_keys(&value, &["csrf_proof", "expires_at"]);
        assert!(!response.headers.contains_key(header::SET_COOKIE));
        value["csrf_proof"].as_str().unwrap().to_owned()
    }

    #[sqlx::test(migrations = false)]
    async fn public_terms_without_authority_returns_503(pool: PgPool) {
        prepare_http_database(&pool).await;
        let artifacts = Artifacts::new();
        let router = router(&pool, artifacts.root.clone()).await;
        request(
            &router,
            "GET",
            "/api/v2/auth/terms",
            &Cookies::default(),
            None,
            &[],
        )
        .await
        .error(StatusCode::SERVICE_UNAVAILABLE, "authority_unavailable");
    }

    #[sqlx::test(migrations = false)]
    async fn company_free_registration_me_and_empty_contexts(pool: PgPool) {
        let router = fixture(&pool).await;
        let before: (i64, i64, i64) = sqlx::query_as("SELECT (SELECT count(*) FROM organizations),(SELECT count(*) FROM employees),(SELECT count(*) FROM persons)").fetch_one(&pool).await.unwrap();
        let terms = request(
            &router,
            "GET",
            "/api/v2/auth/terms",
            &Cookies::default(),
            None,
            &[],
        )
        .await;
        let value = terms.json(StatusCode::OK);
        terms.private();
        exact_keys(&value, &["terms_version", "terms_revision", "manifest_url"]);
        assert_eq!(value["terms_version"], MANIFEST_DIGEST);
        assert_eq!(value["terms_revision"], "1");
        assert_eq!(
            value["manifest_url"],
            format!("/api/v2/auth/terms/manifests/{MANIFEST_DIGEST}")
        );
        let index: Value =
            serde_json::from_slice(fixture_file("fixtures/artifact-index.json")).unwrap();
        let mut artifacts = vec![index["manifest"].clone()];
        artifacts.extend(index["content"].as_array().unwrap().clone());
        for artifact in artifacts {
            let digest = artifact["sha256"].as_str().unwrap();
            let is_manifest = digest == MANIFEST_DIGEST;
            let path = format!(
                "/api/v2/auth/terms/{}/{digest}",
                if is_manifest { "manifests" } else { "content" }
            );
            let response = request(&router, "GET", &path, &Cookies::default(), None, &[]).await;
            assert_eq!(response.status, StatusCode::OK);
            let expected = fixture_file(artifact["path"].as_str().unwrap());
            assert!(
                response.bytes == expected,
                "configured release artifact bytes differ"
            );
            assert_eq!(hex::encode(Sha256::digest(&response.bytes)), digest);
            assert_eq!(
                response.headers.get(header::CONTENT_TYPE).unwrap(),
                if is_manifest {
                    "application/json; charset=utf-8"
                } else {
                    "text/plain; charset=utf-8"
                }
            );
            if !is_manifest {
                assert_eq!(
                    response.headers.get("x-content-type-options").unwrap(),
                    "nosniff"
                );
            }
        }
        let (attempt, cookies) = enrolled(&router).await;
        assert_committed(&pool, &attempt).await;
        let me = request(&router, "GET", "/api/v2/accounts/me", &cookies, None, &[]).await;
        projection(&me.json(StatusCode::OK), attempt.account);
        me.private();
        assert_no_company_identity(&pool, attempt.account).await;
        let contexts = request(
            &router,
            "GET",
            "/api/v2/accounts/me/contexts",
            &cookies,
            None,
            &[],
        )
        .await;
        assert_eq!(contexts.json(StatusCode::OK), json!({"contexts":[]}));
        contexts.private();
        assert_no_company_identity(&pool, attempt.account).await;
        let after: (i64, i64, i64) = sqlx::query_as("SELECT (SELECT count(*) FROM organizations),(SELECT count(*) FROM employees),(SELECT count(*) FROM persons)").fetch_one(&pool).await.unwrap();
        assert_eq!(before, after);
    }

    #[sqlx::test(migrations = false)]
    async fn acknowledgment_errors_leave_valid_attempt_completable(pool: PgPool) {
        let router = fixture(&pool).await;
        let attempt = start(&router).await;
        let before = snapshot(&pool, attempt.account).await;
        let accepted = attempt.finish["accept_items"].as_array().unwrap();
        let variants = [
            json!([]),
            json!([accepted[0]]),
            json!([accepted[0], accepted[0]]),
            json!([{"terms_kind":"unknown","accepted":true}]),
            json!([{"terms_kind":accepted[0]["terms_kind"],"accepted":false},accepted[1]]),
        ];
        for items in variants {
            let mut body = attempt.finish.clone();
            body["accept_items"] = items;
            request(
                &router,
                "POST",
                "/api/v2/auth/registration/finish",
                &attempt.cookies,
                Some(body),
                &[],
            )
            .await
            .error(StatusCode::UNPROCESSABLE_ENTITY, "invalid_request");
            assert!(
                snapshot(&pool, attempt.account).await == before,
                "invalid item acknowledgment partially committed"
            );
        }
        let response = finish(&router, &attempt).await;
        session(
            &response,
            StatusCode::CREATED,
            attempt.account,
            &mut attempt.cookies.clone(),
        );
        assert_committed(&pool, &attempt).await;
    }

    #[sqlx::test(migrations = false)]
    async fn binding_errors_do_not_consume_valid_attempt(pool: PgPool) {
        let router = fixture(&pool).await;
        let attempt = start(&router).await;
        let other = start(&router).await;
        let before = snapshot(&pool, attempt.account).await;
        let before_other = snapshot(&pool, other.account).await;
        for cookies in [&Cookies::default(), &other.cookies] {
            request(
                &router,
                "POST",
                "/api/v2/auth/registration/finish",
                cookies,
                Some(attempt.finish.clone()),
                &[],
            )
            .await
            .error(StatusCode::UNAUTHORIZED, "enrollment_invalid");
            assert!(snapshot(&pool, attempt.account).await == before);
            assert!(snapshot(&pool, other.account).await == before_other);
        }
        // A duplicate Origin is invalid too; request() already supplies the valid one.
        request(
            &router,
            "POST",
            "/api/v2/auth/registration/finish",
            &attempt.cookies,
            Some(attempt.finish.clone()),
            &[("Origin", "https://attacker.invalid")],
        )
        .await
        .error(StatusCode::FORBIDDEN, "request_origin_denied");
        let mut tampered = attempt.finish.clone();
        tampered["credential"]["response"]["clientDataJSON"] =
            other.finish["credential"]["response"]["clientDataJSON"].clone();
        request(
            &router,
            "POST",
            "/api/v2/auth/registration/finish",
            &attempt.cookies,
            Some(tampered),
            &[],
        )
        .await
        .error(StatusCode::UNAUTHORIZED, "enrollment_invalid");
        assert!(snapshot(&pool, attempt.account).await == before);
        assert!(snapshot(&pool, other.account).await == before_other);
        session(
            &finish(&router, &attempt).await,
            StatusCode::CREATED,
            attempt.account,
            &mut attempt.cookies.clone(),
        );
        assert_committed(&pool, &attempt).await;
        session(
            &finish(&router, &other).await,
            StatusCode::CREATED,
            other.account,
            &mut other.cookies.clone(),
        );
        assert_committed(&pool, &other).await;
    }

    #[sqlx::test(migrations = false)]
    async fn concurrent_same_attempt_commits_once_and_replay_cannot_remint(pool: PgPool) {
        let router = fixture(&pool).await;
        let attempt = start(&router).await;
        // Credential exists at the client but no finish was delivered: safe retry.
        let (left, right) = tokio::join!(finish(&router, &attempt), finish(&router, &attempt));
        let (success, refusal) = if left.status == StatusCode::CREATED {
            (left, right)
        } else {
            (right, left)
        };
        session(
            &success,
            StatusCode::CREATED,
            attempt.account,
            &mut attempt.cookies.clone(),
        );
        refusal.error(StatusCode::UNAUTHORIZED, "enrollment_invalid");
        assert_committed(&pool, &attempt).await;
        let before = snapshot(&pool, attempt.account).await;
        finish(&router, &attempt)
            .await
            .error(StatusCode::UNAUTHORIZED, "enrollment_invalid");
        assert!(
            snapshot(&pool, attempt.account).await == before,
            "replay changed committed evidence/session"
        );
    }

    async fn login_attempt(router: &axum::Router, attempt: &mut Attempt) -> (Cookies, Value) {
        let response = request(
            router,
            "POST",
            "/api/v2/auth/passkey/login/start",
            &Cookies::default(),
            Some(json!({})),
            &[],
        )
        .await;
        let value = response.json(StatusCode::OK);
        response.private();
        exact_keys(&value, &["ceremony_id", "public_key_options"]);
        assert_eq!(value["public_key_options"]["mediation"], "conditional");
        assert_eq!(
            value["public_key_options"]["publicKey"]["allowCredentials"],
            json!([])
        );
        let challenge: RequestChallengeResponse =
            serde_json::from_value(value["public_key_options"].clone()).unwrap();
        // CLIENT-ONLY SoftPasskey accommodations: known allow-list + unsigned handle.
        // Actual signature/server ceremony are unchanged; no discovery proof claimed.
        let challenge = inject_allow_credential(
            challenge,
            attempt.finish["credential"]["id"].as_str().unwrap(),
        );
        let assertion = attempt
            .authenticator
            .do_authentication(Url::parse(TEST_ORIGIN).unwrap(), challenge)
            .unwrap();
        let mut assertion = serde_json::to_value(assertion).unwrap();
        assertion["response"]["userHandle"] =
            json!(URL_SAFE_NO_PAD.encode(attempt.account.as_bytes()));
        let mut cookies = Cookies::default();
        cookies.absorb(&response.headers);
        assert!(cookies.0.len() == 1 && cookies.0.contains_key(LOGIN));
        (
            cookies,
            json!({"ceremony_id":value["ceremony_id"],"assertion":assertion}),
        )
    }

    #[sqlx::test(migrations = false)]
    async fn lost_finish_response_recovers_same_account_and_rejects_swapped_handle(pool: PgPool) {
        let router = fixture(&pool).await;
        let mut attempt = start(&router).await;
        let committed = finish(&router, &attempt).await;
        assert_eq!(committed.status, StatusCode::CREATED);
        drop(committed); // Intentionally lose all session cookies and response bytes.
        let (cookies, input) = login_attempt(&router, &mut attempt).await;
        let (other, other_cookies) = enrolled(&router).await;
        projection(
            &request(
                &router,
                "GET",
                "/api/v2/accounts/me",
                &other_cookies,
                None,
                &[],
            )
            .await
            .json(StatusCode::OK),
            other.account,
        );
        let before_other = snapshot(&pool, other.account).await;
        let login_before: Option<OffsetDateTime> =
            sqlx::query_scalar("SELECT consumed_at FROM auth_webauthn_ceremonies WHERE id=$1")
                .bind(Uuid::parse_str(input["ceremony_id"].as_str().unwrap()).unwrap())
                .fetch_one(&pool)
                .await
                .unwrap();
        assert!(login_before.is_none());
        let before = snapshot(&pool, attempt.account).await;
        let mut swapped = input.clone();
        swapped["assertion"]["response"]["userHandle"] =
            json!(URL_SAFE_NO_PAD.encode(other.account.as_bytes()));
        request(
            &router,
            "POST",
            "/api/v2/auth/passkey/login/finish",
            &cookies,
            Some(swapped),
            &[],
        )
        .await
        .error(StatusCode::UNAUTHORIZED, "authentication_invalid");
        assert!(
            snapshot(&pool, attempt.account).await == before,
            "unsigned handle influenced credential owner"
        );
        assert!(snapshot(&pool, other.account).await == before_other);
        let login_after: Option<OffsetDateTime> =
            sqlx::query_scalar("SELECT consumed_at FROM auth_webauthn_ceremonies WHERE id=$1")
                .bind(Uuid::parse_str(input["ceremony_id"].as_str().unwrap()).unwrap())
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(login_after, login_before);
        let response = request(
            &router,
            "POST",
            "/api/v2/auth/passkey/login/finish",
            &cookies,
            Some(input),
            &[],
        )
        .await;
        let mut restored = cookies.clone();
        session(&response, StatusCode::OK, attempt.account, &mut restored);
        projection(
            &request(&router, "GET", "/api/v2/accounts/me", &restored, None, &[])
                .await
                .json(StatusCode::OK),
            attempt.account,
        );
        let keys: i64 =
            sqlx::query_scalar("SELECT count(*) FROM auth_webauthn_credentials WHERE user_id=$1")
                .bind(attempt.account)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(keys, 1);
    }

    #[sqlx::test(migrations = false)]
    async fn csrf_refresh_and_logout_preserve_family_then_revoke(pool: PgPool) {
        let router = fixture(&pool).await;
        let (attempt, mut cookies) = enrolled(&router).await;
        let before = snapshot(&pool, attempt.account).await;
        let proof1 = proof(&router, &cookies).await;
        let proof2 = proof(&router, &cookies).await;
        assert!(
            snapshot(&pool, attempt.account).await == before,
            "proof GET mutated security state"
        );
        request(
            &router,
            "POST",
            "/api/v2/auth/logout",
            &cookies,
            Some(json!({})),
            &[],
        )
        .await
        .error(StatusCode::FORBIDDEN, "csrf_invalid");
        assert!(snapshot(&pool, attempt.account).await == before);
        let old_refresh = cookies.0[REFRESH].clone();
        let refreshed = request(
            &router,
            "POST",
            "/api/v2/auth/token/refresh",
            &cookies,
            Some(json!({})),
            &[("X-Console-CSRF", &proof1)],
        )
        .await;
        session(&refreshed, StatusCode::OK, attempt.account, &mut cookies);
        assert!(
            cookies.0[REFRESH] != old_refresh,
            "refresh cookie did not rotate"
        );
        let after = snapshot(&pool, attempt.account).await;
        assert!(
            before["families"] == after["families"],
            "refresh extended or replaced primary family"
        );
        let response = request(
            &router,
            "POST",
            "/api/v2/auth/logout",
            &cookies,
            Some(json!({})),
            &[("X-Console-CSRF", &proof2)],
        )
        .await;
        assert_eq!(
            response.json(StatusCode::OK),
            json!({"outcome":"COMMITTED"})
        );
        response.private();
        let revoked_cookies = cookies.clone();
        cookies.absorb(&response.headers);
        assert!(cookies.0.is_empty());
        request(
            &router,
            "GET",
            "/api/v2/accounts/me",
            &revoked_cookies,
            None,
            &[],
        )
        .await
        .error(StatusCode::UNAUTHORIZED, "authentication_invalid");
        let revoked = snapshot(&pool, attempt.account).await;
        assert!(!revoked["families"][0]["revoked_at"].is_null());
        request(
            &router,
            "POST",
            "/api/v2/auth/logout",
            &revoked_cookies,
            Some(json!({})),
            &[("X-Console-CSRF", &proof2)],
        )
        .await
        .error(StatusCode::UNAUTHORIZED, "authentication_invalid");
        assert!(snapshot(&pool, attempt.account).await == revoked);
    }

    #[sqlx::test(migrations = false)]
    async fn csrf_refresh_only_is_read_only_and_cross_account_proof_is_denied(pool: PgPool) {
        let router = fixture(&pool).await;
        let (a, mut cookies) = enrolled(&router).await;
        let (b, other) = enrolled(&router).await;
        cookies.0.remove(ACCESS);
        let before_a = snapshot(&pool, a.account).await;
        let before_b = snapshot(&pool, b.account).await;
        let own = proof(&router, &cookies).await;
        let foreign = proof(&router, &other).await;
        request(&router, "GET", "/api/v2/accounts/me", &cookies, None, &[])
            .await
            .error(StatusCode::UNAUTHORIZED, "authentication_invalid");
        request(
            &router,
            "POST",
            "/api/v2/auth/token/refresh",
            &cookies,
            Some(json!({})),
            &[("X-Console-CSRF", &foreign)],
        )
        .await
        .error(StatusCode::FORBIDDEN, "csrf_invalid");
        assert!(
            snapshot(&pool, a.account).await == before_a
                && snapshot(&pool, b.account).await == before_b
        );
        let response = request(
            &router,
            "POST",
            "/api/v2/auth/token/refresh",
            &cookies,
            Some(json!({})),
            &[("X-Console-CSRF", &own)],
        )
        .await;
        session(&response, StatusCode::OK, a.account, &mut cookies);
    }

    #[sqlx::test(migrations = false)]
    async fn browser_rejects_duplicate_cookie_bearer_and_proof_as_access(pool: PgPool) {
        let router = fixture(&pool).await;
        let (attempt, cookies) = enrolled(&router).await;
        let before = snapshot(&pool, attempt.account).await;
        let csrf = proof(&router, &cookies).await;
        request(
            &router,
            "GET",
            "/api/v2/accounts/me",
            &cookies,
            None,
            &[("Cookie", &format!("{ACCESS}={}", cookies.0[ACCESS]))],
        )
        .await
        .error(StatusCode::BAD_REQUEST, "ambiguous_credentials");
        request(
            &router,
            "GET",
            "/api/v2/accounts/me",
            &cookies,
            None,
            &[("Authorization", &format!("Bearer {}", cookies.0[ACCESS]))],
        )
        .await
        .error(StatusCode::BAD_REQUEST, "ambiguous_credentials");
        let mut confused = cookies.clone();
        confused.0.insert(ACCESS.to_owned(), csrf);
        request(&router, "GET", "/api/v2/accounts/me", &confused, None, &[])
            .await
            .error(StatusCode::UNAUTHORIZED, "authentication_invalid");
        request(
            &router,
            "GET",
            "/api/v2/auth/csrf",
            &confused,
            None,
            &[("X-Console-CSRF", "fetch")],
        )
        .await
        .error(StatusCode::UNAUTHORIZED, "authentication_invalid");
        request(
            &router,
            "POST",
            "/api/v2/auth/logout",
            &cookies,
            Some(json!({})),
            &[("X-Console-CSRF", &cookies.0[ACCESS])],
        )
        .await
        .error(StatusCode::FORBIDDEN, "csrf_invalid");
        assert!(snapshot(&pool, attempt.account).await == before);
        projection(
            &request(&router, "GET", "/api/v2/accounts/me", &cookies, None, &[])
                .await
                .json(StatusCode::OK),
            attempt.account,
        );
    }

    #[sqlx::test(migrations = false)]
    async fn configured_missing_artifact_fails_closed(pool: PgPool) {
        prepare_http_database(&pool).await;
        seed_terms(&pool).await;
        let artifacts = Artifacts::new();
        let missing = artifacts.root.join(format!("missing-{}", Uuid::new_v4()));
        assert!(!missing.exists());
        let before: i64 = sqlx::query_scalar("SELECT count(*) FROM accounts")
            .fetch_one(&pool)
            .await
            .unwrap();
        let router = router(&pool, missing).await;
        request(
            &router,
            "GET",
            "/api/v2/auth/terms",
            &Cookies::default(),
            None,
            &[],
        )
        .await
        .error(StatusCode::SERVICE_UNAVAILABLE, "authority_unavailable");
        request(
            &router,
            "POST",
            "/api/v2/auth/registration/start",
            &Cookies::default(),
            Some(json!({"terms_version":MANIFEST_DIGEST})),
            &[],
        )
        .await
        .error(StatusCode::SERVICE_UNAVAILABLE, "authority_unavailable");
        let accounts: i64 = sqlx::query_scalar("SELECT count(*) FROM accounts")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(accounts, before, "artifact failure created an Account");
    }

    #[test]
    fn fixture_hashes_are_exact_and_test_only() {
        let bytes = fixture_file("fixtures/manifest.json");
        assert_eq!(hex::encode(Sha256::digest(bytes)), MANIFEST_DIGEST);
        assert_eq!(manifest()["fixture_only"], true);
        for (file, item) in [("service.txt", 0), ("privacy.txt", 1)] {
            let bytes = fixture_file(&format!("fixtures/{file}"));
            assert_eq!(
                hex::encode(Sha256::digest(bytes)),
                manifest()["items"][item]["content_sha256"]
            );
        }
    }

    #[test]
    fn strict_projection_oracle_refuses_wrong_account_and_hidden_fields() {
        let account = Uuid::new_v4();
        let expiry = (OffsetDateTime::now_utc() + Duration::minutes(5))
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap();
        let valid = json!({"account_id":account,"session":{"assurance":"PASSKEY_PRIMARY","expires_at":expiry},"permitted_self_actions":[{"action_key":"account.session.logout","registration_revision":"1"}]});
        projection(&valid, account);
        assert!(std::panic::catch_unwind(|| projection(&valid, Uuid::new_v4())).is_err());
        let mut leaked = valid.clone();
        leaked["access_token"] = json!("TEST_ONLY_TOKEN_CANARY");
        assert!(std::panic::catch_unwind(|| projection(&leaked, account)).is_err());
        leaked = valid;
        leaked["session"]["org_id"] = json!(Uuid::new_v4());
        assert!(std::panic::catch_unwind(|| projection(&leaked, account)).is_err());
    }

    #[test]
    fn cookie_oracle_rejects_conflicting_security_attributes() {
        let valid = format!(
            "{ENROLLMENT}=TEST_ONLY_NONCE_CANARY; Secure; HttpOnly; Path=/; SameSite=Strict; Max-Age=300"
        );
        let parse = |value: &str| {
            let mut headers = http::HeaderMap::new();
            headers.insert(header::SET_COOKIE, value.parse().unwrap());
            Cookies::default().absorb(&headers);
        };
        parse(&valid);
        for extra in [
            "; SameSite=None",
            "; SameSite = None",
            "; Max-Age = 999999",
            "; Domain = example.com",
            "; Path = /other",
            "; samesite=Lax",
            "; Path=/other",
            "; Max-Age=999999",
            "; Domain=example.com",
            "; Secure=false",
        ] {
            assert!(
                std::panic::catch_unwind(|| parse(&format!("{valid}{extra}"))).is_err(),
                "cookie oracle missed ambiguous security attribute"
            );
        }
    }

    #[test]
    fn response_oracle_rejects_literal_secrets_in_messages_and_headers() {
        let canary = "TEST_ONLY_CSRF_LITERAL_CANARY";
        let make = |message: &str| {
            let mut headers = http::HeaderMap::new();
            headers.insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
            headers.insert(header::PRAGMA, "no-cache".parse().unwrap());
            Response {
                status: StatusCode::FORBIDDEN,
                headers,
                bytes: serde_json::to_vec(
                    &json!({"error":{"code":"csrf_invalid","message":message}}),
                )
                .unwrap(),
                sent_secrets: vec![canary.to_owned()],
            }
        };
        make("Request refused").error(StatusCode::FORBIDDEN, "csrf_invalid");
        assert!(
            std::panic::catch_unwind(|| make(canary).error(StatusCode::FORBIDDEN, "csrf_invalid"))
                .is_err()
        );
        let mut leaked = make("Request refused");
        leaked.headers.insert("x-debug", canary.parse().unwrap());
        assert!(
            std::panic::catch_unwind(|| leaked.error(StatusCode::FORBIDDEN, "csrf_invalid"))
                .is_err()
        );
        // Legitimate successful cookie carrier is permitted; a second debug carrier is not.
        let mut cookie = make("Request complete");
        cookie.status = StatusCode::OK;
        cookie.headers.insert(
            header::SET_COOKIE,
            format!("{ACCESS}={canary}; Secure; HttpOnly; Path=/; SameSite=Lax; Max-Age=300")
                .parse()
                .unwrap(),
        );
        cookie.no_literal_echo();
        cookie.headers.insert("x-debug", canary.parse().unwrap());
        assert!(std::panic::catch_unwind(|| cookie.no_literal_echo()).is_err());
    }

    // B06/B07: this signer is fixture-owned and uses the same real configured
    // ES256 key as the router. Correct signatures isolate claim-validation bugs.
    async fn signed_fixture(pool: &PgPool) -> (Fixture, SigningKey) {
        prepare_http_database(pool).await;
        seed_terms(pool).await;
        let artifacts = Artifacts::new();
        let key = SigningKey::random(&mut OsRng);
        let service = router_with_key(pool, artifacts.root.clone(), &key).await;
        (
            Fixture {
                service,
                _artifacts: artifacts,
                pool: pool.clone(),
            },
            key,
        )
    }

    fn sign_proof_claims(claims: &Value, key: &SigningKey) -> String {
        let pem = key.to_pkcs8_pem(LineEnding::LF).unwrap();
        jsonwebtoken::encode(
            &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::ES256),
            claims,
            &jsonwebtoken::EncodingKey::from_ec_pem(pem.as_bytes()).unwrap(),
        )
        .unwrap()
    }

    // Signature-only decoding is deliberate: invalid time/issuer values must
    // remain inspectable by the oracle. Production validation stays untouched.
    fn signed_claims(token: &str, key: &SigningKey) -> Result<Value, jsonwebtoken::errors::Error> {
        let pem = key
            .verifying_key()
            .to_public_key_pem(LineEnding::LF)
            .unwrap();
        let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::ES256);
        validation.required_spec_claims.clear();
        validation.validate_exp = false;
        validation.validate_nbf = false;
        validation.validate_aud = false;
        jsonwebtoken::decode::<Value>(
            token,
            &jsonwebtoken::DecodingKey::from_ec_pem(pem.as_bytes()).unwrap(),
            &validation,
        )
        .map(|data| data.claims)
    }

    fn claim_generation(claims: &Value) -> i64 {
        let text = claims["security_generation"]
            .as_str()
            .expect("canonical decimal string");
        let generation: i64 = text.parse().unwrap();
        assert!(generation > 0 && generation.to_string() == text);
        generation
    }

    async fn fresh_login(router: &Fixture, attempt: &mut Attempt) -> Cookies {
        let (mut cookies, input) = login_attempt(router, attempt).await;
        let response = request(
            router,
            "POST",
            "/api/v2/auth/passkey/login/finish",
            &cookies,
            Some(input),
            &[],
        )
        .await;
        session(&response, StatusCode::OK, attempt.account, &mut cookies);
        cookies
    }

    #[test]
    fn signed_claim_oracle_rejects_other_keys_and_payload_tampering() {
        let key = SigningKey::random(&mut OsRng);
        let claims = json!({"sub":Uuid::new_v4(), "exp":1, "iss":"invalid-for-production"});
        let token = sign_proof_claims(&claims, &key);
        assert!(signed_claims(&token, &key).unwrap() == claims);
        assert!(signed_claims(&token, &SigningKey::random(&mut OsRng)).is_err());
        let parts: Vec<_> = token.split('.').collect();
        let changed = format!(
            "{}.{}.{}",
            parts[0],
            URL_SAFE_NO_PAD.encode(br#"{"sub":"tampered","exp":1}"#),
            parts[2]
        );
        assert!(signed_claims(&changed, &key).is_err());
        for valid in ["1", "9223372036854775807"] {
            assert_eq!(
                claim_generation(&json!({"security_generation":valid})),
                valid.parse::<i64>().unwrap()
            );
        }
        for invalid in [
            json!(1),
            json!("0"),
            json!("01"),
            json!("+1"),
            json!("-1"),
            json!("9223372036854775808"),
            Value::Null,
        ] {
            assert!(
                std::panic::catch_unwind(|| claim_generation(
                    &json!({"security_generation":invalid})
                ))
                .is_err()
            );
        }
    }

    #[sqlx::test(migrations = false)]
    async fn csrf_issued_claims_are_signed_short_lived_and_read_only(pool: PgPool) {
        let (router, key) = signed_fixture(&pool).await;
        let (attempt, cookies) = enrolled(&router).await;
        let before = snapshot(&pool, attempt.account).await;
        let mut refresh_only = cookies.clone();
        refresh_only.0.remove(ACCESS);
        for credential in [&cookies, &refresh_only] {
            let started = OffsetDateTime::now_utc().unix_timestamp();
            let response = request(
                &router,
                "GET",
                "/api/v2/auth/csrf",
                credential,
                None,
                &[("X-Console-CSRF", "fetch")],
            )
            .await;
            let completed = OffsetDateTime::now_utc().unix_timestamp();
            let body = response.json(StatusCode::OK);
            response.private();
            exact_keys(&body, &["csrf_proof", "expires_at"]);
            assert!(!response.headers.contains_key(header::SET_COOKIE));
            let claims = signed_claims(body["csrf_proof"].as_str().unwrap(), &key).unwrap();
            assert_eq!(claims["iss"], TEST_ISSUER);
            assert_eq!(claims["aud"], "console.account.csrf.v1");
            assert_eq!(claims["token_kind"], "account_csrf_v1");
            assert_eq!(claims["sub"], attempt.account.to_string());
            assert_eq!(claims["sid"], before["families"][0]["id"]);
            assert_eq!(
                claim_generation(&claims),
                before["security"]["security_generation"].as_i64().unwrap()
            );
            exact_keys(
                &claims,
                &[
                    "iss",
                    "aud",
                    "token_kind",
                    "sub",
                    "sid",
                    "security_generation",
                    "iat",
                    "nbf",
                    "exp",
                ],
            );
            let iat = claims["iat"].as_i64().unwrap();
            let nbf = claims["nbf"].as_i64().unwrap();
            let exp = claims["exp"].as_i64().unwrap();
            assert!(started <= iat && iat <= completed);
            assert!(nbf <= completed && nbf <= exp);
            assert!(exp > completed && exp <= iat + 300);
            let expires = OffsetDateTime::parse(
                body["expires_at"].as_str().unwrap(),
                &time::format_description::well_known::Rfc3339,
            )
            .unwrap();
            assert_eq!(expires.unix_timestamp(), exp);
            assert!(snapshot(&pool, attempt.account).await == before);
        }
        assert_no_company_identity(&pool, attempt.account).await;
    }

    #[sqlx::test(migrations = false)]
    async fn csrf_correctly_signed_invalid_claims_do_not_consume_refresh(pool: PgPool) {
        let (router, key) = signed_fixture(&pool).await;
        let (attempt, mut cookies) = enrolled(&router).await;
        let issued = proof(&router, &cookies).await;
        let claims = signed_claims(&issued, &key).unwrap();
        let before = snapshot(&pool, attempt.account).await;
        let now = OffsetDateTime::now_utc().unix_timestamp();
        let generation = claim_generation(&claims);
        let cases = [
            ("iss", json!("untrusted-issuer")),
            ("aud", json!("console-api")),
            ("token_kind", json!("account_access_v1")),
            ("sub", json!(Uuid::nil())),
            ("sid", json!(Uuid::nil())),
            (
                "security_generation",
                json!(generation.checked_add(1).unwrap().to_string()),
            ),
            ("security_generation", json!(generation)),
            ("security_generation", json!("01")),
            ("security_generation", json!("0")),
            ("security_generation", json!("9223372036854775808")),
            ("unrecognized_claim", json!(true)),
            ("exp", json!(now - 1)),
            ("iat", json!(now + 120)),
            ("nbf", json!(now + 120)),
        ];
        for (field, value) in cases {
            let mut invalid = claims.clone();
            invalid[field] = value;
            if field == "exp" {
                invalid["iat"] = json!(now - 120);
                invalid["nbf"] = json!(now - 120);
            }
            let signed = sign_proof_claims(&invalid, &key);
            assert!(signed_claims(&signed, &key).unwrap() == invalid);
            request(
                &router,
                "POST",
                "/api/v2/auth/token/refresh",
                &cookies,
                Some(json!({})),
                &[("X-Console-CSRF", &signed)],
            )
            .await
            .error(StatusCode::FORBIDDEN, "csrf_invalid");
            assert!(
                snapshot(&pool, attempt.account).await == before,
                "invalid proof changed Account state"
            );
        }
        for field in [
            "iss",
            "aud",
            "token_kind",
            "sub",
            "sid",
            "security_generation",
            "iat",
            "nbf",
            "exp",
        ] {
            let mut invalid = claims.clone();
            invalid.as_object_mut().unwrap().remove(field);
            let signed = sign_proof_claims(&invalid, &key);
            assert!(signed_claims(&signed, &key).unwrap() == invalid);
            request(
                &router,
                "POST",
                "/api/v2/auth/token/refresh",
                &cookies,
                Some(json!({})),
                &[("X-Console-CSRF", &signed)],
            )
            .await
            .error(StatusCode::FORBIDDEN, "csrf_invalid");
            assert!(snapshot(&pool, attempt.account).await == before);
        }
        let fresh = proof(&router, &cookies).await;
        let valid = sign_proof_claims(&signed_claims(&fresh, &key).unwrap(), &key);
        let response = request(
            &router,
            "POST",
            "/api/v2/auth/token/refresh",
            &cookies,
            Some(json!({})),
            &[("X-Console-CSRF", &valid)],
        )
        .await;
        session(&response, StatusCode::OK, attempt.account, &mut cookies);
        assert!(snapshot(&pool, attempt.account).await["families"] == before["families"]);
    }

    #[sqlx::test(migrations = false)]
    async fn csrf_wrong_key_and_hmac_algorithm_are_rejected(pool: PgPool) {
        let (router, key) = signed_fixture(&pool).await;
        let (attempt, mut cookies) = enrolled(&router).await;
        let issued = proof(&router, &cookies).await;
        let claims = signed_claims(&issued, &key).unwrap();
        let other_key = SigningKey::random(&mut OsRng);
        let wrong_key = sign_proof_claims(&claims, &other_key);
        assert!(signed_claims(&wrong_key, &other_key).unwrap() == claims);
        assert!(signed_claims(&wrong_key, &key).is_err());
        let public = key
            .verifying_key()
            .to_public_key_pem(LineEnding::LF)
            .unwrap();
        let wrong_algorithm = jsonwebtoken::encode(
            &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS256),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(public.as_bytes()),
        )
        .unwrap();
        assert_eq!(
            jsonwebtoken::decode_header(&wrong_algorithm).unwrap().alg,
            jsonwebtoken::Algorithm::HS256
        );
        let before = snapshot(&pool, attempt.account).await;
        for invalid in [wrong_key, wrong_algorithm] {
            request(
                &router,
                "POST",
                "/api/v2/auth/token/refresh",
                &cookies,
                Some(json!({})),
                &[("X-Console-CSRF", &invalid)],
            )
            .await
            .error(StatusCode::FORBIDDEN, "csrf_invalid");
            assert!(snapshot(&pool, attempt.account).await == before);
        }
        let fresh = proof(&router, &cookies).await;
        let valid = sign_proof_claims(&signed_claims(&fresh, &key).unwrap(), &key);
        let response = request(
            &router,
            "POST",
            "/api/v2/auth/token/refresh",
            &cookies,
            Some(json!({})),
            &[("X-Console-CSRF", &valid)],
        )
        .await;
        session(&response, StatusCode::OK, attempt.account, &mut cookies);
    }

    #[sqlx::test(migrations = false)]
    async fn csrf_same_account_distinct_families_do_not_share_authority(pool: PgPool) {
        let (router, key) = signed_fixture(&pool).await;
        let (mut attempt, first) = enrolled(&router).await;
        let mut second = fresh_login(&router, &mut attempt).await;
        let first_proof = proof(&router, &first).await;
        let second_proof = proof(&router, &second).await;
        let a = signed_claims(&first_proof, &key).unwrap();
        let b = signed_claims(&second_proof, &key).unwrap();
        assert_eq!(a["sub"], b["sub"]);
        assert_ne!(a["sid"], b["sid"]);
        let before = snapshot(&pool, attempt.account).await;
        assert_eq!(before["families"].as_array().unwrap().len(), 2);
        for (cookies, foreign) in [(&first, &second_proof), (&second, &first_proof)] {
            for path in ["/api/v2/auth/token/refresh", "/api/v2/auth/logout"] {
                request(
                    &router,
                    "POST",
                    path,
                    cookies,
                    Some(json!({})),
                    &[("X-Console-CSRF", foreign)],
                )
                .await
                .error(StatusCode::FORBIDDEN, "csrf_invalid");
                assert!(snapshot(&pool, attempt.account).await == before);
            }
        }
        let mut mixed = first.clone();
        mixed
            .0
            .insert(REFRESH.to_owned(), second.0[REFRESH].clone());
        request(
            &router,
            "GET",
            "/api/v2/auth/csrf",
            &mixed,
            None,
            &[("X-Console-CSRF", "fetch")],
        )
        .await
        .error(StatusCode::BAD_REQUEST, "ambiguous_credentials");
        assert!(snapshot(&pool, attempt.account).await == before);
        let response = request(
            &router,
            "POST",
            "/api/v2/auth/logout",
            &first,
            Some(json!({})),
            &[("X-Console-CSRF", &first_proof)],
        )
        .await;
        assert_eq!(
            response.json(StatusCode::OK),
            json!({"outcome":"COMMITTED"})
        );
        response.private();
        let after = snapshot(&pool, attempt.account).await;
        let family = |state: &Value, id: &Value| {
            state["families"]
                .as_array()
                .unwrap()
                .iter()
                .find(|f| &f["id"] == id)
                .unwrap()
                .clone()
        };
        assert!(!family(&after, &a["sid"])["revoked_at"].is_null());
        assert!(family(&after, &b["sid"]) == family(&before, &b["sid"]));
        request(&router, "GET", "/api/v2/accounts/me", &first, None, &[])
            .await
            .error(StatusCode::UNAUTHORIZED, "authentication_invalid");
        projection(
            &request(&router, "GET", "/api/v2/accounts/me", &second, None, &[])
                .await
                .json(StatusCode::OK),
            attempt.account,
        );
        let response = request(
            &router,
            "POST",
            "/api/v2/auth/token/refresh",
            &second,
            Some(json!({})),
            &[("X-Console-CSRF", &second_proof)],
        )
        .await;
        session(&response, StatusCode::OK, attempt.account, &mut second);
    }

    #[sqlx::test(migrations = false)]
    async fn csrf_security_generation_change_denies_old_session_and_proof(pool: PgPool) {
        let (router, key) = signed_fixture(&pool).await;
        let (mut attempt, old) = enrolled(&router).await;
        let old_proof = proof(&router, &old).await;
        let old_claims = signed_claims(&old_proof, &key).unwrap();
        // Fixture-owned state transition; not proof of security-command authority.
        let affected = sqlx::query("UPDATE account_security SET security_generation=security_generation+1, revision=revision+1, updated_at=now() WHERE account_id=$1 AND security_state='ACTIVE'").bind(attempt.account).execute(&pool).await.unwrap().rows_affected();
        assert_eq!(affected, 1);
        let before = snapshot(&pool, attempt.account).await;
        let mut refresh_only = old.clone();
        refresh_only.0.remove(ACCESS);
        for cookies in [&old, &refresh_only] {
            request(
                &router,
                "GET",
                "/api/v2/auth/csrf",
                cookies,
                None,
                &[("X-Console-CSRF", "fetch")],
            )
            .await
            .error(StatusCode::UNAUTHORIZED, "authentication_invalid");
            request(
                &router,
                "POST",
                "/api/v2/auth/token/refresh",
                cookies,
                Some(json!({})),
                &[("X-Console-CSRF", &old_proof)],
            )
            .await
            .error(StatusCode::UNAUTHORIZED, "authentication_invalid");
            assert!(snapshot(&pool, attempt.account).await == before);
        }
        request(&router, "GET", "/api/v2/accounts/me", &old, None, &[])
            .await
            .error(StatusCode::UNAUTHORIZED, "authentication_invalid");
        assert!(snapshot(&pool, attempt.account).await == before);
        let current = fresh_login(&router, &mut attempt).await;
        let current_proof = proof(&router, &current).await;
        let claims = signed_claims(&current_proof, &key).unwrap();
        assert!(claim_generation(&claims) > claim_generation(&old_claims));
        let restored = snapshot(&pool, attempt.account).await;
        assert_eq!(
            claim_generation(&claims),
            restored["security"]["security_generation"]
                .as_i64()
                .unwrap()
        );
        let mut stale_generation = claims.clone();
        stale_generation["security_generation"] = old_claims["security_generation"].clone();
        let stale_proof = sign_proof_claims(&stale_generation, &key);
        assert!(signed_claims(&stale_proof, &key).unwrap() == stale_generation);
        request(
            &router,
            "POST",
            "/api/v2/auth/logout",
            &current,
            Some(json!({})),
            &[("X-Console-CSRF", &stale_proof)],
        )
        .await
        .error(StatusCode::FORBIDDEN, "csrf_invalid");
        assert!(snapshot(&pool, attempt.account).await == restored);
        request(
            &router,
            "POST",
            "/api/v2/auth/logout",
            &current,
            Some(json!({})),
            &[("X-Console-CSRF", &old_proof)],
        )
        .await
        .error(StatusCode::FORBIDDEN, "csrf_invalid");
        assert!(snapshot(&pool, attempt.account).await == restored);
        let response = request(
            &router,
            "POST",
            "/api/v2/auth/logout",
            &current,
            Some(json!({})),
            &[("X-Console-CSRF", &current_proof)],
        )
        .await;
        assert_eq!(
            response.json(StatusCode::OK),
            json!({"outcome":"COMMITTED"})
        );
        response.private();
    }

    // B04/B05/B12 fixture data transition ONLY. This models sequential release
    // states; it neither invokes nor proves publisher custody, privileges or CAS.
    async fn seed_next_terms_head(pool: &PgPool, digest: &str) -> (Uuid, i64) {
        let mut tx = pool.begin().await.unwrap();
        let prior: i64 =
            sqlx::query_scalar("SELECT revision FROM account_terms_head WHERE id=1 FOR UPDATE")
                .fetch_one(&mut *tx)
                .await
                .unwrap();
        let revision = prior.checked_add(1).unwrap();
        let receipt = Uuid::new_v4();
        let approval = serde_json::to_vec(&json!({
            "kind":"ACCOUNT_TERMS_PUBLICATION_APPROVAL", "approval_id":Uuid::new_v4(),
            "manifest_sha256":digest, "expected_revision":prior.to_string(), "next_revision":revision.to_string(),
            "fixture_only":true, "approved_by":"test_only.operator", "approved_at":"2026-09-13T00:00:00Z",
            "content_authority_refs":[hex::encode(Sha256::digest(b"TEST_ONLY content authority, never legal proof"))]
        })).unwrap();
        sqlx::query("INSERT INTO account_terms_release_receipts (id,previous_revision,revision,manifest_sha256,approved_release_ref,approval_bytes,recorded_at) VALUES ($1,$2,$3,$4,$5,$6,now())")
            .bind(receipt).bind(prior).bind(revision).bind(hex::decode(digest).unwrap())
            .bind(json!({"kind":"OPERATOR_RELEASE_APPROVAL","approval_sha256":hex::encode(Sha256::digest(&approval))}))
            .bind(approval).execute(&mut *tx).await.unwrap();
        let rows = sqlx::query("UPDATE account_terms_head SET manifest_sha256=$1,revision=$2,release_receipt_ref=$3,updated_at=now() WHERE id=1")
            .bind(hex::decode(digest).unwrap()).bind(revision).bind(receipt).execute(&mut *tx).await.unwrap().rows_affected();
        assert_eq!(rows, 1);
        tx.commit().await.unwrap();
        (receipt, revision)
    }

    async fn identity_population(pool: &PgPool) -> (i64, i64) {
        sqlx::query_as(
            "SELECT (SELECT count(*) FROM accounts),(SELECT count(*) FROM company_actors)",
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn assert_retained_artifacts(router: &axum::Router) {
        let index: Value =
            serde_json::from_slice(fixture_file("fixtures/artifact-index.json")).unwrap();
        let mut entries = vec![index["manifest"].clone()];
        entries.extend(index["content"].as_array().unwrap().clone());
        for entry in entries {
            let digest = entry["sha256"].as_str().unwrap();
            let manifest = digest == MANIFEST_DIGEST;
            let path = format!(
                "/api/v2/auth/terms/{}/{digest}",
                if manifest { "manifests" } else { "content" }
            );
            let response = request(router, "GET", &path, &Cookies::default(), None, &[]).await;
            assert_eq!(response.status, StatusCode::OK);
            assert!(response.bytes == fixture_file(entry["path"].as_str().unwrap()));
            assert_eq!(hex::encode(Sha256::digest(&response.bytes)), digest);
            assert!(!response.headers.contains_key(header::SET_COOKIE));
            assert!(!response.headers.contains_key(header::LOCATION));
            assert_eq!(
                response.headers.get(header::CONTENT_TYPE).unwrap(),
                if manifest {
                    "application/json; charset=utf-8"
                } else {
                    "text/plain; charset=utf-8"
                }
            );
            if !manifest {
                assert_eq!(
                    response.headers.get("x-content-type-options").unwrap(),
                    "nosniff"
                );
            }
        }
    }

    async fn assert_current_terms(router: &axum::Router, revision: i64) {
        let response = request(
            router,
            "GET",
            "/api/v2/auth/terms",
            &Cookies::default(),
            None,
            &[],
        )
        .await;
        let value = response.json(StatusCode::OK);
        response.private();
        assert!(!response.headers.contains_key(header::SET_COOKIE));
        assert!(!response.headers.contains_key(header::LOCATION));
        exact_keys(&value, &["terms_version", "terms_revision", "manifest_url"]);
        assert_eq!(value["terms_version"], MANIFEST_DIGEST);
        assert_eq!(value["terms_revision"], revision.to_string());
        assert_eq!(
            value["manifest_url"],
            format!("/api/v2/auth/terms/manifests/{MANIFEST_DIGEST}")
        );
    }

    async fn assert_existing_session_during_terms_outage(
        router: &Fixture,
        attempt: &Attempt,
        mut cookies: Cookies,
    ) {
        let before = snapshot(&router.pool, attempt.account).await;
        let me = request(router, "GET", "/api/v2/accounts/me", &cookies, None, &[]).await;
        projection(&me.json(StatusCode::OK), attempt.account);
        me.private();
        let csrf = proof(router, &cookies).await;
        assert!(snapshot(&router.pool, attempt.account).await == before);
        let refreshed = request(
            router,
            "POST",
            "/api/v2/auth/token/refresh",
            &cookies,
            Some(json!({})),
            &[("X-Console-CSRF", &csrf)],
        )
        .await;
        session(&refreshed, StatusCode::OK, attempt.account, &mut cookies);
        let rotated = snapshot(&router.pool, attempt.account).await;
        assert!(rotated["families"] == before["families"]);
        assert!(rotated["terms"] == before["terms"]);
        let logout = request(
            router,
            "POST",
            "/api/v2/auth/logout",
            &cookies,
            Some(json!({})),
            &[("X-Console-CSRF", &csrf)],
        )
        .await;
        assert_eq!(logout.json(StatusCode::OK), json!({"outcome":"COMMITTED"}));
        logout.private();
        let revoked_cookies = cookies.clone();
        cookies.absorb(&logout.headers);
        assert!(cookies.0.is_empty());
        request(
            router,
            "GET",
            "/api/v2/accounts/me",
            &revoked_cookies,
            None,
            &[],
        )
        .await
        .error(StatusCode::UNAUTHORIZED, "authentication_invalid");
        let after = snapshot(&router.pool, attempt.account).await;
        assert!(after["terms"] == before["terms"]);
        assert!(
            after["families"]
                .as_array()
                .unwrap()
                .iter()
                .all(|f| !f["revoked_at"].is_null())
        );
        assert_no_company_identity(&router.pool, attempt.account).await;
    }

    #[sqlx::test(migrations = false)]
    async fn same_digest_new_release_refuses_old_attempt_and_binds_new_receipt(pool: PgPool) {
        let router = fixture(&pool).await;
        let old = start(&router).await;
        let before = snapshot(&pool, old.account).await;
        let population = identity_population(&pool).await;
        let old_receipt: Value = sqlx::query_scalar(
            "SELECT to_jsonb(r) FROM account_terms_release_receipts r WHERE id=$1",
        )
        .bind(Uuid::parse_str(RECEIPT).unwrap())
        .fetch_one(&pool)
        .await
        .unwrap();
        let (receipt, revision) = seed_next_terms_head(&pool, MANIFEST_DIGEST).await;
        assert_eq!(revision, 2);
        assert_current_terms(&router, revision).await;
        finish(&router, &old)
            .await
            .error(StatusCode::CONFLICT, "terms_changed");
        assert!(snapshot(&pool, old.account).await == before);
        assert_eq!(identity_population(&pool).await, population);
        assert_no_company_identity(&pool, old.account).await;
        let fresh = start(&router).await;
        assert_ne!(fresh.account, old.account);
        let result = finish(&router, &fresh).await;
        session(
            &result,
            StatusCode::CREATED,
            fresh.account,
            &mut fresh.cookies.clone(),
        );
        let committed = snapshot(&pool, fresh.account).await;
        assert_eq!(committed["security"]["security_state"], "ACTIVE");
        assert_eq!(committed["keys"].as_array().unwrap().len(), 1);
        assert_eq!(committed["families"].as_array().unwrap().len(), 1);
        assert_eq!(committed["terms"].as_array().unwrap().len(), 2);
        for term in committed["terms"].as_array().unwrap() {
            assert_eq!(term["terms_version"], MANIFEST_DIGEST);
            assert_eq!(term["terms_release_receipt_id"], receipt.to_string());
            assert_eq!(term["terms_release_revision"], revision);
            assert_eq!(
                term["terms_manifest_sha256"],
                format!("\\x{MANIFEST_DIGEST}")
            );
            let event = committed["events"]
                .as_array()
                .unwrap()
                .iter()
                .find(|event| event["id"] == term["security_event_id"])
                .unwrap();
            assert_eq!(event["account_id"], fresh.account.to_string());
            assert_eq!(event["kind"], "TERMS_ACCEPTED");
            let expected = json!({"kind":"ACCOUNT_TERMS_RELEASE", "receipt_id":receipt, "revision":revision.to_string(), "manifest_sha256":MANIFEST_DIGEST});
            assert_eq!(event["evidence_ref"], expected);
            assert_eq!(event["payload"]["evidence"], expected);
        }
        let retained: Value = sqlx::query_scalar(
            "SELECT to_jsonb(r) FROM account_terms_release_receipts r WHERE id=$1",
        )
        .bind(Uuid::parse_str(RECEIPT).unwrap())
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(retained == old_receipt);
        assert!(snapshot(&pool, old.account).await == before);
        assert_no_company_identity(&pool, fresh.account).await;
    }

    #[sqlx::test(migrations = false)]
    async fn unsupported_current_terms_preserve_retained_consent_login_and_sessions(pool: PgPool) {
        let (mut router, key) = signed_fixture(&pool).await;
        let (mut established, cookies) = enrolled(&router).await;
        assert_committed(&pool, &established).await;
        let pending = start(&router).await;
        let established_before = snapshot(&pool, established.account).await;
        let pending_before = snapshot(&pool, pending.account).await;
        let population = identity_population(&pool).await;
        let mut unavailable_manifest = manifest();
        unavailable_manifest["items"][0]["title"] = json!("다음 발행의 테스트 전용 제목");
        let digest = hex::encode(Sha256::digest(
            serde_json::to_vec(&unavailable_manifest).unwrap(),
        ));
        assert_ne!(digest, MANIFEST_DIGEST);
        // Models a replica whose release bundle lacks the new current manifest.
        // Old registered bytes remain intact. Publisher authorization is not tested.
        seed_next_terms_head(&pool, &digest).await;
        router.service = router_with_key(&pool, router._artifacts.root.clone(), &key).await;
        request(
            &router,
            "GET",
            "/api/v2/auth/terms",
            &Cookies::default(),
            None,
            &[],
        )
        .await
        .error(StatusCode::SERVICE_UNAVAILABLE, "authority_unavailable");
        request(
            &router,
            "POST",
            "/api/v2/auth/registration/start",
            &Cookies::default(),
            Some(json!({"terms_version":MANIFEST_DIGEST})),
            &[],
        )
        .await
        .error(StatusCode::SERVICE_UNAVAILABLE, "authority_unavailable");
        finish(&router, &pending)
            .await
            .error(StatusCode::SERVICE_UNAVAILABLE, "authority_unavailable");
        assert_retained_artifacts(&router).await;
        assert!(snapshot(&pool, established.account).await == established_before);
        assert!(snapshot(&pool, pending.account).await == pending_before);
        assert_eq!(identity_population(&pool).await, population);
        assert_existing_session_during_terms_outage(&router, &established, cookies).await;
        let restored = fresh_login(&router, &mut established).await;
        projection(
            &request(&router, "GET", "/api/v2/accounts/me", &restored, None, &[])
                .await
                .json(StatusCode::OK),
            established.account,
        );
        let after = snapshot(&pool, established.account).await;
        assert!(after["terms"] == established_before["terms"]);
        let old_term_events: Vec<_> = established_before["events"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["kind"] == "TERMS_ACCEPTED")
            .collect();
        let new_term_events: Vec<_> = after["events"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["kind"] == "TERMS_ACCEPTED")
            .collect();
        assert!(old_term_events == new_term_events);
        assert!(snapshot(&pool, pending.account).await == pending_before);
        assert_eq!(identity_population(&pool).await, population);
        assert_no_company_identity(&pool, established.account).await;
    }

    #[sqlx::test(migrations = false)]
    async fn registered_artifact_loss_is_503_and_restore_preserves_same_attempt(pool: PgPool) {
        let (mut router, key) = signed_fixture(&pool).await;
        let (established, cookies) = enrolled(&router).await;
        let pending = start(&router).await;
        let before = snapshot(&pool, pending.account).await;
        let established_before = snapshot(&pool, established.account).await;
        let population = identity_population(&pool).await;
        assert_retained_artifacts(&router).await;
        let lost_path = router._artifacts.root.join("fixtures/privacy.txt");
        assert_eq!(std::fs::read(&lost_path).unwrap(), PRIVACY_TEXT.as_bytes());
        std::fs::remove_file(&lost_path).unwrap(); // Only this fixture-owned file.
        router.service = router_with_key(&pool, router._artifacts.root.clone(), &key).await;
        let digest = manifest()["items"][1]["content_sha256"]
            .as_str()
            .unwrap()
            .to_owned();
        request(
            &router,
            "GET",
            &format!("/api/v2/auth/terms/content/{digest}"),
            &Cookies::default(),
            None,
            &[],
        )
        .await
        .error(StatusCode::SERVICE_UNAVAILABLE, "authority_unavailable");
        request(
            &router,
            "GET",
            "/api/v2/auth/terms",
            &Cookies::default(),
            None,
            &[],
        )
        .await
        .error(StatusCode::SERVICE_UNAVAILABLE, "authority_unavailable");
        request(
            &router,
            "POST",
            "/api/v2/auth/registration/start",
            &Cookies::default(),
            Some(json!({"terms_version":MANIFEST_DIGEST})),
            &[],
        )
        .await
        .error(StatusCode::SERVICE_UNAVAILABLE, "authority_unavailable");
        finish(&router, &pending)
            .await
            .error(StatusCode::SERVICE_UNAVAILABLE, "authority_unavailable");
        assert!(snapshot(&pool, pending.account).await == before);
        assert!(snapshot(&pool, established.account).await == established_before);
        assert_eq!(identity_population(&pool).await, population);
        assert_existing_session_during_terms_outage(&router, &established, cookies).await;
        std::fs::write(&lost_path, PRIVACY_TEXT.as_bytes()).unwrap();
        router.service = router_with_key(&pool, router._artifacts.root.clone(), &key).await;
        assert_retained_artifacts(&router).await;
        assert_current_terms(&router, 1).await;
        let completed = finish(&router, &pending).await;
        session(
            &completed,
            StatusCode::CREATED,
            pending.account,
            &mut pending.cookies.clone(),
        );
        assert_committed(&pool, &pending).await;
        finish(&router, &pending)
            .await
            .error(StatusCode::UNAUTHORIZED, "enrollment_invalid");
        assert_eq!(identity_population(&pool).await, population);
        let established_after = snapshot(&pool, established.account).await;
        assert!(established_after["terms"] == established_before["terms"]);
        assert!(
            established_after["families"]
                .as_array()
                .unwrap()
                .iter()
                .all(|f| !f["revoked_at"].is_null())
        );
    }
    // Public read acceptance only. The fixture administrator models sequential
    // TEST_ONLY release states; these tests confer no publication authority.
    mod terms_read {
        use super::*;

        #[sqlx::test(migrations = false)]
        async fn current_head_and_retained_bytes_are_authoritative_and_read_only(pool: PgPool) {
            let app = fixture(&pool).await;
            let before = terms_reference_rows(&pool).await;
            assert_current_terms(&app, 1).await;
            assert_retained_artifacts(&app).await;

            let unsupported = "f".repeat(64);
            assert!(unsupported != MANIFEST_DIGEST);
            for kind in ["manifests", "content"] {
                request(
                    &app,
                    "GET",
                    &format!("/api/v2/auth/terms/{kind}/{unsupported}"),
                    &Cookies::default(),
                    None,
                    &[],
                )
                .await
                .error(StatusCode::NOT_FOUND, "not_found");
            }
            assert!(terms_reference_rows(&pool).await == before);

            // The same bytes at a later authoritative revision cannot be served
            // with a stale revision chosen from the local artifact index.
            let (_, revision) = seed_next_terms_head(&pool, MANIFEST_DIGEST).await;
            assert_eq!(revision, 2);
            let advanced = terms_reference_rows(&pool).await;
            assert_current_terms(&app, revision).await;
            assert_retained_artifacts(&app).await;
            assert!(terms_reference_rows(&pool).await == advanced);

            // An unsupported current head closes current authority while exact
            // previously registered immutable bytes remain publicly readable.
            let (_, revision) = seed_next_terms_head(&pool, &unsupported).await;
            assert_eq!(revision, 3);
            let unavailable = terms_reference_rows(&pool).await;
            request(
                &app,
                "GET",
                "/api/v2/auth/terms",
                &Cookies::default(),
                None,
                &[],
            )
            .await
            .error(StatusCode::SERVICE_UNAVAILABLE, "authority_unavailable");
            assert_retained_artifacts(&app).await;
            assert!(terms_reference_rows(&pool).await == unavailable);
        }

        #[sqlx::test(migrations = false)]
        async fn registered_content_loss_and_restore_preserve_public_read_state(pool: PgPool) {
            let mut app = fixture(&pool).await;
            let before = terms_reference_rows(&pool).await;
            assert_current_terms(&app, 1).await;
            assert_retained_artifacts(&app).await;
            assert!(terms_reference_rows(&pool).await == before);

            let lost_path = app._artifacts.root.join("fixtures/privacy.txt");
            assert!(std::fs::read(&lost_path).unwrap() == PRIVACY_TEXT.as_bytes());
            std::fs::remove_file(&lost_path).unwrap(); // Only this fixture-owned file.
            // Match the existing outage fixture. This does not require live
            // invalidation of already verified immutable bytes in a running router.
            app.service = router(&pool, app._artifacts.root.clone()).await;
            let digest = manifest()["items"][1]["content_sha256"]
                .as_str()
                .unwrap()
                .to_owned();
            for path in [
                format!("/api/v2/auth/terms/content/{digest}"),
                "/api/v2/auth/terms".to_owned(),
            ] {
                request(&app, "GET", &path, &Cookies::default(), None, &[])
                    .await
                    .error(StatusCode::SERVICE_UNAVAILABLE, "authority_unavailable");
            }
            assert!(terms_reference_rows(&pool).await == before);

            std::fs::write(&lost_path, PRIVACY_TEXT.as_bytes()).unwrap();
            app.service = router(&pool, app._artifacts.root.clone()).await;
            assert_current_terms(&app, 1).await;
            assert_retained_artifacts(&app).await;
            assert!(terms_reference_rows(&pool).await == before);
        }

        #[sqlx::test(migrations = false)]
        async fn auth_database_outage_closes_current_authority_but_retains_public_bytes(
            pool: PgPool,
        ) {
            let mut app = fixture(&pool).await;
            let auth_url = account_transport_urls(&pool)
                .into_iter()
                .find(|(key, _)| *key == "AUTH_DATABASE_URL")
                .expect("existing restricted Auth fixture transport")
                .1;
            let auth = PgPool::connect(&auth_url)
                .await
                .expect("connect genuine restricted Auth fixture transport");
            let role: String = sqlx::query_scalar("SELECT current_user")
                .fetch_one(&auth)
                .await
                .unwrap();
            assert_eq!(role, "console_auth_rt");
            let state = state_with_key(
                &pool,
                app._artifacts.root.clone(),
                &SigningKey::random(&mut OsRng),
            )
            .await;
            app.service = build_router(state.with_auth_database(auth.clone()));
            let before = terms_reference_rows(&pool).await;
            assert_current_terms(&app, 1).await;
            assert_retained_artifacts(&app).await;
            assert!(terms_reference_rows(&pool).await == before);

            // Close this actual Auth transport only; the fixture administrator
            // and immutable artifact reads remain available for the oracle.
            auth.close().await;
            request(
                &app,
                "GET",
                "/api/v2/auth/terms",
                &Cookies::default(),
                None,
                &[],
            )
            .await
            .error(StatusCode::SERVICE_UNAVAILABLE, "authority_unavailable");
            assert_retained_artifacts(&app).await;
            assert!(terms_reference_rows(&pool).await == before);
        }
    }
    mod terms_read_security {
        use super::*;

        #[sqlx::test(migrations = false)]
        async fn public_reads_require_neither_signing_keys_nor_webauthn_services(pool: PgPool) {
            let mut app = fixture(&pool).await;
            let state = state_with_key(
                &pool,
                app._artifacts.root.clone(),
                &SigningKey::random(&mut OsRng),
            )
            .await;
            let mut config = state.config().clone();
            config.jwt = None;
            config.auth_rest = None;
            assert!(config.jwt.is_none() && config.auth_rest.is_none());
            // Rebuild through genuine startup admission with no verification,
            // issuance, or WebAuthn services; retain configured artifact custody.
            app.service = build_router(AppState::from_config(config).await.unwrap());
            let before = terms_reference_rows(&pool).await;
            assert_current_terms(&app, 1).await;
            assert_retained_artifacts(&app).await;
            assert!(terms_reference_rows(&pool).await == before);
        }

        #[sqlx::test(migrations = false)]
        async fn registered_tampering_is_unavailable_and_exact_restoration_recovers(pool: PgPool) {
            let mut app = fixture(&pool).await;
            let before = terms_reference_rows(&pool).await;
            assert_current_terms(&app, 1).await;
            assert_retained_artifacts(&app).await;
            for (relative, namespace, digest) in [
                (
                    "fixtures/manifest.json",
                    "manifests",
                    MANIFEST_DIGEST.to_owned(),
                ),
                (
                    "fixtures/privacy.txt",
                    "content",
                    manifest()["items"][1]["content_sha256"]
                        .as_str()
                        .unwrap()
                        .to_owned(),
                ),
            ] {
                let path = app._artifacts.root.join(relative);
                let original = std::fs::read(&path).unwrap();
                assert_eq!(hex::encode(Sha256::digest(&original)), digest);
                // File still exists at its registered path. Identity is not
                // availability, and bad bytes must never become an unknown404.
                std::fs::write(&path, b"TEST_ONLY altered registered artifact").unwrap();
                app.service = router(&pool, app._artifacts.root.clone()).await;
                for uri in [
                    format!("/api/v2/auth/terms/{namespace}/{digest}"),
                    "/api/v2/auth/terms".to_owned(),
                ] {
                    request(&app, "GET", &uri, &Cookies::default(), None, &[])
                        .await
                        .error(StatusCode::SERVICE_UNAVAILABLE, "authority_unavailable");
                }
                assert!(terms_reference_rows(&pool).await == before);
                std::fs::write(&path, &original).unwrap();
                app.service = router(&pool, app._artifacts.root.clone()).await;
                assert_current_terms(&app, 1).await;
                assert_retained_artifacts(&app).await;
                assert!(terms_reference_rows(&pool).await == before);
            }
        }
    }
}

#[test]
fn account_auth_database_retains_transport_and_redacts_config_debug() {
    let mut pairs = account_transport_config_pairs();
    let auth_url = "postgresql://console_auth_rt:auth-debug-canary@localhost:5544/console";
    pairs.push(("AUTH_DATABASE_URL", auth_url.to_owned()));
    let config = AppConfig::from_pairs(pairs.clone()).unwrap();
    assert_eq!(config.auth_database_url.as_deref(), Some(auth_url));
    let debug = format!("{config:?}");
    assert!(debug.contains("AppConfig"));
    assert!(debug.contains("auth_database_configured: true"));
    for (key, value) in pairs {
        if key.ends_with("DATABASE_URL") || key == "CONSOLE_JWT_PRIVATE_KEY_PEM" {
            assert!(!debug.contains(&value), "Debug must not serialize {key}");
        }
    }
    for secret in [
        "auth-debug-canary",
        "runtime-fixture",
        "leave-fixture",
        "ontology-fixture",
        "force-fixture",
        "PRIVATE KEY",
    ] {
        assert!(
            !debug.contains(secret),
            "configuration Debug leaked secret material"
        );
    }
}

#[test]
fn account_auth_database_is_optional_without_serving_account_auth() {
    for mode in ["worker", "migrate", "no_database", "auth_disabled"] {
        let mut pairs = account_transport_config_pairs();
        match mode {
            "worker" => pairs.push(("CONSOLE_APP_ROLE", "worker".into())),
            "migrate" => {
                pairs.push(("CONSOLE_APP_ROLE", "migrate".into()));
                pairs.push((
                    "DATABASE_URL",
                    "postgresql://console_app:migrate-fixture@localhost:5544/console".into(),
                ));
            }
            "no_database" => pairs.retain(|(key, _)| *key != "DATABASE_URL"),
            // Disabled means no JWT verification or issuance services. Keeping
            // only the public key still requires the separate Auth transport.
            "auth_disabled" => pairs.retain(|(key, _)| {
                !key.starts_with("CONSOLE_WEBAUTHN_") && !key.starts_with("CONSOLE_JWT_")
            }),
            _ => unreachable!(),
        }
        let config = AppConfig::from_pairs(pairs).unwrap();
        assert!(
            config.auth_database_url.is_none(),
            "no auth transport inferred for {mode}"
        );
        if mode == "auth_disabled" {
            assert!(config.jwt.is_none() && config.auth_rest.is_none());
        }
    }
}
