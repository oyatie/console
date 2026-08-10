//! `EmploymentPort` — the Postgres implementation of `ObjectKey::Employment`,
//! and the home of every `employees` statement that used to live outside the
//! owning crate. The one `employees` write NOT here is `apply_op`'s
//! `ReassignOrgUnit` arm in this crate's `src/lib.rs`, which the contract's own
//! doc comment names and which is already inside the owner.
//!
//! Owned tables, verbatim from the contract
//! (`backend/crates/ontology/canonical-domain/src/lib.rs`): `employees`,
//! `employment_heads`, `employment_revisions`, `employment_source_bindings`.
//! `employees` is the LEGACY COMPATIBILITY HEAD — 0063 created it, the rest of
//! the tree still reads it, and `employment_source_bindings` is the surface
//! that resolves a canonical employment back to it.
//!
//! # Why the two legacy-head writes are public
//!
//! Before this module, `console-app` held three `employees` statements in
//! `backend/app/src/hr.rs` and `console-gate-writer-ownership` recorded that as
//! a ratcheted dual writer. The REST handlers own the surrounding transaction —
//! the employee row, its employment profile, its lifecycle event, the audit
//! chain and the idempotency reservation all commit or roll back together — so
//! moving those writes behind a port that opens its own connection would have
//! traded a writer-ownership violation for an atomicity one.
//!
//! [`insert_employee_record`] and [`apply_employment_change`] therefore take the
//! CALLER's transaction and hold the SQL here, in the owning crate. They are not
//! wrappers around the port: they are the port's own legacy-head writes, called
//! by [`PgEmploymentPort::execute`] for `hr.promote` and `hr.transfer` and by
//! the REST handlers for the same change arriving over HTTP. One statement, one
//! crate, two callers.
//!
//! Three statements collapsed to two: the confirmed-exit write was
//! `SET employment_status = 'EXITED', exit_date = $3` and the lifecycle write
//! is that plus the company/org-unit/position triple, whose `CASE WHEN $6 =
//! 'EXITED'` arm sets the same `exit_date`. Passing the employee's current
//! company/org-unit/position through the lifecycle statement produces the
//! identical row, so the exit path is a call to the general one.
//!
//! # Append-only, enforced by the database
//!
//! 0214's `canonical_employment_row_immutable` trigger refuses UPDATE and DELETE
//! on `employment_revisions` and UPDATE on `employment_source_bindings`, so a
//! revision is never edited: `Promote`/`Transfer` append `MAX(version) + 1`.
//! `employment_heads` deliberately carries no trigger — `valid_to` is the one
//! legitimate mutation — and this port does not close a window yet.
//!
//! A revision stores only `valid_from`; its half-open interval ends at the next
//! revision's `valid_from`, or at the head's `valid_to`. That is 0214's temporal
//! contract, and it is why there is no `valid_to` on the revision to write.
//!
//! # Where the receipt is stored
//!
//! In `ont_action_command_receipts`, the generalised 0177 store, exactly as
//! `PersonPort` does: `PRIMARY KEY (org_id, command_id)` is what makes a command
//! id tenant-global ACROSS owners, so one client idempotency key can never mean
//! two accepted commands. 0177 carries no `owner`/`target` column yet, so the
//! `DispatchTarget` travels inside the receipt JSONB as its wire string and
//! read-back parses it with `FromStr`. A stored row that names no target is
//! refused, never replayed.
//!
//! # Synchronous port, async driver
//!
//! `CanonicalPort::execute` is synchronous and `sqlx` is async-only, so
//! [`PgEmploymentPort`] holds a `tokio::runtime::Handle` and blocks on it.
//! `Handle::block_on` panics when called from a runtime worker thread; an async
//! caller must reach `execute` through `spawn_blocking`.
//! ponytail: one runtime handle, no thread pool of its own — revisit only if the
//! trait ever gains an async form.

use console_kernel_core::{OrgId, UserId};
use console_ontology_canonical_domain::{
    CanonicalPort, CanonicalQuery, CommandId, CommandReceipt, DispatchTarget, Employment,
    ObjectKey, Preflight, ReceiptOwner,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Postgres, Row, Transaction};
