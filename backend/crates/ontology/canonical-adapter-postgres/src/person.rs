//! `PersonPort` — the Postgres implementation of `ObjectKey::Person`.
//!
//! Owned tables, verbatim from the contract: `persons`, `person_revisions`,
//! `employee_person_bindings`.
//!
//! # Where the receipt is stored
//!
//! In `ont_action_command_receipts` — the generalised store 0177 created, the
//! one `console-ontology-rest` already writes. Its `PRIMARY KEY (org_id,
//! command_id)` is what makes a command id tenant-global across OWNERS, which
//! is the property `CommandId` and `ReceiptOwner` state in `canonical-domain`
//! ("the owner column is deliberately NOT part of the key, so the same command
//! id cannot be replayed under a different owner"). A private per-object store
//! would have given this port its own namespace, and one client idempotency key
//! would then have meant two accepted commands.
//!
//! 0177 carries no `owner` and no `target` column yet — the widening migration
//! is specified in `ReceiptOwner`'s doc, unwritten, and this lane may not write
//! `backend/crates/platform/db/migrations/**` — so the `DispatchTarget` travels
//! inside the receipt JSONB as its wire string and read-back parses it with
//! `FromStr`, spelling the thirteen target literals once, in `canonical-domain`.
//! A stored row that names no target is refused, never replayed.
//!
//! `person_revisions.receipt` is NOT NULL, so the revision row carries the same
//! JSON; the row read on replay is the 0177 one.
//!
//! # Append-only, enforced by the database
//!
//! `person_revisions` refuses UPDATE and DELETE (0213's
//! `canonical_person_row_immutable` trigger), so a revision is never edited:
//! `PersonQuery::Revise` appends `MAX(version) + 1`. `employee_person_bindings`
//! refuses UPDATE and carries `PRIMARY KEY (org_id, employee_id)`, so an
//! employee cannot acquire a second binding — while two employees binding to
//! ONE person stays representable on purpose, because that is what the
//! distinct-natural-person four-eyes bar has to detect.
//!
//! # Deterministic bindings (P5 / console-dgo.2)
//!
//! A trusted uniquely-resolved employee is bound with `person_id = employee_id`:
//! `PersonQuery::Create` with `Some(employee_id)` inserts `persons.id` equal to
//! that employee id, then writes the binding row. Omitting `employee_id`
//! (duplicate / review-required imports) creates the person with **no** binding.
//! Bindings are never inferred from name, phone, org text, or other attributes —
//! only an explicit `employee_id` on the command creates a row in
//! `employee_person_bindings`.
//!
//! The TRIGGER is the whole of that enforcement, not a privilege:
//! `ops/postgres-reconcile-topology.sh` grants `console_rt` UPDATE and DELETE on
//! every table a migration creates, which 0213's own header states, so the
//! runtime role holds UPDATE on `person_revisions` in the deployed database.
//!
//! # Synchronous port, async driver
//!
//! `CanonicalPort::execute` is synchronous and `sqlx` is async-only, so
//! [`PgPersonPort`] holds a `tokio::runtime::Handle` and blocks on it.
//! `Handle::block_on` panics when called from a runtime worker thread; an async
//! caller must therefore reach `execute` through `spawn_blocking`.
//! ponytail: one runtime handle, no thread pool of its own — revisit only if
//! the trait ever gains an async form.

use console_kernel_core::{KernelError, OrgId, UserId};
use console_ontology_canonical_domain::{
    CanonicalPort, CanonicalPortError, CanonicalQuery, CommandId, CommandReceipt, DispatchTarget,
    ObjectKey, Person, Preflight, ReceiptOwner,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};
use std::str::FromStr;
use time::OffsetDateTime;
use uuid::Uuid;

