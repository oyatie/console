// Included inside company_setup only with test-browser. Existing verified browser helpers remain unchanged.
#[sqlx::test(migrations = false)]
async fn native_company_real_browser_create_reopen_and_workspace(pool: PgPool) {
    use futures::FutureExt;
    use std::process::Stdio;
    // Explicit local evidence prerequisites, never a silently skipped browser test.
    let driver = PathBuf::from(
        std::env::var_os("CONSOLE_COMPANY_BROWSER_DRIVER")
            .expect("reviewed browser driver required"),
    );
    let expected =
        std::env::var("CONSOLE_COMPANY_BROWSER_SHA256").expect("reviewed driver SHA256 required");
    assert!(
        driver.is_absolute()
            && !std::fs::symlink_metadata(&driver)
                .unwrap()
                .file_type()
                .is_symlink()
    );
    let driver_bytes = std::fs::read(&driver).unwrap();
    assert_eq!(
        hex::encode(Sha256::digest(&driver_bytes)),
        expected,
        "reviewed browser source differs"
    );
    let output = PathBuf::from(
        std::env::var_os("CONSOLE_COMPANY_BROWSER_OUTPUT").expect("fresh browser output required"),
    );
    assert!(
        output.is_absolute() && !output.exists(),
        "browser output must be a fresh owned directory"
    );
    prepare_http_database(&pool).await;
    seed_terms(&pool).await;
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
        .expect("browser driver launch prerequisite");
    let mut input = child.stdin.take().unwrap();
    let mut events = tokio::io::BufReader::new(child.stdout.take().unwrap());
    let mut server = None;
    let mut shutdown = None;
    let mut state_to_close = None;
    let mut checkpoint_receipts = Vec::new();
    let mut owned_browser_pid = None;
    let mut browser_seen_alive = false;
    let outcome = std::panic::AssertUnwindSafe(async {
        let ready = browser_owner_event(&mut events).await;
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
            .expect("real native browser app prerequisite");
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
        assert!(
            native_extension_rows_equal(&baseline, &native_extension_rows(&pool).await),
            "serving startup changed Account data"
        );
        assert!(
            browser_business_rows_equal(&business_baseline, &browser_business_rows(&pool).await),
            "startup changed Company identity state"
        );
        input.write_all(b"{\"kind\":\"START\"}\n").await.unwrap();
        input.flush().await.unwrap();
        let owned = browser_owner_event(&mut events).await;
        exact_keys(&owned, &["kind", "pid", "executable_sha256"]);
        assert!(
            owned["kind"] == "BROWSER_OWNED" && owned["pid"].as_u64().is_some_and(|pid| pid > 1)
        );
        assert_eq!(
            owned["executable_sha256"],
            "7687bff7cb2db075f250e6d5848bbc8838cac3802ac3952a899c574f8eccab45"
        );

        let pid = u32::try_from(owned["pid"].as_u64().unwrap()).unwrap();
        owned_browser_pid = Some(pid);
        browser_seen_alive = browser_pid_alive(pid) == Some(true);
        assert!(browser_seen_alive, "owned browser PID not observed alive");

        let enrolled_event = browser_owner_event(&mut events).await;
        let account = browser_checkpoint(&enrolled_event, "ENROLLED");
        let ceremonies: Vec<Uuid> =
            sqlx::query_scalar("SELECT id FROM auth_webauthn_ceremonies WHERE user_id=$1")
                .bind(account)
                .fetch_all(&pool)
                .await
                .unwrap();
        assert_eq!(ceremonies.len(), 1);
        assert_account_committed_ids(&pool, account, ceremonies[0]).await;
        let enrolled_global = native_extension_rows(&pool).await;
        assert!(
            browser_global_effects(
                &baseline,
                &enrolled_global,
                account,
                BrowserCheckpointPhase::Enrolled,
                origin
            ),
            "enrollment changed unowned global state or omitted an effect"
        );
        let enrolled = snapshot(&pool, account).await;
        assert_eq!(enrolled["tokens"].as_array().unwrap().len(), 1);
        assert!(
            enrolled["families"][0]["revoked_at"].is_null()
                && enrolled["tokens"][0]["revoked_at"].is_null()
        );
        assert!(
            browser_business_rows_equal(&business_baseline, &browser_business_rows(&pool).await),
            "ENROLLED changed unrelated legacy users, Companies or Groups"
        );
        checkpoint_receipts.push("ENROLLED");

        let startup = startup(&pool).await;
        let designation = designation(&pool, account).await;
        let designated = designate(&startup, &designation).await.unwrap();
        assert!(!designated.0.is_nil() && designated.1 == 1 && !designated.2);
        startup.close().await;
        let before_company = all_rows(&pool).await;
        // The actual browser is still paused; eligibility is now current via the
        // same restricted deployment designation owner used by operators.
        browser_owner_continue(&mut input, "DESIGNATED").await;
        let created = browser_owner_event(&mut events).await;
        exact_keys(
            &created,
            &[
                "kind",
                "phase",
                "account_id",
                "command_id",
                "org_id",
                "group_id",
                "receipt_id",
            ],
        );
        assert!(
            created["kind"] == "CHECKPOINT"
                && created["phase"] == "COMPANY_COMMITTED"
                && created["account_id"] == json!(account)
        );
        let id = |key: &str| {
            let text = created[key].as_str().expect("actual browser UUID");
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
            administrator: account,
            result_path: format!("/account/companies/requests/{command}"),
        };
        let mut submitted = enrollment(command, account);
        submitted["slug"] = json!(format!("browser-{}", account.simple()));
        submitted["name"] = json!("브라우저로 만든 연결 회사 <연구 & 본사>");
        durable(&pool, &result, &submitted, account).await;
        let committed_state = all_rows(&pool).await;
        for (table, expected) in [
            ("organizations", 1),
            ("groups", 1),
            ("group_memberships", 1),
            ("company_actors", 1),
            ("company_enrollment_requests", 1),
            ("company_enrollment_receipts", 1),
        ] {
            let added = added_rows(&before_company[table], &committed_state[table])
                .expect("prior owner history preserved");
            assert_eq!(
                added.len(),
                expected,
                "actual browser Company effect cardinality differs"
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
                before_company[table] == committed_state[table],
                "Company browser mutated Account credentials or legacy identities"
            );
        }
        checkpoint_receipts.push("COMPANY_COMMITTED");
        browser_owner_continue(&mut input, "COMPANY_COMMITTED").await;
        let reopened = browser_owner_event(&mut events).await;
        assert_eq!(browser_checkpoint(&reopened, "COMPANY_REOPENED"), account);
        assert!(
            committed_state == all_rows(&pool).await,
            "reload/back/workspace reads created or altered committed Company effects"
        );
        checkpoint_receipts.push("COMPANY_REOPENED");
        browser_owner_continue(&mut input, "COMPANY_REOPENED").await;
        let final_event = browser_owner_event(&mut events).await;
        exact_keys(&final_event, &["kind", "status", "result_path"]);
        assert!(
            final_event["kind"] == "RESULT" && final_event["status"] == "BROWSER_LEAF_PASSED",
            "browser leaf failed; inspect sanitized result"
        );
        assert_eq!(
            final_event["result_path"].as_str(),
            output.join("result.json").to_str()
        );
    })
    .catch_unwind()
    .await;

    // Even failed assertions close stdin so the owner-controlled browser driver
    // cleans up its exact browser/TLS relay before this test propagates failure.
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
    let browser_exit_observation = owned_browser_pid.and_then(browser_pid_alive);
    let browser_exit_confirmed = browser_seen_alive && browser_exit_observation == Some(false);
    let source_unchanged = std::fs::read(&driver).is_ok_and(|bytes| bytes == driver_bytes);
    let exit_ok = matches!(child_status, Ok(Ok(status)) if status.success());
    let receipt = json!({"kind":"INDEPENDENT_NATIVE_COMPANY_UI_DATABASE_CHECKPOINTS","checkpoints":checkpoint_receipts,"source_unchanged":source_unchanged,"driver_exit_success":exit_ok,"server_shutdown":server_clean,"browser_pid":owned_browser_pid,"browser_seen_alive":browser_seen_alive,"browser_pid_exit_confirmed":browser_exit_confirmed,"browser_final_alive_observation":browser_exit_observation,"limits":"TEST_ONLY terms publication; synthetic authenticator; actual native enrollment/designation/Company route; does not prove grant/revoke, lost-response, human usability, WCAG or production exposure"});
    if output.is_dir() {
        std::fs::write(
            output.join("owner-receipt.json"),
            serde_json::to_vec_pretty(&receipt).unwrap(),
        )
        .unwrap();
    }
    assert!(
        source_unchanged && server_clean && (owned_browser_pid.is_none() || browser_exit_confirmed),
        "browser source or owned app/browser PID cleanup failed"
    );
    if let Err(panic) = outcome {
        std::panic::resume_unwind(panic);
    }
    assert!(
        exit_ok && browser_exit_confirmed,
        "browser did not exit successfully with its owned PID absent after all checkpoints"
    );
}

