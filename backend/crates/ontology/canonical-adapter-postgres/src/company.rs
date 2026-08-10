//! `CompanyPort` — the Postgres implementation of `ObjectKey::Company`.
//!
//! Owned tables, verbatim from the contract: `organizations`,
//! `company_revisions`. `organizations` stays the tenant/current head and
//! `company_revisions` is the append-only history; no `companies` table is
//! created.
//!
//! # Why this port writes ONE of its two owned tables
//!
//! `organizations` is owned by Company and is DELIBERATELY unwritable by the
//! runtime role. `0031_runtime_role_and_immutable_org.sql` grants `console_rt`
//! `SELECT` on it and then `REVOKE INSERT, UPDATE, DELETE ON organizations FROM
//! console_rt`, in its own words because "provisioning a tenant is an owner
//! operation, so the runtime role must never INSERT/UPDATE/DELETE org rows" —
//! the one deliberate exception to the `ALTER DEFAULT PRIVILEGES` blanket grant
//! two lines below it. So an `UPDATE organizations` here would not be a stronger
//! port; it would be a statement that raises `42501 permission denied for table
//! organizations` in every deployed database.
//!
//! Ownership in the contract is the right to write, not an obligation to. The
//! canonical company state therefore lives entirely in
//! `company_revisions.attributes`, and `organizations` keeps exactly the role
//! 0026 and 0031 gave it: the provisioning-owned identity of the tenant, and the
//! `REFERENCES organizations(id)` this table's `org_id` hangs off.
//! `tests/company_port_as_runtime_role.rs::the_runtime_role_may_not_write_the_organizations_head`
//! measures both halves of that — the three revoked privileges and the real
//! refusal — so the claim is executed rather than argued.
//!
//! One consequence to state plainly: `console-gate-writer-ownership` can never
//! be made non-vacuous for `organizations` by this port, because there is no
//! legitimate writer to be a second one of. Its coverage of that table is proven
//! by injection instead (a second writer added to a non-owner crate is charged),
//! not by a green run.
//!
//! # Where the receipt is stored
//!
//! In `ont_action_command_receipts` — the generalised store 0177 created, shared
//! with `console-ontology-rest` and with `PersonPort`. Its `PRIMARY KEY (org_id,
//! command_id)` is what makes a command id tenant-global across OWNERS, which is
//! the property `CommandId` and `ReceiptOwner` state in `canonical-domain`. A
//! private per-object store would have given this port its own namespace, and
//! one client idempotency key would then have meant two accepted commands.
//!
//! 0177 carries no `owner` and no `target` column yet, so the `DispatchTarget`
//! travels inside the receipt JSONB as its wire string and read-back parses it
//! with `FromStr`, spelling the target literals once, in `canonical-domain`. A
//! stored row that names no target is refused, never replayed.
//!
//! # Append-only, enforced by the database
//!
//! `company_revisions` refuses UPDATE and DELETE (0215's
//! `canonical_org_structure_row_immutable` trigger), so a revision is never
//! edited: a revise appends `MAX(version) + 1`. `UNIQUE (org_id, version)`
//! carries no object id because there is exactly one company per tenant — the
//! tenant IS the company.
//!
//! The TRIGGER is the whole of that enforcement, not a privilege:
//! `ops/postgres-reconcile-topology.sh` grants `console_rt` UPDATE and DELETE on
//! every table a migration creates, which 0215's own header states, so the
//! runtime role holds both on `company_revisions` in the deployed database.
//!
//! # Synchronous port, async driver
//!
//! `CanonicalPort::execute` is synchronous and `sqlx` is async-only, so
//! [`PgCompanyPort`] holds a `tokio::runtime::Handle` and blocks on it.
//! `Handle::block_on` panics when called from a runtime worker thread; an async
//! caller must therefore reach `execute` through `spawn_blocking`.
//! ponytail: one runtime handle, no thread pool of its own — revisit only if the
//! trait ever gains an async form.

use console_kernel_core::{OrgId, UserId};
use console_ontology_canonical_domain::{
    CanonicalPort, CanonicalQuery, CommandId, CommandReceipt, Company, DispatchTarget, ObjectKey,
    Preflight, ReceiptOwner,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};
use std::str::FromStr;
use time::OffsetDateTime;
use uuid::Uuid;

/// The typed read this port answers: the write a caller intends, and nothing
/// about how it is performed.
///
/// A STRUCT, not an enum. `Company` has exactly one dispatch target in the
/// contract — `company.revise` — so a one-variant enum would be a `match` with
/// a single arm in every method, and `the_contract_identity_is_copied_verbatim…`
/// asserts that one-ness against `DispatchTarget::ALL` so a second target makes
/// the test fail rather than this shape silently go wrong.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CompanyQuery {
    /// The company's canonical state at the new version. The attribute schema
    /// belongs to this port, and 0215 constrains it only to be a JSON object.
    pub attributes: serde_json::Value,
}

impl CompanyQuery {
    /// The dispatch target this query is, spelled once in `canonical-domain`.
    #[must_use]
    pub const fn target(&self) -> DispatchTarget {
        DispatchTarget::CompanyRevise
    }
}

impl CanonicalQuery for CompanyQuery {
    fn dispatch_target(&self) -> DispatchTarget {
        self.target()
    }

    fn subject_id(&self) -> Option<Uuid> {
        // Company revise is the tenant itself (`org_id`), not a payload row id.
        None
    }
}

/// The typed write this port accepts. `org_id` is the RLS key, the tenant, AND
/// the company; `command_id` is the tenant-global idempotency key, and a repeat
/// replays the stored receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompanyCommand {
    pub org_id: OrgId,
    pub command_id: CommandId,
    pub actor_id: UserId,
    pub query: CompanyQuery,
}

