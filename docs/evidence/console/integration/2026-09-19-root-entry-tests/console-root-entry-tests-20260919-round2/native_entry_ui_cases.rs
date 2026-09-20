// Additive candidate included inside retained auth_rest::account_browser.
// Real mounted router/restricted native owners; SoftPasskey remains synthetic.
// This is HTTP SSR/disclosure proof, never real browser discovery/residency proof.

async fn native_entry_get(
    app: &Fixture,
    target: &str,
    cookies: &Cookies,
    site: &str,
    mode: &str,
    destination: &str,
    extra: &[(&str, &str)],
) -> Response {
    let mut builder = Request::builder().method("GET").uri(target);
    // Empty values here intentionally mean missing Fetch Metadata headers.
    for (name, value) in [
        ("Sec-Fetch-Site", site),
        ("Sec-Fetch-Mode", mode),
        ("Sec-Fetch-Dest", destination),
    ] {
        if !value.is_empty() {
            builder = builder.header(name, value);
        }
    }
    // A document navigation does not manufacture an Origin header.
    let mut sent_secrets: Vec<String> = cookies.0.values().cloned().collect();
    if !cookies.0.is_empty() {
        builder = builder.header(header::COOKIE, cookies.header());
    }
    for (name, value) in extra {
        builder = builder.header(*name, *value);
        if name.eq_ignore_ascii_case("authorization") {
            sent_secrets.push(value.strip_prefix("Bearer ").unwrap_or(value).to_owned());
        }
        if name.eq_ignore_ascii_case("cookie") {
            for pair in value.split(';') {
                if let Some((_, secret)) = pair.trim().split_once('=') {
                    sent_secrets.push(secret.to_owned());
                }
            }
        }
    }
    let mut req = builder.body(Body::empty()).unwrap();
    req.extensions_mut().insert(ConnectInfo(
        "127.0.0.1:41000".parse::<SocketAddr>().unwrap(),
    ));
    let response = app.service.clone().oneshot(req).await.unwrap();
    let (parts, body) = response.into_parts();
    Response {
        status: parts.status,
        headers: parts.headers,
        bytes: to_bytes(body, 256 * 1024).await.unwrap().to_vec(),
        sent_secrets,
    }
}

fn native_entry_frame_policy_is_closed(headers: &http::HeaderMap) -> bool {
    let values: Vec<_> = headers.get_all("content-security-policy").iter().collect();
    if values.len() != 1 {
        return false;
    }
    let Ok(policy) = values[0].to_str() else {
        return false;
    };
    let directives: Vec<Vec<_>> = policy
        .split(';')
        .map(|part| part.split_ascii_whitespace().collect::<Vec<_>>())
        .filter(|parts| {
            parts
                .first()
                .is_some_and(|name| name.eq_ignore_ascii_case("frame-ancestors"))
        })
        .collect();
    directives.len() == 1 && directives[0].len() == 2 && directives[0][1] == "'none'"
}

#[test]
fn native_entry_frame_policy_oracle_rejects_wildcards_duplicates_and_missing() {
    let mut headers = http::HeaderMap::new();
    assert!(!native_entry_frame_policy_is_closed(&headers));
    for policy in [
        "frame-ancestors *",
        "frame-ancestors 'none' *",
        "frame-ancestors 'none';frame-ancestors *",
        "default-src 'none'",
        "frame-ancestors",
    ] {
        headers.insert(
            "content-security-policy",
            http::HeaderValue::from_str(policy).unwrap(),
        );
        assert!(
            !native_entry_frame_policy_is_closed(&headers),
            "unsafe framing control accepted"
        );
    }
    headers.insert(
        "content-security-policy",
        http::HeaderValue::from_static("default-src 'self'; frame-ancestors 'none'"),
    );
    assert!(native_entry_frame_policy_is_closed(&headers));
    headers.append(
        "content-security-policy",
        http::HeaderValue::from_static("frame-ancestors 'none'"),
    );
    assert!(
        !native_entry_frame_policy_is_closed(&headers),
        "ambiguous policy multiplicity accepted"
    );
}

