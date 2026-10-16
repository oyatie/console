// Additive real HTTP SSR regressions in the existing account_browser module.
// No replacement handlers, manufactured Company grants, or browser UX claims.
// Full native/legacy identity rows are compared without printing protected data.

async fn native_document_get(app: &Fixture, target: &str, cookies: &Cookies) -> Response {
    native_entry_get(app, target, cookies, "none", "navigate", "document", &[]).await
}

async fn native_document_snapshot(pool: &PgPool) -> BTreeMap<String, String> {
    let mut rows = native_extension_rows(pool).await;
    assert!(native_extension_rows_equal(&rows, &rows));
    // The retained native roster includes company_actors and context candidates.
    // Supplement it with complete legacy identity tables, never counts or lossy JSON.
    for table in ["users", "organizations", "groups", "employees", "persons"] {
        let sql = format!(
            "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text COLLATE \"C\"),'[]'::jsonb)::text FROM public.{table} t"
        );
        let raw: String = sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
            .fetch_one(pool)
            .await
            .unwrap();
        assert!(serde_json::from_str::<Value>(&raw).unwrap().is_array());
        assert!(rows.insert(table.to_owned(), raw).is_none());
    }
    assert_eq!(rows.len(), NATIVE_EXTENSION_TABLES.len() + 1 + 5);
    rows
}

fn native_document_unavailable(response: &Response) {
    let html = native_entry_html(response, StatusCode::SERVICE_UNAVAILABLE);
    assert!(html.contains("지금은 계정을 확인할 수 없습니다"));
    assert!(html.contains("role=\"alert\""));
    assert!(!html.contains("data-account-state="));
    assert!(!html.contains("data-context-state="));
    assert!(!html.contains("data-native-action="));
    native_entry_no_business_navigation(html);
}

#[sqlx::test(migrations = false)]
async fn native_document_auth_transport_outage_is_unavailable_not_anonymous(pool: PgPool) {
    let (mut app, key) = signed_fixture(&pool).await;
    let auth = logout_auth_pool(&pool).await;
    let state = state_with_key(&pool, app._artifacts.root.clone(), &key).await;
    app.service = build_router(state.with_auth_database(auth.clone()));
    let (attempt, cookies) = enrolled(&app).await;
    let before = native_document_snapshot(&pool).await;

    // Positive controls prove the same mounted router and real restricted owner
    // distinguish anonymous from currently authenticated before the outage.
    let anonymous = native_document_get(&app, "/account", &Cookies::default()).await;
    assert!(
        native_entry_html(&anonymous, StatusCode::OK).contains("data-account-state=\"anonymous\"")
    );
    let active = native_document_get(&app, "/account", &cookies).await;
    let html = native_entry_html(&active, StatusCode::OK);
    assert!(html.contains("data-account-state=\"active\""));
    assert!(html.contains("data-context-state=\"empty\""));
    assert!(before == native_document_snapshot(&pool).await);

    // Close only this actual Auth transport. The independent administrator stays
    // open for full-state evidence; no fake authority result is injected.
    auth.close().await;
    assert!(auth.is_closed());
    let anonymous_cookies = Cookies::default();
    for presented in [&anonymous_cookies, &cookies] {
        for path in ["/account", "/account/register"] {
            let response = native_document_get(&app, path, presented).await;
            native_document_unavailable(&response);
            assert!(
                !std::str::from_utf8(&response.bytes)
                    .unwrap()
                    .contains(&attempt.account.to_string())
            );
            assert!(before == native_document_snapshot(&pool).await);
        }
    }

    let root_unavailable = native_document_get(&app, "/", &cookies).await;
    native_document_unavailable(&root_unavailable);
    native_root_public(&native_document_get(&app, "/", &Cookies::default()).await);
    assert!(before == native_document_snapshot(&pool).await);

    // Exact same signing identity and surviving Account after real transport
    // replacement prove the refusal came from availability, not bad credentials.
    let recovered = logout_auth_pool(&pool).await;
    let state = state_with_key(&pool, app._artifacts.root.clone(), &key).await;
    app.service = build_router(state.with_auth_database(recovered.clone()));
    let active = native_document_get(&app, "/account", &cookies).await;
    let html = native_entry_html(&active, StatusCode::OK);
    assert!(html.contains("data-account-state=\"active\""));
    assert!(html.contains("data-context-state=\"empty\""));
    let anonymous = native_document_get(&app, "/account", &anonymous_cookies).await;
    assert!(
        native_entry_html(&anonymous, StatusCode::OK).contains("data-account-state=\"anonymous\"")
    );
    assert!(before == native_document_snapshot(&pool).await);
    let recovered_root = native_document_get(&app, "/", &cookies).await;
    let recovered_account = native_document_get(&app, "/account", &cookies).await;
    native_entry_html(&recovered_root, StatusCode::OK);
    assert!(recovered_root.bytes == recovered_account.bytes);
    assert!(before == native_document_snapshot(&pool).await);
    recovered.close().await;
}

