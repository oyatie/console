//! Reuse the installed Account/Company intake owners on the Business pool.
use super::PgOrgStore;
use console_identity_application::{
    CompanyEnrollmentV1,
    company_enrollment::{
        CompanyEnrollmentError as Error, CompanyEnrollmentProjection, CompanyEnrollmentReceipt,
        CompanyEnrollmentStatus as Status, CompanyEnrollmentStore,
    },
};
use console_kernel_core::TraceContext;
use console_platform_auth::account::{
    AccountEnrollmentCredentials, AccountOperationError, LockedAccountEnrollment,
};
use sqlx::{Postgres, Row, Transaction};
use uuid::Uuid;

impl CompanyEnrollmentStore for PgOrgStore {
    type Credentials = AccountEnrollmentCredentials;

    async fn execute(
        &self,
        credentials: &Self::Credentials,
        input: &CompanyEnrollmentV1,
        trace: &TraceContext,
    ) -> Result<CompanyEnrollmentReceipt, Error> {
        let config = self.native_read_config().map_err(|_| Error::Unavailable)?;
        let mut tx = self
            .native_read_transaction()
            .await
            .map_err(|_| Error::Unavailable)?;
        let account = credentials
            .account_id_in_tx(&mut tx, &config.verifier, config.absolute_ttl)
            .await
            .map_err(auth_error)?;
        let bytes = input.encode(account).map_err(|_| Error::InvalidInput)?;
        let command = input.command_id();
        let mut guard = credentials
            .lock_submit_in_tx(
                &mut tx,
                &config.verifier,
                config.absolute_ttl,
                command,
                &bytes,
            )
            .await
            .map_err(auth_error)?;
        // Hash the exact actor-bound bytes supplied to this operation. A prior
        // prepare is not authority to silently execute a different request.
        let digest: Vec<u8> = sqlx::query_scalar("SELECT pg_catalog.sha256($1::bytea)")
            .bind(&bytes)
            .fetch_one(guard.connection())
            .await
            .map_err(sql_error)?;
        if guard.planned_input_digest() != Some(digest.as_slice()) {
            return Err(Error::Conflict);
        }
        let rows = sqlx::query(
            "SELECT receipt_id,org_id,group_id,administrative_account_id,replayed \
             FROM public.company_enrollment_execute_v1($1,$2,$3,$4,$5,$6) LIMIT 2",
        )
        .bind(guard.account_id())
        .bind(guard.session_id())
        .bind(command)
        .bind(digest)
        .bind(trace.trace_id())
        .bind(trace.span_id())
        .fetch_all(guard.connection())
        .await
        .map_err(sql_error)?;
        let [row] = rows.as_slice() else {
            return Err(Error::Unavailable);
        };
        let receipt = CompanyEnrollmentReceipt {
            receipt_id: row.try_get("receipt_id").map_err(sql_error)?,
            org_id: row.try_get("org_id").map_err(sql_error)?,
            group_id: row.try_get("group_id").map_err(sql_error)?,
            administrative_account_id: row
                .try_get("administrative_account_id")
                .map_err(sql_error)?,
            replayed: row.try_get("replayed").map_err(sql_error)?,
        };
        if receipt.administrative_account_id != input.administrative_account_id()
            || read_status(&mut guard, command).await?
                != (Status::Committed {
                    receipt_id: receipt.receipt_id,
                    org_id: receipt.org_id,
                    group_id: receipt.group_id,
                    administrative_account_id: receipt.administrative_account_id,
                })
        {
            return Err(Error::Unavailable);
        }
        guard.finish().await.map_err(auth_error)?;
        commit(tx).await?;
        Ok(receipt)
    }

