use console_kernel_core::UserId;
use sqlx::{Column, Row, postgres::PgRow};
use uuid::Uuid;

use super::{TenantRemovalAssembly, TenantRemovalError, TenantRemovalMode, TenantRemovalRequest};
use crate::legacy_topology::{LegacyTopologyCommandV1, LegacyTopologyIntent};

const COLUMNS: &[&str] = &[
    "state",
    "reason",
    "input_bytes",
    "plan_digest",
    "roles",
    "family_id",
    "family_user_id",
    "family_org_id",
    "family_protocol",
    "family_created_at",
    "family_revoked_at",
    "family_account_security_generation",
    "family_auth_time",
    "family_assurance",
];

pub(super) enum PreparationState {
    Ready,
    Absent,
    KnownConflict,
    UpgradeRequired,
}

pub(super) struct PreparedRemoval {
    pub state: PreparationState,
    pub input: Option<Vec<u8>>,
    pub plan: Option<Vec<u8>>,
}

impl PreparedRemoval {
    pub fn parse(
        row: &PgRow,
        request: &TenantRemovalRequest<'_>,
        actor: UserId,
        attempt: Uuid,
    ) -> Result<Self, TenantRemovalError> {
        let invalid = || TenantRemovalError::InvalidMaterial;
        if !row
            .columns()
            .iter()
            .map(Column::name)
            .eq(COLUMNS.iter().copied())
        {
            return Err(invalid());
        }
        let state: String = row.try_get("state").map_err(|_| invalid())?;
        let reason: Option<String> = row.try_get("reason").map_err(|_| invalid())?;
        let input: Option<Vec<u8>> = row.try_get("input_bytes").map_err(|_| invalid())?;
        let plan: Option<Vec<u8>> = row.try_get("plan_digest").map_err(|_| invalid())?;
        let state = match state.as_str() {
            "ready" if reason.is_none() => PreparationState::Ready,
            "absent" if reason.is_none() => PreparationState::Absent,
            "upgrade_required" if reason.is_none() => PreparationState::UpgradeRequired,
            "known_conflict"
                if matches!(
                    reason.as_deref(),
                    Some("topology_conflict" | "native_actors_present" | "plan_changed")
                ) =>
            {
                PreparationState::KnownConflict
            }
            _ => return Err(invalid()),
        };
        match (&state, request.assembly, input.as_deref(), plan.as_deref()) {
            (
                PreparationState::Ready,
                TenantRemovalAssembly::Company,
                Some(bytes),
                Some(digest),
            ) if digest.len() == 32 => match request.mode {
                TenantRemovalMode::Ordinary => {
                    let decoded = LegacyTopologyCommandV1::decode(bytes).map_err(|_| invalid())?;
                    if bytes.len() != 118
                        || decoded.actor() != actor
                        || decoded.command_id() != attempt
                        || decoded.expected_groups().len() != 1
                        || decoded.intent()
                            != &(LegacyTopologyIntent::RemoveEmptyCompany {
                                company: request.company,
                            })
                    {
                        return Err(invalid());
                    }
                }
                TenantRemovalMode::Force => validate_force(bytes, actor, attempt, request.company)?,
            },
            (PreparationState::Ready, TenantRemovalAssembly::Account, None, None) => (),
            (PreparationState::Ready, _, _, _) => return Err(invalid()),
            (_, _, None, None) => (),
            _ => return Err(invalid()),
        }
        Ok(Self { state, input, plan })
    }
}

fn validate_force(
    bytes: &[u8],
    actor: UserId,
    attempt: Uuid,
    company: Uuid,
) -> Result<(), TenantRemovalError> {
    let invalid = || TenantRemovalError::InvalidMaterial;
    if actor.as_uuid().is_nil()
        || attempt.is_nil()
        || company.is_nil()
        || bytes.len() != 120
        || &bytes[..32] != b"console.platform.force_remove\0\0\x01"
    {
        return Err(invalid());
    }
    let id = |offset| Uuid::from_slice(&bytes[offset..offset + 16]).map_err(|_| invalid());
    let revision = i64::from_be_bytes(bytes[112..120].try_into().map_err(|_| invalid())?);
    if id(32)? != *actor.as_uuid()
        || id(48)? != attempt
        || id(64)? != company
        || id(80)?.is_nil()
        || id(96)?.is_nil()
        || revision <= 0
    {
        return Err(invalid());
    }
    Ok(())
}

#[cfg(test)]
mod tests;
