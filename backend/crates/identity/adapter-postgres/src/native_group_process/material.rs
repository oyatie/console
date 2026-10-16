//! Reconstruct every digest preimage with the application-owned codec before
//! raw SQL values become retained authority. Hash checks execute in its scope.
use super::rows::*;
use console_identity_application::group_process::*;
use serde_json::Value;

pub(super) fn projection(
    value: &Value,
    request: &GroupProcessScopeRequest<'_>,
) -> Result<(GroupProcessRetainedProjectionV1, Vec<HashCheck>), Error> {
    let mut checks = Vec::new();
    let mode = GroupProcessMaterialModeV1::from_code(
        i16::try_from(integer(value, "mode")?).map_err(|_| Error::Unavailable)?,
    )?;
    if mode != request.mode() {
        return Err(Error::Unavailable);
    }
    let (bundle, registrations) = source(field(value, "source")?)?;
    let account = account_projection(field(value, "account")?)?;
    let original = status(field(value, "original")?, &mut checks)?;
    let row = match string(value, "variant")? {
        "OwnReceipt" => {
            keys(
                value,
                &[
                    "variant",
                    "mode",
                    "account",
                    "group_id",
                    "group_incarnation",
                    "original",
                    "source",
                ],
            )?;
            let original_locator = request.locator().ok_or(Error::Unavailable)?;
            if group(value, "group_id")? != original_locator.group()
                || incarnation(value, "group_incarnation")? != original_locator.incarnation()
            {
                return Err(Error::Unavailable);
            }
            GroupProcessRetainedProjectionV1::OwnReceipt(GroupProcessOwnReceiptProjectionV1 {
                mode,
                account,
                original: original_locator,
                status: original.ok_or(Error::Unavailable)?,
                evaluated_bundle: bundle,
                registrations,
            })
        }
        "CurrentMutation" => {
            keys(
                value,
                &[
                    "variant",
                    "mode",
                    "account",
                    "group",
                    "topology",
                    "designation",
                    "group_deployment",
                    "source",
                    "policy",
                    "head",
                    "version",
                    "history",
                    "original",
                ],
            )?;
            let group_row = field(value, "group")?;
            keys(
                group_row,
                &[
                    "id",
                    "name",
                    "slug",
                    "status",
                    "created_at",
                    "updated_at",
                    "origin_account_id",
                    "origin_command_id",
                    "origin_receipt_id",
                ],
            )?;
            let group = group(group_row, "id")?;
            string(group_row, "slug")?;
            timestamp(group_row, "created_at")?;
            timestamp(group_row, "updated_at")?;
            let topology = field(value, "topology")?;
            keys(topology, &["group_id", "incarnation", "revision", "state"])?;
            let incarnation = incarnation(topology, "incarnation")?;
            if super::rows::group(topology, "group_id")? != group
                || string(topology, "state")? != "ACTIVE"
            {
                return Err(Error::Unavailable);
            }
            let designation = field(value, "designation")?;
            keys(
                designation,
                &[
                    "singleton",
                    "account_id",
                    "revision",
                    "receipt_id",
                    "system_identifier",
                    "database_name",
                    "database_oid",
                ],
            )?;
            if integer(designation, "singleton")? != 1 {
                return Err(Error::Unavailable);
            }
            let group_deployment = field(value, "group_deployment")?;
            keys(
                group_deployment,
                &["system_identifier", "database_name", "database_oid"],
            )?;
            let (policy_head, head, version, history) =
                process_projection(value, group, incarnation, &bundle, &mut checks)?;
            GroupProcessRetainedProjectionV1::Current(GroupProcessCurrentProjectionV1 {
                mode,
                account,
                group,
                incarnation,
                group_revision: positive(topology, "revision")?,
                group_state: string(group_row, "status")?.to_owned(),
                group_label: string(group_row, "name")?.to_owned(),
                group_origin_actor: super::rows::account(group_row, "origin_account_id")?,
                group_origin_command: uuid(group_row, "origin_command_id")?,
                group_origin_receipt: uuid(group_row, "origin_receipt_id")?,
                designation_actor: super::rows::account(designation, "account_id")?,
                // The reviewed current_context owner returns only after its
                // eligibility owner validates a healthy DESIGNATE receipt.
                // deployment_operator_head has no state column to parse.
                designation_state: "ACTIVE".to_owned(),
                designation_revision: positive(designation, "revision")?,
                designation_receipt: uuid(designation, "receipt_id")?,
                deployment: deployment(designation)?,
                group_deployment: deployment(group_deployment)?,
                policy_head,
                head,
                version,
                history,
                original_status: original,
                evaluated_bundle: bundle,
                registrations,
            })
        }
        _ => return Err(Error::Unavailable),
    };
    Ok((row, checks))
}

