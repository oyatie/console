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
