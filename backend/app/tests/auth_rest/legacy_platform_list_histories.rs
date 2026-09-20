//! Child module of the approved list reader tests. No mocked principal or timer is authority.
use super::*;
use console_platform_auth::{LegacyPlatformSourceError, live_legacy_platform_source_in_tx};
use http::{HeaderMap, Method};
use tokio::task::JoinHandle;

struct Fixture {
    state: AppState,
    router: axum::Router,
    actor: UserId,
    access: String,
    secrets: Vec<String>,
    expected: Value,
    key: SigningKey,
    verifier: JwtVerifier,
    business: PgPool,
}

async fn fixture(pool: &PgPool) -> Fixture {
    prepare_http_database(pool).await;
    let business = login_test_pool(pool, TestDatabaseLogin::Business).await;
    let auth = login_test_pool(pool, TestDatabaseLogin::Auth).await;
    for (connection, role) in [(&business, "console_rt"), (&auth, "console_auth_rt")] {
        let ids: (String, String) = sqlx::query_as("SELECT session_user::text,current_user::text")
            .fetch_one(connection)
            .await
            .unwrap();
        assert!(
            ids == (role.to_owned(), role.to_owned()),
            "genuine login prerequisite"
        );
    }
    auth.close().await;
    let actor = UserId::new();
    sqlx::query("INSERT INTO public.users(id,display_name,roles,org_id) VALUES($1,$2,$3,$4)")
        .bind(actor.as_uuid())
        .bind("플랫폼 대기 및 복구 검증 담당자")
        .bind(vec!["SUPER_ADMIN"])
        .bind(OrgId::platform().as_uuid())
        .execute(pool)
        .await
        .unwrap();
    let root_present: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM public.accounts WHERE id=$1)")
            .bind(actor.as_uuid())
            .fetch_one(pool)
            .await
            .unwrap();
    assert!(root_present, "real automatic Account root prerequisite");
    sqlx::query("INSERT INTO public.organizations(slug,name,status) VALUES('list-history-company','긴 이름의 서울 인사 급여 운영 회사','ACTIVE')")
        .execute(pool).await.unwrap();
    let key = SigningKey::random(&mut OsRng);
    let private = key.to_pkcs8_pem(LineEnding::LF).unwrap().to_string();
    let public = key
        .verifying_key()
        .to_public_key_pem(LineEnding::LF)
        .unwrap();
    let state = app_state(pool.clone(), private, public.clone())
        .await
        .unwrap();
    let router = build_router(state.clone());
    let issued = BootstrapCredentialStore
        .issue_for_zero_credential_user(
            &business,
            *actor.as_uuid(),
            OrgId::platform(),
            db_now(pool).await,
            Duration::hours(1),
        )
        .await
        .unwrap();
    let redeemed: OtpRedeemResponse = post_json(
        router.clone(),
        "/api/v1/auth/otp/redeem",
        None,
        json!({"otp":issued.token.as_str()}),
        StatusCode::OK,
    )
    .await;
    let mut authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let credential = enroll_passkey(&router, &mut authenticator, &redeemed.access_token).await;
    let mut login = usernameless_login(&router, &mut authenticator, &credential).await;
    let verifier = JwtVerifier::from_es256_public_pem(
        JwtSettings {
            issuer: TEST_ISSUER.into(),
            audience: TEST_AUDIENCE.into(),
            access_token_ttl: Duration::minutes(15),
        },
        public.as_bytes(),
    )
    .unwrap();
    // Historical-shape successor: preserve all verified claims and original
    // expiry; only remove the newly produced additive binding in test memory.
    login.access_token = crate::legacy_platform_binding_producer::historical_access(
        &key,
        &verifier,
        &login.access_token,
    );
    let verified = verifier.verify_access_token(&login.access_token).unwrap();
    assert!(
        verified.sub == actor.to_string()
            && verified.org == OrgId::platform().to_string()
            && verified.platform
            && !verified.view_as
            && !verified.read_only
            && verified.legacy_session.is_none(),
        "genuine original unbound platform login"
    );
    let expected = expected_metadata(pool).await;
    assert!(
        expected.as_array().is_some_and(|rows| rows.len() >= 2),
        "nonempty real Companies prerequisite"
    );
    let secrets = vec![
        issued.token.as_str().to_owned(),
        redeemed.access_token,
        redeemed.refresh_token.unwrap(),
        login.refresh_token.unwrap(),
        login.access_token.clone(),
    ];
    Fixture {
        state,
        router,
        actor,
        access: login.access_token,
        secrets,
        expected,
        key,
        verifier,
        business,
    }
}

