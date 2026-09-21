#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! BE-AUTO slice 2 E2E — condition/branch node runtime against the REAL engine
//! on a genuine `PgWorkflowRuntimeStore`, driven as the non-owner `console_rt` role
//! (NOSUPERUSER, NOBYPASSRLS, FORCE RLS) with `app.current_org` armed.
//!
//! Proves the branch walk: a `condition` node's predicate over the run context
//! selects the outgoing `when` edge; each outcome drives a DIFFERENT path; and
//! the untaken (dead) branch NEVER executes — no node run row is written for it.
//! Cross-org isolation of the produced runs is asserted as `console_rt`.

use console_kernel_core::{OrgId, TraceContext};
use console_workflow_domain::TriggerType;
use console_workflow_runtime::{AuditContext, StartRunRequest, TriggeredStart, start_bound_run};
use console_workflow_runtime_adapter_postgres::PgWorkflowRuntimeStore;
use serde_json::{Value, json};
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use time::OffsetDateTime;
use uuid::Uuid;

const BRANCH_TENANT: Uuid = Uuid::from_u128(0x4d32_11d1_0000_0000_0000_0000_0000_00d1);
const OTHER_TENANT: Uuid = Uuid::from_u128(0x4d32_11d2_0000_0000_0000_0000_0000_00d2);

async fn runtime_role_pool(owner_pool: &PgPool) -> PgPool {
    let options = owner_pool.connect_options().as_ref().clone();
    PgPoolOptions::new()
        .max_connections(6)
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                sqlx::query("SET ROLE console_rt").execute(conn).await?;
                Ok(())
            })
        })
        .connect_with(options)
        .await
        .unwrap()
}

async fn arm_org(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>, org: OrgId) {
    sqlx::query("SELECT set_config('app.current_org', $1, true)")
        .bind(org.as_uuid().to_string())
        .execute(&mut **tx)
        .await
        .unwrap();
}

