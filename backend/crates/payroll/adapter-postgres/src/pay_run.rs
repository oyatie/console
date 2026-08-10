//! `PayRunPort` — the Postgres implementation of `ObjectKey::PayRun`, and the
//! home of the ONE `payroll_draft_runs` statement that used to live outside the
//! owning crate.
//!
//! Owned tables, verbatim from the contract
//! (`backend/crates/ontology/canonical-domain/src/lib.rs`):
//! `payroll_draft_runs`, `payroll_draft_lines`, `payroll_line_calculations`,
//! `payroll_run_exceptions`, `payroll_disbursements`,
//! `payroll_payslip_deliveries`. Every one of them already exists (migrations
//! 0074 and 0186); this lane creates no table and appends to no table list.
//!
//! # This port WRAPS the existing writer; it does not become a second one
//!
//! The contract is explicit that `PayRunPort` reuses the payroll tables rather
//! than adding a parallel set, and the writer-ownership gate exists to reject
//! exactly the defect a fresh set of statements here would be. So:
//!
//! * `payroll.submit_run` calls [`crate::lifecycle::submit_run_in_tx`],
//! * `payroll.decide_run` calls [`crate::lifecycle::decide_run_in_tx`],
//!
//! both already in this crate, both already carrying their own state machine
//! (`CALCULATED → SUBMITTED → APPROVED/REJECTED`), their exception gate and
//! their segregation-of-duties check. The port adds the canonical receipt and
//! the idempotency replay around them; it does not restate their SQL.
//!
//! `payroll.create_run` is the one target with no pre-existing owner statement,
//! because the statement that created a draft run lived in
//! `console-workflow-runtime-adapter-postgres` — the ratcheted dual writer this
//! lane removes. [`stage_draft_run_in_tx`] is that statement, moved here
//! unchanged in meaning, and it now has two callers: this port, and the JOB
//! outbox drain through [`PgPayRunPort`]'s [`PayrollDraftStaging`] impl.
//!
//! # Why the drain reaches this crate through a trait
//!
//! `backend/ci/gates/layer-boundary` forbids an adapter crate depending on
//! another adapter crate (`Layer::Adapter`'s allowed edges are application,
//! contracts, domain, kernel and platform — not adapter). The workflow runtime
//! adapter therefore CANNOT name `console-payroll-adapter-postgres` as a
//! dependency, and a "just call the owner" edit would have swapped a
//! writer-ownership violation for a layer violation.
//!
//! The seam is [`console_workflow_domain::PayrollDraftStaging`], declared by the
//! CONSUMER in its own domain crate — the same shape as
//! `console_notifications_application::NotificationSink`, which the very same
//! drain loop already uses for the notification bridge. `console-app` wires the
//! implementer in, exactly as it wires the notification sink.
//!
//! # What moved, and the one property that changed with it
//!
//! Before: claim → INSERT draft → ack DELIVERED → audit, all in ONE
//! `with_audits` transaction. After: claim (its own read transaction) → stage
//! the draft through this port (its own transaction) → ack + audit
//! (`with_audits`). The staging happens BEFORE the ack, and the ack no longer
//! shares the draft's transaction.
//!
//! That is safe in the direction that matters and the ORDER is what makes it so:
//!
//! * crash after staging, before the ack — the event stays `PENDING`, the next
//!   tick re-claims it, [`stage_draft_run_in_tx`]'s
//!   `ON CONFLICT (org_id, period_start, period_end, source_label) DO NOTHING`
//!   makes the restage a no-op returning `false`, and the ack then lands. No
//!   double draft, no lost draft;
//! * crash before staging — nothing was written and the event is still
//!   `PENDING`.
//!
//! The reverse order would be the unsafe one: acking first could lose a draft
//! forever. The property genuinely given up is "the draft and its ack roll back
//! together", which was tidiness rather than safety — the natural key was always
//! the exactly-once mechanism, as 0074's `UNIQUE (org_id, period_start,
//! period_end, source_label)` and the drain's own doc comment already said. It
//! is the identical trade `drain_notification_outbox` documents three functions
//! further down the same file ("emit + mark are each idempotent, so a re-claim
//! is safe").
//!
//! # Where the receipt is stored
//!
//! In `ont_action_command_receipts`, the 0177 store, exactly as `PersonPort` and
//! `EmploymentPort` do. `PRIMARY KEY (org_id, command_id)` is what makes a
//! command id tenant-global ACROSS owners, so one client idempotency key can
//! never mean two accepted commands. 0177 carries no `owner`/`target` column
//! yet, so the `DispatchTarget` travels inside the receipt JSONB as its wire
//! string and read-back parses it with `FromStr`. A stored row that names no
//! target is refused, never replayed.
//!
//! # Synchronous port, async driver
//!
//! `CanonicalPort::execute` is synchronous and `sqlx` is async-only, so
//! [`PgPayRunPort`] holds a `tokio::runtime::Handle` and blocks on it.
//! `Handle::block_on` panics when called from a runtime worker thread; an async
//! caller must reach `execute` through `spawn_blocking`. The
//! [`PayrollDraftStaging`] impl is async and does NOT go through `execute`, so
//! the drain — which is already on a runtime — never blocks a worker.
//! ponytail: one runtime handle, no thread pool of its own — revisit only if the
//! trait ever gains an async form.

