//! Child of extended. Actual PostgreSQL blocker edges are the lock evidence.
use super::*;
use console_platform_auth::guard_legacy_subject_in_tx;
use tokio::task::JoinHandle;

async fn waiting_on(pool: &PgPool, blocker: i32, query_fragment: &str) -> Option<i32> {
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(8);
    loop {
        let pid:Option<i32>=sqlx::query_scalar("SELECT pid FROM pg_catalog.pg_stat_activity WHERE datname=current_database() AND usename='console_auth_rt' AND wait_event_type='Lock' AND $1=ANY(pg_catalog.pg_blocking_pids(pid)) AND strpos(query,$2)>0 ORDER BY pid LIMIT 1")
            .bind(blocker).bind(query_fragment).fetch_optional(pool).await.unwrap();
        if pid.is_some() || tokio::time::Instant::now() >= deadline {
            return pid;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
}
async fn elapsed_database_deadline(pool: &PgPool, deadline: OffsetDateTime) -> bool {
    tokio::time::timeout(std::time::Duration::from_secs(12), async {
        loop {
            if db_now(pool).await >= deadline {
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
    })
    .await
    .is_ok()
}
async fn finish<T>(mut task: JoinHandle<T>) -> Result<T, &'static str> {
    match tokio::time::timeout(std::time::Duration::from_secs(10), &mut task).await {
        Ok(result) => result.map_err(|_| "owned actor panicked"),
        Err(_) => {
            task.abort();
            let _ = task.await;
            Err("owned actor did not finish after blocker release")
        }
    }
}

fn read_task(router: &axum::Router, token: &str) -> JoinHandle<http::Response<Body>> {
    let router = router.clone();
    let token = token.to_owned();
    tokio::spawn(async move { get_legacy_raw(&router, PATH, &token).await })
}
async fn no_remaining_waiters(pool: &PgPool, pids: &[i32]) {
    let waiting:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_stat_activity WHERE datname=current_database() AND ((pid=ANY($1) AND state LIKE 'idle in transaction%') OR ((pid=ANY($1) OR pg_catalog.pg_blocking_pids(pid) && $1) AND wait_event_type='Lock')))")
        .bind(pids).fetch_one(pool).await.unwrap();
    assert!(!waiting, "owned history left a waiting backend");
}
fn json_rows(rows: &Rows, table: &str) -> Vec<Value> {
    serde_json::from_str(&rows[table]).unwrap()
}
fn exact_logout_effect(
    before: &Rows,
    after: &Rows,
    f: &Fixture,
    now: OffsetDateTime,
    completed: OffsetDateTime,
) {
    unchanged_except(
        before,
        after,
        &[
            "auth_refresh_token_families",
            "auth_refresh_tokens",
            "audit_events",
        ],
    );
    for (table, target) in [
        ("auth_refresh_token_families", f.a.family_id),
        ("auth_refresh_tokens", f.a.token_id),
    ] {
        let old = json_rows(before, table);
        let new = json_rows(after, table);
        assert_eq!(old.len(), new.len());
        for row in old {
            let actual = new.iter().find(|r| r["id"] == row["id"]).unwrap();
            if row["id"] != json!(target) {
                assert!(&row == actual, "unrelated logout row changed");
                continue;
            }
            assert!(row["revoked_at"].is_null());
            let at = OffsetDateTime::parse(
                actual["revoked_at"].as_str().unwrap(),
                &time::format_description::well_known::Rfc3339,
            )
            .unwrap();
            assert!(at == now);
            let mut expected = row.clone();
            expected["revoked_at"] = actual["revoked_at"].clone();
            if table == "auth_refresh_token_families" {
                assert!(row["revoked_reason"].is_null());
                expected["revoked_reason"] = json!("logout");
            }
            assert!(&expected == actual, "unexpected logout mutation");
        }
    }
    let old = json_rows(before, "audit_events");
    let new = json_rows(after, "audit_events");
    assert_eq!(new.len(), old.len() + 1);
    assert!(old.iter().all(|r| new.contains(r)));
    let added: Vec<_> = new.iter().filter(|r| !old.contains(r)).collect();
    assert_eq!(added.len(), 1);
    assert!(logout_audit_matches(
        added[0],
        f.legacy.subject,
        f.a.family_id,
        now,
        completed
    ));
}

#[sqlx::test(migrations = false)]
async fn reader_first_retains_subject_and_exact_family_guards_through_projection(pool: PgPool) {
    let f = fixture(&pool).await;
    let before = all_rows(&pool).await;
    let mut projection = pool.begin().await.unwrap();
    let blocker: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *projection)
        .await
        .unwrap();
    sqlx::query("LOCK TABLE public.auth_webauthn_credentials IN ACCESS EXCLUSIVE MODE")
        .execute(&mut *projection)
        .await
        .unwrap();
    let reader = read_task(&f.legacy.router, &f.access_a);
    let reader_pid = waiting_on(&pool, blocker, "auth_legacy_self_passkeys_v1").await;
    // The family lock probe only observes retained SHARE exclusion. It does not
    // revoke credentials or impersonate a new production writer.
    let auth = f.auth.clone();
    let family = f.a.family_id;
    let probe = tokio::spawn(async move {
        let mut tx = auth.begin().await.unwrap();
        sqlx::query("SELECT set_config('app.current_org',$1,true)")
            .bind(OrgId::knl().to_string())
            .execute(&mut *tx)
            .await
            .unwrap();
        let id: Uuid = sqlx::query_scalar(
            "SELECT id FROM public.auth_refresh_token_families WHERE id=$1 FOR UPDATE",
        )
        .bind(family)
        .fetch_one(&mut *tx)
        .await
        .unwrap();
        tx.rollback().await.unwrap();
        id
    });
    let family_wait = if let Some(pid) = reader_pid {
        waiting_on(&pool, pid, "auth_refresh_token_families").await
    } else {
        None
    };
    let now = db_now(&f.auth).await;
    let auth = f.auth.clone();
    let token = f.a.token.as_str().to_owned();
    let revoker = tokio::spawn(async move {
        RefreshTokenStore
            .revoke_family_for_logout(&auth, &token, now)
            .await
    });
    let revoke_wait = if let Some(pid) = reader_pid {
        waiting_on(&pool, pid, "account_company_deactivation_guard_v1").await
    } else {
        None
    };
    // Always release the global fixture blocker before asserting observed edges.
    projection.rollback().await.unwrap();
    let (read, probed, revoked) = tokio::join!(finish(reader), finish(probe), finish(revoker));
    no_remaining_waiters(
        &pool,
        &[
            blocker,
            reader_pid.unwrap_or(-1),
            family_wait.unwrap_or(-1),
            revoke_wait.unwrap_or(-1),
        ],
    )
    .await;
    assert!(
        reader_pid.is_some(),
        "actual projection wait must be observed"
    );
    assert!(
        family_wait.is_some(),
        "reader must retain exact-family SHARE through projection"
    );
    assert!(
        revoke_wait.is_some(),
        "actual logout must wait behind reader's current-subject guard"
    );
    assert!(probed.expect("family observation actor") == f.a.family_id);
    assert!(revoked.expect("logout actor").is_ok());
    let body = response_json(
        read.expect("reader actor"),
        StatusCode::OK,
        &f.secrets(),
        true,
    )
    .await;
    assert!(body == f.expected);
    let after = all_rows(&pool).await;
    exact_logout_effect(&before, &after, &f, now, db_now(&pool).await);
    refused(&f, &f.legacy.router, &f.access_a).await;
    accepted(&f, &f.legacy.router, &f.access_b, &f.expected).await;
    assert!(after == all_rows(&pool).await);
    f.auth.close().await;
}

#[sqlx::test(migrations = false)]
async fn revoker_first_commits_before_waiting_reader_rechecks_exact_family(pool: PgPool) {
    let f = fixture(&pool).await;
    let before = all_rows(&pool).await;
    let now = db_now(&f.auth).await;
    let mut revoker = f.auth.begin().await.unwrap();
    let blocker: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *revoker)
        .await
        .unwrap();
    RefreshTokenStore
        .revoke_family_for_logout_in_tx(&mut revoker, f.a.token.as_str(), now)
        .await
        .unwrap();
    let reader = read_task(&f.legacy.router, &f.access_a);
    let reader_pid = waiting_on(&pool, blocker, "account_company_deactivation_guard_v1").await;
    revoker.commit().await.unwrap();
    let read = finish(reader).await;
    no_remaining_waiters(&pool, &[blocker, reader_pid.unwrap_or(-1)]).await;
    assert!(
        reader_pid.is_some(),
        "actual reader must wait behind actual uncommitted logout"
    );
    let body = response_json(
        read.expect("reader actor"),
        StatusCode::UNAUTHORIZED,
        &f.secrets(),
        true,
    )
    .await;
    assert!(body == error_body("unauthorized", "invalid bearer token"));
    let after = all_rows(&pool).await;
    exact_logout_effect(&before, &after, &f, now, db_now(&pool).await);
    accepted(&f, &f.legacy.router, &f.access_b, &f.expected).await;
    assert!(after == all_rows(&pool).await);
    f.auth.close().await;
}

