use console_kernel_core::{AuditAction, AuditEvent, OrgId, TraceContext, UserId};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::{AuthError, append_legacy_auth_audit_in_tx, guard_legacy_subject_in_tx};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefreshToken(String);

impl RefreshToken {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct RefreshTokenIssue {
    pub token: RefreshToken,
    pub family_id: Uuid,
    pub token_id: Uuid,
    pub user_id: Uuid,
    pub org_id: OrgId,
    pub expires_at: OffsetDateTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum RefreshTokenUseError {
    #[error("invalid refresh token")]
    InvalidToken,
    #[error("refresh token expired")]
    Expired,
    #[error("refresh token family has been revoked")]
    FamilyRevoked,
    #[error("refresh token reuse detected")]
    ReuseDetected,
    #[error("refresh token storage error")]
    Storage,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RefreshTokenStore;

impl RefreshTokenStore {
    pub async fn issue_family(
        &self,
        _pool: &PgPool,
        auth_pool: &PgPool,
        user_id: Uuid,
        org_id: OrgId,
        now: OffsetDateTime,
        ttl: Duration,
    ) -> Result<RefreshTokenIssue, AuthError> {
        let mut tx = auth_pool.begin().await?;
        let issue = self
            .issue_family_in_tx(&mut tx, user_id, org_id, now, ttl)
            .await?;
        tx.commit().await?;
        Ok(issue)
    }

    /// Issue credentials and their audit inside the caller's Auth transaction.
    pub async fn issue_family_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        user_id: Uuid,
        org_id: OrgId,
        now: OffsetDateTime,
        ttl: Duration,
    ) -> Result<RefreshTokenIssue, AuthError> {
        guard_refresh_subject(tx, org_id, user_id).await?;
        let family_id = Uuid::new_v4();
        let token_id = Uuid::new_v4();
        let token = generate_refresh_token();
        let token_hash = hash_token(token.as_str());
        let expires_at = now + ttl;
        let org_uuid = *org_id.as_uuid();

        // `with_org` arms `app.current_org` for the transaction, so the RLS
        // WITH CHECK on `auth_refresh_token_families`/`auth_refresh_tokens`
        // accepts these tenant rows when the app writes as the non-owner role.
        let audit = AuditEvent::new(
            Some(UserId::from_uuid(user_id)),
            AuditAction::new("auth.refresh.issue")?,
            "auth_refresh_token_family",
            family_id.to_string(),
            TraceContext::generate(),
            now,
        )
        .with_org(org_id)
        .with_snapshots(
            None,
            Some(serde_json::json!({
                "family_id": family_id,
                "token_id": token_id,
                "user_id": user_id,
                "expires_at": expires_at,
            })),
        );

        sqlx::query(
            r#"
            INSERT INTO auth_refresh_token_families (id, user_id, created_at, org_id)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(family_id)
        .bind(user_id)
        .bind(now)
        .bind(org_uuid)
        .execute(tx.as_mut())
        .await?;

        sqlx::query(
            r#"
            INSERT INTO auth_refresh_tokens (
                id, family_id, user_id, token_hash, issued_at, expires_at, org_id
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(token_id)
        .bind(family_id)
        .bind(user_id)
        .bind(token_hash)
        .bind(now)
        .bind(expires_at)
        .bind(org_uuid)
        .execute(tx.as_mut())
        .await?;

        append_legacy_auth_audit_in_tx(tx, &audit).await?;

        Ok(RefreshTokenIssue {
            token: RefreshToken(token),
            family_id,
            token_id,
            user_id,
            org_id,
            expires_at,
        })
    }

    pub async fn rotate(
        &self,
        pool: &PgPool,
        auth_pool: &PgPool,
        presented_token: &str,
        now: OffsetDateTime,
        ttl: Duration,
        absolute_ttl: Duration,
    ) -> Result<RefreshTokenIssue, RefreshTokenUseError> {
        self.rotate_inner(pool, auth_pool, presented_token, now, ttl, absolute_ttl)
            .await
            .map_err(|err| match err {
                AuthError::Refresh(refresh) => refresh,
                _ => RefreshTokenUseError::Storage,
            })
    }

    async fn rotate_inner(
        &self,
        _pool: &PgPool,
        auth_pool: &PgPool,
        presented_token: &str,
        now: OffsetDateTime,
        ttl: Duration,
        absolute_ttl: Duration,
    ) -> Result<RefreshTokenIssue, AuthError> {
        let mut tx = auth_pool.begin().await?;
        let outcome = self
            .rotate_in_tx(&mut tx, presented_token, now, ttl, absolute_ttl)
            .await?;
        tx.commit().await?;
        outcome.map_err(AuthError::from)
    }

    /// An inner refusal can include revocation effects and MUST be committed.
    /// An outer storage/authority error MUST roll back the transaction.
    pub async fn rotate_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        presented_token: &str,
        now: OffsetDateTime,
        ttl: Duration,
        absolute_ttl: Duration,
    ) -> Result<Result<RefreshTokenIssue, RefreshTokenUseError>, AuthError> {
        let token_hash = hash_token(presented_token);

        // Resolve the family's tenant from the token hash FIRST, then arm the
        // GUC, THEN do the RLS-gated read. `auth_refresh_tokens` is FORCE RLS
        // (migration 0035), so as the non-owner `console_rt` role a lookup-by-hash
        // returns ZERO rows until `app.current_org` is set — but the org is what
        // we need to set it. The narrow SECURITY DEFINER resolver
        // `platform_resolve_token_org` returns only the family's org_id, breaking
        // that chicken-and-egg so refresh works for ANY tenant (not just KNL).
        let Some(org_uuid) = resolve_token_org(tx, &token_hash).await? else {
            return Ok(Err(RefreshTokenUseError::InvalidToken));
        };
        sqlx::query("SELECT set_config('app.current_org', $1, true)")
            .bind(org_uuid.to_string())
            .execute(tx.as_mut())
            .await?;

        let Some(subject) = correlate_token_subject(tx, &token_hash).await? else {
            return Ok(Err(RefreshTokenUseError::InvalidToken));
        };
        guard_refresh_subject(tx, OrgId::from_uuid(org_uuid), subject.1).await?;
        lock_refresh_family(tx, subject.0, subject.1).await?;

        let row = sqlx::query(
            r#"
            SELECT
                t.id AS token_id,
                t.family_id,
                t.user_id,
                t.expires_at,
                t.used_at,
                t.revoked_at AS token_revoked_at,
                f.revoked_at AS family_revoked_at,
                f.created_at AS family_created_at
            FROM auth_refresh_tokens t
            JOIN auth_refresh_token_families f ON f.id = t.family_id
            WHERE t.token_hash = $1 AND t.family_id = $2 AND t.user_id = $3
              AND f.user_id = t.user_id AND t.org_id = $4 AND f.org_id = $4
            FOR UPDATE OF t
            "#,
        )
        .bind(token_hash)
        .bind(subject.0)
        .bind(subject.1)
        .bind(org_uuid)
        .fetch_optional(tx.as_mut())
        .await?;

        let Some(row) = row else {
            return Ok(Err(RefreshTokenUseError::InvalidToken));
        };

        let token_id: Uuid = row.try_get("token_id")?;
        let family_id: Uuid = row.try_get("family_id")?;
        let user_id: Uuid = row.try_get("user_id")?;
        let expires_at: OffsetDateTime = row.try_get("expires_at")?;
        let used_at: Option<OffsetDateTime> = row.try_get("used_at")?;
        let token_revoked_at: Option<OffsetDateTime> = row.try_get("token_revoked_at")?;
        let family_revoked_at: Option<OffsetDateTime> = row.try_get("family_revoked_at")?;
        let family_created_at: OffsetDateTime = row.try_get("family_created_at")?;

        if family_revoked_at.is_some() {
            return Ok(Err(RefreshTokenUseError::FamilyRevoked));
        }

        // Absolute session-lifetime cap (NIST 800-63B AAL2 reauthentication):
        // a refresh family may rotate freely within `absolute_ttl` of its
        // creation, but past that hard ceiling every rotation is rejected and the
        // family is revoked — forcing a fresh primary authentication. This bounds
        // the lifetime of a silently-rotating session even if individual tokens
        // never expire from inactivity. The revoke commits so a subsequent
        // presentation of any sibling token also fails closed.
        if now > family_created_at + absolute_ttl {
            sqlx::query(
                r#"
                UPDATE auth_refresh_token_families
                SET revoked_at = $1, revoked_reason = 'absolute_ttl_exceeded'
                WHERE id = $2 AND revoked_at IS NULL
                "#,
            )
            .bind(now)
            .bind(family_id)
            .execute(tx.as_mut())
            .await?;
            sqlx::query(
                "UPDATE auth_refresh_tokens SET revoked_at = COALESCE(revoked_at, $1) WHERE family_id = $2",
            )
            .bind(now)
            .bind(family_id)
            .execute(tx.as_mut())
            .await?;
            insert_audit_in_tx(
                tx,
                OrgId::from_uuid(org_uuid),
                user_id,
                family_id,
                "auth.refresh.absolute_ttl_revoked",
                now,
                serde_json::json!({
                    "family_id": family_id,
                    "revoked_reason": "absolute_ttl_exceeded",
                    "family_created_at": family_created_at,
                }),
            )
            .await?;
            return Ok(Err(RefreshTokenUseError::FamilyRevoked));
        }

        if used_at.is_some() || token_revoked_at.is_some() {
            revoke_family_for_reuse(tx, family_id, token_id, now).await?;
            insert_audit_in_tx(
                tx,
                OrgId::from_uuid(org_uuid),
                user_id,
                family_id,
                "auth.refresh.reuse_detected",
                now,
                serde_json::json!({
                    "family_id": family_id,
                    "revoked_reason": "reuse_detected",
                    "reused_token_id": token_id,
                }),
            )
            .await?;
            return Ok(Err(RefreshTokenUseError::ReuseDetected));
        }

        if expires_at <= now {
            sqlx::query("UPDATE auth_refresh_tokens SET revoked_at = $1 WHERE id = $2")
                .bind(now)
                .bind(token_id)
                .execute(tx.as_mut())
                .await?;
            return Ok(Err(RefreshTokenUseError::Expired));
        }

        let replacement_id = Uuid::new_v4();
        let replacement = generate_refresh_token();
        let replacement_hash = hash_token(replacement.as_str());
        let replacement_expires_at = now + ttl;

        sqlx::query(
            r#"
            INSERT INTO auth_refresh_tokens (
                id, family_id, user_id, token_hash, issued_at, expires_at, org_id
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(replacement_id)
        .bind(family_id)
        .bind(user_id)
        .bind(replacement_hash)
        .bind(now)
        .bind(replacement_expires_at)
        .bind(org_uuid)
        .execute(tx.as_mut())
        .await?;

        sqlx::query(
            r#"
            UPDATE auth_refresh_tokens
            SET used_at = $1, replaced_by = $2
            WHERE id = $3
            "#,
        )
        .bind(now)
        .bind(replacement_id)
        .bind(token_id)
        .execute(tx.as_mut())
        .await?;

        insert_audit_in_tx(
            tx,
            OrgId::from_uuid(org_uuid),
            user_id,
            family_id,
            "auth.refresh",
            now,
            serde_json::json!({
                "family_id": family_id,
                "used_token_id": token_id,
                "replacement_token_id": replacement_id,
                "expires_at": replacement_expires_at,
            }),
        )
        .await?;

        Ok(Ok(RefreshTokenIssue {
            token: RefreshToken(replacement),
            family_id,
            token_id: replacement_id,
            user_id,
            org_id: OrgId::from_uuid(org_uuid),
            expires_at: replacement_expires_at,
        }))
    }

    pub async fn revoke_family_for_logout(
        &self,
        pool: &PgPool,
        presented_token: &str,
        now: OffsetDateTime,
    ) -> Result<(), RefreshTokenUseError> {
        self.revoke_family_for_logout_inner(pool, presented_token, now)
            .await
            .map_err(|err| match err {
                AuthError::Refresh(refresh) => refresh,
                _ => RefreshTokenUseError::Storage,
            })
    }

    async fn revoke_family_for_logout_inner(
        &self,
        pool: &PgPool,
        presented_token: &str,
        now: OffsetDateTime,
    ) -> Result<(), AuthError> {
        let mut tx = pool.begin().await?;
        self.revoke_family_for_logout_in_tx(&mut tx, presented_token, now)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn revoke_family_for_logout_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        presented_token: &str,
        now: OffsetDateTime,
    ) -> Result<(), AuthError> {
        let token_hash = hash_token(presented_token);

        // Resolve the family's tenant from the token hash and arm the GUC BEFORE
        // the RLS-gated read, so logout works under `console_rt` for any tenant.
        // See `rotate_inner` for the chicken-and-egg rationale.
        let Some(org_uuid) = resolve_token_org(tx, &token_hash).await? else {
            return Err(RefreshTokenUseError::InvalidToken.into());
        };
        sqlx::query("SELECT set_config('app.current_org', $1, true)")
            .bind(org_uuid.to_string())
            .execute(tx.as_mut())
            .await?;

        let Some(subject) = correlate_token_subject(tx, &token_hash).await? else {
            return Err(RefreshTokenUseError::InvalidToken.into());
        };
        guard_refresh_subject(tx, OrgId::from_uuid(org_uuid), subject.1).await?;
        lock_refresh_family(tx, subject.0, subject.1).await?;

        let row = sqlx::query(
            r#"
            SELECT
                t.family_id,
                t.user_id,
                f.revoked_at AS family_revoked_at
            FROM auth_refresh_tokens t
            JOIN auth_refresh_token_families f ON f.id = t.family_id
            WHERE t.token_hash = $1 AND t.family_id = $2 AND t.user_id = $3
              AND f.user_id = t.user_id AND t.org_id = $4 AND f.org_id = $4
            FOR UPDATE OF t
            "#,
        )
        .bind(token_hash)
        .bind(subject.0)
        .bind(subject.1)
        .bind(org_uuid)
        .fetch_optional(tx.as_mut())
        .await?;

        let Some(row) = row else {
            return Err(RefreshTokenUseError::InvalidToken.into());
        };

        let family_id: Uuid = row.try_get("family_id")?;
        let user_id: Uuid = row.try_get("user_id")?;
        let family_revoked_at: Option<OffsetDateTime> = row.try_get("family_revoked_at")?;

        if family_revoked_at.is_some() {
            return Err(RefreshTokenUseError::FamilyRevoked.into());
        }

        sqlx::query(
            r#"
            UPDATE auth_refresh_token_families
            SET revoked_at = $1, revoked_reason = 'logout'
            WHERE id = $2 AND revoked_at IS NULL
            "#,
        )
        .bind(now)
        .bind(family_id)
        .execute(tx.as_mut())
        .await?;

        sqlx::query(
            r#"
            UPDATE auth_refresh_tokens
            SET revoked_at = COALESCE(revoked_at, $1)
            WHERE family_id = $2
            "#,
        )
        .bind(now)
        .bind(family_id)
        .execute(tx.as_mut())
        .await?;

        insert_audit_in_tx(
            tx,
            OrgId::from_uuid(org_uuid),
            user_id,
            family_id,
            "auth.logout",
            now,
            serde_json::json!({
                "family_id": family_id,
                "revoked_reason": "logout",
            }),
        )
        .await?;

        Ok(())
    }
}

/// Resolve a refresh-token family's tenant from a token hash, via the narrow
/// SECURITY DEFINER resolver `platform_resolve_token_org` (migration 0036).
///
/// `auth_refresh_tokens` is FORCE RLS, so the app's non-owner `console_rt` role
/// cannot read a token row by hash until `app.current_org` is armed — but the
/// org is exactly what we need to arm it. This resolver returns ONLY the org_id
/// (nothing else), breaking that chicken-and-egg without widening any read
/// surface. Returns `None` for an unknown hash.
async fn resolve_token_org(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    token_hash: &[u8],
) -> Result<Option<Uuid>, AuthError> {
    Ok(sqlx::query_scalar("SELECT platform_resolve_token_org($1)")
        .bind(token_hash)
        .fetch_one(tx.as_mut())
        .await?)
}

async fn correlate_token_subject(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    hash: &[u8],
) -> Result<Option<(Uuid, Uuid)>, AuthError> {
    Ok(sqlx::query_as(
        "SELECT family_id, user_id FROM public.auth_refresh_tokens WHERE token_hash = $1",
    )
    .bind(hash)
    .fetch_optional(tx.as_mut())
    .await?)
}

async fn guard_refresh_subject(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    org: OrgId,
    subject: Uuid,
) -> Result<(), AuthError> {
    guard_legacy_subject_in_tx(tx, org, subject)
        .await
        .map_err(|err| match err {
            AuthError::InvalidStoredData(_) | AuthError::Kernel(_) => {
                RefreshTokenUseError::InvalidToken.into()
            }
            other => other,
        })
}

async fn lock_refresh_family(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    family: Uuid,
    subject: Uuid,
) -> Result<(), AuthError> {
    sqlx::query("SELECT id FROM public.auth_refresh_token_families WHERE id = $1 AND user_id = $2 FOR UPDATE")
        .bind(family).bind(subject).fetch_optional(tx.as_mut()).await?;
    Ok(())
}

async fn revoke_family_for_reuse(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    family_id: Uuid,
    reused_token_id: Uuid,
    now: OffsetDateTime,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE auth_refresh_token_families
        SET revoked_at = $1, revoked_reason = 'reuse_detected'
        WHERE id = $2 AND revoked_at IS NULL
        "#,
    )
    .bind(now)
    .bind(family_id)
    .execute(tx.as_mut())
    .await?;

    sqlx::query(
        r#"
        UPDATE auth_refresh_tokens
        SET revoked_at = COALESCE(revoked_at, $1),
            reuse_detected_at = CASE WHEN id = $2 THEN $1 ELSE reuse_detected_at END
        WHERE family_id = $3
        "#,
    )
    .bind(now)
    .bind(reused_token_id)
    .bind(family_id)
    .execute(tx.as_mut())
    .await?;

    Ok(())
}

async fn insert_audit_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    org: OrgId,
    user_id: Uuid,
    family_id: Uuid,
    action: &str,
    now: OffsetDateTime,
    after: serde_json::Value,
) -> Result<(), AuthError> {
    let event = AuditEvent::new(
        Some(UserId::from_uuid(user_id)),
        AuditAction::new(action)?,
        "auth_refresh_token_family",
        family_id.to_string(),
        TraceContext::generate(),
        now,
    )
    .with_org(org)
    .with_snapshots(None, Some(after));

    append_legacy_auth_audit_in_tx(tx, &event).await
}

fn generate_refresh_token() -> String {
    let mut bytes = [0u8; 32];
    bytes[..16].copy_from_slice(Uuid::new_v4().as_bytes());
    bytes[16..].copy_from_slice(Uuid::new_v4().as_bytes());
    format!("console_rt_{}", hex_encode(&bytes))
}

fn hash_token(token: &str) -> Vec<u8> {
    Sha256::digest(token.as_bytes()).to_vec()
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}
