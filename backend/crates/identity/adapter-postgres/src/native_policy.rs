//! Native policy commands retain their Group-first owner transaction through
//! current Auth/Cedar checks and the shared Account audit writer.
use super::{
    PgOrgStore,
    company_policy::{NativeAccountMode, NativeAccountReadConfig},
};
use console_identity_application::company_policy::{
    AccountId, CompanyPolicyDecision, CompanyPolicyDecisionPort, CurrentNativeBootstrapAuthority,
    NativeBootstrapProjectionRow, NativeBootstrapRequestV1, NativePolicySourceBinding,
    business::{NativeBusinessOperationV1, PolicyAssignmentExpectationV1},
    workflow::*,
};
use console_kernel_core::{AuditAction, AuditEvent, TraceContext};
use console_platform_auth::account::{
    AccountEnrollmentCredentials, AccountFormProof, AccountLiveSession, AccountOperationError,
    account_now_in_tx, ensure_account_session_fresh_in_tx,
};
use console_platform_db::insert_audit_event;
use serde_json::{Value, json};
use sqlx::{Postgres, Row, Transaction, postgres::PgRow};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

mod rows;
use NativePolicyWorkflowError as Error;

pub struct PgNativePolicyScope<'a> {
    tx: Transaction<'static, Postgres>,
    config: &'a NativeAccountReadConfig,
    credentials: &'a AccountEnrollmentCredentials,
    request: NativePolicyScopeRequest<'a>,
    selector: NativePolicyCommandRef,
    authority: CurrentNativeBootstrapAuthority,
    binding: NativePolicySourceBinding,
    completed: bool,
    head_effect: Option<(u64, Uuid)>,
    pending_until: Option<OffsetDateTime>,
    grant_until: Option<OffsetDateTime>,
}

impl NativePolicyWorkflowStore for PgOrgStore {
    type Credentials = AccountEnrollmentCredentials;
    type FormProof = AccountFormProof;
    type Scope<'a> = PgNativePolicyScope<'a>;

    async fn lock<'a>(
        &'a self,
        credentials: &'a Self::Credentials,
        request: NativePolicyScopeRequest<'a>,
    ) -> Result<Self::Scope<'a>, Error> {
        // A decodable protocol must not reach the existing SQL owner until its
        // exact catalog and transaction contract have been separately activated.
        if !matches!(request.selector().codec_version(), 1 | 2) {
            return Err(Error::Unavailable);
        }
        let config = self.native_read_config().map_err(|_| Error::Unavailable)?;
        if !matches!(config.mode, NativeAccountMode::Policy { .. }) {
            return Err(Error::Unavailable);
        }
        let mut tx = self
            .native_read_transaction()
            .await
            .map_err(|_| Error::Unavailable)?;
        let (actor, family) = credentials
            .session_ids_in_tx(&mut tx, &config.verifier, config.absolute_ttl)
            .await
            .map_err(auth_error)?;
        let (authority, binding) =
            current_source(&mut tx, config, credentials, &request, actor, family).await?;
        let selector = request.selector();
        let mut scope = PgNativePolicyScope {
            tx,
            config,
            credentials,
            request,
            selector,
            authority,
            binding,
            completed: false,
            head_effect: None,
            pending_until: None,
            grant_until: None,
        };
        if selector.codec_version() == 2
            && selector.operation() != NativeBusinessOperationV1::Install
            && selector.directory_action().is_none()
        {
            if !matches!(
                scope.request,
                NativePolicyScopeRequest::Status(_) | NativePolicyScopeRequest::Execute(_)
            ) {
                return Err(Error::InvalidInput);
            }
            let result = scope.query(rows::STATUS, None).await?;
            let [row] = result.as_slice() else {
                return Err(Error::Unavailable);
            };
            if rows::absent(row, "input_")? {
                if !rows::absent(row, "terminal_")? {
                    return Err(Error::Unavailable);
                }
                return Err(Error::NotFound);
            }
            let original = scope.accepted(row).await?;
            scope.selector =
                selector.resolve(NativePolicyCommandRef::from_command(&original.view.input))?;
        }
        Ok(scope)
    }
}

