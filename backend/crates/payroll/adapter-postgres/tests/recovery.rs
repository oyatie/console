#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Copy into payroll adapter tests/recovery.rs on reviewed test candidate.
//! Actual PgPayRunPort; no model owner. New harness required, not ordinary CI PG.
use console_kernel_core::{OrgId, UserId};
use console_ontology_canonical_domain::{CanonicalPort, CommandId, CommandReceipt};
use console_payroll_adapter_postgres::pay_run::{PayRunCommand, PayRunQuery, PgPayRunPort};
use console_platform_db::durability::DurabilityPolicy;
use console_platform_test_support::{TestDatabaseLogin, login_test_pool, seed_org_and_super_admin};
use console_workflow_domain::{PayrollDraftStaging, StagePayrollDraft};
use serde_json::{Value, json};
use sqlx::{PgPool, Row};
use std::time::Duration;
use time::macros::date;
use uuid::Uuid;

fn control(action: &str) {
    let status = std::process::Command::new(
        std::env::var("CONSOLE_RECOVERY_CONTROL").expect("two-node harness required"),
    )
    .arg(action)
    .status()
    .expect("execute exact owned fixture control");
    assert!(status.success(), "fixture control failed: {action}");
}
struct ResumeOnDrop;
impl Drop for ResumeOnDrop {
    fn drop(&mut self) {
        let _ = std::process::Command::new(std::env::var("CONSOLE_RECOVERY_CONTROL").unwrap())
            .arg("resume-replay")
            .status();
    }
}
fn create(org: OrgId, actor: UserId) -> PayRunCommand {
    PayRunCommand {
        org_id: org,
        actor_id: actor,
        command_id: CommandId::from_uuid(Uuid::new_v4()),
        query: PayRunQuery::CreateRun {
            run_id: Uuid::new_v4(),
            period_start: date!(2026 - 06 - 01),
            period_end: date!(2026 - 06 - 30),
            connector: Some("m2".into()),
            job: Some("payroll_draft".into()),
        },
        action_key: "revise".into(),
        object_type_id: Uuid::nil(),
    }
}

// The descriptor's stable peer identity is admitted by this owned two-node
// fixture, whose isolated network, resource labels and replication secret are
// independently checked by recovery_control. This is not production mTLS proof.
async fn install_reviewed_observer(owner: &PgPool) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../ops/postgres-install-durability-observer.sql");
    let sql = std::fs::read_to_string(path).expect("real shared observer installer is required");
    control("assert-topology");
    let mut transaction = owner.begin().await.unwrap();
    sqlx::raw_sql("SET LOCAL statement_timeout='15s'; SET LOCAL lock_timeout='2s'")
        .execute(transaction.as_mut())
        .await
        .unwrap();
    let (database, admitted): (String, bool) = sqlx::query_as(
        "SELECT current_database(), session_user='console_buck_admin' AND current_user=session_user \
         AND current_setting('console.sqlx_test_bootstrap',true)='buck-sqlx-superuser-v1' \
         AND (SELECT rolsuper FROM pg_catalog.pg_roles WHERE rolname=current_user) \
         AND (SELECT pg_get_userbyid(datdba)=current_user FROM pg_catalog.pg_database WHERE datname=current_database())",
    ).fetch_one(transaction.as_mut()).await.unwrap();
    assert!(
        admitted,
        "marked disposable admin-owned database required before installer"
    );
    assert_eq!(
        owner.connect_options().get_database(),
        Some(database.as_str())
    );
    let suffix = database
        .strip_prefix("_sqlx_test_")
        .expect("SQLx fixture database required");
    assert!(
        suffix.len() == 52
            && suffix
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
    );
    sqlx::raw_sql(sqlx::AssertSqlSafe(sql))
        .execute(transaction.as_mut())
        .await
        .unwrap();
    transaction.commit().await.unwrap();
}

async fn admitted_fixture_descriptor(owner: &PgPool, timeout_ms: u64) -> Value {
    install_reviewed_observer(owner).await;
    control("assert-topology");
    let row = sqlx::query("SELECT c.system_identifier::text AS system_id, pg_postmaster_start_time() AS primary_start, r.usesysid::bigint AS role_oid, r.usename::text AS role_name, host(r.client_addr) AS client_addr, x.ssl FROM pg_control_system() c CROSS JOIN pg_replication_slots s JOIN pg_stat_replication r ON r.pid=s.active_pid JOIN pg_stat_ssl x ON x.pid=r.pid WHERE s.slot_name='console_recovery_s1' AND s.slot_type='physical' AND s.active AND r.application_name='console_recovery_s1' AND r.usename='console_fixture_replica' AND r.state='streaming'")
        .fetch_one(owner).await.unwrap();
    assert!(
        !row.get::<bool, _>("ssl"),
        "fixture transport is explicitly private-network SCRAM"
    );
    let started: time::OffsetDateTime = row.get("primary_start");
    json!({
        "mode": "required_remote_apply",
        "primary_system_id": row.get::<String, _>("system_id"),
        "primary_started_at": started.format(&time::format_description::well_known::Rfc3339).unwrap(),
        "slot": "console_recovery_s1",
        "replication_role_oid": row.get::<i64, _>("role_oid"),
        "replication_role_name": row.get::<String, _>("role_name"),
        "application_name": "console_recovery_s1",
        "peer": {"mode": "admitted_private_network", "client_addr": row.get::<String, _>("client_addr")},
        "timeout_ms": timeout_ms
    })
}

// Bind the real owner to the independently admitted fixture descriptor.
fn required_port(pool: PgPool, descriptor: &Value) -> PgPayRunPort {
    PgPayRunPort::new(
        pool,
        tokio::runtime::Handle::current(),
        DurabilityPolicy::from_json(&descriptor.to_string()).expect("validated fixture policy"),
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Backend {
    pid: i32,
    started: time::OffsetDateTime,
    application_name: String,
}

async fn observed_backend(owner: &PgPool, pid: i32) -> Backend {
    let (started, application_name) = sqlx::query_as("SELECT backend_start, application_name FROM pg_stat_activity WHERE pid=$1 AND datname=current_database()")
        .bind(pid).fetch_one(owner).await.unwrap();
    Backend {
        pid,
        started,
        application_name,
    }
}

async fn runtime_pool(owner: &PgPool, label: &str) -> (PgPool, Backend) {
    let label = label.to_owned();
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .after_connect(move |conn, _| {
            let label = label.clone();
            Box::pin(async move {
                sqlx::query("SELECT set_config('application_name',$1,false)")
                    .bind(label)
                    .execute(conn)
                    .await?;
                Ok(())
            })
        })
        .connect(&console_platform_test_support::login_test_database_url(
            owner,
            TestDatabaseLogin::Business,
        ))
        .await
        .unwrap();
    let pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&pool)
        .await
        .unwrap();
    (pool, observed_backend(owner, pid).await)
}

async fn standby_pool(owner: &PgPool) -> PgPool {
    let port = std::env::var("CONSOLE_RECOVERY_STANDBY_PORT")
        .unwrap()
        .parse()
        .unwrap();
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(2)
        .connect_with(owner.connect_options().as_ref().clone().port(port))
        .await
        .unwrap()
}

async fn wal_bound(owner: &PgPool) -> String {
    sqlx::query_scalar("SELECT pg_current_wal_insert_lsn()::text")
        .fetch_one(owner)
        .await
        .unwrap()
}

async fn assert_paused_below(standby: &PgPool, bound: &str) {
    let paused_below: bool = sqlx::query_scalar("SELECT pg_is_in_recovery() AND pg_get_wal_replay_pause_state()='paused' AND pg_last_wal_replay_lsn() < $1::pg_lsn")
        .bind(bound).fetch_one(standby).await.unwrap();
    assert!(
        paused_below,
        "independent native pause/frontier witness required"
    );
}

async fn observation_or_finished<T>(
    owner: &PgPool,
    backend: &Backend,
    call: &tokio::task::JoinHandle<T>,
) -> bool {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let observed: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_stat_activity WHERE pid=$1 AND backend_start=$2 AND datname=current_database() AND application_name=$3 AND query LIKE '%/* console_durability_observe_v1 */%')")
                .bind(backend.pid).bind(backend.started).bind(&backend.application_name).fetch_one(owner).await.unwrap();
            if observed { return true; }
            if call.is_finished() { return false; }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }).await.expect("native owner observation or actual completed result required")
}

