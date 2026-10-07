//! Additive browser controller parity; unchanged real A/O handoff census precedes every control.
//! Reuses real enrollment/designation owners; never populates Accounts or Company by API/SQL.
use super::*;
use futures::FutureExt;
use std::process::Stdio;

const DRIVER_SHA256: &str = "c2f175092793d8124112c96886f00a45b147ca3f1d88e95222989101e988e31d";
const HELPER_SHA256: &str = "d48aa90e0c9e8488d347198e2b8222f1c33fd109e47a2f45070e206cc3953ab7";

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
            "d3a0762ddcbe27d056dc517461ff404d3807344982ad8462b4731390de86f835",
        ),
        (
            "account_controller_controls.cjs",
            "32af9d85ac30f68c5f8a4cd97369c55c3811db66dc5f4a350f60b25b739ac25c",
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
    let exit_ok = matches!(child_status, Ok(Ok(status)) if status.success());
    let receipt = json!({"kind":"INDEPENDENT_REACT_ACCOUNT_CONTROLLER_DB_CHECKPOINTS_V1",
        "checkpoints":checkpoints,"source_unchanged":source_unchanged,"driver_exit_success":exit_ok,
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
