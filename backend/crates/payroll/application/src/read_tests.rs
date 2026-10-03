//! Application orchestration only. This scripted port is not PostgreSQL,
//! authentication, Cedar, rollback or browser acceptance evidence.
use super::*;
use console_kernel_core::OrgId;
use std::task::{Context, Poll, Waker};

struct Reader {
    events: Vec<&'static str>,
    authorize: Option<Result<(), PayrollRunsReadError>>,
    read: Option<Result<PayrollRunsReadResult, PayrollRunsReadError>>,
    wait_authorize: bool,
    wait_read: bool,
}
impl PayrollRunsReadPort for Reader {
    fn finish_navigation(&mut self) -> PayrollNavigationFuture<'_> {
        panic!("collection-only test must not finalize navigation")
    }

    fn authorize(&mut self) -> PayrollAuthorizeFuture<'_> {
        Box::pin(async move {
            self.events.push("authorize");
            if self.wait_authorize {
                std::future::pending::<()>().await;
            }
            self.authorize.take().expect("authorization called twice")
        })
    }
    fn read_page(&mut self, query: ListPayrollRuns) -> PayrollReadFuture<'_> {
        Box::pin(async move {
            self.events.push("read");
            assert_eq!(query.limit, Some(37));
            assert_eq!(query.offset, Some(74));
            if self.wait_read {
                std::future::pending::<()>().await;
            }
            self.read.take().expect("read called twice")
        })
    }
}
fn query() -> ListPayrollRuns {
    ListPayrollRuns {
        limit: Some(37),
        offset: Some(74),
    }
}
fn page() -> PayrollRunPage {
    let at = OffsetDateTime::from_unix_timestamp(1788220800).unwrap();
    PayrollRunPage {
        items: vec![PayrollRunSummary {
            id: Uuid::from_u128(80),
            period_start: Date::from_calendar_date(2026, time::Month::September, 1).unwrap(),
            period_end: Date::from_calendar_date(2026, time::Month::September, 30).unwrap(),
            source_label: "승인 대기 <급여>".into(),
            status: "SUBMITTED".into(),
            calculation_enabled: false,
            created_by: Some(Uuid::from_u128(81)),
            approved_by: None,
            approved_at: None,
            close_receipt: Some(serde_json::json!({"evidence": "<restricted>", "revision": 9})),
            submitted_by: Some(Uuid::from_u128(82)),
            submitted_at: Some(at),
            decided_by: Some(Uuid::from_u128(83)),
            decided_at: Some(at),
            decision_reason: Some("검토 사유 & 이력".into()),
            approval_ref: Some(Uuid::from_u128(84)),
            created_at: at,
            updated_at: at,
        }],
        total: 101,
        limit: 37,
        offset: 74,
    }
}
fn reader() -> Reader {
    Reader {
        events: vec![],
        authorize: Some(Ok(())),
        read: Some(Ok(PayrollRunsReadResult {
            page: page(),
            company: None,
        })),
        wait_authorize: false,
        wait_read: false,
    }
}
fn ready<F: Future>(future: F) -> F::Output {
    match std::pin::pin!(future).poll(&mut Context::from_waker(Waker::noop())) {
        Poll::Ready(value) => value,
        Poll::Pending => panic!("unexpected wait in ready script"),
    }
}
fn failure(kind: usize) -> PayrollRunsReadError {
    match kind {
        0 => PayrollRunsReadError::Authorization(KernelError::not_found("scope")),
        1 => PayrollRunsReadError::AuthenticationInvalid,
        2 => PayrollRunsReadError::Unavailable,
        4 => PayrollRunsReadError::Authorization(KernelError::forbidden("denied")),
        _ => PayrollRunsReadError::Read(KernelError::internal("read sentinel")),
    }
}
fn assert_failure(error: PayrollRunsReadError, kind: usize) {
    match (kind, error) {
        (0, PayrollRunsReadError::Authorization(error)) => {
            assert_eq!(error, KernelError::not_found("scope"))
        }
        (1, PayrollRunsReadError::AuthenticationInvalid) => {}
        (2, PayrollRunsReadError::Unavailable) => {}
        (3, PayrollRunsReadError::Read(error)) => {
            assert_eq!(error, KernelError::internal("read sentinel"))
        }
        (4, PayrollRunsReadError::Authorization(error)) => {
            assert_eq!(error, KernelError::forbidden("denied"))
        }
        (_, error) => panic!("classification changed: {error:?}"),
    }
}

