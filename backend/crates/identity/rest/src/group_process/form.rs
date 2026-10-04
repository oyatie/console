//! Strict HTML transport. Selectors never establish authority; new content
//! normalizes CRLF while historical command bytes retain their existing codec.
use console_identity_application::group_process::*;
use std::collections::BTreeMap;
use time::{Date, Month, OffsetDateTime, UtcOffset};
use uuid::Uuid;

#[derive(Clone, Copy)]
pub(super) enum Target {
    Adopt,
    Suspend {
        process: Uuid,
    },
    Retry {
        requested: GroupProcessRouteSelectorV1,
    },
}
impl Target {
    pub(super) const fn max_body(self) -> usize {
        match self {
            Self::Adopt => 131_072,
            Self::Suspend { .. } | Self::Retry { .. } => 32_768,
        }
    }
}
pub(super) struct Parsed {
    pub(super) proof: String,
    pub(super) input: Input,
}
pub(super) enum Input {
    Command(GroupProcessCommandV1),
    Validation(Draft),
    Retry(GroupProcessRouteSelectorV1),
}
pub struct Draft {
    pub original: GroupProcessLocator,
    pub process_id: Uuid,
    pub expected_group_revision: u64,
    pub expected_policy_revision: u64,
    pub input: DraftInput,
    pub errors: Vec<&'static str>,
}
pub enum DraftInput {
    Adopt {
        expected_prior_head_revision: u64,
        expiry: String,
        content: RawContent,
    },
    Suspend {
        content_version: u64,
        content_digest: [u8; 32],
        expected_head_revision: u64,
        expected_head_digest: [u8; 32],
        reason: String,
    },
}
pub struct RawContent {
    pub title: String,
    pub method: String,
    pub prose: [String; 8],
}
#[derive(Debug, PartialEq, Eq)]
pub(super) enum FormError {
    Invalid,
    TooLarge,
}
const PROSE: [&str; 8] = [
    "intended_claimant_matching_procedure",
    "account_possession_procedure",
    "physical_human_evidence_procedure",
    "duplicate_contradictory_claim_procedure",
    "qualification_criteria_instruction",
    "escalation_adjudication_procedure",
    "evidence_minimization_retention_description",
    "recipient_responsibility",
];