async fn timeout_state(pool: &PgPool, f: &Fixture) -> AppState {
    // Same existing configuration and admitted transports; only established request budget differs.
    let mut pairs = vec![
        (
            "CONSOLE_DATABASE_DURABILITY",
            r#"{"mode":"local_development"}"#.to_owned(),
        ),
        ("CONSOLE_APP_ROLE", AppRole::Api.to_string()),
        ("CONSOLE_HTTP_ADDR", "127.0.0.1:0".to_owned()),
        ("CONSOLE_JWT_ISSUER", TEST_ISSUER.to_owned()),
        ("CONSOLE_JWT_AUDIENCE", TEST_AUDIENCE.to_owned()),
        (
            "CONSOLE_JWT_PRIVATE_KEY_PEM",
            f.key.to_pkcs8_pem(LineEnding::LF).unwrap().to_string(),
        ),
        (
            "CONSOLE_JWT_PUBLIC_KEY_PEM",
            f.key
                .verifying_key()
                .to_public_key_pem(LineEnding::LF)
                .unwrap(),
        ),
        ("CONSOLE_WEBAUTHN_RP_ID", "example.com".to_owned()),
        ("CONSOLE_WEBAUTHN_RP_ORIGIN", TEST_ORIGIN.to_owned()),
        ("CONSOLE_WEBAUTHN_RP_NAME", "Console".to_owned()),
        ("CONSOLE_REQUEST_TIMEOUT_SECS", "3".to_owned()),
    ];
    pairs.extend(account_transport_urls(pool));
    let config = AppConfig::from_pairs(pairs).unwrap();
    assert!(config.request_timeout == std::time::Duration::from_secs(3));
    AppState::from_config(config).await.unwrap()
}