    async fn prepare(
        &self,
        credentials: &Self::Credentials,
        input: &CompanyEnrollmentV1,
    ) -> Result<Status, Error> {
        let config = self.native_read_config().map_err(|_| Error::Unavailable)?;
        let mut tx = self
            .native_read_transaction()
            .await
            .map_err(|_| Error::Unavailable)?;
        let account = credentials
            .account_id_in_tx(&mut tx, &config.verifier, config.absolute_ttl)
            .await
            .map_err(auth_error)?;
        let bytes = input.encode(account).map_err(|_| Error::InvalidInput)?;
        let command = input.command_id();
        let mut guard = credentials
            .lock_submit_in_tx(
                &mut tx,
                &config.verifier,
                config.absolute_ttl,
                command,
                &bytes,
            )
            .await
            .map_err(auth_error)?;
        let rows: Vec<(String, Option<Uuid>)> = sqlx::query_as(
            "SELECT state,receipt_id FROM public.company_enrollment_prepare_v1($1,$2,$3,$4) LIMIT 2")
            .bind(guard.account_id()).bind(guard.session_id()).bind(command).bind(bytes)
            .fetch_all(guard.connection()).await.map_err(sql_error)?;
        let [(state, receipt)] = rows.as_slice() else {
            return Err(Error::Unavailable);
        };
        let status = read_status(&mut guard, command).await?;
        if !matches_outcome(&status, state, *receipt) {
            return Err(Error::Unavailable);
        }
        guard.finish().await.map_err(auth_error)?;
        commit(tx).await?;
        Ok(status)
    }

    async fn status(
        &self,
        credentials: &Self::Credentials,
        command: Uuid,
    ) -> Result<Status, Error> {
        let config = self.native_read_config().map_err(|_| Error::Unavailable)?;
        let mut tx = self
            .native_read_transaction()
            .await
            .map_err(|_| Error::Unavailable)?;
        let mut guard = credentials
            .lock_status_in_tx(&mut tx, &config.verifier, config.absolute_ttl, command)
            .await
            .map_err(auth_error)?;
        let status = read_status(&mut guard, command).await?;
        guard.finish().await.map_err(auth_error)?;
        commit(tx).await?;
        Ok(status)
    }

    async fn cancel(
        &self,
        credentials: &Self::Credentials,
        command: Uuid,
    ) -> Result<Status, Error> {
        let config = self.native_read_config().map_err(|_| Error::Unavailable)?;
        let mut tx = self
            .native_read_transaction()
            .await
            .map_err(|_| Error::Unavailable)?;
        let mut guard = credentials
            .lock_cancel_in_tx(&mut tx, &config.verifier, config.absolute_ttl, command)
            .await
            .map_err(auth_error)?;
        let rows = sqlx::query(
            "SELECT state,receipt_id,org_id,group_id,administrative_account_id \
            FROM public.company_enrollment_cancel_v1($1,$2,$3) LIMIT 2",
        )
        .bind(guard.account_id())
        .bind(guard.session_id())
        .bind(command)
        .fetch_all(guard.connection())
        .await
        .map_err(sql_error)?;
        let expected = match rows.as_slice() {
            [] => Status::Missing,
            [row] => Status::from_projection(
                guard.account_id(),
                command,
                CompanyEnrollmentProjection {
                    state: row.try_get("state").map_err(sql_error)?,
                    codec_version: 1,
                    input_bytes: None,
                    receipt_id: row.try_get("receipt_id").map_err(sql_error)?,
                    org_id: row.try_get("org_id").map_err(sql_error)?,
                    group_id: row.try_get("group_id").map_err(sql_error)?,
                    administrative_account_id: row
                        .try_get("administrative_account_id")
                        .map_err(sql_error)?,
                },
            )?,
            _ => return Err(Error::Unavailable),
        };
        let status = read_status(&mut guard, command).await?;
        if status != expected {
            return Err(Error::Unavailable);
        }
        guard.finish().await.map_err(auth_error)?;
        commit(tx).await?;
        Ok(status)
    }
}