async fn live_session(
    tx: &mut Transaction<'_, Postgres>,
    config: &NativeAccountReadConfig,
    credentials: &AccountEnrollmentCredentials,
    request: &NativePolicyScopeRequest<'_>,
) -> Result<AccountLiveSession, Error> {
    match request {
        NativePolicyScopeRequest::Accept(_)
        | NativePolicyScopeRequest::Execute(_)
        | NativePolicyScopeRequest::ValidationForm(_) => {
            credentials
                .validate_mutation_in_tx(tx, &config.verifier, config.absolute_ttl)
                .await
        }
        _ => {
            credentials
                .read_session_in_tx(tx, &config.verifier, config.absolute_ttl)
                .await
        }
    }
    .map_err(auth_error)
}

async fn current_source(
    tx: &mut Transaction<'_, Postgres>,
    config: &NativeAccountReadConfig,
    credentials: &AccountEnrollmentCredentials,
    request: &NativePolicyScopeRequest<'_>,
    actor: Uuid,
    family: Uuid,
) -> Result<(CurrentNativeBootstrapAuthority, NativePolicySourceBinding), Error> {
    let selector = request.selector();
    let input = match request {
        NativePolicyScopeRequest::Accept(input) => Some(input.encode(account(actor)?)),
        _ => None,
    };
    let result = sqlx::query(rows::MATERIAL27)
        .bind(actor)
        .bind(family)
        .bind(*selector.company().as_uuid())
        .bind(selector.command_id())
        .bind(operation_number(selector.operation()))
        .bind(input)
        .fetch_all(tx.as_mut())
        .await
        .map_err(sql_error)?;
    let [row] = result.as_slice() else {
        return Err(Error::Unavailable);
    };
    let projection = rows::material(row)?;
    let session = live_session(tx, config, credentials, request).await?;
    if session.account_id != actor || session.session_id != family {
        return Err(Error::Unavailable);
    }
    let (xid, pid, observed_at): (String, i32, OffsetDateTime) = sqlx::query_as(
        "SELECT pg_catalog.pg_current_xact_id()::text,pg_catalog.pg_backend_pid(),clock_timestamp()")
        .fetch_one(tx.as_mut()).await.map_err(sql_error)?;
    let binding = NativePolicySourceBinding {
        account: account(session.account_id)?,
        session_id: session.session_id,
        account_security_generation: session.security_generation,
        source_xid: xid_number(&xid)?,
        source_backend_pid: pid,
        observed_at,
    };
    let authority = CurrentNativeBootstrapAuthority::from_retained_projection(&binding, projection)
        .map_err(|_| Error::Unavailable)?;
    Ok((authority, binding))
}

