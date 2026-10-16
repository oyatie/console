//! Two actual SQL projection boundaries; no synthetic sleeper stands in for a DB wait.
use super::*;
use sqlx::Connection;

#[derive(Clone, Copy)]
enum Projection {
    Health,
    Adoption,
}
impl Projection {
    fn table(self) -> &'static str {
        match self {
            Self::Health => "public.groups",
            Self::Adoption => "public.console_route_telemetry",
        }
    }
    fn lock_sql(self) -> &'static str {
        match self {
            Self::Health => "LOCK TABLE public.groups IN ACCESS EXCLUSIVE MODE",
            Self::Adoption => "LOCK TABLE public.console_route_telemetry IN ACCESS EXCLUSIVE MODE",
        }
    }
    fn query(self) -> &'static str {
        match self {
            Self::Health => "platform_org_health",
            Self::Adoption => "platform_console_route_adoption",
        }
    }
}
fn ops_request(f: &Fixture, access: &str) -> JoinHandle<http::Response<Body>> {
    let router = f.router.clone();
    let access = access.to_owned();
    tokio::spawn(async move {
        router
            .oneshot(
                Request::builder()
                    .uri(OPS_PATH)
                    .header(header::AUTHORIZATION, format!("Bearer {access}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap()
    })
}
async fn permitted(pool: &PgPool, f: &Fixture, access: &str, expected: &Value) {
    let before = all_rows(pool).await;
    let start = db_now(pool).await;
    let response = finish(ops_request(f, access)).await.unwrap();
    let end = db_now(pool).await;
    let after = all_rows(pool).await;
    let (status, headers, body) = response_parts(response).await;
    let mut secrets = f.secrets.clone();
    secrets.push(access.to_owned());
    assert!(
        status == StatusCode::OK
            && health_matches(&serde_json::from_slice::<Value>(&body).unwrap(), expected),
        "complete actual ops positive/recovery"
    );
    assert!(
        private_response(&headers, &body, &secrets),
        "ops positive/recovery privacy"
    );
    assert!(
        health_read_delta(
            &before,
            &after,
            f.actor,
            expected["tenants"].as_array().unwrap().len(),
            start,
            end
        ),
        "one exact health audit and no other durable effects"
    );
}

// These test-only owner-login probes do not authorize business access or mutate rows.
// Each asks for an incompatible lock on one exact retained source identity and rolls back.
async fn source_probe(pool: &PgPool, sql: &'static str, id: Uuid) -> (i32, JoinHandle<Uuid>) {
    let mut tx = pool.begin().await.unwrap();
    let pid = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    let task = tokio::spawn(async move {
        let actual = sqlx::query_scalar(sql)
            .bind(id)
            .fetch_one(&mut *tx)
            .await
            .unwrap();
        tx.rollback().await.unwrap();
        actual
    });
    (pid, task)
}
async fn probe_wait(observer: &mut sqlx::PgConnection, probe: i32, reader: i32, sql: &str) -> bool {
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(3);
    loop {
        let waiting:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_stat_activity WHERE datname=current_database() AND pid=$1 AND wait_event_type='Lock' AND $2=ANY(pg_catalog.pg_blocking_pids(pid)) AND query=$3)")
            .bind(probe).bind(reader).bind(sql).fetch_one(&mut *observer).await.unwrap();
        if waiting || tokio::time::Instant::now() >= deadline {
            return waiting;
        }
        tokio::time::sleep(std::time::Duration::from_millis(15)).await;
    }
}
async fn projection_wait_exact(
    observer: &mut sqlx::PgConnection,
    reader: i32,
    blocker: i32,
    projection: Projection,
) -> bool {
    let exact:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_stat_activity a JOIN pg_catalog.pg_locks l ON l.pid=a.pid WHERE a.datname=current_database() AND a.pid=$1 AND a.usename='console_rt' AND a.wait_event_type='Lock' AND $2=ANY(pg_catalog.pg_blocking_pids(a.pid)) AND strpos(a.query,$3)>0 AND l.relation=$4::regclass AND l.mode='AccessShareLock' AND NOT l.granted)")
        .bind(reader).bind(blocker).bind(projection.query()).bind(projection.table()).fetch_one(&mut *observer).await.unwrap();
    let health_dependencies:bool=sqlx::query_scalar("SELECT count(DISTINCT relation)=2 FROM pg_catalog.pg_locks WHERE pid=$1 AND relation=ANY(ARRAY['public.groups'::regclass,'public.work_orders'::regclass]) AND mode='AccessShareLock' AND granted")
        .bind(reader).fetch_one(&mut *observer).await.unwrap();
    exact && (matches!(projection, Projection::Health) || health_dependencies)
}
async fn reached_db_deadline(observer: &mut sqlx::PgConnection, at: OffsetDateTime) -> bool {
    let budget = tokio::time::Instant::now() + std::time::Duration::from_secs(20);
    loop {
        let now: OffsetDateTime = sqlx::query_scalar("SELECT pg_catalog.clock_timestamp()")
            .fetch_one(&mut *observer)
            .await
            .unwrap();
        if now >= at {
            return true;
        }
        if tokio::time::Instant::now() >= budget {
            return false;
        }
        tokio::time::sleep(std::time::Duration::from_millis(15)).await;
    }
}

fn retained_history(
    blocker: i32,
    reader: Option<i32>,
    probes: &[i32],
    waits: &[bool],
    pending: bool,
    exact_projection: bool,
    clean: bool,
) -> bool {
    let Some(reader) = reader else { return false };
    let identities: BTreeSet<_> = std::iter::once(blocker)
        .chain(std::iter::once(reader))
        .chain(probes.iter().copied())
        .collect();
    blocker > 0
        && reader > 0
        && probes.len() == 3
        && probes.iter().all(|pid| *pid > 0)
        && identities.len() == 5
        && waits == [true, true, true]
        && pending
        && exact_projection
        && clean
}

async fn projection_history(
    pool: &PgPool,
    f: &Fixture,
    expected: &Value,
    projection: Projection,
    expire: bool,
) {
    // Genuine original login claims, signed by the same fixture key. Only exp is
    // narrowed in the expiry branch; no row backdating or production mint change.
    let (access, deadline) = if expire {
        let mut claims =
            serde_json::to_value(f.verifier.verify_access_token(&f.access).unwrap()).unwrap();
        let expires = db_now(pool).await.unix_timestamp() + 15;
        claims["exp"] = json!(expires);
        let access = jsonwebtoken::encode(
            &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::ES256),
            &claims,
            &jsonwebtoken::EncodingKey::from_ec_pem(
                f.key.to_pkcs8_pem(LineEnding::LF).unwrap().as_bytes(),
            )
            .unwrap(),
        )
        .unwrap();
        assert!(
            f.verifier.verify_access_token(&access).is_ok(),
            "actual signed short-lived credential prerequisite"
        );
        (
            access,
            Some(OffsetDateTime::from_unix_timestamp(expires).unwrap()),
        )
    } else {
        (f.access.clone(), None)
    };
    permitted(pool, f, &access, expected).await;
    let before = all_rows(pool).await;
    let start = db_now(pool).await;
    // Dedicated observer acquired first; never recycle an owned backend PID into observer.
    let mut observer = sqlx::PgConnection::connect_with(&pool.connect_options())
        .await
        .unwrap();
    let mut barrier = pool.begin().await.unwrap();
    let blocker: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *barrier)
        .await
        .unwrap();
    sqlx::query(projection.lock_sql())
        .execute(&mut *barrier)
        .await
        .unwrap();
    // A distinct downstream barrier outlives the expired response. A late-only
    // expiry check would hang here; final401/allstate alone cannot prove check placement.
    let (downstream_table, downstream_mode, downstream_sql) = match projection {
        Projection::Health => (
            "public.console_route_telemetry",
            "AccessExclusiveLock",
            "LOCK TABLE public.console_route_telemetry IN ACCESS EXCLUSIVE MODE",
        ),
        Projection::Adoption => (
            "public.audit_events",
            "ShareLock",
            "LOCK TABLE public.audit_events IN SHARE MODE",
        ),
    };
    let mut downstream = if expire {
        let mut tx = pool.begin().await.unwrap();
        sqlx::query(downstream_sql).execute(&mut *tx).await.unwrap();
        Some(tx)
    } else {
        None
    };
    let downstream_pid: Option<i32> = match downstream.as_mut() {
        Some(tx) => Some(
            sqlx::query_scalar("SELECT pg_backend_pid()")
                .fetch_one(&mut **tx)
                .await
                .unwrap(),
        ),
        None => None,
    };
    let reader = ops_request(f, &access);
    let reader_pid = waiting_on(
        pool,
        blocker,
        projection.query(),
        std::time::Duration::from_secs(3),
    )
    .await;
    const COMPANY: &str = "SELECT id FROM public.organizations WHERE id=$1 FOR UPDATE";
    const USER: &str = "SELECT id FROM public.users WHERE id=$1 FOR UPDATE";
    const ACCOUNT: &str = "SELECT id FROM public.accounts WHERE id=$1 FOR UPDATE";
    let (company_pid, company_probe) =
        source_probe(pool, COMPANY, *OrgId::platform().as_uuid()).await;
    let (user_pid, user_probe) = source_probe(pool, USER, *f.actor.as_uuid()).await;
    let (account_pid, account_probe) = source_probe(pool, ACCOUNT, *f.actor.as_uuid()).await;
    let mut waits = Vec::new();
    let mut exact_projection = false;
    if let Some(pid) = reader_pid {
        for (probe, sql) in [
            (company_pid, COMPANY),
            (user_pid, USER),
            (account_pid, ACCOUNT),
        ] {
            waits.push(probe_wait(&mut observer, probe, pid, sql).await);
        }
        exact_projection = projection_wait_exact(&mut observer, pid, blocker, projection).await;
    }
    let pending = !reader.is_finished()
        && !company_probe.is_finished()
        && !user_probe.is_finished()
        && !account_probe.is_finished();
    let observed_before_expiry = match deadline {
        Some(deadline) => {
            let now: OffsetDateTime = sqlx::query_scalar("SELECT pg_catalog.clock_timestamp()")
                .fetch_one(&mut observer)
                .await
                .unwrap();
            now < deadline
        }
        None => true,
    };
    let expired = match deadline {
        Some(deadline) => reached_db_deadline(&mut observer, deadline).await,
        None => false,
    };
    // Reobserve the same named SQL lock and every source guard immediately before release.
    // A poll loop or token timer alone is never evidence that source ownership persisted.
    let mut retained_at_release = false;
    if let Some(pid) = reader_pid {
        retained_at_release = projection_wait_exact(&mut observer, pid, blocker, projection).await;
        for (probe, sql) in [
            (company_pid, COMPANY),
            (user_pid, USER),
            (account_pid, ACCOUNT),
        ] {
            retained_at_release &= probe_wait(&mut observer, probe, pid, sql).await;
        }
    }
    barrier.rollback().await.unwrap();
    let (response, company, user, account) = tokio::join!(
        finish(reader),
        finish(company_probe),
        finish(user_probe),
        finish(account_probe)
    );
    let completed_before_downstream_release = response.is_ok();
    let downstream_still_held = match downstream.as_mut() {
        Some(tx) => sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_locks WHERE pid=pg_backend_pid() AND relation=$1::regclass AND mode=$2 AND granted)")
            .bind(downstream_table).bind(downstream_mode).fetch_one(&mut **tx).await.unwrap(),
        None => false,
    };
    if let Some(tx) = downstream {
        tx.rollback().await.unwrap();
    }
    let owned: Vec<_> = std::iter::once(blocker)
        .chain(reader_pid)
        .chain(downstream_pid)
        .chain([company_pid, user_pid, account_pid])
        .collect();
    let clean = clean_pids(&mut observer, &owned).await;
    let end = db_now(pool).await;
    let after = all_rows(pool).await;
    // Recovery is exercised before interpreting the interrupted history.
    permitted(pool, f, &f.access, expected).await;
    assert!(
        company.unwrap() == *OrgId::platform().as_uuid()
            && user.unwrap() == *f.actor.as_uuid()
            && account.unwrap() == *f.actor.as_uuid(),
        "same exact locked source identities resumed"
    );
    assert!(
        retained_history(
            blocker,
            reader_pid,
            &[company_pid, user_pid, account_pid],
            &waits,
            pending,
            exact_projection,
            clean
        ) && retained_at_release,
        "observed named projection and all three retained source locks through release and clean rollback/commit"
    );
    assert!(
        observed_before_expiry,
        "actual source wait established before signed expiry"
    );
    if expire {
        assert!(
            completed_before_downstream_release && downstream_still_held,
            "expired stage must return before blocked adoption query or audit INSERT can run"
        );
    }
    let (status, headers, body) = response_parts(response.unwrap()).await;
    let mut secrets = f.secrets.clone();
    secrets.push(access);
    assert!(
        private_response(&headers, &body, &secrets),
        "finite private ops response"
    );
    let body: Value = serde_json::from_slice(&body).unwrap();
    if expire {
        assert!(
            expired
                && status == StatusCode::UNAUTHORIZED
                && body
                    == json!({"error":{"code":"unauthorized","message":"invalid bearer token"}}),
            "post-projection signed expiry discards complete health/adoption output"
        );
        assert!(
            before == after,
            "expired projection has no audit or other durable effect"
        );
    } else {
        assert!(
            status == StatusCode::OK && health_matches(&body, expected),
            "resumed complete14/6field projection"
        );
        assert!(
            health_read_delta(
                &before,
                &after,
                f.actor,
                expected["tenants"].as_array().unwrap().len(),
                start,
                end
            ),
            "resumed exactly one health audit and no other state effect"
        );
    }
}

