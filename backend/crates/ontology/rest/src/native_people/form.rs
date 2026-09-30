//! Exact bounded native HTML grammar. No identity, authority or mutations.
use axum::http::StatusCode;
use console_kernel_core::OrgId;
use console_ontology_application::people::{
    DirectoryExpectationsV1, DirectoryInputProblem, DirectoryRegistrationInput,
    workflow::{DirectoryPageQuery, DirectoryRequestRef, DirectorySubmission},
};
use std::collections::BTreeMap;
use uuid::Uuid;

pub(super) const MAX_BODY: usize = 8 * 1024;
pub(super) enum Input {
    Prepare {
        draft: Draft,
        submission: Option<DirectorySubmission>,
    },
    Execute(DirectoryRequestRef),
    Cancel(DirectoryRequestRef),
}
pub(super) struct Parsed {
    pub proof: String,
    pub input: Input,
}
// Raw invalid values and proof-bearing forms are deliberately not Debug/Serialize.
pub struct Draft {
    pub locator: DirectoryRequestRef,
    pub expected: DirectoryExpectationsV1,
    pub legal_name: String,
    pub employee_number: String,
    pub name_error: Option<DirectoryInputProblem>,
    pub number_error: Option<DirectoryInputProblem>,
}
#[derive(Clone, Copy)]
pub enum PostTarget<'a> {
    Prepare,
    Execute(&'a str),
    Cancel(&'a str),
}

pub(super) fn id(raw: &str) -> Result<Uuid, StatusCode> {
    let id = Uuid::parse_str(raw).map_err(|_| StatusCode::NOT_FOUND)?;
    if id.is_nil() || id.to_string() != raw {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(id)
}
pub(super) fn company(raw: &str) -> Result<OrgId, StatusCode> {
    let company = OrgId::from_uuid(id(raw)?);
    if company == OrgId::platform() {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(company)
}
pub(super) fn pagination(raw: Option<&str>) -> Result<DirectoryPageQuery, StatusCode> {
    let mut after = None;
    let mut number = None;
    if let Some(raw) = raw {
        // The bound is on the encoded bytes, before any decoding or allocation.
        if raw.is_empty() || raw.len() > 1024 {
            return Err(StatusCode::BAD_REQUEST);
        }
        for pair in raw.split('&') {
            let (key, value) = pair.split_once('=').ok_or(StatusCode::BAD_REQUEST)?;
            match key {
                "after_employee_id" if after.is_none() => {
                    let decoded = decode(value.as_bytes())?;
                    after = Some(id(&decoded).map_err(|_| StatusCode::BAD_REQUEST)?);
                }
                "employee_number" if number.is_none() => {
                    let decoded = decode(value.as_bytes())?;
                    if decoded.len() > 256 {
                        return Err(StatusCode::BAD_REQUEST);
                    }
                    number = Some(decoded);
                }
                _ => return Err(StatusCode::BAD_REQUEST),
            }
        }
    }
    let number = match number.as_deref() {
        None | Some("") => None,
        Some(value) => Some(
            DirectoryRegistrationInput::normalized_employee_number(value)
                .map_err(|_| StatusCode::BAD_REQUEST)?
                .to_owned(),
        ),
    };
    DirectoryPageQuery::with_number(after, None, number).map_err(|_| StatusCode::BAD_REQUEST)
}
pub(super) fn no_query(raw: Option<&str>) -> Result<(), StatusCode> {
    if raw.is_some() {
        Err(StatusCode::BAD_REQUEST)
    } else {
        Ok(())
    }
}
pub(super) fn parse(
    company: OrgId,
    target: PostTarget<'_>,
    body: &[u8],
) -> Result<Parsed, StatusCode> {
    if body.len() > MAX_BODY {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }
    let allowed: &[&str] = match target {
        PostTarget::Prepare => &[
            "csrf_proof",
            "command_id",
            "expected_company_epoch",
            "object_type_id",
            "action_type_id",
            "expected_action_revision",
            "expected_schema_revision",
            "legal_name_property_id",
            "employee_number_property_id",
            "legal_name",
            "employee_number",
        ],
        _ => &["csrf_proof", "command_id"],
    };
    let mut fields = BTreeMap::new();
    for pair in body.split(|byte| *byte == b'&') {
        let split = pair
            .iter()
            .position(|byte| *byte == b'=')
            .ok_or(StatusCode::BAD_REQUEST)?;
        // Field names are fixed ASCII emitted by our forms, with no aliases.
        let key = std::str::from_utf8(&pair[..split]).map_err(|_| StatusCode::BAD_REQUEST)?;
        if !allowed.contains(&key) || fields.contains_key(key) {
            return Err(StatusCode::BAD_REQUEST);
        }
        fields.insert(key, decode(&pair[split + 1..])?);
    }
    if fields.len() != allowed.len() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let field = |key| {
        fields
            .get(key)
            .map(String::as_str)
            .ok_or(StatusCode::BAD_REQUEST)
    };
    let proof = field("csrf_proof")?.to_owned();
    if proof.is_empty() || proof.len() > 4096 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let uuid = |key| id(field(key)?).map_err(|_| StatusCode::BAD_REQUEST);
    let number = |key| -> Result<u64, StatusCode> {
        let raw = field(key)?;
        let value = raw.parse::<u64>().map_err(|_| StatusCode::BAD_REQUEST)?;
        if value == 0 || value.to_string() != raw {
            return Err(StatusCode::BAD_REQUEST);
        }
        Ok(value)
    };
    let command = uuid("command_id")?;
    let locator =
        DirectoryRequestRef::new(company, command).map_err(|_| StatusCode::BAD_REQUEST)?;
    let input = match target {
        PostTarget::Prepare => {
            let expected = DirectoryExpectationsV1 {
                company_epoch: number("expected_company_epoch")?,
                object_type_id: uuid("object_type_id")?,
                action_type_id: uuid("action_type_id")?,
                action_revision: number("expected_action_revision")?,
                schema_revision: number("expected_schema_revision")?,
                legal_name_property_id: uuid("legal_name_property_id")?,
                employee_number_property_id: uuid("employee_number_property_id")?,
            };
            let legal_name = field("legal_name")?.to_owned();
            let employee_number = field("employee_number")?.to_owned();
            // Both errors use the actual input validator, with a valid counterpart.
            let name_error = DirectoryRegistrationInput::new(&legal_name, "valid")
                .err()
                .map(|e| e.problem);
            let number_error = DirectoryRegistrationInput::new("valid", &employee_number)
                .err()
                .map(|e| e.problem);
            let submission = DirectoryRegistrationInput::new(&legal_name, &employee_number)
                .ok()
                .map(|input| DirectorySubmission::new(locator, expected, input))
                .transpose()
                .map_err(|_| StatusCode::BAD_REQUEST)?;
            Input::Prepare {
                draft: Draft {
                    locator,
                    expected,
                    legal_name,
                    employee_number,
                    name_error,
                    number_error,
                },
                submission,
            }
        }
        PostTarget::Execute(path) | PostTarget::Cancel(path) => {
            if id(path)? != command {
                return Err(StatusCode::BAD_REQUEST);
            }
            match target {
                PostTarget::Execute(_) => Input::Execute(locator),
                _ => Input::Cancel(locator),
            }
        }
    };
    Ok(Parsed { proof, input })
}
fn decode(bytes: &[u8]) -> Result<String, StatusCode> {
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => decoded.push(b' '),
            b'%' => {
                let pair = bytes
                    .get(index + 1..index + 3)
                    .ok_or(StatusCode::BAD_REQUEST)?;
                decoded.push(digit(pair[0])? * 16 + digit(pair[1])?);
                index += 2;
            }
            byte => decoded.push(byte),
        }
        index += 1;
    }
    String::from_utf8(decoded).map_err(|_| StatusCode::BAD_REQUEST)
}
fn digit(byte: u8) -> Result<u8, StatusCode> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(StatusCode::BAD_REQUEST),
    }
}

#[cfg(test)]
mod tests;
