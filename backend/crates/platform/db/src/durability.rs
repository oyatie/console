//! Explicit completion policy for owners that require remote WAL application.
//! Admission is supplied by deployment authority; observations never enroll a peer.

use std::{future::Future, net::IpAddr, pin::Pin, time::Duration};

use console_kernel_core::OrgId;
use serde::Deserialize;
use sqlx::postgres::types::Oid;
use sqlx::{Connection, PgConnection, PgPool, Postgres, Row, Transaction, pool::PoolConnection};
use time::OffsetDateTime;
use tokio::time::{Instant, sleep, timeout_at};

/// Required policy is constructed by `from_json`; local selection is explicit.
/// There is deliberately no default.
#[derive(Debug, Clone)]
pub struct DurabilityPolicy(PolicyMode);

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
enum PolicyMode {
    LocalDevelopment {},
    RequiredRemoteApply(RemoteAdmission),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RemoteAdmission {
    primary_system_id: String,
    #[serde(with = "time::serde::rfc3339")]
    primary_started_at: OffsetDateTime,
    slot: String,
    replication_role_oid: u32,
    replication_role_name: String,
    application_name: String,
    peer: PeerAdmission,
    timeout_ms: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
enum PeerAdmission {
    /// Requires separately admitted network/credential custody. This descriptor
    /// does not establish arbitrary physical-replica identity or production TLS.
    AdmittedPrivateNetwork { client_addr: IpAddr },
}

#[derive(Debug, thiserror::Error)]
#[error("invalid CONSOLE_DATABASE_DURABILITY: {0}")]
pub struct InvalidDurabilityPolicy(&'static str);

/// Completion was not confirmed. The operation may have committed locally.
#[derive(Debug, thiserror::Error)]
#[error("database durability UNKNOWN: {reason}")]
pub struct DurabilityUnknown {
    reason: &'static str,
    #[source]
    source: Option<sqlx::Error>,
}

impl DurabilityUnknown {
    fn new(reason: &'static str) -> Self {
        Self {
            reason,
            source: None,
        }
    }
}

impl From<sqlx::Error> for DurabilityUnknown {
    fn from(source: sqlx::Error) -> Self {
        Self {
            reason: "database transport or observation failed",
            source: Some(source),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CompletionError<E> {
    #[error("{0}")]
    Operation(E),
    #[error(transparent)]
    Unknown(#[from] DurabilityUnknown),
}

impl DurabilityPolicy {
    /// Select local completion intentionally for a single-node development caller.
    #[must_use]
    pub const fn local_development() -> Self {
        Self(PolicyMode::LocalDevelopment {})
    }

    pub fn from_json(value: &str) -> Result<Self, InvalidDurabilityPolicy> {
        if value.len() > 16_384 {
            return Err(InvalidDurabilityPolicy("policy exceeds size bound"));
        }
        let policy: PolicyMode = serde_json::from_str(value)
            .map_err(|_| InvalidDurabilityPolicy("invalid JSON policy schema"))?;
        if let PolicyMode::RequiredRemoteApply(admission) = &policy {
            if admission
                .primary_system_id
                .parse::<u64>()
                .ok()
                .is_none_or(|id| id == 0)
                || admission.replication_role_oid == 0
                || admission.slot.is_empty()
                || admission.slot.len() > 63
                || !admission
                    .slot
                    .bytes()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_')
                || !valid_name(&admission.replication_role_name)
                || !valid_name(&admission.application_name)
                || !(1..=60_000).contains(&admission.timeout_ms)
            {
                return Err(InvalidDurabilityPolicy("invalid identity or timeout bound"));
            }
            let PeerAdmission::AdmittedPrivateNetwork { client_addr } = &admission.peer;
            if client_addr.is_unspecified() || client_addr.is_multicast() {
                return Err(InvalidDurabilityPolicy(
                    "peer must identify one admitted address",
                ));
            }
        }
        Ok(Self(policy))
    }

    /// Validate Required admission before serving startup. Every owner operation
    /// must still revalidate and pin its own sender/session epoch.
    pub async fn validate(&self, pool: &PgPool) -> Result<(), DurabilityUnknown> {
        let PolicyMode::RequiredRemoteApply(admission) = &self.0 else {
            return Ok(());
        };
        let deadline = Instant::now() + Duration::from_millis(admission.timeout_ms);
        timeout_at(deadline, async {
            let mut held = RetainedConnection::new(pool.acquire().await?, pool, true);
            observe(held.connection()?, admission, None).await?;
            ensure_deadline(Some(deadline))?;
            held.reusable = true;
            Ok(())
        })
        .await
        .map_err(|_| DurabilityUnknown::new("admission deadline elapsed"))?
    }
}

fn ensure_deadline(deadline: Option<Instant>) -> Result<(), DurabilityUnknown> {
    if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
        Err(DurabilityUnknown::new("completion deadline elapsed"))
    } else {
        Ok(())
    }
}

fn valid_name(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= 63 && !value.chars().any(char::is_control)
}

/// An uncertain Required operation keeps its pool permit until the same
/// channel reaches ReadyForQuery. A socket close alone cannot wake SyncRep.
struct RetainedConnection {
    connection: Option<PoolConnection<Postgres>>,
    pool: PgPool,
    runtime: tokio::runtime::Handle,
    required: bool,
    cleaning: bool,
    reusable: bool,
}

impl RetainedConnection {
    fn new(connection: PoolConnection<Postgres>, pool: &PgPool, required: bool) -> Self {
        Self {
            connection: Some(connection),
            pool: pool.clone(),
            runtime: tokio::runtime::Handle::current(),
            required,
            cleaning: false,
            reusable: false,
        }
    }

    fn connection(&mut self) -> Result<&mut PgConnection, DurabilityUnknown> {
        self.connection
            .as_deref_mut()
            .ok_or_else(|| DurabilityUnknown::new("retained connection unavailable"))
    }
}

impl Drop for RetainedConnection {
    fn drop(&mut self) {
        if self.reusable {
            return;
        }
        let runtime_available = tokio::runtime::Handle::try_current().is_ok();
        // SQLx PoolConnection::Drop itself spawns. Enter the acquisition runtime
        // even during caller teardown; this does not promise runtime progress.
        let _entered = self.runtime.enter();
        if !self.required {
            if let Some(mut connection) = self.connection.take() {
                connection.close_on_drop();
                drop(connection);
            }
        } else if self.cleaning || !runtime_available {
            // Cleanup was canceled or cannot run. Pool::close marks closed
            // synchronously; do not release uncertainty into replacement work.
            drop(self.pool.close());
            if let Some(mut connection) = self.connection.take() {
                connection.close_on_drop();
                drop(connection);
            }
        } else {
            let mut cleanup = Self {
                connection: self.connection.take(),
                pool: self.pool.clone(),
                runtime: self.runtime.clone(),
                required: true,
                cleaning: true,
                reusable: false,
            };
            self.runtime.spawn(async move {
                let drained = match cleanup.connection() {
                    Ok(connection) => connection.ping().await.is_ok(),
                    Err(_) => false,
                };
                if !drained {
                    // A failed channel is not evidence that its server finished.
                    drop(cleanup.pool.close());
                }
                if let Some(connection) = cleanup.connection.take() {
                    let _ = connection.close().await;
                }
                cleanup.reusable = true;
                drop(cleanup);
            });
        }
    }
}

/// PostgreSQL disables transaction_timeout before SyncRep, and suppresses a
/// statement timer that is >= a positive transaction_timeout. Arm a shorter
/// positive native timer before COMMIT so cancellation can wake that wait.
async fn bound_commit_wait(
    transaction: &mut Transaction<'_, Postgres>,
    deadline: Instant,
) -> Result<(), DurabilityUnknown> {
    let (statement_ms, transaction_ms): (i64, i64) = sqlx::query_as(
        "SELECT (SELECT setting::bigint FROM pg_settings WHERE name='statement_timeout'), \
         (SELECT setting::bigint FROM pg_settings WHERE name='transaction_timeout')",
    )
    .fetch_one(&mut **transaction)
    .await?;
    let remaining = deadline
        .saturating_duration_since(Instant::now())
        .as_millis();
    let mut cap = i64::try_from(remaining)
        .map_err(|_| DurabilityUnknown::new("native completion timer exceeds range"))?;
    if statement_ms > 0 {
        cap = cap.min(statement_ms);
    }
    if transaction_ms > 0 {
        cap = cap.min(transaction_ms - 1);
    }
    if cap <= 0 {
        return Err(DurabilityUnknown::new(
            "no positive native completion timer remains",
        ));
    }
    sqlx::query("SELECT set_config('statement_timeout', $1, true)")
        .bind(format!("{cap}ms"))
        .execute(&mut **transaction)
        .await?;
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
struct OperationEpoch {
    observer_pid: i32,
    observer_started_at: OffsetDateTime,
    sender_pid: i32,
    sender_started_at: OffsetDateTime,
}

const ADMISSION_SQL: &str = "SELECT o.*, o.flush_lsn IS NOT NULL AND o.replay_lsn IS NOT NULL AS frontiers_present \
    FROM public.console_durability_observation_v1($1::name, $2::oid) o LIMIT 2 \
    /* console_durability_admit_v1 */";
const CONFIRMATION_SQL: &str = "SELECT o.*, o.flush_lsn IS NOT NULL AND o.replay_lsn IS NOT NULL AS frontiers_present, \
    o.flush_lsn >= $3::pg_lsn AND o.replay_lsn >= $3::pg_lsn AS confirmed \
    FROM public.console_durability_observation_v1($1::name, $2::oid) o LIMIT 2 \
    /* console_durability_observe_v1 */";

async fn observe(
    connection: &mut PgConnection,
    admission: &RemoteAdmission,
    bound: Option<&str>,
) -> Result<(OperationEpoch, bool), DurabilityUnknown> {
    let mut query = sqlx::query(if bound.is_some() {
        CONFIRMATION_SQL
    } else {
        ADMISSION_SQL
    })
    .bind(&admission.slot)
    .bind(Oid(admission.replication_role_oid));
    if let Some(bound) = bound {
        query = query.bind(bound);
    }
    let rows = query.fetch_all(connection).await?;
    let [row] = rows.as_slice() else {
        return Err(DurabilityUnknown::new(
            "admitted peer observation is not singleton",
        ));
    };
    let PeerAdmission::AdmittedPrivateNetwork { client_addr } = &admission.peer;
    let observed_addr: String = row.try_get("sender_client_addr")?;
    if row.try_get::<String, _>("primary_system_id")? != admission.primary_system_id
        || row.try_get::<OffsetDateTime, _>("primary_started_at")? != admission.primary_started_at
        || row.try_get::<bool, _>("primary_in_recovery")?
        || row.try_get::<String, _>("slot_name")? != admission.slot
        || row.try_get::<Oid, _>("replication_role_oid")?.0 != admission.replication_role_oid
        || row.try_get::<String, _>("replication_role_name")? != admission.replication_role_name
        || row.try_get::<String, _>("application_name")? != admission.application_name
        || row.try_get::<String, _>("sender_state")? != "streaming"
        || observed_addr.parse::<IpAddr>().ok().as_ref() != Some(client_addr)
        || row.try_get::<bool, _>("sender_ssl")?
        || !row.try_get::<bool, _>("frontiers_present")?
    {
        return Err(DurabilityUnknown::new(
            "admitted primary or peer identity changed",
        ));
    }
    let epoch = OperationEpoch {
        observer_pid: row.try_get("observer_pid")?,
        observer_started_at: row.try_get("observer_started_at")?,
        sender_pid: row.try_get("sender_pid")?,
        sender_started_at: row.try_get("sender_started_at")?,
    };
    let confirmed = if bound.is_some() {
        row.try_get("confirmed")?
    } else {
        false
    };
    Ok((epoch, confirmed))
}

/// Run one tenant-scoped owner transaction and confirm its completion. The
/// closure must neither commit nor roll back the borrowed transaction. A known
/// rejection preserves its domain error after confirmed rollback; unknown
/// transport outcomes and dropped futures close the retained connection.
pub async fn with_durability_transaction<F, T, E>(
    pool: &PgPool,
    org: OrgId,
    policy: &DurabilityPolicy,
    operation: F,
    known_rejection: fn(&E) -> bool,
) -> Result<T, CompletionError<E>>
where
    F: for<'tx> FnOnce(
        &'tx mut Transaction<'_, Postgres>,
    ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'tx>>,
{
    // The single budget starts before acquisition, not after COMMIT or capture.
    let deadline = match &policy.0 {
        PolicyMode::LocalDevelopment {} => None,
        PolicyMode::RequiredRemoteApply(admission) => {
            Some(Instant::now() + Duration::from_millis(admission.timeout_ms))
        }
    };
    let work = async {
        let mut held = RetainedConnection::new(
            pool.acquire().await.map_err(DurabilityUnknown::from)?,
            pool,
            deadline.is_some(),
        );
        let admitted_epoch = match &policy.0 {
            PolicyMode::LocalDevelopment {} => None,
            PolicyMode::RequiredRemoteApply(admission) => {
                Some(observe(held.connection()?, admission, None).await?.0)
            }
        };
        let mut transaction = held
            .connection()?
            .begin()
            .await
            .map_err(DurabilityUnknown::from)?;
        sqlx::query("SELECT set_config('app.current_org', $1, true)")
            .bind(org.to_string())
            .execute(&mut *transaction)
            .await
            .map_err(DurabilityUnknown::from)?;
        ensure_deadline(deadline)?;
        let value = match operation(&mut transaction).await {
            Ok(value) => value,
            Err(error) => {
                transaction
                    .rollback()
                    .await
                    .map_err(DurabilityUnknown::from)?;
                ensure_deadline(deadline)?;
                held.reusable = known_rejection(&error);
                return Err(CompletionError::Operation(error));
            }
        };
        // This also ends receipt replay's read transaction before capturing WAL.
        if let Some(deadline) = deadline {
            bound_commit_wait(&mut transaction, deadline).await?;
        }
        ensure_deadline(deadline)?;
        transaction
            .commit()
            .await
            .map_err(DurabilityUnknown::from)?;
        if let (PolicyMode::RequiredRemoteApply(admission), Some(epoch)) =
            (&policy.0, admitted_epoch)
        {
            // SQLx has no pg_lsn Rust type. Text is transport only; both comparisons
            // below are PostgreSQL pg_lsn operations against this one fixed token.
            let bound: String = sqlx::query_scalar(
                "SELECT pg_current_wal_insert_lsn()::text /* console_durability_capture_v1 */",
            )
            .fetch_one(held.connection()?)
            .await
            .map_err(DurabilityUnknown::from)?;
            loop {
                let (observed, confirmed) =
                    observe(held.connection()?, admission, Some(&bound)).await?;
                if observed != epoch {
                    return Err(DurabilityUnknown::new(
                        "operation session or sender epoch changed",
                    )
                    .into());
                }
                if confirmed {
                    break;
                }
                sleep(Duration::from_millis(50)).await;
            }
        }
        ensure_deadline(deadline)?;
        held.reusable = true;
        Ok(value)
    };
    match deadline {
        Some(deadline) => timeout_at(deadline, work)
            .await
            .map_err(|_| DurabilityUnknown::new("completion deadline elapsed"))?,
        None => work.await,
    }
}
