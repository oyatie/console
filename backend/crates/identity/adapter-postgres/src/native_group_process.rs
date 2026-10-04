//! Group-first retained transactions for the native verification-process owner.
//! Auth and shared audit remain their existing owners; Group never arms OrgId.
use super::{
    PgOrgStore,
    company_policy::{NativeAccountMode, NativeAccountReadConfig},
};
use console_identity_application::group_process::*;
use console_kernel_core::{AuditAction, AuditEvent, AuditEventId, TraceContext};
use console_platform_auth::account::{
    AccountEnrollmentCredentials, AccountFormProof, AccountLiveSession, AccountOperationError,
    ensure_account_session_fresh_in_tx,
};
use console_platform_db::{DbError, insert_audit_event};
use serde_json::{Value, json};
use sqlx::{Postgres, Row, Transaction};
use time::OffsetDateTime;
use uuid::Uuid;

mod material;
#[cfg(test)]
mod owner_jsonb_contract_tests;
mod rows;
mod snapshot_shapes;
use rows::Error;

/// Owns the failed transaction too: a SQL exception discards it before any
/// fresh Auth-only recovery. Recovery never returns to Group acquisition.
struct Observation<'a> {
    store: &'a PgOrgStore,
    config: &'a NativeAccountReadConfig,
    credentials: &'a AccountEnrollmentCredentials,
    tx: Option<Transaction<'static, Postgres>>,
    actor: Uuid,
    family: Uuid,
}

