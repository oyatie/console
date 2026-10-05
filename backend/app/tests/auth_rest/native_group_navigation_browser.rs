// Held real HTTP/browser boundary. Account/Company population is the preceding
// real UI; genuine issued cookies stay in process memory and never in evidence.
use super::*;

#[path = "native_group_navigation_owner_interruption.rs"]
mod mounted_owner_interruption;
use axum::http::{HeaderValue, header};
use futures::FutureExt;
use std::{
    panic::AssertUnwindSafe,
    sync::{Arc, Mutex},
    time::Duration,
};

pub(super) type CapturedCookie = Arc<Mutex<Option<HeaderValue>>>;
pub(super) const DRIVER_SHA256: &str =
    "2f45b8e0b6393cbe82c2e5745bf5489c99b4fca042e50f8de59cb2a2742524c5";

struct Document {
    status: StatusCode,
    headers: axum::http::HeaderMap,
    bytes: Vec<u8>,
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap()
}

fn credential_values(cookie: &HeaderValue) -> Vec<String> {
    cookie
        .to_str()
        .unwrap()
        .split(';')
        .filter_map(|part| {
            let (name, value) = part.trim().split_once('=')?;
            name.starts_with("__Host-console_account_")
                .then(|| value.to_owned())
        })
        .collect()
}

async fn document(
    client: &reqwest::Client,
    address: SocketAddr,
    method: &str,
    path: &str,
    cookie: Option<&HeaderValue>,
) -> Document {
    let mut request = client.request(method.parse().unwrap(), format!("http://{address}{path}"));
    if let Some(cookie) = cookie {
        request = request.header(header::COOKIE, cookie.clone());
    }
    let response = request
        .send()
        .await
        .expect("owned real HTTP listener request");
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response
        .bytes()
        .await
        .expect("owned bounded HTTP document")
        .to_vec();
    assert!(
        bytes.len() <= 256 * 1024,
        "HTTP document exceeded test bound"
    );
    Document {
        status,
        headers,
        bytes,
    }
}

