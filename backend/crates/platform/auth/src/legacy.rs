use console_kernel_core::{AuditEvent, KernelError, OrgId};
use sqlx::{Postgres, Row, Transaction};
use uuid::Uuid;

use crate::{AccessClaims, AuthError, JwtVerifier, LegacySessionKind};
use time::{Duration, OffsetDateTime};

#[derive(Debug, PartialEq, Eq)]
pub enum LegacySelfPasskeyReadError {
    Unauthorized,
    Unavailable,
}

#[derive(sqlx::FromRow)]
pub struct LegacySelfPasskeySummary {
    pub id: Uuid,
    pub created_at: OffsetDateTime,
    pub last_used_at: Option<OffsetDateTime>,
}

#[derive(sqlx::FromRow)]
struct LegacySelfReadFamily {
    id: Uuid,
    user_id: Uuid,
    org_id: Option<Uuid>,
    protocol: String,
    created_at: OffsetDateTime,
    revoked_at: Option<OffsetDateTime>,
    account_security_generation: Option<i64>,
    auth_time: Option<OffsetDateTime>,
    assurance: Option<String>,
}

fn legacy_self_read_guard_error(error: AuthError) -> LegacySelfPasskeyReadError {
    use LegacySelfPasskeyReadError::{Unauthorized, Unavailable};
    match error {
        AuthError::Sqlx(error) => match error.as_database_error() {
            Some(db)
                if db.code().as_deref() == Some("P0002")
                    && matches!(
                        db.message(),
                        "auth_legacy.company_not_found"
                            | "account_company_deactivation.subject_not_found"
                            | "auth_legacy.subject_not_found"
                    ) =>
            {
                Unauthorized
            }
            _ => Unavailable,
        },
        AuthError::InvalidStoredData(message)
            if message == "legacy credential operation is unavailable" =>
        {
            Unauthorized
        }
        AuthError::Kernel(error)
            if error.kind == console_kernel_core::ErrorKind::Conflict
                && error.message == "비활성화된 사용자는 인증 정보를 변경할 수 없습니다." =>
        {
            Unauthorized
        }
        _ => Unavailable,
    }
}

fn validate_legacy_self_read_family(
    claims: &AccessClaims,
    family: &LegacySelfReadFamily,
    absolute_family_ttl: Duration,
    now: OffsetDateTime,
) -> Result<(), LegacySelfPasskeyReadError> {
    use LegacySelfPasskeyReadError::{Unauthorized, Unavailable};
    if absolute_family_ttl <= Duration::ZERO {
        return Err(Unavailable);
    }
    let deadline = family
        .created_at
        .checked_add(absolute_family_ttl)
        .ok_or(Unavailable)?;
    let binding = claims.legacy_session.as_ref().ok_or(Unauthorized)?;
    let subject = Uuid::parse_str(&claims.sub).map_err(|_| Unauthorized)?;
    let org = Uuid::parse_str(&claims.org).map_err(|_| Unauthorized)?;
    let iat = OffsetDateTime::from_unix_timestamp(claims.iat).map_err(|_| Unauthorized)?;
    let nbf = OffsetDateTime::from_unix_timestamp(claims.nbf).map_err(|_| Unauthorized)?;
    let exp = OffsetDateTime::from_unix_timestamp(claims.exp).map_err(|_| Unauthorized)?;
    if family.id != binding.family_id
        || family.user_id != subject
        || family.org_id != Some(org)
        || binding.home_org != org
        || family.protocol != "LEGACY_COMPANY"
        || family.revoked_at.is_some()
        || family.account_security_generation.is_some()
        || family.auth_time.is_some()
        || family.assurance.is_some()
        || family.created_at > now
        || family.created_at.unix_timestamp() > claims.iat
        || deadline <= now
        || iat > nbf
        || nbf >= exp
        || iat > now
        || nbf > now
        || exp <= now
    {
        return Err(Unauthorized);
    }
    Ok(())
}

