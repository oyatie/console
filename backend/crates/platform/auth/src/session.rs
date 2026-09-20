mod company_enrollment;

pub use company_enrollment::{AccountEnrollmentCredentials, LockedAccountEnrollment};

use std::fmt;

use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    JwtVerifier, LegacySessionContext, LegacySessionContextError, legacy_session_context_in_tx,
};
use console_kernel_core::OrgId;

/// The token verifier and mandatory Auth transport for current session checks.
///
/// Construction only binds existing dependencies. The composition root admits
/// the Auth connection; a reader never reconnects or substitutes a Business pool.
#[derive(Clone)]
pub struct SessionVerification {
    verifier: JwtVerifier,
    auth_database: PgPool,
}

impl SessionVerification {
    #[must_use]
    pub fn new(verifier: JwtVerifier, auth_database: PgPool) -> Self {
        Self {
            verifier,
            auth_database,
        }
    }

    /// Cryptographic and claim validation only. This does not authenticate a
    /// current session or produce a principal; callers must also check the
    /// verified subject through [`Self::legacy_subject_is_fenced`].
    #[must_use]
    pub fn token_verifier(&self) -> &JwtVerifier {
        &self.verifier
    }

    /// The admitted Auth transport for credential owners; never a Business pool.
    #[must_use]
    pub fn auth_pool(&self) -> &PgPool {
        &self.auth_database
    }

    /// Observe current authority through the admitted Auth connection. This is
    /// a request snapshot, not a lock held through later Business effects.
    pub async fn legacy_session_context(
        &self,
        org: OrgId,
        subject: Uuid,
    ) -> Result<LegacySessionContext, LegacySessionContextError> {
        let mut tx = self.auth_database.begin().await?;
        sqlx::query("SELECT set_config('app.current_org', $1, true)")
            .bind(org.as_uuid().to_string())
            .execute(tx.as_mut())
            .await?;
        match legacy_session_context_in_tx(&mut tx, org, subject).await {
            Ok(context) => {
                tx.commit().await?;
                Ok(context)
            }
            Err(error) => {
                // A function exception aborts this transaction. Finish it before
                // a caller can attempt a separate platform-home lookup.
                tx.rollback().await?;
                Err(error)
            }
        }
    }

    /// Read the strict Account presence projection through the bound Auth pool.
    /// Only a successful `false` permits a legacy subject. Missing/NULL results
    /// and unavailable transport remain errors, never an unfenced result.
    pub async fn legacy_subject_is_fenced(&self, subject: Uuid) -> Result<bool, sqlx::Error> {
        legacy_subject_is_fenced(&self.auth_database, subject).await
    }
}

impl fmt::Debug for SessionVerification {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SessionVerification")
            .finish_non_exhaustive()
    }
}

pub(crate) async fn legacy_subject_is_fenced(
    auth_pool: &PgPool,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT public.account_legacy_fenced_v1($1)")
        .bind(user_id)
        // rls-arming: ok narrow Account identity projection is global, not Company data
        .fetch_one(auth_pool)
        .await
}

use std::collections::BTreeMap;

use sqlx::{Postgres, Row, Transaction};
use time::{Duration, OffsetDateTime};

use crate::account::{
    AccountLiveSession, AccountOperationError, AccountSecurityState, account_now_in_tx,
    account_security_from_row, lock_account_in_tx,
};
use crate::{AccountAccessClaims, AccountAccessVerification, AccountAssurance};

/// A current signature is only the first check. These row guards remain held
/// through the caller's projection and commit; revocation cannot interleave.
pub async fn live_account_session_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    verifier: &JwtVerifier,
    access: &str,
    absolute_ttl: Duration,
) -> Result<AccountLiveSession, AccountOperationError> {
    resolve_account_session_in_tx(
        tx,
        verifier,
        Some(access),
        None,
        absolute_ttl,
        SessionPurpose::Read,
    )
    .await
    .map(|resolved| resolved.session)
}

