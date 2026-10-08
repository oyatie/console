//! Serving admission over catalog metadata, without access to Account rows.
use crate::AppError;
use sqlx::PgPool;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum VerifiedCustodyProfile {
    NativeAccount,
    CompanyEnrollment,
    NativeCompanyPolicy,
    NativeCompanyInformationManagerCurrentPolicyV1,
    NativeCompanyPolicyV2,
    // Verified predecessor substrate; Directory needs the row-lock correction.
    NativePeopleDirectory,
    NativePeopleDirectoryRowLock,
    // Exact Directory predecessor or classifier-only successor; OrgUnit closed.
    NativeOrgBridgeCompatible,
    NativeGroupProcess,
    NativeGroupProcessNavigation,
}

impl VerifiedCustodyProfile {
    pub(crate) fn supports_company_information_manager_current(self) -> bool {
        matches!(self, Self::NativeCompanyInformationManagerCurrentPolicyV1)
    }
    pub(crate) fn supports_native_group_process(self) -> bool {
        matches!(
            self,
            Self::NativeGroupProcess | Self::NativeGroupProcessNavigation
        )
    }
    pub(crate) fn supports_native_directory(self) -> bool {
        matches!(
            self,
            Self::NativePeopleDirectoryRowLock
                | Self::NativeOrgBridgeCompatible
                | Self::NativeGroupProcess
                | Self::NativeGroupProcessNavigation
        )
    }
    pub(crate) fn requires_current_company_provenance(self) -> bool {
        matches!(
            self,
            Self::NativePeopleDirectoryRowLock
                | Self::NativeOrgBridgeCompatible
                | Self::NativeGroupProcess
                | Self::NativeGroupProcessNavigation
        )
    }
    pub(crate) fn supports_policy(self) -> bool {
        matches!(
            self,
            Self::NativeCompanyPolicy
                | Self::NativeCompanyPolicyV2
                | Self::NativePeopleDirectory
                | Self::NativePeopleDirectoryRowLock
                | Self::NativeOrgBridgeCompatible
                | Self::NativeGroupProcess
                | Self::NativeGroupProcessNavigation
        ) || self.supports_company_information_manager_current()
    }
    pub(crate) fn supports_people(self) -> bool {
        matches!(
            self,
            Self::NativeCompanyPolicyV2
                | Self::NativePeopleDirectory
                | Self::NativePeopleDirectoryRowLock
                | Self::NativeOrgBridgeCompatible
                | Self::NativeGroupProcess
                | Self::NativeGroupProcessNavigation
        )
    }
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
    let manager: String = sqlx::query_scalar(include_str!(
        "company_information_manager_current_policy_v1_custody_state.sql"
    ))
    .fetch_one(&mut *transaction)
    .await?;
    if manager == "company_information_manager_current_policy_v1.finalized" {
        transaction.commit().await?;
        return Ok(VerifiedCustodyProfile::NativeCompanyInformationManagerCurrentPolicyV1);
    }
    if !matches!(
        manager.as_str(),
        "company_information_manager_current_policy_v1.absent"
            | "company_information_manager_current_policy_v1.install_required"
    ) {
        return Err(AppError::Config(manager));
    }
    let navigation: String = sqlx::query_scalar(include_str!(
        "native_group_process_navigation_serving_v1_custody_state.sql"
    ))
    .fetch_one(&mut *transaction)
    .await?;
    if navigation == "native_group_process_navigation.finalized" {
        transaction.commit().await?;
        return Ok(VerifiedCustodyProfile::NativeGroupProcessNavigation);
    }
    let group: String =
        sqlx::query_scalar(include_str!("native_group_process_v1_custody_state.sql"))
            .fetch_one(&mut *transaction)
            .await?;
    if group == "native_group_process.finalized" {
        transaction.commit().await?;
        return Ok(VerifiedCustodyProfile::NativeGroupProcess);
    }
    if !matches!(
        group.as_str(),
        "native_group_process.absent" | "native_group_process.install_required"
    ) {
        return Err(AppError::Config(group));
    }
    let provenance: String =
        sqlx::query_scalar(include_str!("company_provenance_v1_custody_state.sql"))
            .fetch_one(&mut *transaction)
            .await?;
    if matches!(
        provenance.as_str(),
        "company_provenance.predecessor_compatible" | "company_provenance.installed_compatible"
    ) {
        transaction.commit().await?;
        return Ok(VerifiedCustodyProfile::NativeOrgBridgeCompatible);
    }
    if provenance != "company_provenance.absent" {
        let closed: String = sqlx::query_scalar(include_str!(
            "native_org_unit_closed_perimeter_v1_custody_state.sql"
        ))
        .fetch_one(&mut *transaction)
        .await?;
        if closed == "native_org_unit.closed_perimeter_compatible" {
            transaction.commit().await?;
            return Ok(VerifiedCustodyProfile::NativeOrgBridgeCompatible);
        }
        return Err(AppError::Config(provenance));
    }
    let directory: String = sqlx::query_scalar(include_str!(
        "native_people_directory_row_lock_custody_state.sql"
    ))
    .fetch_one(&mut *transaction)
    .await?;
    if directory == "native_people_directory.finalized" {
        transaction.commit().await?;
        return Ok(VerifiedCustodyProfile::NativePeopleDirectoryRowLock);
    }
    if directory == "native_people_directory.row_lock_required" {
        // Exact predecessor still supports Account/Company/policy. Composition
        // withholds Directory owner/navigation until the corrected profile.
        transaction.commit().await?;
        return Ok(VerifiedCustodyProfile::NativePeopleDirectory);
    }
    if !matches!(
        directory.as_str(),
        "native_people_directory.absent" | "native_people_directory.staged_closed"
    ) {
        return Err(AppError::Config(directory));
    }
    // Exact closed230 metadata permits only the independently checked historical
    // profile below. It never enables native directory operations.
    let successor: String =
        sqlx::query_scalar(include_str!("native_company_policy_v2_custody_state.sql"))
            .fetch_one(&mut *transaction)
            .await?;
    if successor == "native_company_policy_v2.finalized" {
        transaction.commit().await?;
        return Ok(VerifiedCustodyProfile::NativeCompanyPolicyV2);
    }
    if successor != "native_company_policy_v2.absent" {
        return Err(AppError::Config(successor));
    }
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#[path = "account_custody_manager_policy_v1_tests.rs"]
mod manager_policy_v1_tests;