use console_kernel_core::{OrgId, UserId};
use console_ontology_canonical_domain::{
    CanonicalPort, CanonicalQuery, CommandId, CommandReceipt, DispatchTarget, ObjectKey, PayRun,
    Preflight, ReceiptOwner,
};
use console_workflow_domain::{PayrollDraftStaging, PortFuture, StagePayrollDraft};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Postgres, Row, Transaction};
use std::str::FromStr;
use time::{Date, OffsetDateTime};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// The one statement that moved
// ---------------------------------------------------------------------------

/// Stages one `payroll_draft_runs` row inside the CALLER's transaction,
/// idempotently on the natural key. `true` when this call created the row.
///
/// The status is `BLOCKED_LEGAL_GATE` and `calculation_enabled` is left at its
/// column default (`FALSE`), so nothing calculates without the legal gate —
/// 0074's `payroll_draft_runs_enabled_requires_review` CHECK would refuse the
/// combination anyway, and stating the status rather than defaulting it is what
/// the workflow drain did.
///
/// `period_start`/`period_end` are bound as `Option`: an outbox payload that
/// carries neither produces the same `23502 not_null_violation` the previous
/// `(payload->>'period_start')::date` form produced, rather than a silently
/// defaulted period.
///
/// # Errors
/// Returns the driver error verbatim so the caller can distinguish a missing
/// period (`23502`) from a tenant it may not write (`42501`).
pub async fn stage_draft_run_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    org_id: Uuid,
    draft: &StagePayrollDraft,
) -> Result<bool, sqlx::Error> {
    stage_draft_run_returning_id_in_tx(tx, org_id, draft)
        .await
        .map(|(_, created)| created)
}

/// The same staging INSERT, returning the row's own identifier.
///
/// # Why this exists, and why `DO NOTHING` could not be kept
///
/// `payroll_draft_runs.id` is `DEFAULT gen_random_uuid()` (migration 0074), and
/// `StagePayrollDraft::run_id` is a different identifier entirely: the workflow
/// drain's deterministic natural key, which reaches this table only inside
/// `source_label` and the `source_summary` JSONB. That split is by design and
/// predates the canonical port.
///
/// What did NOT predate it is a caller who must go from one to the other.
/// `PayRunQuery::SubmitRun` and `DecideRun` resolve through `run_head`, which is
/// `WHERE id = $1` against the PRIMARY KEY, so a `CreateRun` receipt carrying
/// only the workflow run id names a row that `WHERE id` can never find. The
/// three targets are one lifecycle and must be traversable in order.
///
/// `DO NOTHING` cannot serve that: on conflict it returns no row, so precisely
/// the idempotent replay — the case the receipt store exists for — would get no
/// identifier. The conflict arm is therefore a no-op `DO UPDATE` that rewrites
/// `source_label` to the value it already holds, purely so `RETURNING` yields
/// the row. `xmax = 0` distinguishes a fresh insert from a conflict, which is
/// what the previous `is_some()` meant.
pub async fn stage_draft_run_returning_id_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    org_id: Uuid,
    draft: &StagePayrollDraft,
) -> Result<(Uuid, bool), sqlx::Error> {
    let row: (Uuid, bool) = sqlx::query_as(
        "INSERT INTO payroll_draft_runs \
             (org_id, period_start, period_end, source_label, status, source_summary) \
         VALUES ($1, $2, $3, $4, 'BLOCKED_LEGAL_GATE', $5) \
         ON CONFLICT (org_id, period_start, period_end, source_label) \
         DO UPDATE SET source_label = EXCLUDED.source_label \
         RETURNING id, (xmax = 0) AS created",
    )
    .bind(org_id)
    .bind(draft.period_start)
    .bind(draft.period_end)
    .bind(draft.source_label())
    .bind(serde_json::json!({
        "outbox_event_id": draft.outbox_event_id,
        "run_id": draft.run_id,
        "connector": draft.connector,
        "job": draft.job,
    }))
    .fetch_one(tx.as_mut())
    .await?;
    Ok(row)
}