async fn two_outcomes(pool: PgPool, projection: Projection) {
    let f = fixture(&pool).await;
    let _ = seed_ops_content(&pool).await;
    let expected = expected_health(&pool).await;
    projection_history(&pool, &f, &expected, projection, false).await;
    projection_history(&pool, &f, &expected, projection, true).await;
    f.state.shutdown_realtime().await;
    f.business.close().await;
}
#[sqlx::test(migrations = false)]
async fn health_query_wait_retains_all_source_guards_and_expiry_discards_projection(pool: PgPool) {
    two_outcomes(pool, Projection::Health).await;
}
#[sqlx::test(migrations = false)]
async fn adoption_query_wait_retains_all_source_guards_and_expiry_discards_projection(
    pool: PgPool,
) {
    two_outcomes(pool, Projection::Adoption).await;
}
#[test]
fn projection_wait_history_oracle_rejects_missing_duplicate_and_unobserved_guards() {
    assert!(retained_history(
        1,
        Some(2),
        &[3, 4, 5],
        &[true, true, true],
        true,
        true,
        true
    ));
    for reader in [None, Some(0), Some(1), Some(3)] {
        assert!(!retained_history(
            1,
            reader,
            &[3, 4, 5],
            &[true, true, true],
            true,
            true,
            true
        ));
    }
    for probes in [
        &[3, 4][..],
        &[3, 4, 4][..],
        &[3, 4, 0][..],
        &[3, 4, 2][..],
        &[3, 4, 5, 6][..],
    ] {
        assert!(!retained_history(
            1,
            Some(2),
            probes,
            &[true, true, true],
            true,
            true,
            true
        ));
    }
    for waits in [
        &[true, true][..],
        &[false, true, true][..],
        &[true, false, true][..],
        &[true, true, false][..],
    ] {
        assert!(!retained_history(
            1,
            Some(2),
            &[3, 4, 5],
            waits,
            true,
            true,
            true
        ));
    }
    for (pending, projection, clean) in [
        (false, true, true),
        (true, false, true),
        (true, true, false),
    ] {
        assert!(!retained_history(
            1,
            Some(2),
            &[3, 4, 5],
            &[true, true, true],
            pending,
            projection,
            clean
        ));
    }
}
