//! Mounted ops transport/failure boundaries; child of reviewed ops credentials.
use super::super::super::legacy_platform_list_credentials::no_raw_input_echo;
use super::*;

pub(super) fn ops_task(
    router: &axum::Router,
    token: &str,
    method: Method,
) -> JoinHandle<http::Response<Body>> {
    let router = router.clone();
    let token = token.to_owned();
    tokio::spawn(async move {
        router
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(OPS_PATH)
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap()
    })
}

#[sqlx::test(migrations = false)]
async fn mounted_ops_get_head_reject_ambiguous_headers_before_effects_and_accept_exact_bearer(
    pool: PgPool,
) {
    let (f, expected) = populated_fixture(&pool).await;
    for method in [Method::GET, Method::HEAD] {
        for case in [
            "absent",
            "cookie_only",
            "empty",
            "basic",
            "lowercase",
            "leading_space",
            "double_space",
            "trailing_space",
            "tab",
            "comma",
            "duplicate_same",
            "duplicate_conflict",
            "non_utf8",
        ] {
            let mut request = Request::builder()
                .method(method.clone())
                .uri(OPS_PATH)
                .body(Body::empty())
                .unwrap();
            let mut secrets = f.secrets.clone();
            let values: Vec<Vec<u8>> = match case {
                "absent" => vec![],
                "cookie_only" => {
                    request.headers_mut().insert(
                        header::COOKIE,
                        format!("access_token={}", f.access).parse().unwrap(),
                    );
                    vec![]
                }
                "empty" => vec![b"Bearer ".to_vec()],
                "basic" => vec![format!("Basic {}", f.access).into_bytes()],
                "lowercase" => vec![format!("bearer {}", f.access).into_bytes()],
                "leading_space" => vec![format!(" Bearer {}", f.access).into_bytes()],
                "double_space" => vec![format!("Bearer  {}", f.access).into_bytes()],
                "trailing_space" => vec![format!("Bearer {} ", f.access).into_bytes()],
                "tab" => vec![format!("Bearer\t{}", f.access).into_bytes()],
                "comma" => vec![format!("Bearer {},Bearer {}", f.access, f.access).into_bytes()],
                "duplicate_same" => vec![format!("Bearer {}", f.access).into_bytes(); 2],
                "duplicate_conflict" => vec![
                    format!("Bearer {}", f.access).into_bytes(),
                    b"Bearer other-private-credential".to_vec(),
                ],
                "non_utf8" => vec![b"Bearer \xff".to_vec()],
                _ => unreachable!(),
            };
            // Keep original bytes too: obs-text cannot be represented by the string oracle.
            let mut raw_inputs = values.clone();
            if let Some(cookie) = request.headers().get(header::COOKIE) {
                raw_inputs.push(cookie.as_bytes().to_vec());
            }
            for value in values {
                if let Ok(value) = std::str::from_utf8(&value)
                    && !value.is_empty()
                {
                    secrets.push(value.to_owned());
                }
                request.headers_mut().append(
                    header::AUTHORIZATION,
                    http::HeaderValue::from_bytes(&value).unwrap(),
                );
            }
            let before = all_rows(&pool).await;
            let response = f.router.clone().oneshot(request).await.unwrap();
            let (status, headers, body) = response_parts(response).await;
            assert!(
                no_raw_input_echo(&headers, &body, &raw_inputs),
                "raw current Authorization/Cookie input reflection: {case}"
            );
            assert!(
                status == StatusCode::UNAUTHORIZED && private_response(&headers, &body, &secrets),
                "exact early finite privacy: {case}"
            );
            if method == Method::HEAD {
                assert!(body.is_empty(), "HEAD refusal body must be empty");
            } else {
                assert!(
                    serde_json::from_slice::<Value>(&body).unwrap()
                        == json!({"error":{"code":"unauthorized","message":"invalid bearer token"}})
                );
            }
            assert!(
                before == all_rows(&pool).await,
                "early rejection must leave every table unchanged: {case}"
            );
        }
        let before = all_rows(&pool).await;
        let start = db_now(&pool).await;
        let response = f
            .router
            .clone()
            .oneshot(
                Request::builder()
                    .method(method.clone())
                    .uri(format!("{OPS_PATH}?display=all"))
                    .header(header::AUTHORIZATION, format!("Bearer {}", f.access))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let (status, headers, body) = response_parts(response).await;
        let end = db_now(&pool).await;
        assert!(status == StatusCode::OK && private_response(&headers, &body, &f.secrets));
        if method == Method::HEAD {
            assert!(body.is_empty());
        } else {
            assert!(health_matches(
                &serde_json::from_slice::<Value>(&body).unwrap(),
                &expected
            ));
        }
        assert!(
            health_read_delta(
                &before,
                &all_rows(&pool).await,
                f.actor,
                expected["tenants"].as_array().unwrap().len(),
                start,
                end
            ),
            "GET and actual HEAD fallback each execute one owned read"
        );
    }
    close_ops(f).await;
}

#[tokio::test]
async fn ops_transport_selection_preserves_list_and_sibling_routes_and_methods() {
    async fn sibling() -> http::Response<Body> {
        http::Response::builder()
            .status(StatusCode::IM_A_TEAPOT)
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::CACHE_CONTROL, "max-age=17")
            .header(header::ETAG, "fixture-validator")
            .header(header::VARY, "Origin")
            .body(Body::from("fixture sibling"))
            .unwrap()
    }
    let router =
        console_platform_rest::with_platform_list_transport(axum::Router::new().fallback(sibling));
    for (method, path, selected) in [
        (Method::GET, "/api/platform/ops", true),
        (Method::GET, "/api/platform/ops?display=all", true),
        (Method::HEAD, "/api/platform/ops", true),
        (Method::HEAD, "/api/platform/ops?display=all", true),
        (Method::GET, "/api/platform/orgs", true),
        (Method::HEAD, "/api/platform/orgs", true),
        (Method::GET, "/api/platform/ops/", false),
        (Method::GET, "/api/platform/groups", false),
        (Method::POST, "/api/platform/ops", false),
        (Method::PUT, "/api/platform/ops", false),
    ] {
        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(method.clone())
                    .uri(path)
                    // Wrapper selection needs a shaped header to reach next; this
                    // isolated fallback has no verifier and proves no authentication.
                    .header(header::AUTHORIZATION, "Bearer transport-fixture-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let (status, headers, body) = response_parts(response).await;
        assert!(
            status == StatusCode::IM_A_TEAPOT,
            "transport wrapper retains status"
        );
        if selected {
            assert!(private_response(
                &headers,
                &body,
                &["transport-fixture-token".to_owned()]
            ));
            let vary = token_set(&headers, header::VARY).unwrap();
            assert!(
                vary.iter().filter(|v| v.as_str() == "origin").count() == 1,
                "existing Vary Origin retained"
            );
            if method == Method::HEAD {
                assert!(body.is_empty());
            } else {
                assert!(body == b"fixture sibling");
            }
        } else {
            assert!(
                body == b"fixture sibling"
                    && headers[header::CACHE_CONTROL] == "max-age=17"
                    && headers[header::ETAG] == "fixture-validator"
                    && headers[header::VARY] == "Origin",
                "unrelated route and method transport bytes retained"
            );
        }
    }
}

async fn ops_fault_catalog(pool: &PgPool) -> Value {
    sqlx::query_scalar(r#"
        SELECT jsonb_build_object(
          'functions',(SELECT jsonb_agg((to_jsonb(p)-'proacl') || jsonb_build_object('acl',
            (SELECT jsonb_agg(to_jsonb(a) ORDER BY grantor,grantee,privilege_type,is_grantable)
             FROM aclexplode(COALESCE(p.proacl,acldefault('f',p.proowner))) a)) ORDER BY p.oid)
            FROM pg_proc p WHERE p.pronamespace='public'::regnamespace AND p.proname IN
            ('auth_legacy_platform_source_material_v1','platform_org_health','platform_console_route_adoption','platform_ops_test_audit_failure')),
          'triggers',(SELECT jsonb_agg(to_jsonb(t) ORDER BY t.oid) FROM pg_trigger t WHERE t.tgrelid='public.audit_events'::regclass))
    "#).fetch_one(pool).await.unwrap()
}
#[sqlx::test(migrations = false)]
async fn mounted_ops_each_infrastructure_stage_503_is_private_atomic_and_recovers(pool: PgPool) {
    let (f, expected) = populated_fixture(&pool).await;
    mounted_token(&pool, &f, &f.access, StatusCode::OK, &expected).await;
    let faults = [
        (
            "REVOKE EXECUTE ON FUNCTION public.auth_legacy_platform_source_material_v1(uuid,uuid) FROM console_rt",
            "GRANT EXECUTE ON FUNCTION public.auth_legacy_platform_source_material_v1(uuid,uuid) TO console_rt",
            Some("public.auth_legacy_platform_source_material_v1(uuid,uuid)"),
        ),
        (
            "REVOKE EXECUTE ON FUNCTION public.platform_org_health() FROM console_rt",
            "GRANT EXECUTE ON FUNCTION public.platform_org_health() TO console_rt",
            Some("public.platform_org_health()"),
        ),
        (
            "REVOKE EXECUTE ON FUNCTION public.platform_console_route_adoption() FROM console_rt",
            "GRANT EXECUTE ON FUNCTION public.platform_console_route_adoption() TO console_rt",
            Some("public.platform_console_route_adoption()"),
        ),
        (
            "CREATE FUNCTION public.platform_ops_test_audit_failure() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION USING ERRCODE='P0002', MESSAGE='auth_legacy.subject_not_found'; END $$; CREATE TRIGGER platform_ops_test_audit_failure BEFORE INSERT ON public.audit_events FOR EACH ROW EXECUTE FUNCTION public.platform_ops_test_audit_failure()",
            "DROP TRIGGER platform_ops_test_audit_failure ON public.audit_events; DROP FUNCTION public.platform_ops_test_audit_failure()",
            None,
        ),
    ];
    for (install, restore, function) in faults {
        let original = ops_fault_catalog(&pool).await;
        let rows = all_rows(&pool).await;
        if let Some(function) = function {
            let allowed: bool =
                sqlx::query_scalar("SELECT has_function_privilege('console_rt',$1,'EXECUTE')")
                    .bind(function)
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            assert!(
                allowed,
                "positive real Business function grant prerequisite"
            );
        }
        sqlx::raw_sql(sqlx::AssertSqlSafe(install))
            .execute(&pool)
            .await
            .unwrap();
        let installed = ops_fault_catalog(&pool).await;
        assert!(
            installed != original,
            "fault changes actual function/ACL/trigger catalog"
        );
        if let Some(function) = function {
            let allowed: bool =
                sqlx::query_scalar("SELECT has_function_privilege('console_rt',$1,'EXECUTE')")
                    .bind(function)
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            assert!(!allowed, "exact Business capability actually revoked");
        }
        let mut outcomes = Vec::new();
        for method in [Method::GET, Method::HEAD] {
            let response = finish(ops_task(&f.router, &f.access, method.clone())).await;
            let outcome = match response {
                Ok(response) => {
                    let (status, headers, body) = response_parts(response).await;
                    let payload = if method == Method::HEAD {
                        body.is_empty()
                    } else {
                        serde_json::from_slice::<Value>(&body).ok()
                            == Some(
                                json!({"error":{"code":"service_unavailable","message":"platform health is unavailable"}}),
                            )
                    };
                    status == StatusCode::SERVICE_UNAVAILABLE
                        && payload
                        && private_response(&headers, &body, &f.secrets)
                        && !body
                            .windows(b"auth_legacy.subject_not_found".len())
                            .any(|w| w == b"auth_legacy.subject_not_found")
                }
                Err(_) => false,
            };
            outcomes.push(
                outcome
                    && all_rows(&pool).await == rows
                    && ops_fault_catalog(&pool).await == installed,
            );
        }
        // Restore every test-owned injection before interpreting captured responses.
        sqlx::raw_sql(sqlx::AssertSqlSafe(restore))
            .execute(&pool)
            .await
            .unwrap();
        assert!(
            ops_fault_catalog(&pool).await == original,
            "exact catalog and grants restored"
        );
        assert!(
            all_rows(&pool).await == rows,
            "no durable effect escaped failing read"
        );
        assert!(
            outcomes.into_iter().all(|ok| ok),
            "GET/HEAD finite private503 at each exact owner stage"
        );
        mounted_token(&pool, &f, &f.access, StatusCode::OK, &expected).await;
    }
    close_ops(f).await;
}

async fn ops_outer_timeout(pool: PgPool, method: Method) {
    let (mut f, expected) = populated_fixture(&pool).await;
    let narrow = timeout_state(&pool, &f).await;
    f.router = build_router(narrow.clone());
    mounted_token(&pool, &f, &f.access, StatusCode::OK, &expected).await;
    let before = all_rows(&pool).await;
    let mut observer = pool.acquire().await.unwrap();
    let mut barrier = pool.begin().await.unwrap();
    let blocker: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *barrier)
        .await
        .unwrap();
    sqlx::query("LOCK TABLE public.groups IN ACCESS EXCLUSIVE MODE")
        .execute(&mut *barrier)
        .await
        .unwrap();
    let request = ops_task(&f.router, &f.access, method.clone());
    let reader = waiting_on(
        &pool,
        blocker,
        "platform_org_health",
        std::time::Duration::from_secs(2),
    )
    .await;
    let pending = !request.is_finished();
    let response = finish(request).await;
    let barrier_held:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_locks WHERE pid=pg_backend_pid() AND relation='public.groups'::regclass AND mode='AccessExclusiveLock' AND granted)")
        .fetch_one(&mut *barrier).await.unwrap();
    barrier.rollback().await.unwrap();
    let pids: Vec<_> = std::iter::once(blocker).chain(reader).collect();
    let clean = clean_pids(&mut observer, &pids).await;
    let after = all_rows(&pool).await;
    mounted_token(&pool, &f, &f.access, StatusCode::OK, &expected).await;
    let secrets = f.secrets.clone();
    narrow.shutdown_realtime().await;
    close_ops(f).await;
    let (status, headers, body) = response_parts(response.unwrap()).await;
    assert!(
        waited_cleanly(blocker, reader, pending, clean) && barrier_held,
        "actual health projection barrier outlives configured App timeout"
    );
    assert!(
        status == StatusCode::REQUEST_TIMEOUT && private_response(&headers, &body, &secrets),
        "actual outer408 remains private"
    );
    if method == Method::HEAD {
        assert!(body.is_empty(), "outer HEAD timeout has no body");
    } else {
        assert!(
            serde_json::from_slice::<Value>(&body).unwrap()
                == json!({"error":{"code":"request_timeout","message":"request timed out"}})
        );
    }
    assert!(
        before == after,
        "cancelled ops transaction leaves no audit or durable effect"
    );
}
#[sqlx::test(migrations = false)]
async fn mounted_ops_get_outer_timeout_is_private_atomic_and_recovers(pool: PgPool) {
    ops_outer_timeout(pool, Method::GET).await;
}
#[sqlx::test(migrations = false)]
async fn mounted_ops_head_outer_timeout_is_private_empty_atomic_and_recovers(pool: PgPool) {
    ops_outer_timeout(pool, Method::HEAD).await;
}
