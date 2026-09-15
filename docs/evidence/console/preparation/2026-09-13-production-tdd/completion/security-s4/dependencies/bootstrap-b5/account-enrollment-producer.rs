//! Authored Account13 producer using the existing real WebAuthn authenticator
//! and verifier libraries. New Account-aware OWNER use cases are prerequisites;
//! this does not use legacy seed_user INSERT, Company JWT, or fake context.
use console_platform_auth::{PasskeyService,WebauthnSettings,JwtVerifier};
use console_platform_auth::account_enrollment as accounts;
use console_platform_auth::account_session::{verify_account_session};
use webauthn_authenticator_rs::{prelude::{RequestChallengeResponse,WebauthnAuthenticator},softpasskey::SoftPasskey};
use sqlx::PgPool;
use url::Url;
use sha2::{Digest,Sha256};
use rand::RngCore;
use uuid::Uuid;
use crate::native_fixture::TestResult;

pub struct EnrolledAccount {
    pub account_id:Uuid,
    pub credential_id:String,
    pub account_access_token:String,
    pub authenticator:WebauthnAuthenticator<SoftPasskey>,
}

pub async fn enroll_account(
    auth_pool:&PgPool,service:&PasskeyService,verifier:&JwtVerifier,origin:&Url,display_name:&str,
)->TestResult<EnrolledAccount> {
    // Terms version/digest is an actual current Account owner read. The fixture
    // explicitly accepts this configured TEST terms version, never old unknown consent.
    let terms=accounts::read_current_account_terms(auth_pool).await?;
    let mut browser_nonce=[0u8;32];rand::rngs::OsRng.fill_bytes(&mut browser_nonce);
    let started=accounts::start_account_enrollment(auth_pool,service,accounts::EnrollmentStart {
        display_name:display_name.to_owned(),terms:terms.reference.clone(),
        browser_nonce_digest:hex::encode(Sha256::digest(browser_nonce)),
    }).await?;
    // Actual challenge signed by real software passkey. No verified flag, server
    // nonce replacement or assertion bypass. Server consumes its stored ceremony.
    let mut authenticator=WebauthnAuthenticator::new(SoftPasskey::new(true));
    let credential=authenticator.do_registration(origin.clone(),started.challenge)?;
    let finished=accounts::finish_account_enrollment(auth_pool,service,accounts::EnrollmentFinish {
        enrollment_id:started.enrollment_id,ceremony_id:started.ceremony_id,
        browser_nonce:hex::encode(browser_nonce),terms:terms.reference,
        credential,
    }).await?;
    assert_eq!(finished.account_id,started.account_id);
    let session=verify_account_session(auth_pool,verifier,&finished.account_access_token).await?;
    assert_eq!(session.account_id(),finished.account_id);
    assert_eq!(session.family_id(),finished.session_family_id);
    assert_eq!(session.assurance(),accounts::AccountAssurance::PasskeyPrimary);
    Ok(EnrolledAccount{account_id:finished.account_id,credential_id:finished.credential_id,
        account_access_token:finished.account_access_token,authenticator})
}

pub async fn fresh_primary_login(
    auth_pool:&PgPool,service:&PasskeyService,verifier:&JwtVerifier,origin:&Url,account:&mut EnrolledAccount,
)->TestResult<String> {
    let started=accounts::start_account_authentication(auth_pool,service).await?;
    // Same local SoftPasskey compatibility as existing auth/tests/webauthn_ceremony.rs:
    // server ceremony stays discoverable; only local key-selection hint changes.
    let mut challenge=serde_json::to_value(&started.challenge)?;
    challenge["publicKey"]["allowCredentials"].as_array_mut().ok_or("missing challenge credential list")?
        .push(serde_json::json!({"type":"public-key","id":account.credential_id}));
    let challenge:RequestChallengeResponse=serde_json::from_value(challenge)?;
    let assertion=account.authenticator.do_authentication(origin.clone(),challenge)?;
    let actual=accounts::finish_account_authentication(auth_pool,service,started.ceremony_id,assertion).await?;
    assert_eq!(actual.account_id,account.account_id);
    let verified=verify_account_session(auth_pool,verifier,&actual.account_access_token).await?;
    assert_eq!(verified.account_id(),account.account_id);
    assert_eq!(verified.family_id(),actual.session_family_id);
    Ok(actual.account_access_token)
}

pub struct AccountBootstrapPhase {
    pub submitter:EnrolledAccount,
    pub reviewer:EnrolledAccount,
    pub same_human_alias:EnrolledAccount,
}
pub async fn enroll_fixture_accounts(auth_pool:&PgPool,service:&PasskeyService,verifier:&JwtVerifier,origin:&Url)->TestResult<AccountBootstrapPhase> {
    let submitter=enroll_account(auth_pool,service,verifier,origin,"Native fixture submitter").await?;
    let reviewer=enroll_account(auth_pool,service,verifier,origin,"Native fixture independent reviewer").await?;
    let same_human_alias=enroll_account(auth_pool,service,verifier,origin,"Native fixture alias").await?;
    assert_ne!(submitter.account_id,reviewer.account_id);
    assert_ne!(submitter.account_id,same_human_alias.account_id);
    // Distinct Account UUIDs DO NOT prove distinct Humans. Return actual Account
    // identities for the separately authorized supervised registry root process.
    // Company/Human authority is deliberately not manufactured here.
    Ok(AccountBootstrapPhase{submitter,reviewer,same_human_alias})
}