impl<'a> Observation<'a> {
    async fn open(
        store: &'a PgOrgStore,
        credentials: &'a AccountEnrollmentCredentials,
    ) -> Result<Self, Error> {
        let config = store.native_read_config().map_err(|_| Error::Unavailable)?;
        if !matches!(config.mode, NativeAccountMode::Policy { .. }) {
            return Err(Error::Unavailable);
        }
        let mut tx = store
            .native_read_transaction()
            .await
            .map_err(|_| Error::Unavailable)?;
        sqlx::raw_sql("SET LOCAL TimeZone='UTC'; SET LOCAL bytea_output='hex'; SET LOCAL DateStyle='ISO, YMD'; SET LOCAL IntervalStyle='postgres'")
            .execute(tx.as_mut()).await.map_err(sql_error)?;
        let (actor, family) = credentials
            .session_ids_in_tx(&mut tx, &config.verifier, config.absolute_ttl)
            .await
            .map_err(auth_error)?;
        Ok(Self {
            store,
            config,
            credentials,
            tx: Some(tx),
            actor,
            family,
        })
    }
    fn tx(&mut self) -> Result<&mut Transaction<'static, Postgres>, Error> {
        self.tx.as_mut().ok_or(Error::Unavailable)
    }
    async fn sql<T>(&mut self, result: Result<T, sqlx::Error>) -> Result<T, Error> {
        match result {
            Ok(value) => Ok(value),
            Err(error) => Err(self.recover(sql_error(error)).await),
        }
    }
    async fn recover(&mut self, error: Error) -> Error {
        // No failed source transaction survives into recovery, including when
        // rollback itself cannot be confirmed. Such uncertainty fails closed.
        let rollback_ok = match self.tx.take() {
            Some(tx) => tx.rollback().await.is_ok(),
            None => false,
        };
        match recover_auth_only(self.store, self.credentials).await {
            Err(Error::AuthenticationInvalid) => Error::AuthenticationInvalid,
            Err(_) => Error::Unavailable,
            Ok(()) if !rollback_ok => Error::Unavailable,
            Ok(()) => error,
        }
    }
    async fn session(&mut self, mutation: bool) -> Result<AccountLiveSession, Error> {
        let config = self.config;
        let credentials = self.credentials;
        let session = if mutation {
            credentials
                .validate_mutation_in_tx(self.tx()?, &config.verifier, config.absolute_ttl)
                .await
        } else {
            credentials
                .read_session_in_tx(self.tx()?, &config.verifier, config.absolute_ttl)
                .await
        }
        .map_err(auth_error)?;
        if session.account_id != self.actor || session.session_id != self.family {
            return Err(Error::AuthenticationInvalid);
        }
        Ok(session)
    }
    async fn binding(&mut self, mutation: bool) -> Result<GroupProcessRetainedBindingV1, Error> {
        let session = self.session(mutation).await?;
        let result=sqlx::query_as::<_,(String,i32,OffsetDateTime)>("SELECT pg_catalog.pg_current_xact_id()::text,pg_catalog.pg_backend_pid(),clock_timestamp()")
            .fetch_one(self.tx()?.as_mut()).await;
        let (xid, pid, now) = self.sql(result).await?;
        GroupProcessRetainedBindingV1::new(
            AccountId::from_uuid(self.actor).map_err(|_| Error::Unavailable)?,
            self.family,
            u64::try_from(session.security_generation).map_err(|_| Error::Unavailable)?,
            exact_time_us(now)?,
            rows::decimal(&xid)?,
            u32::try_from(pid).map_err(|_| Error::Unavailable)?,
        )
    }
    async fn hashes(&mut self, checks: Vec<rows::HashCheck>) -> Result<(), Error> {
        if checks.is_empty() {
            return Ok(());
        }
        // Rust constructs the preimages; PostgreSQL provides the installed
        // SHA-256 primitive. No native_* SQL encoder validates its own output.
        let (bytes, digests): (Vec<Vec<u8>>, Vec<Vec<u8>>) = checks
            .into_iter()
            .map(|(bytes, digest)| (bytes, digest.to_vec()))
            .unzip();
        let result=sqlx::query_scalar::<_,bool>("SELECT coalesce(bool_and(pg_catalog.sha256(x.bytes)=x.digest),false) FROM unnest($1::bytea[],$2::bytea[]) AS x(bytes,digest)")
            .bind(bytes).bind(digests).fetch_one(self.tx()?.as_mut()).await;
        if !self.sql(result).await? {
            return Err(Error::Unavailable);
        }
        Ok(())
    }
    async fn selector(
        &mut self,
        group: GroupId,
        command: Option<Uuid>,
    ) -> Result<GroupIncarnation, Error> {
        let result = sqlx::query_scalar::<_, Uuid>(rows::SELECTOR)
            .bind(self.actor)
            .bind(self.family)
            .bind(*group.as_uuid())
            .bind(command)
            .fetch_one(self.tx()?.as_mut())
            .await;
        let id = self.sql(result).await?;
        match GroupIncarnation::from_uuid(id) {
            Ok(incarnation) => Ok(incarnation),
            Err(_) => Err(self.recover(Error::Unavailable).await),
        }
    }
    async fn navigation(
        &mut self,
        original: Option<&GroupProcessNavigationSnapshotV1>,
    ) -> Result<GroupProcessNavigationCandidatesV1, Error> {
        let result = sqlx::query_scalar::<_, String>(rows::NAVIGATION)
            .bind(self.actor)
            .bind(self.family)
            .bind(original.map(GroupProcessNavigationSnapshotV1::source_bytes))
            .fetch_one(self.tx()?.as_mut())
            .await;
        let raw = self.sql(result).await?;
        let parsed = rows::document(&raw, 2_200_000).and_then(|value| {
            material::navigation(&value, self.actor, self.family, original.is_some())
        });
        match parsed {
            Ok((value, checks)) => {
                if let Err(error) = self.hashes(checks).await {
                    return Err(if self.tx.is_some() {
                        self.recover(error).await
                    } else {
                        error
                    });
                }
                let snapshot = value.snapshot().source_bytes();
                let text = match std::str::from_utf8(snapshot) {
                    Ok(text) => text,
                    Err(_) => return Err(self.recover(Error::Unavailable).await),
                };
                let result = sqlx::query_scalar::<_, bool>(
                    "SELECT pg_catalog.convert_to($1::jsonb::text,'UTF8')=$2::bytea",
                )
                .bind(text)
                .bind(snapshot)
                .fetch_one(self.tx()?.as_mut())
                .await;
                if !self.sql(result).await? {
                    return Err(self.recover(Error::Unavailable).await);
                }
                Ok(value)
            }
            // CONFLICT is the complete, healthy protected recheck outcome.
            // Its caller still consumes fresh Auth and a confirmed read commit.
            Err(Error::Conflict) => Err(Error::Conflict),
            Err(error) => Err(self.recover(error).await),
        }
    }
    async fn source(
        &mut self,
        request: &GroupProcessScopeRequest<'_>,
    ) -> Result<
        (
            GroupProcessRetainedBindingV1,
            GroupProcessRetainedProjectionV1,
        ),
        Error,
    > {
        let input = match request {
            GroupProcessScopeRequest::Accept(input) => Some(
                input.encode(AccountId::from_uuid(self.actor).map_err(|_| Error::Unavailable)?)?,
            ),
            _ => None,
        };
        let result = sqlx::query_scalar::<_, String>(rows::MATERIAL)
            .bind(self.actor)
            .bind(self.family)
            .bind(*request.group().as_uuid())
            .bind(*request.incarnation().as_uuid())
            .bind(request.locator().map(GroupProcessLocator::command_id))
            .bind(request.mode().code())
            .bind(input)
            .fetch_one(self.tx()?.as_mut())
            .await;
        let raw = self.sql(result).await?;
        let binding = self.binding(mutation(request.mode())).await?;
        let (row, checks) = material::projection(&rows::document(&raw, 2_200_000)?, request)?;
        self.hashes(checks).await?;
        Ok((binding, row))
    }
    async fn registration(
        &mut self,
        authority: &CurrentGroupProcessAuthority,
    ) -> Result<(), Error> {
        self.hashes(vec![(
            encode_group_process_registration_v1(
                authority.group(),
                authority.incarnation(),
                authority.evaluated_bundle(),
                authority.registrations(),
            )?,
            *authority.evaluated_bundle().registration_manifest_digest(),
        )])
        .await
    }
    async fn read_commit(mut self) -> Result<(), Error> {
        let session = self.session(false).await?;
        ensure_account_session_fresh_in_tx(self.tx()?, &session)
            .await
            .map_err(auth_error)?;
        self.commit(false).await
    }
    async fn commit(&mut self, writes: bool) -> Result<(), Error> {
        let tx = self.tx.take().ok_or(Error::Unavailable)?;
        if let Err(error) = tx.commit().await {
            let known = error
                .as_database_error()
                .and_then(|db| db.code())
                .is_some_and(|code| {
                    matches!(
                        code.as_ref(),
                        "23502" | "23503" | "23505" | "23514" | "23P01" | "40001" | "40P01"
                    )
                });
            let outcome = if writes && !known {
                Error::Unconfirmed
            } else {
                Error::Unavailable
            };
            return Err(
                match recover_auth_only(self.store, self.credentials).await {
                    Err(Error::AuthenticationInvalid) => Error::AuthenticationInvalid,
                    Err(_) => Error::Unavailable,
                    Ok(()) => outcome,
                },
            );
        }
        Ok(())
    }
}