/// The typed read this port answers: the write a caller intends, and nothing
/// about how it is performed. Each variant is bound to exactly one of the two
/// dispatch targets the contract assigns to `Person`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "target")]
pub enum PersonQuery {
    /// `people.create_person`. When `employee_id` is set (trusted uniquely
    /// resolved), the new person's id **is** that employee id and a binding
    /// row is written. When omitted, the person is created unbound — attributes
    /// never invent a binding.
    #[serde(rename = "people.create_person")]
    Create {
        #[serde(default)]
        employee_id: Option<Uuid>,
        attributes: serde_json::Value,
    },
    /// `people.revise_person`. Appends a revision, and optionally binds a
    /// FURTHER employee record to the same natural person (second employment
    /// of one person — person_id need not equal that further employee_id).
    #[serde(rename = "people.revise_person")]
    Revise {
        person_id: Uuid,
        #[serde(default)]
        employee_id: Option<Uuid>,
        attributes: serde_json::Value,
    },
}

impl PersonQuery {
    /// The dispatch target this query is, spelled once in `canonical-domain`.
    #[must_use]
    pub const fn target(&self) -> DispatchTarget {
        match self {
            Self::Create { .. } => DispatchTarget::PeopleCreatePerson,
            Self::Revise { .. } => DispatchTarget::PeopleRevisePerson,
        }
    }

    #[must_use]
    pub const fn attributes(&self) -> &serde_json::Value {
        match self {
            Self::Create { attributes, .. } | Self::Revise { attributes, .. } => attributes,
        }
    }

    #[must_use]
    pub const fn employee_id(&self) -> Option<Uuid> {
        match self {
            Self::Create { employee_id, .. } | Self::Revise { employee_id, .. } => *employee_id,
        }
    }
}

impl CanonicalQuery for PersonQuery {
    fn dispatch_target(&self) -> DispatchTarget {
        self.target()
    }

    fn subject_id(&self) -> Option<Uuid> {
        match self {
            Self::Create { .. } => None,
            Self::Revise { person_id, .. } => Some(*person_id),
        }
    }
}

/// The typed write this port accepts. `org_id` is the RLS key and `command_id`
/// the tenant-global idempotency key; a repeat replays the stored receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonCommand {
    pub org_id: OrgId,
    pub command_id: CommandId,
    pub actor_id: UserId,
    pub query: PersonQuery,
}

#[derive(Debug, thiserror::Error)]
pub enum PersonError {
    #[error("preflight blocked the command: {0:?}")]
    Blocked(Vec<String>),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error("command {0} was already applied with a different payload")]
    DigestConflict(Uuid),
    #[error("stored receipt for command {0} names no dispatch target: {1}")]
    UnreadableReceipt(Uuid, String),
}

impl CanonicalPortError for PersonError {
    fn into_kernel_error(self) -> KernelError {
        let message = self.to_string();
        match self {
            Self::Blocked(_) => KernelError::validation(message),
            Self::DigestConflict(_) => KernelError::conflict(message),
            Self::Database(_) | Self::UnreadableReceipt(_, _) => KernelError::internal(message),
        }
    }
}

/// The one permitted holder of production DML against `persons`,
/// `person_revisions` and `employee_person_bindings`.
#[derive(Debug, Clone)]
pub struct PgPersonPort {
    pool: PgPool,
    runtime: tokio::runtime::Handle,
}

impl PgPersonPort {
    #[must_use]
    pub const fn new(pool: PgPool, runtime: tokio::runtime::Handle) -> Self {
        Self { pool, runtime }
    }

