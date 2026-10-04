//! Closed owner-jsonb ABI. These helpers never parse HTTP or confer authority.
use console_identity_application::group_process::*;
use serde_json::Value;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use uuid::Uuid;

pub(super) type Error = GroupProcessError;
pub(super) type HashCheck = (Vec<u8>, [u8; 32]);

pub(super) const MATERIAL: &str = "SELECT public.identity_native_group_process_material_v1($1::uuid,$2::uuid,$3::uuid,$4::uuid,$5::uuid,$6::smallint,$7::bytea)::text";
pub(super) const SELECTOR: &str = "SELECT public.identity_native_group_process_incarnation_selector_v1($1::uuid,$2::uuid,$3::uuid,$4::uuid)";
pub(super) const NAVIGATION: &str = "SELECT public.identity_native_group_process_navigation_candidates_v1($1::uuid,$2::uuid,$3::bytea)::text";
pub(super) const PREPARE: &str = "SELECT q.inserted, to_jsonb(q.accepted_input)::text AS accepted, CASE WHEN (q.terminal).actor_account_id IS NOT NULL THEN to_jsonb(q.terminal)::text END AS terminal FROM public.identity_native_group_process_prepare_v1($1::uuid,$2::uuid,$3::uuid,$4::bytea) q LIMIT 2";
pub(super) const EXECUTE: &str = "SELECT q.inserted, to_jsonb(q.accepted_input)::text AS accepted, to_jsonb(q.terminal)::text AS terminal FROM public.identity_native_group_process_execute_v1($1::uuid,$2::uuid,$3::uuid) q LIMIT 2";

pub(super) fn document(raw: &str, max: usize) -> Result<Value, Error> {
    if raw.is_empty() || raw.len() > max {
        return Err(Error::Unavailable);
    }
    serde_json::from_str(raw).map_err(|_| Error::Unavailable)
}
pub(super) fn keys(value: &Value, expected: &[&str]) -> Result<(), Error> {
    let object = value.as_object().ok_or(Error::Unavailable)?;
    if object.len() != expected.len() || expected.iter().any(|key| !object.contains_key(*key)) {
        return Err(Error::Unavailable);
    }
    Ok(())
}
pub(super) fn field<'a>(value: &'a Value, key: &str) -> Result<&'a Value, Error> {
    value
        .as_object()
        .and_then(|o| o.get(key))
        .ok_or(Error::Unavailable)
}
pub(super) fn string<'a>(value: &'a Value, key: &str) -> Result<&'a str, Error> {
    field(value, key)?.as_str().ok_or(Error::Unavailable)
}
pub(super) fn integer(value: &Value, key: &str) -> Result<i64, Error> {
    field(value, key)?.as_i64().ok_or(Error::Unavailable)
}
pub(super) fn positive(value: &Value, key: &str) -> Result<u64, Error> {
    let n = integer(value, key)?;
    if n <= 0 {
        return Err(Error::Unavailable);
    }
    u64::try_from(n).map_err(|_| Error::Unavailable)
}
pub(super) fn u32_value(value: &Value, key: &str) -> Result<u32, Error> {
    u32::try_from(positive(value, key)?).map_err(|_| Error::Unavailable)
}
pub(super) fn uuid(value: &Value, key: &str) -> Result<Uuid, Error> {
    canonical_group_process_uuid(string(value, key)?).map_err(|_| Error::Unavailable)
}
pub(super) fn account(value: &Value, key: &str) -> Result<AccountId, Error> {
    AccountId::from_uuid(uuid(value, key)?).map_err(|_| Error::Unavailable)
}
pub(super) fn group(value: &Value, key: &str) -> Result<GroupId, Error> {
    GroupId::from_uuid(uuid(value, key)?).map_err(|_| Error::Unavailable)
}
pub(super) fn incarnation(value: &Value, key: &str) -> Result<GroupIncarnation, Error> {
    GroupIncarnation::from_uuid(uuid(value, key)?).map_err(|_| Error::Unavailable)
}
pub(super) fn decimal(raw: &str) -> Result<u64, Error> {
    if raw.is_empty()
        || raw.len() > 20
        || raw.starts_with('0')
        || !raw.bytes().all(|b| b.is_ascii_digit())
    {
        return Err(Error::Unavailable);
    }
    raw.parse().map_err(|_| Error::Unavailable)
}
pub(super) fn bytes(value: &Value, key: &str, min: usize, max: usize) -> Result<Vec<u8>, Error> {
    let raw = string(value, key)?
        .strip_prefix("\\x")
        .ok_or(Error::Unavailable)?;
    if raw.len() % 2 != 0 || raw.len() / 2 < min || raw.len() / 2 > max {
        return Err(Error::Unavailable);
    }
    fn nibble(b: u8) -> Result<u8, Error> {
        match b {
            b'0'..=b'9' => Ok(b - b'0'),
            b'a'..=b'f' => Ok(b - b'a' + 10),
            _ => Err(Error::Unavailable),
        }
    }
    raw.as_bytes()
        .chunks_exact(2)
        .map(|pair| Ok(nibble(pair[0])? * 16 + nibble(pair[1])?))
        .collect()
}
pub(super) fn digest(value: &Value, key: &str) -> Result<[u8; 32], Error> {
    bytes(value, key, 32, 32)?
        .try_into()
        .map_err(|_| Error::Unavailable)
}
pub(super) fn timestamp(value: &Value, key: &str) -> Result<i64, Error> {
    let raw = string(value, key)?;
    if raw.len() > 40 {
        return Err(Error::Unavailable);
    }
    exact_time_us(OffsetDateTime::parse(raw, &Rfc3339).map_err(|_| Error::Unavailable)?)
}
pub(super) fn bundle(value: &Value) -> Result<EvaluatedPolicyBundleV1, Error> {
    EvaluatedPolicyBundleV1::from_projection(EvaluatedPolicyBundleProjectionV1 {
        schema_id: string(value, "schema_id")?.to_owned(),
        schema_digest: digest(value, "schema_digest")?,
        policy_digest: digest(value, "policy_digest")?,
        codec_contract_digest: digest(value, "codec_contract_digest")?,
        registration_manifest_version: positive(value, "registration_manifest_version")?,
        registration_manifest_digest: digest(value, "registration_manifest_digest")?,
        cedar_sdk_version: string(value, "cedar_sdk_version")?.to_owned(),
        cedar_language_version: string(value, "cedar_language_version")?.to_owned(),
    })
}
const SOURCE_KEYS: &[&str] = &[
    "schema_id",
    "schema_digest",
    "policy_digest",
    "codec_contract_digest",
    "registration_manifest_version",
    "registration_manifest_digest",
    "cedar_sdk_version",
    "cedar_language_version",
    "registered_actions",
];
pub(super) fn source(
    value: &Value,
) -> Result<
    (
        EvaluatedPolicyBundleV1,
        Vec<GroupProcessRegistrationProjectionV1>,
    ),
    Error,