#[derive(Debug, thiserror::Error)]
pub enum CompanyError {
    #[error("preflight blocked the command: {0:?}")]
    Blocked(Vec<String>),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error("command {0} was already applied with a different payload")]
    DigestConflict(Uuid),
    #[error("stored receipt for command {0} names no dispatch target: {1}")]
    UnreadableReceipt(Uuid, String),
}

/// The one permitted holder of production DML against `company_revisions`, and
/// the declared owner of `organizations` — which no runtime identity may write.
#[derive(Debug, Clone)]
pub struct PgCompanyPort {
    pool: PgPool,
    runtime: tokio::runtime::Handle,
}

impl PgCompanyPort {
    #[must_use]
    pub const fn new(pool: PgPool, runtime: tokio::runtime::Handle) -> Self {
        Self { pool, runtime }
    }

    async fn write(&self, command: &CompanyCommand) -> Result<CommandReceipt, CompanyError> {
        let preflight = <Self as CanonicalPort>::preflight(&command.query);
        if !preflight.is_ok() {
            return Err(CompanyError::Blocked(preflight.blockers().to_vec()));
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
                return Err(CompanyError::DigestConflict(command_uuid));
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

        // ponytail: MAX + 1 under the row's own transaction. A concurrent revise
        // of the same tenant loses to UNIQUE (org_id, version) with 23505 rather
        // than silently overwriting; add SELECT ... FOR UPDATE on the head if
        // that contention is ever measured.
        let version: i64 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(version), 0) + 1 FROM company_revisions WHERE org_id = $1",
        )
        .bind(org)
        .fetch_one(&mut *tx)
        .await?;

        let target = command.query.target();
        let result = serde_json::json!({
            "org_id": org.to_string(),
            "version": version,
            "target": target.as_str(),
        });

        let created_at: OffsetDateTime = sqlx::query_scalar(
            "INSERT INTO company_revisions \
             (org_id, version, command_id, actor_id, payload_digest, attributes, receipt) \
             VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING created_at",
        )
        .bind(org)
        .bind(version)
        .bind(command_uuid)
        .bind(actor)
        .bind(digest.as_slice())
        .bind(&command.query.attributes)
        .bind(&result)
        .fetch_one(&mut *tx)
        .await?;

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

impl CanonicalPort for PgCompanyPort {
    type Object = Company;
    type Query = CompanyQuery;
    type Command = CompanyCommand;
    type Error = CompanyError;

    /// PURE: no `&self`, no IO, no persistence. A blocked preflight has written
    /// nothing, so it can never spend an approval.
    fn preflight(query: &Self::Query) -> Preflight {
        if query.attributes.is_object() {
            Preflight::ok()
        } else {
            Preflight::blocked(vec!["attributes must be a JSON object".to_owned()])
        }
    }

    fn command(
        org_id: OrgId,
        command_id: CommandId,
        actor_id: UserId,
        query: Self::Query,
    ) -> Self::Command {
        CompanyCommand {
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
    command: &CompanyCommand,
    target: DispatchTarget,
    actor_id: UserId,
    digest: [u8; 32],
    result: serde_json::Value,
    created_at: OffsetDateTime,
) -> CommandReceipt {
    CommandReceipt::new(
        command.org_id,
        command.command_id,
        ReceiptOwner::Canonical(ObjectKey::Company),
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
) -> Result<DispatchTarget, CompanyError> {
    let stored = result["target"]
        .as_str()
        .ok_or_else(|| CompanyError::UnreadableReceipt(command_id, result.to_string()))?;
    DispatchTarget::from_str(stored)
        .map_err(|error| CompanyError::UnreadableReceipt(command_id, error.to_string()))
}

/// The 32 bytes the `payload_digest` CHECK is sized for.
///
/// The attributes go in through [`canonical_json`], never through
/// `Value::to_string()` directly: `serde_json` resolves with `preserve_order`
/// in this workspace, so a `Value` serialises its object keys in INSERTION
/// order and two payloads that compare EQUAL serialise to different bytes. The
/// retry a client performs after a timeout — or after a round-trip through the
/// `attributes` JSONB column, which PostgreSQL stores in its own key order —
/// must digest to the same 32 bytes, or it comes back as a
/// [`CompanyError::DigestConflict`] instead of the documented replay.
fn payload_digest(command: &CompanyCommand) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(command.org_id.as_uuid().as_bytes());
    hasher.update(command.command_id.as_uuid().as_bytes());
    hasher.update(command.actor_id.as_uuid().as_bytes());
    hasher.update(command.query.target().as_str().as_bytes());
    hasher.update(
        canonical_json(&command.query.attributes)
            .to_string()
            .as_bytes(),
    );
    hasher.finalize().into()
}

/// The same value with every object's keys SORTED, at every depth — the form
/// `Value::to_string()` would already emit if `serde_json` were not built with
/// `preserve_order`. The sort is explicit because collecting back into a `Map`
/// preserves the source's iteration order under that feature, so a rebuild that
/// does not sort is a no-op.
fn canonical_json(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Array(values) => {
            serde_json::Value::Array(values.iter().map(canonical_json).collect())
        }
        serde_json::Value::Object(values) => {
            let mut entries: Vec<(String, serde_json::Value)> = values
                .iter()
                .map(|(key, value)| (key.clone(), canonical_json(value)))
                .collect();
            entries.sort_by(|left, right| left.0.cmp(&right.0));
            serde_json::Value::Object(entries.into_iter().collect())
        }
        primitive => primitive.clone(),
    }
}
