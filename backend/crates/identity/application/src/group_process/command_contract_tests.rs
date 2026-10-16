//! Group command grammar against the reviewed independent wire declaration.
//! Pure tests: fixture IDs/digests establish no live Auth, source or SQL custody.
use super::*;
use time::UtcOffset;
use uuid::Uuid;

// An independent literal vector produced from the declared byte order, with
// actor1/command2/Group3/incarnation4/process5, Group revision7, policy0/prior0.
// No production encoder was used to produce these bytes.
const INDEPENDENT_COMMAND_V1_HEX: &str = concat!(
    "434f4e534f4c452e4944454e544954592e47524f55500000010001",
    "0000000000000000000000000000000100000000000000000000000000000002",
    "0000000000000000000000000000000300000000000000000000000000000004",
    "0000000000000007000000000000000000000000000000000000000000000005",
    "000000000000000000065cbcef614400000100000014eb8f85eba6bd20eab280eca69d20eca088ecb0a8",
    "00010000000e636c61696d616e74206d61746368000000126163636f756e7420706f7373657373696f6e",
    "00000011706879736963616c2065766964656e636500000013636f6e747261646963746f727920636c61696d",
    "000000197175616c696669636174696f6e20696e737472756374696f6e00000017657363616c6174696f6e",
    "2061646a756469636174696f6e000000116d696e696d616c20726574656e74696f6e000000187265636970",
    "69656e7420726573706f6e736962696c697479",
);
const EXPIRY_US: i64 = 1_790_816_400_000_000;
// Calendar oracle: 0001-01-01T00:00:00+09:00 through
// 9999-12-31T23:59:59+09:00, inclusive, converted once to UTC microseconds.
const KST_FIRST_SECOND_US: i64 = -62_135_629_200_000_000;
const KST_LAST_SECOND_US: i64 = 253_402_268_399_000_000;
const POLICY_REVISION_OFFSET: usize = 99;
const PRIOR_HEAD_REVISION_OFFSET: usize = 123;
const EXPIRY_OFFSET: usize = 131;

fn actor() -> AccountId {
    AccountId::from_uuid(Uuid::from_u128(1)).unwrap()
}

