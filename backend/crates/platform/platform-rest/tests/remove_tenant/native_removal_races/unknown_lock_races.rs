//! Genuine relation-lock failures must stay unknown; no owner body is replaced.

use super::*;
use std::time::Duration as Wait;

async fn bounded_pool(f: &Fixture, force: bool) -> (PgPool, Vec<i32>) {
    let pool = f.removal_pool(force);
    let role = if force {
        "console_platform_force_cmd"
    } else {
        "console_rt"
    };
    // login_test_pool creates max4 sessions. Own all four before releasing any,
    // so an HTTP request cannot borrow an unconfigured session from this pool.
    let mut sessions = Vec::new();
    let mut pids = Vec::new();
    for _ in 0..4 {
        let mut session = pool.acquire().await.unwrap();
        sqlx::raw_sql("SET lock_timeout='3s'; SET statement_timeout='15s'; SET enable_seqscan=off")
            .execute(session.as_mut())
            .await
            .unwrap();
        let actual: (String, String, i32, String, String, String) = sqlx::query_as("SELECT session_user::text,current_user::text,pg_backend_pid(),current_setting('lock_timeout'),current_setting('statement_timeout'),current_setting('enable_seqscan')")
            .fetch_one(session.as_mut()).await.unwrap();
        assert_eq!(actual.0, role);
        assert_eq!(actual.1, role);
        assert_eq!(
            (actual.3.as_str(), actual.4.as_str(), actual.5.as_str()),
            ("3s", "15s", "off")
        );
        assert!(!pids.contains(&actual.2));
        pids.push(actual.2);
        sessions.push(session);
    }
    drop(sessions);
    (pool, pids)
}

enum Observation {
    Sql(Result<String, sqlx::Error>),
    Http(Reply),
}

async fn relation_wait(
    f: &Fixture,
    tasks: &mut JoinSet<Observation>,
    blocker: i32,
    relations: &[i64],
    pids: &[i32],
) -> (i32, i64) {
    tokio::time::timeout(Wait::from_secs(12), async {
        loop {
            let origin: Option<(i32,i64)> = sqlx::query_as("SELECT a.pid,w.relation::bigint FROM pg_catalog.pg_stat_activity a JOIN pg_catalog.pg_locks w ON w.pid=a.pid AND w.locktype='relation' AND NOT w.granted JOIN pg_catalog.pg_locks b ON b.pid=$1 AND b.locktype=w.locktype AND b.database=w.database AND b.relation=w.relation AND b.granted WHERE a.datname=current_database() AND a.state='active' AND a.pid=ANY($3) AND w.relation::bigint=ANY($2) AND $1=ANY(pg_catalog.pg_blocking_pids(a.pid))")
                .bind(blocker).bind(relations).bind(pids).fetch_optional(&f.owner).await.unwrap();
            if let Some(origin) = origin { return origin; }
            if let Some(done) = tasks.try_join_next() {
                done.unwrap();
                panic!("expected actual lock wait on {relations:?}; command completed first");
            }
            tokio::time::sleep(Wait::from_millis(10)).await;
        }
    }).await.expect("exact PostgreSQL relation origin is required, not merely the blocker PID")
}

async fn completed(tasks: &mut JoinSet<Observation>) -> Observation {
    tokio::time::timeout(Wait::from_secs(12), tasks.join_next())
        .await
        .expect("real lock timeout must finish while the blocker stays held")
        .expect("expected an owned operation")
        .expect("owned command task failed")
}

async fn direct(f: &Fixture, pool: &PgPool, force: bool) -> Result<String, sqlx::Error> {
    if force {
        let trace = console_kernel_core::TraceContext::generate();
        sqlx::query_scalar(
            "SELECT public.platform_force_remove_organization_command($1,$2,$3,$4,$5)",
        )
        .bind(f.target)
        .bind(f.actor.as_uuid())
        .bind(trace.trace_id())
        .bind(trace.span_id())
        .bind(OffsetDateTime::now_utc())
        .fetch_one(pool)
        .await
    } else {
        sqlx::query_scalar("SELECT public.platform_remove_organization($1)")
            .bind(f.target)
            .fetch_one(pool)
            .await
    }
}

