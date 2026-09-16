//! Payroll use cases (Clean Architecture Use Cases ring).
//!
//! This crate owns the admitted draft-run **calculate** path. Persistence is a
//! port implemented outside this crate. Domain arithmetic and SQL stay in
//! Entities / Interface Adapters. `payable` is not flipped here.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::future::Future;
use std::pin::Pin;

use uuid::Uuid;

pub type CalculateFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, CalculatePayrollRunError>> + Send + 'a>>;

/// Command: calculate (or replay) one draft payroll run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CalculatePayrollRun {
    pub run_id: Uuid,
}

/// Facts the use case needs before deciding persist vs replay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunCalculationSnapshot {
    pub status: String,
    pub current_version: Option<i32>,
    pub calculated_lines: i64,
    pub blocked_lines: i64,
}

/// Result of calculate. `idempotent` means this call wrote nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalculatePayrollRunOutcome {
    pub version: i32,
    pub calculated_lines: i64,
    pub blocked_lines: i64,
    pub exceptions_created: i64,
    pub idempotent: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CalculatePayrollRunError {
    #[error("payroll run not found")]
    NotFound,
    #[error("{message}")]
    InvalidState { message: String },
    #[error("payroll calculate port failed")]
    Internal,
}

/// Storage port for the calculate use case. Implemented by adapters / REST.
pub trait PayrollCalculatePort: Send {
    fn load_run(&mut self, run_id: Uuid) -> CalculateFuture<'_, Option<RunCalculationSnapshot>>;
    fn persist_calculation(
        &mut self,
        run_id: Uuid,
    ) -> CalculateFuture<'_, CalculatePayrollRunOutcome>;
}

/// Calculate a draft run. Duplicate calculate of an already-stored version is
/// a no-write replay (`idempotent = true`).
pub fn calculate_payroll_run<'a, P: PayrollCalculatePort + ?Sized>(
    port: &'a mut P,
    command: CalculatePayrollRun,
) -> CalculateFuture<'a, CalculatePayrollRunOutcome> {
    Box::pin(async move {
        let Some(snapshot) = port.load_run(command.run_id).await? else {
            return Err(CalculatePayrollRunError::NotFound);
        };
        if snapshot.status == "CALCULATED" {
            let Some(version) = snapshot.current_version else {
                return Err(invalid_calculate_state(&snapshot.status));
            };
            return Ok(CalculatePayrollRunOutcome {
                version,
                calculated_lines: snapshot.calculated_lines,
                blocked_lines: snapshot.blocked_lines,
                exceptions_created: 0,
                idempotent: true,
            });
        }
        if snapshot.status != "ATTENDANCE_CLOSED" {
            return Err(invalid_calculate_state(&snapshot.status));
        }
        let mut outcome = port.persist_calculation(command.run_id).await?;
        outcome.idempotent = false;
        Ok(outcome)
    })
}