async fn recover_auth_only(
    store: &PgOrgStore,
    credentials: &AccountEnrollmentCredentials,
) -> Result<(), Error> {
    let config = store.native_read_config().map_err(|_| Error::Unavailable)?;
    let mut tx = store
        .native_read_transaction()
        .await
        .map_err(|_| Error::Unavailable)?;
    let session = credentials
        .read_session_in_tx(&mut tx, &config.verifier, config.absolute_ttl)
        .await
        .map_err(auth_error)?;
    ensure_account_session_fresh_in_tx(&mut tx, &session)
        .await
        .map_err(auth_error)?;
    tx.commit().await.map_err(|_| Error::Unavailable)
}

pub struct PgGroupProcessScope<'a> {
    observation: Observation<'a>,
    request: GroupProcessScopeRequest<'a>,
    authority: CurrentGroupProcessAuthority,
    completed: bool,
    retry_pending: bool,
    accepted: Option<GroupProcessAcceptance>,
    execution: Option<GroupProcessExecution>,
    issued_proof: Option<(String, OffsetDateTime)>,
}

impl GroupProcessStore for PgOrgStore {
    type Credentials = AccountEnrollmentCredentials;
    type FormProof = AccountFormProof;
    type Scope<'a> = PgGroupProcessScope<'a>;
    async fn resolve_current_incarnation(
        &self,
        credentials: &Self::Credentials,
        group: GroupId,
    ) -> Result<GroupIncarnation, Error> {
        let mut observation = Observation::open(self, credentials).await?;
        let incarnation = observation.selector(group, None).await?;
        observation.read_commit().await?;
        Ok(incarnation)
    }
    async fn resolve_original_locator(
        &self,
        credentials: &Self::Credentials,
        requested: GroupProcessRouteSelectorV1,
    ) -> Result<GroupProcessLocator, Error> {
        let mut observation = Observation::open(self, credentials).await?;
        let incarnation = observation
            .selector(requested.group(), Some(requested.command_id()))
            .await?;
        observation.read_commit().await?;
        GroupProcessLocator::new(requested.group(), incarnation, requested.command_id())
    }
    async fn lock<'a>(
        &'a self,
        credentials: &'a Self::Credentials,
        request: GroupProcessScopeRequest<'a>,
    ) -> Result<Self::Scope<'a>, Error> {
        let mut observation = Observation::open(self, credentials).await?;
        let (binding, row) = observation.source(&request).await?;
        let authority =
            CurrentGroupProcessAuthority::from_retained_projection(binding, &request, row)?;
        observation.registration(&authority).await?;
        Ok(PgGroupProcessScope {
            observation,
            request,
            authority,
            completed: false,
            retry_pending: false,
            accepted: None,
            execution: None,
            issued_proof: None,
        })
    }
    async fn enumerate_navigation_candidates(
        &self,
        credentials: &Self::Credentials,
    ) -> Result<GroupProcessNavigationCandidatesV1, Error> {
        let mut observation = Observation::open(self, credentials).await?;
        let candidates = observation.navigation(None).await?;
        observation.read_commit().await?;
        Ok(candidates)
    }
    async fn recheck_navigation_candidates(
        &self,
        credentials: &Self::Credentials,
        original: &GroupProcessNavigationSnapshotV1,
    ) -> Result<(), Error> {
        let mut observation = Observation::open(self, credentials).await?;
        let result = observation.navigation(Some(original)).await;
        match result {
            Ok(candidates) if candidates.snapshot() != original => {
                return Err(observation.recover(Error::Unavailable).await);
            }
            Ok(_) => {}
            Err(Error::Conflict) => {
                observation.read_commit().await?;
                return Err(Error::Conflict);
            }
            Err(error) => return Err(error),
        }
        observation.read_commit().await
    }
}

