//! `OrgUnitPort` — the Postgres implementation of `ObjectKey::OrgUnit`.
//!
//! Owned tables, verbatim from the contract: `org_units`,
//! `org_unit_revisions`, `org_unit_source_bindings`. Sites stay operational and
//! are not OrgUnits.
//!
//! # What this port does NOT touch
//!
//! `regions` and `branches` are the branch-scoped AUTHORIZATION spine and have
//! three legitimate writers of their own; the contract names neither, and bead
//! console-1qw.3 was closed decided-no on exactly that question. A branch is a
//! SOURCE that is BOUND to a canonical org unit, so binding one writes a row in
//! `org_unit_source_bindings` — never a write to `branches`. That is the whole
//! of the seam.
//!
//! # Where the receipt is stored
//!
//! In `ont_action_command_receipts` — the generalised store 0177 created and the
//! one `PersonPort` already shares. Its `PRIMARY KEY (org_id, command_id)` is
//! what makes a command id tenant-global across OWNERS, which is the property
//! `CommandId` states in `canonical-domain`. A private per-object store would
//! have given this port its own namespace, and one client idempotency key would
//! then have meant two accepted commands.
//!
//! 0177 carries no `owner` and no `target` column yet — the widening migration
//! is specified in `ReceiptOwner`'s doc, unwritten, and this lane may not write
//! `backend/crates/platform/db/migrations/**` — so the [`DispatchTarget`]
//! travels inside the receipt JSONB as its wire string and read-back parses it
//! with `FromStr`, spelling the thirteen target literals once, in
//! `canonical-domain`. A stored row that names no target is refused, never
//! replayed.
//!
//! # Append-only, enforced by the database
//!
//! `org_unit_revisions` refuses UPDATE and DELETE (0215's
//! `canonical_org_structure_row_immutable` trigger), so a revision is never
//! edited: [`OrgUnitQuery::Revise`] appends `MAX(version) + 1`. `org_units`
//! carries no trigger and needs none — it is an identity anchor with no mutable
//! state, every attribute living in the revision.
//!
//! `org_unit_source_bindings` refuses UPDATE but PERMITS DELETE, and that
//! asymmetry is load-bearing: silently re-pointing a legacy record at a
//! different unit by editing a column is what an audit must not tolerate, so a
//! rebind is an explicit DELETE then INSERT, while DELETE itself stays available
//! for erasure. Its `PRIMARY KEY (org_id, source_kind, source_id)` makes "one
//! legacy record resolves to at most one canonical unit" unrepresentable rather
//! than merely discouraged; the reverse is deliberately not unique, because one
//! unit legitimately absorbs several legacy records.
//!
//! The TRIGGER is the whole of that enforcement, not a privilege:
//! `ops/postgres-reconcile-topology.sh` grants `console_rt` UPDATE and DELETE on
//! every table a migration creates, which 0215's own header states, so the
//! runtime role holds UPDATE on `org_unit_revisions` in the deployed database.
//!
//! # Synchronous port, async driver
//!
//! `CanonicalPort::execute` is synchronous and `sqlx` is async-only, so
//! [`PgOrgUnitPort`] holds a `tokio::runtime::Handle` and blocks on it.
//! `Handle::block_on` panics when called from a runtime worker thread; an async
//! caller must therefore reach `execute` through `spawn_blocking`.
//! ponytail: one runtime handle, no thread pool of its own — revisit only if
//! the trait ever gains an async form.

use console_kernel_core::{OrgId, UserId};
use console_ontology_canonical_domain::{
    CanonicalPort, CanonicalQuery, CommandId, CommandReceipt, DispatchTarget, ObjectKey, OrgUnit,
    Preflight, ReceiptOwner,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};
use std::str::FromStr;
use time::OffsetDateTime;
use uuid::Uuid;

/// The legacy record an org unit is built from. `source_kind` and `source_id`
/// are TEXT in 0215 and constrained only to be non-empty: the closed set of
/// kinds is not yet enumerable and legacy identifiers are not all UUIDs.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SourceBinding {
    pub kind: String,
    pub id: String,
}

/// The typed read this port answers: the write a caller intends, and nothing
/// about how it is performed. Each variant is bound to exactly one of the two
/// dispatch targets the contract assigns to `OrgUnit`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "target")]
pub enum OrgUnitQuery {
    /// `organization.create_org_unit`. Optionally binds the new unit to the
    /// legacy record it was built from, in the same command.
    #[serde(rename = "organization.create_org_unit")]
    Create {
        #[serde(default)]
        source: Option<SourceBinding>,
        attributes: serde_json::Value,
    },
    /// `organization.revise_org_unit`. Appends a revision, and optionally binds
    /// a FURTHER legacy record to the same canonical unit.
    #[serde(rename = "organization.revise_org_unit")]
    Revise {
        org_unit_id: Uuid,
        #[serde(default)]
        source: Option<SourceBinding>,
        attributes: serde_json::Value,
    },
}

