//! Native Company reads through the existing identity store and Auth owner.
use super::PgOrgStore;
use console_identity_application::company_policy::{
    AccountId, CompanyContextCandidates, CompanyPolicyError, CompanyPolicyScope,
    CompanyPolicyStore, CompanyProjectionRow, CurrentCompanyAuthority,
};
use console_kernel_core::OrgId;
use console_platform_auth::{
    JwtIssuer, JwtVerifier,
    account::{AccountEnrollmentCredentials, AccountOperationError, account_now_in_tx},
};
use sqlx::{Postgres, Row, Transaction};
use time::Duration;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub(super) struct NativeAccountReadConfig {
    pub verifier: JwtVerifier,
    pub absolute_ttl: Duration,
    pub mode: NativeAccountMode,
}

#[derive(Clone)]
pub(super) enum NativeAccountMode {
    Initial,
    Policy { issuer: JwtIssuer },
}

impl std::fmt::Debug for NativeAccountMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Initial => "Initial",
            Self::Policy { .. } => "Policy { issuer: [REDACTED] }",
        })
    }
}

pub struct PgCompanyPolicyScope<'a> {
    tx: Transaction<'static, Postgres>,
    credentials: &'a AccountEnrollmentCredentials,
    config: &'a NativeAccountReadConfig,
    authority: Option<CurrentCompanyAuthority>,
}

impl CompanyPolicyScope for PgCompanyPolicyScope<'_> {
    fn authority(&self) -> Option<&CurrentCompanyAuthority> {
        self.authority.as_ref()
    }

    async fn finish(mut self) -> Result<(), CompanyPolicyError> {
        self.credentials
            .read_session_in_tx(
                &mut self.tx,
                &self.config.verifier,
                self.config.absolute_ttl,
            )
            .await
            .map_err(auth_error)?;
        self.tx.commit().await.map_err(sql_error)
    }
}

impl PgOrgStore {
    pub(super) fn native_read_config(
        &self,
    ) -> Result<&NativeAccountReadConfig, CompanyPolicyError> {
        self.native_account_read
            .as_ref()
            .filter(|c| c.absolute_ttl > Duration::ZERO)
            .ok_or(CompanyPolicyError::MaterialUnavailable)
    }

    pub(super) async fn native_read_transaction(
        &self,
    ) -> Result<Transaction<'static, Postgres>, CompanyPolicyError> {
        let mut tx = self.pool.begin().await.map_err(sql_error)?;
        sqlx::raw_sql(
            "SET TRANSACTION ISOLATION LEVEL READ COMMITTED; \
             SET LOCAL statement_timeout='10s'; SET LOCAL lock_timeout='1s'",
        )
        .execute(tx.as_mut())
        .await
        .map_err(sql_error)?;
        Ok(tx)
    }
}

impl CompanyPolicyStore for PgOrgStore {
    type Credentials = AccountEnrollmentCredentials;
    type Scope<'a> = PgCompanyPolicyScope<'a>;

