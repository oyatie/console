// Additive real-owner HTTP cases. Included only after independent test approval;
// retained auth_rest.rs stays byte-identical until the original RED is GREEN.

fn assert_native_reuse_transition(before: &Value, after: &Value, account: Uuid, reused: &Value) {
    assert_eq!(before["families"].as_array().unwrap().len(), 1);
    assert_eq!(before["tokens"].as_array().unwrap().len(), 2);
    let family = &before["families"][0];
    let revoked = &after["families"][0];
    assert!(!revoked["revoked_at"].is_null());
    let events: Vec<_> = after["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["kind"] == "SESSION_REVOKED")
        .collect();
    assert_eq!(events.len(), 1, "exactly one durable reuse event");
    let event = events[0];
    let generation = before["security"]["security_generation"]
        .as_i64()
        .unwrap()
        .to_string();
    let evidence = json!({
        "kind":"ACCOUNT_SESSION_TRANSITION", "operation":"REFRESH_REUSE",
        "account_id":account.to_string(), "session_id":family["id"],
        "security_generation":generation, "auth_time":logout_micros(&family["auth_time"]),
        "assurance":family["assurance"], "before_state":"LIVE", "after_state":"REVOKED",
        "revoked_at":logout_micros(&revoked["revoked_at"]), "reason":"reuse_detected",
        "reused_token_id":reused
    });
    assert_eq!(event["account_id"], account.to_string());
    assert_eq!(event["actor_account_id"], account.to_string());
    assert_eq!(event["session_id"], family["id"]);
    assert_eq!(event["occurred_at"], revoked["revoked_at"]);
    assert_eq!(event["evidence_ref"], evidence);
    assert_eq!(
        event["payload"],
        json!({
            "kind":"SESSION_REVOKED", "account_id":account.to_string(),
            "before_generation":generation,"after_generation":generation,
            "credential_id":null,"session_id":family["id"],"evidence":evidence
        })
    );
    let mut expected_tokens = before["tokens"].clone();
    for token in expected_tokens.as_array_mut().unwrap() {
        if token["revoked_at"].is_null() {
            token["revoked_at"] = revoked["revoked_at"].clone();
        }
        if token["id"] == *reused {
            token["reuse_detected_at"] = revoked["revoked_at"].clone();
        }
    }
    assert!(
        after["tokens"] == expected_tokens,
        "reuse lost or modified retained token material"
    );
    let mut expected_families = before["families"].clone();
    expected_families[0]["revoked_at"] = revoked["revoked_at"].clone();
    expected_families[0]["revoked_reason"] = json!("reuse_detected");
    assert!(
        after["families"] == expected_families,
        "reuse changed unrelated family state"
    );
    let retained_events: Vec<_> = after["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|entry| entry["id"] != event["id"])
        .cloned()
        .collect();
    assert!(
        json!(retained_events) == before["events"],
        "reuse rewrote or omitted history"
    );
    assert_eq!(
        after["events"].as_array().unwrap().len(),
        before["events"].as_array().unwrap().len() + 1
    );
    for field in ["security", "keys", "terms", "ceremonies"] {
        assert!(
            before[field] == after[field],
            "reuse changed unrelated Account state"
        );
    }
}

async fn native_rotation(router: &axum::Router, cookies: &Cookies, csrf: &str) -> Response {
    request(
        router,
        "POST",
        "/api/v2/auth/token/refresh",
        cookies,
        Some(json!({})),
        &[("X-Console-CSRF", csrf)],
    )
    .await
}

#[sqlx::test(migrations = false)]
async fn native_refresh_reuse_commits_revocation_and_denies_successor(pool: PgPool) {
    let app = fixture(&pool).await;
    let (attempt, original) = enrolled(&app).await;
    let (other, _) = enrolled(&app).await;
    let other_before = snapshot(&pool, other.account).await;
    let csrf = proof(&app, &original).await;
    let mut current = original.clone();
    session(
        &native_rotation(&app, &original, &csrf).await,
        StatusCode::OK,
        attempt.account,
        &mut current,
    );
    let before = snapshot(&pool, attempt.account).await;
    let reused = before["tokens"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| !t["used_at"].is_null())
        .unwrap()["id"]
        .clone();
    let refused = native_rotation(&app, &original, &csrf).await;
    refused.error(StatusCode::UNAUTHORIZED, "authentication_invalid");
    assert!(!refused.headers.contains_key(header::SET_COOKIE));
    let after = snapshot(&pool, attempt.account).await;
    assert_native_reuse_transition(&before, &after, attempt.account, &reused);
    request(&app, "GET", "/api/v2/accounts/me", &current, None, &[])
        .await
        .error(StatusCode::UNAUTHORIZED, "authentication_invalid");
    native_rotation(&app, &current, &csrf)
        .await
        .error(StatusCode::UNAUTHORIZED, "authentication_invalid");
    assert!(
        snapshot(&pool, attempt.account).await == after,
        "repeat minted effect"
    );
    assert!(snapshot(&pool, other.account).await == other_before);
}

#[sqlx::test(migrations = false)]
async fn native_refresh_wrong_proof_cannot_revoke_used_token_family(pool: PgPool) {
    let app = fixture(&pool).await;
    let (attempt, original) = enrolled(&app).await;
    let (other, other_cookies) = enrolled(&app).await;
    let own = proof(&app, &original).await;
    let foreign = proof(&app, &other_cookies).await;
    let mut current = original.clone();
    session(
        &native_rotation(&app, &original, &own).await,
        StatusCode::OK,
        attempt.account,
        &mut current,
    );
    let before = snapshot(&pool, attempt.account).await;
    let other_before = snapshot(&pool, other.account).await;
    for invalid in [&foreign, "invalid-signature"] {
        native_rotation(&app, &original, invalid)
            .await
            .error(StatusCode::FORBIDDEN, "csrf_invalid");
        assert!(snapshot(&pool, attempt.account).await == before);
        assert!(snapshot(&pool, other.account).await == other_before);
    }
    // Current successor remains genuinely usable after refused replay requests.
    session(
        &native_rotation(&app, &current.clone(), &own).await,
        StatusCode::OK,
        attempt.account,
        &mut current,
    );
    assert!(snapshot(&pool, other.account).await == other_before);
}

#[sqlx::test(migrations = false)]
async fn native_refresh_expired_consumed_is_reuse_but_unused_expiry_is_inert(pool: PgPool) {
    let app = fixture(&pool).await;
    let mut observed = 0;
    for consume in [false, true] {
        let (attempt, cookies) = enrolled(&app).await;
        let csrf = proof(&app, &cookies).await;
        // Fixture-owned expiry change isolates token expiry, preserving family,
        // signature and proof. It is not a production lifetime writer claim.
        let expires: OffsetDateTime = sqlx::query_scalar("UPDATE public.auth_refresh_tokens SET expires_at=clock_timestamp()+interval '3 seconds' WHERE user_id=$1 AND used_at IS NULL RETURNING expires_at")
            .bind(attempt.account).fetch_one(&pool).await.unwrap();
        let initial = snapshot(&pool, attempt.account).await;
        let reused = initial["tokens"][0]["id"].clone();
        let mut successor = cookies.clone();
        if consume {
            session(
                &native_rotation(&app, &cookies, &csrf).await,
                StatusCode::OK,
                attempt.account,
                &mut successor,
            );
        }
        let before = snapshot(&pool, attempt.account).await;
        sqlx::query("SELECT pg_sleep(GREATEST(0.0,extract(epoch FROM $1::timestamptz-clock_timestamp()))+0.025)")
            .bind(expires).execute(&pool).await.unwrap();
        native_rotation(&app, &cookies, &csrf)
            .await
            .error(StatusCode::UNAUTHORIZED, "authentication_invalid");
        let after = snapshot(&pool, attempt.account).await;
        if consume {
            assert_native_reuse_transition(&before, &after, attempt.account, &reused);
            request(&app, "GET", "/api/v2/accounts/me", &successor, None, &[])
                .await
                .error(StatusCode::UNAUTHORIZED, "authentication_invalid");
        } else {
            assert!(
                after == before,
                "unused expiry must not be mislabeled replay or revoke family"
            );
            request(&app, "GET", "/api/v2/accounts/me", &cookies, None, &[])
                .await
                .json(StatusCode::OK);
        }
        observed += 1;
    }
    assert_eq!(observed, 2);
}

#[sqlx::test(migrations = false)]
async fn native_refresh_same_token_concurrency_creates_one_successor_then_revokes(pool: PgPool) {
    let app = fixture(&pool).await;
    let (attempt, cookies) = enrolled(&app).await;
    let csrf = proof(&app, &cookies).await;
    let before = snapshot(&pool, attempt.account).await;
    let mut blocker = pool.begin().await.unwrap();
    let blocker_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *blocker)
        .await
        .unwrap();
    sqlx::query("SELECT account_id FROM public.account_security WHERE account_id=$1 FOR UPDATE")
        .bind(attempt.account)
        .fetch_one(&mut *blocker)
        .await
        .unwrap();
    let barrier = std::sync::Arc::new(tokio::sync::Barrier::new(3));
    let mut workers = tokio::task::JoinSet::new();
    for _ in 0..2 {
        let router = app.service.clone();
        let cookies = cookies.clone();
        let csrf = csrf.clone();
        let barrier = barrier.clone();
        workers.spawn(async move {
            barrier.wait().await;
            native_rotation(&router, &cookies, &csrf).await
        });
    }
    barrier.wait().await;
    let contended = tokio::time::timeout(std::time::Duration::from_secs(3), async {
        loop {
            if workers.try_join_next().is_some() { break false; }
            let blocked: i64 = sqlx::query_scalar("WITH RECURSIVE waiting(pid) AS (SELECT a.pid FROM pg_catalog.pg_stat_activity a WHERE a.datname=current_database() AND $1=ANY(pg_catalog.pg_blocking_pids(a.pid)) UNION SELECT a.pid FROM pg_catalog.pg_stat_activity a JOIN waiting w ON w.pid=ANY(pg_catalog.pg_blocking_pids(a.pid)) WHERE a.datname=current_database()) SELECT count(DISTINCT a.pid) FROM pg_catalog.pg_stat_activity a JOIN waiting w ON w.pid=a.pid WHERE a.datname=current_database() AND a.usename='console_auth_rt' AND a.query LIKE '%account_security_lock_exclusive_v1%'")
                .bind(blocker_pid).fetch_one(&pool).await.unwrap();
            if blocked == 2 { break workers.try_join_next().is_none(); }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
    }).await;
    if !matches!(contended, Ok(true)) {
        workers.abort_all();
        while workers.join_next().await.is_some() {}
        tokio::time::timeout(std::time::Duration::from_secs(5), blocker.rollback())
            .await
            .unwrap()
            .unwrap();
        panic!("both real requests must be observed contending on the exact Account guard");
    }
    tokio::time::timeout(std::time::Duration::from_secs(5), blocker.rollback())
        .await
        .unwrap()
        .unwrap();
    let mut responses = Vec::new();
    while !workers.is_empty() {
        match tokio::time::timeout(std::time::Duration::from_secs(10), workers.join_next()).await {
            Ok(Some(result)) => responses.push(result.unwrap()),
            Ok(None) => break,
            Err(_) => {
                workers.abort_all();
                while workers.join_next().await.is_some() {}
                panic!("real refresh did not complete");
            }
        }
    }
    assert_eq!(
        responses
            .iter()
            .filter(|r| r.status == StatusCode::OK)
            .count(),
        1
    );
    assert_eq!(
        responses
            .iter()
            .filter(|r| r.status == StatusCode::UNAUTHORIZED)
            .count(),
        1
    );
    let mut winner = cookies.clone();
    session(
        responses
            .iter()
            .find(|r| r.status == StatusCode::OK)
            .unwrap(),
        StatusCode::OK,
        attempt.account,
        &mut winner,
    );
    responses
        .iter()
        .find(|r| r.status == StatusCode::UNAUTHORIZED)
        .unwrap()
        .error(StatusCode::UNAUTHORIZED, "authentication_invalid");
    let after = snapshot(&pool, attempt.account).await;
    assert_eq!(after["families"].as_array().unwrap().len(), 1);
    assert_eq!(after["families"][0]["revoked_reason"], "reuse_detected");
    let tokens = after["tokens"].as_array().unwrap();
    assert_eq!(tokens.len(), 2, "both requests cannot create a successor");
    assert!(tokens.iter().all(|t| !t["revoked_at"].is_null()));
    let predecessor = tokens
        .iter()
        .find(|t| t["id"] == before["tokens"][0]["id"])
        .unwrap();
    let successor = tokens
        .iter()
        .find(|t| t["id"] != predecessor["id"])
        .unwrap();
    assert_eq!(predecessor["replaced_by"], successor["id"]);
    assert_eq!(predecessor["used_at"], successor["issued_at"]);
    assert!(successor["used_at"].is_null() && successor["replaced_by"].is_null());
    assert_eq!(successor["user_id"], predecessor["user_id"]);
    assert_eq!(successor["family_id"], predecessor["family_id"]);
    assert!(successor["org_id"].is_null());
    assert!(successor["reuse_detected_at"].is_null());
    exact_keys(
        successor,
        &[
            "id",
            "family_id",
            "user_id",
            "token_hash",
            "issued_at",
            "expires_at",
            "used_at",
            "replaced_by",
            "revoked_at",
            "reuse_detected_at",
            "org_id",
        ],
    );
    let expected_hash: String = Sha256::digest(winner.0[REFRESH].as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    assert!(
        successor["token_hash"] == json!(format!("\\x{expected_hash}")),
        "replacement cookie does not match independently read token hash"
    );
    let issuance = OffsetDateTime::parse(
        successor["issued_at"].as_str().unwrap(),
        &time::format_description::well_known::Rfc3339,
    )
    .unwrap();
    let expiry = OffsetDateTime::parse(
        successor["expires_at"].as_str().unwrap(),
        &time::format_description::well_known::Rfc3339,
    )
    .unwrap();
    assert!(expiry > issuance);
    // Independently model the one allowed rotation from the original snapshot.
    // Only allocated successor identity/hash/timestamps are observed outcomes;
    // every existing field/history comes from BEFORE, not an adjusted AFTER.
    let mut rotated = before.clone();
    rotated["tokens"][0]["used_at"] = successor["issued_at"].clone();
    rotated["tokens"][0]["replaced_by"] = successor["id"].clone();
    let modeled_successor = json!({
        "id":successor["id"],"family_id":before["families"][0]["id"],
        "user_id":attempt.account,"org_id":null,"token_hash":format!("\\x{expected_hash}"),
        "issued_at":successor["issued_at"],"expires_at":successor["expires_at"],
        "used_at":null,"replaced_by":null,"revoked_at":null,"reuse_detected_at":null
    });
    rotated["tokens"]
        .as_array_mut()
        .unwrap()
        .push(modeled_successor);
    rotated["tokens"]
        .as_array_mut()
        .unwrap()
        .sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
    assert_native_reuse_transition(
        &rotated,
        &after,
        attempt.account,
        &before["tokens"][0]["id"],
    );
    request(&app, "GET", "/api/v2/accounts/me", &winner, None, &[])
        .await
        .error(StatusCode::UNAUTHORIZED, "authentication_invalid");
}

async fn release_native_refresh_wait_after_expiry(
    pool: &PgPool,
    blocker: sqlx::Transaction<'_, sqlx::Postgres>,
    blocker_pid: i32,
    expires: i64,
    query_fragment: &str,
    mut work: tokio::task::JoinHandle<Response>,
) -> Response {
    let reached = tokio::time::timeout(std::time::Duration::from_secs(3), async {
        loop {
            let blocked: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_stat_activity a WHERE a.datname=current_database() AND a.usename='console_auth_rt' AND $1=ANY(pg_catalog.pg_blocking_pids(a.pid)) AND a.query LIKE $2)")
                .bind(blocker_pid).bind(format!("%{query_fragment}%")).fetch_one(pool).await.unwrap();
            if blocked { break true; }
            if work.is_finished() { break false; }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
    }).await;
    if !matches!(reached, Ok(true)) {
        work.abort();
        let _ = (&mut work).await;
        tokio::time::timeout(std::time::Duration::from_secs(5), blocker.rollback())
            .await
            .unwrap()
            .unwrap();
        panic!("exact restricted-owner wait not observed; expiry assertion unreached");
    }
    let elapsed = tokio::time::timeout(std::time::Duration::from_secs(6),
        sqlx::query("SELECT pg_sleep(GREATEST(0.0,$1::double precision-extract(epoch FROM clock_timestamp()))+0.025)")
            .bind(expires as f64).execute(pool),
    ).await;
    tokio::time::timeout(std::time::Duration::from_secs(5), blocker.rollback())
        .await
        .unwrap()
        .unwrap();
    if !matches!(elapsed, Ok(Ok(_))) {
        work.abort();
        let _ = (&mut work).await;
        panic!("database authority clock did not cross tested expiry");
    }
    match tokio::time::timeout(std::time::Duration::from_secs(5), &mut work).await {
        Ok(response) => response.unwrap(),
        Err(_) => {
            work.abort();
            let _ = work.await;
            panic!("request did not finish after observed wait");
        }
    }
}

#[sqlx::test(migrations = false)]
async fn native_refresh_account_wait_rechecks_proof_before_consuming_token(pool: PgPool) {
    let (app, key) = signed_fixture(&pool).await;
    let (attempt, cookies) = enrolled(&app).await;
    let ordinary = proof(&app, &cookies).await;
    let before = snapshot(&pool, attempt.account).await;
    let mut blocker = pool.begin().await.unwrap();
    let blocker_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *blocker)
        .await
        .unwrap();
    sqlx::query("SELECT account_id FROM public.account_security WHERE account_id=$1 FOR UPDATE")
        .bind(attempt.account)
        .fetch_one(&mut *blocker)
        .await
        .unwrap();
    let now: OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
        .fetch_one(&pool)
        .await
        .unwrap();
    let expires = now.unix_timestamp() + 5;
    let mut claims = signed_claims(&ordinary, &key).unwrap();
    claims["exp"] = json!(expires);
    let short = sign_proof_claims(&claims, &key);
    let service = app.service.clone();
    let sent = cookies.clone();
    let work = tokio::spawn(async move { native_rotation(&service, &sent, &short).await });
    let denied = release_native_refresh_wait_after_expiry(
        &pool,
        blocker,
        blocker_pid,
        expires,
        "account_security_lock_exclusive_v1",
        work,
    )
    .await;
    denied.error(StatusCode::FORBIDDEN, "csrf_invalid");
    assert!(!denied.headers.contains_key(header::SET_COOKIE));
    assert!(
        snapshot(&pool, attempt.account).await == before,
        "post-wait proof expiry consumed token"
    );
    let mut current = cookies.clone();
    session(
        &native_rotation(&app, &cookies, &ordinary).await,
        StatusCode::OK,
        attempt.account,
        &mut current,
    );
}

#[sqlx::test(migrations = false)]
async fn native_refresh_reuse_event_wait_expiry_rolls_back_revocation_and_audit(pool: PgPool) {
    let (app, key) = signed_fixture(&pool).await;
    let (attempt, original) = enrolled(&app).await;
    let ordinary = proof(&app, &original).await;
    let mut current = original.clone();
    session(
        &native_rotation(&app, &original, &ordinary).await,
        StatusCode::OK,
        attempt.account,
        &mut current,
    );
    let before = snapshot(&pool, attempt.account).await;
    let reused = before["tokens"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| !t["used_at"].is_null())
        .unwrap()["id"]
        .clone();
    let mut blocker = pool.begin().await.unwrap();
    let blocker_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *blocker)
        .await
        .unwrap();
    sqlx::query("LOCK TABLE public.account_security_events IN ACCESS EXCLUSIVE MODE")
        .execute(&mut *blocker)
        .await
        .unwrap();
    let now: OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
        .fetch_one(&pool)
        .await
        .unwrap();
    let expires = now.unix_timestamp() + 5;
    let mut claims = signed_claims(&ordinary, &key).unwrap();
    claims["exp"] = json!(expires);
    let short = sign_proof_claims(&claims, &key);
    let service = app.service.clone();
    let sent = original.clone();
    let work = tokio::spawn(async move { native_rotation(&service, &sent, &short).await });
    let denied = release_native_refresh_wait_after_expiry(
        &pool,
        blocker,
        blocker_pid,
        expires,
        "account_session_refresh_reuse_v1",
        work,
    )
    .await;
    denied.error(StatusCode::FORBIDDEN, "csrf_invalid");
    assert!(!denied.headers.contains_key(header::SET_COOKIE));
    assert!(
        snapshot(&pool, attempt.account).await == before,
        "expired proof committed reuse event or family revocation"
    );
    request(&app, "GET", "/api/v2/accounts/me", &current, None, &[])
        .await
        .json(StatusCode::OK);
    native_rotation(&app, &original, &ordinary)
        .await
        .error(StatusCode::UNAUTHORIZED, "authentication_invalid");
    assert_native_reuse_transition(
        &before,
        &snapshot(&pool, attempt.account).await,
        attempt.account,
        &reused,
    );
}

#[test]
fn native_reuse_oracle_rejects_missing_effect_and_omitted_history() {
    let account = Uuid::new_v4();
    let family = Uuid::new_v4();
    let reused = json!(Uuid::new_v4());
    let timestamp = "2026-09-16T12:00:00Z";
    let revoked_at = "2026-09-16T12:01:00Z";
    let before = json!({
        "security":{"security_generation":1},"keys":[],"terms":[],"ceremonies":[],
        "families":[{"id":family,"auth_time":timestamp,"assurance":"PASSKEY_PRIMARY","revoked_at":null,"revoked_reason":null}],
        "tokens":[{"id":reused,"revoked_at":null,"reuse_detected_at":null},{"id":Uuid::new_v4(),"revoked_at":null,"reuse_detected_at":null}],
        "events":[{"id":Uuid::new_v4(),"kind":"TERMS_ACCEPTED"}]
    });
    let mut after = before.clone();
    after["families"][0]["revoked_at"] = json!(revoked_at);
    after["families"][0]["revoked_reason"] = json!("reuse_detected");
    for token in after["tokens"].as_array_mut().unwrap() {
        token["revoked_at"] = json!(revoked_at);
        if token["id"] == reused {
            token["reuse_detected_at"] = json!(revoked_at);
        }
    }
    let evidence = json!({"kind":"ACCOUNT_SESSION_TRANSITION","operation":"REFRESH_REUSE","account_id":account,"session_id":family,"security_generation":"1","auth_time":logout_micros(&json!(timestamp)),"assurance":"PASSKEY_PRIMARY","before_state":"LIVE","after_state":"REVOKED","revoked_at":logout_micros(&json!(revoked_at)),"reason":"reuse_detected","reused_token_id":reused});
    after["events"].as_array_mut().unwrap().push(json!({"id":Uuid::new_v4(),"kind":"SESSION_REVOKED","account_id":account,"actor_account_id":account,"session_id":family,"occurred_at":revoked_at,"evidence_ref":evidence,"payload":{"kind":"SESSION_REVOKED","account_id":account,"before_generation":"1","after_generation":"1","credential_id":null,"session_id":family,"evidence":evidence}}));
    assert_native_reuse_transition(&before, &after, account, &reused);
    for defect in [
        "missing_effect",
        "missing_marker",
        "wrong_marker",
        "missing_token",
        "missing_event",
        "missing_history",
        "logout_reason",
    ] {
        let mut invalid = after.clone();
        match defect {
            "missing_effect" => invalid["families"][0]["revoked_at"] = Value::Null,
            "missing_marker" => invalid["tokens"][0]["reuse_detected_at"] = Value::Null,
            "wrong_marker" => invalid["tokens"][1]["reuse_detected_at"] = json!(revoked_at),
            "missing_token" => {
                invalid["tokens"].as_array_mut().unwrap().pop();
            }
            "missing_event" => {
                invalid["events"].as_array_mut().unwrap().pop();
            }
            "missing_history" => {
                invalid["events"].as_array_mut().unwrap().remove(0);
            }
            "logout_reason" => invalid["events"][1]["evidence_ref"]["operation"] = json!("LOGOUT"),
            _ => unreachable!(),
        }
        assert!(
            std::panic::catch_unwind(|| assert_native_reuse_transition(
                &before, &invalid, account, &reused
            ))
            .is_err(),
            "oracle accepted {defect}"
        );
    }
}
