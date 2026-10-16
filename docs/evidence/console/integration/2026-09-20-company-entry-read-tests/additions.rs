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

    fn preview(response: &Response) {
        let html = native_entry_html(response, StatusCode::OK);
        for required in [
            "data-company-setup",
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
            "<form",
            "type=\"submit\"",
            "data-native-action=",
            "<leptos-island",
            "/_ui",
            "name=\"role\"",
            "name=\"root_secret\"",
        ] {
            assert!(
                !html.contains(forbidden),
                "unreleased preview must not submit or invent authority"
            );
        }
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
    async fn company_entry_anonymous_and_ordinary_reads_are_private_and_effect_free(pool: PgPool) {
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
    async fn company_entry_preview_metadata_cookie_and_method_contract_is_current(pool: PgPool) {
        let (app, account, cookies, startup, _) = designated(&pool).await;
        owner_ready(&pool).await;
        let before = all_rows(&pool).await;
        for site in ["none", "same-origin", "same-site", "cross-site"] {
            preview(
                &native_entry_get(&app, ENTRY, &cookies, site, "navigate", "document", &[]).await,
            );
            assert!(before == all_rows(&pool).await);
        }
        preview(&native_entry_get(&app, ENTRY, &cookies, "", "", "", &[]).await);
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
                preview(&response);
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
    async fn company_entry_actual_auth_outage_and_repair_preserve_status_and_history(pool: PgPool) {
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
        preview(&document(&app, ENTRY, &cookies).await);
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
            &native_entry_get(&app, ENTRY, &cookies, "cross-site", "no-cors", "image", &[]).await,
            StatusCode::FORBIDDEN,
            Some(account.account),
        );
        let recovered = logout_auth_pool(&pool).await;
        let state = state_with_key(&pool, app._artifacts.root.clone(), &key).await;
        app.service = build_router(state.with_auth_database(recovered.clone()));
        preview(&document(&app, ENTRY, &cookies).await);
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
        preview(&document(&app, ENTRY, &cookies).await);
        // Fixture-only real permission failure, with exact restoration below.
        sqlx::query("REVOKE EXECUTE ON FUNCTION public.account_company_setup_eligibility_v1(uuid) FROM console_auth_rt").execute(&pool).await.unwrap();
        let failed =
            sqlx::query_scalar::<_, bool>("SELECT public.account_company_setup_eligibility_v1($1)")
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
        let _csrf = proof(&app, &cookies).await;
        assert!(
            before == all_rows(&pool).await,
            "eligibility failure erased identity or made durable effects"
        );
        sqlx::query("GRANT EXECUTE ON FUNCTION public.account_company_setup_eligibility_v1(uuid) TO console_auth_rt").execute(&pool).await.unwrap();
        assert!(
            original_metadata == eligibility_metadata(&pool).await,
            "fixture did not restore exact capability metadata"
        );
        assert!(
            sqlx::query_scalar::<_, bool>("SELECT public.account_company_setup_eligibility_v1($1)")
                .bind(account.account)
                .fetch_one(&runtime)
                .await
                .unwrap()
        );
        preview(&document(&app, ENTRY, &cookies).await);
        assert!(before == all_rows(&pool).await);
        runtime.close().await;
        startup.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn company_entry_independent_eligibility_survives_real_context_proof_failure(
        pool: PgPool,
    ) {
        let (app, account, cookies, startup, _) = designated(&pool).await;
        owner_ready(&pool).await;
        preview(&document(&app, ENTRY, &cookies).await);
        assert_eq!(sqlx::query("UPDATE public.account_security SET context_generation=2,revision=revision+1,updated_at=clock_timestamp() WHERE account_id=$1 AND context_generation=1").bind(account.account).execute(&pool).await.unwrap().rows_affected(), 1);
        let before = all_rows(&pool).await;
        let auth = logout_auth_pool(&pool).await;
        let failed = sqlx::query("SELECT * FROM public.account_context_presence_v1($1)")
            .bind(account.account)
            .fetch_one(&auth)
            .await
            .unwrap_err();
        assert!(failed.as_database_error().unwrap().message() == "account.navigation_unavailable");
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
        preview(&document(&app, ENTRY, &cookies).await);
        assert_no_company_identity(&pool, account.account).await;
        assert!(
            before == all_rows(&pool).await,
            "independent projection made context or identity effects"
        );
        startup.close().await;
    }
}
