//! Exact owner projections and checked immutable receipt decoding.
use super::*;
use sqlx::{Column, ValueRef, postgres::PgRow};

pub(super) const MATERIAL27: &str = r#"SELECT m.actor_account_id,
 m.session_id,
 m.account_security_generation,
 m.designation_system_identifier,
 m.designation_database_name,
 m.designation_database_oid,
 m.designation_revision,
 m.designation_receipt_id,
 m.org_id,
 m.current_group_id,
 m.group_revision,
 m.group_incarnation,
 m.membership_id,
 m.membership_revision,
 m.membership_incarnation,
 m.company_epoch,
 m.current_policy_receipt_id,
 m.origin_account_id,
 m.origin_command_id,
 m.origin_receipt_id,
 m.administrative_account_id,
 m.company_actor_admission_receipt_id,
 m.birth_assignment_id,
 m.birth_role_id,
 m.observed_at,
 m.source_xid::text AS source_xid,
 m.source_backend_pid
FROM public.identity_native_policy_material_v1($1::uuid,$2::uuid,$3::uuid,$4::uuid,$5::smallint,$6::bytea) AS m LIMIT 2"#;

pub(super) const OPERATOR8: &str = r#"SELECT installed_object_type_id,role_id,role_revision,assignment_id,assignment_revision,assignment_state,assignment_valid_from,assignment_valid_until FROM public.native_company_policy_form_v1($1::uuid,$2::uuid,$3::uuid,$4::uuid,$5::smallint) LIMIT 2"#;

pub(super) const OPERATOR8_PEOPLE: &str = r#"SELECT installed_object_type_id,role_id,role_revision,assignment_id,assignment_revision,assignment_state,assignment_valid_from,assignment_valid_until FROM public.native_company_policy_form_v2($1::uuid,$2::uuid,$3::uuid,$4::uuid,$5::smallint,$6::smallint,$7::text) LIMIT 2"#;

pub(super) const PREPARE: &str = r#"SELECT q.inserted,
 (q.accepted_input).actor_account_id AS input_actor_account_id,
 (q.accepted_input).command_id AS input_command_id,
 (q.accepted_input).org_id AS input_org_id,
 (q.accepted_input).operation AS input_operation,
 (q.accepted_input).codec_version AS input_codec_version,
 (q.accepted_input).input_bytes AS input_input_bytes,
 (q.accepted_input).input_digest AS input_input_digest,
 (q.accepted_input).intake_receipt_id AS input_intake_receipt_id,
 (q.accepted_input).accepted_at AS input_accepted_at,
 (q.accepted_input).execution_not_after AS input_execution_not_after,
 (q.accepted_input).accepting_session_id AS input_accepting_session_id,
 (q.accepted_input).acceptance_xid::text AS input_acceptance_xid,
 (q.accepted_input).acceptance_backend_pid AS input_acceptance_backend_pid,
 (q.terminal).actor_account_id AS terminal_actor_account_id,
 (q.terminal).command_id AS terminal_command_id,
 (q.terminal).org_id AS terminal_org_id,
 (q.terminal).operation AS terminal_operation,
 (q.terminal).codec_version AS terminal_codec_version,
 (q.terminal).intake_receipt_id AS terminal_intake_receipt_id,
 (q.terminal).input_digest AS terminal_input_digest,
 (q.terminal).receipt_id AS terminal_receipt_id,
 (q.terminal).outcome AS terminal_outcome,
 (q.terminal).result_code AS terminal_result_code,
 (q.terminal).execution_session_id AS terminal_execution_session_id,
 (q.terminal).executed_at AS terminal_executed_at,
 (q.terminal).effect_xid::text AS terminal_effect_xid,
 (q.terminal).effect_backend_pid AS terminal_effect_backend_pid,
 (q.terminal).epoch_before AS terminal_epoch_before,
 (q.terminal).epoch_after AS terminal_epoch_after,
 (q.terminal).catalog_version AS terminal_catalog_version,
 (q.terminal).manifest_digest AS terminal_manifest_digest,
 (q.terminal).predecessor_receipt_id AS terminal_predecessor_receipt_id,
 (q.terminal).committed_epoch AS terminal_committed_epoch,
 (q.terminal).installed_object_type_id AS terminal_installed_object_type_id,
 (q.terminal).recipient_account_id AS terminal_recipient_account_id,
 (q.terminal).role_id AS terminal_role_id,
 (q.terminal).role_revision AS terminal_role_revision,
 (q.terminal).assignment_id AS terminal_assignment_id,
 (q.terminal).assignment_revision_before AS terminal_assignment_revision_before,
 (q.terminal).assignment_revision_after AS terminal_assignment_revision_after,
 (q.terminal).assignment_state_after AS terminal_assignment_state_after,
 (q.terminal).assignment_valid_from AS terminal_assignment_valid_from,
 (q.terminal).assignment_valid_until AS terminal_assignment_valid_until