fn content() -> ProcessContentV1 {
    ProcessContentV1::from_projection(ProcessContentProjectionV1 {
        title: "독립 검증 절차".into(),
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

fn adoption(
    policy: u64,
    prior: u64,
    expiry_us: i64,
) -> Result<GroupProcessCommandV1, GroupProcessError> {
    GroupProcessCommandV1::adopt(
        Uuid::from_u128(2),
        GroupId::from_uuid(Uuid::from_u128(3)).unwrap(),
        GroupIncarnation::from_uuid(Uuid::from_u128(4)).unwrap(),
        7,
        policy,
        Uuid::from_u128(5),
        prior,
        time_from_us(expiry_us).unwrap(),
        AcceptedOperatorResponsibilityV1::from_form("yes").unwrap(),
        content(),
    )
}

fn independent_wire(policy: u64, prior: u64, expiry_us: i64) -> Vec<u8> {
    let mut wire: Vec<u8> = INDEPENDENT_COMMAND_V1_HEX
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect();
    assert_eq!(wire.len(), 356);
    // Only the three declared scalar fields vary; this never asks the owner
    // codec to generate the evidence subsequently fed into its decoder.
    wire[POLICY_REVISION_OFFSET..POLICY_REVISION_OFFSET + 8].copy_from_slice(&policy.to_be_bytes());
    wire[PRIOR_HEAD_REVISION_OFFSET..PRIOR_HEAD_REVISION_OFFSET + 8]
        .copy_from_slice(&prior.to_be_bytes());
    wire[EXPIRY_OFFSET..EXPIRY_OFFSET + 8].copy_from_slice(&expiry_us.to_be_bytes());
    wire
}

fn require_transport_valid_stale_expectation(policy: u64, prior: u64) {
    let wire = independent_wire(policy, prior, EXPIRY_US);
    let constructed = adoption(policy, prior, EXPIRY_US);
    let decoded = GroupProcessCommandV1::decode(&wire);
    assert_eq!(
        (constructed.is_ok(), decoded.is_ok()),
        (true, true),
        "each independently bounded expectation is command grammar; actual retained source decides staleness"
    );
    let constructed = constructed.unwrap();
    let (decoded_actor, decoded) = decoded.unwrap();
    assert_eq!(decoded_actor, actor());
    assert_eq!(decoded, constructed);
    assert_eq!(decoded.expected_policy_revision(), policy);
    assert_eq!(
        decoded.adoption().unwrap().expected_prior_head_revision(),
        prior
    );
    assert_eq!(constructed.encode(actor()).unwrap(), wire);
}

#[test]
fn group_command_contract_policy1_prior0_is_transport_valid_stale_expectation() {
    require_transport_valid_stale_expectation(1, 0);
}

#[test]
fn group_command_contract_policy0_prior1_is_transport_valid_stale_expectation() {
    require_transport_valid_stale_expectation(0, 1);
}

#[test]
fn group_command_contract_independent_vector_and_inclusive_kst_calendar_endpoints() {
    let base = adoption(0, 0, EXPIRY_US).unwrap();
    assert_eq!(
        base.encode(actor()).unwrap(),
        independent_wire(0, 0, EXPIRY_US)
    );
    let kst = UtcOffset::from_hms(9, 0, 0).unwrap();
    let first = time_from_us(KST_FIRST_SECOND_US).unwrap().to_offset(kst);
    let last = time_from_us(KST_LAST_SECOND_US).unwrap().to_offset(kst);
    assert_eq!(
        (
            first.year(),
            first.month(),
            first.day(),
            first.hour(),
            first.minute(),
            first.second()
        ),
        (1, time::Month::January, 1, 0, 0, 0)
    );
    assert_eq!(
        (
            last.year(),
            last.month(),
            last.day(),
            last.hour(),
            last.minute(),
            last.second()
        ),
        (9999, time::Month::December, 31, 23, 59, 59)
    );
    for expiry_us in [KST_FIRST_SECOND_US, KST_LAST_SECOND_US] {
        let constructed = adoption(0, 0, expiry_us).unwrap();
        let wire = independent_wire(0, 0, expiry_us);
        let (decoded_actor, decoded) = GroupProcessCommandV1::decode(&wire).unwrap();
        assert_eq!(decoded_actor, actor());
        assert_eq!(decoded, constructed);
        assert_eq!(
            exact_time_us(decoded.adoption().unwrap().expiry()).unwrap(),
            expiry_us
        );
        assert_eq!(constructed.encode(actor()).unwrap(), wire);
    }
}

#[test]
fn group_command_contract_rejects_whole_seconds_outside_kst_year0001_to9999() {
    for expiry_us in [
        KST_FIRST_SECOND_US - 1_000_000,
        KST_LAST_SECOND_US + 1_000_000,
    ] {
        let constructed = adoption(0, 0, expiry_us);
        let decoded = GroupProcessCommandV1::decode(&independent_wire(0, 0, expiry_us));
        assert_eq!(
            (constructed.is_err(), decoded.is_err()),
            (true, true),
            "command expiry must fit the exact KST calendar, independently of owner-time eligibility"
        );
    }
}

#[test]
fn group_command_contract_rejects_subsecond_expiry_in_constructor_and_independent_wire() {
    for expiry_us in [-1, -1_000_001, EXPIRY_US + 1, EXPIRY_US + 999_999] {
        let constructed = adoption(0, 0, expiry_us);
        let decoded = GroupProcessCommandV1::decode(&independent_wire(0, 0, expiry_us));
        assert_eq!(
            (constructed.is_err(), decoded.is_err()),
            (true, true),
            "strict KST YYYY-MM-DDTHH:MM:SS command grammar admits no fractional seconds"
        );
    }
}

#[test]
fn group_command_contract_preserves_fractional_database_and_result2_timestamps() {
    let accepted_at_us = EXPIRY_US - 1_000_000 + 123_456;
    let executed_at_us = EXPIRY_US + 654_321;
    for micros in [accepted_at_us, executed_at_us, -1, -1_000_001] {
        let timestamp = time_from_us(micros).unwrap();
        assert_ne!(timestamp.nanosecond(), 0);
        assert_eq!(
            exact_time_us(timestamp).unwrap(),
            micros,
            "database source/result timestamps remain exact UTC microseconds"
        );
    }
    let accepted = GroupProcessAcceptedV1::checked(
        actor(),
        independent_wire(0, 0, EXPIRY_US),
        [0x11; 32],
        Uuid::from_u128(6),
        accepted_at_us,
    )
    .unwrap();
    assert_eq!(accepted.accepted_at_us, accepted_at_us);
    let result = ProcessResultV2::from_projection(ProcessResultProjectionV2 {
        actor: actor(),
        command: Uuid::from_u128(2),
        group: GroupId::from_uuid(Uuid::from_u128(3)).unwrap(),
        incarnation: GroupIncarnation::from_uuid(Uuid::from_u128(4)).unwrap(),
        opcode: 1,
        input_digest: [0x11; 32],
        intake_receipt: Uuid::from_u128(6),
        result_receipt: Uuid::from_u128(7),
        terminal_code: ProcessTerminalCodeV1::RejectedProcessExpired,
        accepted_at_us,
        executed_at_us,
        session: Uuid::from_u128(8),
        security_generation: 1,
        designation_receipt: Uuid::from_u128(9),
        designation_revision: 1,
        observed_group_revision: 7,
        policy_before: PolicyHeadReferenceV1::Absent,
        policy_after: PolicyHeadReferenceV1::Absent,
        evaluated_bundle: EvaluatedPolicyBundleV1::from_projection(
            EvaluatedPolicyBundleProjectionV1 {
                schema_id: "native-group-process-v1".into(),
                schema_digest: [0x22; 32],
                policy_digest: [0x33; 32],
                codec_contract_digest: [0x44; 32],
                registration_manifest_version: 1,
                registration_manifest_digest: [0x55; 32],
                cedar_sdk_version: "4.11.0".into(),
                cedar_language_version: "4.11".into(),
            },
        )
        .unwrap(),
        requested_process_id: Uuid::from_u128(5),
        before_head: None,
        after_head: None,
        effect_xid8: 1,
        effect_pid: 123,
    })
    .unwrap();
    let bytes = result.encode();
    let decoded = ProcessResultV2::decode(&bytes).unwrap();
    assert_eq!(decoded.accepted_at_us(), accepted_at_us);
    assert_eq!(decoded.executed_at_us(), executed_at_us);
    assert_eq!(decoded, result);
    let terminal = GroupProcessTerminalV2::checked(accepted, bytes.clone(), [0x66; 32]).unwrap();
    assert_eq!(terminal.result.accepted_at_us(), accepted_at_us);
    assert_eq!(terminal.result.executed_at_us(), executed_at_us);
    assert_eq!(terminal.result_bytes, bytes);
}
