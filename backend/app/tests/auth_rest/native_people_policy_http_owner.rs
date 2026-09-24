// Proposal only. Include as child of native_policy_startup_tests after review.
// Uses existing APIs and string routes: missing HTTP entry is runtime RED, not
// a missing Rust symbol. Root compiles/runs; this author executed no product test.
mod native_people_policy_http_owner {
    use super::*;
    use console_identity_application::company_policy::{
        AccountId,
        business::{NativeBusinessOperationV1, PolicyAssignmentExpectationV1},
        people_business::{DirectoryActionV1, NativePeoplePolicyCommandV1},
        workflow::{
            NativePolicyCommand, NativePolicyEffect, NativePolicyOutcome, NativePolicyStatus,
            NativePolicyTerminalView, native_policy_command_status,
        },
    };

    fn field(html: &str, name: &str) -> String {
        let needle = format!("name=\"{name}\"");
        let tags: Vec<_> = html
            .split("<input")
            .skip(1)
            .map(|s| s.split('>').next().unwrap())
            .filter(|s| s.contains(&needle))
            .collect();
        assert_eq!(tags.len(), 1, "required input missing/duplicated: {name}");
        let raw = tags[0]
            .split("value=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap();
        // This oracle captures only canonical UUIDs/revisions/JWTs and empty
        // assignment values. Unexpected HTML entity encoding must not be silently
        // resubmitted as different accepted bytes.
        assert!(!raw.contains('&') && !raw.contains('<'));
        raw.to_owned()
    }
    fn form_body(pairs: &[(String, String)]) -> String {
        fn encode(s: &str) -> String {
            let mut out = String::new();
            for b in s.bytes() {
                if b.is_ascii_alphanumeric() || b"-._~".contains(&b) {
                    out.push(b as char)
                } else {
                    out.push_str(&format!("%{b:02X}"))
                }
            }
            out
        }
        pairs
            .iter()
            .map(|(k, v)| format!("{}={}", encode(k), encode(v)))
            .collect::<Vec<_>>()
            .join("&")
    }
    async fn post(
        app: &Fixture,
        path: &str,
        cookies: &Cookies,
        pairs: &[(String, String)],
    ) -> Response {
        let mut req = Request::builder()
            .method("POST")
            .uri(path)
            .header(header::ORIGIN, TEST_ORIGIN)
            .header("Sec-Fetch-Site", "same-origin")
            .header("Sec-Fetch-Mode", "navigate")
            .header("Sec-Fetch-Dest", "document")
            .header(header::COOKIE, cookies.header())
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(form_body(pairs)))
            .unwrap();
        req.extensions_mut().insert(ConnectInfo(
            "127.0.0.1:41000".parse::<SocketAddr>().unwrap(),
        ));
        let (parts, body) = app.service.clone().oneshot(req).await.unwrap().into_parts();
        Response {
            status: parts.status,
            headers: parts.headers,
            bytes: to_bytes(body, 256 * 1024).await.unwrap().to_vec(),
            sent_secrets: cookies
                .0
                .values()
                .cloned()
                .chain(
                    pairs
                        .iter()
                        .filter(|(k, _)| k == "csrf_proof")
                        .map(|(_, v)| v.clone()),
                )
                .collect(),
        }
    }
    async fn clock(pool: &PgPool) -> time::OffsetDateTime {
        sqlx::query_scalar("SELECT clock_timestamp()")
            .fetch_one(pool)
            .await
            .unwrap()
    }
    async fn preflight(
        pool: &PgPool,
        app: &Fixture,
        path: &str,
        cookies: &Cookies,
        expected: StatusCode,
    ) -> Response {
        let before = all_rows(pool).await;
        let started = clock(pool).await;
        let response = document(app, path, cookies).await;
        let finished = clock(pool).await;
        assert_eq!(response.status, expected);
        assert!(
            policy_preflight_effects(&before, &all_rows(pool).await, started, finished),
            "preflight changed more than exact Account CSRF global/IP limiter effects"
        );
        response
    }
    fn rows(raw: &str) -> BTreeMap<String, Value> {
        serde_json::from_str::<Vec<Value>>(raw)
            .unwrap()
            .into_iter()
            .map(|v| {
                (
                    v["id"]
                        .as_str()
                        .unwrap_or_else(|| v["org_id"].as_str().unwrap())
                        .to_owned(),
                    v,
                )
            })
            .collect()
    }
    fn census(
        before: &BTreeMap<String, String>,
        after: &BTreeMap<String, String>,
        input: &NativePolicyCommand,
        terminal: &NativePolicyTerminalView,
        previous: Option<Uuid>,
    ) {
        let mut counts = BTreeMap::from([
            ("native_company_policy_inputs_v1", 1usize),
            ("native_company_policy_receipts_v1", 1),
            ("audit_events", 2),
        ]);
        let assignment = match &terminal.outcome {
            NativePolicyOutcome::Committed(NativePolicyEffect::Installed { .. }) => {
                counts.insert("audit_events", 3);
                counts.extend([
                    ("ont_object_type_key_revisions", 1),
                    ("ont_object_types", 1),
                    ("ont_property_defs", 6),
                    ("ont_action_types", 2),
                    ("ont_builtin_catalog_installs", 1),
                    ("native_company_catalog_installs", 1),
                    ("native_company_object_refs", 1),
                    ("native_company_action_refs", 2),
                    ("native_company_property_refs", 6),
                ]);
                None
            }
            NativePolicyOutcome::Committed(NativePolicyEffect::Granted {
                assignment,
                assignment_revision_before,
                ..
            }) => {
                assert!(assignment_revision_before.is_none());
                counts.extend([
                    ("policy_assignment_revisions", 1),
                    ("policy_roles", 1),
                    ("policy_role_revisions", 1),
                    ("policy_capability_clauses", 1),
                    (
                        "policy_capability_clause_fields",
                        if NativePolicyCommandRef::from_command(input).directory_action()
                            == Some(DirectoryActionV1::Read)
                        {
                            6
                        } else {
                            2
                        },
                    ),
                    ("user_role_assignments", 1),
                ]);
                assert_eq!(assignment.expectation.assignment_revision, 1);
                None
            }
            NativePolicyOutcome::Committed(NativePolicyEffect::Revoked { assignment, .. }) => {
                counts.insert("policy_assignment_revisions", 1);
                Some(assignment.expectation)
            }
            _ => panic!("HTTP request did not commit its operation"),
        };
        assert!(before.keys().eq(after.keys()));
        for table in counts.keys() {
            assert!(before.contains_key(*table));
        }
        for (table, old) in before {
            if table == "company_authority_heads"
                || (table == "user_role_assignments" && assignment.is_some())
            {
                continue;
            }
            assert_eq!(
                added_rows(old, &after[table])
                    .expect("HTTP command rewrote history")
                    .len(),
                counts.get(table.as_str()).copied().unwrap_or(0),
                "unexpected HTTP effect in {table}"
            );
        }
        let mut heads = rows(&after["company_authority_heads"]);
        let head = heads
            .get_mut(&input.company().as_uuid().to_string())
            .unwrap();
        assert_eq!(head["epoch"], json!(input.expected_company_epoch() + 1));
        assert_eq!(
            head["current_policy_receipt_id"],
            json!(terminal.receipt_id)
        );
        head["epoch"] = json!(input.expected_company_epoch());
        head["current_policy_receipt_id"] = json!(previous);
        assert_eq!(heads, rows(&before["company_authority_heads"]));
        if let Some(a) = assignment {
            let mut updated = rows(&after["user_role_assignments"]);
            let row = updated.get_mut(&a.assignment_id.to_string()).unwrap();
            assert_eq!(row["native_current_revision"], json!(a.assignment_revision));
            row["native_current_revision"] = json!(a.assignment_revision - 1);
            assert_eq!(updated, rows(&before["user_role_assignments"]));
        }
    }
    async fn transition(
        pool: &PgPool,
        app: &Fixture,
        cookies: &Cookies,
        store: &PgOrgStore,
        policy: &CompanyPolicy,
        company: OrgId,
        actor: AccountId,
        op: NativeBusinessOperationV1,
        action: Option<DirectoryActionV1>,
        epoch: u64,
        assignment: Option<PolicyAssignmentExpectationV1>,
        previous: Option<Uuid>,
    ) -> NativePolicyTerminalView {
        let root = format!("/companies/{company}/policy/people-directory");
        let kind = match op {
            NativeBusinessOperationV1::Install => "install",
            NativeBusinessOperationV1::Grant => "grant",
            NativeBusinessOperationV1::Revoke => "revoke",
        };
        let action_path = match action {
            Some(DirectoryActionV1::Read) => "read",
            Some(DirectoryActionV1::Create) => "create",
            None => "",
        };
        let form_path = if action.is_none() {
            format!("{root}/install")
        } else {
            format!("{root}/{action_path}/{kind}")
        };
        let before = all_rows(pool).await;
        let started = clock(pool).await;
        let page = document(app, &form_path, cookies).await;
        let finished = clock(pool).await;
        if op == NativeBusinessOperationV1::Install && page.status == StatusCode::NOT_FOUND {
            assert!(before == all_rows(pool).await);
            panic!(
                "PEOPLE_POLICY_HTTP_ENTRY_RED: verified successor owner has no People install HTTP document"
            );
        }
        let html = native_entry_html(&page, StatusCode::OK);
        assert!(
            policy_preflight_effects(&before, &all_rows(pool).await, started, finished),
            "preflight changed more than exact limiter effects"
        );
        let before = all_rows(pool).await; // actual command baseline, after validated limiter writes
        let command: Uuid = field(&html, "command_id").parse().unwrap();
        assert!(!command.is_nil());
        assert_eq!(field(&html, "expected_company_epoch"), epoch.to_string());
        let mut pairs = vec![
            ("command_id".into(), command.to_string()),
            ("expected_company_epoch".into(), epoch.to_string()),
            ("csrf_proof".into(), field(&html, "csrf_proof")),
        ];
        let now: time::OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
            .fetch_one(pool)
            .await
            .unwrap();
        let until =
            time::OffsetDateTime::from_unix_timestamp((now.unix_timestamp() / 60 + 1440) * 60)
                .unwrap();
        let local = until.to_offset(time::macros::offset!(+9));
        let input = match op {
            NativeBusinessOperationV1::Install => {
                NativePeoplePolicyCommandV1::install(command, company, epoch).unwrap()
            }
            NativeBusinessOperationV1::Grant => {
                assert_eq!(
                    field(&html, "recipient_account_id"),
                    actor.as_uuid().to_string()
                );
                for key in [
                    "recipient_account_id",
                    "expected_role_revision",
                    "assignment_id",
                    "expected_assignment_revision",
                ] {
                    pairs.push((key.into(), field(&html, key)));
                }
                for key in [
                    "expected_role_revision",
                    "assignment_id",
                    "expected_assignment_revision",
                ] {
                    assert!(field(&html, key).is_empty());
                }
                pairs.push((
                    "expires_at_local".into(),
                    format!(
                        "{:04}-{:02}-{:02}T{:02}:{:02}",
                        local.year(),
                        u8::from(local.month()),
                        local.day(),
                        local.hour(),
                        local.minute()
                    ),
                ));
                NativePeoplePolicyCommandV1::grant(
                    command,
                    company,
                    epoch,
                    action.unwrap(),
                    actor,
                    None,
                    until,
                )
                .unwrap()
            }
            NativeBusinessOperationV1::Revoke => {
                let a = assignment.unwrap();
                assert_eq!(
                    field(&html, "expected_role_revision"),
                    a.role_revision.to_string()
                );
                assert_eq!(
                    field(&html, "expected_assignment_revision"),
                    a.assignment_revision.to_string()
                );
                pairs.extend([
                    ("expected_role_revision".into(), a.role_revision.to_string()),
                    (
                        "expected_assignment_revision".into(),
                        a.assignment_revision.to_string(),
                    ),
                ]);
                NativePeoplePolicyCommandV1::revoke(command, company, epoch, action.unwrap(), a)
                    .unwrap()
            }
        };
        let path = match op {
            NativeBusinessOperationV1::Install => format!("{root}/catalog"),
            NativeBusinessOperationV1::Grant => format!("{root}/{action_path}/grants"),
            NativeBusinessOperationV1::Revoke => format!(
                "{root}/{action_path}/grants/{}/revoke",
                assignment.unwrap().assignment_id
            ),
        };
        assert!(html.contains(&format!("action=\"{path}\"")));
        let mut corrupt = pairs.clone();
        corrupt
            .iter_mut()
            .find(|(k, _)| k == "csrf_proof")
            .unwrap()
            .1 = "invalid-proof".into();
        let refusal = post(app, &path, cookies, &corrupt).await;
        assert_eq!(refusal.status, StatusCode::FORBIDDEN);
        assert!(
            before == all_rows(pool).await,
            "invalid proof wrote effects"
        );
        let result_path = format!("{root}/requests/{kind}/{command}");
        let response = post(app, &path, cookies, &pairs).await;
        assert_eq!(response.status, StatusCode::SEE_OTHER);
        response.private();
        assert_eq!(
            response.headers.get(header::LOCATION).unwrap(),
            result_path.as_str()
        );
        let input = NativePolicyCommand::People(input);
        let selector = NativePolicyCommandRef::from_command(&input);
        let terminal =
            match native_policy_command_status(store, policy, &read_credentials(cookies), selector)
                .await
                .unwrap()
            {
                NativePolicyStatus::Terminal(t) => t,
                _ => panic!("HTTP redirect lacks durable terminal"),
            };
        assert_eq!(terminal.accepted.input, input);
        assert!(matches!(
            (op, &terminal.outcome),
            (
                NativeBusinessOperationV1::Install,
                NativePolicyOutcome::Committed(NativePolicyEffect::Installed { .. })
            ) | (
                NativeBusinessOperationV1::Grant,
                NativePolicyOutcome::Committed(NativePolicyEffect::Granted { .. })
            ) | (
                NativeBusinessOperationV1::Revoke,
                NativePolicyOutcome::Committed(NativePolicyEffect::Revoked { .. })
            )
        ));
        assert_eq!(
            (terminal.epoch_before, terminal.epoch_after),
            (epoch, epoch + 1)
        );
        let stored:(i16,Vec<u8>,Vec<u8>,Uuid,Option<Uuid>,bool)=sqlx::query_as(
            "SELECT i.codec_version,i.input_bytes,i.input_digest,r.receipt_id,r.predecessor_receipt_id,r.intake_receipt_id=i.intake_receipt_id AND r.input_digest=i.input_digest AND r.effect_xid<>i.acceptance_xid AND r.outcome='COMMITTED' AND r.catalog_version='native-people-directory-v1' FROM public.native_company_policy_inputs_v1 i JOIN public.native_company_policy_receipts_v1 r USING(actor_account_id,command_id,org_id) WHERE i.actor_account_id=$1 AND i.command_id=$2 AND i.org_id=$3")
            .bind(actor.as_uuid()).bind(command).bind(company.as_uuid()).fetch_one(pool).await.unwrap();
        assert_eq!(stored.0, 2);
        assert_eq!(stored.1, input.encode(actor));
        assert_eq!(stored.2, Sha256::digest(&stored.1).as_slice());
        assert_eq!(stored.3, terminal.receipt_id);
        assert_eq!(stored.4, previous);
        assert!(stored.5);
        let after = all_rows(pool).await;
        census(&before, &after, &input, &terminal, previous);
        let reopened = document(app, &result_path, cookies).await;
        let reopened = native_entry_html(&reopened, StatusCode::OK);
        assert!(reopened.contains("data-policy-outcome=\"committed\""));
        assert!(reopened.contains(&terminal.receipt_id.to_string()));
        let replay = post(app, &path, cookies, &pairs).await;
        assert_eq!(replay.status, StatusCode::SEE_OTHER);
        assert_eq!(
            replay.headers.get(header::LOCATION).unwrap(),
            result_path.as_str()
        );
        assert!(
            after == all_rows(pool).await,
            "HTTP reopening/replay wrote effects"
        );
        terminal
    }