/// Read-only proof acquisition, including refresh-only recovery. Never consume,
/// rotate, extend or issue an access token here.
pub async fn account_csrf_session_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    verifier: &JwtVerifier,
    access: Option<&str>,
    refresh: Option<&str>,
    absolute_ttl: Duration,
) -> Result<AccountLiveSession, AccountOperationError> {
    resolve_account_session_in_tx(
        tx,
        verifier,
        access,
        refresh,
        absolute_ttl,
        SessionPurpose::Read,
    )
    .await
    .map(|resolved| resolved.session)
}

pub async fn logout_account_session_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    verifier: &JwtVerifier,
    access: Option<&str>,
    refresh: Option<&str>,
    proof: &str,
    absolute_ttl: Duration,
) -> Result<(), AccountOperationError> {
    // Parse a genuine proof before credential row reads, then recheck its lifetime after
    // all lock waits. Exclusive Account precedes family and token locks.
    verifier
        .verify_account_csrf_token(proof, account_now_in_tx(tx).await?)
        .map_err(|_| AccountOperationError::CsrfInvalid)?;
    let session = resolve_account_session_in_tx(
        tx,
        verifier,
        access,
        refresh,
        absolute_ttl,
        SessionPurpose::Logout,
    )
    .await?
    .session;
    let now = account_now_in_tx(tx).await?;
    let claims = verifier
        .verify_account_csrf_token(proof, now)
        .map_err(|_| AccountOperationError::CsrfInvalid)?;
    if claims.sub != session.account_id
        || claims.sid != session.session_id
        || claims.security_generation != session.security_generation
        || claims.exp > session.family_expires_at.unix_timestamp()
        || session.expires_at <= now
        || session.family_expires_at <= now
    {
        return Err(AccountOperationError::CsrfInvalid);
    }
    let row = sqlx::query("SELECT * FROM public.account_session_logout_v1($1,$2,$3,$4)")
        .bind(session.account_id)
        .bind(session.session_id)
        .bind(session.security_generation)
        .bind(absolute_ttl)
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
    let event_id: Uuid = row.try_get("event_id")?;
    let revoked_at: OffsetDateTime = row.try_get("revoked_at")?;
    if event_id.is_nil() || revoked_at < session.auth_time {
        return Err(AccountOperationError::AuthorityUnavailable);
    }
    // The event read inside the owner can wait after the resolver's guards.
    // Validate at its actual mutation timestamp before the caller may commit;
    // any expired authority rolls back both the revocation and its audit event.
    verifier
        .verify_account_csrf_token(proof, revoked_at)
        .map_err(|_| AccountOperationError::CsrfInvalid)?;
    if session.expires_at <= revoked_at || session.family_expires_at <= revoked_at {
        return Err(AccountOperationError::CsrfInvalid);
    }
    Ok(())
}

struct NativeFamily {
    account_id: Uuid,
    family_id: Uuid,
    generation: i64,
    created_at: OffsetDateTime,
    auth_time: OffsetDateTime,
    expires_at: OffsetDateTime,
}

struct NativeRefresh {
    account_id: Uuid,
    family_id: Uuid,
    token_id: Uuid,
    token_hash: Vec<u8>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SessionPurpose {
    Read,
    Logout,
    Refresh,
}

struct ResolvedAccountSession {
    session: AccountLiveSession,
    refresh_token_id: Option<Uuid>,
    refresh_used: bool,
}

pub(crate) struct LockedAccountRefresh {
    pub session: AccountLiveSession,
    pub token_id: Uuid,
    pub used: bool,
}

/// Only the refresh writer may correlate consumed tokens. Proof is checked
/// before row reads and again after waits; ordinary session/proof readers keep
/// rejecting every consumed token. This never issues a session from a proof.
pub(crate) async fn account_refresh_session_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    verifier: &JwtVerifier,
    access: Option<&str>,
    refresh: &str,
    proof: &str,
    absolute_ttl: Duration,
) -> Result<LockedAccountRefresh, AccountOperationError> {
    verifier
        .verify_account_csrf_token(proof, account_now_in_tx(tx).await?)
        .map_err(|_| AccountOperationError::CsrfInvalid)?;
    let resolved = resolve_account_session_in_tx(
        tx,
        verifier,
        access,
        Some(refresh),
        absolute_ttl,
        SessionPurpose::Refresh,
    )
    .await?;
    validate_account_refresh_proof_in_tx(tx, verifier, proof, &resolved.session).await?;
    Ok(LockedAccountRefresh {
        session: resolved.session,
        token_id: resolved
            .refresh_token_id
            .ok_or(AccountOperationError::AuthenticationInvalid)?,
        used: resolved.refresh_used,
    })
}