impl OrgUnitQuery {
    /// The dispatch target this query is, spelled once in `canonical-domain`.
    #[must_use]
    pub const fn target(&self) -> DispatchTarget {
        match self {
            Self::Create { .. } => DispatchTarget::OrganizationCreateOrgUnit,
            Self::Revise { .. } => DispatchTarget::OrganizationReviseOrgUnit,
        }
    }

    #[must_use]
    pub const fn attributes(&self) -> &serde_json::Value {
        match self {
            Self::Create { attributes, .. } | Self::Revise { attributes, .. } => attributes,
        }
    }

    #[must_use]
    pub const fn source(&self) -> Option<&SourceBinding> {
        match self {
            Self::Create { source, .. } | Self::Revise { source, .. } => source.as_ref(),
        }
    }
}

impl CanonicalQuery for OrgUnitQuery {
    fn dispatch_target(&self) -> DispatchTarget {
        self.target()
    }
}

/// The typed write this port accepts. `org_id` is the RLS key and `command_id`
/// the tenant-global idempotency key; a repeat replays the stored receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrgUnitCommand {
    pub org_id: OrgId,
    pub command_id: CommandId,
    pub actor_id: UserId,
    pub query: OrgUnitQuery,
}

#[derive(Debug, thiserror::Error)]
pub enum OrgUnitError {
    #[error("preflight blocked the command: {0:?}")]
    Blocked(Vec<String>),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error("command {0} was already applied with a different payload")]
    DigestConflict(Uuid),
    #[error("stored receipt for command {0} names no dispatch target: {1}")]
    UnreadableReceipt(Uuid, String),
}

/// The one permitted holder of production DML against `org_units`,
/// `org_unit_revisions` and `org_unit_source_bindings`.
#[derive(Debug, Clone)]
pub struct PgOrgUnitPort {
    pool: PgPool,
    runtime: tokio::runtime::Handle,
}

impl PgOrgUnitPort {
    #[must_use]
    pub const fn new(pool: PgPool, runtime: tokio::runtime::Handle) -> Self {
        Self { pool, runtime }
    }

