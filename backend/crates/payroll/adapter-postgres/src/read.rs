//! Authorized collection reads and their audit commit in the existing scoped transaction.
use console_kernel_core::{AuditAction, AuditEvent, KernelError, TraceContext};
use console_payroll_application::read::{
    ListPayrollRuns, PayrollAuthorizeFuture, PayrollReadFuture, PayrollRunsReadError,
    PayrollRunsReadPort, PayrollRunsReadResult,
};
use console_platform_authz::{Action, Feature, Principal, authorize_org_wide};
use console_platform_db::with_audits;
use sqlx::PgPool;
use time::OffsetDateTime;

use crate::{PayrollRunPage, PgPayrollError, list_runs_in_tx};

pub struct PgPayrollRunsReadPort {
    pool: PgPool,
    principal: Principal,
}

impl PgPayrollRunsReadPort {
    pub fn new(pool: PgPool, principal: Principal) -> Self {
        Self { pool, principal }
    }
}

impl PayrollRunsReadPort for PgPayrollRunsReadPort {
    fn authorize(&mut self) -> PayrollAuthorizeFuture<'_> {
        Box::pin(async move {
            authorize_org_wide(&self.principal, Action::new(Feature::PayrollRunRead))
                .map_err(PayrollRunsReadError::Authorization)
        })
    }

    fn read_page(&mut self, query: ListPayrollRuns) -> PayrollReadFuture<'_> {
        let org = self.principal.org_id;
        let actor = self.principal.user_id;
        Box::pin(async move {
            let result =
                with_audits::<_, PayrollRunPage, PgPayrollError>(&self.pool, org, move |tx| {
                    Box::pin(async move {
                        let page = list_runs_in_tx(tx, query.limit, query.offset).await?;
                        let event = AuditEvent::new(
                            Some(actor),
                            AuditAction::new("payroll_run.list_read")?,
                            "payroll_draft_run",
                            "query",
                            TraceContext::generate(),
                            OffsetDateTime::now_utc(),
                        )
                        .with_org(org);
                        Ok((page, vec![event]))
                    })
                })
                .await;
            // Preserve the established wire error contract. The generic store
            // conversion includes SQL diagnostics and must not cross this boundary.
            let page = result.map_err(|error| match error {
                PgPayrollError::Domain(error) => error,
                PgPayrollError::Db(error) => {
                    tracing::error!(error = %error, "payroll list read failed");
                    KernelError::internal("internal server error")
                }
            }).map_err(PayrollRunsReadError::Read)?;
            Ok(PayrollRunsReadResult { page, company: None })
        })
    }
}
