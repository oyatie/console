#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Real-login transport comparison for existing, unmigrated legacy subjects.
//! Credential operations use console_auth_rt; Company business setup/reads use
//! console_rt. Migration-owner pools are confined to seed data and observations.
//! Preserve cryptographic verification, replay and exact subject correlation.
//! Legacy employer OTP/reset/signup expectations here do NOT authorize those
//! actions for migrated Accounts; cutover and Company-free enrollment need their
//! own tests before Account activation. Missing auth topology is a prerequisite
//! failure, never successful execution of the downstream crypto assertions.

use console_kernel_core::OrgId;
use console_platform_auth::{
    PasskeyRegistrationStart, PasskeyService, RefreshTokenStore, WebauthnSettings,
};
use console_platform_provisioning::{BootstrapCredentialStore, RosterProvisioner};
use console_platform_test_support::{
    TestDatabaseLogin, login_test_pool, prepare_account_test_database,
};
use sqlx::PgPool;
use time::{Duration, OffsetDateTime};
use url::Url;
use uuid::Uuid;
use webauthn_authenticator_rs::WebauthnAuthenticator;
use webauthn_authenticator_rs::prelude::RequestChallengeResponse;
use webauthn_authenticator_rs::softpasskey::SoftPasskey;

/// A second, non-KNL tenant id, to prove the cross-tenant paths.
const ORG_T2: Uuid = Uuid::from_u128(0x2222_2222_2222_2222_2222_2222_2222_2222);

/// Inject one `allowCredentials` entry into a discoverable challenge so the
/// SoftPasskey harness (no resident-key store) can locate its key. The SERVER
/// ceremony stays fully discoverable; the assertion still carries the credential
/// id the server resolves by. Mirrors the helper in the auth crate's tests.
fn inject_allow_credential(
    challenge: RequestChallengeResponse,
    credential_id: &str,
) -> RequestChallengeResponse {
    let mut value = serde_json::to_value(&challenge).unwrap();
    let allow = value
        .get_mut("publicKey")
        .and_then(|pk| pk.get_mut("allowCredentials"))
        .and_then(serde_json::Value::as_array_mut)
        .expect("discoverable challenge must have an allowCredentials array");
    allow.push(serde_json::json!({ "type": "public-key", "id": credential_id }));
    serde_json::from_value(value).unwrap()
}

fn passkey_service() -> PasskeyService {
    PasskeyService::new(WebauthnSettings {
        rp_id: "example.com".to_owned(),
        rp_origin: Url::parse("https://auth.example.com").unwrap(),
        rp_name: "Console".to_owned(),
        extra_allowed_origins: vec![],
        ceremony_ttl: Duration::minutes(5),
    })
    .unwrap()
}

async fn auth_role_pool(owner_pool: &PgPool) -> PgPool {
    login_test_pool(owner_pool, TestDatabaseLogin::Auth).await
}

