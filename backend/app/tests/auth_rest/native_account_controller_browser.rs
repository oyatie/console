//! Additive browser controller parity; unchanged real A/O handoff census precedes every control.
//! Reuses real enrollment/designation owners; never populates Accounts or Company by API/SQL.
use super::*;
use futures::FutureExt;
use std::process::Stdio;

const DRIVER_SHA256: &str = "ee8e799bd55024e2274bde98195d6a03ab564cc5b063cb70d8f445b593a3c92d";
const HELPER_SHA256: &str = "68d5e45686d19088cb3690b1703dc8ad83f8ea1a340935ca0ff1f473c72f0b78";

async fn event(reader: &mut tokio::io::BufReader<tokio::process::ChildStdout>) -> Value {
    let value = browser_owner_event(reader).await;
    if value["kind"] == "RESULT" && value["status"] != "BROWSER_LEAF_PASSED" {
        exact_keys(
            &value,
            &[
                "kind",
                "status",
                "result_path",
                "failure_stage",
                "failure_code",
            ],
        );
        let code = match value["failure_code"].as_str() {
            Some("ACCOUNT_REFERENCE_MISSING") => "ACCOUNT_REFERENCE_MISSING",
            Some("SEPARATE_ADMIN_CONTROL_MISSING") => "SEPARATE_ADMIN_CONTROL_MISSING",
            Some("REACT_ACCOUNT_MOUNT_MISSING") => "REACT_ACCOUNT_MOUNT_MISSING",
            Some("HANDOFF_RECEIPT") => "HANDOFF_RECEIPT",
            Some("HANDOFF_DISCLOSURE") => "HANDOFF_DISCLOSURE",
            Some("ACCOUNT_CONTROLLER_INVARIANT") => "ACCOUNT_CONTROLLER_INVARIANT",
            _ => "NON_ADMITTING_BROWSER_FAILURE",
        };
        let stage = match value["failure_stage"].as_str() {
            Some("own_account_reference") => "own_account_reference",
            Some("separate_administrator") => "separate_administrator",
            Some("company_submit") => "company_submit",
            Some("handoff_reopen") => "handoff_reopen",
            Some(value) if value.starts_with("controller/") => "controller",
            _ => "prerequisite_or_other_stage",
        };
        panic!("ACCOUNT_CONTROLLER_BROWSER_FAILED stage={stage} code={code}");
    }
    value
}