async fn read_status(
    guard: &mut LockedAccountEnrollment<'_, '_>,
    command: Uuid,
) -> Result<Status, Error> {
    let rows = sqlx::query("SELECT state,codec_version,input_bytes,receipt_id,org_id,group_id,administrative_account_id \
        FROM public.company_enrollment_status_v1($1,$2,$3) LIMIT 2")
        .bind(guard.account_id()).bind(guard.session_id()).bind(command)
        .fetch_all(guard.connection()).await.map_err(sql_error)?;
    match rows.as_slice() {
        [] => Ok(Status::Missing),
        [row] => Status::from_projection(
            guard.account_id(),
            command,
            CompanyEnrollmentProjection {
                state: row.try_get("state").map_err(sql_error)?,
                codec_version: row.try_get("codec_version").map_err(sql_error)?,
                input_bytes: row.try_get("input_bytes").map_err(sql_error)?,
                receipt_id: row.try_get("receipt_id").map_err(sql_error)?,
                org_id: row.try_get("org_id").map_err(sql_error)?,
                group_id: row.try_get("group_id").map_err(sql_error)?,
                administrative_account_id: row
                    .try_get("administrative_account_id")
                    .map_err(sql_error)?,
            },
        ),
        _ => Err(Error::Unavailable),
    }
}

fn matches_outcome(status: &Status, state: &str, receipt: Option<Uuid>) -> bool {
    match status {
        Status::Pending(_) => state == "PENDING" && receipt.is_none(),
        Status::Cancelled => state == "CANCELLED" && receipt.is_none(),
        // The status owner can expire a request while reopening it.
        Status::Expired => matches!(state, "PENDING" | "EXPIRED") && receipt.is_none(),
        Status::Committed { receipt_id, .. } => {
            state == "COMMITTED" && receipt == Some(*receipt_id)
        }
        Status::Missing => false,
    }
}

async fn commit(tx: Transaction<'_, Postgres>) -> Result<(), Error> {
    tx.commit().await.map_err(|error| {
        match error
            .as_database_error()
            .map(|db| (db.code(), db.message()))
        {
            // Explicit server refusals known to abort this transaction. A lost
            // response or statement-completion-unknown is never called a failure.
            Some((Some(code), _))
                if matches!(
                    code.as_ref(),
                    "23502" | "23503" | "23505" | "23514" | "23P01" | "40001" | "40P01"
                ) =>
            {
                Error::Unavailable
            }
            Some((Some(code), message))
                if code == "P0001"
                    && matches!(
                        message,
                        "company_enrollment.intake_closure_invalid"
                            | "company_enrollment.effect_unavailable"
                            | "company_enrollment.guard_context_invalid"
                    ) =>
            {
                Error::Unavailable
            }
            _ => Error::Unconfirmed,
        }
    })
}

fn auth_error(error: AccountOperationError) -> Error {
    match error {
        AccountOperationError::AuthenticationInvalid => Error::AuthenticationInvalid,
        AccountOperationError::CsrfInvalid => Error::CsrfInvalid,
        AccountOperationError::TermsAcceptanceRequired | AccountOperationError::TermsChanged => {
            Error::Forbidden
        }
        _ => Error::Unavailable,
    }
}

fn sql_error(error: sqlx::Error) -> Error {
    let Some(db) = error.as_database_error() else {
        return Error::Unavailable;
    };
    match (db.code().as_deref(), db.message()) {
        (Some("P0001"), "account.authentication_invalid") => Error::AuthenticationInvalid,
        (Some("P0001"), "account.terms_acceptance_required" | "account.terms_changed") => {
            Error::Forbidden
        }
        (Some("P0001"), "company_enrollment.forbidden") => Error::Forbidden,
        (Some("P0001"), "company_enrollment.group_unavailable") => Error::GroupUnavailable,
        (Some("P0001"), "company_enrollment.conflict") => Error::Conflict,
        (Some("P0001"), "company_enrollment.capacity") => Error::Capacity,
        _ => Error::Unavailable,
    }
}