/// Seed an `organizations` row + one user in it, as the OWNER (superuser) pool
/// with `row_security` off so the cross-org bootstrap rows go in regardless of
/// any GUC. Returns the new user's id.
async fn seed_org_and_user(owner_pool: &PgPool, org: Uuid, tag: &str) -> Uuid {
    let mut tx = owner_pool.begin().await.unwrap();
    sqlx::query("SET LOCAL row_security = off")
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::query("INSERT INTO organizations (id, slug, name) VALUES ($1, $2, $3) ON CONFLICT (id) DO NOTHING")
        .bind(org)
        .bind(format!("org-{}", tag.to_lowercase()))
        .bind(format!("Org {tag}"))
        .execute(&mut *tx)
        .await
        .unwrap();
    let user_id: Uuid = sqlx::query_scalar(
        "INSERT INTO users (display_name, roles, org_id) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(format!("User {tag}"))
    .bind(vec!["MECHANIC".to_string()])
    .bind(org)
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();
    user_id
}

/// Seed an admin SUPER_ADMIN user in `org` (so admin-issued OTP authz passes).
/// Creates the `organizations` row first (idempotent) so the user FK is satisfied
/// even when this runs before [`seed_org_and_user`].
async fn seed_admin(owner_pool: &PgPool, org: Uuid, tag: &str) -> Uuid {
    let mut tx = owner_pool.begin().await.unwrap();
    sqlx::query("SET LOCAL row_security = off")
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::query("INSERT INTO organizations (id, slug, name) VALUES ($1, $2, $3) ON CONFLICT (id) DO NOTHING")
        .bind(org)
        .bind(format!("org-{}", tag.to_lowercase()))
        .bind(format!("Org {tag}"))
        .execute(&mut *tx)
        .await
        .unwrap();
    let admin_id: Uuid = sqlx::query_scalar(
        "INSERT INTO users (display_name, roles, org_id) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(format!("Admin {tag}"))
    .bind(vec!["SUPER_ADMIN".to_string()])
    .bind(org)
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();
    admin_id
}

/// Register a discoverable passkey for `user_id` in `org` through its supplied restricted pool, returning
/// the stored credential id. Exercises start/finish registration (the
/// registration-finish INSERT is the org-stamped write the fix unblocks).
async fn register_passkey_as_runtime(
    service: &PasskeyService,
    rt_pool: &PgPool,
    org: OrgId,
    user_id: Uuid,
) -> (String, WebauthnAuthenticator<SoftPasskey>) {
    let registration = service
        .start_registration(
            rt_pool,
            org,
            PasskeyRegistrationStart {
                user_id,
                username: "rls.user".to_owned(),
                display_name: "RLS User".to_owned(),
            },
        )
        .await
        .expect("start_registration must succeed through its supplied restricted pool");

    let mut authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let credential = authenticator
        .do_registration(
            Url::parse("https://auth.example.com").unwrap(),
            registration.challenge,
        )
        .unwrap();

    let stored = service
        .finish_registration(rt_pool, org, registration.ceremony_id, credential)
        .await
        .expect("finish_registration must INSERT the passkey through its supplied restricted pool");
    assert_eq!(stored.user_id, user_id);

    // The credential row must carry the REAL org (the OrgId::knl() hardcode bug
    // would mis-stamp a non-KNL tenant). Verify through its supplied restricted pool under the right GUC.
    let stamped_org = credential_org_as_runtime(rt_pool, org, &stored.credential_id).await;
    assert_eq!(
        stamped_org,
        Some(*org.as_uuid()),
        "passkey row must be stamped with the authenticated org, not a hardcoded one"
    );

    (stored.credential_id, authenticator)
}

/// Read a credential's org_id through its supplied restricted pool with the GUC armed to `org`.
async fn credential_org_as_runtime(
    rt_pool: &PgPool,
    org: OrgId,
    credential_id: &str,
) -> Option<Uuid> {
    let mut tx = rt_pool.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.current_org', $1, true)")
        .bind(org.as_uuid().to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
    let org_id: Option<Uuid> =
        sqlx::query_scalar("SELECT org_id FROM auth_webauthn_credentials WHERE credential_id = $1")
            .bind(credential_id)
            .fetch_optional(&mut *tx)
            .await
            .unwrap();
    tx.commit().await.unwrap();
    org_id
}

/// Issue an admin OTP for `user_id` in `org` via the provisioning store
/// (the admin path that hardcoded KNL + armed no GUC before the fix).
async fn issue_admin_otp_as_runtime(rt_pool: &PgPool, org: OrgId, user_id: Uuid) -> String {
    let issue = BootstrapCredentialStore
        .issue_for_zero_credential_user(
            rt_pool,
            user_id,
            org,
            OffsetDateTime::now_utc(),
            Duration::hours(24),
        )
        .await
        .expect(
            "admin OTP issuance must succeed for any tenant through its supplied restricted pool",
        );
    issue.token.as_str().to_owned()
}

// ===========================================================================
// (1) KNL: full chain — admin OTP -> redeem -> passkey register -> passkey login.
// ===========================================================================
#[sqlx::test(migrations = false)]
async fn knl_auth_chain_works_as_runtime_role(owner_pool: PgPool) {
    prepare_account_test_database(&owner_pool).await;
    let rt_pool = auth_role_pool(&owner_pool).await;
    let knl = OrgId::knl();
    let user_id = seed_org_and_user(&owner_pool, *knl.as_uuid(), "KNL").await;

    // Admin issues a one-time code for the pre-provisioned user (through its supplied restricted pool).
    let otp = issue_admin_otp_as_runtime(&rt_pool, knl, user_id).await;

    // OTP first sign-in: redeem must FIND the bootstrap credential through its supplied restricted pool.
    let redemption = BootstrapCredentialStore
        .redeem_otp(&rt_pool, &otp, OffsetDateTime::now_utc())
        .await
        .expect("OTP redeem must find the seeded credential through its supplied restricted pool");
    assert_eq!(redemption.user_id, user_id);
    assert_eq!(redemption.org_id, knl);
    assert!(redemption.requires_passkey_setup);

    // The redeemed user can mint a session (refresh family issue is RLS-gated).
    RefreshTokenStore
        .issue_family(
            &rt_pool,
            &rt_pool,
            user_id,
            knl,
            OffsetDateTime::now_utc(),
            Duration::days(30),
        )
        .await
        .expect("session mint (refresh family) must pass RLS through its supplied restricted pool");

    // Passkey registration-finish INSERTs the credential with the correct org.
    let service = passkey_service();
    let (credential_id, mut authenticator) =
        register_passkey_as_runtime(&service, &rt_pool, knl, user_id).await;

    // Passkey LOGIN: usernameless discoverable auth must resolve the user FROM
    // the credential through its supplied restricted pool and authenticate.
    let authentication = service
        .start_authentication(&rt_pool)
        .await
        .expect("start_authentication through its supplied restricted pool");
    let challenge = inject_allow_credential(authentication.challenge, &credential_id);
    let assertion = authenticator
        .do_authentication(Url::parse("https://auth.example.com").unwrap(), challenge)
        .unwrap();
    let outcome = service
        .finish_authentication(&rt_pool, authentication.ceremony_id, assertion)
        .await
        .expect("passkey login must authenticate through its supplied restricted pool");
    assert_eq!(outcome.user_id, user_id);
    assert_eq!(outcome.org_id, knl);
}

// ===========================================================================
// (2) NON-KNL tenant: admin-issues-OTP -> that tenant's user redeems it.
// Proves cross-tenant new-account registration works through its supplied restricted pool (the KNL hardcode
// + no-GUC bug broke this for every tenant other than KNL).
// ===========================================================================
#[sqlx::test(migrations = false)]
async fn non_knl_admin_otp_and_redeem_work_as_runtime_role(owner_pool: PgPool) {
    prepare_account_test_database(&owner_pool).await;
    let rt_pool = auth_role_pool(&owner_pool).await;
    let org2 = OrgId::from_uuid(ORG_T2);
    let _admin = seed_admin(&owner_pool, ORG_T2, "T2").await;
    let user_id = seed_org_and_user(&owner_pool, ORG_T2, "T2").await;

    // Admin issues a one-time code for a NON-KNL tenant user (through its supplied restricted pool). Before
    // the fix this either mis-stamped KNL or failed the WITH CHECK outright.
    let otp = issue_admin_otp_as_runtime(&rt_pool, org2, user_id).await;

    // The credential must be stamped with org2, not KNL.
    let stamped = bootstrap_org_as_runtime(&rt_pool, org2, &otp).await;
    assert_eq!(
        stamped,
        Some(ORG_T2),
        "admin-issued OTP must be stamped with the request's tenant, not KNL"
    );

    // That tenant's user redeems it and gets a session, all through its supplied restricted pool.
    let redemption = BootstrapCredentialStore
        .redeem_otp(&rt_pool, &otp, OffsetDateTime::now_utc())
        .await
        .expect("non-KNL tenant OTP redeem must succeed through its supplied restricted pool");
    assert_eq!(redemption.user_id, user_id);
    assert_eq!(redemption.org_id, org2);

    RefreshTokenStore
        .issue_family(
            &rt_pool,
            &rt_pool,
            user_id,
            org2,
            OffsetDateTime::now_utc(),
            Duration::days(30),
        )
        .await
        .expect("non-KNL session mint must pass RLS through its supplied restricted pool");
}

/// Read a bootstrap credential's org_id by its OTP, through its supplied restricted pool with the GUC
/// armed to `org` (so the read is allowed and we can confirm the stamp).
async fn bootstrap_org_as_runtime(rt_pool: &PgPool, org: OrgId, otp: &str) -> Option<Uuid> {
    use sha2::{Digest, Sha256};
    let token_hash = Sha256::digest(otp.as_bytes()).to_vec();
    let mut tx = rt_pool.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.current_org', $1, true)")
        .bind(org.as_uuid().to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
    let org_id: Option<Uuid> =
        sqlx::query_scalar("SELECT org_id FROM auth_bootstrap_credentials WHERE token_hash = $1")
            .bind(&token_hash)
            .fetch_optional(&mut *tx)
            .await
            .unwrap();
    tx.commit().await.unwrap();
    org_id
}

// ===========================================================================
// (3) Cross-org isolation: a passkey login for tenant A's credential must NOT
// leak tenant B, and an OTP from one tenant must redeem as ITS tenant only.
// ===========================================================================
#[sqlx::test(migrations = "../db/migrations")]
async fn cross_org_credential_is_isolated_as_runtime_role(owner_pool: PgPool) {
    let auth = auth_role_pool(&owner_pool).await;
    let business = login_test_pool(&owner_pool, TestDatabaseLogin::Business).await;
    let knl = OrgId::knl();
    let org2 = OrgId::from_uuid(ORG_T2);
    let knl_user = seed_org_and_user(&owner_pool, *knl.as_uuid(), "KNL").await;
    let t2_user = seed_org_and_user(&owner_pool, ORG_T2, "T2").await;
    let service = passkey_service();
    let (knl_cred, mut knl_authenticator) =
        register_passkey_as_runtime(&service, &auth, knl, knl_user).await;
    let (t2_cred, mut t2_authenticator) =
        register_passkey_as_runtime(&service, &auth, org2, t2_user).await;

    // Neither an own-Company nor another-Company GUC grants credential access.
    // SQLSTATE42501 proves privilege denial; an empty RLS result is insufficient.
    for org in [knl, org2] {
        for credential_id in [&knl_cred, &t2_cred] {
            let mut tx = business.begin().await.unwrap();
            sqlx::query("SELECT set_config('app.current_org', $1, true)")
                .bind(org.as_uuid().to_string())
                .execute(&mut *tx)
                .await
                .unwrap();
            let error = sqlx::query(
                "SELECT credential_id FROM auth_webauthn_credentials WHERE credential_id = $1",
            )
            .bind(credential_id)
            .fetch_all(&mut *tx)
            .await
            .expect_err("Company runtime must not inspect any credential");
            assert_eq!(
                error.as_database_error().and_then(|e| e.code()).as_deref(),
                Some("42501")
            );
            tx.rollback().await.unwrap();
        }
    }

    // Positive controls retain actual WebAuthn proof and exact subject correlation.
    for (credential, authenticator, user, org) in [
        (&knl_cred, &mut knl_authenticator, knl_user, knl),
        (&t2_cred, &mut t2_authenticator, t2_user, org2),
    ] {
        let authentication = service.start_authentication(&auth).await.unwrap();
        let challenge = inject_allow_credential(authentication.challenge, credential);
        let assertion = authenticator
            .do_authentication(Url::parse("https://auth.example.com").unwrap(), challenge)
            .unwrap();
        let outcome = service
            .finish_authentication(&auth, authentication.ceremony_id, assertion)
            .await
            .unwrap();
        assert_eq!(outcome.user_id, user);
        assert_eq!(outcome.org_id, org);
    }
}

// ===========================================================================
// (4) Roster import (KNL) must also pass RLS through its supplied restricted pool: it writes users +
// bootstrap credentials stamped KNL, so the GUC must be armed.
// ===========================================================================
#[sqlx::test(migrations = "../db/migrations")]
async fn roster_import_works_as_runtime_role(owner_pool: PgPool) {
    let rt_pool = login_test_pool(&owner_pool, TestDatabaseLogin::Business).await;
    // The KNL org must exist for the FK on users.org_id.
    seed_org_and_user(&owner_pool, *OrgId::knl().as_uuid(), "KNL").await;
    // A region + branch are required for the roster's branch memberships.
    let (region, branch) = seed_region_branch(&owner_pool, *OrgId::knl().as_uuid()).await;

    let roster = serde_json::json!({
        "users": [{
            "display_name": "Roster User",
            "phone": "010-0000-0001",
            "team": "정비",
            "roles": ["MECHANIC"],
            "branches": [{ "region": region, "branch": branch }],
        }]
    })
    .to_string();

    let report = RosterProvisioner::new(Duration::hours(24))
        .import_json(&rt_pool, &roster, OffsetDateTime::now_utc())
        .await
        .expect(
            "roster import must pass RLS through its supplied restricted pool (GUC armed to KNL)",
        );
    assert_eq!(report.users_created, 1);
    assert_eq!(report.bootstrap_credentials_issued.len(), 1);
}

/// Seed a region + branch (owner pool, row_security off) and return their names.
async fn seed_region_branch(owner_pool: &PgPool, org: Uuid) -> (String, String) {
    let mut tx = owner_pool.begin().await.unwrap();
    sqlx::query("SET LOCAL row_security = off")
        .execute(&mut *tx)
        .await
        .unwrap();
    let region_name = "수도권".to_owned();
    let branch_name = "Seed Branch".to_owned();
    let region_id: Uuid =
        sqlx::query_scalar("INSERT INTO regions (name, org_id) VALUES ($1, $2) RETURNING id")
            .bind(&region_name)
            .bind(org)
            .fetch_one(&mut *tx)
            .await
            .unwrap();
    sqlx::query("INSERT INTO branches (region_id, name, org_id) VALUES ($1, $2, $3)")
        .bind(region_id)
        .bind(&branch_name)
        .bind(org)
        .execute(&mut *tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    (region_name, branch_name)
}

/// Count `user_id`'s passkeys through its supplied restricted pool with the GUC armed to `org`.
async fn passkey_count_as_runtime(rt_pool: &PgPool, org: OrgId, user_id: Uuid) -> i64 {
    let mut tx = rt_pool.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.current_org', $1, true)")
        .bind(org.as_uuid().to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM auth_webauthn_credentials WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&mut *tx)
            .await
            .unwrap();
    tx.commit().await.unwrap();
    count
}

// ===========================================================================
// (5) Admin credential RESET (account-recovery escape hatch): a user who already
// has a passkey gets it revoked AND a fresh OTP minted, atomically, through its supplied restricted pool.
// The OLD passkey then no longer authenticates and the new OTP redeems + lets the
// user re-enroll. Proves the lockout the security trace found is recoverable.
// ===========================================================================
#[sqlx::test(migrations = "../db/migrations")]
async fn admin_credential_reset_revokes_passkey_and_issues_otp_as_runtime_role(owner_pool: PgPool) {
    let rt_pool = auth_role_pool(&owner_pool).await;
    let knl = OrgId::knl();
    let user_id = seed_org_and_user(&owner_pool, *knl.as_uuid(), "KNL").await;

    // The user has a registered passkey (their only login method).
    let service = passkey_service();
    let (old_credential_id, mut old_authenticator) =
        register_passkey_as_runtime(&service, &rt_pool, knl, user_id).await;
    assert_eq!(passkey_count_as_runtime(&rt_pool, knl, user_id).await, 1);

    // The normal admin-OTP path REFUSES a user who already has a passkey — proving
    // the lockout: an admin cannot recover this user via the ordinary path.
    let blocked = BootstrapCredentialStore
        .issue_for_zero_credential_user(
            &rt_pool,
            user_id,
            knl,
            OffsetDateTime::now_utc(),
            Duration::hours(24),
        )
        .await;
    assert!(
        blocked.is_err(),
        "issue_for_zero_credential_user must reject a user that already has a passkey"
    );

    // The RESET escape hatch: revoke ALL passkeys + mint a fresh OTP atomically.
    let issue = BootstrapCredentialStore
        .reset_credentials_for_user(
            &rt_pool,
            user_id,
            knl,
            OffsetDateTime::now_utc(),
            Duration::hours(24),
        )
        .await
        .expect("admin credential reset must succeed through its supplied restricted pool");

    // The user's passkeys are gone.
    assert_eq!(
        passkey_count_as_runtime(&rt_pool, knl, user_id).await,
        0,
        "reset must revoke every passkey for the user"
    );

    // The OLD passkey no longer authenticates: the credential row is gone, so the
    // usernameless discoverable login cannot resolve it.
    let authentication = service.start_authentication(&rt_pool).await.unwrap();
    let challenge = inject_allow_credential(authentication.challenge, &old_credential_id);
    let assertion = old_authenticator
        .do_authentication(Url::parse("https://auth.example.com").unwrap(), challenge)
        .unwrap();
    let login = service
        .finish_authentication(&rt_pool, authentication.ceremony_id, assertion)
        .await;
    assert!(
        login.is_err(),
        "the revoked passkey must no longer authenticate after a reset"
    );

    // The NEW OTP redeems for a first sign-in.
    let redemption = BootstrapCredentialStore
        .redeem_otp(&rt_pool, issue.token.as_str(), OffsetDateTime::now_utc())
        .await
        .expect("the freshly minted reset OTP must redeem through its supplied restricted pool");
    assert_eq!(redemption.user_id, user_id);
    assert_eq!(redemption.org_id, knl);
    assert!(
        redemption.requires_passkey_setup,
        "after a reset the user has no passkey and must re-enroll"
    );

    // The user can RE-ENROLL a fresh passkey (the recovery completes).
    let (new_credential_id, _) =
        register_passkey_as_runtime(&service, &rt_pool, knl, user_id).await;
    assert_ne!(new_credential_id, old_credential_id);
    assert_eq!(passkey_count_as_runtime(&rt_pool, knl, user_id).await, 1);

    // An admin-reset audit row was written (auth.passkey.admin_reset) for the old
    // credential, proving the revoke is audited.
    let reset_audits = admin_reset_audit_count(&owner_pool, user_id).await;
    assert!(
        reset_audits >= 1,
        "the passkey revoke must be audited as auth.passkey.admin_reset"
    );
}

// ===========================================================================
// (7) Self-service add-passkey STEP-UP gate: an already-enrolled user must assert
// an EXISTING passkey (user verification) before a NEW credential is issued, so a
// stolen session (bearer token, no authenticator) cannot silently add a device.
// Proves, through its supplied restricted pool: count > 0 requires step-up; a valid step-up of the user's
// OWN passkey (UV=true) is accepted; another user's passkey is rejected.
// ===========================================================================
#[sqlx::test(migrations = "../db/migrations")]
async fn add_passkey_step_up_gate_as_runtime_role(owner_pool: PgPool) {
    let rt_pool = auth_role_pool(&owner_pool).await;
    let knl = OrgId::knl();
    let user_id = seed_org_and_user(&owner_pool, *knl.as_uuid(), "KNL").await;
    let other_id = seed_org_and_user(&owner_pool, *knl.as_uuid(), "KNL2").await;

    let service = passkey_service();

    // The user already has one passkey; a second user has their own.
    let (cred_id, mut authenticator) =
        register_passkey_as_runtime(&service, &rt_pool, knl, user_id).await;
    let (other_cred_id, mut other_authenticator) =
        register_passkey_as_runtime(&service, &rt_pool, knl, other_id).await;

    // An already-enrolled user => a step-up IS required (count > 0).
    assert_eq!(
        service
            .count_user_passkeys(&rt_pool, knl, user_id)
            .await
            .unwrap(),
        1
    );

    // A VALID step-up: assert the user's OWN existing passkey (SoftPasskey sets
    // UV=true) — must be accepted.
    let auth = service.start_authentication(&rt_pool).await.unwrap();
    let challenge = inject_allow_credential(auth.challenge, &cred_id);
    let assertion = authenticator
        .do_authentication(Url::parse("https://auth.example.com").unwrap(), challenge)
        .unwrap();
    service
        .verify_step_up_for_user(&rt_pool, auth.ceremony_id, assertion, user_id)
        .await
        .expect("a fresh step-up of the user's own passkey must be accepted");

    // A step-up using ANOTHER user's passkey must NOT unlock add-device for this
    // user, even though the assertion itself is valid.
    let auth2 = service.start_authentication(&rt_pool).await.unwrap();
    let challenge2 = inject_allow_credential(auth2.challenge, &other_cred_id);
    let assertion2 = other_authenticator
        .do_authentication(Url::parse("https://auth.example.com").unwrap(), challenge2)
        .unwrap();
    let wrong_owner = service
        .verify_step_up_for_user(&rt_pool, auth2.ceremony_id, assertion2, user_id)
        .await;
    assert!(
        wrong_owner.is_err(),
        "a step-up asserting another user's passkey must be rejected for this user"
    );

    // A consumed step-up ceremony cannot be replayed (single-use), so a captured
    // assertion can't be reused to add another credential.
    let auth3 = service.start_authentication(&rt_pool).await.unwrap();
    let challenge3 = inject_allow_credential(auth3.challenge, &cred_id);
    let assertion3 = authenticator
        .do_authentication(Url::parse("https://auth.example.com").unwrap(), challenge3)
        .unwrap();
    service
        .verify_step_up_for_user(&rt_pool, auth3.ceremony_id, assertion3.clone(), user_id)
        .await
        .expect("first use of the ceremony succeeds");
    let replay = service
        .verify_step_up_for_user(&rt_pool, auth3.ceremony_id, assertion3, user_id)
        .await;
    assert!(
        replay.is_err(),
        "a step-up ceremony is single-use and cannot be replayed"
    );
}

/// Count `auth.passkey.admin_reset` audit rows for `user_id` (owner pool, RLS off).
async fn admin_reset_audit_count(owner_pool: &PgPool, user_id: Uuid) -> i64 {
    let mut tx = owner_pool.begin().await.unwrap();
    sqlx::query("SET LOCAL row_security = off")
        .execute(&mut *tx)
        .await
        .unwrap();
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM audit_events
        WHERE action = 'auth.passkey.admin_reset'
          AND actor = $1
        "#,
    )
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();
    count
}

// ===========================================================================
// (6) Cross-org isolation for the reset: a reset issued under tenant A's GUC must
// NOT touch tenant B's user. As console_rt, a reset run with the WRONG tenant armed
// sees zero of the target's passkeys (RLS) and cannot revoke them — the escape
// hatch is tenant-scoped exactly like every other auth path.
// ===========================================================================
#[sqlx::test(migrations = "../db/migrations")]
async fn admin_credential_reset_is_tenant_scoped_as_runtime_role(owner_pool: PgPool) {
    let rt_pool = auth_role_pool(&owner_pool).await;
    let knl = OrgId::knl();
    let org2 = OrgId::from_uuid(ORG_T2);

    // A user in tenant T2 with a passkey.
    let t2_user = seed_org_and_user(&owner_pool, ORG_T2, "T2").await;
    let service = passkey_service();
    let (t2_credential_id, _) =
        register_passkey_as_runtime(&service, &rt_pool, org2, t2_user).await;
    assert_eq!(passkey_count_as_runtime(&rt_pool, org2, t2_user).await, 1);

    // A reset run with the WRONG tenant (KNL) armed must NOT revoke T2's passkey:
    // under KNL's GUC the T2 passkey row is invisible (RLS), so the DELETE matches
    // zero rows. The reset would still mint a KNL-stamped OTP for the (KNL-invisible)
    // user id, but the cross-tenant DELETE is the security-critical assertion.
    let _ = BootstrapCredentialStore
        .reset_credentials_for_user(
            &rt_pool,
            t2_user,
            knl,
            OffsetDateTime::now_utc(),
            Duration::hours(24),
        )
        .await;

    // T2's passkey is untouched: a cross-org reset cannot revoke another tenant's
    // credential.
    assert_eq!(
        passkey_count_as_runtime(&rt_pool, org2, t2_user).await,
        1,
        "a reset armed to the wrong tenant must NOT revoke another org's passkey"
    );
    assert_eq!(
        credential_org_as_runtime(&rt_pool, org2, &t2_credential_id).await,
        Some(ORG_T2),
        "T2's credential must survive a cross-org reset attempt"
    );
}

// ===========================================================================
// (5) Open self-service signup (#38): create a NEW MEMBER user in KNL + mint its
// OTP, ATOMICALLY, through its supplied restricted pool. The signup INSERTs into `users` (FORCE RLS) and
// `auth_bootstrap_credentials` (FORCE RLS) stamped KNL, so the GUC must be armed
// by `with_audits` or the WITH CHECK rejects the row. Then the new user redeems
// its own code and gets a session — the same first-sign-in path, all through its supplied restricted pool.
// ===========================================================================
#[sqlx::test(migrations = false)]
async fn open_signup_creates_member_and_redeems_as_runtime_role(owner_pool: PgPool) {
    prepare_account_test_database(&owner_pool).await;
    let rt_pool = login_test_pool(&owner_pool, TestDatabaseLogin::Business).await;
    let auth_pool = auth_role_pool(&owner_pool).await;
    let knl = OrgId::knl();

    // Self-service signup: create the MEMBER user in KNL + mint its OTP through its supplied restricted pool.
    let issue = BootstrapCredentialStore
        .signup_open_member(
            &rt_pool,
            "newcomer",
            OffsetDateTime::now_utc(),
            Duration::hours(1),
        )
        .await
        .expect(
            "open signup must create the user + OTP under RLS through its supplied restricted pool",
        );

    // The new user exists in KNL with exactly the lowest-privilege MEMBER role —
    // verified through its supplied restricted pool under KNL's GUC (it would be invisible under any other).
    let roles = user_roles_as_runtime(&rt_pool, knl, issue.user_id).await;
    assert_eq!(
        roles,
        Some(vec!["MEMBER".to_owned()]),
        "an open-signup user must hold exactly the MEMBER role"
    );

    // The credential is stamped KNL (not a foreign tenant), proving the WITH CHECK
    // accepted it under the armed GUC.
    let stamped = bootstrap_org_as_runtime(&auth_pool, knl, issue.token.as_str()).await;
    assert_eq!(stamped, Some(*knl.as_uuid()));

    // First sign-in: the new MEMBER redeems its own emailed code through its supplied restricted pool.
    let redemption = BootstrapCredentialStore
        .redeem_otp(&auth_pool, issue.token.as_str(), OffsetDateTime::now_utc())
        .await
        .expect("the open-signup OTP must redeem through its supplied restricted pool");
    assert_eq!(redemption.user_id, issue.user_id);
    assert_eq!(redemption.org_id, knl);
    assert!(redemption.requires_passkey_setup);

    // And the redeemed MEMBER can mint a session (refresh-family issue is RLS-gated).
    RefreshTokenStore
        .issue_family(
            &auth_pool,
            &auth_pool,
            issue.user_id,
            knl,
            OffsetDateTime::now_utc(),
            Duration::days(30),
        )
        .await
        .expect("the new MEMBER's session mint must pass RLS through its supplied restricted pool");
}

/// Read a user's `roles` array by id, through its supplied restricted pool with the GUC armed to `org`
/// (so the FORCE-RLS read on `users` is allowed). `None` when the row is invisible
/// under that tenant.
async fn user_roles_as_runtime(rt_pool: &PgPool, org: OrgId, user_id: Uuid) -> Option<Vec<String>> {
    let mut tx = rt_pool.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.current_org', $1, true)")
        .bind(org.as_uuid().to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
    let roles: Option<Vec<String>> = sqlx::query_scalar("SELECT roles FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    roles
}
