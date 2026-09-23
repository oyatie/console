//! Closed native People policy commands. Selectors never establish authority.
//! The local wire version is 1; the shared policy store discriminates it as codec 2.
use super::{
    AccountId,
    business::{
        NativeBusinessOperationV1, PolicyAssignmentExpectationV1, invalid, read_assignment,
        read_revision, read_uuid, take, validate_assignment, validate_expiry, validate_revision,
        write_assignment,
    },
};
use console_kernel_core::{KernelError, OrgId};
use time::{OffsetDateTime, UtcOffset};
use uuid::Uuid;

const PREFIX: &[u8] = b"console.company.people-policy\0\0\x01";
/// SHA-256 of the exact `native-people-directory-v1` PostgreSQL JSONB manifest.
pub const MANIFEST: [u8; 32] = [
    0x59, 0x1e, 0x8f, 0xe6, 0x26, 0xa1, 0x1c, 0xe7, 0x24, 0xc8, 0x13, 0x30, 0xf0, 0x35, 0x8f, 0x6f,
    0x4f, 0xb3, 0xdf, 0x4f, 0xa1, 0x53, 0x2f, 0xa3, 0x29, 0xee, 0x28, 0xd7, 0xc5, 0xb3, 0x8e, 0x5e,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectoryActionV1 {
    Read,
    Create,
}

impl DirectoryActionV1 {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Read => "people.directory.read",
            Self::Create => "people.directory.create",
        }
    }

    const fn byte(self) -> u8 {
        match self {
            Self::Read => 1,
            Self::Create => 2,
        }
    }

    fn decode(bytes: &mut &[u8]) -> Result<Self, KernelError> {
        match take(bytes, 1)? {
            [1] => Ok(Self::Read),
            [2] => Ok(Self::Create),
            _ => Err(invalid()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Input {
    Install,
    Grant {
        action: DirectoryActionV1,
        recipient: AccountId,
        assignment: Option<PolicyAssignmentExpectationV1>,
        expires_at: OffsetDateTime,
    },
    Revoke {
        action: DirectoryActionV1,
        assignment: PolicyAssignmentExpectationV1,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativePeoplePolicyCommandV1 {
    command_id: Uuid,
    company: OrgId,
    expected_company_epoch: u64,
    input: Input,
}

impl NativePeoplePolicyCommandV1 {
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
        action: DirectoryActionV1,
        recipient: AccountId,
        expected_assignment: Option<PolicyAssignmentExpectationV1>,
        expires_at: OffsetDateTime,
    ) -> Result<Self, KernelError> {
        validate_expiry(expires_at)?;
        if let Some(assignment) = expected_assignment {
            validate_assignment(assignment)?;
        }
        Self::new(
            command_id,
            company,
            expected_company_epoch,
            Input::Grant {
                action,
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
        action: DirectoryActionV1,
        expected_assignment: PolicyAssignmentExpectationV1,
    ) -> Result<Self, KernelError> {
        validate_assignment(expected_assignment)?;
        Self::new(
            command_id,
            company,
            expected_company_epoch,
            Input::Revoke {
                action,
                assignment: expected_assignment,
            },
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
            Input::Revoke { .. } => NativeBusinessOperationV1::Revoke,
        }
    }

    pub const fn action(&self) -> Option<DirectoryActionV1> {
        match self.input {
            Input::Install => None,
            Input::Grant { action, .. } | Input::Revoke { action, .. } => Some(action),
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
            Input::Revoke { assignment, .. } => Some(assignment),
        }
    }

    pub const fn expires_at(&self) -> Option<OffsetDateTime> {
        match self.input {
            Input::Grant { expires_at, .. } => Some(expires_at),
            _ => None,
        }
    }

    pub fn encode(&self, actor: AccountId) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(179);
        bytes.extend_from_slice(PREFIX);
        bytes.extend_from_slice(actor.as_uuid().as_bytes());
        bytes.extend_from_slice(self.company.as_uuid().as_bytes());
        bytes.extend_from_slice(self.command_id.as_bytes());
        bytes.extend_from_slice(&self.expected_company_epoch.to_be_bytes());
        bytes.extend_from_slice(&MANIFEST);
        match self.input {
            Input::Install => bytes.push(1),
            Input::Grant {
                action,
                recipient,
                assignment,
                expires_at,
            } => {
                bytes.extend_from_slice(&[2, action.byte()]);
                bytes.extend_from_slice(recipient.as_uuid().as_bytes());
                bytes.push(u8::from(assignment.is_some()));
                if let Some(assignment) = assignment {
                    write_assignment(&mut bytes, assignment);
                }
                // The constructor proves exact whole minutes in the signed range.
                bytes.extend_from_slice(
                    &((expires_at.unix_timestamp_nanos() / 1_000) as i64).to_be_bytes(),
                );
            }
            Input::Revoke { action, assignment } => {
                bytes.extend_from_slice(&[3, action.byte()]);
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
                let action = DirectoryActionV1::decode(&mut bytes)?;
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
                Self::grant(
                    command, company, epoch, action, recipient, assignment, expires,
                )?
            }
            [3] => {
                let action = DirectoryActionV1::decode(&mut bytes)?;
                Self::revoke(
                    command,
                    company,
                    epoch,
                    action,
                    read_assignment(&mut bytes)?,
                )?
            }
            _ => return Err(invalid()),
        };
        if !bytes.is_empty() {
            return Err(invalid());
        }
        Ok((actor, result))
    }
}