impl PgGroupProcessScope<'_> {
    fn unused(&self, mode: GroupProcessMaterialModeV1) -> Result<(), Error> {
        if self.completed || self.retry_pending {
            return Err(Error::Unavailable);
        }
        self.authority.require_method(mode)
    }
    async fn proof(&mut self) -> Result<AccountFormProof, Error> {
        let observation = &mut self.observation;
        let config = observation.config;
        let credentials = observation.credentials;
        let NativeAccountMode::Policy { issuer } = &config.mode else {
            return Err(Error::Unavailable);
        };
        let proof = credentials
            .issue_form_proof_in_tx(
                observation.tx()?,
                &config.verifier,
                issuer,
                config.absolute_ttl,
            )
            .await
            .map_err(auth_error)?;
        self.issued_proof = Some((proof.as_str().to_owned(), proof.expires_at()));
        Ok(proof)
    }
    async fn writer(&mut self, prepare: bool) -> Result<(bool, Value, Option<Value>), Error> {
        let locator = self.request.locator().ok_or(Error::Unavailable)?;
        let observation = &mut self.observation;
        let query = sqlx::query(sqlx::AssertSqlSafe(if prepare {
            rows::PREPARE
        } else {
            rows::EXECUTE
        }))
        .bind(observation.actor)
        .bind(observation.family)
        .bind(locator.command_id());
        let query = if prepare {
            let GroupProcessScopeRequest::Accept(input) = &self.request else {
                return Err(Error::Unavailable);
            };
            query.bind(input.encode(self.authority.account())?)
        } else {
            query
        };
        let result = query.fetch_all(observation.tx()?.as_mut()).await;
        let rows = observation.sql(result).await?;
        let [row] = rows.as_slice() else {
            return Err(Error::Unavailable);
        };
        let inserted: bool = row.try_get("inserted").map_err(sql_error)?;
        let accepted: String = row.try_get("accepted").map_err(sql_error)?;
        let terminal: Option<String> = row.try_get("terminal").map_err(sql_error)?;
        Ok((
            inserted,
            rows::document(&accepted, 50_000)?,
            terminal
                .as_deref()
                .map(|raw| rows::document(raw, 16_000))
                .transpose()?,
        ))
    }
    async fn append_audit(
        &mut self,
        trace: &TraceContext,
        value: &Value,
        complete: bool,
    ) -> Result<(), Error> {
        let receipt_key = if complete {
            "result_receipt_id"
        } else {
            "intake_receipt_id"
        };
        let at_key = if complete {
            "executed_at"
        } else {
            "accepted_at"
        };
        let session_key = if complete {
            "execution_session_id"
        } else {
            "accepted_session_id"
        };
        let mut payload = json!({"protocol":"GROUP_PROCESS_V1","command_id":rows::uuid(value,"command_id")?,
            "group_id":rows::uuid(value,"group_id")?,"group_incarnation":rows::uuid(value,"group_incarnation")?,
            "operation":rows::integer(value,"operation")?,"input_digest":hex(&rows::digest(value,"input_digest")?),
            "intake_receipt_id":rows::uuid(value,"intake_receipt_id")?,"session_id":rows::uuid(value,session_key)?});
        if complete {
            payload["result_receipt_id"] = json!(rows::uuid(value, "result_receipt_id")?);
            payload["terminal_code"] = json!(rows::string(value, "terminal_code")?);
        }
        let mut event = AuditEvent::new_account(
            self.authority.account(),
            AuditAction::new(if complete {
                "identity.group_process.complete"
            } else {
                "identity.group_process.accept"
            })
            .map_err(|_| Error::Unavailable)?,
            if complete {
                "native_group_process_results_v1"
            } else {
                "native_group_process_inputs_v1"
            },
            rows::uuid(value, receipt_key)?.to_string(),
            trace.clone(),
            time_from_us(rows::timestamp(value, at_key)?)?,
        )
        .with_snapshots(None, Some(payload));
        event.id = AuditEventId::from_uuid(rows::uuid(value, "audit_id")?);
        let result = insert_audit_event(self.observation.tx()?, &event).await;
        match result {
            Ok(()) => Ok(()),
            Err(DbError::Sqlx(error)) => Err(self.observation.recover(sql_error(error)).await),
            Err(_) => Err(Error::Unavailable),
        }
    }
    fn same_binding(&self, binding: GroupProcessRetainedBindingV1) -> Result<(), Error> {
        let original = self.authority.binding();
        if binding.account() != original.account()
            || binding.session() != original.session()
            || binding.security_generation() != original.security_generation()
            || binding.xid8() != original.xid8()
            || binding.backend_pid() != original.backend_pid()
            || binding.observed_at_us() < original.observed_at_us()
        {
            return Err(Error::Unavailable);
        }
        Ok(())
    }
}