    #[sqlx::test(migrations = false)]
    async fn people_policy_http_install_action_grants_revokes_and_reopening_reach_real_owner(
        pool: PgPool,
    ) {
        let (app, key, state) =
            super::native_people_codec2_install_probe::configured_successor_fixture(&pool).await;
        let mut cleanup = None;
        let outcome=AssertUnwindSafe(async {
            let (app,cookies,created)=create_owned_company(&pool,app).await;
            let (verifier,issuer,ttl)=bindings(&account_browser_config(&pool,app._artifacts.root.clone(),&key));
            let runtime=login_test_pool(&pool,TestDatabaseLogin::Business).await;cleanup=Some(runtime.clone());
            let login:(String,String,bool,bool)=sqlx::query_as("SELECT session_user::text,current_user::text,rolsuper,rolbypassrls FROM pg_roles WHERE rolname=current_user").fetch_one(&runtime).await.unwrap();
            assert_eq!(login,("console_rt".into(),"console_rt".into(),false,false));
            let store=PgOrgStore::new(runtime).with_native_account_policy(verifier,issuer,ttl);
            let policy=CompanyPolicy::new().unwrap();let company=OrgId::from_uuid(created.company);let actor=AccountId::from_uuid(created.administrator).unwrap();
            // Existing Payroll entry is a mounted HTTP positive control before
            // the exact People404 RED. No business command is installed here.
            let control=preflight(&pool,&app,&format!("/companies/{company}/policy/payroll-read/install"),&cookies,StatusCode::OK).await;
            native_entry_html(&control,StatusCode::OK);
            eprintln!("PEOPLE_POLICY_HTTP_PAYROLL_CONTROL_OK: verified successor Payroll form mounted");
            let mut previous=None;let mut assignments=BTreeMap::new();
            let steps=[(NativeBusinessOperationV1::Install,None),(NativeBusinessOperationV1::Grant,Some(DirectoryActionV1::Read)),
                (NativeBusinessOperationV1::Grant,Some(DirectoryActionV1::Create)),(NativeBusinessOperationV1::Revoke,Some(DirectoryActionV1::Read)),
                (NativeBusinessOperationV1::Revoke,Some(DirectoryActionV1::Create))];
            for (index,(op,action)) in steps.into_iter().enumerate(){
                let key=action.map(|a|a.as_str());let expected=if op==NativeBusinessOperationV1::Revoke{Some(assignments[key.unwrap()])}else{None};
                let done=transition(&pool,&app,&cookies,&store,&policy,company,actor,op,action,index as u64+1,expected,previous).await;
                if let NativePolicyOutcome::Committed(NativePolicyEffect::Granted{assignment,..})=&done.outcome{assignments.insert(key.unwrap(),assignment.expectation);}
                previous=Some(done.receipt_id);
                // Per-action current projection must drive workspace links:
                // install.assignment=None must never erase another action grant.
                let before=all_rows(&pool).await;let page=document(&app,&format!("/companies/{company}"),&cookies).await;
                let html=native_entry_html(&page,StatusCode::OK);
                for (a,active) in [("read",index==1||index==2),("create",index==2||index==3)]{
                    let next=if active{"revoke"}else{"grant"};let wrong=if active{"grant"}else{"revoke"};
                    assert!(html.contains(&format!("href=\"/companies/{company}/policy/people-directory/{a}/{next}\"")));
                    assert!(!html.contains(&format!("href=\"/companies/{company}/policy/people-directory/{a}/{wrong}\"")));
                }
                assert!(before==all_rows(&pool).await);
            }
            let (_,outsider)=enrolled(&app).await;
            for path in [format!("/companies/{company}/policy/people-directory/install"),format!("/companies/{company}/policy/people-directory/read/grant")]{
                let denied=preflight(&pool,&app,&path,&outsider,StatusCode::NOT_FOUND).await;
                assert!(!String::from_utf8(denied.bytes).unwrap().contains(&created.administrator.to_string()));
            }
        }).catch_unwind().await;
        if let Some(runtime) = cleanup {
            runtime.close().await;
        }
        close_states(&[state], outcome).await;
    }
    #[sqlx::test(migrations = false)]
    async fn predecessor_workspace_retains_payroll_without_exposing_people_policy(pool: PgPool) {
        let (app, _key, state) = configured_fixture(&pool, true).await;
        let outcome = AssertUnwindSafe(async {
            let (app, cookies, created) = create_owned_company(&pool, app).await;
            let root = format!("/companies/{}", created.company);
            let payroll = preflight(
                &pool,
                &app,
                &format!("{root}/policy/payroll-read/install"),
                &cookies,
                StatusCode::OK,
            )
            .await;
            native_entry_html(&payroll, StatusCode::OK);
            let before = all_rows(&pool).await;
            let workspace = document(&app, &root, &cookies).await;
            let html = native_entry_html(&workspace, StatusCode::OK);
            assert!(!html.contains("/policy/people-directory"));
            for suffix in ["install", "read/grant", "create/grant"] {
                assert_eq!(
                    document(
                        &app,
                        &format!("{root}/policy/people-directory/{suffix}"),
                        &cookies
                    )
                    .await
                    .status,
                    StatusCode::NOT_FOUND
                );
            }
            assert!(before == all_rows(&pool).await);
        })
        .catch_unwind()
        .await;
        close_states(&[state], outcome).await;
    }

