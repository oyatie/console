//! Finite Company-information input selectors, never current permission proof.
//! Codec 4 remains inactive until its Company owner is separately admitted.
use super::{
    AccountId, ActionRef, PropertyRef,
    business::{invalid, validate_revision},
};
use console_kernel_core::{KernelError, OrgId};
use time::{OffsetDateTime, UtcOffset};
use uuid::Uuid;

mod codec;

const PREFIX: &[u8] = b"console.company.information-policy\0\0\x04";
pub const MANIFEST: [u8; 32] = [
    0x0d, 0x3d, 0x0c, 0x3b, 0xc0, 0x35, 0x7c, 0x03, 0x94, 0xb0, 0x24, 0x00, 0x29, 0x5f, 0x77, 0x23,
    0x11, 0x78, 0xcd, 0x5d, 0xc2, 0x2a, 0x66, 0x8a, 0x90, 0x88, 0x0f, 0xc9, 0x2a, 0x08, 0x99, 0x35,
];

/// An exact Company-local parent or child selector, with no inferred role revision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompanyInformationAssignmentRefV1 {
    assignment_id: Uuid,
    revision: u64,
}

impl CompanyInformationAssignmentRefV1 {
    pub fn new(assignment_id: Uuid, revision: u64) -> Result<Self, KernelError> {
        if assignment_id.is_nil() {
            return Err(invalid());
        }
        validate_revision(revision)?;
        Ok(Self {
            assignment_id,
            revision,
        })
    }
    pub const fn assignment_id(&self) -> Uuid {
        self.assignment_id
    }
    pub const fn revision(&self) -> u64 {
        self.revision
    }
}

/// Exactly two nondelegable clauses: Discover then ReadIdentity. Action and field
/// IDs must still be compared with the owner's actual registered source before use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeCompanyInformationGrantV1 {
    recipient: AccountId,
    parent: CompanyInformationAssignmentRefV1,
    valid_from: OffsetDateTime,
    valid_until: OffsetDateTime,
    actions: [ActionRef; 2],
    fields: [PropertyRef; 2],
}