use std::str::FromStr;
use time::OffsetDateTime;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// The legacy compatibility head
// ---------------------------------------------------------------------------

/// The `employees` row an appointment creates.
///
/// `home_branch_id` is deliberately absent: since 0166 the branch routing
/// authority is command-only and the `console_rt` guard rejects it here, so the
/// caller establishes it post-commit.
#[derive(Debug, Clone, Copy)]
pub struct NewEmployeeRecord<'a> {
    pub employee_id: Uuid,
    pub company: &'a str,
    pub name: &'a str,
    pub employee_number: &'a str,
    pub org_unit: &'a str,
    pub position: &'a str,
    pub worksite_name: &'a str,
}

/// One employment change applied to the legacy head: the company/org-unit/
/// position triple plus the status, with `effective_date` becoming `exit_date`
/// exactly when the status is `EXITED`.
#[derive(Debug, Clone, Copy)]
pub struct EmploymentChange<'a> {
    pub company: &'a str,
    pub org_unit: Option<&'a str>,
    pub position: Option<&'a str>,
    pub employment_status: &'a str,
    pub effective_date: &'a str,
}

/// Creates the legacy `employees` row for a new appointment, inside the
/// caller's transaction.
///
/// # Errors
/// Returns the driver error verbatim so the caller can map `23505` (employee
/// number already in use) to its own conflict.
pub async fn insert_employee_record(
    tx: &mut Transaction<'_, Postgres>,
    org_id: Uuid,
    record: NewEmployeeRecord<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO employees (
            id, org_id, company, name, employee_number, org_unit, position,
            worksite_name, source_filename, source_sheet,
            source_row, source_key, raw_row, source_metadata,
            identity_resolution_strategy, identity_resolution_confidence,
            identity_review_required, identity_name_only_merge
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, 'console', 'people', 1,
            $9, '{}'::jsonb, '{}'::jsonb, 'employee_number', 'high', FALSE, FALSE
        )"#,
    )
    .bind(record.employee_id)
    .bind(org_id)
    .bind(record.company)
    .bind(record.name)
    .bind(record.employee_number)
    .bind(record.org_unit)
    .bind(record.position)
    .bind(record.worksite_name)
    .bind(format!("console:{}", record.employee_number))
    .execute(tx.as_mut())
    .await
    .map(|_| ())
}

/// Applies one employment change to the legacy `employees` head, inside the
/// caller's transaction.
///
/// # Errors
/// Returns the driver error verbatim.
pub async fn apply_employment_change(
    tx: &mut Transaction<'_, Postgres>,
    org_id: Uuid,
    employee_id: Uuid,
    change: EmploymentChange<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE employees
        SET
            company = $3,
            org_unit = $4,
            position = $5,
            employment_status = $6,
            exit_date = CASE WHEN $6 = 'EXITED' THEN $7 ELSE exit_date END,
            updated_at = now()
        WHERE org_id = $1 AND id = $2
        "#,
    )
    .bind(org_id)
    .bind(employee_id)
    .bind(change.company)
    .bind(change.org_unit)
    .bind(change.position)
    .bind(change.employment_status)
    .bind(change.effective_date)
    .execute(tx.as_mut())
    .await
    .map(|_| ())
}

// ---------------------------------------------------------------------------
// The canonical port
// ---------------------------------------------------------------------------

/// The canonical state of one employment at one version. Typed rather than raw
/// JSONB because the port applies the same four values to the legacy head, and
/// `employees.employment_status` carries a CHECK that a free-form payload would
/// only discover at 23514.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct EmploymentAttributes {
    pub company: String,
    pub org_unit: Option<String>,
    pub position: Option<String>,
    pub employment_status: String,
}

/// The three statuses `employees.employment_status` admits (0066's CHECK).
const EMPLOYMENT_STATUSES: [&str; 3] = ["ACTIVE", "EXITED", "UNKNOWN"];

