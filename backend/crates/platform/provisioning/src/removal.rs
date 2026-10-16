//! The compatible DELETE transport owns one current-authority transaction.
mod material;

use console_kernel_core::{OrgId, TraceContext};
use console_platform_auth::{
    JwtVerifier, LegacyPlatformSource, LegacyPlatformSourceError, VerifiedLegacyPlatformCredential,
    ensure_legacy_platform_source_fresh_in_tx, legacy_platform_source_from_material,
};
use console_platform_authz::{PlatformFeature, platform_policy::*};
use sqlx::{PgPool, Postgres, Transaction};
use time::Duration;
use uuid::Uuid;

use crate::{PlatformProvisioner, TenantRemovalOutcome, direct_platform_policy_input};
use material::{PreparationState, PreparedRemoval};

/// Constructed only from the exact serving profile verified by the root.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TenantRemovalAssembly {
    Account,
    Company,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TenantRemovalMode {
    Ordinary,
    Force,
}

pub struct TenantRemovalRequest<'a> {
    pub verifier: &'a JwtVerifier,
    pub access: &'a str,
    pub absolute_family_ttl: Duration,
    pub policy: &'a PlatformPolicy,
    pub assembly: TenantRemovalAssembly,
    pub mode: TenantRemovalMode,
    pub company: Uuid,
}

#[derive(Debug, thiserror::Error)]
pub enum TenantRemovalError {
    #[error("platform authentication refused")]
    Unauthorized,
    #[error("platform policy refused")]
    Forbidden,
    #[error("platform removal is unavailable")]
    Unavailable,
    #[error("organization removal conflicts with current state")]
    Conflict,
    #[error("invalid removal owner material")]
    InvalidMaterial,
    #[error("removal storage error: {0}")]
    Storage(#[from] sqlx::Error),
}

impl TenantRemovalError {
    fn source(error: LegacyPlatformSourceError) -> Self {
        match error {
            LegacyPlatformSourceError::Unauthorized => Self::Unauthorized,
            LegacyPlatformSourceError::Unavailable => Self::Unavailable,
        }
    }

