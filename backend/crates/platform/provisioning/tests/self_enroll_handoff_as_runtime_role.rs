#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Real-login compatibility tests for unmigrated legacy self-handoff.
//! Auth proof, handoff, passkey and consume operations use console_auth_rt after
//! actual migration/operator finalization. Company audit reads use console_rt;
//! raw credential reads are denied to Business under every Company selector.
//! Preserve exact owner/provenance, expiry, supersession and single consumption.
//! These legacy fixtures do not authorize employer recovery for migrated Accounts.

use console_kernel_core::OrgId;
use console_platform_auth::{
    PasskeyRegistrationStart, PasskeyService, RefreshTokenStore, WebauthnSettings,
};
use console_platform_provisioning::{BootstrapCredentialStore, ProvisioningError};
use console_platform_test_support::{
    TestDatabaseLogin, login_test_pool, prepare_account_test_database,
};
use sqlx::PgPool;
use time::{Duration, OffsetDateTime};
use url::Url;
use uuid::Uuid;
use webauthn_authenticator_rs::WebauthnAuthenticator;
use webauthn_authenticator_rs::softpasskey::SoftPasskey;

/// A second legacy Company selector, used to verify raw Business credential denial.
const ORG_T2: Uuid = Uuid::from_u128(0x3333_3333_3333_3333_3333_3333_3333_3333);

/// Short handoff TTL the REST layer uses (5 min); mirrored here.
const HANDOFF_TTL: Duration = Duration::minutes(5);

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

/// Authenticate directly as the same restricted Auth login used by production.
async fn auth_role_pool(owner_pool: &PgPool) -> PgPool {
    login_test_pool(owner_pool, TestDatabaseLogin::Auth).await
}

/// Seed an `organizations` row + one user in it, as the OWNER (superuser) pool
/// with `row_security` off so the rows go in regardless of any GUC. Returns the
/// new user's id.
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

/// Register a discoverable passkey for `user_id` in `org` as Auth, returning
/// the stored credential id. The registration-finish INSERT is org-stamped.
async fn register_passkey_as_runtime(
    service: &PasskeyService,
    auth_pool: &PgPool,
    org: OrgId,
    user_id: Uuid,
) -> String {
    let registration = service
        .start_registration(
            auth_pool,
            org,
            PasskeyRegistrationStart {
                user_id,
                username: "handoff.user".to_owned(),
                display_name: "Handoff User".to_owned(),
            },
        )
        .await
        .expect("start_registration must succeed as the restricted Auth login");

    let mut authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let credential = authenticator
        .do_registration(
            Url::parse("https://auth.example.com").unwrap(),
            registration.challenge,
        )
        .unwrap();

    let stored = service
        .finish_registration(auth_pool, org, registration.ceremony_id, credential)
        .await
        .expect("finish_registration must INSERT the passkey as the restricted Auth login");
    stored.credential_id
}

