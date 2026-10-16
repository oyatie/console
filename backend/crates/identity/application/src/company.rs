//! Validated native Company enrollment input and its immutable v1 record.
//! Decoded Account identifiers are namespace data, never authentication proof.

use console_kernel_core::KernelError;
use serde::{Deserialize, Deserializer, de};
use uuid::Uuid;

const PREFIX: &[u8] = b"console.company.enrollment\0\0\x01";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompanyEnrollmentV1 {
    command_id: Uuid,
    group_id: Option<Uuid>,
    slug: String,
    name: String,
    administrative_account_id: Uuid,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    command_id: String,
    // A nullable member is required: omission must not silently select a Group.
    #[serde(deserialize_with = "required_group")]
    group_id: Option<String>,
    slug: String,
    name: String,
    administrative_account_id: String,
}

fn required_group<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    Option::<String>::deserialize(d)
}

struct InputObject;

impl<'de> de::Visitor<'de> for InputObject {
    type Value = Input;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a Company enrollment object")
    }

    fn visit_map<M: de::MapAccess<'de>>(self, map: M) -> Result<Input, M::Error> {
        Input::deserialize(de::value::MapAccessDeserializer::new(map))
    }
}

impl CompanyEnrollmentV1 {
    pub const fn command_id(&self) -> Uuid {
        self.command_id
    }
    pub const fn group_id(&self) -> Option<Uuid> {
        self.group_id
    }
    pub const fn administrative_account_id(&self) -> Uuid {
        self.administrative_account_id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn slug(&self) -> &str {
        &self.slug
    }

    /// Parse the closed v1 input without losing duplicate-member evidence.
    /// The HTTP adapter must also bound body reads before allocating the body.
    pub fn from_json_slice(input: &[u8]) -> Result<Self, KernelError> {
        if input.len() > 4096 {
            return Err(invalid());
        }
        let mut parser = serde_json::Deserializer::from_slice(input);
        let fields = (&mut parser)
            .deserialize_map(InputObject)
            .map_err(|_| invalid())?;
        parser.end().map_err(|_| invalid())?;
        let value = Self {
            command_id: canonical_uuid(&fields.command_id)?,
            group_id: fields.group_id.as_deref().map(canonical_uuid).transpose()?,
            slug: fields.slug,
            name: fields.name,
            administrative_account_id: canonical_uuid(&fields.administrative_account_id)?,
        };
        value.validate()?;
        Ok(value)
    }

    /// Encode `company-enrollment-input/1` with exact original name bytes.
    /// The caller must obtain the Account namespace from its verified session.
    pub fn encode(&self, account_id: Uuid) -> Result<Vec<u8>, KernelError> {
        non_nil(account_id)?;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(PREFIX);
        bytes.extend_from_slice(account_id.as_bytes());
        bytes.extend_from_slice(self.command_id.as_bytes());
        bytes.push(u8::from(self.group_id.is_some()));
        if let Some(group) = self.group_id {
            bytes.extend_from_slice(group.as_bytes());
        }
        bytes.extend_from_slice(self.administrative_account_id.as_bytes());
        for value in [&self.slug, &self.name] {
            let length = u32::try_from(value.len()).map_err(|_| invalid())?;
            bytes.extend_from_slice(&length.to_be_bytes());
            bytes.extend_from_slice(value.as_bytes());
        }
        Ok(bytes)
    }

    /// Decode only v1, rejecting alternate encodings and trailing bytes.
    /// The owner must compare both returned identities with its request key.
    pub fn decode(mut input: &[u8]) -> Result<(Uuid, Self), KernelError> {
        if take(&mut input, PREFIX.len())? != PREFIX {
            return Err(invalid());
        }
        let account = read_uuid(&mut input)?;
        let command_id = read_uuid(&mut input)?;
        let group_id = match take(&mut input, 1)? {
            [0] => None,
            [1] => Some(read_uuid(&mut input)?),
            _ => return Err(invalid()),
        };
        let administrative_account_id = read_uuid(&mut input)?;
        let value = Self {
            command_id,
            group_id,
            administrative_account_id,
            slug: read_text(&mut input, 63)?,
            name: read_text(&mut input, 256)?,
        };
        if !input.is_empty() {
            return Err(invalid());
        }
        value.validate()?;
        Ok((account, value))
    }

    fn validate(&self) -> Result<(), KernelError> {
        non_nil(self.command_id)?;
        non_nil(self.administrative_account_id)?;
        if let Some(group) = self.group_id {
            non_nil(group)?;
        }
        validate_name_slug(&self.name, &self.slug)
    }
}

pub(crate) fn validate_name_slug(name: &str, slug: &str) -> Result<(), KernelError> {
    let slug = slug.as_bytes();
    if !(1..=63).contains(&slug.len())
        || !slug.first().is_some_and(u8::is_ascii_alphanumeric)
        || !slug.last().is_some_and(u8::is_ascii_alphanumeric)
        || !slug
            .iter()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'-')
        || !(1..=256).contains(&name.len())
        || name.trim().is_empty()
        || name.chars().any(char::is_control)
    {
        return Err(invalid());
    }
    Ok(())
}

fn invalid() -> KernelError {
    // Do not echo command contents or identifiers into errors or logs.
    KernelError::validation("invalid Company enrollment input")
}

fn non_nil(id: Uuid) -> Result<Uuid, KernelError> {
    if id.is_nil() { Err(invalid()) } else { Ok(id) }
}

fn canonical_uuid(raw: &str) -> Result<Uuid, KernelError> {
    let id = non_nil(Uuid::parse_str(raw).map_err(|_| invalid())?)?;
    if id.hyphenated().to_string() != raw {
        return Err(invalid());
    }
    Ok(id)
}

fn take<'a>(input: &mut &'a [u8], len: usize) -> Result<&'a [u8], KernelError> {
    let (value, rest) = input.split_at_checked(len).ok_or_else(invalid)?;
    *input = rest;
    Ok(value)
}

fn read_uuid(input: &mut &[u8]) -> Result<Uuid, KernelError> {
    non_nil(Uuid::from_slice(take(input, 16)?).map_err(|_| invalid())?)
}

fn read_text(input: &mut &[u8], max: usize) -> Result<String, KernelError> {
    let length = u32::from_be_bytes(take(input, 4)?.try_into().map_err(|_| invalid())?);
    let length = usize::try_from(length).map_err(|_| invalid())?;
    if !(1..=max).contains(&length) {
        return Err(invalid());
    }
    // Check the length and remaining slice before allocating decoded text.
    let value = std::str::from_utf8(take(input, length)?).map_err(|_| invalid())?;
    Ok(value.to_owned())
}

#[cfg(test)]
#[path = "company_enrollment_tests.rs"]
mod tests;
