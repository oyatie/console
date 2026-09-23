//! Immutable Company business-policy command bytes. Selectors are not authority.
use super::AccountId;
use console_kernel_core::{KernelError, OrgId};
use time::{OffsetDateTime, UtcOffset};
use uuid::Uuid;

const PREFIX: &[u8] = b"console.company.business-policy\0\0\x01";
pub const MANIFEST: [u8; 32] = [
    0x07, 0x78, 0x15, 0x14, 0x02, 0x9d, 0x5f, 0x8f, 0x7e, 0x96, 0x22, 0x1d, 0x65, 0x04, 0x38, 0x73,
    0x24, 0xc8, 0xc0, 0xf2, 0x51, 0x3d, 0xed, 0x2b, 0x21, 0x4d, 0x0b, 0xd6, 0x83, 0xce, 0x3d, 0xdd,
];
const MIN_EXPIRY_MICROS: i64 = -62_135_596_800_000_000;
const MAX_EXPIRY_MICROS: i64 = 253_402_268_340_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PolicyAssignmentExpectationV1 {
    pub role_revision: u64,
    pub assignment_id: Uuid,
    pub assignment_revision: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeBusinessOperationV1 {
    Install,
    Grant,
    Revoke,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Input {
    Install,
    Grant {
        recipient: AccountId,
        assignment: Option<PolicyAssignmentExpectationV1>,
        expires_at: OffsetDateTime,
    },
    Revoke(PolicyAssignmentExpectationV1),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeCompanyBusinessCommandV1 {
    command_id: Uuid,
    company: OrgId,
    expected_company_epoch: u64,
    input: Input,
}

impl NativeCompanyBusinessCommandV1 {
    pub fn install(
        command_id: Uuid,
        company: OrgId,
        expected_company_epoch: u64,
    ) -> Result<Self, KernelError> {
        Self::new(command_id, company, expected_company_epoch, Input::Install)
    }

    pub fn grant(
        command_id: Uuid,
        company: OrgId,
        expected_company_epoch: u64,
        recipient: AccountId,
        expected_assignment: Option<PolicyAssignmentExpectationV1>,
        expires_at: OffsetDateTime,
    ) -> Result<Self, KernelError> {
        // Whole-minute instants only; never round historical command input.
        let nanos = expires_at.unix_timestamp_nanos();
        if nanos % 60_000_000_000 != 0
            || !(i128::from(MIN_EXPIRY_MICROS) * 1_000..=i128::from(MAX_EXPIRY_MICROS) * 1_000)
                .contains(&nanos)
        {
            return Err(invalid());
        }
        if let Some(assignment) = expected_assignment {
            validate_assignment(assignment)?;
        }
        Self::new(
            command_id,
            company,
            expected_company_epoch,
            Input::Grant {
                recipient,
                assignment: expected_assignment,
                expires_at: expires_at.to_offset(UtcOffset::UTC),
            },
        )
    }

    pub fn revoke(
        command_id: Uuid,
        company: OrgId,
        expected_company_epoch: u64,
        expected_assignment: PolicyAssignmentExpectationV1,
    ) -> Result<Self, KernelError> {
        validate_assignment(expected_assignment)?;
        Self::new(
            command_id,
            company,
            expected_company_epoch,
            Input::Revoke(expected_assignment),
        )
    }

    fn new(
        command_id: Uuid,
        company: OrgId,
        expected_company_epoch: u64,
        input: Input,
    ) -> Result<Self, KernelError> {
        if command_id.is_nil() || company.as_uuid().is_nil() || company == OrgId::platform() {
            return Err(invalid());
        }
        validate_revision(expected_company_epoch)?;
        Ok(Self {
            command_id,
            company,
            expected_company_epoch,
            input,
        })
    }

    pub const fn command_id(&self) -> Uuid {
        self.command_id
    }
    pub const fn company(&self) -> OrgId {
        self.company
    }
    pub const fn expected_company_epoch(&self) -> u64 {
        self.expected_company_epoch
    }
    pub const fn operation(&self) -> NativeBusinessOperationV1 {
        match self.input {
            Input::Install => NativeBusinessOperationV1::Install,
            Input::Grant { .. } => NativeBusinessOperationV1::Grant,
            Input::Revoke(_) => NativeBusinessOperationV1::Revoke,
        }
    }
    pub const fn recipient_account_id(&self) -> Option<AccountId> {
        match self.input {
            Input::Grant { recipient, .. } => Some(recipient),
            _ => None,
        }
    }
    pub const fn assignment_expectation(&self) -> Option<PolicyAssignmentExpectationV1> {
        match self.input {
            Input::Install => None,
            Input::Grant { assignment, .. } => assignment,
            Input::Revoke(assignment) => Some(assignment),
        }
    }
    pub const fn expires_at(&self) -> Option<OffsetDateTime> {
        match self.input {
            Input::Grant { expires_at, .. } => Some(expires_at),
            _ => None,
        }
    }

    pub fn encode(&self, actor: AccountId) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(180);
        bytes.extend_from_slice(PREFIX);
        bytes.extend_from_slice(actor.as_uuid().as_bytes());
        bytes.extend_from_slice(self.company.as_uuid().as_bytes());
        bytes.extend_from_slice(self.command_id.as_bytes());
        bytes.extend_from_slice(&self.expected_company_epoch.to_be_bytes());
        bytes.extend_from_slice(&MANIFEST);
        match self.input {
            Input::Install => bytes.push(1),
            Input::Grant {
                recipient,
                assignment,
                expires_at,
            } => {
                bytes.push(2);
                bytes.extend_from_slice(recipient.as_uuid().as_bytes());
                bytes.push(u8::from(assignment.is_some()));
                if let Some(assignment) = assignment {
                    write_assignment(&mut bytes, assignment);
                }
                // Constructor bounds make this exact signed conversion safe.
                let micros = (expires_at.unix_timestamp_nanos() / 1_000) as i64;
                bytes.extend_from_slice(&micros.to_be_bytes());
            }
            Input::Revoke(assignment) => {
                bytes.push(3);
                write_assignment(&mut bytes, assignment);
            }
        }
        bytes
    }

    pub fn decode(mut bytes: &[u8]) -> Result<(AccountId, Self), KernelError> {
        if take(&mut bytes, PREFIX.len())? != PREFIX {
            return Err(invalid());
        }
        let actor = AccountId::from_uuid(read_uuid(&mut bytes)?).map_err(|_| invalid())?;
        let company = OrgId::from_uuid(read_uuid(&mut bytes)?);
        let command = read_uuid(&mut bytes)?;
        let epoch = read_revision(&mut bytes)?;
        if take(&mut bytes, MANIFEST.len())? != MANIFEST {
            return Err(invalid());
        }
        let result = match take(&mut bytes, 1)? {
            [1] => Self::install(command, company, epoch)?,
            [2] => {
                let recipient =
                    AccountId::from_uuid(read_uuid(&mut bytes)?).map_err(|_| invalid())?;
                let assignment = match take(&mut bytes, 1)? {
                    [0] => None,
                    [1] => Some(read_assignment(&mut bytes)?),
                    _ => return Err(invalid()),
                };
                let micros =
                    i64::from_be_bytes(take(&mut bytes, 8)?.try_into().map_err(|_| invalid())?);
                let expires = OffsetDateTime::from_unix_timestamp_nanos(i128::from(micros) * 1_000)
                    .map_err(|_| invalid())?;
                Self::grant(command, company, epoch, recipient, assignment, expires)?
            }
            [3] => Self::revoke(command, company, epoch, read_assignment(&mut bytes)?)?,
            _ => return Err(invalid()),
        };
        if !bytes.is_empty() {
            return Err(invalid());
        }
        Ok((actor, result))
    }
}

fn invalid() -> KernelError {
    KernelError::validation("invalid Company business policy input")
}

fn validate_revision(value: u64) -> Result<(), KernelError> {
    if value == 0 || value > i64::MAX as u64 {
        Err(invalid())
    } else {
        Ok(())
    }
}

fn validate_assignment(value: PolicyAssignmentExpectationV1) -> Result<(), KernelError> {
    if value.role_revision != 1 || value.assignment_id.is_nil() {
        return Err(invalid());
    }
    validate_revision(value.assignment_revision)
}

fn take<'a>(input: &mut &'a [u8], len: usize) -> Result<&'a [u8], KernelError> {
    let (value, rest) = input.split_at_checked(len).ok_or_else(invalid)?;
    *input = rest;
    Ok(value)
}

fn read_uuid(input: &mut &[u8]) -> Result<Uuid, KernelError> {
    let value = Uuid::from_slice(take(input, 16)?).map_err(|_| invalid())?;
    if value.is_nil() {
        Err(invalid())
    } else {
        Ok(value)
    }
}

fn read_revision(input: &mut &[u8]) -> Result<u64, KernelError> {
    let value = u64::from_be_bytes(take(input, 8)?.try_into().map_err(|_| invalid())?);
    validate_revision(value)?;
    Ok(value)
}

fn read_assignment(input: &mut &[u8]) -> Result<PolicyAssignmentExpectationV1, KernelError> {
    let value = PolicyAssignmentExpectationV1 {
        role_revision: read_revision(input)?,
        assignment_id: read_uuid(input)?,
        assignment_revision: read_revision(input)?,
    };
    validate_assignment(value)?;
    Ok(value)
}

fn write_assignment(bytes: &mut Vec<u8>, value: PolicyAssignmentExpectationV1) {
    bytes.extend_from_slice(&value.role_revision.to_be_bytes());
    bytes.extend_from_slice(value.assignment_id.as_bytes());
    bytes.extend_from_slice(&value.assignment_revision.to_be_bytes());
}

#[cfg(test)]
mod tests;
