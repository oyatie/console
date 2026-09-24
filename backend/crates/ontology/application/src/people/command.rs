//! Versioned, Account-attributed directory registration command bytes.
use crate::people::DirectoryRegistrationInput;
use console_kernel_core::{AccountId, KernelError, OrgId};
use uuid::Uuid;

const PREFIX: &[u8] = b"console.people.directory-register\0\0\x01";
const EFFECT_PREFIX: &[u8] = b"console.people.directory-effect\0\0\x01";
pub const DIRECTORY_CODEC_VERSION: i16 = 1;
pub const DIRECTORY_MAX_INPUT_BYTES: usize = 1280;
/// Exact existing native-people-directory-v1 policy catalog manifest.
pub const DIRECTORY_MANIFEST: [u8; 32] = [
    0x59, 0x1e, 0x8f, 0xe6, 0x26, 0xa1, 0x1c, 0xe7, 0x24, 0xc8, 0x13, 0x30, 0xf0, 0x35, 0x8f, 0x6f,
    0x4f, 0xb3, 0xdf, 0x4f, 0xa1, 0x53, 0x2f, 0xa3, 0x29, 0xee, 0x28, 0xd7, 0xc5, 0xb3, 0x8e, 0x5e,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirectoryExpectationsV1 {
    pub company_epoch: u64,
    pub object_type_id: Uuid,
    pub action_type_id: Uuid,
    pub action_revision: u64,
    pub schema_revision: u64,
    pub legal_name_property_id: Uuid,
    pub employee_number_property_id: Uuid,
}

impl DirectoryExpectationsV1 {
    pub(super) fn validate(self) -> Result<(), KernelError> {
        for value in [
            self.company_epoch,
            self.action_revision,
            self.schema_revision,
        ] {
            if value == 0 || value > i64::MAX as u64 {
                return Err(invalid());
            }
        }
        for id in [
            self.object_type_id,
            self.action_type_id,
            self.legal_name_property_id,
            self.employee_number_property_id,
        ] {
            if id.is_nil() {
                return Err(invalid());
            }
        }
        if self.legal_name_property_id == self.employee_number_property_id {
            return Err(invalid());
        }
        Ok(())
    }
}

/// Accepted registration only. Server allocates employee_id once under prepare.
/// Execute/cancel/status select the original Company+command and do not re-encode
/// new authority, a new identity, or a changed request under this command ID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeDirectoryCommandV1 {
    command_id: Uuid,
    company: OrgId,
    employee_id: Uuid,
    expected: DirectoryExpectationsV1,
    input: DirectoryRegistrationInput,
}

impl NativeDirectoryCommandV1 {
    pub fn new(
        command_id: Uuid,
        company: OrgId,
        employee_id: Uuid,
        expected: DirectoryExpectationsV1,
        input: DirectoryRegistrationInput,
    ) -> Result<Self, KernelError> {
        if command_id.is_nil()
            || company.as_uuid().is_nil()
            || company == OrgId::platform()
            || employee_id.is_nil()
        {
            return Err(invalid());
        }
        expected.validate()?;
        Ok(Self {
            command_id,
            company,
            employee_id,
            expected,
            input,
        })
    }

    pub const fn command_id(&self) -> Uuid {
        self.command_id
    }
    pub const fn company(&self) -> OrgId {
        self.company
    }
    pub const fn employee_id(&self) -> Uuid {
        self.employee_id
    }
    pub const fn expected(&self) -> DirectoryExpectationsV1 {
        self.expected
    }
    pub fn input(&self) -> &DirectoryRegistrationInput {
        &self.input
    }