fn native_entry_html(response: &Response, expected: StatusCode) -> &str {
    assert_eq!(response.status, expected, "native SSR status differs");
    response.private();
    assert!(
        native_entry_frame_policy_is_closed(&response.headers),
        "SSR framing protection absent or ambiguous"
    );
    let vary: BTreeSet<_> = response
        .headers
        .get_all(header::VARY)
        .iter()
        .flat_map(|value| value.to_str().unwrap().split(','))
        .map(|value| value.trim().to_ascii_lowercase())
        .collect();
    assert!(
        vary.contains("cookie") && vary.contains("origin"),
        "native private response Vary boundary absent"
    );
    assert!(
        !response.headers.contains_key(header::SET_COOKIE),
        "read changed identity cookies"
    );
    assert!(
        response
            .headers
            .get(header::CONTENT_TYPE)
            .is_some_and(|value| {
                value
                    .to_str()
                    .is_ok_and(|value| value.starts_with("text/html"))
            }),
        "native entry must be SSR HTML"
    );
    let html = std::str::from_utf8(&response.bytes).unwrap();
    assert!(html.contains("<main"), "missing main landmark");
    assert!(html.contains("lang=\"ko\""), "document language missing");
    assert!(
        html.contains("name=\"viewport\""),
        "viewport metadata missing"
    );
    html
}

fn native_entry_no_business_navigation(html: &str) {
    for forbidden in [
        "href=\"/payroll\"",
        "href=\"/people\"",
        "href=\"/organization\"",
        "data-run-id=",
        "data-person-id=",
    ] {
        assert!(
            !html.contains(forbidden),
            "Account entry inferred business authority"
        );
    }
}

#[sqlx::test(migrations = false)]
async fn native_entry_public_root_exposes_discoverable_routes(pool: PgPool) {
    let app = fixture(&pool).await;
    let before = native_extension_rows(&pool).await;
    let response = native_entry_get(
        &app,
        "/",
        &Cookies::default(),
        "none",
        "navigate",
        "document",
        &[],
    )
    .await;
    let html = native_entry_html(&response, StatusCode::OK);
    for required in [
        "href=\"/account\"",
        "href=\"/account/register\"",
        "로그인",
        "계정 만들기",
    ] {
        assert!(
            html.contains(required),
            "public native entry action missing"
        );
    }
    native_entry_no_business_navigation(html);
    assert!(
        native_extension_rows_equal(&before, &native_extension_rows(&pool).await),
        "public GET created or changed auth state"
    );
}

#[sqlx::test(migrations = false)]
async fn native_entry_anonymous_navigation_has_actionable_ssr_login(pool: PgPool) {
    let app = fixture(&pool).await;
    let before = native_extension_rows(&pool).await;
    let response = native_entry_get(
        &app,
        "/account",
        &Cookies::default(),
        "none",
        "navigate",
        "document",
        &[],
    )
    .await;
    let html = native_entry_html(&response, StatusCode::OK);
    assert!(html.contains("data-account-state=\"anonymous\""));
    assert!(html.contains("패스키로 로그인"));
    assert!(html.contains("href=\"/account/register\""));
    assert!(!html.contains("data-account-state=\"active\""));
    assert!(
        !html.contains("data-context-state=\"empty\""),
        "anonymous is not an authorized empty context"
    );
    native_entry_no_business_navigation(html);
    assert!(
        native_extension_rows_equal(&before, &native_extension_rows(&pool).await),
        "GET started a ceremony"
    );
}

