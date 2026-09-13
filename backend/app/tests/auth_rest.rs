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
    let mut connection = pool.acquire().await.expect("disposable admin connection");
    let identity: (String, String, String, bool, bool) = sqlx::query_as(
        r#"
        SELECT session_user::text, current_user::text, current_database(),
            current_setting('console.sqlx_test_bootstrap', true) = 'buck-sqlx-superuser-v1'
            AND (SELECT rolsuper FROM pg_catalog.pg_roles WHERE rolname = current_user)
            AND (SELECT pg_get_userbyid(datdba) = current_user
                 FROM pg_catalog.pg_database WHERE datname = current_database()),
            NOT EXISTS (
                SELECT 1 FROM pg_catalog.pg_class c
                JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
                WHERE n.nspname !~ '^pg_' AND n.nspname <> 'information_schema'
            )
        "#,
    )
    .fetch_one(&mut *connection)
    .await
    .expect("inspect empty disposable HTTP test database");
    assert_eq!(identity.0, "console_buck_admin");
    assert_eq!(identity.1, "console_buck_admin");
    assert!(
        identity.3 && identity.4,
        "requires marked empty SQLx database"
    );
    let suffix = identity
        .2
        .strip_prefix("_sqlx_test_")
        .expect("SQLx database");
    assert!(
        suffix.len() == 52
            && suffix
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
    );

    let owner_binding = std::env::var("CONSOLE_APALIS_OWNER_DATABASE_URL")
        .expect("missing disposable migration-owner transport");
    let mut owner_url = Url::parse(&owner_binding).expect("valid migration-owner URL");
    assert!(matches!(owner_url.scheme(), "postgres" | "postgresql"));
    assert_eq!(owner_url.username(), "console_app");
    assert!(owner_url.password().is_some_and(|p| !p.is_empty()));
    assert!(owner_url.query().is_none() && owner_url.fragment().is_none());
    let options = pool.connect_options();
    assert_eq!(Some(identity.2.as_str()), options.get_database());
    assert_eq!(owner_url.host_str(), Some(options.get_host()));
    assert_eq!(owner_url.port().unwrap_or(5432), options.get_port());
    owner_url.set_path(&identity.2);

    // Provision only the empty database container. Product tables, grants and
    // queue schema are created by the existing production migration boundary.
    sqlx::raw_sql(
        "DO $owner$ BEGIN          EXECUTE format('ALTER DATABASE %I OWNER TO console_app', current_database());          END $owner$;",
    )
    .execute(&mut *connection)
    .await
    .expect("assign empty test database to its real migration owner");
    drop(connection);
    let config = AppConfig::from_pairs([
        ("CONSOLE_APP_ROLE", AppRole::Migrate.to_string()),
        ("DATABASE_URL", owner_url.to_string()),
    ])
    .expect("production migration configuration");
    run_migrations(&config)
        .await
        .expect("complete production schema migration before HTTP fixtures");
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
    for response in responses {
        assert!(
            legacy_denial_is_safe(response, known_secrets).await,
            "legacy refusal must be401 with v1 error JSON, no credential cookie/token fields or known secret echoes"
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