/// Read the verified self's credentials while retaining current identity and
/// exact-family locks through projection. Historical unbound reads keep their
/// original expiry; no read creates or extends session authority.
pub async fn read_legacy_self_passkeys(
    verifier: &JwtVerifier,
    access_token: &str,
    absolute_family_ttl: Duration,
    auth_pool: &sqlx::PgPool,
) -> Result<Vec<LegacySelfPasskeySummary>, LegacySelfPasskeyReadError> {
    use LegacySelfPasskeyReadError::{Unauthorized, Unavailable};
    let claims = verifier
        .verify_access_token(access_token)
        .map_err(|_| Unauthorized)?;
    let subject = Uuid::parse_str(&claims.sub).map_err(|_| Unauthorized)?;
    let org = Uuid::parse_str(&claims.org).map_err(|_| Unauthorized)?;
    if subject.is_nil()
        || org.is_nil()
        || claims.platform != (org == *OrgId::platform().as_uuid())
        || claims.view_as
        || claims.read_only
        || claims.tenant_context.is_some()
    {
        return Err(Unauthorized);
    }
    if let Some(binding) = &claims.legacy_session {
        if binding.kind != LegacySessionKind::Direct || claims.platform {
            return Err(Unauthorized);
        }
        if absolute_family_ttl <= Duration::ZERO {
            return Err(Unavailable);
        }
    }
    let mut tx = auth_pool.begin().await.map_err(|_| Unavailable)?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL READ COMMITTED")
        .execute(tx.as_mut())
        .await
        .map_err(|_| Unavailable)?;
    guard_legacy_subject_in_tx(&mut tx, OrgId::from_uuid(org), subject)
        .await
        .map_err(legacy_self_read_guard_error)?;
    let family = if let Some(binding) = &claims.legacy_session {
        Some(sqlx::query_as::<_, LegacySelfReadFamily>(
            "SELECT id,user_id,org_id,protocol,created_at,revoked_at,account_security_generation,auth_time,assurance FROM public.auth_refresh_token_families WHERE id=$1 FOR SHARE",
        ).bind(binding.family_id).fetch_optional(tx.as_mut()).await.map_err(|_| Unavailable)?.ok_or(Unauthorized)?)
    } else {
        None
    };
    let check_time = |now| {
        if let Some(family) = &family {
            validate_legacy_self_read_family(&claims, family, absolute_family_ttl, now)
        } else {
            let exp = OffsetDateTime::from_unix_timestamp(claims.exp).map_err(|_| Unauthorized)?;
            if exp <= now {
                Err(Unauthorized)
            } else {
                Ok(())
            }
        }
    };
    check_time(
        crate::account::account_now_in_tx(&mut tx)
            .await
            .map_err(|_| Unavailable)?,
    )?;
    let summaries = sqlx::query_as::<_, LegacySelfPasskeySummary>(
        "SELECT id,created_at,last_used_at FROM public.auth_legacy_self_passkeys_v1($1,$2)",
    )
    .bind(org)
    .bind(subject)
    .fetch_all(tx.as_mut())
    .await
    .map_err(|_| Unavailable)?;
    check_time(
        crate::account::account_now_in_tx(&mut tx)
            .await
            .map_err(|_| Unavailable)?,
    )?;
    tx.commit().await.map_err(|_| Unavailable)?;
    Ok(summaries)
}

/// Serialize legacy credential changes with Company purge and Account cutover.
/// The caller derives the subject and Company from verified authority, then
/// rereads its credential binding after this guard. It owns the transaction.
pub async fn guard_legacy_subject_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    org: OrgId,
    subject: Uuid,
) -> Result<(), AuthError> {
    sqlx::query("SELECT set_config('app.current_org', $1, true)")
        .bind(org.as_uuid().to_string())
        .execute(tx.as_mut())
        .await?;
    sqlx::query("SELECT public.auth_legacy_company_lock_v1($1)")
        .bind(*org.as_uuid())
        .execute(tx.as_mut())
        .await?;
    let legacy: bool =
        sqlx::query_scalar("SELECT public.account_company_deactivation_guard_v1($1, $2)")
            .bind(*org.as_uuid())
            .bind(subject)
            .fetch_one(tx.as_mut())
            .await?;
    if !legacy {
        return Err(AuthError::InvalidStoredData(
            "legacy credential operation is unavailable".to_owned(),
        ));
    }
    let active: bool = sqlx::query_scalar("SELECT public.auth_legacy_user_active_v1($1, $2)")
        .bind(*org.as_uuid())
        .bind(subject)
        .fetch_one(tx.as_mut())
        .await?;
    if !active {
        return Err(AuthError::Kernel(KernelError::conflict(
            "비활성화된 사용자는 인증 정보를 변경할 수 없습니다.",
        )));
    }
    Ok(())
}

