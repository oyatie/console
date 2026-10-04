//! Exact Group command1/result2 and immutable digest preimages. Hashing belongs
//! to the adapter's installed SHA-256 implementation; no historical codec changes.
use super::{AccountId, GroupId, GroupIncarnation, GroupProcessError, nonnil, revision, text};
use time::OffsetDateTime;
use uuid::Uuid;

const COMMAND_PREFIX: &[u8] = b"CONSOLE.IDENTITY.GROUP\0";
const RESULT_PREFIX: &[u8] = b"CONSOLE.IDENTITY.PROCESS.RESULT\0";
pub const GROUP_PROCESS_SCHEMA_V1: &str = "GROUP_VERIFIER_PROCESS_V1";
pub const GROUP_PROCESS_POLICY_SCHEMA_V1: &str = "native-group-process-v1";
pub const GROUP_PROCESS_ADOPT_MAX_BYTES: usize = 16_521;
pub const GROUP_PROCESS_SUSPEND_MAX_BYTES: usize = 2_255;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AcceptedOperatorResponsibilityV1;

impl AcceptedOperatorResponsibilityV1 {
    pub fn from_form(value: &str) -> Result<Self, GroupProcessError> {
        if value == "yes" {
            Ok(Self)
        } else {
            Err(GroupProcessError::InvalidInput)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessMethodV1 {
    AttendedAccountAndDocumentaryReview,
}

impl ProcessMethodV1 {
    pub const fn as_str(self) -> &'static str {
        "ATTENDED_ACCOUNT_AND_DOCUMENTARY_REVIEW_V1"
    }

    pub fn from_str(value: &str) -> Result<Self, GroupProcessError> {
        if value == Self::AttendedAccountAndDocumentaryReview.as_str() {
            Ok(Self::AttendedAccountAndDocumentaryReview)
        } else {
            Err(GroupProcessError::InvalidInput)
        }
    }
}

/// Raw rows/inputs confer no authority. Construction checks exact stored bytes.
pub struct ProcessContentProjectionV1 {
    pub title: String,
    pub method: ProcessMethodV1,
    pub intended_claimant_matching_procedure: String,
    pub account_possession_procedure: String,
    pub physical_human_evidence_procedure: String,
    pub duplicate_contradictory_claim_procedure: String,
    pub qualification_criteria_instruction: String,
    pub escalation_adjudication_procedure: String,
    pub evidence_minimization_retention_description: String,
    pub recipient_responsibility: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessContentV1 {
    title: String,
    method: ProcessMethodV1,
    prose: [String; 8],
}

impl ProcessContentV1 {
    pub fn from_projection(row: ProcessContentProjectionV1) -> Result<Self, GroupProcessError> {
        let value = Self {
            title: row.title,
            method: row.method,
            prose: [
                row.intended_claimant_matching_procedure,
                row.account_possession_procedure,
                row.physical_human_evidence_procedure,
                row.duplicate_contradictory_claim_procedure,
                row.qualification_criteria_instruction,
                row.escalation_adjudication_procedure,
                row.evidence_minimization_retention_description,
                row.recipient_responsibility,
            ],
        };
        text(&value.title, 120)?;
        for field in &value.prose {
            text(field, 2048)?;
        }
        if value.title.len()
            + value.method.as_str().len()
            + value.prose.iter().map(String::len).sum::<usize>()
            > 16_384
        {
            return Err(GroupProcessError::Unavailable);
        }
        Ok(value)
    }

    pub fn title(&self) -> &str {
        &self.title
    }
    pub const fn method(&self) -> ProcessMethodV1 {
        self.method
    }
    pub fn intended_claimant_matching_procedure(&self) -> &str {
        &self.prose[0]
    }
    pub fn account_possession_procedure(&self) -> &str {
        &self.prose[1]
    }
    pub fn physical_human_evidence_procedure(&self) -> &str {
        &self.prose[2]
    }
    pub fn duplicate_contradictory_claim_procedure(&self) -> &str {
        &self.prose[3]
    }
    pub fn qualification_criteria_instruction(&self) -> &str {
        &self.prose[4]
    }
    pub fn escalation_adjudication_procedure(&self) -> &str {
        &self.prose[5]
    }
    pub fn evidence_minimization_retention_description(&self) -> &str {
        &self.prose[6]
    }
    pub fn recipient_responsibility(&self) -> &str {
        &self.prose[7]
    }

    fn append(&self, bytes: &mut Vec<u8>) {
        append_text(bytes, &self.title);
        append_u16(bytes, 1);
        for field in &self.prose {
            append_text(bytes, field);
        }
    }

    fn read(input: &mut &[u8]) -> Result<Self, GroupProcessError> {
        let title = read_text(input, 120)?;
        if read_u16(input)? != 1 {
            return Err(GroupProcessError::Unavailable);
        }
        Self::from_projection(ProcessContentProjectionV1 {
            title,
            method: ProcessMethodV1::AttendedAccountAndDocumentaryReview,
            intended_claimant_matching_procedure: read_text(input, 2048)?,
            account_possession_procedure: read_text(input, 2048)?,
            physical_human_evidence_procedure: read_text(input, 2048)?,
            duplicate_contradictory_claim_procedure: read_text(input, 2048)?,
            qualification_criteria_instruction: read_text(input, 2048)?,
            escalation_adjudication_procedure: read_text(input, 2048)?,
            evidence_minimization_retention_description: read_text(input, 2048)?,
            recipient_responsibility: read_text(input, 2048)?,
        })
    }
}

/// Exact content-version preimage; its separately verified SHA-256 is the
/// immutable content digest referenced by every head and command.
pub struct ProcessVersionProjectionV1 {
    pub group: GroupId,
    pub incarnation: GroupIncarnation,
    pub process_id: Uuid,
    pub version: u64,
    pub actor: AccountId,
    pub designation_receipt: Uuid,
    pub designation_revision: u64,
    pub policy_revision: PositivePolicyRevisionV1,
    pub policy_head_digest: [u8; 32],
    pub adopt_command: Uuid,
    pub input_digest: [u8; 32],
    pub admitted_at_us: i64,
    pub expiry_us: i64,
    pub responsibility: AcceptedOperatorResponsibilityV1,
    pub content: ProcessContentV1,
    pub content_digest: [u8; 32],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessVersionV1 {
    group: GroupId,
    incarnation: GroupIncarnation,
    process_id: Uuid,
    version: u64,
    actor: AccountId,
    designation_receipt: Uuid,
    designation_revision: u64,
    policy_revision: PositivePolicyRevisionV1,
    policy_head_digest: [u8; 32],
    adopt_command: Uuid,
    input_digest: [u8; 32],
    admitted_at_us: i64,
    expiry_us: i64,
    responsibility: AcceptedOperatorResponsibilityV1,
    content: ProcessContentV1,
    content_digest: [u8; 32],
}

impl ProcessVersionV1 {
    pub fn from_projection(row: ProcessVersionProjectionV1) -> Result<Self, GroupProcessError> {
        for id in [
            row.process_id,
            *row.actor.as_uuid(),
            row.designation_receipt,
            row.adopt_command,
        ] {
            nonnil(id)?;
        }
        revision(row.version, false)?;
        revision(row.designation_revision, false)?;
        time_from_us(row.admitted_at_us)?;
        time_from_us(row.expiry_us)?;
        if row.policy_revision.get() != 1
            || row.expiry_us <= row.admitted_at_us
            || i128::from(row.expiry_us) - i128::from(row.admitted_at_us)
                > 365_i128 * 86_400 * 1_000_000
        {
            return Err(GroupProcessError::Unavailable);
        }
        Ok(Self {
            group: row.group,
            incarnation: row.incarnation,
            process_id: row.process_id,
            version: row.version,
            actor: row.actor,
            designation_receipt: row.designation_receipt,
            designation_revision: row.designation_revision,
            policy_revision: row.policy_revision,
            policy_head_digest: row.policy_head_digest,
            adopt_command: row.adopt_command,
            input_digest: row.input_digest,
            admitted_at_us: row.admitted_at_us,
            expiry_us: row.expiry_us,
            responsibility: row.responsibility,
            content: row.content,
            content_digest: row.content_digest,
        })
    }
    pub const fn group(&self) -> GroupId {
        self.group
    }
    pub const fn incarnation(&self) -> GroupIncarnation {
        self.incarnation
    }
    pub const fn process_id(&self) -> Uuid {
        self.process_id
    }
    pub const fn version(&self) -> u64 {
        self.version
    }
    pub const fn actor(&self) -> AccountId {
        self.actor
    }
    pub const fn designation_receipt(&self) -> Uuid {
        self.designation_receipt
    }
    pub const fn designation_revision(&self) -> u64 {
        self.designation_revision
    }
    pub const fn policy_revision(&self) -> PositivePolicyRevisionV1 {
        self.policy_revision
    }
    pub const fn policy_head_digest(&self) -> &[u8; 32] {
        &self.policy_head_digest
    }
    pub const fn adopt_command(&self) -> Uuid {
        self.adopt_command
    }
    pub const fn input_digest(&self) -> &[u8; 32] {
        &self.input_digest
    }
    pub const fn admitted_at_us(&self) -> i64 {
        self.admitted_at_us
    }
    pub const fn expiry_us(&self) -> i64 {
        self.expiry_us
    }
    pub const fn responsibility(&self) -> AcceptedOperatorResponsibilityV1 {
        self.responsibility
    }
    pub fn content(&self) -> &ProcessContentV1 {
        &self.content
    }
    pub const fn content_digest(&self) -> &[u8; 32] {
        &self.content_digest
    }
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = prefix(b"CONSOLE.IDENTITY.PROCESS.VERSION\0", 1);
        append_uuid(&mut bytes, *self.group.as_uuid());
        append_uuid(&mut bytes, *self.incarnation.as_uuid());
        append_uuid(&mut bytes, self.process_id);
        append_u64(&mut bytes, self.version);
        append_text(&mut bytes, GROUP_PROCESS_SCHEMA_V1);
        append_uuid(&mut bytes, *self.actor.as_uuid());
        append_uuid(&mut bytes, self.designation_receipt);
        append_u64(&mut bytes, self.designation_revision);
        append_u64(&mut bytes, self.policy_revision.get());
        bytes.extend_from_slice(&self.policy_head_digest);
        append_uuid(&mut bytes, self.adopt_command);
        bytes.extend_from_slice(&self.input_digest);
        append_i64(&mut bytes, self.admitted_at_us);
        append_i64(&mut bytes, self.expiry_us);
        append_u16(&mut bytes, 1);
        self.content.append(&mut bytes);
        bytes
    }
}

/// Only new HTTP descriptive inputs normalize CRLF; stored codecs never do.
pub fn normalize_group_process_form_text(
    raw: &str,
    raw_max: usize,
) -> Result<String, GroupProcessError> {
    if raw.len() > raw_max {
        return Err(GroupProcessError::InvalidInput);
    }
    let value = raw.replace("\r\n", "\n");
    if value.contains('\r') {
        return Err(GroupProcessError::InvalidInput);
    }
    Ok(value)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessAdoptV1 {
    process: Uuid,
    expected_prior_head_revision: u64,
    expiry: OffsetDateTime,
    responsibility: AcceptedOperatorResponsibilityV1,
    content: ProcessContentV1,
}

impl ProcessAdoptV1 {
    pub const fn process_id(&self) -> Uuid {
        self.process
    }
    pub const fn expected_prior_head_revision(&self) -> u64 {
        self.expected_prior_head_revision
    }
    pub const fn expiry(&self) -> OffsetDateTime {
        self.expiry
    }
    pub const fn responsibility(&self) -> AcceptedOperatorResponsibilityV1 {
        self.responsibility
    }
    pub fn content(&self) -> &ProcessContentV1 {
        &self.content
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessSuspendV1 {
    process: Uuid,
    content_version: u64,
    content_digest: [u8; 32],
    expected_head_revision: u64,
    expected_head_digest: [u8; 32],
    reason: String,
}

impl ProcessSuspendV1 {
    pub const fn process_id(&self) -> Uuid {
        self.process
    }
    pub const fn content_version(&self) -> u64 {
        self.content_version
    }
    pub const fn content_digest(&self) -> &[u8; 32] {
        &self.content_digest
    }
    pub const fn expected_head_revision(&self) -> u64 {
        self.expected_head_revision
    }
    pub const fn expected_head_digest(&self) -> &[u8; 32] {
        &self.expected_head_digest
    }
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ProcessInputV1 {
    Adopt(ProcessAdoptV1),
    Suspend(ProcessSuspendV1),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupProcessCommandV1 {
    command_id: Uuid,
    group: GroupId,
    incarnation: GroupIncarnation,
    expected_group_revision: u64,
    expected_policy_revision: u64,
    input: ProcessInputV1,
}

impl GroupProcessCommandV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn adopt(
        command_id: Uuid,
        group: GroupId,
        incarnation: GroupIncarnation,
        expected_group_revision: u64,
        expected_policy_revision: u64,
        process: Uuid,
        expected_prior_head_revision: u64,
        expiry: OffsetDateTime,
        responsibility: AcceptedOperatorResponsibilityV1,
        content: ProcessContentV1,
    ) -> Result<Self, GroupProcessError> {
        nonnil(command_id)?;
        nonnil(process)?;
        revision(expected_group_revision, false)?;
        revision(expected_policy_revision, true)?;
        revision(expected_prior_head_revision, true)?;
        let expiry_us = exact_time_us(expiry).map_err(|_| GroupProcessError::InvalidInput)?;
        // Command expiry follows the exact seconds-bearing KST form grammar.
        // Database/result timestamps retain their independent µs precision.
        if expiry_us % 1_000_000 != 0
            || !(-62_135_629_200_000_000..=253_402_268_399_000_000).contains(&expiry_us)
        {
            return Err(GroupProcessError::InvalidInput);
        }
        Ok(Self {
            command_id,
            group,
            incarnation,
            expected_group_revision,
            expected_policy_revision,
            input: ProcessInputV1::Adopt(ProcessAdoptV1 {
                process,
                expected_prior_head_revision,
                expiry,
                responsibility,
                content,
            }),
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn suspend(
        command_id: Uuid,
        group: GroupId,
        incarnation: GroupIncarnation,
        expected_group_revision: u64,
        expected_policy_revision: u64,
        process: Uuid,
        content_version: u64,
        content_digest: [u8; 32],
        expected_head_revision: u64,
        expected_head_digest: [u8; 32],
        reason: String,
    ) -> Result<Self, GroupProcessError> {
        nonnil(command_id)?;
        nonnil(process)?;
        revision(expected_group_revision, false)?;
        revision(expected_policy_revision, false)?;
        revision(content_version, false)?;
        revision(expected_head_revision, false)?;
        text(&reason, 2048)?;
        Ok(Self {
            command_id,
            group,
            incarnation,
            expected_group_revision,
            expected_policy_revision,
            input: ProcessInputV1::Suspend(ProcessSuspendV1 {
                process,
                content_version,
                content_digest,
                expected_head_revision,
                expected_head_digest,
                reason,
            }),
        })
    }

    pub const fn command_id(&self) -> Uuid {
        self.command_id
    }
    pub const fn group(&self) -> GroupId {
        self.group
    }
    pub const fn incarnation(&self) -> GroupIncarnation {
        self.incarnation
    }
    pub const fn expected_group_revision(&self) -> u64 {
        self.expected_group_revision
    }
    pub const fn expected_policy_revision(&self) -> u64 {
        self.expected_policy_revision
    }
    pub const fn opcode(&self) -> u16 {
        match self.input {
            ProcessInputV1::Adopt(_) => 1,
            ProcessInputV1::Suspend(_) => 6,
        }
    }
    pub fn adoption(&self) -> Option<&ProcessAdoptV1> {
        match &self.input {
            ProcessInputV1::Adopt(input) => Some(input),
            _ => None,
        }
    }
    pub fn suspension(&self) -> Option<&ProcessSuspendV1> {
        match &self.input {
            ProcessInputV1::Suspend(input) => Some(input),
            _ => None,
        }
    }
    pub fn process_id(&self) -> Uuid {
        match &self.input {
            ProcessInputV1::Adopt(input) => input.process,
            ProcessInputV1::Suspend(input) => input.process,
        }
    }
    pub fn encode(&self, actor: AccountId) -> Result<Vec<u8>, GroupProcessError> {
        nonnil(*actor.as_uuid())?;
        let mut bytes = prefix(COMMAND_PREFIX, 1);
        append_u16(&mut bytes, self.opcode());
        append_uuid(&mut bytes, *actor.as_uuid());
        append_uuid(&mut bytes, self.command_id);
        append_uuid(&mut bytes, *self.group.as_uuid());
        append_uuid(&mut bytes, *self.incarnation.as_uuid());
        append_u64(&mut bytes, self.expected_group_revision);
        append_u64(&mut bytes, self.expected_policy_revision);
        match &self.input {
            ProcessInputV1::Adopt(input) => {
                append_uuid(&mut bytes, input.process);
                append_u64(&mut bytes, input.expected_prior_head_revision);
                append_i64(&mut bytes, exact_time_us(input.expiry)?);
                append_u16(&mut bytes, 1);
                input.content.append(&mut bytes);
            }
            ProcessInputV1::Suspend(input) => {
                append_uuid(&mut bytes, input.process);
                append_u64(&mut bytes, input.content_version);
                bytes.extend_from_slice(&input.content_digest);
                append_u64(&mut bytes, input.expected_head_revision);
                bytes.extend_from_slice(&input.expected_head_digest);
                append_text(&mut bytes, &input.reason);
            }
        }
        if bytes.len()
            > if self.opcode() == 1 {
                GROUP_PROCESS_ADOPT_MAX_BYTES
            } else {
                GROUP_PROCESS_SUSPEND_MAX_BYTES
            }
        {
            return Err(GroupProcessError::Unavailable);
        }
        Ok(bytes)
    }

    pub fn decode(mut input: &[u8]) -> Result<(AccountId, Self), GroupProcessError> {
        if input.len() > GROUP_PROCESS_ADOPT_MAX_BYTES {
            return Err(GroupProcessError::Unavailable);
        }
        read_prefix(&mut input, COMMAND_PREFIX, 1)?;
        let opcode = read_u16(&mut input)?;
        let actor = AccountId::from_uuid(read_uuid(&mut input)?)
            .map_err(|_| GroupProcessError::Unavailable)?;
        let command = read_uuid(&mut input)?;
        let group = GroupId::from_uuid(read_uuid(&mut input)?)
            .map_err(|_| GroupProcessError::Unavailable)?;
        let incarnation = GroupIncarnation::from_uuid(read_uuid(&mut input)?)
            .map_err(|_| GroupProcessError::Unavailable)?;
        let group_revision = read_revision(&mut input, false)?;
        let policy_revision = read_revision(&mut input, true)?;
        let process = read_uuid(&mut input)?;
        let value = match opcode {
            1 => {
                let prior = read_revision(&mut input, true)?;
                let expiry = time_from_us(read_i64(&mut input)?)?;
                if read_u16(&mut input)? != 1 {
                    return Err(GroupProcessError::Unavailable);
                }
                Self::adopt(
                    command,
                    group,
                    incarnation,
                    group_revision,
                    policy_revision,
                    process,
                    prior,
                    expiry,
                    AcceptedOperatorResponsibilityV1,
                    ProcessContentV1::read(&mut input)?,
                )?
            }
            6 => Self::suspend(
                command,
                group,
                incarnation,
                group_revision,
                policy_revision,
                process,
                read_revision(&mut input, false)?,
                read_digest(&mut input)?,
                read_revision(&mut input, false)?,
                read_digest(&mut input)?,
                read_text(&mut input, 2048)?,
            )?,
            _ => return Err(GroupProcessError::Unavailable),
        };
        if !input.is_empty() {
            return Err(GroupProcessError::Unavailable);
        }
        Ok((actor, value))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PositivePolicyRevisionV1(u64);

impl PositivePolicyRevisionV1 {
    pub fn new(value: u64) -> Result<Self, GroupProcessError> {
        Ok(Self(revision(value, false)?))
    }
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolicyHeadReferenceV1 {
    Absent,
    Installed {
        revision: PositivePolicyRevisionV1,
        head_digest: [u8; 32],
    },
}

impl PolicyHeadReferenceV1 {
    pub const fn state_str(self) -> &'static str {
        match self {
            Self::Absent => "ABSENT",
            Self::Installed { .. } => "INSTALLED",
        }
    }
    pub const fn revision(self) -> Option<u64> {
        match self {
            Self::Absent => None,
            Self::Installed { revision, .. } => Some(revision.get()),
        }
    }
    pub fn encode(self) -> Vec<u8> {
        let mut bytes = Vec::new();
        self.append(&mut bytes);
        bytes
    }
    pub fn decode(mut input: &[u8]) -> Result<Self, GroupProcessError> {
        let value = Self::read(&mut input)?;
        if !input.is_empty() {
            return Err(GroupProcessError::Unavailable);
        }
        Ok(value)
    }
    fn append(self, bytes: &mut Vec<u8>) {
        match self {
            Self::Absent => bytes.push(0),
            Self::Installed {
                revision,
                head_digest,
            } => {
                bytes.push(1);
                append_u64(bytes, revision.get());
                bytes.extend_from_slice(&head_digest);
            }
        }
    }
    fn read(input: &mut &[u8]) -> Result<Self, GroupProcessError> {
        match take(input, 1)? {
            [0] => Ok(Self::Absent),
            [1] => Ok(Self::Installed {
                revision: PositivePolicyRevisionV1::new(read_u64(input)?)?,
                head_digest: read_digest(input)?,
            }),
            _ => Err(GroupProcessError::Unavailable),
        }
    }
}

pub struct EvaluatedPolicyBundleProjectionV1 {
    pub schema_id: String,
    pub schema_digest: [u8; 32],
    pub policy_digest: [u8; 32],
    pub codec_contract_digest: [u8; 32],
    pub registration_manifest_version: u64,
    pub registration_manifest_digest: [u8; 32],
    pub cedar_sdk_version: String,
    pub cedar_language_version: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvaluatedPolicyBundleV1 {
    schema_id: String,
    schema_digest: [u8; 32],
    policy_digest: [u8; 32],
    codec_contract_digest: [u8; 32],
    registration_manifest_version: u64,
    registration_manifest_digest: [u8; 32],
    cedar_sdk_version: String,
    cedar_language_version: String,
}

impl EvaluatedPolicyBundleV1 {
    pub fn from_projection(
        row: EvaluatedPolicyBundleProjectionV1,
    ) -> Result<Self, GroupProcessError> {
        text(&row.schema_id, 128)?;
        text(&row.cedar_sdk_version, 128)?;
        text(&row.cedar_language_version, 128)?;
        if row.schema_id != GROUP_PROCESS_POLICY_SCHEMA_V1 || row.registration_manifest_version != 1
        {
            return Err(GroupProcessError::Unavailable);
        }
        Ok(Self {
            schema_id: row.schema_id,
            schema_digest: row.schema_digest,
            policy_digest: row.policy_digest,
            codec_contract_digest: row.codec_contract_digest,
            registration_manifest_version: row.registration_manifest_version,
            registration_manifest_digest: row.registration_manifest_digest,
            cedar_sdk_version: row.cedar_sdk_version,
            cedar_language_version: row.cedar_language_version,
        })
    }
    pub fn schema_id(&self) -> &str {
        &self.schema_id
    }
    pub const fn schema_digest(&self) -> &[u8; 32] {
        &self.schema_digest
    }
    pub const fn policy_digest(&self) -> &[u8; 32] {
        &self.policy_digest
    }
    pub const fn codec_contract_digest(&self) -> &[u8; 32] {
        &self.codec_contract_digest
    }
    pub const fn registration_manifest_version(&self) -> u64 {
        self.registration_manifest_version
    }
    pub const fn registration_manifest_digest(&self) -> &[u8; 32] {
        &self.registration_manifest_digest
    }
    pub fn cedar_sdk_version(&self) -> &str {
        &self.cedar_sdk_version
    }
    pub fn cedar_language_version(&self) -> &str {
        &self.cedar_language_version
    }
    fn append(&self, bytes: &mut Vec<u8>) {
        append_text(bytes, &self.schema_id);
        bytes.extend_from_slice(&self.schema_digest);
        bytes.extend_from_slice(&self.policy_digest);
        bytes.extend_from_slice(&self.codec_contract_digest);
        append_u64(bytes, self.registration_manifest_version);
        bytes.extend_from_slice(&self.registration_manifest_digest);
        append_text(bytes, &self.cedar_sdk_version);
        append_text(bytes, &self.cedar_language_version);
    }
    fn read(input: &mut &[u8]) -> Result<Self, GroupProcessError> {
        Self::from_projection(EvaluatedPolicyBundleProjectionV1 {
            schema_id: read_text(input, 128)?,
            schema_digest: read_digest(input)?,
            policy_digest: read_digest(input)?,
            codec_contract_digest: read_digest(input)?,
            registration_manifest_version: read_u64(input)?,
            registration_manifest_digest: read_digest(input)?,
            cedar_sdk_version: read_text(input, 128)?,
            cedar_language_version: read_text(input, 128)?,
        })
    }
}

/// Installed policy1 immutable preimage. The digest is not included in itself.
pub struct GroupProcessPolicyHeadProjectionV1 {
    pub group: GroupId,
    pub incarnation: GroupIncarnation,
    pub revision: PositivePolicyRevisionV1,
    pub evaluated_bundle: EvaluatedPolicyBundleV1,
    pub first_actor: AccountId,
    pub first_command: Uuid,
    pub first_input_digest: [u8; 32],
    pub activation_receipt_id: Uuid,
    pub activated_at_us: i64,
}

pub fn encode_group_process_policy_head_v1(
    row: &GroupProcessPolicyHeadProjectionV1,
) -> Result<Vec<u8>, GroupProcessError> {
    nonnil(*row.first_actor.as_uuid())?;
    nonnil(row.first_command)?;
    nonnil(row.activation_receipt_id)?;
    time_from_us(row.activated_at_us)?;
    if row.revision.get() != 1 {
        return Err(GroupProcessError::Unavailable);
    }
    let mut bytes = prefix(b"CONSOLE.IDENTITY.GROUP.POLICY.HEAD\0", 1);
    append_uuid(&mut bytes, *row.group.as_uuid());
    append_uuid(&mut bytes, *row.incarnation.as_uuid());
    append_u64(&mut bytes, row.revision.get());
    append_text(&mut bytes, row.evaluated_bundle.schema_id());
    bytes.extend_from_slice(row.evaluated_bundle.schema_digest());
    bytes.extend_from_slice(row.evaluated_bundle.policy_digest());
    bytes.extend_from_slice(row.evaluated_bundle.codec_contract_digest());
    bytes.extend_from_slice(row.evaluated_bundle.registration_manifest_digest());
    append_uuid(&mut bytes, *row.first_actor.as_uuid());
    append_uuid(&mut bytes, row.first_command);
    bytes.extend_from_slice(&row.first_input_digest);
    append_uuid(&mut bytes, row.activation_receipt_id);
    append_i64(&mut bytes, row.activated_at_us);
    Ok(bytes)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessStateV1 {
    Active,
    Suspended,
}

impl ProcessStateV1 {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "ACTIVE",
            Self::Suspended => "SUSPENDED",
        }
    }
    pub const fn code(self) -> u16 {
        match self {
            Self::Active => 1,
            Self::Suspended => 2,
        }
    }
    pub fn from_code(value: u16) -> Result<Self, GroupProcessError> {
        match value {
            1 => Ok(Self::Active),
            2 => Ok(Self::Suspended),
            _ => Err(GroupProcessError::Unavailable),
        }
    }
}

pub struct ProcessHeadReferenceProjectionV1 {
    pub process_id: Uuid,
    pub head_revision: u64,
    pub content_version: u64,
    pub content_digest: [u8; 32],
    pub head_digest: [u8; 32],
    pub state: ProcessStateV1,
    pub expiry_us: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessHeadReferenceV1 {
    process_id: Uuid,
    head_revision: u64,
    content_version: u64,
    content_digest: [u8; 32],
    head_digest: [u8; 32],
    state: ProcessStateV1,
    expiry_us: i64,
}

impl ProcessHeadReferenceV1 {
    pub fn from_projection(
        row: ProcessHeadReferenceProjectionV1,
    ) -> Result<Self, GroupProcessError> {
        nonnil(row.process_id)?;
        revision(row.head_revision, false)?;
        revision(row.content_version, false)?;
        time_from_us(row.expiry_us)?;
        if row.content_version > row.head_revision {
            return Err(GroupProcessError::Unavailable);
        }
        Ok(Self {
            process_id: row.process_id,
            head_revision: row.head_revision,
            content_version: row.content_version,
            content_digest: row.content_digest,
            head_digest: row.head_digest,
            state: row.state,
            expiry_us: row.expiry_us,
        })
    }
    pub const fn process_id(&self) -> Uuid {
        self.process_id
    }
    pub const fn head_revision(&self) -> u64 {
        self.head_revision
    }
    pub const fn content_version(&self) -> u64 {
        self.content_version
    }
    pub const fn content_digest(&self) -> &[u8; 32] {
        &self.content_digest
    }
    pub const fn head_digest(&self) -> &[u8; 32] {
        &self.head_digest
    }
    pub const fn state(&self) -> ProcessStateV1 {
        self.state
    }
    pub const fn expiry_us(&self) -> i64 {
        self.expiry_us
    }
    pub fn expiry(&self) -> Result<OffsetDateTime, GroupProcessError> {
        time_from_us(self.expiry_us)
    }
    fn append(&self, bytes: &mut Vec<u8>) {
        append_uuid(bytes, self.process_id);
        append_u64(bytes, self.head_revision);
        append_u64(bytes, self.content_version);
        bytes.extend_from_slice(&self.content_digest);
        bytes.extend_from_slice(&self.head_digest);
        append_u16(bytes, self.state.code());
        append_i64(bytes, self.expiry_us);
    }
    fn read(input: &mut &[u8]) -> Result<Self, GroupProcessError> {
        Self::from_projection(ProcessHeadReferenceProjectionV1 {
            process_id: read_uuid(input)?,
            head_revision: read_revision(input, false)?,
            content_version: read_revision(input, false)?,
            content_digest: read_digest(input)?,
            head_digest: read_digest(input)?,
            state: ProcessStateV1::from_code(read_u16(input)?)?,
            expiry_us: read_i64(input)?,
        })
    }
}

pub struct ProcessHeadDigestProjectionV1 {
    pub group: GroupId,
    pub incarnation: GroupIncarnation,
    pub reference: ProcessHeadReferenceV1,
    pub last_actor: AccountId,
    pub last_command: Uuid,
    pub last_input_digest: [u8; 32],
    pub result_receipt_id: Uuid,
    pub updated_at_us: i64,
}

pub fn encode_group_process_head_v1(
    row: &ProcessHeadDigestProjectionV1,
) -> Result<Vec<u8>, GroupProcessError> {
    nonnil(*row.last_actor.as_uuid())?;
    nonnil(row.last_command)?;
    nonnil(row.result_receipt_id)?;
    time_from_us(row.updated_at_us)?;
    let mut bytes = prefix(b"CONSOLE.IDENTITY.PROCESS.HEAD\0", 1);
    append_uuid(&mut bytes, *row.group.as_uuid());
    append_uuid(&mut bytes, *row.incarnation.as_uuid());
    append_uuid(&mut bytes, row.reference.process_id());
    append_u64(&mut bytes, row.reference.head_revision());
    append_u64(&mut bytes, row.reference.content_version());
    bytes.extend_from_slice(row.reference.content_digest());
    append_u16(&mut bytes, row.reference.state().code());
    append_i64(&mut bytes, row.reference.expiry_us());
    append_uuid(&mut bytes, *row.last_actor.as_uuid());
    append_uuid(&mut bytes, row.last_command);
    bytes.extend_from_slice(&row.last_input_digest);
    append_uuid(&mut bytes, row.result_receipt_id);
    append_i64(&mut bytes, row.updated_at_us);
    Ok(bytes)
}

pub fn encode_group_process_registration_v1(
    group: GroupId,
    incarnation: GroupIncarnation,
    bundle: &EvaluatedPolicyBundleV1,
    registrations: &[super::GroupProcessRegistrationV1],
) -> Result<Vec<u8>, GroupProcessError> {
    if registrations.len() != 4 {
        return Err(GroupProcessError::Unavailable);
    }
    let mut bytes = prefix(b"CONSOLE.IDENTITY.GROUP.PROCESS.REGISTRATION\0", 1);
    append_uuid(&mut bytes, *group.as_uuid());
    append_uuid(&mut bytes, *incarnation.as_uuid());
    append_u64(&mut bytes, 1);
    append_text(&mut bytes, bundle.schema_id());
    bytes.extend_from_slice(bundle.schema_digest());
    bytes.extend_from_slice(bundle.policy_digest());
    bytes.extend_from_slice(bundle.codec_contract_digest());
    append_u16(&mut bytes, 4);
    for (entry, action) in registrations.iter().zip(super::GroupProcessActionV1::ALL) {
        if entry.action() != action
            || entry.fields() != super::group_process_registered_fields(action)
        {
            return Err(GroupProcessError::Unavailable);
        }
        append_text(&mut bytes, action.as_str());
        append_uuid(&mut bytes, entry.action_id());
        append_u64(&mut bytes, 1);
        append_u16(
            &mut bytes,
            u16::try_from(entry.fields().len()).map_err(|_| GroupProcessError::Unavailable)?,
        );
        for field in entry.fields() {
            append_text(&mut bytes, field.as_str());
        }
    }
    Ok(bytes)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessTerminalCodeV1 {
    Adopted,
    Replaced,
    Suspended,
    RejectedStaleExpectation,
    RejectedProcessExpired,
    RejectedAlreadySuspended,
}

impl ProcessTerminalCodeV1 {
    pub const fn code(self) -> u16 {
        match self {
            Self::Adopted => 1,
            Self::Replaced => 2,
            Self::Suspended => 3,
            Self::RejectedStaleExpectation => 4,
            Self::RejectedProcessExpired => 5,
            Self::RejectedAlreadySuspended => 6,
        }
    }
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Adopted => "ADOPTED",
            Self::Replaced => "REPLACED",
            Self::Suspended => "SUSPENDED",
            Self::RejectedStaleExpectation => "REJECTED_STALE_EXPECTATION",
            Self::RejectedProcessExpired => "REJECTED_PROCESS_EXPIRED",
            Self::RejectedAlreadySuspended => "REJECTED_ALREADY_SUSPENDED",
        }
    }
    pub const fn is_success(self) -> bool {
        matches!(self, Self::Adopted | Self::Replaced | Self::Suspended)
    }
    pub fn from_code(value: u16) -> Result<Self, GroupProcessError> {
        match value {
            1 => Ok(Self::Adopted),
            2 => Ok(Self::Replaced),
            3 => Ok(Self::Suspended),
            4 => Ok(Self::RejectedStaleExpectation),
            5 => Ok(Self::RejectedProcessExpired),
            6 => Ok(Self::RejectedAlreadySuspended),
            _ => Err(GroupProcessError::Unavailable),
        }
    }
}

pub struct ProcessResultProjectionV2 {
    pub actor: AccountId,
    pub command: Uuid,
    pub group: GroupId,
    pub incarnation: GroupIncarnation,
    pub opcode: u16,
    pub input_digest: [u8; 32],
    pub intake_receipt: Uuid,
    pub result_receipt: Uuid,
    pub terminal_code: ProcessTerminalCodeV1,
    pub accepted_at_us: i64,
    pub executed_at_us: i64,
    pub session: Uuid,
    pub security_generation: u64,
    pub designation_receipt: Uuid,
    pub designation_revision: u64,
    pub observed_group_revision: u64,
    pub policy_before: PolicyHeadReferenceV1,
    pub policy_after: PolicyHeadReferenceV1,
    pub evaluated_bundle: EvaluatedPolicyBundleV1,
    pub requested_process_id: Uuid,
    pub before_head: Option<ProcessHeadReferenceV1>,
    pub after_head: Option<ProcessHeadReferenceV1>,
    pub effect_xid8: u64,
    pub effect_pid: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessResultV2 {
    actor: AccountId,
    command: Uuid,
    group: GroupId,
    incarnation: GroupIncarnation,
    opcode: u16,
    input_digest: [u8; 32],
    intake_receipt: Uuid,
    result_receipt: Uuid,
    terminal_code: ProcessTerminalCodeV1,
    accepted_at_us: i64,
    executed_at_us: i64,
    session: Uuid,
    security_generation: u64,
    designation_receipt: Uuid,
    designation_revision: u64,
    observed_group_revision: u64,
    policy_before: PolicyHeadReferenceV1,
    policy_after: PolicyHeadReferenceV1,
    evaluated_bundle: EvaluatedPolicyBundleV1,
    requested_process_id: Uuid,
    before_head: Option<ProcessHeadReferenceV1>,
    after_head: Option<ProcessHeadReferenceV1>,
    effect_xid8: u64,
    effect_pid: u32,
}

impl ProcessResultV2 {
    pub fn from_projection(row: ProcessResultProjectionV2) -> Result<Self, GroupProcessError> {
        for id in [
            *row.actor.as_uuid(),
            row.command,
            row.intake_receipt,
            row.result_receipt,
            row.session,
            row.designation_receipt,
            row.requested_process_id,
        ] {
            nonnil(id)?;
        }
        for rev in [
            row.security_generation,
            row.designation_revision,
            row.observed_group_revision,
        ] {
            revision(rev, false)?;
        }
        time_from_us(row.accepted_at_us)?;
        time_from_us(row.executed_at_us)?;
        if ![1, 6].contains(&row.opcode)
            || row.executed_at_us < row.accepted_at_us
            || row.effect_xid8 == 0
            || row.effect_pid == 0
            || row.effect_pid > i32::MAX as u32
            || (row.policy_before == PolicyHeadReferenceV1::Absent) != row.before_head.is_none()
            || (row.policy_after == PolicyHeadReferenceV1::Absent) != row.after_head.is_none()
        {
            return Err(GroupProcessError::Unavailable);
        }
        if !row.terminal_code.is_success() {
            if row.policy_before != row.policy_after
                || row.before_head != row.after_head
                || (row.terminal_code == ProcessTerminalCodeV1::RejectedAlreadySuspended
                    && row.opcode != 6)
                || (row.policy_before == PolicyHeadReferenceV1::Absent
                    && (row.opcode != 1
                        || row.terminal_code == ProcessTerminalCodeV1::RejectedAlreadySuspended))
            {
                return Err(GroupProcessError::Unavailable);
            }
        } else {
            let after = row
                .after_head
                .as_ref()
                .ok_or(GroupProcessError::Unavailable)?;
            if after.process_id != row.requested_process_id
                || after.expiry_us <= row.executed_at_us
                || !matches!(row.policy_after, PolicyHeadReferenceV1::Installed { revision, .. } if revision.get() == 1)
            {
                return Err(GroupProcessError::Unavailable);
            }
            match row.terminal_code {
                ProcessTerminalCodeV1::Adopted => {
                    if row.opcode != 1
                        || row.before_head.is_some()
                        || row.policy_before != PolicyHeadReferenceV1::Absent
                        || after.head_revision != 1
                        || after.content_version != 1
                        || after.state != ProcessStateV1::Active
                    {
                        return Err(GroupProcessError::Unavailable);
                    }
                }
                ProcessTerminalCodeV1::Replaced | ProcessTerminalCodeV1::Suspended => {
                    let before = row
                        .before_head
                        .as_ref()
                        .ok_or(GroupProcessError::Unavailable)?;
                    if row.policy_before != row.policy_after
                        || before.process_id != after.process_id
                        || before.head_revision.checked_add(1) != Some(after.head_revision)
                    {
                        return Err(GroupProcessError::Unavailable);
                    }
                    if row.terminal_code == ProcessTerminalCodeV1::Replaced {
                        if row.opcode != 1
                            || before.content_version.checked_add(1) != Some(after.content_version)
                            || after.state != ProcessStateV1::Active
                        {
                            return Err(GroupProcessError::Unavailable);
                        }
                    } else if row.opcode != 6
                        || before.content_version != after.content_version
                        || before.content_digest != after.content_digest
                        || before.expiry_us != after.expiry_us
                        || before.state != ProcessStateV1::Active
                        || after.state != ProcessStateV1::Suspended
                    {
                        return Err(GroupProcessError::Unavailable);
                    }
                }
                _ => return Err(GroupProcessError::Unavailable),
            }
        }
        Ok(Self {
            actor: row.actor,
            command: row.command,
            group: row.group,
            incarnation: row.incarnation,
            opcode: row.opcode,
            input_digest: row.input_digest,
            intake_receipt: row.intake_receipt,
            result_receipt: row.result_receipt,
            terminal_code: row.terminal_code,
            accepted_at_us: row.accepted_at_us,
            executed_at_us: row.executed_at_us,
            session: row.session,
            security_generation: row.security_generation,
            designation_receipt: row.designation_receipt,
            designation_revision: row.designation_revision,
            observed_group_revision: row.observed_group_revision,
            policy_before: row.policy_before,
            policy_after: row.policy_after,
            evaluated_bundle: row.evaluated_bundle,
            requested_process_id: row.requested_process_id,
            before_head: row.before_head,
            after_head: row.after_head,
            effect_xid8: row.effect_xid8,
            effect_pid: row.effect_pid,
        })
    }
    pub const fn actor(&self) -> AccountId {
        self.actor
    }
    pub const fn command_id(&self) -> Uuid {
        self.command
    }
    pub const fn group(&self) -> GroupId {
        self.group
    }
    pub const fn incarnation(&self) -> GroupIncarnation {
        self.incarnation
    }
    pub const fn opcode(&self) -> u16 {
        self.opcode
    }
    pub const fn input_digest(&self) -> &[u8; 32] {
        &self.input_digest
    }
    pub const fn intake_receipt(&self) -> Uuid {
        self.intake_receipt
    }
    pub const fn result_receipt(&self) -> Uuid {
        self.result_receipt
    }
    pub const fn terminal_code(&self) -> ProcessTerminalCodeV1 {
        self.terminal_code
    }
    pub const fn accepted_at_us(&self) -> i64 {
        self.accepted_at_us
    }
    pub const fn executed_at_us(&self) -> i64 {
        self.executed_at_us
    }
    pub const fn session(&self) -> Uuid {
        self.session
    }
    pub const fn security_generation(&self) -> u64 {
        self.security_generation
    }
    pub const fn designation_receipt(&self) -> Uuid {
        self.designation_receipt
    }
    pub const fn designation_revision(&self) -> u64 {
        self.designation_revision
    }
    pub const fn observed_group_revision(&self) -> u64 {
        self.observed_group_revision
    }
    pub const fn policy_before(&self) -> PolicyHeadReferenceV1 {
        self.policy_before
    }
    pub const fn policy_after(&self) -> PolicyHeadReferenceV1 {
        self.policy_after
    }
    pub fn evaluated_bundle(&self) -> &EvaluatedPolicyBundleV1 {
        &self.evaluated_bundle
    }
    pub const fn requested_process_id(&self) -> Uuid {
        self.requested_process_id
    }
    pub fn before_head(&self) -> Option<&ProcessHeadReferenceV1> {
        self.before_head.as_ref()
    }
    pub fn after_head(&self) -> Option<&ProcessHeadReferenceV1> {
        self.after_head.as_ref()
    }
    pub const fn effect_xid8(&self) -> u64 {
        self.effect_xid8
    }
    pub const fn effect_pid(&self) -> u32 {
        self.effect_pid
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = prefix(RESULT_PREFIX, 2);
        for id in [
            *self.actor.as_uuid(),
            self.command,
            *self.group.as_uuid(),
            *self.incarnation.as_uuid(),
        ] {
            append_uuid(&mut bytes, id);
        }
        append_u16(&mut bytes, self.opcode);
        bytes.extend_from_slice(&self.input_digest);
        append_uuid(&mut bytes, self.intake_receipt);
        append_uuid(&mut bytes, self.result_receipt);
        append_u16(&mut bytes, self.terminal_code.code());
        append_i64(&mut bytes, self.accepted_at_us);
        append_i64(&mut bytes, self.executed_at_us);
        append_uuid(&mut bytes, self.session);
        append_u64(&mut bytes, self.security_generation);
        append_uuid(&mut bytes, self.designation_receipt);
        append_u64(&mut bytes, self.designation_revision);
        append_u64(&mut bytes, self.observed_group_revision);
        self.policy_before.append(&mut bytes);
        self.policy_after.append(&mut bytes);
        self.evaluated_bundle.append(&mut bytes);
        append_uuid(&mut bytes, self.requested_process_id);
        for head in [&self.before_head, &self.after_head] {
            bytes.push(u8::from(head.is_some()));
            if let Some(head) = head {
                head.append(&mut bytes);
            }
        }
        append_u64(&mut bytes, self.effect_xid8);
        bytes.extend_from_slice(&self.effect_pid.to_be_bytes());
        bytes
    }

    pub fn decode(mut input: &[u8]) -> Result<Self, GroupProcessError> {
        if input.len() > 4096 {
            return Err(GroupProcessError::Unavailable);
        }
        read_prefix(&mut input, RESULT_PREFIX, 2)?;
        let row = ProcessResultProjectionV2 {
            actor: AccountId::from_uuid(read_uuid(&mut input)?)
                .map_err(|_| GroupProcessError::Unavailable)?,
            command: read_uuid(&mut input)?,
            group: GroupId::from_uuid(read_uuid(&mut input)?)
                .map_err(|_| GroupProcessError::Unavailable)?,
            incarnation: GroupIncarnation::from_uuid(read_uuid(&mut input)?)
                .map_err(|_| GroupProcessError::Unavailable)?,
            opcode: read_u16(&mut input)?,
            input_digest: read_digest(&mut input)?,
            intake_receipt: read_uuid(&mut input)?,
            result_receipt: read_uuid(&mut input)?,
            terminal_code: ProcessTerminalCodeV1::from_code(read_u16(&mut input)?)?,
            accepted_at_us: read_i64(&mut input)?,
            executed_at_us: read_i64(&mut input)?,
            session: read_uuid(&mut input)?,
            security_generation: read_revision(&mut input, false)?,
            designation_receipt: read_uuid(&mut input)?,
            designation_revision: read_revision(&mut input, false)?,
            observed_group_revision: read_revision(&mut input, false)?,
            policy_before: PolicyHeadReferenceV1::read(&mut input)?,
            policy_after: PolicyHeadReferenceV1::read(&mut input)?,
            evaluated_bundle: EvaluatedPolicyBundleV1::read(&mut input)?,
            requested_process_id: read_uuid(&mut input)?,
            before_head: read_optional_head(&mut input)?,
            after_head: read_optional_head(&mut input)?,
            effect_xid8: read_u64(&mut input)?,
            effect_pid: u32::from_be_bytes(
                take(&mut input, 4)?
                    .try_into()
                    .map_err(|_| GroupProcessError::Unavailable)?,
            ),
        };
        if !input.is_empty() {
            return Err(GroupProcessError::Unavailable);
        }
        Self::from_projection(row)
    }
}

fn read_optional_head(
    input: &mut &[u8],
) -> Result<Option<ProcessHeadReferenceV1>, GroupProcessError> {
    match take(input, 1)? {
        [0] => Ok(None),
        [1] => Ok(Some(ProcessHeadReferenceV1::read(input)?)),
        _ => Err(GroupProcessError::Unavailable),
    }
}

pub fn exact_time_us(value: OffsetDateTime) -> Result<i64, GroupProcessError> {
    let nanos = value.unix_timestamp_nanos();
    if nanos % 1000 != 0 {
        return Err(GroupProcessError::Unavailable);
    }
    i64::try_from(nanos / 1000).map_err(|_| GroupProcessError::Unavailable)
}
pub fn time_from_us(value: i64) -> Result<OffsetDateTime, GroupProcessError> {
    OffsetDateTime::from_unix_timestamp_nanos(i128::from(value) * 1000)
        .map_err(|_| GroupProcessError::Unavailable)
}
fn prefix(value: &[u8], version: u16) -> Vec<u8> {
    let mut bytes = value.to_vec();
    append_u16(&mut bytes, version);
    bytes
}
fn append_u16(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_be_bytes());
}
fn append_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_be_bytes());
}
fn append_i64(bytes: &mut Vec<u8>, value: i64) {
    bytes.extend_from_slice(&value.to_be_bytes());
}
fn append_uuid(bytes: &mut Vec<u8>, value: Uuid) {
    bytes.extend_from_slice(value.as_bytes());
}
fn append_text(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend_from_slice(&(value.len() as u32).to_be_bytes());
    bytes.extend_from_slice(value.as_bytes());
}
fn take<'a>(input: &mut &'a [u8], len: usize) -> Result<&'a [u8], GroupProcessError> {
    let (value, remaining) = input
        .split_at_checked(len)
        .ok_or(GroupProcessError::Unavailable)?;
    *input = remaining;
    Ok(value)
}
fn read_prefix(input: &mut &[u8], value: &[u8], version: u16) -> Result<(), GroupProcessError> {
    if take(input, value.len())? != value || read_u16(input)? != version {
        return Err(GroupProcessError::Unavailable);
    }
    Ok(())
}
fn read_u16(input: &mut &[u8]) -> Result<u16, GroupProcessError> {
    Ok(u16::from_be_bytes(
        take(input, 2)?
            .try_into()
            .map_err(|_| GroupProcessError::Unavailable)?,
    ))
}
fn read_u64(input: &mut &[u8]) -> Result<u64, GroupProcessError> {
    Ok(u64::from_be_bytes(
        take(input, 8)?
            .try_into()
            .map_err(|_| GroupProcessError::Unavailable)?,
    ))
}
fn read_i64(input: &mut &[u8]) -> Result<i64, GroupProcessError> {
    Ok(i64::from_be_bytes(
        take(input, 8)?
            .try_into()
            .map_err(|_| GroupProcessError::Unavailable)?,
    ))
}
fn read_revision(input: &mut &[u8], zero: bool) -> Result<u64, GroupProcessError> {
    revision(read_u64(input)?, zero)
}
fn read_uuid(input: &mut &[u8]) -> Result<Uuid, GroupProcessError> {
    nonnil(Uuid::from_slice(take(input, 16)?).map_err(|_| GroupProcessError::Unavailable)?)
}
fn read_digest(input: &mut &[u8]) -> Result<[u8; 32], GroupProcessError> {
    take(input, 32)?
        .try_into()
        .map_err(|_| GroupProcessError::Unavailable)
}
fn read_text(input: &mut &[u8], max: usize) -> Result<String, GroupProcessError> {
    let len = u32::from_be_bytes(
        take(input, 4)?
            .try_into()
            .map_err(|_| GroupProcessError::Unavailable)?,
    ) as usize;
    if len == 0 || len > max {
        return Err(GroupProcessError::Unavailable);
    }
    let value =
        std::str::from_utf8(take(input, len)?).map_err(|_| GroupProcessError::Unavailable)?;
    text(value, max)?;
    Ok(value.to_owned())
}