impl EmploymentAttributes {
    /// The revision's `attributes` JSONB. Keys are written in sorted order, so
    /// the value and its digest do not depend on this struct's field order.
    #[must_use]
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "company": self.company,
            "employment_status": self.employment_status,
            "org_unit": self.org_unit,
            "position": self.position,
        })
    }

    fn as_change<'a>(&'a self, effective_date: &'a str) -> EmploymentChange<'a> {
        EmploymentChange {
            company: &self.company,
            org_unit: self.org_unit.as_deref(),
            position: self.position.as_deref(),
            employment_status: &self.employment_status,
            effective_date,
        }
    }
}

/// The typed read this port answers: the write a caller intends, and nothing
/// about how it is performed. Each variant is bound to exactly one of the three
/// dispatch targets the contract assigns to `Employment`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "target")]
pub enum EmploymentQuery {
    /// `hr.appoint`. Opens a head at `valid_from`, appends revision 1, and binds
    /// the head to the legacy `employees` row this employment IS. The legacy row
    /// is not rewritten: it was just created with these values.
    #[serde(rename = "hr.appoint")]
    Appoint {
        employee_id: Uuid,
        #[serde(with = "time::serde::rfc3339")]
        valid_from: OffsetDateTime,
        attributes: EmploymentAttributes,
    },
    /// `hr.promote`. Appends a revision to an existing head and carries the new
    /// state onto the legacy head.
    #[serde(rename = "hr.promote")]
    Promote {
        employment_id: Uuid,
        #[serde(with = "time::serde::rfc3339")]
        valid_from: OffsetDateTime,
        attributes: EmploymentAttributes,
    },
    /// `hr.transfer`. Same shape as a promotion — the difference is the
    /// dispatch target the receipt records, which is what an auditor reads.
    #[serde(rename = "hr.transfer")]
    Transfer {
        employment_id: Uuid,
        #[serde(with = "time::serde::rfc3339")]
        valid_from: OffsetDateTime,
        attributes: EmploymentAttributes,
    },
}

impl EmploymentQuery {
    /// The dispatch target this query is, spelled once in `canonical-domain`.
    #[must_use]
    pub const fn target(&self) -> DispatchTarget {
        match self {
            Self::Appoint { .. } => DispatchTarget::HrAppoint,
            Self::Promote { .. } => DispatchTarget::HrPromote,
            Self::Transfer { .. } => DispatchTarget::HrTransfer,
        }
    }

    #[must_use]
    pub const fn attributes(&self) -> &EmploymentAttributes {
        match self {
            Self::Appoint { attributes, .. }
            | Self::Promote { attributes, .. }
            | Self::Transfer { attributes, .. } => attributes,
        }
    }

    #[must_use]
    pub const fn valid_from(&self) -> OffsetDateTime {
        match self {
            Self::Appoint { valid_from, .. }
            | Self::Promote { valid_from, .. }
            | Self::Transfer { valid_from, .. } => *valid_from,
        }
    }
}

impl CanonicalQuery for EmploymentQuery {
    fn dispatch_target(&self) -> DispatchTarget {
        self.target()
    }
}

/// The typed write this port accepts. `org_id` is the RLS key and `command_id`
/// the tenant-global idempotency key; a repeat replays the stored receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmploymentCommand {
    pub org_id: OrgId,
    pub command_id: CommandId,
    pub actor_id: UserId,
    pub query: EmploymentQuery,
}

#[derive(Debug, thiserror::Error)]
pub enum EmploymentError {
    #[error("preflight blocked the command: {0:?}")]
    Blocked(Vec<String>),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error("command {0} was already applied with a different payload")]
    DigestConflict(Uuid),
    #[error("stored receipt for command {0} names no dispatch target: {1}")]
    UnreadableReceipt(Uuid, String),
}

/// The one permitted holder of production DML against `employees`,
/// `employment_heads`, `employment_revisions` and
/// `employment_source_bindings`.
#[derive(Debug, Clone)]
pub struct PgEmploymentPort {
    pool: PgPool,
    runtime: tokio::runtime::Handle,
}

impl PgEmploymentPort {
    #[must_use]
    pub const fn new(pool: PgPool, runtime: tokio::runtime::Handle) -> Self {
        Self { pool, runtime }
    }