fn token_set(headers: &HeaderMap, name: http::header::HeaderName) -> Option<Vec<String>> {
    let mut result = Vec::new();
    for value in headers.get_all(name) {
        for part in value.to_str().ok()?.split(',') {
            let part = part.trim();
            if part.is_empty() {
                return None;
            }
            result.push(part.to_ascii_lowercase());
        }
    }
    Some(result)
}
fn private_response(headers: &HeaderMap, body: &[u8], secrets: &[String]) -> bool {
    let cache = token_set(headers, header::CACHE_CONTROL);
    let vary = token_set(headers, header::VARY);
    let Some(cache) = cache else { return false };
    let Some(vary) = vary else { return false };
    let cache_set: BTreeSet<_> = cache.iter().map(String::as_str).collect();
    let types: Vec<_> = headers.get_all(header::CONTENT_TYPE).iter().collect();
    cache.len() == 2
        && cache_set == BTreeSet::from(["no-store", "private"])
        && vary
            .iter()
            .filter(|v| v.as_str() == "authorization")
            .count()
            == 1
        && !vary.iter().any(|v| v == "*")
        && types.len() == 1
        && types[0]
            .to_str()
            .is_ok_and(|v| v.split(';').next() == Some("application/json"))
        && [
            header::ETAG,
            header::LAST_MODIFIED,
            header::LOCATION,
            header::SET_COOKIE,
        ]
        .iter()
        .all(|name| !headers.contains_key(name))
        && !secrets.is_empty()
        && secrets.iter().all(|secret| {
            !secret.is_empty()
                && !body.windows(secret.len()).any(|w| w == secret.as_bytes())
                && headers.iter().all(|(_, value)| {
                    !value
                        .as_bytes()
                        .windows(secret.len())
                        .any(|w| w == secret.as_bytes())
                })
        })
}
async fn response_parts(response: http::Response<Body>) -> (StatusCode, HeaderMap, Vec<u8>) {
    let (parts, body) = response.into_parts();
    (
        parts.status,
        parts.headers,
        to_bytes(body, 1024 * 1024).await.unwrap().to_vec(),
    )
}
fn request_task(
    router: &axum::Router,
    access: &str,
    method: Method,
) -> JoinHandle<http::Response<Body>> {
    let router = router.clone();
    let access = access.to_owned();
    tokio::spawn(async move {
        router
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(PATH)
                    .header(header::AUTHORIZATION, format!("Bearer {access}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap()
    })
}
async fn finish<T>(mut task: JoinHandle<T>) -> Result<T, &'static str> {
    match tokio::time::timeout(std::time::Duration::from_secs(8), &mut task).await {
        Ok(result) => result.map_err(|_| "owned request panicked"),
        Err(_) => {
            task.abort();
            let _ = task.await;
            Err("owned request exceeded cleanup deadline")
        }
    }
}
async fn waiting_on(
    pool: &PgPool,
    blocker: i32,
    query: &str,
    budget: std::time::Duration,
) -> Option<i32> {
    let deadline = tokio::time::Instant::now() + budget;
    loop {
        let pid:Option<i32>=sqlx::query_scalar("SELECT pid FROM pg_catalog.pg_stat_activity WHERE datname=current_database() AND usename='console_rt' AND wait_event_type='Lock' AND $1=ANY(pg_catalog.pg_blocking_pids(pid)) AND strpos(query,$2)>0 ORDER BY pid LIMIT 1")
            .bind(blocker).bind(query).fetch_optional(pool).await.unwrap();
        if pid.is_some() || tokio::time::Instant::now() >= deadline {
            return pid;
        }
        tokio::time::sleep(std::time::Duration::from_millis(15)).await;
    }
}
fn waited_cleanly(blocker: i32, waiter: Option<i32>, pending: bool, clean: bool) -> bool {
    blocker > 0 && waiter.is_some_and(|pid| pid > 0 && pid != blocker) && pending && clean
}

fn cancellation_complete(open_transactions: i64, dependent_waiters: i64) -> bool {
    open_transactions == 0 && dependent_waiters == 0
}

async fn clean_pids(observer: &mut sqlx::PgConnection, pids: &[i32]) -> bool {
    let observer_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *observer)
        .await
        .unwrap();
    if pids.contains(&observer_pid) {
        return false;
    }
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(8);
    loop {
        let (open, dependents): (i64, i64) = sqlx::query_as(
            "SELECT count(*) FILTER (WHERE pid=ANY($1) AND xact_start IS NOT NULL), count(*) FILTER (WHERE pg_catalog.pg_blocking_pids(pid) && $1) FROM pg_catalog.pg_stat_activity WHERE datname=current_database()"
        ).bind(pids).fetch_one(&mut *observer).await.unwrap();
        if cancellation_complete(open, dependents) {
            return true;
        }
        if tokio::time::Instant::now() >= deadline {
            return false;
        }
        tokio::time::sleep(std::time::Duration::from_millis(15)).await;
    }
}
fn only_role_changed(before: &Rows, after: &Rows, actor: UserId) -> bool {
    if !before.keys().eq(after.keys())
        || before
            .iter()
            .any(|(table, rows)| table != "users" && after.get(table) != Some(rows))
    {
        return false;
    }
    let parse = |rows: &Rows| {
        rows.get("users")
            .and_then(|raw| serde_json::from_str::<Vec<Value>>(raw).ok())
    };
    let (Some(mut expected), Some(actual)) = (parse(before), parse(after)) else {
        return false;
    };
    let mut count = 0;
    for row in &mut expected {
        if row["id"] == json!(actor) {
            if row["roles"] != json!(["SUPER_ADMIN"]) {
                return false;
            }
            row["roles"] = json!(["MEMBER"]);
            count += 1;
        }
    }
    count == 1 && expected.len() == actual.len() && expected.iter().all(|row| actual.contains(row))
}
async fn positive(pool: &PgPool, f: &Fixture, router: &axum::Router) {
    let before = all_rows(pool).await;
    let start = db_now(pool).await;
    let response = finish(request_task(router, &f.access, Method::GET))
        .await
        .unwrap();
    let end = db_now(pool).await;
    let after = all_rows(pool).await;
    let (status, headers, body) = response_parts(response).await;
    let value: Value = serde_json::from_slice(&body).unwrap();
    assert!(
        status == StatusCode::OK && metadata_matches(&value, &f.expected),
        "real permitted metadata prerequisite"
    );
    assert!(
        private_response(&headers, &body, &f.secrets),
        "real permitted response privacy"
    );
    assert!(
        exact_read_delta(
            &before,
            &after,
            f.actor,
            f.expected.as_array().unwrap().len(),
            start,
            end
        ),
        "exact permitted audit/no-other-effects prerequisite"
    );
}

