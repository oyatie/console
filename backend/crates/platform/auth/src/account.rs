//! Native Account authority participants. Every operation borrows the admitted
//! Auth transaction; callers roll back on refusal and release secrets only after commit.

use sqlx::{Postgres, Row, Transaction};
use time::OffsetDateTime;
use uuid::Uuid;

pub use crate::session::{
    AccountEnrollmentCredentials, LockedAccountEnrollment, account_csrf_session_in_tx,
    ensure_account_session_fresh_in_tx, live_account_session_in_tx, logout_account_session_in_tx,
};
use crate::{AccountAssurance, RefreshToken, RegistrationCeremony};

#[derive(Debug, thiserror::Error)]
pub enum AccountOperationError {
    #[error("authentication invalid")]
    AuthenticationInvalid,
    #[error("enrollment invalid")]
    EnrollmentInvalid,
    #[error("request proof invalid")]
    CsrfInvalid,
    #[error("ambiguous credentials")]
    AmbiguousCredentials,
    #[error("terms acceptance required")]
    TermsAcceptanceRequired,
    #[error("terms changed")]
    TermsChanged,
    #[error("authority unavailable")]
    AuthorityUnavailable,
    #[error("navigation unavailable")]
    NavigationUnavailable,
}

// Storage diagnostics may contain credential material. Never retain them in
// native public errors, Debug, or tracing.
impl From<sqlx::Error> for AccountOperationError {
    fn from(_: sqlx::Error) -> Self {
        Self::AuthorityUnavailable
    }
}

pub struct PendingAccount {
    pub account_id: Uuid,
    pub security_generation: i64,
    pub revision: i64,
    pub context_generation: i64,
    pub created_at: OffsetDateTime,
}

pub struct AccountSecurityState {
    pub account_id: Uuid,
    pub security_generation: i64,
    pub revision: i64,
    pub context_generation: i64,
}

pub struct AccountTermsHead {
    pub manifest_sha256: Vec<u8>,
    pub revision: i64,
    pub release_receipt_id: Uuid,
}

pub struct AccountRegistrationBinding {
    pub account_id: Uuid,
    pub terms_manifest_sha256: Vec<u8>,
    pub terms_head_revision: i64,
}

pub struct AccountRegistrationChallenge {
    pub ceremony: RegistrationCeremony,
    pub browser_nonce: RefreshToken,
}

pub struct AccountLoginChallenge {
    pub ceremony: crate::AuthenticationCeremony,
    pub browser_nonce: RefreshToken,
}

/// A verified native primary assertion. Its family and key/ceremony updates
/// remain tentative until historical consent, signing and HTTP commit succeed.
pub struct AccountPrimaryLogin {
    pub account_id: Uuid,
    pub security_generation: i64,
    pub family: AccountFamilyIssue,
    pub ceremony_expires_at: OffsetDateTime,
}

pub struct AccountHistoricalConsent {
    pub manifest_sha256: [u8; 32],
    pub items: Vec<(String, [u8; 32])>,
}

