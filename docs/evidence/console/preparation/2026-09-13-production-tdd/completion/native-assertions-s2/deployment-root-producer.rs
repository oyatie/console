//! Ordinary Account-aware replacement of provisioning's existing cold-start
//! credential flow. Same production path with isolated test startup configuration.
//! No legacy sentinel Company, OTP->business JWT, or assumed initial Account session.
use console_platform_provisioning::BootstrapCredentialStore;
use console_platform_auth::{PasskeyService,JwtVerifier,account_enrollment as accounts};
use console_platform_auth::account_session::{verify_account_session,VerifiedAccountSession};
use webauthn_authenticator_rs::{prelude::WebauthnAuthenticator,softpasskey::SoftPasskey};
use rand::RngCore;
use sha2::{Digest,Sha256};
use sqlx::PgPool;
use url::Url;
use crate::native_fixture::TestResult;

pub async fn enroll_deployment_operator(
    startup_auth_pool:&PgPool,service:&PasskeyService,verifier:&JwtVerifier,origin:&Url,
    configured_root_secret:&str,published:&console_platform_auth::terms_publication::PublicationReceipt,
)->TestResult<VerifiedAccountSession> {
    // Actual deployment-publisher receipt is checked against the current owner
    // terms head BEFORE root seeding; no Account authorizes genesis publication.
    let terms=accounts::read_current_account_terms(startup_auth_pool).await?;
    assert_eq!(terms.reference.receipt_id,published.id);
    assert_eq!(terms.reference.revision,published.revision);
    assert_eq!(terms.reference.manifest_sha256,published.manifest_sha256);
    let store=BootstrapCredentialStore;
    // Account-aware extension of seed_cold_start_credential in this SAME owner.
    // Startup auth-pool role is real privileged deployment configuration. The
    // owner creates/locates the designated pending Account root, hashes the secret,
    // and atomically checks uninitialized-root/expiry/no-existing-key constraints.
    // Not callable via Company runtime SQL grants. No test-specific namespace.
    let seed=store.seed_account_cold_start_credential(startup_auth_pool,configured_root_secret,
        time::Duration::minutes(5),time::OffsetDateTime::now_utc()).await?;
    let mut nonce=[0u8;32];rand::rngs::OsRng.fill_bytes(&mut nonce);
    // Actual redemption of configured one-time root proof creates only an
    // Account-bound registration ceremony; it never directly returns a session.
    let started=store.start_account_root_enrollment(startup_auth_pool,service,
        accounts::RootEnrollmentStart{credential_id:seed.credential_id,root_secret:configured_root_secret.to_owned(),terms:terms.reference.clone(),browser_nonce_digest:hex::encode(Sha256::digest(nonce))}).await?;
    assert_eq!(started.account_id,seed.account_id);
    let mut authenticator=WebauthnAuthenticator::new(SoftPasskey::new(true));
    let credential=authenticator.do_registration(origin.clone(),started.challenge)?;
    // SAME real generic registration verifier/terms/Account/family path used by
    // ordinary enrollment. Designated-root role activation is an identity-owned
    // guarded consequence of its stored root ceremony proof, never input bool.
    let finished=accounts::finish_account_enrollment(startup_auth_pool,service,
        accounts::EnrollmentFinish{enrollment_id:started.enrollment_id,ceremony_id:started.ceremony_id,browser_nonce:hex::encode(nonce),terms:terms.reference,credential}).await?;
    let actual=verify_account_session(startup_auth_pool,verifier,&finished.account_access_token).await?;
    assert_eq!(actual.account_id(),seed.account_id);
    assert_eq!(actual.family_id(),finished.session_family_id);
    Ok(actual)
}
