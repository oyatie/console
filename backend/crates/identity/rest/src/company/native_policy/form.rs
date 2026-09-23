//! Bounded native HTML form grammar; authority and execution-time validation
//! remain with Auth and the owning application command.
use console_identity_application::company_policy::{
    AccountId,
    business::{
        NativeBusinessOperationV1, NativeCompanyBusinessCommandV1, PolicyAssignmentExpectationV1,
    },
    workflow::NativePolicyCommandRef,
};
use console_kernel_core::OrgId;
use std::collections::BTreeMap;
use time::{Date, Month, OffsetDateTime, UtcOffset};
use uuid::Uuid;

pub(super) const MAX_BODY_BYTES: usize = 16 * 1024;

#[derive(Clone, Copy)]
pub(super) enum Target {
    Install,
    Grant,
    Revoke {
        assignment: Uuid,
    },
    Retry {
        operation: NativeBusinessOperationV1,
        command_id: Uuid,
    },
}
pub(super) enum Input {
    Command(NativeCompanyBusinessCommandV1),
    Retry(NativePolicyCommandRef),
}
pub struct NativePolicyGrantDraft {
    pub selector: NativePolicyCommandRef,
    pub expected_company_epoch: u64,
    pub recipient_account_id: AccountId,
    pub assignment: Option<PolicyAssignmentExpectationV1>,
    pub expires_at_local: String,
}
pub(super) enum DocumentInput {
    Ready(Input),
    GrantValidation(NativePolicyGrantDraft),
}
pub(super) struct ParsedDocument {
    pub(super) proof: String,
    pub(super) input: DocumentInput,
}
// Proofs intentionally have no Debug/Serialize implementation.
#[cfg(test)]
pub(super) struct Parsed {
    pub(super) proof: String,
    pub(super) input: Input,
}
#[derive(Debug, PartialEq, Eq)]
pub(super) enum FormError {
    TooLarge,
    Invalid,
}

#[cfg(test)]
pub(super) fn parse(company: OrgId, target: Target, body: &[u8]) -> Result<Parsed, FormError> {
    let parsed = parse_document(company, target, body)?;
    match parsed.input {
        DocumentInput::Ready(input) => Ok(Parsed {
            proof: parsed.proof,
            input,
        }),
        DocumentInput::GrantValidation(_) => Err(FormError::Invalid),
    }
}
pub(super) fn parse_document(
    company: OrgId,
    target: Target,
    body: &[u8],
) -> Result<ParsedDocument, FormError> {
    use FormError::{Invalid, TooLarge};
    if body.len() > MAX_BODY_BYTES {
        return Err(TooLarge);
    }
    let allowed: &[&str] = match target {
        Target::Install => &["command_id", "expected_company_epoch", "csrf_proof"],
        Target::Grant => &[
            "command_id",
            "expected_company_epoch",
            "csrf_proof",
            "recipient_account_id",
            "expected_role_revision",
            "assignment_id",
            "expected_assignment_revision",
            "expires_at_local",
        ],
        Target::Revoke { .. } => &[
            "command_id",
            "expected_company_epoch",
            "csrf_proof",
            "expected_role_revision",
            "expected_assignment_revision",
        ],
        Target::Retry { .. } => &["csrf_proof"],
    };
    let mut fields = BTreeMap::new();
    for pair in body.split(|byte| *byte == b'&') {
        if fields.len() == allowed.len() {
            return Err(Invalid);
        }
        let separator = pair.iter().position(|byte| *byte == b'=').ok_or(Invalid)?;
        let key = decode(&pair[..separator])?;
        if !allowed.contains(&key.as_str()) || fields.contains_key(&key) {
            return Err(Invalid);
        }
        fields.insert(key, decode(&pair[separator + 1..])?);
    }
    if fields.len() != allowed.len() {
        return Err(Invalid);
    }
    let proof = fields.remove("csrf_proof").ok_or(Invalid)?;
    if proof.len() > 4096 {
        return Err(TooLarge);
    }
    if proof.is_empty() {
        return Err(Invalid);
    }
    let field = |name: &str| fields.get(name).map(String::as_str).ok_or(Invalid);
    let input = if let Target::Retry {
        operation,
        command_id,
    } = target
    {
        Input::Retry(
            NativePolicyCommandRef::new(company, command_id, operation).map_err(|_| Invalid)?,
        )
    } else {
        let command = canonical_uuid(field("command_id")?)?;
        let epoch = revision(field("expected_company_epoch")?)?;
        let command = match target {
            Target::Install => NativeCompanyBusinessCommandV1::install(command, company, epoch),
            Target::Grant => {
                let recipient =
                    AccountId::from_uuid(canonical_uuid(field("recipient_account_id")?)?)
                        .map_err(|_| Invalid)?;
                let role = field("expected_role_revision")?;
                let assignment = field("assignment_id")?;
                let current = field("expected_assignment_revision")?;
                let expected = if role.is_empty() && assignment.is_empty() && current.is_empty() {
                    None
                } else {
                    Some(PolicyAssignmentExpectationV1 {
                        role_revision: revision(role)?,
                        assignment_id: canonical_uuid(assignment)?,
                        assignment_revision: revision(current)?,
                    })
                };
                let selector =
                    NativePolicyCommandRef::new(company, command, NativeBusinessOperationV1::Grant)
                        .map_err(|_| Invalid)?;
                if expected.is_some_and(|a| a.role_revision != 1) {
                    return Err(Invalid);
                }
                let expiry = field("expires_at_local")?;
                if expiry.len() > 64 {
                    return Err(TooLarge);
                }
                if expiry.chars().any(char::is_control) {
                    return Err(Invalid);
                }
                // All non-expiry invariants were checked above. The constructor
                // also bounds the exact historical timestamp codec.
                let result = korean_instant(expiry).and_then(|at| {
                    NativeCompanyBusinessCommandV1::grant(
                        command, company, epoch, recipient, expected, at,
                    )
                    .map_err(|_| Invalid)
                });
                match result {
                    Ok(command) => Ok(command),
                    Err(_) => {
                        return Ok(ParsedDocument {
                            input: DocumentInput::GrantValidation(NativePolicyGrantDraft {
                                selector,
                                expected_company_epoch: epoch,
                                recipient_account_id: recipient,
                                assignment: expected,
                                expires_at_local: expiry.to_owned(),
                            }),
                            proof,
                        });
                    }
                }
            }
            Target::Revoke { assignment } => NativeCompanyBusinessCommandV1::revoke(
                command,
                company,
                epoch,
                PolicyAssignmentExpectationV1 {
                    role_revision: revision(field("expected_role_revision")?)?,
                    assignment_id: assignment,
                    assignment_revision: revision(field("expected_assignment_revision")?)?,
                },
            ),
            Target::Retry { .. } => return Err(Invalid),
        }
        .map_err(|_| Invalid)?;
        Input::Command(command)
    };
    Ok(ParsedDocument {
        proof,
        input: DocumentInput::Ready(input),
    })
}