    #[sqlx::test(migrations = false)]
    async fn predecessor_process_requires_restart_after_actual_successor_upgrade(pool: PgPool) {
        let (app, key, state) = configured_fixture(&pool, true).await;
        let mut fresh_cleanup = None;
        let outcome=AssertUnwindSafe(async {
            let (app,cookies,created)=create_owned_company(&pool,app).await;
            assert_eq!(ready_status(&state).await,StatusCode::OK);
            const FINALIZER:&str=include_str!("../../../../ops/postgres-finalize-native-company-policy-v2.sql");
            assert_eq!(hex::encode(Sha256::digest(FINALIZER.as_bytes())),"ec945607e209b93843116ae2b2a20772797dce38ff7884fb96081f09651f7d8e");
            let mut tx=pool.begin().await.unwrap();
            sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL READ COMMITTED; SET LOCAL search_path=pg_catalog,pg_temp; SET LOCAL lock_timeout='1s'; SET LOCAL statement_timeout='120s'").execute(tx.as_mut()).await.unwrap();
            sqlx::raw_sql(FINALIZER).execute(tx.as_mut()).await.unwrap();
            sqlx::raw_sql("SET CONSTRAINTS ALL IMMEDIATE").execute(tx.as_mut()).await.unwrap();
            tx.commit().await.unwrap();
            let upgraded=all_rows(&pool).await;
            assert_eq!(ready_status(&state).await,StatusCode::SERVICE_UNAVAILABLE,
                "PEOPLE_POLICY_READMISSION_RED: old process silently retained ready after capability upgrade");
            let fresh=AppState::from_config(account_browser_config(&pool,app._artifacts.root.clone(),&key)).await.unwrap();
            fresh_cleanup=Some(fresh.clone());assert_eq!(ready_status(&fresh).await,StatusCode::OK);
            assert!(upgraded==all_rows(&pool).await,"readmission wrote business effects");
            let reopened=Fixture{service:build_router(fresh),_artifacts:app._artifacts,pool:pool.clone()};
            let root=format!("/companies/{}",created.company);
            for suffix in ["payroll-read/install","people-directory/install"] {
                let response=preflight(&pool,&reopened,&format!("{root}/policy/{suffix}"),&cookies,StatusCode::OK).await;
                native_entry_html(&response,StatusCode::OK);
            }
        }).catch_unwind().await;
        if let Some(fresh) = fresh_cleanup {
            fresh.shutdown_realtime().await;
        }
        close_states(&[state], outcome).await;
    }
    include!("native_people_policy_http_recovery.rs");
    include!("native_people_directory_http_owner.rs");
}