#[sqlx::test(migrations = false)]
async fn native_entry_registration_read_presents_exact_terms_without_starting_attempt(
    pool: PgPool,
) {
    let app = fixture(&pool).await;
    let before = native_extension_rows(&pool).await;
    let response = native_entry_get(
        &app,
        "/account/register",
        &Cookies::default(),
        "none",
        "navigate",
        "document",
        &[],
    )
    .await;
    let html = native_entry_html(&response, StatusCode::OK);
    for item in manifest()["items"].as_array().unwrap() {
        assert!(
            html.contains(item["title"].as_str().unwrap()),
            "required terms title not presented"
        );
        let content = format!(
            "/api/v2/auth/terms/content/{}",
            item["content_sha256"].as_str().unwrap()
        );
        assert!(
            html.contains(&content),
            "exact release content entry absent"
        );
        assert!(
            html.contains(item["terms_kind"].as_str().unwrap()),
            "required acknowledgement identity absent"
        );
    }
    assert!(html.contains("패스키로 계정 만들기"));
    // DOM label/control binding and unchecked properties require separate browser acceptance.
    native_entry_no_business_navigation(html);
    assert!(
        native_extension_rows_equal(&before, &native_extension_rows(&pool).await),
        "terms GET created a pending Account"
    );
}

#[sqlx::test(migrations = false)]
async fn native_entry_live_cookie_renders_account_without_company_or_secret_payload(pool: PgPool) {
    let app = fixture(&pool).await;
    let (attempt, cookies) = enrolled(&app).await;
    let before = native_extension_rows(&pool).await;
    let response = native_entry_get(
        &app,
        "/account",
        &cookies,
        "none",
        "navigate",
        "document",
        &[],
    )
    .await;
    let html = native_entry_html(&response, StatusCode::OK);
    assert!(html.contains("data-account-state=\"active\""));
    assert!(html.contains("data-context-state=\"empty\""));
    assert!(html.contains("로그아웃"));
    native_entry_no_business_navigation(html);
    assert_no_company_identity(&pool, attempt.account).await;
    assert!(
        native_extension_rows_equal(&before, &native_extension_rows(&pool).await),
        "Account SSR mutated owner state"
    );
}

#[sqlx::test(migrations = false)]
async fn native_entry_external_document_navigation_uses_only_current_lax_access(pool: PgPool) {
    let app = fixture(&pool).await;
    let (attempt, mut cookies) = enrolled(&app).await;
    cookies.0.retain(|name, _| name == ACCESS); // Browser Strict cookies absent on initial external GET.
    let before = native_extension_rows(&pool).await;
    // Valid external same-site navigation is treated like cross-site navigation;
    // all metadata absent is the explicit legacy-user-agent compatibility branch.
    for (site, mode, destination) in [
        ("none", "navigate", "document"),
        ("same-origin", "navigate", "document"),
        ("same-site", "navigate", "document"),
        ("cross-site", "navigate", "document"),
        ("", "", ""),
    ] {
        let response =
            native_entry_get(&app, "/account", &cookies, site, mode, destination, &[]).await;
        let html = native_entry_html(&response, StatusCode::OK);
        assert!(html.contains("data-account-state=\"active\""));
        assert!(html.contains("data-context-state=\"empty\""));
        native_entry_no_business_navigation(html);
    }
    assert_no_company_identity(&pool, attempt.account).await;
    assert!(
        native_extension_rows_equal(&before, &native_extension_rows(&pool).await),
        "external SSR refreshed or changed identity"
    );
}