FROM public.native_company_policy_prepare_v1($1::uuid,$2::uuid,$3::uuid,$4::uuid,$5::smallint,$6::bytea) AS q LIMIT 2"#;

pub(super) const STATUS: &str = r#"SELECT q.state,
 (q.accepted_input).actor_account_id AS input_actor_account_id,
 (q.accepted_input).command_id AS input_command_id,
 (q.accepted_input).org_id AS input_org_id,
 (q.accepted_input).operation AS input_operation,
 (q.accepted_input).codec_version AS input_codec_version,
 (q.accepted_input).input_bytes AS input_input_bytes,
 (q.accepted_input).input_digest AS input_input_digest,
 (q.accepted_input).intake_receipt_id AS input_intake_receipt_id,
 (q.accepted_input).accepted_at AS input_accepted_at,
 (q.accepted_input).execution_not_after AS input_execution_not_after,
 (q.accepted_input).accepting_session_id AS input_accepting_session_id,
 (q.accepted_input).acceptance_xid::text AS input_acceptance_xid,
 (q.accepted_input).acceptance_backend_pid AS input_acceptance_backend_pid,
 (q.terminal).actor_account_id AS terminal_actor_account_id,
 (q.terminal).command_id AS terminal_command_id,
 (q.terminal).org_id AS terminal_org_id,
 (q.terminal).operation AS terminal_operation,
 (q.terminal).codec_version AS terminal_codec_version,
 (q.terminal).intake_receipt_id AS terminal_intake_receipt_id,
 (q.terminal).input_digest AS terminal_input_digest,
 (q.terminal).receipt_id AS terminal_receipt_id,
 (q.terminal).outcome AS terminal_outcome,
 (q.terminal).result_code AS terminal_result_code,
 (q.terminal).execution_session_id AS terminal_execution_session_id,
 (q.terminal).executed_at AS terminal_executed_at,
 (q.terminal).effect_xid::text AS terminal_effect_xid,
 (q.terminal).effect_backend_pid AS terminal_effect_backend_pid,
 (q.terminal).epoch_before AS terminal_epoch_before,
 (q.terminal).epoch_after AS terminal_epoch_after,
 (q.terminal).catalog_version AS terminal_catalog_version,
 (q.terminal).manifest_digest AS terminal_manifest_digest,
 (q.terminal).predecessor_receipt_id AS terminal_predecessor_receipt_id,
 (q.terminal).committed_epoch AS terminal_committed_epoch,
 (q.terminal).installed_object_type_id AS terminal_installed_object_type_id,
 (q.terminal).recipient_account_id AS terminal_recipient_account_id,
 (q.terminal).role_id AS terminal_role_id,
 (q.terminal).role_revision AS terminal_role_revision,
 (q.terminal).assignment_id AS terminal_assignment_id,
 (q.terminal).assignment_revision_before AS terminal_assignment_revision_before,
 (q.terminal).assignment_revision_after AS terminal_assignment_revision_after,
 (q.terminal).assignment_state_after AS terminal_assignment_state_after,
 (q.terminal).assignment_valid_from AS terminal_assignment_valid_from,
 (q.terminal).assignment_valid_until AS terminal_assignment_valid_until
FROM public.native_company_policy_status_v1($1::uuid,$2::uuid,$3::uuid,$4::uuid,$5::smallint) AS q LIMIT 2"#;