async fn expires_during_wait(pool: &PgPool, projection_wait: bool, bound: bool) {
    let f = fixture(pool).await;
    let mut changed = claims(&f, if bound { &f.access_a } else { &f.legacy.access });
    let now = db_now(&f.auth).await;
    let expiry = now.unix_timestamp() + 6;
    changed["exp"] = json!(expiry);
    let token = sign(&f, &changed);
    let mut secrets = f.secrets();
    secrets.push(token.clone());
    accepted(&f, &f.legacy.router, &token, &f.expected).await;
    let before = all_rows(pool).await;
    let mut blocker = if projection_wait {
        pool.begin().await.unwrap()
    } else {
        f.auth.begin().await.unwrap()
    };
    let pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *blocker)
        .await
        .unwrap();
    if projection_wait {
        sqlx::query("LOCK TABLE public.auth_webauthn_credentials IN ACCESS EXCLUSIVE MODE")
            .execute(&mut *blocker)
            .await
            .unwrap();
    } else {
        guard_legacy_subject_in_tx(&mut blocker, OrgId::knl(), *f.legacy.subject.as_uuid())
            .await
            .unwrap();
    }
    let reader = read_task(&f.legacy.router, &token);
    let observed = waiting_on(
        pool,
        pid,
        if projection_wait {
            "auth_legacy_self_passkeys_v1"
        } else {
            "account_company_deactivation_guard_v1"
        },
    )
    .await;
    let observed_before = if observed.is_some() {
        Some(db_now(pool).await)
    } else {
        None
    };
    let elapsed =
        elapsed_database_deadline(pool, OffsetDateTime::from_unix_timestamp(expiry).unwrap()).await;
    blocker.rollback().await.unwrap();
    let read = finish(reader).await;
    no_remaining_waiters(pool, &[pid, observed.unwrap_or(-1)]).await;
    assert!(observed.is_some(), "real owner wait must precede expiry");
    assert!(
        observed_before.is_some_and(|at| at < OffsetDateTime::from_unix_timestamp(expiry).unwrap()),
        "observed owner wait must occur before signed expiry"
    );
    assert!(elapsed, "actual DB expiry passed");
    let body = response_json(
        read.expect("reader actor"),
        StatusCode::UNAUTHORIZED,
        &secrets,
        true,
    )
    .await;
    assert!(body == error_body("unauthorized", "invalid bearer token"));
    accepted(&f, &f.legacy.router, &f.access_b, &f.expected).await;
    assert!(before == all_rows(pool).await);
    f.auth.close().await;
}

