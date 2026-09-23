// Included inside company_setup only with test-browser. Existing verified browser helpers remain unchanged.
#[sqlx::test(migrations = false)]
async fn native_company_real_browser_create_reopen_and_workspace(pool: PgPool) {
    company_browser_journey(pool, false).await;
}

#[sqlx::test(migrations = false)]
async fn native_company_real_browser_policy_grant_payroll_reopen_revoke(pool: PgPool) {
    company_browser_journey(pool, true).await;
}

#[sqlx::test(migrations = false)]
async fn native_people_real_browser_register_reopen_and_revoke(pool: PgPool) {
    company_browser_journey_mode(pool, true, true).await;
}

#[path = "native_people_browser.rs"]
mod native_people_browser;

async fn company_browser_journey(pool: PgPool, policy_entry: bool) {
    company_browser_journey_mode(pool, policy_entry, false).await;
}

async fn company_browser_journey_mode(pool: PgPool, policy_entry: bool, people_entry: bool) {
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
    // Source-bound companion helpers are regular files in the same fresh stage.
    let mut policy_helpers = Vec::new();
    if policy_entry {
        for (name, digest) in [
            (
                "policy_journey.cjs",
                "b1735ef0ab51c3a14a0db0c511dabc468516ac2f60bd2e3e31c416a6c1c4ef5a",
            ),
            (
                "recovery_controls.cjs",
                "fe3afc43196cc034d6a5f9bd0b12ad787eaeddf2cb1b1b837b595cb9f49036c6",
            ),
        ] {
            let path = driver.parent().unwrap().join(name);
            assert!(
                std::fs::symlink_metadata(&path)
                    .unwrap()
                    .file_type()
                    .is_file()
            );
            let bytes = std::fs::read(&path).unwrap();
            assert_eq!(hex::encode(Sha256::digest(&bytes)), digest);
            policy_helpers.push((path, bytes));
        }
    }
    if people_entry {
        let path = driver.parent().unwrap().join("people_journey.cjs");
        assert!(
            std::fs::symlink_metadata(&path)
                .unwrap()
                .file_type()
                .is_file()
        );
        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(
            hex::encode(Sha256::digest(&bytes)),
            native_people_browser::DRIVER_SHA256
        );
        policy_helpers.push((path, bytes));
    }
    let output = PathBuf::from(
        std::env::var_os("CONSOLE_COMPANY_BROWSER_OUTPUT").expect("fresh browser output required"),
    );
    assert!(
        output.is_absolute() && !output.exists(),
        "browser output must be a fresh owned directory"
    );
    if policy_entry {
        native_policy_startup_tests::prepare_policy_ready_database(&pool).await;
    } else {
        prepare_ready_database(&pool).await;
    }
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
        .args(if people_entry {
            Some("people-entry")
        } else {
            policy_entry.then_some("policy-entry")
        })
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
        let router = build_router(state).layer(axum::middleware::from_fn(
            |request: axum::extract::Request, next: axum::middleware::Next| async move {
                if request.method() == axum::http::Method::POST
                    && request.uri().path().starts_with("/companies/")
                {
                    let headers = request.headers();
                    eprintln!("native-policy-form-wire: origin_present={} origin_null={} urlencoded={} metadata_document={}",
                        headers.contains_key("origin"),
                        headers.get("origin").is_some_and(|v| v == "null"),
                        headers.get("content-type").is_some_and(|v| v == "application/x-www-form-urlencoded"),
                        headers.get("sec-fetch-mode").is_some_and(|v| v == "navigate")
                            && headers.get("sec-fetch-dest").is_some_and(|v| v == "document"));
                }
                next.run(request).await
            },
        ));
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
            match (std::env::consts::OS, std::env::consts::ARCH) {
                ("macos", "aarch64") =>
                    "a0bfe7b4da4787b66058477d696cd1d09065d25f06a548947722b9af77ee8282",
                ("linux", "x86_64") =>
                    "ded93a9c9a53a1ae040f08124badcca95c938e9d5015ff340c3b5538c41bf39e",
                _ => panic!("unsupported reviewed browser platform"),
            }
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
        if policy_entry {
            let ready = browser_owner_event(&mut events).await;
            assert_eq!(browser_checkpoint(&ready, "POLICY_ENTRY_READY"), account);
            assert!(committed_state == all_rows(&pool).await,
                "current Account/Company/policy navigation changed durable state");
            checkpoint_receipts.push("POLICY_ENTRY_READY");
            let started: OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
                .fetch_one(&pool).await.unwrap();
            browser_owner_continue(&mut input, "POLICY_ENTRY_READY").await;
            let preflight = browser_owner_event(&mut events).await;
            let finished: OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
                .fetch_one(&pool).await.unwrap();
            exact_keys(&preflight, &["kind", "phase", "account_id", "org_id", "group_id", "command_id", "expected_company_epoch"]);
            assert!(preflight["kind"] == "CHECKPOINT" && preflight["phase"] == "POLICY_PREFLIGHT"
                && preflight["account_id"] == json!(account)
                && preflight["org_id"] == json!(result.company)
                && preflight["group_id"] == json!(result.group));
            let command_text = preflight["command_id"].as_str().unwrap();
            let proposed = Uuid::parse_str(command_text).unwrap();
            assert!(!proposed.is_nil() && proposed != command && proposed.to_string() == command_text);
            let epoch: i64 = sqlx::query_scalar("SELECT epoch FROM public.company_authority_heads WHERE org_id=$1")
                .bind(result.company).fetch_one(&pool).await.unwrap();
            assert_eq!(preflight["expected_company_epoch"], json!(epoch.to_string()));
            let after = all_rows(&pool).await;
            if !policy_preflight_effects(&committed_state, &after, started, finished) {
                let changed: Vec<_> = committed_state.keys().chain(after.keys())
                    .filter(|key| committed_state.get(*key) != after.get(*key))
                    .collect::<BTreeSet<_>>().into_iter().collect();
                let counters = |rows: &BTreeMap<String, String>| {
                    serde_json::from_str::<Vec<Value>>(&rows["auth_rate_limit"]).unwrap()
                        .into_iter().filter(|row| row["endpoint"] == "account_csrf"
                            && ["global", "ip:127.0.0.1"].iter().any(|key| row["client_key"] == *key))
                        .map(|row| json!({"global":row["client_key"]=="global",
                            "attempts":row["attempts"].as_i64(),
                            "window_epoch":row["window_start"].as_str().and_then(|s|
                                OffsetDateTime::parse(s, &time::format_description::well_known::Rfc3339).ok())
                                .map(|at| at.unix_timestamp())})).collect::<Vec<_>>()
                };
                eprintln!("preflight census: changed_tables={changed:?}; start={}; finish={}; before={:?}; after={:?}",
                    started.unix_timestamp(), finished.unix_timestamp(), counters(&committed_state), counters(&after));
            }
            assert!(policy_preflight_effects(&committed_state, &after, started, finished),
                "install preflight changed business state or exceeded exact Auth proof limiter effects");
            checkpoint_receipts.push("POLICY_PREFLIGHT");
            browser_owner_continue(&mut input, "POLICY_PREFLIGHT").await;
            let mut prior=after;
            let mut pending:Option<Value>=None;
            let mut last_witness=Value::Null;
            let mut observed=Vec::new();
            let mut phase_started=finished;
            loop {
                let event=browser_owner_event(&mut events).await;
                let phase=event["phase"].as_str().expect("actual policy checkpoint");
                assert_eq!(event["kind"],"CHECKPOINT");assert_eq!(event["account_id"],json!(account));assert_eq!(event["org_id"],json!(result.company));
                let now:OffsetDateTime=sqlx::query_scalar("SELECT clock_timestamp()").fetch_one(&pool).await.unwrap();
                let current=all_rows(&pool).await;
                let mut witness=Value::Null;
                match phase {
                    "POLICY_ACTION_READY"=>{
                        exact_keys(&event,&["kind","phase","account_id","org_id","operation","command_id","expected_company_epoch","fields","assignment_id"]);
                        assert!(pending.is_none());
                        if observed.is_empty() {assert!(prior==current,"first preflight changed after acknowledgement");}
                        else {assert!(policy_preflight_effects(&prior,&current,phase_started,now),"next operation preflight changed more than exact proof limiter buckets");}
                        pending=Some(event.clone());
                    }
                    "CATALOG_INSTALLED"|"GRANT_COMMITTED"|"REVOKE_COMMITTED"=>{
                        witness=policy_committed_witness(&pool,&prior,&current,&event,&pending.take().expect("explicit action preflight"),account,result.company).await;
                        last_witness=witness.clone();observed.push(phase.to_owned());checkpoint_receipts.push(match phase {"CATALOG_INSTALLED"=>"CATALOG_INSTALLED","GRANT_COMMITTED"=>"GRANT_COMMITTED",_=>"REVOKE_COMMITTED"});
                    }
                    "GRANT_REOPENED"|"REVOKE_REOPENED"=>{
                        assert!(prior==current,"receipt reload changed durable rows");assert_eq!(event["command_id"],last_witness["command_id"]);
                        witness=last_witness.clone();witness["phase"]=json!(phase);
                        if phase=="REVOKE_REOPENED" {witness["current_state"]=json!("REVOKED");witness["reload_had_no_effects"]=json!(true);}
                        observed.push(phase.to_owned());checkpoint_receipts.push(if phase=="GRANT_REOPENED"{"GRANT_REOPENED"}else{"REVOKE_REOPENED"});
                    }
                    "PAYROLL_READ"|"PAYROLL_REOPENED"|"PAYROLL_JSON"=>{
                        assert!(policy_read_census(&prior,&current,account,result.company),"payroll read missing its exact audit or changed other rows");
                        observed.push(phase.to_owned());checkpoint_receipts.push(match phase {"PAYROLL_READ"=>"PAYROLL_READ","PAYROLL_REOPENED"=>"PAYROLL_REOPENED",_=>"PAYROLL_JSON"});
                    }
                    "PAYROLL_DENIED"|"PAYROLL_JSON_DENIED"|"ADMIN_REOPENED"=>{
                        assert!(prior==current,"denied read changed durable rows");observed.push(phase.to_owned());checkpoint_receipts.push(match phase {"PAYROLL_DENIED"=>"PAYROLL_DENIED","PAYROLL_JSON_DENIED"=>"PAYROLL_JSON_DENIED",_=>"ADMIN_REOPENED"});
                    }
                    _=>panic!("unexpected policy checkpoint"),
                }
                prior=current;phase_started=now;policy_browser_ack(&mut input,phase,witness).await;
                if phase=="ADMIN_REOPENED" {break;}
            }
            assert!(pending.is_none());
            assert_eq!(observed, ["CATALOG_INSTALLED","GRANT_COMMITTED","GRANT_REOPENED","PAYROLL_READ","PAYROLL_REOPENED","PAYROLL_JSON","REVOKE_COMMITTED","REVOKE_REOPENED","PAYROLL_DENIED","PAYROLL_JSON_DENIED","ADMIN_REOPENED"]);

        }
        if people_entry {
            native_people_browser::observe(&pool, &mut input, &mut events, account, result.company).await;
            checkpoint_receipts.push("PEOPLE_JOURNEY_VERIFIED");
        }
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
    let source_unchanged = std::fs::read(&driver).is_ok_and(|bytes| bytes == driver_bytes)
        && policy_helpers
            .iter()
            .all(|(path, original)| std::fs::read(path).is_ok_and(|bytes| &bytes == original));
    let exit_ok = matches!(child_status, Ok(Ok(status)) if status.success());
    let receipt = json!({"kind":"INDEPENDENT_NATIVE_COMPANY_UI_DATABASE_CHECKPOINTS","policy_entry":policy_entry,"people_entry":people_entry,"checkpoints":checkpoint_receipts,"source_unchanged":source_unchanged,"driver_exit_success":exit_ok,"server_shutdown":server_clean,"browser_pid":owned_browser_pid,"browser_seen_alive":browser_seen_alive,"browser_pid_exit_confirmed":browser_exit_confirmed,"browser_final_alive_observation":browser_exit_observation,"limits":"TEST_ONLY terms publication; synthetic authenticator; actual native enrollment/designation/Company route; does not prove grant/revoke, lost-response, human usability, WCAG or production exposure"});
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

// Stable protected leaf name retained; v2 validates invalid Enter on the active form.
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
            match (std::env::consts::OS, std::env::consts::ARCH) {
                ("macos", "aarch64") =>
                    "a0bfe7b4da4787b66058477d696cd1d09065d25f06a548947722b9af77ee8282",
                ("linux", "x86_64") =>
                    "ded93a9c9a53a1ae040f08124badcca95c938e9d5015ff340c3b5538c41bf39e",
                _ => panic!("unsupported reviewed browser platform"),
            }
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
            "invalid Company input Enter made durable effects"
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
        let browser_result: Value =
            serde_json::from_slice(&std::fs::read(output.join("result.json")).unwrap()).unwrap();
        assert_eq!(
            browser_result["contract_revision"],
            "company-invalid-enter-v2"
        );
        assert_eq!(browser_result["account_id"], json!(account));
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
    let receipt = json!({"kind":"INDEPENDENT_NATIVE_COMPANY_INPUT_VALIDATION_DATABASE_CHECKPOINTS","contract_revision":"company-invalid-enter-v2","checkpoints":checkpoint_receipts,"source_unchanged":source_unchanged,"driver_exit_success":exit_ok,"server_shutdown":server_clean,"browser_pid":owned_browser_pid,"browser_seen_alive":browser_seen_alive,"browser_pid_exit_confirmed":browser_exit_confirmed,"browser_final_alive_observation":browser_exit_observation,"limits":"TEST_ONLY terms publication; synthetic authenticator; actual native enrollment/designation/invalid Company input Enter; complete no-effect census; successful Company commands and recovery retain separate browser leaves; no release acceptance; no human usability/WCAG/production exposure"});
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

// Full browser successor: no business setup writes. Only the original real UI
// and owner designation may create business facts. New relation reads occur
// after a browser-observed successful policy action, never before missing entry.
async fn policy_browser_ack(input: &mut tokio::process::ChildStdin, phase: &str, witness: Value) {
    input
        .write_all(
            format!(
                "{}\n",
                json!({"kind":"CONTINUE","phase":phase,"witness":witness})
            )
            .as_bytes(),
        )
        .await
        .unwrap();
    input.flush().await.unwrap();
}
fn policy_rows(rows: &BTreeMap<String, String>, table: &str) -> Vec<Value> {
    serde_json::from_str(&rows[table]).unwrap()
}
fn policy_only_row(
    rows: &BTreeMap<String, String>,
    table: &str,
    key: &str,
    value: &Value,
) -> Value {
    let selected: Vec<_> = policy_rows(rows, table)
        .into_iter()
        .filter(|r| &r[key] == value)
        .collect();
    assert_eq!(selected.len(), 1, "policy row cardinality");
    selected[0].clone()
}
fn policy_uuid(value: &Value) -> Uuid {
    let text = value.as_str().expect("canonical policy UUID");
    let parsed = Uuid::parse_str(text).unwrap();
    assert!(!parsed.is_nil() && parsed.to_string() == text);
    parsed
}
fn policy_operation(code: i64) -> &'static str {
    match code {
        1 => "InstallPayrollReadCatalogV1",
        2 => "GrantPayrollReadV1",
        3 => "RevokePayrollReadV1",
        _ => panic!("closed policy operation"),
    }
}
fn policy_audit_payload(input: &Value, terminal: Option<&Value>) -> Value {
    let digest = input["input_digest"]
        .as_str()
        .unwrap()
        .strip_prefix("\\x")
        .unwrap();
    assert_eq!(digest.len(), 64);
    assert!(
        digest
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    );
    let mut result = json!({"protocol":"COMPANY_BUSINESS_POLICY_V1","command_id":input["command_id"],
        "intake_receipt_id":input["intake_receipt_id"],"operation":policy_operation(input["operation"].as_i64().unwrap()),
        "input_digest":digest,"session_id":input["accepting_session_id"]});
    if let Some(t) = terminal {
        result["session_id"] = t["execution_session_id"].clone();
        for key in [
            "receipt_id",
            "outcome",
            "result_code",
            "predecessor_receipt_id",
        ] {
            result[key] = t[key].clone();
        }
        for key in ["epoch_before", "epoch_after"] {
            result[key] = json!(t[key].as_i64().unwrap().to_string());
        }
    }
    result
}
fn policy_audit_matches(
    audit: &Value,
    input: &Value,
    terminal: Option<&Value>,
    sql_null: bool,
) -> bool {
    let (action, target, id, time) = match terminal {
        Some(t) => (
            "policy.company_command.complete",
            "native_company_policy_receipts_v1",
            &t["receipt_id"],
            &t["executed_at"],
        ),
        None => (
            "policy.company_command.accept",
            "native_company_policy_inputs_v1",
            &input["intake_receipt_id"],
            &input["accepted_at"],
        ),
    };
    sql_null
        && audit["before_snap"].is_null()
        && audit["actor"] == input["actor_account_id"]
        && audit["org_id"] == input["org_id"]
        && audit["action"] == action
        && audit["target_type"] == target
        && &audit["target_id"] == id
        && &audit["occurred_at"] == time
        && audit["after_snap"] == policy_audit_payload(input, terminal)
}
// All public rows are compared. Historical rows stay byte-identical except the
// single current Company head and, on revoke, its one assignment pointer.
fn policy_effect_census(
    before: &BTreeMap<String, String>,
    after: &BTreeMap<String, String>,
    t: &Value,
) -> bool {
    let check = || -> Option<()> {
        if before.keys().ne(after.keys()) {
            return None;
        }
        let op = t["operation"].as_i64()?;
        let mut counts = BTreeMap::from([
            ("native_company_policy_inputs_v1", 1usize),
            ("native_company_policy_receipts_v1", 1),
            ("audit_events", if op == 1 { 3 } else { 2 }),
        ]);
        match op {
            1 => counts.extend([
                ("ont_object_type_key_revisions", 1),
                ("ont_object_types", 1),
                ("ont_property_defs", 18),
                ("ont_action_types", 1),
                ("ont_builtin_catalog_installs", 1),
                ("native_company_catalog_installs", 1),
                ("native_company_object_refs", 1),
                ("native_company_action_refs", 1),
                ("native_company_property_refs", 18),
            ]),
            2 => counts.extend([
                ("policy_roles", 1),
                ("policy_role_revisions", 1),
                ("policy_capability_clauses", 1),
                ("policy_capability_clause_fields", 18),
                ("user_role_assignments", 1),
                ("policy_assignment_revisions", 1),
            ]),
            3 => {
                counts.insert("policy_assignment_revisions", 1);
            }
            _ => return None,
        }
        for (table, old) in before {
            let current = after.get(table)?;
            if table == "company_authority_heads" || (op == 3 && table == "user_role_assignments") {
                let mut expected: Vec<Value> = serde_json::from_str(old).ok()?;
                let mut changed = 0;
                for row in &mut expected {
                    if table == "company_authority_heads" && row["org_id"] == t["org_id"] {
                        if row["epoch"] != t["epoch_before"]
                            || row["current_policy_receipt_id"] != t["predecessor_receipt_id"]
                        {
                            return None;
                        }
                        row["epoch"] = t["epoch_after"].clone();
                        row["current_policy_receipt_id"] = t["receipt_id"].clone();
                        changed += 1;
                    } else if table == "user_role_assignments" && row["id"] == t["assignment_id"] {
                        if row["native_current_revision"] != t["assignment_revision_before"] {
                            return None;
                        }
                        row["native_current_revision"] = t["assignment_revision_after"].clone();
                        changed += 1;
                    }
                }
                if changed != 1 {
                    return None;
                }
                let canonical = |v: Vec<Value>| {
                    let mut v: Vec<_> = v.iter().map(Value::to_string).collect();
                    v.sort();
                    v
                };
                if canonical(expected) != canonical(serde_json::from_str(current).ok()?) {
                    return None;
                }
            } else if let Some(&count) = counts.get(table.as_str()) {
                if added_rows(old, current)?.len() != count {
                    return None;
                }
            } else if old != current {
                return None;
            }
        }
        if counts.keys().any(|table| !before.contains_key(*table)) {
            return None;
        }
        Some(())
    };
    check().is_some()
}
async fn policy_committed_witness(
    pool: &PgPool,
    before: &BTreeMap<String, String>,
    after: &BTreeMap<String, String>,
    event: &Value,
    request: &Value,
    account: Uuid,
    org: Uuid,
) -> Value {
    let command = policy_uuid(&event["command_id"]);
    assert_eq!(event["company"], json!(org));
    assert_eq!(event["recipient"], json!(account));
    assert_eq!(event["operator"], json!(account));
    assert_eq!(request["command_id"], json!(command));
    let input = policy_only_row(
        after,
        "native_company_policy_inputs_v1",
        "command_id",
        &json!(command),
    );
    let terminal = policy_only_row(
        after,
        "native_company_policy_receipts_v1",
        "command_id",
        &json!(command),
    );
    for row in [&input, &terminal] {
        assert_eq!(row["actor_account_id"], json!(account));
        assert_eq!(row["org_id"], json!(org));
        assert_eq!(row["codec_version"], 1);
    }
    for key in ["operation", "intake_receipt_id", "input_digest"] {
        assert_eq!(input[key], terminal[key]);
    }
    let op = terminal["operation"].as_i64().unwrap();
    assert_eq!(request["operation"], policy_operation(op));
    let epoch = request["expected_company_epoch"]
        .as_str()
        .unwrap()
        .parse::<i64>()
        .unwrap();
    assert_eq!(terminal["epoch_before"], epoch);
    assert_eq!(terminal["epoch_after"], epoch + 1);
    assert_eq!(terminal["outcome"], "COMMITTED");
    assert_eq!(
        terminal["result_code"],
        ["installed", "granted", "revoked"][(op - 1) as usize]
    );
    assert_eq!(
        terminal["catalog_version"],
        "native-payroll-collection-read-v1"
    );
    assert_eq!(
        terminal["manifest_digest"],
        "\\x07781514029d5f8f7e96221d6504387324c8c0f2513ded2b214d0bd683ce3ddd"
    );
    let bytes = hex::decode(
        input["input_bytes"]
            .as_str()
            .unwrap()
            .strip_prefix("\\x")
            .unwrap(),
    )
    .unwrap();
    assert_eq!(bytes.len(), [123, 148, 155][(op - 1) as usize]);
    assert_eq!(
        format!("\\x{}", hex::encode(Sha256::digest(&bytes))),
        input["input_digest"].as_str().unwrap()
    );
    assert_eq!(
        bytes,
        policy_expected_bytes(request, account, org, account),
        "accepted bytes differ from actual browser command"
    );
    assert_policy_catalog_and_grant(pool, after, &terminal, account, org).await;
    assert!(
        policy_effect_census(before, after, &terminal),
        "unexpected policy effect delta or changed historical bytes"
    );
    let audits = added_rows(&before["audit_events"], &after["audit_events"]).unwrap();
    for completed in [false, true] {
        let expected = if completed { Some(&terminal) } else { None };
        let target = if completed {
            &terminal["receipt_id"]
        } else {
            &input["intake_receipt_id"]
        };
        let rows: Vec<_> = audits
            .iter()
            .filter(|a| &a["target_id"] == target)
            .collect();
        assert_eq!(rows.len(), 1);
        let sql_null: bool =
            sqlx::query_scalar("SELECT before_snap IS NULL FROM public.audit_events WHERE id=$1")
                .bind(policy_uuid(&rows[0]["id"]))
                .fetch_one(pool)
                .await
                .unwrap();
        assert!(
            policy_audit_matches(rows[0], &input, expected, sql_null),
            "exact policy audit closure"
        );
    }
    let session: Uuid = sqlx::query_scalar(
        "SELECT id FROM public.auth_refresh_token_families WHERE user_id=$1 AND revoked_at IS NULL",
    )
    .bind(account)
    .fetch_one(pool)
    .await
    .unwrap();
    assert_eq!(input["accepting_session_id"], json!(session));
    assert_eq!(terminal["execution_session_id"], json!(session));
    let mut witness = json!({"phase":event["phase"],"company":org,"recipient":account,"actor_account_id":account,"command_id":command,
        "receipt_id":terminal["receipt_id"],"company_epoch":(epoch+1).to_string(),"owner_effects_verified":true,"unrelated_bytes_preserved":true,"original_discovery_preserved":true});
    let roles: Vec<_> = policy_rows(after, "policy_roles")
        .into_iter()
        .filter(|r| r["org_id"] == json!(org) && r["role_key"] == "native_payroll_collection_read")
        .collect();
    if op == 1 {
        assert!(roles.is_empty());
        witness["business_assignment_count"] = json!(0);
        let ontology: Vec<_> = audits
            .iter()
            .filter(|a| a["action"] == "ontology.object_type.builtin_install")
            .collect();
        assert_eq!(ontology.len(), 1);
        assert_eq!(
            ontology[0]["target_id"],
            terminal["installed_object_type_id"]
        );
        assert_eq!(ontology[0]["actor"], json!(account));
        assert_eq!(ontology[0]["org_id"], json!(org));
    } else {
        assert_eq!(roles.len(), 1);
        assert_eq!(roles[0]["id"], terminal["role_id"]);
        assert_eq!(roles[0]["native_current_revision"], 1);
        let assignment = policy_only_row(
            after,
            "user_role_assignments",
            "id",
            &terminal["assignment_id"],
        );
        assert_eq!(assignment["account_id"], json!(account));
        assert_eq!(assignment["org_id"], json!(org));
        assert_eq!(assignment["role_id"], terminal["role_id"]);
        assert_eq!(
            assignment["native_current_revision"],
            terminal["assignment_revision_after"]
        );
        let revisions: Vec<_> = policy_rows(after, "policy_assignment_revisions")
            .into_iter()
            .filter(|r| {
                r["assignment_id"] == terminal["assignment_id"]
                    && r["revision"] == terminal["assignment_revision_after"]
            })
            .collect();
        assert_eq!(revisions.len(), 1);
        if op == 3 {
            let prior_revisions: Vec<_> = policy_rows(before, "policy_assignment_revisions")
                .into_iter()
                .filter(|r| {
                    r["assignment_id"] == terminal["assignment_id"]
                        && r["revision"] == terminal["assignment_revision_before"]
                })
                .collect();
            assert_eq!(prior_revisions.len(), 1);
            for key in [
                "valid_from",
                "valid_until",
                "origin_account_id",
                "origin_command_id",
                "origin_receipt_id",
                "account_id",
                "role_id",
                "role_revision",
                "ceiling_digest",
            ] {
                assert_eq!(
                    prior_revisions[0][key], revisions[0][key],
                    "revoke changed retained grant fact {key}"
                );
            }
        }
        let revision = &revisions[0];
        assert_eq!(revision["policy_receipt_id"], terminal["receipt_id"]);
        assert_eq!(
            revision["state"],
            if op == 2 { "ACTIVE" } else { "REVOKED" }
        );
        assert_eq!(revision["role_revision"], 1);
        assert_eq!(revision["actor_account_id"], json!(account));
        assert_eq!(revision["session_id"], json!(session));
        assert_eq!(revision["valid_from"], terminal["assignment_valid_from"]);
        assert_eq!(revision["valid_until"], terminal["assignment_valid_until"]);
        witness["assignment_id"] = terminal["assignment_id"].clone();
        witness["assignment_revision"] = json!(
            terminal["assignment_revision_after"]
                .as_i64()
                .unwrap()
                .to_string()
        );
        witness["state"] = revision["state"].clone();
        witness["valid_from"] = revision["valid_from"].clone();
        witness["valid_until"] = revision["valid_until"].clone();
        witness["role_revision"] = json!("1");
        let rr = policy_only_row(
            after,
            "policy_role_revisions",
            "role_id",
            &terminal["role_id"],
        );
        assert!(rr["valid_until"].is_null());
        witness["role_valid_until"] = Value::Null;
    }
    witness
}
fn policy_read_census(
    before: &BTreeMap<String, String>,
    after: &BTreeMap<String, String>,
    account: Uuid,
    org: Uuid,
) -> bool {
    if before.keys().ne(after.keys()) {
        return false;
    }
    before.iter().all(|(table, prior)| {
        if table != "audit_events" {
            return after.get(table) == Some(prior);
        }
        added_rows(prior, &after[table]).is_some_and(|rows| {
            rows.len() == 1
                && rows[0]["action"] == "payroll_run.list_read"
                && rows[0]["actor"] == json!(account)
                && rows[0]["org_id"] == json!(org)
                && rows[0]["target_type"] == "payroll_draft_run"
                && rows[0]["target_id"] == "query"
        })
    })
}
#[test]
fn policy_audit_oracle_rejects_missing_extra_wrong_fields_and_sql_json_null_confusion() {
    let input = json!({"actor_account_id":"00000000-0000-0000-0000-000000000001","org_id":"00000000-0000-0000-0000-000000000002","command_id":"00000000-0000-0000-0000-000000000003","intake_receipt_id":"00000000-0000-0000-0000-000000000004","operation":1,"input_digest":format!("\\x{}","01".repeat(32)),"accepting_session_id":"00000000-0000-0000-0000-000000000005","accepted_at":"2026-09-21T12:00:00Z"});
    let terminal = json!({"execution_session_id":"00000000-0000-0000-0000-000000000006","receipt_id":"00000000-0000-0000-0000-000000000007","outcome":"COMMITTED","result_code":"installed","epoch_before":1,"epoch_after":2,"predecessor_receipt_id":null,"executed_at":"2026-09-21T12:00:01Z"});
    for completed in [false, true] {
        let t = completed.then_some(&terminal);
        let audit = json!({"actor":input["actor_account_id"],"org_id":input["org_id"],"before_snap":null,
            "action":if completed{"policy.company_command.complete"}else{"policy.company_command.accept"},
            "target_type":if completed{"native_company_policy_receipts_v1"}else{"native_company_policy_inputs_v1"},
            "target_id":if completed{&terminal["receipt_id"]}else{&input["intake_receipt_id"]},
            "occurred_at":if completed{&terminal["executed_at"]}else{&input["accepted_at"]},"after_snap":policy_audit_payload(&input,t)});
        assert!(policy_audit_matches(&audit, &input, t, true));
        assert!(!policy_audit_matches(&audit, &input, t, false));
        for key in audit["after_snap"].as_object().unwrap().keys() {
            let mut omitted = audit.clone();
            omitted["after_snap"].as_object_mut().unwrap().remove(key);
            assert!(!policy_audit_matches(&omitted, &input, t, true));
            let mut wrong = audit.clone();
            wrong["after_snap"][key] = json!("incorrect");
            assert!(!policy_audit_matches(&wrong, &input, t, true));
        }
        let mut extra = audit.clone();
        extra["after_snap"]["extra"] = json!(true);
        assert!(!policy_audit_matches(&extra, &input, t, true));
        for key in [
            "actor",
            "org_id",
            "action",
            "target_type",
            "target_id",
            "occurred_at",
        ] {
            let mut wrong = audit.clone();
            wrong[key] = Value::Null;
            assert!(!policy_audit_matches(&wrong, &input, t, true));
        }
    }
}
fn policy_expected_bytes(request: &Value, account: Uuid, org: Uuid, recipient: Uuid) -> Vec<u8> {
    let fields = request["fields"].as_array().unwrap();
    let mut values = BTreeMap::new();
    for pair in fields {
        let pair = pair.as_array().unwrap();
        assert_eq!(pair.len(), 2);
        assert!(
            values
                .insert(pair[0].as_str().unwrap(), pair[1].as_str().unwrap())
                .is_none()
        );
    }
    let mut bytes = b"console.company.business-policy\0\0\x01".to_vec();
    bytes.extend(account.as_bytes());
    bytes.extend(org.as_bytes());
    bytes.extend(policy_uuid(&request["command_id"]).as_bytes());
    assert_eq!(
        values["command_id"],
        request["command_id"].as_str().unwrap()
    );
    assert_eq!(
        values["expected_company_epoch"],
        request["expected_company_epoch"].as_str().unwrap()
    );
    bytes.extend(
        values["expected_company_epoch"]
            .parse::<i64>()
            .unwrap()
            .to_be_bytes(),
    );
    bytes.extend(
        hex::decode("07781514029d5f8f7e96221d6504387324c8c0f2513ded2b214d0bd683ce3ddd").unwrap(),
    );
    match request["operation"].as_str().unwrap() {
        "InstallPayrollReadCatalogV1" => {
            assert_eq!(values.len(), 2);
            bytes.push(1);
        }
        "GrantPayrollReadV1" => {
            assert_eq!(values.len(), 7);
            bytes.push(2);
            assert_eq!(values["recipient_account_id"], recipient.to_string());
            bytes.extend(recipient.as_bytes());
            for key in [
                "expected_role_revision",
                "assignment_id",
                "expected_assignment_revision",
            ] {
                assert_eq!(values[key], "");
            }
            bytes.push(0);
            let text = format!("{}:00+09:00", values["expires_at_local"]);
            let instant =
                OffsetDateTime::parse(&text, &time::format_description::well_known::Rfc3339)
                    .unwrap();
            let micros = i64::try_from(instant.unix_timestamp_nanos() / 1000).unwrap();
            bytes.extend(micros.to_be_bytes());
        }
        "RevokePayrollReadV1" => {
            assert_eq!(values.len(), 4);
            bytes.push(3);
            bytes.extend(
                values["expected_role_revision"]
                    .parse::<i64>()
                    .unwrap()
                    .to_be_bytes(),
            );
            // This selector is supplied independently from the exact native form action.
            bytes.extend(policy_uuid(&request["assignment_id"]).as_bytes());
            bytes.extend(
                values["expected_assignment_revision"]
                    .parse::<i64>()
                    .unwrap()
                    .to_be_bytes(),
            );
        }
        _ => panic!("closed operation"),
    }
    bytes
}
fn policy_fields_match(row: &Value, expected: &Value) -> bool {
    expected
        .as_object()
        .is_some_and(|keys| keys.iter().all(|(key, value)| row.get(key) == Some(value)))
}
async fn assert_policy_catalog_and_grant(
    pool: &PgPool,
    rows: &BTreeMap<String, String>,
    t: &Value,
    account: Uuid,
    org: Uuid,
) {
    let manifest: Value = serde_json::from_str(POLICY_BROWSER_CATALOG).unwrap();
    let logical = &manifest["object_types"][0];
    let version = &manifest["catalog_version"];
    let scoped = |table: &str| -> Vec<Value> {
        policy_rows(rows, table)
            .into_iter()
            .filter(|r| r["org_id"] == json!(org))
            .collect()
    };
    let objects: Vec<_> = scoped("ont_object_types")
        .into_iter()
        .filter(|r| r["stable_key"] == "pay_run")
        .collect();
    assert_eq!(objects.len(), 1);
    let object = &objects[0];
    assert_eq!(object["schema_version"], 1);
    assert_eq!(object["lifecycle_state"], "published");
    assert!(object["created_by"].is_null());
    for key in [
        "stable_key",
        "title",
        "title_property_key",
        "backing_kind",
        "backing_table",
        "primary_key_property",
    ] {
        assert_eq!(object[key], logical[key]);
    }
    let id = policy_uuid(&object["id"]);
    let properties: Vec<_> = scoped("ont_property_defs")
        .into_iter()
        .filter(|r| r["object_type_id"] == json!(id))
        .collect();
    let actions: Vec<_> = scoped("ont_action_types")
        .into_iter()
        .filter(|r| r["object_type_id"] == json!(id))
        .collect();
    assert_eq!(actions.len(), 1);
    assert_eq!(properties.len(), 18);
    for table in ["ont_link_types", "ont_analytics"] {
        assert!(
            scoped(table)
                .iter()
                .all(|r| r["object_type_id"] != json!(id))
        );
    }
    let action = &actions[0];
    assert_eq!(action["object_type_id"], json!(id));
    assert!(policy_fields_match(action, &logical["actions"][0]));
    let mapped = |table: &str| -> Vec<Value> {
        scoped(table)
            .into_iter()
            .filter(|r| &r["catalog_version"] == version)
            .collect()
    };
    let object_refs = mapped("native_company_object_refs");
    let action_refs = mapped("native_company_action_refs");
    let property_refs = mapped("native_company_property_refs");
    assert_eq!(object_refs.len(), 1);
    assert_eq!(action_refs.len(), 1);
    assert_eq!(property_refs.len(), 18);
    assert!(policy_fields_match(
        &object_refs[0],
        &json!({"object_type_id":id,"object_key":"pay_run","schema_revision":1})
    ));
    assert!(policy_fields_match(
        &action_refs[0],
        &json!({"object_type_id":id,"action_type_id":action["id"],"action_key":"payroll.collection.read","registration_revision":1})
    ));
    let mut expected_object = json!({"schema_version":1,"lifecycle_state":"published"});
    for key in [
        "stable_key",
        "title",
        "title_property_key",
        "backing_kind",
        "backing_table",
        "primary_key_property",
    ] {
        expected_object[key] = logical[key].clone();
    }
    assert_eq!(
        object_refs[0]["content_digest"],
        policy_json_digest(pool, &expected_object).await
    );
    assert_eq!(
        action_refs[0]["content_digest"],
        policy_json_digest(pool, &logical["actions"][0]).await
    );
    for r in object_refs
        .iter()
        .chain(action_refs.iter())
        .chain(property_refs.iter())
    {
        assert_eq!(r["manifest_digest"], t["manifest_digest"]);
    }
    for expected in logical["properties"].as_array().unwrap() {
        let found: Vec<_> = properties
            .iter()
            .filter(|r| r["key"] == expected["key"])
            .collect();
        assert_eq!(found.len(), 1);
        let actual = found[0];
        let mut expected = expected.clone();
        expected["type"] = expected["field_type"].clone();
        expected.as_object_mut().unwrap().remove("field_type");
        assert!(policy_fields_match(actual, &expected));
        let content_digest = policy_json_digest(pool, &expected).await;
        let property_key = format!("pay_run.{}", actual["key"].as_str().unwrap());
        let refs: Vec<_> = property_refs
            .iter()
            .filter(|r| r["property_key"] == property_key)
            .collect();
        assert_eq!(refs.len(), 1);
        assert!(policy_fields_match(
            refs[0],
            &json!({"object_type_id":id,"property_id":actual["id"],"schema_revision":1})
        ));
        assert_eq!(refs[0]["content_digest"], content_digest);
    }
    let birth = policy_only_row(rows, "company_enrollment_receipts", "org_id", &json!(org));
    let origin = json!({"origin_account_id":birth["account_id"],"origin_command_id":birth["command_id"],"origin_receipt_id":birth["receipt_id"]});
    let installs = mapped("native_company_catalog_installs");
    let builtin = mapped("ont_builtin_catalog_installs");
    assert_eq!(installs.len(), 1);
    assert_eq!(builtin.len(), 1);
    let install_receipts: Vec<_> = scoped("native_company_policy_receipts_v1")
        .into_iter()
        .filter(|r| r["operation"] == 1 && r["outcome"] == "COMMITTED")
        .collect();
    assert_eq!(install_receipts.len(), 1);
    let installed = &install_receipts[0];
    assert_eq!(installed["installed_object_type_id"], json!(id));
    for row in [&installs[0], &builtin[0], object] {
        assert!(policy_fields_match(row, &origin));
        assert_eq!(row["policy_receipt_id"], installed["receipt_id"]);
    }
    for row in [&installs[0], &builtin[0]] {
        assert_eq!(row["manifest_digest"], t["manifest_digest"]);
    }
    if t["operation"] == 1 {
        return;
    }
    let role = policy_only_row(rows, "policy_roles", "id", &t["role_id"]);
    let revision = policy_only_row(rows, "policy_role_revisions", "role_id", &t["role_id"]);
    let assignment = policy_only_row(rows, "user_role_assignments", "id", &t["assignment_id"]);
    let grants: Vec<_> = scoped("native_company_policy_receipts_v1")
        .into_iter()
        .filter(|r| r["operation"] == 2 && r["outcome"] == "COMMITTED")
        .collect();
    assert_eq!(grants.len(), 1);
    let grant = &grants[0];
    for row in [&role, &revision, &assignment] {
        assert!(policy_fields_match(row, &origin));
        assert_eq!(row["policy_receipt_id"], grant["receipt_id"]);
        assert_eq!(row["subject_protocol"], "NATIVE_ACCOUNT");
    }
    assert!(policy_fields_match(
        &role,
        &json!({"org_id":org,"role_key":"native_payroll_collection_read","native_current_revision":1,"status":"ACTIVE","created_by":null,"updated_by":null,"created_by_account_id":account,"updated_by_account_id":account})
    ));
    assert!(policy_fields_match(
        &revision,
        &json!({"org_id":org,"revision":1,"state":"ACTIVE","valid_until":null,"manifest_digest":t["manifest_digest"],"catalog_version":version,"actor_account_id":account,"session_id":grant["execution_session_id"],"valid_from":grant["executed_at"]})
    ));
    assert!(policy_fields_match(
        &assignment,
        &json!({"org_id":org,"account_id":account,"user_id":null,"assigned_by":null,"assigned_by_account_id":account,"role_id":role["id"]})
    ));
    let clauses: Vec<_> = scoped("policy_capability_clauses")
        .into_iter()
        .filter(|r| r["role_id"] == role["id"])
        .collect();
    assert_eq!(clauses.len(), 1);
    let clause = &clauses[0];
    assert!(policy_fields_match(
        clause,
        &json!({"role_revision":1,"clause_index":1,"effect":"ALLOW","delegable":false,"resource_org_id":org,"action_type_id":action["id"],"action_object_type_id":id,"registration_revision":1,"manifest_digest":t["manifest_digest"],"valid_from":grant["executed_at"],"valid_until":null})
    ));
    let fields: Vec<_> = scoped("policy_capability_clause_fields")
        .into_iter()
        .filter(|r| r["role_id"] == role["id"])
        .collect();
    assert_eq!(fields.len(), 18);
    for reference in property_refs {
        let found: Vec<_> = fields
            .iter()
            .filter(|f| f["property_id"] == reference["property_id"])
            .collect();
        assert_eq!(found.len(), 1);
        assert!(policy_fields_match(
            found[0],
            &json!({"role_revision":1,"clause_index":1,"object_type_id":id,"schema_revision":1,"manifest_digest":t["manifest_digest"],"content_digest":reference["content_digest"]})
        ));
    }
    let mut capability_fields:Vec<Value>=fields.iter().map(|f|json!({"org_id":org.to_string(),"object_type_id":id.to_string(),"property_id":f["property_id"],"schema_revision":"1"})).collect();
    capability_fields.sort_by_key(|f| f["property_id"].as_str().unwrap().to_owned());
    let canonical_time: String = sqlx::query_scalar(
        "SELECT to_char($1::timestamptz AT TIME ZONE 'UTC','YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"')",
    )
    .bind(
        OffsetDateTime::parse(
            grant["executed_at"].as_str().unwrap(),
            &time::format_description::well_known::Rfc3339,
        )
        .unwrap(),
    )
    .fetch_one(pool)
    .await
    .unwrap();
    let capability = json!({"kind":"COMPANY_CAPABILITY_CLAUSE_V1","action":{"org_id":org.to_string(),"object_type_id":id.to_string(),"action_type_id":action["id"],"registration_revision":"1","manifest_digest":"07781514029d5f8f7e96221d6504387324c8c0f2513ded2b214d0bd683ce3ddd"},"resource":{"kind":"COMPANY","org_id":org.to_string()},"fields":capability_fields,"valid_from":canonical_time,"valid_until":null,"delegable":false});
    let clause_digest = policy_json_digest(pool, &capability).await;
    assert_eq!(clause["clause_digest"], clause_digest);
    let role_digest=policy_json_digest(pool,&json!([{"clause_index":1,"clause_digest":clause_digest.as_str().unwrap().strip_prefix("\\x").unwrap()}])).await;
    assert_eq!(revision["clause_digest"], role_digest);
    for ar in scoped("policy_assignment_revisions")
        .iter()
        .filter(|r| r["assignment_id"] == assignment["id"])
    {
        assert!(policy_fields_match(ar, &origin));
        assert_eq!(ar["ceiling_digest"], revision["clause_digest"]);
    }
}
#[test]
fn policy_participant_content_oracle_rejects_same_cardinality_corruption() {
    let expected = json!({"effect":"ALLOW","delegable":false,"role_revision":1,"object_type_id":"id","manifest_digest":"digest","valid_until":null,"origin_receipt_id":"birth"});
    assert!(policy_fields_match(&expected, &expected));
    for key in expected.as_object().unwrap().keys() {
        let mut corrupt = expected.clone();
        corrupt[key] = json!("changed");
        assert_eq!(
            corrupt.as_object().unwrap().len(),
            expected.as_object().unwrap().len()
        );
        assert!(!policy_fields_match(&corrupt, &expected));
    }
    let logical: Value = serde_json::from_str(POLICY_BROWSER_CATALOG).unwrap();
    for property in logical["object_types"][0]["properties"].as_array().unwrap() {
        assert!(policy_fields_match(property, property));
        for key in property.as_object().unwrap().keys() {
            let mut wrong = property.clone();
            wrong[key] = Value::Null;
            assert!(!policy_fields_match(&wrong, property));
        }
    }
}

