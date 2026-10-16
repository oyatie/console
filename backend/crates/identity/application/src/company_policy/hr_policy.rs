//! Frozen HR parsed claims; current policy, sources and qualification remain owner checks.
use super::AccountId;
use super::business::{invalid, read_revision, read_uuid, take, validate_expiry};
use super::enrollment_initial_admin_source::{self, EnrollmentInitialAdminSourceV1};
use super::qualification_wire::{decode_snapshot, read_account, read_digest, read_instant};
use console_identity_domain::qualification::QualifiedPersonSnapshotV1;
use console_kernel_core::{KernelError, OrgId};
use time::OffsetDateTime;
use uuid::Uuid;

const PROTOCOL: &str = "COMPANY_HR_APPOINTMENT_CAPABILITY_V1";
const PREFIX: &[u8] = b"console.company.hr-appointment-capability\0\0\x01";
const MANIFEST: [u8; 32] = [
    0xb5, 0xc2, 0xe0, 0x36, 0x6a, 0x16, 0x22, 0x22, 0x48, 0x94, 0x77, 0xd3, 0xb2, 0x31, 0x46, 0x3c,
    0xe8, 0xdc, 0x76, 0x86, 0xca, 0xd8, 0x2f, 0x39, 0x21, 0x9f, 0xe0, 0xbf, 0x22, 0x3f, 0x28, 0x03,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeHrPolicyCommandV1 {
    pub company: OrgId,
    pub command_id: Uuid,
    pub expected_company_epoch: u64,
    pub current_policy_receipt_id: Option<Uuid>,
    pub admin_source: EnrollmentInitialAdminSourceV1,
    pub expected_admin_assignment: Option<(Uuid, u64)>,
    pub expected_target_assignment: Option<(Uuid, u64)>,
    pub operation: u8,
    pub role_kind: Option<u8>,
    pub recipient_account_id: Option<AccountId>,
    pub expires_at: Option<OffsetDateTime>,
    pub recipient_source_digest: Option<[u8; 32]>,
    pub qualification_pair: Option<(QualifiedPersonSnapshotV1, QualifiedPersonSnapshotV1)>,
}

pub fn decode_native_hr_policy_command_v1(
    command_protocol: &str,
    local_codec: i16,
    input: &[u8],
) -> Result<(AccountId, NativeHrPolicyCommandV1), KernelError> {
    if command_protocol != PROTOCOL || local_codec != 1 {
        return Err(hr_invalid());
    }
    if input.len() > 1825 {
        return Err(hr_invalid());
    }
    decode(input).map_err(|_| hr_invalid())
}

fn hr_invalid() -> KernelError {
    KernelError::validation("invalid native HR policy command")
}

fn decode(mut input: &[u8]) -> Result<(AccountId, NativeHrPolicyCommandV1), KernelError> {
    let length = input.len();
    if take(&mut input, PREFIX.len())? != PREFIX {
        return Err(invalid());
    }
    let operation = match take(&mut input, 1)? {
        [value @ 1..=3] => *value,
        _ => return Err(invalid()),
    };
    if take(&mut input, 32)? != MANIFEST {
        return Err(invalid());
    }
    let actor = read_account(&mut input)?;
    let company = OrgId::from_uuid(read_uuid(&mut input)?);
    if company == OrgId::platform() {
        return Err(invalid());
    }
    let command_id = read_uuid(&mut input)?;
    let epoch = read_revision(&mut input)?;
    let current_policy_receipt_id = match take(&mut input, 1)? {
        [0] => None,
        [1] => Some(read_uuid(&mut input)?),
        _ => return Err(invalid()),
    };
    if current_policy_receipt_id.is_some() != (epoch > 1) {
        return Err(invalid());
    }
    let admin_source = enrollment_initial_admin_source::decode(company, take(&mut input, 345)?)?;
    if admin_source.administrator_b != actor {
        return Err(invalid());
    }
    let expected_admin_assignment = read_assignment(&mut input)?;
    let expected_target_assignment = read_assignment(&mut input)?;
    let assignments_valid = match operation {
        1 => expected_admin_assignment.is_none() && expected_target_assignment.is_none(),
        2 => epoch > 1 && expected_admin_assignment.is_some(),
        3 => {
            epoch > 1 && expected_admin_assignment.is_some() && expected_target_assignment.is_some()
        }
        _ => false,
    };
    if !assignments_valid {
        return Err(invalid());
    }
    let mut command = NativeHrPolicyCommandV1 {
        company,
        command_id,
        expected_company_epoch: epoch,
        current_policy_receipt_id,
        admin_source,
        expected_admin_assignment,
        expected_target_assignment,
        operation,
        role_kind: None,
        recipient_account_id: None,
        expires_at: None,
        recipient_source_digest: None,
        qualification_pair: None,
    };
    if operation != 1 {
        let (role, recipient) = read_recipient(&mut input, actor)?;
        command.role_kind = Some(role);
        command.recipient_account_id = Some(recipient);
        if operation == 2 {
            let expiry = read_instant(&mut input)?;
            validate_expiry(expiry)?;
            command.expires_at = Some(expiry);
            command.recipient_source_digest = Some(read_digest(&mut input)?);
            command.qualification_pair = match take(&mut input, 1)? {
                [0] if recipient == actor => None,
                [1] if recipient != actor => {
                    Some(read_pair(&mut input, company, actor, recipient)?)
                }
                _ => return Err(invalid()),
            };
        }
    }
    let expected_length = match operation {
        1 => {
            481 + if current_policy_receipt_id.is_some() {
                16
            } else {
                0
            }
        }
        2 => {
            579 + if expected_target_assignment.is_some() {
                24
            } else {
                0
            } + command.qualification_pair.as_ref().map_or(0, |(b, c)| {
                1206 + if b.individual_valid_until.is_some() {
                    8
                } else {
                    0
                } + if c.individual_valid_until.is_some() {
                    8
                } else {
                    0
                }
            })
        }
        3 => 562,
        _ => return Err(invalid()),
    };
    if !input.is_empty() || length != expected_length {
        return Err(invalid());
    }
    Ok((actor, command))
}

fn read_assignment(input: &mut &[u8]) -> Result<Option<(Uuid, u64)>, KernelError> {
    match take(input, 1)? {
        [0] => Ok(None),
        [1] => Ok(Some((read_uuid(input)?, read_revision(input)?))),
        _ => Err(invalid()),
    }
}

fn read_recipient(
    input: &mut &[u8],
    administrator: AccountId,
) -> Result<(u8, AccountId), KernelError> {
    let role = match take(input, 1)? {
        [value @ 1..=3] => *value,
        _ => return Err(invalid()),
    };
    let recipient = read_account(input)?;
    if (role == 1 && recipient != administrator) || (role == 2 && recipient == administrator) {
        return Err(invalid());
    }
    Ok((role, recipient))
}

fn read_pair(
    input: &mut &[u8],
    company: OrgId,
    administrator: AccountId,
    recipient: AccountId,
) -> Result<(QualifiedPersonSnapshotV1, QualifiedPersonSnapshotV1), KernelError> {
    let b = read_snapshot(input)?;
    let c = read_snapshot(input)?;
    if b.company != company
        || c.company != company
        || b.account != administrator
        || c.account != recipient
        || b.account == c.account
        || b.human_anchor == c.human_anchor
    {
        return Err(invalid());
    }
    Ok((b, c))
}

fn read_snapshot(input: &mut &[u8]) -> Result<QualifiedPersonSnapshotV1, KernelError> {
    let length = u16::from_be_bytes(take(input, 2)?.try_into().map_err(|_| invalid())?);
    if !matches!(length, 601 | 609) {
        return Err(invalid());
    }
    decode_snapshot(take(input, usize::from(length))?)
}