#[sqlx::test(migrations = false)]
async fn bound_expiry_is_rechecked_after_observed_subject_wait(pool: PgPool) {
    expires_during_wait(&pool, false, true).await;
}
#[sqlx::test(migrations = false)]
async fn bound_expiry_is_rechecked_after_observed_projection_wait(pool: PgPool) {
    expires_during_wait(&pool, true, true).await;
}
#[sqlx::test(migrations = false)]
async fn historical_expiry_is_rechecked_after_observed_projection_wait(pool: PgPool) {
    expires_during_wait(&pool, true, false).await;
}

#[sqlx::test(migrations = false)]
async fn family_deadline_is_rechecked_after_observed_projection_wait(pool: PgPool) {
    let f = fixture(&pool).await;
    let narrow = family_ttl_router(&pool, &f, 8).await;
    let (family, created) = issue(&pool, &f.auth, f.legacy.subject).await;
    let token = bound_access(&f.legacy, &family, created);
    let mut secrets = f.secrets();
    secrets.extend([
        token.clone(),
        family.family_id.to_string(),
        family.token.as_str().to_owned(),
    ]);
    accepted(&f, &narrow, &token, &f.expected).await;
    let before = all_rows(&pool).await;
    let mut projection = pool.begin().await.unwrap();
    let pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *projection)
        .await
        .unwrap();
    sqlx::query("LOCK TABLE public.auth_webauthn_credentials IN ACCESS EXCLUSIVE MODE")
        .execute(&mut *projection)
        .await
        .unwrap();
    let reader = read_task(&narrow, &token);
    let observed = waiting_on(&pool, pid, "auth_legacy_self_passkeys_v1").await;
    let observed_before = if observed.is_some() {
        Some(db_now(&pool).await)
    } else {
        None
    };
    let elapsed = elapsed_database_deadline(&pool, created + Duration::seconds(8)).await;
    projection.rollback().await.unwrap();
    let read = finish(reader).await;
    no_remaining_waiters(&pool, &[pid, observed.unwrap_or(-1)]).await;
    assert!(
        observed.is_some()
            && observed_before.is_some_and(|at| at < created + Duration::seconds(8))
            && elapsed,
        "actual projection wait observed before and through real family deadline"
    );
    let body = response_json(
        read.expect("reader actor"),
        StatusCode::UNAUTHORIZED,
        &secrets,
        true,
    )
    .await;
    assert!(body == error_body("unauthorized", "invalid bearer token"));
    accepted(&f, &f.legacy.router, &token, &f.expected).await;
    assert!(before == all_rows(&pool).await);
    f.auth.close().await;
}