fn canonical_uuid(value: &str) -> Result<Uuid, FormError> {
    super::super::command_id(value).map_err(|_| FormError::Invalid)
}
fn revision(value: &str) -> Result<u64, FormError> {
    if value.is_empty()
        || value.len() > 19
        || value.starts_with('0')
        || !value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(FormError::Invalid);
    }
    let number: u64 = value.parse().map_err(|_| FormError::Invalid)?;
    if number > i64::MAX as u64 {
        return Err(FormError::Invalid);
    }
    Ok(number)
}
fn korean_instant(value: &str) -> Result<OffsetDateTime, FormError> {
    use FormError::Invalid;
    let bytes = value.as_bytes();
    if bytes.len() != 16
        || !bytes.is_ascii()
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes
            .iter()
            .enumerate()
            .any(|(index, byte)| ![4, 7, 10, 13].contains(&index) && !byte.is_ascii_digit())
    {
        return Err(Invalid);
    }
    let year: i32 = value[..4].parse().map_err(|_| Invalid)?;
    if year == 0 {
        return Err(Invalid);
    }
    let month =
        Month::try_from(value[5..7].parse::<u8>().map_err(|_| Invalid)?).map_err(|_| Invalid)?;
    let day: u8 = value[8..10].parse().map_err(|_| Invalid)?;
    let hour: u8 = value[11..13].parse().map_err(|_| Invalid)?;
    let minute: u8 = value[14..16].parse().map_err(|_| Invalid)?;
    let local = Date::from_calendar_date(year, month, day)
        .map_err(|_| Invalid)?
        .with_hms(hour, minute, 0)
        .map_err(|_| Invalid)?;
    Ok(local
        .assume_offset(UtcOffset::from_hms(9, 0, 0).map_err(|_| Invalid)?)
        .to_offset(UtcOffset::UTC))
}
fn decode(bytes: &[u8]) -> Result<String, FormError> {
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => decoded.push(b' '),
            b'%' => {
                let pair = bytes.get(index + 1..index + 3).ok_or(FormError::Invalid)?;
                decoded.push(digit(pair[0])? * 16 + digit(pair[1])?);
                index += 2;
            }
            byte => decoded.push(byte),
        }
        index += 1;
    }
    String::from_utf8(decoded).map_err(|_| FormError::Invalid)
}
fn digit(byte: u8) -> Result<u8, FormError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(FormError::Invalid),
    }
}

#[cfg(test)]
mod tests;