    pub fn encode(&self, actor: AccountId) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(
            224 + self.input.legal_name().len() + self.input.employee_number().len(),
        );
        bytes.extend_from_slice(PREFIX);
        for id in [
            actor.as_uuid(),
            self.company.as_uuid(),
            &self.command_id,
            &self.employee_id,
        ] {
            bytes.extend_from_slice(id.as_bytes());
        }
        bytes.extend_from_slice(&self.expected.company_epoch.to_be_bytes());
        bytes.extend_from_slice(self.expected.object_type_id.as_bytes());
        bytes.extend_from_slice(self.expected.action_type_id.as_bytes());
        bytes.extend_from_slice(&self.expected.action_revision.to_be_bytes());
        bytes.extend_from_slice(&self.expected.schema_revision.to_be_bytes());
        bytes.extend_from_slice(self.expected.legal_name_property_id.as_bytes());
        bytes.extend_from_slice(self.expected.employee_number_property_id.as_bytes());
        bytes.extend_from_slice(&DIRECTORY_MANIFEST);
        for text in [self.input.legal_name(), self.input.employee_number()] {
            // DirectoryRegistrationInput proves UTF-8 lengths <=800/256.
            bytes.extend_from_slice(&(text.len() as u16).to_be_bytes());
            bytes.extend_from_slice(text.as_bytes());
        }
        bytes
    }

    /// Adapter hashes these exact bytes with SHA-256 for canonical receipt,
    /// Person revision and binding. Input digest hashes encode(actor) alone.
    /// Hashing stays with the existing adapter sha2 dependency.
    pub fn effect_payload(&self, actor: AccountId) -> Vec<u8> {
        let mut bytes = EFFECT_PREFIX.to_vec();
        bytes.extend_from_slice(&self.encode(actor));
        bytes
    }

    pub fn decode(bytes: &[u8]) -> Result<(AccountId, Self), KernelError> {
        if !(226..=DIRECTORY_MAX_INPUT_BYTES).contains(&bytes.len()) {
            return Err(invalid());
        }
        let mut rest = bytes;
        if take(&mut rest, PREFIX.len())? != PREFIX {
            return Err(invalid());
        }
        let actor = AccountId::from_uuid(read_uuid(&mut rest)?).map_err(|_| invalid())?;
        let company = OrgId::from_uuid(read_uuid(&mut rest)?);
        let command_id = read_uuid(&mut rest)?;
        let employee_id = read_uuid(&mut rest)?;
        let expected = DirectoryExpectationsV1 {
            company_epoch: read_u64(&mut rest)?,
            object_type_id: read_uuid(&mut rest)?,
            action_type_id: read_uuid(&mut rest)?,
            action_revision: read_u64(&mut rest)?,
            schema_revision: read_u64(&mut rest)?,
            legal_name_property_id: read_uuid(&mut rest)?,
            employee_number_property_id: read_uuid(&mut rest)?,
        };
        if take(&mut rest, DIRECTORY_MANIFEST.len())? != DIRECTORY_MANIFEST {
            return Err(invalid());
        }
        let legal_name = read_text(&mut rest, 800)?;
        let employee_number = read_text(&mut rest, 256)?;
        if !rest.is_empty() {
            return Err(invalid());
        }
        let input =
            DirectoryRegistrationInput::new(legal_name, employee_number).map_err(|_| invalid())?;
        // Persisted command bytes are canonical. Never silently reinterpret old
        // accepted whitespace as a newly trimmed value during replay.
        if input.legal_name() != legal_name || input.employee_number() != employee_number {
            return Err(invalid());
        }
        let command = Self::new(command_id, company, employee_id, expected, input)?;
        Ok((actor, command))
    }
}

fn invalid() -> KernelError {
    KernelError::validation("invalid native directory command")
}
fn take<'a>(bytes: &mut &'a [u8], size: usize) -> Result<&'a [u8], KernelError> {
    let (value, rest) = bytes.split_at_checked(size).ok_or_else(invalid)?;
    *bytes = rest;
    Ok(value)
}
fn read_uuid(bytes: &mut &[u8]) -> Result<Uuid, KernelError> {
    let value = Uuid::from_slice(take(bytes, 16)?).map_err(|_| invalid())?;
    if value.is_nil() {
        Err(invalid())
    } else {
        Ok(value)
    }
}
fn read_u64(bytes: &mut &[u8]) -> Result<u64, KernelError> {
    Ok(u64::from_be_bytes(
        take(bytes, 8)?.try_into().map_err(|_| invalid())?,
    ))
}
fn read_text<'a>(bytes: &mut &'a [u8], max_bytes: usize) -> Result<&'a str, KernelError> {
    let len = u16::from_be_bytes(take(bytes, 2)?.try_into().map_err(|_| invalid())?) as usize;
    if len == 0 || len > max_bytes {
        return Err(invalid());
    }
    std::str::from_utf8(take(bytes, len)?).map_err(|_| invalid())
}

#[cfg(test)]
#[path = "command_tests.rs"]
mod tests;