fn account_projection(value: &Value) -> Result<GroupProcessAccountProjectionV1, Error> {
    keys(
        value,
        &[
            "actor_account_id",
            "session_id",
            "account_security_generation",
            "account_state",
            "registration_receipt",
            "current_terms_receipt",
            "observed_at_us",
            "source_xid",
            "source_backend_pid",
        ],
    )?;
    Ok(GroupProcessAccountProjectionV1 {
        actor: account(value, "actor_account_id")?,
        session: uuid(value, "session_id")?,
        security_generation: positive(value, "account_security_generation")?,
        state: string(value, "account_state")?.to_owned(),
        registration_receipt: uuid(value, "registration_receipt")?,
        current_terms_receipt: uuid(value, "current_terms_receipt")?,
        observed_at_us: integer(value, "observed_at_us")?,
        xid8: decimal(string(value, "source_xid")?)?,
        backend_pid: u32_value(value, "source_backend_pid")?,
    })
}
fn deployment(value: &Value) -> Result<GroupProcessDeploymentV1, Error> {
    GroupProcessDeploymentV1::new(
        string(value, "system_identifier")?.to_owned(),
        string(value, "database_name")?.to_owned(),
        u32_value(value, "database_oid")?,
    )
}
fn policy(
    value: &Value,
    group: GroupId,
    incarnation: GroupIncarnation,
    source: &Value,
    bundle: &EvaluatedPolicyBundleV1,
    checks: &mut Vec<HashCheck>,
) -> Result<PolicyHeadReferenceV1, Error> {
    if value.is_null() {
        return Ok(PolicyHeadReferenceV1::Absent);
    }
    keys(
        value,
        &[
            "group_id",
            "group_incarnation",
            "revision",
            "schema_id",
            "schema_digest",
            "policy_digest",
            "codec_contract_digest",
            "registration_manifest_version",
            "registration_manifest_digest",
            "registered_actions",
            "first_actor_account_id",
            "first_command_id",
            "first_input_digest",
            "activation_receipt_id",
            "activated_at",
            "head_digest",
        ],
    )?;
    if super::rows::group(value, "group_id")? != group
        || super::rows::incarnation(value, "group_incarnation")? != incarnation
    {
        return Err(Error::Unavailable);
    }
    for key in [
        "schema_id",
        "schema_digest",
        "policy_digest",
        "codec_contract_digest",
        "registration_manifest_version",
        "registration_manifest_digest",
        "registered_actions",
    ] {
        if field(value, key)? != field(source, key)? {
            return Err(Error::Unavailable);
        }
    }
    let revision = PositivePolicyRevisionV1::new(positive(value, "revision")?)?;
    let digest = digest(value, "head_digest")?;
    checks.push((
        encode_group_process_policy_head_v1(&GroupProcessPolicyHeadProjectionV1 {
            group,
            incarnation,
            revision,
            evaluated_bundle: bundle.clone(),
            first_actor: account(value, "first_actor_account_id")?,
            first_command: uuid(value, "first_command_id")?,
            first_input_digest: super::rows::digest(value, "first_input_digest")?,
            activation_receipt_id: uuid(value, "activation_receipt_id")?,
            activated_at_us: timestamp(value, "activated_at")?,
        })?,
        digest,
    ));
    Ok(PolicyHeadReferenceV1::Installed {
        revision,
        head_digest: digest,
    })
}
fn head(
    value: &Value,
    group: GroupId,
    incarnation: GroupIncarnation,
    checks: &mut Vec<HashCheck>,
) -> Result<ProcessHeadV1, Error> {
    keys(
        value,
        &[
            "group_id",
            "group_incarnation",
            "process_id",
            "head_revision",
            "content_version",
            "content_digest",
            "state",
            "expires_at",
            "last_actor_account_id",
            "last_command_id",
            "last_input_digest",
            "result_receipt_id",
            "updated_at",
            "head_digest",
            "before_head_digest",
        ],
    )?;
    if super::rows::group(value, "group_id")? != group
        || super::rows::incarnation(value, "group_incarnation")? != incarnation
    {
        return Err(Error::Unavailable);
    }
    let reference = head_reference(value, "")?;
    let causing_receipt = uuid(value, "result_receipt_id")?;
    checks.push((
        encode_group_process_head_v1(&ProcessHeadDigestProjectionV1 {
            group,
            incarnation,
            reference: reference.clone(),
            last_actor: account(value, "last_actor_account_id")?,
            last_command: uuid(value, "last_command_id")?,
            last_input_digest: digest(value, "last_input_digest")?,
            result_receipt_id: causing_receipt,
            updated_at_us: timestamp(value, "updated_at")?,
        })?,
        *reference.head_digest(),
    ));
    // The predecessor digest is not in the head hash; validate its nullability
    // here and its actual chain against the entire history in projection().
    if reference.head_revision() == 1 {
        if !field(value, "before_head_digest")?.is_null() {
            return Err(Error::Unavailable);
        }
    } else {
        digest(value, "before_head_digest")?;
    }
    Ok(ProcessHeadV1 {
        reference,
        causing_receipt,
    })
}
fn version(value: &Value, checks: &mut Vec<HashCheck>) -> Result<ProcessVersionV1, Error> {
    keys(
        value,
        &[
            "group_id",
            "group_incarnation",
            "process_id",
            "version",
            "schema_id",
            "actor_account_id",
            "designation_receipt_id",
            "designation_revision",
            "policy_revision",
            "policy_head_digest",
            "adopt_command_id",
            "input_digest",
            "admitted_at",
            "expires_at",
            "operator_responsibility",
            "title",
            "method",
            "intended_claimant_matching_procedure",
            "account_possession_procedure",
            "physical_human_evidence_procedure",
            "duplicate_contradictory_claim_procedure",
            "qualification_criteria_instruction",
            "escalation_adjudication_procedure",
            "evidence_minimization_retention_description",
            "recipient_responsibility",
            "content_digest",
        ],
    )?;
    if string(value, "schema_id")? != GROUP_PROCESS_SCHEMA_V1
        || integer(value, "operator_responsibility")? != 1
    {
        return Err(Error::Unavailable);
    }
    let content = ProcessContentV1::from_projection(ProcessContentProjectionV1 {
        title: string(value, "title")?.to_owned(),
        method: ProcessMethodV1::from_str(string(value, "method")?)
            .map_err(|_| Error::Unavailable)?,
        intended_claimant_matching_procedure: string(
            value,
            "intended_claimant_matching_procedure",
        )?
        .to_owned(),
        account_possession_procedure: string(value, "account_possession_procedure")?.to_owned(),
        physical_human_evidence_procedure: string(value, "physical_human_evidence_procedure")?
            .to_owned(),
        duplicate_contradictory_claim_procedure: string(
            value,
            "duplicate_contradictory_claim_procedure",
        )?
        .to_owned(),
        qualification_criteria_instruction: string(value, "qualification_criteria_instruction")?
            .to_owned(),
        escalation_adjudication_procedure: string(value, "escalation_adjudication_procedure")?
            .to_owned(),
        evidence_minimization_retention_description: string(
            value,
            "evidence_minimization_retention_description",
        )?
        .to_owned(),
        recipient_responsibility: string(value, "recipient_responsibility")?.to_owned(),
    })?;
    let version = ProcessVersionV1::from_projection(ProcessVersionProjectionV1 {
        group: group(value, "group_id")?,
        incarnation: incarnation(value, "group_incarnation")?,
        process_id: uuid(value, "process_id")?,
        version: positive(value, "version")?,
        actor: account(value, "actor_account_id")?,
        designation_receipt: uuid(value, "designation_receipt_id")?,
        designation_revision: positive(value, "designation_revision")?,
        policy_revision: PositivePolicyRevisionV1::new(positive(value, "policy_revision")?)?,
        policy_head_digest: digest(value, "policy_head_digest")?,
        adopt_command: uuid(value, "adopt_command_id")?,
        input_digest: digest(value, "input_digest")?,
        admitted_at_us: timestamp(value, "admitted_at")?,
        expiry_us: timestamp(value, "expires_at")?,
        responsibility: AcceptedOperatorResponsibilityV1,
        content,
        content_digest: digest(value, "content_digest")?,
    })?;
    checks.push((version.encode(), *version.content_digest()));
    Ok(version)
}