#[sqlx::test(migrations = false)]
async fn mounted_list_waits_for_source_then_denies_committed_demotion(pool: PgPool) {
    let f = fixture(&pool).await;
    positive(&pool, &f, &f.router).await;
    let before = all_rows(&pool).await;
    // Hold this observer before the blocker so it cannot reuse an owned PID
    // and confuse its own active query with cancellation rollback evidence.
    let mut observer = pool.acquire().await.unwrap();
    let mut blocker = pool.begin().await.unwrap();
    let blocker_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *blocker)
        .await
        .unwrap();
    sqlx::query("UPDATE public.users SET roles=ARRAY['MEMBER']::text[] WHERE id=$1")
        .bind(f.actor.as_uuid())
        .execute(&mut *blocker)
        .await
        .unwrap();
    let request = request_task(&f.router, &f.access, Method::GET);
    let waiter = waiting_on(
        &pool,
        blocker_pid,
        "auth_legacy_platform_source_material_v1",
        std::time::Duration::from_secs(4),
    )
    .await;
    let was_pending = !request.is_finished();
    blocker.commit().await.unwrap();
    let result = finish(request).await;
    let pids: Vec<_> = std::iter::once(blocker_pid).chain(waiter).collect();
    let clean = clean_pids(&mut observer, &pids).await;
    let after = all_rows(&pool).await;
    set_role(&pool, f.actor, "SUPER_ADMIN").await;
    // Restore and exercise the identical route before asserting the observed history.
    positive(&pool, &f, &f.router).await;
    f.state.shutdown_realtime().await;
    f.business.close().await;
    let (status, headers, body) = response_parts(result.unwrap()).await;
    assert!(
        waited_cleanly(blocker_pid, waiter, was_pending, clean),
        "real owning Business source wait required"
    );
    assert!(
        status == StatusCode::FORBIDDEN,
        "post-wait current MEMBER must deny"
    );
    assert!(
        serde_json::from_slice::<Value>(&body).unwrap()
            == json!({"error":{"code":"forbidden","message":"platform principal cannot list tenants"}})
    );
    assert!(private_response(&headers, &body, &f.secrets));
    assert!(
        only_role_changed(&before, &after, f.actor),
        "exact committed role delta only; denied list writes no audit"
    );
}

async fn timeout_history(pool: PgPool, method: Method) {
    let f = fixture(&pool).await;
    let state = timeout_state(&pool, &f).await;
    let router = build_router(state.clone());
    positive(&pool, &f, &router).await;
    let before = all_rows(&pool).await;
    // Hold this observer before the blocker so it cannot reuse an owned PID
    // and confuse its own active query with cancellation rollback evidence.
    let mut observer = pool.acquire().await.unwrap();
    let mut blocker = pool.begin().await.unwrap();
    let blocker_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *blocker)
        .await
        .unwrap();
    // groups stalls the actual metadata projection AFTER source guards; organizations would not.
    sqlx::query("LOCK TABLE public.groups IN ACCESS EXCLUSIVE MODE")
        .execute(&mut *blocker)
        .await
        .unwrap();
    let request = request_task(&router, &f.access, method.clone());
    let waiter = waiting_on(
        &pool,
        blocker_pid,
        "platform_list_organizations",
        std::time::Duration::from_secs(2),
    )
    .await;
    let was_pending = !request.is_finished();
    let result = finish(request).await;
    // The actual outer timeout must return while the real DB blocker still exists.
    let blocker_held:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_locks WHERE pid=pg_backend_pid() AND relation='public.groups'::regclass AND mode='AccessExclusiveLock' AND granted)")
        .fetch_one(&mut *blocker).await.unwrap();
    blocker.rollback().await.unwrap();
    let pids: Vec<_> = std::iter::once(blocker_pid).chain(waiter).collect();
    let clean = clean_pids(&mut observer, &pids).await;
    let after = all_rows(&pool).await;
    positive(&pool, &f, &router).await;
    state.shutdown_realtime().await;
    f.state.shutdown_realtime().await;
    f.business.close().await;
    let (status, headers, body) = response_parts(result.unwrap()).await;
    assert!(
        waited_cleanly(blocker_pid, waiter, was_pending, clean) && blocker_held,
        "actual projection blocker must outlive HTTP timeout"
    );
    assert!(
        status == StatusCode::REQUEST_TIMEOUT,
        "actual configured App outer timeout status"
    );
    assert!(
        private_response(&headers, &body, &f.secrets),
        "outer envelope must preserve final private response"
    );
    if method == Method::HEAD {
        assert!(
            body.is_empty(),
            "HEAD envelope must not regain a timeout body"
        );
    } else {
        assert!(
            serde_json::from_slice::<Value>(&body).unwrap()
                == json!({"error":{"code":"request_timeout","message":"request timed out"}})
        );
    }
    assert!(
        before == after,
        "cancelled projection must publish no audit or other durable effect"
    );
}
#[sqlx::test(migrations = false)]
async fn mounted_list_get_outer_timeout_is_private_and_recovers(pool: PgPool) {
    timeout_history(pool, Method::GET).await;
}
#[sqlx::test(migrations = false)]
async fn mounted_list_head_outer_timeout_is_private_empty_and_recovers(pool: PgPool) {
    timeout_history(pool, Method::HEAD).await;
}