> {
    keys(value, SOURCE_KEYS)?;
    let rows = field(value, "registered_actions")?
        .as_array()
        .ok_or(Error::Unavailable)?;
    if rows.len() != 4 {
        return Err(Error::Unavailable);
    }
    let registrations = rows
        .iter()
        .map(|row| {
            keys(row, &["key", "action_id", "revision", "fields"])?;
            let fields = field(row, "fields")?.as_array().ok_or(Error::Unavailable)?;
            if fields.len() > 32 {
                return Err(Error::Unavailable);
            }
            Ok(GroupProcessRegistrationProjectionV1 {
                action: GroupProcessActionV1::from_str(string(row, "key")?)?,
                action_id: uuid(row, "action_id")?,
                revision: positive(row, "revision")?,
                fields: fields
                    .iter()
                    .map(|v| GroupProcessFieldV1::from_str(v.as_str().ok_or(Error::Unavailable)?))
                    .collect::<Result<_, _>>()?,
            })
        })
        .collect::<Result<_, _>>()?;
    Ok((bundle(value)?, registrations))
}
pub(super) fn policy_reference(
    value: &Value,
    prefix: &str,
) -> Result<PolicyHeadReferenceV1, Error> {
    let revision_key = format!("{prefix}revision");
    let digest_key = format!("{prefix}head_digest");
    match integer(value, &format!("{prefix}tag"))? {
        0 if field(value, &revision_key)?.is_null() && field(value, &digest_key)?.is_null() => {
            Ok(PolicyHeadReferenceV1::Absent)
        }
        1 => Ok(PolicyHeadReferenceV1::Installed {
            revision: PositivePolicyRevisionV1::new(positive(value, &revision_key)?)?,
            head_digest: digest(value, &digest_key)?,
        }),
        _ => Err(Error::Unavailable),
    }
}
pub(super) fn head_reference(value: &Value, prefix: &str) -> Result<ProcessHeadReferenceV1, Error> {
    let state = match string(value, &format!("{prefix}state"))? {
        "ACTIVE" => ProcessStateV1::Active,
        "SUSPENDED" => ProcessStateV1::Suspended,
        _ => return Err(Error::Unavailable),
    };
    ProcessHeadReferenceV1::from_projection(ProcessHeadReferenceProjectionV1 {
        process_id: uuid(value, &format!("{prefix}process_id"))?,
        head_revision: positive(value, &format!("{prefix}head_revision"))?,
        content_version: positive(value, &format!("{prefix}content_version"))?,
        content_digest: digest(value, &format!("{prefix}content_digest"))?,
        head_digest: digest(value, &format!("{prefix}head_digest"))?,
        state,
        expiry_us: timestamp(value, &format!("{prefix}expires_at"))?,
    })
}
fn optional_head(value: &Value, prefix: &str) -> Result<Option<ProcessHeadReferenceV1>, Error> {
    let names = [
        "process_id",
        "head_revision",
        "content_version",
        "content_digest",
        "head_digest",
        "state",
        "expires_at",
    ];
    let null_count = names
        .iter()
        .map(|name| Ok(field(value, &format!("{prefix}{name}"))?.is_null() as usize))
        .collect::<Result<Vec<_>, Error>>()?
        .iter()
        .sum::<usize>();
    match null_count {
        7 => Ok(None),
        0 => Ok(Some(head_reference(value, prefix)?)),
        _ => Err(Error::Unavailable),
    }
}
const INPUT_KEYS: &[&str] = &[
    "actor_account_id",
    "command_id",
    "group_id",
    "group_incarnation",
    "operation",
    "codec_version",
    "input_bytes",
    "input_digest",
    "intake_receipt_id",
    "accepted_at",
    "accepted_session_id",
    "account_security_generation",
    "designation_receipt_id",
    "designation_revision",
    "observed_group_revision",
    "accepted_policy_tag",
    "accepted_policy_revision",
    "accepted_policy_head_digest",
    "schema_id",
    "schema_digest",
    "policy_digest",
    "codec_contract_digest",
    "registration_manifest_version",
    "registration_manifest_digest",
    "cedar_sdk_version",
    "cedar_language_version",
    "acceptance_xid",
    "acceptance_backend_pid",
    "audit_id",
];
pub(super) fn accepted(
    value: &Value,
    checks: &mut Vec<HashCheck>,
) -> Result<GroupProcessAcceptedV1, Error> {
    keys(value, INPUT_KEYS)?;
    if integer(value, "codec_version")? != 1 {
        return Err(Error::Unavailable);
    }
    let input_bytes = bytes(value, "input_bytes", 188, GROUP_PROCESS_ADOPT_MAX_BYTES)?;
    let input_digest = digest(value, "input_digest")?;
    let accepted = GroupProcessAcceptedV1::checked(
        account(value, "actor_account_id")?,
        input_bytes.clone(),
        input_digest,
        uuid(value, "intake_receipt_id")?,
        timestamp(value, "accepted_at")?,
    )?;
    if accepted.input.command_id() != uuid(value, "command_id")?
        || accepted.input.group() != group(value, "group_id")?
        || accepted.input.incarnation() != incarnation(value, "group_incarnation")?
        || i64::from(accepted.input.opcode()) != integer(value, "operation")?
    {
        return Err(Error::Unavailable);
    }
    uuid(value, "accepted_session_id")?;
    positive(value, "account_security_generation")?;
    uuid(value, "designation_receipt_id")?;
    positive(value, "designation_revision")?;
    positive(value, "observed_group_revision")?;
    policy_reference(value, "accepted_policy_")?;
    bundle(value)?;
    decimal(string(value, "acceptance_xid")?)?;
    u32_value(value, "acceptance_backend_pid")?;
    uuid(value, "audit_id")?;
    checks.push((input_bytes, input_digest));
    Ok(accepted)
}
const RESULT_KEYS: &[&str] = &[
    "actor_account_id",
    "command_id",
    "group_id",
    "group_incarnation",
    "operation",
    "input_digest",
    "intake_receipt_id",
    "effect_id",
    "result_receipt_id",
    "terminal_code",
    "accepted_at",
    "executed_at",
    "execution_session_id",
    "account_security_generation",
    "designation_receipt_id",
    "designation_revision",
    "observed_group_revision",
    "policy_before_tag",
    "policy_before_revision",
    "policy_before_head_digest",
    "policy_after_tag",
    "policy_after_revision",
    "policy_after_head_digest",
    "schema_id",
    "schema_digest",
    "policy_digest",
    "codec_contract_digest",
    "registration_manifest_version",
    "registration_manifest_digest",
    "cedar_sdk_version",
    "cedar_language_version",
    "requested_process_id",
    "before_process_id",
    "before_head_revision",
    "before_content_version",
    "before_content_digest",
    "before_head_digest",
    "before_state",
    "before_expires_at",
    "after_process_id",
    "after_head_revision",
    "after_content_version",
    "after_content_digest",
    "after_head_digest",
    "after_state",
    "after_expires_at",
    "effect_xid",
    "effect_backend_pid",
    "effect_census",
    "layout_version",
    "result_bytes",
    "result_digest",
    "audit_id",
];
pub(super) fn terminal(
    value: &Value,
    accepted: GroupProcessAcceptedV1,
    checks: &mut Vec<HashCheck>,
) -> Result<GroupProcessTerminalV2, Error> {
    keys(value, RESULT_KEYS)?;
    if integer(value, "layout_version")? != 2 {
        return Err(Error::Unavailable);
    }
    let code = match string(value, "terminal_code")? {
        "ADOPTED" => ProcessTerminalCodeV1::Adopted,
        "REPLACED" => ProcessTerminalCodeV1::Replaced,
        "SUSPENDED" => ProcessTerminalCodeV1::Suspended,
        "REJECTED_STALE_EXPECTATION" => ProcessTerminalCodeV1::RejectedStaleExpectation,
        "REJECTED_PROCESS_EXPIRED" => ProcessTerminalCodeV1::RejectedProcessExpired,
        "REJECTED_ALREADY_SUSPENDED" => ProcessTerminalCodeV1::RejectedAlreadySuspended,
        _ => return Err(Error::Unavailable),
    };
    let result = ProcessResultV2::from_projection(ProcessResultProjectionV2 {
        actor: account(value, "actor_account_id")?,
        command: uuid(value, "command_id")?,
        group: group(value, "group_id")?,
        incarnation: incarnation(value, "group_incarnation")?,
        opcode: u16::try_from(integer(value, "operation")?).map_err(|_| Error::Unavailable)?,
        input_digest: digest(value, "input_digest")?,
        intake_receipt: uuid(value, "intake_receipt_id")?,
        result_receipt: uuid(value, "result_receipt_id")?,
        terminal_code: code,
        accepted_at_us: timestamp(value, "accepted_at")?,
        executed_at_us: timestamp(value, "executed_at")?,
        session: uuid(value, "execution_session_id")?,
        security_generation: positive(value, "account_security_generation")?,
        designation_receipt: uuid(value, "designation_receipt_id")?,
        designation_revision: positive(value, "designation_revision")?,
        observed_group_revision: positive(value, "observed_group_revision")?,
        policy_before: policy_reference(value, "policy_before_")?,
        policy_after: policy_reference(value, "policy_after_")?,
        evaluated_bundle: bundle(value)?,
        requested_process_id: uuid(value, "requested_process_id")?,
        before_head: optional_head(value, "before_")?,
        after_head: optional_head(value, "after_")?,
        effect_xid8: decimal(string(value, "effect_xid")?)?,
        effect_pid: u32_value(value, "effect_backend_pid")?,
    })?;
    let result_bytes = bytes(value, "result_bytes", 300, 2048)?;
    let result_digest = digest(value, "result_digest")?;
    if result.encode() != result_bytes {
        return Err(Error::Unavailable);
    }
    uuid(value, "effect_id")?;
    uuid(value, "audit_id")?;
    let expected = serde_json::json!({"policy_heads":i32::from(code==ProcessTerminalCodeV1::Adopted),
        "versions":i32::from(matches!(code,ProcessTerminalCodeV1::Adopted|ProcessTerminalCodeV1::Replaced)),
        "heads":i32::from(code.is_success()),"head_revisions":i32::from(code.is_success()),"effects":1,"results":1,"audits":1});
    if field(value, "effect_census")? != &expected {
        return Err(Error::Unavailable);
    }
    checks.push((result_bytes.clone(), result_digest));
    GroupProcessTerminalV2::checked(accepted, result_bytes, result_digest)
}
pub(super) fn status(
    value: &Value,
    checks: &mut Vec<HashCheck>,
) -> Result<Option<GroupProcessStatus>, Error> {
    if value.is_null() {
        return Ok(None);
    }
    keys(value, &["accepted", "terminal"])?;
    let accepted = accepted(field(value, "accepted")?, checks)?;
    Ok(Some(if field(value, "terminal")?.is_null() {
        GroupProcessStatus::AcceptedPending(accepted)
    } else {
        GroupProcessStatus::Terminal(terminal(field(value, "terminal")?, accepted, checks)?)
    }))
}