/// Append an existing Auth event through the fixed Company audit projection.
/// SQL enforces the finite action/target/snapshot schemas and caller grants.
pub async fn append_legacy_auth_audit_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    event: &AuditEvent,
) -> Result<(), AuthError> {
    sqlx::query(
        "SELECT public.auth_legacy_audit_append_v1(\
         $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19)",
    )
    .bind(*event.id.as_uuid())
    .bind(event.actor.map(|actor| *actor.as_uuid()))
    .bind(event.action.as_str())
    .bind(&event.target_type)
    .bind(&event.target_id)
    .bind(event.branch_id.map(|branch| *branch.as_uuid()))
    .bind(&event.before)
    .bind(&event.after)
    .bind(event.trace.trace_id())
    .bind(event.trace.span_id())
    .bind(event.occurred_at)
    .bind(event.org_id.map(|org| *org.as_uuid()))
    .bind(event.request_context.ip.as_deref())
    .bind(event.request_context.user_agent.as_deref())
    .bind(event.request_context.auth_method.as_deref())
    .bind(event.request_context.device.as_deref())
    .bind(event.classification.badges.as_deref())
    .bind(event.classification.anomaly)
    .bind(event.classification.reason.as_deref())
    .execute(tx.as_mut())
    .await?;
    Ok(())
}

/// Current legacy identity projected by the credential owner, without Business reads.
pub struct LegacySessionContext {
    pub display_name: String,
    pub username: String,
    pub roles: Vec<String>,
    pub branches: Vec<Uuid>,
    pub group_roles: Vec<String>,
    pub authz_subject_version: u64,
    pub authz_policy_version: u64,
    pub session_generation: u64,
}

#[derive(Debug)]
pub enum LegacySessionContextError {
    NotFound,
    Inactive,
    Fenced,
    Unavailable,
}

impl From<sqlx::Error> for LegacySessionContextError {
    fn from(error: sqlx::Error) -> Self {
        match error.as_database_error() {
            Some(db) => match (db.code().as_deref(), db.message()) {
                (Some("P0002"), "auth_legacy.subject_not_found") => Self::NotFound,
                (
                    Some("28000"),
                    "auth_legacy.subject_inactive" | "auth_legacy.subject_has_no_roles",
                ) => Self::Inactive,
                (Some("P0001"), "auth_legacy.fenced") => Self::Fenced,
                _ => Self::Unavailable,
            },
            None => Self::Unavailable,
        }
    }
}

/// Caller owns the scoped transaction and must finish it before using the result.
pub async fn legacy_session_context_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    org: OrgId,
    subject: Uuid,
) -> Result<LegacySessionContext, LegacySessionContextError> {
    let row = sqlx::query("SELECT display_name, username, roles, branches, group_roles, authz_subject_version, authz_policy_version, session_generation FROM public.auth_legacy_session_context_v1($1,$2)")
        .bind(*org.as_uuid()).bind(subject).fetch_one(tx.as_mut()).await?;
    Ok(LegacySessionContext {
        display_name: row.try_get("display_name")?,
        username: row.try_get("username")?,
        roles: row.try_get("roles")?,
        branches: row.try_get("branches")?,
        group_roles: row.try_get("group_roles")?,
        authz_subject_version: u64::try_from(row.try_get::<i64, _>("authz_subject_version")?)
            .unwrap_or(0),
        authz_policy_version: u64::try_from(row.try_get::<i64, _>("authz_policy_version")?)
            .unwrap_or(0),
        session_generation: u64::try_from(row.try_get::<i64, _>("session_generation")?)
            .unwrap_or(0),
    })
}

#[cfg(test)]
#[path = "legacy_self_read_tests.rs"]
mod legacy_self_read_tests;