    async fn write(&self, command: &EmploymentCommand) -> Result<CommandReceipt, EmploymentError> {
        let preflight = <Self as CanonicalPort>::preflight(&command.query);
        if !preflight.is_ok() {
            return Err(EmploymentError::Blocked(preflight.blockers().to_vec()));
        }

        let digest = payload_digest(command);
        let org = *command.org_id.as_uuid();
        let actor = *command.actor_id.as_uuid();
        let command_uuid = *command.command_id.as_uuid();

        let mut tx = self.pool.begin().await?;
        // Transaction-local, so it is cleared on COMMIT/ROLLBACK and never
        // leaks to the next checkout of a pooled connection. Unset fails
        // closed: RLS shows no rows and accepts no writes.
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
                return Err(EmploymentError::DigestConflict(command_uuid));
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
        let valid_from = command.query.valid_from();
        let attributes = command.query.attributes();

        let (employment_id, version) = match &command.query {
            EmploymentQuery::Appoint { .. } => {
                let employment_id: Uuid = sqlx::query_scalar(
                    "INSERT INTO employment_heads (org_id, valid_from) VALUES ($1, $2) \
                     RETURNING id",
                )
                .bind(org)
                .bind(valid_from)
                .fetch_one(&mut *tx)
                .await?;
                (employment_id, 1_i64)
            }
            EmploymentQuery::Promote { employment_id, .. }
            | EmploymentQuery::Transfer { employment_id, .. } => {
                // ponytail: MAX + 1 under the row's own transaction. A
                // concurrent revise of the same employment loses to
                // UNIQUE (org_id, employment_id, version) with 23505 rather
                // than silently overwriting; add SELECT ... FOR UPDATE on
                // `employment_heads` if that contention is ever measured.
                let next: i64 = sqlx::query_scalar(
                    "SELECT COALESCE(MAX(version), 0) + 1 FROM employment_revisions \
                     WHERE org_id = $1 AND employment_id = $2",
                )
                .bind(org)
                .bind(employment_id)
                .fetch_one(&mut *tx)
                .await?;
                (*employment_id, next)
            }
        };

        let result = serde_json::json!({
            "employment_id": employment_id.to_string(),
            "version": version,
            "target": target.as_str(),
        });

        let created_at: OffsetDateTime = sqlx::query_scalar(
            "INSERT INTO employment_revisions \
             (org_id, employment_id, version, command_id, actor_id, payload_digest, \
              valid_from, attributes, receipt) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING created_at",
        )
        .bind(org)
        .bind(employment_id)
        .bind(version)
        .bind(command_uuid)
        .bind(actor)
        .bind(digest.as_slice())
        .bind(valid_from)
        .bind(attributes.to_json())
        .bind(&result)
        .fetch_one(&mut *tx)
        .await?;

        match &command.query {
            EmploymentQuery::Appoint { employee_id, .. } => {
                sqlx::query(
                    "INSERT INTO employment_source_bindings \
                     (org_id, employee_id, employment_id, actor_id, payload_digest) \
                     VALUES ($1, $2, $3, $4, $5)",
                )
                .bind(org)
                .bind(employee_id)
                .bind(employment_id)
                .bind(actor)
                .bind(digest.as_slice())
                .execute(&mut *tx)
                .await?;
            }
            // The legacy compatibility head carries the new state, through the
            // same statement the REST lifecycle handler calls. `valid_from` is
            // the effective date, and the statement writes `exit_date` from it
            // exactly when the status is `EXITED`.
            EmploymentQuery::Promote { .. } | EmploymentQuery::Transfer { .. } => {
                let employee_id = bound_employee(&mut tx, org, employment_id).await?;
                let effective_date = valid_from.date().to_string();
                apply_employment_change(
                    &mut tx,
                    org,
                    employee_id,
                    attributes.as_change(&effective_date),
                )
                .await?;
            }
        }

        // The receipt store, and with it the tenant-global command-id namespace
        // this port shares with every other receipt owner.
        sqlx::query(
            "INSERT INTO ont_action_command_receipts \
             (org_id, command_id, actor_id, payload_digest, receipt, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(org)
        .bind(command_uuid)
        .bind(actor)
        .bind(digest.as_slice())
        .bind(&result)
        .bind(created_at)
        .execute(&mut *tx)
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

/// The legacy `employees` row this employment IS, read through the binding.
///
/// `RowNotFound` when nothing is bound is deliberate: a promotion whose legacy
/// head cannot be found must fail the whole command rather than append a
/// canonical revision the rest of the tree cannot see.
async fn bound_employee(
    tx: &mut Transaction<'_, Postgres>,
    org_id: Uuid,
    employment_id: Uuid,
) -> Result<Uuid, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT employee_id FROM employment_source_bindings \
         WHERE org_id = $1 AND employment_id = $2",
    )
    .bind(org_id)
    .bind(employment_id)
    .fetch_one(tx.as_mut())
    .await
}

impl CanonicalPort for PgEmploymentPort {
    type Object = Employment;
    type Query = EmploymentQuery;
    type Command = EmploymentCommand;
    type Error = EmploymentError;

