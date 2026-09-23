//! Closed protocols carried by the shared policy owner. The stored codec selects
//! one grammar; decoding never guesses and never changes historical bytes.
use super::super::{
    AccountId,
    business::{
        self, NativeBusinessOperationV1, NativeCompanyBusinessCommandV1,
        PolicyAssignmentExpectationV1,
    },
    people_business::{self, NativePeoplePolicyCommandV1},
};
use console_kernel_core::{KernelError, OrgId};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativePolicyCommand {
    Payroll(NativeCompanyBusinessCommandV1),
    People(NativePeoplePolicyCommandV1),
}

impl NativePolicyCommand {
    pub const fn codec_version(&self) -> i16 {
        match self {
            Self::Payroll(_) => 1,
            Self::People(_) => 2,
        }
    }

    pub const fn catalog_version(&self) -> &'static str {
        match self {
            Self::Payroll(_) => "native-payroll-collection-read-v1",
            Self::People(_) => "native-people-directory-v1",
        }
    }

    pub const fn manifest_digest(&self) -> &'static [u8; 32] {
        match self {
            Self::Payroll(_) => &business::MANIFEST,
            Self::People(_) => &people_business::MANIFEST,
        }
    }

    pub const fn command_id(&self) -> Uuid {
        match self {
            Self::Payroll(input) => input.command_id(),
            Self::People(input) => input.command_id(),
        }
    }

    pub const fn company(&self) -> OrgId {
        match self {
            Self::Payroll(input) => input.company(),
            Self::People(input) => input.company(),
        }
    }

    pub const fn expected_company_epoch(&self) -> u64 {
        match self {
            Self::Payroll(input) => input.expected_company_epoch(),
            Self::People(input) => input.expected_company_epoch(),
        }
    }

    pub const fn operation(&self) -> NativeBusinessOperationV1 {
        match self {
            Self::Payroll(input) => input.operation(),
            Self::People(input) => input.operation(),
        }
    }

    pub const fn recipient_account_id(&self) -> Option<AccountId> {
        match self {
            Self::Payroll(input) => input.recipient_account_id(),
            Self::People(input) => input.recipient_account_id(),
        }
    }

    pub const fn assignment_expectation(&self) -> Option<PolicyAssignmentExpectationV1> {
        match self {
            Self::Payroll(input) => input.assignment_expectation(),
            Self::People(input) => input.assignment_expectation(),
        }
    }

    pub const fn expires_at(&self) -> Option<OffsetDateTime> {
        match self {
            Self::Payroll(input) => input.expires_at(),
            Self::People(input) => input.expires_at(),
        }
    }

    pub fn encode(&self, actor: AccountId) -> Vec<u8> {
        match self {
            Self::Payroll(input) => input.encode(actor),
            Self::People(input) => input.encode(actor),
        }
    }

    pub fn decode(codec: i16, bytes: &[u8]) -> Result<(AccountId, Self), KernelError> {
        match codec {
            1 => NativeCompanyBusinessCommandV1::decode(bytes)
                .map(|(actor, input)| (actor, Self::Payroll(input))),
            2 => NativePeoplePolicyCommandV1::decode(bytes)
                .map(|(actor, input)| (actor, Self::People(input))),
            _ => Err(business::invalid()),
        }
    }
}