fn invalid_calculate_state(status: &str) -> CalculatePayrollRunError {
    CalculatePayrollRunError::InvalidState {
        message: format!("cannot calculate a run in status {status}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::task::{Context, Poll, Waker};

    struct FakePort {
        snapshot: Option<RunCalculationSnapshot>,
        persist_calls: u32,
        persist_outcome: CalculatePayrollRunOutcome,
        after_persist: Option<RunCalculationSnapshot>,
    }

    impl PayrollCalculatePort for FakePort {
        fn load_run(
            &mut self,
            _run_id: Uuid,
        ) -> CalculateFuture<'_, Option<RunCalculationSnapshot>> {
            let snapshot = self.snapshot.clone();
            Box::pin(async move { Ok(snapshot) })
        }

        fn persist_calculation(
            &mut self,
            _run_id: Uuid,
        ) -> CalculateFuture<'_, CalculatePayrollRunOutcome> {
            self.persist_calls += 1;
            if let Some(after) = self.after_persist.clone() {
                self.snapshot = Some(after);
            }
            let outcome = self.persist_outcome.clone();
            Box::pin(async move { Ok(outcome) })
        }
    }

    fn block_on<T>(fut: impl Future<Output = T>) -> T {
        let waker = Waker::noop();
        let mut cx = Context::from_waker(waker);
        let mut fut = std::pin::pin!(fut);
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(value) => value,
            Poll::Pending => panic!("test future unexpectedly pending"),
        }
    }

    fn run_id() -> Uuid {
        Uuid::from_u128(1)
    }

    fn closed_port() -> FakePort {
        FakePort {
            snapshot: Some(RunCalculationSnapshot {
                status: "ATTENDANCE_CLOSED".to_owned(),
                current_version: None,
                calculated_lines: 0,
                blocked_lines: 0,
            }),
            persist_calls: 0,
            persist_outcome: CalculatePayrollRunOutcome {
                version: 1,
                calculated_lines: 2,
                blocked_lines: 0,
                exceptions_created: 1,
                idempotent: false,
            },
            after_persist: Some(RunCalculationSnapshot {
                status: "CALCULATED".to_owned(),
                current_version: Some(1),
                calculated_lines: 2,
                blocked_lines: 0,
            }),
        }
    }

    #[test]
    fn application_manifest_forbids_framework_deps() {
        let manifest = include_str!("../Cargo.toml");
        for forbidden in ["sqlx", "axum", "tokio"] {
            let present = manifest.lines().any(|line| {
                let trimmed = line.trim_start();
                !trimmed.starts_with('#') && trimmed.contains(forbidden)
            });
            assert!(
                !present,
                "{forbidden} must not appear in console-payroll-application"
            );
        }
    }

    #[test]
    fn calculate_persists_when_attendance_is_closed() {
        let mut port = closed_port();
        let outcome = block_on(calculate_payroll_run(
            &mut port,
            CalculatePayrollRun { run_id: run_id() },
        ))
        .unwrap();
        assert_eq!(port.persist_calls, 1);
        assert!(!outcome.idempotent);
        assert_eq!(outcome.version, 1);
        assert_eq!(outcome.calculated_lines, 2);
        assert_eq!(outcome.exceptions_created, 1);
    }

    #[test]
    fn duplicate_calculate_on_same_version_does_not_persist() {
        let mut port = closed_port();
        let first = block_on(calculate_payroll_run(
            &mut port,
            CalculatePayrollRun { run_id: run_id() },
        ))
        .unwrap();
        assert!(!first.idempotent);
        assert_eq!(port.persist_calls, 1);

        let second = block_on(calculate_payroll_run(
            &mut port,
            CalculatePayrollRun { run_id: run_id() },
        ))
        .unwrap();
        assert!(second.idempotent);
        assert_eq!(second.version, first.version);
        assert_eq!(second.calculated_lines, first.calculated_lines);
        assert_eq!(second.exceptions_created, 0);
        assert_eq!(port.persist_calls, 1);
    }

    #[test]
    fn missing_run_is_not_found() {
        let mut port = FakePort {
            snapshot: None,
            persist_calls: 0,
            persist_outcome: CalculatePayrollRunOutcome {
                version: 1,
                calculated_lines: 0,
                blocked_lines: 0,
                exceptions_created: 0,
                idempotent: false,
            },
            after_persist: None,
        };
        let err = block_on(calculate_payroll_run(
            &mut port,
            CalculatePayrollRun { run_id: run_id() },
        ))
        .unwrap_err();
        assert_eq!(err, CalculatePayrollRunError::NotFound);
        assert_eq!(port.persist_calls, 0);
    }

    #[test]
    fn other_status_is_invalid_state_and_does_not_persist() {
        let mut port = FakePort {
            snapshot: Some(RunCalculationSnapshot {
                status: "SUBMITTED".to_owned(),
                current_version: Some(1),
                calculated_lines: 2,
                blocked_lines: 0,
            }),
            persist_calls: 0,
            persist_outcome: CalculatePayrollRunOutcome {
                version: 2,
                calculated_lines: 2,
                blocked_lines: 0,
                exceptions_created: 0,
                idempotent: false,
            },
            after_persist: None,
        };
        let err = block_on(calculate_payroll_run(
            &mut port,
            CalculatePayrollRun { run_id: run_id() },
        ))
        .unwrap_err();
        assert!(
            matches!(err, CalculatePayrollRunError::InvalidState { .. }),
            "{err:?}"
        );
        assert_eq!(port.persist_calls, 0);
    }
}
