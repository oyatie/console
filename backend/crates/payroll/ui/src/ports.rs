//! UI-owned payroll ports. The client never computes won amounts.

use serde::{Deserialize, Serialize};

pub const POST_RUN_CREATE: &str = "/ui/payroll/runs";
pub const POST_ATTENDANCE_HANDOFF: &str = "/ui/attendance/handoff";
pub const POST_CLOSE_ATTENDANCE: &str = "/ui/payroll/runs/close-attendance";
pub const POST_CALCULATE: &str = "/ui/payroll/runs/calculate";
pub const POST_RESOLVE_EXCEPTION: &str = "/ui/payroll/runs/exceptions/resolve";
pub const POST_SUBMIT: &str = "/ui/payroll/runs/submit";
pub const POST_ISSUE: &str = "/ui/payroll/runs/issue-payslips";
pub const POST_DECIDE: &str = "/ui/approvals/decide";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PayRunSummary {
    pub id: String,
    pub period_start: String,
    pub period_end: String,
    pub status: String,
    pub submitted_by: Option<String>,
    pub decided_by: Option<String>,
    pub exceptions_open: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lineage {
    pub label_ko: String,
    pub source_ko: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoneyLine {
    pub code: String,
    pub label_ko: String,
    pub amount_won: Option<i64>,
    pub lineage: Lineage,
    pub overridable: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PayRunDetail {
    pub run: PayRunSummary,
    pub lines: Vec<String>,
    pub calculation_version: Option<i32>,
    pub total_net_won: Option<i64>,
    pub total_net_lineage: Option<Lineage>,
    pub exceptions: Vec<PayrollExceptionView>,
    pub payable: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PayrollExceptionView {
    pub id: String,
    pub run_id: String,
    pub summary_ko: String,
    pub status: String,
    pub amount_delta_won: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttendancePeriod {
    pub period_start: String,
    pub period_end: String,
    pub worked_days: i64,
    pub clock_in_events: i64,
    pub clock_out_events: i64,
    pub closed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MyPayslip {
    pub period_start: String,
    pub period_end: String,
    pub employee_name: String,
    pub base_pay_won: Option<i64>,
    pub earnings: Vec<MoneyLine>,
    pub deductions: Vec<MoneyLine>,
    pub net_pay_won: Option<i64>,
    pub net_pay_unavailable_reason_ko: Option<String>,
    pub citations: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecideInboxItem {
    pub run_id: String,
    pub period_start: String,
    pub period_end: String,
    pub submitted_by: String,
    pub submitted_at: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PayrollSnapshot {
    pub runs: Vec<PayRunSummary>,
    pub selected: Option<PayRunDetail>,
    pub attendance: Option<AttendancePeriod>,
    pub rates_present: bool,
    pub my_payslip: Option<MyPayslip>,
    pub inbox: Vec<DecideInboxItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteError {
    FailClosed,
    Unauthorized,
    Sod,
}

impl std::fmt::Display for WriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FailClosed => write!(f, "write port is not wired"),
            Self::Unauthorized => write!(f, "unauthorized"),
            Self::Sod => write!(f, "decider must not be the submitter"),
        }
    }
}

pub trait PayrollReadPort {
    fn snapshot(&self) -> PayrollSnapshot;
}

pub trait PayrollWritePort {
    fn decide(&self, actor_id: &str, run_id: &str, submitted_by: &str) -> Result<(), WriteError>;
}

#[derive(Clone, Debug, Default)]
pub struct FailClosedPayroll;

impl PayrollReadPort for FailClosedPayroll {
    fn snapshot(&self) -> PayrollSnapshot {
        PayrollSnapshot::default()
    }
}

impl PayrollWritePort for FailClosedPayroll {
    fn decide(&self, actor_id: &str, _run_id: &str, submitted_by: &str) -> Result<(), WriteError> {
        if actor_id == submitted_by {
            return Err(WriteError::Sod);
        }
        Err(WriteError::FailClosed)
    }
}
