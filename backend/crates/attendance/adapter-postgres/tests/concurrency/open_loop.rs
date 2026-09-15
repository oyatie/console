//! Bounded local owner characterization. This is not a capacity benchmark,
//! an application admission queue, or a design30 command-receipt protocol.
use super::*;
use console_platform_test_support::{TestDatabaseLogin, login_test_database_url};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::time::Duration;
use tokio::task::{Id, JoinSet};
use tokio::time::{Instant, sleep_until, timeout};

const TASK_CAP: usize = 4;
const POOL_CAP: u32 = 2;
const INTENT_CAP: usize = 12;
const RELEASE_MS: u64 = 1000;
const MAX_SCHEDULER_LAG_US: u64 = 100_000;
const MAX_INTENT_AGE_US: u64 = 3_000_000;
const MAX_HISTORY_BYTES: usize = 16_384;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Intent {
    id: usize,
    branch: Uuid,
    month: String,
    arrival_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Outcome {
    Pending,
    ClientRejected,
    Committed(Uuid),
    Refused,
    Unknown,
}

#[derive(Clone, Debug)]
struct Observation {
    intent: Intent,
    admitted_us: Option<u64>,
    invoked_us: Option<u64>,
    completed_us: u64,
    outcome: Outcome,
}

#[derive(Clone, Debug, sqlx::FromRow)]
struct Effect {
    id: Uuid,
    org: Uuid,
    branch: Uuid,
    month: String,
    actor: Uuid,
}

#[derive(Clone, Debug, sqlx::FromRow)]
struct Audit {
    target: String,
    org: Uuid,
    branch: Option<Uuid>,
    actor: Option<Uuid>,
    target_type: String,
}

fn micros(start: Instant) -> u64 {
    u64::try_from(start.elapsed().as_micros()).expect("bounded local duration")
}

type Completion = (usize, u64, u64, Outcome);

fn collect(
    result: Result<(Id, Completion), tokio::task::JoinError>,
    ids: &mut HashMap<Id, usize>,
    history: &mut [Observation],
    start: Instant,
) {
    match result {
        Ok((id, (index, invoked, completed, outcome))) => {
            assert_eq!(ids.remove(&id), Some(index));
            history[index].invoked_us = Some(invoked);
            history[index].completed_us = completed;
            history[index].outcome = outcome;
        }
        Err(error) => {
            let index = ids
                .remove(&error.id())
                .expect("every task has a prior intent");
            history[index].completed_us = micros(start);
            history[index].outcome = Outcome::Unknown;
        }
    }
}

async fn offer(
    start: Instant,
    schedule: &[Intent],
    pool: &PgPool,
    actor: UserId,
) -> (Vec<Observation>, usize, bool) {
    assert_eq!(schedule.len(), INTENT_CAP);
    let mut history: Vec<_> = schedule
        .iter()
        .map(|intent| Observation {
            intent: intent.clone(),
            admitted_us: None,
            invoked_us: None,
            completed_us: 0,
            outcome: Outcome::Pending,
        })
        .collect();
    let mut tasks = JoinSet::new();
    let mut ids = HashMap::new();
    let mut maximum_tasks = 0;
    for intent in schedule {
        // Absolute arrivals never wait for owner results or the DB witness.
        sleep_until(start + Duration::from_millis(intent.arrival_ms)).await;
        while let Some(result) = tasks.try_join_next_with_id() {
            collect(result, &mut ids, &mut history, start);
        }
        if tasks.len() == TASK_CAP {
            history[intent.id].outcome = Outcome::ClientRejected;
            history[intent.id].completed_us = micros(start);
            continue;
        }
        history[intent.id].admitted_us = Some(micros(start));
        let owned = intent.clone();
        let store = PgAttendanceStore::new(pool.clone());
        let handle = tasks.spawn(async move {
            let invoked = micros(start);
            let caller = branch_caller(actor, BranchId::from_uuid(owned.branch));
            let result = scope_org(
                OrgId::knl(),
                store.close_month(
                    &caller,
                    CloseMonth {
                        month: owned.month,
                        branch_scope: Some(owned.branch),
                        attest: true,
                    },
                ),
            )
            .await;
            let outcome = match result {
                Ok(row) => Outcome::Committed(row.id),
                Err(AttendanceStoreError::CloseBlocked | AttendanceStoreError::Conflict) => {
                    Outcome::Refused
                }
                Err(_) => Outcome::Unknown,
            };
            (owned.id, invoked, micros(start), outcome)
        });
        assert!(ids.insert(handle.id(), intent.id).is_none());
        maximum_tasks = maximum_tasks.max(tasks.len());
    }
    let drained = timeout(Duration::from_secs(5), async {
        while let Some(result) = tasks.join_next_with_id().await {
            collect(result, &mut ids, &mut history, start);
        }
    })
    .await
    .is_ok();
    if !drained {
        tasks.abort_all();
        while let Some(result) = tasks.join_next_with_id().await {
            collect(result, &mut ids, &mut history, start);
        }
    }
    assert!(ids.is_empty() && tasks.is_empty());
    (history, maximum_tasks, drained)
}

// Independent complete-table readbacks: no matching JOIN that hides orphans.
async fn readback(owner: &PgPool) -> (Vec<Effect>, Vec<Audit>) {
    let effects = sqlx::query_as(
        "SELECT id,org_id AS org,branch_id AS branch,to_char(month,'YYYY-MM') AS month,attested_by AS actor FROM attendance_month_closes",
    ).fetch_all(owner).await.unwrap();
    let audits = sqlx::query_as(
        "SELECT target_id AS target,org_id AS org,branch_id AS branch,actor,target_type FROM audit_events WHERE action='attendance.close.confirm'",
    ).fetch_all(owner).await.unwrap();
    (effects, audits)
}

fn reconcile(
    schedule: &[Intent],
    history: &[Observation],
    effects: &[Effect],
    audits: &[Audit],
    actor: Uuid,
    maximum_tasks: usize,
    drained: bool,
) -> Result<(), &'static str> {
    if !drained {
        return Err("unresolved resources");
    }
    if maximum_tasks > TASK_CAP || history.len() != INTENT_CAP || schedule.len() != INTENT_CAP {
        return Err("bounds or missing intent");
    }
    let mut expected = BTreeMap::new();
    for (index, (intent, observation)) in schedule.iter().zip(history).enumerate() {
        if intent.id != index || observation.intent != *intent {
            return Err("intent identity/order");
        }
        let intended_us = intent.arrival_ms * 1000;
        if observation.completed_us < intended_us {
            return Err("completion before arrival");
        }
        let disposition_us = observation.admitted_us.unwrap_or(observation.completed_us);
        if disposition_us < intended_us || disposition_us - intended_us > MAX_SCHEDULER_LAG_US {
            return Err("invalid arrival schedule");
        }
        if observation.completed_us - intended_us > MAX_INTENT_AGE_US {
            return Err("intent age bound");
        }
        match observation.outcome {
            Outcome::ClientRejected => {
                if observation.admitted_us.is_some() || observation.invoked_us.is_some() {
                    return Err("client rejection was invoked");
                }
            }
            Outcome::Committed(id) => {
                let (Some(admitted), Some(invoked)) =
                    (observation.admitted_us, observation.invoked_us)
                else {
                    return Err("missing invocation");
                };
                if admitted < intended_us
                    || invoked < admitted
                    || observation.completed_us < invoked
                {
                    return Err("latency ordering");
                }
                if invoked - intended_us > MAX_SCHEDULER_LAG_US {
                    return Err("invalid invocation schedule");
                }
                if expected.insert(id, intent).is_some() {
                    return Err("success counted twice");
                }
            }
            // Unique fixture keys have no predeclared semantic conflicts.
            // An unknown is inconclusive, never inferred to mean no effect.
            _ => return Err("unexpected refusal or unresolved outcome"),
        }
    }
    if effects.len() != expected.len() || audits.len() != expected.len() {
        return Err("effect/audit count");
    }
    let mut seen = BTreeSet::new();
    for effect in effects {
        if !seen.insert(effect.id) {
            return Err("duplicate effect");
        }
        let intent = expected.get(&effect.id).ok_or("unexpected effect")?;
        if effect.org != *OrgId::knl().as_uuid()
            || effect.actor != actor
            || effect.branch != intent.branch
            || effect.month != intent.month
        {
            return Err("effect attribution");
        }
        let matching: Vec<_> = audits
            .iter()
            .filter(|audit| audit.target == effect.id.to_string())
            .collect();
        if matching.len() != 1 {
            return Err("audit bijection");
        }
        let audit = matching[0];
        if audit.org != effect.org
            || audit.branch != Some(effect.branch)
            || audit.actor != Some(actor)
            || audit.target_type != "attendance_month_close"
        {
            return Err("audit attribution");
        }
    }
    Ok(())
}