async fn seed_org(owner_pool: &PgPool, org: OrgId, slug: &str) {
    sqlx::query(
        "INSERT INTO organizations (id, slug, name) VALUES ($1, $2, $3) \
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(*org.as_uuid())
    .bind(slug)
    .bind(format!("Org {slug}"))
    .execute(owner_pool)
    .await
    .unwrap();
}

/// gate → decide(amount > 1000) → [true] escalate.exec (human) / [false] auto.approve (terminal)
fn branching_definition() -> Value {
    json!({
        "schema_version": "wf.exec.v1",
        "nodes": [
            {"node_key": "gate", "node_type": "object_gate"},
            {"node_key": "decide", "node_type": "condition",
             "predicate": {"field": "amount", "op": "gt", "value": 1000}},
            {"node_key": "escalate.exec", "node_type": "human_task",
             "assignee_role_key": "executive", "required_policy": "approval_decide"},
            {"node_key": "auto.approve", "node_type": "object_mutation"}
        ],
        "edges": [
            {"from": "gate", "to": "decide"},
            {"from": "decide", "to": "escalate.exec", "when": "true"},
            {"from": "decide", "to": "auto.approve", "when": "false"}
        ]
    })
}

async fn seed_branching_definition(rt_pool: &PgPool, org: OrgId) -> Uuid {
    let mut tx = rt_pool.begin().await.unwrap();
    arm_org(&mut tx, org).await;
    let definition_id: Uuid = sqlx::query_scalar(
        "INSERT INTO workflow_definitions \
             (org_id, workflow_key, display_name, object_type, status, latest_version, active_version) \
         VALUES ($1, 'automation.branching', 'Branching', 'work_order', 'ACTIVE', 1, 1) \
         RETURNING id",
    )
    .bind(*org.as_uuid())
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO workflow_definition_versions \
             (org_id, definition_id, version, status, definition, \
              required_approval_line, required_payment_line) \
         VALUES ($1, $2, 1, 'PUBLISHED', $3, FALSE, FALSE)",
    )
    .bind(*org.as_uuid())
    .bind(definition_id)
    .bind(branching_definition())
    .execute(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();
    definition_id
}

fn audit() -> AuditContext {
    // System-initiated fire (no per-request principal), like the trigger/schedule
    // producers — avoids an actor FK to a seeded user.
    AuditContext {
        actor: None,
        trace: TraceContext::generate(),
        occurred_at: OffsetDateTime::now_utc(),
    }
}

fn request(org: OrgId, definition_id: Uuid, key: &str, context: Value) -> StartRunRequest {
    StartRunRequest {
        run_id: Uuid::new_v4(),
        org_id: org,
        definition_id,
        definition_version: 1,
        trigger_type: TriggerType::Manual,
        object_type: Some("work_order".to_owned()),
        object_id: Some(Uuid::new_v4()),
        idempotency_key: key.to_owned(),
        correlation_id: format!("branch-test:{key}"),
        trace_id: None,
        input_payload: json!({}),
        context_payload: context,
        initiated_by: None,
        schedule_id: None,
    }
}

async fn node_keys(rt_pool: &PgPool, org: OrgId, run_id: Uuid) -> Vec<String> {
    let mut tx = rt_pool.begin().await.unwrap();
    arm_org(&mut tx, org).await;
    let rows = sqlx::query(
        "SELECT node_key FROM workflow_node_runs WHERE run_id = $1 ORDER BY started_at",
    )
    .bind(run_id)
    .fetch_all(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();
    rows.iter()
        .map(|r| r.get::<String, _>("node_key"))
        .collect()
}

#[sqlx::test(migrations = "../crates/platform/db/migrations")]
async fn condition_true_branch_parks_and_dead_branch_never_runs(owner_pool: PgPool) {
    let org = OrgId::from_uuid(BRANCH_TENANT);
    seed_org(&owner_pool, org, "org-branch").await;
    let rt_pool = runtime_role_pool(&owner_pool).await;
    let definition_id = seed_branching_definition(&rt_pool, org).await;
    let store = PgWorkflowRuntimeStore::new(rt_pool.clone());

    // amount > 1000 ⇒ TRUE branch ⇒ parks at the executive human task.
    let req = request(
        org,
        definition_id,
        "branch-true-0000000001",
        json!({ "amount": 5000 }),
    );
    let run_id = req.run_id;
    let outcome = start_bound_run(&store, req, &branching_definition(), &audit())
        .await
        .unwrap();
    assert!(matches!(
        outcome,
        TriggeredStart::Started { run_status, .. } if run_status == console_workflow_domain::RunStatus::Waiting
    ));

    let keys = node_keys(&rt_pool, org, run_id).await;
    assert!(keys.contains(&"gate".to_owned()));
    assert!(keys.contains(&"decide".to_owned()));
    assert!(keys.contains(&"escalate.exec".to_owned()));
    assert!(
        !keys.contains(&"auto.approve".to_owned()),
        "the dead (false) branch must never execute; node runs = {keys:?}"
    );
}

#[sqlx::test(migrations = "../crates/platform/db/migrations")]
async fn condition_false_branch_runs_to_terminal_success(owner_pool: PgPool) {
    let org = OrgId::from_uuid(BRANCH_TENANT);
    seed_org(&owner_pool, org, "org-branch").await;
    let rt_pool = runtime_role_pool(&owner_pool).await;
    let definition_id = seed_branching_definition(&rt_pool, org).await;
    let store = PgWorkflowRuntimeStore::new(rt_pool.clone());

    // amount <= 1000 ⇒ FALSE branch ⇒ drives to the terminal object_mutation.
    let req = request(
        org,
        definition_id,
        "branch-false-000000001",
        json!({ "amount": 100 }),
    );
    let run_id = req.run_id;
    let outcome = start_bound_run(&store, req, &branching_definition(), &audit())
        .await
        .unwrap();
    assert!(matches!(
        outcome,
        TriggeredStart::Started { run_status, .. } if run_status == console_workflow_domain::RunStatus::Succeeded
    ));

    let keys = node_keys(&rt_pool, org, run_id).await;
    assert!(keys.contains(&"gate".to_owned()));
    assert!(keys.contains(&"decide".to_owned()));
    assert!(keys.contains(&"auto.approve".to_owned()));
    assert!(
        !keys.contains(&"escalate.exec".to_owned()),
        "the dead (true) branch must never execute; node runs = {keys:?}"
    );

    // Cross-org isolation: another tenant sees none of these runs.
    let other = OrgId::from_uuid(OTHER_TENANT);
    seed_org(&owner_pool, other, "org-branch-other").await;
    let mut tx = rt_pool.begin().await.unwrap();
    arm_org(&mut tx, other).await;
    let visible: i64 = sqlx::query_scalar("SELECT count(*) FROM workflow_runs")
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    assert_eq!(visible, 0, "runs must be RLS-isolated to their own tenant");
}

// Canonical process_node -> WorkflowRuntimePort::commit_node_step replay proof.
// Reuse this binary's existing owner-only tenant seed and console_rt role pool.
// This proves effective restricted-role behavior, not an authenticated LOGIN flow.
async fn node_replay_rows(pool: &PgPool, org: OrgId) -> Vec<Vec<String>> {
    let mut tx = pool.begin().await.unwrap();
    arm_org(&mut tx, org).await;
    let mut snapshot = Vec::new();
    for query in [
        "SELECT to_jsonb(r)::text FROM workflow_runs r ORDER BY id",
        "SELECT to_jsonb(r)::text FROM workflow_node_runs r ORDER BY id",
        "SELECT to_jsonb(r)::text FROM workflow_outbox_events r ORDER BY id",
        "SELECT to_jsonb(r)::text FROM workflow_waiting_tasks r ORDER BY id",
        "SELECT to_jsonb(r)::text FROM audit_events r ORDER BY id",
    ] {
        snapshot.push(sqlx::query_scalar(query).fetch_all(&mut *tx).await.unwrap());
    }
    tx.commit().await.unwrap();
    snapshot
}

async fn assert_node_replay_preserves_all_effects(owner_pool: PgPool, node: Value) {
    use console_kernel_core::ErrorKind;
    use console_workflow_domain::{NodeStatus, RunStatus};
    use console_workflow_runtime::{NodeSpec, ProcessNodeRequest, process_node, start_run};

    let org = OrgId::from_uuid(BRANCH_TENANT);
    seed_org(&owner_pool, org, "org-node-replay").await;
    let rt_pool = runtime_role_pool(&owner_pool).await;
    let role: (String, bool, bool) = sqlx::query_as(
        "SELECT current_user::text, rolsuper, rolbypassrls FROM pg_roles WHERE rolname=current_user",
    ).fetch_one(&rt_pool).await.unwrap();
    assert_eq!(role, ("console_rt".to_owned(), false, false));

    // Seed an exact single-node executable definition. Effects below are written
    // only by the actual engine/adapter, never fixture INSERTs into runtime tables.
    let mut tx = rt_pool.begin().await.unwrap();
    arm_org(&mut tx, org).await;
    let definition_id: Uuid = sqlx::query_scalar(
        "INSERT INTO workflow_definitions \
         (org_id, workflow_key, display_name, object_type, status, latest_version, active_version) \
         VALUES ($1, 'automation.node_replay', 'Node replay', 'work_order', 'ACTIVE', 1, 1) RETURNING id",
    ).bind(*org.as_uuid()).fetch_one(&mut *tx).await.unwrap();
    sqlx::query(
        "INSERT INTO workflow_definition_versions \
         (org_id, definition_id, version, status, definition, required_approval_line, required_payment_line) \
         VALUES ($1, $2, 1, 'PUBLISHED', $3, FALSE, FALSE)",
    ).bind(*org.as_uuid()).bind(definition_id)
        .bind(json!({"schema_version": "wf.exec.v1", "nodes": [node.clone()], "edges": []}))
        .execute(&mut *tx).await.unwrap();
    tx.commit().await.unwrap();
    let store = PgWorkflowRuntimeStore::new(rt_pool.clone());
    let run_id = start_run(
        &store,
        request(org, definition_id, "node-replay-0000000001", json!({})),
        &audit(),
    )
    .await
    .unwrap();
    let parked = node["node_type"] == "human_task";
    let job = node["node_type"] == "job";
    let first_id = Uuid::new_v4();
    let first = ProcessNodeRequest {
        org_id: org,
        run_id,
        node_run_id: first_id,
        current_run_status: RunStatus::Running,
        run_target: if parked {
            RunStatus::Waiting
        } else {
            RunStatus::Succeeded
        },
        spec: NodeSpec::from_execution_node(&node).unwrap(),
        attempt: 1,
        input_payload: json!({"amount": 123, "memo": "기록 보존"}),
        guard_audits: Vec::new(),
    };
    let before = node_replay_rows(&rt_pool, org).await;
    assert_eq!(
        (
            before[0].len(),
            before[1].len(),
            before[2].len(),
            before[3].len()
        ),
        (1, 0, 0, 0)
    );
    let outcome = process_node(&store, first.clone(), &audit()).await.unwrap();
    assert_eq!(
        outcome.node_final_status,
        if parked {
            NodeStatus::Waiting
        } else {
            NodeStatus::Succeeded
        }
    );
    assert_eq!(outcome.run_status, first.run_target);
    let committed = node_replay_rows(&rt_pool, org).await;
    assert_eq!((committed[0].len(), committed[1].len()), (1, 1));
    assert_eq!(committed[2].len(), usize::from(job));
    assert_eq!(committed[3].len(), usize::from(parked));
    assert_eq!(
        committed[4].len(),
        before[4].len() + 1,
        "first node adds its real commit audit"
    );

    // The graph walker allocates a new node UUID on re-drive. Attempt/key/input
    // stay identical; neither fresh audit metadata nor UUID permits new effects.
    let mut replay = first.clone();
    replay.node_run_id = Uuid::new_v4();
    assert_ne!(replay.node_run_id, first_id);
    let result = process_node(&store, replay.clone(), &audit()).await;
    assert_eq!(
        node_replay_rows(&rt_pool, org).await,
        committed,
        "replay must preserve complete run/node/outbox/waiting/audit rows, including timestamps"
    );
    assert_eq!(
        result.expect("same logical node must replay successfully"),
        outcome
    );

    // Narrow immutable-input collision control: the same deterministic key
    // cannot silently claim that a different supplied input was committed.
    replay.node_run_id = Uuid::new_v4();
    replay.input_payload["amount"] = json!(124);
    let result = process_node(&store, replay, &audit()).await;
    assert_eq!(
        node_replay_rows(&rt_pool, org).await,
        committed,
        "a rejected input collision must leave every committed effect unchanged"
    );
    assert_eq!(
        result
            .expect_err("same key with different input must conflict")
            .kind,
        ErrorKind::Conflict
    );
}

#[sqlx::test(migrations = "../crates/platform/db/migrations")]
async fn replayed_gate_preserves_every_row_and_rejects_changed_input(owner_pool: PgPool) {
    assert_node_replay_preserves_all_effects(
        owner_pool,
        json!({
            "node_key": "replay.step", "node_type": "object_gate"
        }),
    )
    .await;
}

#[sqlx::test(migrations = "../crates/platform/db/migrations")]
async fn replayed_job_preserves_every_row_and_rejects_changed_input(owner_pool: PgPool) {
    assert_node_replay_preserves_all_effects(
        owner_pool,
        json!({
            "node_key": "replay.step", "node_type": "job",
            "connector_key": "internal.jobs", "job": "replay_probe"
        }),
    )
    .await;
}

#[sqlx::test(migrations = "../crates/platform/db/migrations")]
async fn replayed_human_task_preserves_every_row_and_rejects_changed_input(owner_pool: PgPool) {
    assert_node_replay_preserves_all_effects(
        owner_pool,
        json!({
            "node_key": "replay.step", "node_type": "human_task", "title": "승인 대기",
            "assignee_role_key": "executive", "required_policy": "approval_decide"
        }),
    )
    .await;
}

async fn assert_node_replay_rejects_changed_effect_material(
    owner_pool: PgPool,
    node: Value,
    changes: Vec<(&'static str, Value)>,
) {
    use console_kernel_core::ErrorKind;
    use console_workflow_domain::RunStatus;
    use console_workflow_runtime::{
        NodeOutcome, NodeSpec, ProcessNodeRequest, interpret_node, process_node,
    };

    // Retain the independently approved positive replay and changed-input proof
    // unchanged, then exercise fields that do not change the stored node output.
    assert_node_replay_preserves_all_effects(owner_pool.clone(), node.clone()).await;
    let org = OrgId::from_uuid(BRANCH_TENANT);
    let rt_pool = runtime_role_pool(&owner_pool).await;
    let store = PgWorkflowRuntimeStore::new(rt_pool.clone());
    let mut tx = rt_pool.begin().await.unwrap();
    arm_org(&mut tx, org).await;
    let (run_id, stored_node_id): (Uuid, Uuid) = sqlx::query_as(
        "SELECT r.id,n.id FROM workflow_runs r JOIN workflow_node_runs n ON n.run_id=r.id AND n.org_id=r.org_id",
    ).fetch_one(&mut *tx).await.unwrap();
    tx.commit().await.unwrap();
    let committed = node_replay_rows(&rt_pool, org).await;
    let original_spec = NodeSpec::from_execution_node(&node).unwrap();
    let original = ProcessNodeRequest {
        org_id: org,
        run_id,
        node_run_id: Uuid::new_v4(),
        current_run_status: RunStatus::Running,
        run_target: if node["node_type"] == "human_task" {
            RunStatus::Waiting
        } else {
            RunStatus::Succeeded
        },
        spec: original_spec.clone(),
        attempt: 1,
        input_payload: json!({"amount": 123, "memo": "기록 보존"}),
        guard_audits: Vec::new(),
    };
    for (field, changed) in changes {
        let mut changed_node = node.clone();
        assert_ne!(
            changed_node[field], changed,
            "material mutation must change {field}"
        );
        changed_node[field] = changed;
        let mut replay = original.clone();
        replay.node_run_id = Uuid::new_v4();
        assert_ne!(replay.node_run_id, stored_node_id);
        replay.spec = NodeSpec::from_execution_node(&changed_node).unwrap();
        assert_eq!(replay.spec.node_key, original.spec.node_key);
        assert_eq!(replay.spec.node_type, original.spec.node_type);
        assert_eq!(replay.input_payload, original.input_payload);
        // Positive mutation control: the actual interpreter changes only the
        // relevant effect material while the node's committed outcome stays equal.
        match (
            interpret_node(
                &original_spec,
                run_id,
                replay.node_run_id,
                &replay.input_payload,
            ),
            interpret_node(
                &replay.spec,
                run_id,
                replay.node_run_id,
                &replay.input_payload,
            ),
        ) {
            (
                NodeOutcome::Succeeded {
                    output: before,
                    emissions: a,
                    ..
                },
                NodeOutcome::Succeeded {
                    output: after,
                    emissions: b,
                    ..
                },
            ) => {
                assert_eq!(before, after);
                assert_eq!((a.len(), b.len()), (1, 1));
                assert_ne!(a[0].payload, b[0].payload);
                assert_eq!(a[0].idempotency_key, b[0].idempotency_key);
            }
            (NodeOutcome::Waiting { task: a, .. }, NodeOutcome::Waiting { task: b, .. }) => {
                assert_eq!(a.form_payload, b.form_payload);
                assert_eq!(a.waiting_key, b.waiting_key);
                assert_ne!(
                    (&a.title, &a.assignee_role_key, &a.required_policy),
                    (&b.title, &b.assignee_role_key, &b.required_policy)
                );
            }
            _ => panic!("material fixture must preserve the original node outcome"),
        }
        let result = process_node(&store, replay, &audit()).await;
        assert_eq!(
            node_replay_rows(&rt_pool, org).await,
            committed,
            "changed {field} must preserve all five tables exactly"
        );
        assert_eq!(
            result
                .expect_err("same-key effect material drift must conflict")
                .kind,
            ErrorKind::Conflict,
            "{field}"
        );
    }
    // Rejections must not poison a valid replay of the acknowledged material.
    let outcome = process_node(&store, original.clone(), &audit())
        .await
        .unwrap();
    assert_eq!(outcome.run_status, original.run_target);
    assert_eq!(node_replay_rows(&rt_pool, org).await, committed);
}

#[sqlx::test(migrations = "../crates/platform/db/migrations")]
async fn replayed_job_rejects_changed_emits_status_with_identical_node_output(owner_pool: PgPool) {
    assert_node_replay_rejects_changed_effect_material(
        owner_pool,
        json!({
            "node_key": "replay.step", "node_type": "job",
            "connector_key": "internal.jobs", "job": "replay_probe", "emits_status": "READY"
        }),
        vec![("emits_status", json!("CHANGED"))],
    )
    .await;
}

#[sqlx::test(migrations = "../crates/platform/db/migrations")]
async fn replayed_human_task_rejects_changed_creation_material(owner_pool: PgPool) {
    assert_node_replay_rejects_changed_effect_material(
        owner_pool,
        json!({
            "node_key": "replay.step", "node_type": "human_task", "title": "승인 대기",
            "assignee_role_key": "executive", "required_policy": "approval_decide"
        }),
        vec![
            ("required_policy", json!("different_policy")),
            ("title", json!("다른 승인 요청")),
            ("assignee_role_key", json!("different_reviewer")),
        ],
    )
    .await;
}
