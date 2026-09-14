#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Copy into payroll adapter tests/recovery.rs on reviewed test candidate.
//! Actual PgPayRunPort; no model owner. New harness required, not ordinary CI PG.
use console_kernel_core::{OrgId, UserId};
use console_ontology_canonical_domain::{CanonicalPort, CommandId};
use console_payroll_adapter_postgres::pay_run::{PayRunCommand, PayRunQuery, PgPayRunPort};
use console_platform_test_support::{TestDatabaseLogin, login_test_pool, seed_org_and_super_admin};
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
    let first = PgPayRunPort::new(first_pool.clone(), tokio::runtime::Handle::current());
    let second = PgPayRunPort::new(second_pool.clone(), tokio::runtime::Handle::current());
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
    let mut replay = tokio::task::spawn_blocking(move || second.execute(&replay_command));
    // Never accept unscheduled silence as a durability barrier. A replay must
    // complete or reach a witnessed SyncRep wait on its own exact backend.
    // A future application-side wait requires its own owner-bound witness;
    // absent that witness this candidate fails INCONCLUSIVE, never GREEN.
    let progress_witnessed = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if replay.is_finished() { return; }
            let waiting: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_stat_activity WHERE pid=$1 AND application_name='console-recovery-replay' AND wait_event='SyncRep')")
                .bind(second_pid).fetch_one(&owner).await.unwrap();
            if waiting { return; }
            let paused: bool = sqlx::query_scalar("SELECT pg_get_wal_replay_pause_state()='paused'").fetch_one(&standby).await.unwrap();
            assert!(paused, "standby replay fault must remain active");
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }).await.is_ok();
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
    let first = PgPayRunPort::new(transport.clone(), tokio::runtime::Handle::current());
    let retry = PgPayRunPort::new(direct.clone(), tokio::runtime::Handle::current());
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
