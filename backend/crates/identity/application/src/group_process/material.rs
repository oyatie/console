//! Purpose-specific protected material. Raw rows never confer authorization;
//! checked values are tied to actual Auth, transaction and source observations.
use super::*;
use console_kernel_core::TraceContext;
use std::future::Future;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum GroupProcessActionV1 {
    Adopt,
    Suspend,
    Read,
    ReadOwnReceipt,
}

impl GroupProcessActionV1 {
    pub const ALL: [Self; 4] = [Self::Adopt, Self::Suspend, Self::Read, Self::ReadOwnReceipt];
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Adopt => "identity.verifier.process.adopt/1",
            Self::Suspend => "identity.verifier.process.suspend/1",
            Self::Read => "identity.verifier.process.read/1",
            Self::ReadOwnReceipt => "identity.verifier.process.receipt.read-own/1",
        }
    }
    pub fn from_str(value: &str) -> Result<Self, GroupProcessError> {
        Self::ALL
            .into_iter()
            .find(|action| action.as_str() == value)
            .ok_or(GroupProcessError::Unavailable)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum GroupProcessFieldV1 {
    GroupContext,
    ProcessId,
    ProcessVersion,
    ProcessDigest,
    ProcessHeadRevision,
    ProcessHeadDigest,
    State,
    Expiry,
    Title,
    Method,
    IntendedClaimantMatchingProcedure,
    AccountPossessionProcedure,
    PhysicalHumanEvidenceProcedure,
    DuplicateContradictoryClaimProcedure,
    QualificationCriteriaInstruction,
    EscalationAdjudicationProcedure,
    EvidenceMinimizationRetentionDescription,
    RecipientResponsibility,
    History,
    AllowedActions,
    ReceiptLocator,
    Reason,
    CommandId,
    InputDigest,
    IntakeReceiptId,
    ResultReceiptId,
    TerminalCode,
    AcceptedAt,
    ExecutedAt,
    BeforeHead,
    AfterHead,
    OriginalContent,
    OriginalReason,
}

impl GroupProcessFieldV1 {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::GroupContext => "group_context",
            Self::ProcessId => "process_id",
            Self::ProcessVersion => "process_version",
            Self::ProcessDigest => "process_digest",
            Self::ProcessHeadRevision => "process_head_revision",
            Self::ProcessHeadDigest => "process_head_digest",
            Self::State => "state",
            Self::Expiry => "expiry",
            Self::Title => "title",
            Self::Method => "method",
            Self::IntendedClaimantMatchingProcedure => "intended_claimant_matching_procedure",
            Self::AccountPossessionProcedure => "account_possession_procedure",
            Self::PhysicalHumanEvidenceProcedure => "physical_human_evidence_procedure",
            Self::DuplicateContradictoryClaimProcedure => "duplicate_contradictory_claim_procedure",
            Self::QualificationCriteriaInstruction => "qualification_criteria_instruction",
            Self::EscalationAdjudicationProcedure => "escalation_adjudication_procedure",
            Self::EvidenceMinimizationRetentionDescription => {
                "evidence_minimization_retention_description"
            }
            Self::RecipientResponsibility => "recipient_responsibility",
            Self::History => "history",
            Self::AllowedActions => "allowed_actions",
            Self::ReceiptLocator => "receipt_locator",
            Self::Reason => "reason",
            Self::CommandId => "command_id",
            Self::InputDigest => "input_digest",
            Self::IntakeReceiptId => "intake_receipt_id",
            Self::ResultReceiptId => "result_receipt_id",
            Self::TerminalCode => "terminal_code",
            Self::AcceptedAt => "accepted_at",
            Self::ExecutedAt => "executed_at",
            Self::BeforeHead => "before_head",
            Self::AfterHead => "after_head",
            Self::OriginalContent => "original_content",
            Self::OriginalReason => "original_reason",
        }
    }
    pub fn from_str(value: &str) -> Result<Self, GroupProcessError> {
        let all = [
            Self::GroupContext,
            Self::ProcessId,
            Self::ProcessVersion,
            Self::ProcessDigest,
            Self::ProcessHeadRevision,
            Self::ProcessHeadDigest,
            Self::State,
            Self::Expiry,
            Self::Title,
            Self::Method,
            Self::IntendedClaimantMatchingProcedure,
            Self::AccountPossessionProcedure,
            Self::PhysicalHumanEvidenceProcedure,
            Self::DuplicateContradictoryClaimProcedure,
            Self::QualificationCriteriaInstruction,
            Self::EscalationAdjudicationProcedure,
            Self::EvidenceMinimizationRetentionDescription,
            Self::RecipientResponsibility,
            Self::History,
            Self::AllowedActions,
            Self::ReceiptLocator,
            Self::Reason,
            Self::CommandId,
            Self::InputDigest,
            Self::IntakeReceiptId,
            Self::ResultReceiptId,
            Self::TerminalCode,
            Self::AcceptedAt,
            Self::ExecutedAt,
            Self::BeforeHead,
            Self::AfterHead,
            Self::OriginalContent,
            Self::OriginalReason,
        ];
        all.into_iter()
            .find(|field| field.as_str() == value)
            .ok_or(GroupProcessError::Unavailable)
    }
}

use GroupProcessFieldV1 as Field;
const ADOPT_FIELDS: &[Field] = &[
    Field::ProcessId,
    Field::ProcessVersion,
    Field::ProcessDigest,
    Field::ProcessHeadRevision,
    Field::ProcessHeadDigest,
    Field::State,
    Field::Expiry,
    Field::Title,
    Field::Method,
    Field::IntendedClaimantMatchingProcedure,
    Field::AccountPossessionProcedure,
    Field::PhysicalHumanEvidenceProcedure,
    Field::DuplicateContradictoryClaimProcedure,
    Field::QualificationCriteriaInstruction,
    Field::EscalationAdjudicationProcedure,
    Field::EvidenceMinimizationRetentionDescription,
    Field::RecipientResponsibility,
    Field::ReceiptLocator,
];
const SUSPEND_FIELDS: &[Field] = &[
    Field::ProcessId,
    Field::ProcessVersion,
    Field::ProcessDigest,
    Field::ProcessHeadRevision,
    Field::ProcessHeadDigest,
    Field::State,
    Field::Expiry,
    Field::Reason,
    Field::ReceiptLocator,
];
const READ_FIELDS: &[Field] = &[
    Field::GroupContext,
    Field::ProcessId,
    Field::ProcessVersion,
    Field::ProcessDigest,
    Field::ProcessHeadRevision,
    Field::ProcessHeadDigest,
    Field::State,
    Field::Expiry,
    Field::Title,
    Field::Method,
    Field::IntendedClaimantMatchingProcedure,
    Field::AccountPossessionProcedure,
    Field::PhysicalHumanEvidenceProcedure,
    Field::DuplicateContradictoryClaimProcedure,
    Field::QualificationCriteriaInstruction,
    Field::EscalationAdjudicationProcedure,
    Field::EvidenceMinimizationRetentionDescription,
    Field::RecipientResponsibility,
    Field::History,
    Field::AllowedActions,
];
const OWN_FIELDS: &[Field] = &[
    Field::CommandId,
    Field::InputDigest,
    Field::IntakeReceiptId,
    Field::ResultReceiptId,
    Field::TerminalCode,
    Field::AcceptedAt,
    Field::ExecutedAt,
    Field::ProcessId,
    Field::BeforeHead,
    Field::AfterHead,
    Field::OriginalContent,
    Field::OriginalReason,
    Field::ReceiptLocator,
];
const EMPTY_FIELDS: &[Field] = &[Field::GroupContext, Field::AllowedActions];
const RETRY_FORM_FIELDS: &[Field] = &[Field::CommandId, Field::ReceiptLocator];