// ---------------------------------------------------------------------------
// The canonical port
// ---------------------------------------------------------------------------

/// The two decisions `payroll.decide_run` admits, spelled here so a typo is a
/// preflight blocker rather than a `LifecycleError::Validation` discovered after
/// a transaction is already open.
const DECISIONS: [&str; 2] = ["APPROVE", "REJECT"];

/// The typed read this port answers: the write a caller intends, one variant per
/// dispatch target the contract assigns to `PayRun`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "target")]
pub enum PayRunQuery {
    /// `payroll.create_run`. Stages a draft run under a caller-chosen source
    /// label, idempotent on the natural key.
    #[serde(rename = "payroll.create_run")]
    CreateRun {
        run_id: Uuid,
        period_start: Date,
        period_end: Date,
        #[serde(default)]
        connector: Option<String>,
        #[serde(default)]
        job: Option<String>,
    },
    /// `payroll.submit_run`. `CALCULATED → SUBMITTED`, refused while any
    /// exception is open.
    #[serde(rename = "payroll.submit_run")]
    SubmitRun { run_id: Uuid },
    /// `payroll.decide_run`. `SUBMITTED → APPROVED|REJECTED`, refused when the
    /// decider is the submitter.
    #[serde(rename = "payroll.decide_run")]
    DecideRun {
        run_id: Uuid,
        decision: String,
        #[serde(default)]
        reason: Option<String>,
    },
}

impl PayRunQuery {
    /// The dispatch target this query is, spelled once in `canonical-domain`.
    #[must_use]
    pub const fn target(&self) -> DispatchTarget {
        match self {
            Self::CreateRun { .. } => DispatchTarget::PayrollCreateRun,
            Self::SubmitRun { .. } => DispatchTarget::PayrollSubmitRun,
            Self::DecideRun { .. } => DispatchTarget::PayrollDecideRun,
        }
    }

    #[must_use]
    pub const fn run_id(&self) -> Uuid {
        match self {
            Self::CreateRun { run_id, .. }
            | Self::SubmitRun { run_id }
            | Self::DecideRun { run_id, .. } => *run_id,
        }
    }
}

impl CanonicalQuery for PayRunQuery {
    fn dispatch_target(&self) -> DispatchTarget {
        self.target()
    }
}

/// The typed write this port accepts. `org_id` is the RLS key and `command_id`
/// the tenant-global idempotency key; a repeat replays the stored receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PayRunCommand {
    pub org_id: OrgId,
    pub command_id: CommandId,
    pub actor_id: UserId,
    pub query: PayRunQuery,
}

#[derive(Debug, thiserror::Error)]
pub enum PayRunError {
    #[error("preflight blocked the command: {0:?}")]
    Blocked(Vec<String>),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Lifecycle(#[from] crate::lifecycle::LifecycleError),
    #[error("command {0} was already applied with a different payload")]
    DigestConflict(Uuid),
    #[error("stored receipt for command {0} names no dispatch target: {1}")]
    UnreadableReceipt(Uuid, String),
}

/// The one permitted holder of production DML against the six `payroll_*` tables
/// the contract assigns to `ObjectKey::PayRun`.
#[derive(Debug, Clone)]
pub struct PgPayRunPort {
    pool: PgPool,
    runtime: tokio::runtime::Handle,
}

impl PgPayRunPort {
    #[must_use]
    pub const fn new(pool: PgPool, runtime: tokio::runtime::Handle) -> Self {
        Self { pool, runtime }
    }

