//! Serving admission over catalog metadata, without access to Account rows.
use crate::AppError;
use sqlx::PgPool;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum VerifiedCustodyProfile {
    NativeAccount,
    CompanyEnrollment,
    NativeCompanyPolicy,
}

pub(crate) async fn verify(pool: &PgPool) -> Result<VerifiedCustodyProfile, AppError> {
    let mut transaction = pool.begin().await?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
        .execute(&mut *transaction)
        .await?;
    // Catalog deparsing requires a fixed path; settings are local and
    // cannot escape on connection return, including failure/cancellation.
    sqlx::raw_sql(include_str!("account_custody_session.sql"))
        .execute(&mut *transaction)
        .await?;
    let policy: String =
        sqlx::query_scalar(include_str!("native_company_policy_custody_state.sql"))
            .fetch_one(&mut *transaction)
            .await?;
    if policy == "native_company_policy.finalized" {
        transaction.commit().await?;
        return Ok(VerifiedCustodyProfile::NativeCompanyPolicy);
    }
    if policy != "native_company_policy.absent" {
        return Err(AppError::Config(policy));
    }
    let company: String = sqlx::query_scalar(include_str!("company_enrollment_custody_state.sql"))
        .fetch_one(&mut *transaction)
        .await?;
    if company == "company_enrollment.finalized" {
        // This exact profile includes Account, credentials and Company custody.
        transaction.commit().await?;
        return Ok(VerifiedCustodyProfile::CompanyEnrollment);
    }
    if company != "company_enrollment.absent" {
        return Err(AppError::Config(company));
    }
    let state: String = sqlx::query_scalar(include_str!("account_custody_state.sql"))
        .fetch_one(&mut *transaction)
        .await?;
    let credentials: String =
        sqlx::query_scalar(include_str!("account_credential_custody_state.sql"))
            .fetch_one(&mut *transaction)
            .await?;
    transaction.commit().await?;
    if state == "account_custody.native_finalized"
        && credentials == "account_credentials.native_finalized"
    {
        Ok(VerifiedCustodyProfile::NativeAccount)
    } else if state == "account_custody.finalized"
        && matches!(
            credentials.as_str(),
            "account_credentials.pending" | "account_credentials.finalized"
        )
    {
        // Historical metadata is an operator input, not native-route readiness.
        Err(AppError::Config(
            "account_native.upgrade_required".to_owned(),
        ))
    } else if state != "account_custody.native_finalized" {
        Err(AppError::Config(state))
    } else {
        // Fixed coarse status only: no row/SQL diagnostics or tenant identifiers.
        Err(AppError::Config(credentials))
    }
}