pub(super) fn parse(group: GroupId, target: Target, body: &[u8]) -> Result<Parsed, FormError> {
    use FormError::{Invalid, TooLarge};
    if body.len() > target.max_body() {
        return Err(TooLarge);
    }
    let mut allowed = vec!["csrf_proof"];
    if !matches!(target, Target::Retry { .. }) {
        allowed.extend([
            "command_id",
            "expected_group_revision",
            "expected_group_incarnation",
            "expected_group_identity_policy_revision",
        ]);
        match target {
            Target::Adopt => {
                allowed.extend([
                    "process_id",
                    "expected_prior_process_revision",
                    "process_expiry",
                    "operator_responsibility",
                    "title",
                    "method",
                ]);
                allowed.extend(PROSE);
            }
            Target::Suspend { .. } => allowed.extend([
                "process_version",
                "process_digest",
                "expected_process_head_revision",
                "expected_process_head_digest",
                "reason",
            ]),
            Target::Retry { .. } => return Err(Invalid),
        }
    }
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
    if let Target::Retry { requested } = target {
        if requested.group() != group {
            return Err(Invalid);
        }
        return Ok(Parsed {
            proof,
            input: Input::Retry(requested),
        });
    }
    let field = |key: &str| fields.get(key).map(String::as_str).ok_or(Invalid);
    let command = canonical_group_process_uuid(field("command_id")?).map_err(|_| Invalid)?;
    let incarnation = GroupIncarnation::from_uuid(
        canonical_group_process_uuid(field("expected_group_incarnation")?).map_err(|_| Invalid)?,
    )
    .map_err(|_| Invalid)?;
    let original = GroupProcessLocator::new(group, incarnation, command).map_err(|_| Invalid)?;
    let expected_group_revision = revision(field("expected_group_revision")?, false)?;
    let expected_policy_revision = revision(
        field("expected_group_identity_policy_revision")?,
        matches!(target, Target::Adopt),
    )?;
    let mut errors = Vec::new();
    let (process_id, draft_input, ready) = match target {
        Target::Adopt => {
            let process =
                canonical_group_process_uuid(field("process_id")?).map_err(|_| Invalid)?;
            let prior = revision(field("expected_prior_process_revision")?, true)?;
            if field("operator_responsibility")? != "1" {
                return Err(Invalid);
            }
            let responsibility =
                AcceptedOperatorResponsibilityV1::from_form("yes").map_err(|_| Invalid)?;
            let raw = RawContent {
                title: field("title")?.to_owned(),
                method: field("method")?.to_owned(),
                prose: PROSE.map(|key| fields[key].clone()),
            };
            let expiry = field("process_expiry")?.to_owned();
            if raw.title.len() > 240
                || raw.prose.iter().any(|v| v.len() > 4096)
                || raw.title.len()
                    + raw.method.len()
                    + raw.prose.iter().map(String::len).sum::<usize>()
                    > 32_768
                || expiry.len() > 64
            {
                return Err(TooLarge);
            }
            let title = raw.title.replace("\r\n", "\n");
            let prose = raw
                .prose
                .each_ref()
                .map(|value| value.replace("\r\n", "\n"));
            if !valid_text(&title, 120) {
                errors.push("title");
            }
            for (key, value) in PROSE.iter().zip(&prose) {
                if !valid_text(value, 2048) {
                    errors.push(*key);
                }
            }
            let method = ProcessMethodV1::from_str(&raw.method);
            if method.is_err() {
                errors.push("method");
            }
            if title.len() + raw.method.len() + prose.iter().map(String::len).sum::<usize>()
                > 16_384
            {
                errors.push("content");
            }
            let at = korean_instant(&expiry);
            if at.is_err() {
                errors.push("process_expiry");
            }
            let ready = if errors.is_empty() {
                let [
                    claimant,
                    account,
                    human,
                    duplicate,
                    qualification,
                    escalation,
                    retention,
                    recipient,
                ] = prose;
                let content = ProcessContentV1::from_projection(ProcessContentProjectionV1 {
                    title,
                    method: method.map_err(|_| Invalid)?,
                    intended_claimant_matching_procedure: claimant,
                    account_possession_procedure: account,
                    physical_human_evidence_procedure: human,
                    duplicate_contradictory_claim_procedure: duplicate,
                    qualification_criteria_instruction: qualification,
                    escalation_adjudication_procedure: escalation,
                    evidence_minimization_retention_description: retention,
                    recipient_responsibility: recipient,
                })
                .map_err(|_| Invalid)?;
                Some(
                    GroupProcessCommandV1::adopt(
                        command,
                        group,
                        incarnation,
                        expected_group_revision,
                        expected_policy_revision,
                        process,
                        prior,
                        at.map_err(|_| Invalid)?,
                        responsibility,
                        content,
                    )
                    .map_err(|_| Invalid)?,
                )
            } else {
                None
            };
            (
                process,
                DraftInput::Adopt {
                    expected_prior_head_revision: prior,
                    expiry,
                    content: raw,
                },
                ready,
            )
        }
        Target::Suspend { process } => {
            if process.is_nil() {
                return Err(Invalid);
            }
            let version = revision(field("process_version")?, false)?;
            let content_digest = digest(field("process_digest")?)?;
            let head_revision = revision(field("expected_process_head_revision")?, false)?;
            let head_digest = digest(field("expected_process_head_digest")?)?;
            let reason = field("reason")?.to_owned();
            if reason.len() > 4096 {
                return Err(TooLarge);
            }
            let normalized = reason.replace("\r\n", "\n");
            let ready = if valid_text(&normalized, 2048) {
                Some(
                    GroupProcessCommandV1::suspend(
                        command,
                        group,
                        incarnation,
                        expected_group_revision,
                        expected_policy_revision,
                        process,
                        version,
                        content_digest,
                        head_revision,
                        head_digest,
                        normalized,
                    )
                    .map_err(|_| Invalid)?,
                )
            } else {
                errors.push("reason");
                None
            };
            (
                process,
                DraftInput::Suspend {
                    content_version: version,
                    content_digest,
                    expected_head_revision: head_revision,
                    expected_head_digest: head_digest,
                    reason,
                },
                ready,
            )
        }
        Target::Retry { .. } => return Err(Invalid),
    };
    Ok(Parsed {
        proof,
        input: match ready {
            Some(command) => Input::Command(command),
            None => Input::Validation(Draft {
                original,
                process_id,
                expected_group_revision,
                expected_policy_revision,
                input: draft_input,
                errors,
            }),
        },
    })
}

fn valid_text(value: &str, max: usize) -> bool {
    !value.trim().is_empty()
        && value.len() <= max
        && !value
            .chars()
            .any(|c| c.is_control() && c != '\n' && c != '\t')
}
fn revision(value: &str, zero: bool) -> Result<u64, FormError> {
    if value.is_empty()
        || value.len() > 19
        || !value.bytes().all(|b| b.is_ascii_digit())
        || (value.starts_with('0') && !(zero && value == "0"))
    {
        return Err(FormError::Invalid);
    }
    let value: u64 = value.parse().map_err(|_| FormError::Invalid)?;
    if value > i64::MAX as u64 {
        return Err(FormError::Invalid);
    }
    Ok(value)
}
fn digest(value: &str) -> Result<[u8; 32], FormError> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(FormError::Invalid);
    }
    let mut digest = [0; 32];
    for (result, bytes) in digest.iter_mut().zip(value.as_bytes().chunks_exact(2)) {
        *result = digit(bytes[0])? * 16 + digit(bytes[1])?;
    }
    Ok(digest)
}
fn korean_instant(value: &str) -> Result<OffsetDateTime, FormError> {
    use FormError::Invalid;
    let bytes = value.as_bytes();
    if bytes.len() != 19
        || !bytes.is_ascii()
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || bytes
            .iter()
            .enumerate()
            .any(|(i, b)| ![4, 7, 10, 13, 16].contains(&i) && !b.is_ascii_digit())
    {
        return Err(Invalid);
    }
    let year: i32 = value[..4].parse().map_err(|_| Invalid)?;
    if year == 0 {
        return Err(Invalid);
    }
    let number = |from, to| value[from..to].parse::<u8>().map_err(|_| Invalid);
    let month = Month::try_from(number(5, 7)?).map_err(|_| Invalid)?;
    let at = Date::from_calendar_date(year, month, number(8, 10)?)
        .map_err(|_| Invalid)?
        .with_hms(number(11, 13)?, number(14, 16)?, number(17, 19)?)
        .map_err(|_| Invalid)?;
    Ok(at
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