impl NativeCompanyInformationGrantV1 {
    pub fn new(
        recipient: AccountId,
        parent: CompanyInformationAssignmentRefV1,
        valid_from: OffsetDateTime,
        valid_until: OffsetDateTime,
        actions: [ActionRef; 2],
        fields: [PropertyRef; 2],
    ) -> Result<Self, KernelError> {
        validate_time(valid_from)?;
        validate_time(valid_until)?;
        let duration = valid_until.unix_timestamp_nanos() - valid_from.unix_timestamp_nanos();
        if duration <= 0 || duration > 2_592_000_000_000_000 {
            return Err(invalid());
        }
        let company = actions[0].org_id();
        let object_type = actions[0].object_type_id();
        if actions[1].org_id() != company
            || actions[1].object_type_id() != object_type
            || actions[0].action_type_id() == actions[1].action_type_id()
            || actions.iter().any(|a| a.manifest_digest() != &MANIFEST)
            || fields[0] >= fields[1]
            || fields
                .iter()
                .any(|f| f.org_id() != company || f.object_type_id() != object_type)
        {
            return Err(invalid());
        }
        Ok(Self {
            recipient,
            parent,
            valid_from: valid_from.to_offset(UtcOffset::UTC),
            valid_until: valid_until.to_offset(UtcOffset::UTC),
            actions,
            fields,
        })
    }
    pub const fn recipient(&self) -> AccountId {
        self.recipient
    }
    pub const fn parent(&self) -> CompanyInformationAssignmentRefV1 {
        self.parent
    }
    pub const fn valid_from(&self) -> OffsetDateTime {
        self.valid_from
    }
    pub const fn valid_until(&self) -> OffsetDateTime {
        self.valid_until
    }
    /// Position zero is Discover; position one is ReadIdentity.
    pub fn actions(&self) -> &[ActionRef; 2] {
        &self.actions
    }
    /// The owner must resolve this sorted pair to exactly Company name and slug.
    pub fn fields(&self) -> &[PropertyRef; 2] {
        &self.fields
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Input {
    Grant(Box<NativeCompanyInformationGrantV1>),
    Revoke(CompanyInformationAssignmentRefV1),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeCompanyInformationCommandV1 {
    command_id: Uuid,
    company: OrgId,
    expected_company_epoch: u64,
    reason: String,
    input: Input,
}

impl NativeCompanyInformationCommandV1 {
    pub fn grant(
        command_id: Uuid,
        company: OrgId,
        expected_company_epoch: u64,
        reason: String,
        grant: NativeCompanyInformationGrantV1,
    ) -> Result<Self, KernelError> {
        if grant.actions[0].org_id() != company {
            return Err(invalid());
        }
        Self::new(
            command_id,
            company,
            expected_company_epoch,
            reason,
            Input::Grant(Box::new(grant)),
        )
    }
    pub fn revoke(
        command_id: Uuid,
        company: OrgId,
        expected_company_epoch: u64,
        reason: String,
        assignment: CompanyInformationAssignmentRefV1,
    ) -> Result<Self, KernelError> {
        Self::new(
            command_id,
            company,
            expected_company_epoch,
            reason,
            Input::Revoke(assignment),
        )
    }
    fn new(
        command_id: Uuid,
        company: OrgId,
        expected_company_epoch: u64,
        reason: String,
        input: Input,
    ) -> Result<Self, KernelError> {
        if command_id.is_nil()
            || company.as_uuid().is_nil()
            || company == OrgId::platform()
            || reason.is_empty()
            || reason.len() > 2048
            || reason.chars().count() > 512
            || reason.trim().is_empty()
            || reason.chars().any(char::is_control)
        {
            return Err(invalid());
        }
        validate_revision(expected_company_epoch)?;
        Ok(Self {
            command_id,
            company,
            expected_company_epoch,
            reason,
            input,
        })
    }
    pub const fn command_id(&self) -> Uuid {
        self.command_id
    }
    pub const fn company(&self) -> OrgId {
        self.company
    }
    pub const fn expected_company_epoch(&self) -> u64 {
        self.expected_company_epoch
    }
    pub fn reason(&self) -> &str {
        &self.reason
    }
    pub const fn operation(&self) -> super::business::NativeBusinessOperationV1 {
        match self.input {
            Input::Grant(_) => super::business::NativeBusinessOperationV1::Grant,
            Input::Revoke(_) => super::business::NativeBusinessOperationV1::Revoke,
        }
    }
    pub const fn recipient_account_id(&self) -> Option<AccountId> {
        match &self.input {
            Input::Grant(grant) => Some(grant.recipient),
            Input::Revoke(_) => None,
        }
    }
    pub const fn expires_at(&self) -> Option<OffsetDateTime> {
        match &self.input {
            Input::Grant(grant) => Some(grant.valid_until),
            Input::Revoke(_) => None,
        }
    }
    pub fn grant_input(&self) -> Option<&NativeCompanyInformationGrantV1> {
        match &self.input {
            Input::Grant(grant) => Some(grant),
            Input::Revoke(_) => None,
        }
    }
    pub const fn child_assignment(&self) -> Option<CompanyInformationAssignmentRefV1> {
        match self.input {
            Input::Grant(_) => None,
            Input::Revoke(assignment) => Some(assignment),
        }
    }
}

fn validate_time(value: OffsetDateTime) -> Result<(), KernelError> {
    // RFC3339 years 0001..9999 and exact UTC microseconds; never round input.
    let nanos = value.unix_timestamp_nanos();
    if nanos % 1_000 != 0
        || !(-62_135_596_800_000_000_000..=253_402_300_799_999_999_000).contains(&nanos)
    {
        Err(invalid())
    } else {
        Ok(())
    }
}