#[sqlx::test(migrations = false)]
async fn native_entry_cross_site_subresource_cannot_read_account_projection(pool: PgPool) {
    let app = fixture(&pool).await;
    let (attempt, cookies) = enrolled(&app).await;
    let before = native_extension_rows(&pool).await;
    let cases = [
        (
            "cross-site",
            "no-cors",
            "image",
            vec![],
            StatusCode::FORBIDDEN,
        ),
        (
            "cross-site",
            "navigate",
            "iframe",
            vec![],
            StatusCode::FORBIDDEN,
        ),
        (
            "same-origin",
            "cors",
            "empty",
            vec![],
            StatusCode::FORBIDDEN,
        ),
        ("none", "navigate", "image", vec![], StatusCode::FORBIDDEN),
        (
            "cross-site",
            "",
            "document",
            vec![],
            StatusCode::BAD_REQUEST,
        ),
        (
            "invalid",
            "navigate",
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
            vec![("Sec-Fetch-Site", "same-origin")],
            StatusCode::BAD_REQUEST,
        ),
        (
            "same-origin",
            "navigate",
            "document",
            vec![("Sec-Fetch-Mode", "navigate")],
            StatusCode::BAD_REQUEST,
        ),
        (
            "same-origin",
            "navigate",
            "document",
            vec![("Sec-Fetch-Dest", "document")],
            StatusCode::BAD_REQUEST,
        ),
        (
            "same-origin",
            "navigate",
            "document",
            vec![("Origin", TEST_ORIGIN), ("Origin", TEST_ORIGIN)],
            StatusCode::BAD_REQUEST,
        ),
        (
            "same-origin",
            "navigate",
            "document",
            vec![("Sec-Fetch-User", "?0")],
            StatusCode::BAD_REQUEST,
        ),
        (
            "same-origin",
            "navigate",
            "document",
            vec![("Sec-Fetch-User", "?1"), ("Sec-Fetch-User", "?1")],
            StatusCode::BAD_REQUEST,
        ),
    ];
    for (site, mode, destination, extra, status) in &cases {
        let response =
            native_entry_get(&app, "/account", &cookies, site, mode, destination, extra).await;
        let html = native_entry_html(&response, *status);
        assert!(!html.contains("data-account-state=\"active\""));
        assert!(!html.contains("data-context-state=\"empty\""));
        assert!(
            !html.contains(&attempt.account.to_string()),
            "denied Account identity disclosed"
        );
    }
    assert!(native_extension_rows_equal(
        &before,
        &native_extension_rows(&pool).await
    ));
}

#[sqlx::test(migrations = false)]
async fn native_entry_expired_access_never_silently_consumes_refresh(pool: PgPool) {
    let (app, key) = signed_fixture(&pool).await;
    let (attempt, mut cookies) = enrolled(&app).await;
    let mut claims = signed_claims(&cookies.0[ACCESS], &key).unwrap();
    let now: OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
        .fetch_one(&pool)
        .await
        .unwrap();
    claims["exp"] = json!(now.unix_timestamp() - 1);
    cookies
        .0
        .insert(ACCESS.to_owned(), sign_proof_claims(&claims, &key));
    let before = native_extension_rows(&pool).await;
    let response = native_entry_get(
        &app,
        "/account",
        &cookies,
        "same-origin",
        "navigate",
        "document",
        &[],
    )
    .await;
    let html = native_entry_html(&response, StatusCode::OK);
    assert!(html.contains("data-account-state=\"anonymous\""));
    assert!(!html.contains("data-account-state=\"active\""));
    assert!(!html.contains(&attempt.account.to_string()));
    assert!(
        native_extension_rows_equal(&before, &native_extension_rows(&pool).await),
        "protected SSR performed silent refresh"
    );
}

#[sqlx::test(migrations = false)]
async fn native_entry_ambiguous_cookie_and_bearer_never_fall_back_to_legacy_identity(pool: PgPool) {
    let app = fixture(&pool).await;
    let (attempt, cookies) = enrolled(&app).await;
    let before = native_extension_rows(&pool).await;
    let duplicate = format!("{ACCESS}={}", cookies.0[ACCESS]);
    let bearer = format!("Bearer {}", cookies.0[ACCESS]);
    for extra in [
        [("Cookie", duplicate.as_str())],
        [("Authorization", bearer.as_str())],
    ] {
        let response = native_entry_get(
            &app,
            "/account",
            &cookies,
            "same-origin",
            "navigate",
            "document",
            &extra,
        )
        .await;
        let html = native_entry_html(&response, StatusCode::BAD_REQUEST);
        assert!(!html.contains("data-account-state=\"active\""));
        assert!(!html.contains(&attempt.account.to_string()));
        native_entry_no_business_navigation(html);
    }
    assert!(native_extension_rows_equal(
        &before,
        &native_extension_rows(&pool).await
    ));
}

