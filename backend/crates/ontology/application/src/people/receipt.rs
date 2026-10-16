//! Directory owner result types; construction does not authorize disclosure.
use crate::people::NativeDirectoryCommandV1;
use console_kernel_core::{AccountId, KernelError};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptedDirectoryRequestV1 {
    actor: AccountId,
    command: NativeDirectoryCommandV1,
    intake_receipt_id: Uuid,
    accepted_at: OffsetDateTime,
    execution_not_after: OffsetDateTime,
}
impl AcceptedDirectoryRequestV1 {
    pub fn new(
        actor: AccountId,
        command: NativeDirectoryCommandV1,
        intake_receipt_id: Uuid,
        accepted_at: OffsetDateTime,
    ) -> Result<Self, KernelError> {
        if intake_receipt_id.is_nil() || accepted_at.unix_timestamp_nanos() % 1_000 != 0 {
            return Err(invalid());
        }
        let execution_not_after = accepted_at
            .checked_add(Duration::hours(168))
            .ok_or_else(invalid)?;
        Ok(Self {
            actor,
            command,
            intake_receipt_id,
            accepted_at,
            execution_not_after,
        })
    }
    pub const fn actor(&self) -> AccountId {
        self.actor
    }
    pub fn command(&self) -> &NativeDirectoryCommandV1 {
        &self.command
    }
    pub const fn intake_receipt_id(&self) -> Uuid {
        self.intake_receipt_id
    }
    pub const fn accepted_at(&self) -> OffsetDateTime {
        self.accepted_at
    }
    pub const fn execution_not_after(&self) -> OffsetDateTime {
        self.execution_not_after
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectoryRejectionV1 {
    RevisionConflict,
    EmployeeNumberConflict,
    CommandConflict,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectoryTerminalOutcomeV1 {
    Committed,
    Rejected(DirectoryRejectionV1),
    Cancelled,
    Expired,
}
impl DirectoryTerminalOutcomeV1 {
    pub const fn outcome(self) -> &'static str {
        match self {
            Self::Committed => "COMMITTED",
            Self::Rejected(_) => "REJECTED",
            Self::Cancelled => "CANCELLED",
            Self::Expired => "EXPIRED",
        }
    }
    pub const fn result_code(self) -> &'static str {
        match self {
            Self::Committed => "registered",
            Self::Rejected(DirectoryRejectionV1::RevisionConflict) => "revision_conflict",
            Self::Rejected(DirectoryRejectionV1::EmployeeNumberConflict) => {
                "employee_number_conflict"
            }
            Self::Rejected(DirectoryRejectionV1::CommandConflict) => "command_conflict",
            Self::Cancelled => "cancelled",
            Self::Expired => "intake_expired",
        }
    }
    pub fn from_storage(outcome: &str, code: &str) -> Result<Self, KernelError> {
        match (outcome, code) {
            ("COMMITTED", "registered") => Ok(Self::Committed),
            ("REJECTED", "revision_conflict") => {
                Ok(Self::Rejected(DirectoryRejectionV1::RevisionConflict))
            }
            ("REJECTED", "employee_number_conflict") => {
                Ok(Self::Rejected(DirectoryRejectionV1::EmployeeNumberConflict))
            }
            ("REJECTED", "command_conflict") => {
                Ok(Self::Rejected(DirectoryRejectionV1::CommandConflict))
            }
            ("CANCELLED", "cancelled") => Ok(Self::Cancelled),
            ("EXPIRED", "intake_expired") => Ok(Self::Expired),
            _ => Err(invalid()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectoryTerminalV1 {
    accepted: AcceptedDirectoryRequestV1,
    outcome: DirectoryTerminalOutcomeV1,
    terminal_at: OffsetDateTime,
}
impl DirectoryTerminalV1 {
    pub fn new(
        accepted: AcceptedDirectoryRequestV1,
        outcome: DirectoryTerminalOutcomeV1,
        terminal_at: OffsetDateTime,
    ) -> Result<Self, KernelError> {
        if terminal_at.unix_timestamp_nanos() % 1_000 != 0
            || terminal_at < accepted.accepted_at
            || ((terminal_at >= accepted.execution_not_after)
                != matches!(outcome, DirectoryTerminalOutcomeV1::Expired))
        {
            return Err(invalid());
        }
        Ok(Self {
            accepted,
            outcome,
            terminal_at,
        })
    }
    pub fn accepted(&self) -> &AcceptedDirectoryRequestV1 {
        &self.accepted
    }
    pub const fn outcome(&self) -> DirectoryTerminalOutcomeV1 {
        self.outcome
    }
    pub const fn terminal_at(&self) -> OffsetDateTime {
        self.terminal_at
    }
    pub fn employee_id(&self) -> Option<Uuid> {
        matches!(self.outcome, DirectoryTerminalOutcomeV1::Committed)
            .then_some(self.accepted.command.employee_id())
    }
    pub fn person_id(&self) -> Option<Uuid> {
        self.employee_id()
    }
    pub fn canonical_command_id(&self) -> Option<Uuid> {
        matches!(self.outcome, DirectoryTerminalOutcomeV1::Committed)
            .then_some(self.accepted.command.command_id())
    }
    pub fn canonical_result(&self) -> Option<serde_json::Value> {
        self.employee_id().map(
            |id| serde_json::json!({"person_id":id,"version":1,"target":"people.create_person"}),
        )
    }
}
fn invalid() -> KernelError {
    KernelError::validation("invalid native directory receipt")
}

#[cfg(test)]
#[path = "receipt_tests.rs"]
mod tests;