async fn serving_pool(owner: &PgPool) -> PgPool {
    let pool = PgPoolOptions::new()
        .max_connections(POOL_CAP)
        .min_connections(POOL_CAP)
        .after_connect(|connection, _| {
            Box::pin(async move {
                sqlx::query("SELECT set_config('application_name','console-open-loop',false)")
                    .execute(connection)
                    .await?;
                Ok(())
            })
        })
        .connect(&login_test_database_url(owner, TestDatabaseLogin::Business))
        .await
        .unwrap();
    drain_pool(&pool).await;
    pool
}

async fn drain_pool(pool: &PgPool) {
    timeout(Duration::from_secs(5), async {
        // Hold all slots simultaneously; repeated single checkout could reuse
        // one healthy slot and miss an abandoned transaction in another.
        let mut slots = Vec::new();
        for _ in 0..POOL_CAP { slots.push(pool.acquire().await.unwrap()); }
        for slot in &mut slots {
            let identity: (String,String,bool) = sqlx::query_as(
                "SELECT session_user::text,current_user::text,rolsuper OR rolbypassrls OR rolcreaterole OR rolcreatedb OR rolreplication FROM pg_roles WHERE rolname=current_user",
            ).fetch_one(&mut **slot).await.unwrap();
            assert_eq!(identity, ("console_rt".to_owned(),"console_rt".to_owned(),false));
        }
    }).await.expect("all serving slots must be reusable");
}