pub(super) const EXECUTE: &str = r#"SELECT q.inserted,
 (q.terminal).actor_account_id AS terminal_actor_account_id,
 (q.terminal).command_id AS terminal_command_id,
 (q.terminal).org_id AS terminal_org_id,
 (q.terminal).operation AS terminal_operation,
 (q.terminal).codec_version AS terminal_codec_version,
 (q.terminal).intake_receipt_id AS terminal_intake_receipt_id,
 (q.terminal).input_digest AS terminal_input_digest,
 (q.terminal).receipt_id AS terminal_receipt_id,
 (q.terminal).outcome AS terminal_outcome,
 (q.terminal).result_code AS terminal_result_code,
 (q.terminal).execution_session_id AS terminal_execution_session_id,
 (q.terminal).executed_at AS terminal_executed_at,
 (q.terminal).effect_xid::text AS terminal_effect_xid,
 (q.terminal).effect_backend_pid AS terminal_effect_backend_pid,
 (q.terminal).epoch_before AS terminal_epoch_before,
 (q.terminal).epoch_after AS terminal_epoch_after,
 (q.terminal).catalog_version AS terminal_catalog_version,
 (q.terminal).manifest_digest AS terminal_manifest_digest,
 (q.terminal).predecessor_receipt_id AS terminal_predecessor_receipt_id,
 (q.terminal).committed_epoch AS terminal_committed_epoch,
 (q.terminal).installed_object_type_id AS terminal_installed_object_type_id,
 (q.terminal).recipient_account_id AS terminal_recipient_account_id,
 (q.terminal).role_id AS terminal_role_id,
 (q.terminal).role_revision AS terminal_role_revision,
 (q.terminal).assignment_id AS terminal_assignment_id,
 (q.terminal).assignment_revision_before AS terminal_assignment_revision_before,
 (q.terminal).assignment_revision_after AS terminal_assignment_revision_after,
 (q.terminal).assignment_state_after AS terminal_assignment_state_after,
 (q.terminal).assignment_valid_from AS terminal_assignment_valid_from,
 (q.terminal).assignment_valid_until AS terminal_assignment_valid_until
FROM public.native_company_policy_execute_v1($1::uuid,$2::uuid,$3::uuid,$4::uuid,$5::smallint) AS q LIMIT 2"#;

pub(super) fn material(row: &PgRow) -> Result<NativeBootstrapProjectionRow, Error> {
    Ok(NativeBootstrapProjectionRow {
        actor_account_id: row.try_get("actor_account_id").map_err(sql_error)?,
        session_id: row.try_get("session_id").map_err(sql_error)?,
        account_security_generation: row
            .try_get("account_security_generation")
            .map_err(sql_error)?,
        designation_system_identifier: row
            .try_get("designation_system_identifier")
            .map_err(sql_error)?,
        designation_database_name: row
            .try_get("designation_database_name")
            .map_err(sql_error)?,
        designation_database_oid: row.try_get("designation_database_oid").map_err(sql_error)?,
        designation_revision: row.try_get("designation_revision").map_err(sql_error)?,
        designation_receipt_id: row.try_get("designation_receipt_id").map_err(sql_error)?,
        org_id: row.try_get("org_id").map_err(sql_error)?,
        current_group_id: row.try_get("current_group_id").map_err(sql_error)?,
        group_revision: row.try_get("group_revision").map_err(sql_error)?,
        group_incarnation: row.try_get("group_incarnation").map_err(sql_error)?,
        membership_id: row.try_get("membership_id").map_err(sql_error)?,
        membership_revision: row.try_get("membership_revision").map_err(sql_error)?,
        membership_incarnation: row.try_get("membership_incarnation").map_err(sql_error)?,
        company_epoch: row.try_get("company_epoch").map_err(sql_error)?,
        current_policy_receipt_id: row
            .try_get("current_policy_receipt_id")
            .map_err(sql_error)?,
        origin_account_id: row.try_get("origin_account_id").map_err(sql_error)?,
        origin_command_id: row.try_get("origin_command_id").map_err(sql_error)?,
        origin_receipt_id: row.try_get("origin_receipt_id").map_err(sql_error)?,
        administrative_account_id: row
            .try_get("administrative_account_id")
            .map_err(sql_error)?,
        company_actor_admission_receipt_id: row
            .try_get("company_actor_admission_receipt_id")
            .map_err(sql_error)?,
        birth_assignment_id: row.try_get("birth_assignment_id").map_err(sql_error)?,
        birth_role_id: row.try_get("birth_role_id").map_err(sql_error)?,
        observed_at: row.try_get("observed_at").map_err(sql_error)?,
        source_xid: row.try_get("source_xid").map_err(sql_error)?,
        source_backend_pid: row.try_get("source_backend_pid").map_err(sql_error)?,
    })
}