impl PgNativePolicyScope<'_> {
    async fn current_view(&mut self) -> Result<NativePolicyFormView, Error> {
        let query = if self.selector.codec_version() == 2 {
            rows::OPERATOR8_PEOPLE
        } else {
            rows::OPERATOR8
        };
        let result = self.query(query, None).await?;
        let [row] = result.as_slice() else {
            return Err(Error::Unavailable);
        };
        let installed: Option<Uuid> = row.try_get("installed_object_type_id").map_err(sql_error)?;
        if installed.is_some_and(|id| id.is_nil()) {
            return Err(Error::Unavailable);
        }
        let assignment = rows::assignment_view(
            row.try_get("role_id").map_err(sql_error)?,
            row.try_get("role_revision").map_err(sql_error)?,
            row.try_get("assignment_id").map_err(sql_error)?,
            row.try_get("assignment_revision").map_err(sql_error)?,
            row.try_get("assignment_state").map_err(sql_error)?,
            row.try_get("assignment_valid_from").map_err(sql_error)?,
            row.try_get("assignment_valid_until").map_err(sql_error)?,
        )?;
        if installed.is_none() && assignment.is_some() {
            return Err(Error::Unavailable);
        }
        Ok(NativePolicyFormView {
            selector: self.selector,
            group_id: self.authority.source().current_group_id,
            company_epoch: positive(self.authority.source().company_epoch)?,
            acting_account_id: self.binding.account,
            administrative_account_id: account(self.authority.source().administrative_account_id)?,
            installed_object_type_id: installed,
            assignment,
        })
    }
    fn unused(&self) -> Result<(), Error> {
        if self.completed {
            Err(Error::Unavailable)
        } else {
            Ok(())
        }
    }
    async fn query(
        &mut self,
        sql: &'static str,
        input: Option<Vec<u8>>,
    ) -> Result<Vec<PgRow>, Error> {
        let s = self.selector;
        let query = sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(*self.binding.account.as_uuid())
            .bind(self.binding.session_id)
            .bind(*s.company().as_uuid())
            .bind(s.command_id())
            .bind(operation_number(s.operation()));
        let query = if sql == rows::PREPARE {
            query.bind(input)
        } else if sql == rows::OPERATOR8_PEOPLE {
            query
                .bind(s.codec_version())
                .bind(s.directory_action().map(|action| action.as_str()))
        } else {
            query
        };
        query.fetch_all(self.tx.as_mut()).await.map_err(sql_error)
    }
    async fn accepted(&mut self, row: &PgRow) -> Result<rows::Accepted, Error> {
        let accepted = rows::accepted(row, self.binding.account, self.selector)?;
        let correct: bool = sqlx::query_scalar("SELECT pg_catalog.sha256($1::bytea)=$2::bytea")
            .bind(accepted.view.input.encode(accepted.actor))
            .bind(&accepted.digest)
            .fetch_one(self.tx.as_mut())
            .await
            .map_err(sql_error)?;
        let now = account_now_in_tx(&mut self.tx).await.map_err(auth_error)?;
        if !correct || accepted.view.accepted_at > now {
            return Err(Error::Unavailable);
        }
        Ok(accepted)
    }
    async fn state_from(
        &mut self,
        row: &PgRow,
        accepted: &rows::Accepted,
    ) -> Result<NativePolicyStatus, Error> {
        if !rows::absent(row, "terminal_")? {
            let terminal = rows::terminal(
                row,
                accepted,
                account(self.authority.source().administrative_account_id)?,
            )?;
            if terminal.view.executed_at
                > account_now_in_tx(&mut self.tx).await.map_err(auth_error)?
            {
                return Err(Error::Unavailable);
            }
            return Ok(NativePolicyStatus::Terminal(terminal.view));
        }
        let now = account_now_in_tx(&mut self.tx).await.map_err(auth_error)?;
        if now >= accepted.view.execution_not_after {
            Ok(NativePolicyStatus::AcceptedExpired(accepted.view.clone()))
        } else {
            self.pending_until = Some(accepted.view.execution_not_after);
            Ok(NativePolicyStatus::AcceptedPending(accepted.view.clone()))
        }
    }
    async fn append_audit(
        &mut self,
        trace: &TraceContext,
        action: &str,
        target: &str,
        id: Uuid,
        at: OffsetDateTime,
        payload: Value,
    ) -> Result<(), Error> {
        sqlx::query("SELECT set_config('app.current_org',$1,true)")
            .bind(self.request.selector().company().to_string())
            .execute(self.tx.as_mut())
            .await
            .map_err(sql_error)?;
        let event = AuditEvent::new_account(
            self.binding.account,
            AuditAction::new(action).map_err(|_| Error::Unavailable)?,
            target,
            id.to_string(),
            trace.clone(),
            at,
        )
        .with_org(self.request.selector().company())
        .with_snapshots(None, Some(payload));
        insert_audit_event(&mut self.tx, &event)
            .await
            .map_err(|_| Error::Unavailable)
    }
}

