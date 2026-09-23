//! Native Account reads retain current Company authority through the audited query.
use console_identity_application::company_policy::{
    AccountId, CompanyPolicyDecision, CompanyPolicyDecisionPort, CompanyPolicyError,
    CompanyProjectionRow, CurrentCompanyAuthority, CurrentPayrollReadAuthority,
    NativePayrollReadProjectionRow, NativePolicySourceBinding, decode_native_payroll_read_clause,
    project_company_identity,
};
use console_kernel_core::{AuditAction, AuditEvent, KernelError, OrgId, TraceContext};
use console_payroll_application::read::{
    ListPayrollRuns, PayrollAuthorizeFuture, PayrollCompanyContext, PayrollCompanyIdentity,
    PayrollNavigationFuture, PayrollReadFuture, PayrollRunsReadError as Error, PayrollRunsReadPort,
    PayrollRunsReadResult,
};
use console_platform_auth::{
    JwtVerifier,
    account::{AccountEnrollmentCredentials, AccountOperationError, account_now_in_tx},
};
use console_platform_db::insert_audit_event;
use sqlx::{PgPool, Postgres, Transaction};
use std::sync::Arc;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

pub struct PgNativePayrollRunsReadPort {
    pool: PgPool,
    verifier: JwtVerifier,
    absolute_ttl: Duration,
    policy: Arc<dyn CompanyPolicyDecisionPort>,
    credentials: AccountEnrollmentCredentials,
    company: OrgId,
    state: State,
}

enum State {
    Unused,
    Authorized(Box<Retained>),
    Consumed,
}
struct Retained {
    tx: Transaction<'static, Postgres>,
    source: Source,
    identity_allowed: bool,
}
struct Source {
    payroll: PayrollSource,
    identity: IdentitySource,
    binding: NativePolicySourceBinding,
}

// Exact canonical owner projections. Names in these private rows are material,
// not disclosure authority; only the separate Company decision projects them.
#[derive(sqlx::FromRow, PartialEq, Eq)]
struct PayrollSource {
    company_epoch: i64,
    current_policy_receipt_id: Uuid,
    assignment_id: Uuid,
    assignment_revision: i64,
    role_id: Uuid,
    role_revision: i64,
    registered_clauses: String,
    company_name: String,
    company_slug: String,
    assignment_valid_from: OffsetDateTime,
    assignment_valid_until: OffsetDateTime,
}
#[derive(sqlx::FromRow, PartialEq, Eq)]
struct IdentitySource {
    company_epoch: i64,
    context_generation: i64,
    assignment_id: Uuid,
    assignment_revision: i64,
    role_id: Uuid,
    role_revision: i64,
    registered_clauses: String,
    company_name: String,
    company_slug: String,
    current_policy_receipt_id: Option<Uuid>,
}

impl PgNativePayrollRunsReadPort {
    pub fn new(
        pool: PgPool,
        verifier: JwtVerifier,
        absolute_ttl: Duration,
        policy: Arc<dyn CompanyPolicyDecisionPort>,
        credentials: AccountEnrollmentCredentials,
        company: OrgId,
    ) -> Self {
        Self {
            pool,
            verifier,
            absolute_ttl,
            policy,
            credentials,
            company,
            state: State::Unused,
        }
    }