    /// PURE: no `&self`, no IO, no persistence. A blocked preflight has written
    /// nothing, so it can never spend an approval.
    fn preflight(query: &Self::Query) -> Preflight {
        let mut blockers = Vec::new();
        let attributes = query.attributes();
        if attributes.company.trim().is_empty() {
            blockers.push("company must not be blank".to_owned());
        }
        if !EMPLOYMENT_STATUSES.contains(&attributes.employment_status.as_str()) {
            blockers.push(format!(
                "employment_status must be one of {EMPLOYMENT_STATUSES:?}"
            ));
        }
        match query {
            EmploymentQuery::Appoint { employee_id, .. } if employee_id.is_nil() => {
                blockers.push("employee_id must not be nil".to_owned());
            }
            EmploymentQuery::Promote { employment_id, .. }
            | EmploymentQuery::Transfer { employment_id, .. }
                if employment_id.is_nil() =>
            {
                blockers.push("employment_id must not be nil".to_owned());
            }
            _ => {}
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
        EmploymentCommand {
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

fn receipt(
    command: &EmploymentCommand,
    target: DispatchTarget,
    actor_id: UserId,
    digest: [u8; 32],
    result: serde_json::Value,
    created_at: OffsetDateTime,
) -> CommandReceipt {
    CommandReceipt::new(
        command.org_id,
        command.command_id,
        ReceiptOwner::Canonical(ObjectKey::Employment),
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
) -> Result<DispatchTarget, EmploymentError> {
    let stored = result["target"]
        .as_str()
        .ok_or_else(|| EmploymentError::UnreadableReceipt(command_id, result.to_string()))?;
    DispatchTarget::from_str(stored)
        .map_err(|error| EmploymentError::UnreadableReceipt(command_id, error.to_string()))
}

/// The 32 bytes the `payload_digest` CHECK is sized for.
///
/// Every field is hashed in a fixed order from TYPED values, never from a
/// `serde_json::Value`: `serde_json` resolves with `preserve_order` in this
/// workspace, so a `Value` serialises its object keys in INSERTION order and two
/// payloads that compare EQUAL serialise to different bytes. The retry a client
/// performs after a timeout must digest to the same 32 bytes, or it comes back
/// as a [`EmploymentError::DigestConflict`] instead of the documented replay.
/// `None` and `Some("")` are separated by a tag byte, so an absent org unit and
/// a blank one are not the same command.
fn payload_digest(command: &EmploymentCommand) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(command.org_id.as_uuid().as_bytes());
    hasher.update(command.command_id.as_uuid().as_bytes());
    hasher.update(command.actor_id.as_uuid().as_bytes());
    hasher.update(command.query.target().as_str().as_bytes());
    match &command.query {
        EmploymentQuery::Appoint { employee_id, .. } => hasher.update(employee_id.as_bytes()),
        EmploymentQuery::Promote { employment_id, .. }
        | EmploymentQuery::Transfer { employment_id, .. } => {
            hasher.update(employment_id.as_bytes());
        }
    }
    hasher.update(
        command
            .query
            .valid_from()
            .unix_timestamp_nanos()
            .to_be_bytes(),
    );
    let attributes = command.query.attributes();
    for field in [
        Some(attributes.company.as_str()),
        attributes.org_unit.as_deref(),
        attributes.position.as_deref(),
        Some(attributes.employment_status.as_str()),
    ] {
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