#[sqlx::test(migrations = false)]
async fn direct_source_rejects_borrowed_repeatable_read_without_downgrade(pool: PgPool) {
    let f = fixture(&pool).await;
    positive(&pool, &f, &f.router).await;
    let before = all_rows(&pool).await;
    let mut tx = f.business.begin().await.unwrap();
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ")
        .execute(&mut *tx)
        .await
        .unwrap();
    let refused =
        live_legacy_platform_source_in_tx(&mut tx, &f.verifier, &f.access, Duration::days(30))
            .await;
    // The SQL exception aborts this transaction; rolling back is mandatory, not downgrade/retry.
    tx.rollback().await.unwrap();
    let after = all_rows(&pool).await;
    let mut accepted = f.business.begin().await.unwrap();
    sqlx::query("SET TRANSACTION ISOLATION LEVEL READ COMMITTED")
        .execute(&mut *accepted)
        .await
        .unwrap();
    let live = live_legacy_platform_source_in_tx(
        &mut accepted,
        &f.verifier,
        &f.access,
        Duration::days(30),
    )
    .await
    .unwrap();
    let isolation: String = sqlx::query_scalar("SHOW transaction_isolation")
        .fetch_one(&mut *accepted)
        .await
        .unwrap();
    let current = live.subject() == f.actor
        && live.home() == OrgId::platform()
        && live.current_subject() == f.actor
        && live.current_home() == OrgId::platform()
        && live.current_active()
        && !live.current_account_fenced()
        && live.current_roles() == ["SUPER_ADMIN"]
        && live.binding().is_none()
        && live.family().is_none();
    accepted.rollback().await.unwrap();
    let final_rows = all_rows(&pool).await;
    f.state.shutdown_realtime().await;
    f.business.close().await;
    assert!(
        matches!(refused, Err(LegacyPlatformSourceError::Unavailable)),
        "unsupported caller isolation must refuse"
    );
    assert!(
        isolation == "read committed" && current,
        "actual borrowed source positive control"
    );
    assert!(
        before == after && before == final_rows,
        "source material projection has no durable effects"
    );
}

