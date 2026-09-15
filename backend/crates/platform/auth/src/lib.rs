//! Authentication platform crate.
//!
//! T0.5 covers server-side passkey ceremonies, ES256 access JWTs, rotating
//! refresh-token families, and static app-link metadata for the RP domain.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

mod error;
mod handoff;
mod jwt;
pub use handoff::{
    DeviceLoginApproval, DeviceLoginHandoffError, DeviceLoginPoll,
    approve_device_login_with_passkey_in_tx, approve_targeted_device_login_session_in_tx,
    create_device_login_handoff_in_tx, create_self_enroll_device_handoff_in_tx,
    poll_and_consume_device_login_in_tx,
};

mod legacy;
mod refresh;
mod session;
mod webauthn;
mod well_known;

pub use error::AuthError;
pub use jwt::{
    AccessClaims, AccessTokenInput, AccountAccessClaims, AccountAccessTokenInput,
    AccountAccessVerification, AccountAssurance, AccountCsrfClaims, AccountCsrfTokenInput,
    JwtIssuer, JwtSettings, JwtVerifier, SignedAccountToken, TenantAccessContext,
};
pub use legacy::{append_legacy_auth_audit_in_tx, guard_legacy_subject_in_tx};
pub use refresh::{RefreshToken, RefreshTokenIssue, RefreshTokenStore, RefreshTokenUseError};
pub use session::SessionVerification;
pub use webauthn::{
    AuthenticationCeremony, AuthenticationOutcome, MobilePasskeyStepUpAssertion,
    MobilePasskeyStepUpBinding, MobilePasskeyStepUpEnvelope, MobilePasskeyStepUpVerificationError,
    MobileStepUpActionKind, MobileStepUpBindingError, PasskeyAuthenticationCredential,
    PasskeyRegistrationCredential, PasskeyRegistrationStart, PasskeyService, RegistrationCeremony,
    StoredPasskey, WebauthnSettings,
};
pub use well_known::{
    AndroidAssetLinksConfig, AppleAppSiteAssociationConfig, WELL_KNOWN_AASA_PATH,
    WELL_KNOWN_ASSETLINKS_PATH, android_assetlinks_json, apple_app_site_association_json,
};
