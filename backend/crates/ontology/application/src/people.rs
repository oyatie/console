//! Validated native directory input, shared by preparation and execution.

mod command;
mod receipt;

pub use command::{
    DIRECTORY_CODEC_VERSION, DIRECTORY_MANIFEST, DIRECTORY_MAX_INPUT_BYTES,
    DirectoryExpectationsV1, NativeDirectoryCommandV1,
};
pub use receipt::{
    AcceptedDirectoryRequestV1, DirectoryRejectionV1, DirectoryTerminalOutcomeV1,
    DirectoryTerminalV1,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectoryInputField {
    LegalName,
    EmployeeNumber,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectoryInputProblem {
    Required,
    TooLong,
    ControlCharacter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirectoryInputError {
    pub field: DirectoryInputField,
    pub problem: DirectoryInputProblem,
}

/// A directory identity is not evidence of employment or a verified person.
/// Private fields ensure every caller uses the same validation boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectoryRegistrationInput {
    legal_name: String,
    employee_number: String,
}

impl DirectoryRegistrationInput {
    pub fn new(legal_name: &str, employee_number: &str) -> Result<Self, DirectoryInputError> {
        Ok(Self {
            legal_name: validated(legal_name, DirectoryInputField::LegalName, 200)?.to_owned(),
            employee_number: validated(employee_number, DirectoryInputField::EmployeeNumber, 64)?
                .to_owned(),
        })
    }

    pub fn legal_name(&self) -> &str {
        &self.legal_name
    }

    pub fn employee_number(&self) -> &str {
        &self.employee_number
    }
}

fn validated(
    raw: &str,
    field: DirectoryInputField,
    max_scalars: usize,
) -> Result<&str, DirectoryInputError> {
    // Check before trimming so edge tabs/newlines cannot become valid input.
    if raw.chars().any(char::is_control) {
        return Err(DirectoryInputError {
            field,
            problem: DirectoryInputProblem::ControlCharacter,
        });
    }
    let value = raw.trim();
    let problem = if value.is_empty() {
        Some(DirectoryInputProblem::Required)
    } else if value.len() > max_scalars * 4 || value.chars().count() > max_scalars {
        Some(DirectoryInputProblem::TooLong)
    } else {
        None
    };
    match problem {
        Some(problem) => Err(DirectoryInputError { field, problem }),
        None => Ok(value),
    }
}