pub(super) fn navigation(
    value: &Value,
    actor: uuid::Uuid,
    family: uuid::Uuid,
    recheck: bool,
) -> Result<(GroupProcessNavigationCandidatesV1, Vec<HashCheck>), Error> {
    keys(
        value,
        &[
            "protocol",
            "actor_account_id",
            "session_id",
            "status",
            "candidates",
            "snapshot",
        ],
    )?;
    if string(value, "protocol")? != "GROUP_PROCESS_NAVIGATION_V1"
        || uuid(value, "actor_account_id")? != actor
        || uuid(value, "session_id")? != family
    {
        return Err(Error::Unavailable);
    }
    let candidates = field(value, "candidates")?
        .as_array()
        .ok_or(Error::Unavailable)?;
    if string(value, "status")? == "CONFLICT" {
        if !recheck || !candidates.is_empty() || !field(value, "snapshot")?.is_null() {
            return Err(Error::Unavailable);
        }
        return Err(Error::Conflict);
    }
    if string(value, "status")? != "MATCH" || candidates.len() > 256 {
        return Err(Error::Unavailable);
    }
    let candidates = candidates
        .iter()
        .map(|row| {
            keys(row, &["group_id", "group_incarnation"])?;
            Ok(GroupProcessNavigationCandidateV1::new(
                group(row, "group_id")?,
                incarnation(row, "group_incarnation")?,
            ))
        })
        .collect::<Result<_, Error>>()?;
    let snapshot = GroupProcessNavigationSnapshotV1::from_protected_source_bytes(bytes(
        value, "snapshot", 1, 1_048_576,
    )?)?;
    let candidates = GroupProcessNavigationCandidatesV1::from_projection(candidates, snapshot)?;
    let checks = snapshot_checks(&candidates, actor, family)?;
    Ok((candidates, checks))
}

