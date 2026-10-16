//! Legacy device-login handoffs owned by the Auth transport. The caller keeps
//! one transaction through proof, handoff mutation, session issuance and audit.

use std::fmt;

use console_kernel_core::OrgId;
use sha2::{Digest, Sha256};
use sqlx::{Postgres, Row, Transaction};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::{
    AuthError, PasskeyAuthenticationCredential, PasskeyService, guard_legacy_subject_in_tx,
};

#[derive(thiserror::Error)]
pub enum DeviceLoginHandoffError {
    #[error("invalid or expired login handoff")]
    Invalid,
    #[error("desktop login approval requires an enrolled passkey")]
    PasskeyRequired,
    #[error("login handoff authority unavailable")]
    Auth(#[from] AuthError),
}

impl fmt::Debug for DeviceLoginHandoffError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

impl From<sqlx::Error> for DeviceLoginHandoffError {
    fn from(error: sqlx::Error) -> Self {
        Self::Auth(AuthError::Sqlx(error))
    }
}

/// Data for the outer use case's existing audit and session issuance. No owner
/// operation accepts this output as proof of an authentication or approval.
#[derive(Debug)]
pub struct DeviceLoginApproval {
    pub handoff_id: Uuid,
    pub user_id: Uuid,
    pub org_id: OrgId,
    pub passkey_id: Option<Uuid>,
}

#[derive(Debug)]
pub enum DeviceLoginPoll {
    Pending,
    Expired,
    Consumed(DeviceLoginApproval),
}

fn token_hash(raw: &str, prefix: &str) -> Result<Vec<u8>, DeviceLoginHandoffError> {
    let token = raw.trim();
    let suffix = token
        .strip_prefix(prefix)
        .ok_or(DeviceLoginHandoffError::Invalid)?;
    if suffix.len() != 64 || !suffix.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(DeviceLoginHandoffError::Invalid);
    }
    Ok(Sha256::digest(token.as_bytes()).to_vec())
}

fn validate_expiry(
    now: OffsetDateTime,
    expires_at: OffsetDateTime,
) -> Result<(), DeviceLoginHandoffError> {
    if expires_at <= now || expires_at > now + Duration::minutes(5) {
        return Err(DeviceLoginHandoffError::Invalid);
    }
    Ok(())
}

/// Anonymous start retains distinct poll/approve tokens and the existing TTL.
/// The outer use case appends its fixed anonymous start audit before commit.
pub async fn create_device_login_handoff_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    poll_token: &str,
    approve_token: &str,
    now: OffsetDateTime,
    expires_at: OffsetDateTime,
) -> Result<Uuid, DeviceLoginHandoffError> {
    validate_expiry(now, expires_at)?;
    let poll_hash = token_hash(poll_token, "console_dlp_")?;
    let approve_hash = token_hash(approve_token, "console_dla_")?;
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO public.auth_device_login_handoffs (id,poll_token_hash,approve_token_hash,issued_at,expires_at) VALUES ($1,$2,$3,$4,$5)",
    )
    .bind(id).bind(poll_hash).bind(approve_hash).bind(now).bind(expires_at)
    .execute(tx.as_mut()).await?;
    Ok(id)
}

/// Create the desktop-return handoff for the same authenticated user whose
/// bootstrap enrollment code was issued in this transaction. Never anonymous.
#[allow(clippy::too_many_arguments)]
pub async fn create_self_enroll_device_handoff_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    org: OrgId,
    user: Uuid,
    poll_token: &str,
    approve_token: &str,
    now: OffsetDateTime,
    expires_at: OffsetDateTime,
) -> Result<Uuid, DeviceLoginHandoffError> {
    validate_expiry(now, expires_at)?;
    let poll_hash = token_hash(poll_token, "console_dlp_")?;
    let approve_hash = token_hash(approve_token, "console_dla_")?;
    guard_legacy_subject_in_tx(tx, org, user).await?;
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO public.auth_device_login_handoffs (id,poll_token_hash,approve_token_hash,issued_at,expires_at,target_user_id,target_org_id) VALUES ($1,$2,$3,$4,$5,$6,$7)",
    )
    .bind(id).bind(poll_hash).bind(approve_hash).bind(now).bind(expires_at)
    .bind(user).bind(*org.as_uuid()).execute(tx.as_mut()).await?;
    Ok(id)
}

