use console_kernel_core::{AuditEvent, KernelError, OrgId};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::AuthError;

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