async fn sync_wait<T>(owner: &PgPool, backend: &Backend, call: &tokio::task::JoinHandle<T>) {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let waiting: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_stat_activity WHERE pid=$1 AND backend_start=$2 AND datname=current_database() AND application_name=$3 AND wait_event='SyncRep')")
                .bind(backend.pid).bind(backend.started).bind(&backend.application_name).fetch_one(owner).await.unwrap();
            if waiting { break; }
            assert!(!call.is_finished(), "fresh owner did not reach actual SyncRep COMMIT");
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }).await.expect("exact owner SyncRep witness");
}

async fn cancel_exact_commit(owner: &PgPool, backend: &Backend) {
    let canceled: Option<bool> = sqlx::query_scalar("SELECT pg_cancel_backend(pid) FROM pg_stat_activity WHERE pid=$1 AND backend_start=$2 AND datname=current_database() AND application_name=$3 AND wait_event='SyncRep'")
        .bind(backend.pid).bind(backend.started).bind(&backend.application_name).fetch_optional(owner).await.unwrap();
    assert_eq!(canceled, Some(true));
}

async fn command_receipt_rows(pool: &PgPool, org: OrgId) -> Value {
    sqlx::query_scalar("SELECT jsonb_build_object('receipts',(SELECT COALESCE(jsonb_agg(to_jsonb(r) || jsonb_build_object('xmin',r.xmin::text) ORDER BY r.command_id),'[]'::jsonb) FROM ont_action_command_receipts r WHERE r.org_id=$1),'drafts',(SELECT COALESCE(jsonb_agg(to_jsonb(d) ORDER BY d.id),'[]'::jsonb) FROM payroll_draft_runs d WHERE d.org_id=$1))")
        .bind(org.as_uuid()).fetch_one(pool).await.unwrap()
}

