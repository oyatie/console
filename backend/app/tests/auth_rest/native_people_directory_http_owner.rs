// Include inside native_people_policy_http_owner. Isolated actual HTTP/database
// evidence, not browser or production qualification. No fixture business SQL.
mod directory_http {
    use super::super::native_people_directory_finalizer_tests::native_people_directory_row_lock_tests::configured_row_lock_native_directory_fixture as configured_native_directory_fixture;
    use super::*;
    use console_ontology_application::people::NativeDirectoryCommandV1;
    const NAME: &str = "김하늘 <연구 & 운영>";
    const NUMBER: &str = "HTTP-사람-001";
    const FIELDS: &[&str] = &[
        "csrf_proof",
        "command_id",
        "expected_company_epoch",
        "object_type_id",
        "action_type_id",
        "expected_action_revision",
        "expected_schema_revision",
        "legal_name_property_id",
        "employee_number_property_id",
        "legal_name",
        "employee_number",
    ];
    fn operation<'a>(html: &'a str, op: &str) -> &'a str {
        let needle = format!("data-people-operation=\"{op}\"");
        let forms: Vec<_> = html
            .split("<form")
            .skip(1)
            .map(|part| part.split("</form>").next().unwrap())
            .filter(|part| part.split('>').next().unwrap().contains(&needle))
            .collect();
        assert_eq!(forms.len(), 1, "missing or duplicate {op} form");
        forms[0]
    }
    fn change(pairs: &mut [(String, String)], name: &str, value: &str) {
        let entries: Vec<_> = pairs.iter_mut().filter(|(key, _)| key == name).collect();
        assert_eq!(entries.len(), 1);
        entries.into_iter().next().unwrap().1 = value.to_owned();
    }
    fn pairs(html: &str) -> Vec<(String, String)> {
        let form = operation(html, "prepare");
        FIELDS
            .iter()
            .map(|key| ((*key).into(), field(form, key)))
            .collect()
    }
    fn exact_delta(
        before: &BTreeMap<String, String>,
        after: &BTreeMap<String, String>,
        expected: &[(&str, usize)],
    ) {
        assert!(before.keys().eq(after.keys()));
        let expected: BTreeMap<_, _> = expected.iter().copied().collect();
        for table in expected.keys() {
            assert!(before.contains_key(*table));
        }
        for (table, old) in before {
            assert_eq!(
                added_rows(old, &after[table])
                    .expect("HTTP rewrote old row bytes")
                    .len(),
                expected.get(table.as_str()).copied().unwrap_or(0),
                "unexpected effect in {table}"
            );
        }
    }
    fn one(
        before: &BTreeMap<String, String>,
        after: &BTreeMap<String, String>,
        table: &str,
    ) -> Value {
        let added = added_rows(&before[table], &after[table]).unwrap();
        assert_eq!(added.len(), 1);
        added[0].clone()
    }
    fn redirect(response: &Response, path: &str) {
        assert_eq!(response.status, StatusCode::SEE_OTHER);
        response.private();
        assert_eq!(response.headers.get(header::LOCATION).unwrap(), path);
    }
    fn accepted(
        before: &BTreeMap<String, String>,
        after: &BTreeMap<String, String>,
        company: OrgId,
        actor: AccountId,
        submitted: &[(String, String)],
    ) -> (Value, NativeDirectoryCommandV1) {
        exact_delta(
            before,
            after,
            &[("native_people_inputs_v1", 1), ("audit_events", 1)],
        );
        let row = one(before, after, "native_people_inputs_v1");
        let bytes = hex::decode(
            row["input_bytes"]
                .as_str()
                .unwrap()
                .strip_prefix("\\x")
                .unwrap(),
        )
        .unwrap();
        let (who, command) = NativeDirectoryCommandV1::decode(&bytes).unwrap();
        assert_eq!(who, actor);
        assert_eq!(command.company(), company);
        assert_eq!(command.encode(who), bytes);
        assert_eq!(
            row["input_digest"],
            format!("\\x{}", hex::encode(Sha256::digest(&bytes)))
        );
        assert_eq!(row["org_id"], json!(company.as_uuid()));
        assert_eq!(row["actor_account_id"], json!(actor.as_uuid()));
        assert_eq!(row["command_id"], json!(command.command_id()));
        assert_eq!(row["employee_id"], json!(command.employee_id()));
        assert_eq!(row["legal_name"], NAME);
        assert_eq!(row["employee_number"], NUMBER);
        assert_eq!(command.input().legal_name(), NAME);
        assert_eq!(command.input().employee_number(), NUMBER);
        let sent: BTreeMap<_, _> = submitted
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        let e = command.expected();
        for (key, value) in [
            ("command_id", command.command_id().to_string()),
            ("expected_company_epoch", e.company_epoch.to_string()),
            ("object_type_id", e.object_type_id.to_string()),
            ("action_type_id", e.action_type_id.to_string()),
            ("expected_action_revision", e.action_revision.to_string()),
            ("expected_schema_revision", e.schema_revision.to_string()),
            (
                "legal_name_property_id",
                e.legal_name_property_id.to_string(),
            ),
            (
                "employee_number_property_id",
                e.employee_number_property_id.to_string(),
            ),
        ] {
            assert_eq!(sent[key], value, "HTTP replaced accepted {key}");
        }
        let audit = one(before, after, "audit_events");
        assert_eq!(audit["action"], "people.directory.prepare");
        assert_eq!(audit["actor"], json!(actor.as_uuid()));
        assert_eq!(audit["org_id"], json!(company.as_uuid()));
        (row, command)
    }
    async fn install_and_grant(
        pool: &PgPool,
        app: &Fixture,
        cookies: &Cookies,
        store: &PgOrgStore,
        policy: &CompanyPolicy,
        company: OrgId,
        actor: AccountId,
    ) -> (
        Uuid,
        PolicyAssignmentExpectationV1,
        PolicyAssignmentExpectationV1,
    ) {
        let installed = transition(
            pool,
            app,
            cookies,
            store,
            policy,
            company,
            actor,
            NativeBusinessOperationV1::Install,
            None,
            1,
            None,
            None,
        )
        .await;
        let read = transition(
            pool,
            app,
            cookies,
            store,
            policy,
            company,
            actor,
            NativeBusinessOperationV1::Grant,
            Some(DirectoryActionV1::Read),
            2,
            None,
            Some(installed.receipt_id),
        )
        .await;
        let create = transition(
            pool,
            app,
            cookies,
            store,
            policy,
            company,
            actor,
            NativeBusinessOperationV1::Grant,
            Some(DirectoryActionV1::Create),
            3,
            None,
            Some(read.receipt_id),
        )
        .await;
        let assignment = |terminal: NativePolicyTerminalView| match terminal.outcome {
            NativePolicyOutcome::Committed(NativePolicyEffect::Granted { assignment, .. }) => {
                assignment.expectation
            }
            _ => panic!("actual HTTP grant did not commit"),
        };
        (create.receipt_id, assignment(read), assignment(create))
    }
    async fn registration(
        pool: &PgPool,
        app: &Fixture,
        cookies: &Cookies,
        root: &str,
    ) -> Vec<(String, String)> {
        let response = preflight(pool, app, &format!("{root}/new"), cookies, StatusCode::OK).await;
        let html = native_entry_html(&response, StatusCode::OK);
        assert!(operation(html, "prepare").contains(&format!("action=\"{root}/requests\"")));
        let mut fields = pairs(html);
        change(&mut fields, "legal_name", NAME);
        change(&mut fields, "employee_number", NUMBER);
        fields
    }
    async fn readonly_document(
        pool: &PgPool,
        app: &Fixture,
        cookies: &Cookies,
        path: &str,
        status: StatusCode,
    ) -> Response {
        let before = all_rows(pool).await;
        let response = document(app, path, cookies).await;
        assert_eq!(response.status, status);
        response.private();
        assert!(
            before == all_rows(pool).await,
            "read-only status consumed admission or changed business state"
        );
        response
    }
    async fn pending(
        pool: &PgPool,
        app: &Fixture,
        cookies: &Cookies,
        path: &str,
        command: Uuid,
    ) -> Vec<(String, String)> {
        let response = preflight(pool, app, path, cookies, StatusCode::OK).await;
        let html = native_entry_html(&response, StatusCode::OK);
        assert!(html.contains("data-people-outcome=\"pending\""));
        let form = operation(html, "execute");
        assert!(form.contains(&format!("action=\"{path}/execute\"")));
        assert_eq!(field(form, "command_id"), command.to_string());
        vec![
            ("command_id".into(), command.to_string()),
            ("csrf_proof".into(), field(form, "csrf_proof")),
        ]
    }

    #[sqlx::test(migrations = false)]
    async fn directory_http_validation_prepare_execute_reopen_replay_preserves_original_identity(
        pool: PgPool,
    ) {
        // Bound by root to the actual finalized230 fixture; never predecessor
        // custody or a hand-constructed owner/profile.
        let (app, key, state) = configured_native_directory_fixture(&pool).await;
        let mut cleanup = None;
        let outcome=AssertUnwindSafe(async {
            let(app,cookies,created)=create_owned_company(&pool,app).await;
            let(verifier,issuer,ttl)=bindings(&account_browser_config(&pool,app._artifacts.root.clone(),&key));
            let runtime=login_test_pool(&pool,TestDatabaseLogin::Business).await;cleanup=Some(runtime.clone());
            let role:(String,bool,bool)=sqlx::query_as("SELECT current_user::text,rolsuper,rolbypassrls FROM pg_roles WHERE rolname=current_user").fetch_one(&runtime).await.unwrap();
            assert_eq!(role,("console_rt".into(),false,false));
            let store=PgOrgStore::new(runtime).with_native_account_policy(verifier,issuer,ttl);let policy=CompanyPolicy::new().unwrap();
            let company=OrgId::from_uuid(created.company);let actor=AccountId::from_uuid(created.administrator).unwrap();
            install_and_grant(&pool,&app,&cookies,&store,&policy,company,actor).await;
            let root=format!("/companies/{company}/people");let mut fields=registration(&pool,&app,&cookies,&root).await;
            let original=fields.clone();change(&mut fields,"legal_name","");change(&mut fields,"employee_number","HTTP-invalid");
            let before=all_rows(&pool).await;let mut invalid=post(&app,&format!("{root}/requests"),&cookies,&fields).await;
            let proof=fields.iter().find(|(k,_)|k=="csrf_proof").unwrap().1.clone();
            assert_eq!(invalid.bytes.windows(proof.len()).filter(|w|*w==proof.as_bytes()).count(),1);
            for(_,value)in &invalid.headers {assert!(!value.as_bytes().windows(proof.len()).any(|w|w==proof.as_bytes()));}
            invalid.sent_secrets.retain(|s|s!=&proof);
            let html=native_entry_html(&invalid,StatusCode::UNPROCESSABLE_ENTITY);let form=operation(html,"prepare");
            for(key,value)in &fields{assert_eq!(field(form,key),*value,"invalid redisplay replaced {key}");}
            assert!(html.contains("aria-invalid=\"true\""));assert!(before==all_rows(&pool).await,"unexpected state mutation");
            let command:Uuid=original.iter().find(|(k,_)|k=="command_id").unwrap().1.parse().unwrap();
            let path=format!("{root}/requests/{command}");
            redirect(&post(&app,&format!("{root}/requests"),&cookies,&original).await,&path);
            let prepared=all_rows(&pool).await;let(intake,accepted)=accepted(&before,&prepared,company,actor,&original);
            let execution=pending(&pool,&app,&cookies,&path,command).await;
            // Reopen pending before mutation, preserving exact durable input.
            let _=pending(&pool,&app,&cookies,&path,command).await;
            let before_execute=all_rows(&pool).await;
            redirect(&post(&app,&format!("{path}/execute"),&cookies,&execution).await,&path);
            let committed=all_rows(&pool).await;
            exact_delta(&before_execute,&committed,&[("employees",1),("persons",1),("person_revisions",1),("employee_person_bindings",1),("ont_action_command_receipts",1),("native_people_terminals_v1",1),("audit_events",1)]);
            let employee=one(&before_execute,&committed,"employees");let person=one(&before_execute,&committed,"persons");
            let revision=one(&before_execute,&committed,"person_revisions");let binding=one(&before_execute,&committed,"employee_person_bindings");
            let receipt=one(&before_execute,&committed,"ont_action_command_receipts");let terminal=one(&before_execute,&committed,"native_people_terminals_v1");
            assert_eq!(employee["id"],intake["employee_id"]);assert_eq!(person["id"],employee["id"]);
            assert_eq!(employee["name"],NAME);assert_eq!(employee["employee_number"],NUMBER);
            assert_eq!(employee["employment_status"],"UNKNOWN");assert_eq!(employee["source_kind"],"NATIVE_DIRECTORY");assert_eq!(employee["native_command_id"],json!(command));
            for key in ["hire_date","exit_date","source_filename","source_sheet","source_row","home_branch_id","org_unit","job","position"] {assert_eq!(employee[key],Value::Null,"invented {key}");}
            assert_eq!(revision["attributes"],json!({"legal_name":NAME}));assert_eq!(revision["version"],1);
            for row in [&employee,&person,&revision,&binding,&receipt,&terminal] {assert_eq!(row["org_id"],json!(created.company));}
            for row in [&revision,&binding,&receipt] {assert_eq!(row["actor_kind"],"ACCOUNT");assert_eq!(row["actor_account_id"],json!(created.administrator));assert!(row["actor_id"].is_null());}
            assert_eq!(binding["employee_id"],employee["id"]);assert_eq!(binding["person_id"],person["id"]);assert_eq!(revision["person_id"],person["id"]);
            let result=json!({"person_id":accepted.employee_id(),"version":1,"target":"people.create_person"});
            assert_eq!(receipt["receipt"],result);assert_eq!(revision["receipt"],result);assert_eq!(receipt["owner"],"person");assert_eq!(receipt["target"],"people.create_person");
            let digest=format!("\\x{}",hex::encode(Sha256::digest(accepted.effect_payload(actor))));
            for row in [&receipt,&revision,&binding] {assert_eq!(row["payload_digest"],digest);}
            for row in [&receipt,&revision,&terminal] {assert_eq!(row["command_id"],json!(command));}
            for key in ["intake_receipt_id","input_digest","employee_id"]{assert_eq!(terminal[key],intake[key]);}
            assert_eq!(terminal["actor_account_id"],json!(created.administrator));assert_eq!(terminal["person_id"],person["id"]);assert_eq!(terminal["outcome"],"COMMITTED");
            let audit=one(&before_execute,&committed,"audit_events");assert_eq!(audit["action"],"people.directory.register");assert_eq!(audit["actor"],json!(created.administrator));assert_eq!(audit["org_id"],json!(created.company));
            redirect(&post(&app,&format!("{path}/execute"),&cookies,&execution).await,&path);
            redirect(&post(&app,&format!("{root}/requests"),&cookies,&original).await,&path);
            assert!(committed==all_rows(&pool).await,"HTTP replay duplicated effects");
            let reopened=readonly_document(&pool,&app,&cookies,&path,StatusCode::OK).await;
            let html=native_entry_html(&reopened,StatusCode::OK);assert!(html.contains("data-people-outcome=\"committed\""));
            assert!(html.contains(&format!("data-people-employee=\"{}\"",accepted.employee_id())));assert!(!html.contains("data-people-operation=\"execute\""));
            for destination in [root.clone(),format!("{root}/{}",accepted.employee_id())] {
                let before=all_rows(&pool).await;let response=document(&app,&destination,&cookies).await;let html=native_entry_html(&response,StatusCode::OK);
                assert!(html.contains(&format!("data-people-record=\"{}\"",accepted.employee_id())));assert!(!html.contains(NAME));assert!(html.contains("연구 &amp; 운영"));
                exact_delta(&before,&all_rows(&pool).await,&[("audit_events",1)]);
            }
        }).catch_unwind().await;
        if let Some(runtime) = cleanup {
            runtime.close().await;
        }
        close_states(&[state], outcome).await;
    }

    #[sqlx::test(migrations = false)]
    async fn directory_http_strict_input_cross_account_and_current_revoke_never_write_business_effects(
        pool: PgPool,
    ) {
        let (app, key, state) = configured_native_directory_fixture(&pool).await;
        let mut cleanup = None;
        let outcome = AssertUnwindSafe(async {
            let (app, cookies, created) = create_owned_company(&pool, app).await;
            let (verifier, issuer, ttl) = bindings(&account_browser_config(
                &pool,
                app._artifacts.root.clone(),
                &key,
            ));
            let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
            cleanup = Some(runtime.clone());
            let store = PgOrgStore::new(runtime).with_native_account_policy(verifier, issuer, ttl);
            let policy = CompanyPolicy::new().unwrap();
            let company = OrgId::from_uuid(created.company);
            let actor = AccountId::from_uuid(created.administrator).unwrap();
            let (previous, read_assignment, create_assignment) =
                install_and_grant(&pool, &app, &cookies, &store, &policy, company, actor).await;
            let root = format!("/companies/{company}/people");
            let fields = registration(&pool, &app, &cookies, &root).await;
            let command: Uuid = fields
                .iter()
                .find(|(k, _)| k == "command_id")
                .unwrap()
                .1
                .parse()
                .unwrap();
            let path = format!("{root}/requests/{command}");
            let before = all_rows(&pool).await;
            let mut bad = fields.clone();
            bad.push(("salary".into(), "1000000".into()));
            assert_eq!(
                post(&app, &format!("{root}/requests"), &cookies, &bad)
                    .await
                    .status,
                StatusCode::BAD_REQUEST
            );
            bad = fields.clone();
            bad.push(fields[0].clone());
            assert_eq!(
                post(&app, &format!("{root}/requests"), &cookies, &bad)
                    .await
                    .status,
                StatusCode::BAD_REQUEST
            );
            bad = fields.clone();
            change(&mut bad, "csrf_proof", "invalid-proof");
            assert_eq!(
                post(&app, &format!("{root}/requests"), &cookies, &bad)
                    .await
                    .status,
                StatusCode::FORBIDDEN
            );
            assert!(before == all_rows(&pool).await, "unexpected state mutation");
            redirect(
                &post(&app, &format!("{root}/requests"), &cookies, &fields).await,
                &path,
            );
            accepted(&before, &all_rows(&pool).await, company, actor, &fields);
            let execution = pending(&pool, &app, &cookies, &path, command).await;
            let (_, outsider) = enrolled(&app).await;
            let before = all_rows(&pool).await;
            let denied = post(&app, &format!("{path}/execute"), &outsider, &execution).await;
            assert_eq!(denied.status, StatusCode::FORBIDDEN);
            denied.private();
            assert!(before == all_rows(&pool).await, "unexpected state mutation");
            let denied =
                readonly_document(&pool, &app, &outsider, &path, StatusCode::NOT_FOUND).await;
            assert!(
                !std::str::from_utf8(&denied.bytes)
                    .unwrap()
                    .contains("HTTP-사람")
            );
            let read_revoke = transition(
                &pool,
                &app,
                &cookies,
                &store,
                &policy,
                company,
                actor,
                NativeBusinessOperationV1::Revoke,
                Some(DirectoryActionV1::Read),
                4,
                Some(read_assignment),
                Some(previous),
            )
            .await;
            let before = all_rows(&pool).await;
            assert_eq!(
                document(&app, &root, &cookies).await.status,
                StatusCode::NOT_FOUND
            );
            assert!(before == all_rows(&pool).await, "unexpected state mutation");
            // Create remains separately authorized after Read is revoked.
            let recovery = pending(&pool, &app, &cookies, &path, command).await;
            transition(
                &pool,
                &app,
                &cookies,
                &store,
                &policy,
                company,
                actor,
                NativeBusinessOperationV1::Revoke,
                Some(DirectoryActionV1::Create),
                5,
                Some(create_assignment),
                Some(read_revoke.receipt_id),
            )
            .await;
            let before = all_rows(&pool).await;
            for suffix in ["execute", "cancel"] {
                let response = post(&app, &format!("{path}/{suffix}"), &cookies, &recovery).await;
                assert_eq!(response.status, StatusCode::NOT_FOUND);
                response.private();
                assert!(before == all_rows(&pool).await, "unexpected state mutation");
            }
            assert_eq!(
                post(&app, &format!("{root}/requests"), &cookies, &fields)
                    .await
                    .status,
                StatusCode::NOT_FOUND
            );
            assert!(before == all_rows(&pool).await, "unexpected state mutation");
            let denied =
                readonly_document(&pool, &app, &cookies, &path, StatusCode::NOT_FOUND).await;
            assert!(
                !std::str::from_utf8(&denied.bytes)
                    .unwrap()
                    .contains("HTTP-사람")
            );
            let final_rows = all_rows(&pool).await;
            for table in [
                "employees",
                "persons",
                "person_revisions",
                "employee_person_bindings",
                "ont_action_command_receipts",
                "native_people_terminals_v1",
            ] {
                assert_eq!(
                    before[table], final_rows[table],
                    "revocation left effect in {table}"
                );
            }
        })
        .catch_unwind()
        .await;
        if let Some(runtime) = cleanup {
            runtime.close().await;
        }
        close_states(&[state], outcome).await;
    }

    // Actual current policy denial must not consume fresh-proof admission.
    #[sqlx::test(migrations = false)]
    async fn directory_http_denied_registration_does_not_charge_form_admission(pool: PgPool) {
        let (app, key, state) = configured_native_directory_fixture(&pool).await;
        let mut cleanup = None;
        let outcome = AssertUnwindSafe(async {
            let (app, cookies, created) = create_owned_company(&pool, app).await;
            let (verifier, issuer, ttl) = bindings(&account_browser_config(
                &pool,
                app._artifacts.root.clone(),
                &key,
            ));
            let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
            cleanup = Some(runtime.clone());
            let store = PgOrgStore::new(runtime).with_native_account_policy(verifier, issuer, ttl);
            let policy = CompanyPolicy::new().unwrap();
            let company = OrgId::from_uuid(created.company);
            let actor = AccountId::from_uuid(created.administrator).unwrap();
            let installed = transition(
                &pool,
                &app,
                &cookies,
                &store,
                &policy,
                company,
                actor,
                NativeBusinessOperationV1::Install,
                None,
                1,
                None,
                None,
            )
            .await;
            let read = transition(
                &pool,
                &app,
                &cookies,
                &store,
                &policy,
                company,
                actor,
                NativeBusinessOperationV1::Grant,
                Some(DirectoryActionV1::Read),
                2,
                None,
                Some(installed.receipt_id),
            )
            .await;
            let root = format!("/companies/{company}/people");
            let denied = readonly_document(
                &pool,
                &app,
                &cookies,
                &format!("{root}/new"),
                StatusCode::NOT_FOUND,
            )
            .await;
            let html = native_entry_html(&denied, StatusCode::NOT_FOUND);
            assert!(!html.contains("<form") && !html.contains("csrf_proof"));
            // Same Account legitimately receives Create through its real owner;
            // existing registration/preflight oracle requires exactly one pair
            // of AccountCsrf buckets and no unrelated durable effects.
            transition(
                &pool,
                &app,
                &cookies,
                &store,
                &policy,
                company,
                actor,
                NativeBusinessOperationV1::Grant,
                Some(DirectoryActionV1::Create),
                3,
                None,
                Some(read.receipt_id),
            )
            .await;
            let fields = registration(&pool, &app, &cookies, &root).await;
            assert!(
                !fields
                    .iter()
                    .find(|(key, _)| key == "csrf_proof")
                    .unwrap()
                    .1
                    .is_empty()
            );
        })
        .catch_unwind()
        .await;
        if let Some(runtime) = cleanup {
            runtime.close().await;
        }
        close_states(&[state], outcome).await;
    }

    // Real HTTP owner oracle. No UI seed/stub or synthetic owner response.
    fn conflict_recovery(
        response: &Response,
        root: &str,
        command: Uuid,
        proof: &str,
        name: &str,
        require_original: bool,
    ) {
        assert_eq!(response.status, StatusCode::CONFLICT);
        let html = std::str::from_utf8(&response.bytes).unwrap();
        let main = html
            .split("<main ")
            .nth(1)
            .unwrap()
            .split("</main>")
            .next()
            .unwrap();
        assert!(
            !main.contains("<form") && !main.contains("type=\"submit\""),
            "known conflict must not offer a stale primary submission"
        );
        assert!(
            !html.contains(proof),
            "known conflict must not expose a reusable proof"
        );
        let original = format!("{root}/requests/{command}");
        let fresh = format!("{root}/new");
        let paths = if require_original {
            vec![original.as_str(), fresh.as_str()]
        } else {
            vec![fresh.as_str()]
        };
        for path in paths {
            let links: Vec<_> = main
                .split("<a ")
                .skip(1)
                .filter(|tail| {
                    tail.split('>')
                        .next()
                        .unwrap()
                        .contains(&format!("href=\"{path}\""))
                })
                .collect();
            assert!(
                !links.is_empty(),
                "recovery link must be beside conflict, not only global navigation: {path}"
            );
            assert!(links.iter().any(|link| {
                !link
                    .split("</a>")
                    .next()
                    .unwrap()
                    .split_once('>')
                    .unwrap()
                    .1
                    .trim()
                    .is_empty()
            }));
        }
        assert!(
            main.contains(name) && main.contains(NUMBER),
            "retain submitted values for deliberate recovery"
        );
        assert!(!html.contains("<script") && !html.contains("/pkg/"));
        native_entry_html(response, StatusCode::CONFLICT);
    }

    #[sqlx::test(migrations = false)]
    async fn directory_http_conflict_recovery_preserves_capacity_retry_and_original_request(
        pool: PgPool,
    ) {
        let (app, key, state) = configured_native_directory_fixture(&pool).await;
        let mut cleanup = None;
        let outcome = AssertUnwindSafe(async {
            let (app, cookies, created) = create_owned_company(&pool, app).await;
            let (verifier, issuer, ttl) = bindings(&account_browser_config(
                &pool,
                app._artifacts.root.clone(),
                &key,
            ));
            let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
            cleanup = Some(runtime.clone());
            let store = PgOrgStore::new(runtime).with_native_account_policy(verifier, issuer, ttl);
            let policy = CompanyPolicy::new().unwrap();
            let company = OrgId::from_uuid(created.company);
            let actor = AccountId::from_uuid(created.administrator).unwrap();
            let (previous, read_assignment, _) =
                install_and_grant(&pool, &app, &cookies, &store, &policy, company, actor).await;
            let root = format!("/companies/{company}/people");
            // Obtain a real form once. Each subsequent request has a distinct
            // command identity, with the exact returned proof/expectations.
            let initial = registration(&pool, &app, &cookies, &root).await;
            let mut submissions = Vec::new();
            for _ in 0..16 {
                let mut fields = initial.clone();
                let command = Uuid::new_v4();
                change(&mut fields, "command_id", &command.to_string());
                let before = all_rows(&pool).await;
                redirect(
                    &post(&app, &format!("{root}/requests"), &cookies, &fields).await,
                    &format!("{root}/requests/{command}"),
                );
                accepted(&before, &all_rows(&pool).await, company, actor, &fields);
                submissions.push((command, fields));
            }
            let mut capacity_fields = initial.clone();
            let capacity_command = Uuid::new_v4();
            change(
                &mut capacity_fields,
                "command_id",
                &capacity_command.to_string(),
            );
            let before = all_rows(&pool).await;
            let mut capacity = post(
                &app,
                &format!("{root}/requests"),
                &cookies,
                &capacity_fields,
            )
            .await;
            assert_eq!(capacity.status, StatusCode::TOO_MANY_REQUESTS);
            assert!(
                before == all_rows(&pool).await,
                "capacity rejection changed durable state"
            );
            let proof = capacity_fields
                .iter()
                .find(|(key, _)| key == "csrf_proof")
                .unwrap()
                .1
                .clone();
            // Ordinary retry deliberately returns the original proof exactly
            // once in its body, never in response headers (existing validation contract).
            assert_eq!(
                capacity
                    .bytes
                    .windows(proof.len())
                    .filter(|window| *window == proof.as_bytes())
                    .count(),
                1
            );
            for (_, value) in &capacity.headers {
                assert!(
                    !value
                        .as_bytes()
                        .windows(proof.len())
                        .any(|window| window == proof.as_bytes())
                );
            }
            capacity.sent_secrets.retain(|secret| secret != &proof);
            let html = native_entry_html(&capacity, StatusCode::TOO_MANY_REQUESTS);
            let form = operation(html, "prepare");
            assert!(
                form.split('>')
                    .next()
                    .unwrap()
                    .contains(&format!("action=\"{root}/requests\""))
            );
            assert!(form.contains("type=\"submit\""));
            for (key, value) in &capacity_fields {
                if key == "legal_name" {
                    // field() intentionally accepts only unescaped canonical
                    // policy fields. Check this known display value separately.
                    assert_eq!(value, NAME);
                    let inputs: Vec<_> = form
                        .split("<input")
                        .skip(1)
                        .map(|tag| tag.split('>').next().unwrap())
                        .filter(|tag| tag.contains("name=\"legal_name\""))
                        .collect();
                    assert_eq!(inputs.len(), 1, "legal name input missing/duplicated");
                    let values: Vec<_> = inputs[0]
                        .split("value=\"")
                        .skip(1)
                        .map(|attribute| attribute.split('"').next().unwrap())
                        .collect();
                    assert_eq!(values.len(), 1, "legal name value missing/duplicated");
                    assert_eq!(values[0], "김하늘 &lt;연구 &amp; 운영&gt;");
                } else {
                    assert_eq!(field(form, key), *value, "capacity replaced original {key}");
                }
            }
            // Free one slot through its real cancel owner, then retry the exact
            // form bytes/identity that previously received Capacity.
            let (cancelled, _) = &submissions[0];
            let cancelled_path = format!("{root}/requests/{cancelled}");
            let cancellation = pending(&pool, &app, &cookies, &cancelled_path, *cancelled).await;
            let before_cancel = all_rows(&pool).await;
            redirect(
                &post(
                    &app,
                    &format!("{cancelled_path}/cancel"),
                    &cookies,
                    &cancellation,
                )
                .await,
                &cancelled_path,
            );
            exact_delta(
                &before_cancel,
                &all_rows(&pool).await,
                &[("native_people_terminals_v1", 1), ("audit_events", 1)],
            );
            let before_retry = all_rows(&pool).await;
            let capacity_path = format!("{root}/requests/{capacity_command}");
            redirect(
                &post(
                    &app,
                    &format!("{root}/requests"),
                    &cookies,
                    &capacity_fields,
                )
                .await,
                &capacity_path,
            );
            accepted(
                &before_retry,
                &all_rows(&pool).await,
                company,
                actor,
                &capacity_fields,
            );
            // Same accepted command, changed input is a known typed Conflict.
            let mut conflicting = capacity_fields.clone();
            change(&mut conflicting, "legal_name", "박충돌");
            let before_conflict = all_rows(&pool).await;
            let response = post(&app, &format!("{root}/requests"), &cookies, &conflicting).await;
            assert!(
                before_conflict == all_rows(&pool).await,
                "conflict rewrote acknowledged history"
            );
            conflict_recovery(&response, &root, capacity_command, &proof, "박충돌", true);
            let _ = pending(&pool, &app, &cookies, &capacity_path, capacity_command).await;
            // Current Create remains allowed after a genuine Read revocation,
            // but the form's old Company epoch must never be silently replaced.
            let stale = registration(&pool, &app, &cookies, &root).await;
            let stale_command: Uuid = stale
                .iter()
                .find(|(key, _)| key == "command_id")
                .unwrap()
                .1
                .parse()
                .unwrap();
            transition(
                &pool,
                &app,
                &cookies,
                &store,
                &policy,
                company,
                actor,
                NativeBusinessOperationV1::Revoke,
                Some(DirectoryActionV1::Read),
                4,
                Some(read_assignment),
                Some(previous),
            )
            .await;
            let before_stale = all_rows(&pool).await;
            let response = post(&app, &format!("{root}/requests"), &cookies, &stale).await;
            assert!(
                before_stale == all_rows(&pool).await,
                "stale submission mutated state"
            );
            let stale_proof = stale
                .iter()
                .find(|(key, _)| key == "csrf_proof")
                .unwrap()
                .1
                .as_str();
            // This fresh stale command was never accepted. Do not mandate a
            // dead original-record link; generic Conflict is not existence proof.
            conflict_recovery(
                &response,
                &root,
                stale_command,
                stale_proof,
                "연구 &amp; 운영",
                false,
            );
            let fresh = registration(&pool, &app, &cookies, &root).await;
            for name in ["command_id", "expected_company_epoch"] {
                assert_ne!(
                    fresh.iter().find(|(key, _)| key == name).unwrap().1,
                    stale.iter().find(|(key, _)| key == name).unwrap().1,
                    "new request did not refresh {name}"
                );
            }
        })
        .catch_unwind()
        .await;
        if let Some(runtime) = cleanup {
            runtime.close().await;
        }
        close_states(&[state], outcome).await;
    }

    // Additive actual HTTP oracle. No new authority fixture or direct business writes.
    fn missing_request_recovery(response: &Response, root: &str, command: Uuid) -> String {
        let html = native_entry_html(response, StatusCode::NOT_FOUND);
        let main = html
            .split("<main ")
            .nth(1)
            .unwrap()
            .split("</main>")
            .next()
            .unwrap();
        assert!(main.contains("data-people-outcome=\"not-visible\""));
        assert!(main.contains("이 화면은 등록이 실패했거나 취소되었다는 뜻이 아닙니다."));
        for absent in [
            "<form",
            "csrf_proof",
            "type=\"submit\"",
            "접수 시각",
            "접수 기록",
        ] {
            assert!(
                !html.contains(absent),
                "missing status exposed mutation/acceptance material"
            );
        }
        for (path, label) in [
            (
                format!("{root}/requests/{command}"),
                "같은 요청 상태 다시 확인",
            ),
            (format!("{root}/new"), "새 등록 요청 작성"),
        ] {
            let links: Vec<_> = main
                .split("<a ")
                .skip(1)
                .filter(|tail| {
                    tail.split('>')
                        .next()
                        .unwrap()
                        .contains(&format!("href=\"{path}\""))
                })
                .collect();
            assert_eq!(links.len(), 1, "local recovery link must be unique: {path}");
            assert!(links[0].split("</a>").next().unwrap().contains(label));
        }
        assert!(!html.contains("<script") && !html.contains("/pkg/"));
        html.to_owned()
    }

    fn no_request_disclosure(response: &Response, secrets: &[&str]) {
        let html = native_entry_html(response, StatusCode::NOT_FOUND);
        for secret in secrets {
            assert!(!secret.is_empty());
            assert!(!html.contains(secret), "404 disclosed request data");
            for (_, value) in &response.headers {
                assert!(
                    !value
                        .as_bytes()
                        .windows(secret.len())
                        .any(|w| w == secret.as_bytes()),
                    "404 header disclosed request data"
                );
            }
        }
        assert!(!html.contains("<form") && !html.contains("csrf_proof"));
    }

    #[sqlx::test(migrations = false)]
    async fn directory_http_missing_request_recovery_preserves_404_and_current_authority(
        pool: PgPool,
    ) {
        let (app, key, state) = configured_native_directory_fixture(&pool).await;
        let mut cleanup = None;
        let outcome = AssertUnwindSafe(async {
            let (app, cookies, created) = create_owned_company(&pool, app).await;
            let (verifier, issuer, ttl) = bindings(&account_browser_config(
                &pool,
                app._artifacts.root.clone(),
                &key,
            ));
            let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
            cleanup = Some(runtime.clone());
            let store = PgOrgStore::new(runtime).with_native_account_policy(verifier, issuer, ttl);
            let policy = CompanyPolicy::new().unwrap();
            let company = OrgId::from_uuid(created.company);
            let actor = AccountId::from_uuid(created.administrator).unwrap();
            let (previous, _, create_assignment) =
                install_and_grant(&pool, &app, &cookies, &store, &policy, company, actor).await;
            let root = format!("/companies/{company}/people");
            // Actual owner form issues a locator, but no prepare is submitted for it.
            let unsent = registration(&pool, &app, &cookies, &root).await;
            let unsent_command: Uuid = unsent
                .iter()
                .find(|(k, _)| k == "command_id")
                .unwrap()
                .1
                .parse()
                .unwrap();
            let unsent_path = format!("{root}/requests/{unsent_command}");
            let unsent_proof = &unsent.iter().find(|(k, _)| k == "csrf_proof").unwrap().1;
            let missing =
                readonly_document(&pool, &app, &cookies, &unsent_path, StatusCode::NOT_FOUND).await;
            let missing_html = missing_request_recovery(&missing, &root, unsent_command);
            no_request_disclosure(&missing, &[NAME, NUMBER, unsent_proof]);
            let again =
                readonly_document(&pool, &app, &cookies, &unsent_path, StatusCode::NOT_FOUND).await;
            assert!(
                missing.bytes == again.bytes,
                "status recheck changed missing-request representation"
            );
            // Follow the asserted fresh URL through the real owner. Its form is new;
            // neither the GET nor the prior404 claims the original operation rolled back.
            let submitted = registration(&pool, &app, &cookies, &root).await;
            let command: Uuid = submitted
                .iter()
                .find(|(k, _)| k == "command_id")
                .unwrap()
                .1
                .parse()
                .unwrap();
            assert_ne!(command, unsent_command);
            let path = format!("{root}/requests/{command}");
            let proof = &submitted.iter().find(|(k, _)| k == "csrf_proof").unwrap().1;
            let before = all_rows(&pool).await;
            redirect(
                &post(&app, &format!("{root}/requests"), &cookies, &submitted).await,
                &path,
            );
            accepted(&before, &all_rows(&pool).await, company, actor, &submitted);
            let _ = pending(&pool, &app, &cookies, &path, command).await;
            let missing_after =
                readonly_document(&pool, &app, &cookies, &unsent_path, StatusCode::NOT_FOUND).await;
            assert!(
                missing_html == missing_request_recovery(&missing_after, &root, unsent_command),
                "unrelated accepted request altered missing-request disclosure"
            );
            no_request_disclosure(
                &missing_after,
                &[NAME, "김하늘 &lt;연구 &amp; 운영&gt;", NUMBER, proof],
            );
            // This foreign Account is genuinely unauthorized for this Company.
            // It is not a substitute for the unavailable second-authorized-actor case.
            let (_, outsider) = enrolled(&app).await;
            let foreign_known =
                readonly_document(&pool, &app, &outsider, &path, StatusCode::NOT_FOUND).await;
            let foreign_missing =
                readonly_document(&pool, &app, &outsider, &unsent_path, StatusCode::NOT_FOUND)
                    .await;
            for response in [&foreign_known, &foreign_missing] {
                no_request_disclosure(
                    response,
                    &[NAME, "김하늘 &lt;연구 &amp; 운영&gt;", NUMBER, proof],
                );
                assert!(
                    !std::str::from_utf8(&response.bytes)
                        .unwrap()
                        .contains("data-people-outcome=\"not-visible\"")
                );
            }
            for response in [&foreign_known, &foreign_missing] {
                let html = std::str::from_utf8(&response.bytes).unwrap();
                assert!(html.contains("<h1>이 페이지를 열 수 없습니다</h1>"));
                assert!(html.contains("href=\"/account\""));
                no_request_disclosure(
                    response,
                    &[
                        &company.to_string(),
                        "연결된 업무 회사",
                        &format!("/companies/{company}"),
                        &root,
                        &format!("/companies/{company}/policy"),
                    ],
                );
            }
            assert!(
                foreign_known.bytes == foreign_missing.bytes,
                "unauthorized404 revealed request existence"
            );
            // Read remains separately granted; losing Create must deny request status.
            transition(
                &pool,
                &app,
                &cookies,
                &store,
                &policy,
                company,
                actor,
                NativeBusinessOperationV1::Revoke,
                Some(DirectoryActionV1::Create),
                4,
                Some(create_assignment),
                Some(previous),
            )
            .await;
            let denied_known =
                readonly_document(&pool, &app, &cookies, &path, StatusCode::NOT_FOUND).await;
            let denied_missing =
                readonly_document(&pool, &app, &cookies, &unsent_path, StatusCode::NOT_FOUND).await;
            for response in [&denied_known, &denied_missing] {
                no_request_disclosure(
                    response,
                    &[NAME, "김하늘 &lt;연구 &amp; 운영&gt;", NUMBER, proof],
                );
                let html = std::str::from_utf8(&response.bytes).unwrap();
                assert!(!html.contains("data-people-outcome=\"not-visible\""));
                assert!(!html.contains(&path) && !html.contains(&unsent_path));
            }
            assert!(
                denied_known.bytes == denied_missing.bytes,
                "revoked404 revealed request existence"
            );
        })
        .catch_unwind()
        .await;
        if let Some(runtime) = cleanup {
            runtime.close().await;
        }
        close_states(&[state], outcome).await;
    }
}