// Included inside company_setup only with test-browser. Existing verified browser helpers remain unchanged.
#[sqlx::test(migrations = false)]
async fn native_company_real_browser_preview_preserves_enter_without_commands(pool: PgPool) {
    use futures::FutureExt;
    use std::process::Stdio;
    // Explicit local evidence prerequisites, never a silently skipped browser test.
    let driver = PathBuf::from(
        std::env::var_os("CONSOLE_COMPANY_PREVIEW_BROWSER_DRIVER")
            .expect("reviewed browser driver required"),
    );
    let expected = std::env::var("CONSOLE_COMPANY_PREVIEW_BROWSER_SHA256")
        .expect("reviewed driver SHA256 required");
    assert!(
        driver.is_absolute()
            && !std::fs::symlink_metadata(&driver)
                .unwrap()
                .file_type()
                .is_symlink()
    );
    let driver_bytes = std::fs::read(&driver).unwrap();
    assert_eq!(
        hex::encode(Sha256::digest(&driver_bytes)),
        expected,
        "reviewed browser source differs"
    );
    let output = PathBuf::from(
        std::env::var_os("CONSOLE_COMPANY_PREVIEW_BROWSER_OUTPUT")
            .expect("fresh browser output required"),
    );
    assert!(
        output.is_absolute() && !output.exists(),
        "browser output must be a fresh owned directory"
    );
    prepare_http_database(&pool).await;
    seed_terms(&pool).await;
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
        .expect("browser driver launch prerequisite");
    let mut input = child.stdin.take().unwrap();
    let mut events = tokio::io::BufReader::new(child.stdout.take().unwrap());
    let mut server = None;
    let mut shutdown = None;
    let mut state_to_close = None;
    let mut checkpoint_receipts = Vec::new();
    let mut owned_browser_pid = None;
    let mut browser_seen_alive = false;
    let outcome = std::panic::AssertUnwindSafe(async {
        let ready = browser_owner_event(&mut events).await;
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
            .expect("real native browser app prerequisite");
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
        assert!(
            native_extension_rows_equal(&baseline, &native_extension_rows(&pool).await),
            "serving startup changed Account data"
        );
        assert!(
            browser_business_rows_equal(&business_baseline, &browser_business_rows(&pool).await),
            "startup changed Company identity state"
        );
        input.write_all(b"{\"kind\":\"START\"}\n").await.unwrap();
        input.flush().await.unwrap();
        let owned = browser_owner_event(&mut events).await;
        exact_keys(&owned, &["kind", "pid", "executable_sha256"]);
        assert!(
            owned["kind"] == "BROWSER_OWNED" && owned["pid"].as_u64().is_some_and(|pid| pid > 1)
        );
        assert_eq!(
            owned["executable_sha256"],
            "7687bff7cb2db075f250e6d5848bbc8838cac3802ac3952a899c574f8eccab45"
        );

        let pid = u32::try_from(owned["pid"].as_u64().unwrap()).unwrap();
        owned_browser_pid = Some(pid);
        browser_seen_alive = browser_pid_alive(pid) == Some(true);
        assert!(browser_seen_alive, "owned browser PID not observed alive");

        let enrolled_event = browser_owner_event(&mut events).await;
        let account = browser_checkpoint(&enrolled_event, "ENROLLED");
        let ceremonies: Vec<Uuid> =
            sqlx::query_scalar("SELECT id FROM auth_webauthn_ceremonies WHERE user_id=$1")
                .bind(account)
                .fetch_all(&pool)
                .await
                .unwrap();
        assert_eq!(ceremonies.len(), 1);
        assert_account_committed_ids(&pool, account, ceremonies[0]).await;
        let enrolled_global = native_extension_rows(&pool).await;
        assert!(
            browser_global_effects(
                &baseline,
                &enrolled_global,
                account,
                BrowserCheckpointPhase::Enrolled,
                origin
            ),
            "enrollment changed unowned global state or omitted an effect"
        );
        let enrolled = snapshot(&pool, account).await;
        assert_eq!(enrolled["tokens"].as_array().unwrap().len(), 1);
        assert!(
            enrolled["families"][0]["revoked_at"].is_null()
                && enrolled["tokens"][0]["revoked_at"].is_null()
        );
        assert!(
            browser_business_rows_equal(&business_baseline, &browser_business_rows(&pool).await),
            "ENROLLED changed unrelated legacy users, Companies or Groups"
        );
        checkpoint_receipts.push("ENROLLED");

        let startup = startup(&pool).await;
        let designation = designation(&pool, account).await;
        let designated = designate(&startup, &designation).await.unwrap();
        assert!(!designated.0.is_nil() && designated.1 == 1 && !designated.2);
        startup.close().await;
        let before_company = all_rows(&pool).await;
        // The actual browser is still paused; eligibility is now current via the
        // same restricted deployment designation owner used by operators.
        browser_owner_continue(&mut input, "DESIGNATED").await;
        let observed = browser_owner_event(&mut events).await;
        assert_eq!(browser_checkpoint(&observed, "PREVIEW_PRESERVED"), account);
        assert!(
            before_company == all_rows(&pool).await,
            "preview typing/Enter made durable effects"
        );
        assert_no_company_identity(&pool, account).await;
        checkpoint_receipts.push("PREVIEW_PRESERVED");
        browser_owner_continue(&mut input, "PREVIEW_PRESERVED").await;
        let final_event = browser_owner_event(&mut events).await;
        exact_keys(&final_event, &["kind", "status", "result_path"]);
        assert!(
            final_event["kind"] == "RESULT" && final_event["status"] == "BROWSER_LEAF_PASSED",
            "browser leaf failed; inspect sanitized result"
        );
        assert_eq!(
            final_event["result_path"].as_str(),
            output.join("result.json").to_str()
        );
    })
    .catch_unwind()
    .await;

    // Even failed assertions close stdin so the owner-controlled browser driver
    // cleans up its exact browser/TLS relay before this test propagates failure.
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
    let browser_exit_observation = owned_browser_pid.and_then(browser_pid_alive);
    let browser_exit_confirmed = browser_seen_alive && browser_exit_observation == Some(false);
    let source_unchanged = std::fs::read(&driver).is_ok_and(|bytes| bytes == driver_bytes);
    let exit_ok = matches!(child_status, Ok(Ok(status)) if status.success());
    let receipt = json!({"kind":"INDEPENDENT_NATIVE_COMPANY_PREVIEW_DATABASE_CHECKPOINTS","checkpoints":checkpoint_receipts,"source_unchanged":source_unchanged,"driver_exit_success":exit_ok,"server_shutdown":server_clean,"browser_pid":owned_browser_pid,"browser_seen_alive":browser_seen_alive,"browser_pid_exit_confirmed":browser_exit_confirmed,"browser_final_alive_observation":browser_exit_observation,"limits":"TEST_ONLY terms publication; synthetic authenticator; actual native enrollment/designation/preview only; no Company command or release acceptance; no human usability/WCAG/production exposure"});
    if output.is_dir() {
        std::fs::write(
            output.join("owner-receipt.json"),
            serde_json::to_vec_pretty(&receipt).unwrap(),
        )
        .unwrap();
    }
    assert!(
        source_unchanged && server_clean && (owned_browser_pid.is_none() || browser_exit_confirmed),
        "browser source or owned app/browser PID cleanup failed"
    );
    if let Err(panic) = outcome {
        std::panic::resume_unwind(panic);
    }
    assert!(
        exit_ok && browser_exit_confirmed,
        "browser did not exit successfully with its owned PID absent after all checkpoints"
    );
}
