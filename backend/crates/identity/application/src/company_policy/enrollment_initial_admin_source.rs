//! Canonical enrollment source fields; decoding establishes no current authority.
use super::business::{invalid, read_revision, read_uuid, take};
use super::qualification_wire::{read_account, read_digest};
use super::{AccountId, ActionRef, INITIAL_MANIFEST};
use console_kernel_core::{KernelError, OrgId};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnrollmentInitialAdminSourceV1 {
    pub source_tag: u8,
    pub origin_account_id: AccountId,
    pub origin_command_id: Uuid,
    pub origin_receipt_id: Uuid,
    pub administrator_b: AccountId,
    pub group_id: Uuid,
    pub group_revision: u64,
    pub group_incarnation: Uuid,
    pub membership_id: Uuid,
    pub membership_revision: u64,
    pub membership_incarnation: Uuid,
    pub context_generation: u64,
    pub root_assignment_id: Uuid,
    pub root_assignment_revision: u64,
    pub root_role_id: Uuid,
    pub root_role_revision: u64,
    pub assign_identity: ActionRef,
    pub read_policy: ActionRef,
}

pub(super) fn decode(
    company: OrgId,
    mut input: &[u8],
) -> Result<EnrollmentInitialAdminSourceV1, KernelError> {
    if input.len() != 345
        || company.as_uuid().is_nil()
        || company == OrgId::platform()
        || take(&mut input, 1)? != [1]
    {
        return Err(invalid());
    }
    let value = EnrollmentInitialAdminSourceV1 {
        source_tag: 1,
        origin_account_id: read_account(&mut input)?,
        origin_command_id: read_uuid(&mut input)?,
        origin_receipt_id: read_uuid(&mut input)?,
        administrator_b: read_account(&mut input)?,
        group_id: read_uuid(&mut input)?,
        group_revision: read_revision(&mut input)?,
        group_incarnation: read_uuid(&mut input)?,
        membership_id: read_uuid(&mut input)?,
        membership_revision: read_revision(&mut input)?,
        membership_incarnation: read_uuid(&mut input)?,
        context_generation: read_revision(&mut input)?,
        root_assignment_id: read_uuid(&mut input)?,
        root_assignment_revision: read_revision(&mut input)?,
        root_role_id: read_uuid(&mut input)?,
        root_role_revision: read_revision(&mut input)?,
        assign_identity: read_action(company, &mut input)?,
        read_policy: read_action(company, &mut input)?,
    };
    if !input.is_empty() || value.root_assignment_revision != 1 || value.root_role_revision != 1 {
        return Err(invalid());
    }
    Ok(value)
}

fn read_action(company: OrgId, input: &mut &[u8]) -> Result<ActionRef, KernelError> {
    let action = read_uuid(input)?;
    let object = read_uuid(input)?;
    let revision = read_revision(input)?;
    let manifest = read_digest(input)?;
    const HEX: &[u8; 16] = b"0123456789abcdef";
    if revision != 1
        || INITIAL_MANIFEST.len() != 64
        || !manifest
            .iter()
            .zip(INITIAL_MANIFEST.as_bytes().chunks_exact(2))
            .all(|(byte, pair)| pair == [HEX[usize::from(byte >> 4)], HEX[usize::from(byte & 15)]])
    {
        return Err(invalid());
    }
    ActionRef::new(company, action, object, revision, manifest).map_err(|_| invalid())
}