#[sqlx::test(migrations = false)]
async fn native_document_unproven_context_retains_live_account_and_lawful_logout(pool: PgPool) {
    let app = fixture(&pool).await;
    let (attempt, cookies) = enrolled(&app).await;
    let original = native_document_snapshot(&pool).await;
    let positive = native_document_get(&app, "/account", &cookies).await;
    assert!(native_entry_html(&positive, StatusCode::OK).contains("data-context-state=\"empty\""));
    assert!(original == native_document_snapshot(&pool).await);

    // Negative administrator fixture: the initial empty-context proof no longer
    // applies. This does not implement a grant, membership, or Company workflow.
    // The unchanged security generation keeps the genuine current session live.
    assert_eq!(
        sqlx::query("UPDATE public.account_security SET context_generation=2,revision=revision+1,updated_at=clock_timestamp() WHERE account_id=$1 AND context_generation=1")
            .bind(attempt.account)
            .execute(&pool)
            .await
            .unwrap()
            .rows_affected(),
        1
    );
    let before = native_document_snapshot(&pool).await;
    let current = request(&app, "GET", "/api/v2/accounts/me", &cookies, None, &[]).await;
    current.private();
    let identity = current.json(StatusCode::OK);
    assert!(identity["account_id"] == json!(attempt.account));
    assert!(
        identity["permitted_self_actions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|action| {
                action["action_key"] == "account.session.logout"
                    && action["registration_revision"] == "1"
            })
    );

    // Witness the exact real owner's SQL exception independently. Do not replace
    // it with a mocked NavigationUnavailable or fabricate populated Company data.
    let auth = logout_auth_pool(&pool).await;
    let failed = sqlx::query("SELECT * FROM public.account_context_presence_v1($1)")
        .bind(attempt.account)
        .fetch_one(&auth)
        .await;
    let error = failed.expect_err("advanced context must refuse the initial absence proof");
    let database = error.as_database_error().unwrap();
    assert!(database.code().as_deref() == Some("P0001"));
    assert!(database.message() == "account.navigation_unavailable");
    auth.close().await;
    assert!(before == native_document_snapshot(&pool).await);

    for path in ["/account", "/account/register", "/"] {
        let response = native_document_get(&app, path, &cookies).await;
        let html = native_entry_html(&response, StatusCode::SERVICE_UNAVAILABLE);
        assert!(html.contains("data-account-state=\"active\""));
        assert!(html.contains("data-context-state=\"unavailable\""));
        assert!(html.contains("data-native-action=\"logout\""));
        assert!(!html.contains("data-account-state=\"anonymous\""));
        assert!(!html.contains("data-context-state=\"empty\""));
        assert!(!html.contains("data-native-action=\"register\""));
        assert!(!html.contains("data-native-action=\"login\""));
        native_entry_no_business_navigation(html);
        assert!(before == native_document_snapshot(&pool).await);
    }

    // The displayed action must still invoke the real owner successfully, not
    // merely render a button while context proof failure disables all actions.
    let scoped_before = snapshot(&pool, attempt.account).await;
    let csrf = proof(&app, &cookies).await;
    assert!(before == native_document_snapshot(&pool).await);
    let response = request(
        &app,
        "POST",
        "/api/v2/auth/logout",
        &cookies,
        Some(json!({})),
        &[("X-Console-CSRF", &csrf)],
    )
    .await;
    response.private();
    assert!(response.json(StatusCode::OK) == json!({"outcome":"COMMITTED"}));
    let scoped_after = snapshot(&pool, attempt.account).await;
    assert_logout_transition(&scoped_before, &scoped_after, attempt.account);
    let after = native_document_snapshot(&pool).await;
    let changed = [
        ("auth_refresh_token_families", "families"),
        ("auth_refresh_tokens", "tokens"),
        ("account_security_events", "events"),
    ];
    for (table, raw) in &before {
        if !changed.iter().any(|(name, _)| name == table) {
            assert!(
                after.get(table) == Some(raw),
                "logout changed an unrelated table"
            );
        }
    }
    // This fixture creates exactly one Account. The complete global mutable
    // tables must equal that Account's independently checked rows, both before
    // and after: extra rows, missing rows, and unrelated effects cannot hide.
    for (global, scoped) in [(&before, &scoped_before), (&after, &scoped_after)] {
        for (table, field) in changed {
            let actual: Value = serde_json::from_str(&global[table]).unwrap();
            let sorted = |value: &Value| {
                let mut rows: Vec<String> = value
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|row| serde_json::to_string(row).unwrap())
                    .collect();
                rows.sort();
                rows
            };
            assert!(
                sorted(&actual) == sorted(&scoped[field]),
                "logout effect scope differs"
            );
        }
    }
    let mut cleared = cookies.clone();
    cleared.absorb(&response.headers);
    assert!(!cleared.0.contains_key(ACCESS) && !cleared.0.contains_key(REFRESH));
    let expired = native_document_get(&app, "/account", &cookies).await;
    let html = native_entry_html(&expired, StatusCode::OK);
    assert!(html.contains("data-account-state=\"anonymous\""));
    assert!(!html.contains("data-account-state=\"active\""));
    assert!(!html.contains("data-context-state="));
    native_root_public(&native_document_get(&app, "/", &cookies).await);
    assert!(after == native_document_snapshot(&pool).await);
}