impl NativePolicyWorkflowScope for PgNativePolicyScope<'_> {
    type FormProof = AccountFormProof;
    fn authority(&self) -> &CurrentNativeBootstrapAuthority {
        &self.authority
    }
    fn selector(&self) -> NativePolicyCommandRef {
        self.selector
    }

    async fn current(&mut self) -> Result<NativePolicyFormView, Error> {
        self.unused()?;
        if !matches!(self.request, NativePolicyScopeRequest::Current(_)) {
            return Err(Error::Unavailable);
        }
        let view = self.current_view().await?;
        self.completed = true;
        Ok(view)
    }

    async fn form(&mut self) -> Result<NativePolicyForm<AccountFormProof>, Error> {
        self.unused()?;
        if !matches!(
            self.request,
            NativePolicyScopeRequest::Form(_) | NativePolicyScopeRequest::ValidationForm(_)
        ) {
            return Err(Error::Unavailable);
        }
        let view = self.current_view().await?;
        let NativeAccountMode::Policy { issuer } = &self.config.mode else {
            return Err(Error::Unavailable);
        };
        let proof = if matches!(self.request, NativePolicyScopeRequest::ValidationForm(_)) {
            self.credentials
                .retain_submitted_form_proof_in_tx(
                    &mut self.tx,
                    &self.config.verifier,
                    self.config.absolute_ttl,
                )
                .await
        } else {
            self.credentials
                .issue_form_proof_in_tx(
                    &mut self.tx,
                    &self.config.verifier,
                    issuer,
                    self.config.absolute_ttl,
                )
                .await
        }
        .map_err(auth_error)?;
        self.completed = true;
        Ok(NativePolicyForm { proof, view })
    }

    async fn accept(&mut self, trace: &TraceContext) -> Result<NativePolicyAcceptance, Error> {
        self.unused()?;
        let NativePolicyScopeRequest::Accept(command) = self.request else {
            return Err(Error::Unavailable);
        };
        let result = self
            .query(rows::PREPARE, Some(command.encode(self.binding.account)))
            .await?;
        let [row] = result.as_slice() else {
            return Err(Error::Unavailable);
        };
        let inserted: bool = row.try_get("inserted").map_err(sql_error)?;
        let accepted = self.accepted(row).await?;
        if &accepted.view.input != command {
            return Err(Error::Conflict);
        }
        if inserted {
            if accepted.xid != self.binding.source_xid
                || accepted.pid != self.binding.source_backend_pid
                || accepted.family != self.binding.session_id
                || !rows::absent(row, "terminal_")?
            {
                return Err(Error::Unavailable);
            }
            self.append_audit(
                trace,
                "policy.company_command.accept",
                "native_company_policy_inputs_v1",
                accepted.view.intake_receipt_id,
                accepted.view.accepted_at,
                accept_payload(&accepted, accepted.family)?,
            )
            .await?;
        }
        let status = self.state_from(row, &accepted).await?;
        self.completed = true;
        Ok(NativePolicyAcceptance { inserted, status })
    }

    async fn execute(&mut self, trace: &TraceContext) -> Result<NativePolicyExecution, Error> {
        self.unused()?;
        if !matches!(self.request, NativePolicyScopeRequest::Execute(_)) {
            return Err(Error::Unavailable);
        }
        // Read original input before execution; after a head change the common
        // audit must be appended before any source projection checks custody.
        let prior = self.query(rows::STATUS, None).await?;
        let [prior] = prior.as_slice() else {
            return Err(Error::Unavailable);
        };
        if rows::absent(prior, "input_")? {
            return Err(Error::NotFound);
        }
        let accepted = self.accepted(prior).await?;
        let result = self.query(rows::EXECUTE, None).await?;
        if result.is_empty() {
            return Err(Error::NotFound);
        }
        let [row] = result.as_slice() else {
            return Err(Error::Unavailable);
        };
        let inserted: bool = row.try_get("inserted").map_err(sql_error)?;
        let terminal = rows::terminal(
            row,
            &accepted,
            account(self.authority.source().administrative_account_id)?,
        )?;
        if inserted {
            if terminal.xid != self.binding.source_xid
                || terminal.pid != self.binding.source_backend_pid
                || terminal.family != self.binding.session_id
                || terminal.view.epoch_before != positive(self.authority.source().company_epoch)?
                || terminal.predecessor != self.authority.source().current_policy_receipt_id
            {
                return Err(Error::Unavailable);
            }
            let mut payload = accept_payload(&accepted, terminal.family)?;
            let fields = payload.as_object_mut().ok_or(Error::Unavailable)?;
            fields.extend(json!({
                "receipt_id": terminal.view.receipt_id,
                "outcome": if matches!(terminal.view.outcome, NativePolicyOutcome::Committed(_)) { "COMMITTED" } else { "REJECTED" },
                "result_code": terminal.code,
                "epoch_before": terminal.view.epoch_before.to_string(), "epoch_after": terminal.view.epoch_after.to_string(),
                "predecessor_receipt_id": terminal.predecessor,
            }).as_object().ok_or(Error::Unavailable)?.clone());
            self.append_audit(
                trace,
                "policy.company_command.complete",
                "native_company_policy_receipts_v1",
                terminal.view.receipt_id,
                terminal.view.executed_at,
                payload,
            )
            .await?;
            if let NativePolicyOutcome::Committed(effect) = &terminal.view.outcome {
                self.head_effect = Some((terminal.view.epoch_after, terminal.view.receipt_id));
                let input = &terminal.view.accepted.input;
                let stable_key = match input {
                    NativePolicyCommand::Payroll(_) => "pay_run",
                    NativePolicyCommand::People(_) => "person",
                    NativePolicyCommand::OrgUnit(_)
                    | NativePolicyCommand::CompanyInformation(_) => {
                        return Err(Error::Unavailable);
                    }
                };
                match effect {
                    NativePolicyEffect::Installed { object_type_id } => self.append_audit(trace,
                        "ontology.object_type.builtin_install", "ont_object_types", *object_type_id,
                        terminal.view.executed_at, json!({ "stable_key": stable_key, "schema_version": 1,
                            "lifecycle_state": "published", "catalog_version": input.catalog_version(), "manifest_digest": hex(input.manifest_digest()) })).await?,
                    NativePolicyEffect::Granted { assignment, .. } => self.grant_until = Some(assignment.valid_until),
                    NativePolicyEffect::Revoked { .. } => {},
                }
            }
        }
        if terminal.view.executed_at > account_now_in_tx(&mut self.tx).await.map_err(auth_error)? {
            return Err(Error::Unavailable);
        }
        self.completed = true;
        Ok(NativePolicyExecution {
            inserted,
            terminal: terminal.view,
        })
    }

    async fn status(&mut self) -> Result<NativePolicyStatus, Error> {
        self.unused()?;
        if !matches!(self.request, NativePolicyScopeRequest::Status(_)) {
            return Err(Error::Unavailable);
        }
        let result = self.query(rows::STATUS, None).await?;
        let [row] = result.as_slice() else {
            return Err(Error::Unavailable);
        };
        let state: String = row.try_get("state").map_err(sql_error)?;
        let status = if rows::absent(row, "input_")? {
            if !rows::absent(row, "terminal_")? || state != "NotVisible" {
                return Err(Error::Unavailable);
            }
            NativePolicyStatus::NotVisible
        } else {
            let accepted = self.accepted(row).await?;
            let status = self.state_from(row, &accepted).await?;
            let valid = matches!(
                (&status, state.as_str()),
                (NativePolicyStatus::AcceptedPending(_), "AcceptedPending")
                    | (
                        NativePolicyStatus::AcceptedExpired(_),
                        "AcceptedExpired" | "AcceptedPending"
                    )
                    | (
                        NativePolicyStatus::Terminal(NativePolicyTerminalView {
                            outcome: NativePolicyOutcome::Committed(_),
                            ..
                        }),
                        "Committed"
                    )
                    | (
                        NativePolicyStatus::Terminal(NativePolicyTerminalView {
                            outcome: NativePolicyOutcome::Rejected(_),
                            ..
                        }),
                        "Rejected"
                    )
            );
            if !valid {
                return Err(Error::Unavailable);
            }
            status
        };
        self.completed = true;
        Ok(status)
    }

    async fn finish<P: CompanyPolicyDecisionPort + ?Sized>(
        mut self,
        policy: &P,
        proof: Option<&AccountFormProof>,
    ) -> Result<(), Error> {
        if !self.completed
            || matches!(
                self.request,
                NativePolicyScopeRequest::Form(_) | NativePolicyScopeRequest::ValidationForm(_)
            ) != proof.is_some()
        {
            return Err(Error::Unavailable);
        }
        sqlx::raw_sql("SET CONSTRAINTS ALL IMMEDIATE")
            .execute(self.tx.as_mut())
            .await
            .map_err(sql_error)?;
        let (current, binding) = current_source(
            &mut self.tx,
            self.config,
            self.credentials,
            &self.request,
            *self.binding.account.as_uuid(),
            self.binding.session_id,
        )
        .await?;
        same_source(self.authority.source(), current.source(), self.head_effect)?;
        if binding.source_xid != self.binding.source_xid
            || binding.source_backend_pid != self.binding.source_backend_pid
        {
            return Err(Error::Unavailable);
        }
        let selector = self.selector;
        let recipient = match &self.request {
            NativePolicyScopeRequest::Accept(command) => command.recipient_account_id(),
            _ => None,
        }
        .unwrap_or(account(current.source().administrative_account_id)?);
        let request = NativeBootstrapRequestV1::new(
            selector.company(),
            current.source().current_group_id,
            recipient,
            selector.operation(),
            *selector.manifest_digest(),
        )
        .map_err(|_| Error::Unavailable)?;
        match policy
            .decide_native_bootstrap(&current, &request)
            .map_err(|_| Error::Unavailable)?
        {
            CompanyPolicyDecision::Allow => {}
            CompanyPolicyDecision::Deny => return Err(Error::NotFound),
        }
        if let Some(proof) = proof {
            self.credentials
                .recheck_form_proof_in_tx(
                    &mut self.tx,
                    &self.config.verifier,
                    self.config.absolute_ttl,
                    proof,
                )
                .await
                .map_err(auth_error)?;
        }
        let session =
            live_session(&mut self.tx, self.config, self.credentials, &self.request).await?;
        let now = ensure_account_session_fresh_in_tx(&mut self.tx, &session)
            .await
            .map_err(auth_error)?;
        if self.pending_until.is_some_and(|until| now >= until)
            || self.grant_until.is_some_and(|until| now >= until)
        {
            return Err(Error::Conflict);
        }
        self.tx.commit().await.map_err(|error| {
            match error.as_database_error().and_then(|db| db.code()) {
                // An explicit server rollback is a known failure. Lost replies
                // and statement-completion-unknown preserve the original locator.
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
        })
    }
}