    async fn write(&self, command: &OrgUnitCommand) -> Result<CommandReceipt, OrgUnitError> {
        let preflight = <Self as CanonicalPort>::preflight(&command.query);
        if !preflight.is_ok() {
            return Err(OrgUnitError::Blocked(preflight.blockers().to_vec()));
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
                return Err(OrgUnitError::DigestConflict(command_uuid));
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
        let (org_unit_id, version) = match &command.query {
            OrgUnitQuery::Create { .. } => {
                let org_unit_id: Uuid =
                    sqlx::query_scalar("INSERT INTO org_units (org_id) VALUES ($1) RETURNING id")
                        .bind(org)
                        .fetch_one(&mut *tx)
                        .await?;
                (org_unit_id, 1_i64)
            }
            OrgUnitQuery::Revise { org_unit_id, .. } => {
                // ponytail: MAX + 1 under the row's own transaction. A
                // concurrent revise of the same unit loses to
                // UNIQUE (org_id, org_unit_id, version) with 23505 rather than
                // silently overwriting; add SELECT ... FOR UPDATE on `org_units`
                // if that contention is ever measured.
                let next: i64 = sqlx::query_scalar(
                    "SELECT COALESCE(MAX(version), 0) + 1 FROM org_unit_revisions \
                     WHERE org_id = $1 AND org_unit_id = $2",
                )
                .bind(org)
                .bind(org_unit_id)
                .fetch_one(&mut *tx)
                .await?;
                (*org_unit_id, next)
            }
        };

        let result = serde_json::json!({
            "org_unit_id": org_unit_id.to_string(),
            "version": version,
            "target": target.as_str(),
        });

        let created_at: OffsetDateTime = sqlx::query_scalar(
            "INSERT INTO org_unit_revisions \
             (org_id, org_unit_id, version, command_id, actor_id, payload_digest, attributes, receipt) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING created_at",
        )
        .bind(org)
        .bind(org_unit_id)
        .bind(version)
        .bind(command_uuid)
        .bind(actor)
        .bind(digest.as_slice())
        .bind(command.query.attributes())
        .bind(&result)
        .fetch_one(&mut *tx)
        .await?;

        if let Some(source) = command.query.source() {
            sqlx::query(
                "INSERT INTO org_unit_source_bindings \
                 (org_id, source_kind, source_id, org_unit_id, actor_id, payload_digest) \
                 VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(org)
            .bind(&source.kind)
            .bind(&source.id)
            .bind(org_unit_id)
            .bind(actor)
            .bind(digest.as_slice())
            .execute(&mut *tx)
            .await?;
        }

        // The receipt store, and with it the tenant-global command-id
        // namespace this port shares with every other receipt owner.
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

impl CanonicalPort for PgOrgUnitPort {
    type Object = OrgUnit;
    type Query = OrgUnitQuery;
    type Command = OrgUnitCommand;
    type Error = OrgUnitError;

    /// PURE: no `&self`, no IO, no persistence. A blocked preflight has written
    /// nothing, so it can never spend an approval.
    fn preflight(query: &Self::Query) -> Preflight {
        let mut blockers = Vec::new();
        if !query.attributes().is_object() {
            blockers.push("attributes must be a JSON object".to_owned());
        }
        if let OrgUnitQuery::Revise { org_unit_id, .. } = query
            && org_unit_id.is_nil()
        {
            blockers.push("org_unit_id must not be nil".to_owned());
        }
        // 0215's `CHECK (source_kind <> '')` and `CHECK (source_id <> '')`,
        // restated purely so a caller learns both at once instead of one round
        // trip at a time. The CHECK remains the enforcement.
        if let Some(source) = query.source() {
            if source.kind.is_empty() {
                blockers.push("source_kind must not be empty".to_owned());
            }
            if source.id.is_empty() {
                blockers.push("source_id must not be empty".to_owned());
            }
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
        OrgUnitCommand {
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
    command: &OrgUnitCommand,
    target: DispatchTarget,
    actor_id: UserId,
    digest: [u8; 32],
    result: serde_json::Value,
    created_at: OffsetDateTime,
) -> CommandReceipt {
    CommandReceipt::new(
        command.org_id,
        command.command_id,
        ReceiptOwner::Canonical(ObjectKey::OrgUnit),
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
) -> Result<DispatchTarget, OrgUnitError> {
    let stored = result["target"]
        .as_str()
        .ok_or_else(|| OrgUnitError::UnreadableReceipt(command_id, result.to_string()))?;
    DispatchTarget::from_str(stored)
        .map_err(|error| OrgUnitError::UnreadableReceipt(command_id, error.to_string()))
}

/// The 32 bytes the `payload_digest` CHECK is sized for.
///
/// The attributes go in through [`canonical_json`], never through
/// `Value::to_string()` directly: `serde_json` resolves with `preserve_order`
/// in this workspace, so a `Value` serialises its object keys in INSERTION
/// order and two payloads that compare EQUAL serialise to different bytes. The
/// retry a client performs after a timeout — or after a round-trip through the
/// `attributes` JSONB column, which PostgreSQL stores in its own key order —
/// must digest to the same 32 bytes, or it comes back as an
/// [`OrgUnitError::DigestConflict`] instead of the documented replay.
///
/// ponytail: a byte-for-byte twin of `person::payload_digest`. The two cannot
/// share one helper today — `src/lib.rs` is out of this lane's owned root, so no
/// `mod digest` can be declared, and the sibling's copy is private. Hoisting
/// both into one module is a follow-up for after the three port lanes land.
fn payload_digest(command: &OrgUnitCommand) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(command.org_id.as_uuid().as_bytes());
    hasher.update(command.command_id.as_uuid().as_bytes());
    hasher.update(command.actor_id.as_uuid().as_bytes());
    hasher.update(command.query.target().as_str().as_bytes());
    if let OrgUnitQuery::Revise { org_unit_id, .. } = &command.query {
        hasher.update(org_unit_id.as_bytes());
    }
    if let Some(source) = command.query.source() {
        // LENGTH-PREFIXED, because plain concatenation is ambiguous: ("hris", "emp-1") and
        // ("hrise", "mp-1") produce identical bytes and therefore an identical digest. The port
        // would then replay the FIRST command's receipt and report success for a source binding it
        // never wrote, and a reconciler reading that receipt records the mapping as synced.
        // `employment.rs` already hashes its variable-length fields this way; this was the one
        // canonical port that did not.
        for value in [source.kind.as_str(), source.id.as_str()] {
            hasher.update((value.len() as u64).to_be_bytes());
            hasher.update(value.as_bytes());
        }
    }
    hasher.update(
        canonical_json(command.query.attributes())
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