// Failure-only diagnostic; it cannot replace the original outcome or any admission oracle.
fn producer_failure_diagnostic(output: &std::path::Path) -> Value {
    use rustix::fs::{Mode, OFlags};
    use std::io::Read as _;
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Diagnostic {
        kind: String,
        original_stage: Option<String>,
        original_code: Option<String>,
        finalization_stage: Option<String>,
        finalization_code: Option<String>,
    }
    let unavailable = |status| json!({"status":status,"record":null});
    let file = output.join("producer-diagnostic.json");
    match std::fs::symlink_metadata(&file) {
        Ok(metadata) if !metadata.is_file() => return unavailable("nonregular"),
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return unavailable("missing");
        }
        Err(_) => return unavailable("unavailable"),
    }
    let fd = match rustix::fs::open(
        &file,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
        Mode::empty(),
    ) {
        Ok(fd) => fd,
        Err(rustix::io::Errno::LOOP) => return unavailable("nonregular"),
        Err(_) => return unavailable("unavailable"),
    };
    let file = std::fs::File::from(fd);
    match file.metadata() {
        Ok(metadata) if !metadata.is_file() => return unavailable("nonregular"),
        Ok(metadata) if metadata.len() > 1024 => return unavailable("oversized"),
        Ok(_) => {}
        Err(_) => return unavailable("unavailable"),
    }
    let mut bytes = Vec::new();
    if file.take(1025).read_to_end(&mut bytes).is_err() {
        return unavailable("unavailable");
    }
    if bytes.len() > 1024 {
        return unavailable("oversized");
    }
    match serde_json::from_slice::<Value>(&bytes) {
        Ok(Value::Object(object))
            if object.len() == 5
                && [
                    "kind",
                    "original_stage",
                    "original_code",
                    "finalization_stage",
                    "finalization_code",
                ]
                .iter()
                .all(|key| object.contains_key(*key)) => {}
        Ok(_) => return unavailable("invalid"),
        Err(_) => return unavailable("unparseable"),
    }
    let diagnostic: Diagnostic = match serde_json::from_slice(&bytes) {
        Ok(value) => value,
        Err(_) => return unavailable("unparseable"),
    };
    let stages = [
        "prerequisites",
        "owner_start",
        "browser_launch",
        "a_registration",
        "o_registration",
        "o_logout",
        "o_login",
        "healthy_handoff_entry",
        "own_account_reference",
        "separate_administrator",
        "company_submit",
        "handoff_reopen",
        "owner_cancel",
        "controller/native-first",
        "controller/react-first",
        "controller/keyboard-before-react",
        "controller/autofill-before-react",
        "controller/eventless-value-before-react",
        "controller/ime-before-react",
        "controller/preclaim-focus",
        "controller/partial-bind-rollback",
        "controller/capability-denied",
        "controller/freeze-at-csrf",
        "cleanup",
        "evidence",
        "result_write",
        "result_emit",
        "OTHER",
    ];
    let codes = [
        "TIMEOUT",
        "OWNER_PROTOCOL",
        "OWNER_EOF",
        "OWNER_REFUSED",
        "PREREQUISITE",
        "TLS_RELAY_FAILED",
        "EXTERNAL_REQUEST",
        "ACCOUNT_SSR",
        "ACCOUNT_REFERENCE_MISSING",
        "ACCOUNT_REFERENCE_INVALID",
        "SEPARATE_ADMIN_CONTROL_MISSING",
        "ADMINISTRATOR_INPUT_INVALID",
        "REACT_ACCOUNT_MOUNT_MISSING",
        "HANDOFF_RECEIPT",
        "HANDOFF_DISCLOSURE",
        "HANDOFF_EVIDENCE",
        "CONTROLLER_EVIDENCE",
        "ACCOUNT_CONTROLLER_INVARIANT",
        "BROWSER_TIMEOUT",
        "INVALID_JSON",
        "BROWSER_TYPE_ERROR",
        "RESPONSE_BODY_UNAVAILABLE",
        "BROWSER_CONTEXT_DESTROYED",
        "UNCLASSIFIED_FAILURE",
        "OTHER",
    ];
    let valid_pair = |stage: &Option<String>, code: &Option<String>| match (stage, code) {
        (None, None) => true,
        (Some(stage), Some(code)) => {
            stages.contains(&stage.as_str()) && codes.contains(&code.as_str())
        }
        _ => false,
    };
    if diagnostic.kind != "ACCOUNT_CONTROLLER_PRODUCER_FAILURE_V1"
        || !valid_pair(&diagnostic.original_stage, &diagnostic.original_code)
        || !valid_pair(
            &diagnostic.finalization_stage,
            &diagnostic.finalization_code,
        )
        || (diagnostic.original_stage.is_none() && diagnostic.finalization_stage.is_none())
    {
        return unavailable("invalid");
    }
    json!({"status":"present","record":{
        "kind":diagnostic.kind,"original_stage":diagnostic.original_stage,
        "original_code":diagnostic.original_code,"finalization_stage":diagnostic.finalization_stage,
        "finalization_code":diagnostic.finalization_code}})
}

