//! Supplemental semantic closure tests at the actual checked application API.
//! Synthetic fixtures are not authentication, installed SQL custody or release
//! evidence. Every byte/hash pair is independently declared in ORACLE_JSON.
use super::*;
use serde_json::Value;
use uuid::Uuid;

const A_US: i64 = 1_790_816_400_123_456;
const OBSERVED_US: i64 = 1_790_816_401_234_567;
const B_US: i64 = 1_790_816_402_345_678;
const FINAL_US: i64 = 1_790_816_403_456_789;
const EXPIRY_US: i64 = 1_790_902_800_000_000;
const PRIOR_B_US: i64 = 1_790_816_391_000_000;
const ORIGINAL_TITLE: &str = "독립 검증 절차";
const ORIGINAL_REASON: &str = "검토한 원문 중지 사유";

fn id(n: u128) -> Uuid {
    Uuid::from_u128(n)
}

fn actor(n: u128) -> AccountId {
    AccountId::from_uuid(id(n)).unwrap()
}

fn group() -> GroupId {
    GroupId::from_uuid(id(3)).unwrap()
}

fn incarnation() -> GroupIncarnation {
    GroupIncarnation::from_uuid(id(4)).unwrap()
}

fn unhex(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0);
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

fn digest(vector: &Value) -> [u8; 32] {
    unhex(vector["sha256"].as_str().unwrap())
        .try_into()
        .unwrap()
}

fn check_vector(vector: &Value, actual: &[u8]) {
    assert_eq!(actual, unhex(vector["hex"].as_str().unwrap()));
    assert_eq!(actual.len(), vector["bytes"].as_u64().unwrap() as usize);
}

fn content(title: &str) -> ProcessContentV1 {
    ProcessContentV1::from_projection(ProcessContentProjectionV1 {
        title: title.into(),
        method: ProcessMethodV1::AttendedAccountAndDocumentaryReview,
        intended_claimant_matching_procedure: "claimant match".into(),
        account_possession_procedure: "account possession".into(),
        physical_human_evidence_procedure: "physical evidence".into(),
        duplicate_contradictory_claim_procedure: "contradictory claim".into(),
        qualification_criteria_instruction: "qualification instruction".into(),
        escalation_adjudication_procedure: "escalation adjudication".into(),
        evidence_minimization_retention_description: "minimal retention".into(),
        recipient_responsibility: "recipient responsibility".into(),
    })
    .unwrap()
}

fn bundle(oracles: &Value) -> EvaluatedPolicyBundleV1 {
    let b = &oracles["bundle"];
    let d = |key: &str| unhex(b[key].as_str().unwrap()).try_into().unwrap();
    EvaluatedPolicyBundleV1::from_projection(EvaluatedPolicyBundleProjectionV1 {
        schema_id: b["schema_id"].as_str().unwrap().into(),
        schema_digest: d("schema_digest"),
        policy_digest: d("policy_digest"),
        codec_contract_digest: d("codec_contract_digest"),
        registration_manifest_version: 1,
        registration_manifest_digest: d("registration_manifest_digest"),
        cedar_sdk_version: b["cedar_sdk_version"].as_str().unwrap().into(),
        cedar_language_version: b["cedar_language_version"].as_str().unwrap().into(),
    })
    .unwrap()
}

fn policy(
    oracle: &Value,
    evaluated_bundle: &EvaluatedPolicyBundleV1,
    first_command: u128,
    input_digest: [u8; 32],
    activation_receipt: u128,
    activated_at_us: i64,
) -> PolicyHeadReferenceV1 {
    let bytes = encode_group_process_policy_head_v1(&GroupProcessPolicyHeadProjectionV1 {
        group: group(),
        incarnation: incarnation(),
        revision: PositivePolicyRevisionV1::new(1).unwrap(),
        evaluated_bundle: evaluated_bundle.clone(),
        first_actor: actor(1),
        first_command: id(first_command),
        first_input_digest: input_digest,
        activation_receipt_id: id(activation_receipt),
        activated_at_us,
    })
    .unwrap();
    check_vector(oracle, &bytes);
    PolicyHeadReferenceV1::Installed {
        revision: PositivePolicyRevisionV1::new(1).unwrap(),
        head_digest: digest(oracle),
    }
}

fn version(meta: &Value, oracle: &Value, policy: PolicyHeadReferenceV1) -> ProcessVersionV1 {
    let PolicyHeadReferenceV1::Installed {
        revision,
        head_digest,
    } = policy
    else {
        panic!("fixture version requires genuinely declared installed policy bytes");
    };
    let value = ProcessVersionV1::from_projection(ProcessVersionProjectionV1 {
        group: group(),
        incarnation: incarnation(),
        process_id: id(5),
        version: 1,
        actor: actor(u128::from(meta["actor"].as_u64().unwrap())),
        designation_receipt: id(u128::from(meta["designation_receipt"].as_u64().unwrap())),
        designation_revision: meta["designation_revision"].as_u64().unwrap(),
        policy_revision: revision,
        policy_head_digest: head_digest,
        adopt_command: id(u128::from(meta["command"].as_u64().unwrap())),
        input_digest: unhex(meta["input_digest"].as_str().unwrap())
            .try_into()
            .unwrap(),
        admitted_at_us: meta["admitted_at_us"].as_i64().unwrap(),
        expiry_us: meta["expiry_us"].as_i64().unwrap(),
        responsibility: AcceptedOperatorResponsibilityV1::from_form("yes").unwrap(),
        content: content(meta["title"].as_str().unwrap()),
        content_digest: digest(oracle),
    })
    .unwrap();
    check_vector(oracle, &value.encode());
    value
}

#[allow(clippy::too_many_arguments)]
fn head(
    oracle: &Value,
    version_digest: [u8; 32],
    state: ProcessStateV1,
    revision: u64,
    expiry_us: i64,
    command: u128,
    input_digest: [u8; 32],
    receipt: u128,
    updated_at_us: i64,
) -> ProcessHeadV1 {
    let reference = ProcessHeadReferenceV1::from_projection(ProcessHeadReferenceProjectionV1 {
        process_id: id(5),
        head_revision: revision,
        content_version: 1,
        content_digest: version_digest,
        head_digest: digest(oracle),
        state,
        expiry_us,
    })
    .unwrap();
    let bytes = encode_group_process_head_v1(&ProcessHeadDigestProjectionV1 {
        group: group(),
        incarnation: incarnation(),
        reference: reference.clone(),
        last_actor: actor(1),
        last_command: id(command),
        last_input_digest: input_digest,
        result_receipt_id: id(receipt),
        updated_at_us,
    })
    .unwrap();
    check_vector(oracle, &bytes);
    ProcessHeadV1 {
        reference,
        causing_receipt: id(receipt),
    }
}

fn binding(at: i64) -> GroupProcessRetainedBindingV1 {
    GroupProcessRetainedBindingV1::new(actor(1), id(8), 1, at, 21, 123).unwrap()
}

fn row(
    oracles: &Value,
    at: i64,
    policy_head: PolicyHeadReferenceV1,
    head: Option<ProcessHeadV1>,
    version: Option<ProcessVersionV1>,
    history: Vec<GroupProcessHistoryViewV1>,
    status: GroupProcessStatus,
) -> GroupProcessCurrentProjectionV1 {
    let deployment =
        GroupProcessDeploymentV1::new("123456789".into(), "constructor_fixture".into(), 42)
            .unwrap();
    let registrations = GroupProcessActionV1::ALL
        .into_iter()
        .enumerate()
        .map(|(index, action)| GroupProcessRegistrationProjectionV1 {
            action,
            action_id: id(31 + index as u128),
            revision: 1,
            fields: oracles["fields"][index]
                .as_array()
                .unwrap()
                .iter()
                .map(|field| GroupProcessFieldV1::from_str(field.as_str().unwrap()).unwrap())
                .collect(),
        })
        .collect();
    GroupProcessCurrentProjectionV1 {
        mode: GroupProcessMaterialModeV1::Execute,
        account: GroupProcessAccountProjectionV1 {
            actor: actor(1),
            session: id(8),
            security_generation: 1,
            state: "ACTIVE".into(),
            registration_receipt: id(12),
            current_terms_receipt: id(13),
            observed_at_us: at,
            xid8: 21,
            backend_pid: 123,
        },
        group: group(),
        incarnation: incarnation(),
        group_revision: 7,
        group_state: "ACTIVE".into(),
        group_label: "실제 절차를 검사하는 단위 테스트".into(),
        group_origin_actor: actor(1),
        group_origin_command: id(10),
        group_origin_receipt: id(11),
        designation_actor: actor(1),
        designation_state: "ACTIVE".into(),
        designation_revision: 1,
        designation_receipt: id(9),
        deployment: deployment.clone(),
        group_deployment: deployment,
        policy_head,
        head,
        version,
        history,
        original_status: Some(status),
        evaluated_bundle: bundle(oracles),
        registrations,
    }
}

struct Fixture {
    preimage: CurrentGroupProcessAuthority,
    request: GroupProcessScopeRequest<'static>,
    projected: GroupProcessCurrentProjectionV1,
    execution: GroupProcessExecution,
    final_binding: GroupProcessRetainedBindingV1,
    prior_version_bytes: Option<Vec<u8>>,
    projected_version_bytes: Vec<u8>,
}

