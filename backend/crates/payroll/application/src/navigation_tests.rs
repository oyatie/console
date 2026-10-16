// Orchestration-only port: real Auth/Cedar/transaction proof is in native owner tests.
use super::*;
use std::task::{Context, Poll, Waker};

struct Navigation {
    events: Vec<&'static str>,
    fail_at: Option<&'static str>,
    company: OrgId,
}
impl PayrollRunsReadPort for Navigation {
    fn authorize(&mut self) -> PayrollAuthorizeFuture<'_> {
        Box::pin(async move {
            self.events.push("authorize");
            if self.fail_at == Some("authorize") {
                Err(PayrollRunsReadError::AuthenticationInvalid)
            } else {
                Ok(())
            }
        })
    }
    fn finish_navigation(&mut self) -> PayrollNavigationFuture<'_> {
        Box::pin(async move {
            self.events.push("finish_navigation");
            if self.fail_at == Some("finish_navigation") {
                Err(PayrollRunsReadError::Unavailable)
            } else {
                Ok(self.company)
            }
        })
    }
    fn read_page(&mut self, _: ListPayrollRuns) -> PayrollReadFuture<'_> {
        panic!("navigation must never request payroll rows")
    }
}
#[test]
fn navigation_waits_for_both_owner_phases_and_never_reads_rows() {
    for fail_at in [None, Some("authorize"), Some("finish_navigation")] {
        let company = OrgId::from_uuid(Uuid::from_u128(117));
        let mut port = Navigation {
            events: vec![],
            fail_at,
            company,
        };
        let result = {
            let mut future = std::pin::pin!(payroll_navigation(&mut port));
            let Poll::Ready(result) = future
                .as_mut()
                .poll(&mut Context::from_waker(Waker::noop()))
            else {
                panic!("scripted immediate result did not complete");
            };
            result
        };
        match (fail_at, result) {
            (None, Ok(actual)) => assert_eq!(actual, company),
            (Some("authorize"), Err(PayrollRunsReadError::AuthenticationInvalid)) => {}
            (Some("finish_navigation"), Err(PayrollRunsReadError::Unavailable)) => {}
            (_, result) => panic!("changed navigation outcome {result:?}"),
        }
        assert_eq!(
            port.events,
            if fail_at == Some("authorize") {
                vec!["authorize"]
            } else {
                vec!["authorize", "finish_navigation"]
            }
        );
    }
}