#[sqlx::test(migrations = false)]
async fn actual_table_and_reindex_wait_errors_remain_generic_and_preserve_every_row(owner: PgPool) {
    console_platform_test_support::prepare_account_test_database(&owner).await;
    require_removal_keys(&owner).await;
    // Exercise the membership admission itself: catching relation/index55P03
    // around a SKIP LOCKED tuple claim would misclassify these unknown failures.
    for (force, index, tag) in [
        (false, false, "table-normal"),
        (true, false, "table-force"),
        (false, true, "index-normal"),
        (true, true, "index-force"),
    ] {
        let fixture = Fixture::new(&owner, tag).await;
        let worker = Arc::clone(&fixture);
        finish(fixture, tokio::spawn(async move {
            let (pool, pids) = bounded_pool(&worker, force).await;
            let before = census(&worker.owner).await;
            let mut blocker = worker.owner.begin().await.unwrap();
            let blocker_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
                .fetch_one(blocker.as_mut()).await.unwrap();
            let (statement, relations) = if index {
                // Lock all actual indexes so a correct plan may choose either
                // unique org_id or the composite primary key. Heap waits fail.
                let indexes: Vec<i64> = sqlx::query_scalar("SELECT indexrelid::bigint FROM pg_catalog.pg_index WHERE indrelid='public.group_memberships'::regclass ORDER BY indexrelid")
                    .fetch_all(&worker.owner).await.unwrap();
                assert!(!indexes.is_empty());
                ("REINDEX TABLE public.group_memberships", indexes)
            } else {
                let heap: i64 = sqlx::query_scalar("SELECT 'public.group_memberships'::regclass::oid::bigint")
                    .fetch_one(&worker.owner).await.unwrap();
                ("LOCK TABLE public.group_memberships IN ACCESS EXCLUSIVE MODE", vec![heap])
            };
            sqlx::raw_sql(sqlx::AssertSqlSafe(statement)).execute(blocker.as_mut()).await.unwrap();
            let mut tasks = JoinSet::new();
            let target = Arc::clone(&worker);
            tasks.spawn(async move { Observation::Sql(direct(&target, &pool, force).await) });
            let (_, sql_origin) = relation_wait(&worker, &mut tasks, blocker_pid, &relations, &pids).await;
            let Observation::Sql(result) = completed(&mut tasks).await else { panic!("expected SQL positive control") };
            let error = result.expect_err("actual blocked SQL must not succeed");
            assert_eq!(error.as_database_error().unwrap().code().as_deref(), Some("55P03"));
            let target = Arc::clone(&worker);
            tasks.spawn(async move { Observation::Http(request(&target, Kind::Remove(force), target.target).await) });
            let (_, http_origin) = relation_wait(&worker, &mut tasks, blocker_pid, &relations, &pids).await;
            let Observation::Http(reply) = completed(&mut tasks).await else { panic!("expected actual HTTP command") };
            expect(&reply, StatusCode::INTERNAL_SERVER_ERROR, Some("internal"));
            assert_eq!(reply.body, serde_json::json!({"error":{"code":"internal","message":"internal server error"}}));
            assert!(tasks.is_empty());
            for origin in [sql_origin, http_origin] {
                let held: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_locks WHERE pid=$1 AND locktype='relation' AND relation::bigint=$2 AND granted AND mode='AccessExclusiveLock')")
                    .bind(blocker_pid).bind(origin).fetch_one(&worker.owner).await.unwrap();
                assert!(held, "same exact relation blocker must remain held through both outcomes");
            }
            blocker.rollback().await.unwrap();
            assert_eq!(census(&worker.owner).await, before);
        })).await;
    }
}