/// Call only after a real primary assertion. This narrow owner projection reads
/// the original immutable release and its complete Account evidence, never the
/// publication head, context candidates, or a caller-provided manifest.
pub async fn account_login_consent_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    primary: &AccountPrimaryLogin,
) -> Result<AccountHistoricalConsent, AccountOperationError> {
    let rows = sqlx::query("SELECT manifest_sha256, terms_kind, content_sha256, accepted_at FROM public.account_login_consent_v1($1)")
        .bind(primary.account_id).fetch_all(tx.as_mut()).await.map_err(|error| {
            match error.as_database_error() {
                Some(db) if db.code().as_deref() == Some("P0001")
                    && db.message() == "account.terms_acceptance_required" =>
                    AccountOperationError::TermsAcceptanceRequired,
                Some(db) if db.code().as_deref() == Some("P0001")
                    && db.message() == "account.authentication_invalid" =>
                    AccountOperationError::AuthenticationInvalid,
                _ => AccountOperationError::AuthorityUnavailable,
            }
        })?;
    if !(1..=8).contains(&rows.len()) {
        return Err(AccountOperationError::AuthorityUnavailable);
    }
    let now = account_now_in_tx(tx).await?;
    if primary.ceremony_expires_at <= now {
        return Err(AccountOperationError::AuthenticationInvalid);
    }
    let mut manifest = None;
    let mut accepted = None;
    let mut kinds = std::collections::BTreeSet::new();
    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        let digest: Vec<u8> = row.try_get("manifest_sha256")?;
        let digest: [u8; 32] = digest
            .try_into()
            .map_err(|_| AccountOperationError::AuthorityUnavailable)?;
        let content: Vec<u8> = row.try_get("content_sha256")?;
        let content = content
            .try_into()
            .map_err(|_| AccountOperationError::AuthorityUnavailable)?;
        let kind: String = row.try_get("terms_kind")?;
        let accepted_at: OffsetDateTime = row.try_get("accepted_at")?;
        if manifest.is_some_and(|previous| previous != digest)
            || accepted.is_some_and(|previous| previous != accepted_at)
            || accepted_at > now
            || !kinds.insert(kind.clone())
        {
            return Err(AccountOperationError::AuthorityUnavailable);
        }
        manifest = Some(digest);
        accepted = Some(accepted_at);
        items.push((kind, content));
    }
    Ok(AccountHistoricalConsent {
        manifest_sha256: manifest.ok_or(AccountOperationError::AuthorityUnavailable)?,
        items,
    })
}

pub struct AccountFamilyIssue {
    pub family_id: Uuid,
    pub token: RefreshToken,
    pub token_expires_at: OffsetDateTime,
    pub family_expires_at: OffsetDateTime,
    pub auth_time: OffsetDateTime,
}

pub struct AccountLiveSession {
    pub account_id: Uuid,
    pub session_id: Uuid,
    pub security_generation: i64,
    pub auth_time: OffsetDateTime,
    pub assurance: AccountAssurance,
    pub expires_at: OffsetDateTime,
    pub family_expires_at: OffsetDateTime,
}

pub async fn begin_account_registration_in_tx(
    tx: &mut Transaction<'_, Postgres>,
) -> Result<PendingAccount, AccountOperationError> {
    let row = sqlx::query("SELECT * FROM public.account_registration_begin_v1()")
        .fetch_one(tx.as_mut())
        .await?;
    let pending = PendingAccount {
        account_id: row.try_get("account_id")?,
        security_generation: row.try_get("security_generation")?,
        revision: row.try_get("revision")?,
        context_generation: row.try_get("context_generation")?,
        created_at: row.try_get("created_at")?,
    };
    if pending.account_id.is_nil()
        || pending.security_generation != 1
        || pending.revision != 1
        || pending.context_generation != 1
        || pending.created_at > account_now_in_tx(tx).await?
    {
        return Err(AccountOperationError::AuthorityUnavailable);
    }
    Ok(pending)
}

pub async fn account_terms_registration_head_in_tx(
    tx: &mut Transaction<'_, Postgres>,
) -> Result<AccountTermsHead, AccountOperationError> {
    let row = sqlx::query("SELECT * FROM public.account_terms_registration_head_v1()")
        .fetch_one(tx.as_mut())
        .await?;
    let head = AccountTermsHead {
        manifest_sha256: row.try_get("manifest_sha256")?,
        revision: row.try_get("revision")?,
        release_receipt_id: row.try_get("release_receipt_id")?,
    };
    if head.manifest_sha256.len() != 32 || head.revision <= 0 || head.release_receipt_id.is_nil() {
        return Err(AccountOperationError::AuthorityUnavailable);
    }
    Ok(head)
}

pub async fn lock_pending_account_registration_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    account: Uuid,
) -> Result<AccountSecurityState, AccountOperationError> {
    lock_account_in_tx(tx, account, true, "PENDING_ENROLLMENT")
        .await
        .map_err(|error| match error {
            AccountOperationError::AuthenticationInvalid => {
                AccountOperationError::EnrollmentInvalid
            }
            other => other,
        })
}