impl GroupProcessScope for PgGroupProcessScope<'_> {
    type FormProof = AccountFormProof;
    fn authority(&self) -> &CurrentGroupProcessAuthority {
        &self.authority
    }
    async fn current(&mut self) -> Result<GroupProcessCurrentView, Error> {
        self.unused(GroupProcessMaterialModeV1::Current)?;
        let view = self.authority.current_view()?;
        self.completed = true;
        Ok(view)
    }
    async fn form(&mut self) -> Result<GroupProcessForm<Self::FormProof>, Error> {
        self.unused(GroupProcessMaterialModeV1::Form)?;
        let view = self.authority.current_view()?;
        let (command_id, process_id, proof) = if let GroupProcessScopeRequest::ValidationForm {
            original,
            process_id,
        } = self.request
        {
            let config = self.observation.config;
            let credentials = self.observation.credentials;
            let proof = credentials
                .retain_submitted_form_proof_in_tx(
                    self.observation.tx()?,
                    &config.verifier,
                    config.absolute_ttl,
                )
                .await
                .map_err(auth_error)?;
            self.issued_proof = Some((proof.as_str().to_owned(), proof.expires_at()));
            (original.command_id(), process_id, proof)
        } else {
            let process_id = view
                .head
                .as_ref()
                .map_or_else(Uuid::new_v4, |h| h.reference.process_id());
            (Uuid::new_v4(), process_id, self.proof().await?)
        };
        self.completed = true;
        Ok(GroupProcessForm {
            view,
            command_id,
            process_id,
            proof,
        })
    }
    async fn landing(&mut self) -> Result<GroupProcessLandingViewV1, Error> {
        self.unused(GroupProcessMaterialModeV1::Landing)?;
        let view = self.authority.current_view()?;
        let view = if view.head.is_none() {
            GroupProcessLandingViewV1::Empty(GroupProcessLandingMetadataV1 {
                context: view.context,
                allowed_actions: view.allowed_actions,
            })
        } else {
            GroupProcessLandingViewV1::Current(view)
        };
        self.completed = true;
        Ok(view)
    }
    async fn status(&mut self) -> Result<GroupProcessStatus, Error> {
        self.unused(GroupProcessMaterialModeV1::Status)?;
        let status = self
            .authority
            .original_status()
            .ok_or(Error::Unavailable)?
            .clone();
        self.completed = true;
        Ok(status)
    }
    async fn retry_form(&mut self) -> Result<GroupProcessOwnRetryFormV1<Self::FormProof>, Error> {
        self.unused(GroupProcessMaterialModeV1::OwnRetryForm)?;
        let original = self
            .authority
            .original_status()
            .and_then(GroupProcessStatus::accepted)
            .ok_or(Error::Unavailable)?
            .locator();
        let proof = self.proof().await?;
        self.completed = true;
        Ok(GroupProcessOwnRetryFormV1 { original, proof })
    }
    async fn retry_resolve(&mut self) -> Result<GroupProcessRetryResolution, Error> {
        self.unused(GroupProcessMaterialModeV1::RetryResolve)?;
        let status = self
            .authority
            .original_status()
            .ok_or(Error::Unavailable)?
            .clone();
        match status {
            GroupProcessStatus::Terminal(terminal)
                if self.authority.finish_purpose()
                    == GroupProcessFinishPurposeV1::TerminalReplay =>
            {
                self.completed = true;
                Ok(GroupProcessRetryResolution::Terminal(terminal))
            }
            GroupProcessStatus::AcceptedPending(accepted)
                if self.authority.finish_purpose() == GroupProcessFinishPurposeV1::Execute =>
            {
                self.retry_pending = true;
                Ok(GroupProcessRetryResolution::Pending(accepted))
            }
            _ => Err(Error::Unavailable),
        }
    }
    async fn accept(&mut self, trace: &TraceContext) -> Result<GroupProcessAcceptance, Error> {
        self.unused(GroupProcessMaterialModeV1::Accept)?;
        let (inserted, raw, terminal_raw) = self.writer(true).await?;
        let mut checks = Vec::new();
        let accepted = rows::accepted(&raw, &mut checks)?;
        let GroupProcessScopeRequest::Accept(input) = self.request else {
            return Err(Error::Unavailable);
        };
        if &accepted.input != input || accepted.actor != self.authority.account() {
            return Err(Error::Conflict);
        }
        if inserted {
            check_write_binding(&raw, self.authority.binding(), false)?;
            if terminal_raw.is_some() {
                return Err(Error::Unavailable);
            }
            self.append_audit(trace, &raw, false).await?;
        }
        let status = match terminal_raw {
            Some(raw) => GroupProcessStatus::Terminal(rows::terminal(&raw, accepted, &mut checks)?),
            None => GroupProcessStatus::AcceptedPending(accepted),
        };
        self.observation.hashes(checks).await?;
        let result = GroupProcessAcceptance { inserted, status };
        self.accepted = Some(result.clone());
        self.completed = true;
        Ok(result)
    }
    async fn execute(&mut self, trace: &TraceContext) -> Result<GroupProcessExecution, Error> {
        if self.completed
            || !matches!(
                self.request,
                GroupProcessScopeRequest::Execute(_) | GroupProcessScopeRequest::RetryResolve(_)
            )
            || (self.request.mode() == GroupProcessMaterialModeV1::RetryResolve
                && !self.retry_pending)
        {
            return Err(Error::Unavailable);
        }
        let (inserted, raw, terminal_raw) = self.writer(false).await?;
        if !inserted {
            return Err(Error::Unavailable);
        }
        let mut checks = Vec::new();
        let accepted = rows::accepted(&raw, &mut checks)?;
        if self.authority.original_status()
            != Some(&GroupProcessStatus::AcceptedPending(accepted.clone()))
        {
            return Err(Error::Unavailable);
        }
        let terminal_raw = terminal_raw.ok_or(Error::Unavailable)?;
        let terminal = rows::terminal(&terminal_raw, accepted, &mut checks)?;
        check_write_binding(&terminal_raw, self.authority.binding(), true)?;
        self.append_audit(trace, &terminal_raw, true).await?;
        self.observation.hashes(checks).await?;
        let result = GroupProcessExecution { inserted, terminal };
        self.execution = Some(result.clone());
        self.completed = true;
        Ok(result)
    }
    async fn finish<P: GroupProcessDecisionPort + ?Sized>(
        mut self,
        policy: &P,
        proof: Option<&Self::FormProof>,
    ) -> Result<(), Error> {
        if !self.completed {
            return Err(Error::Unavailable);
        }
        self.authority.check_finish_proof(proof.is_some())?;
        match (&self.issued_proof, proof) {
            (None, None) => {}
            (Some((issued, until)), Some(proof))
                if issued == proof.as_str() && *until == proof.expires_at() => {}
            _ => return Err(Error::Unavailable),
        }
        let result = sqlx::raw_sql("SET CONSTRAINTS ALL IMMEDIATE")
            .execute(self.observation.tx()?.as_mut())
            .await;
        self.observation.sql(result).await?;
        let (binding, row) = self.observation.source(&self.request).await?;
        self.same_binding(binding)?;
        let current = if let Some(execution) = &self.execution {
            let GroupProcessRetainedProjectionV1::Current(row) = row else {
                return Err(Error::Unavailable);
            };
            self.authority
                .execution_postimage(binding, &self.request, row, execution)?
        } else {
            let current = CurrentGroupProcessAuthority::from_retained_projection(
                binding,
                &self.request,
                row,
            )?;
            if current.current_material() != self.authority.current_material()
                || current.original_status()
                    != self
                        .accepted
                        .as_ref()
                        .map(|a| &a.status)
                        .or(self.authority.original_status())
            {
                return Err(Error::Unavailable);
            }
            current
        };
        if current.mode() != self.authority.mode()
            || current.finish_purpose() != self.authority.finish_purpose()
            || current.group() != self.authority.group()
            || current.incarnation() != self.authority.incarnation()
            || current.registration_receipt() != self.authority.registration_receipt()
            || current.current_terms_receipt() != self.authority.current_terms_receipt()
            || current.evaluated_bundle() != self.authority.evaluated_bundle()
            || current.registrations() != self.authority.registrations()
            || current.policy_request() != self.authority.policy_request()
        {
            return Err(Error::Unavailable);
        }
        self.observation.registration(&current).await?;
        current.authorize(policy)?;
        let config = self.observation.config;
        let credentials = self.observation.credentials;
        if let Some(proof) = proof {
            credentials
                .recheck_form_proof_in_tx(
                    self.observation.tx()?,
                    &config.verifier,
                    config.absolute_ttl,
                    proof,
                )
                .await
                .map_err(auth_error)?;
        }
        let submitted_proof = if mutation(self.request.mode()) {
            Some(
                credentials
                    .retain_submitted_form_proof_in_tx(
                        self.observation.tx()?,
                        &config.verifier,
                        config.absolute_ttl,
                    )
                    .await
                    .map_err(auth_error)?,
            )
        } else {
            None
        };
        let session = self
            .observation
            .session(mutation(self.request.mode()))
            .await?;
        let now = ensure_account_session_fresh_in_tx(self.observation.tx()?, &session)
            .await
            .map_err(auth_error)?;
        if proof
            .or(submitted_proof.as_ref())
            .is_some_and(|proof| proof.expires_at() <= now)
        {
            return Err(Error::CsrfInvalid);
        }
        let now_us = exact_time_us(now)?;
        if current
            .original_status()
            .and_then(GroupProcessStatus::accepted)
            .is_some_and(|a| a.accepted_at_us > now_us)
            || self.execution.as_ref().is_some_and(|e| {
                e.terminal.result.executed_at_us() > now_us
                    || (e.terminal.result.terminal_code().is_success()
                        && e.terminal
                            .result
                            .after_head()
                            .is_none_or(|head| head.expiry_us() <= now_us))
            })
        {
            return Err(Error::Unavailable);
        }
        self.observation
            .commit(self.accepted.as_ref().is_some_and(|a| a.inserted) || self.execution.is_some())
            .await
    }
}