/// An approved handoff is consumed only after rechecking its guarded legacy
/// subject and any retained key binding. A concurrent consume returns Invalid.
/// Family/token issuance and the existing consume audit follow on this same tx.
pub async fn poll_and_consume_device_login_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    poll_token: &str,
    now: OffsetDateTime,
) -> Result<DeviceLoginPoll, DeviceLoginHandoffError> {
    let poll_hash = token_hash(poll_token, "console_dlp_")?;
    let status = sqlx::query(
        "SELECT id,expires_at,approved_at,consumed_at,approved_user_id,approved_org_id,approved_passkey_id FROM public.auth_device_login_handoffs WHERE poll_token_hash=$1",
    )
    .bind(&poll_hash).fetch_optional(tx.as_mut()).await?
    .ok_or(DeviceLoginHandoffError::Invalid)?;
    let consumed_at: Option<OffsetDateTime> = status.try_get("consumed_at")?;
    if consumed_at.is_some() {
        return Err(DeviceLoginHandoffError::Invalid);
    }
    let expires_at: OffsetDateTime = status.try_get("expires_at")?;
    if expires_at <= now {
        return Ok(DeviceLoginPoll::Expired);
    }
    let approved_at: Option<OffsetDateTime> = status.try_get("approved_at")?;
    if approved_at.is_none() {
        return Ok(DeviceLoginPoll::Pending);
    }
    let handoff_id: Uuid = status.try_get("id")?;
    let user_id = status
        .try_get::<Option<Uuid>, _>("approved_user_id")?
        .ok_or(DeviceLoginHandoffError::Invalid)?;
    let org_uuid = status
        .try_get::<Option<Uuid>, _>("approved_org_id")?
        .ok_or(DeviceLoginHandoffError::Invalid)?;
    let passkey_id: Option<Uuid> = status.try_get("approved_passkey_id")?;
    let org_id = OrgId::from_uuid(org_uuid);
    guard_legacy_subject_in_tx(tx, org_id, user_id).await?;
    if let Some(key) = passkey_id {
        let bound_key: Option<Uuid> = sqlx::query_scalar(
            "SELECT id FROM public.auth_webauthn_credentials WHERE id=$1 AND user_id=$2 AND org_id=$3 FOR UPDATE",
        )
        .bind(key).bind(user_id).bind(org_uuid).fetch_optional(tx.as_mut()).await?;
        if bound_key.is_none() {
            return Err(DeviceLoginHandoffError::Invalid);
        }
    }
    // Recheck every correlated identity field after the guard/key waits. No
    // handoff lock is taken before Company/users/Account or a credential key.
    let consumed: Option<Uuid> = sqlx::query_scalar(
        r#"UPDATE public.auth_device_login_handoffs SET consumed_at=$2
        WHERE poll_token_hash=$1 AND id=$3
          AND consumed_at IS NULL AND approved_at IS NOT NULL AND expires_at>$2
          AND approved_user_id=$4 AND approved_org_id=$5
          AND approved_passkey_id IS NOT DISTINCT FROM $6
          AND (target_user_id IS NULL OR target_user_id=$4)
          AND (target_org_id IS NULL OR target_org_id=$5)
        RETURNING id"#,
    )
    .bind(&poll_hash)
    .bind(now)
    .bind(handoff_id)
    .bind(user_id)
    .bind(org_uuid)
    .bind(passkey_id)
    .fetch_optional(tx.as_mut())
    .await?;
    let handoff_id = consumed.ok_or(DeviceLoginHandoffError::Invalid)?;
    Ok(DeviceLoginPoll::Consumed(DeviceLoginApproval {
        handoff_id,
        user_id,
        org_id,
        passkey_id,
    }))
}

