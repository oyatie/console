//! Native Account enrollment orchestration. The caller owns one Auth transaction;
//! only the existing Auth participants and fixed identity owners mutate state.

use std::collections::BTreeSet;

use console_platform_auth::account::{
    AccountFamilyIssue, AccountOperationError, AccountRegistrationChallenge, AccountTermsHead,
    account_now_in_tx, account_terms_registration_head_in_tx, activate_account_registration_in_tx,
    begin_account_registration_in_tx, lock_pending_account_registration_in_tx,
    replace_account_registration_in_tx,
};
use console_platform_auth::{
    AccountAccessTokenInput, AccountAssurance, JwtIssuer, PasskeyRegistrationCredential,
    PasskeyService, RefreshTokenStore, SignedAccountToken,
};
use sqlx::{Postgres, Transaction};
use time::Duration;
use uuid::Uuid;

use super::ProvisioningError;

/// Exact registered content selected by the trusted artifact loader. This is
/// structured data, not a caller assertion that content was verified.
pub struct AccountTermsItem {
    pub terms_kind: String,
    pub content_sha256: [u8; 32],
}

pub struct AccountTermsArtifacts {
    manifest_sha256: [u8; 32],
    items: Vec<AccountTermsItem>,
}

impl AccountTermsArtifacts {
    /// The trusted loader calls this only after hashing the manifest and every
    /// referenced content file. Domain shape is validated again at this boundary.
    pub fn new(
        manifest_sha256: [u8; 32],
        items: Vec<AccountTermsItem>,
    ) -> Result<Self, AccountOperationError> {
        let mut kinds = BTreeSet::new();
        if !(1..=8).contains(&items.len())
            || items.iter().any(|item| {
                !valid_kind(&item.terms_kind) || !kinds.insert(item.terms_kind.as_str())
            })
        {
            return Err(AccountOperationError::AuthorityUnavailable);
        }
        Ok(Self {
            manifest_sha256,
            items,
        })
    }

    pub fn required_kinds(&self) -> impl Iterator<Item = &str> {
        self.items.iter().map(|item| item.terms_kind.as_str())
    }

    /// The Auth owner has reconstructed immutable same-Account evidence. The
    /// trusted loader must supply that exact historical manifest and contents;
    /// existence of a release row or an unrelated local bundle is insufficient.
    pub fn require_historical_consent(
        &self,
        consent: &console_platform_auth::account::AccountHistoricalConsent,
    ) -> Result<(), AccountOperationError> {
        if consent.manifest_sha256 != self.manifest_sha256 {
            return Err(AccountOperationError::AuthorityUnavailable);
        }
        let mut accepted = std::collections::BTreeMap::new();
        for (kind, digest) in &consent.items {
            if accepted.insert(kind.as_str(), digest).is_some() {
                return Err(AccountOperationError::TermsAcceptanceRequired);
            }
        }
        if accepted.len() != self.items.len()
            || self.items.iter().any(|item| {
                accepted
                    .get(item.terms_kind.as_str())
                    .is_none_or(|digest| **digest != item.content_sha256)
            })
        {
            return Err(AccountOperationError::TermsAcceptanceRequired);
        }
        Ok(())
    }

    fn require_current(
        &self,
        head: &AccountTermsHead,
        requested_digest: &[u8; 32],
    ) -> Result<(), AccountOperationError> {
        if head.revision <= 0
            || head.release_receipt_id.is_nil()
            || head.manifest_sha256.as_slice() != self.manifest_sha256.as_slice()
        {
            // Unsupported current publication is unavailable, not fabricated from
            // the local artifact index or the browser's supplied terms version.
            return Err(AccountOperationError::AuthorityUnavailable);
        }
        if requested_digest != &self.manifest_sha256 {
            return Err(AccountOperationError::TermsChanged);
        }
        Ok(())
    }
}

fn valid_kind(kind: &str) -> bool {
    (1..=64).contains(&kind.len())
        && kind.as_bytes()[0].is_ascii_lowercase()
        && kind.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'.' | b'-')
        })
}

pub struct AccountRegistrationStartInput<'a> {
    pub terms: &'a AccountTermsArtifacts,
    pub requested_terms: [u8; 32],
    pub previous_nonce: Option<&'a str>,
    pub origin: &'a str,
}