pub(crate) async fn lock_account_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    account: Uuid,
    exclusive: bool,
    required_state: &str,
) -> Result<AccountSecurityState, AccountOperationError> {
    if account.is_nil() {
        return Err(AccountOperationError::AuthenticationInvalid);
    }
    // Two fixed statements: no caller-controlled identifiers or role changes.
    let query = if exclusive {
        "SELECT * FROM public.account_security_lock_exclusive_v1($1)"
    } else {
        "SELECT * FROM public.account_security_lock_shared_v1($1)"
    };
    let row = sqlx::query(query)
        .bind(account)
        .fetch_optional(tx.as_mut())
        .await
        .map_err(|error| {
            if error
                .as_database_error()
                .is_some_and(|db| db.code().as_deref() == Some("P0002"))
            {
                AccountOperationError::AuthenticationInvalid
            } else {
                AccountOperationError::AuthorityUnavailable
            }
        })?
        .ok_or(AccountOperationError::AuthenticationInvalid)?;
    account_security_from_row(&row, account, required_state)
}

pub(crate) fn account_security_from_row(
    row: &sqlx::postgres::PgRow,
    account: Uuid,
    required_state: &str,
) -> Result<AccountSecurityState, AccountOperationError> {
    let state: String = row.try_get("security_state")?;
    let security = AccountSecurityState {
        account_id: account,
        security_generation: row.try_get("security_generation")?,
        revision: row.try_get("revision")?,
        context_generation: row.try_get("context_generation")?,
    };
    if security.security_generation <= 0
        || security.revision <= 0
        || security.context_generation <= 0
    {
        return Err(AccountOperationError::AuthorityUnavailable);
    }
    if state != required_state {
        return Err(AccountOperationError::AuthenticationInvalid);
    }
    Ok(security)
}

pub async fn replace_account_registration_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    previous_nonce: Option<&str>,
    origin: &str,
) -> Result<(), AccountOperationError> {
    let Some(nonce) = previous_nonce else {
        return Ok(());
    };
    if nonce.is_empty() || nonce.len() > 512 {
        return Err(AccountOperationError::EnrollmentInvalid);
    }
    let nonce_hash = crate::refresh::hash_token(nonce);
    // Correlation only, before acquiring the old Account guard.
    let row = sqlx::query("SELECT id, user_id FROM public.auth_webauthn_ceremonies WHERE account_browser_flow = 'ACCOUNT_REGISTRATION' AND browser_nonce_sha256 = $1 AND browser_origin = $2")
        .bind(&nonce_hash).bind(origin).fetch_optional(tx.as_mut()).await?
        .ok_or(AccountOperationError::EnrollmentInvalid)?;
    let account: Uuid = row.try_get("user_id")?;
    let ceremony: Uuid = row.try_get("id")?;
    lock_pending_account_registration_in_tx(tx, account).await?;
    // Acquire row before reading the clock; lock waits never extend validity.
    let row = sqlx::query("SELECT expires_at, consumed_at FROM public.auth_webauthn_ceremonies WHERE id = $1 AND user_id = $2 AND account_browser_flow = 'ACCOUNT_REGISTRATION' AND browser_nonce_sha256 = $3 AND browser_origin = $4 FOR UPDATE")
        .bind(ceremony).bind(account).bind(nonce_hash).bind(origin)
        .fetch_optional(tx.as_mut()).await?.ok_or(AccountOperationError::EnrollmentInvalid)?;
    let expires_at: OffsetDateTime = row.try_get("expires_at")?;
    let consumed_at: Option<OffsetDateTime> = row.try_get("consumed_at")?;
    let now = account_now_in_tx(tx).await?;
    if consumed_at.is_some() || expires_at <= now {
        return Err(AccountOperationError::EnrollmentInvalid);
    }
    sqlx::query("UPDATE public.auth_webauthn_ceremonies SET consumed_at = $2 WHERE id = $1")
        .bind(ceremony)
        .bind(now)
        .execute(tx.as_mut())
        .await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn activate_account_registration_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    account: Uuid,
    ceremony: Uuid,
    key: Uuid,
    family: Uuid,
    head: &AccountTermsHead,
    terms_kinds: &[String],
    content_sha256: &[Vec<u8>],
) -> Result<(), AccountOperationError> {
    let row = sqlx::query(
        "SELECT * FROM public.account_registration_activate_v1($1,$2,$3,$4,$5,$6,$7,$8,$9)",
    )
    .bind(account)
    .bind(ceremony)
    .bind(key)
    .bind(family)
    .bind(head.release_receipt_id)
    .bind(head.revision)
    .bind(&head.manifest_sha256)
    .bind(terms_kinds)
    .bind(content_sha256)
    .fetch_one(tx.as_mut())
    .await
    .map_err(|error| match error.as_database_error() {
        Some(db)
            if db.code().as_deref() == Some("P0001")
                && db.message() == "account.enrollment_invalid" =>
        {
            AccountOperationError::EnrollmentInvalid
        }
        Some(db)
            if db.code().as_deref() == Some("P0001") && db.message() == "account.terms_changed" =>
        {
            AccountOperationError::TermsChanged
        }
        _ => AccountOperationError::AuthorityUnavailable,
    })?;
    let generation: i64 = row.try_get("security_generation")?;
    let revision: i64 = row.try_get("revision")?;
    if generation != 1 || revision != 2 {
        return Err(AccountOperationError::AuthorityUnavailable);
    }
    Ok(())
}