pub(crate) async fn validate_account_refresh_proof_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    verifier: &JwtVerifier,
    proof: &str,
    session: &AccountLiveSession,
) -> Result<OffsetDateTime, AccountOperationError> {
    let now = account_now_in_tx(tx).await?;
    let claims = verifier
        .verify_account_csrf_token(proof, now)
        .map_err(|_| AccountOperationError::CsrfInvalid)?;
    if claims.sub != session.account_id
        || claims.sid != session.session_id
        || claims.security_generation != session.security_generation
        || claims.exp > session.family_expires_at.unix_timestamp()
    {
        return Err(AccountOperationError::CsrfInvalid);
    }
    if session.expires_at <= now || session.family_expires_at <= now || session.auth_time > now {
        return Err(AccountOperationError::AuthenticationInvalid);
    }
    Ok(now)
}

async fn resolve_account_session_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    verifier: &JwtVerifier,
    access: Option<&str>,
    refresh: Option<&str>,
    absolute_ttl: Duration,
    purpose: SessionPurpose,
) -> Result<ResolvedAccountSession, AccountOperationError> {
    let exclusive = purpose != SessionPurpose::Read;
    let invalid = || AccountOperationError::AuthenticationInvalid;
    if absolute_ttl <= Duration::ZERO {
        return Err(AccountOperationError::AuthorityUnavailable);
    }
    // Malformed/signature/kind/time errors never permit a refresh downgrade.
    let verification_time = account_now_in_tx(tx).await?;
    let access_claims: Option<AccountAccessClaims> = access
        .map(|token| {
            verifier
                .verify_account_access_token(token, verification_time)
                .map(|verified| match verified {
                    AccountAccessVerification::Current(claims)
                    | AccountAccessVerification::Expired(claims) => claims,
                })
                .map_err(|_| invalid())
        })
        .transpose()?;
    let refresh_key = if let Some(token) = refresh {
        if token.is_empty() || token.len() > 512 {
            return Err(invalid());
        }
        let token_hash = crate::refresh::hash_token(token);
        // Correlation is not authority. Recheck this exact tuple under locks.
        let row = sqlx::query("SELECT id, family_id, user_id FROM public.auth_refresh_tokens WHERE token_hash = $1 AND org_id IS NULL")
            .bind(&token_hash).fetch_optional(tx.as_mut()).await?.ok_or_else(invalid)?;
        Some(NativeRefresh {
            account_id: row.try_get("user_id")?,
            family_id: row.try_get("family_id")?,
            token_id: row.try_get("id")?,
            token_hash,
        })
    } else {
        None
    };
    if access_claims.is_none() && refresh_key.is_none() {
        return Err(invalid());
    }

    // A mixed-cookie request may touch two Accounts/families. Acquire each class
    // in UUID order before descending to the next class, including logout.
    let mut wanted = BTreeMap::new();
    if let Some(claims) = &access_claims {
        wanted.insert(claims.sid, claims.sub);
    }
    if let Some(token) = &refresh_key {
        if wanted
            .get(&token.family_id)
            .is_some_and(|id| *id != token.account_id)
        {
            return Err(invalid());
        }
        wanted.insert(token.family_id, token.account_id);
    }
    let mut families = BTreeMap::new();
    if purpose == SessionPurpose::Read && refresh_key.is_none() {
        // One signed Account/family pair. Both owners retain their guards on
        // this transaction; Business receives no raw identity/credential rights.
        let claims = access_claims.as_ref().ok_or_else(invalid)?;
        let row = sqlx::query("SELECT * FROM public.account_session_shared_material_v1($1,$2)")
            .bind(claims.sub)
            .bind(claims.sid)
            .fetch_optional(tx.as_mut())
            .await
            .map_err(|error| match error.as_database_error() {
                Some(db)
                    if db.code().as_deref() == Some("P0001")
                        && db.message() == "account.authentication_invalid" =>
                {
                    invalid()
                }
                _ => AccountOperationError::AuthorityUnavailable,
            })?
            .ok_or_else(invalid)?;
        let guard = account_security_from_row(&row, claims.sub, "ACTIVE")?;
        families.insert(
            claims.sid,
            family_from_row(&row, &guard, claims.sid, absolute_ttl)?,
        );
    } else {
        // Mixed credentials retain all Accounts -> all families -> tokens.
        // Applying the paired projection here would invert that lock order.
        let mut account_ids: Vec<_> = wanted.values().copied().collect();
        account_ids.sort_unstable();
        account_ids.dedup();
        let mut guards = BTreeMap::new();
        for account_id in account_ids {
            let guard = lock_account_in_tx(tx, account_id, exclusive, "ACTIVE").await?;
            guards.insert(account_id, guard);
        }
        for (family_id, account_id) in wanted {
            let query = if exclusive {
                "SELECT user_id, protocol, account_security_generation, auth_time, assurance, created_at, revoked_at, org_id FROM public.auth_refresh_token_families WHERE id = $1 FOR UPDATE"
            } else {
                "SELECT user_id, protocol, account_security_generation, auth_time, assurance, created_at, revoked_at, org_id FROM public.auth_refresh_token_families WHERE id = $1 FOR SHARE"
            };
            let row = sqlx::query(query)
                .bind(family_id)
                .fetch_optional(tx.as_mut())
                .await?
                .ok_or_else(invalid)?;
            let guard = guards.get(&account_id).ok_or_else(invalid)?;
            families.insert(
                family_id,
                family_from_row(&row, guard, family_id, absolute_ttl)?,
            );
        }
    }
    let mut refresh_used = false;
    let refresh_expiry = if let Some(token) = &refresh_key {
        let query = if exclusive {
            "SELECT issued_at, expires_at, used_at, revoked_at FROM public.auth_refresh_tokens WHERE id = $1 AND family_id = $2 AND user_id = $3 AND token_hash = $4 AND org_id IS NULL FOR UPDATE"
        } else {
            "SELECT issued_at, expires_at, used_at, revoked_at FROM public.auth_refresh_tokens WHERE id = $1 AND family_id = $2 AND user_id = $3 AND token_hash = $4 AND org_id IS NULL FOR SHARE"
        };
        let row = sqlx::query(query)
            .bind(token.token_id)
            .bind(token.family_id)
            .bind(token.account_id)
            .bind(&token.token_hash)
            .fetch_optional(tx.as_mut())
            .await?
            .ok_or_else(invalid)?;
        let issued_at: OffsetDateTime = row.try_get("issued_at")?;
        let expires_at: OffsetDateTime = row.try_get("expires_at")?;
        let used_at: Option<OffsetDateTime> = row.try_get("used_at")?;
        let revoked_at: Option<OffsetDateTime> = row.try_get("revoked_at")?;
        let family = families.get(&token.family_id).ok_or_else(invalid)?;
        let now = account_now_in_tx(tx).await?;
        refresh_used = used_at.is_some();
        // AS1 preserves consumed-token reuse detection ahead of token expiry.
        // A consumed secret is only replay evidence under current bound proof
        // and a live native family; it never authorizes successful rotation.
        let replay = purpose == SessionPurpose::Refresh && refresh_used;
        if (refresh_used && !replay)
            || used_at.is_some_and(|used| used < issued_at || used > now)
            || revoked_at.is_some()
            || (!replay && expires_at <= now)
            || issued_at > now
            || issued_at < family.created_at
            || expires_at <= issued_at
            || expires_at > family.expires_at
        {
            return Err(invalid());
        }
        Some(if replay {
            family.expires_at
        } else {
            expires_at
        })
    } else {
        None
    };

    let now = account_now_in_tx(tx).await?;
    for family in families.values() {
        if family.created_at > now || family.auth_time > now || family.expires_at <= now {
            return Err(invalid());
        }
    }
    if let Some(claims) = &access_claims {
        let family = families.get(&claims.sid).ok_or_else(invalid)?;
        if claims.sub != family.account_id
            || claims.security_generation != family.generation
            || claims.auth_time != family.auth_time.unix_timestamp()
            || claims.assurance != AccountAssurance::PasskeyPrimary
            || claims.iat > now.unix_timestamp()
            || claims.nbf > now.unix_timestamp()
            || claims.iat < family.created_at.unix_timestamp()
            || claims.exp > family.expires_at.unix_timestamp()
        {
            return Err(invalid());
        }
        if let Some(token) = &refresh_key
            && (claims.sub != token.account_id || claims.sid != token.family_id)
        {
            return Err(AccountOperationError::AmbiguousCredentials);
        }
        if purpose != SessionPurpose::Refresh && claims.exp > now.unix_timestamp() {
            return Ok(ResolvedAccountSession {
                session: live_projection(
                    family,
                    OffsetDateTime::from_unix_timestamp(claims.exp).map_err(|_| invalid())?,
                ),
                refresh_token_id: refresh_key.as_ref().map(|token| token.token_id),
                refresh_used,
            });
        }
    }
    let token = refresh_key.as_ref().ok_or_else(invalid)?;
    let family = families.get(&token.family_id).ok_or_else(invalid)?;
    let expires_at = refresh_expiry.ok_or_else(invalid)?;
    if expires_at <= account_now_in_tx(tx).await? {
        return Err(invalid());
    }
    Ok(ResolvedAccountSession {
        session: live_projection(family, expires_at),
        refresh_token_id: Some(token.token_id),
        refresh_used,
    })
}