// Root selection is presentation over the existing native owner. These cases
// compare full owner state, not just response status or row counts.
async fn native_root_snapshot(pool: &PgPool) -> BTreeMap<String, String> {
    let mut rows = native_document_snapshot(pool).await;
    let audit: String = sqlx::query_scalar(
        "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text COLLATE \"C\"),'[]'::jsonb)::text FROM public.audit_events t"
    ).fetch_one(pool).await.unwrap();
    assert!(serde_json::from_str::<Value>(&audit).unwrap().is_array());
    assert!(rows.insert("audit_events".to_owned(), audit).is_none());
    rows
}

fn native_root_public(response: &Response) {
    let html = native_entry_html(response, StatusCode::OK);
    assert!(
        html == console_payroll_ui::native_account::render(
            console_payroll_ui::native_account::Page::Public
        )
    );
    for action in [
        "href=\"/account\"",
        "href=\"/account/register\"",
        "로그인",
        "계정 만들기",
    ] {
        assert!(
            html.contains(action),
            "ROOT_PUBLIC: discoverable action absent"
        );
    }
    assert!(!html.contains("data-account-state="));
    assert!(!html.contains("data-context-state="));
    assert!(!html.contains("leptos-island") && !html.contains("/pkg/"));
    native_entry_no_business_navigation(html);
}

#[sqlx::test(migrations = false)]
async fn native_root_unrelated_cookies_are_public_without_owner_effects(pool: PgPool) {
    let app = fixture(&pool).await;
    let before = native_root_snapshot(&pool).await;
    for cookie in [
        "theme=root-theme-marker",
        "console_refresh=root-legacy-refresh-marker",
        "console_session=root-proposed-session-marker",
        "__Host-console_account_session_extra=root-near-name-marker",
        "theme=root-theme-marker; locale=root-locale-marker",
    ] {
        let response = native_entry_get(
            &app,
            "/",
            &Cookies::default(),
            "none",
            "navigate",
            "document",
            &[("Cookie", cookie)],
        )
        .await;
        native_root_public(&response);
        assert!(
            before == native_root_snapshot(&pool).await,
            "ROOT_PUBLIC: unknown cookies changed state"
        );
    }
}

#[sqlx::test(migrations = false)]
async fn native_root_native_cookie_classes_and_head_preserve_owner_selection(pool: PgPool) {
    let app = fixture(&pool).await;
    let (attempt, cookies) = enrolled(&app).await;
    let before = native_root_snapshot(&pool).await;
    let root = native_document_get(&app, "/", &cookies).await;
    let account = native_document_get(&app, "/account", &cookies).await;
    let html = native_entry_html(&root, StatusCode::OK);
    native_entry_html(&account, StatusCode::OK);
    assert!(
        root.bytes == account.bytes,
        "ROOT_NATIVE: active projection differs from owner"
    );
    assert!(
        html.contains("data-account-state=\"active\"")
            && html.contains("data-context-state=\"empty\"")
    );
    native_entry_no_business_navigation(html);
    assert_no_company_identity(&pool, attempt.account).await;

    // HEAD makes all four supplied native classes observably distinct from
    // anonymous Public. Refresh/enrollment/login alone are never consumed.
    for name in [ACCESS, REFRESH, ENROLLMENT, LOGIN] {
        let value = if name == ACCESS {
            cookies.0[ACCESS].clone()
        } else {
            "root-unused-proof-marker".to_owned()
        };
        let presented = Cookies(BTreeMap::from([(name.to_owned(), value)]));
        let head = request(
            &app,
            "HEAD",
            "/",
            &presented,
            None,
            &[
                ("Sec-Fetch-Mode", "navigate"),
                ("Sec-Fetch-Dest", "document"),
            ],
        )
        .await;
        assert_eq!(
            head.status,
            StatusCode::METHOD_NOT_ALLOWED,
            "ROOT_NATIVE_HEAD: wrong branch for {name}"
        );
        assert!(head.bytes.is_empty());
        assert!(!head.headers.contains_key(header::SET_COOKIE));
        head.private();
        if name != ACCESS {
            native_root_public(&native_document_get(&app, "/", &presented).await);
        }
        assert!(
            before == native_root_snapshot(&pool).await,
            "ROOT_NATIVE_HEAD: credential class caused effects"
        );
    }
    for extra in [vec![], vec![("Cookie", "theme=root-head-theme-marker")]] {
        let head = request(&app, "HEAD", "/", &Cookies::default(), None, &extra).await;
        assert_eq!(head.status, StatusCode::OK);
        assert!(head.bytes.is_empty());
        assert!(!head.headers.contains_key(header::SET_COOKIE));
        head.private();
        assert!(native_entry_frame_policy_is_closed(&head.headers));
    }
    assert!(before == native_root_snapshot(&pool).await);
}