fn mutation(mode: GroupProcessMaterialModeV1) -> bool {
    matches!(
        mode,
        GroupProcessMaterialModeV1::Accept
            | GroupProcessMaterialModeV1::Execute
            | GroupProcessMaterialModeV1::RetryResolve
    )
}
fn check_write_binding(
    value: &Value,
    binding: GroupProcessRetainedBindingV1,
    complete: bool,
) -> Result<(), Error> {
    let (xid, pid, session, at) = if complete {
        (
            "effect_xid",
            "effect_backend_pid",
            "execution_session_id",
            "executed_at",
        )
    } else {
        (
            "acceptance_xid",
            "acceptance_backend_pid",
            "accepted_session_id",
            "accepted_at",
        )
    };
    if rows::account(value, "actor_account_id")? != binding.account()
        || rows::uuid(value, session)? != binding.session()
        || rows::positive(value, "account_security_generation")? != binding.security_generation()
        || rows::decimal(rows::string(value, xid)?)? != binding.xid8()
        || rows::u32_value(value, pid)? != binding.backend_pid()
        || rows::timestamp(value, at)? < binding.observed_at_us()
    {
        return Err(Error::Unavailable);
    }
    Ok(())
}
fn hex(value: &[u8]) -> String {
    use std::fmt::Write;
    let mut output = String::with_capacity(value.len() * 2);
    for byte in value {
        let _ = write!(output, "{byte:02x}");
    }
    output
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
        Some(db) if db.message() == "account.authentication_invalid" => {
            Error::AuthenticationInvalid
        }
        Some(db)
            if db.message() == "native_group_process.not_found"
                && db.code().as_deref() == Some("42501") =>
        {
            Error::NotFound
        }
        Some(db)
            if db.message() == "native_group_process.invalid_input"
                && db.code().as_deref() == Some("22023") =>
        {
            Error::InvalidInput
        }
        Some(db) if db.message() == "native_group_process.conflict" => Error::Conflict,
        _ => Error::Unavailable,
    }
}

#[cfg(test)]
mod navigation_contract_tests;