pub fn group_process_registered_fields(action: GroupProcessActionV1) -> &'static [Field] {
    match action {
        GroupProcessActionV1::Adopt => ADOPT_FIELDS,
        GroupProcessActionV1::Suspend => SUSPEND_FIELDS,
        GroupProcessActionV1::Read => READ_FIELDS,
        GroupProcessActionV1::ReadOwnReceipt => OWN_FIELDS,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroupProcessResourceKindV1 {
    ProcessMutation,
    CurrentProcess,
    ProcessForm,
    ProcessLanding,
    OwnReceipt,
    OwnReceiptRetryForm,
}
impl GroupProcessResourceKindV1 {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProcessMutation => "PROCESS_MUTATION",
            Self::CurrentProcess => "CURRENT_PROCESS",
            Self::ProcessForm => "PROCESS_FORM",
            Self::ProcessLanding => "PROCESS_LANDING",
            Self::OwnReceipt => "OWN_RECEIPT",
            Self::OwnReceiptRetryForm => "OWN_RECEIPT_RETRY_FORM",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroupProcessBootstrapStageV1 {
    None,
    EmptyForm,
    FirstIntake,
    FirstExecution,
    EmptyLanding,
}
impl GroupProcessBootstrapStageV1 {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "NONE",
            Self::EmptyForm => "EMPTY_FORM",
            Self::FirstIntake => "FIRST_INTAKE",
            Self::FirstExecution => "FIRST_EXECUTION",
            Self::EmptyLanding => "EMPTY_LANDING",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupProcessPolicyRequestV1 {
    action: GroupProcessActionV1,
    resource_kind: GroupProcessResourceKindV1,
    requested_fields: Vec<Field>,
}
impl GroupProcessPolicyRequestV1 {
    pub fn new(
        action: GroupProcessActionV1,
        resource_kind: GroupProcessResourceKindV1,
        requested_fields: Vec<Field>,
    ) -> Result<Self, GroupProcessError> {
        let exact = match (action, resource_kind) {
            (GroupProcessActionV1::Adopt, GroupProcessResourceKindV1::ProcessMutation) => {
                ADOPT_FIELDS
            }
            (GroupProcessActionV1::Suspend, GroupProcessResourceKindV1::ProcessMutation) => {
                SUSPEND_FIELDS
            }
            (GroupProcessActionV1::Read, GroupProcessResourceKindV1::CurrentProcess) => READ_FIELDS,
            (GroupProcessActionV1::Read, GroupProcessResourceKindV1::ProcessForm) => READ_FIELDS,
            (GroupProcessActionV1::Read, GroupProcessResourceKindV1::ProcessLanding) => {
                EMPTY_FIELDS
            }
            (GroupProcessActionV1::ReadOwnReceipt, GroupProcessResourceKindV1::OwnReceipt) => {
                OWN_FIELDS
            }
            (
                GroupProcessActionV1::ReadOwnReceipt,
                GroupProcessResourceKindV1::OwnReceiptRetryForm,
            ) => RETRY_FORM_FIELDS,
            _ => return Err(GroupProcessError::Unavailable),
        };
        // Empty PROCESS_FORM requests are metadata only; installed forms request
        // the full current projection. All other request shapes are exact.
        if requested_fields != exact
            && !(resource_kind == GroupProcessResourceKindV1::ProcessForm
                && requested_fields == EMPTY_FIELDS)
        {
            return Err(GroupProcessError::Unavailable);
        }
        Ok(Self {
            action,
            resource_kind,
            requested_fields,
        })
    }
    pub const fn action(&self) -> GroupProcessActionV1 {
        self.action
    }
    pub const fn resource_kind(&self) -> GroupProcessResourceKindV1 {
        self.resource_kind
    }
    pub fn requested_fields(&self) -> &[Field] {
        &self.requested_fields
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroupProcessDecision {
    Allow,
    Deny,
}
pub trait GroupProcessDecisionPort: Sync {
    fn decide(
        &self,
        authority: &CurrentGroupProcessAuthority,
        request: &GroupProcessPolicyRequestV1,
    ) -> Result<GroupProcessDecision, GroupProcessError>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GroupProcessLocator {
    group: GroupId,
    incarnation: GroupIncarnation,
    command_id: Uuid,
}
impl GroupProcessLocator {
    pub fn new(
        group: GroupId,
        incarnation: GroupIncarnation,
        command_id: Uuid,
    ) -> Result<Self, GroupProcessError> {
        nonnil(command_id).map_err(|_| GroupProcessError::InvalidInput)?;
        Ok(Self {
            group,
            incarnation,
            command_id,
        })
    }
    pub fn from_command(input: &GroupProcessCommandV1) -> Self {
        Self {
            group: input.group(),
            incarnation: input.incarnation(),
            command_id: input.command_id(),
        }
    }
    pub const fn group(self) -> GroupId {
        self.group
    }
    pub const fn incarnation(self) -> GroupIncarnation {
        self.incarnation
    }
    pub const fn command_id(self) -> Uuid {
        self.command_id
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GroupProcessRouteSelectorV1 {
    group: GroupId,
    command_id: Uuid,
}
impl GroupProcessRouteSelectorV1 {
    pub fn new(group: GroupId, command_id: Uuid) -> Result<Self, GroupProcessError> {
        nonnil(command_id).map_err(|_| GroupProcessError::InvalidInput)?;
        Ok(Self { group, command_id })
    }
    pub const fn group(self) -> GroupId {
        self.group
    }
    pub const fn command_id(self) -> Uuid {
        self.command_id
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i16)]
pub enum GroupProcessMaterialModeV1 {
    Current = 1,
    Form = 2,
    Accept = 3,
    Execute = 4,
    Status = 5,
    RetryResolve = 6,
    Landing = 7,
    OwnRetryForm = 8,
}
impl GroupProcessMaterialModeV1 {
    pub const fn code(self) -> i16 {
        self as i16
    }
    pub fn from_code(value: i16) -> Result<Self, GroupProcessError> {
        match value {
            1 => Ok(Self::Current),
            2 => Ok(Self::Form),
            3 => Ok(Self::Accept),
            4 => Ok(Self::Execute),
            5 => Ok(Self::Status),
            6 => Ok(Self::RetryResolve),
            7 => Ok(Self::Landing),
            8 => Ok(Self::OwnRetryForm),
            _ => Err(GroupProcessError::Unavailable),
        }
    }
}

pub enum GroupProcessScopeRequest<'a> {
    Current {
        group: GroupId,
        incarnation: GroupIncarnation,
    },
    Form {
        group: GroupId,
        incarnation: GroupIncarnation,
    },
    ValidationForm {
        original: GroupProcessLocator,
        process_id: Uuid,
    },
    Accept(&'a GroupProcessCommandV1),
    Execute(GroupProcessLocator),
    Status(GroupProcessLocator),
    RetryResolve(GroupProcessLocator),
    Landing {
        group: GroupId,
        incarnation: GroupIncarnation,
    },
    OwnRetryForm(GroupProcessLocator),
}
impl GroupProcessScopeRequest<'_> {
    pub const fn mode(&self) -> GroupProcessMaterialModeV1 {
        match self {
            Self::Current { .. } => GroupProcessMaterialModeV1::Current,
            Self::Form { .. } | Self::ValidationForm { .. } => GroupProcessMaterialModeV1::Form,
            Self::Accept(_) => GroupProcessMaterialModeV1::Accept,
            Self::Execute(_) => GroupProcessMaterialModeV1::Execute,
            Self::Status(_) => GroupProcessMaterialModeV1::Status,
            Self::RetryResolve(_) => GroupProcessMaterialModeV1::RetryResolve,
            Self::Landing { .. } => GroupProcessMaterialModeV1::Landing,
            Self::OwnRetryForm(_) => GroupProcessMaterialModeV1::OwnRetryForm,
        }
    }
    pub fn group(&self) -> GroupId {
        match self {
            Self::Current { group, .. }
            | Self::Form { group, .. }
            | Self::Landing { group, .. } => *group,
            Self::ValidationForm { original, .. } => original.group(),
            Self::Accept(input) => input.group(),
            Self::Execute(locator)
            | Self::Status(locator)
            | Self::RetryResolve(locator)
            | Self::OwnRetryForm(locator) => locator.group,
        }
    }
    pub fn incarnation(&self) -> GroupIncarnation {
        match self {
            Self::Current { incarnation, .. }
            | Self::Form { incarnation, .. }
            | Self::Landing { incarnation, .. } => *incarnation,
            Self::ValidationForm { original, .. } => original.incarnation(),
            Self::Accept(input) => input.incarnation(),
            Self::Execute(locator)
            | Self::Status(locator)
            | Self::RetryResolve(locator)
            | Self::OwnRetryForm(locator) => locator.incarnation,
        }
    }
    pub fn locator(&self) -> Option<GroupProcessLocator> {
        match self {
            Self::Accept(input) => Some(GroupProcessLocator::from_command(input)),
            Self::Execute(locator)
            | Self::Status(locator)
            | Self::RetryResolve(locator)
            | Self::OwnRetryForm(locator) => Some(*locator),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupProcessDeploymentV1 {
    system_identifier: String,
    database_name: String,
    database_oid: u32,
}
impl GroupProcessDeploymentV1 {
    pub fn new(
        system_identifier: String,
        database_name: String,
        database_oid: u32,
    ) -> Result<Self, GroupProcessError> {
        if system_identifier.is_empty()
            || system_identifier.len() > 20
            || system_identifier.starts_with('0')
            || !system_identifier.bytes().all(|c| c.is_ascii_digit())
            || system_identifier.parse::<u64>().ok().is_none_or(|n| n == 0)
            || database_oid == 0
        {
            return Err(GroupProcessError::Unavailable);
        }
        text(&database_name, 63)?;
        if database_name.chars().any(char::is_control) {
            return Err(GroupProcessError::Unavailable);
        }
        Ok(Self {
            system_identifier,
            database_name,
            database_oid,
        })
    }
    pub fn system_identifier(&self) -> &str {
        &self.system_identifier
    }
    pub fn database_name(&self) -> &str {
        &self.database_name
    }
    pub const fn database_oid(&self) -> u32 {
        self.database_oid
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GroupProcessRetainedBindingV1 {
    actor: AccountId,
    session: Uuid,
    security_generation: u64,
    observed_at_us: i64,
    xid8: u64,
    backend_pid: u32,
}
impl GroupProcessRetainedBindingV1 {
    /// The adapter obtains these values independently from actual Auth and its
    /// live transaction; this syntactic constructor is not authentication.
    pub fn new(
        actor: AccountId,
        session: Uuid,
        security_generation: u64,
        observed_at_us: i64,
        xid8: u64,
        backend_pid: u32,
    ) -> Result<Self, GroupProcessError> {
        nonnil(*actor.as_uuid())?;
        nonnil(session)?;
        revision(security_generation, false)?;
        time_from_us(observed_at_us)?;
        if xid8 == 0 || backend_pid == 0 || backend_pid > i32::MAX as u32 {
            return Err(GroupProcessError::Unavailable);
        }
        Ok(Self {
            actor,
            session,
            security_generation,
            observed_at_us,
            xid8,
            backend_pid,
        })
    }
    pub const fn account(self) -> AccountId {
        self.actor
    }
    pub const fn session(self) -> Uuid {
        self.session
    }
    pub const fn security_generation(self) -> u64 {
        self.security_generation
    }
    pub const fn observed_at_us(self) -> i64 {
        self.observed_at_us
    }
    pub const fn xid8(self) -> u64 {
        self.xid8
    }
    pub const fn backend_pid(self) -> u32 {
        self.backend_pid
    }
}

pub struct GroupProcessAccountProjectionV1 {
    pub actor: AccountId,
    pub session: Uuid,
    pub security_generation: u64,
    pub state: String,
    pub registration_receipt: Uuid,
    pub current_terms_receipt: Uuid,
    pub observed_at_us: i64,
    pub xid8: u64,
    pub backend_pid: u32,
}
impl GroupProcessAccountProjectionV1 {
    fn check(&self, binding: GroupProcessRetainedBindingV1) -> Result<(), GroupProcessError> {
        nonnil(self.registration_receipt)?;
        nonnil(self.current_terms_receipt)?;
        if self.actor != binding.actor
            || self.session != binding.session
            || self.security_generation != binding.security_generation
        {
            return Err(GroupProcessError::AuthenticationInvalid);
        }
        // The binding observes the real database independently after protected
        // material. Bounded statement timeout is 10s; two clock samples are
        // ordered, never required to be accidentally identical.
        if self.observed_at_us > binding.observed_at_us
            || i128::from(binding.observed_at_us) - i128::from(self.observed_at_us) > 10_000_000
            || self.xid8 != binding.xid8
            || self.backend_pid != binding.backend_pid
        {
            return Err(GroupProcessError::Unavailable);
        }
        if self.state != "ACTIVE" {
            return Err(GroupProcessError::AuthenticationInvalid);
        }
        Ok(())
    }
}

pub struct GroupProcessRegistrationProjectionV1 {
    pub action: GroupProcessActionV1,
    pub action_id: Uuid,
    pub revision: u64,
    pub fields: Vec<Field>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupProcessRegistrationV1 {
    action: GroupProcessActionV1,
    action_id: Uuid,
    fields: Vec<Field>,
}
impl GroupProcessRegistrationV1 {
    fn checked(
        rows: Vec<GroupProcessRegistrationProjectionV1>,
    ) -> Result<Vec<Self>, GroupProcessError> {
        if rows.len() != 4 {
            return Err(GroupProcessError::Unavailable);
        }
        let mut ids = std::collections::BTreeSet::new();
        rows.into_iter()
            .zip(GroupProcessActionV1::ALL)
            .map(|(row, expected)| {
                nonnil(row.action_id)?;
                if row.action != expected
                    || row.revision != 1
                    || row.fields != group_process_registered_fields(expected)
                    || !ids.insert(row.action_id)
                {
                    return Err(GroupProcessError::Unavailable);
                }
                Ok(Self {
                    action: row.action,
                    action_id: row.action_id,
                    fields: row.fields,
                })
            })
            .collect()
    }
    pub const fn action(&self) -> GroupProcessActionV1 {
        self.action
    }
    pub const fn action_id(&self) -> Uuid {
        self.action_id
    }
    pub fn fields(&self) -> &[Field] {
        &self.fields
    }
}

/// Validate the finite source roster and reconstruct its digest preimage.
/// This pure check grants no Account, designation or resource authority; it
/// also verifies navigation observations for candidates denied by current policy.
pub fn encode_group_process_registration_projection_v1(
    group: GroupId,
    incarnation: GroupIncarnation,
    bundle: &EvaluatedPolicyBundleV1,
    rows: Vec<GroupProcessRegistrationProjectionV1>,
) -> Result<Vec<u8>, GroupProcessError> {
    encode_group_process_registration_v1(
        group,
        incarnation,
        bundle,
        &GroupProcessRegistrationV1::checked(rows)?,
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupProcessContextV1 {
    pub group: GroupId,
    pub incarnation: GroupIncarnation,
    pub label: String,
    pub revision: u64,
    pub account: AccountId,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessHeadV1 {
    pub reference: ProcessHeadReferenceV1,
    pub causing_receipt: Uuid,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupProcessHistoryViewV1 {
    pub head: ProcessHeadV1,
    pub actor: AccountId,
    pub command_id: Uuid,
    pub occurred_at_us: i64,
    pub reason: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupProcessCurrentView {
    pub context: GroupProcessContextV1,
    /// Actual retained database observation, never browser or response time.
    pub observed_at_us: i64,
    pub policy_head: PolicyHeadReferenceV1,
    pub head: Option<ProcessHeadV1>,
    pub content: Option<ProcessContentV1>,
    pub history: Vec<GroupProcessHistoryViewV1>,
    pub allowed_actions: Vec<GroupProcessActionV1>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupProcessLandingMetadataV1 {
    pub context: GroupProcessContextV1,
    pub allowed_actions: Vec<GroupProcessActionV1>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GroupProcessLandingViewV1 {
    Empty(GroupProcessLandingMetadataV1),
    Current(GroupProcessCurrentView),
}
pub struct GroupProcessForm<F> {
    pub view: GroupProcessCurrentView,
    pub command_id: Uuid,
    pub process_id: Uuid,
    pub proof: F,
}
pub struct GroupProcessOwnRetryFormV1<F> {
    pub original: GroupProcessLocator,
    pub proof: F,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupProcessAcceptedV1 {
    pub actor: AccountId,
    pub input: GroupProcessCommandV1,
    pub input_bytes: Vec<u8>,
    pub input_digest: [u8; 32],
    pub intake_receipt: Uuid,
    pub accepted_at_us: i64,
}
impl GroupProcessAcceptedV1 {
    /// Adapter verifies SHA-256 independently before admitting this projection.
    pub fn checked(
        actor: AccountId,
        input_bytes: Vec<u8>,
        input_digest: [u8; 32],
        intake_receipt: Uuid,
        accepted_at_us: i64,
    ) -> Result<Self, GroupProcessError> {
        let (encoded_actor, input) = GroupProcessCommandV1::decode(&input_bytes)?;
        if encoded_actor != actor || input.encode(actor)? != input_bytes {
            return Err(GroupProcessError::Unavailable);
        }
        nonnil(intake_receipt)?;
        time_from_us(accepted_at_us)?;
        if let Some(adopt) = input.adoption() {
            let expiry = exact_time_us(adopt.expiry())?;
            if expiry <= accepted_at_us
                || i128::from(expiry) - i128::from(accepted_at_us) > 365_i128 * 86_400 * 1_000_000
            {
                return Err(GroupProcessError::Unavailable);
            }
        }
        Ok(Self {
            actor,
            input,
            input_bytes,
            input_digest,
            intake_receipt,
            accepted_at_us,
        })
    }
    pub fn locator(&self) -> GroupProcessLocator {
        GroupProcessLocator::from_command(&self.input)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupProcessTerminalV2 {
    pub accepted: GroupProcessAcceptedV1,
    pub result: ProcessResultV2,
    pub result_bytes: Vec<u8>,
    pub result_digest: [u8; 32],
}
impl GroupProcessTerminalV2 {
    pub fn checked(
        accepted: GroupProcessAcceptedV1,
        result_bytes: Vec<u8>,
        result_digest: [u8; 32],
    ) -> Result<Self, GroupProcessError> {
        let result = ProcessResultV2::decode(&result_bytes)?;
        if result.encode() != result_bytes
            || result.actor() != accepted.actor
            || result.command_id() != accepted.input.command_id()
            || result.group() != accepted.input.group()
            || result.incarnation() != accepted.input.incarnation()
            || result.opcode() != accepted.input.opcode()
            || result.input_digest() != &accepted.input_digest
            || result.intake_receipt() != accepted.intake_receipt
            || result.accepted_at_us() != accepted.accepted_at_us
            || result.requested_process_id() != accepted.input.process_id()
        {
            return Err(GroupProcessError::Unavailable);
        }
        Ok(Self {
            accepted,
            result,
            result_bytes,
            result_digest,
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GroupProcessStatus {
    NotVisible,
    AcceptedPending(GroupProcessAcceptedV1),
    Terminal(GroupProcessTerminalV2),
}
impl GroupProcessStatus {
    pub fn accepted(&self) -> Option<&GroupProcessAcceptedV1> {
        match self {
            Self::NotVisible => None,
            Self::AcceptedPending(input) => Some(input),
            Self::Terminal(result) => Some(&result.accepted),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupProcessAcceptance {
    pub inserted: bool,
    pub status: GroupProcessStatus,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupProcessExecution {
    pub inserted: bool,
    pub terminal: GroupProcessTerminalV2,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GroupProcessRetryResolution {
    Terminal(GroupProcessTerminalV2),
    Pending(GroupProcessAcceptedV1),
    NotVisible,
}

pub struct GroupProcessCurrentProjectionV1 {
    pub mode: GroupProcessMaterialModeV1,
    pub account: GroupProcessAccountProjectionV1,
    pub group: GroupId,
    pub incarnation: GroupIncarnation,
    pub group_revision: u64,
    pub group_state: String,
    pub group_label: String,
    pub group_origin_actor: AccountId,
    pub group_origin_command: Uuid,
    pub group_origin_receipt: Uuid,
    pub designation_actor: AccountId,
    pub designation_state: String,
    pub designation_revision: u64,
    pub designation_receipt: Uuid,
    pub deployment: GroupProcessDeploymentV1,
    pub group_deployment: GroupProcessDeploymentV1,
    pub policy_head: PolicyHeadReferenceV1,
    pub head: Option<ProcessHeadV1>,
    pub version: Option<ProcessVersionV1>,
    pub history: Vec<GroupProcessHistoryViewV1>,
    pub original_status: Option<GroupProcessStatus>,
    pub evaluated_bundle: EvaluatedPolicyBundleV1,
    pub registrations: Vec<GroupProcessRegistrationProjectionV1>,
}
pub struct GroupProcessOwnReceiptProjectionV1 {
    pub mode: GroupProcessMaterialModeV1,
    pub account: GroupProcessAccountProjectionV1,
    pub original: GroupProcessLocator,
    pub status: GroupProcessStatus,
    pub evaluated_bundle: EvaluatedPolicyBundleV1,
    pub registrations: Vec<GroupProcessRegistrationProjectionV1>,
}
pub enum GroupProcessRetainedProjectionV1 {
    Current(GroupProcessCurrentProjectionV1),
    OwnReceipt(GroupProcessOwnReceiptProjectionV1),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CurrentGroupProcessMaterialV1 {
    context: GroupProcessContextV1,
    group_state: String,
    group_origin_actor: AccountId,
    group_origin_command: Uuid,
    group_origin_receipt: Uuid,
    designation_account: AccountId,
    designation_state: String,
    designation_revision: u64,
    designation_receipt: Uuid,
    deployment: GroupProcessDeploymentV1,
    group_deployment: GroupProcessDeploymentV1,
    policy_head: PolicyHeadReferenceV1,
    head: Option<ProcessHeadV1>,
    version: Option<ProcessVersionV1>,
    history: Vec<GroupProcessHistoryViewV1>,
    stage: GroupProcessBootstrapStageV1,
}
impl CurrentGroupProcessMaterialV1 {
    pub const fn designation_account(&self) -> AccountId {
        self.designation_account
    }
    pub fn designation_state(&self) -> &str {
        &self.designation_state
    }
    pub fn group_state(&self) -> &str {
        &self.group_state
    }
    pub const fn designation_revision(&self) -> u64 {
        self.designation_revision
    }
    pub const fn designation_receipt(&self) -> Uuid {
        self.designation_receipt
    }
    pub fn deployment(&self) -> &GroupProcessDeploymentV1 {
        &self.deployment
    }
    pub fn group_deployment(&self) -> &GroupProcessDeploymentV1 {
        &self.group_deployment
    }
    pub fn context(&self) -> &GroupProcessContextV1 {
        &self.context
    }
    pub fn head(&self) -> Option<&ProcessHeadV1> {
        self.head.as_ref()
    }
    pub fn content(&self) -> Option<&ProcessContentV1> {
        self.version.as_ref().map(ProcessVersionV1::content)
    }
    pub fn history(&self) -> &[GroupProcessHistoryViewV1] {
        &self.history
    }
    pub const fn group_origin_actor(&self) -> AccountId {
        self.group_origin_actor
    }
    pub const fn group_origin_command(&self) -> Uuid {
        self.group_origin_command
    }
    pub const fn group_origin_receipt(&self) -> Uuid {
        self.group_origin_receipt
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroupProcessFinishPurposeV1 {
    CurrentForm,
    Accept,
    Execute,
    OwnReceipt,
    TerminalReplay,
    LandingRead,
    OwnReceiptForm,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CurrentGroupProcessAuthority {
    binding: GroupProcessRetainedBindingV1,
    account_state: String,
    registration_receipt: Uuid,
    current_terms_receipt: Uuid,
    group: GroupId,
    incarnation: GroupIncarnation,
    mode: GroupProcessMaterialModeV1,
    finish_purpose: GroupProcessFinishPurposeV1,
    current: Option<CurrentGroupProcessMaterialV1>,
    original: Option<GroupProcessStatus>,
    evaluated_bundle: EvaluatedPolicyBundleV1,
    registrations: Vec<GroupProcessRegistrationV1>,
    request: GroupProcessPolicyRequestV1,
}

impl CurrentGroupProcessAuthority {
    pub fn from_retained_projection(
        binding: GroupProcessRetainedBindingV1,
        request: &GroupProcessScopeRequest<'_>,
        row: GroupProcessRetainedProjectionV1,
    ) -> Result<Self, GroupProcessError> {
        match row {
            GroupProcessRetainedProjectionV1::Current(row) => {
                Self::from_current(binding, request, row)
            }
            GroupProcessRetainedProjectionV1::OwnReceipt(row) => {
                Self::from_own(binding, request, row)
            }
        }
    }

    fn from_current(
        binding: GroupProcessRetainedBindingV1,
        requested: &GroupProcessScopeRequest<'_>,
        row: GroupProcessCurrentProjectionV1,
    ) -> Result<Self, GroupProcessError> {
        Self::from_current_checked(binding, requested, row, None)
    }

    /// Reconstruct final Cedar material only for this retained transaction's
    /// exact provisional B. Initial admission still rejects terminal Execute
    /// material; this value never replaces the original transition authority.
    pub fn execution_postimage(
        &self,
        binding: GroupProcessRetainedBindingV1,
        requested: &GroupProcessScopeRequest<'_>,
        row: GroupProcessCurrentProjectionV1,
        execution: &GroupProcessExecution,
    ) -> Result<Self, GroupProcessError> {
        if self.finish_purpose != GroupProcessFinishPurposeV1::Execute
            || requested.mode() != self.mode
            || requested.locator() != Some(execution.terminal.accepted.locator())
            || !execution.inserted
            || binding.actor != self.binding.actor
            || binding.session != self.binding.session
            || binding.security_generation != self.binding.security_generation
            || binding.xid8 != self.binding.xid8
            || binding.backend_pid != self.binding.backend_pid
            || binding.observed_at_us < self.binding.observed_at_us
            || self.original.as_ref()
                != Some(&GroupProcessStatus::AcceptedPending(
                    execution.terminal.accepted.clone(),
                ))
            || row.original_status.as_ref()
                != Some(&GroupProcessStatus::Terminal(execution.terminal.clone()))
        {
            return Err(GroupProcessError::Unavailable);
        }
        super::workflow::check_execution(self, execution)?;
        let result = &execution.terminal.result;
        if result.executed_at_us() > binding.observed_at_us
            || result.policy_after() != row.policy_head
            || result.after_head() != row.head.as_ref().map(|head| &head.reference)
        {
            return Err(GroupProcessError::Unavailable);
        }
        let before = self
            .current
            .as_ref()
            .ok_or(GroupProcessError::Unavailable)?;
        let after = Self::from_current_checked(binding, requested, row, Some(execution))?;
        let current = after
            .current
            .as_ref()
            .ok_or(GroupProcessError::Unavailable)?;
        if after.account_state != self.account_state
            || after.registration_receipt != self.registration_receipt
            || after.current_terms_receipt != self.current_terms_receipt
            || after.evaluated_bundle != self.evaluated_bundle
            || after.registrations != self.registrations
            || current.context != before.context
            || current.group_state != before.group_state
            || current.group_origin_actor != before.group_origin_actor
            || current.group_origin_command != before.group_origin_command
            || current.group_origin_receipt != before.group_origin_receipt
            || current.designation_account != before.designation_account
            || current.designation_state != before.designation_state
            || current.designation_revision != before.designation_revision
            || current.designation_receipt != before.designation_receipt
            || current.deployment != before.deployment
            || current.group_deployment != before.group_deployment
        {
            return Err(GroupProcessError::Unavailable);
        }
        let committed = matches!(
            result.terminal_code(),
            ProcessTerminalCodeV1::Adopted
                | ProcessTerminalCodeV1::Replaced
                | ProcessTerminalCodeV1::Suspended
        );
        if committed {
            let head = current
                .head
                .as_ref()
                .ok_or(GroupProcessError::Unavailable)?;
            if head.causing_receipt != result.result_receipt()
                || head.reference.expiry_us() <= binding.observed_at_us
                || current.history.len() != before.history.len() + 1
                || !current.history.starts_with(&before.history)
            {
                return Err(GroupProcessError::Unavailable);
            }
            let last = current
                .history
                .last()
                .ok_or(GroupProcessError::Unavailable)?;
            if last.actor != binding.actor
                || last.command_id != result.command_id()
                || last.occurred_at_us != result.executed_at_us()
            {
                return Err(GroupProcessError::Unavailable);
            }
            let accepted = &execution.terminal.accepted;
            if let Some(adopt) = accepted.input.adoption() {
                let version = current
                    .version
                    .as_ref()
                    .ok_or(GroupProcessError::Unavailable)?;
                // A genuine source digest identifies bytes; these comparisons
                // establish that those bytes are the accepted command's effect.
                if version.actor() != accepted.actor
                    || version.designation_receipt() != current.designation_receipt
                    || version.designation_revision() != current.designation_revision
                    || version.adopt_command() != accepted.input.command_id()
                    || version.input_digest() != &accepted.input_digest
                    || version.admitted_at_us() != accepted.accepted_at_us
                    || version.expiry_us() != exact_time_us(adopt.expiry())?
                    || version.responsibility() != adopt.responsibility()
                    || version.content() != adopt.content()
                    || last.reason.is_some()
                {
                    return Err(GroupProcessError::Unavailable);
                }
            } else if let Some(suspend) = accepted.input.suspension() {
                if current.version != before.version
                    || last.reason.as_deref() != Some(suspend.reason())
                {
                    return Err(GroupProcessError::Unavailable);
                }
            } else {
                return Err(GroupProcessError::Unavailable);
            }
        } else if current.policy_head != before.policy_head
            || current.head != before.head
            || current.version != before.version
            || current.history != before.history
        {
            return Err(GroupProcessError::Unavailable);
        }
        Ok(after)
    }

    fn from_current_checked(
        binding: GroupProcessRetainedBindingV1,
        requested: &GroupProcessScopeRequest<'_>,
        row: GroupProcessCurrentProjectionV1,
        execution: Option<&GroupProcessExecution>,
    ) -> Result<Self, GroupProcessError> {
        row.account.check(binding)?;
        if row.mode != requested.mode() {
            return Err(GroupProcessError::Unavailable);
        }
        if row.group != requested.group() || row.incarnation != requested.incarnation() {
            return Err(GroupProcessError::NotFound);
        }
        revision(row.group_revision, false)?;
        revision(row.designation_revision, false)?;
        for id in [
            *row.group_origin_actor.as_uuid(),
            row.group_origin_command,
            row.group_origin_receipt,
            *row.designation_actor.as_uuid(),
            row.designation_receipt,
        ] {
            nonnil(id)?;
        }
        text(&row.group_label, 256)?;
        if row.deployment != row.group_deployment {
            return Err(GroupProcessError::Unavailable);
        }
        if row.group_state != "ACTIVE"
            || row.designation_state != "ACTIVE"
            || row.designation_actor != binding.actor
        {
            return Err(GroupProcessError::NotFound);
        }
        let registrations = GroupProcessRegistrationV1::checked(row.registrations)?;
        check_group_process_projection_v1(
            row.group,
            row.incarnation,
            row.policy_head,
            row.head.as_ref(),
            row.version.as_ref(),
            &row.history,
        )?;
        if let Some(status) = &row.original_status {
            check_status_closure(status)?;
        }
        let original = row
            .original_status
            .as_ref()
            .and_then(GroupProcessStatus::accepted);
        if let Some(accepted) = original {
            if accepted.actor != binding.actor
                || accepted.input.group() != row.group
                || accepted.input.incarnation() != row.incarnation
                || requested
                    .locator()
                    .is_none_or(|locator| locator != accepted.locator())
            {
                return Err(GroupProcessError::Unavailable);
            }
        }
        let mode = requested.mode();
        let (action, kind, finish) = match requested {
            GroupProcessScopeRequest::Current { .. } => (
                GroupProcessActionV1::Read,
                GroupProcessResourceKindV1::CurrentProcess,
                GroupProcessFinishPurposeV1::CurrentForm,
            ),
            GroupProcessScopeRequest::Form { .. }
            | GroupProcessScopeRequest::ValidationForm { .. } => (
                GroupProcessActionV1::Read,
                GroupProcessResourceKindV1::ProcessForm,
                GroupProcessFinishPurposeV1::CurrentForm,
            ),
            GroupProcessScopeRequest::Landing { .. } => (
                GroupProcessActionV1::Read,
                if row.policy_head == PolicyHeadReferenceV1::Absent {
                    GroupProcessResourceKindV1::ProcessLanding
                } else {
                    GroupProcessResourceKindV1::CurrentProcess
                },
                GroupProcessFinishPurposeV1::LandingRead,
            ),
            GroupProcessScopeRequest::Accept(input) => {
                if let Some(accepted) = original {
                    if &accepted.input != *input {
                        return Err(GroupProcessError::Conflict);
                    }
                }
                (
                    command_action(input),
                    GroupProcessResourceKindV1::ProcessMutation,
                    GroupProcessFinishPurposeV1::Accept,
                )
            }
            GroupProcessScopeRequest::Execute(_) | GroupProcessScopeRequest::RetryResolve(_) => {
                let accepted = original.ok_or(GroupProcessError::NotFound)?;
                if !matches!(
                    row.original_status,
                    Some(GroupProcessStatus::AcceptedPending(_))
                ) && !execution.is_some_and(|value| {
                    row.original_status.as_ref()
                        == Some(&GroupProcessStatus::Terminal(value.terminal.clone()))
                        && value.terminal.result.effect_xid8() == binding.xid8
                        && value.terminal.result.effect_pid() == binding.backend_pid
                }) {
                    return Err(GroupProcessError::Unavailable);
                }
                (
                    command_action(&accepted.input),
                    GroupProcessResourceKindV1::ProcessMutation,
                    GroupProcessFinishPurposeV1::Execute,
                )
            }
            _ => return Err(GroupProcessError::Unavailable),
        };
        let stage = if row.policy_head == PolicyHeadReferenceV1::Absent {
            match requested {
                GroupProcessScopeRequest::Form { .. }
                | GroupProcessScopeRequest::ValidationForm { .. } => {
                    GroupProcessBootstrapStageV1::EmptyForm
                }
                GroupProcessScopeRequest::Landing { .. } => {
                    GroupProcessBootstrapStageV1::EmptyLanding
                }
                GroupProcessScopeRequest::Accept(input) if first_adopt(input) => {
                    GroupProcessBootstrapStageV1::FirstIntake
                }
                GroupProcessScopeRequest::Execute(_)
                | GroupProcessScopeRequest::RetryResolve(_)
                    if original.is_some_and(|accepted| first_adopt(&accepted.input)) =>
                {
                    GroupProcessBootstrapStageV1::FirstExecution
                }
                _ => return Err(GroupProcessError::NotFound),
            }
        } else {
            GroupProcessBootstrapStageV1::None
        };
        let fields = if matches!(
            stage,
            GroupProcessBootstrapStageV1::EmptyForm | GroupProcessBootstrapStageV1::EmptyLanding
        ) {
            EMPTY_FIELDS
        } else {
            group_process_registered_fields(action)
        };
        let policy_request = GroupProcessPolicyRequestV1::new(action, kind, fields.to_vec())?;
        Ok(Self {
            binding,
            account_state: row.account.state,
            registration_receipt: row.account.registration_receipt,
            current_terms_receipt: row.account.current_terms_receipt,
            group: row.group,
            incarnation: row.incarnation,
            mode,
            finish_purpose: finish,
            current: Some(CurrentGroupProcessMaterialV1 {
                context: GroupProcessContextV1 {
                    group: row.group,
                    incarnation: row.incarnation,
                    label: row.group_label,
                    revision: row.group_revision,
                    account: binding.actor,
                },
                group_state: row.group_state,
                group_origin_actor: row.group_origin_actor,
                group_origin_command: row.group_origin_command,
                group_origin_receipt: row.group_origin_receipt,
                designation_account: row.designation_actor,
                designation_state: row.designation_state,
                designation_revision: row.designation_revision,
                designation_receipt: row.designation_receipt,
                deployment: row.deployment,
                group_deployment: row.group_deployment,
                policy_head: row.policy_head,
                head: row.head,
                version: row.version,
                history: row.history,
                stage,
            }),
            original: row.original_status,
            evaluated_bundle: row.evaluated_bundle,
            registrations,
            request: policy_request,
        })
    }

    fn from_own(
        binding: GroupProcessRetainedBindingV1,
        requested: &GroupProcessScopeRequest<'_>,
        row: GroupProcessOwnReceiptProjectionV1,
    ) -> Result<Self, GroupProcessError> {
        row.account.check(binding)?;
        if row.mode != requested.mode() {
            return Err(GroupProcessError::Unavailable);
        }
        check_status_closure(&row.status)?;
        if requested.locator() != Some(row.original) {
            return Err(GroupProcessError::NotFound);
        }
        let accepted = row.status.accepted().ok_or(GroupProcessError::NotFound)?;
        if accepted.actor != binding.actor || accepted.locator() != row.original {
            return Err(GroupProcessError::NotFound);
        }
        let (kind, finish) = match requested {
            GroupProcessScopeRequest::Status(_) => (
                GroupProcessResourceKindV1::OwnReceipt,
                GroupProcessFinishPurposeV1::OwnReceipt,
            ),
            GroupProcessScopeRequest::OwnRetryForm(_) => (
                GroupProcessResourceKindV1::OwnReceiptRetryForm,
                GroupProcessFinishPurposeV1::OwnReceiptForm,
            ),
            GroupProcessScopeRequest::RetryResolve(_)
                if matches!(row.status, GroupProcessStatus::Terminal(_)) =>
            {
                (
                    GroupProcessResourceKindV1::OwnReceipt,
                    GroupProcessFinishPurposeV1::TerminalReplay,
                )
            }
            _ => return Err(GroupProcessError::Unavailable),
        };
        let fields = if kind == GroupProcessResourceKindV1::OwnReceiptRetryForm {
            RETRY_FORM_FIELDS
        } else {
            OWN_FIELDS
        };
        let registrations = GroupProcessRegistrationV1::checked(row.registrations)?;
        Ok(Self {
            binding,
            account_state: row.account.state,
            registration_receipt: row.account.registration_receipt,
            current_terms_receipt: row.account.current_terms_receipt,
            group: row.original.group,
            incarnation: row.original.incarnation,
            mode: requested.mode(),
            finish_purpose: finish,
            current: None,
            original: Some(row.status),
            evaluated_bundle: row.evaluated_bundle,
            registrations,
            request: GroupProcessPolicyRequestV1::new(
                GroupProcessActionV1::ReadOwnReceipt,
                kind,
                fields.to_vec(),
            )?,
        })
    }
    pub const fn group(&self) -> GroupId {
        self.group
    }
    pub const fn incarnation(&self) -> GroupIncarnation {
        self.incarnation
    }
    pub const fn account(&self) -> AccountId {
        self.binding.actor
    }
    pub fn account_state(&self) -> &str {
        &self.account_state
    }
    pub const fn binding(&self) -> GroupProcessRetainedBindingV1 {
        self.binding
    }
    pub const fn registration_receipt(&self) -> Uuid {
        self.registration_receipt
    }
    pub const fn current_terms_receipt(&self) -> Uuid {
        self.current_terms_receipt
    }
    pub const fn mode(&self) -> GroupProcessMaterialModeV1 {
        self.mode
    }
    pub const fn finish_purpose(&self) -> GroupProcessFinishPurposeV1 {
        self.finish_purpose
    }
    pub fn current_material(&self) -> Option<&CurrentGroupProcessMaterialV1> {
        self.current.as_ref()
    }
    pub fn receipt_actor(&self) -> Option<AccountId> {
        self.original
            .as_ref()
            .and_then(GroupProcessStatus::accepted)
            .map(|input| input.actor)
    }
    pub fn policy_head(&self) -> Option<PolicyHeadReferenceV1> {
        self.current.as_ref().map(|material| material.policy_head)
    }
    pub fn bootstrap_stage(&self) -> Option<GroupProcessBootstrapStageV1> {
        self.current.as_ref().map(|material| material.stage)
    }
    pub fn evaluated_bundle(&self) -> &EvaluatedPolicyBundleV1 {
        &self.evaluated_bundle
    }
    pub const fn action(&self) -> GroupProcessActionV1 {
        self.request.action
    }
    pub const fn resource_kind(&self) -> GroupProcessResourceKindV1 {
        self.request.resource_kind
    }
    pub fn policy_request(&self) -> &GroupProcessPolicyRequestV1 {
        &self.request
    }
    pub fn registrations(&self) -> &[GroupProcessRegistrationV1] {
        &self.registrations
    }
    pub fn allowed_fields(&self, action: GroupProcessActionV1) -> &[Field] {
        self.registrations
            .iter()
            .find(|row| row.action == action)
            .map_or(&[], |row| row.fields.as_slice())
    }
    pub fn original_status(&self) -> Option<&GroupProcessStatus> {
        self.original.as_ref()
    }
    pub fn require_method(
        &self,
        mode: GroupProcessMaterialModeV1,
    ) -> Result<(), GroupProcessError> {
        if self.mode == mode {
            Ok(())
        } else {
            Err(GroupProcessError::Unavailable)
        }
    }
    pub fn check_finish_proof(&self, proof_present: bool) -> Result<(), GroupProcessError> {
        if proof_present
            == matches!(
                self.mode,
                GroupProcessMaterialModeV1::Form | GroupProcessMaterialModeV1::OwnRetryForm
            )
        {
            Ok(())
        } else {
            Err(GroupProcessError::Unavailable)
        }
    }
    pub fn authorize<P: GroupProcessDecisionPort + ?Sized>(
        &self,
        policy: &P,
    ) -> Result<(), GroupProcessError> {
        match policy.decide(self, &self.request)? {
            GroupProcessDecision::Allow => Ok(()),
            GroupProcessDecision::Deny => Err(GroupProcessError::NotFound),
        }
    }
    pub fn current_view(&self) -> Result<GroupProcessCurrentView, GroupProcessError> {
        let current = self
            .current
            .as_ref()
            .ok_or(GroupProcessError::Unavailable)?;
        if !matches!(
            self.mode,
            GroupProcessMaterialModeV1::Current
                | GroupProcessMaterialModeV1::Form
                | GroupProcessMaterialModeV1::Landing
        ) {
            return Err(GroupProcessError::Unavailable);
        }
        Ok(GroupProcessCurrentView {
            context: current.context.clone(),
            observed_at_us: self.binding.observed_at_us,
            policy_head: current.policy_head,
            head: current.head.clone(),
            content: current.content().cloned(),
            history: current.history.clone(),
            // These are destinations within the adopted procedure owner, never
            // grants to the verifier, Company, employment or payroll owners.
            allowed_actions: if current.head.as_ref().is_some_and(|head| {
                head.reference.state() == ProcessStateV1::Active
                    && head.reference.expiry_us() > self.binding.observed_at_us
            }) {
                vec![GroupProcessActionV1::Adopt, GroupProcessActionV1::Suspend]
            } else {
                vec![GroupProcessActionV1::Adopt]
            },
        })
    }
}

fn first_adopt(input: &GroupProcessCommandV1) -> bool {
    input.expected_policy_revision() == 0
        && input
            .adoption()
            .is_some_and(|adopt| adopt.expected_prior_head_revision() == 0)
}
fn command_action(input: &GroupProcessCommandV1) -> GroupProcessActionV1 {
    if input.adoption().is_some() {
        GroupProcessActionV1::Adopt
    } else {
        GroupProcessActionV1::Suspend
    }
}
/// Pure source closure validation, including denied navigation candidates.
/// This checks references and history, and does not grant resource authority.
pub fn check_group_process_projection_v1(
    group: GroupId,
    incarnation: GroupIncarnation,
    policy: PolicyHeadReferenceV1,
    head: Option<&ProcessHeadV1>,
    version: Option<&ProcessVersionV1>,
    history: &[GroupProcessHistoryViewV1],
) -> Result<(), GroupProcessError> {
    if policy == PolicyHeadReferenceV1::Absent {
        if head.is_some() || version.is_some() || !history.is_empty() {
            return Err(GroupProcessError::Unavailable);
        }
        return Ok(());
    }
    if policy.revision() != Some(1) {
        return Err(GroupProcessError::Unavailable);
    }
    let head = head.ok_or(GroupProcessError::Unavailable)?;
    let version = version.ok_or(GroupProcessError::Unavailable)?;
    if version.group() != group
        || version.incarnation() != incarnation
        || version.process_id() != head.reference.process_id()
        || version.version() != head.reference.content_version()
        || version.content_digest() != head.reference.content_digest()
        || version.expiry_us() != head.reference.expiry_us()
        || !matches!(policy, PolicyHeadReferenceV1::Installed { revision, head_digest } if revision == version.policy_revision() && &head_digest == version.policy_head_digest())
        || history.is_empty()
        || history.len() as u64 != head.reference.head_revision()
    {
        return Err(GroupProcessError::Unavailable);
    }
    nonnil(head.causing_receipt)?;
    for (index, entry) in history.iter().enumerate() {
        nonnil(entry.head.causing_receipt)?;
        nonnil(*entry.actor.as_uuid())?;
        nonnil(entry.command_id)?;
        time_from_us(entry.occurred_at_us)?;
        if entry.head.reference.head_revision() != index as u64 + 1
            || entry.head.reference.process_id() != head.reference.process_id()
        {
            return Err(GroupProcessError::Unavailable);
        }
        if let Some(reason) = &entry.reason {
            text(reason, 2048)?;
        }
        if index == 0 {
            if entry.head.reference.content_version() != 1
                || entry.head.reference.state() != ProcessStateV1::Active
                || entry.reason.is_some()
            {
                return Err(GroupProcessError::Unavailable);
            }
        } else {
            let previous = &history[index - 1];
            if entry.occurred_at_us < previous.occurred_at_us {
                return Err(GroupProcessError::Unavailable);
            }
            let before = &previous.head.reference;
            let after = &entry.head.reference;
            if after.content_version() == before.content_version() {
                if before.state() != ProcessStateV1::Active
                    || after.state() != ProcessStateV1::Suspended
                    || after.content_digest() != before.content_digest()
                    || after.expiry_us() != before.expiry_us()
                    || entry.reason.is_none()
                {
                    return Err(GroupProcessError::Unavailable);
                }
            } else if before.content_version().checked_add(1) != Some(after.content_version())
                || after.state() != ProcessStateV1::Active
                || entry.reason.is_some()
            {
                return Err(GroupProcessError::Unavailable);
            }
        }
    }
    if history.last().is_none_or(|entry| &entry.head != head) {
        return Err(GroupProcessError::Unavailable);
    }
    Ok(())
}

fn check_status_closure(status: &GroupProcessStatus) -> Result<(), GroupProcessError> {
    let Some(accepted) = status.accepted() else {
        return Ok(());
    };
    let original = GroupProcessAcceptedV1::checked(
        accepted.actor,
        accepted.input_bytes.clone(),
        accepted.input_digest,
        accepted.intake_receipt,
        accepted.accepted_at_us,
    )?;
    if &original != accepted {
        return Err(GroupProcessError::Unavailable);
    }
    if let GroupProcessStatus::Terminal(terminal) = status {
        let original_result = GroupProcessTerminalV2::checked(
            original,
            terminal.result_bytes.clone(),
            terminal.result_digest,
        )?;
        if &original_result != terminal {
            return Err(GroupProcessError::Unavailable);
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroupProcessTransitionPlanV1 {
    FirstAdopt {
        content_version: u64,
        head_revision: u64,
    },
    Replace {
        content_version: u64,
        head_revision: u64,
    },
    Suspend {
        content_version: u64,
        head_revision: u64,
    },
    Refuse(ProcessTerminalCodeV1),
}

/// Business classification uses original accepted input and actual retained
/// source. Source/incarnation/Auth loss never becomes a terminal refusal.
pub fn classify_group_process_transition_v1(
    authority: &CurrentGroupProcessAuthority,
) -> Result<GroupProcessTransitionPlanV1, GroupProcessError> {
    classify_group_process_transition_at_v1(authority, authority.binding.observed_at_us)
}

/// `observed_at_us` comes from the adapter's fresh actual database observation
/// immediately before effects. Client clocks never enter the owning scope.
pub fn classify_group_process_transition_at_v1(
    authority: &CurrentGroupProcessAuthority,
    observed_at_us: i64,
) -> Result<GroupProcessTransitionPlanV1, GroupProcessError> {
    time_from_us(observed_at_us)?;
    if observed_at_us < authority.binding.observed_at_us
        || i128::from(observed_at_us) - i128::from(authority.binding.observed_at_us) > 10_000_000
    {
        return Err(GroupProcessError::Unavailable);
    }
    if authority.finish_purpose != GroupProcessFinishPurposeV1::Execute {
        return Err(GroupProcessError::Unavailable);
    }
    let current = authority
        .current
        .as_ref()
        .ok_or(GroupProcessError::Unavailable)?;
    let GroupProcessStatus::AcceptedPending(accepted) = authority
        .original
        .as_ref()
        .ok_or(GroupProcessError::Unavailable)?
    else {
        return Err(GroupProcessError::Unavailable);
    };
    let input = &accepted.input;
    if input.group() != authority.group || input.incarnation() != authority.incarnation {
        return Err(GroupProcessError::Unavailable);
    }
    let expected_policy = input.expected_policy_revision();
    let actual_policy = current.policy_head.revision().unwrap_or(0);
    let stale_head = match (input.adoption(), input.suspension(), current.head.as_ref()) {
        (Some(adopt), None, None) => adopt.expected_prior_head_revision() != 0,
        (Some(adopt), None, Some(head)) => {
            adopt.process_id() != head.reference.process_id()
                || adopt.expected_prior_head_revision() != head.reference.head_revision()
        }
        (None, Some(_), None) => true,
        (None, Some(suspend), Some(head)) => {
            suspend.process_id() != head.reference.process_id()
                || suspend.content_version() != head.reference.content_version()
                || suspend.content_digest() != head.reference.content_digest()
                || suspend.expected_head_revision() != head.reference.head_revision()
                || suspend.expected_head_digest() != head.reference.head_digest()
        }
        _ => return Err(GroupProcessError::Unavailable),
    };
    if input.expected_group_revision() != current.context.revision
        || expected_policy != actual_policy
        || stale_head
    {
        return Ok(GroupProcessTransitionPlanV1::Refuse(
            ProcessTerminalCodeV1::RejectedStaleExpectation,
        ));
    }
    let observed_at = observed_at_us;
    if let Some(adopt) = input.adoption() {
        if exact_time_us(adopt.expiry())? <= observed_at {
            return Ok(GroupProcessTransitionPlanV1::Refuse(
                ProcessTerminalCodeV1::RejectedProcessExpired,
            ));
        }
        if let Some(head) = &current.head {
            let content_version = revision(
                head.reference
                    .content_version()
                    .checked_add(1)
                    .ok_or(GroupProcessError::Unavailable)?,
                false,
            )?;
            let head_revision = revision(
                head.reference
                    .head_revision()
                    .checked_add(1)
                    .ok_or(GroupProcessError::Unavailable)?,
                false,
            )?;
            Ok(GroupProcessTransitionPlanV1::Replace {
                content_version,
                head_revision,
            })
        } else {
            if authority.bootstrap_stage() != Some(GroupProcessBootstrapStageV1::FirstExecution) {
                return Err(GroupProcessError::Unavailable);
            }
            Ok(GroupProcessTransitionPlanV1::FirstAdopt {
                content_version: 1,
                head_revision: 1,
            })
        }
    } else {
        let head = current
            .head
            .as_ref()
            .ok_or(GroupProcessError::Unavailable)?;
        if head.reference.state() == ProcessStateV1::Suspended {
            return Ok(GroupProcessTransitionPlanV1::Refuse(
                ProcessTerminalCodeV1::RejectedAlreadySuspended,
            ));
        }
        if head.reference.expiry_us() <= observed_at {
            return Ok(GroupProcessTransitionPlanV1::Refuse(
                ProcessTerminalCodeV1::RejectedProcessExpired,
            ));
        }
        Ok(GroupProcessTransitionPlanV1::Suspend {
            content_version: head.reference.content_version(),
            head_revision: revision(
                head.reference
                    .head_revision()
                    .checked_add(1)
                    .ok_or(GroupProcessError::Unavailable)?,
                false,
            )?,
        })
    }
}

/// Method results are provisional. The store retains a real transaction, checks
/// its private purpose, then reconfirms current Auth/source/policy/effects/time
/// and commit. Dropping it rolls back; no caller releases a result early.
pub trait GroupProcessScope {
    type FormProof: Send + Sync;
    fn authority(&self) -> &CurrentGroupProcessAuthority;
    fn current(
        &mut self,
    ) -> impl Future<Output = Result<GroupProcessCurrentView, GroupProcessError>> + Send;
    fn form(
        &mut self,
    ) -> impl Future<Output = Result<GroupProcessForm<Self::FormProof>, GroupProcessError>> + Send;
    fn accept(
        &mut self,
        trace: &TraceContext,
    ) -> impl Future<Output = Result<GroupProcessAcceptance, GroupProcessError>> + Send;
    fn execute(
        &mut self,
        trace: &TraceContext,
    ) -> impl Future<Output = Result<GroupProcessExecution, GroupProcessError>> + Send;
    fn status(
        &mut self,
    ) -> impl Future<Output = Result<GroupProcessStatus, GroupProcessError>> + Send;
    fn retry_resolve(
        &mut self,
    ) -> impl Future<Output = Result<GroupProcessRetryResolution, GroupProcessError>> + Send;
    fn landing(
        &mut self,
    ) -> impl Future<Output = Result<GroupProcessLandingViewV1, GroupProcessError>> + Send;
    fn retry_form(
        &mut self,
    ) -> impl Future<Output = Result<GroupProcessOwnRetryFormV1<Self::FormProof>, GroupProcessError>>
    + Send;
    fn finish<P: GroupProcessDecisionPort + ?Sized>(
        self,
        policy: &P,
        proof: Option<&Self::FormProof>,
    ) -> impl Future<Output = Result<(), GroupProcessError>> + Send;
}

pub trait GroupProcessStore: Sync {
    type Credentials: Sync;
    type FormProof: Send + Sync;
    type Scope<'a>: GroupProcessScope<FormProof = Self::FormProof> + Send
    where
        Self: 'a;
    /// Namespace/locator preplanning returns selectors only. Current source and
    /// actor-bound historical identity are rechecked under fixed retained locks.
    fn resolve_current_incarnation(
        &self,
        credentials: &Self::Credentials,
        group: GroupId,
    ) -> impl Future<Output = Result<GroupIncarnation, GroupProcessError>> + Send;
    fn resolve_original_locator(
        &self,
        credentials: &Self::Credentials,
        requested: GroupProcessRouteSelectorV1,
    ) -> impl Future<Output = Result<GroupProcessLocator, GroupProcessError>> + Send;
    fn lock<'a>(
        &'a self,
        credentials: &'a Self::Credentials,
        request: GroupProcessScopeRequest<'a>,
    ) -> impl Future<Output = Result<Self::Scope<'a>, GroupProcessError>> + Send;
    fn enumerate_navigation_candidates(
        &self,
        credentials: &Self::Credentials,
    ) -> impl Future<Output = Result<GroupProcessNavigationCandidatesV1, GroupProcessError>> + Send;
    /// Preplan sorted Group guards before Account/family, compare the complete
    /// original observation including denied candidates and source identities.
    fn recheck_navigation_candidates(
        &self,
        credentials: &Self::Credentials,
        original: &GroupProcessNavigationSnapshotV1,
    ) -> impl Future<Output = Result<(), GroupProcessError>> + Send;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct GroupProcessNavigationCandidateV1 {
    group: GroupId,
    incarnation: GroupIncarnation,
}
impl GroupProcessNavigationCandidateV1 {
    pub const fn new(group: GroupId, incarnation: GroupIncarnation) -> Self {
        Self { group, incarnation }
    }
    pub const fn group(self) -> GroupId {
        self.group
    }
    pub const fn incarnation(self) -> GroupIncarnation {
        self.incarnation
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupProcessNavigationSnapshotV1 {
    source_bytes: Vec<u8>,
}
impl GroupProcessNavigationSnapshotV1 {
    /// Opaque bytes are the complete deterministic protected source snapshot,
    /// not a caller's generation, a hash-only substitute or authority token.
    pub fn from_protected_source_bytes(source_bytes: Vec<u8>) -> Result<Self, GroupProcessError> {
        if source_bytes.is_empty() || source_bytes.len() > 1_048_576 {
            return Err(GroupProcessError::Unavailable);
        }
        Ok(Self { source_bytes })
    }
    pub fn source_bytes(&self) -> &[u8] {
        &self.source_bytes
    }
}
pub struct GroupProcessNavigationCandidatesV1 {
    candidates: Vec<GroupProcessNavigationCandidateV1>,
    snapshot: GroupProcessNavigationSnapshotV1,
}
impl GroupProcessNavigationCandidatesV1 {
    /// Inspect the 257th source row rather than truncating. Hints confer no
    /// resource authority and must never be serialized to a product surface.
    pub fn from_projection(
        candidates: Vec<GroupProcessNavigationCandidateV1>,
        snapshot: GroupProcessNavigationSnapshotV1,
    ) -> Result<Self, GroupProcessError> {
        if candidates.len() > 256
            || candidates
                .windows(2)
                .any(|pair| pair[0].group() >= pair[1].group())
        {
            return Err(GroupProcessError::Unavailable);
        }
        Ok(Self {
            candidates,
            snapshot,
        })
    }
    pub fn candidates(&self) -> &[GroupProcessNavigationCandidateV1] {
        &self.candidates
    }
    pub fn snapshot(&self) -> &GroupProcessNavigationSnapshotV1 {
        &self.snapshot
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupProcessNavigationViewV1 {
    pub group: GroupId,
    pub incarnation: GroupIncarnation,
    pub label: String,
    pub revision: u64,
}
