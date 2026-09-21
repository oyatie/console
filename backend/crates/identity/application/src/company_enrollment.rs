//! Durable Company intake through the owning transaction, before Company birth.
use crate::CompanyEnrollmentV1;
use console_kernel_core::{OrgId, TraceContext};
use std::future::Future;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompanyEnrollmentError {
    InvalidInput,
    AuthenticationInvalid,
    CsrfInvalid,
    Forbidden,
    Conflict,
    Capacity,
    GroupUnavailable,
    Unavailable,
    /// Commit was not confirmed. Reconcile the same command before retrying.
    Unconfirmed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompanyEnrollmentStatus {
    Missing,
    Pending(CompanyEnrollmentV1),
    Cancelled,
    Expired,
    Committed {
        receipt_id: Uuid,
        org_id: Uuid,
        group_id: Uuid,
        administrative_account_id: Uuid,
    },
}

/// Confirmed durable result. The owning adapter releases it only after commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompanyEnrollmentReceipt {
    pub receipt_id: Uuid,
    pub org_id: Uuid,
    pub group_id: Uuid,
    pub administrative_account_id: Uuid,
    pub replayed: bool,
}

/// Untrusted persistence projection; construction grants no authority.
pub struct CompanyEnrollmentProjection {
    pub state: String,
    pub codec_version: i16,
    pub input_bytes: Option<Vec<u8>>,
    pub receipt_id: Option<Uuid>,
    pub org_id: Option<Uuid>,
    pub group_id: Option<Uuid>,
    pub administrative_account_id: Option<Uuid>,
}

impl CompanyEnrollmentStatus {
    pub fn from_projection(
        account: Uuid,
        command: Uuid,
        row: CompanyEnrollmentProjection,
    ) -> Result<Self, CompanyEnrollmentError> {
        let invalid = CompanyEnrollmentError::Unavailable;
        if account.is_nil() || command.is_nil() || row.codec_version != 1 {
            return Err(invalid);
        }
        let ids = [
            row.receipt_id,
            row.org_id,
            row.group_id,
            row.administrative_account_id,
        ];
        if row.state == "COMMITTED" {
            let [
                Some(receipt_id),
                Some(org_id),
                Some(group_id),
                Some(administrative_account_id),
            ] = ids
            else {
                return Err(invalid);
            };
            if row.input_bytes.is_some()
                || ids.iter().flatten().any(Uuid::is_nil)
                || org_id == *OrgId::platform().as_uuid()
            {
                return Err(invalid);
            }
            return Ok(Self::Committed {
                receipt_id,
                org_id,
                group_id,
                administrative_account_id,
            });
        }
        if ids.iter().any(Option::is_some) {
            return Err(invalid);
        }
        match (row.state.as_str(), row.input_bytes) {
            ("PENDING", Some(bytes)) => {
                let (owner, input) = CompanyEnrollmentV1::decode(&bytes).map_err(|_| invalid)?;
                if owner != account || input.command_id() != command {
                    return Err(invalid);
                }
                Ok(Self::Pending(input))
            }
            ("CANCELLED", None) => Ok(Self::Cancelled),
            ("EXPIRED", None) => Ok(Self::Expired),
            _ => Err(invalid),
        }
    }
}

/// Each operation authenticates and retains its owner transaction through final
/// freshness validation and commit. Errors never release a provisional view.
pub trait CompanyEnrollmentStore {
    type Credentials: Sync;

    fn prepare(
        &self,
        credentials: &Self::Credentials,
        input: &CompanyEnrollmentV1,
    ) -> impl Future<Output = Result<CompanyEnrollmentStatus, CompanyEnrollmentError>> + Send;

    fn status(
        &self,
        credentials: &Self::Credentials,
        command: Uuid,
    ) -> impl Future<Output = Result<CompanyEnrollmentStatus, CompanyEnrollmentError>> + Send;

    fn cancel(
        &self,
        credentials: &Self::Credentials,
        command: Uuid,
    ) -> impl Future<Output = Result<CompanyEnrollmentStatus, CompanyEnrollmentError>> + Send;

    fn execute(
        &self,
        credentials: &Self::Credentials,
        input: &CompanyEnrollmentV1,
        trace: &TraceContext,
    ) -> impl Future<Output = Result<CompanyEnrollmentReceipt, CompanyEnrollmentError>> + Send;
}

/// Keep intake durable before the effect transaction. Recovery always uses the
/// original command and bytes, including after an unconfirmed effect commit.
pub async fn enroll_company<S: CompanyEnrollmentStore>(
    store: &S,
    credentials: &S::Credentials,
    input: &[u8],
    trace: &TraceContext,
) -> Result<CompanyEnrollmentReceipt, CompanyEnrollmentError> {
    let input = CompanyEnrollmentV1::from_json_slice(input)
        .map_err(|_| CompanyEnrollmentError::InvalidInput)?;
    if input.group_id().is_some() {
        return Err(CompanyEnrollmentError::GroupUnavailable);
    }
    store.prepare(credentials, &input).await?;
    store.execute(credentials, &input, trace).await
}

pub async fn prepare_company_enrollment<S: CompanyEnrollmentStore>(
    store: &S,
    credentials: &S::Credentials,
    input: &[u8],
) -> Result<CompanyEnrollmentStatus, CompanyEnrollmentError> {
    let input = CompanyEnrollmentV1::from_json_slice(input)
        .map_err(|_| CompanyEnrollmentError::InvalidInput)?;
    store.prepare(credentials, &input).await
}

pub async fn company_enrollment_status<S: CompanyEnrollmentStore>(
    store: &S,
    credentials: &S::Credentials,
    command: Uuid,
) -> Result<CompanyEnrollmentStatus, CompanyEnrollmentError> {
    if command.is_nil() {
        return Err(CompanyEnrollmentError::InvalidInput);
    }
    store.status(credentials, command).await
}

pub async fn cancel_company_enrollment<S: CompanyEnrollmentStore>(
    store: &S,
    credentials: &S::Credentials,
    command: Uuid,
) -> Result<CompanyEnrollmentStatus, CompanyEnrollmentError> {
    if command.is_nil() {
        return Err(CompanyEnrollmentError::InvalidInput);
    }
    store.cancel(credentials, command).await
}

#[cfg(test)]
#[path = "company/enrollment_projection_tests.rs"]
mod enrollment_projection_tests;