fn snapshot_checks(
    candidates: &GroupProcessNavigationCandidatesV1,
    actor: uuid::Uuid,
    family: uuid::Uuid,
) -> Result<Vec<HashCheck>, Error> {
    let bytes = candidates.snapshot().source_bytes();
    let raw = std::str::from_utf8(bytes).map_err(|_| Error::Unavailable)?;
    let snapshot = document(raw, 1_048_576)?;
    keys(&snapshot, &["protocol", "account", "designation", "groups"])?;
    if string(&snapshot, "protocol")? != "GROUP_PROCESS_NAVIGATION_SOURCE_V1" {
        return Err(Error::Unavailable);
    }
    // The fixed ABI types also include required explicit nulls. SQL owns the
    // relational origin/audit closure; this independent parser rejects every
    // missing, extra or mistyped row before digest preimages are reconstructed.
    let shapes = document(super::snapshot_shapes::SHAPES, 20_000)?;
    let account = field(&snapshot, "account")?;
    keys(
        account,
        &[
            "actor_account_id",
            "session_id",
            "root",
            "security",
            "family",
            "enrollment",
            "consent",
            "terms_head",
        ],
    )?;
    if uuid(account, "actor_account_id")? != actor || uuid(account, "session_id")? != family {
        return Err(Error::Unavailable);
    }
    for (key, schema, identity) in [
        ("root", "accounts", "id"),
        ("security", "account_security", "account_id"),
        ("family", "family", "user_id"),
        ("enrollment", "account_security_events", "account_id"),
    ] {
        let value = field(account, key)?;
        snapshot_row(value, &shapes, schema)?;
        if uuid(value, identity)? != actor {
            return Err(Error::Unavailable);
        }
    }
    if string(field(account, "enrollment")?, "kind")? != "ENROLLED" {
        return Err(Error::Unavailable);
    }
    snapshot_row(field(account, "terms_head")?, &shapes, "terms_head")?;
    let consent = field(account, "consent")?
        .as_array()
        .ok_or(Error::Unavailable)?;
    if consent.is_empty() || consent.len() > 8 {
        return Err(Error::Unavailable);
    }
    let mut previous_kind: Option<&str> = None;
    for row in consent {
        snapshot_row(row, &shapes, "consent")?;
        let kind = string(row, "terms_kind")?;
        if kind.is_empty()
            || kind.len() > 64
            || !kind.as_bytes()[0].is_ascii_lowercase()
            || !kind
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"_.-".contains(&b))
            || previous_kind.is_some_and(|previous| previous >= kind)
        {
            return Err(Error::Unavailable);
        }
        previous_kind = Some(kind);
    }
    let designation = field(&snapshot, "designation")?;
    if !designation.is_null() {
        keys(designation, &["head", "receipt"])?;
        snapshot_row(
            field(designation, "head")?,
            &shapes,
            "deployment_operator_head",
        )?;
        snapshot_row(
            field(designation, "receipt")?,
            &shapes,
            "deployment_operator_receipts",
        )?;
    }
    let groups = field(&snapshot, "groups")?
        .as_array()
        .ok_or(Error::Unavailable)?;
    if groups.len() != candidates.candidates().len() {
        return Err(Error::Unavailable);
    }
    let mut checks = Vec::new();
    for (row, candidate) in groups.iter().zip(candidates.candidates()) {
        keys(
            row,
            &[
                "group_id",
                "group_incarnation",
                "group",
                "topology",
                "birth_request",
                "birth_receipt",
                "birth_designation_receipt",
                "source",
                "policy",
                "head",
                "version",
                "history",
            ],
        )?;
        if group(row, "group_id")? != candidate.group()
            || incarnation(row, "group_incarnation")? != candidate.incarnation()
        {
            return Err(Error::Unavailable);
        }
        for (key, schema) in [
            ("group", "groups"),
            ("topology", "group_authority_heads"),
            ("birth_request", "company_enrollment_requests"),
            ("birth_receipt", "company_enrollment_receipts"),
            ("birth_designation_receipt", "deployment_operator_receipts"),
            ("source", "source"),
        ] {
            snapshot_row(field(row, key)?, &shapes, schema)?;
        }
        let root = field(row, "group")?;
        let topology = field(row, "topology")?;
        if group(root, "id")? != candidate.group()
            || group(topology, "group_id")? != candidate.group()
            || incarnation(topology, "incarnation")? != candidate.incarnation()
        {
            return Err(Error::Unavailable);
        }
        let birth_request = field(row, "birth_request")?;
        let birth_receipt = field(row, "birth_receipt")?;
        if uuid(root, "origin_account_id")? != uuid(birth_request, "account_id")?
            || uuid(root, "origin_command_id")? != uuid(birth_request, "command_id")?
            || uuid(root, "origin_receipt_id")? != uuid(birth_request, "committed_receipt_id")?
            || uuid(root, "origin_account_id")? != uuid(birth_receipt, "account_id")?
            || uuid(root, "origin_command_id")? != uuid(birth_receipt, "command_id")?
            || uuid(root, "origin_receipt_id")? != uuid(birth_receipt, "receipt_id")?
            || candidate.group() != group(birth_receipt, "group_id")?
            || string(birth_request, "state")? != "COMMITTED"
        {
            return Err(Error::Unavailable);
        }
        for (key, schema) in [
            ("policy", "native_group_identity_policy_heads_v1"),
            ("head", "native_group_process_heads_v1"),
            ("version", "native_group_process_versions_v1"),
        ] {
            let value = field(row, key)?;
            if !value.is_null() {
                snapshot_row(value, &shapes, schema)?;
            }
        }
        let history = field(row, "history")?
            .as_array()
            .ok_or(Error::Unavailable)?;
        for item in history {
            keys(item, &["head", "reason"])?;
            snapshot_row(
                field(item, "head")?,
                &shapes,
                "native_group_process_head_revisions_v1",
            )?;
        }
        let (bundle, registrations) = source(field(row, "source")?)?;
        checks.push((
            encode_group_process_registration_projection_v1(
                candidate.group(),
                candidate.incarnation(),
                &bundle,
                registrations,
            )?,
            *bundle.registration_manifest_digest(),
        ));
        let (policy, head, version, history) = process_projection(
            row,
            candidate.group(),
            candidate.incarnation(),
            &bundle,
            &mut checks,
        )?;
        check_group_process_projection_v1(
            candidate.group(),
            candidate.incarnation(),
            policy,
            head.as_ref(),
            version.as_ref(),
            &history,
        )?;
    }
    Ok(checks)
}

