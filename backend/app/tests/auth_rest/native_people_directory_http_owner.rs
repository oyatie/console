// Include inside native_people_policy_http_owner. Isolated actual HTTP/database
// evidence, not browser or production qualification. No fixture business SQL.
mod directory_http {
    use super::super::native_people_directory_finalizer_tests::configured_native_directory_fixture;
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
}
