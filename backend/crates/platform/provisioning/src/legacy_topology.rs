//! Immutable v1 inputs for the existing legacy topology owner.
//! Decoded actors and expected heads are data, never current authorization.
use console_kernel_core::{KernelError, UserId};
use sha2::{Digest, Sha256};
use uuid::Uuid;

const PREFIX: &[u8] = b"console.platform.topology\0\0\x01";
const MAX_BYTES: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpectedGroupHead {
    group_id: Uuid,
    incarnation: Uuid,
    revision: u64,
}

impl ExpectedGroupHead {
    pub fn new(group_id: Uuid, incarnation: Uuid, revision: u64) -> Result<Self, KernelError> {
        nonnil(group_id)?;
        nonnil(incarnation)?;
        if !(1..=i64::MAX as u64).contains(&revision) {
            return Err(invalid());
        }
        Ok(Self {
            group_id,
            incarnation,
            revision,
        })
    }
    pub fn group_id(&self) -> Uuid {
        self.group_id
    }
    pub fn incarnation(&self) -> Uuid {
        self.incarnation
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
}

// Wire tags are explicit matches, independent of Rust enum layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompanyStatus {
    Active,
    Suspended,
    Archived,
}
impl CompanyStatus {
    fn tag(self) -> u8 {
        match self {
            Self::Active => 1,
            Self::Suspended => 2,
            Self::Archived => 3,
        }
    }
    fn from_tag(tag: u8) -> Result<Self, KernelError> {
        match tag {
            1 => Ok(Self::Active),
            2 => Ok(Self::Suspended),
            3 => Ok(Self::Archived),
            _ => Err(invalid()),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupRole {
    Admin,
    Viewer,
    Finance,
}
impl GroupRole {
    fn tag(self) -> u8 {
        match self {
            Self::Admin => 1,
            Self::Viewer => 2,
            Self::Finance => 3,
        }
    }
    fn from_tag(tag: u8) -> Result<Self, KernelError> {
        match tag {
            1 => Ok(Self::Admin),
            2 => Ok(Self::Viewer),
            3 => Ok(Self::Finance),
            _ => Err(invalid()),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TenantRole {
    SuperAdmin,
    Admin,
    Mechanic,
    Receptionist,
    Executive,
    Member,
}
impl TenantRole {
    fn tag(self) -> u8 {
        match self {
            Self::SuperAdmin => 1,
            Self::Admin => 2,
            Self::Mechanic => 3,
            Self::Receptionist => 4,
            Self::Executive => 5,
            Self::Member => 6,
        }
    }
    fn from_tag(tag: u8) -> Result<Self, KernelError> {
        match tag {
            1 => Ok(Self::SuperAdmin),
            2 => Ok(Self::Admin),
            3 => Ok(Self::Mechanic),
            4 => Ok(Self::Receptionist),
            5 => Ok(Self::Executive),
            6 => Ok(Self::Member),
            _ => Err(invalid()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LegacyTopologyIntent {
    CreateCompany {
        slug: String,
        name: String,
    },
    CreateGroup {
        slug: String,
        name: String,
    },
    UpdateGroup {
        group: Uuid,
        slug: Option<String>,
        name: Option<String>,
        status: Option<CompanyStatus>,
    },
    AssignCompany {
        group: Uuid,
        company: Uuid,
    },
    RemoveCompanyFromGroup {
        group: Uuid,
        company: Uuid,
    },
    CreateLegacyGroupAccount {
        group: Uuid,
        company: Uuid,
        display_name: String,
        phone: Option<String>,
        tenant_roles: Vec<TenantRole>,
        group_role: GroupRole,
    },
    RevokeLegacyGroupRole {
        group: Uuid,
        user: Uuid,
        group_role: GroupRole,
    },
    RemoveEmptyCompany {
        company: Uuid,
    },
    SetCompanyStatus {
        company: Uuid,
        status: CompanyStatus,
    },
}

#[derive(Clone)]
pub struct LegacyTopologyCommandV1 {
    actor: UserId,
    command_id: Uuid,
    expected_groups: Vec<ExpectedGroupHead>,
    intent: LegacyTopologyIntent,
    bytes: Vec<u8>,
    digest: [u8; 32],
}

impl LegacyTopologyCommandV1 {
    /// The caller derives actor from its signed source. This constructor checks
    /// syntax only; the owning transaction must recheck source and membership.
    pub fn new(
        actor: UserId,
        command_id: Uuid,
        expected_groups: Vec<ExpectedGroupHead>,
        mut intent: LegacyTopologyIntent,
    ) -> Result<Self, KernelError> {
        intent.normalize()?;
        Self::validate(actor, command_id, &expected_groups, &intent)?;
        let mut bytes = PREFIX.to_vec();
        bytes.extend_from_slice(actor.as_uuid().as_bytes());
        bytes.extend_from_slice(command_id.as_bytes());
        bytes.push(intent.tag());
        bytes.push(u8::try_from(expected_groups.len()).map_err(|_| invalid())?);
        for head in &expected_groups {
            bytes.extend_from_slice(head.group_id.as_bytes());
            bytes.extend_from_slice(head.incarnation.as_bytes());
            bytes.extend_from_slice(&head.revision.to_be_bytes());
        }
        intent.write(&mut bytes)?;
        if bytes.len() > MAX_BYTES {
            return Err(invalid());
        }
        let digest = Sha256::digest(&bytes).into();
        Ok(Self {
            actor,
            command_id,
            expected_groups,
            intent,
            bytes,
            digest,
        })
    }

    /// Never trim or reinterpret persisted input. Retain its original bytes.
    pub fn decode(bytes: &[u8]) -> Result<Self, KernelError> {
        if bytes.len() > MAX_BYTES {
            return Err(invalid());
        }
        let mut input = bytes;
        if take(&mut input, PREFIX.len())? != PREFIX {
            return Err(invalid());
        }
        let actor = UserId::from_uuid(read_uuid(&mut input)?);
        let command_id = read_uuid(&mut input)?;
        let kind = byte(&mut input)?;
        let count = byte(&mut input)?;
        if count > 2 {
            return Err(invalid());
        }
        let mut expected_groups = Vec::with_capacity(usize::from(count));
        for _ in 0..count {
            let group = read_uuid(&mut input)?;
            let incarnation = read_uuid(&mut input)?;
            let revision =
                u64::from_be_bytes(take(&mut input, 8)?.try_into().map_err(|_| invalid())?);
            expected_groups.push(ExpectedGroupHead::new(group, incarnation, revision)?);
        }
        let intent = LegacyTopologyIntent::read(kind, &mut input)?;
        if !input.is_empty() {
            return Err(invalid());
        }
        Self::validate(actor, command_id, &expected_groups, &intent)?;
        Ok(Self {
            actor,
            command_id,
            expected_groups,
            intent,
            bytes: bytes.to_vec(),
            digest: Sha256::digest(bytes).into(),
        })
    }

    fn validate(
        actor: UserId,
        command: Uuid,
        heads: &[ExpectedGroupHead],
        intent: &LegacyTopologyIntent,
    ) -> Result<(), KernelError> {
        nonnil(*actor.as_uuid())?;
        nonnil(command)?;
        if heads.len() > 2
            || heads
                .windows(2)
                .any(|pair| pair[0].group_id >= pair[1].group_id)
        {
            return Err(invalid());
        }
        intent.validate()?;
        use LegacyTopologyIntent::*;
        let (min, max, group) = match intent {
            CreateCompany { .. } | CreateGroup { .. } => (0, 0, None),
            UpdateGroup { group, .. }
            | RemoveCompanyFromGroup { group, .. }
            | CreateLegacyGroupAccount { group, .. } => (1, 1, Some(*group)),
            AssignCompany { group, .. } | RevokeLegacyGroupRole { group, .. } => {
                (1, 2, Some(*group))
            }
            RemoveEmptyCompany { .. } | SetCompanyStatus { .. } => (1, 1, None),
        };
        if !(min..=max).contains(&heads.len())
            || group.is_some_and(|id| !heads.iter().any(|head| head.group_id == id))
        {
            return Err(invalid());
        }
        Ok(())
    }

    pub fn actor(&self) -> UserId {
        self.actor
    }
    pub fn command_id(&self) -> Uuid {
        self.command_id
    }
    pub fn expected_groups(&self) -> &[ExpectedGroupHead] {
        &self.expected_groups
    }
    pub fn intent(&self) -> &LegacyTopologyIntent {
        &self.intent
    }
    pub fn encode(&self) -> &[u8] {
        &self.bytes
    }
    pub fn digest(&self) -> &[u8; 32] {
        &self.digest
    }
}

impl LegacyTopologyIntent {
    fn tag(&self) -> u8 {
        match self {
            Self::CreateCompany { .. } => 1,
            Self::CreateGroup { .. } => 2,
            Self::UpdateGroup { .. } => 3,
            Self::AssignCompany { .. } => 4,
            Self::RemoveCompanyFromGroup { .. } => 5,
            Self::CreateLegacyGroupAccount { .. } => 6,
            Self::RevokeLegacyGroupRole { .. } => 7,
            Self::RemoveEmptyCompany { .. } => 8,
            Self::SetCompanyStatus { .. } => 9,
        }
    }
    fn normalize(&mut self) -> Result<(), KernelError> {
        match self {
            Self::CreateCompany { slug, name } | Self::CreateGroup { slug, name } => {
                normalize_text(slug, 40)?;
                normalize_text(name, 256)?;
            }
            Self::UpdateGroup { slug, name, .. } => {
                normalize_optional(slug, 40)?;
                normalize_optional(name, 256)?;
            }
            Self::CreateLegacyGroupAccount {
                display_name,
                phone,
                ..
            } => {
                normalize_text(display_name, 256)?;
                normalize_optional(phone, 64)?;
            }
            _ => {}
        }
        Ok(())
    }
    fn validate(&self) -> Result<(), KernelError> {
        match self {
            Self::CreateCompany { slug, name } | Self::CreateGroup { slug, name } => {
                validate_slug(slug)?;
                validate_text(name, 256)?;
            }
            Self::UpdateGroup {
                group,
                slug,
                name,
                status,
            } => {
                nonnil(*group)?;
                if let Some(slug) = slug {
                    validate_slug(slug)?;
                }
                if let Some(name) = name {
                    validate_text(name, 256)?;
                }
                if slug.is_none() && name.is_none() && status.is_none() {
                    return Err(invalid());
                }
            }
            Self::AssignCompany { group, company }
            | Self::RemoveCompanyFromGroup { group, company } => {
                nonnil(*group)?;
                nonnil(*company)?;
            }
            Self::CreateLegacyGroupAccount {
                group,
                company,
                display_name,
                phone,
                tenant_roles,
                ..
            } => {
                nonnil(*group)?;
                nonnil(*company)?;
                validate_text(display_name, 256)?;
                if let Some(phone) = phone {
                    validate_text(phone, 64)?;
                }
                if !(1..=64).contains(&tenant_roles.len()) {
                    return Err(invalid());
                }
            }
            Self::RevokeLegacyGroupRole { group, user, .. } => {
                nonnil(*group)?;
                nonnil(*user)?;
            }
            Self::RemoveEmptyCompany { company } | Self::SetCompanyStatus { company, .. } => {
                nonnil(*company)?;
            }
        }
        Ok(())
    }
    fn write(&self, bytes: &mut Vec<u8>) -> Result<(), KernelError> {
        match self {
            Self::CreateCompany { slug, name } | Self::CreateGroup { slug, name } => {
                write_text(bytes, slug)?;
                write_text(bytes, name)?;
            }
            Self::UpdateGroup {
                group,
                slug,
                name,
                status,
            } => {
                bytes.extend_from_slice(group.as_bytes());
                write_optional(bytes, slug)?;
                write_optional(bytes, name)?;
                bytes.push(u8::from(status.is_some()));
                if let Some(status) = status {
                    bytes.push(status.tag());
                }
            }
            Self::AssignCompany { group, company }
            | Self::RemoveCompanyFromGroup { group, company } => {
                bytes.extend_from_slice(group.as_bytes());
                bytes.extend_from_slice(company.as_bytes());
            }
            Self::CreateLegacyGroupAccount {
                group,
                company,
                display_name,
                phone,
                tenant_roles,
                group_role,
            } => {
                bytes.extend_from_slice(group.as_bytes());
                bytes.extend_from_slice(company.as_bytes());
                write_text(bytes, display_name)?;
                write_optional(bytes, phone)?;
                bytes.push(u8::try_from(tenant_roles.len()).map_err(|_| invalid())?);
                bytes.extend(tenant_roles.iter().map(|role| role.tag()));
                bytes.push(group_role.tag());
            }
            Self::RevokeLegacyGroupRole {
                group,
                user,
                group_role,
            } => {
                bytes.extend_from_slice(group.as_bytes());
                bytes.extend_from_slice(user.as_bytes());
                bytes.push(group_role.tag());
            }
            Self::RemoveEmptyCompany { company } => bytes.extend_from_slice(company.as_bytes()),
            Self::SetCompanyStatus { company, status } => {
                bytes.extend_from_slice(company.as_bytes());
                bytes.push(status.tag());
            }
        }
        Ok(())
    }
    fn read(kind: u8, input: &mut &[u8]) -> Result<Self, KernelError> {
        Ok(match kind {
            1 => Self::CreateCompany {
                slug: read_text(input, 40)?,
                name: read_text(input, 256)?,
            },
            2 => Self::CreateGroup {
                slug: read_text(input, 40)?,
                name: read_text(input, 256)?,
            },
            3 => Self::UpdateGroup {
                group: read_uuid(input)?,
                slug: read_optional(input, 40)?,
                name: read_optional(input, 256)?,
                status: match byte(input)? {
                    0 => None,
                    1 => Some(CompanyStatus::from_tag(byte(input)?)?),
                    _ => return Err(invalid()),
                },
            },
            4 => Self::AssignCompany {
                group: read_uuid(input)?,
                company: read_uuid(input)?,
            },
            5 => Self::RemoveCompanyFromGroup {
                group: read_uuid(input)?,
                company: read_uuid(input)?,
            },
            6 => {
                let group = read_uuid(input)?;
                let company = read_uuid(input)?;
                let display_name = read_text(input, 256)?;
                let phone = read_optional(input, 64)?;
                let count = byte(input)?;
                if !(1..=64).contains(&count) {
                    return Err(invalid());
                }
                let tenant_roles = take(input, usize::from(count))?
                    .iter()
                    .map(|tag| TenantRole::from_tag(*tag))
                    .collect::<Result<Vec<_>, _>>()?;
                Self::CreateLegacyGroupAccount {
                    group,
                    company,
                    display_name,
                    phone,
                    tenant_roles,
                    group_role: GroupRole::from_tag(byte(input)?)?,
                }
            }
            7 => Self::RevokeLegacyGroupRole {
                group: read_uuid(input)?,
                user: read_uuid(input)?,
                group_role: GroupRole::from_tag(byte(input)?)?,
            },
            8 => Self::RemoveEmptyCompany {
                company: read_uuid(input)?,
            },
            9 => Self::SetCompanyStatus {
                company: read_uuid(input)?,
                status: CompanyStatus::from_tag(byte(input)?)?,
            },
            _ => return Err(invalid()),
        })
    }
}

fn invalid() -> KernelError {
    KernelError::validation("invalid legacy topology command")
}
fn nonnil(id: Uuid) -> Result<Uuid, KernelError> {
    if id.is_nil() { Err(invalid()) } else { Ok(id) }
}
fn validate_text(text: &str, max: usize) -> Result<(), KernelError> {
    if !(1..=max).contains(&text.len()) || text != text.trim() || text.chars().any(char::is_control)
    {
        return Err(invalid());
    }
    Ok(())
}
fn normalize_text(text: &mut String, max: usize) -> Result<(), KernelError> {
    let trimmed = text.trim();
    validate_text(trimmed, max)?;
    if trimmed.len() != text.len() {
        *text = trimmed.to_owned();
    }
    Ok(())
}
fn normalize_optional(text: &mut Option<String>, max: usize) -> Result<(), KernelError> {
    if let Some(value) = text {
        if value.trim().is_empty() {
            *text = None;
        } else {
            normalize_text(value, max)?;
        }
    }
    Ok(())
}
fn validate_slug(slug: &str) -> Result<(), KernelError> {
    let bytes = slug.as_bytes();
    if !(3..=40).contains(&bytes.len())
        || !bytes.first().is_some_and(u8::is_ascii_alphanumeric)
        || !bytes.last().is_some_and(u8::is_ascii_alphanumeric)
        || !bytes
            .iter()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'-')
    {
        return Err(invalid());
    }
    Ok(())
}
fn write_text(bytes: &mut Vec<u8>, text: &str) -> Result<(), KernelError> {
    bytes.extend_from_slice(
        &u32::try_from(text.len())
            .map_err(|_| invalid())?
            .to_be_bytes(),
    );
    bytes.extend_from_slice(text.as_bytes());
    Ok(())
}
fn write_optional(bytes: &mut Vec<u8>, text: &Option<String>) -> Result<(), KernelError> {
    bytes.push(u8::from(text.is_some()));
    if let Some(text) = text {
        write_text(bytes, text)?;
    }
    Ok(())
}
fn take<'a>(input: &mut &'a [u8], len: usize) -> Result<&'a [u8], KernelError> {
    let (value, rest) = input.split_at_checked(len).ok_or_else(invalid)?;
    *input = rest;
    Ok(value)
}
fn byte(input: &mut &[u8]) -> Result<u8, KernelError> {
    Ok(take(input, 1)?[0])
}
fn read_uuid(input: &mut &[u8]) -> Result<Uuid, KernelError> {
    nonnil(Uuid::from_slice(take(input, 16)?).map_err(|_| invalid())?)
}
fn read_text(input: &mut &[u8], max: usize) -> Result<String, KernelError> {
    let len = u32::from_be_bytes(take(input, 4)?.try_into().map_err(|_| invalid())?);
    let len = usize::try_from(len).map_err(|_| invalid())?;
    if !(1..=max).contains(&len) {
        return Err(invalid());
    }
    let text = std::str::from_utf8(take(input, len)?).map_err(|_| invalid())?;
    validate_text(text, max)?;
    Ok(text.to_owned())
}
fn read_optional(input: &mut &[u8], max: usize) -> Result<Option<String>, KernelError> {
    match byte(input)? {
        0 => Ok(None),
        1 => Ok(Some(read_text(input, max)?)),
        _ => Err(invalid()),
    }
}

#[cfg(test)]
#[path = "legacy_topology_tests.rs"]
mod tests;