pub struct AccountRegistrationFinishInput<'a> {
    pub terms: &'a AccountTermsArtifacts,
    pub accepted_terms: [u8; 32],
    pub accepted_kinds: &'a [&'a str],
    pub ceremony_id: Uuid,
    pub browser_nonce: &'a str,
    pub origin: &'a str,
    pub credential: PasskeyRegistrationCredential,
    pub refresh_ttl: Duration,
    pub absolute_family_ttl: Duration,
}

/// Tokens remain private until the caller validates the resulting live session
/// and commits. Drop/rollback on any failure; never emit a partial credential.
pub struct AccountRegistrationIssued {
    pub access: SignedAccountToken,
    pub family: AccountFamilyIssue,
}

pub async fn start_account_registration_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    passkeys: &PasskeyService,
    input: AccountRegistrationStartInput<'_>,
) -> Result<AccountRegistrationChallenge, ProvisioningError> {
    // Old Account guard(s) precede a new, private UUID allocation. Replacement is
    // transactional and touches only a nonce-bound still-pending ceremony.
    replace_account_registration_in_tx(tx, input.previous_nonce, input.origin).await?;
    let pending = begin_account_registration_in_tx(tx).await?;
    let head = account_terms_registration_head_in_tx(tx).await?;
    input.terms.require_current(&head, &input.requested_terms)?;
    let challenge = passkeys
        .start_account_registration_in_tx(tx, &pending, &head, input.origin)
        .await?;
    Ok(challenge)
}

pub async fn finish_account_registration_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    passkeys: &PasskeyService,
    issuer: &JwtIssuer,
    input: AccountRegistrationFinishInput<'_>,
) -> Result<AccountRegistrationIssued, ProvisioningError> {
    let supplied: BTreeSet<_> = input.accepted_kinds.iter().copied().collect();
    let required: BTreeSet<_> = input.terms.required_kinds().collect();
    if supplied.len() != input.accepted_kinds.len() || supplied != required {
        return Err(ProvisioningError::AccountTermsAcknowledgments);
    }
    let binding = passkeys
        .correlate_account_registration_in_tx(
            tx,
            input.ceremony_id,
            input.browser_nonce,
            input.origin,
        )
        .await?;
    let pending = lock_pending_account_registration_in_tx(tx, binding.account_id).await?;
    let head = account_terms_registration_head_in_tx(tx).await?;
    input.terms.require_current(&head, &input.accepted_terms)?;
    if binding.terms_head_revision != head.revision
        || binding.terms_manifest_sha256 != head.manifest_sha256
    {
        return Err(AccountOperationError::TermsChanged.into());
    }
    if pending.account_id != binding.account_id
        || pending.security_generation != 1
        || pending.revision != 1
        || pending.context_generation != 1
    {
        return Err(AccountOperationError::EnrollmentInvalid.into());
    }
    // Family/token insertion precedes key/ceremony locks. These rows are only
    // tentative until real passkey verification, acceptance and activation commit.
    let family = RefreshTokenStore
        .issue_account_family_in_tx(tx, &pending, input.refresh_ttl, input.absolute_family_ttl)
        .await?;
    let key = passkeys
        .finish_account_registration_in_tx(
            tx,
            &binding,
            input.ceremony_id,
            input.browser_nonce,
            input.origin,
            input.credential,
        )
        .await?;
    let kinds: Vec<_> = input
        .terms
        .items
        .iter()
        .map(|item| item.terms_kind.clone())
        .collect();
    let digests: Vec<_> = input
        .terms
        .items
        .iter()
        .map(|item| item.content_sha256.to_vec())
        .collect();
    activate_account_registration_in_tx(
        tx,
        pending.account_id,
        input.ceremony_id,
        key,
        family.family_id,
        &head,
        &kinds,
        &digests,
    )
    .await?;
    let access = issuer
        .issue_account_access_token(AccountAccessTokenInput {
            account_id: pending.account_id,
            session_id: family.family_id,
            security_generation: pending.security_generation,
            auth_time: family.auth_time,
            assurance: AccountAssurance::PasskeyPrimary,
            issued_at: account_now_in_tx(tx).await?,
            family_expires_at: family.family_expires_at,
        })
        .map_err(|_| AccountOperationError::AuthorityUnavailable)?;
    Ok(AccountRegistrationIssued { access, family })
}