fn accept_payload(accepted: &rows::Accepted, session: Uuid) -> Result<Value, Error> {
    let input = &accepted.view.input;
    let (protocol, operation) = match input {
        NativePolicyCommand::Payroll(command) => (
            "COMPANY_BUSINESS_POLICY_V1",
            match command.operation() {
                NativeBusinessOperationV1::Install => "InstallPayrollReadCatalogV1",
                NativeBusinessOperationV1::Grant => "GrantPayrollReadV1",
                NativeBusinessOperationV1::Revoke => "RevokePayrollReadV1",
            },
        ),
        NativePolicyCommand::People(command) => (
            "COMPANY_PEOPLE_POLICY_V1",
            match command.operation() {
                NativeBusinessOperationV1::Install => "InstallPeopleDirectoryCatalogV1",
                NativeBusinessOperationV1::Grant => "GrantPeopleDirectoryV1",
                NativeBusinessOperationV1::Revoke => "RevokePeopleDirectoryV1",
            },
        ),
        NativePolicyCommand::OrgUnit(_) | NativePolicyCommand::CompanyInformation(_) => {
            return Err(Error::Unavailable);
        }
    };
    let mut payload = json!({ "protocol": protocol, "command_id": input.command_id(),
        "intake_receipt_id": accepted.view.intake_receipt_id, "operation": operation,
        "input_digest": hex(&accepted.digest), "session_id": session });
    if let NativePolicyCommand::People(command) = input {
        payload["action"] = json!(command.action().map(|action| action.as_str()));
    }
    Ok(payload)
}
fn operation_number(op: NativeBusinessOperationV1) -> i16 {
    match op {
        NativeBusinessOperationV1::Install => 1,
        NativeBusinessOperationV1::Grant => 2,
        NativeBusinessOperationV1::Revoke => 3,
    }
}
fn account(id: Uuid) -> Result<AccountId, Error> {
    AccountId::from_uuid(id).map_err(|_| Error::Unavailable)
}
fn nonnil(id: Uuid) -> Result<Uuid, Error> {
    if id.is_nil() {
        Err(Error::Unavailable)
    } else {
        Ok(id)
    }
}
fn positive(value: i64) -> Result<u64, Error> {
    if value < 1 {
        Err(Error::Unavailable)
    } else {
        Ok(value as u64)
    }
}
fn xid_number(value: &str) -> Result<u64, Error> {
    let parsed: u64 = value.parse().map_err(|_| Error::Unavailable)?;
    if parsed == 0 || parsed.to_string() != value {
        Err(Error::Unavailable)
    } else {
        Ok(parsed)
    }
}
fn exact_time(value: OffsetDateTime) -> Result<(), Error> {
    if value.unix_timestamp_nanos() % 1_000 != 0 {
        Err(Error::Unavailable)
    } else {
        Ok(())
    }
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
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
        Some(db) if db.code().as_deref() == Some("40001") => Error::Conflict,
        Some(db)
            if db.code().as_deref() == Some("42501")
                && db.message() == "native_company_policy.forbidden" =>
        {
            Error::NotFound
        }
        Some(db)
            if db.code().as_deref() == Some("22023")
                && db.message() == "native_company_policy.invalid_input" =>
        {
            Error::InvalidInput
        }
        Some(db) if db.code().as_deref() == Some("P0001") => match db.message() {
            "account.authentication_invalid" => Error::AuthenticationInvalid,
            "native_company_policy.conflict"
            | "native_company_policy.revision_conflict"
            | "native_company_policy.catalog_already_installed"
            | "native_company_policy.catalog_required"
            | "native_company_policy.assignment_active"
            | "native_company_policy.assignment_not_active"
            | "native_company_policy.recipient_ineligible" => Error::Conflict,
            "native_company_policy.grant_expiry_invalid" => Error::InvalidInput,
            "native_company_policy.capacity_exceeded" => Error::Capacity,
            _ => Error::Unavailable,
        },
        _ => Error::Unavailable,
    }
}