#[sqlx::test(migrations = false)]
async fn native_root_expired_and_revoked_access_never_consumes_refresh(pool: PgPool) {
    let (app, key) = signed_fixture(&pool).await;
    let (attempt, cookies) = enrolled(&app).await;
    let mut expired = cookies.clone();
    let mut claims = signed_claims(&expired.0[ACCESS], &key).unwrap();
    let now: OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
        .fetch_one(&pool)
        .await
        .unwrap();
    claims["exp"] = json!(now.unix_timestamp() - 1);
    expired
        .0
        .insert(ACCESS.to_owned(), sign_proof_claims(&claims, &key));
    let before = native_root_snapshot(&pool).await;
    let response = native_document_get(&app, "/", &expired).await;
    native_root_public(&response);
    assert!(
        !std::str::from_utf8(&response.bytes)
            .unwrap()
            .contains(&attempt.account.to_string())
    );
    assert!(
        before == native_root_snapshot(&pool).await,
        "ROOT_EXPIRED: silent refresh or state repair"
    );
    let current = native_document_get(&app, "/", &cookies).await;
    assert!(native_entry_html(&current, StatusCode::OK).contains("data-account-state=\"active\""));
    assert!(before == native_root_snapshot(&pool).await);

    // Revoke through the existing command owner, retaining stale browser bytes.
    let scoped_before = snapshot(&pool, attempt.account).await;
    let csrf = proof(&app, &cookies).await;
    let logout = request(
        &app,
        "POST",
        "/api/v2/auth/logout",
        &cookies,
        Some(json!({})),
        &[("X-Console-CSRF", &csrf)],
    )
    .await;
    assert!(logout.json(StatusCode::OK) == json!({"outcome":"COMMITTED"}));
    logout.private();
    assert_logout_transition(
        &scoped_before,
        &snapshot(&pool, attempt.account).await,
        attempt.account,
    );
    let revoked = native_root_snapshot(&pool).await;
    let response = native_document_get(&app, "/", &cookies).await;
    native_root_public(&response);
    assert!(
        !std::str::from_utf8(&response.bytes)
            .unwrap()
            .contains(&attempt.account.to_string())
    );
    assert!(
        revoked == native_root_snapshot(&pool).await,
        "ROOT_REVOKED: resurrected or refreshed session"
    );
}

fn native_root_parser_refusal(response: &Response, expected: StatusCode) {
    if expected == StatusCode::PAYLOAD_TOO_LARGE {
        // The retained outer HTTP envelope replaces native HTML413 with JSON.
        // Pin the actual mounted contract; do not pretend its privacy headers survive.
        assert!(
            response.json(expected)
                == json!({"error": {
                    "code": "payload_too_large", "message": "request body too large"
                }})
        );
        assert!(
            response
                .headers
                .get(header::CONTENT_TYPE)
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("application/json")
        );
        assert!(!response.headers.contains_key(header::SET_COOKIE));
        response.no_literal_echo();
    } else {
        native_entry_html(response, expected);
    }
}