async fn locally_visible_receipt(owner: &PgPool, command: &PayRunCommand) -> Value {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if let Some(receipt) = sqlx::query_scalar(
                "SELECT receipt FROM ont_action_command_receipts WHERE org_id=$1 AND command_id=$2",
            )
            .bind(command.org_id.as_uuid())
            .bind(command.command_id.as_uuid())
            .fetch_optional(owner)
            .await
            .unwrap()
            {
                return receipt;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("actual local commit visibility")
}

async fn finish_command(
    call: tokio::task::JoinHandle<
        Result<CommandReceipt, console_payroll_adapter_postgres::pay_run::PayRunError>,
    >,
) -> Result<CommandReceipt, console_payroll_adapter_postgres::pay_run::PayRunError> {
    tokio::time::timeout(Duration::from_secs(20), call)
        .await
        .unwrap()
        .unwrap()
}

struct RestoreSyncPolicy;
impl Drop for RestoreSyncPolicy {
    fn drop(&mut self) {
        let _ = std::process::Command::new(std::env::var("CONSOLE_RECOVERY_CONTROL").unwrap())
            .arg("restore-sync-policy")
            .status();
    }
}

// Leave a real original command committed locally but not replayed; never
// inject a receipt or simulate a canonical owner to create this condition.
async fn original_lost_while_paused(
    owner: &PgPool,
    descriptor: &Value,
    command: &PayRunCommand,
) -> Value {
    let (pool, backend) = runtime_pool(owner, "console-recovery-local-original").await;
    let port = required_port(pool.clone(), descriptor);
    let sent = command.clone();
    let call = tokio::task::spawn_blocking(move || port.execute(&sent));
    sync_wait(owner, &backend, &call).await;
    let terminated: Option<bool> = sqlx::query_scalar("SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE pid=$1 AND backend_start=$2 AND datname=current_database() AND application_name=$3 AND wait_event='SyncRep'")
        .bind(backend.pid).bind(backend.started).bind(&backend.application_name).fetch_optional(owner).await.unwrap();
    assert_eq!(terminated, Some(true));
    assert!(finish_command(call).await.is_err());
    let receipt = locally_visible_receipt(owner, command).await;
    pool.close().await;
    receipt
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn replay_cannot_acknowledge_a_locally_visible_receipt_before_remote_apply(owner: PgPool) {
    control("assert-topology");
    let org = OrgId::from_uuid(Uuid::from_u128(0xe3b0_0000_0000_0000_0000_0000_0000_0081));
    let actor = seed_org_and_super_admin(&owner, *org.as_uuid(), "recovery-owner").await;
    let login_url =
        console_platform_test_support::login_test_database_url(&owner, TestDatabaseLogin::Business);
    let first_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .after_connect(|c, _| {
            Box::pin(async move {
                sqlx::query("SET application_name='console-recovery-first'")
                    .execute(c)
                    .await?;
                Ok(())
            })
        })
        .connect(&login_url)
        .await
        .unwrap();
    let second_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .after_connect(|c, _| {
            Box::pin(async move {
                sqlx::query("SET application_name='console-recovery-replay'")
                    .execute(c)
                    .await?;
                Ok(())
            })
        })
        .connect(&login_url)
        .await
        .unwrap();
    for pool in [&first_pool, &second_pool] {
        let mode: String = sqlx::query_scalar("SHOW synchronous_commit")
            .fetch_one(pool)
            .await
            .unwrap();
        assert_eq!(
            mode, "remote_apply",
            "actual writer has no session durability override"
        );
    }
    let second_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&second_pool)
        .await
        .unwrap();
    let baseline: i64 =
        sqlx::query_scalar("SELECT count(*) FROM payroll_draft_runs WHERE org_id=$1")
            .bind(org.as_uuid())
            .fetch_one(&owner)
            .await
            .unwrap();
    assert_eq!(
        baseline, 0,
        "isolated Company begins without payroll effects"
    );
    let pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&first_pool)
        .await
        .unwrap();
    let backend_start: time::OffsetDateTime = sqlx::query_scalar(
        "SELECT backend_start FROM pg_stat_activity WHERE pid=$1 AND datname=current_database()",
    )
    .bind(pid)
    .fetch_one(&owner)
    .await
    .unwrap();
    let standby_port: u16 = std::env::var("CONSOLE_RECOVERY_STANDBY_PORT")
        .unwrap()
        .parse()
        .unwrap();
    let standby = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect_with(owner.connect_options().as_ref().clone().port(standby_port))
        .await
        .unwrap();
    let descriptor = admitted_fixture_descriptor(&owner, 15_000).await;
    let first = required_port(first_pool.clone(), &descriptor);
    let second = required_port(second_pool.clone(), &descriptor);
    let command = create(org, actor);
    let command_uuid = *command.command_id.as_uuid();
    control("pause-replay");
    let resume = ResumeOnDrop;
    let original_command = command.clone();
    let original = tokio::task::spawn_blocking(move || first.execute(&original_command));
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let waiting: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_stat_activity WHERE pid=$1 AND application_name='console-recovery-first' AND wait_event='SyncRep')")
                .bind(pid).fetch_one(&owner).await.unwrap();
            if waiting { break; }
            assert!(!original.is_finished(), "owner completed before SyncRep fault witness");
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }).await.expect("actual owner COMMIT must wait on SyncRep");
    // A live SyncRep waiter is still present in PostgreSQL's procarray: its
    // committed WAL is not yet visible to other transactions. This positive
    // control preserves the falsification of the original fixture premise.
    let live_visible: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM ont_action_command_receipts WHERE org_id=$1 AND command_id=$2",
    )
    .bind(org.as_uuid())
    .bind(command_uuid)
    .fetch_one(&owner)
    .await
    .unwrap();
    assert_eq!(
        live_visible, 0,
        "live SyncRep transaction remains snapshot-invisible"
    );
    let terminated: Option<bool> = sqlx::query_scalar("SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE pid=$1 AND backend_start=$2 AND datname=current_database() AND application_name='console-recovery-first' AND wait_event='SyncRep'")
        .bind(pid).bind(backend_start).fetch_optional(&owner).await.unwrap();
    assert_eq!(
        terminated,
        Some(true),
        "terminate only the exact witnessed synthetic backend"
    );
    let original_outcome = tokio::time::timeout(Duration::from_secs(10), original)
        .await
        .unwrap()
        .unwrap();
    assert!(
        original_outcome.is_err(),
        "terminated COMMIT transport must not return a known success"
    );
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM pg_stat_activity WHERE pid=$1 AND backend_start=$2)",
            )
            .bind(pid)
            .bind(backend_start)
            .fetch_one(&owner)
            .await
            .unwrap();
            if !exists {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("terminated backend disappearance");
    let paused: bool = sqlx::query_scalar(
        "SELECT pg_is_in_recovery() AND pg_get_wal_replay_pause_state()='paused'",
    )
    .fetch_one(&standby)
    .await
    .unwrap();
    assert!(paused);
    let remote_visible: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM ont_action_command_receipts WHERE org_id=$1 AND command_id=$2",
    )
    .bind(org.as_uuid())
    .bind(command_uuid)
    .fetch_one(&standby)
    .await
    .unwrap();
    assert_eq!(remote_visible, 0, "receipt is not yet applied remotely");
    let receipt: sqlx::postgres::PgRow = sqlx::query("SELECT payload_digest,receipt FROM ont_action_command_receipts WHERE org_id=$1 AND command_id=$2")
        .bind(org.as_uuid()).bind(command_uuid).fetch_one(&owner).await.unwrap();
    let original_bytes: serde_json::Value = receipt.get("receipt");
    let replay_command = command.clone();
    let replay_backend = observed_backend(&owner, second_pid).await;
    let lower = wal_bound(&owner).await;
    let mut replay = tokio::task::spawn_blocking(move || second.execute(&replay_command));
    // The primary-local receipt precedes this native lower bound. The real
    // retained replay backend must issue the marked observation while the
    // admitted standby remains paused below that bound. Finished-before-witness
    // still reaches the original premature-success assertion after cleanup.
    let progress_witnessed =
        observation_or_finished(&owner, &replay_backend, &replay).await || replay.is_finished();
    assert_paused_below(&standby, &lower).await;
    let early = tokio::time::timeout(Duration::from_millis(300), &mut replay).await;
    let premature_success = matches!(&early, Ok(Ok(Ok(_))));
    control("resume-replay");
    drop(resume);
    let replayed = match early {
        Ok(result) => result.unwrap().unwrap(),
        Err(_) => tokio::time::timeout(Duration::from_secs(10), replay)
            .await
            .unwrap()
            .unwrap()
            .unwrap(),
    };
    let durable: serde_json::Value = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            if let Some(value) = sqlx::query_scalar(
                "SELECT receipt FROM ont_action_command_receipts WHERE org_id=$1 AND command_id=$2",
            )
            .bind(org.as_uuid())
            .bind(command_uuid)
            .fetch_optional(&standby)
            .await
            .unwrap()
            {
                return value;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("retained receipt becomes visible on resumed standby");
    assert_eq!(durable, original_bytes);
    assert_eq!(replayed.result(), &durable);
    let count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM ont_action_command_receipts WHERE org_id=$1 AND command_id=$2",
    )
    .bind(org.as_uuid())
    .bind(command_uuid)
    .fetch_one(&owner)
    .await
    .unwrap();
    assert_eq!(count, 1);
    let draft_id: Uuid = replayed.result()["draft_run_id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    let effects: i64 =
        sqlx::query_scalar("SELECT count(*) FROM payroll_draft_runs WHERE org_id=$1 AND id=$2")
            .bind(org.as_uuid())
            .bind(draft_id)
            .fetch_one(&owner)
            .await
            .unwrap();
    assert_eq!(effects, 1);
    let all_effects: i64 =
        sqlx::query_scalar("SELECT count(*) FROM payroll_draft_runs WHERE org_id=$1")
            .bind(org.as_uuid())
            .fetch_one(&owner)
            .await
            .unwrap();
    assert_eq!(
        all_effects, 1,
        "replay cannot create a second effect with a different id"
    );
    println!(
        "recovery-accounting {{\"offered_intents\":1,\"attempts\":2,\"final_success_intents\":1,\"effects\":1,\"receipts\":1,\"premature_replay_success\":{premature_success}}}"
    );
    control("watermarks");
    standby.close().await;
    first_pool.close().await;
    second_pool.close().await;
    assert!(
        progress_witnessed,
        "INCONCLUSIVE: replay owner progress/wait was not witnessed; silence cannot prove durability"
    );
    assert!(
        !premature_success,
        "current owner replay acknowledged primary-local receipt after original COMMIT transport terminated while standby replay remained paused"
    );
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn lost_commit_transport_reply_replays_one_real_payroll_effect(owner: PgPool) {
    control("assert-topology");
    assert_eq!(
        std::env::var("CONSOLE_RECOVERY_CUT").unwrap(),
        "after-commit-response"
    );
    let org = OrgId::from_uuid(Uuid::from_u128(0xe3b0_0000_0000_0000_0000_0000_0000_0082));
    let actor = seed_org_and_super_admin(&owner, *org.as_uuid(), "transport-recovery").await;
    let direct = login_test_pool(&owner, TestDatabaseLogin::Business).await;
    let options: sqlx::postgres::PgConnectOptions =
        console_platform_test_support::login_test_database_url(&owner, TestDatabaseLogin::Business)
            .parse()
            .unwrap();
    let port: u16 = std::env::var("CONSOLE_RECOVERY_RELAY_PORT")
        .unwrap()
        .parse()
        .unwrap();
    let transport = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect_with(
            options
                .port(port)
                .ssl_mode(sqlx::postgres::PgSslMode::Disable),
        )
        .await
        .unwrap();
    let identity: (String, String) = sqlx::query_as("SELECT session_user::text,current_user::text")
        .fetch_one(&transport)
        .await
        .unwrap();
    assert_eq!(identity, ("console_rt".to_owned(), "console_rt".to_owned()));
    let mode: String = sqlx::query_scalar("SHOW synchronous_commit")
        .fetch_one(&transport)
        .await
        .unwrap();
    assert_eq!(
        mode, "remote_apply",
        "actual relay writer cannot override durability"
    );
    let baseline: i64 =
        sqlx::query_scalar("SELECT count(*) FROM payroll_draft_runs WHERE org_id=$1")
            .bind(org.as_uuid())
            .fetch_one(&owner)
            .await
            .unwrap();
    assert_eq!(baseline, 0);
    let command = create(org, actor);
    let command_uuid = *command.command_id.as_uuid();
    let descriptor = admitted_fixture_descriptor(&owner, 15_000).await;
    let first = required_port(transport.clone(), &descriptor);
    let retry = required_port(direct.clone(), &descriptor);
    let sent = command.clone();
    let unknown = tokio::time::timeout(
        Duration::from_secs(10),
        tokio::task::spawn_blocking(move || first.execute(&sent)),
    )
    .await
    .unwrap()
    .unwrap();
    assert!(
        unknown.is_err(),
        "withheld COMMIT reply must not reach owner as success"
    );
    let evidence = std::path::PathBuf::from(std::env::var("CONSOLE_RECOVERY_EVIDENCE").unwrap());
    let events: Vec<serde_json::Value> = std::fs::read_to_string(evidence.join("relay.jsonl"))
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(
        events
            .iter()
            .filter(|e| e["event"] == "cut_before_commit_response_delivery")
            .count(),
        1
    );
    assert!(
        events
            .iter()
            .any(|e| e["event"] == "commit_backend_success_witness")
    );
    assert!(!events.iter().any(|e| e["event"] == "relay_fixture_error"));
    let standby_port: u16 = std::env::var("CONSOLE_RECOVERY_STANDBY_PORT")
        .unwrap()
        .parse()
        .unwrap();
    let standby = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect_with(owner.connect_options().as_ref().clone().port(standby_port))
        .await
        .unwrap();
    let durable: serde_json::Value = sqlx::query_scalar(
        "SELECT receipt FROM ont_action_command_receipts WHERE org_id=$1 AND command_id=$2",
    )
    .bind(org.as_uuid())
    .bind(command_uuid)
    .fetch_one(&standby)
    .await
    .unwrap();
    let retry_command = command.clone();
    let replayed = tokio::task::spawn_blocking(move || retry.execute(&retry_command))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(replayed.result(), &durable);
    let draft_id: Uuid = durable["draft_run_id"].as_str().unwrap().parse().unwrap();
    let counts: (i64, i64) = sqlx::query_as("SELECT (SELECT count(*) FROM ont_action_command_receipts WHERE org_id=$1 AND command_id=$2), (SELECT count(*) FROM payroll_draft_runs WHERE org_id=$1 AND id=$3)")
        .bind(org.as_uuid()).bind(command_uuid).bind(draft_id).fetch_one(&standby).await.unwrap();
    assert_eq!(counts, (1, 1));
    let all_effects: i64 =
        sqlx::query_scalar("SELECT count(*) FROM payroll_draft_runs WHERE org_id=$1")
            .bind(org.as_uuid())
            .fetch_one(&standby)
            .await
            .unwrap();
    assert_eq!(all_effects, 1, "extra ids are duplicate effects too");
    println!(
        "recovery-accounting {{\"offered_intents\":1,\"attempts\":2,\"unknown_response_observations\":1,\"final_success_intents\":1,\"effects\":1,\"receipts\":1}}"
    );
    standby.close().await;
    transport.close().await;
    direct.close().await;
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn fresh_commit_wait_release_requires_explicit_remote_confirmation(owner: PgPool) {
    let org = OrgId::from_uuid(Uuid::new_v4());
    let actor = seed_org_and_super_admin(&owner, *org.as_uuid(), "fresh-confirmation").await;
    let descriptor = admitted_fixture_descriptor(&owner, 15_000).await;
    let standby = standby_pool(&owner).await;
    let mut premature = Vec::new();
    let mut progressed = Vec::new();
    for clear_config in [false, true] {
        let (pool, backend) = runtime_pool(&owner, "console-recovery-fresh-release").await;
        let port = required_port(pool.clone(), &descriptor);
        let command = create(org, actor);
        let sent = command.clone();
        control("pause-replay");
        let resume = ResumeOnDrop;
        let mut call = tokio::task::spawn_blocking(move || port.execute(&sent));
        sync_wait(&owner, &backend, &call).await;
        let restore = clear_config.then(|| RestoreSyncPolicy);
        if clear_config {
            control("clear-sync-policy");
        } else {
            cancel_exact_commit(&owner, &backend).await;
        }
        let original = locally_visible_receipt(&owner, &command).await;
        let lower = wal_bound(&owner).await;
        progressed.push(observation_or_finished(&owner, &backend, &call).await);
        assert_paused_below(&standby, &lower).await;
        let early = tokio::time::timeout(Duration::from_millis(300), &mut call).await;
        premature.push(matches!(&early, Ok(Ok(Ok(_)))));
        control("resume-replay");
        drop(resume);
        drop(restore);
        let completed = match early {
            Ok(result) => result.unwrap().unwrap(),
            Err(_) => finish_command(call).await.unwrap(),
        };
        assert_eq!(completed.result(), &original);
        let remote = locally_visible_receipt(&standby, &command).await;
        assert_eq!(remote, original);
        pool.close().await;
    }
    let rows = command_receipt_rows(&owner, org).await;
    assert_eq!(rows["receipts"].as_array().unwrap().len(), 2);
    assert_eq!(rows["drafts"].as_array().unwrap().len(), 2);
    assert_eq!(command_receipt_rows(&standby, org).await, rows);
    println!(
        "recovery-accounting {{\"offered_intents\":2,\"attempts\":2,\"effects\":2,\"receipts\":2,\"fault_subcases_executed\":2,\"premature_successes\":{}}}",
        premature.iter().filter(|v| **v).count()
    );
    standby.close().await;
    assert_eq!(
        premature,
        vec![false, false],
        "fresh COMMIT warning/config release must never bypass explicit remote confirmation"
    );
    assert_eq!(
        progressed,
        vec![true, true],
        "each actual owner must reach native confirmation"
    );
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn staging_success_and_idempotent_restage_wait_for_remote_confirmation(owner: PgPool) {
    let org = OrgId::from_uuid(Uuid::new_v4());
    seed_org_and_super_admin(&owner, *org.as_uuid(), "stage-confirmation").await;
    let descriptor = admitted_fixture_descriptor(&owner, 15_000).await;
    let standby = standby_pool(&owner).await;
    let (first_pool, first_backend) = runtime_pool(&owner, "console-recovery-stage-new").await;
    let (second_pool, second_backend) = runtime_pool(&owner, "console-recovery-stage-replay").await;
    let first = required_port(first_pool.clone(), &descriptor);
    let second = required_port(second_pool.clone(), &descriptor);
    let draft = StagePayrollDraft {
        org,
        outbox_event_id: Uuid::new_v4(),
        run_id: Uuid::new_v4(),
        period_start: Some(date!(2026 - 06 - 01)),
        period_end: Some(date!(2026 - 06 - 30)),
        connector: Some("m2".into()),
        job: Some("payroll_draft".into()),
    };
    control("pause-replay");
    let resume = ResumeOnDrop;
    let sent = draft.clone();
    let mut created = tokio::spawn(async move { first.stage(sent).await });
    sync_wait(&owner, &first_backend, &created).await;
    cancel_exact_commit(&owner, &first_backend).await;
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let count: i64 =
                sqlx::query_scalar("SELECT count(*) FROM payroll_draft_runs WHERE org_id=$1")
                    .bind(org.as_uuid())
                    .fetch_one(&owner)
                    .await
                    .unwrap();
            if count == 1 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    let lower = wal_bound(&owner).await;
    let initial = command_receipt_rows(&owner, org).await;
    let first_progress = observation_or_finished(&owner, &first_backend, &created).await;
    let mut replay = tokio::spawn(async move { second.stage(draft).await });
    let replay_progress = observation_or_finished(&owner, &second_backend, &replay).await;
    assert_paused_below(&standby, &lower).await;
    let early_created = tokio::time::timeout(Duration::from_millis(300), &mut created).await;
    let early_replay = tokio::time::timeout(Duration::from_millis(300), &mut replay).await;
    let premature_created = matches!(&early_created, Ok(Ok(Ok(_))));
    let premature_replay = matches!(&early_replay, Ok(Ok(Ok(_))));
    control("resume-replay");
    drop(resume);
    let created = match early_created {
        Ok(r) => r.unwrap().unwrap(),
        Err(_) => tokio::time::timeout(Duration::from_secs(15), created)
            .await
            .unwrap()
            .unwrap()
            .unwrap(),
    };
    let replay = match early_replay {
        Ok(r) => r.unwrap().unwrap(),
        Err(_) => tokio::time::timeout(Duration::from_secs(15), replay)
            .await
            .unwrap()
            .unwrap()
            .unwrap(),
    };
    assert!(created);
    assert!(!replay);
    assert_eq!(command_receipt_rows(&owner, org).await, initial);
    assert_eq!(command_receipt_rows(&standby, org).await, initial);
    assert_eq!(initial["drafts"].as_array().unwrap().len(), 1);
    assert_eq!(
        initial["receipts"],
        json!([]),
        "stage must not invent canonical receipts"
    );
    first_pool.close().await;
    second_pool.close().await;
    standby.close().await;
    println!(
        "recovery-accounting {{\"offered_stage_intents\":1,\"attempts\":2,\"effects\":1,\"receipts\":0,\"premature_created\":{premature_created},\"premature_restage\":{premature_replay}}}"
    );
    assert!(
        !premature_created && !premature_replay,
        "both stage result paths must await actual remote confirmation before workflow can ACK"
    );
    assert!(first_progress && replay_progress);
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn required_remote_unknown_is_bounded_and_never_local_fallback(owner: PgPool) {
    let org = OrgId::from_uuid(Uuid::new_v4());
    let actor = seed_org_and_super_admin(&owner, *org.as_uuid(), "bounded-confirmation").await;
    let descriptor = admitted_fixture_descriptor(&owner, 15_000).await;
    let standby = standby_pool(&owner).await;
    let command = create(org, actor);
    control("pause-replay");
    let resume = ResumeOnDrop;
    let original = original_lost_while_paused(&owner, &descriptor, &command).await;
    let before = command_receipt_rows(&owner, org).await;
    let lower = wal_bound(&owner).await;
    let (pool, backend) = runtime_pool(&owner, "console-recovery-deadline").await;
    let mut short = descriptor.clone();
    short["timeout_ms"] = json!(700);
    let port = required_port(pool.clone(), &short);
    let sent = command.clone();
    let started = tokio::time::Instant::now();
    let call = tokio::task::spawn_blocking(move || port.execute(&sent));
    let progress = observation_or_finished(&owner, &backend, &call).await;
    let outcome = tokio::time::timeout(Duration::from_secs(3), call)
        .await
        .unwrap()
        .unwrap();
    assert_paused_below(&standby, &lower).await;
    assert!(
        outcome.is_err(),
        "paused beyond policy deadline must return unknown, never a delayed success"
    );
    assert!(progress, "deadline must follow actual native observation");
    assert!(started.elapsed() < Duration::from_secs(3));
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM pg_stat_activity WHERE pid=$1 AND backend_start=$2)",
            )
            .bind(backend.pid)
            .bind(backend.started)
            .fetch_one(&owner)
            .await
            .unwrap();
            if !exists {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("unknown outcome closes exact retained backend");
    let replacement_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_ne!(replacement_pid, backend.pid);
    assert_eq!(command_receipt_rows(&owner, org).await, before);

    // Dropping the asynchronous staging future must close its unconfirmed
    // retained transport; aborting a spawn_blocking command is not this proof.
    let (cancel_pool, cancel_backend) = runtime_pool(&owner, "console-recovery-stage-cancel").await;
    let cancel_port = required_port(cancel_pool.clone(), &descriptor);
    let draft = match &command.query {
        PayRunQuery::CreateRun {
            run_id,
            period_start,
            period_end,
            connector,
            job,
        } => StagePayrollDraft {
            org,
            outbox_event_id: *command.command_id.as_uuid(),
            run_id: *run_id,
            period_start: Some(*period_start),
            period_end: Some(*period_end),
            connector: connector.clone(),
            job: job.clone(),
        },
        _ => panic!("fixture requires actual CreateRun input"),
    };
    let canceled_stage = tokio::spawn(async move { cancel_port.stage(draft).await });
    assert!(observation_or_finished(&owner, &cancel_backend, &canceled_stage).await);
    canceled_stage.abort();
    assert!(canceled_stage.await.unwrap_err().is_cancelled());
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM pg_stat_activity WHERE pid=$1 AND backend_start=$2)",
            )
            .bind(cancel_backend.pid)
            .bind(cancel_backend.started)
            .fetch_one(&owner)
            .await
            .unwrap();
            if !exists {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("dropped stage future closes exact unconfirmed connection");
    cancel_pool.close().await;
    assert_eq!(command_receipt_rows(&owner, org).await, before);

    // An in-flight operation refuses a sender epoch change. The same immutable
    // stable policy may admit the same authenticated peer for a NEW operation.
    let stable_port = required_port(pool.clone(), &descriptor);
    let sent = command.clone();
    let active_port = stable_port.clone();
    let call = tokio::task::spawn_blocking(move || active_port.execute(&sent));
    let active_backend = observed_backend(&owner, replacement_pid).await;
    assert!(observation_or_finished(&owner, &active_backend, &call).await);
    let sender: (i32, time::OffsetDateTime) = sqlx::query_as("SELECT r.pid,r.backend_start FROM pg_replication_slots s JOIN pg_stat_replication r ON r.pid=s.active_pid WHERE s.slot_name='console_recovery_s1' AND r.usename='console_fixture_replica'").fetch_one(&owner).await.unwrap();
    let terminated: Option<bool> = sqlx::query_scalar("SELECT pg_terminate_backend(r.pid) FROM pg_replication_slots s JOIN pg_stat_replication r ON r.pid=s.active_pid WHERE s.slot_name='console_recovery_s1' AND r.usename='console_fixture_replica' AND r.pid=$1 AND r.backend_start=$2")
        .bind(sender.0).bind(sender.1).fetch_optional(&owner).await.unwrap();
    assert_eq!(terminated, Some(true));
    assert!(
        finish_command(call).await.is_err(),
        "in-flight sender epoch must not rebind"
    );
    control("resume-replay");
    drop(resume);
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let rebound: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_replication_slots s JOIN pg_stat_replication r ON r.pid=s.active_pid WHERE s.slot_name='console_recovery_s1' AND r.usename='console_fixture_replica' AND r.state='streaming' AND (r.pid<>$1 OR r.backend_start<>$2))")
                .bind(sender.0).bind(sender.1).fetch_one(&owner).await.unwrap();
            if rebound { break; }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }).await.expect("owned standby reconnects with a new sender epoch");
    let sent = command.clone();
    let replayed = finish_command(tokio::task::spawn_blocking(move || {
        stable_port.execute(&sent)
    }))
    .await
    .unwrap();
    assert_eq!(
        replayed.result(),
        &original,
        "new operation revalidates unchanged stable admission without descriptor edit"
    );

    let mut rejected = 0;
    for (key, value) in [
        ("primary_system_id", json!("1")),
        ("primary_started_at", json!("2000-01-01T00:00:00Z")),
        ("slot", json!("not_the_admitted_slot")),
        ("replication_role_oid", json!(1)),
        ("replication_role_name", json!("not_the_admitted_role")),
        ("application_name", json!("not_the_admitted_sender")),
    ] {
        let mut bad = descriptor.clone();
        bad[key] = value;
        let port = required_port(pool.clone(), &bad);
        let sent = command.clone();
        assert!(
            finish_command(tokio::task::spawn_blocking(move || port.execute(&sent)))
                .await
                .is_err(),
            "descriptor mismatch {key} must not fallback"
        );
        rejected += 1;
    }
    let mut bad_peer = descriptor.clone();
    bad_peer["peer"]["client_addr"] = json!("192.0.2.1");
    let port = required_port(pool.clone(), &bad_peer);
    let sent = command.clone();
    assert!(
        finish_command(tokio::task::spawn_blocking(move || port.execute(&sent)))
            .await
            .is_err()
    );
    rejected += 1;
    let forbidden: bool = sqlx::query_scalar("SELECT pg_has_role(current_user,'pg_monitor','MEMBER') OR pg_has_role(current_user,'pg_read_all_stats','MEMBER') OR pg_has_role(current_user,'console_durability_observer','MEMBER') OR pg_has_role(current_user,'console_fixture_replica','MEMBER') OR has_function_privilege(current_user,'pg_catalog.pg_control_system()','EXECUTE')")
        .fetch_one(&pool).await.unwrap();
    assert!(
        !forbidden,
        "runtime must not gain monitoring/control or owner membership"
    );
    sqlx::query("REVOKE EXECUTE ON FUNCTION public.console_durability_observation_v1(name,oid) FROM console_rt").execute(&owner).await.unwrap();
    let port = required_port(pool.clone(), &descriptor);
    let sent = command.clone();
    assert!(
        finish_command(tokio::task::spawn_blocking(move || port.execute(&sent)))
            .await
            .is_err(),
        "missing observer capability is not local policy"
    );
    sqlx::query("GRANT EXECUTE ON FUNCTION public.console_durability_observation_v1(name,oid) TO console_rt").execute(&owner).await.unwrap();

    // Confirmed rollback of a known domain rejection preserves pool reuse.
    let port = required_port(pool.clone(), &descriptor);
    let pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&pool)
        .await
        .unwrap();
    let exact = observed_backend(&owner, pid).await;
    let mut conflict = command.clone();
    if let PayRunQuery::CreateRun { connector, .. } = &mut conflict.query {
        *connector = Some("payload-digest-conflict".into());
    }
    let error = finish_command(tokio::task::spawn_blocking(move || port.execute(&conflict)))
        .await
        .unwrap_err();
    assert!(matches!(
        error,
        console_payroll_adapter_postgres::pay_run::PayRunError::DigestConflict(_)
    ));
    let after_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        observed_backend(&owner, after_pid).await,
        exact,
        "confirmed domain rollback releases a reusable connection"
    );
    assert_eq!(command_receipt_rows(&owner, org).await, before);
    assert_eq!(command_receipt_rows(&standby, org).await, before);
    println!(
        "recovery-accounting {{\"effects\":1,\"receipts\":1,\"descriptor_negatives\":{rejected},\"deadline_unknown\":1,\"inflight_epoch_unknown\":1,\"stable_reconnect_replay\":1,\"observer_denial\":1,\"known_rollback_reused\":1}}"
    );
    pool.close().await;
    standby.close().await;
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn finite_remote_bound_and_repeated_replay_preserve_exact_rows(owner: PgPool) {
    let org = OrgId::from_uuid(Uuid::new_v4());
    let actor = seed_org_and_super_admin(&owner, *org.as_uuid(), "finite-confirmation").await;
    let descriptor = admitted_fixture_descriptor(&owner, 15_000).await;
    sqlx::raw_sql("CREATE TABLE public.durability_unrelated_fixture(id integer PRIMARY KEY); INSERT INTO public.durability_unrelated_fixture VALUES(1)").execute(&owner).await.unwrap();
    let standby = standby_pool(&owner).await;
    let mut reader = standby.acquire().await.unwrap();
    sqlx::query("BEGIN").execute(&mut *reader).await.unwrap();
    let value: i32 = sqlx::query_scalar("SELECT id FROM public.durability_unrelated_fixture")
        .fetch_one(&mut *reader)
        .await
        .unwrap();
    assert_eq!(value, 1);
    let reader_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *reader)
        .await
        .unwrap();
    let command = create(org, actor);
    control("pause-replay");
    let resume = ResumeOnDrop;
    let original = original_lost_while_paused(&owner, &descriptor, &command).await;
    let before = command_receipt_rows(&owner, org).await;
    let (pool, backend) = runtime_pool(&owner, "console-recovery-fixed-target").await;
    let port = required_port(pool.clone(), &descriptor);
    let sent = command.clone();
    let mut replay = tokio::task::spawn_blocking(move || port.execute(&sent));
    let progressed = observation_or_finished(&owner, &backend, &replay).await;
    let earlier_upper_bound = wal_bound(&owner).await;
    assert_paused_below(&standby, &earlier_upper_bound).await;
    let early = tokio::time::timeout(Duration::from_millis(200), &mut replay).await;
    let premature = matches!(&early, Ok(Ok(Ok(_))));
    // This is unrelated workload, not a durability barrier in the product.
    // Its later AccessExclusiveLock WAL cannot replay until reader releases.
    let mut unrelated = owner.acquire().await.unwrap();
    sqlx::query("SET synchronous_commit='local'")
        .execute(&mut *unrelated)
        .await
        .unwrap();
    sqlx::query("TRUNCATE public.durability_unrelated_fixture")
        .execute(&mut *unrelated)
        .await
        .unwrap();
    let later: String = sqlx::query_scalar("SELECT pg_current_wal_insert_lsn()::text")
        .fetch_one(&mut *unrelated)
        .await
        .unwrap();
    sqlx::query("RESET synchronous_commit")
        .execute(&mut *unrelated)
        .await
        .unwrap();
    drop(unrelated);
    control("resume-replay");
    drop(resume);
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let conflict: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_locks held JOIN pg_locks waiting ON held.locktype=waiting.locktype AND held.database=waiting.database AND held.relation=waiting.relation WHERE held.pid=$1 AND held.granted AND NOT waiting.granted AND waiting.mode='AccessExclusiveLock')")
                .bind(reader_pid).fetch_one(&standby).await.unwrap();
            if conflict { break; }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }).await.expect("actual later hot-standby replay lock conflict");
    let replayed = match early {
        Ok(result) => result.unwrap().unwrap(),
        Err(_) => tokio::time::timeout(Duration::from_secs(5), replay)
            .await
            .unwrap()
            .unwrap()
            .unwrap(),
    };
    let bounded: bool = sqlx::query_scalar(
        "SELECT pg_last_wal_replay_lsn() >= $1::pg_lsn AND pg_last_wal_replay_lsn() < $2::pg_lsn",
    )
    .bind(&earlier_upper_bound)
    .bind(&later)
    .fetch_one(&standby)
    .await
    .unwrap();
    assert!(
        bounded,
        "complete while later unrelated WAL remains unapplied"
    );
    assert_eq!(replayed.result(), &original);
    sqlx::query("ROLLBACK").execute(&mut *reader).await.unwrap();
    drop(reader);
    let mut attempts = Vec::new();
    for index in 0..4 {
        let (attempt_pool, _) =
            runtime_pool(&owner, &format!("console-recovery-repeat-{index}")).await;
        let port = required_port(attempt_pool.clone(), &descriptor);
        let sent = command.clone();
        attempts.push((
            attempt_pool,
            tokio::task::spawn_blocking(move || port.execute(&sent)),
        ));
    }
    for (attempt_pool, attempt) in attempts {
        assert_eq!(finish_command(attempt).await.unwrap().result(), &original);
        attempt_pool.close().await;
    }
    assert_eq!(command_receipt_rows(&owner, org).await, before);
    assert_eq!(command_receipt_rows(&standby, org).await, before);
    assert_eq!(before["receipts"].as_array().unwrap().len(), 1);
    assert_eq!(before["drafts"].as_array().unwrap().len(), 1);
    pool.close().await;
    standby.close().await;
    println!(
        "recovery-accounting {{\"offered_intents\":1,\"attempts\":6,\"effects\":1,\"receipts\":1,\"concurrent_replays\":4,\"premature_success\":{premature}}}"
    );
    assert!(
        !premature,
        "fixed-bound replay must first wait while standby is paused"
    );
    assert!(
        progressed,
        "actual native observation must precede unrelated WAL"
    );
}

