//! Payroll execution SSR surfaces. The kernel remains the oracle: this crate
//! never computes tax, insurance, or net pay.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod approvals;
pub mod attendance;
pub mod ess;
pub mod islands;
pub mod ports;
pub mod runs;
pub mod status;

pub use approvals::ApprovalsPage;
pub use attendance::AttendanceHandoffPage;
pub use ess::EssPage;
pub use islands::{
    AttendanceHandoffForm, CalculateRunForm, CloseAttendanceForm, CreateRunForm, DecideRunForm,
    IssuePayslipsForm, ResolveExceptionForm, SubmitRunForm, link_islands,
};
pub use ports::{
    AttendancePeriod, DecideInboxItem, FailClosedPayroll, Lineage, MoneyLine, MyPayslip,
    PayRunDetail, PayRunSummary, PayrollExceptionView, PayrollReadPort, PayrollSnapshot,
    PayrollWritePort, WriteError,
};
pub use runs::{RunDetailPage, RunsPage};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn island_modules_exist() {
        let src = include_str!("islands.rs");
        assert!(src.contains("#[island]"));
        for name in islands::ISLAND_NAMES {
            assert!(src.contains(name), "missing island {name}");
        }
        link_islands();
    }

    #[test]
    fn empty_state_is_create_not_import() {
        let src = [include_str!("runs.rs"), include_str!("islands.rs")].join("\n");
        let lower = src.to_ascii_lowercase();
        assert!(src.contains("등록"));
        assert!(!lower.contains("import"));
        assert!(!src.contains("가져오기"));
        assert!(!src.contains("엑셀"));
        assert!(!lower.contains("storeexport"));
    }

    #[test]
    fn decide_is_inbox_not_inline() {
        let detail = include_str!("runs.rs");
        let inbox = include_str!("approvals.rs");
        assert!(
            !detail.contains("DecideRunForm"),
            "run detail must not embed the decide island"
        );
        assert!(inbox.contains("DecideRunForm"));
        assert!(detail.contains("결재는 수신함에서"));
    }

    #[test]
    fn sod_fail_closed_on_same_actor() {
        let err = FailClosedPayroll
            .decide("user-1", "run-1", "user-1")
            .unwrap_err();
        assert_eq!(err, WriteError::Sod);
    }

    #[test]
    fn statutory_and_base_pay_not_overridable() {
        let ess = include_str!("ess.rs");
        assert!(ess.contains("수정 불가") || ess.contains("기본급"));
        assert!(!ess.contains("name=\"base_pay\""));
        assert!(!ess.contains("name=\"national_pension\""));
    }
}