    async fn source(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        account: Uuid,
        family: Uuid,
    ) -> Result<Source, Error> {
        // This canonical owner takes Group before Account/family/Company guards.
        let mut rows = sqlx::query_as::<_, PayrollSource>(
            "SELECT company_epoch,current_policy_receipt_id,assignment_id,assignment_revision,\
             role_id,role_revision,registered_clauses::text AS registered_clauses,company_name,\
             company_slug,assignment_valid_from,assignment_valid_until \
             FROM public.identity_company_payroll_projection_v1($1,$2,$3) LIMIT 2",
        )
        .bind(account)
        .bind(family)
        .bind(*self.company.as_uuid())
        .fetch_all(tx.as_mut())
        .await
        .map_err(projection_error)?;
        // Even absent source requires a current live session, not just signed IDs.
        let session = self
            .credentials
            .read_session_in_tx(tx, &self.verifier, self.absolute_ttl)
            .await
            .map_err(auth_error)?;
        if (session.account_id, session.session_id) != (account, family) || rows.len() > 1 {
            return Err(Error::Unavailable);
        }
        let payroll = rows.pop().ok_or_else(denied)?;
        let mut rows = sqlx::query_as::<_, IdentitySource>(
            "SELECT company_epoch,context_generation,assignment_id,assignment_revision,role_id,\
             role_revision,registered_clauses::text AS registered_clauses,company_name,company_slug,\
             current_policy_receipt_id FROM public.identity_company_projection_v2($1,$2,$3) LIMIT 2",
        ).bind(account).bind(family).bind(*self.company.as_uuid())
            .fetch_all(tx.as_mut()).await.map_err(projection_error)?;
        if rows.len() != 1 {
            return Err(Error::Unavailable);
        }
        let identity = rows.pop().ok_or(Error::Unavailable)?;
        if identity.company_epoch != payroll.company_epoch
            || identity.current_policy_receipt_id != Some(payroll.current_policy_receipt_id)
            || identity.company_name != payroll.company_name
            || identity.company_slug != payroll.company_slug
        {
            return Err(Error::Unavailable);
        }
        // Source acquisition may wait. Revalidate Auth after its last query.
        let current = self
            .credentials
            .read_session_in_tx(tx, &self.verifier, self.absolute_ttl)
            .await
            .map_err(auth_error)?;
        if current.account_id != account
            || current.session_id != family
            || current.security_generation != session.security_generation
        {
            return Err(Error::Unavailable);
        }
        let (xid, pid, observed_at): (String, i32, OffsetDateTime) = sqlx::query_as(
            "SELECT pg_catalog.pg_current_xact_id()::text,pg_catalog.pg_backend_pid(),clock_timestamp()",
        ).fetch_one(tx.as_mut()).await.map_err(unavailable)?;
        if current.expires_at <= observed_at
            || current.family_expires_at <= observed_at
            || current.auth_time > observed_at
        {
            return Err(Error::AuthenticationInvalid);
        }
        let source_xid: u64 = xid.parse().map_err(|_| Error::Unavailable)?;
        if source_xid == 0 || source_xid.to_string() != xid {
            return Err(Error::Unavailable);
        }
        Ok(Source {
            payroll,
            identity,
            binding: NativePolicySourceBinding {
                account: AccountId::from_uuid(account).map_err(|_| Error::Unavailable)?,
                session_id: family,
                account_security_generation: current.security_generation,
                source_xid,
                source_backend_pid: pid,
                observed_at,
            },
        })
    }

    fn authorize_payroll(&self, source: &Source) -> Result<(), Error> {
        let p = &source.payroll;
        let b = &source.binding;
        let (action, properties) =
            decode_native_payroll_read_clause(self.company, &p.registered_clauses)
                .map_err(|_| Error::Unavailable)?;
        let authority = CurrentPayrollReadAuthority::from_retained_projection(
            b,
            NativePayrollReadProjectionRow {
                account_id: *b.account.as_uuid(),
                session_id: b.session_id,
                account_security_generation: b.account_security_generation,
                org_id: *self.company.as_uuid(),
                company_epoch: p.company_epoch,
                current_policy_receipt_id: p.current_policy_receipt_id,
                assignment_id: p.assignment_id,
                assignment_revision: p.assignment_revision,
                assignment_state: "ACTIVE".into(),
                role_id: p.role_id,
                role_revision: p.role_revision,
                role_state: "ACTIVE".into(),
                valid_from: p.assignment_valid_from,
                valid_until: p.assignment_valid_until,
                delegable: false,
                action,
                properties,
                observed_at: b.observed_at,
                source_xid: b.source_xid.to_string(),
                source_backend_pid: b.source_backend_pid,
            },
        )
        .map_err(|_| Error::Unavailable)?;
        match self
            .policy
            .decide_native_payroll_collection(&authority)
            .map_err(|_| Error::Unavailable)?
        {
            CompanyPolicyDecision::Allow => Ok(()),
            CompanyPolicyDecision::Deny => Err(denied()),
        }
    }

    fn identity(&self, source: &Source) -> Result<Option<PayrollCompanyIdentity>, Error> {
        let i = &source.identity;
        let authority = CurrentCompanyAuthority::from_current_projection(
            source.binding.account,
            self.company,
            source.binding.observed_at,
            CompanyProjectionRow {
                company_epoch: i.company_epoch,
                context_generation: i.context_generation,
                assignment_id: i.assignment_id,
                assignment_revision: i.assignment_revision,
                role_id: i.role_id,
                role_revision: i.role_revision,
                registered_clauses: i.registered_clauses.clone(),
                company_name: i.company_name.clone(),
                company_slug: i.company_slug.clone(),
            },
            i.current_policy_receipt_id,
        )
        .map_err(|_| Error::Unavailable)?;
        match project_company_identity(self.policy.as_ref(), &authority) {
            Ok(identity) => Ok(Some(PayrollCompanyIdentity {
                name: identity.name,
                slug: identity.slug,
            })),
            Err(CompanyPolicyError::NotFound) => Ok(None),
            Err(_) => Err(Error::Unavailable),
        }
    }
    async fn finish(&self, retained: Retained) -> Result<PayrollCompanyContext, Error> {
        let Retained {
            mut tx,
            source,
            identity_allowed,
        } = retained;
        // Reacquire current authority after all preceding waits.
        let current = self
            .source(
                &mut tx,
                *source.binding.account.as_uuid(),
                source.binding.session_id,
            )
            .await?;
        if source.payroll != current.payroll
            || source.identity != current.identity
            || source.binding.account != current.binding.account
            || source.binding.session_id != current.binding.session_id
            || source.binding.account_security_generation
                != current.binding.account_security_generation
            || source.binding.source_xid != current.binding.source_xid
            || source.binding.source_backend_pid != current.binding.source_backend_pid
            || current.binding.observed_at < source.binding.observed_at
        {
            return Err(Error::Unavailable);
        }
        self.authorize_payroll(&current)?;
        let identity = self.identity(&current)?;
        // An uncertain COMMIT never releases a projection; any list audit outcome is unknown.
        tx.commit().await.map_err(unavailable)?;
        Ok(PayrollCompanyContext {
            id: self.company,
            identity: if identity_allowed { identity } else { None },
        })
    }
}