async fn other_company(pool: &PgPool, f: &mut Fixture) -> (String, Value) {
    let company = Uuid::new_v4();
    let subject = console_platform_test_support::seed_org_and_super_admin(
        pool,
        company,
        "Independent passkey reader",
    )
    .await;
    let bootstrap = BootstrapCredentialStore
        .issue_for_zero_credential_user(
            pool,
            *subject.as_uuid(),
            OrgId::from_uuid(company),
            db_now(&f.auth).await,
            Duration::hours(1),
        )
        .await
        .unwrap();
    let redeem: OtpRedeemResponse = post_json(
        f.legacy.router.clone(),
        "/api/v1/auth/otp/redeem",
        None,
        json!({"otp":bootstrap.token.as_str()}),
        StatusCode::OK,
    )
    .await;
    let mut authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
    enroll_passkey(&f.legacy.router, &mut authenticator, &redeem.access_token).await;
    let now = db_now(&f.auth).await;
    let mut tx = f.auth.begin().await.unwrap();
    let issued = RefreshTokenStore
        .issue_family_in_tx(
            &mut tx,
            *subject.as_uuid(),
            OrgId::from_uuid(company),
            now,
            Duration::hours(1),
        )
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let mut changed = claims(f, &redeem.access_token);
    assert!(changed["org"] == json!(company.to_string()));
    changed["iat"] = json!(now.unix_timestamp());
    changed["nbf"] = json!(now.unix_timestamp());
    changed["legacy_session"] =
        json!({"version":1,"family_id":issued.family_id,"home_org":company,"kind":"direct"});
    let token = sign(f, &changed);
    let expected = expected_summary(pool, subject).await;
    f.credential_secrets.extend([
        bootstrap.token.as_str().to_owned(),
        redeem.access_token,
        issued.token.as_str().to_owned(),
        issued.family_id.to_string(),
    ]);
    for (_, credential, passkey) in fence_credential_snapshot(pool, subject).await {
        f.credential_secrets.push(credential);
        f.credential_secrets.push(
            serde_json::to_string(&serde_json::from_str::<Value>(&passkey).unwrap()).unwrap(),
        );
        f.credential_secrets.push(passkey);
    }
    (token, expected)
}

#[sqlx::test(migrations = false)]
async fn independent_company_read_progresses_while_actual_subject_guard_is_held(pool: PgPool) {
    let mut f = fixture(&pool).await;
    let (other, expected) = other_company(&pool, &mut f).await;
    accepted(&f, &f.legacy.router, &other, &expected).await;
    let before = all_rows(&pool).await;
    let mut guard = f.auth.begin().await.unwrap();
    let pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *guard)
        .await
        .unwrap();
    guard_legacy_subject_in_tx(&mut guard, OrgId::knl(), *f.legacy.subject.as_uuid())
        .await
        .unwrap();
    let a = read_task(&f.legacy.router, &f.access_a);
    let observed = waiting_on(&pool, pid, "account_company_deactivation_guard_v1").await;
    let mut b = read_task(&f.legacy.router, &other);
    let b_completed = tokio::time::timeout(std::time::Duration::from_secs(4), &mut b).await;
    let a_still_blocked = if let Some(a_pid) = observed {
        sqlx::query_scalar::<_, bool>("SELECT $2=ANY(pg_catalog.pg_blocking_pids($1))")
            .bind(a_pid)
            .bind(pid)
            .fetch_one(&pool)
            .await
            .unwrap()
    } else {
        false
    };
    guard.rollback().await.unwrap();
    let a_response = finish(a).await;
    let (timely, b_response) = match b_completed {
        Ok(result) => (true, result.map_err(|_| "CompanyB actor panicked")),
        Err(_) => (false, finish(b).await),
    };
    no_remaining_waiters(&pool, &[pid, observed.unwrap_or(-1)]).await;
    assert!(
        observed.is_some() && a_still_blocked,
        "actual CompanyA subject wait must persist while B completes"
    );
    assert!(
        timely,
        "independent CompanyB must complete inside bound without releasing A"
    );
    let body = response_json(
        a_response.expect("CompanyA actor"),
        StatusCode::OK,
        &f.secrets(),
        true,
    )
    .await;
    assert!(body == f.expected);
    let mut secrets = f.secrets();
    secrets.push(other);
    let body = response_json(
        b_response.expect("CompanyB actor"),
        StatusCode::OK,
        &secrets,
        true,
    )
    .await;
    assert!(body == expected);
    assert!(before == all_rows(&pool).await);
    f.auth.close().await;
}
