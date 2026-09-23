// Included inside deployment_operator_designation; genuine startup designation,
// ordinary native enrollment, mounted HTTP routes and restricted product owners.
pub(crate) mod company_setup {
    use super::*;

    const ENTRY: &str = "/account/companies/new";
    const CREATE: &str = "/api/v2/companies/enroll";
    const REQUIRED_TABLES: &[&str] = &[
        "_sqlx_migrations",
        "accounts",
        "account_security",
        "company_actors",
        "account_context_candidates",
        "organizations",
        "groups",
        "group_memberships",
        "group_role_grants",
        "users",
        "policy_roles",
        "user_role_assignments",
        "deployment_operator_head",
        "deployment_operator_receipts",
        "auth_refresh_tokens",
        "auth_refresh_token_families",
        "auth_webauthn_credentials",
        "audit_events",
    ];
    const INITIAL_ACTIONS: &[&str] = &[
        "company.identity.read",
        "company.policy.assign",
        "company.policy.read",
        "company.policy.revoke",
        "context.discover",
    ];
    const DELEGABLE_ACTIONS: &[&str] = &["company.identity.read", "context.discover"];
    const COMPANY_FIELDS: &[&str] = &["company.name", "company.slug"];

    fn complete(tables: &[String], captured: &BTreeMap<String, String>) -> bool {
        let census: BTreeSet<_> = tables.iter().map(String::as_str).collect();
        !census.is_empty()
            && census.len() == tables.len()
            && REQUIRED_TABLES.iter().all(|name| census.contains(name))
            && captured.keys().map(String::as_str).collect::<BTreeSet<_>>() == census
            && captured.values().all(|raw| {
                serde_json::from_str::<Value>(raw).is_ok_and(|v| {
                    v.as_array()
                        .is_some_and(|rows| rows.iter().all(Value::is_object))
                })
            })
            && captured.get("_sqlx_migrations").is_some_and(|raw| {
                serde_json::from_str::<Value>(raw)
                    .is_ok_and(|v| v.as_array().is_some_and(|a| !a.is_empty()))
            })
    }

    pub(crate) async fn all_rows(pool: &PgPool) -> BTreeMap<String, String> {
        // Actual complete public base-table census, including later owner tables.
        // Only trusted catalog identifiers enter SQL; exact JSON text preserves
        // PostgreSQL numeric fidelity. No raw rows appear in assertion output.
        let tables: Vec<String> = sqlx::query_scalar("SELECT c.relname::text FROM pg_class c WHERE c.relnamespace='public'::regnamespace AND c.relkind IN ('r','p') ORDER BY c.relname COLLATE \"C\"")
            .fetch_all(pool).await.unwrap();
        assert!(
            tables.len() <= 1024,
            "fixture table census exceeds reviewed bound"
        );
        let mut result = BTreeMap::new();
        for table in &tables {
            let quoted = table.replace('"', "\"\"");
            let query = format!(
                "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text COLLATE \"C\"),'[]'::jsonb)::text FROM public.\"{quoted}\" t"
            );
            let rows: String = sqlx::query_scalar(sqlx::AssertSqlSafe(query))
                .fetch_one(pool)
                .await
                .unwrap();
            assert!(result.insert(table.clone(), rows).is_none());
        }
        assert!(
            complete(&tables, &result),
            "complete real table snapshot required"
        );
        result
    }

    // Byte-preserving history subtraction. The snapshot is an independent complete
    // table read, never a response projection or a list of expected changed rows.
    fn added_rows(before: &str, after: &str) -> Option<Vec<Value>> {
        use serde_json::value::RawValue;
        let before: Vec<&RawValue> = serde_json::from_str(before).ok()?;
        let after: Vec<&RawValue> = serde_json::from_str(after).ok()?;
        let mut retained = BTreeSet::new();
        for row in before {
            if !row.get().trim_start().starts_with('{') || !retained.insert(row.get()) {
                return None;
            }
        }
        let mut seen = BTreeSet::new();
        let mut added = Vec::new();
        for row in after {
            if !row.get().trim_start().starts_with('{') || !seen.insert(row.get()) {
                return None;
            }
            if !retained.remove(row.get()) {
                added.push(serde_json::from_str(row.get()).ok()?);
            }
        }
        retained.is_empty().then_some(added)
    }