// PRIVATE APPEND-ONLY CANDIDATE for the existing recovery.rs integration target.
// Uses its imports and helpers. No existing test body changes. Not compiled/run.

async fn fresh_cleanup_backend_exists(owner: &PgPool, backend: &Backend) -> bool {
    sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM pg_stat_activity WHERE pid=$1 AND backend_start=$2 AND datname=current_database() AND application_name=$3)",
    )
    .bind(backend.pid)
    .bind(backend.started)
    .bind(&backend.application_name)
    .fetch_one(owner)
    .await
    .unwrap()
}

async fn assert_fresh_backend_closed(owner: &PgPool, backend: &Backend) {
    tokio::time::timeout(Duration::from_secs(2), async {
        while fresh_cleanup_backend_exists(owner, backend).await {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("FRESH_SYNCREP_CLEANUP: exact abandoned backend survives bounded cleanup while replay remains paused");
}

async fn set_fresh_cleanup_timers(
    pool: &PgPool,
    backend: &Backend,
    statement_ms: u64,
    transaction_ms: u64,
) {
    // Configure the existing restricted one-slot pool's actual session. These
    // are test inputs; the owner still chooses its own transaction-local cap.
    let row = sqlx::query(
        "SELECT pg_backend_pid(), set_config('statement_timeout',$1,false), set_config('transaction_timeout',$2,false)",
    )
    .bind(format!("{statement_ms}ms"))
    .bind(format!("{transaction_ms}ms"))
    .fetch_one(pool)
    .await
    .unwrap();
    assert_eq!(row.get::<i32, _>(0), backend.pid);
    let actual: (i32, i64, i64) = sqlx::query_as(
        "SELECT pg_backend_pid(), (SELECT setting::bigint FROM pg_settings WHERE name='statement_timeout'), (SELECT setting::bigint FROM pg_settings WHERE name='transaction_timeout')",
    )
    .fetch_one(pool)
    .await
    .unwrap();
    assert_eq!(
        actual,
        (backend.pid, statement_ms as i64, transaction_ms as i64)
    );
}

async fn assert_fresh_sync_wait_ended(owner: &PgPool, backend: &Backend) {
    // In the 5s policy rows this bound expires before the caller deadline.
    // Thus replacing a shorter native cap with the full budget cannot pass.
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let waiting: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM pg_stat_activity WHERE pid=$1 AND backend_start=$2 AND datname=current_database() AND application_name=$3 AND wait_event='SyncRep')",
            )
            .bind(backend.pid)
            .bind(backend.started)
            .bind(&backend.application_name)
            .fetch_one(owner)
            .await
            .unwrap();
            if !waiting {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("FRESH_SYNCREP_NATIVE_TIMER: exact witnessed COMMIT did not leave SyncRep before the shorter native cap bound");
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn fresh_commit_deadline_reclaims_backend_before_replay_resumes(owner: PgPool) {
    // Positive transaction_timeout disables an equal/longer statement timer.
    // The shorter-statement row detects overwriting an existing tighter cap.
    for (scenario, policy_ms, statement_ms, transaction_ms) in [
        ("default", 700_u64, 0_u64, 0_u64),
        ("equal-positive-timers", 5_000, 1_200, 1_200),
        ("shorter-transaction-timer", 5_000, 5_000, 1_200),
        ("shorter-statement-timer", 5_000, 400, 0),
    ] {
        println!(
            "fresh-cleanup canonical scenario={scenario} policy_ms={policy_ms} statement_ms={statement_ms} transaction_ms={transaction_ms}"
        );
        let org = OrgId::from_uuid(Uuid::new_v4());
        let actor =
            seed_org_and_super_admin(&owner, *org.as_uuid(), "fresh-deadline-cleanup").await;
        let descriptor = admitted_fixture_descriptor(&owner, policy_ms).await;
        let label = format!("console-recovery-fresh-deadline-{scenario}");
        let (pool, backend) = runtime_pool(&owner, &label).await;
        set_fresh_cleanup_timers(&pool, &backend, statement_ms, transaction_ms).await;
        let port = required_port(pool.clone(), &descriptor);
        let command = create(org, actor);
        let sent = command.clone();
        control("pause-replay");
        let resume = ResumeOnDrop;
        let started = tokio::time::Instant::now();
        let call = tokio::task::spawn_blocking(move || port.execute(&sent));
        sync_wait(&owner, &backend, &call).await;
        if statement_ms > 0 {
            assert!(
                started.elapsed() < Duration::from_millis(1_500),
                "FRESH_SYNCREP_TIMER_PREREQUISITE: witness arrived too late to distinguish native cap from remaining policy budget"
            );
        }
        assert_fresh_sync_wait_ended(&owner, &backend).await;
        let caller_bound = Duration::from_millis(policy_ms + 1_300);
        let error = tokio::time::timeout_at(started + caller_bound, call)
            .await
            .expect("FRESH_SYNCREP_DEADLINE: owner exceeded its caller budget")
            .unwrap()
            .unwrap_err();
        assert!(
            matches!(
                error,
                console_payroll_adapter_postgres::pay_run::PayRunError::DurabilityUnknown(_)
            ),
            "FRESH_SYNCREP_UNKNOWN: actual fresh COMMIT cannot return success or known rollback: {error:?}"
        );
        assert!(started.elapsed() < caller_bound);
        assert_fresh_backend_closed(&owner, &backend).await;
        let committed = command_receipt_rows(&owner, org).await;
        assert_eq!(committed["receipts"].as_array().unwrap().len(), 1);
        assert_eq!(committed["drafts"].as_array().unwrap().len(), 1);
        control("resume-replay");
        drop(resume);
        let mut retry_descriptor = descriptor.clone();
        retry_descriptor["timeout_ms"] = json!(15_000);
        let retry = required_port(pool.clone(), &retry_descriptor);
        let sent = command.clone();
        finish_command(tokio::task::spawn_blocking(move || retry.execute(&sent)))
            .await
            .unwrap();
        let standby = standby_pool(&owner).await;
        assert_eq!(command_receipt_rows(&owner, org).await, committed);
        assert_eq!(command_receipt_rows(&standby, org).await, committed);
        standby.close().await;
        pool.close().await;
    }
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn aborted_fresh_stage_retains_capacity_until_native_wait_ends(owner: PgPool) {
    for (scenario, policy_ms, statement_ms, transaction_ms) in [
        ("default", 700_u64, 0_u64, 0_u64),
        ("equal-positive-timers", 5_000, 1_200, 1_200),
        ("shorter-transaction-timer", 5_000, 5_000, 1_200),
        ("shorter-statement-timer", 5_000, 400, 0),
    ] {
        println!(
            "fresh-cleanup stage scenario={scenario} policy_ms={policy_ms} statement_ms={statement_ms} transaction_ms={transaction_ms}"
        );
        let org = OrgId::from_uuid(Uuid::new_v4());
        seed_org_and_super_admin(&owner, *org.as_uuid(), "fresh-abort-cleanup").await;
        let descriptor = admitted_fixture_descriptor(&owner, policy_ms).await;
        let label = format!("console-recovery-fresh-abort-{scenario}");
        let (pool, backend) = runtime_pool(&owner, &label).await;
        set_fresh_cleanup_timers(&pool, &backend, statement_ms, transaction_ms).await;
        let draft = StagePayrollDraft {
            org,
            outbox_event_id: Uuid::new_v4(),
            run_id: Uuid::new_v4(),
            period_start: Some(date!(2026 - 06 - 01)),
            period_end: Some(date!(2026 - 06 - 30)),
            connector: Some("m2".into()),
            job: Some("payroll_draft".into()),
        };
        let retry_draft = draft.clone();
        let port = required_port(pool.clone(), &descriptor);
        control("pause-replay");
        let resume = ResumeOnDrop;
        let started = tokio::time::Instant::now();
        let call = tokio::spawn(async move { port.stage(draft).await });
        sync_wait(&owner, &backend, &call).await;
        if statement_ms > 0 {
            assert!(
                started.elapsed() < Duration::from_millis(1_500),
                "FRESH_SYNCREP_TIMER_PREREQUISITE: witness arrived too late to distinguish native cap from remaining policy budget"
            );
        }
        call.abort();
        assert!(call.await.unwrap_err().is_cancelled());
        // Query immediately through the one-slot pool. The replacement's first
        // requested SQL observes its PID and the exact old SyncRep waiter in
        // one server snapshot; no later owner RTT can hide a sampled overlap.
        let (replacement, abandoned_waiter): (i32, bool) = tokio::time::timeout(
            Duration::from_secs(3),
            sqlx::query_as(
                "SELECT pg_backend_pid(), EXISTS(SELECT 1 FROM pg_stat_activity WHERE pid=$1 AND backend_start=$2 AND datname=current_database() AND application_name=$3 AND wait_event='SyncRep')",
            )
            .bind(backend.pid)
            .bind(backend.started)
            .bind(&backend.application_name)
            .fetch_one(&pool),
        )
        .await
        .expect("FRESH_SYNCREP_POOL_RECOVERY: retained capacity was never released within the native cap bound")
        .unwrap();
        assert_ne!(replacement, backend.pid);
        assert!(
            !abandoned_waiter,
            "FRESH_SYNCREP_CAPACITY: replacement admitted while abandoned fresh COMMIT still owns server resources"
        );
        assert_fresh_backend_closed(&owner, &backend).await;
        let committed = command_receipt_rows(&owner, org).await;
        assert_eq!(committed["receipts"].as_array().unwrap().len(), 0);
        assert_eq!(committed["drafts"].as_array().unwrap().len(), 1);
        control("resume-replay");
        drop(resume);
        let mut retry_descriptor = descriptor.clone();
        retry_descriptor["timeout_ms"] = json!(15_000);
        let retry = required_port(pool.clone(), &retry_descriptor);
        assert!(!retry.stage(retry_draft).await.unwrap());
        let standby = standby_pool(&owner).await;
        assert_eq!(command_receipt_rows(&owner, org).await, committed);
        assert_eq!(command_receipt_rows(&standby, org).await, committed);
        standby.close().await;
        pool.close().await;
    }
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn fresh_stage_transport_error_closes_pool_before_reconciliation(owner: PgPool) {
    let org = OrgId::from_uuid(Uuid::new_v4());
    seed_org_and_super_admin(&owner, *org.as_uuid(), "fresh-transport-cleanup").await;
    let descriptor = admitted_fixture_descriptor(&owner, 5_000).await;
    let (pool, backend) = runtime_pool(&owner, "console-recovery-fresh-transport").await;
    let draft = StagePayrollDraft {
        org,
        outbox_event_id: Uuid::new_v4(),
        run_id: Uuid::new_v4(),
        period_start: Some(date!(2026 - 06 - 01)),
        period_end: Some(date!(2026 - 06 - 30)),
        connector: Some("m2".into()),
        job: Some("payroll_draft".into()),
    };
    let retry_draft = draft.clone();
    let port = required_port(pool.clone(), &descriptor);
    control("pause-replay");
    let resume = ResumeOnDrop;
    let call = tokio::spawn(async move { port.stage(draft).await });
    sync_wait(&owner, &backend, &call).await;
    // Fault injection is confined to the fixture operator and the exact
    // witnessed backend. The production runtime receives no terminate grant.
    let terminated: Option<bool> = sqlx::query_scalar(
        "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE pid=$1 AND backend_start=$2 AND datname=current_database() AND application_name=$3 AND usename='console_rt' AND wait_event='SyncRep'",
    )
    .bind(backend.pid)
    .bind(backend.started)
    .bind(&backend.application_name)
    .fetch_optional(&owner)
    .await
    .unwrap();
    assert_eq!(terminated, Some(true));
    let error = tokio::time::timeout(Duration::from_secs(2), call)
        .await
        .expect("FRESH_SYNCREP_TRANSPORT: owner did not report the terminated COMMIT transport")
        .unwrap()
        .unwrap_err();
    assert_eq!(error.kind, console_kernel_core::ErrorKind::Internal);
    assert!(
        error.message.starts_with("database durability UNKNOWN:"),
        "FRESH_SYNCREP_UNKNOWN: stage must retain the existing UNKNOWN error boundary: {error}"
    );
    // Request capacity immediately after UNKNOWN, while owned async cleanup
    // may still be running. A new physical connection is a failure, even if
    // the pool would later close. PoolClosed is the control under test; a
    // transport/ping error is not treated as proof that the server backend died.
    let acquisition = tokio::time::timeout(Duration::from_secs(2), pool.acquire())
        .await
        .expect(
            "FRESH_SYNCREP_FAIL_CLOSED: transport-error cleanup did not close the original pool",
        );
    assert!(
        matches!(acquisition, Err(sqlx::Error::PoolClosed)),
        "FRESH_SYNCREP_FAIL_CLOSED: original pool must refuse replacement capacity with PoolClosed"
    );
    assert!(pool.is_closed());
    let committed = command_receipt_rows(&owner, org).await;
    assert_eq!(committed["receipts"].as_array().unwrap().len(), 0);
    assert_eq!(committed["drafts"].as_array().unwrap().len(), 1);
    control("resume-replay");
    drop(resume);
    // Recovery requires an explicit fresh pool and the unchanged policy.
    let (retry_pool, _) = runtime_pool(&owner, "console-recovery-fresh-transport-retry").await;
    let retry = required_port(retry_pool.clone(), &descriptor);
    let restaged = tokio::time::timeout(Duration::from_secs(7), retry.stage(retry_draft))
        .await
        .expect("FRESH_SYNCREP_RECONCILIATION: fresh pool did not resolve the committed draft")
        .unwrap();
    assert!(!restaged);
    let standby = standby_pool(&owner).await;
    assert_eq!(command_receipt_rows(&owner, org).await, committed);
    assert_eq!(command_receipt_rows(&standby, org).await, committed);
    assert!(pool.is_closed());
    standby.close().await;
    retry_pool.close().await;
    pool.close().await;
}