    async fn lock_current<'a>(
        &'a self,
        credentials: &'a Self::Credentials,
        company: OrgId,
    ) -> Result<Self::Scope<'a>, CompanyPolicyError> {
        let config = self.native_read_config()?;
        if company.as_uuid().is_nil() || company == OrgId::platform() {
            return Err(CompanyPolicyError::MaterialUnavailable);
        }
        for attempt in 0..2 {
            let mut tx = self.native_read_transaction().await?;
            // Cryptographic namespace only. Acquiring Account guards here would
            // invert the SQL owner's Group -> Account/family -> Company order.
            let (account, family) = credentials
                .session_ids_in_tx(&mut tx, &config.verifier, config.absolute_ttl)
                .await
                .map_err(auth_error)?;
            let query = match &config.mode {
                NativeAccountMode::Initial => {
                    "SELECT company_epoch,context_generation,assignment_id,assignment_revision,\
                     role_id,role_revision,registered_clauses::text AS registered_clauses,\
                     company_name,company_slug \
                     FROM public.identity_company_projection_v1($1,$2,$3) LIMIT 2"
                }
                NativeAccountMode::Policy { .. } => {
                    "SELECT company_epoch,context_generation,assignment_id,assignment_revision,\
                     role_id,role_revision,registered_clauses::text AS registered_clauses,\
                     company_name,company_slug,current_policy_receipt_id \
                     FROM public.identity_company_projection_v2($1,$2,$3) LIMIT 2"
                }
            };
            let rows = sqlx::query(sqlx::AssertSqlSafe(query))
                .bind(account)
                .bind(family)
                .bind(*company.as_uuid())
                .fetch_all(tx.as_mut())
                .await;
            let rows = match rows {
                Ok(rows) => rows,
                Err(error) => {
                    let error = sql_error(error);
                    tx.rollback().await.map_err(sql_error)?;
                    if error == CompanyPolicyError::Conflict && attempt == 0 {
                        continue;
                    }
                    return Err(error);
                }
            };
            // Zero rows also need current authenticated authority; signed IDs
            // and a healthy absent Company source cannot prove a live session.
            let session = credentials
                .read_session_in_tx(&mut tx, &config.verifier, config.absolute_ttl)
                .await
                .map_err(auth_error)?;
            if (session.account_id, session.session_id) != (account, family) || rows.len() > 1 {
                return Err(CompanyPolicyError::MaterialUnavailable);
            }
            let authority = if let Some(row) = rows.first() {
                let observed_at = account_now_in_tx(&mut tx).await.map_err(auth_error)?;
                let projection = CompanyProjectionRow {
                    company_epoch: row.try_get("company_epoch").map_err(sql_error)?,
                    context_generation: row.try_get("context_generation").map_err(sql_error)?,
                    assignment_id: row.try_get("assignment_id").map_err(sql_error)?,
                    assignment_revision: row.try_get("assignment_revision").map_err(sql_error)?,
                    role_id: row.try_get("role_id").map_err(sql_error)?,
                    role_revision: row.try_get("role_revision").map_err(sql_error)?,
                    registered_clauses: row.try_get("registered_clauses").map_err(sql_error)?,
                    company_name: row.try_get("company_name").map_err(sql_error)?,
                    company_slug: row.try_get("company_slug").map_err(sql_error)?,
                };
                let account = AccountId::from_uuid(account)
                    .map_err(|_| CompanyPolicyError::MaterialUnavailable)?;
                Some(match &config.mode {
                    NativeAccountMode::Initial => CurrentCompanyAuthority::from_initial_projection(
                        account,
                        company,
                        observed_at,
                        projection,
                    )?,
                    NativeAccountMode::Policy { .. } => {
                        CurrentCompanyAuthority::from_current_projection(
                            account,
                            company,
                            observed_at,
                            projection,
                            row.try_get("current_policy_receipt_id")
                                .map_err(sql_error)?,
                        )?
                    }
                })
            } else {
                None
            };
            return Ok(PgCompanyPolicyScope {
                tx,
                credentials,
                config,
                authority,
            });
        }
        Err(CompanyPolicyError::Conflict)
    }

    async fn enumerate_company_candidates(
        &self,
        credentials: &Self::Credentials,
    ) -> Result<CompanyContextCandidates, CompanyPolicyError> {
        let config = self.native_read_config()?;
        let mut tx = self.native_read_transaction().await?;
        let session = credentials
            .read_session_in_tx(&mut tx, &config.verifier, config.absolute_ttl)
            .await
            .map_err(auth_error)?;
        let rows = sqlx::query(
            "SELECT context_generation,candidate_org_ids \
             FROM public.account_company_context_candidates_v1($1,$2) LIMIT 2",
        )
        .bind(session.account_id)
        .bind(session.session_id)
        .fetch_all(tx.as_mut())
        .await
        .map_err(sql_error)?;
        let [row] = rows.as_slice() else {
            return Err(CompanyPolicyError::MaterialUnavailable);
        };
        let generation: i64 = row.try_get("context_generation").map_err(sql_error)?;
        let ids: Vec<Uuid> = row.try_get("candidate_org_ids").map_err(sql_error)?;
        let candidates = CompanyContextCandidates::new(
            u64::try_from(generation).map_err(|_| CompanyPolicyError::MaterialUnavailable)?,
            ids.into_iter().map(OrgId::from_uuid).collect(),
        )?;
        credentials
            .read_session_in_tx(&mut tx, &config.verifier, config.absolute_ttl)
            .await
            .map_err(auth_error)?;
        // No Account guard escapes enumeration into the later Group scopes.
        tx.commit().await.map_err(sql_error)?;
        Ok(candidates)
    }

    async fn check_context_generation(
        &self,
        credentials: &Self::Credentials,
        expected_generation: u64,
    ) -> Result<(), CompanyPolicyError> {
        let current = self.enumerate_company_candidates(credentials).await?;
        if current.generation() == expected_generation {
            Ok(())
        } else {
            Err(CompanyPolicyError::Conflict)
        }
    }
}

fn auth_error(error: AccountOperationError) -> CompanyPolicyError {
    match error {
        AccountOperationError::AuthenticationInvalid => CompanyPolicyError::AuthenticationInvalid,
        _ => CompanyPolicyError::MaterialUnavailable,
    }
}

fn sql_error(error: sqlx::Error) -> CompanyPolicyError {
    match error.as_database_error() {
        Some(db) if db.code().as_deref() == Some("40001") => CompanyPolicyError::Conflict,
        Some(db)
            if db.code().as_deref() == Some("P0001")
                && db.message() == "account.authentication_invalid" =>
        {
            CompanyPolicyError::AuthenticationInvalid
        }
        _ => CompanyPolicyError::MaterialUnavailable,
    }
}
