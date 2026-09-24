//! Native Account People workflow. This owner retains one transaction through
//! current credentials, actual Cedar, canonical effects, closure and COMMIT.
use DirectoryWorkflowError as Error;
use console_identity_application::company_policy::CompanyPolicyDecisionPort;
use console_kernel_core::{AccountId, AuditAction, AuditEvent, OrgId, TraceContext};
use console_ontology_application::people::{workflow::*, *};
use console_platform_auth::{
    JwtIssuer, JwtVerifier,
    account::{
        AccountEnrollmentCredentials, AccountFormProof, AccountOperationError, account_now_in_tx,
    },
};
use console_platform_db::insert_audit_event;
use sqlx::{PgPool, Postgres, Row, Transaction};
use std::sync::Arc;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;
mod rows;
mod source;
pub use source::{DirectoryCedarDecision, NativeDirectoryAuthority};

pub struct PgNativeDirectoryStore {
    pool: PgPool,
    verifier: JwtVerifier,
    issuer: JwtIssuer,
    absolute_ttl: Duration,
}
impl PgNativeDirectoryStore {
    pub fn new(
        pool: PgPool,
        verifier: JwtVerifier,
        issuer: JwtIssuer,
        absolute_ttl: Duration,
    ) -> Result<Self, Error> {
        if absolute_ttl <= Duration::ZERO {
            return Err(Error::Unavailable);
        }
        Ok(Self {
            pool,
            verifier,
            issuer,
            absolute_ttl,
        })
    }
}
pub struct PgNativeDirectoryScope<'a> {
    tx: Transaction<'static, Postgres>,
    store: &'a PgNativeDirectoryStore,
    credentials: &'a AccountEnrollmentCredentials,
    request: DirectoryScopeRequest<'a>,
    source: source::Source,
    completed: bool,
    proof_expected: bool,
    pending_until: Option<OffsetDateTime>,
}
impl DirectoryWorkflowStore for PgNativeDirectoryStore {
    type Credentials = AccountEnrollmentCredentials;
    type Authority = NativeDirectoryAuthority;
    type FormProof = AccountFormProof;
    type Scope<'a>
        = PgNativeDirectoryScope<'a>
    where
        Self: 'a;
    async fn lock<'a>(
        &'a self,
        credentials: &'a AccountEnrollmentCredentials,
        request: DirectoryScopeRequest<'a>,
    ) -> Result<Self::Scope<'a>, Error> {
        let company = request.company();
        if company.as_uuid().is_nil() || company == OrgId::platform() {
            return Err(Error::InvalidInput);
        }
        let mut tx = self.pool.begin().await.map_err(sql_error)?;
        sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL READ COMMITTED; SET LOCAL statement_timeout='10s'; SET LOCAL lock_timeout='1s'").execute(tx.as_mut()).await.map_err(sql_error)?;
        let (actor, family) = credentials
            .session_ids_in_tx(&mut tx, &self.verifier, self.absolute_ttl)
            .await
            .map_err(auth_error)?;
        let source = source::current(
            &mut tx,
            self,
            credentials,
            request.kind(),
            company,
            actor,
            family,
        )
        .await?;
        Ok(PgNativeDirectoryScope {
            tx,
            store: self,
            credentials,
            request,
            source,
            completed: false,
            proof_expected: false,
            pending_until: None,
        })
    }
}
impl PgNativeDirectoryScope<'_> {
    fn unused(&self) -> Result<(), Error> {
        if self.completed {
            Err(Error::Unavailable)
        } else {
            Ok(())
        }
    }
    async fn arm(&mut self) -> Result<(), Error> {
        sqlx::query("SELECT set_config('app.current_org',$1,true)")
            .bind(self.request.company().to_string())
            .execute(self.tx.as_mut())
            .await
            .map_err(sql_error)?;
        Ok(())
    }
    async fn refresh(&mut self) -> Result<(), Error> {
        let current = source::current(
            &mut self.tx,
            self.store,
            self.credentials,
            self.request.kind(),
            self.request.company(),
            *self.source.binding.account.as_uuid(),
            self.source.binding.session_id,
        )
        .await?;
        source::same(&self.source, &current)?;
        self.source = current;
        Ok(())
    }
    async fn form_proof(&mut self, validation: bool) -> Result<AccountFormProof, Error> {
        if validation {
            self.credentials
                .retain_submitted_form_proof_in_tx(
                    &mut self.tx,
                    &self.store.verifier,
                    self.store.absolute_ttl,
                )
                .await
        } else {
            self.credentials
                .issue_form_proof_in_tx(
                    &mut self.tx,
                    &self.store.verifier,
                    &self.store.issuer,
                    self.store.absolute_ttl,
                )
                .await
        }
        .map_err(auth_error)
    }
    async fn audit_transition(
        &mut self,
        trace: &TraceContext,
        command: Uuid,
        action: &str,
    ) -> Result<(), Error> {
        let rows=sqlx::query("SELECT target_type,target_id,occurred_at,payload FROM public.native_people_audit_material_v1($1,$2,$3,$4,$5) LIMIT 2")
            .bind(*self.source.binding.account.as_uuid()).bind(self.source.binding.session_id).bind(*self.request.company().as_uuid()).bind(command).bind(action).fetch_all(self.tx.as_mut()).await.map_err(sql_error)?;
        let [row] = rows.as_slice() else {
            return Err(Error::Unavailable);
        };
        let target: String = row.try_get("target_type").map_err(sql_error)?;
        let id: String = row.try_get("target_id").map_err(sql_error)?;
        let at: OffsetDateTime = row.try_get("occurred_at").map_err(sql_error)?;
        let payload: serde_json::Value = row.try_get("payload").map_err(sql_error)?;
        self.arm().await?;
        let event = AuditEvent::new_account(
            self.source.binding.account,
            AuditAction::new(action).map_err(|_| Error::Unavailable)?,
            target,
            id,
            trace.clone(),
            at,
        )
        .with_org(self.request.company())
        .with_snapshots(None, Some(payload));
        insert_audit_event(&mut self.tx, &event)
            .await
            .map_err(|_| Error::Unavailable)
    }
    async fn audit_read(&mut self, target: &str) -> Result<(), Error> {
        let at = account_now_in_tx(&mut self.tx).await.map_err(auth_error)?;
        let event = AuditEvent::new_account(
            self.source.binding.account,
            AuditAction::new("people.directory.read").map_err(|_| Error::Unavailable)?,
            "person_directory",
            target,
            TraceContext::generate(),
            at,
        )
        .with_org(self.request.company());
        insert_audit_event(&mut self.tx, &event)
            .await
            .map_err(|_| Error::Unavailable)
    }
    async fn submission_query(
        &mut self,
        prepare: bool,
    ) -> Result<Vec<sqlx::postgres::PgRow>, Error> {
        let submission = match (&self.request, prepare) {
            (DirectoryScopeRequest::Prepare(s), true)
            | (DirectoryScopeRequest::Preflight(s), false) => *s,
            _ => return Err(Error::Unavailable),
        };
        let sql: &'static str = if prepare {
            rows::PREPARE_SQL
        } else {
            "SELECT public.native_people_preflight_v1($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)"
        };
        let expected = submission.expected();
        sqlx::query(sql)
            .bind(*self.source.binding.account.as_uuid())
            .bind(self.source.binding.session_id)
            .bind(*submission.locator().company().as_uuid())
            .bind(submission.locator().command_id())
            .bind(expected.company_epoch as i64)
            .bind(expected.object_type_id)
            .bind(expected.action_type_id)
            .bind(expected.action_revision as i64)
            .bind(expected.schema_revision as i64)
            .bind(expected.legal_name_property_id)
            .bind(expected.employee_number_property_id)
            .bind(submission.input().legal_name())
            .bind(submission.input().employee_number())
            .fetch_all(self.tx.as_mut())
            .await
            .map_err(sql_error)
    }
    async fn transition(
        &mut self,
        operation: &'static str,
        trace: &TraceContext,
    ) -> Result<Option<rows::Opened>, Error> {
        let command = match (&self.request, operation) {
            (DirectoryScopeRequest::Prepare(s), "STATUS") => s.locator().command_id(),
            (DirectoryScopeRequest::Execute(r), "EXECUTE")
            | (DirectoryScopeRequest::Cancel(r), "CANCEL")
            | (DirectoryScopeRequest::Status(r), "STATUS") => r.command_id(),
            _ => return Err(Error::Unavailable),
        };
        let rows = sqlx::query(rows::TERMINAL_SQL)
            .bind(*self.source.binding.account.as_uuid())
            .bind(self.source.binding.session_id)
            .bind(*self.request.company().as_uuid())
            .bind(command)
            .bind(operation)
            .fetch_all(self.tx.as_mut())
            .await
            .map_err(sql_error)?;
        if rows.is_empty() {
            return Ok(None);
        }
        let [row] = rows.as_slice() else {
            return Err(Error::Unavailable);
        };
        let now = account_now_in_tx(&mut self.tx).await.map_err(auth_error)?;
        let opened = rows::decode(
            row,
            &self.source,
            self.request.company(),
            command,
            rows::Opening::Transition(operation),
            now,
        )?;
        if opened.inserted {
            let terminal = opened.terminal.as_ref().ok_or(Error::Unavailable)?;
            if terminal.outcome() == DirectoryTerminalOutcomeV1::Committed {
                self.arm().await?;
                crate::employment::insert_native_directory_employee_in_tx(&mut self.tx, terminal)
                    .await
                    .map_err(sql_error)?;
                crate::person::write_native_directory_person_in_tx(
                    &mut self.tx,
                    terminal,
                    &opened.effect_digest,
                )
                .await
                .map_err(sql_error)?;
            }
            self.audit_transition(trace, command, "people.directory.register")
                .await?;
            if terminal.outcome() != DirectoryTerminalOutcomeV1::Expired {
                self.pending_until = Some(opened.accepted.execution_not_after());
            }
        }
        Ok(Some(opened))
    }
}
#[derive(sqlx::FromRow)]
struct RecordRow {
    employee_id: Uuid,
    person_id: Uuid,
    name_attributes: serde_json::Value,
    employee_number: Option<String>,
    source_kind: String,
    person_version: i64,
    registered_at: OffsetDateTime,
}
impl RecordRow {
    fn view(self) -> Result<DirectoryRecord, Error> {
        let legal_name = crate::person::attr_string(&self.name_attributes, "legal_name");
        let employee_number = self.employee_number.filter(|value| !value.is_empty());
        match self.source_kind.as_str() {
            "LEGACY" => {}
            "NATIVE_DIRECTORY" => {
                let name = legal_name.as_deref().ok_or(Error::Unavailable)?;
                let number = employee_number.as_deref().ok_or(Error::Unavailable)?;
                let validated = DirectoryRegistrationInput::new(name, number)
                    .map_err(|_| Error::Unavailable)?;
                if validated.legal_name() != name || validated.employee_number() != number {
                    return Err(Error::Unavailable);
                }
            }
            _ => return Err(Error::Unavailable),
        }
        Ok(DirectoryRecord {
            employee_id: nonnil(self.employee_id)?,
            person_id: nonnil(self.person_id)?,
            legal_name,
            employee_number,
            person_version: positive(self.person_version)?,
            registered_at: self.registered_at,
        })
    }
}
macro_rules! record_query { ($tail:literal) => {concat!("SELECT e.id AS employee_id,b.person_id,jsonb_build_object('legal_name',r.attributes->'legal_name') AS name_attributes,e.employee_number,e.source_kind,r.version AS person_version,e.created_at AS registered_at FROM public.employees e JOIN public.employee_person_bindings b ON b.org_id=e.org_id AND b.employee_id=e.id JOIN LATERAL (SELECT version,attributes FROM public.person_revisions WHERE org_id=b.org_id AND person_id=b.person_id ORDER BY version DESC LIMIT 1) r ON true WHERE e.org_id=$1", $tail)}; }
const LIST_SQL: &str = record_query!(" AND ($2::uuid IS NULL OR e.id>$2) ORDER BY e.id LIMIT $3");
const DETAIL_SQL: &str = record_query!(" AND e.id=$2 LIMIT 2");
impl DirectoryWorkflowScope for PgNativeDirectoryScope<'_> {
    type Authority = NativeDirectoryAuthority;
    type FormProof = AccountFormProof;
    fn authority(&self) -> &NativeDirectoryAuthority {
        &self.source.authority
    }
    fn kind(&self) -> DirectoryScopeKind {
        self.request.kind()
    }
    async fn list(&mut self) -> Result<DirectoryPage, Error> {
        self.unused()?;
        let DirectoryScopeRequest::List(_, query) = self.request else {
            return Err(Error::Unavailable);
        };
        self.arm().await?;
        let rows = sqlx::query_as::<_, RecordRow>(LIST_SQL)
            .bind(*self.request.company().as_uuid())
            .bind(query.after())
            .bind(i64::from(query.limit()) + 1)
            .fetch_all(self.tx.as_mut())
            .await
            .map_err(sql_error)?;
        let mut records = rows
            .into_iter()
            .map(RecordRow::view)
            .collect::<Result<Vec<_>, _>>()?;
        let more = records.len() > usize::from(query.limit());
        records.truncate(usize::from(query.limit()));
        let next_after = if more {
            records.last().map(|r| r.employee_id)
        } else {
            None
        };
        self.audit_read("collection").await?;
        self.completed = true;
        Ok(DirectoryPage {
            records,
            next_after,
        })
    }
    async fn detail(&mut self) -> Result<Option<DirectoryRecord>, Error> {
        self.unused()?;
        let DirectoryScopeRequest::Detail(_, employee) = self.request else {
            return Err(Error::Unavailable);
        };
        self.arm().await?;
        let rows = sqlx::query_as::<_, RecordRow>(DETAIL_SQL)
            .bind(*self.request.company().as_uuid())
            .bind(employee)
            .fetch_all(self.tx.as_mut())
            .await
            .map_err(sql_error)?;
        if rows.len() > 1 {
            return Err(Error::Unavailable);
        }
        let result = rows.into_iter().next().map(RecordRow::view).transpose()?;
        self.audit_read(&employee.to_string()).await?;
        self.completed = true;
        Ok(result)
    }
    async fn form(&mut self) -> Result<DirectoryForm<AccountFormProof>, Error> {
        self.unused()?;
        let (locator, expected, validation) = match self.request {
            DirectoryScopeRequest::Form(r) => (
                r,
                self.source
                    .authority
                    .creation_expectations()
                    .ok_or(Error::Unavailable)?,
                false,
            ),
            DirectoryScopeRequest::ValidationForm(r, e) => (r, e, true),
            _ => return Err(Error::Unavailable),
        };
        let proof = self.form_proof(validation).await?;
        self.proof_expected = true;
        self.completed = true;
        Ok(DirectoryForm {
            locator,
            expected,
            proof,
        })
    }
    async fn preflight(&mut self) -> Result<(), Error> {
        self.unused()?;
        self.submission_query(false).await?;
        self.completed = true;
        Ok(())
    }
    async fn prepare(&mut self, trace: &TraceContext) -> Result<DirectoryAcceptance, Error> {
        self.unused()?;
        let command = match &self.request {
            DirectoryScopeRequest::Prepare(s) => s.locator().command_id(),
            _ => return Err(Error::Unavailable),
        };
        let rows = self.submission_query(true).await?;
        let [row] = rows.as_slice() else {
            return Err(Error::Unavailable);
        };
        let now = account_now_in_tx(&mut self.tx).await.map_err(auth_error)?;
        let mut opened = rows::decode(
            row,
            &self.source,
            self.request.company(),
            command,
            rows::Opening::Prepare,
            now,
        )?;
        if !opened.inserted && opened.terminal.is_none() {
            // Exact intake replay preserves its original deadline. Resolve an
            // overdue pending request now, without allocating another intake.
            let recovered = self
                .transition("STATUS", trace)
                .await?
                .ok_or(Error::Unavailable)?;
            if recovered.accepted != opened.accepted
                || recovered.effect_digest != opened.effect_digest
            {
                return Err(Error::Unavailable);
            }
            opened.terminal = recovered.terminal;
        }
        if opened.inserted {
            self.audit_transition(trace, command, "people.directory.prepare")
                .await?;
            self.pending_until = Some(opened.accepted.execution_not_after());
        }
        let status = match opened.terminal {
            Some(t) => DirectoryStatus::Terminal(t),
            None => {
                // Every Pending disclosure, including exact historical intake
                // replay, must remain actionable through final readmission.
                pending_status(opened.accepted, &mut self.pending_until)
            }
        };
        self.completed = true;
        Ok(DirectoryAcceptance {
            inserted: opened.inserted,
            status,
        })
    }
    async fn execute(&mut self, trace: &TraceContext) -> Result<DirectoryExecution, Error> {
        self.unused()?;
        let r = self
            .transition("EXECUTE", trace)
            .await?
            .ok_or(Error::NotFound)?;
        let terminal = r.terminal.ok_or(Error::Unavailable)?;
        self.completed = true;
        Ok(DirectoryExecution {
            inserted: r.inserted,
            terminal,
        })
    }
    async fn cancel(&mut self, trace: &TraceContext) -> Result<DirectoryExecution, Error> {
        self.unused()?;
        let r = self
            .transition("CANCEL", trace)
            .await?
            .ok_or(Error::NotFound)?;
        let terminal = r.terminal.ok_or(Error::Unavailable)?;
        self.completed = true;
        Ok(DirectoryExecution {
            inserted: r.inserted,
            terminal,
        })
    }
    async fn status(
        &mut self,
        trace: &TraceContext,
    ) -> Result<DirectoryRecovery<AccountFormProof>, Error> {
        self.unused()?;
        if !matches!(self.request, DirectoryScopeRequest::Status(_)) {
            return Err(Error::Unavailable);
        }
        let opened = self.transition("STATUS", trace).await?;
        let (status, proof) = match opened {
            None => (DirectoryStatus::NotVisible, None),
            Some(r) => match r.terminal {
                Some(t) => (DirectoryStatus::Terminal(t), None),
                None => {
                    let status = pending_status(r.accepted, &mut self.pending_until);
                    let proof = self.form_proof(false).await?;
                    self.proof_expected = true;
                    (status, Some(proof))
                }
            },
        };
        // Terminal creation and proof issuance occur after initial lock time.
        // Reconstruct complete checked authority; never patch only a timestamp.
        self.refresh().await?;
        self.completed = true;
        Ok(DirectoryRecovery { status, proof })
    }
    async fn finish<P: DirectoryDecisionPort<NativeDirectoryAuthority> + ?Sized>(
        mut self,
        policy: &P,
        proof: Option<&AccountFormProof>,
    ) -> Result<(), Error> {
        // Navigation has no data/effect method to complete. It still uses the
        // identical retained-source final Auth/Cedar path and cannot carry proof.
        let navigation = matches!(self.request, DirectoryScopeRequest::Navigation(_, _));
        if (!self.completed && !navigation) || self.proof_expected != proof.is_some() {
            return Err(Error::Unavailable);
        }
        sqlx::query("SET CONSTRAINTS ALL IMMEDIATE")
            .execute(self.tx.as_mut())
            .await
            .map_err(sql_error)?;
        if let Some(proof) = proof {
            self.credentials
                .recheck_form_proof_in_tx(
                    &mut self.tx,
                    &self.store.verifier,
                    self.store.absolute_ttl,
                    proof,
                )
                .await
                .map_err(auth_error)?;
        }
        self.refresh().await?;
        let now = self.source.binding.observed_at;
        if proof.is_some_and(|p| p.expires_at() <= now) {
            return Err(Error::CsrfInvalid);
        }
        check_pending_readmission(self.pending_until, now)?;
        let access = DirectoryAccess {
            company: self.request.company(),
            actor: self.source.binding.account,
            action: source::action(self.request.kind()),
            resource: source::resource(self.request.kind()),
        };
        if !policy.permits(&self.source.authority, access)? {
            return Err(Error::NotFound);
        }
        // Nothing which can wait or write remains between final decision and
        // COMMIT dispatch. Lost response cannot release a provisional outcome.
        self.tx.commit().await.map_err(commit_error)
    }
}
fn positive(value: i64) -> Result<u64, Error> {
    if value < 1 {
        Err(Error::Unavailable)
    } else {
        Ok(value as u64)
    }
}
fn nonnil(value: Uuid) -> Result<Uuid, Error> {
    if value.is_nil() {
        Err(Error::Unavailable)
    } else {
        Ok(value)
    }
}
fn xid_number(value: &str) -> Result<u64, Error> {
    let xid = value.parse::<u64>().map_err(|_| Error::Unavailable)?;
    if xid == 0 || xid.to_string() != value {
        Err(Error::Unavailable)
    } else {
        Ok(xid)
    }
}
fn auth_error(error: AccountOperationError) -> Error {
    match error {
        AccountOperationError::AuthenticationInvalid => Error::AuthenticationInvalid,
        AccountOperationError::CsrfInvalid => Error::CsrfInvalid,
        _ => Error::Unavailable,
    }
}
fn sql_error(error: sqlx::Error) -> Error {
    match error.as_database_error() {
        Some(db) => match (db.code().as_deref(), db.message()) {
            (Some("P0001"), "account.authentication_invalid") => Error::AuthenticationInvalid,
            (Some("42501"), "people.directory.not_visible") => Error::NotFound,
            (Some("22023"), "people.directory.invalid_input") => Error::InvalidInput,
            (Some("P0001"), "people.directory.capacity") => Error::Capacity,
            (
                Some("P0001"),
                "people.directory.conflict"
                | "people.directory.revision_conflict"
                | "people.directory.command_conflict"
                | "people.directory.employee_number_conflict",
            ) => Error::Conflict,
            _ => Error::Unavailable,
        },
        _ => Error::Unavailable,
    }
}
fn commit_error(error: sqlx::Error) -> Error {
    match error.as_database_error().and_then(|db| db.code()) {
        Some(code)
            if matches!(
                code.as_ref(),
                "23502" | "23503" | "23505" | "23514" | "23P01" | "40001" | "40P01"
            ) =>
        {
            Error::Unavailable
        }
        _ => Error::Unconfirmed,
    }
}

// Shared by prepare replay and status, preserving the original accepted deadline.
fn pending_status(
    accepted: AcceptedDirectoryRequestV1,
    until: &mut Option<OffsetDateTime>,
) -> DirectoryStatus {
    *until = Some(accepted.execution_not_after());
    DirectoryStatus::Pending(accepted)
}
fn check_pending_readmission(
    until: Option<OffsetDateTime>,
    now: OffsetDateTime,
) -> Result<(), Error> {
    if until.is_some_and(|deadline| now >= deadline) {
        Err(Error::Conflict)
    } else {
        Ok(())
    }
}
#[cfg(test)]
#[path = "native_directory/tests.rs"]
mod tests;