async fn scenario(owner: PgPool, hot: bool) {
    let mut schedule = Vec::new();
    let actor_branch = seed_branch(&owner, "arrival-actor", "operations").await;
    let actor = seed_user(&owner, "Arrival fixture closer", "ADMIN", actor_branch).await;
    let arrivals = [0, 100, 400, 500, 600, 600, 600, 600, 1500, 1600, 1700, 1800];
    for (id, arrival_ms) in arrivals.into_iter().enumerate() {
        let branch = seed_branch(&owner, "arrival-intent", "operations").await;
        let month = if hot && id != 1 && id != 3 && id < 8 {
            "2026-07".to_owned()
        } else {
            format!("2025-{:02}", id + 1)
        };
        schedule.push(Intent {
            id,
            branch: *branch.as_uuid(),
            month,
            arrival_ms,
        });
    }
    let pool = serving_pool(&owner).await;
    let mut gate = if hot {
        Some(
            hold_exact_advisory_gate(
                &owner,
                &format!("attendance-close-v1|{}|2026-07-01", OrgId::knl().as_uuid()),
            )
            .await,
        )
    } else {
        None
    };
    let gate_session = match &mut gate {
        Some(gate) => Some(transaction_session(gate).await),
        None => None,
    };
    let start = Instant::now() + Duration::from_millis(100);
    let control = async {
        if let Some(gate_session) = gate_session {
            // This observer is separate from the independent arrival task.
            let observation = timeout(Duration::from_secs(2), async {
                let deadline = start + Duration::from_millis(900);
                let mut witnessed_hot = false;
                let mut witnessed_neighbor = false;
                let mut witnessed_saturation = false;
                while Instant::now() < deadline {
                    let waiting: i64 = sqlx::query_scalar(
                        "SELECT count(DISTINCT a.pid) FROM pg_stat_activity a JOIN pg_locks w ON w.pid=a.pid AND w.locktype='advisory' AND NOT w.granted JOIN pg_locks g ON g.pid=$1 AND g.locktype='advisory' AND g.granted AND g.database IS NOT DISTINCT FROM w.database AND g.classid=w.classid AND g.objid=w.objid AND g.objsubid=w.objsubid WHERE a.datname=current_database() AND a.application_name='console-open-loop' AND a.usename='console_rt' AND a.query LIKE '%pg_advisory_xact_lock%' AND $1=ANY(pg_blocking_pids(a.pid))",
                    ).bind(gate_session.backend_pid).fetch_one(&owner).await.unwrap();
                    witnessed_hot |= waiting >= 1;
                    witnessed_saturation |= waiting == i64::from(POOL_CAP);
                    let (effects,audits) = readback(&owner).await;
                    witnessed_neighbor |= effects.iter().any(|effect| effect.branch == schedule[1].branch
                        && audits.iter().any(|audit| audit.target == effect.id.to_string()));
                    if witnessed_hot && witnessed_neighbor && witnessed_saturation { break; }
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
                (witnessed_hot,witnessed_neighbor,witnessed_saturation)
            }).await;
            // Even an observation timeout releases the owned barrier before
            // the driver is joined and the invalid schedule is reported.
            sleep_until(start + Duration::from_millis(RELEASE_MS)).await;
            let released = timeout(Duration::from_secs(3), gate.take().unwrap().rollback()).await;
            let release_ok = matches!(released, Ok(Ok(())));
            let (one, neighbor, saturated) = observation.unwrap_or((false, false, false));
            (one, neighbor, saturated, micros(start), release_ok)
        } else {
            (false, false, false, 0, true)
        }
    };

    let ((history, maximum_tasks, drained), witness) =
        tokio::join!(offer(start, &schedule, &pool, actor), control);
    drain_pool(&pool).await;
    let observed = timeout(Duration::from_secs(3), async {
    let active: i64 = sqlx::query_scalar("SELECT count(*) FROM pg_stat_activity WHERE datname=current_database() AND application_name='console-open-loop' AND (state <> 'idle' OR xact_start IS NOT NULL OR wait_event_type='Lock')")
        .fetch_one(&owner).await.unwrap();
    let (effects, audits) = readback(&owner).await;
    (active,effects,audits)
    }).await;
    // Close all serving sessions even if the final observer timed out.
    timeout(Duration::from_secs(3), pool.close())
        .await
        .expect("bounded pool close; timeout is inconclusive cleanup");
    let (active, effects, audits) = observed.expect("bounded independent readback");
    assert!(witness.4, "gate rollback confirmed before acceptance");
    assert_eq!(
        active, 0,
        "all serving sessions quiescent before readback claim"
    );
    assert_eq!(
        reconcile(
            &schedule,
            &history,
            &effects,
            &audits,
            *actor.as_uuid(),
            maximum_tasks,
            drained
        ),
        Ok(())
    );
    if hot {
        assert_eq!(
            validate_hot_schedule(&history, maximum_tasks, witness),
            Ok(())
        );
        let mut late_invocation = history.clone();
        late_invocation[3].invoked_us = Some(witness.3 + 1);
        late_invocation[3].completed_us = witness.3 + 2;
        assert_eq!(
            validate_hot_schedule(&late_invocation, maximum_tasks, witness),
            Err("saturated pool neighbor witness")
        );
        let mut delayed = history.clone();
        for row in &mut delayed[2..8] {
            row.admitted_us = Some(witness.3 + 1);
            row.invoked_us = Some(witness.3 + 2);
            row.completed_us = witness.3 + 3;
        }
        assert_eq!(
            validate_hot_schedule(&delayed, maximum_tasks, witness),
            Err("hot offers missed gate")
        );
        assert_eq!(
            validate_hot_schedule(
                &history,
                maximum_tasks,
                (true, true, false, witness.3, true)
            ),
            Err("missing hot witness")
        );
    }
    assert!(
        history[8..]
            .iter()
            .all(|row| matches!(row.outcome, Outcome::Committed(_))),
        "actual further arrivals succeed after release"
    );
    // Each offered intent appears once, including generator rejection. No
    // retries, server rejection claims, percentile estimates or throughput SLA.
    let rows: Vec<_> = history
        .iter()
        .map(|row| {
            serde_json::json!({
                "id":row.intent.id,"branch":row.intent.branch,"month":row.intent.month,
                "intended_us":row.intent.arrival_ms*1000,"admitted_us":row.admitted_us,
                "invoked_us":row.invoked_us,"completed_us":row.completed_us,
                "outcome":format!("{:?}",row.outcome),
            })
        })
        .collect();
    let wire = serde_json::to_vec(&serde_json::json!({"hot":hot,"task_cap":TASK_CAP,"pool_cap":POOL_CAP,
        "generator_queue_count":0,"generator_queue_bytes":0,"maximum_tasks":maximum_tasks,"drained":drained,
        "sqlx_waiter_conservative_cap":TASK_CAP,"max_scheduler_lag_us":MAX_SCHEDULER_LAG_US,
        "max_intent_age_us":MAX_INTENT_AGE_US,"max_history_bytes":MAX_HISTORY_BYTES,
        "release_us":witness.3,"hot_witness":witness.0,"neighbor_witness":witness.1,"saturation_witness":witness.2,"rows":rows})).unwrap();
    assert!(
        wire.len() <= MAX_HISTORY_BYTES,
        "bounded history serialization"
    );
    println!("OPEN_LOOP {}", std::str::from_utf8(&wire).unwrap());
    challenge_oracle(
        &schedule,
        &history,
        &effects,
        &audits,
        *actor.as_uuid(),
        maximum_tasks,
    );
}

fn validate_hot_schedule(
    history: &[Observation],
    maximum_tasks: usize,
    witness: (bool, bool, bool, u64, bool),
) -> Result<(), &'static str> {
    if !witness.0 || !witness.1 || !witness.2 || !witness.4 {
        return Err("missing hot witness");
    }
    if witness.3 < RELEASE_MS * 1000 || witness.3 >= 1_500_000 {
        return Err("missed release deadline");
    }
    if history[..8]
        .iter()
        .any(|row| row.admitted_us.unwrap_or(row.completed_us) >= witness.3)
    {
        return Err("hot offers missed gate");
    }
    if maximum_tasks != TASK_CAP
        || history[4..8]
            .iter()
            .filter(|row| row.outcome == Outcome::ClientRejected)
            .count()
            != 3
    {
        return Err("missing prescribed overload");
    }
    if !matches!(history[3].outcome, Outcome::Committed(_))
        || history[3]
            .invoked_us
            .is_none_or(|invoked| invoked >= witness.3)
        || history[3].completed_us < witness.3
    {
        return Err("saturated pool neighbor witness");
    }
    Ok(())
}