/// Current-session read projection only; this boolean is never command authority.
/// The caller retains the admitted transaction and its Account/family/head guards.
pub async fn account_company_setup_eligible_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    session: &AccountLiveSession,
) -> Result<bool, AccountOperationError> {
    let eligible: bool =
        sqlx::query_scalar("SELECT public.account_company_setup_eligibility_v1($1)")
            .bind(session.account_id)
            .fetch_one(tx.as_mut())
            .await
            .map_err(|error| match error.as_database_error() {
                Some(db)
                    if db.code().as_deref() == Some("P0001")
                        && db.message() == "account.authentication_invalid" =>
                {
                    AccountOperationError::AuthenticationInvalid
                }
                _ => AccountOperationError::AuthorityUnavailable,
            })?;
    // Even an ineligible result may have waited for a designation-head lock.
    ensure_account_session_fresh_in_tx(tx, session).await?;
    Ok(eligible)
}

pub async fn account_contexts_empty_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    session: &AccountLiveSession,
) -> Result<(), AccountOperationError> {
    // The resolver retains Account/family SHARE until this transaction ends.
    // The helper proves profile, durable birth, generation and actual absence.
    let result: Result<(), AccountOperationError> = async {
        let row = sqlx::query("SELECT * FROM public.account_context_presence_v1($1)")
            .bind(session.account_id)
            .fetch_one(tx.as_mut())
            .await?;
        let generation: i64 = row.try_get("context_generation")?;
        let has_candidates: bool = row.try_get("has_candidates")?;
        if generation != 1 || has_candidates {
            return Err(AccountOperationError::NavigationUnavailable);
        }
        Ok(())
    }
    .await;
    result.map_err(|_| AccountOperationError::NavigationUnavailable)?;
    // Birth/source proof can wait or be expensive. A previously current access
    // token must still be current before the protected projection is released.
    ensure_account_session_fresh_in_tx(tx, session)
        .await
        .map(|_| ())
}

/// Native Account authority uses the database clock that stamps its durable
/// guards, ceremonies and families. Transaction-start `now()` would go stale
/// during waits; callers read this again after the locks relevant to their check.
pub async fn account_now_in_tx(
    tx: &mut Transaction<'_, Postgres>,
) -> Result<OffsetDateTime, AccountOperationError> {
    Ok(sqlx::query_scalar("SELECT pg_catalog.clock_timestamp()")
        .fetch_one(tx.as_mut())
        .await?)
}