    // These tags are emitted only by the new ports' narrow source/role stages.
    fn port(error: sqlx::Error) -> Self {
        match error
            .as_database_error()
            .map(|db| (db.code(), db.message()))
        {
            Some((code, "platform_removal.source_refused")) if code.as_deref() == Some("28000") => {
                Self::Unauthorized
            }
            Some((code, "platform_removal.policy_refused")) if code.as_deref() == Some("42501") => {
                Self::Forbidden
            }
            _ => Self::Storage(error),
        }
    }
}

impl PlatformProvisioner {
    /// No automatic retries: the attempt and observed baseline stay immutable.
    pub async fn remove_tenant_current(
        &self,
        pool: &PgPool,
        request: TenantRemovalRequest<'_>,
    ) -> Result<TenantRemovalOutcome, TenantRemovalError> {
        let credential = VerifiedLegacyPlatformCredential::verify(
            request.verifier,
            request.access,
            request.absolute_family_ttl,
        )
        .map_err(TenantRemovalError::source)?;
        // Invalid resource selectors disclose no resource or Company-policy result.
        if request.company.is_nil() || request.company == *OrgId::platform().as_uuid() {
            return Ok(TenantRemovalOutcome::NotFound);
        }
        let actor = credential.subject();
        let family = credential.family_id();
        let attempt = Uuid::new_v4();
        let trace = TraceContext::generate();
        let mut tx = pool.begin().await?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL READ COMMITTED")
            .execute(tx.as_mut())
            .await?;
        let prepare = match request.mode {
            TenantRemovalMode::Ordinary => {
                "SELECT * FROM public.platform_prepare_organization_removal_v1($1,$2,$3,$4) LIMIT 2"
            }
            TenantRemovalMode::Force => {
                "SELECT * FROM public.platform_prepare_force_organization_removal_v1($1,$2,$3,$4) LIMIT 2"
            }
        };
        let rows = sqlx::query(prepare)
            .bind(actor.as_uuid())
            .bind(family)
            .bind(attempt)
            .bind(request.company)
            .fetch_all(tx.as_mut())
            .await
            .map_err(TenantRemovalError::port)?;
        let [row] = rows.as_slice() else {
            return Err(TenantRemovalError::InvalidMaterial);
        };
        let prepared = PreparedRemoval::parse(row, &request, actor, attempt)?;
        // The row is the exact Auth-owner material captured after the target prefix.
        let source = legacy_platform_source_from_material(credential, row)
            .map_err(|_| TenantRemovalError::InvalidMaterial)?;
        check_authority(&mut tx, &source, &request).await?;
        match prepared.state {
            PreparationState::Absent => {
                tx.rollback().await?;
                return Ok(TenantRemovalOutcome::NotFound);
            }
            PreparationState::KnownConflict => {
                tx.rollback().await?;
                return Err(TenantRemovalError::Conflict);
            }
            PreparationState::UpgradeRequired => return Err(TenantRemovalError::InvalidMaterial),
            PreparationState::Ready => (),
        }
        let execute = match request.mode {
            TenantRemovalMode::Ordinary => {
                "SELECT public.platform_remove_organization_command_v1($1,$2,$3,$4,$5,$6,$7,$8)"
            }
            TenantRemovalMode::Force => {
                "SELECT public.platform_force_remove_organization_command($1,$2,$3,$4,$5,$6,$7,$8)"
            }
        };
        let code: String = sqlx::query_scalar(execute)
            .bind(request.company)
            .bind(actor.as_uuid())
            .bind(family)
            .bind(attempt)
            .bind(prepared.input.as_deref())
            .bind(prepared.plan.as_deref())
            .bind(trace.trace_id())
            .bind(trace.span_id())
            .fetch_one(tx.as_mut())
            .await
            .map_err(TenantRemovalError::port)?;
        let outcome = match (request.mode, code.as_str()) {
            (_, "removed") => Ok(TenantRemovalOutcome::Removed),
            (_, "not_found") => Ok(TenantRemovalOutcome::NotFound),
            (_, "blocked_has_data") => Ok(TenantRemovalOutcome::BlockedHasData),
            (TenantRemovalMode::Force, "blocked_active") => Ok(TenantRemovalOutcome::BlockedActive),
            (_, "known_conflict") => Err(TenantRemovalError::Conflict),
            _ => return Err(TenantRemovalError::InvalidMaterial),
        };
        // Drain closure work before the last current-policy/DB-clock decision.
        sqlx::query("SET CONSTRAINTS ALL IMMEDIATE")
            .execute(tx.as_mut())
            .await?;
        check_authority(&mut tx, &source, &request).await?;
        // Lost acknowledgement is unknown; never retry or claim known rollback.
        tx.commit()
            .await
            .map_err(|_| TenantRemovalError::Unavailable)?;
        outcome
    }
}

async fn check_authority(
    tx: &mut Transaction<'_, Postgres>,
    source: &LegacyPlatformSource,
    request: &TenantRemovalRequest<'_>,
) -> Result<(), TenantRemovalError> {
    let now = ensure_legacy_platform_source_fresh_in_tx(tx, source)
        .await
        .map_err(TenantRemovalError::source)?;
    let input = direct_platform_policy_input(
        source,
        request.absolute_family_ttl,
        now,
        PlatformFeature::TenantRemove,
        PlatformTarget::Company(OrgId::from_uuid(request.company)),
        PlatformUse::Effect,
    );
    match request.policy.evaluate_current(&input) {
        PlatformPolicyDecision::Allow { .. } => Ok(()),
        PlatformPolicyDecision::Deny {
            reason: PlatformPolicyDenyReason::InvalidMaterial,
        } => Err(TenantRemovalError::Unauthorized),
        PlatformPolicyDecision::Deny {
            reason: PlatformPolicyDenyReason::PolicyDenied,
        } => Err(TenantRemovalError::Forbidden),
        PlatformPolicyDecision::Deny {
            reason: PlatformPolicyDenyReason::EvaluatorUnavailable,
        }
        | PlatformPolicyDecision::UpgradeRequired => Err(TenantRemovalError::Unavailable),
    }
}