#[test]
fn authorization_is_awaited_before_read_and_failure_never_reads() {
    for kind in 0..5 {
        let mut port = reader();
        port.authorize = Some(Err(failure(kind)));
        let result = ready(list_payroll_runs(&mut port, query()));
        assert_failure(
            result.expect_err("failed authorization released data"),
            kind,
        );
        assert_eq!(port.events, ["authorize"]);
        assert!(port.read.is_some());
    }
    let mut port = reader();
    port.wait_authorize = true;
    let mut future = Box::pin(list_payroll_runs(&mut port, query()));
    assert!(
        future
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop()))
            .is_pending()
    );
    drop(future);
    // Keep the reader alive: this proves no read happened after cancellation,
    // not release of a real retained PostgreSQL transaction.
    assert_eq!(port.events, ["authorize"]);
    assert!(port.authorize.is_some() && port.read.is_some());
}

#[test]
fn final_read_failures_preserve_authority_authentication_and_uncertainty() {
    for kind in 0..5 {
        let mut port = reader();
        port.read = Some(Err(failure(kind)));
        assert_failure(
            ready(list_payroll_runs(&mut port, query())).err().unwrap(),
            kind,
        );
        assert_eq!(port.events, ["authorize", "read"]);
    }
}

#[test]
fn committed_scope_context_stays_outside_legacy_page_wire_shape() {
    for identity in [false, true] {
        let mut port = reader();
        let company = OrgId::from_uuid(Uuid::from_u128(71));
        port.read = Some(Ok(PayrollRunsReadResult {
            page: page(),
            company: Some(PayrollCompanyContext {
                id: company,
                identity: identity.then(|| PayrollCompanyIdentity {
                    name: "authorized Company".into(),
                    slug: "authorized-company".into(),
                }),
            }),
        }));
        let result = ready(list_payroll_runs(&mut port, query())).unwrap();
        assert_eq!(port.events, ["authorize", "read"]);
        let context = result.company.unwrap();
        assert_eq!(context.id, company);
        assert_eq!(context.identity.is_some(), identity);
        if let Some(label) = context.identity {
            assert_eq!(label.name, "authorized Company");
            assert_eq!(label.slug, "authorized-company");
        }
        let expected = serde_json::to_value(page()).unwrap();
        let actual = serde_json::to_value(result.page).unwrap();
        let keys = |value: &serde_json::Value| {
            value
                .as_object()
                .unwrap()
                .keys()
                .cloned()
                .collect::<std::collections::BTreeSet<_>>()
        };
        assert_eq!(
            keys(&actual),
            ["items", "total", "limit", "offset"]
                .map(str::to_owned)
                .into()
        );
        assert_eq!(
            keys(&actual["items"][0]),
            [
                "id",
                "period_start",
                "period_end",
                "source_label",
                "status",
                "calculation_enabled",
                "created_by",
                "approved_by",
                "approved_at",
                "close_receipt",
                "submitted_by",
                "submitted_at",
                "decided_by",
                "decided_at",
                "decision_reason",
                "approval_ref",
                "created_at",
                "updated_at"
            ]
            .map(str::to_owned)
            .into()
        );
        assert_eq!(actual, expected);
    }
    let mut port = reader();
    assert!(
        ready(list_payroll_runs(&mut port, query()))
            .unwrap()
            .company
            .is_none()
    );
}

#[test]
fn pending_read_never_publishes_a_result_before_completion() {
    let mut port = reader();
    port.wait_read = true;
    let mut future = Box::pin(list_payroll_runs(&mut port, query()));
    assert!(
        future
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop()))
            .is_pending()
    );
    drop(future);
    assert_eq!(port.events, ["authorize", "read"]);
    assert!(
        port.read.is_some(),
        "unconfirmed read released the scripted result"
    );
    // Actual transaction/lock release is verified by the native PostgreSQL owner test.
}