    async fn write(&self, command: &PersonCommand) -> Result<CommandReceipt, PersonError> {
        let preflight = <Self as CanonicalPort>::preflight(&command.query);
        if !preflight.is_ok() {
            return Err(PersonError::Blocked(preflight.blockers().to_vec()));
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
                return Err(PersonError::DigestConflict(command_uuid));
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
        let (person_id, version) = match &command.query {
            PersonQuery::Create { employee_id, .. } => {
                // Trusted uniquely-resolved: person_id = employee_id (P5).
                // Unbound / review-required: omit employee_id → random id, no binding.
                let person_id: Uuid = match employee_id {
                    Some(trusted) => {
                        sqlx::query_scalar(
                            "INSERT INTO persons (org_id, id) VALUES ($1, $2) RETURNING id",
                        )
                        .bind(org)
                        .bind(trusted)
                        .fetch_one(&mut *tx)
                        .await?
                    }
                    None => {
                        sqlx::query_scalar("INSERT INTO persons (org_id) VALUES ($1) RETURNING id")
                            .bind(org)
                            .fetch_one(&mut *tx)
                            .await?
                    }
                };
                (person_id, 1_i64)
            }
            PersonQuery::Revise { person_id, .. } => {
                // ponytail: MAX + 1 under the row's own transaction. A
                // concurrent revise of the same person loses to
                // UNIQUE (org_id, person_id, version) with 23505 rather than
                // silently overwriting; add SELECT ... FOR UPDATE on `persons`
                // if that contention is ever measured.
                let next: i64 = sqlx::query_scalar(
                    "SELECT COALESCE(MAX(version), 0) + 1 FROM person_revisions \
                     WHERE org_id = $1 AND person_id = $2",
                )
                .bind(org)
                .bind(person_id)
                .fetch_one(&mut *tx)
                .await?;
                (*person_id, next)
            }
        };

        let result = serde_json::json!({
            "person_id": person_id.to_string(),
            "version": version,
            "target": target.as_str(),
        });

        let created_at: OffsetDateTime = sqlx::query_scalar(
            "INSERT INTO person_revisions \
             (org_id, person_id, version, command_id, actor_id, payload_digest, attributes, receipt) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING created_at",
        )
        .bind(org)
        .bind(person_id)
        .bind(version)
        .bind(command_uuid)
        .bind(actor)
        .bind(digest.as_slice())
        .bind(command.query.attributes())
        .bind(&result)
        .fetch_one(&mut *tx)
        .await?;

        if let Some(employee_id) = command.query.employee_id() {
            sqlx::query(
                "INSERT INTO employee_person_bindings \
                 (org_id, employee_id, person_id, actor_id, payload_digest) \
                 VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(org)
            .bind(employee_id)
            .bind(person_id)
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

impl CanonicalPort for PgPersonPort {
    type Object = Person;
    type Query = PersonQuery;
    type Command = PersonCommand;
    type Error = PersonError;

    /// PURE: no `&self`, no IO, no persistence. A blocked preflight has written
    /// nothing, so it can never spend an approval.
    fn preflight(query: &Self::Query) -> Preflight {
        let mut blockers = Vec::new();
        if !query.attributes().is_object() {
            blockers.push("attributes must be a JSON object".to_owned());
        }
        if let PersonQuery::Revise { person_id, .. } = query
            && person_id.is_nil()
        {
            blockers.push("person_id must not be nil".to_owned());
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
        PersonCommand {
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
    command: &PersonCommand,
    target: DispatchTarget,
    actor_id: UserId,
    digest: [u8; 32],
    result: serde_json::Value,
    created_at: OffsetDateTime,
) -> CommandReceipt {
    CommandReceipt::new(
        command.org_id,
        command.command_id,
        ReceiptOwner::Canonical(ObjectKey::Person),
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
) -> Result<DispatchTarget, PersonError> {
    let stored = result["target"]
        .as_str()
        .ok_or_else(|| PersonError::UnreadableReceipt(command_id, result.to_string()))?;
    DispatchTarget::from_str(stored)
        .map_err(|error| PersonError::UnreadableReceipt(command_id, error.to_string()))
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
/// [`PersonError::DigestConflict`] instead of the documented replay.
fn payload_digest(command: &PersonCommand) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(command.org_id.as_uuid().as_bytes());
    hasher.update(command.command_id.as_uuid().as_bytes());
    hasher.update(command.actor_id.as_uuid().as_bytes());
    hasher.update(command.query.target().as_str().as_bytes());
    if let PersonQuery::Revise { person_id, .. } = &command.query {
        hasher.update(person_id.as_bytes());
    }
    if let Some(employee_id) = command.query.employee_id() {
        hasher.update(employee_id.as_bytes());
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