    async fn write(&self, command: &PayRunCommand) -> Result<CommandReceipt, PayRunError> {
        let preflight = <Self as CanonicalPort>::preflight(&command.query);
        if !preflight.is_ok() {
            return Err(PayRunError::Blocked(preflight.blockers().to_vec()));
        }

        let digest = payload_digest(command);
        let org = *command.org_id.as_uuid();
        let actor = *command.actor_id.as_uuid();
        let command_uuid = *command.command_id.as_uuid();

        let mut tx = self.pool.begin().await?;
        // Transaction-local, so it is cleared on COMMIT/ROLLBACK and never leaks
        // to the next checkout of a pooled connection. Unset fails closed: RLS
        // shows no rows and accepts no writes.
        sqlx::query("SELECT set_config('app.current_org', $1, true)")
            .bind(org.to_string())
            .execute(&mut *tx)
            .await?;

        if let Some(stored) = sqlx::query(
            "SELECT actor_id, payload_digest, receipt, created_at \
             FROM ont_action_command_receipts WHERE org_id = $1 AND command_id = $2",
        )
        .bind(org)
        .bind(command_uuid)
        .fetch_optional(&mut *tx)
        .await?
        {
            let stored_digest: Vec<u8> = stored.get("payload_digest");
            if stored_digest != digest {
                return Err(PayRunError::DigestConflict(command_uuid));
            }
            let result: serde_json::Value = stored.get("receipt");
            let target = stored_target(command_uuid, &result)?;
            let stored_actor: Uuid = stored.get("actor_id");
            let created_at: OffsetDateTime = stored.get("created_at");
            return Ok(receipt(
                command,
                target,
                UserId::from_uuid(stored_actor),
                digest,
                result,
                created_at,
            ));
        }

        let target = command.query.target();
        let run_id = command.query.run_id();
        // The row's own identifier, which is NOT `run_id`: see
        // `stage_draft_run_returning_id_in_tx`. Only CreateRun learns it, because only CreateRun
        // makes the row; Submit and Decide were handed it by a prior CreateRun receipt.
        let mut draft_run_id: Option<Uuid> = None;

        // The three arms are the three targets, and each one is a CALL into the
        // statements this crate already owns — never a restatement of them.
        let created = match &command.query {
            PayRunQuery::CreateRun {
                period_start,
                period_end,
                connector,
                job,
                ..
            } => {
                let draft = StagePayrollDraft {
                    org: command.org_id,
                    outbox_event_id: command_uuid,
                    run_id,
                    period_start: Some(*period_start),
                    period_end: Some(*period_end),
                    connector: connector.clone(),
                    job: job.clone(),
                };
                let (id, created) =
                    stage_draft_run_returning_id_in_tx(&mut tx, org, &draft).await?;
                draft_run_id = Some(id);
                created
            }
            PayRunQuery::SubmitRun { .. } => {
                crate::lifecycle::submit_run_in_tx(&mut tx, run_id, actor).await?;
                true
            }
            PayRunQuery::DecideRun {
                decision, reason, ..
            } => {
                crate::lifecycle::decide_run_in_tx(
                    &mut tx,
                    run_id,
                    actor,
                    decision,
                    reason.as_deref(),
                )
                .await?;
                true
            }
        };

        // `run_id` is echoed back because the caller chose it and correlates on it. `draft_run_id`
        // is the value `SubmitRun`/`DecideRun` must be given: they resolve through `run_head`, which
        // keys the PRIMARY KEY. Emitting only `run_id` shipped a create whose own successors could
        // never resolve its output, and the port's test masked it by fetching the id behind the port
        // through the owner pool -- a back channel no caller on the action surface has.
        let mut result = serde_json::json!({
            "run_id": run_id.to_string(),
            "created": created,
            "target": target.as_str(),
        });
        if let Some(id) = draft_run_id {
            result["draft_run_id"] = serde_json::Value::String(id.to_string());
        }

        // The receipt store, and with it the tenant-global command-id namespace
        // this port shares with every other receipt owner. `created_at` is
        // supplied rather than defaulted: 0177 declares it `NOT NULL` with NO
        // DEFAULT, so an INSERT that omits it is a `23502`.
        let created_at: OffsetDateTime = sqlx::query_scalar(
            "INSERT INTO ont_action_command_receipts \
             (org_id, command_id, actor_id, payload_digest, receipt, created_at) \
             VALUES ($1, $2, $3, $4, $5, now()) RETURNING created_at",
        )
        .bind(org)
        .bind(command_uuid)
        .bind(actor)
        .bind(digest.as_slice())
        .bind(&result)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(receipt(
            command,
            target,
            command.actor_id,
            digest,
            result,
            created_at,
        ))
    }
}

impl CanonicalPort for PgPayRunPort {
    type Object = PayRun;
    type Query = PayRunQuery;
    type Command = PayRunCommand;
    type Error = PayRunError;

    /// PURE: no `&self`, no IO, no persistence. A blocked preflight has written
    /// nothing, so it can never spend an approval.
    fn preflight(query: &Self::Query) -> Preflight {
        let mut blockers = Vec::new();
        if query.run_id().is_nil() {
            blockers.push("run_id must not be nil".to_owned());
        }
        match query {
            PayRunQuery::CreateRun {
                period_start,
                period_end,
                ..
            } => {
                // 0074's `payroll_draft_runs_valid_period` CHECK, asked here so
                // an inverted period never opens a transaction.
                if period_end < period_start {
                    blockers.push("period_end must not precede period_start".to_owned());
                }
            }
            PayRunQuery::DecideRun {
                decision, reason, ..
            } => {
                if !DECISIONS.contains(&decision.as_str()) {
                    blockers.push(format!("decision must be one of {DECISIONS:?}"));
                }
                if decision == "REJECT" && reason.as_ref().is_none_or(|r| r.trim().is_empty()) {
                    blockers.push("a reason is required to reject a payroll run".to_owned());
                }
            }
            PayRunQuery::SubmitRun { .. } => {}
        }
        if blockers.is_empty() {
            Preflight::ok()
        } else {
            Preflight::blocked(blockers)
        }
    }