pub(super) struct Accepted {
    pub view: NativePolicyAcceptedView,
    pub actor: AccountId,
    pub family: Uuid,
    pub xid: u64,
    pub pid: i32,
    pub digest: Vec<u8>,
}

pub(super) struct Terminal {
    pub view: NativePolicyTerminalView,
    pub family: Uuid,
    pub xid: u64,
    pub pid: i32,
    pub predecessor: Option<Uuid>,
    pub code: String,
}

pub(super) fn absent(row: &PgRow, prefix: &str) -> Result<bool, Error> {
    let actor: Option<Uuid> = row
        .try_get(format!("{prefix}actor_account_id").as_str())
        .map_err(sql_error)?;
    if actor.is_some() {
        return Ok(false);
    }
    for (index, column) in row.columns().iter().enumerate() {
        if column.name().starts_with(prefix)
            && !row.try_get_raw(index).map_err(sql_error)?.is_null()
        {
            return Err(Error::Unavailable);
        }
    }
    Ok(true)
}

pub(super) fn accepted(
    row: &PgRow,
    actor: AccountId,
    selector: NativePolicyCommandRef,
) -> Result<Accepted, Error> {
    macro_rules! r {
        ($field:literal) => {
            row.try_get(concat!("input_", $field)).map_err(sql_error)?
        };
    }
    let bytes: Vec<u8> = r!("input_bytes");
    let stored_actor: Uuid = r!("actor_account_id");
    let company: Uuid = r!("org_id");
    let command: Uuid = r!("command_id");
    let operation: i16 = r!("operation");
    let codec: i16 = r!("codec_version");
    let (decoded_actor, input) =
        NativePolicyCommand::decode(codec, &bytes).map_err(|_| Error::Unavailable)?;
    let digest: Vec<u8> = r!("input_digest");
    let receipt: Uuid = r!("intake_receipt_id");
    let family: Uuid = r!("accepting_session_id");
    let accepted_at: OffsetDateTime = r!("accepted_at");
    let expires: OffsetDateTime = r!("execution_not_after");
    let xid: String = r!("acceptance_xid");
    let pid: i32 = r!("acceptance_backend_pid");
    if decoded_actor != actor
        || stored_actor != *actor.as_uuid()
        || company != *selector.company().as_uuid()
        || command != selector.command_id()
        || selector
            .resolve(NativePolicyCommandRef::from_command(&input))
            .is_err()
        || operation != operation_number(selector.operation())
        || codec != selector.codec_version()
        || input.encode(actor) != bytes
        || digest.len() != 32
        || receipt.is_nil()
        || family.is_nil()
        || pid < 1
        || accepted_at.checked_add(Duration::hours(168)) != Some(expires)
    {
        return Err(Error::Unavailable);
    }
    exact_time(accepted_at)?;
    exact_time(expires)?;
    Ok(Accepted {
        view: NativePolicyAcceptedView {
            input,
            intake_receipt_id: receipt,
            accepted_at,
            execution_not_after: expires,
        },
        actor,
        family,
        xid: xid_number(&xid)?,
        pid,
        digest,
    })
}