fn same_source(
    before: &NativeBootstrapProjectionRow,
    after: &NativeBootstrapProjectionRow,
    effect: Option<(u64, Uuid)>,
) -> Result<(), Error> {
    macro_rules! same { ($($field:ident),+ $(,)?) => { if $(before.$field != after.$field)||+ { return Err(Error::Unavailable); } }; }
    same!(
        actor_account_id,
        session_id,
        account_security_generation,
        designation_system_identifier,
        designation_database_name,
        designation_database_oid,
        designation_revision,
        designation_receipt_id,
        org_id,
        current_group_id,
        group_revision,
        group_incarnation,
        membership_id,
        membership_revision,
        membership_incarnation,
        origin_account_id,
        origin_command_id,
        origin_receipt_id,
        administrative_account_id,
        company_actor_admission_receipt_id,
        birth_assignment_id,
        birth_role_id,
        source_xid,
        source_backend_pid
    );
    let expected = effect
        .map(|(epoch, receipt)| (epoch, Some(receipt)))
        .unwrap_or((
            positive(before.company_epoch)?,
            before.current_policy_receipt_id,
        ));
    if (
        positive(after.company_epoch)?,
        after.current_policy_receipt_id,
    ) != expected
        || after.observed_at < before.observed_at
    {
        return Err(Error::Unavailable);
    }
    Ok(())
}