#[sqlx::test(migrations = false)]
async fn native_document_current_terms_unavailable_cannot_create_pending_identity_or_consent(
    pool: PgPool,
) {
    let app = fixture(&pool).await;
    let (attempt, cookies) = enrolled(&app).await;
    let before = native_document_snapshot(&pool).await;
    let positive = native_document_get(&app, "/account/register", &Cookies::default()).await;
    let html = native_entry_html(&positive, StatusCode::OK);
    assert!(html.contains("data-native-action=\"register\""));
    assert!(html.contains(&format!("data-terms-version=\"{MANIFEST_DIGEST}\"")));
    assert!(before == native_document_snapshot(&pool).await);

    // Reuse the existing explicit TEST_ONLY publication fixture. Advance the
    // genuine authoritative head to unregistered bytes; never fake a terms API.
    let unavailable_digest = "f".repeat(64);
    assert!(unavailable_digest != MANIFEST_DIGEST);
    let (_, revision) = seed_next_terms_head(&pool, &unavailable_digest).await;
    assert_eq!(revision, 2);
    let unavailable = native_document_snapshot(&pool).await;
    request(
        &app,
        "GET",
        "/api/v2/auth/terms",
        &Cookies::default(),
        None,
        &[],
    )
    .await
    .error(StatusCode::SERVICE_UNAVAILABLE, "authority_unavailable");
    let response = native_document_get(&app, "/account/register", &Cookies::default()).await;
    native_document_unavailable(&response);
    assert!(
        !std::str::from_utf8(&response.bytes)
            .unwrap()
            .contains(MANIFEST_DIGEST)
    );
    assert!(unavailable == native_document_snapshot(&pool).await);

    // A terms-publication outage does not retroactively erase a live Account.
    let current = request(&app, "GET", "/api/v2/accounts/me", &cookies, None, &[]).await;
    current.private();
    assert!(current.json(StatusCode::OK)["account_id"] == json!(attempt.account));
    let active = native_document_get(&app, "/account", &cookies).await;
    let html = native_entry_html(&active, StatusCode::OK);
    assert!(html.contains("data-account-state=\"active\""));
    assert!(html.contains("data-context-state=\"empty\""));
    assert!(html.contains("data-native-action=\"logout\""));
    let anonymous = native_document_get(&app, "/account", &Cookies::default()).await;
    assert!(
        native_entry_html(&anonymous, StatusCode::OK).contains("data-account-state=\"anonymous\"")
    );
    assert!(unavailable == native_document_snapshot(&pool).await);

    // Restore via another authoritative revision, preserving publication history.
    let (_, revision) = seed_next_terms_head(&pool, MANIFEST_DIGEST).await;
    assert_eq!(revision, 3);
    let restored = native_document_snapshot(&pool).await;
    let response = native_document_get(&app, "/account/register", &Cookies::default()).await;
    let html = native_entry_html(&response, StatusCode::OK);
    assert!(html.contains("data-native-action=\"register\""));
    assert!(html.contains(&format!("data-terms-version=\"{MANIFEST_DIGEST}\"")));
    for item in manifest()["items"].as_array().unwrap() {
        assert!(html.contains(item["title"].as_str().unwrap()));
        assert!(html.contains(item["terms_kind"].as_str().unwrap()));
    }
    assert!(restored == native_document_snapshot(&pool).await);
}
