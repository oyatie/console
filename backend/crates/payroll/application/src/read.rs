//! Payroll collection read ownership; no persistence or transport dependency.
use console_kernel_core::{KernelError, OrgId};
use serde::Serialize;
use std::{future::Future, pin::Pin};
use time::{Date, OffsetDateTime};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct PayrollRunSummary {
    pub id: Uuid,
    pub period_start: Date,
    pub period_end: Date,
    pub source_label: String,
    pub status: String,
    pub calculation_enabled: bool,
    pub created_by: Option<Uuid>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<OffsetDateTime>,
    pub close_receipt: Option<serde_json::Value>,
    pub submitted_by: Option<Uuid>,
    pub submitted_at: Option<OffsetDateTime>,
    pub decided_by: Option<Uuid>,
    pub decided_at: Option<OffsetDateTime>,
    pub decision_reason: Option<String>,
    pub approval_ref: Option<Uuid>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize)]
pub struct PayrollRunPage {
    pub items: Vec<PayrollRunSummary>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// Pagination selectors only. Identity and authority are supplied by the server's port.
#[derive(Debug, Clone, Copy)]
pub struct ListPayrollRuns {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Committed scope accompanies the page without changing its public JSON shape.
#[derive(Debug)]
pub struct PayrollRunsReadResult {
    pub page: PayrollRunPage,
    pub company: Option<PayrollCompanyContext>,
}

#[derive(Debug)]
pub struct PayrollCompanyContext {
    pub id: OrgId,
    pub identity: Option<PayrollCompanyIdentity>,
}

#[derive(Debug)]
pub struct PayrollCompanyIdentity {
    pub name: String,
    pub slug: String,
}

pub type PayrollAuthorizeFuture<'a> =
    Pin<Box<dyn Future<Output = Result<(), PayrollRunsReadError>> + Send + 'a>>;
pub type PayrollReadFuture<'a> =
    Pin<Box<dyn Future<Output = Result<PayrollRunsReadResult, PayrollRunsReadError>> + Send + 'a>>;

pub trait PayrollRunsReadPort: Send {
    fn authorize(&mut self) -> PayrollAuthorizeFuture<'_>;
    fn read_page(&mut self, query: ListPayrollRuns) -> PayrollReadFuture<'_>;
}

/// A failed attempt to establish authority must never disclose a read failure.
#[derive(Debug)]
pub enum PayrollRunsReadError {
    Authorization(KernelError),
    Read(KernelError),
    AuthenticationInvalid,
    Unavailable,
}

pub async fn list_payroll_runs<P: PayrollRunsReadPort + ?Sized>(
    port: &mut P,
    query: ListPayrollRuns,
) -> Result<PayrollRunsReadResult, PayrollRunsReadError> {
    port.authorize().await?;
    port.read_page(query).await
}

#[cfg(test)]
#[path = "read_tests.rs"]
mod tests;