pub(super) fn terminal(
    row: &PgRow,
    accepted: &Accepted,
    administrator: AccountId,
) -> Result<Terminal, Error> {
    macro_rules! r {
        ($field:literal) => {
            row.try_get(concat!("terminal_", $field))
                .map_err(sql_error)?
        };
    }
    let actor: Uuid = r!("actor_account_id");
    let command: Uuid = r!("command_id");
    let company: Uuid = r!("org_id");
    let operation: i16 = r!("operation");
    let codec: i16 = r!("codec_version");
    let intake: Uuid = r!("intake_receipt_id");
    let digest: Vec<u8> = r!("input_digest");
    let receipt: Uuid = r!("receipt_id");
    let outcome: String = r!("outcome");
    let code: String = r!("result_code");
    let family: Uuid = r!("execution_session_id");
    let executed_at: OffsetDateTime = r!("executed_at");
    let xid: String = r!("effect_xid");
    let pid: i32 = r!("effect_backend_pid");
    let before: i64 = r!("epoch_before");
    let after: i64 = r!("epoch_after");
    let catalog: String = r!("catalog_version");
    let manifest: Vec<u8> = r!("manifest_digest");
    let predecessor: Option<Uuid> = r!("predecessor_receipt_id");
    let committed: Option<i64> = r!("committed_epoch");
    let installed: Option<Uuid> = r!("installed_object_type_id");
    let recipient: Option<Uuid> = r!("recipient_account_id");
    let role: Option<Uuid> = r!("role_id");
    let role_revision: Option<i64> = r!("role_revision");
    let assignment: Option<Uuid> = r!("assignment_id");
    let assignment_before: Option<i64> = r!("assignment_revision_before");
    let assignment_after: Option<i64> = r!("assignment_revision_after");
    let state: Option<String> = r!("assignment_state_after");
    let from: Option<OffsetDateTime> = r!("assignment_valid_from");
    let until: Option<OffsetDateTime> = r!("assignment_valid_until");
    let input = &accepted.view.input;
    if actor != *accepted.actor.as_uuid()
        || command != input.command_id()
        || company != *input.company().as_uuid()
        || operation != operation_number(input.operation())
        || codec != input.codec_version()
        || intake != accepted.view.intake_receipt_id
        || digest != accepted.digest
        || receipt.is_nil()
        || family.is_nil()
        || pid < 1
        || executed_at < accepted.view.accepted_at
        || before < 1
        || after < 1
        || catalog != input.catalog_version()
        || manifest != *input.manifest_digest()
        || (before == 1) != predecessor.is_none()
        || predecessor.is_some_and(|id| id.is_nil())
    {
        return Err(Error::Unavailable);
    }
    exact_time(executed_at)?;
    let xid = xid_number(&xid)?;
    if xid == accepted.xid {
        return Err(Error::Unavailable);
    }
    let assignment_empty = recipient.is_none()
        && role.is_none()
        && role_revision.is_none()
        && assignment.is_none()
        && assignment_before.is_none()
        && assignment_after.is_none()
        && state.is_none()
        && from.is_none()
        && until.is_none();
    let result = match outcome.as_str() {
        "REJECTED"
            if after == before
                && committed.is_none()
                && installed.is_none()
                && assignment_empty =>
        {
            let expired = executed_at >= accepted.view.execution_not_after;
            let grant_expiry_valid = input.expires_at().is_some_and(|expiry| {
                expiry > executed_at && expiry - executed_at <= Duration::days(30)
            });
            NativePolicyOutcome::Rejected(match code.as_str() {
                "intake_expired" if expired => NativePolicyRejection::IntakeExpired,
                "revision_conflict" if !expired => NativePolicyRejection::RevisionConflict,
                "grant_expiry_invalid" | "recipient_ineligible"
                    if !expired
                        && input.operation() == NativeBusinessOperationV1::Grant
                        && input.expected_company_epoch() == positive(before)?
                        && before != i64::MAX =>
                {
                    match (code.as_str(), grant_expiry_valid) {
                        ("grant_expiry_invalid", false) => {
                            NativePolicyRejection::GrantExpiryInvalid
                        }
                        ("recipient_ineligible", true) => {
                            NativePolicyRejection::RecipientIneligible
                        }
                        _ => return Err(Error::Unavailable),
                    }
                }
                _ => return Err(Error::Unavailable),
            })
        }
        "COMMITTED"
            if before.checked_add(1) == Some(after)
                && committed == Some(after)
                && u64::try_from(before).ok() == Some(input.expected_company_epoch())
                && executed_at < accepted.view.execution_not_after =>
        {
            let effect = match input.operation() {
                NativeBusinessOperationV1::Install if code == "installed" && assignment_empty => {
                    NativePolicyEffect::Installed {
                        object_type_id: nonnil(installed.ok_or(Error::Unavailable)?)?,
                    }
                }
                NativeBusinessOperationV1::Grant | NativeBusinessOperationV1::Revoke
                    if installed.is_none() =>
                {
                    let recipient = account(recipient.ok_or(Error::Unavailable)?)?;
                    if recipient != administrator {
                        return Err(Error::Unavailable);
                    }
                    let assignment = assignment_view(
                        role,
                        role_revision,
                        assignment,
                        assignment_after,
                        state,
                        from,
                        until,
                    )?
                    .ok_or(Error::Unavailable)?;
                    let revision_before = assignment_before.map(positive).transpose()?;
                    if revision_before.map_or(Some(1), |v| v.checked_add(1))
                        != Some(assignment.expectation.assignment_revision)
                    {
                        return Err(Error::Unavailable);
                    }
                    match input.operation() {
                        NativeBusinessOperationV1::Grant
                            if code == "granted"
                                && assignment.state == NativePolicyAssignmentState::Active
                                && input.recipient_account_id() == Some(recipient)
                                && assignment.valid_from == executed_at
                                && input.expires_at() == Some(assignment.valid_until)
                                && input
                                    .assignment_expectation()
                                    .map(|v| v.assignment_revision)
                                    == revision_before =>
                        {
                            if let Some(expected) = input.assignment_expectation()
                                && (expected.assignment_id != assignment.expectation.assignment_id
                                    || expected.role_revision
                                        != assignment.expectation.role_revision)
                            {
                                return Err(Error::Unavailable);
                            }
                            NativePolicyEffect::Granted {
                                recipient,
                                assignment,
                                assignment_revision_before: revision_before,
                            }
                        }
                        NativeBusinessOperationV1::Revoke
                            if code == "revoked"
                                && assignment.state == NativePolicyAssignmentState::Revoked
                                && assignment.valid_from <= executed_at =>
                        {
                            let revision_before = revision_before.ok_or(Error::Unavailable)?;
                            if input.assignment_expectation()
                                != Some(PolicyAssignmentExpectationV1 {
                                    role_revision: assignment.expectation.role_revision,
                                    assignment_id: assignment.expectation.assignment_id,
                                    assignment_revision: revision_before,
                                })
                            {
                                return Err(Error::Unavailable);
                            }
                            NativePolicyEffect::Revoked {
                                recipient,
                                assignment,
                                assignment_revision_before: revision_before,
                            }
                        }
                        _ => return Err(Error::Unavailable),
                    }
                }
                _ => return Err(Error::Unavailable),
            };
            NativePolicyOutcome::Committed(effect)
        }
        _ => return Err(Error::Unavailable),
    };
    Ok(Terminal {
        view: NativePolicyTerminalView {
            accepted: accepted.view.clone(),
            receipt_id: receipt,
            executed_at,
            epoch_before: positive(before)?,
            epoch_after: positive(after)?,
            outcome: result,
        },
        family,
        xid,
        pid,
        predecessor,
        code,
    })
}