#[sqlx::test(migrations = false)]
async fn native_root_cookie_refusals_preserve_native_parser_precedence(pool: PgPool) {
    let app = fixture(&pool).await;
    let before = native_root_snapshot(&pool).await;
    let oversized = format!("theme={}", "z".repeat(16 * 1024 + 1));
    let bearer = "Bearer root-ambiguity-marker";
    let mut cases = vec![
        (
            vec![("Cookie", oversized.clone())],
            StatusCode::PAYLOAD_TOO_LARGE,
        ),
        (
            vec![
                ("Cookie", oversized.clone()),
                ("Authorization", bearer.to_owned()),
            ],
            StatusCode::BAD_REQUEST,
        ),
    ];
    for name in [ACCESS, REFRESH, ENROLLMENT, LOGIN] {
        let supplied = format!("{name}=root-proof-marker");
        cases.extend([
            (
                vec![("Cookie", format!("{name}="))],
                StatusCode::BAD_REQUEST,
            ),
            (
                vec![("Cookie", format!("{name} =root-proof-marker"))],
                StatusCode::BAD_REQUEST,
            ),
            (
                vec![("Cookie", format!("{supplied}; {supplied}"))],
                StatusCode::BAD_REQUEST,
            ),
            (
                vec![("Cookie", supplied.clone()), ("Cookie", supplied.clone())],
                StatusCode::BAD_REQUEST,
            ),
            (
                vec![
                    ("Cookie", supplied.clone()),
                    ("Authorization", bearer.to_owned()),
                ],
                StatusCode::BAD_REQUEST,
            ),
            (
                vec![
                    ("Cookie", format!("{supplied}; {supplied}")),
                    ("Authorization", bearer.to_owned()),
                ],
                StatusCode::BAD_REQUEST,
            ),
        ]);
    }
    for (headers, expected) in cases {
        let headers: Vec<_> = headers
            .iter()
            .map(|(name, value)| (*name, value.as_str()))
            .collect();
        let account = native_entry_get(
            &app,
            "/account",
            &Cookies::default(),
            "same-origin",
            "navigate",
            "document",
            &headers,
        )
        .await;
        native_root_parser_refusal(&account, expected); // Actual mounted owner is independently pinned.
        let root = native_entry_get(
            &app,
            "/",
            &Cookies::default(),
            "same-origin",
            "navigate",
            "document",
            &headers,
        )
        .await;
        native_root_parser_refusal(&root, expected);
        assert!(
            root.bytes == account.bytes,
            "ROOT_PARSER: root bypassed exact native refusal"
        );
        assert!(
            before == native_root_snapshot(&pool).await,
            "ROOT_PARSER: refused input changed state"
        );
    }
    // HeaderMap permits opaque field bytes; the owner refuses them rather than
    // treating a failed UTF8 conversion as absent/native-free credentials.
    for path in ["/account", "/"] {
        let mut req = Request::builder()
            .uri(path)
            .header("Sec-Fetch-Site", "same-origin")
            .header("Sec-Fetch-Mode", "navigate")
            .header("Sec-Fetch-Dest", "document")
            .header(
                header::COOKIE,
                http::HeaderValue::from_bytes(b"theme=\xff").unwrap(),
            )
            .body(Body::empty())
            .unwrap();
        req.extensions_mut().insert(ConnectInfo(
            "127.0.0.1:41000".parse::<SocketAddr>().unwrap(),
        ));
        let (parts, body) = app.service.clone().oneshot(req).await.unwrap().into_parts();
        let response = Response {
            status: parts.status,
            headers: parts.headers,
            bytes: to_bytes(body, 256 * 1024).await.unwrap().to_vec(),
            sent_secrets: vec![],
        };
        native_entry_html(&response, StatusCode::BAD_REQUEST);
        assert!(before == native_root_snapshot(&pool).await);
    }
}