/// Consume the user's open code atomically (the single point of single-use
/// enforcement; in production this runs in the same tx as the passkey insert).
async fn consume_open_code_as_runtime(auth_pool: &PgPool, org: OrgId, user_id: Uuid) {
    let mut tx = auth_pool.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.current_org', $1, true)")
        .bind(org.as_uuid().to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
    BootstrapCredentialStore
        .consume_open_credentials_tx(&mut tx, org, user_id, OffsetDateTime::now_utc())
        .await
        .expect("consume must succeed as the restricted Auth login");
    tx.commit().await.unwrap();
}

/// Inspect the exact OTP's owning user and legacy Company through Auth custody.
/// This observation is not Company-GUC authorization; Business denial is separate.
async fn handoff_owner_as_runtime(
    auth_pool: &PgPool,
    org: OrgId,
    otp: &str,
) -> Option<(Uuid, Uuid)> {
    use sha2::{Digest, Sha256};
    let token_hash = Sha256::digest(otp.as_bytes()).to_vec();
    let mut tx = auth_pool.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.current_org', $1, true)")
        .bind(org.as_uuid().to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
    let row: Option<(Uuid, Uuid)> = sqlx::query_as(
        "SELECT user_id, org_id FROM auth_bootstrap_credentials WHERE token_hash = $1",
    )
    .bind(&token_hash)
    .fetch_optional(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();
    row
}

// ===========================================================================
// (1) SELF-ONLY: the minted handoff belongs to exactly the issuing user, is
// stamped with the issuer's own org, with raw Business reads denied in both scopes.
// ===========================================================================
#[sqlx::test(migrations = false)]
async fn handoff_is_scoped_to_the_issuing_user_only(owner_pool: PgPool) {
    prepare_account_test_database(&owner_pool).await;
    let auth_pool = auth_role_pool(&owner_pool).await;
    let knl = OrgId::knl();
    let user_id = seed_org_and_user(&owner_pool, *knl.as_uuid(), "KNL").await;
    // A second Company exists so both selectors are tested for credential denial.
    let _other = seed_org_and_user(&owner_pool, ORG_T2, "T2").await;

    let issue = BootstrapCredentialStore
        .issue_self_enroll_handoff(
            &auth_pool,
            user_id,
            knl,
            OffsetDateTime::now_utc(),
            HANDOFF_TTL,
        )
        .await
        .expect("self-handoff issuance must succeed as the restricted Auth login");

    // Owned by exactly the issuing user, stamped with the issuer's own org.
    let owner = handoff_owner_as_runtime(&auth_pool, knl, issue.token.as_str()).await;
    assert_eq!(
        owner,
        Some((user_id, *knl.as_uuid())),
        "handoff must be owned by the issuing user and stamped with their own org"
    );

    // Neither the owning nor another Company selector grants Business access.
    // Auth's positive provenance read above must not be mistaken for Business RLS.
    let business = login_test_pool(&owner_pool, TestDatabaseLogin::Business).await;
    let token_hash = {
        use sha2::{Digest, Sha256};
        Sha256::digest(issue.token.as_str().as_bytes()).to_vec()
    };
    for org in [knl, OrgId::from_uuid(ORG_T2)] {
        let mut tx = business.begin().await.unwrap();
        sqlx::query("SELECT set_config('app.current_org', $1, true)")
            .bind(org.as_uuid().to_string())
            .execute(&mut *tx)
            .await
            .unwrap();
        let error = sqlx::query_as::<_, (Uuid, Uuid)>(
            "SELECT user_id, org_id FROM auth_bootstrap_credentials WHERE token_hash = $1",
        )
        .bind(&token_hash)
        .fetch_optional(&mut *tx)
        .await
        .expect_err("Business cannot inspect any handoff credential");
        assert_eq!(
            error
                .as_database_error()
                .and_then(|error| error.code())
                .as_deref(),
            Some("42501"),
            "both Company selectors must retain credential custody"
        );
        tx.rollback().await.unwrap();
    }
}

// ===========================================================================
// (2) HANDOFF → ENROLL → SINGLE-USE: a handoff redeems, the user enrolls a
// passkey (which consumes the code), and a SECOND redeem of the same code fails.
// ===========================================================================
#[sqlx::test(migrations = false)]
async fn handoff_redeems_then_enroll_consumes_it_single_use(owner_pool: PgPool) {
    prepare_account_test_database(&owner_pool).await;
    let auth_pool = auth_role_pool(&owner_pool).await;
    let business = login_test_pool(&owner_pool, TestDatabaseLogin::Business).await;
    let knl = OrgId::knl();
    let user_id = seed_org_and_user(&owner_pool, *knl.as_uuid(), "KNL").await;

    let issue = BootstrapCredentialStore
        .issue_self_enroll_handoff(
            &auth_pool,
            user_id,
            knl,
            OffsetDateTime::now_utc(),
            HANDOFF_TTL,
        )
        .await
        .expect("self-handoff issuance must succeed as the restricted Auth login");
    let otp = issue.token.as_str().to_owned();

    // The phone redeems the handoff (first sign-in path) and gets a session.
    let redemption = BootstrapCredentialStore
        .redeem_otp(&auth_pool, &otp, OffsetDateTime::now_utc())
        .await
        .expect("handoff redeem must find the credential as the restricted Auth login");
    assert_eq!(redemption.user_id, user_id);
    assert_eq!(redemption.org_id, knl);
    assert!(
        redemption.requires_passkey_setup,
        "a zero-passkey user redeeming a handoff must be flagged for enrollment"
    );

    RefreshTokenStore
        .issue_family(
            &business,
            &auth_pool,
            user_id,
            knl,
            OffsetDateTime::now_utc(),
            Duration::days(30),
        )
        .await
        .expect("session mint must pass RLS as the restricted Auth login");

    // The phone enrolls a passkey; enrollment consumes the open handoff code
    // atomically (production runs this in the register-finish transaction).
    let service = passkey_service();
    register_passkey_as_runtime(&service, &auth_pool, knl, user_id).await;
    consume_open_code_as_runtime(&auth_pool, knl, user_id).await;

    // SINGLE-USE: the same handoff code can never mint another session.
    let replay = BootstrapCredentialStore
        .redeem_otp(&auth_pool, &otp, OffsetDateTime::now_utc())
        .await;
    assert!(
        matches!(replay, Err(ProvisioningError::InvalidBootstrapCredential)),
        "a consumed handoff must be rejected on replay, got {replay:?}"
    );
}

// ===========================================================================
// (2b) CONSUME IS TENANT-VISIBLE: the `auth.otp.consume` audit row must carry
// the caller's org so a tenant-scoped `/api/audit` read sees it.
//
// Regression for the task #26 deferred site: the consume event was built without
// `.with_org`, landing the row with NULL org_id. The FORCE-RLS WITH CHECK permits
// NULL so the write succeeded, but RLS `USING (org_id = app.current_org)` then
// hid it from every tenant read. RED (pre-fix): the KNL-armed read returns 0.
// ===========================================================================
#[sqlx::test(migrations = false)]
async fn consume_audit_row_is_tenant_scoped_and_visible_as_runtime_role(owner_pool: PgPool) {
    prepare_account_test_database(&owner_pool).await;
    let auth_pool = auth_role_pool(&owner_pool).await;
    let business = login_test_pool(&owner_pool, TestDatabaseLogin::Business).await;
    let knl = OrgId::knl();
    let user_id = seed_org_and_user(&owner_pool, *knl.as_uuid(), "KNL").await;

    // Auth issues and consumes; Business retains the tenant-scoped audit read.
    BootstrapCredentialStore
        .issue_self_enroll_handoff(
            &auth_pool,
            user_id,
            knl,
            OffsetDateTime::now_utc(),
            HANDOFF_TTL,
        )
        .await
        .expect("self-handoff issuance must succeed as the restricted Auth login");
    consume_open_code_as_runtime(&auth_pool, knl, user_id).await;

    // As `console_rt`, armed to KNL: the consume event must be visible AND stamped
    // with KNL. A NULL-org row would be invisible here (RLS USING excludes it).
    let mut tx = business.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.current_org', $1, true)")
        .bind(knl.as_uuid().to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
    let org_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT org_id FROM audit_events WHERE action = 'auth.otp.consume' AND actor = $1",
    )
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await
    .expect("KNL-armed read must see the consume audit row");
    tx.commit().await.unwrap();
    assert_eq!(
        org_id,
        Some(*knl.as_uuid()),
        "the consume audit row must carry the caller's org, not NULL"
    );
}

// ===========================================================================
// (3) EXPIRY: a handoff that has lapsed does not redeem.
// ===========================================================================
#[sqlx::test(migrations = false)]
async fn expired_handoff_does_not_redeem(owner_pool: PgPool) {
    prepare_account_test_database(&owner_pool).await;
    let auth_pool = auth_role_pool(&owner_pool).await;
    let knl = OrgId::knl();
    let user_id = seed_org_and_user(&owner_pool, *knl.as_uuid(), "KNL").await;

    // Issue a handoff timestamped in the past so it is already expired.
    let issued_at = OffsetDateTime::now_utc() - Duration::hours(1);
    let issue = BootstrapCredentialStore
        .issue_self_enroll_handoff(&auth_pool, user_id, knl, issued_at, HANDOFF_TTL)
        .await
        .expect("self-handoff issuance must succeed as the restricted Auth login");

    let result = BootstrapCredentialStore
        .redeem_otp(&auth_pool, issue.token.as_str(), OffsetDateTime::now_utc())
        .await;
    assert!(
        matches!(result, Err(ProvisioningError::InvalidBootstrapCredential)),
        "an expired handoff must not redeem, got {result:?}"
    );
}

// ===========================================================================
// (4) SUPERSEDE: minting a handoff while the user already holds an OPEN code (the
// one they redeemed but have not yet enrolled against) revokes the stale code and
// the FRESH one redeems while the OLD one no longer does. The one-open-per-user
// partial-unique index forbids two live codes; this proves the supersede path.
// ===========================================================================
#[sqlx::test(migrations = false)]
async fn handoff_supersedes_a_users_existing_open_code(owner_pool: PgPool) {
    prepare_account_test_database(&owner_pool).await;
    let auth_pool = auth_role_pool(&owner_pool).await;
    let knl = OrgId::knl();
    let user_id = seed_org_and_user(&owner_pool, *knl.as_uuid(), "KNL").await;

    // First handoff: the user's initial open code.
    let first = BootstrapCredentialStore
        .issue_self_enroll_handoff(
            &auth_pool,
            user_id,
            knl,
            OffsetDateTime::now_utc(),
            HANDOFF_TTL,
        )
        .await
        .expect("first handoff issuance must succeed");

    // Second handoff (e.g. the desktop re-requests a QR): must revoke the first
    // and mint a fresh one — without erroring on the one-open-per-user index.
    let second = BootstrapCredentialStore
        .issue_self_enroll_handoff(
            &auth_pool,
            user_id,
            knl,
            OffsetDateTime::now_utc(),
            HANDOFF_TTL,
        )
        .await
        .expect("a second handoff must supersede the first, not conflict");
    assert_ne!(
        first.token.as_str(),
        second.token.as_str(),
        "the superseding handoff must be a fresh code"
    );

    // The OLD code is now dead; the FRESH code redeems.
    let stale = BootstrapCredentialStore
        .redeem_otp(&auth_pool, first.token.as_str(), OffsetDateTime::now_utc())
        .await;
    assert!(
        matches!(stale, Err(ProvisioningError::InvalidBootstrapCredential)),
        "the superseded handoff must no longer redeem, got {stale:?}"
    );

    let fresh = BootstrapCredentialStore
        .redeem_otp(&auth_pool, second.token.as_str(), OffsetDateTime::now_utc())
        .await
        .expect("the fresh handoff must redeem as the restricted Auth login");
    assert_eq!(fresh.user_id, user_id);
    assert_eq!(fresh.org_id, knl);
}