fn challenge_oracle(
    schedule: &[Intent],
    history: &[Observation],
    effects: &[Effect],
    audits: &[Audit],
    actor: Uuid,
    maximum_tasks: usize,
) {
    let check = |h: &[Observation], e: &[Effect], a: &[Audit], cap, drain| {
        reconcile(schedule, h, e, a, actor, cap, drain)
    };
    assert!(check(history, effects, audits, maximum_tasks, true).is_ok());
    assert_eq!(
        check(&history[1..], effects, audits, maximum_tasks, true),
        Err("bounds or missing intent")
    );
    let mut h = history.to_vec();
    h.swap(0, 1);
    assert_eq!(
        check(&h, effects, audits, maximum_tasks, true),
        Err("intent identity/order")
    );
    let mut h = history.to_vec();
    h[0].intent.month = "2030-01".to_owned();
    assert_eq!(
        check(&h, effects, audits, maximum_tasks, true),
        Err("intent identity/order")
    );
    assert_eq!(
        check(history, &effects[1..], audits, maximum_tasks, true),
        Err("effect/audit count")
    );
    assert_eq!(
        check(history, effects, &audits[1..], maximum_tasks, true),
        Err("effect/audit count")
    );
    let mut e = effects.to_vec();
    e[0].org = *OrgId::new().as_uuid();
    assert_eq!(
        check(history, &e, audits, maximum_tasks, true),
        Err("effect attribution")
    );
    let mut e = effects.to_vec();
    e[0].branch = Uuid::new_v4();
    assert_eq!(
        check(history, &e, audits, maximum_tasks, true),
        Err("effect attribution")
    );
    let mut a = audits.to_vec();
    a[0].target = Uuid::new_v4().to_string();
    assert_eq!(
        check(history, effects, &a, maximum_tasks, true),
        Err("audit bijection")
    );
    let mut a = audits.to_vec();
    a[0].actor = Some(Uuid::new_v4());
    assert_eq!(
        check(history, effects, &a, maximum_tasks, true),
        Err("audit attribution")
    );
    let mut a = audits.to_vec();
    a[0].target_type = "foreign_type".to_owned();
    assert_eq!(
        check(history, effects, &a, maximum_tasks, true),
        Err("audit attribution")
    );
    let mut h = history.to_vec();
    h[0].outcome = Outcome::Unknown;
    assert_eq!(
        check(&h, effects, audits, maximum_tasks, true),
        Err("unexpected refusal or unresolved outcome")
    );
    let mut h = history.to_vec();
    h[8].completed_us = 0;
    assert_eq!(
        check(&h, effects, audits, maximum_tasks, true),
        Err("completion before arrival")
    );
    assert_eq!(
        check(history, effects, audits, TASK_CAP + 1, true),
        Err("bounds or missing intent")
    );
    assert_eq!(
        check(history, effects, audits, maximum_tasks, false),
        Err("unresolved resources")
    );
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn disjoint_absolute_arrivals_reconcile_each_intent_effect_and_audit(owner: PgPool) {
    timeout(Duration::from_secs(20), scenario(owner, false))
        .await
        .expect(
            "scenario deadline; timeout is NOT recovery proof; disposable runner must clean DB",
        );
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn hot_month_preserves_neighbor_and_fixed_recovery_arrivals(owner: PgPool) {
    timeout(Duration::from_secs(20), scenario(owner, true))
        .await
        .expect(
            "scenario deadline; timeout is NOT recovery proof; disposable runner must clean DB",
        );
}
