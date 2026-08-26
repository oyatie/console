//! UI-owned ports. App composition wires real readers; this crate never
//! depends on domain/application/adapter/rest.

use serde::{Deserialize, Serialize};

pub const POST_COMPANY_REVISE: &str = "/ui/company/revise";
pub const POST_ORG_UNIT_CREATE: &str = "/ui/org-units";
pub const POST_JOB_POSITION_CREATE: &str = "/ui/job-positions";
pub const POST_PERSON_CREATE: &str = "/ui/people";
pub const POST_EMPLOYMENT_APPOINT: &str = "/ui/employments/appoint";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompanyHead {
    pub org_id: String,
    pub legal_name: String,
    pub reg_no: String,
    pub version: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrgUnitHead {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub version: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobPositionHead {
    pub id: String,
    pub title: String,
    pub org_unit_id: String,
    pub version: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonHead {
    pub id: String,
    pub display_name: String,
    pub legal_name: String,
    pub version: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmploymentHead {
    pub id: String,
    pub person_id: String,
    pub org_unit_id: String,
    pub job_position_id: String,
    pub appointed_on: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OrgSnapshot {
    pub company: Option<CompanyHead>,
    pub org_units: Vec<OrgUnitHead>,
    pub job_positions: Vec<JobPositionHead>,
    pub people: Vec<PersonHead>,
    pub employments: Vec<EmploymentHead>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteError {
    FailClosed,
    Unauthorized,
}

impl std::fmt::Display for WriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FailClosed => write!(f, "write port is not wired"),
            Self::Unauthorized => write!(f, "unauthorized"),
        }
    }
}

pub trait CompanyReadPort {
    fn head(&self) -> Option<CompanyHead>;
}

pub trait OrgReadPort {
    fn snapshot(&self) -> OrgSnapshot;
}

pub trait CompanyWritePort {
    fn revise(
        &self,
        actor_id: &str,
        org_id: &str,
        version: i32,
        legal_name: &str,
        reg_no: &str,
    ) -> Result<(), WriteError>;
}

/// Fail-closed dummy used by unit tests and until app composition wires ports.
#[derive(Clone, Debug, Default)]
pub struct FailClosedOrg;

impl CompanyReadPort for FailClosedOrg {
    fn head(&self) -> Option<CompanyHead> {
        None
    }
}

impl OrgReadPort for FailClosedOrg {
    fn snapshot(&self) -> OrgSnapshot {
        OrgSnapshot::default()
    }
}

impl CompanyWritePort for FailClosedOrg {
    fn revise(
        &self,
        _actor_id: &str,
        _org_id: &str,
        _version: i32,
        _legal_name: &str,
        _reg_no: &str,
    ) -> Result<(), WriteError> {
        Err(WriteError::FailClosed)
    }
}