fn fixture(key: &str, final_us: i64) -> Fixture {
    let oracles: Value = serde_json::from_str(ORACLE_JSON).unwrap();
    let spec = &oracles["cases"][key];
    assert!(spec.is_object(), "unknown independently declared fixture");
    let adopting = spec["family"].as_str().unwrap() == "adopt";
    let evaluated = bundle(&oracles);
    let prior = &oracles["prior"];
    let prior_policy = policy(
        &prior["policy"],
        &evaluated,
        15,
        digest(&prior["command"]),
        14,
        PRIOR_B_US,
    );
    let prior_version = version(&prior["meta"], &prior["version"], prior_policy);
    let prior_head = head(
        &prior["head"],
        digest(&prior["version"]),
        ProcessStateV1::Active,
        1,
        EXPIRY_US,
        15,
        digest(&prior["command"]),
        14,
        PRIOR_B_US,
    );
    let prior_history = GroupProcessHistoryViewV1 {
        head: prior_head.clone(),
        actor: actor(1),
        command_id: id(15),
        occurred_at_us: PRIOR_B_US,
        reason: None,
    };
    let input = if adopting {
        GroupProcessCommandV1::adopt(
            id(2),
            group(),
            incarnation(),
            7,
            0,
            id(5),
            0,
            time_from_us(EXPIRY_US).unwrap(),
            AcceptedOperatorResponsibilityV1::from_form("yes").unwrap(),
            content(ORIGINAL_TITLE),
        )
        .unwrap()
    } else {
        GroupProcessCommandV1::suspend(
            id(2),
            group(),
            incarnation(),
            7,
            1,
            id(5),
            1,
            digest(&prior["version"]),
            1,
            digest(&prior["head"]),
            ORIGINAL_REASON.into(),
        )
        .unwrap()
    };
    let input_bytes = input.encode(actor(1)).unwrap();
    check_vector(&spec["command"], &input_bytes);
    let accepted = GroupProcessAcceptedV1::checked(
        actor(1),
        input_bytes,
        digest(&spec["command"]),
        id(6),
        A_US,
    )
    .unwrap();
    let request = GroupProcessScopeRequest::Execute(accepted.locator());
    let before_row = row(
        &oracles,
        OBSERVED_US,
        if adopting {
            PolicyHeadReferenceV1::Absent
        } else {
            prior_policy
        },
        if adopting {
            None
        } else {
            Some(prior_head.clone())
        },
        if adopting {
            None
        } else {
            Some(prior_version.clone())
        },
        if adopting {
            vec![]
        } else {
            vec![prior_history.clone()]
        },
        GroupProcessStatus::AcceptedPending(accepted.clone()),
    );
    let preimage = CurrentGroupProcessAuthority::from_retained_projection(
        binding(OBSERVED_US),
        &request,
        GroupProcessRetainedProjectionV1::Current(before_row),
    )
    .unwrap();
    let after_policy = if adopting {
        policy(
            &spec["policy"],
            &evaluated,
            2,
            accepted.input_digest,
            7,
            B_US,
        )
    } else {
        prior_policy
    };
    let after_version = version(&spec["meta"], &spec["version"], after_policy);
    let version_bytes = after_version.encode();
    let expiry = if adopting {
        spec["meta"]["expiry_us"].as_i64().unwrap()
    } else {
        EXPIRY_US
    };
    let after_head = head(
        &spec["head"],
        if adopting {
            digest(&spec["version"])
        } else {
            digest(&prior["version"])
        },
        if adopting {
            ProcessStateV1::Active
        } else {
            ProcessStateV1::Suspended
        },
        if adopting { 1 } else { 2 },
        expiry,
        2,
        accepted.input_digest,
        7,
        B_US,
    );
    let result = ProcessResultV2::from_projection(ProcessResultProjectionV2 {
        actor: actor(1),
        command: id(2),
        group: group(),
        incarnation: incarnation(),
        opcode: if adopting { 1 } else { 6 },
        input_digest: accepted.input_digest,
        intake_receipt: id(6),
        result_receipt: id(7),
        terminal_code: if adopting {
            ProcessTerminalCodeV1::Adopted
        } else {
            ProcessTerminalCodeV1::Suspended
        },
        accepted_at_us: A_US,
        executed_at_us: B_US,
        session: id(8),
        security_generation: 1,
        designation_receipt: id(9),
        designation_revision: 1,
        observed_group_revision: 7,
        policy_before: if adopting {
            PolicyHeadReferenceV1::Absent
        } else {
            prior_policy
        },
        policy_after: after_policy,
        evaluated_bundle: evaluated,
        requested_process_id: id(5),
        before_head: if adopting {
            None
        } else {
            Some(prior_head.reference)
        },
        after_head: Some(after_head.reference.clone()),
        effect_xid8: 21,
        effect_pid: 123,
    })
    .unwrap();
    let result_bytes = result.encode();
    check_vector(&spec["result"], &result_bytes);
    let terminal =
        GroupProcessTerminalV2::checked(accepted, result_bytes, digest(&spec["result"])).unwrap();
    let mut history = if adopting {
        vec![]
    } else {
        vec![prior_history]
    };
    history.push(GroupProcessHistoryViewV1 {
        head: after_head.clone(),
        actor: actor(1),
        command_id: id(2),
        occurred_at_us: B_US,
        reason: if adopting {
            None
        } else {
            Some(spec["reason"].as_str().unwrap().into())
        },
    });
    let projected = row(
        &oracles,
        final_us,
        after_policy,
        Some(after_head),
        Some(after_version),
        history,
        GroupProcessStatus::Terminal(terminal.clone()),
    );
    Fixture {
        preimage,
        request,
        projected,
        execution: GroupProcessExecution {
            inserted: true,
            terminal,
        },
        final_binding: binding(final_us),
        prior_version_bytes: if adopting {
            None
        } else {
            Some(prior_version.encode())
        },
        projected_version_bytes: version_bytes,
    }
}

fn postimage(f: Fixture) -> Result<CurrentGroupProcessAuthority, GroupProcessError> {
    f.preimage
        .execution_postimage(f.final_binding, &f.request, f.projected, &f.execution)
}

#[test]
fn group_postimage_contract_accepts_exact_adoption_and_keeps_original_a_b_times() {
    let f = fixture("first_valid", FINAL_US);
    assert_ne!(A_US, B_US);
    assert_eq!(f.projected.version.as_ref().unwrap().admitted_at_us(), A_US);
    assert_eq!(f.projected.history.last().unwrap().occurred_at_us, B_US);
    let authority =
        postimage(f).expect("the exact accepted adoption must reconstruct final authority");
    assert!(matches!(
        authority.original_status(),
        Some(GroupProcessStatus::Terminal(_))
    ));
}

#[test]
fn group_postimage_contract_accepts_suspension_with_byte_identical_original_version() {
    let f = fixture("suspend_valid", FINAL_US);
    assert_eq!(
        f.prior_version_bytes.as_ref().unwrap(),
        &f.projected_version_bytes
    );
    assert!(
        postimage(f).is_ok(),
        "suspension preserves the original immutable version"
    );
}

#[test]
fn group_postimage_contract_rejects_alternate_adopted_content_with_genuine_hashes() {
    assert!(
        postimage(fixture("first_content", FINAL_US)).is_err(),
        "self-consistent source/hash/result for different prose is not the accepted command"
    );
}

#[test]
fn group_postimage_contract_rejects_alternate_adopted_expiry_with_genuine_hashes() {
    assert!(
        postimage(fixture("first_expiry", FINAL_US)).is_err(),
        "a self-consistent longer expiry must not substitute the accepted expiry"
    );
}

#[test]
fn group_postimage_contract_rejects_alternate_version_attribution_and_admission_metadata() {
    let cases = [
        "first_actor",
        "first_designation_receipt",
        "first_designation_revision",
        "first_command",
        "first_input_digest",
        "first_admitted_at",
    ];
    // Exercise every injected case before asserting, rather than masking later
    // variants after the first expected RED. Every variant has independently
    // coherent version/head/result bytes and genuine SHA-256 identities.
    let outcomes: Vec<_> = cases
        .into_iter()
        .map(|case| (case, postimage(fixture(case, FINAL_US)).is_err()))
        .collect();
    assert!(
        outcomes.iter().all(|(_, rejected)| *rejected),
        "version attribution/admission must bind original input and retained actor/designation: {outcomes:?}"
    );
}

#[test]
fn group_postimage_contract_rejects_alternate_suspension_reason() {
    assert!(
        postimage(fixture("suspend_reason", FINAL_US)).is_err(),
        "a nonblank history reason still must equal the original submitted reason"
    );
}

#[test]
fn group_postimage_contract_rejects_suspension_version_substitution_at_source_closure() {
    let f = fixture("suspend_version", FINAL_US);
    assert_ne!(
        f.prior_version_bytes.as_ref().unwrap(),
        &f.projected_version_bytes
    );
    assert!(
        postimage(f).is_err(),
        "genuine new version hash cannot satisfy an unchanged suspension head/version reference"
    );
}

#[test]
fn group_postimage_contract_rejects_final_expiry_without_rewriting_terminal_outcome() {
    assert!(
        postimage(fixture("first_valid", EXPIRY_US + 1)).is_err(),
        "late final expiry rolls back B; it cannot become another terminal outcome"
    );
}

#[test]
fn group_postimage_contract_rejects_substituted_expiry_after_original_expiry_passes() {
    assert!(
        postimage(fixture("first_expiry", EXPIRY_US + 1)).is_err(),
        "substitute longer source expiry cannot make an expired original command finalizable"
    );
}

#[test]
fn group_postimage_contract_terminal_source_cannot_enter_initial_execute_authority() {
    let f = fixture("first_valid", FINAL_US);
    assert!(
        CurrentGroupProcessAuthority::from_retained_projection(
            f.final_binding,
            &f.request,
            GroupProcessRetainedProjectionV1::Current(f.projected)
        )
        .is_err(),
        "only the original pending preimage may admit an effect"
    );
}

#[test]
fn group_postimage_contract_final_authority_cannot_start_or_finalize_another_effect() {
    let first = postimage(fixture("first_valid", FINAL_US)).unwrap();
    assert!(classify_group_process_transition_v1(&first).is_err());
    let f = fixture("first_valid", FINAL_US);
    assert!(
        first
            .execution_postimage(f.final_binding, &f.request, f.projected, &f.execution)
            .is_err(),
        "a reconstructed terminal postimage is not reusable pending authority"
    );
}