impl PayrollRunsReadPort for PgNativePayrollRunsReadPort {
    fn finish_navigation(&mut self) -> PayrollNavigationFuture<'_> {
        // Consume before constructing the future: even an unpolled drop releases
        // the retained transaction while the reader itself remains alive.
        let prior = std::mem::replace(&mut self.state, State::Consumed);
        Box::pin(async move {
            let State::Authorized(retained) = prior else {
                return Err(Error::Unavailable);
            };
            self.finish(*retained).await.map(|company| company.id)
        })
    }

    fn authorize(&mut self) -> PayrollAuthorizeFuture<'_> {
        // Consume before the first await; cancelling this future cannot leave an
        // apparently unused reader or a transaction retained in self.
        let prior = std::mem::replace(&mut self.state, State::Consumed);
        Box::pin(async move {
            if !matches!(prior, State::Unused) {
                return Err(Error::Unavailable);
            }
            if self.company.as_uuid().is_nil()
                || self.company == OrgId::platform()
                || self.absolute_ttl <= Duration::ZERO
            {
                return Err(Error::Unavailable);
            }
            let mut tx = self.pool.begin().await.map_err(unavailable)?;
            sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL READ COMMITTED; SET LOCAL statement_timeout='10s'; SET LOCAL lock_timeout='1s'")
                .execute(tx.as_mut()).await.map_err(unavailable)?;
            let (account, family) = self
                .credentials
                .session_ids_in_tx(&mut tx, &self.verifier, self.absolute_ttl)
                .await
                .map_err(auth_error)?;
            let source = self.source(&mut tx, account, family).await?;
            self.authorize_payroll(&source)?;
            let identity_allowed = self.identity(&source)?.is_some();
            self.state = State::Authorized(Box::new(Retained {
                tx,
                source,
                identity_allowed,
            }));
            Ok(())
        })
    }

    fn read_page(&mut self, query: ListPayrollRuns) -> PayrollReadFuture<'_> {
        let prior = std::mem::replace(&mut self.state, State::Consumed);
        Box::pin(async move {
            let State::Authorized(retained) = prior else {
                return Err(Error::Unavailable);
            };
            let Retained {
                mut tx,
                source,
                identity_allowed,
            } = *retained;
            sqlx::query("SELECT set_config('app.current_org',$1,true)")
                .bind(self.company.to_string())
                .execute(tx.as_mut())
                .await
                .map_err(unavailable)?;
            let page = crate::list_runs_in_tx(&mut tx, query.limit, query.offset)
                .await
                .map_err(unavailable)?;
            let event = AuditEvent::new_account(
                source.binding.account,
                AuditAction::new("payroll_run.list_read").map_err(|_| Error::Unavailable)?,
                "payroll_draft_run",
                "query",
                TraceContext::generate(),
                account_now_in_tx(&mut tx).await.map_err(auth_error)?,
            )
            .with_org(self.company);
            insert_audit_event(&mut tx, &event)
                .await
                .map_err(unavailable)?;
            sqlx::query("SET CONSTRAINTS ALL IMMEDIATE")
                .execute(tx.as_mut())
                .await
                .map_err(unavailable)?;
            let company = self
                .finish(Retained {
                    tx,
                    source,
                    identity_allowed,
                })
                .await?;
            Ok(PayrollRunsReadResult {
                page,
                company: Some(company),
            })
        })
    }
}

fn denied() -> Error {
    Error::Authorization(KernelError::not_found("not found"))
}
fn auth_error(error: AccountOperationError) -> Error {
    match error {
        AccountOperationError::AuthenticationInvalid => Error::AuthenticationInvalid,
        _ => Error::Unavailable,
    }
}
fn projection_error(error: sqlx::Error) -> Error {
    if error.as_database_error().is_some_and(|db| {
        db.code().as_deref() == Some("P0001") && db.message() == "account.authentication_invalid"
    }) {
        Error::AuthenticationInvalid
    } else {
        unavailable(error)
    }
}
fn unavailable(error: impl std::fmt::Display) -> Error {
    tracing::error!(error = %error, "native payroll read unavailable");
    Error::Unavailable
}
