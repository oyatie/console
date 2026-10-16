//! The Group verification-process owner. Identifiers select resources; checked
//! source material and current policy grant access inside retained transactions.
pub use console_identity_domain::{
    AccountId,
    group_process::{GroupId, GroupIncarnation},
};

mod codec;
mod material;
mod workflow;

pub use codec::*;
pub use material::*;
pub use workflow::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroupProcessError {
    InvalidInput,
    AuthenticationInvalid,
    CsrfInvalid,
    NotFound,
    Conflict,
    Unavailable,
    /// Commit was not confirmed; reconcile the original immutable locator.
    Unconfirmed,
}

pub(crate) const MAX_REVISION: u64 = i64::MAX as u64;

pub(crate) fn nonnil(value: uuid::Uuid) -> Result<uuid::Uuid, GroupProcessError> {
    if value.is_nil() {
        Err(GroupProcessError::Unavailable)
    } else {
        Ok(value)
    }
}

pub(crate) fn revision(value: u64, allow_zero: bool) -> Result<u64, GroupProcessError> {
    if value > MAX_REVISION || (!allow_zero && value == 0) {
        Err(GroupProcessError::Unavailable)
    } else {
        Ok(value)
    }
}

pub(crate) fn text(value: &str, max: usize) -> Result<(), GroupProcessError> {
    if value.is_empty()
        || value.len() > max
        || value.trim().is_empty()
        || value
            .chars()
            .any(|c| c.is_control() && c != '\n' && c != '\t')
    {
        Err(GroupProcessError::Unavailable)
    } else {
        Ok(())
    }
}

#[cfg(test)]
#[path = "group_process/command_contract_tests.rs"]
mod command_contract_tests;

#[cfg(test)]
#[path = "group_process/postimage_contract_tests.rs"]
mod postimage_contract_tests;