    fn command(
        org_id: OrgId,
        command_id: CommandId,
        actor_id: UserId,
        query: Self::Query,
    ) -> Self::Command {
        PayRunCommand {
            org_id,
            command_id,
            actor_id,
            query,
        }
    }

    fn execute(&self, command: &Self::Command) -> Result<CommandReceipt, Self::Error> {
        self.runtime.block_on(self.write(command))
    }
}

/// The seam the JOB outbox drain reaches this crate through. One transaction per
/// draft, armed for the tenant, idempotent on the natural key — which is what
/// lets the drain stage before it acks and still be exactly-once.
impl PayrollDraftStaging for PgPayRunPort {
    fn stage<'a>(&'a self, draft: StagePayrollDraft) -> PortFuture<'a, bool> {
        Box::pin(async move {
            let org = draft.org;
            console_platform_db::with_org_conn::<_, bool, crate::PgPayrollError>(
                &self.pool,
                org,
                move |tx| {
                    Box::pin(async move {
                        stage_draft_run_in_tx(tx, *org.as_uuid(), &draft)
                            .await
                            .map_err(crate::PgPayrollError::from)
                    })
                },
            )
            .await
            .map_err(console_kernel_core::KernelError::from)
        })
    }
}

fn receipt(
    command: &PayRunCommand,
    target: DispatchTarget,
    actor_id: UserId,
    digest: [u8; 32],
    result: serde_json::Value,
    created_at: OffsetDateTime,
) -> CommandReceipt {
    CommandReceipt::new(
        command.org_id,
        command.command_id,
        ReceiptOwner::Canonical(ObjectKey::PayRun),
        target,
        actor_id,
        digest,
        result,
        created_at,
    )
}

/// The target a stored receipt names, read back through the roster's `FromStr`
/// rather than re-spelled here.
fn stored_target(
    command_id: Uuid,
    result: &serde_json::Value,
) -> Result<DispatchTarget, PayRunError> {
    let stored = result["target"]
        .as_str()
        .ok_or_else(|| PayRunError::UnreadableReceipt(command_id, result.to_string()))?;
    DispatchTarget::from_str(stored)
        .map_err(|error| PayRunError::UnreadableReceipt(command_id, error.to_string()))
}

/// The 32 bytes the `payload_digest` CHECK is sized for.
///
/// Every field is hashed in a fixed order from TYPED values, never from a
/// `serde_json::Value`: `serde_json` resolves with `preserve_order` in this
/// workspace, so a `Value` serialises its object keys in INSERTION order and two
/// payloads that compare EQUAL serialise to different bytes. The retry a client
/// performs after a timeout must digest to the same 32 bytes, or it comes back
/// as a [`PayRunError::DigestConflict`] instead of the documented replay.
/// `None` and `Some("")` are separated by a tag byte, so an absent reason and a
/// blank one are not the same command.
fn payload_digest(command: &PayRunCommand) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(command.org_id.as_uuid().as_bytes());
    hasher.update(command.command_id.as_uuid().as_bytes());
    hasher.update(command.actor_id.as_uuid().as_bytes());
    hasher.update(command.query.target().as_str().as_bytes());
    hasher.update(command.query.run_id().as_bytes());
    let fields: Vec<Option<String>> = match &command.query {
        PayRunQuery::CreateRun {
            period_start,
            period_end,
            connector,
            job,
            ..
        } => vec![
            Some(period_start.to_string()),
            Some(period_end.to_string()),
            connector.clone(),
            job.clone(),
        ],
        PayRunQuery::SubmitRun { .. } => Vec::new(),
        PayRunQuery::DecideRun {
            decision, reason, ..
        } => vec![Some(decision.clone()), reason.clone()],
    };
    for field in fields {
        match field {
            Some(value) => {
                hasher.update([1_u8]);
                hasher.update((value.len() as u64).to_be_bytes());
                hasher.update(value.as_bytes());
            }
            None => hasher.update([0_u8]),
        }
    }
    hasher.finalize().into()
}