/// Primary proof is verified here, never supplied as an approval boolean or a
/// caller-created AuthenticationOutcome. The same tx retains its credential
/// and subject locks through the conditional target update and outer audit.
pub async fn approve_device_login_with_passkey_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    passkeys: &PasskeyService,
    approve_token: &str,
    ceremony_id: Uuid,
    credential: PasskeyAuthenticationCredential,
    now: OffsetDateTime,
) -> Result<DeviceLoginApproval, DeviceLoginHandoffError> {
    let approve_hash = token_hash(approve_token, "console_dla_")?;
    let pending: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM public.auth_device_login_handoffs WHERE approve_token_hash=$1 AND approved_at IS NULL AND consumed_at IS NULL AND expires_at>$2",
    )
    .bind(&approve_hash).bind(now).fetch_optional(tx.as_mut()).await?;
    let handoff_id = pending.ok_or(DeviceLoginHandoffError::Invalid)?;
    let outcome = passkeys
        .finish_authentication_in_tx(tx, ceremony_id, credential)
        .await?;
    let updated: Option<Uuid> = sqlx::query_scalar(
        r#"UPDATE public.auth_device_login_handoffs
        SET approved_at=$2,approved_user_id=$3,approved_org_id=$4,approved_passkey_id=$5
        WHERE approve_token_hash=$1 AND id=$6
          AND approved_at IS NULL AND consumed_at IS NULL AND expires_at>$2
          AND (target_user_id IS NULL OR target_user_id=$3)
          AND (target_org_id IS NULL OR target_org_id=$4)
        RETURNING id"#,
    )
    .bind(&approve_hash)
    .bind(now)
    .bind(outcome.user_id)
    .bind(*outcome.org_id.as_uuid())
    .bind(outcome.passkey_id)
    .bind(handoff_id)
    .fetch_optional(tx.as_mut())
    .await?;
    let handoff_id = updated.ok_or(DeviceLoginHandoffError::Invalid)?;
    Ok(DeviceLoginApproval {
        handoff_id,
        user_id: outcome.user_id,
        org_id: outcome.org_id,
        passkey_id: Some(outcome.passkey_id),
    })
}

/// A verified existing session can approve only its explicitly targeted self
/// handoff and must still own an enrolled passkey under the same subject guard.
pub async fn approve_targeted_device_login_session_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    org: OrgId,
    user: Uuid,
    approve_token: &str,
    now: OffsetDateTime,
) -> Result<DeviceLoginApproval, DeviceLoginHandoffError> {
    let approve_hash = token_hash(approve_token, "console_dla_")?;
    guard_legacy_subject_in_tx(tx, org, user).await?;
    let latest_key: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM public.auth_webauthn_credentials WHERE user_id=$1 AND org_id=$2 ORDER BY created_at DESC LIMIT 1 FOR UPDATE",
    )
    .bind(user).bind(*org.as_uuid()).fetch_optional(tx.as_mut()).await?;
    let key = latest_key.ok_or(DeviceLoginHandoffError::PasskeyRequired)?;
    let approved: Option<Uuid> = sqlx::query_scalar(
        r#"UPDATE public.auth_device_login_handoffs
        SET approved_at=$2,approved_user_id=$3,approved_org_id=$4,approved_passkey_id=$5
        WHERE approve_token_hash=$1
          AND approved_at IS NULL AND consumed_at IS NULL AND expires_at>$2
          AND target_user_id=$3 AND target_org_id=$4
        RETURNING id"#,
    )
    .bind(&approve_hash)
    .bind(now)
    .bind(user)
    .bind(*org.as_uuid())
    .bind(key)
    .fetch_optional(tx.as_mut())
    .await?;
    let handoff_id = approved.ok_or(DeviceLoginHandoffError::Invalid)?;
    Ok(DeviceLoginApproval {
        handoff_id,
        user_id: user,
        org_id: org,
        passkey_id: Some(key),
    })
}