const POLICY_BROWSER_CATALOG: &str = r###"{"object_types": [{"links": [], "title": "급여 실행", "actions": [{"edits": [], "title": "회사 급여 목록 전체 항목 보기", "dispatch": "projected_usecase", "stable_key": "collection_read", "side_effects": [], "params_schema": {"type": "object", "required": [], "properties": {"limit": {"type": "integer", "maximum": 9223372036854775807, "minimum": -9223372036854775808}, "offset": {"type": "integer", "maximum": 9223372036854775807, "minimum": -9223372036854775808}}, "additionalProperties": false}, "control_points": ["authority"], "dispatch_target": "payroll.collection.read", "submission_criteria": []}], "analytics": [], "properties": [{"key": "id", "title": "식별자", "config": {}, "required": true, "field_type": "reference", "backing_column": "id", "in_property_policy": true}, {"key": "period_start", "title": "기간 시작", "config": {}, "required": true, "field_type": "date", "backing_column": "period_start", "in_property_policy": true}, {"key": "period_end", "title": "기간 종료", "config": {}, "required": true, "field_type": "date", "backing_column": "period_end", "in_property_policy": true}, {"key": "source_label", "title": "자료 이름", "config": {}, "required": true, "field_type": "text", "backing_column": "source_label", "in_property_policy": true}, {"key": "status", "title": "상태", "config": {}, "required": true, "field_type": "text", "backing_column": "status", "in_property_policy": true}, {"key": "calculation_enabled", "title": "계산 허용", "config": {}, "required": true, "field_type": "boolean", "backing_column": "calculation_enabled", "in_property_policy": true}, {"key": "created_by", "title": "작성자", "config": {"nullable": true}, "required": false, "field_type": "reference", "backing_column": "created_by", "in_property_policy": true}, {"key": "approved_by", "title": "승인자", "config": {"nullable": true}, "required": false, "field_type": "reference", "backing_column": "approved_by", "in_property_policy": true}, {"key": "approved_at", "title": "승인 시각", "config": {"nullable": true, "timezone": "UTC", "precision": "microsecond"}, "required": false, "field_type": "timestamp", "backing_column": "approved_at", "in_property_policy": true}, {"key": "close_receipt", "title": "근태 마감 증빙 전체", "config": {"nullable": true}, "required": false, "field_type": "json", "backing_column": "close_receipt", "in_property_policy": true}, {"key": "submitted_by", "title": "제출자", "config": {"nullable": true}, "required": false, "field_type": "reference", "backing_column": "submitted_by", "in_property_policy": true}, {"key": "submitted_at", "title": "제출 시각", "config": {"nullable": true, "timezone": "UTC", "precision": "microsecond"}, "required": false, "field_type": "timestamp", "backing_column": "submitted_at", "in_property_policy": true}, {"key": "decided_by", "title": "결정자", "config": {"nullable": true}, "required": false, "field_type": "reference", "backing_column": "decided_by", "in_property_policy": true}, {"key": "decided_at", "title": "결정 시각", "config": {"nullable": true, "timezone": "UTC", "precision": "microsecond"}, "required": false, "field_type": "timestamp", "backing_column": "decided_at", "in_property_policy": true}, {"key": "decision_reason", "title": "결정 사유", "config": {"nullable": true}, "required": false, "field_type": "text", "backing_column": "decision_reason", "in_property_policy": true}, {"key": "approval_ref", "title": "승인 참조", "config": {"nullable": true}, "required": false, "field_type": "reference", "backing_column": "approval_ref", "in_property_policy": true}, {"key": "created_at", "title": "생성 시각", "config": {"timezone": "UTC", "precision": "microsecond"}, "required": true, "field_type": "timestamp", "backing_column": "created_at", "in_property_policy": true}, {"key": "updated_at", "title": "수정 시각", "config": {"timezone": "UTC", "precision": "microsecond"}, "required": true, "field_type": "timestamp", "backing_column": "updated_at", "in_property_policy": true}], "stable_key": "pay_run", "backing_kind": "projected", "backing_table": "payroll_draft_runs", "title_property_key": "source_label", "primary_key_property": "id"}], "catalog_version": "native-payroll-collection-read-v1"}"###;