fn family_from_row(
    row: &sqlx::postgres::PgRow,
    guard: &AccountSecurityState,
    family_id: Uuid,
    absolute_ttl: Duration,
) -> Result<NativeFamily, AccountOperationError> {
    let invalid = || AccountOperationError::AuthenticationInvalid;
    let owner: Uuid = row.try_get("user_id")?;
    let protocol: String = row.try_get("protocol")?;
    let generation: Option<i64> = row.try_get("account_security_generation")?;
    let auth_time: Option<OffsetDateTime> = row.try_get("auth_time")?;
    let assurance: Option<String> = row.try_get("assurance")?;
    let created_at: OffsetDateTime = row.try_get("created_at")?;
    let revoked: Option<OffsetDateTime> = row.try_get("revoked_at")?;
    let org: Option<Uuid> = row.try_get("org_id")?;
    let auth_time = auth_time.ok_or_else(invalid)?;
    if owner != guard.account_id
        || protocol != "ACCOUNT_V1"
        || org.is_some()
        || generation != Some(guard.security_generation)
        || revoked.is_some()
        || assurance.as_deref() != Some("PASSKEY_PRIMARY")
        || auth_time > created_at
    {
        return Err(invalid());
    }
    let expires_at = created_at
        .checked_add(absolute_ttl)
        .ok_or(AccountOperationError::AuthorityUnavailable)?;
    Ok(NativeFamily {
        account_id: guard.account_id,
        family_id,
        generation: guard.security_generation,
        created_at,
        auth_time,
        expires_at,
    })
}

fn live_projection(family: &NativeFamily, expires_at: OffsetDateTime) -> AccountLiveSession {
    AccountLiveSession {
        account_id: family.account_id,
        session_id: family.family_id,
        security_generation: family.generation,
        auth_time: family.auth_time,
        assurance: AccountAssurance::PasskeyPrimary,
        expires_at,
        family_expires_at: family.expires_at,
    }
}

/// Final freshness check for a projection whose live resolver still holds the
/// same Account/family locks. This never substitutes for that resolver.
pub async fn ensure_account_session_fresh_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    session: &AccountLiveSession,
) -> Result<OffsetDateTime, AccountOperationError> {
    let now = account_now_in_tx(tx).await?;
    if session.expires_at <= now || session.family_expires_at <= now || session.auth_time > now {
        return Err(AccountOperationError::AuthenticationInvalid);
    }
    Ok(now)
}