// Generated once outside product code from reviewed declared byte layouts by
// Python standard struct/hashlib. The sealed oracle author and mapping are in
// the review packet; no product encoder generated these independent vectors.
const ORACLE_JSON: &str = r###"{
  "bundle": {
    "schema_id": "native-group-process-v1",
    "schema_digest": "7e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683",
    "policy_digest": "573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727",
    "codec_contract_digest": "338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e5129769",
    "registration_manifest_version": 1,
    "registration_manifest_digest": "4172e4cf6afcbfb97e879eff3d02061db35cf4a0a02042724b710351dcc60b25",
    "cedar_sdk_version": "4.13.0",
    "cedar_language_version": "4.5"
  },
  "fields": [
    [
      "process_id",
      "process_version",
      "process_digest",
      "process_head_revision",
      "process_head_digest",
      "state",
      "expiry",
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
      "receipt_locator"
    ],
    [
      "process_id",
      "process_version",
      "process_digest",
      "process_head_revision",
      "process_head_digest",
      "state",
      "expiry",
      "reason",
      "receipt_locator"
    ],
    [
      "group_context",
      "process_id",
      "process_version",
      "process_digest",
      "process_head_revision",
      "process_head_digest",
      "state",
      "expiry",
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
      "history",
      "allowed_actions"
    ],
    [
      "command_id",
      "input_digest",
      "intake_receipt_id",
      "result_receipt_id",
      "terminal_code",
      "accepted_at",
      "executed_at",
      "process_id",
      "before_head",
      "after_head",
      "original_content",
      "original_reason",
      "receipt_locator"
    ]
  ],
  "prior": {
    "meta": {
      "actor": 1,
      "designation_receipt": 9,
      "designation_revision": 1,
      "command": 15,
      "input_digest": "19a911a6eb9b79ff986bddb45fc24ea17f6e01b2589f33ebcefc3fcebf63e128",
      "admitted_at_us": 1790816390000000,
      "expiry_us": 1790902800000000,
      "title": "독립 검증 절차"
    },
    "command": {
      "hex": "434f4e534f4c452e4944454e544954592e47524f55500000010001000000000000000000000000000000010000000000000000000000000000000f00000000000000000000000000000003000000000000000000000000000000040000000000000007000000000000000000000000000000000000000000000005000000000000000000065cd10d38a400000100000014eb8f85eba6bd20eab280eca69d20eca088ecb0a800010000000e636c61696d616e74206d61746368000000126163636f756e7420706f7373657373696f6e00000011706879736963616c2065766964656e636500000013636f6e747261646963746f727920636c61696d000000197175616c696669636174696f6e20696e737472756374696f6e00000017657363616c6174696f6e2061646a756469636174696f6e000000116d696e696d616c20726574656e74696f6e00000018726563697069656e7420726573706f6e736962696c697479",
      "sha256": "19a911a6eb9b79ff986bddb45fc24ea17f6e01b2589f33ebcefc3fcebf63e128",
      "bytes": 356
    },
    "policy": {
      "hex": "434f4e534f4c452e4944454e544954592e47524f55502e504f4c4943592e4845414400000100000000000000000000000000000003000000000000000000000000000000040000000000000001000000176e61746976652d67726f75702d70726f636573732d76317e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e51297694172e4cf6afcbfb97e879eff3d02061db35cf4a0a02042724b710351dcc60b25000000000000000000000000000000010000000000000000000000000000000f19a911a6eb9b79ff986bddb45fc24ea17f6e01b2589f33ebcefc3fcebf63e1280000000000000000000000000000000e00065cbceed7efc0",
      "sha256": "748a626ff665ced3a5bd1165facad0ef297af222154b051246bf7abe25c838b0",
      "bytes": 320
    },
    "version": {
      "hex": "434f4e534f4c452e4944454e544954592e50524f434553532e56455253494f4e00000100000000000000000000000000000003000000000000000000000000000000040000000000000000000000000000000500000000000000010000001947524f55505f56455249464945525f50524f434553535f5631000000000000000000000000000000010000000000000000000000000000000900000000000000010000000000000001748a626ff665ced3a5bd1165facad0ef297af222154b051246bf7abe25c838b00000000000000000000000000000000f19a911a6eb9b79ff986bddb45fc24ea17f6e01b2589f33ebcefc3fcebf63e12800065cbceec8ad8000065cd10d38a400000100000014eb8f85eba6bd20eab280eca69d20eca088ecb0a800010000000e636c61696d616e74206d61746368000000126163636f756e7420706f7373657373696f6e00000011706879736963616c2065766964656e636500000013636f6e747261646963746f727920636c61696d000000197175616c696669636174696f6e20696e737472756374696f6e00000017657363616c6174696f6e2061646a756469636174696f6e000000116d696e696d616c20726574656e74696f6e00000018726563697069656e7420726573706f6e736962696c697479",
      "sha256": "98bd7ea46629f0169a6eac2015c367196bc7e30431b5af269323e5d189b9b4e0",
      "bytes": 481
    },
    "head": {
      "hex": "434f4e534f4c452e4944454e544954592e50524f434553532e484541440000010000000000000000000000000000000300000000000000000000000000000004000000000000000000000000000000050000000000000001000000000000000198bd7ea46629f0169a6eac2015c367196bc7e30431b5af269323e5d189b9b4e0000100065cd10d38a400000000000000000000000000000000010000000000000000000000000000000f19a911a6eb9b79ff986bddb45fc24ea17f6e01b2589f33ebcefc3fcebf63e1280000000000000000000000000000000e00065cbceed7efc0",
      "sha256": "938698d8c6ad82400ead9e9c9d887179904ab0fa743182fe98c15c9fab5a1c28",
      "bytes": 226
    }
  },
  "cases": {
    "first_valid": {
      "family": "adopt",
      "meta": {
        "actor": 1,
        "designation_receipt": 9,
        "designation_revision": 1,
        "command": 2,
        "input_digest": "ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe",
        "admitted_at_us": 1790816400123456,
        "expiry_us": 1790902800000000,
        "title": "독립 검증 절차"
      },
      "command": {
        "hex": "434f4e534f4c452e4944454e544954592e47524f55500000010001000000000000000000000000000000010000000000000000000000000000000200000000000000000000000000000003000000000000000000000000000000040000000000000007000000000000000000000000000000000000000000000005000000000000000000065cd10d38a400000100000014eb8f85eba6bd20eab280eca69d20eca088ecb0a800010000000e636c61696d616e74206d61746368000000126163636f756e7420706f7373657373696f6e00000011706879736963616c2065766964656e636500000013636f6e747261646963746f727920636c61696d000000197175616c696669636174696f6e20696e737472756374696f6e00000017657363616c6174696f6e2061646a756469636174696f6e000000116d696e696d616c20726574656e74696f6e00000018726563697069656e7420726573706f6e736962696c697479",
        "sha256": "ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe",
        "bytes": 356
      },
      "policy": {
        "hex": "434f4e534f4c452e4944454e544954592e47524f55502e504f4c4943592e4845414400000100000000000000000000000000000003000000000000000000000000000000040000000000000001000000176e61746976652d67726f75702d70726f636573732d76317e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e51297694172e4cf6afcbfb97e879eff3d02061db35cf4a0a02042724b710351dcc60b250000000000000000000000000000000100000000000000000000000000000002ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe0000000000000000000000000000000700065cbcef850ece",
        "sha256": "c4507daab34ef2a018b2f906e310eac2d76a3f01763e5b9a5268eba72694111d",
        "bytes": 320
      },
      "version": {
        "hex": "434f4e534f4c452e4944454e544954592e50524f434553532e56455253494f4e00000100000000000000000000000000000003000000000000000000000000000000040000000000000000000000000000000500000000000000010000001947524f55505f56455249464945525f50524f434553535f5631000000000000000000000000000000010000000000000000000000000000000900000000000000010000000000000001c4507daab34ef2a018b2f906e310eac2d76a3f01763e5b9a5268eba72694111d00000000000000000000000000000002ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe00065cbcef63264000065cd10d38a400000100000014eb8f85eba6bd20eab280eca69d20eca088ecb0a800010000000e636c61696d616e74206d61746368000000126163636f756e7420706f7373657373696f6e00000011706879736963616c2065766964656e636500000013636f6e747261646963746f727920636c61696d000000197175616c696669636174696f6e20696e737472756374696f6e00000017657363616c6174696f6e2061646a756469636174696f6e000000116d696e696d616c20726574656e74696f6e00000018726563697069656e7420726573706f6e736962696c697479",
        "sha256": "c01f8908bef0d77d52b989ae9e8d4142c21e49d3fb6422e6a48e6b7dbfc90a9e",
        "bytes": 481
      },
      "head": {
        "hex": "434f4e534f4c452e4944454e544954592e50524f434553532e4845414400000100000000000000000000000000000003000000000000000000000000000000040000000000000000000000000000000500000000000000010000000000000001c01f8908bef0d77d52b989ae9e8d4142c21e49d3fb6422e6a48e6b7dbfc90a9e000100065cd10d38a4000000000000000000000000000000000100000000000000000000000000000002ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe0000000000000000000000000000000700065cbcef850ece",
        "sha256": "9c5ee0e6463543162b3686b799f4e726d4e4b55801e9729e3211624152f1cd29",
        "bytes": 226
      },
      "result": {
        "hex": "434f4e534f4c452e4944454e544954592e50524f434553532e524553554c54000002000000000000000000000000000000010000000000000000000000000000000200000000000000000000000000000003000000000000000000000000000000040001ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe0000000000000000000000000000000600000000000000000000000000000007000100065cbcef63264000065cbcef850ece000000000000000000000000000000080000000000000001000000000000000000000000000000090000000000000001000000000000000700010000000000000001c4507daab34ef2a018b2f906e310eac2d76a3f01763e5b9a5268eba72694111d000000176e61746976652d67726f75702d70726f636573732d76317e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e512976900000000000000014172e4cf6afcbfb97e879eff3d02061db35cf4a0a02042724b710351dcc60b2500000006342e31332e3000000003342e350000000000000000000000000000000500010000000000000000000000000000000500000000000000010000000000000001c01f8908bef0d77d52b989ae9e8d4142c21e49d3fb6422e6a48e6b7dbfc90a9e9c5ee0e6463543162b3686b799f4e726d4e4b55801e9729e3211624152f1cd29000100065cd10d38a40000000000000000150000007b",
        "sha256": "265fc32fe611ae01977cf3a8689cfbf7b91aa39b852b0d94b2ce6d1c4863559d",
        "bytes": 596
      }
    },
    "first_content": {
      "family": "adopt",
      "meta": {
        "actor": 1,
        "designation_receipt": 9,
        "designation_revision": 1,
        "command": 2,
        "input_digest": "ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe",
        "admitted_at_us": 1790816400123456,
        "expiry_us": 1790902800000000,
        "title": "다른 검증 절차"
      },
      "command": {
        "hex": "434f4e534f4c452e4944454e544954592e47524f55500000010001000000000000000000000000000000010000000000000000000000000000000200000000000000000000000000000003000000000000000000000000000000040000000000000007000000000000000000000000000000000000000000000005000000000000000000065cd10d38a400000100000014eb8f85eba6bd20eab280eca69d20eca088ecb0a800010000000e636c61696d616e74206d61746368000000126163636f756e7420706f7373657373696f6e00000011706879736963616c2065766964656e636500000013636f6e747261646963746f727920636c61696d000000197175616c696669636174696f6e20696e737472756374696f6e00000017657363616c6174696f6e2061646a756469636174696f6e000000116d696e696d616c20726574656e74696f6e00000018726563697069656e7420726573706f6e736962696c697479",
        "sha256": "ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe",
        "bytes": 356
      },
      "policy": {
        "hex": "434f4e534f4c452e4944454e544954592e47524f55502e504f4c4943592e4845414400000100000000000000000000000000000003000000000000000000000000000000040000000000000001000000176e61746976652d67726f75702d70726f636573732d76317e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e51297694172e4cf6afcbfb97e879eff3d02061db35cf4a0a02042724b710351dcc60b250000000000000000000000000000000100000000000000000000000000000002ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe0000000000000000000000000000000700065cbcef850ece",
        "sha256": "c4507daab34ef2a018b2f906e310eac2d76a3f01763e5b9a5268eba72694111d",
        "bytes": 320
      },
      "version": {
        "hex": "434f4e534f4c452e4944454e544954592e50524f434553532e56455253494f4e00000100000000000000000000000000000003000000000000000000000000000000040000000000000000000000000000000500000000000000010000001947524f55505f56455249464945525f50524f434553535f5631000000000000000000000000000000010000000000000000000000000000000900000000000000010000000000000001c4507daab34ef2a018b2f906e310eac2d76a3f01763e5b9a5268eba72694111d00000000000000000000000000000002ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe00065cbcef63264000065cd10d38a400000100000014eb8ba4eba5b820eab280eca69d20eca088ecb0a800010000000e636c61696d616e74206d61746368000000126163636f756e7420706f7373657373696f6e00000011706879736963616c2065766964656e636500000013636f6e747261646963746f727920636c61696d000000197175616c696669636174696f6e20696e737472756374696f6e00000017657363616c6174696f6e2061646a756469636174696f6e000000116d696e696d616c20726574656e74696f6e00000018726563697069656e7420726573706f6e736962696c697479",
        "sha256": "0a37de595498f20741b24df2bb6ddfb9f4c446a81486f004d2423f741d1b0182",
        "bytes": 481
      },
      "head": {
        "hex": "434f4e534f4c452e4944454e544954592e50524f434553532e48454144000001000000000000000000000000000000030000000000000000000000000000000400000000000000000000000000000005000000000000000100000000000000010a37de595498f20741b24df2bb6ddfb9f4c446a81486f004d2423f741d1b0182000100065cd10d38a4000000000000000000000000000000000100000000000000000000000000000002ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe0000000000000000000000000000000700065cbcef850ece",
        "sha256": "8b5b9c4c7cc311fee6ef90bbdda6b6feb536f37dad46cda706fad367bb59d377",
        "bytes": 226
      },
      "result": {
        "hex": "434f4e534f4c452e4944454e544954592e50524f434553532e524553554c54000002000000000000000000000000000000010000000000000000000000000000000200000000000000000000000000000003000000000000000000000000000000040001ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe0000000000000000000000000000000600000000000000000000000000000007000100065cbcef63264000065cbcef850ece000000000000000000000000000000080000000000000001000000000000000000000000000000090000000000000001000000000000000700010000000000000001c4507daab34ef2a018b2f906e310eac2d76a3f01763e5b9a5268eba72694111d000000176e61746976652d67726f75702d70726f636573732d76317e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e512976900000000000000014172e4cf6afcbfb97e879eff3d02061db35cf4a0a02042724b710351dcc60b2500000006342e31332e3000000003342e3500000000000000000000000000000005000100000000000000000000000000000005000000000000000100000000000000010a37de595498f20741b24df2bb6ddfb9f4c446a81486f004d2423f741d1b01828b5b9c4c7cc311fee6ef90bbdda6b6feb536f37dad46cda706fad367bb59d377000100065cd10d38a40000000000000000150000007b",
        "sha256": "40d926ec48f807d4503cb601a1cc1a4065ce0072a05ff05465a8193c4fcd2e58",
        "bytes": 596
      }
    },
    "first_expiry": {
      "family": "adopt",
      "meta": {
        "actor": 1,
        "designation_receipt": 9,
        "designation_revision": 1,
        "command": 2,
        "input_digest": "ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe",
        "admitted_at_us": 1790816400123456,
        "expiry_us": 1790989200000000,
        "title": "독립 검증 절차"
      },
      "command": {
        "hex": "434f4e534f4c452e4944454e544954592e47524f55500000010001000000000000000000000000000000010000000000000000000000000000000200000000000000000000000000000003000000000000000000000000000000040000000000000007000000000000000000000000000000000000000000000005000000000000000000065cd10d38a400000100000014eb8f85eba6bd20eab280eca69d20eca088ecb0a800010000000e636c61696d616e74206d61746368000000126163636f756e7420706f7373657373696f6e00000011706879736963616c2065766964656e636500000013636f6e747261646963746f727920636c61696d000000197175616c696669636174696f6e20696e737472756374696f6e00000017657363616c6174696f6e2061646a756469636174696f6e000000116d696e696d616c20726574656e74696f6e00000018726563697069656e7420726573706f6e736962696c697479",
        "sha256": "ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe",
        "bytes": 356
      },
      "policy": {
        "hex": "434f4e534f4c452e4944454e544954592e47524f55502e504f4c4943592e4845414400000100000000000000000000000000000003000000000000000000000000000000040000000000000001000000176e61746976652d67726f75702d70726f636573732d76317e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e51297694172e4cf6afcbfb97e879eff3d02061db35cf4a0a02042724b710351dcc60b250000000000000000000000000000000100000000000000000000000000000002ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe0000000000000000000000000000000700065cbcef850ece",
        "sha256": "c4507daab34ef2a018b2f906e310eac2d76a3f01763e5b9a5268eba72694111d",
        "bytes": 320
      },
      "version": {
        "hex": "434f4e534f4c452e4944454e544954592e50524f434553532e56455253494f4e00000100000000000000000000000000000003000000000000000000000000000000040000000000000000000000000000000500000000000000010000001947524f55505f56455249464945525f50524f434553535f5631000000000000000000000000000000010000000000000000000000000000000900000000000000010000000000000001c4507daab34ef2a018b2f906e310eac2d76a3f01763e5b9a5268eba72694111d00000000000000000000000000000002ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe00065cbcef63264000065ce52b100400000100000014eb8f85eba6bd20eab280eca69d20eca088ecb0a800010000000e636c61696d616e74206d61746368000000126163636f756e7420706f7373657373696f6e00000011706879736963616c2065766964656e636500000013636f6e747261646963746f727920636c61696d000000197175616c696669636174696f6e20696e737472756374696f6e00000017657363616c6174696f6e2061646a756469636174696f6e000000116d696e696d616c20726574656e74696f6e00000018726563697069656e7420726573706f6e736962696c697479",
        "sha256": "e6de1ffcd52d88040c1923d268665b917c9ed544ee19c22e602899d2be2f9901",
        "bytes": 481
      },
      "head": {
        "hex": "434f4e534f4c452e4944454e544954592e50524f434553532e4845414400000100000000000000000000000000000003000000000000000000000000000000040000000000000000000000000000000500000000000000010000000000000001e6de1ffcd52d88040c1923d268665b917c9ed544ee19c22e602899d2be2f9901000100065ce52b1004000000000000000000000000000000000100000000000000000000000000000002ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe0000000000000000000000000000000700065cbcef850ece",
        "sha256": "88393a52fd89e7bdaac9c454675d7323178c5d746aa4e0614c8ad43a6cdcc60b",
        "bytes": 226
      },
      "result": {
        "hex": "434f4e534f4c452e4944454e544954592e50524f434553532e524553554c54000002000000000000000000000000000000010000000000000000000000000000000200000000000000000000000000000003000000000000000000000000000000040001ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe0000000000000000000000000000000600000000000000000000000000000007000100065cbcef63264000065cbcef850ece000000000000000000000000000000080000000000000001000000000000000000000000000000090000000000000001000000000000000700010000000000000001c4507daab34ef2a018b2f906e310eac2d76a3f01763e5b9a5268eba72694111d000000176e61746976652d67726f75702d70726f636573732d76317e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e512976900000000000000014172e4cf6afcbfb97e879eff3d02061db35cf4a0a02042724b710351dcc60b2500000006342e31332e3000000003342e350000000000000000000000000000000500010000000000000000000000000000000500000000000000010000000000000001e6de1ffcd52d88040c1923d268665b917c9ed544ee19c22e602899d2be2f990188393a52fd89e7bdaac9c454675d7323178c5d746aa4e0614c8ad43a6cdcc60b000100065ce52b10040000000000000000150000007b",
        "sha256": "cfc0448e65da1f3b18cbd8d48d93c49e248aa24adedefe9eece0cd12142d11ea",
        "bytes": 596
      }
    },
    "first_actor": {
      "family": "adopt",
      "meta": {
        "actor": 101,
        "designation_receipt": 9,
        "designation_revision": 1,
        "command": 2,
        "input_digest": "ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe",
        "admitted_at_us": 1790816400123456,
        "expiry_us": 1790902800000000,
        "title": "독립 검증 절차"
      },
      "command": {
        "hex": "434f4e534f4c452e4944454e544954592e47524f55500000010001000000000000000000000000000000010000000000000000000000000000000200000000000000000000000000000003000000000000000000000000000000040000000000000007000000000000000000000000000000000000000000000005000000000000000000065cd10d38a400000100000014eb8f85eba6bd20eab280eca69d20eca088ecb0a800010000000e636c61696d616e74206d61746368000000126163636f756e7420706f7373657373696f6e00000011706879736963616c2065766964656e636500000013636f6e747261646963746f727920636c61696d000000197175616c696669636174696f6e20696e737472756374696f6e00000017657363616c6174696f6e2061646a756469636174696f6e000000116d696e696d616c20726574656e74696f6e00000018726563697069656e7420726573706f6e736962696c697479",
        "sha256": "ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe",
        "bytes": 356
      },
      "policy": {
        "hex": "434f4e534f4c452e4944454e544954592e47524f55502e504f4c4943592e4845414400000100000000000000000000000000000003000000000000000000000000000000040000000000000001000000176e61746976652d67726f75702d70726f636573732d76317e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e51297694172e4cf6afcbfb97e879eff3d02061db35cf4a0a02042724b710351dcc60b250000000000000000000000000000000100000000000000000000000000000002ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe0000000000000000000000000000000700065cbcef850ece",
        "sha256": "c4507daab34ef2a018b2f906e310eac2d76a3f01763e5b9a5268eba72694111d",
        "bytes": 320
      },
      "version": {
        "hex": "434f4e534f4c452e4944454e544954592e50524f434553532e56455253494f4e00000100000000000000000000000000000003000000000000000000000000000000040000000000000000000000000000000500000000000000010000001947524f55505f56455249464945525f50524f434553535f5631000000000000000000000000000000650000000000000000000000000000000900000000000000010000000000000001c4507daab34ef2a018b2f906e310eac2d76a3f01763e5b9a5268eba72694111d00000000000000000000000000000002ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe00065cbcef63264000065cd10d38a400000100000014eb8f85eba6bd20eab280eca69d20eca088ecb0a800010000000e636c61696d616e74206d61746368000000126163636f756e7420706f7373657373696f6e00000011706879736963616c2065766964656e636500000013636f6e747261646963746f727920636c61696d000000197175616c696669636174696f6e20696e737472756374696f6e00000017657363616c6174696f6e2061646a756469636174696f6e000000116d696e696d616c20726574656e74696f6e00000018726563697069656e7420726573706f6e736962696c697479",
        "sha256": "98e88fcd88131a1d4560268d14d1d2c6890035f58d57fc108fb8496f0572ed52",
        "bytes": 481
      },
      "head": {
        "hex": "434f4e534f4c452e4944454e544954592e50524f434553532e484541440000010000000000000000000000000000000300000000000000000000000000000004000000000000000000000000000000050000000000000001000000000000000198e88fcd88131a1d4560268d14d1d2c6890035f58d57fc108fb8496f0572ed52000100065cd10d38a4000000000000000000000000000000000100000000000000000000000000000002ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe0000000000000000000000000000000700065cbcef850ece",
        "sha256": "f937390ce0c353ddc0f6d71c5719b8255406874e9d935ab12e23631307a37df6",
        "bytes": 226
      },
      "result": {
        "hex": "434f4e534f4c452e4944454e544954592e50524f434553532e524553554c54000002000000000000000000000000000000010000000000000000000000000000000200000000000000000000000000000003000000000000000000000000000000040001ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe0000000000000000000000000000000600000000000000000000000000000007000100065cbcef63264000065cbcef850ece000000000000000000000000000000080000000000000001000000000000000000000000000000090000000000000001000000000000000700010000000000000001c4507daab34ef2a018b2f906e310eac2d76a3f01763e5b9a5268eba72694111d000000176e61746976652d67726f75702d70726f636573732d76317e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e512976900000000000000014172e4cf6afcbfb97e879eff3d02061db35cf4a0a02042724b710351dcc60b2500000006342e31332e3000000003342e35000000000000000000000000000000050001000000000000000000000000000000050000000000000001000000000000000198e88fcd88131a1d4560268d14d1d2c6890035f58d57fc108fb8496f0572ed52f937390ce0c353ddc0f6d71c5719b8255406874e9d935ab12e23631307a37df6000100065cd10d38a40000000000000000150000007b",
        "sha256": "87d563029beabdc2414293239bb558953553adf1a35caca7b71c012dc6dfc0a9",
        "bytes": 596
      }
    },
    "first_designation_receipt": {
      "family": "adopt",
      "meta": {
        "actor": 1,
        "designation_receipt": 109,
        "designation_revision": 1,
        "command": 2,
        "input_digest": "ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe",
        "admitted_at_us": 1790816400123456,
        "expiry_us": 1790902800000000,
        "title": "독립 검증 절차"
      },
      "command": {
        "hex": "434f4e534f4c452e4944454e544954592e47524f55500000010001000000000000000000000000000000010000000000000000000000000000000200000000000000000000000000000003000000000000000000000000000000040000000000000007000000000000000000000000000000000000000000000005000000000000000000065cd10d38a400000100000014eb8f85eba6bd20eab280eca69d20eca088ecb0a800010000000e636c61696d616e74206d61746368000000126163636f756e7420706f7373657373696f6e00000011706879736963616c2065766964656e636500000013636f6e747261646963746f727920636c61696d000000197175616c696669636174696f6e20696e737472756374696f6e00000017657363616c6174696f6e2061646a756469636174696f6e000000116d696e696d616c20726574656e74696f6e00000018726563697069656e7420726573706f6e736962696c697479",
        "sha256": "ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe",
        "bytes": 356
      },
      "policy": {
        "hex": "434f4e534f4c452e4944454e544954592e47524f55502e504f4c4943592e4845414400000100000000000000000000000000000003000000000000000000000000000000040000000000000001000000176e61746976652d67726f75702d70726f636573732d76317e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e51297694172e4cf6afcbfb97e879eff3d02061db35cf4a0a02042724b710351dcc60b250000000000000000000000000000000100000000000000000000000000000002ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe0000000000000000000000000000000700065cbcef850ece",
        "sha256": "c4507daab34ef2a018b2f906e310eac2d76a3f01763e5b9a5268eba72694111d",
        "bytes": 320
      },
      "version": {
        "hex": "434f4e534f4c452e4944454e544954592e50524f434553532e56455253494f4e00000100000000000000000000000000000003000000000000000000000000000000040000000000000000000000000000000500000000000000010000001947524f55505f56455249464945525f50524f434553535f5631000000000000000000000000000000010000000000000000000000000000006d00000000000000010000000000000001c4507daab34ef2a018b2f906e310eac2d76a3f01763e5b9a5268eba72694111d00000000000000000000000000000002ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe00065cbcef63264000065cd10d38a400000100000014eb8f85eba6bd20eab280eca69d20eca088ecb0a800010000000e636c61696d616e74206d61746368000000126163636f756e7420706f7373657373696f6e00000011706879736963616c2065766964656e636500000013636f6e747261646963746f727920636c61696d000000197175616c696669636174696f6e20696e737472756374696f6e00000017657363616c6174696f6e2061646a756469636174696f6e000000116d696e696d616c20726574656e74696f6e00000018726563697069656e7420726573706f6e736962696c697479",
        "sha256": "8b58ba177b0d330cca9fe94e7d4a44518effcf7499883a07714f76fe22d4d9ea",
        "bytes": 481
      },
      "head": {
        "hex": "434f4e534f4c452e4944454e544954592e50524f434553532e48454144000001000000000000000000000000000000030000000000000000000000000000000400000000000000000000000000000005000000000000000100000000000000018b58ba177b0d330cca9fe94e7d4a44518effcf7499883a07714f76fe22d4d9ea000100065cd10d38a4000000000000000000000000000000000100000000000000000000000000000002ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe0000000000000000000000000000000700065cbcef850ece",
        "sha256": "7575b35d860bbb0fc59f09266e81bd1cc9cb3fa18d6cc132c26c83f6b98eb994",
        "bytes": 226
      },
      "result": {
        "hex": "434f4e534f4c452e4944454e544954592e50524f434553532e524553554c54000002000000000000000000000000000000010000000000000000000000000000000200000000000000000000000000000003000000000000000000000000000000040001ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe0000000000000000000000000000000600000000000000000000000000000007000100065cbcef63264000065cbcef850ece000000000000000000000000000000080000000000000001000000000000000000000000000000090000000000000001000000000000000700010000000000000001c4507daab34ef2a018b2f906e310eac2d76a3f01763e5b9a5268eba72694111d000000176e61746976652d67726f75702d70726f636573732d76317e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e512976900000000000000014172e4cf6afcbfb97e879eff3d02061db35cf4a0a02042724b710351dcc60b2500000006342e31332e3000000003342e3500000000000000000000000000000005000100000000000000000000000000000005000000000000000100000000000000018b58ba177b0d330cca9fe94e7d4a44518effcf7499883a07714f76fe22d4d9ea7575b35d860bbb0fc59f09266e81bd1cc9cb3fa18d6cc132c26c83f6b98eb994000100065cd10d38a40000000000000000150000007b",
        "sha256": "ef82d870762d821b3132bc3a307e6f5b1a136c212472eccb98d94e5c3cb4ae5a",
        "bytes": 596
      }
    },
    "first_designation_revision": {
      "family": "adopt",
      "meta": {
        "actor": 1,
        "designation_receipt": 9,
        "designation_revision": 2,
        "command": 2,
        "input_digest": "ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe",
        "admitted_at_us": 1790816400123456,
        "expiry_us": 1790902800000000,
        "title": "독립 검증 절차"
      },
      "command": {
        "hex": "434f4e534f4c452e4944454e544954592e47524f55500000010001000000000000000000000000000000010000000000000000000000000000000200000000000000000000000000000003000000000000000000000000000000040000000000000007000000000000000000000000000000000000000000000005000000000000000000065cd10d38a400000100000014eb8f85eba6bd20eab280eca69d20eca088ecb0a800010000000e636c61696d616e74206d61746368000000126163636f756e7420706f7373657373696f6e00000011706879736963616c2065766964656e636500000013636f6e747261646963746f727920636c61696d000000197175616c696669636174696f6e20696e737472756374696f6e00000017657363616c6174696f6e2061646a756469636174696f6e000000116d696e696d616c20726574656e74696f6e00000018726563697069656e7420726573706f6e736962696c697479",
        "sha256": "ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe",
        "bytes": 356
      },
      "policy": {
        "hex": "434f4e534f4c452e4944454e544954592e47524f55502e504f4c4943592e4845414400000100000000000000000000000000000003000000000000000000000000000000040000000000000001000000176e61746976652d67726f75702d70726f636573732d76317e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e51297694172e4cf6afcbfb97e879eff3d02061db35cf4a0a02042724b710351dcc60b250000000000000000000000000000000100000000000000000000000000000002ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe0000000000000000000000000000000700065cbcef850ece",
        "sha256": "c4507daab34ef2a018b2f906e310eac2d76a3f01763e5b9a5268eba72694111d",
        "bytes": 320
      },
      "version": {
        "hex": "434f4e534f4c452e4944454e544954592e50524f434553532e56455253494f4e00000100000000000000000000000000000003000000000000000000000000000000040000000000000000000000000000000500000000000000010000001947524f55505f56455249464945525f50524f434553535f5631000000000000000000000000000000010000000000000000000000000000000900000000000000020000000000000001c4507daab34ef2a018b2f906e310eac2d76a3f01763e5b9a5268eba72694111d00000000000000000000000000000002ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe00065cbcef63264000065cd10d38a400000100000014eb8f85eba6bd20eab280eca69d20eca088ecb0a800010000000e636c61696d616e74206d61746368000000126163636f756e7420706f7373657373696f6e00000011706879736963616c2065766964656e636500000013636f6e747261646963746f727920636c61696d000000197175616c696669636174696f6e20696e737472756374696f6e00000017657363616c6174696f6e2061646a756469636174696f6e000000116d696e696d616c20726574656e74696f6e00000018726563697069656e7420726573706f6e736962696c697479",
        "sha256": "5915a63041575a9ae0bf4c42314dd4918ae848ce6756ae79c1dcd55c31b33dbe",
        "bytes": 481
      },
      "head": {
        "hex": "434f4e534f4c452e4944454e544954592e50524f434553532e48454144000001000000000000000000000000000000030000000000000000000000000000000400000000000000000000000000000005000000000000000100000000000000015915a63041575a9ae0bf4c42314dd4918ae848ce6756ae79c1dcd55c31b33dbe000100065cd10d38a4000000000000000000000000000000000100000000000000000000000000000002ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe0000000000000000000000000000000700065cbcef850ece",
        "sha256": "a9270fb22753f737e4bce6c27d960596100c9104185a6a9c7d14b308616d9da8",
        "bytes": 226
      },
      "result": {
        "hex": "434f4e534f4c452e4944454e544954592e50524f434553532e524553554c54000002000000000000000000000000000000010000000000000000000000000000000200000000000000000000000000000003000000000000000000000000000000040001ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe0000000000000000000000000000000600000000000000000000000000000007000100065cbcef63264000065cbcef850ece000000000000000000000000000000080000000000000001000000000000000000000000000000090000000000000001000000000000000700010000000000000001c4507daab34ef2a018b2f906e310eac2d76a3f01763e5b9a5268eba72694111d000000176e61746976652d67726f75702d70726f636573732d76317e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e512976900000000000000014172e4cf6afcbfb97e879eff3d02061db35cf4a0a02042724b710351dcc60b2500000006342e31332e3000000003342e3500000000000000000000000000000005000100000000000000000000000000000005000000000000000100000000000000015915a63041575a9ae0bf4c42314dd4918ae848ce6756ae79c1dcd55c31b33dbea9270fb22753f737e4bce6c27d960596100c9104185a6a9c7d14b308616d9da8000100065cd10d38a40000000000000000150000007b",
        "sha256": "6b66a5c9c3952a136f903e8a26bdd7cedb7c97ee674a8294da374aedb2a7661b",
        "bytes": 596
      }
    },
    "first_command": {
      "family": "adopt",
      "meta": {
        "actor": 1,
        "designation_receipt": 9,
        "designation_revision": 1,
        "command": 102,
        "input_digest": "ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe",
        "admitted_at_us": 1790816400123456,
        "expiry_us": 1790902800000000,
        "title": "독립 검증 절차"
      },
      "command": {
        "hex": "434f4e534f4c452e4944454e544954592e47524f55500000010001000000000000000000000000000000010000000000000000000000000000000200000000000000000000000000000003000000000000000000000000000000040000000000000007000000000000000000000000000000000000000000000005000000000000000000065cd10d38a400000100000014eb8f85eba6bd20eab280eca69d20eca088ecb0a800010000000e636c61696d616e74206d61746368000000126163636f756e7420706f7373657373696f6e00000011706879736963616c2065766964656e636500000013636f6e747261646963746f727920636c61696d000000197175616c696669636174696f6e20696e737472756374696f6e00000017657363616c6174696f6e2061646a756469636174696f6e000000116d696e696d616c20726574656e74696f6e00000018726563697069656e7420726573706f6e736962696c697479",
        "sha256": "ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe",
        "bytes": 356
      },
      "policy": {
        "hex": "434f4e534f4c452e4944454e544954592e47524f55502e504f4c4943592e4845414400000100000000000000000000000000000003000000000000000000000000000000040000000000000001000000176e61746976652d67726f75702d70726f636573732d76317e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e51297694172e4cf6afcbfb97e879eff3d02061db35cf4a0a02042724b710351dcc60b250000000000000000000000000000000100000000000000000000000000000002ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe0000000000000000000000000000000700065cbcef850ece",
        "sha256": "c4507daab34ef2a018b2f906e310eac2d76a3f01763e5b9a5268eba72694111d",
        "bytes": 320
      },
      "version": {
        "hex": "434f4e534f4c452e4944454e544954592e50524f434553532e56455253494f4e00000100000000000000000000000000000003000000000000000000000000000000040000000000000000000000000000000500000000000000010000001947524f55505f56455249464945525f50524f434553535f5631000000000000000000000000000000010000000000000000000000000000000900000000000000010000000000000001c4507daab34ef2a018b2f906e310eac2d76a3f01763e5b9a5268eba72694111d00000000000000000000000000000066ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe00065cbcef63264000065cd10d38a400000100000014eb8f85eba6bd20eab280eca69d20eca088ecb0a800010000000e636c61696d616e74206d61746368000000126163636f756e7420706f7373657373696f6e00000011706879736963616c2065766964656e636500000013636f6e747261646963746f727920636c61696d000000197175616c696669636174696f6e20696e737472756374696f6e00000017657363616c6174696f6e2061646a756469636174696f6e000000116d696e696d616c20726574656e74696f6e00000018726563697069656e7420726573706f6e736962696c697479",
        "sha256": "a402876a0587d5dfbe07a554ed77ed3c28d30823f5c22335e4d57c117a12091e",
        "bytes": 481
      },
      "head": {
        "hex": "434f4e534f4c452e4944454e544954592e50524f434553532e4845414400000100000000000000000000000000000003000000000000000000000000000000040000000000000000000000000000000500000000000000010000000000000001a402876a0587d5dfbe07a554ed77ed3c28d30823f5c22335e4d57c117a12091e000100065cd10d38a4000000000000000000000000000000000100000000000000000000000000000002ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe0000000000000000000000000000000700065cbcef850ece",
        "sha256": "00a52661267c64884027f71b3f17f309b243be528d4639953f627d8e66e2fce5",
        "bytes": 226
      },
      "result": {
        "hex": "434f4e534f4c452e4944454e544954592e50524f434553532e524553554c54000002000000000000000000000000000000010000000000000000000000000000000200000000000000000000000000000003000000000000000000000000000000040001ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe0000000000000000000000000000000600000000000000000000000000000007000100065cbcef63264000065cbcef850ece000000000000000000000000000000080000000000000001000000000000000000000000000000090000000000000001000000000000000700010000000000000001c4507daab34ef2a018b2f906e310eac2d76a3f01763e5b9a5268eba72694111d000000176e61746976652d67726f75702d70726f636573732d76317e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e512976900000000000000014172e4cf6afcbfb97e879eff3d02061db35cf4a0a02042724b710351dcc60b2500000006342e31332e3000000003342e350000000000000000000000000000000500010000000000000000000000000000000500000000000000010000000000000001a402876a0587d5dfbe07a554ed77ed3c28d30823f5c22335e4d57c117a12091e00a52661267c64884027f71b3f17f309b243be528d4639953f627d8e66e2fce5000100065cd10d38a40000000000000000150000007b",
        "sha256": "4ee9c7bfd3be900d9915372eca97424a0bf2ae7eed5d530e5115bed8f7557f71",
        "bytes": 596
      }
    },
    "first_input_digest": {
      "family": "adopt",
      "meta": {
        "actor": 1,
        "designation_receipt": 9,
        "designation_revision": 1,
        "command": 2,
        "input_digest": "97ccb4f47f20600dc3af5e1d007ead774ab63b4e7bb399697a43c73f454c50ea",
        "admitted_at_us": 1790816400123456,
        "expiry_us": 1790902800000000,
        "title": "독립 검증 절차"
      },
      "command": {
        "hex": "434f4e534f4c452e4944454e544954592e47524f55500000010001000000000000000000000000000000010000000000000000000000000000000200000000000000000000000000000003000000000000000000000000000000040000000000000007000000000000000000000000000000000000000000000005000000000000000000065cd10d38a400000100000014eb8f85eba6bd20eab280eca69d20eca088ecb0a800010000000e636c61696d616e74206d61746368000000126163636f756e7420706f7373657373696f6e00000011706879736963616c2065766964656e636500000013636f6e747261646963746f727920636c61696d000000197175616c696669636174696f6e20696e737472756374696f6e00000017657363616c6174696f6e2061646a756469636174696f6e000000116d696e696d616c20726574656e74696f6e00000018726563697069656e7420726573706f6e736962696c697479",
        "sha256": "ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe",
        "bytes": 356
      },
      "policy": {
        "hex": "434f4e534f4c452e4944454e544954592e47524f55502e504f4c4943592e4845414400000100000000000000000000000000000003000000000000000000000000000000040000000000000001000000176e61746976652d67726f75702d70726f636573732d76317e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e51297694172e4cf6afcbfb97e879eff3d02061db35cf4a0a02042724b710351dcc60b250000000000000000000000000000000100000000000000000000000000000002ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe0000000000000000000000000000000700065cbcef850ece",
        "sha256": "c4507daab34ef2a018b2f906e310eac2d76a3f01763e5b9a5268eba72694111d",
        "bytes": 320
      },
      "version": {
        "hex": "434f4e534f4c452e4944454e544954592e50524f434553532e56455253494f4e00000100000000000000000000000000000003000000000000000000000000000000040000000000000000000000000000000500000000000000010000001947524f55505f56455249464945525f50524f434553535f5631000000000000000000000000000000010000000000000000000000000000000900000000000000010000000000000001c4507daab34ef2a018b2f906e310eac2d76a3f01763e5b9a5268eba72694111d0000000000000000000000000000000297ccb4f47f20600dc3af5e1d007ead774ab63b4e7bb399697a43c73f454c50ea00065cbcef63264000065cd10d38a400000100000014eb8f85eba6bd20eab280eca69d20eca088ecb0a800010000000e636c61696d616e74206d61746368000000126163636f756e7420706f7373657373696f6e00000011706879736963616c2065766964656e636500000013636f6e747261646963746f727920636c61696d000000197175616c696669636174696f6e20696e737472756374696f6e00000017657363616c6174696f6e2061646a756469636174696f6e000000116d696e696d616c20726574656e74696f6e00000018726563697069656e7420726573706f6e736962696c697479",
        "sha256": "6fe282d841216dffe61b6f486251a67b757595be91e275b735f28aa229ac8374",
        "bytes": 481
      },
      "head": {
        "hex": "434f4e534f4c452e4944454e544954592e50524f434553532e48454144000001000000000000000000000000000000030000000000000000000000000000000400000000000000000000000000000005000000000000000100000000000000016fe282d841216dffe61b6f486251a67b757595be91e275b735f28aa229ac8374000100065cd10d38a4000000000000000000000000000000000100000000000000000000000000000002ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe0000000000000000000000000000000700065cbcef850ece",
        "sha256": "565b9c247bf09607f62eb461c2f8adbc7b2cd33e5dee296fb569db619102a248",
        "bytes": 226
      },
      "result": {
        "hex": "434f4e534f4c452e4944454e544954592e50524f434553532e524553554c54000002000000000000000000000000000000010000000000000000000000000000000200000000000000000000000000000003000000000000000000000000000000040001ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe0000000000000000000000000000000600000000000000000000000000000007000100065cbcef63264000065cbcef850ece000000000000000000000000000000080000000000000001000000000000000000000000000000090000000000000001000000000000000700010000000000000001c4507daab34ef2a018b2f906e310eac2d76a3f01763e5b9a5268eba72694111d000000176e61746976652d67726f75702d70726f636573732d76317e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e512976900000000000000014172e4cf6afcbfb97e879eff3d02061db35cf4a0a02042724b710351dcc60b2500000006342e31332e3000000003342e3500000000000000000000000000000005000100000000000000000000000000000005000000000000000100000000000000016fe282d841216dffe61b6f486251a67b757595be91e275b735f28aa229ac8374565b9c247bf09607f62eb461c2f8adbc7b2cd33e5dee296fb569db619102a248000100065cd10d38a40000000000000000150000007b",
        "sha256": "753359dcc8a32219c1f2ff473b4bf60c5342431216915032ac4fb4828035903f",
        "bytes": 596
      }
    },
    "first_admitted_at": {
      "family": "adopt",
      "meta": {
        "actor": 1,
        "designation_receipt": 9,
        "designation_revision": 1,
        "command": 2,
        "input_digest": "ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe",
        "admitted_at_us": 1790816400124456,
        "expiry_us": 1790902800000000,
        "title": "독립 검증 절차"
      },
      "command": {
        "hex": "434f4e534f4c452e4944454e544954592e47524f55500000010001000000000000000000000000000000010000000000000000000000000000000200000000000000000000000000000003000000000000000000000000000000040000000000000007000000000000000000000000000000000000000000000005000000000000000000065cd10d38a400000100000014eb8f85eba6bd20eab280eca69d20eca088ecb0a800010000000e636c61696d616e74206d61746368000000126163636f756e7420706f7373657373696f6e00000011706879736963616c2065766964656e636500000013636f6e747261646963746f727920636c61696d000000197175616c696669636174696f6e20696e737472756374696f6e00000017657363616c6174696f6e2061646a756469636174696f6e000000116d696e696d616c20726574656e74696f6e00000018726563697069656e7420726573706f6e736962696c697479",
        "sha256": "ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe",
        "bytes": 356
      },
      "policy": {
        "hex": "434f4e534f4c452e4944454e544954592e47524f55502e504f4c4943592e4845414400000100000000000000000000000000000003000000000000000000000000000000040000000000000001000000176e61746976652d67726f75702d70726f636573732d76317e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e51297694172e4cf6afcbfb97e879eff3d02061db35cf4a0a02042724b710351dcc60b250000000000000000000000000000000100000000000000000000000000000002ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe0000000000000000000000000000000700065cbcef850ece",
        "sha256": "c4507daab34ef2a018b2f906e310eac2d76a3f01763e5b9a5268eba72694111d",
        "bytes": 320
      },
      "version": {
        "hex": "434f4e534f4c452e4944454e544954592e50524f434553532e56455253494f4e00000100000000000000000000000000000003000000000000000000000000000000040000000000000000000000000000000500000000000000010000001947524f55505f56455249464945525f50524f434553535f5631000000000000000000000000000000010000000000000000000000000000000900000000000000010000000000000001c4507daab34ef2a018b2f906e310eac2d76a3f01763e5b9a5268eba72694111d00000000000000000000000000000002ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe00065cbcef632a2800065cd10d38a400000100000014eb8f85eba6bd20eab280eca69d20eca088ecb0a800010000000e636c61696d616e74206d61746368000000126163636f756e7420706f7373657373696f6e00000011706879736963616c2065766964656e636500000013636f6e747261646963746f727920636c61696d000000197175616c696669636174696f6e20696e737472756374696f6e00000017657363616c6174696f6e2061646a756469636174696f6e000000116d696e696d616c20726574656e74696f6e00000018726563697069656e7420726573706f6e736962696c697479",
        "sha256": "3b5a921974d866c8b185ef5ae9c8689bdc2f42d402d4a0967a0913f6971aa62e",
        "bytes": 481
      },
      "head": {
        "hex": "434f4e534f4c452e4944454e544954592e50524f434553532e48454144000001000000000000000000000000000000030000000000000000000000000000000400000000000000000000000000000005000000000000000100000000000000013b5a921974d866c8b185ef5ae9c8689bdc2f42d402d4a0967a0913f6971aa62e000100065cd10d38a4000000000000000000000000000000000100000000000000000000000000000002ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe0000000000000000000000000000000700065cbcef850ece",
        "sha256": "e730c8e66ad0788e953013c963678aaa390e8c8d9a01bd542fcd6db28d0f1fa3",
        "bytes": 226
      },
      "result": {
        "hex": "434f4e534f4c452e4944454e544954592e50524f434553532e524553554c54000002000000000000000000000000000000010000000000000000000000000000000200000000000000000000000000000003000000000000000000000000000000040001ffa92490117fe21783a3ad3093a94a0630cfe13ccb32561da6d1f00265e1febe0000000000000000000000000000000600000000000000000000000000000007000100065cbcef63264000065cbcef850ece000000000000000000000000000000080000000000000001000000000000000000000000000000090000000000000001000000000000000700010000000000000001c4507daab34ef2a018b2f906e310eac2d76a3f01763e5b9a5268eba72694111d000000176e61746976652d67726f75702d70726f636573732d76317e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e512976900000000000000014172e4cf6afcbfb97e879eff3d02061db35cf4a0a02042724b710351dcc60b2500000006342e31332e3000000003342e3500000000000000000000000000000005000100000000000000000000000000000005000000000000000100000000000000013b5a921974d866c8b185ef5ae9c8689bdc2f42d402d4a0967a0913f6971aa62ee730c8e66ad0788e953013c963678aaa390e8c8d9a01bd542fcd6db28d0f1fa3000100065cd10d38a40000000000000000150000007b",
        "sha256": "2d1cd5f07a66dda3f075dfbeab5160d18b34031641221b214c5842a70f475030",
        "bytes": 596
      }
    },
    "suspend_valid": {
      "family": "suspend",
      "meta": {
        "actor": 1,
        "designation_receipt": 9,
        "designation_revision": 1,
        "command": 15,
        "input_digest": "19a911a6eb9b79ff986bddb45fc24ea17f6e01b2589f33ebcefc3fcebf63e128",
        "admitted_at_us": 1790816390000000,
        "expiry_us": 1790902800000000,
        "title": "독립 검증 절차"
      },
      "reason": "검토한 원문 중지 사유",
      "command": {
        "hex": "434f4e534f4c452e4944454e544954592e47524f55500000010006000000000000000000000000000000010000000000000000000000000000000200000000000000000000000000000003000000000000000000000000000000040000000000000007000000000000000100000000000000000000000000000005000000000000000198bd7ea46629f0169a6eac2015c367196bc7e30431b5af269323e5d189b9b4e00000000000000001938698d8c6ad82400ead9e9c9d887179904ab0fa743182fe98c15c9fab5a1c280000001eeab280ed86a0ed959c20ec9b90ebacb820eca491eca78020ec82acec9ca0",
        "sha256": "0cc7fc6d11ad78000890d6bdf1d509207c57b8aa972ca073fb1e8abe9e8d978b",
        "bytes": 237
      },
      "policy": {
        "hex": "434f4e534f4c452e4944454e544954592e47524f55502e504f4c4943592e4845414400000100000000000000000000000000000003000000000000000000000000000000040000000000000001000000176e61746976652d67726f75702d70726f636573732d76317e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e51297694172e4cf6afcbfb97e879eff3d02061db35cf4a0a02042724b710351dcc60b25000000000000000000000000000000010000000000000000000000000000000f19a911a6eb9b79ff986bddb45fc24ea17f6e01b2589f33ebcefc3fcebf63e1280000000000000000000000000000000e00065cbceed7efc0",
        "sha256": "748a626ff665ced3a5bd1165facad0ef297af222154b051246bf7abe25c838b0",
        "bytes": 320
      },
      "version": {
        "hex": "434f4e534f4c452e4944454e544954592e50524f434553532e56455253494f4e00000100000000000000000000000000000003000000000000000000000000000000040000000000000000000000000000000500000000000000010000001947524f55505f56455249464945525f50524f434553535f5631000000000000000000000000000000010000000000000000000000000000000900000000000000010000000000000001748a626ff665ced3a5bd1165facad0ef297af222154b051246bf7abe25c838b00000000000000000000000000000000f19a911a6eb9b79ff986bddb45fc24ea17f6e01b2589f33ebcefc3fcebf63e12800065cbceec8ad8000065cd10d38a400000100000014eb8f85eba6bd20eab280eca69d20eca088ecb0a800010000000e636c61696d616e74206d61746368000000126163636f756e7420706f7373657373696f6e00000011706879736963616c2065766964656e636500000013636f6e747261646963746f727920636c61696d000000197175616c696669636174696f6e20696e737472756374696f6e00000017657363616c6174696f6e2061646a756469636174696f6e000000116d696e696d616c20726574656e74696f6e00000018726563697069656e7420726573706f6e736962696c697479",
        "sha256": "98bd7ea46629f0169a6eac2015c367196bc7e30431b5af269323e5d189b9b4e0",
        "bytes": 481
      },
      "head": {
        "hex": "434f4e534f4c452e4944454e544954592e50524f434553532e484541440000010000000000000000000000000000000300000000000000000000000000000004000000000000000000000000000000050000000000000002000000000000000198bd7ea46629f0169a6eac2015c367196bc7e30431b5af269323e5d189b9b4e0000200065cd10d38a40000000000000000000000000000000001000000000000000000000000000000020cc7fc6d11ad78000890d6bdf1d509207c57b8aa972ca073fb1e8abe9e8d978b0000000000000000000000000000000700065cbcef850ece",
        "sha256": "69ab1c9d0b376e732e0d0f02bb4ce3675b87146f6e7f2bf0d342ecbf524fa59d",
        "bytes": 226
      },
      "result": {
        "hex": "434f4e534f4c452e4944454e544954592e50524f434553532e524553554c540000020000000000000000000000000000000100000000000000000000000000000002000000000000000000000000000000030000000000000000000000000000000400060cc7fc6d11ad78000890d6bdf1d509207c57b8aa972ca073fb1e8abe9e8d978b0000000000000000000000000000000600000000000000000000000000000007000300065cbcef63264000065cbcef850ece0000000000000000000000000000000800000000000000010000000000000000000000000000000900000000000000010000000000000007010000000000000001748a626ff665ced3a5bd1165facad0ef297af222154b051246bf7abe25c838b0010000000000000001748a626ff665ced3a5bd1165facad0ef297af222154b051246bf7abe25c838b0000000176e61746976652d67726f75702d70726f636573732d76317e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e512976900000000000000014172e4cf6afcbfb97e879eff3d02061db35cf4a0a02042724b710351dcc60b2500000006342e31332e3000000003342e350000000000000000000000000000000501000000000000000000000000000000050000000000000001000000000000000198bd7ea46629f0169a6eac2015c367196bc7e30431b5af269323e5d189b9b4e0938698d8c6ad82400ead9e9c9d887179904ab0fa743182fe98c15c9fab5a1c28000100065cd10d38a40001000000000000000000000000000000050000000000000002000000000000000198bd7ea46629f0169a6eac2015c367196bc7e30431b5af269323e5d189b9b4e069ab1c9d0b376e732e0d0f02bb4ce3675b87146f6e7f2bf0d342ecbf524fa59d000200065cd10d38a40000000000000000150000007b",
        "sha256": "830a587fa92fea0be151fb5a5a0461635d2ffea4760498be6d4269815b9d5d46",
        "bytes": 742
      }
    },
    "suspend_reason": {
      "family": "suspend",
      "meta": {
        "actor": 1,
        "designation_receipt": 9,
        "designation_revision": 1,
        "command": 15,
        "input_digest": "19a911a6eb9b79ff986bddb45fc24ea17f6e01b2589f33ebcefc3fcebf63e128",
        "admitted_at_us": 1790816390000000,
        "expiry_us": 1790902800000000,
        "title": "독립 검증 절차"
      },
      "reason": "제출하지 않은 다른 사유",
      "command": {
        "hex": "434f4e534f4c452e4944454e544954592e47524f55500000010006000000000000000000000000000000010000000000000000000000000000000200000000000000000000000000000003000000000000000000000000000000040000000000000007000000000000000100000000000000000000000000000005000000000000000198bd7ea46629f0169a6eac2015c367196bc7e30431b5af269323e5d189b9b4e00000000000000001938698d8c6ad82400ead9e9c9d887179904ab0fa743182fe98c15c9fab5a1c280000001eeab280ed86a0ed959c20ec9b90ebacb820eca491eca78020ec82acec9ca0",
        "sha256": "0cc7fc6d11ad78000890d6bdf1d509207c57b8aa972ca073fb1e8abe9e8d978b",
        "bytes": 237
      },
      "policy": {
        "hex": "434f4e534f4c452e4944454e544954592e47524f55502e504f4c4943592e4845414400000100000000000000000000000000000003000000000000000000000000000000040000000000000001000000176e61746976652d67726f75702d70726f636573732d76317e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e51297694172e4cf6afcbfb97e879eff3d02061db35cf4a0a02042724b710351dcc60b25000000000000000000000000000000010000000000000000000000000000000f19a911a6eb9b79ff986bddb45fc24ea17f6e01b2589f33ebcefc3fcebf63e1280000000000000000000000000000000e00065cbceed7efc0",
        "sha256": "748a626ff665ced3a5bd1165facad0ef297af222154b051246bf7abe25c838b0",
        "bytes": 320
      },
      "version": {
        "hex": "434f4e534f4c452e4944454e544954592e50524f434553532e56455253494f4e00000100000000000000000000000000000003000000000000000000000000000000040000000000000000000000000000000500000000000000010000001947524f55505f56455249464945525f50524f434553535f5631000000000000000000000000000000010000000000000000000000000000000900000000000000010000000000000001748a626ff665ced3a5bd1165facad0ef297af222154b051246bf7abe25c838b00000000000000000000000000000000f19a911a6eb9b79ff986bddb45fc24ea17f6e01b2589f33ebcefc3fcebf63e12800065cbceec8ad8000065cd10d38a400000100000014eb8f85eba6bd20eab280eca69d20eca088ecb0a800010000000e636c61696d616e74206d61746368000000126163636f756e7420706f7373657373696f6e00000011706879736963616c2065766964656e636500000013636f6e747261646963746f727920636c61696d000000197175616c696669636174696f6e20696e737472756374696f6e00000017657363616c6174696f6e2061646a756469636174696f6e000000116d696e696d616c20726574656e74696f6e00000018726563697069656e7420726573706f6e736962696c697479",
        "sha256": "98bd7ea46629f0169a6eac2015c367196bc7e30431b5af269323e5d189b9b4e0",
        "bytes": 481
      },
      "head": {
        "hex": "434f4e534f4c452e4944454e544954592e50524f434553532e484541440000010000000000000000000000000000000300000000000000000000000000000004000000000000000000000000000000050000000000000002000000000000000198bd7ea46629f0169a6eac2015c367196bc7e30431b5af269323e5d189b9b4e0000200065cd10d38a40000000000000000000000000000000001000000000000000000000000000000020cc7fc6d11ad78000890d6bdf1d509207c57b8aa972ca073fb1e8abe9e8d978b0000000000000000000000000000000700065cbcef850ece",
        "sha256": "69ab1c9d0b376e732e0d0f02bb4ce3675b87146f6e7f2bf0d342ecbf524fa59d",
        "bytes": 226
      },
      "result": {
        "hex": "434f4e534f4c452e4944454e544954592e50524f434553532e524553554c540000020000000000000000000000000000000100000000000000000000000000000002000000000000000000000000000000030000000000000000000000000000000400060cc7fc6d11ad78000890d6bdf1d509207c57b8aa972ca073fb1e8abe9e8d978b0000000000000000000000000000000600000000000000000000000000000007000300065cbcef63264000065cbcef850ece0000000000000000000000000000000800000000000000010000000000000000000000000000000900000000000000010000000000000007010000000000000001748a626ff665ced3a5bd1165facad0ef297af222154b051246bf7abe25c838b0010000000000000001748a626ff665ced3a5bd1165facad0ef297af222154b051246bf7abe25c838b0000000176e61746976652d67726f75702d70726f636573732d76317e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e512976900000000000000014172e4cf6afcbfb97e879eff3d02061db35cf4a0a02042724b710351dcc60b2500000006342e31332e3000000003342e350000000000000000000000000000000501000000000000000000000000000000050000000000000001000000000000000198bd7ea46629f0169a6eac2015c367196bc7e30431b5af269323e5d189b9b4e0938698d8c6ad82400ead9e9c9d887179904ab0fa743182fe98c15c9fab5a1c28000100065cd10d38a40001000000000000000000000000000000050000000000000002000000000000000198bd7ea46629f0169a6eac2015c367196bc7e30431b5af269323e5d189b9b4e069ab1c9d0b376e732e0d0f02bb4ce3675b87146f6e7f2bf0d342ecbf524fa59d000200065cd10d38a40000000000000000150000007b",
        "sha256": "830a587fa92fea0be151fb5a5a0461635d2ffea4760498be6d4269815b9d5d46",
        "bytes": 742
      }
    },
    "suspend_version": {
      "family": "suspend",
      "meta": {
        "actor": 101,
        "designation_receipt": 9,
        "designation_revision": 1,
        "command": 15,
        "input_digest": "19a911a6eb9b79ff986bddb45fc24ea17f6e01b2589f33ebcefc3fcebf63e128",
        "admitted_at_us": 1790816390000000,
        "expiry_us": 1790902800000000,
        "title": "독립 검증 절차"
      },
      "reason": "검토한 원문 중지 사유",
      "command": {
        "hex": "434f4e534f4c452e4944454e544954592e47524f55500000010006000000000000000000000000000000010000000000000000000000000000000200000000000000000000000000000003000000000000000000000000000000040000000000000007000000000000000100000000000000000000000000000005000000000000000198bd7ea46629f0169a6eac2015c367196bc7e30431b5af269323e5d189b9b4e00000000000000001938698d8c6ad82400ead9e9c9d887179904ab0fa743182fe98c15c9fab5a1c280000001eeab280ed86a0ed959c20ec9b90ebacb820eca491eca78020ec82acec9ca0",
        "sha256": "0cc7fc6d11ad78000890d6bdf1d509207c57b8aa972ca073fb1e8abe9e8d978b",
        "bytes": 237
      },
      "policy": {
        "hex": "434f4e534f4c452e4944454e544954592e47524f55502e504f4c4943592e4845414400000100000000000000000000000000000003000000000000000000000000000000040000000000000001000000176e61746976652d67726f75702d70726f636573732d76317e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e51297694172e4cf6afcbfb97e879eff3d02061db35cf4a0a02042724b710351dcc60b25000000000000000000000000000000010000000000000000000000000000000f19a911a6eb9b79ff986bddb45fc24ea17f6e01b2589f33ebcefc3fcebf63e1280000000000000000000000000000000e00065cbceed7efc0",
        "sha256": "748a626ff665ced3a5bd1165facad0ef297af222154b051246bf7abe25c838b0",
        "bytes": 320
      },
      "version": {
        "hex": "434f4e534f4c452e4944454e544954592e50524f434553532e56455253494f4e00000100000000000000000000000000000003000000000000000000000000000000040000000000000000000000000000000500000000000000010000001947524f55505f56455249464945525f50524f434553535f5631000000000000000000000000000000650000000000000000000000000000000900000000000000010000000000000001748a626ff665ced3a5bd1165facad0ef297af222154b051246bf7abe25c838b00000000000000000000000000000000f19a911a6eb9b79ff986bddb45fc24ea17f6e01b2589f33ebcefc3fcebf63e12800065cbceec8ad8000065cd10d38a400000100000014eb8f85eba6bd20eab280eca69d20eca088ecb0a800010000000e636c61696d616e74206d61746368000000126163636f756e7420706f7373657373696f6e00000011706879736963616c2065766964656e636500000013636f6e747261646963746f727920636c61696d000000197175616c696669636174696f6e20696e737472756374696f6e00000017657363616c6174696f6e2061646a756469636174696f6e000000116d696e696d616c20726574656e74696f6e00000018726563697069656e7420726573706f6e736962696c697479",
        "sha256": "ea6d734c9ebc1ef5df407383f3591ab60de7d1e12a366e3bab9a9e270e95f511",
        "bytes": 481
      },
      "head": {
        "hex": "434f4e534f4c452e4944454e544954592e50524f434553532e484541440000010000000000000000000000000000000300000000000000000000000000000004000000000000000000000000000000050000000000000002000000000000000198bd7ea46629f0169a6eac2015c367196bc7e30431b5af269323e5d189b9b4e0000200065cd10d38a40000000000000000000000000000000001000000000000000000000000000000020cc7fc6d11ad78000890d6bdf1d509207c57b8aa972ca073fb1e8abe9e8d978b0000000000000000000000000000000700065cbcef850ece",
        "sha256": "69ab1c9d0b376e732e0d0f02bb4ce3675b87146f6e7f2bf0d342ecbf524fa59d",
        "bytes": 226
      },
      "result": {
        "hex": "434f4e534f4c452e4944454e544954592e50524f434553532e524553554c540000020000000000000000000000000000000100000000000000000000000000000002000000000000000000000000000000030000000000000000000000000000000400060cc7fc6d11ad78000890d6bdf1d509207c57b8aa972ca073fb1e8abe9e8d978b0000000000000000000000000000000600000000000000000000000000000007000300065cbcef63264000065cbcef850ece0000000000000000000000000000000800000000000000010000000000000000000000000000000900000000000000010000000000000007010000000000000001748a626ff665ced3a5bd1165facad0ef297af222154b051246bf7abe25c838b0010000000000000001748a626ff665ced3a5bd1165facad0ef297af222154b051246bf7abe25c838b0000000176e61746976652d67726f75702d70726f636573732d76317e16e381efc48b06a18d6389bb40087171f053ad2c5c45bc3fab7aa514233683573b7e1b64fbb4abe2e726b793ea886c68f42a38c2dde2c52856678ec0ac6727338c1263632e1c8cf005a7d913bc0ab46ab91d3a212f837f69bff645e512976900000000000000014172e4cf6afcbfb97e879eff3d02061db35cf4a0a02042724b710351dcc60b2500000006342e31332e3000000003342e350000000000000000000000000000000501000000000000000000000000000000050000000000000001000000000000000198bd7ea46629f0169a6eac2015c367196bc7e30431b5af269323e5d189b9b4e0938698d8c6ad82400ead9e9c9d887179904ab0fa743182fe98c15c9fab5a1c28000100065cd10d38a40001000000000000000000000000000000050000000000000002000000000000000198bd7ea46629f0169a6eac2015c367196bc7e30431b5af269323e5d189b9b4e069ab1c9d0b376e732e0d0f02bb4ce3675b87146f6e7f2bf0d342ecbf524fa59d000200065cd10d38a40000000000000000150000007b",
        "sha256": "830a587fa92fea0be151fb5a5a0461635d2ffea4760498be6d4269815b9d5d46",
        "bytes": 742
      }
    }
  }
}
"###;

// Scripted workflow orchestration; live Auth/SQL evidence remains separate.
include!("validation_form_workflow_tests.rs");