#[sqlx::test(migrations = false)]
async fn native_root_document_admission_matches_native_owner_before_credentials(pool: PgPool) {
    let app = fixture(&pool).await;
    let (_, cookies) = enrolled(&app).await;
    let before = native_root_snapshot(&pool).await;
    let oversized = format!("theme={}", "z".repeat(16 * 1024 + 1));
    for (site, mode, destination, extra, expected) in [
        (
            "cross-site",
            "no-cors",
            "image",
            vec![],
            StatusCode::FORBIDDEN,
        ),
        (
            "cross-site",
            "navigate",
            "iframe",
            vec![],
            StatusCode::FORBIDDEN,
        ),
        (
            "same-origin",
            "navigate",
            "document",
            vec![("Origin", "https://foreign.invalid")],
            StatusCode::FORBIDDEN,
        ),
        ("none", "", "document", vec![], StatusCode::BAD_REQUEST),
        // Metadata admission precedes even Authorization/cookie ambiguity.
        (
            "cross-site",
            "no-cors",
            "image",
            vec![
                ("Authorization", "Bearer root-metadata-marker"),
                ("Cookie", oversized.as_str()),
            ],
            StatusCode::FORBIDDEN,
        ),
    ] {
        let account =
            native_entry_get(&app, "/account", &cookies, site, mode, destination, &extra).await;
        native_entry_html(&account, expected);
        let root = native_entry_get(&app, "/", &cookies, site, mode, destination, &extra).await;
        native_entry_html(&root, expected);
        assert!(
            root.bytes == account.bytes,
            "ROOT_ADMISSION: current native owner precedence changed"
        );
        assert!(before == native_root_snapshot(&pool).await);
    }
    // Valid cross-site top-level navigation remains permitted with live access.
    let root = native_entry_get(
        &app,
        "/",
        &cookies,
        "cross-site",
        "navigate",
        "document",
        &[],
    )
    .await;
    assert!(native_entry_html(&root, StatusCode::OK).contains("data-account-state=\"active\""));
    assert!(before == native_root_snapshot(&pool).await);
}

#[sqlx::test(migrations = false)]
async fn native_root_invalid_legacy_authorization_never_becomes_anonymous_public(pool: PgPool) {
    let app = fixture(&pool).await;
    let before = native_root_snapshot(&pool).await;
    for headers in [
        vec![("Authorization", "")],
        vec![("Authorization", "Bearer root-invalid-legacy-marker")],
        vec![
            ("Authorization", "Bearer root-invalid-legacy-marker"),
            ("Authorization", "Bearer root-second-legacy-marker"),
        ],
        vec![
            ("Authorization", "Bearer root-invalid-legacy-marker"),
            ("Cookie", "theme=root-legacy-error-theme-marker"),
        ],
    ] {
        let root = native_entry_get(
            &app,
            "/",
            &Cookies::default(),
            "none",
            "navigate",
            "document",
            &headers,
        )
        .await;
        let work = native_entry_get(
            &app,
            "/work",
            &Cookies::default(),
            "none",
            "navigate",
            "document",
            &headers,
        )
        .await;
        assert_eq!(root.status, StatusCode::OK);
        assert_eq!(work.status, StatusCode::OK);
        root.no_literal_echo();
        assert!(!root.headers.contains_key(header::SET_COOKIE));
        assert!(root.bytes == work.bytes);
        assert!(std::str::from_utf8(&root.bytes).unwrap() == console_payroll_ui::render_shell());
        assert!(before == native_root_snapshot(&pool).await);
    }
}

#[sqlx::test(migrations = false)]
async fn native_root_parser_refactor_preserves_api_authorization_first(pool: PgPool) {
    let app = fixture(&pool).await;
    let before = native_root_snapshot(&pool).await;
    let oversized = format!("theme={}", "z".repeat(16 * 1024 + 1));
    let mut values = vec![oversized];
    for name in [ACCESS, REFRESH, ENROLLMENT, LOGIN] {
        values.push(format!("{name}="));
        values.push(format!(
            "{name}=root-api-proof-marker; {name}=root-api-proof-marker"
        ));
    }
    for cookie in values {
        request(
            &app,
            "GET",
            "/api/v2/accounts/me",
            &Cookies::default(),
            None,
            &[
                ("Cookie", cookie.as_str()),
                ("Authorization", "Bearer root-api-ambiguity-marker"),
            ],
        )
        .await
        .error(StatusCode::BAD_REQUEST, "ambiguous_credentials");
        assert!(
            before == native_root_snapshot(&pool).await,
            "ROOT_PARSER_API: rejection ordering changed owner state"
        );
    }
}