fn nondisclosing(response: &Document, secrets: &[String], group_private: bool) {
    let html = std::str::from_utf8(&response.bytes).unwrap();
    assert!(
        !response.headers.contains_key(header::SET_COOKIE),
        "probe minted a cookie"
    );
    for secret in secrets {
        assert!(
            !html.contains(secret),
            "interrupted/denied document disclosed object or credential"
        );
    }
    for marker in [
        "csrf_proof",
        "<form",
        "data-group-process-state",
        "data-group-process-terminal-code",
        "data-group-process-outcome=\"committed\"",
    ] {
        assert!(
            !html.contains(marker),
            "interrupted/denied document contains business action/result"
        );
    }
    assert_eq!(
        response.headers.get(header::CACHE_CONTROL).unwrap(),
        "no-store"
    );
    if group_private {
        for (name, value) in [
            (header::CONTENT_TYPE, "text/html; charset=utf-8"),
            (header::PRAGMA, "no-cache"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
            (header::VARY, "Authorization, Cookie, Origin"),
            (header::REFERRER_POLICY, "same-origin"),
            (
                header::CONTENT_SECURITY_POLICY,
                "default-src 'self'; script-src 'none'; style-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'",
            ),
        ] {
            let values: Vec<_> = response.headers.get_all(name).iter().collect();
            assert!(
                values.len() == 1 && values[0].as_bytes() == value.as_bytes(),
                "native Group private header changed or duplicated"
            );
        }
    }
}

// Anonymous Account retains its legitimate passkey login form. It must not
// disclose credentials/identity or carry business forms/projections.
fn account_signin_nondisclosing(response: &Document, secrets: &[String]) {
    let html = std::str::from_utf8(&response.bytes).unwrap();
    assert!(html.contains("data-account-state=\"anonymous\"") && html.contains("로그인"));
    assert_eq!(html.matches("data-native-action=\"login\"").count(), 1);
    assert_eq!(
        html.matches("<form").count(),
        1,
        "SignIn must retain only its login form"
    );
    assert!(
        !response.headers.contains_key(header::SET_COOKIE),
        "SignIn probe minted a cookie"
    );
    for secret in secrets {
        assert!(
            !secret.is_empty(),
            "empty nondisclosure secret invalidates the oracle"
        );
        assert!(
            !html.contains(secret),
            "Account SignIn disclosed a supplied credential/object/identity"
        );
        assert!(
            response.headers.values().all(|value| !value
                .as_bytes()
                .windows(secret.len())
                .any(|part| part == secret.as_bytes())),
            "Account SignIn header disclosed a supplied secret"
        );
    }
    for marker in [
        "data-account-state=\"active\"",
        "data-account-id",
        "data-company-enrollment",
        "data-native-action=\"logout\"",
        "/companies/",
        "/groups/",
        "그룹 신원 확인",
        "csrf_proof",
        "data-group-process-state",
        "data-group-process-terminal-code",
        "data-group-process-outcome=\"committed\"",
    ] {
        assert!(
            !html.contains(marker),
            "Account SignIn disclosed a business identity/action/result"
        );
    }
    for (name, value) in [
        (header::CONTENT_TYPE, "text/html; charset=utf-8"),
        (header::CACHE_CONTROL, "no-store"),
        (header::PRAGMA, "no-cache"),
        (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        (header::VARY, "Authorization, Cookie, Origin"),
        (header::REFERRER_POLICY, "no-referrer"),
        (
            header::CONTENT_SECURITY_POLICY,
            "default-src 'self'; script-src 'self'; style-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'",
        ),
    ] {
        let values: Vec<_> = response.headers.get_all(name).iter().collect();
        assert!(
            values.len() == 1 && values[0].as_bytes() == value.as_bytes(),
            "native Account SignIn private header changed or duplicated"
        );
    }
}

async fn no_effects(pool: &PgPool, before: &BTreeMap<String, String>) {
    assert!(
        *before == all_rows(pool).await,
        "probe changed complete public-table census"
    );
}

async fn auth_controls(
    pool: &PgPool,
    address: SocketAddr,
    group: Uuid,
    cookie: Option<&HeaderValue>,
    secrets: &[String],
) {
    let before = all_rows(pool).await;
    let http = client();
    let mut supplied_secrets = secrets.to_vec();
    if let Some(cookie) = cookie {
        supplied_secrets.extend(credential_values(cookie));
    }
    let account = document(&http, address, "GET", "/account", cookie).await;
    assert_eq!(
        account.status,
        StatusCode::OK,
        "missing/invalid/revoked Account must retain SignIn"
    );
    account_signin_nondisclosing(&account, &supplied_secrets);
    no_effects(pool, &before).await;
    for suffix in ["", "/processes/new"] {
        let path = format!("/groups/{group}/identity{suffix}");
        let response = document(&http, address, "GET", &path, cookie).await;
        assert_eq!(
            response.status,
            StatusCode::UNAUTHORIZED,
            "ordinary Auth refusal must precede custody interruption"
        );
        nondisclosing(&response, &supplied_secrets, true);
        no_effects(pool, &before).await;
    }
}

async fn account_lock_history(
    pool: &PgPool,
    address: SocketAddr,
    cookie: &HeaderValue,
    company: Uuid,
    positive: bool,
) -> Value {
    use sqlx::Connection as _;
    let before = all_rows(pool).await;
    // All fallible connection creation precedes the blocker and spawned request.
    // These are owned direct sockets, never pooled/background rollback actors.
    let mut observer = tokio::time::timeout(
        Duration::from_secs(5),
        sqlx::PgConnection::connect_with(pool.connect_options().as_ref()),
    )
    .await
    .expect("direct observer connection deadline")
    .unwrap();
    let mut blocker = tokio::time::timeout(
        Duration::from_secs(5),
        sqlx::PgConnection::connect_with(pool.connect_options().as_ref()),
    )
    .await
    .expect("direct Company blocker connection deadline")
    .unwrap();
    let blocker_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut blocker)
        .await
        .unwrap();
    let observer_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut observer)
        .await
        .unwrap();
    let mut request: Option<tokio::task::JoinHandle<Document>> = None;
    let mut joined = false;
    let mut waited = None;
    let mut lock_retained = false;
    let observation_deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    let outcome=tokio::time::timeout_at(observation_deadline,AssertUnwindSafe(async {
        sqlx::raw_sql("BEGIN ISOLATION LEVEL READ COMMITTED; SET LOCAL lock_timeout='1s'; SET LOCAL statement_timeout='5s'")
            .execute(&mut blocker).await.unwrap();
        let locked:Uuid=sqlx::query_scalar("SELECT org_id FROM public.company_authority_heads WHERE org_id=$1 FOR UPDATE")
            .bind(company).fetch_one(&mut blocker).await.unwrap();
        assert_eq!(locked,company,"actual UI-created Company head witness");
        let http=client();let credential=cookie.clone();
        request=Some(tokio::spawn(async move {document(&http,address,"GET","/account",Some(&credential)).await}));
        loop {
            waited=sqlx::query_scalar("SELECT pid FROM pg_catalog.pg_stat_activity WHERE datname=current_database() AND usename='console_rt' AND wait_event_type='Lock' AND $1=ANY(pg_catalog.pg_blocking_pids(pid)) AND strpos(query,'identity_company_projection_v2')>0 ORDER BY pid LIMIT 1")
                .bind(blocker_pid).fetch_optional(&mut observer).await.unwrap();
            if waited.is_some() || request.as_ref().unwrap().is_finished() {break;}
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        if positive {
            // Prompt release after observing the real projection wait, inside
            // the same absolute observation deadline as polling and joining.
            sqlx::raw_sql("ROLLBACK").execute(&mut blocker).await.unwrap();
        }
        let response=request.as_mut().unwrap().await;
        joined=true;
        let response=response.expect("owned Account HTTP request task");
        if !positive {
            lock_retained=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_locks WHERE pid=$1 AND locktype='transactionid' AND mode='ExclusiveLock' AND granted)")
                .bind(blocker_pid).fetch_one(&mut observer).await.unwrap();
        }
        response
    }).catch_unwind()).await;

    // Rollback, forced cancellation, task join, direct socket shutdown and the
    // entire backend/lock/row census share one separate five-second deadline.
    // Record every result and report any failure only after owning cleanup.
    let cleanup_deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    let rollback = tokio::time::timeout_at(
        cleanup_deadline,
        sqlx::raw_sql("ROLLBACK").execute(&mut blocker),
    )
    .await;
    let mut forced_cancel = None;
    let mut join_result = None;
    if !joined && let Some(task) = request.as_mut() {
        if let Some(waiter) = waited {
            forced_cancel = Some(
                tokio::time::timeout_at(
                    cleanup_deadline,
                    sqlx::query_scalar::<_, bool>("SELECT pg_catalog.pg_cancel_backend($1)")
                        .bind(waiter)
                        .fetch_one(&mut observer),
                )
                .await,
            );
        }
        task.abort();
        join_result = Some(tokio::time::timeout_at(cleanup_deadline, task).await);
    }
    let blocker_close = tokio::time::timeout_at(cleanup_deadline, blocker.close()).await;
    let census=tokio::time::timeout_at(cleanup_deadline,AssertUnwindSafe(async {
        loop {
            let (blocker_backends,blocker_locks,waiter_locks,waiter_waits):(i64,i64,i64,i64)=sqlx::query_as(
                "SELECT (SELECT count(*) FROM pg_catalog.pg_stat_activity WHERE pid=$1), \
                 (SELECT count(*) FROM pg_catalog.pg_locks WHERE pid=$1), \
                 (SELECT count(*) FROM pg_catalog.pg_locks WHERE pid=$2 AND locktype IN ('relation','transactionid','tuple')), \
                 (SELECT count(*) FROM pg_catalog.pg_stat_activity WHERE pid=$2 AND wait_event_type='Lock')")
                .bind(blocker_pid).bind(waited.unwrap_or(-1)).fetch_one(&mut observer).await.unwrap();
            if (blocker_backends,blocker_locks,waiter_locks,waiter_waits)==(0,0,0,0) {break;}
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        no_effects(pool,&before).await;
    }).catch_unwind()).await;
    // close() owns its socket. Timing out drops that owning future/socket;
    // independent pool readback then confirms the observer backend disappeared.
    let observer_close = tokio::time::timeout_at(cleanup_deadline, observer.close()).await;
    let observer_absent = tokio::time::timeout_at(cleanup_deadline, async {
        loop {
            let present: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_stat_activity WHERE pid IN ($1,$2))",
            )
            .bind(observer_pid)
            .bind(blocker_pid)
            .fetch_one(pool)
            .await
            .unwrap();
            if !present {
                return;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await;
    rollback
        .expect("Company blocker rollback cleanup deadline")
        .expect("Company blocker rollback failed");
    blocker_close
        .expect("Company blocker direct close cleanup deadline")
        .expect("Company blocker direct close failed");
    observer_close
        .expect("direct observer close cleanup deadline")
        .expect("direct observer close failed");
    observer_absent.expect("actual direct blocker/observer backend absence deadline");
    if let Some(cancel) = forced_cancel {
        cancel
            .expect("forced waiter cancellation cleanup deadline")
            .expect("forced waiter cancellation transport failed");
    }
    if let Some(join) = join_result {
        match join.expect("owned Account HTTP request join cleanup deadline") {
            Ok(_) => {}
            Err(error) if error.is_cancelled() => {}
            Err(_) => panic!("owned Account HTTP request task panicked during forced cleanup"),
        }
    }
    match census {
        Ok(Ok(())) => {}
        Ok(Err(panic)) => std::panic::resume_unwind(panic),
        Err(_) => panic!(
            "actual blocker/backend/waiter/complete-row cleanup exceeded its five-second budget"
        ),
    }
    let response = match outcome {
        Ok(Ok(value)) => value,
        Ok(Err(panic)) => std::panic::resume_unwind(panic),
        Err(_) => panic!(
            "Account ordering observation exceeded its five-second budget; owned cleanup completed"
        ),
    };
    let witness = json!({"positive":positive,"actual_projection_wait":waited.is_some(),
        "status":response.status.as_u16(),"blocker_retained_until_held_response":!positive&&lock_retained,
        "complete_rows_unchanged":true,"actual_lock_cleanup":true});
    // Preserve actual late-guard wait evidence before propagating RED.
    eprintln!("GROUP_NAVIGATION_ACCOUNT_LOCK_WITNESS {witness}");
    if positive {
        assert!(
            waited.is_some(),
            "fresh corrected Account did not reach actual Company projection wait"
        );
        assert_eq!(
            response.status,
            StatusCode::OK,
            "fresh corrected positive after actual Company lock release"
        );
    } else {
        assert!(
            waited.is_none(),
            "held Account reached Company owner before custody fence (late-guard mutant)"
        );
        assert!(
            lock_retained,
            "held Account result required Company lock release"
        );
        assert_eq!(
            response.status,
            StatusCode::SERVICE_UNAVAILABLE,
            "held Account native custody interruption"
        );
        let mut secrets = credential_values(cookie);
        secrets.push(company.to_string());
        nondisclosing(&response, &secrets, false);
    }
    witness
}

async fn http_matrix(
    pool: &PgPool,
    address: SocketAddr,
    cookie: &HeaderValue,
    group: Uuid,
    company: Uuid,
    process: Uuid,
    command: Uuid,
) {
    let before = all_rows(pool).await;
    let http = client();
    let base = format!("/groups/{group}/identity");
    let mut secrets = credential_values(cookie);
    secrets.extend([
        group.to_string(),
        company.to_string(),
        process.to_string(),
        command.to_string(),
        "브라우저로 만든 연결 회사".to_owned(),
    ]);
    for path in [
        "/account".to_owned(),
        base.clone(),
        format!("{base}/processes/new"),
        format!("{base}/processes/{process}/replace"),
        format!("{base}/processes/{process}/suspend"),
        format!("{base}/requests/{command}"),
        format!("{base}/requests/{command}/retry"),
    ] {
        let response = document(&http, address, "GET", &path, Some(cookie)).await;
        assert_eq!(
            response.status,
            StatusCode::SERVICE_UNAVAILABLE,
            "held authenticated read/proof must fence"
        );
        nondisclosing(&response, &secrets, path != "/account");
        no_effects(pool, &before).await;
    }
    for (method, path, status, allow, empty) in [
        (
            "HEAD",
            base.clone(),
            StatusCode::METHOD_NOT_ALLOWED,
            Some("GET"),
            true,
        ),
        (
            "PUT",
            base.clone(),
            StatusCode::METHOD_NOT_ALLOWED,
            Some("GET"),
            false,
        ),
        (
            "GET",
            format!("{base}/processes"),
            StatusCode::METHOD_NOT_ALLOWED,
            Some("POST"),
            false,
        ),
        (
            "GET",
            format!("{base}/unsupported"),
            StatusCode::NOT_FOUND,
            None,
            false,
        ),
    ] {
        let response = document(&http, address, method, &path, Some(cookie)).await;
        assert_eq!(
            response.status, status,
            "structure/method authority precedes custody"
        );
        assert_eq!(
            response
                .headers
                .get(header::ALLOW)
                .map(|v| v.to_str().unwrap()),
            allow
        );
        if empty {
            assert!(
                response.bytes.is_empty(),
                "matched Group HEAD must have zero bytes"
            );
        }
        if allow.is_some() {
            nondisclosing(&response, &secrets, true);
        }
        no_effects(pool, &before).await;
    }
}

async fn ack(input: &mut tokio::process::ChildStdin, event: &Value, witness: Value) {
    let mut bytes =
        serde_json::to_vec(&json!({"kind":"CONTINUE","phase":event["phase"],"witness":witness}))
            .unwrap();
    bytes.push(b'\n');
    input.write_all(&bytes).await.unwrap();
    input.flush().await.unwrap();
}

fn event_identity(event: &Value, phase: &str, account: Uuid, company: Uuid, group: Uuid) {
    assert_eq!(event["kind"], "CHECKPOINT");
    assert_eq!(event["phase"], phase);
    assert_eq!(event["account_id"], json!(account));
    assert_eq!(event["org_id"], json!(company));
    assert_eq!(event["group_id"], json!(group));
    assert!(
        event.as_object().unwrap().keys().all(|key| [
            "kind",
            "phase",
            "account_id",
            "org_id",
            "group_id",
            "command_id",
            "process_id",
            "fields",
            "body_sha256",
            "status"
        ]
        .contains(&key.as_str())),
        "browser emitted undeclared or secret fields"
    );
}

#[expect(
    clippy::too_many_arguments,
    reason = "Keep exact UI population, held HTTP state, credential capture and before-form census explicit in the test oracle"
)]
pub(super) async fn observe(
    pool: &PgPool,
    input: &mut tokio::process::ChildStdin,
    events: &mut tokio::io::BufReader<tokio::process::ChildStdout>,
    config: &console_app::AppConfig,
    address: SocketAddr,
    captured: &CapturedCookie,
    account: Uuid,
    company: Uuid,
    group: Uuid,
    before_form: &BTreeMap<String, String>,
    started: OffsetDateTime,
) {
    let ready = browser_owner_event(events).await;
    event_identity(&ready, "GROUP_HELD_FORM_READY", account, company, group);
    let command = policy_uuid(&ready["command_id"]);
    let process = policy_uuid(&ready["process_id"]);
    let credential = captured
        .lock()
        .unwrap()
        .clone()
        .expect("actual Group form request cookie captured in memory");
    let form_rows = all_rows(pool).await;
    let finished: OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
        .fetch_one(pool)
        .await
        .unwrap();
    assert!(
        policy_preflight_effects(before_form, &form_rows, started, finished),
        "actual pre-correction form must mint exactly its legitimate proof admission"
    );
    let named: BTreeMap<String, String> = ready["fields"]
        .as_array()
        .unwrap()
        .iter()
        .map(|pair| {
            let pair = pair.as_array().unwrap();
            assert_eq!(pair.len(), 2);
            (
                pair[0].as_str().unwrap().to_owned(),
                pair[1].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    assert_eq!(
        named.len(),
        ready["fields"].as_array().unwrap().len(),
        "duplicate genuine form field"
    );
    assert_eq!(
        named.keys().map(String::as_str).collect::<BTreeSet<_>>(),
        BTreeSet::from([
            "command_id",
            "expected_group_revision",
            "expected_group_incarnation",
            "expected_group_identity_policy_revision",
            "process_id",
            "expected_prior_process_revision",
            "process_expiry",
            "operator_responsibility",
            "title",
            "method",
            "intended_claimant_matching_procedure",
            "account_possession_procedure",
            "physical_human_evidence_procedure",
            "duplicate_contradictory_claim_procedure",
            "qualification_criteria_instruction",
            "escalation_adjudication_procedure",
            "evidence_minimization_retention_description",
            "recipient_responsibility"
        ])
    );
    let authority = policy_only_row(
        &form_rows,
        "group_authority_heads",
        "group_id",
        &json!(group),
    );
    assert_eq!(named["command_id"], command.to_string());
    assert_eq!(named["process_id"], process.to_string());
    assert_eq!(
        named["expected_group_revision"],
        authority["revision"].as_u64().unwrap().to_string()
    );
    assert_eq!(
        named["expected_group_incarnation"],
        authority["incarnation"].as_str().unwrap()
    );
    assert_eq!(named["expected_group_identity_policy_revision"], "0");
    assert_eq!(named["expected_prior_process_revision"], "0");
    assert_eq!(named["operator_responsibility"], "1");
    assert_eq!(
        named["method"],
        "ATTENDED_ACCOUNT_AND_DOCUMENTARY_REVIEW_V1"
    );
    for table in [
        "native_group_identity_policy_heads_v1",
        "native_group_process_versions_v1",
        "native_group_process_heads_v1",
        "native_group_process_head_revisions_v1",
        "native_group_process_inputs_v1",
        "native_group_process_effects_v1",
        "native_group_process_results_v1",
    ] {
        assert!(
            form_rows[table] == "[]",
            "UI-created Group must have no prepopulated process rows"
        );
    }
    let company_identity = policy_only_row(&form_rows, "organizations", "id", &json!(company));
    let company_name = company_identity["name"].as_str().unwrap();
    assert!(
        !company_name.is_empty(),
        "actual UI Company label must exist"
    );
    let mut secrets = credential_values(&credential);
    secrets.extend([
        account.to_string(),
        company_name.to_owned(),
        company_name
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;"),
        group.to_string(),
        company.to_string(),
        process.to_string(),
        command.to_string(),
    ]);
    let business_secrets = [
        "title",
        "method",
        "intended_claimant_matching_procedure",
        "account_possession_procedure",
        "physical_human_evidence_procedure",
        "duplicate_contradictory_claim_procedure",
        "qualification_criteria_instruction",
        "escalation_adjudication_procedure",
        "evidence_minimization_retention_description",
        "recipient_responsibility",
    ]
    .map(|field| named[field].clone());
    secrets.extend(business_secrets.into_iter().flat_map(|value| {
        let escaped = value
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");
        [value, escaped]
    }));
    let invalid = HeaderValue::from_static("__Host-console_account_session=opaque-invalid-session");
    auth_controls(pool, address, group, None, &secrets).await;
    auth_controls(pool, address, group, Some(&invalid), &secrets).await;
    let http = client();
    assert_eq!(
        document(&http, address, "GET", "/readyz", None)
            .await
            .status,
        StatusCode::OK
    );
    let receipt = {
        let mut correction = tokio::task::JoinSet::new();
        let correction_pool = pool.clone();
        correction.spawn(async move {
            native_policy_startup_tests::correct_native_group_browser_database(&correction_pool)
                .await
        });
        match correction
            .join_next()
            .await
            .expect("owned Group correction task must exist")
        {
            Ok(receipt) => receipt,
            Err(error) if error.is_panic() => std::panic::resume_unwind(error.into_panic()),
            Err(_) => panic!("owned Group correction task unexpectedly cancelled"),
        }
    };
    assert_eq!(receipt["confirmed"], true);
    no_effects(pool, &form_rows).await;
    assert_eq!(
        document(&http, address, "GET", "/readyz", None)
            .await
            .status,
        StatusCode::SERVICE_UNAVAILABLE
    );
    let mut fresh_config = config.clone();
    fresh_config.request_timeout = Duration::from_millis(500);
    let fresh = AppState::from_config(fresh_config)
        .await
        .expect("fresh corrected same-target HTTP state");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let fresh_address = listener.local_addr().unwrap();
    let router = build_router(fresh.clone());
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let mut server = tokio::spawn(async move {
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(async {
            let _ = stopped.await;
        })
        .await
    });
    let outcome=AssertUnwindSafe(async {
        assert_eq!(document(&http,fresh_address,"GET","/readyz",None).await.status,StatusCode::OK);
        let positive=account_lock_history(pool,fresh_address,&credential,company,true).await;
        let held_order=account_lock_history(pool,address,&credential,company,false).await;
        // Actual corrected readmission follows held blocker/request cleanup,
        // before any browser continuation or logout can revoke this credential.
        let recovered=document(&http,fresh_address,"GET","/account",Some(&credential)).await;
        assert_eq!(recovered.status,StatusCode::OK,"fresh corrected Account after held cleanup");
        let recovered_html=std::str::from_utf8(&recovered.bytes).unwrap();
        assert!(recovered_html.contains("data-account-state=\"active\"")
            && recovered_html.contains(&format!("href=\"/companies/{company}\""))
            && recovered_html.contains(&format!("href=\"/groups/{group}/identity\"")),
            "fresh post-held200 omitted actual authorized Company/Group navigation");
        assert!(!recovered.headers.contains_key(header::SET_COOKIE));
        no_effects(pool,&form_rows).await;
        let fresh_after_held=json!({"path":"/account","status":200,"authorized_account":true,
            "after_held_lock_cleanup":true,"complete_rows_unchanged":true});
        // Only the requested recovery locator may disclose these two selectors.
        let requested = [group.to_string(), command.to_string()];
        let owner_secrets = secrets.iter()
            .filter(|secret| !requested.contains(*secret)).cloned().collect::<Vec<_>>();
        let mounted_owner=mounted_owner_interruption::observe(pool,fresh_address,&credential,account,company,group,command,&owner_secrets).await;
        http_matrix(pool,address,&credential,group,company,process,command).await;
        auth_controls(pool,address,group,None,&secrets).await;
        auth_controls(pool,address,group,Some(&invalid),&secrets).await;
        no_effects(pool,&form_rows).await;
        assert_eq!(native_policy_startup_tests::native_group_navigation_browser_metadata(pool).await,receipt["metadata"]);
        ack(input,&ready,json!({"phase":"GROUP_HELD_FORM_READY","confirmed_correction":true,
            "owner_effects_verified":true,"fresh_account_positive":positive,"held_account_ordering":held_order,
            "fresh_account_after_held":fresh_after_held,"mounted_owner_interruption":mounted_owner})).await;
        let submitted=browser_owner_event(events).await;
        event_identity(&submitted,"GROUP_HELD_SUBMITTED",account,company,group);
        assert_eq!(submitted["command_id"],ready["command_id"]);
        assert_eq!(submitted["fields"],ready["fields"]);
        assert_eq!(submitted["status"],503);
        assert!(submitted["body_sha256"].as_str().is_some_and(|v|v.len()==64&&v.bytes().all(|b|b.is_ascii_hexdigit()&&(!b.is_ascii_alphabetic()||b.is_ascii_lowercase()))));
        no_effects(pool,&form_rows).await;
        ack(input,&submitted,json!({"phase":"GROUP_HELD_SUBMITTED","owner_effects_verified":true})).await;
        let reads=browser_owner_event(events).await;
        event_identity(&reads,"GROUP_HELD_READS_FENCED",account,company,group);
        no_effects(pool,&form_rows).await;
        let before_logout=snapshot(pool,account).await;
        let logout_start:OffsetDateTime=sqlx::query_scalar("SELECT clock_timestamp()").fetch_one(pool).await.unwrap();
        ack(input,&reads,json!({"phase":"GROUP_HELD_READS_FENCED","owner_effects_verified":true})).await;
        let logout=browser_owner_event(events).await;
        event_identity(&logout,"GROUP_HELD_LOGGED_OUT",account,company,group);
        let logout_finish:OffsetDateTime=sqlx::query_scalar("SELECT clock_timestamp()").fetch_one(pool).await.unwrap();
        let after_logout=snapshot(pool,account).await;
        assert_logout_transition(&before_logout,&after_logout,account);
        let revoked=all_rows(pool).await;
        let mut remainder=revoked.clone();
        for (table,owner) in [("auth_refresh_token_families","user_id"),("auth_refresh_tokens","user_id"),("account_security_events","account_id")] {
            let unchanged=|raw:&str| -> Vec<String> {
                let rows:Vec<&serde_json::value::RawValue>=serde_json::from_str(raw).unwrap();
                rows.into_iter().filter(|row|serde_json::from_str::<Value>(row.get()).unwrap()[owner]!=json!(account))
                    .map(|row|row.get().to_owned()).collect()
            };
            assert!(unchanged(&form_rows[table])==unchanged(&revoked[table]),"logout changed unrelated Account history");
            remainder.insert(table.into(),form_rows[table].clone());
        }
        assert!(policy_preflight_effects(&form_rows,&remainder,logout_start,logout_finish),
            "logout changed more than exact own revocation history and one CSRF admission");
        auth_controls(pool,address,group,Some(&credential),&secrets).await;
        auth_controls(pool,address,group,None,&secrets).await;
        no_effects(pool,&revoked).await;
        assert_eq!(native_policy_startup_tests::native_group_navigation_browser_metadata(pool).await,receipt["metadata"]);
        ack(input,&logout,json!({"phase":"GROUP_HELD_LOGGED_OUT","owner_effects_verified":true,
            "real_logout_revocation_verified":true,"revoked_session_denied_without_effects":true})).await;
        json!({"correction":receipt,"real_ui_population":true,"held_post_no_effects":true,
            "account_ordering_positive":positive,"account_ordering_held":held_order,
            "fresh_account_after_held":fresh_after_held,"mounted_owner_interruption":mounted_owner,
            "full_public_census_each_probe":true,"metadata_and_ledger_unchanged":true,
            "genuine_ui_logout":true,"missing_invalid_revoked_auth_controls":true,
            "late_guard_mutant_execution_required":true,"production_qualified":false})
    }).catch_unwind().await;
    let _ = stop.send(());
    let clean = matches!(
        tokio::time::timeout(Duration::from_secs(5), &mut server).await,
        Ok(Ok(Ok(())))
    );
    if !clean {
        server.abort();
        let _ = server.await;
    }
    fresh.shutdown_realtime().await;
    assert!(clean, "fresh corrected actual HTTP listener cleanup failed");
    let record = match outcome {
        Ok(value) => value,
        Err(panic) => std::panic::resume_unwind(panic),
    };
    eprintln!("GROUP_NAVIGATION_HELD_BROWSER_OWNER {record}");
}