fn snapshot_row(value: &Value, shapes: &Value, name: &str) -> Result<(), Error> {
    let shape = field(shapes, name)?.as_object().ok_or(Error::Unavailable)?;
    let object = value.as_object().ok_or(Error::Unavailable)?;
    if shape.len() != object.len() || shape.keys().any(|key| !object.contains_key(key)) {
        return Err(Error::Unavailable);
    }
    for (key, rule) in shape {
        let rule = rule.as_array().ok_or(Error::Unavailable)?;
        let [kind, nullable] = rule.as_slice() else {
            return Err(Error::Unavailable);
        };
        let member = field(value, key)?;
        if member.is_null() && nullable.as_bool() == Some(true) {
            continue;
        }
        match kind.as_str() {
            Some("uuid") => {
                uuid(value, key)?;
            }
            Some("timestamp") => {
                let instant = time_from_us(timestamp(value, key)?)?;
                if instant.year() < 1 {
                    return Err(Error::Unavailable);
                }
                let formatted = instant
                    .format(&time::format_description::well_known::Rfc3339)
                    .map_err(|_| Error::Unavailable)?;
                // Snapshot SQL renders timestamptz under fixed UTC. JSON
                // canonicalization alone does not normalize string contents.
                let utc = formatted.strip_suffix('Z').ok_or(Error::Unavailable)?;
                if string(value, key)? != format!("{utc}+00:00") {
                    return Err(Error::Unavailable);
                }
            }
            Some("bytea") => {
                if key.ends_with("digest") || key.ends_with("sha256") {
                    digest(value, key)?;
                } else {
                    bytes(value, key, 0, 1_048_576)?;
                }
            }
            Some("integer")
                if member
                    .as_i64()
                    .is_some_and(|n| n > 0 || (n == 0 && key == "expected_revision")) => {}
            Some("string") if member.is_string() => {}
            Some("object") if member.is_object() => {}
            Some("array") if member.is_array() => {}
            _ => return Err(Error::Unavailable),
        }
    }
    Ok(())
}

