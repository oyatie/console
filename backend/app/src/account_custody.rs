//! Serving admission over catalog metadata, without access to Account rows.
use crate::AppError;
use sqlx::PgPool;

pub(crate) async fn verify(pool: &PgPool) -> Result<(), AppError> {
    let mut transaction = pool.begin().await?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
        .execute(&mut *transaction)
        .await?;
    // Explicit pg_temp last prevents pooled temporary catalogs shadowing real
    // metadata; catalog deparsing also requires this path. Both settings are
    // transaction-local, including on error/cancellation, and cannot escape a
    // returned pool connection. Bound metadata work separately from app work.
    sqlx::raw_sql("SET LOCAL search_path=pg_catalog,pg_temp; SET LOCAL statement_timeout='3s'")
        .execute(&mut *transaction)
        .await?;
    let state: String = sqlx::query_scalar(include_str!("account_custody_state.sql"))
        .fetch_one(&mut *transaction)
        .await?;
    let credentials: String =
        sqlx::query_scalar(include_str!("account_credential_custody_state.sql"))
            .fetch_one(&mut *transaction)
            .await?;
    transaction.commit().await?;
    if state != "account_custody.finalized" {
        Err(AppError::Config(state))
    } else if credentials == "account_credentials.finalized" {
        Ok(())
    } else {
        // The canonical query returns only fixed coarse status codes, never
        // row contents, SQL diagnostics or another tenant's identifiers.
        Err(AppError::Config(credentials))
    }
}