pub(super) fn assignment_view(
    role: Option<Uuid>,
    role_revision: Option<i64>,
    id: Option<Uuid>,
    revision: Option<i64>,
    state: Option<String>,
    from: Option<OffsetDateTime>,
    until: Option<OffsetDateTime>,
) -> Result<Option<NativePolicyAssignmentView>, Error> {
    if role.is_none()
        && role_revision.is_none()
        && id.is_none()
        && revision.is_none()
        && state.is_none()
        && from.is_none()
        && until.is_none()
    {
        return Ok(None);
    }
    if role_revision != Some(1) {
        return Err(Error::Unavailable);
    }
    let role_id = nonnil(role.ok_or(Error::Unavailable)?)?;
    let assignment_id = nonnil(id.ok_or(Error::Unavailable)?)?;
    let assignment_revision = positive(revision.ok_or(Error::Unavailable)?)?;
    let state = match state.as_deref() {
        Some("ACTIVE") => NativePolicyAssignmentState::Active,
        Some("REVOKED") => NativePolicyAssignmentState::Revoked,
        _ => return Err(Error::Unavailable),
    };
    let valid_from = from.ok_or(Error::Unavailable)?;
    let valid_until = until.ok_or(Error::Unavailable)?;
    exact_time(valid_from)?;
    exact_time(valid_until)?;
    if valid_from >= valid_until || valid_until - valid_from > Duration::days(30) {
        return Err(Error::Unavailable);
    }
    Ok(Some(NativePolicyAssignmentView {
        expectation: PolicyAssignmentExpectationV1 {
            role_revision: 1,
            assignment_id,
            assignment_revision,
        },
        role_id,
        state,
        valid_from,
        valid_until,
    }))
}