#[sqlx::test(migrations = false)]
async fn real_browser_account_controller_promotion_and_frozen_administrator(pool: PgPool) {
    let original = PathBuf::from(
        std::env::var_os("CONSOLE_COMPANY_BROWSER_DRIVER")
            .expect("reviewed Company browser stage required"),
    );
    assert!(original.is_absolute());
    assert_eq!(
        std::env::var("CONSOLE_COMPANY_BROWSER_SHA256").unwrap(),
        "10edae6d0f4d6f66ccf0cbe2d17eaba8cbbc977ede47b4b5fc94b3a201f7fcc2"
    );
    let stage = original.parent().unwrap();
    let mut sources = Vec::new();
    for (name, digest) in [
        (
            "company.cjs",
            "10edae6d0f4d6f66ccf0cbe2d17eaba8cbbc977ede47b4b5fc94b3a201f7fcc2",
        ),
        (
            "native_header.cjs",
            "915f37ffb3b7580202158ca23425e047e4587278aaaa5725687f9a6eb71a34c4",
        ),
        ("account-company-handoff.cjs", DRIVER_SHA256),
        ("account_company_handoff.cjs", HELPER_SHA256),
        (
            "account-controller.cjs",
            "c91cbff5a96176fb67f34753483a52c4960c9552fd6677d476906ef98d9aa67a",
        ),
        (
            "account_controller_controls.cjs",
            "540382452084a0e40205c78ba502bf9360a2ec647a0aeb30425e20bbc4d1b8dc",
        ),
        (
            "account_controller_evidence.cjs",
            "50e37ea2b53fdf8aa6b4f649e43ed0500946cbbcffaff139914d516ff0485341",
        ),
    ] {
        let file = stage.join(name);
        assert!(
            std::fs::symlink_metadata(&file)
                .unwrap()
                .file_type()
                .is_file()
        );
        let bytes = std::fs::read(&file).unwrap();
        assert_eq!(
            hex::encode(Sha256::digest(&bytes)),
            digest,
            "reviewed handoff source differs"
        );
        sources.push((file, bytes));
    }
    let driver = stage.join("account-controller.cjs");
    let original_output = PathBuf::from(
        std::env::var_os("CONSOLE_COMPANY_BROWSER_OUTPUT")
            .expect("fresh Company browser output required"),
    );
    assert!(original_output.is_absolute() && !original_output.exists());
    let output = original_output.with_file_name("account-controller");
    assert!(!output.exists(), "handoff browser output must be fresh");
    prepare_ready_database(&pool).await;
    let artifacts = Artifacts::new();
    let key = SigningKey::random(&mut OsRng);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let baseline = native_extension_rows(&pool).await;
    let business_baseline = browser_business_rows(&pool).await;
    let mut child = tokio::process::Command::new("node")
        .arg(&driver)
        .arg(address.port().to_string())
        .arg(&output)
        .env_remove("DEBUG")
        .env_remove("PWDEBUG")
        .env_remove("NODE_DEBUG")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .expect("handoff browser producer prerequisite");
    let mut input = child.stdin.take().unwrap();
    let mut events = tokio::io::BufReader::new(child.stdout.take().unwrap());
    let mut server = None;
    let mut shutdown = None;
    let mut state_to_close = None;
    let mut owned_pid = None;
    let mut seen_alive = false;
    let mut checkpoints = Vec::new();
    let outcome = Box::pin(
        std::panic::AssertUnwindSafe(async {
            let ready = event(&mut events).await;
            exact_keys(
                &ready,
                &[
                    "kind",
                    "origin",
                    "rp_id",
                    "tls_spki_sha256",
                    "upstream_port",
                ],
            );
            assert!(ready["kind"] == "READY" && ready["rp_id"] == "localhost");
            assert_eq!(
                ready["upstream_port"].as_u64(),
                Some(u64::from(address.port()))
            );
            let origin = ready["origin"].as_str().unwrap();
            let parsed = url::Url::parse(origin).unwrap();
            assert!(
                parsed.scheme() == "https"
                    && parsed.host_str() == Some("localhost")
                    && parsed.port().is_some()
                    && parsed.username().is_empty()
                    && parsed.password().is_none()
                    && parsed.path() == "/"
                    && parsed.query().is_none()
                    && parsed.fragment().is_none()
            );
            assert_eq!(
                base64::engine::general_purpose::STANDARD
                    .decode(ready["tls_spki_sha256"].as_str().unwrap())
                    .unwrap()
                    .len(),
                32
            );
            let mut config = account_browser_config(&pool, artifacts.root.clone(), &key);
            let auth = config.auth_rest.as_mut().unwrap();
            auth.rp_id = "localhost".to_owned();
            auth.rp_origin = origin.to_owned();
            auth.cookie_secure = true;
            let state = AppState::from_config(config)
                .await
                .expect("healthy actual Account/Company owner required");
            state_to_close = Some(state.clone());
            let router = build_router(state);
            let (stop, stopped) = tokio::sync::oneshot::channel();
            shutdown = Some(stop);
            server = Some(tokio::spawn(async move {
                axum::serve(
                    listener,
                    router.into_make_service_with_connect_info::<SocketAddr>(),
                )
                .with_graceful_shutdown(async {
                    let _ = stopped.await;
                })
                .await
            }));
            assert!(native_extension_rows_equal(
                &baseline,
                &native_extension_rows(&pool).await
            ));
            assert!(browser_business_rows_equal(
                &business_baseline,
                &browser_business_rows(&pool).await
            ));
            input.write_all(b"{\"kind\":\"START\"}\n").await.unwrap();
            input.flush().await.unwrap();
            let owned = event(&mut events).await;
            exact_keys(&owned, &["kind", "pid", "executable_sha256"]);
            assert!(owned["kind"] == "BROWSER_OWNED");
            assert_eq!(
                owned["executable_sha256"],
                match (std::env::consts::OS, std::env::consts::ARCH) {
                    ("macos", "aarch64") =>
                        "a0bfe7b4da4787b66058477d696cd1d09065d25f06a548947722b9af77ee8282",
                    ("linux", "x86_64") =>
                        "ded93a9c9a53a1ae040f08124badcca95c938e9d5015ff340c3b5538c41bf39e",
                    _ => panic!("unsupported reviewed browser platform"),
                }
            );
            let pid = u32::try_from(owned["pid"].as_u64().unwrap()).unwrap();
            assert!(pid > 1);
            owned_pid = Some(pid);
            seen_alive = browser_pid_alive(pid) == Some(true);
            assert!(seen_alive, "owned browser PID must be observed alive");
            let mut previous = baseline.clone();
            let mut accounts = Vec::new();
            for phase in ["A_ENROLLED", "O_ENROLLED"] {
                let account = browser_checkpoint(&event(&mut events).await, phase);
                assert!(
                    !accounts.contains(&account),
                    "independent browser Accounts required"
                );
                let ceremonies: Vec<Uuid> =
                    sqlx::query_scalar("SELECT id FROM auth_webauthn_ceremonies WHERE user_id=$1")
                        .bind(account)
                        .fetch_all(&pool)
                        .await
                        .unwrap();
                assert_eq!(ceremonies.len(), 1);
                assert_account_committed_ids(&pool, account, ceremonies[0]).await;
                let enrolled = native_extension_rows(&pool).await;
                assert!(
                    browser_global_effects(
                        &previous,
                        &enrolled,
                        account,
                        BrowserCheckpointPhase::Enrolled,
                        origin
                    ),
                    "real browser enrollment omitted effects or changed retained history"
                );
                assert!(browser_business_rows_equal(
                    &business_baseline,
                    &browser_business_rows(&pool).await
                ));
                accounts.push(account);
                if phase == "A_ENROLLED" {
                    previous = enrolled;
                }
                checkpoints.push(phase);
                browser_owner_continue(&mut input, phase).await;
            }
            let administrator = accounts[0];
            let operator = accounts[1];
            for (phase, state) in [
                ("O_LOGGED_OUT", BrowserCheckpointPhase::LoggedOut),
                ("O_LOGGED_IN", BrowserCheckpointPhase::LoggedIn),
            ] {
                assert_eq!(
                    browser_checkpoint(&event(&mut events).await, phase),
                    operator
                );
                assert!(
                    browser_global_effects(
                        &previous,
                        &native_extension_rows(&pool).await,
                        operator,
                        state,
                        origin
                    ),
                    "real operator logout/login omitted effects or changed recipient history"
                );
                assert!(browser_business_rows_equal(
                    &business_baseline,
                    &browser_business_rows(&pool).await
                ));
                if phase == "O_LOGGED_IN" {
                    let startup = startup(&pool).await;
                    let designation = designation(&pool, operator).await;
                    let designated = designate(&startup, &designation).await.unwrap();
                    assert!(!designated.0.is_nil() && designated.1 == 1 && !designated.2);
                    startup.close().await;
                }
                checkpoints.push(phase);
                browser_owner_continue(&mut input, phase).await;
            }
            let before_company = all_rows(&pool).await;
            assert_eq!(
                browser_checkpoint(&event(&mut events).await, "HEALTHY_HANDOFF_ENTRY"),
                operator
            );
            assert!(
                before_company == all_rows(&pool).await,
                "healthy entry reads changed durable state"
            );
            checkpoints.push("HEALTHY_HANDOFF_ENTRY");
            browser_owner_continue(&mut input, "HEALTHY_HANDOFF_ENTRY").await;
            let created = event(&mut events).await;
            exact_keys(
                &created,
                &[
                    "kind",
                    "phase",
                    "account_id",
                    "administrator_account_id",
                    "command_id",
                    "org_id",
                    "group_id",
                    "receipt_id",
                ],
            );
            assert!(
                created["kind"] == "CHECKPOINT"
                    && created["phase"] == "COMPANY_COMMITTED"
                    && created["account_id"] == json!(operator)
                    && created["administrator_account_id"] == json!(administrator)
            );
            let id = |key: &str| {
                let text = created[key].as_str().unwrap();
                let id = Uuid::parse_str(text).unwrap();
                assert!(!id.is_nil() && id.to_string() == text);
                id
            };
            let command = id("command_id");
            let result = Committed {
                command,
                receipt: id("receipt_id"),
                company: id("org_id"),
                group: id("group_id"),
                administrator,
                result_path: format!("/account/companies/requests/{command}"),
            };
            let mut submitted = enrollment(command, administrator);
            submitted["slug"] = json!(format!("handoff-{}", operator.simple()));
            submitted["name"] = json!("브라우저로 연결한 독립 관리 회사 <연구 & 본사>");
            durable(&pool, &result, &submitted, operator).await;
            let committed = all_rows(&pool).await;
            // Preserve the original Company birth census and complete reopening history.
            for (table, count) in [
                ("organizations", 1),
                ("groups", 1),
                ("group_memberships", 1),
                ("company_actors", 1),
                ("company_enrollment_requests", 1),
                ("company_enrollment_receipts", 1),
            ] {
                assert_eq!(
                    added_rows(&before_company[table], &committed[table])
                        .expect("prior owner history changed")
                        .len(),
                    count,
                    "Company birth census differs"
                );
            }
            for table in [
                "accounts",
                "account_terms_acceptances",
                "auth_refresh_tokens",
                "auth_refresh_token_families",
                "auth_webauthn_credentials",
                "users",
            ] {
                assert!(
                    before_company[table] == committed[table],
                    "Company changed Account/credential/legacy history"
                );
            }
            checkpoints.push("COMPANY_COMMITTED");
            browser_owner_continue(&mut input, "COMPANY_COMMITTED").await;
            assert_eq!(
                browser_checkpoint(&event(&mut events).await, "HANDOFF_REOPENED"),
                operator
            );
            assert!(
                committed == all_rows(&pool).await,
                "reopening/denied reads changed any public table bytes"
            );
            checkpoints.push("HANDOFF_REOPENED");
            browser_owner_continue(&mut input, "HANDOFF_REOPENED").await;
            verify_controller_events(
                &pool,
                &mut input,
                &mut events,
                operator,
                administrator,
                result.company,
                result.command,
                &committed,
                &mut checkpoints,
            )
            .await;
            let final_event = event(&mut events).await;
            exact_keys(
                &final_event,
                &[
                    "kind",
                    "status",
                    "result_path",
                    "failure_stage",
                    "failure_code",
                ],
            );
            assert!(
                final_event["kind"] == "RESULT"
                    && final_event["status"] == "BROWSER_LEAF_PASSED"
                    && final_event["failure_stage"].is_null()
                    && final_event["failure_code"].is_null()
            );
            assert_eq!(
                final_event["result_path"].as_str(),
                output.join("result.json").to_str()
            );
        })
        .catch_unwind(),
    )
    .await;
    drop(input);
    let child_status = tokio::time::timeout(std::time::Duration::from_secs(40), child.wait()).await;
    if child_status.is_err() {
        let _ = child.kill().await;
    }
    if let Some(stop) = shutdown {
        let _ = stop.send(());
    }
    let mut server_clean = true;
    if let Some(mut task) = server {
        match tokio::time::timeout(std::time::Duration::from_secs(5), &mut task).await {
            Ok(Ok(Ok(()))) => {}
            _ => {
                server_clean = false;
                task.abort();
                let _ = task.await;
            }
        }
    }
    if let Some(state) = state_to_close {
        state.shutdown_realtime().await;
    }
    let exited = owned_pid.and_then(browser_pid_alive);
    let browser_clean = seen_alive && exited == Some(false);
    let source_unchanged = sources
        .iter()
        .all(|(file, bytes)| std::fs::read(file).is_ok_and(|current| current == *bytes));
    let exit_ok = matches!(&child_status, Ok(Ok(status)) if status.success());
    let (wait_state, exit_code, signal) = match &child_status {
        Ok(Ok(status)) => {
            #[cfg(unix)]
            let signal = std::os::unix::process::ExitStatusExt::signal(status);
            #[cfg(not(unix))]
            let signal: Option<i32> = None;
            ("exited", status.code(), signal)
        }
        Ok(Err(_)) => ("wait_failed", None, None),
        Err(_) => ("timed_out", None, None),
    };
    let producer_diagnostic = producer_failure_diagnostic(&output);
    let receipt = json!({"kind":"INDEPENDENT_REACT_ACCOUNT_CONTROLLER_DB_CHECKPOINTS_V1",
        "checkpoints":checkpoints,"source_unchanged":source_unchanged,"driver_exit_success":exit_ok,
        "driver_wait_state":wait_state,"driver_exit_code":exit_code,"driver_signal":signal,
        "producer_failure_diagnostic":producer_diagnostic,
        "server_shutdown":server_clean,"browser_pid":owned_pid,"browser_seen_alive":seen_alive,
        "browser_pid_exit_confirmed":browser_clean,"browser_final_alive_observation":exited,
        "limits":"TEST_ONLY terms and virtual authenticator; browser owner handoff only; complete original census retained; no native-device, full parity, usability, release or production claim"});
    if output.is_dir() {
        std::fs::write(
            output.join("owner-receipt.json"),
            serde_json::to_vec_pretty(&receipt).unwrap(),
        )
        .unwrap();
    }
    assert!(
        source_unchanged && server_clean && (owned_pid.is_none() || browser_clean),
        "source or owned-process cleanup failed"
    );
    if let Err(panic) = outcome {
        std::panic::resume_unwind(panic);
    }
    assert!(
        exit_ok && browser_clean,
        "handoff browser did not exit successfully with its exact PID absent"
    );
}

include!("native_account_controller_events.rs");

#[path = "company_setup/native_birth_oracles.rs"]
mod birth_oracles;
use birth_oracles::{
    assert_native_catalog_content, identity_digests_match, identity_graph_matches,
    native_catalog_attribution_with_schema,
};

include!("native_account_controller_effects.rs");