#[test]
fn private_response_oracle_rejects_header_and_current_input_corruption() {
    let secret = "fixture-private-token".to_owned();
    let invalid = "Basic current-invalid-input".to_owned();
    let secrets = vec![secret, invalid.clone()];
    let mut headers = HeaderMap::new();
    headers.insert(header::CACHE_CONTROL, "no-store, private".parse().unwrap());
    headers.insert(header::VARY, "Origin, Authorization".parse().unwrap());
    headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
    let body = br#"{"error":{"code":"request_timeout","message":"request timed out"}}"#;
    assert!(private_response(&headers, body, &secrets));
    assert!(private_response(&headers, b"", &secrets));
    for value in [
        "no-store",
        "private",
        "no-store, public",
        "no-store, private, public",
    ] {
        let mut changed = headers.clone();
        changed.insert(header::CACHE_CONTROL, value.parse().unwrap());
        assert!(!private_response(&changed, body, &secrets));
    }
    for value in ["Origin", "*", "Authorization, authorization"] {
        let mut changed = headers.clone();
        changed.insert(header::VARY, value.parse().unwrap());
        assert!(!private_response(&changed, body, &secrets));
    }
    for name in [header::CACHE_CONTROL, header::VARY, header::CONTENT_TYPE] {
        let mut changed = headers.clone();
        changed.remove(name);
        assert!(!private_response(&changed, body, &secrets));
    }
    for name in [
        header::ETAG,
        header::LAST_MODIFIED,
        header::LOCATION,
        header::SET_COOKIE,
    ] {
        let mut changed = headers.clone();
        changed.insert(name, "private".parse().unwrap());
        assert!(!private_response(&changed, body, &secrets));
    }
    let mut echo = headers.clone();
    echo.insert("x-debug", invalid.parse().unwrap());
    assert!(!private_response(&echo, body, &secrets));
    assert!(!private_response(&headers, invalid.as_bytes(), &secrets));
    let mut duplicate = headers.clone();
    duplicate.append(header::CONTENT_TYPE, "application/json".parse().unwrap());
    assert!(!private_response(&duplicate, body, &secrets));
    let mut nonjson = headers.clone();
    nonjson.insert(header::CONTENT_TYPE, "text/html".parse().unwrap());
    assert!(!private_response(&nonjson, body, &secrets));
    assert!(!private_response(&headers, body, &[]));
}

#[test]
fn wait_and_role_delta_oracles_reject_omissions_and_extra_effects() {
    assert!(
        cancellation_complete(0, 0),
        "absent or idle-without-transaction backends are clean"
    );
    assert!(
        !cancellation_complete(1, 0),
        "ACTIVE open transaction must not appear clean"
    );
    assert!(
        !cancellation_complete(2, 0),
        "idle-in-transaction remains unclean"
    );
    assert!(
        !cancellation_complete(0, 1),
        "dependent blocker edge remains unclean"
    );
    assert!(
        !cancellation_complete(1, 1),
        "open transaction plus dependent wait remains unclean"
    );

    assert!(waited_cleanly(1, Some(2), true, true));
    for (blocker, waiter, pending, clean) in [
        (1, None, true, true),
        (1, Some(1), true, true),
        (0, Some(2), true, true),
        (1, Some(0), true, true),
        (1, Some(2), false, true),
        (1, Some(2), true, false),
    ] {
        assert!(!waited_cleanly(blocker, waiter, pending, clean));
    }
    let actor = UserId::new();
    let source = json!({"id": actor, "roles": ["SUPER_ADMIN"], "retained": "original"});
    let control = json!({"id": UserId::new(), "roles": ["MEMBER"], "retained": "control"});
    let before = Rows::from([
        (
            "users".to_owned(),
            json!([source.clone(), control.clone()]).to_string(),
        ),
        ("audit_events".to_owned(), "[]".to_owned()),
    ]);
    let mut member = source.clone();
    member["roles"] = json!(["MEMBER"]);
    let mut after = before.clone();
    after.insert(
        "users".to_owned(),
        json!([member.clone(), control.clone()]).to_string(),
    );
    assert!(only_role_changed(&before, &after, actor));
    assert!(!only_role_changed(&before, &before, actor));
    for value in [
        json!([member.clone()]),
        json!([member.clone(), member.clone()]),
        json!([]),
    ] {
        let mut changed = after.clone();
        changed.insert("users".to_owned(), value.to_string());
        assert!(!only_role_changed(&before, &changed, actor));
    }
    let mut extra_field = member.clone();
    extra_field["retained"] = json!("changed");
    let mut changed = after.clone();
    changed.insert(
        "users".to_owned(),
        json!([extra_field, control]).to_string(),
    );
    assert!(!only_role_changed(&before, &changed, actor));
    let mut added_audit = after.clone();
    added_audit.insert("audit_events".to_owned(), "[{}]".to_owned());
    assert!(!only_role_changed(&before, &added_audit, actor));
    let mut omitted = after.clone();
    omitted.remove("audit_events");
    assert!(!only_role_changed(&before, &omitted, actor));
}

#[path = "legacy_platform_list_credentials.rs"]
mod legacy_platform_list_credentials;

#[path = "legacy_platform_list_unavailable.rs"]
mod legacy_platform_list_unavailable;

#[path = "legacy_platform_ops_reads.rs"]
mod legacy_platform_ops_reads;

#[path = "legacy_platform_group_reads.rs"]
mod legacy_platform_group_reads;
