// Actual mounted routes, real Account/Company owners and production Cedar.
// Transport tests may use owner fixtures; populated browser acceptance may not.
mod native_payroll_transport_tests {
    use super::*;

    fn paths(f: &FixtureRead) -> [String; 2] {
        [
            format!("/companies/{}/payroll", f.created.company),
            format!("/api/v1/companies/{}/payroll/runs", f.created.company),
        ]
    }
    async fn read_route(f: &FixtureRead, path: &str, cookies: &Cookies) -> Response {
        let response = if path.starts_with("/api/") {
            request(
                &f.app,
                "GET",
                path,
                cookies,
                None,
                &[("Sec-Fetch-Mode", "cors"), ("Sec-Fetch-Dest", "empty")],
            )
            .await
        } else {
            document(&f.app, path, cookies).await
        };
        shape(&response, path);
        response
    }
    fn shape(response: &Response, path: &str) {
        assert!(
            !response.bytes.is_empty(),
            "native GET/non-HEAD response body required"
        );
        assert_eq!(
            response
                .headers
                .get_all(header::CONTENT_TYPE)
                .iter()
                .count(),
            1
        );
        if !path.starts_with("/api/") {
            assert_eq!(
                response
                    .headers
                    .get_all("content-security-policy")
                    .iter()
                    .count(),
                1
            );
            assert_eq!(
                response.headers.get("content-security-policy").unwrap(),
                "default-src 'self'; script-src 'none'; style-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'"
            );
            assert_eq!(
                response.headers.get_all("referrer-policy").iter().count(),
                1
            );
            assert_eq!(
                response.headers.get("referrer-policy").unwrap(),
                "no-referrer"
            );
        }
        let content = response
            .headers
            .get(header::CONTENT_TYPE)
            .expect("native response content type")
            .to_str()
            .unwrap();
        assert!(
            content.split(';').next().unwrap().trim()
                == if path.starts_with("/api/") {
                    "application/json"
                } else {
                    "text/html"
                },
            "wrong transport representation for {path}"
        );
    }
    fn private(response: &Response) {
        response.private();
        for name in ["cache-control", "pragma", "x-content-type-options", "vary"] {
            assert_eq!(
                response.headers.get_all(name).iter().count(),
                1,
                "duplicate {name}"
            );
        }
        assert_eq!(response.headers.get("cache-control").unwrap(), "no-store");
        assert!(!response.headers.contains_key(header::SET_COOKIE));
        assert_eq!(
            response.headers.get("x-content-type-options").unwrap(),
            "nosniff"
        );
        let vary = response
            .headers
            .get("vary")
            .unwrap()
            .to_str()
            .unwrap()
            .to_ascii_lowercase();
        for key in ["cookie", "origin"] {
            assert!(vary.split(',').any(|v| v.trim() == key));
        }
    }
    async fn success(pool: &PgPool, f: &FixtureRead, path: &str, limit: i64, offset: i64) {
        let before = all_rows(pool).await;
        let response = read_route(f, path, &f.cookies).await;
        assert_eq!(
            response.status,
            StatusCode::OK,
            "mounted native route {path}"
        );
        private(&response);
        if path.starts_with("/api/") {
            assert_eq!(
                response.json(StatusCode::OK),
                json!({"items":[],"total":0,"limit":limit,"offset":offset})
            );
        } else {
            let body = String::from_utf8(response.bytes.clone()).unwrap();
            assert!(body.contains("data-screen=\"payroll\""));
            assert!(body.contains("data-state=\"empty\""));
            assert!(body.contains("연결된 업무 회사"));
            assert!(!body.contains("<script"));
            assert!(
                body.contains(&format!("<option value=\"{limit}\" selected")),
                "SSR dropped normalized limit"
            );
            if offset > 0 {
                assert!(
                    body.contains(&format!(
                        "?limit={limit}&amp;offset={}",
                        offset.saturating_sub(limit).max(0)
                    )) || body.contains(&format!(
                        "?limit={limit}&offset={}",
                        offset.saturating_sub(limit).max(0)
                    )),
                    "SSR dropped offset"
                );
                assert!(body.contains("rel=\"prev\""));
            } else {
                assert!(!body.contains("rel=\"prev\""));
            }
            assert_eq!(
                response.headers.get("referrer-policy").unwrap(),
                "no-referrer"
            );
            assert!(
                response
                    .headers
                    .get("content-security-policy")
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .contains("script-src 'none'")
            );
        }
        let after = all_rows(pool).await;
        assert!(before.keys().eq(after.keys()));
        for (table, value) in &before {
            if table != "audit_events" {
                assert!(value == &after[table], "changed {table}");
            }
        }
        let audits = added_rows(&before["audit_events"], &after["audit_events"]).unwrap();
        assert_eq!(audits.len(), 1, "exactly one committed read audit");
        let a = &audits[0];
        assert_eq!(a["actor"], json!(f.created.administrator));
        assert_eq!(a["org_id"], json!(f.created.company));
        assert_eq!(a["action"], "payroll_run.list_read");
        assert_eq!(a["target_type"], "payroll_draft_run");
        assert_eq!(a["target_id"], "query");
    }
    fn refused(response: &Response, status: StatusCode, f: &FixtureRead) {
        assert_eq!(response.status, status);
        private(response);
        let body = String::from_utf8(response.bytes.clone()).unwrap();
        for forbidden in [
            "data-screen=\"payroll\"",
            "data-run-id",
            "<form",
            "data-state=\"empty\"",
            "연결된 업무 회사",
            "private-read-fault",
        ] {
            assert!(!body.contains(forbidden), "disclosed {forbidden}");
        }
        assert!(!body.contains(&f.created.company.to_string()));
        assert!(!body.contains(&format!("native-{}", f.created.command.simple())));
        // Source name and command-based slug are real Company fixture values.
        if !response.bytes.is_empty() {
            let content = response
                .headers
                .get(header::CONTENT_TYPE)
                .unwrap()
                .to_str()
                .unwrap();
            if content.starts_with("application/json") {
                let value = response.json(status);
                exact_keys(&value, &["error"]);
                exact_keys(&value["error"], &["code", "message"]);
                assert!(
                    value["error"]["message"]
                        .as_str()
                        .is_some_and(|s| !s.is_empty() && s.len() <= 256)
                );
                assert!(
                    value["error"]["code"]
                        .as_str()
                        .is_some_and(|s| !s.is_empty())
                );
            } else {
                assert!(content.starts_with("text/html"));
                assert!(body.starts_with(concat!("<!", "DOCTYPE html>")));
                assert_eq!(
                    response
                        .headers
                        .get_all("content-security-policy")
                        .iter()
                        .count(),
                    1
                );
                assert_eq!(
                    response.headers.get("content-security-policy").unwrap(),
                    "default-src 'self'; script-src 'none'; style-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'"
                );
            }
        }
        assert!(!response.headers.contains_key(header::SET_COOKIE));
    }
    #[sqlx::test(migrations = false)]
    async fn native_routes_reopen_paginate_audit_revoke_and_logout(pool: PgPool) {
        let f = FixtureRead::new(&pool).await;
        let result = AssertUnwindSafe(async {
            for path in paths(&f) {
                let before = all_rows(&pool).await;
                refused(
                    &read_route(&f, &path, &f.cookies).await,
                    StatusCode::NOT_FOUND,
                    &f,
                );
                assert!(before == all_rows(&pool).await);
            }
            f.install().await;
            let assignment = f.grant(&pool).await;
            for path in paths(&f) {
                success(&pool, &f, &path, 100, 0).await;
                success(&pool, &f, &path, 100, 0).await;
                success(&pool, &f, &format!("{path}?limit=999&offset=-2"), 500, 0).await;
                success(&pool, &f, &format!("{path}?limit=0&offset=23"), 1, 23).await;
            }
            let revoked = f
                .command(
                    NativeCompanyBusinessCommandV1::revoke(
                        Uuid::new_v4(),
                        f.company(),
                        3,
                        assignment,
                    )
                    .unwrap(),
                )
                .await;
            assert!(matches!(
                revoked.outcome,
                NativePolicyOutcome::Committed(NativePolicyEffect::Revoked { .. })
            ));
            for path in paths(&f) {
                let before = all_rows(&pool).await;
                refused(
                    &read_route(&f, &path, &f.cookies).await,
                    StatusCode::NOT_FOUND,
                    &f,
                );
                assert!(before == all_rows(&pool).await);
            }
            let csrf = proof(&f.app, &f.cookies).await;
            assert_eq!(
                request(
                    &f.app,
                    "POST",
                    "/api/v2/auth/logout",
                    &f.cookies,
                    Some(json!({})),
                    &[("X-Console-CSRF", &csrf)]
                )
                .await
                .status,
                StatusCode::OK
            );
            for path in paths(&f) {
                let before = all_rows(&pool).await;
                refused(
                    &read_route(&f, &path, &f.cookies).await,
                    StatusCode::UNAUTHORIZED,
                    &f,
                );
                assert!(before == all_rows(&pool).await);
            }
        })
        .catch_unwind()
        .await;
        f.close(result).await;
    }
    #[sqlx::test(migrations = false)]
    async fn native_routes_reject_ambiguous_selectors_credentials_and_head_without_effects(
        pool: PgPool,
    ) {
        let f = FixtureRead::new(&pool).await;
        let result = AssertUnwindSafe(async {
            f.install().await;
            f.grant(&pool).await;
            let (_, foreign) = enrolled(&f.app).await;
            for path in paths(&f) {
                success(&pool, &f, &path, 100, 0).await;
                let before = all_rows(&pool).await;
                for suffix in [
                    "?limit=1&limit=2",
                    "?offset=0&offset=0",
                    "?unknown=1",
                    "?limit=x",
                    "?limit=9223372036854775808",
                    "?offset=",
                    "?limit=1.0",
                    "?limit=1&",
                ] {
                    refused(
                        &read_route(&f, &format!("{path}{suffix}"), &f.cookies).await,
                        StatusCode::BAD_REQUEST,
                        &f,
                    );
                }
                for target in [
                    Uuid::nil().to_string(),
                    OrgId::platform().to_string(),
                    f.created.company.simple().to_string(),
                    f.created.company.to_string().to_uppercase(),
                ] {
                    let path = path.replace(&f.created.company.to_string(), &target);
                    refused(
                        &read_route(&f, &path, &f.cookies).await,
                        StatusCode::NOT_FOUND,
                        &f,
                    );
                }
                refused(
                    &read_route(&f, &path, &foreign).await,
                    StatusCode::NOT_FOUND,
                    &f,
                );
                refused(
                    &read_route(&f, &path, &Cookies::default()).await,
                    StatusCode::UNAUTHORIZED,
                    &f,
                );
                let mut refresh = f.cookies.clone();
                refresh.0.remove(ACCESS);
                refused(
                    &read_route(&f, &path, &refresh).await,
                    StatusCode::UNAUTHORIZED,
                    &f,
                );
                let duplicate = format!("{ACCESS}={}", f.cookies.0[ACCESS]);
                for extra in [
                    vec![("Cookie", duplicate.as_str())],
                    vec![("Authorization", "Bearer mixed-credential")],
                ] {
                    let response = native_entry_get(
                        &f.app,
                        &path,
                        &f.cookies,
                        "same-origin",
                        "navigate",
                        "document",
                        &extra,
                    )
                    .await;
                    shape(&response, &path);
                    refused(&response, StatusCode::BAD_REQUEST, &f);
                }
                for method in ["HEAD", "POST", "PUT", "DELETE"] {
                    let response = request(
                        &f.app,
                        method,
                        &path,
                        &f.cookies,
                        None,
                        &[
                            ("Sec-Fetch-Mode", "navigate"),
                            ("Sec-Fetch-Dest", "document"),
                        ],
                    )
                    .await;
                    if method == "HEAD" {
                        assert!(response.bytes.is_empty());
                    } else {
                        shape(&response, &path);
                    }
                    refused(&response, StatusCode::METHOD_NOT_ALLOWED, &f);
                }
                if !path.starts_with("/api/") {
                    let wrong = request(
                        &f.app,
                        "GET",
                        &path,
                        &f.cookies,
                        None,
                        &[("Sec-Fetch-Mode", "cors"), ("Sec-Fetch-Dest", "empty")],
                    )
                    .await;
                    shape(&wrong, &path);
                    refused(&wrong, StatusCode::FORBIDDEN, &f);
                }
                let cross = native_entry_get(
                    &f.app,
                    &path,
                    &f.cookies,
                    "cross-site",
                    "navigate",
                    "document",
                    &[("Origin", "https://other.example.com")],
                )
                .await;
                shape(&cross, &path);
                refused(&cross, StatusCode::FORBIDDEN, &f);
                assert!(
                    before == all_rows(&pool).await,
                    "rejected request wrote state/audit"
                );
                success(&pool, &f, &path, 100, 0).await;
            }
        })
        .catch_unwind()
        .await;
        f.close(result).await;
    }
    #[sqlx::test(migrations = false)]
    async fn native_routes_withhold_audit_failure_and_recover(pool: PgPool) {
        let f = FixtureRead::new(&pool).await;
        let result=AssertUnwindSafe(async {
            f.install().await;f.grant(&pool).await;
            for path in paths(&f) {success(&pool,&f,&path,100,0).await;}
            sqlx::raw_sql("CREATE SEQUENCE public.test_native_transport_witness; GRANT USAGE,SELECT ON SEQUENCE public.test_native_transport_witness TO console_rt; CREATE FUNCTION public.test_native_transport_refusal() RETURNS trigger LANGUAGE plpgsql SET search_path=pg_catalog,pg_temp AS $body$ BEGIN IF NEW.action='payroll_run.list_read' THEN PERFORM nextval('public.test_native_transport_witness'); RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='private-read-fault'; END IF; RETURN NEW; END $body$; CREATE TRIGGER test_native_transport_refusal BEFORE INSERT ON public.audit_events FOR EACH ROW EXECUTE FUNCTION public.test_native_transport_refusal()")
                .execute(&pool).await.unwrap();
            let before=all_rows(&pool).await;
            for (index,path) in paths(&f).into_iter().enumerate() {
                refused(&read_route(&f,&path,&f.cookies).await,StatusCode::SERVICE_UNAVAILABLE,&f);
                let fired:(bool,i64)=sqlx::query_as("SELECT is_called,last_value FROM public.test_native_transport_witness").fetch_one(&pool).await.unwrap();
                assert_eq!(fired,(true,index as i64+1),"each transport reached audit exactly once");
            }
            let fired:i64=sqlx::query_scalar("SELECT last_value FROM public.test_native_transport_witness").fetch_one(&pool).await.unwrap();
            assert_eq!(fired,2,"both transports reached actual audit owner");
            assert!(before==all_rows(&pool).await);
            sqlx::raw_sql("DROP TRIGGER test_native_transport_refusal ON public.audit_events").execute(&pool).await.unwrap();
            for path in paths(&f) {success(&pool,&f,&path,100,0).await;}
        }).catch_unwind().await;
        let cleanup=sqlx::raw_sql("DROP TRIGGER IF EXISTS test_native_transport_refusal ON public.audit_events; DROP FUNCTION IF EXISTS public.test_native_transport_refusal(); DROP SEQUENCE IF EXISTS public.test_native_transport_witness;").execute(&pool).await;
        f.close(result).await;
        cleanup.unwrap();
    }
}