async fn policy_json_digest(pool: &PgPool, value: &Value) -> Value {
    let digest: String =
        sqlx::query_scalar("SELECT encode(sha256(convert_to($1::jsonb::text,'UTF8')),'hex')")
            .bind(value)
            .fetch_one(pool)
            .await
            .unwrap();
    json!(format!("\\x{digest}"))
}

const POLICY_BROWSER_CODEC_VECTORS: &str = r###"[
  {
    "name": "install",
    "actor": "11111111-1111-4111-8111-111111111111",
    "company": "33333333-3333-4333-8333-333333333333",
    "command": "22222222-2222-4222-8222-222222222222",
    "expected_company_epoch": 7,
    "operation": 1,
    "hex": "636f6e736f6c652e636f6d70616e792e627573696e6573732d706f6c696379000001111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222000000000000000707781514029d5f8f7e96221d6504387324c8c0f2513ded2b214d0bd683ce3ddd01",
    "length": 123,
    "sha256": "c55d20ad085288e191676623f4052064d78e8620f063aa553a7f24ac20d6339e"
  },
  {
    "name": "initial_grant",
    "actor": "11111111-1111-4111-8111-111111111111",
    "company": "33333333-3333-4333-8333-333333333333",
    "command": "22222222-2222-4222-8222-222222222222",
    "expected_company_epoch": 7,
    "operation": 2,
    "recipient": "44444444-4444-4444-8444-444444444444",
    "witness": null,
    "expires_at_unix_microseconds": 1790607600000000,
    "hex": "636f6e736f6c652e636f6d70616e792e627573696e6573732d706f6c696379000001111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222000000000000000707781514029d5f8f7e96221d6504387324c8c0f2513ded2b214d0bd683ce3ddd02444444444444444484444444444444440000065c8c51ee1c00",
    "length": 148,
    "sha256": "5d9a48f0f4c0d87d15aaa9f352e69770df182368d29f9a21ea256fd5e031a746"
  },
  {
    "name": "regrant",
    "actor": "11111111-1111-4111-8111-111111111111",
    "company": "33333333-3333-4333-8333-333333333333",
    "command": "22222222-2222-4222-8222-222222222222",
    "expected_company_epoch": 7,
    "operation": 2,
    "recipient": "44444444-4444-4444-8444-444444444444",
    "role_revision": 1,
    "assignment": "55555555-5555-4555-8555-555555555555",
    "assignment_revision": 9,
    "expires_at_unix_microseconds": 1790607600000000,
    "hex": "636f6e736f6c652e636f6d70616e792e627573696e6573732d706f6c696379000001111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222000000000000000707781514029d5f8f7e96221d6504387324c8c0f2513ded2b214d0bd683ce3ddd024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "length": 180,
    "sha256": "a21cd1af531e185a1dbb3a689cd6576739da4b599ff0cb8cdb886f02144902b7"
  },
  {
    "name": "revoke",
    "actor": "11111111-1111-4111-8111-111111111111",
    "company": "33333333-3333-4333-8333-333333333333",
    "command": "22222222-2222-4222-8222-222222222222",
    "expected_company_epoch": 7,
    "operation": 3,
    "role_revision": 1,
    "assignment": "55555555-5555-4555-8555-555555555555",
    "assignment_revision": 9,
    "hex": "636f6e736f6c652e636f6d70616e792e627573696e6573732d706f6c696379000001111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222000000000000000707781514029d5f8f7e96221d6504387324c8c0f2513ded2b214d0bd683ce3ddd030000000000000001555555555555455585555555555555550000000000000009",
    "length": 155,
    "sha256": "7c9447335b8a1388b0c162a900102fc0e0eb994e35b85de439d5d2d1a2c2cb9f"
  }
]
"###;
#[test]
fn policy_browser_wire_expectation_matches_independent_frozen_vectors() {
    let vectors: Vec<Value> = serde_json::from_str(POLICY_BROWSER_CODEC_VECTORS).unwrap();
    for name in ["install", "initial_grant", "revoke"] {
        let v = vectors.iter().find(|v| v["name"] == name).unwrap();
        let mut fields = vec![
            json!(["command_id", v["command"]]),
            json!([
                "expected_company_epoch",
                v["expected_company_epoch"].as_i64().unwrap().to_string()
            ]),
        ];
        if name == "initial_grant" {
            fields.extend([
                json!(["recipient_account_id", v["recipient"]]),
                json!(["expected_role_revision", ""]),
                json!(["assignment_id", ""]),
                json!(["expected_assignment_revision", ""]),
                json!(["expires_at_local", "2026-09-29T00:00"]),
            ]);
        }
        if name == "revoke" {
            fields.extend([
                json!([
                    "expected_role_revision",
                    v["role_revision"].as_i64().unwrap().to_string()
                ]),
                json!([
                    "expected_assignment_revision",
                    v["assignment_revision"].as_i64().unwrap().to_string()
                ]),
            ]);
        }
        let request = json!({"command_id":v["command"],"expected_company_epoch":v["expected_company_epoch"].as_i64().unwrap().to_string(),"operation":policy_operation(v["operation"].as_i64().unwrap()),"assignment_id":v["assignment"],"fields":fields});
        let actor = policy_uuid(&v["actor"]);
        let recipient = if name == "initial_grant" {
            policy_uuid(&v["recipient"])
        } else {
            actor
        };
        let bytes = policy_expected_bytes(&request, actor, policy_uuid(&v["company"]), recipient);
        assert_eq!(hex::encode(&bytes), v["hex"]);
        assert_eq!(hex::encode(Sha256::digest(&bytes)), v["sha256"]);
        for at in [34, 50, 66, 82, 90, 122] {
            let mut corrupt = bytes.clone();
            corrupt[at] ^= 1;
            assert_ne!(
                corrupt,
                policy_expected_bytes(&request, actor, policy_uuid(&v["company"]), recipient)
            );
        }
    }
}