fn process_projection(
    value: &Value,
    group: GroupId,
    incarnation: GroupIncarnation,
    bundle: &EvaluatedPolicyBundleV1,
    checks: &mut Vec<HashCheck>,
) -> Result<
    (
        PolicyHeadReferenceV1,
        Option<ProcessHeadV1>,
        Option<ProcessVersionV1>,
        Vec<GroupProcessHistoryViewV1>,
    ),
    Error,
> {
    let policy_head = policy(
        field(value, "policy")?,
        group,
        incarnation,
        field(value, "source")?,
        bundle,
        checks,
    )?;
    let head_raw = field(value, "head")?;
    let head = if head_raw.is_null() {
        None
    } else {
        Some(head(head_raw, group, incarnation, checks)?)
    };
    let version_raw = field(value, "version")?;
    let version = if version_raw.is_null() {
        None
    } else {
        Some(version(version_raw, checks)?)
    };
    let history_raw = field(value, "history")?
        .as_array()
        .ok_or(Error::Unavailable)?;
    let mut history = Vec::with_capacity(history_raw.len());
    let mut before_digest: Option<[u8; 32]> = None;
    for item in history_raw {
        keys(item, &["head", "reason"])?;
        let raw_head = field(item, "head")?;
        let head = self::head(raw_head, group, incarnation, checks)?;
        let raw_before = field(raw_head, "before_head_digest")?;
        match before_digest {
            None if !raw_before.is_null() => return Err(Error::Unavailable),
            Some(previous) if digest(raw_head, "before_head_digest")? != previous => {
                return Err(Error::Unavailable);
            }
            _ => {}
        }
        before_digest = Some(*head.reference.head_digest());
        let reason = field(item, "reason")?;
        history.push(GroupProcessHistoryViewV1 {
            head,
            actor: super::rows::account(raw_head, "last_actor_account_id")?,
            command_id: uuid(raw_head, "last_command_id")?,
            occurred_at_us: timestamp(raw_head, "updated_at")?,
            reason: if reason.is_null() {
                None
            } else {
                Some(reason.as_str().ok_or(Error::Unavailable)?.to_owned())
            },
        });
    }
    if !head_raw.is_null() && history_raw.last().and_then(|last| last.get("head")) != Some(head_raw)
    {
        return Err(Error::Unavailable);
    }
    Ok((policy_head, head, version, history))
}