    #[test]
    fn company_history_oracle_rejects_omission_mutation_duplicate_and_scalar_rows() {
        assert!(
            added_rows(r#"[{"a":1}]"#, r#"[{"b":2},{"a":1}]"#)
                .is_some_and(|v| v == vec![json!({"b":2})])
        );
        for (before, after) in [
            (r#"[{"a":1}]"#, "[]"),
            (r#"[{"a":1}]"#, r#"[{"a":2}]"#),
            ("[]", r#"[{"a":1},{"a":1}]"#),
            (r#"[{"a":1},{"a":1}]"#, r#"[{"a":1}]"#),
            ("[]", "[null]"),
            ("[null]", "[null]"),
        ] {
            assert!(added_rows(before, after).is_none());
        }
    }

    #[test]
    fn company_snapshot_oracle_rejects_missing_table_non_array_and_empty_ledger() {
        let census: Vec<_> = REQUIRED_TABLES.iter().map(|s| (*s).to_owned()).collect();
        let rows: BTreeMap<_, _> = census
            .iter()
            .map(|name| {
                (
                    name.clone(),
                    if name == "_sqlx_migrations" {
                        "[{\"version\":1}]"
                    } else {
                        "[]"
                    }
                    .to_owned(),
                )
            })
            .collect();
        assert!(complete(&census, &rows));
        for name in &census {
            let mut missing = rows.clone();
            missing.remove(name);
            assert!(!complete(&census, &missing));
            let mut invalid = rows.clone();
            invalid.insert(name.clone(), "null".into());
            assert!(!complete(&census, &invalid));
            for scalar_rows in ["[null]", "[0]", "[\"text\"]", "[[]]"] {
                let mut invalid_rows = rows.clone();
                invalid_rows.insert(name.clone(), scalar_rows.into());
                assert!(!complete(&census, &invalid_rows));
            }
        }
        let mut empty = rows.clone();
        empty.insert("_sqlx_migrations".into(), "[]".into());
        assert!(!complete(&census, &empty));
        let mut duplicate = census.clone();
        duplicate.push(census[0].clone());
        assert!(!complete(&duplicate, &rows));
    }

    // Original Account-only prerequisite remains for historical intake controls.
    async fn designated(pool: &PgPool) -> (Fixture, Attempt, Cookies, PgPool, Designation) {
        designated_fixture(pool, fixture(pool).await).await
    }

    async fn prepare_ready_database(pool: &PgPool) {
        prepare_http_database(pool).await;
        let mut tx = pool.begin().await.unwrap();
        sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL READ COMMITTED; SET LOCAL search_path=pg_catalog,pg_temp")
            .execute(tx.as_mut()).await.unwrap();
        let source = include_str!("../../../../ops/postgres-finalize-company-enrollment.sql");
        assert_eq!(
            hex::encode(sha2::Sha256::digest(source.as_bytes())),
            "bc35b52d5692e474a3c890dde73112f56e7b85a241ab390075d58b5d5b43a92f"
        );
        sqlx::raw_sql(source).execute(tx.as_mut()).await.unwrap();
        tx.commit().await.unwrap();
        seed_terms(pool).await;
    }

    async fn ready_fixture(pool: &PgPool) -> Fixture {
        prepare_ready_database(pool).await;
        let artifacts = Artifacts::new();
        // Actual App startup verifies the Company profile before enrollment.
        let service = router(pool, artifacts.root.clone()).await;
        Fixture {
            service,
            _artifacts: artifacts,
            pool: pool.clone(),
        }
    }

    async fn ready_designated(pool: &PgPool) -> (Fixture, Attempt, Cookies, PgPool, Designation) {
        designated_fixture(pool, ready_fixture(pool).await).await
    }

    async fn designated_fixture(
        pool: &PgPool,
        app: Fixture,
    ) -> (Fixture, Attempt, Cookies, PgPool, Designation) {
        let (account, cookies) = enrolled(&app).await;
        let startup = startup(pool).await;
        let input = designation(pool, account.account).await;
        let receipt = designate(&startup, &input).await.unwrap();
        assert!(!receipt.0.is_nil() && receipt.1 == 1 && !receipt.2);
        let bound: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM public.deployment_operator_head h JOIN public.deployment_operator_receipts r ON r.receipt_id=h.receipt_id WHERE h.account_id=$1 AND r.account_id=$1 AND r.command_id=$2 AND r.receipt_id=$3 AND r.kind='DESIGNATE' AND h.revision=1)")
            .bind(account.account).bind(input.command).bind(receipt.0).fetch_one(pool).await.unwrap();
        assert!(bound, "genuine current designation prerequisite");
        assert_no_company_identity(pool, account.account).await;
        (app, account, cookies, startup, input)
    }

    async fn document(app: &Fixture, target: &str, cookies: &Cookies) -> Response {
        native_entry_get(
            app,
            target,
            cookies,
            "same-origin",
            "navigate",
            "document",
            &[],
        )
        .await
    }

    fn enrollment(command: Uuid, administrator: Uuid) -> Value {
        json!({"command_id":command,"group_id":null,"slug":format!("native-{}",command.simple()),"name":"연결된 업무 회사","administrative_account_id":administrator})
    }

    async fn submit(app: &Fixture, cookies: &Cookies, csrf: &str, input: &Value) -> Response {
        request(
            app,
            "POST",
            CREATE,
            cookies,
            Some(input.clone()),
            &[("X-Console-CSRF", &csrf)],
        )
        .await
    }

    async fn submit_raw(app: &Fixture, cookies: &Cookies, csrf: &str, body: String) -> Response {
        let mut sent_secrets: Vec<_> = cookies.0.values().cloned().collect();
        sent_secrets.push(csrf.to_owned());
        let mut request = Request::builder()
            .method("POST")
            .uri(CREATE)
            .header(header::ORIGIN, TEST_ORIGIN)
            .header("Sec-Fetch-Site", "same-origin")
            .header(header::COOKIE, cookies.header())
            .header(header::CONTENT_TYPE, "application/json")
            .header("X-Console-CSRF", csrf)
            .body(Body::from(body))
            .unwrap();
        request.extensions_mut().insert(ConnectInfo(
            "127.0.0.1:41000".parse::<SocketAddr>().unwrap(),
        ));
        let (parts, body) = app
            .service
            .clone()
            .oneshot(request)
            .await
            .unwrap()
            .into_parts();
        Response {
            status: parts.status,
            headers: parts.headers,
            bytes: to_bytes(body, 256 * 1024).await.unwrap().to_vec(),
            sent_secrets,
        }
    }

    #[derive(Clone)]
    struct Committed {
        command: Uuid,
        receipt: Uuid,
        company: Uuid,
        group: Uuid,
        administrator: Uuid,
        result_path: String,
    }

    fn committed(
        response: &Response,
        status: StatusCode,
        command: Uuid,
        administrator: Uuid,
        replayed: bool,
    ) -> Committed {
        let data = response.json(status);
        response.private();
        exact_keys(
            &data,
            &[
                "outcome",
                "original_command_id",
                "receipt_id",
                "org_id",
                "group_id",
                "administrative_account_id",
                "replayed",
                "result_path",
            ],
        );
        let id = |key: &str| {
            let id: Uuid = data[key].as_str().unwrap().parse().unwrap();
            assert!(!id.is_nil());
            id
        };
        let result = Committed {
            command: id("original_command_id"),
            receipt: id("receipt_id"),
            company: id("org_id"),
            group: id("group_id"),
            administrator: id("administrative_account_id"),
            result_path: data["result_path"].as_str().unwrap().to_owned(),
        };
        assert!(data["outcome"] == "COMMITTED" && data["replayed"] == replayed);
        assert!(result.command == command && result.administrator == administrator);
        assert!(result.result_path == format!("/account/companies/requests/{command}"));
        assert!(
            !response.headers.contains_key(header::SET_COOKIE),
            "Company setup cannot issue or replace Account credentials"
        );
        result
    }

    async fn durable(pool: &PgPool, result: &Committed, input: &Value, original: Uuid) {
        let found: (Uuid, String, String) =
            sqlx::query_as("SELECT group_id,slug,name FROM public.organizations WHERE id=$1")
                .bind(result.company)
                .fetch_one(pool)
                .await
                .unwrap();
        assert!(
            found.0 == result.group
                && json!(found.1) == input["slug"]
                && json!(found.2) == input["name"]
        );
        let topology: (i64,i64) = sqlx::query_as("SELECT (SELECT count(*) FROM public.groups WHERE id=$1),(SELECT count(*) FROM public.group_memberships WHERE group_id=$1 AND org_id=$2)")
            .bind(result.group).bind(result.company).fetch_one(pool).await.unwrap();
        assert!(
            topology == (1, 1),
            "one real Group and current Company membership"
        );
        let actors: i64 = sqlx::query_scalar("SELECT count(*) FROM public.company_actors WHERE org_id=$1 AND account_id=$2 AND admission_receipt_id=$3")
            .bind(result.company).bind(result.administrator).bind(result.receipt).fetch_one(pool).await.unwrap();
        assert_eq!(actors, 1, "actual admitted administrative CompanyActor");
        let no_legacy: bool =
            sqlx::query_scalar("SELECT NOT EXISTS(SELECT 1 FROM public.users WHERE id=ANY($1))")
                .bind(vec![original, result.administrator])
                .fetch_one(pool)
                .await
                .unwrap();
        assert!(no_legacy, "native Company enrollment invented legacy users");
        // Exact proposed provisioning receipt table is part of the transport
        // refinement, not a fixture-created table or future Rust API import.
        let receipt: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM public.company_enrollment_receipts r JOIN public.company_enrollment_requests q ON (q.account_id,q.command_id,q.committed_receipt_id)=(r.account_id,r.command_id,r.receipt_id) JOIN public.deployment_operator_receipts d ON d.receipt_id=r.designation_receipt_id WHERE q.state='COMMITTED' AND q.input_digest IS NOT NULL AND r.input_digest IS NOT NULL AND q.input_digest=r.input_digest AND q.designation_receipt_id=r.designation_receipt_id AND r.command_id=$1 AND r.account_id=$2 AND r.receipt_id=$3 AND r.org_id=$4 AND r.group_id=$5 AND r.administrative_account_id=$6 AND d.account_id=$2 AND d.kind='DESIGNATE')")
            .bind(result.command).bind(original).bind(result.receipt).bind(result.company).bind(result.group).bind(result.administrator).fetch_one(pool).await.unwrap();
        assert!(
            receipt,
            "committed result must have same-command COMMITTED request, exact terminal receipt, identical nonnull input digest and Account/designation provenance"
        );
    }

    fn exact_string_set(value: &Value, expected: &[&str]) {
        let values = value.as_array().expect("bounded registration key list");
        let actual: BTreeSet<_> = values
            .iter()
            .map(|v| v.as_str().expect("registered key"))
            .collect();
        assert!(
            actual.len() == values.len() && actual == expected.iter().copied().collect(),
            "finite capability roster differs"
        );
    }

    // This HTTP roster oracle is intentionally not proof of registered-ref custody.
    // Exact ActionRef/PropertyRef + durable PA1.2 binding remains a separate gate.
    async fn initial_ceiling(app: &Fixture, cookies: &Cookies, result: &Committed) {
        let response = request(
            app,
            "GET",
            &format!("/api/v2/companies/{}/policy", result.company),
            cookies,
            None,
            &[],
        )
        .await;
        let body = response.json(StatusCode::OK);
        response.private();
        // Keys are display addresses only; real corresponding registered refs
        // must accompany every action/property in the ordinary owner projection.
        let ceiling = &body["initial_ceiling"];
        assert!(
            ceiling["org_id"] == json!(result.company)
                && ceiling["account_id"] == json!(result.administrator)
        );
        exact_string_set(&ceiling["action_keys"], INITIAL_ACTIONS);
        exact_string_set(&ceiling["delegable_action_keys"], DELEGABLE_ACTIONS);
        exact_string_set(&ceiling["company_property_keys"], COMPANY_FIELDS);
        exact_string_set(&ceiling["delegable_company_property_keys"], COMPANY_FIELDS);
        assert!(ceiling["future_registrations"] == false && ceiling["group_control"] == false);
    }

    #[sqlx::test(migrations = false)]
    async fn designated_account_discovers_mounted_company_setup_without_business_identity(
        pool: PgPool,
    ) {
        let (app, account, cookies, startup, input) = designated(&pool).await;
        let before = all_rows(&pool).await;
        let response = document(&app, "/account", &cookies).await;
        let html = native_entry_html(&response, StatusCode::OK);
        assert!(html.contains("data-account-state=\"active\""));
        assert!(
            html.contains("href=\"/account/companies/new\""),
            "NATIVE_COMPANY_ENTRY: current designated Account cannot discover actual setup"
        );
        assert!(html.contains("회사 업무 공간 만들기"));
        let form = document(&app, ENTRY, &cookies).await;
        let html = native_entry_html(&form, StatusCode::OK);
        assert!(
            html.contains("name=\"name\"")
                && html.contains("name=\"slug\"")
                && html.contains("회사 이름")
                && html.contains("업무 공간 식별자")
        );
        assert!(
            html.contains("내 계정")
                && html.contains("data-company-setup")
                && html.contains("기존 회사가 사용할 콘솔 업무 공간을 등록합니다.")
        );
        for disallowed in [
            "SUPER_ADMIN",
            "PLATFORM_ADMIN",
            "name=\"role\"",
            "name=\"root_secret\"",
        ] {
            assert!(
                !html.contains(disallowed),
                "setup exposed legacy/root authority choices"
            );
        }
        assert_no_company_identity(&pool, account.account).await;
        assert!(
            before == all_rows(&pool).await,
            "entry/form reads mutated persisted state"
        );
        revoke(
            &startup,
            &input,
            Uuid::new_v4(),
            1,
            "fixture removes current Company create eligibility",
        )
        .await
        .unwrap();
        let revoked = all_rows(&pool).await;
        let response = document(&app, "/account", &cookies).await;
        let html = native_entry_html(&response, StatusCode::OK);
        assert!(!html.contains("href=\"/account/companies/new\""));
        let denied = document(&app, ENTRY, &cookies).await;
        native_entry_html(&denied, StatusCode::NOT_FOUND);
        assert!(revoked == all_rows(&pool).await);
        startup.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn ordinary_account_cannot_get_or_submit_company_setup(pool: PgPool) {
        let app = fixture(&pool).await;
        let (account, cookies) = enrolled(&app).await;
        let csrf = proof(&app, &cookies).await;
        let before = all_rows(&pool).await;
        let response = document(&app, "/account", &cookies).await;
        assert!(
            !native_entry_html(&response, StatusCode::OK)
                .contains("href=\"/account/companies/new\"")
        );
        native_entry_html(
            &document(&app, ENTRY, &cookies).await,
            StatusCode::NOT_FOUND,
        );
        let attempted = enrollment(Uuid::new_v4(), account.account);
        submit(&app, &cookies, &csrf, &attempted)
            .await
            .error(StatusCode::FORBIDDEN, "company_enrollment_forbidden");
        assert!(
            before == all_rows(&pool).await,
            "ordinary Account created Company/grant/receipt through setup"
        );
    }

    #[sqlx::test(migrations = false)]
    async fn company_setup_commits_once_reopens_and_preserves_account_credentials(pool: PgPool) {
        let (app, account, cookies, startup, designation) = ready_designated(&pool).await;
        let csrf = proof(&app, &cookies).await;
        let command = Uuid::new_v4();
        let input = enrollment(command, account.account);
        let prior = all_rows(&pool).await;
        let created = committed(
            &submit(&app, &cookies, &csrf, &input).await,
            StatusCode::CREATED,
            command,
            account.account,
            false,
        );
        durable(&pool, &created, &input, account.account).await;
        initial_ceiling(&app, &cookies, &created).await;
        let after = all_rows(&pool).await;
        for key in [
            "accounts",
            "auth_refresh_token_families",
            "auth_refresh_tokens",
            "auth_webauthn_credentials",
            "account_terms_acceptances",
            "users",
        ] {
            assert!(
                prior[key] == after[key],
                "Company command changed unrelated Account/credential stream"
            );
        }
        let saved = request(
            &app,
            "GET",
            &format!("/api/v2/companies/enrollments/{command}"),
            &cookies,
            None,
            &[],
        )
        .await;
        let reopened = committed(&saved, StatusCode::OK, command, account.account, true);
        assert!(
            created.company == reopened.company
                && created.group == reopened.group
                && created.receipt == reopened.receipt
        );
        let page = document(&app, &created.result_path, &cookies).await;
        let html = native_entry_html(&page, StatusCode::OK);
        assert!(
            html.contains("생성 완료")
                && html.contains(&format!("href=\"/companies/{}\"", created.company))
        );
        let workspace = document(&app, &format!("/companies/{}", created.company), &cookies).await;
        let html = native_entry_html(&workspace, StatusCode::OK);
        assert!(html.contains("연결된 업무 회사") && html.contains("권한 관리"));
        let retried = committed(
            &submit(&app, &cookies, &csrf, &input).await,
            StatusCode::OK,
            command,
            account.account,
            true,
        );
        assert!(
            created.company == retried.company
                && created.group == retried.group
                && created.receipt == retried.receipt
        );
        assert!(
            after == all_rows(&pool).await,
            "reopen/read/retry repeated or changed business effects"
        );
        let mut changed = input.clone();
        changed["name"] = json!("같은 명령의 다른 내용");
        submit(&app, &cookies, &csrf, &changed)
            .await
            .error(StatusCode::CONFLICT, "command_conflict");
        assert!(after == all_rows(&pool).await);
        revoke(
            &startup,
            &designation,
            Uuid::new_v4(),
            1,
            "no more new Company enrollment",
        )
        .await
        .unwrap();
        let revoked = all_rows(&pool).await;
        let historical = committed(
            &submit(&app, &cookies, &csrf, &input).await,
            StatusCode::OK,
            command,
            account.account,
            true,
        );
        assert!(historical.receipt == created.receipt);
        submit(
            &app,
            &cookies,
            &csrf,
            &enrollment(Uuid::new_v4(), account.account),
        )
        .await
        .error(StatusCode::FORBIDDEN, "company_enrollment_forbidden");
        assert!(
            revoked == all_rows(&pool).await,
            "history reconciliation created new effect after designation revocation"
        );
        startup.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn company_setup_rejects_csrf_and_supplied_group_without_topology_authority(
        pool: PgPool,
    ) {
        let (app, account, cookies, startup, _) = designated(&pool).await;
        let csrf = proof(&app, &cookies).await;
        let before = all_rows(&pool).await;
        let input = enrollment(Uuid::new_v4(), account.account);
        request(&app, "POST", CREATE, &cookies, Some(input.clone()), &[])
            .await
            .error(StatusCode::FORBIDDEN, "csrf_invalid");
        let mut targeted = input.clone();
        targeted["group_id"] = json!(Uuid::new_v4());
        submit(&app, &cookies, &csrf, &targeted).await.error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "group_enrollment_unavailable",
        );
        let existing: Uuid = sqlx::query_scalar("SELECT id FROM public.groups ORDER BY id LIMIT 1")
            .fetch_one(&pool)
            .await
            .expect("existing real migration-created Group prerequisite");
        targeted["group_id"] = json!(existing);
        submit(&app, &cookies, &csrf, &targeted).await.error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "group_enrollment_unavailable",
        );
        assert!(
            before == all_rows(&pool).await,
            "forbidden selector/CSRF request created partial enrollment"
        );
        startup.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn selected_administrator_receives_only_company_ceiling_without_operator_inheritance(
        pool: PgPool,
    ) {
        let (app, operator, cookies, startup, _) = ready_designated(&pool).await;
        let csrf = proof(&app, &cookies).await;
        let (recipient, recipient_cookies) = enrolled(&app).await;
        let recipient_csrf = proof(&app, &recipient_cookies).await;
        let input = enrollment(Uuid::new_v4(), recipient.account);
        let command: Uuid = input["command_id"].as_str().unwrap().parse().unwrap();
        let before = all_rows(&pool).await;
        let created = committed(
            &submit(&app, &cookies, &csrf, &input).await,
            StatusCode::CREATED,
            command,
            recipient.account,
            false,
        );
        durable(&pool, &created, &input, operator.account).await;
        initial_ceiling(&app, &recipient_cookies, &created).await;
        let recipient_page = document(
            &app,
            &format!("/companies/{}", created.company),
            &recipient_cookies,
        )
        .await;
        let html = native_entry_html(&recipient_page, StatusCode::OK);
        assert!(html.contains("연결된 업무 회사") && html.contains("권한 관리"));
        let operator_page =
            document(&app, &format!("/companies/{}", created.company), &cookies).await;
        native_entry_html(&operator_page, StatusCode::NOT_FOUND);
        let result_page = document(&app, &created.result_path, &cookies).await;
        let html = native_entry_html(&result_page, StatusCode::OK);
        assert!(
            html.contains("생성 완료")
                && !html.contains("연결된 업무 회사")
                && !html.contains(&format!("href=\"/companies/{}\"", created.company)),
            "original command history must not imply current Company disclosure"
        );
        let after = all_rows(&pool).await;
        for key in [
            "accounts",
            "auth_refresh_token_families",
            "auth_refresh_tokens",
            "auth_webauthn_credentials",
            "account_terms_acceptances",
            "users",
        ] {
            assert!(
                before[key] == after[key],
                "separate administrator changed credential/legacy identity stream"
            );
        }
        submit(
            &app,
            &recipient_cookies,
            &recipient_csrf,
            &enrollment(Uuid::new_v4(), recipient.account),
        )
        .await
        .error(StatusCode::FORBIDDEN, "company_enrollment_forbidden");
        assert!(
            after == all_rows(&pool).await,
            "Company management authority escalated into deployment creation"
        );
        startup.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn failed_actor_admission_rolls_back_topology_and_same_command_retries_once(
        pool: PgPool,
    ) {
        let (app, account, cookies, startup, _) = ready_designated(&pool).await;
        let csrf = proof(&app, &cookies).await;
        let command = Uuid::new_v4();
        let input = enrollment(command, account.account);
        let before = all_rows(&pool).await;
        // Explicit negative fault only, after real topology must exist in the
        // owner's same transaction. No successful business fixture is inserted.
        sqlx::raw_sql(r#"CREATE SEQUENCE public.company_setup_fault_seen;
          CREATE FUNCTION public.company_setup_actor_fault() RETURNS trigger LANGUAGE plpgsql SECURITY DEFINER SET search_path=pg_catalog,pg_temp AS $$
          BEGIN
            IF NOT EXISTS(SELECT 1 FROM public.organizations WHERE id=NEW.org_id) THEN
              RAISE EXCEPTION 'COMPANY_SETUP_TOPOLOGY_NOT_REACHED';
            END IF;
            PERFORM nextval('public.company_setup_fault_seen'::regclass);
            RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='COMPANY_SETUP_AFTER_REAL_TOPOLOGY';
          END $$;
          CREATE TRIGGER company_setup_actor_fault BEFORE INSERT ON public.company_actors
            FOR EACH ROW EXECUTE FUNCTION public.company_setup_actor_fault()"#)
            .execute(&pool).await.unwrap();
        let response = submit(&app, &cookies, &csrf, &input).await;
        // Sequence advancement is deliberate nontransactional fault evidence:
        // the real trigger saw the real Company before refusing. A generic503
        // from unrelated admission failure cannot satisfy this witness.
        let fired: bool = sqlx::query_scalar(
            "SELECT is_called AND last_value=1 FROM public.company_setup_fault_seen",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        // Always remove the precise disposable fault before response assertions.
        sqlx::raw_sql("DROP TRIGGER company_setup_actor_fault ON public.company_actors; DROP FUNCTION public.company_setup_actor_fault(); DROP SEQUENCE public.company_setup_fault_seen")
            .execute(&pool).await.unwrap();
        assert!(
            fired,
            "real Company topology/actor fault boundary was not reached"
        );
        response.error(
            StatusCode::SERVICE_UNAVAILABLE,
            "company_enrollment_unavailable",
        );
        let failed = all_rows(&pool).await;
        let mut before_effects = before.clone();
        let mut after_effects = failed.clone();
        let old_requests = before_effects
            .remove("company_enrollment_requests")
            .expect("admitted request owner must exist before submission");
        let requests = after_effects
            .remove("company_enrollment_requests")
            .expect("durable input request required after composition failure");
        let old_events = before_effects
            .remove("company_enrollment_request_events")
            .expect("admitted request event owner must exist before submission");
        let events = after_effects
            .remove("company_enrollment_request_events")
            .expect("durable preparation event required after composition failure");
        assert!(
            before_effects == after_effects,
            "failed composition retained partial Company/Group/catalog/grant/context/receipt/audit effect"
        );
        let added = added_rows(&old_requests, &requests).expect("request history preserved");
        assert_eq!(added.len(), 1, "exactly one durable input request");
        assert!(
            added[0]["account_id"] == json!(account.account)
                && added[0]["command_id"] == json!(command)
                && added[0]["state"] == "PENDING"
        );
        // Prepare commits exactly one request/event pair before the effect
        // transaction. Neither prior history nor any other effect is excluded.
        let added_events =
            added_rows(&old_events, &events).expect("request event history preserved");
        assert_eq!(
            added_events.len(),
            1,
            "exactly one durable preparation event"
        );
        let tokens: Vec<Value> = serde_json::from_str(&before["auth_refresh_tokens"]).unwrap();
        let token_hash = format!(
            "\\x{}",
            hex::encode(Sha256::digest(cookies.0[REFRESH].as_bytes()))
        );
        let tokens: Vec<_> = tokens
            .iter()
            .filter(|token| token["token_hash"] == token_hash)
            .collect();
        assert_eq!(
            tokens.len(),
            1,
            "submitted cookie must identify one original token"
        );
        let token = tokens[0];
        assert!(
            token["user_id"] == json!(account.account)
                && token["used_at"].is_null()
                && token["revoked_at"].is_null()
        );
        let families: Vec<Value> =
            serde_json::from_str(&before["auth_refresh_token_families"]).unwrap();
        let families: Vec<_> = families
            .iter()
            .filter(|family| family["id"] == token["family_id"])
            .collect();
        assert_eq!(
            families.len(),
            1,
            "original token must identify one native session"
        );
        let family = families[0];
        assert!(
            family["user_id"] == json!(account.account)
                && family["protocol"] == "ACCOUNT_V1"
                && family["revoked_at"].is_null()
        );
        assert!(
            added[0]["created_at"].is_string(),
            "request must retain authoritative creation time"
        );
        assert_eq!(
            added_events[0],
            json!({
                "account_id": account.account,
                "command_id": command,
                "event_revision": 1,
                "from_state": null,
                "to_state": "PENDING",
                "occurred_at": added[0]["created_at"],
                "actor_account_id": account.account,
                "session_id": family["id"],
                "reason_code": "PREPARED"
            }),
            "only the exact request-correlated preparation event may survive"
        );
        let pending = request(
            &app,
            "GET",
            &format!("/api/v2/companies/enrollments/{command}"),
            &cookies,
            None,
            &[],
        )
        .await;
        pending.private();
        let pending = pending.json(StatusCode::OK);
        exact_keys(
            &pending,
            &["outcome", "original_command_id", "input", "result_path"],
        );
        assert!(
            pending["outcome"] == "PENDING"
                && pending["original_command_id"] == json!(command)
                && pending["input"] == input
                && pending["result_path"] == format!("/account/companies/requests/{command}")
        );
        let reopened = document(
            &app,
            &format!("/account/companies/requests/{command}"),
            &cookies,
        )
        .await;
        let reopened = native_entry_html(&reopened, StatusCode::OK);
        assert!(
            reopened.contains("연결된 업무 회사")
                && reopened.contains("다시 시도")
                && !reopened.contains("생성 완료"),
            "durable pending input must reopen without invented success"
        );
        assert!(
            failed == all_rows(&pool).await,
            "pending read modified command or effects"
        );
        let created = committed(
            &submit(&app, &cookies, &csrf, &input).await,
            StatusCode::CREATED,
            command,
            account.account,
            false,
        );
        durable(&pool, &created, &input, account.account).await;
        let after = all_rows(&pool).await;
        let replay = committed(
            &submit(&app, &cookies, &csrf, &input).await,
            StatusCode::OK,
            command,
            account.account,
            true,
        );
        assert!(
            replay.receipt == created.receipt
                && replay.company == created.company
                && replay.group == created.group
        );
        assert!(after == all_rows(&pool).await);
        startup.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn company_input_is_closed_bounded_preserved_and_html_escaped(pool: PgPool) {
        let (app, account, cookies, startup, _) = ready_designated(&pool).await;
        let csrf = proof(&app, &cookies).await;
        let before = all_rows(&pool).await;
        let base = enrollment(Uuid::new_v4(), account.account);
        let mut invalid = Vec::new();
        for name in [
            "".to_owned(),
            " \t\n".to_owned(),
            "가".repeat(86),
            "회사\u{0000}".to_owned(),
        ] {
            let mut value = base.clone();
            value["name"] = json!(name);
            invalid.push(value);
        }
        for slug in [
            "".to_owned(),
            "a".repeat(64),
            "-start".into(),
            "end-".into(),
            "HasCapital".into(),
            "a/b".into(),
            "회사".into(),
        ] {
            let mut value = base.clone();
            value["slug"] = json!(slug);
            invalid.push(value);
        }
        let mut unknown = base.clone();
        unknown["root_secret"] = json!("not-authority");
        invalid.push(unknown);
        let mut nil = base.clone();
        nil["administrative_account_id"] = json!(Uuid::nil());
        invalid.push(nil);
        for value in invalid {
            submit(&app, &cookies, &csrf, &value).await.error(
                StatusCode::UNPROCESSABLE_ENTITY,
                "company_enrollment_invalid",
            );
            assert!(
                before == all_rows(&pool).await,
                "invalid input persisted request or partial effect"
            );
        }
        let encoded = serde_json::to_string(&base).unwrap();
        let duplicate = format!(
            "{},\"name\":\"duplicate field\"}}",
            encoded.trim_end_matches('}')
        );
        submit_raw(&app, &cookies, &csrf, duplicate).await.error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "company_enrollment_invalid",
        );
        assert!(
            before == all_rows(&pool).await,
            "duplicate field input persisted an effect"
        );
        let oversized = format!("{}{}", encoded, " ".repeat(4097));
        submit_raw(&app, &cookies, &csrf, oversized).await.error(
            StatusCode::PAYLOAD_TOO_LARGE,
            "company_enrollment_too_large",
        );
        assert!(
            before == all_rows(&pool).await,
            "oversized body persisted an effect"
        );
        let command = Uuid::new_v4();
        let mut valid = enrollment(command, account.account);
        let name = format!("{} <연구소 & 본사>", "가".repeat(70));
        assert!(name.len() <= 256 && name.len() > 200);
        valid["name"] = json!(name);
        let created = committed(
            &submit(&app, &cookies, &csrf, &valid).await,
            StatusCode::CREATED,
            command,
            account.account,
            false,
        );
        durable(&pool, &created, &valid, account.account).await;
        let page = document(&app, &format!("/companies/{}", created.company), &cookies).await;
        let html = native_entry_html(&page, StatusCode::OK);
        assert!(
            html.contains(&"가".repeat(70))
                && html.contains("&lt;연구소")
                && html.contains("&amp;")
                && !html.contains("<연구소 & 본사>"),
            "valid Korean legal name must persist and escape"
        );
        startup.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn concurrent_same_command_commits_once_and_foreign_account_cannot_reopen(pool: PgPool) {
        let (app, account, cookies, startup, _) = ready_designated(&pool).await;
        let csrf = proof(&app, &cookies).await;
        let (foreign, foreign_cookies) = enrolled(&app).await;
        let foreign_csrf = proof(&app, &foreign_cookies).await;
        let command = Uuid::new_v4();
        let input = enrollment(command, account.account);
        let (left, right) = tokio::join!(
            submit(&app, &cookies, &csrf, &input),
            submit(&app, &cookies, &csrf, &input)
        );
        let (fresh, repeated) = if left.status == StatusCode::CREATED {
            (&left, &right)
        } else {
            (&right, &left)
        };
        let created = committed(fresh, StatusCode::CREATED, command, account.account, false);
        let replayed = committed(repeated, StatusCode::OK, command, account.account, true);
        assert!(
            created.receipt == replayed.receipt
                && created.company == replayed.company
                && created.group == replayed.group
        );
        durable(&pool, &created, &input, account.account).await;
        let count: (i64,i64) = sqlx::query_as("SELECT (SELECT count(*) FROM public.company_enrollment_requests WHERE account_id=$1 AND command_id=$2),(SELECT count(*) FROM public.company_enrollment_receipts WHERE account_id=$1 AND command_id=$2)")
            .bind(account.account).bind(command).fetch_one(&pool).await.unwrap();
        assert_eq!(count, (1, 1));
        let before = all_rows(&pool).await;
        let denied = request(
            &app,
            "GET",
            &format!("/api/v2/companies/enrollments/{command}"),
            &foreign_cookies,
            None,
            &[],
        )
        .await;
        denied.error(StatusCode::NOT_FOUND, "company_enrollment_not_found");
        native_entry_html(
            &document(&app, &created.result_path, &foreign_cookies).await,
            StatusCode::NOT_FOUND,
        );
        submit(
            &app,
            &foreign_cookies,
            &foreign_csrf,
            &enrollment(command, foreign.account),
        )
        .await
        .error(StatusCode::FORBIDDEN, "company_enrollment_forbidden");
        assert!(
            before == all_rows(&pool).await,
            "foreign command use disclosed or changed original request"
        );
        startup.close().await;
    }
    // Additive read-only dependency. Existing Company command and browser tests
    // remain required unchanged; none are replaced by these entry projections.
    mod entry_read {
        use super::*;

        async fn owner_ready(pool: &PgPool) {
            let present: bool = sqlx::query_scalar("SELECT to_regprocedure('public.account_company_setup_eligibility_v1(uuid)') IS NOT NULL")
                    .fetch_one(pool).await.unwrap();
            assert!(
                present,
                "PREREQUISITE: finalized real eligibility owner required"
            );
        }

        fn preview(response: &Response, account: Uuid) {
            let html = native_entry_html(response, StatusCode::OK);
            for required in [
                "data-company-setup",
                "data-company-enrollment",
                "type=\"submit\"",
                "name=\"name\"",
                "name=\"slug\"",
                "회사 이름",
                "업무 공간 식별자",
                "내 계정",
                "기존 회사가 사용할 콘솔 업무 공간을 등록합니다.",
            ] {
                assert!(
                    html.contains(required),
                    "native Company preview missing required subject/input"
                );
            }
            for forbidden in [
                "data-native-action=",
                "<leptos-island",
                "/_ui",
                "name=\"role\"",
                "name=\"root_secret\"",
            ] {
                assert!(
                    !html.contains(forbidden),
                    "Company form must not invent authority or ship unnecessary islands"
                );
            }
            assert_eq!(html.matches("<form").count(), 1);
            assert_eq!(html.matches("type=\"submit\"").count(), 1);
            assert!(html.contains(&format!("data-account-id=\"{account}\"")));
            native_entry_no_business_navigation(html);
        }

        fn denied(response: &Response, status: StatusCode, account: Option<Uuid>) {
            let html = native_entry_html(response, status);
            for forbidden in [
                "data-company-setup",
                "name=\"name\"",
                "name=\"slug\"",
                "data-account-state=",
                "data-native-action=",
                "deployment_operator",
                "account_company_setup_eligibility",
                "console_auth_rt",
            ] {
                assert!(
                    !html.contains(forbidden),
                    "denied setup leaked protected projection or diagnostics"
                );
            }
            if let Some(account) = account {
                assert!(
                    !html.contains(&account.to_string()),
                    "denied setup leaked Account reference"
                );
            }
            native_entry_no_business_navigation(html);
        }

        #[sqlx::test(migrations = false)]
        async fn company_entry_anonymous_and_ordinary_reads_are_private_and_effect_free(
            pool: PgPool,
        ) {
            let app = fixture(&pool).await;
            owner_ready(&pool).await;
            let (account, cookies) = enrolled(&app).await;
            let before = all_rows(&pool).await;
            denied(
                &document(&app, ENTRY, &Cookies::default()).await,
                StatusCode::UNAUTHORIZED,
                Some(account.account),
            );
            denied(
                &document(&app, ENTRY, &cookies).await,
                StatusCode::NOT_FOUND,
                Some(account.account),
            );
            let current = document(&app, "/account", &cookies).await;
            let html = native_entry_html(&current, StatusCode::OK);
            assert!(html.contains("data-account-state=\"active\""));
            assert!(html.contains("data-native-action=\"logout\""));
            assert!(!html.contains("href=\"/account/companies/new\""));
            assert!(
                before == all_rows(&pool).await,
                "read refusal changed durable state"
            );
        }

        #[sqlx::test(migrations = false)]
        async fn company_entry_preview_metadata_cookie_and_method_contract_is_current(
            pool: PgPool,
        ) {
            let (app, account, cookies, startup, _) = designated(&pool).await;
            owner_ready(&pool).await;
            let before = all_rows(&pool).await;
            for site in ["none", "same-origin", "same-site", "cross-site"] {
                preview(
                    &native_entry_get(&app, ENTRY, &cookies, site, "navigate", "document", &[])
                        .await,
                    account.account,
                );
                assert!(before == all_rows(&pool).await);
            }
            preview(
                &native_entry_get(&app, ENTRY, &cookies, "", "", "", &[]).await,
                account.account,
            );
            for (site, mode, dest, extra, expected) in [
                (
                    "cross-site",
                    "no-cors",
                    "image",
                    vec![],
                    StatusCode::FORBIDDEN,
                ),
                (
                    "same-origin",
                    "navigate",
                    "iframe",
                    vec![],
                    StatusCode::FORBIDDEN,
                ),
                (
                    "same-origin",
                    "",
                    "document",
                    vec![],
                    StatusCode::BAD_REQUEST,
                ),
                (
                    "same-origin",
                    "navigate",
                    "document",
                    vec![("Origin", "https://foreign.invalid")],
                    StatusCode::FORBIDDEN,
                ),
                (
                    "same-origin",
                    "navigate",
                    "document",
                    vec![("Sec-Fetch-User", "?0")],
                    StatusCode::BAD_REQUEST,
                ),
            ] {
                denied(
                    &native_entry_get(&app, ENTRY, &cookies, site, mode, dest, &extra).await,
                    expected,
                    Some(account.account),
                );
                assert!(before == all_rows(&pool).await);
            }
            for name in [ACCESS, REFRESH, ENROLLMENT, LOGIN] {
                let value = if name == ACCESS {
                    cookies.0[ACCESS].clone()
                } else {
                    "company-unused-proof-marker".to_owned()
                };
                let presented = Cookies(BTreeMap::from([(name.to_owned(), value)]));
                let response = document(&app, ENTRY, &presented).await;
                if name == ACCESS {
                    preview(&response, account.account);
                } else {
                    denied(&response, StatusCode::UNAUTHORIZED, Some(account.account));
                }
                let head = request(
                    &app,
                    "HEAD",
                    ENTRY,
                    &presented,
                    None,
                    &[
                        ("Sec-Fetch-Mode", "navigate"),
                        ("Sec-Fetch-Dest", "document"),
                    ],
                )
                .await;
                assert_eq!(head.status, StatusCode::METHOD_NOT_ALLOWED);
                assert!(head.bytes.is_empty() && !head.headers.contains_key(header::SET_COOKIE));
                head.private();
                for value in [
                    format!("{name}="),
                    format!("{name} =company-proof-marker"),
                    format!("{name}=company-proof-marker; {name}=company-proof-marker"),
                ] {
                    denied(
                        &native_entry_get(
                            &app,
                            ENTRY,
                            &Cookies::default(),
                            "none",
                            "navigate",
                            "document",
                            &[("Cookie", &value)],
                        )
                        .await,
                        StatusCode::BAD_REQUEST,
                        Some(account.account),
                    );
                }
                let header = format!("{name}=company-proof-marker");
                for extras in [
                    vec![("Cookie", header.as_str()), ("Cookie", header.as_str())],
                    vec![
                        ("Cookie", header.as_str()),
                        ("Authorization", "Bearer company-mixed-marker"),
                    ],
                ] {
                    denied(
                        &native_entry_get(
                            &app,
                            ENTRY,
                            &Cookies::default(),
                            "none",
                            "navigate",
                            "document",
                            &extras,
                        )
                        .await,
                        StatusCode::BAD_REQUEST,
                        Some(account.account),
                    );
                }
                assert!(
                    before == all_rows(&pool).await,
                    "native credential admission consumed or changed state"
                );
            }
            let oversized = format!("theme={}", "x".repeat(16 * 1024 + 1));
            let response = native_entry_get(
                &app,
                ENTRY,
                &Cookies::default(),
                "none",
                "navigate",
                "document",
                &[("Cookie", &oversized)],
            )
            .await;
            native_root_parser_refusal(&response, StatusCode::PAYLOAD_TOO_LARGE);
            assert!(before == all_rows(&pool).await);
            startup.close().await;
        }

        #[sqlx::test(migrations = false)]
        async fn company_entry_actual_auth_outage_and_repair_preserve_status_and_history(
            pool: PgPool,
        ) {
            let (mut app, key) = signed_fixture(&pool).await;
            owner_ready(&pool).await;
            let auth = logout_auth_pool(&pool).await;
            let state = state_with_key(&pool, app._artifacts.root.clone(), &key).await;
            app.service = build_router(state.with_auth_database(auth.clone()));
            let (account, cookies) = enrolled(&app).await;
            let startup = startup(&pool).await;
            designate(&startup, &designation(&pool, account.account).await)
                .await
                .unwrap();
            let before = all_rows(&pool).await;
            preview(&document(&app, ENTRY, &cookies).await, account.account);
            auth.close().await;
            assert!(auth.is_closed());
            for presented in [&cookies, &Cookies::default()] {
                denied(
                    &document(&app, ENTRY, presented).await,
                    StatusCode::SERVICE_UNAVAILABLE,
                    Some(account.account),
                );
                assert!(before == all_rows(&pool).await);
            }
            // Metadata admission precedes the failed transport, not vice versa.
            denied(
                &native_entry_get(&app, ENTRY, &cookies, "cross-site", "no-cors", "image", &[])
                    .await,
                StatusCode::FORBIDDEN,
                Some(account.account),
            );
            let recovered = logout_auth_pool(&pool).await;
            let state = state_with_key(&pool, app._artifacts.root.clone(), &key).await;
            app.service = build_router(state.with_auth_database(recovered.clone()));
            preview(&document(&app, ENTRY, &cookies).await, account.account);
            assert!(
                before == all_rows(&pool).await,
                "actual transport replacement changed state"
            );
            recovered.close().await;
            startup.close().await;
        }

        async fn eligibility_metadata(pool: &PgPool) -> String {
            sqlx::query_scalar("SELECT jsonb_build_object('definition',pg_get_functiondef(p.oid),'owner',pg_get_userbyid(p.proowner),'config',p.proconfig,'acl',(SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,a.privilege_type,a.is_grantable) ORDER BY a.grantee,a.privilege_type,a.is_grantable) FROM aclexplode(p.proacl) a))::text FROM pg_proc p WHERE p.oid='public.account_company_setup_eligibility_v1(uuid)'::regprocedure")
                    .fetch_one(pool).await.unwrap()
        }

        #[sqlx::test(migrations = false)]
        async fn company_entry_eligibility_sql_outage_keeps_current_account_and_restores_exact_owner(
            pool: PgPool,
        ) {
            let (app, account, cookies, startup, _) = designated(&pool).await;
            owner_ready(&pool).await;
            let runtime = login_test_pool(&pool, TestDatabaseLogin::Auth).await;
            let original_metadata = eligibility_metadata(&pool).await;
            let before = all_rows(&pool).await;
            let original: bool =
                sqlx::query_scalar("SELECT public.account_company_setup_eligibility_v1($1)")
                    .bind(account.account)
                    .fetch_one(&runtime)
                    .await
                    .unwrap();
            assert!(original);
            preview(&document(&app, ENTRY, &cookies).await, account.account);
            // Fixture-only real permission failure, with exact restoration below.
            sqlx::query("REVOKE EXECUTE ON FUNCTION public.account_company_setup_eligibility_v1(uuid) FROM console_auth_rt").execute(&pool).await.unwrap();
            let failed = sqlx::query_scalar::<_, bool>(
                "SELECT public.account_company_setup_eligibility_v1($1)",
            )
            .bind(account.account)
            .fetch_one(&runtime)
            .await
            .unwrap_err();
            assert!(failed.as_database_error().unwrap().code().as_deref() == Some("42501"));
            assert!(original_metadata != eligibility_metadata(&pool).await);
            for route in ["/account", "/account/register", "/"] {
                let response = document(&app, route, &cookies).await;
                let html = native_entry_html(&response, StatusCode::SERVICE_UNAVAILABLE);
                assert!(
                    html.contains("data-account-state=\"active\"")
                        && html.contains("data-context-state=\"empty\"")
                );
                assert!(html.contains("data-native-action=\"logout\""));
                assert!(
                    !html.contains("href=\"/account/companies/new\"")
                        && !html.contains("data-company-setup")
                );
                assert!(
                    !html.contains("console_auth_rt")
                        && !html.contains("account_company_setup_eligibility")
                );
                native_entry_no_business_navigation(html);
            }
            denied(
                &document(&app, ENTRY, &cookies).await,
                StatusCode::SERVICE_UNAVAILABLE,
                Some(account.account),
            );
            let current = request(&app, "GET", "/api/v2/accounts/me", &cookies, None, &[]).await;
            current.private();
            assert!(current.json(StatusCode::OK)["account_id"] == json!(account.account));
            assert!(
                before == all_rows(&pool).await,
                "eligibility document/me reads erased identity or made durable effects"
            );
            // The real CSRF proof remains available during eligibility failure.
            // Its admitted auth rate limiter is a separate, precisely bounded
            // effect; document and current-Account reads above must stay inert.
            let prior_limits: Vec<Value> =
                serde_json::from_str(&before["auth_rate_limit"]).unwrap();
            assert!(
                prior_limits
                    .iter()
                    .all(|row| row["endpoint"] != "account_csrf")
            );
            let proof_started = OffsetDateTime::now_utc().unix_timestamp();
            let _csrf = proof(&app, &cookies).await;
            let proof_finished = OffsetDateTime::now_utc().unix_timestamp();
            assert!(proof_started <= proof_finished);
            let after_proof = all_rows(&pool).await;
            let limits = added_rows(&before["auth_rate_limit"], &after_proof["auth_rate_limit"])
                .expect("CSRF fetch altered or omitted existing limiter history");
            assert_eq!(
                limits.len(),
                2,
                "one IP and one global CSRF increment required"
            );
            let mut clients = BTreeSet::new();
            let mut windows = BTreeSet::new();
            for row in limits {
                exact_keys(
                    &row,
                    &["client_key", "endpoint", "window_start", "attempts"],
                );
                assert!(row["endpoint"] == "account_csrf" && row["attempts"] == 1);
                assert!(clients.insert(row["client_key"].as_str().unwrap().to_owned()));
                let window = OffsetDateTime::parse(
                    row["window_start"].as_str().unwrap(),
                    &time::format_description::well_known::Rfc3339,
                )
                .unwrap();
                assert_eq!(window.nanosecond(), 0);
                let seconds = window.unix_timestamp();
                assert!(
                    seconds.rem_euclid(60) == 0
                        && seconds >= proof_started - proof_started.rem_euclid(60)
                        && seconds <= proof_finished - proof_finished.rem_euclid(60)
                );
                windows.insert(seconds);
            }
            assert!(clients == BTreeSet::from(["global".to_owned(), "ip:127.0.0.1".to_owned()]));
            assert_eq!(
                windows.len(),
                1,
                "both limiter buckets use one owner timestamp"
            );
            let mut expected_after_proof = before.clone();
            expected_after_proof.insert(
                "auth_rate_limit".to_owned(),
                after_proof["auth_rate_limit"].clone(),
            );
            assert!(
                expected_after_proof == after_proof,
                "CSRF fetch made effects beyond its exact two limiter increments"
            );
            sqlx::query("GRANT EXECUTE ON FUNCTION public.account_company_setup_eligibility_v1(uuid) TO console_auth_rt").execute(&pool).await.unwrap();
            assert!(
                original_metadata == eligibility_metadata(&pool).await,
                "fixture did not restore exact capability metadata"
            );
            assert!(
                sqlx::query_scalar::<_, bool>(
                    "SELECT public.account_company_setup_eligibility_v1($1)"
                )
                .bind(account.account)
                .fetch_one(&runtime)
                .await
                .unwrap()
            );
            preview(&document(&app, ENTRY, &cookies).await, account.account);
            assert!(
                after_proof == all_rows(&pool).await,
                "restored eligibility document changed the validated post-CSRF state"
            );
            runtime.close().await;
            startup.close().await;
        }

        #[sqlx::test(migrations = false)]
        async fn company_entry_independent_eligibility_survives_real_context_proof_failure(
            pool: PgPool,
        ) {
            let (app, account, cookies, startup, _) = designated(&pool).await;
            owner_ready(&pool).await;
            preview(&document(&app, ENTRY, &cookies).await, account.account);
            assert_eq!(sqlx::query("UPDATE public.account_security SET context_generation=2,revision=revision+1,updated_at=clock_timestamp() WHERE account_id=$1 AND context_generation=1").bind(account.account).execute(&pool).await.unwrap().rows_affected(), 1);
            let before = all_rows(&pool).await;
            let auth = logout_auth_pool(&pool).await;
            let failed = sqlx::query("SELECT * FROM public.account_context_presence_v1($1)")
                .bind(account.account)
                .fetch_one(&auth)
                .await
                .unwrap_err();
            assert!(
                failed.as_database_error().unwrap().message() == "account.navigation_unavailable"
            );
            auth.close().await;
            for route in ["/account", "/account/register", "/"] {
                let response = document(&app, route, &cookies).await;
                let html = native_entry_html(&response, StatusCode::SERVICE_UNAVAILABLE);
                assert!(
                    html.contains("data-account-state=\"active\"")
                        && html.contains("data-context-state=\"unavailable\"")
                );
                assert!(
                    html.contains("data-native-action=\"logout\"")
                        && html.contains("href=\"/account/companies/new\"")
                );
            }
            preview(&document(&app, ENTRY, &cookies).await, account.account);
            assert_no_company_identity(&pool, account.account).await;
            assert!(
                before == all_rows(&pool).await,
                "independent projection made context or identity effects"
            );
            startup.close().await;
        }
        #[sqlx::test(migrations = false)]
        async fn company_entry_optional_eligibility_expiry_never_retains_active_identity(
            pool: PgPool,
        ) {
            let (app, signer) = signed_fixture(&pool).await;
            owner_ready(&pool).await;
            let (account, cookies) = enrolled(&app).await;
            let startup = startup(&pool).await;
            designate(&startup, &designation(&pool, account.account).await)
                .await
                .unwrap();
            for route in ["/account", "/account/register", "/"] {
                // Complete census before starting the intentionally short token lifetime.
                let before = all_rows(&pool).await;
                let now: OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
                    .fetch_one(&pool)
                    .await
                    .unwrap();
                let expires = now.unix_timestamp() + 6;
                let mut claims = signed_claims(&cookies.0[ACCESS], &signer).unwrap();
                claims["exp"] = json!(expires);
                let mut short = cookies.clone();
                short
                    .0
                    .insert(ACCESS.into(), sign_proof_claims(&claims, &signer));
                assert!(
                    native_entry_html(&document(&app, route, &short).await, StatusCode::OK)
                        .contains("data-account-state=\"active\"")
                );
                let mut blocker = pool.begin().await.unwrap();
                let holder: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
                    .fetch_one(&mut *blocker)
                    .await
                    .unwrap();
                sqlx::query(
                    "SELECT receipt_id FROM public.deployment_operator_head WHERE singleton=1 FOR UPDATE",
                )
                .fetch_one(&mut *blocker)
                .await
                .unwrap();
                let observation = async {
                    let reached = tokio::time::timeout(std::time::Duration::from_secs(3), async {
                        loop {
                            let waiting: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_stat_activity a WHERE a.datname=current_database() AND a.usename='console_auth_rt' AND a.wait_event_type='Lock' AND $1=ANY(pg_catalog.pg_blocking_pids(a.pid))) AND extract(epoch FROM clock_timestamp()) < $2::bigint")
                                .bind(holder).bind(expires).fetch_one(&pool).await.unwrap();
                            if waiting { break; }
                            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                        }
                    }).await;
                    let elapsed = if reached.is_ok() {
                        tokio::time::timeout(std::time::Duration::from_secs(7), sqlx::query("SELECT pg_sleep(GREATEST(0.0,$1::double precision-extract(epoch FROM clock_timestamp()))+0.025)")
                            .bind(expires as f64).execute(&pool)).await.is_ok_and(|r| r.is_ok())
                    } else {
                        false
                    };
                    blocker.rollback().await.unwrap();
                    (reached.is_ok(), elapsed)
                };
                let (response, witness) =
                    tokio::time::timeout(std::time::Duration::from_secs(15), async {
                        tokio::join!(document(&app, route, &short), observation)
                    })
                    .await
                    .expect("bounded document and lock release");
                assert!(
                    witness == (true, true),
                    "actual designation-head wait before expiry required"
                );
                let html = native_entry_html(&response, StatusCode::OK);
                assert!(!html.contains("data-account-state=\"active\""));
                assert!(!html.contains("data-native-action=\"logout\""));
                assert!(!html.contains("href=\"/account/companies/new\""));
                // Preserve the existing owner distinction: invalid registration
                // access returns SignIn; fresh anonymous registration shows terms.
                let anonymous_route = if route == "/account/register" {
                    assert!(html.contains("data-native-action=\"login\""));
                    assert!(!html.contains("data-native-action=\"register\""));
                    let fresh_registration =
                        document(&app, "/account/register", &Cookies::default()).await;
                    assert!(
                        native_entry_html(&fresh_registration, StatusCode::OK)
                            .contains("data-native-action=\"register\"")
                    );
                    "/account"
                } else {
                    route
                };
                let anonymous = document(&app, anonymous_route, &Cookies::default()).await;
                native_entry_html(&anonymous, StatusCode::OK);
                assert!(
                    response.bytes == anonymous.bytes,
                    "expired optional eligibility retained Account identity"
                );
                assert!(
                    before == all_rows(&pool).await,
                    "expired projection changed persistent state"
                );
                assert!(
                    native_entry_html(&document(&app, route, &cookies).await, StatusCode::OK)
                        .contains("data-account-state=\"active\"")
                );
            }
            startup.close().await;
        }
    }

    include!("native_company_eligibility.rs");

    fn native_413_headers_are_private(headers: &http::HeaderMap) -> bool {
        let vary: Vec<_> = headers.get_all(header::VARY).iter().collect();
        let valid_vary = vary.len() == 1
            && vary[0].to_str().is_ok_and(|value| {
                let tokens: Vec<_> = value
                    .split(',')
                    .map(|part| part.trim().to_ascii_lowercase())
                    .collect();
                tokens.len() == 3
                    && tokens.into_iter().collect::<BTreeSet<_>>()
                        == BTreeSet::from([
                            "authorization".to_owned(),
                            "cookie".to_owned(),
                            "origin".to_owned(),
                        ])
            });
        valid_vary && [
            ("content-type", "text/html; charset=utf-8"),
            ("cache-control", "no-store"),
            ("pragma", "no-cache"),
            ("x-content-type-options", "nosniff"),
            ("referrer-policy", "no-referrer"),
            ("content-security-policy", "default-src 'self'; script-src 'self'; style-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'"),
        ]
        .into_iter()
        .all(|(name, expected)| {
            let mut values = headers.get_all(name).iter();
            values.next().is_some_and(|value| value == expected) && values.next().is_none()
        }) && !headers.contains_key(header::SET_COOKIE)
    }

    #[test]
    fn native_413_header_oracle_rejects_omitted_changed_and_duplicate_values() {
        let positive = console_payroll_ui::native_account::document(
            console_payroll_ui::native_account::Page::Refused,
            StatusCode::PAYLOAD_TOO_LARGE,
        );
        assert!(native_413_headers_are_private(positive.headers()));
        for name in [
            "content-type",
            "cache-control",
            "pragma",
            "vary",
            "x-content-type-options",
            "referrer-policy",
            "content-security-policy",
        ] {
            let mut missing = positive.headers().clone();
            missing.remove(name);
            assert!(!native_413_headers_are_private(&missing), "omitted {name}");
            let mut changed = positive.headers().clone();
            changed.insert(name, http::HeaderValue::from_static("corrupt"));
            assert!(!native_413_headers_are_private(&changed), "corrupt {name}");
            let mut duplicated = positive.headers().clone();
            duplicated.append(name, positive.headers().get(name).unwrap().clone());
            assert!(
                !native_413_headers_are_private(&duplicated),
                "duplicate {name}"
            );
        }
        let mut reordered = positive.headers().clone();
        reordered.insert(
            header::VARY,
            http::HeaderValue::from_static("origin, AUTHORIZATION, Cookie"),
        );
        assert!(native_413_headers_are_private(&reordered));
        for wrong in [
            "Cookie, Origin",
            "Authorization, Origin",
            "Authorization, Cookie",
            "Authorization, Cookie, Origin, Cookie",
            "Authorization, Cookie, Origin, Accept",
        ] {
            let mut changed = positive.headers().clone();
            changed.insert(header::VARY, http::HeaderValue::from_static(wrong));
            assert!(
                !native_413_headers_are_private(&changed),
                "invalid Vary key set"
            );
        }
        let mut cookie = positive.headers().clone();
        cookie.insert(
            header::SET_COOKIE,
            http::HeaderValue::from_static("unexpected=value"),
        );
        assert!(!native_413_headers_are_private(&cookie));
    }

    #[sqlx::test(migrations = false)]
    async fn native_cookie_413_preserves_document_privacy_and_api_json(pool: PgPool) {
        let (app, account, cookies, startup, _designation) = designated(&pool).await;
        let before = all_rows(&pool).await;
        let large = format!("theme={}", "x".repeat(16 * 1024 + 1));
        let first = format!("theme_a={}", "y".repeat(8500));
        let second = format!("theme_b={}", "z".repeat(8500));
        let mut refused_bytes = None;
        for route in ["/", "/account", "/account/register", ENTRY] {
            for extra in [
                vec![("Cookie", large.as_str())],
                vec![("Cookie", first.as_str()), ("Cookie", second.as_str())],
            ] {
                let response = native_entry_get(
                    &app,
                    route,
                    &cookies,
                    "same-origin",
                    "navigate",
                    "document",
                    &extra,
                )
                .await;
                assert_eq!(response.status, StatusCode::PAYLOAD_TOO_LARGE);
                assert!(
                    native_413_headers_are_private(&response.headers),
                    "native owner413 lost its rendered privacy/security envelope"
                );
                let html = native_entry_html(&response, StatusCode::PAYLOAD_TOO_LARGE);
                assert!(html.contains("이 요청을 열 수 없습니다"));
                for forbidden in [
                    "<form",
                    "data-account-state",
                    "data-company-setup",
                    "data-native-action",
                    "data-island",
                    "/_ui",
                ] {
                    assert!(
                        !html.contains(forbidden),
                        "denied document leaked {forbidden}"
                    );
                }
                assert!(!html.contains(&account.account.to_string()));
                if let Some(bytes) = &refused_bytes {
                    assert_eq!(
                        &response.bytes, bytes,
                        "same owner refusal differs by document route"
                    );
                } else {
                    refused_bytes = Some(response.bytes.clone());
                }
                assert!(
                    before == all_rows(&pool).await,
                    "oversized Cookie changed durable owner state"
                );
            }
            let recovered = native_entry_get(
                &app,
                route,
                &cookies,
                "same-origin",
                "navigate",
                "document",
                &[],
            )
            .await;
            native_entry_html(&recovered, StatusCode::OK);
            assert!(
                before == all_rows(&pool).await,
                "document recovery consumed session or changed history"
            );
        }
        let api = request(
            &app,
            "GET",
            "/api/v2/accounts/me",
            &cookies,
            None,
            &[
                ("Cookie", large.as_str()),
                ("Accept", "text/html"),
                ("X-Console-Native-Error", "true"),
            ],
        )
        .await;
        api.error(StatusCode::PAYLOAD_TOO_LARGE, "request_too_large");
        assert!(
            api.headers
                .get(header::CONTENT_TYPE)
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("application/json")
        );
        request(
            &app,
            "GET",
            "/api/v2/accounts/me",
            &cookies,
            None,
            &[
                ("Cookie", large.as_str()),
                ("Authorization", "Bearer oversized-ambiguity-marker"),
            ],
        )
        .await
        .error(StatusCode::BAD_REQUEST, "ambiguous_credentials");
        assert!(
            before == all_rows(&pool).await,
            "API refusal changed durable owner state"
        );
        startup.close().await;
    }

    include!("native_company_input_parser.rs");
    include!("native_company_intake_schema.rs");
    include!("native_company_intake_owner.rs");

    include!("native_policy_store_smoke.rs");
    include!("native_policy_store_projection_tests.rs");
    include!("native_policy_company_birth_guard.rs");

    // Existing Auth proof issuance changes exactly one global and one IP bucket.
    // Compare the complete remaining census byte-for-byte; no business GET effects.
    fn policy_preflight_effects(
        before: &BTreeMap<String, String>,
        after: &BTreeMap<String, String>,
        started: OffsetDateTime,
        finished: OffsetDateTime,
    ) -> bool {
        let check = || -> Option<bool> {
            if started > finished || (finished - started).whole_seconds() > 30 {
                return Some(false);
            }
            let before_raw = before.get("auth_rate_limit")?;
            let after_raw = after.get("auth_rate_limit")?;
            let prior: Vec<Value> = serde_json::from_str(before_raw).ok()?;
            let actual: Vec<Value> = serde_json::from_str(after_raw).ok()?;
            let mut unaffected = before.clone();
            unaffected.insert("auth_rate_limit".to_owned(), after_raw.clone());
            if &unaffected != after {
                return Some(false);
            }
            let first = started.unix_timestamp().div_euclid(60) * 60;
            let last = finished.unix_timestamp().div_euclid(60) * 60;
            'window: for window in (first..=last).step_by(60) {
                let mut expected = prior.clone();
                for client in ["global", "ip:127.0.0.1"] {
                    let matches = |row: &Value| -> bool {
                        row["endpoint"] == "account_csrf"
                            && row["client_key"] == client
                            && row["window_start"]
                                .as_str()
                                .and_then(|s| {
                                    OffsetDateTime::parse(
                                        s,
                                        &time::format_description::well_known::Rfc3339,
                                    )
                                    .ok()
                                })
                                .is_some_and(|t| {
                                    t.unix_timestamp() == window && t.nanosecond() == 0
                                })
                    };
                    let indices: Vec<_> = expected
                        .iter()
                        .enumerate()
                        .filter_map(|(i, row)| matches(row).then_some(i))
                        .collect();
                    if indices.len() > 1 {
                        return Some(false);
                    }
                    if let Some(&index) = indices.first() {
                        let count = expected[index]["attempts"].as_i64()?.checked_add(1)?;
                        expected[index]["attempts"] = json!(count);
                    } else {
                        let rows: Vec<_> = actual.iter().filter(|row| matches(row)).collect();
                        if rows.is_empty() {
                            continue 'window;
                        }
                        if rows.len() != 1 || rows[0]["attempts"] != 1 {
                            return Some(false);
                        }
                        let row = rows[0];
                        let keys: BTreeSet<_> =
                            row.as_object()?.keys().map(String::as_str).collect();
                        if keys
                            != BTreeSet::from([
                                "client_key",
                                "endpoint",
                                "window_start",
                                "attempts",
                            ])
                        {
                            return Some(false);
                        }
                        expected.push(row.clone());
                    }
                }
                let canonical = |rows: Vec<Value>| {
                    let mut rows: Vec<_> = rows.into_iter().map(|v| v.to_string()).collect();
                    rows.sort();
                    rows
                };
                if canonical(expected) == canonical(actual.clone()) {
                    return Some(true);
                }
            }
            Some(false)
        };
        check() == Some(true)
    }

    #[test]
    fn policy_preflight_effect_oracle_preserves_business_and_exact_limiter_counts() {
        let at = OffsetDateTime::from_unix_timestamp(1_800_000_000).unwrap();
        let window = at
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap();
        let before = BTreeMap::from([
            ("auth_rate_limit".into(), "[]".into()),
            ("business".into(), "[]".into()),
        ]);
        let limits = json!([
            {"client_key":"global","endpoint":"account_csrf","window_start":window,"attempts":1},
            {"client_key":"ip:127.0.0.1","endpoint":"account_csrf","window_start":window,"attempts":1}
        ]);
        let mut after = before.clone();
        after.insert("auth_rate_limit".into(), limits.to_string());
        assert!(policy_preflight_effects(&before, &after, at, at));
        assert!(!policy_preflight_effects(&before, &before, at, at));
        let mut changed = after.clone();
        changed.insert("business".into(), "[{}]".into());
        assert!(!policy_preflight_effects(&before, &changed, at, at));
        for index in 0..2 {
            let mut corrupt = limits.clone();
            corrupt[index]["attempts"] = json!(2);
            changed = after.clone();
            changed.insert("auth_rate_limit".into(), corrupt.to_string());
            assert!(!policy_preflight_effects(&before, &changed, at, at));
        }
        let mut incremented = limits.clone();
        for row in incremented.as_array_mut().unwrap() {
            row["attempts"] = json!(2);
        }
        changed = after.clone();
        changed.insert("auth_rate_limit".into(), incremented.to_string());
        assert!(policy_preflight_effects(&after, &changed, at, at));
        assert!(!policy_preflight_effects(&before, &changed, at, at));
        for corrupt in [json!([limits[0]]), json!([limits[0], limits[0], limits[1]])] {
            changed = after.clone();
            changed.insert("auth_rate_limit".into(), corrupt.to_string());
            assert!(!policy_preflight_effects(&before, &changed, at, at));
        }
        let next = at + time::Duration::minutes(1);
        let next_text = next
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap();
        let mut crossed = limits.clone();
        for row in crossed.as_array_mut().unwrap() {
            row["window_start"] = json!(next_text);
        }
        changed = before.clone();
        changed.insert("auth_rate_limit".into(), crossed.to_string());
        assert!(policy_preflight_effects(
            &before,
            &changed,
            next - time::Duration::seconds(1),
            next
        ));
        assert!(!policy_preflight_effects(&before, &changed, at, at));
    }

    mod native_policy_startup_tests {
        include!("native_policy_startup_tests.rs");
        include!("native_policy_system_acl_tests.rs");
        include!("native_policy_validation_owner_tests.rs");
        include!("native_people_codec2_install_probe.rs");
        include!("native_people_policy_mixed_owner.rs");
        include!("native_people_policy_http_owner.rs");
        include!("native_policy_declared_successor_capture.rs");
        include!("native_policy_declared_successor_schema_usage.rs");
        include!("native_policy_successor_contract.rs");
        include!("native_payroll_read_owner_tests.rs");
    }

    mod native_policy_physical {
        include!("native_policy_physical_tests.rs");
    }

    #[cfg(feature = "test-browser")]
    include!("native_company_browser.rs");
}
