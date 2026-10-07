//! Closed protocols carried by the shared policy owner. The stored codec selects
//! one grammar; decoding never guesses and never changes historical bytes.
use super::super::{
    AccountId,
    business::{
        self, NativeBusinessOperationV1, NativeCompanyBusinessCommandV1,
        PolicyAssignmentExpectationV1,
    },
    company_information::{self, NativeCompanyInformationCommandV1},
    org_unit_business::{self, NativeOrgUnitPolicyCommandV1},
    people_business::{self, NativePeoplePolicyCommandV1},
};
use console_kernel_core::{KernelError, OrgId};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativePolicyCommand {
    Payroll(NativeCompanyBusinessCommandV1),
    People(NativePeoplePolicyCommandV1),
    OrgUnit(NativeOrgUnitPolicyCommandV1),
    CompanyInformation(NativeCompanyInformationCommandV1),
}

impl From<NativeCompanyBusinessCommandV1> for NativePolicyCommand {
    fn from(input: NativeCompanyBusinessCommandV1) -> Self {
        Self::Payroll(input)
    }
}

impl From<NativePeoplePolicyCommandV1> for NativePolicyCommand {
    fn from(input: NativePeoplePolicyCommandV1) -> Self {
        Self::People(input)
    }
}

impl From<&NativeCompanyBusinessCommandV1> for NativePolicyCommand {
    fn from(input: &NativeCompanyBusinessCommandV1) -> Self {
        Self::Payroll(input.clone())
    }
}

impl From<&NativePeoplePolicyCommandV1> for NativePolicyCommand {
    fn from(input: &NativePeoplePolicyCommandV1) -> Self {
        Self::People(input.clone())
    }
}

impl From<NativeOrgUnitPolicyCommandV1> for NativePolicyCommand {
    fn from(input: NativeOrgUnitPolicyCommandV1) -> Self {
        Self::OrgUnit(input)
    }
}

impl From<&NativeOrgUnitPolicyCommandV1> for NativePolicyCommand {
    fn from(input: &NativeOrgUnitPolicyCommandV1) -> Self {
        Self::OrgUnit(input.clone())
    }
}

impl From<NativeCompanyInformationCommandV1> for NativePolicyCommand {
    fn from(input: NativeCompanyInformationCommandV1) -> Self {
        Self::CompanyInformation(input)
    }
}

impl From<&NativeCompanyInformationCommandV1> for NativePolicyCommand {
    fn from(input: &NativeCompanyInformationCommandV1) -> Self {
        Self::CompanyInformation(input.clone())
    }
}

impl From<&NativePolicyCommand> for NativePolicyCommand {
    fn from(input: &NativePolicyCommand) -> Self {
        input.clone()
    }
}

impl NativePolicyCommand {
    pub const fn codec_version(&self) -> i16 {
        match self {
            Self::Payroll(_) => 1,
            Self::People(_) => 2,
            Self::OrgUnit(_) => 3,
            Self::CompanyInformation(_) => 4,
        }
    }

    pub const fn catalog_version(&self) -> &'static str {
        match self {
            Self::Payroll(_) => "native-payroll-collection-read-v1",
            Self::People(_) => "native-people-directory-v1",
            Self::OrgUnit(_) => "native-org-unit-work-v1",
            Self::CompanyInformation(_) => "native-company-identity-2026-09-19.1",
        }
    }

    pub const fn manifest_digest(&self) -> &'static [u8; 32] {
        match self {
            Self::Payroll(_) => &business::MANIFEST,
            Self::People(_) => &people_business::MANIFEST,
            Self::OrgUnit(_) => &org_unit_business::MANIFEST,
            Self::CompanyInformation(_) => &company_information::MANIFEST,
        }
    }

    pub const fn command_id(&self) -> Uuid {
        match self {
            Self::Payroll(input) => input.command_id(),
            Self::People(input) => input.command_id(),
            Self::OrgUnit(input) => input.command_id(),
            Self::CompanyInformation(input) => input.command_id(),
        }
    }

    pub const fn company(&self) -> OrgId {
        match self {
            Self::Payroll(input) => input.company(),
            Self::People(input) => input.company(),
            Self::OrgUnit(input) => input.company(),
            Self::CompanyInformation(input) => input.company(),
        }
    }

    pub const fn expected_company_epoch(&self) -> u64 {
        match self {
            Self::Payroll(input) => input.expected_company_epoch(),
            Self::People(input) => input.expected_company_epoch(),
            Self::OrgUnit(input) => input.expected_company_epoch(),
            Self::CompanyInformation(input) => input.expected_company_epoch(),
        }
    }

    pub const fn operation(&self) -> NativeBusinessOperationV1 {
        match self {
            Self::Payroll(input) => input.operation(),
            Self::People(input) => input.operation(),
            Self::OrgUnit(input) => input.operation(),
            Self::CompanyInformation(input) => input.operation(),
        }
    }

    pub const fn recipient_account_id(&self) -> Option<AccountId> {
        match self {
            Self::Payroll(input) => input.recipient_account_id(),
            Self::People(input) => input.recipient_account_id(),
            Self::OrgUnit(input) => input.recipient_account_id(),
            Self::CompanyInformation(input) => input.recipient_account_id(),
        }
    }

    pub const fn assignment_expectation(&self) -> Option<PolicyAssignmentExpectationV1> {
        match self {
            Self::Payroll(input) => input.assignment_expectation(),
            Self::People(input) => input.assignment_expectation(),
            Self::OrgUnit(input) => input.assignment_expectation(),
            // Codec 4 carries an exact child ref without a legacy role revision.
            Self::CompanyInformation(_) => None,
        }
    }

    pub const fn expires_at(&self) -> Option<OffsetDateTime> {
        match self {
            Self::Payroll(input) => input.expires_at(),
            Self::People(input) => input.expires_at(),
            Self::OrgUnit(input) => input.expires_at(),
            Self::CompanyInformation(input) => input.expires_at(),
        }
    }

    pub fn encode(&self, actor: AccountId) -> Vec<u8> {
        match self {
            Self::Payroll(input) => input.encode(actor),
            Self::People(input) => input.encode(actor),
            Self::OrgUnit(input) => input.encode(actor),
            Self::CompanyInformation(input) => input.encode(actor),
        }
    }

    pub fn decode(codec: i16, bytes: &[u8]) -> Result<(AccountId, Self), KernelError> {
        match codec {
            1 => NativeCompanyBusinessCommandV1::decode(bytes)
                .map(|(actor, input)| (actor, Self::Payroll(input))),
            2 => NativePeoplePolicyCommandV1::decode(bytes)
                .map(|(actor, input)| (actor, Self::People(input))),
            3 => NativeOrgUnitPolicyCommandV1::decode(bytes)
                .map(|(actor, input)| (actor, Self::OrgUnit(input))),
            4 => NativeCompanyInformationCommandV1::decode(bytes)
                .map(|(actor, input)| (actor, Self::CompanyInformation(input))),
            _ => Err(business::invalid()),
        }
    }
}
