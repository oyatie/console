use std::time::Duration;

use axum::body::Body;
use console_app::{AppConfig, AppRole, AppState, DatabaseDependency, build_router};
use http::{Request, StatusCode};
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;

#[tokio::test]
async fn healthz_reports_process_liveness_and_role() -> Result<(), Box<dyn std::error::Error>> {
    let config = app_config(AppRole::Api)?;
    let state = AppState::new(config, DatabaseDependency::NotConfigured)?;
    let response = build_router(state)
        .oneshot(Request::builder().uri("/healthz").body(Body::empty())?)
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    Ok(())
}

#[tokio::test]
async fn readyz_is_ready_without_configured_dependencies() -> Result<(), Box<dyn std::error::Error>>
{
    let config = app_config(AppRole::Worker)?;
    let state = AppState::new(config, DatabaseDependency::NotConfigured)?;
    let response = build_router(state)
        .oneshot(Request::builder().uri("/readyz").body(Body::empty())?)
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    Ok(())
}

#[tokio::test]
async fn readyz_returns_503_when_configured_database_is_unreachable()
-> Result<(), Box<dyn std::error::Error>> {
    let mut config = app_config(AppRole::Api)?;
    config.database_durability =
        Some(console_platform_db::durability::DurabilityPolicy::local_development());
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_millis(100))
        .connect_lazy("postgres://console_app:wrong@127.0.0.1:1/console_missing")?;
    let state = AppState::new(config, DatabaseDependency::Postgres(pool))?;
    let response = build_router(state)
        .oneshot(Request::builder().uri("/readyz").body(Body::empty())?)
        .await?;

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    Ok(())
}

#[tokio::test]
async fn metrics_endpoint_exposes_the_slo_http_duration_histogram()
-> Result<(), Box<dyn std::error::Error>> {
    // The global recorder is process-wide and shared across this test binary;
    // installation is idempotent and the unique service_name isolates this
    // test's series from any other test's measured requests.
    console_app::install_metrics_recorder()?;
    let config = AppConfig::from_pairs([
        ("CONSOLE_APP_ROLE", AppRole::Api.to_string()),
        ("CONSOLE_HTTP_ADDR", "127.0.0.1:0".to_owned()),
        ("CONSOLE_SERVICE_NAME", "console-app-api".to_owned()),
    ])?;
    let state = AppState::new(config, DatabaseDependency::NotConfigured)?;
    let app = build_router(state);

    // One measured request so the histogram has at least one observation.
    let health = app
        .clone()
        .oneshot(Request::builder().uri("/healthz").body(Body::empty())?)
        .await?;
    assert_eq!(health.status(), StatusCode::OK);

    // Policy Studio emits a feature counter from the identity router. Exercise
    // the same bounded label shape here so the scrape path proves both the
    // generic RED histogram and feature-specific operation counters are exposed.
    metrics::counter!(
        "policy_studio_operation_total",
        "operation" => "preview_assignments",
        "outcome" => "success",
    )
    .increment(1);

    let metrics = app
        .oneshot(Request::builder().uri("/metrics").body(Body::empty())?)
        .await?;
    assert_eq!(metrics.status(), StatusCode::OK);
    let body = axum::body::to_bytes(metrics.into_body(), usize::MAX).await?;
    let text = String::from_utf8(body.to_vec())?;
    assert!(
        text.contains("http_server_request_duration_seconds_bucket"),
        "exposition must include the SLO latency histogram buckets; got:\n{text}"
    );
    assert!(
        text.contains("service_name=\"console-app-api\""),
        "histogram series must carry the service_name label the SLO filters on; got:\n{text}"
    );
    assert!(
        text.contains("policy_studio_operation_total")
            && text.contains("operation=\"preview_assignments\"")
            && text.contains("outcome=\"success\""),
        "policy studio counter must expose only bounded operation/outcome labels; got:\n{text}"
    );
    Ok(())
}

#[tokio::test]
async fn ui_shell_serves_empty_ssr_html() -> Result<(), Box<dyn std::error::Error>> {
    let config = app_config(AppRole::Api)?;
    let state = AppState::new(config, DatabaseDependency::NotConfigured)?;
    let app = build_router(state);
    // Public root is a concrete native document even without configured Auth.
    // Every protected route is checked separately; no early-success escape.
    for cookie in [None, Some("theme=root-no-database-marker")] {
        for method in ["GET", "HEAD"] {
            let mut request = Request::builder().method(method).uri("/");
            if let Some(cookie) = cookie {
                request = request.header("Cookie", cookie);
            }
            let response = app.clone().oneshot(request.body(Body::empty())?).await?;
            assert_eq!(response.status(), StatusCode::OK);
            assert_eq!(
                response
                    .headers()
                    .get("cache-control")
                    .and_then(|v| v.to_str().ok()),
                Some("no-store")
            );
            assert_eq!(
                response
                    .headers()
                    .get("pragma")
                    .and_then(|v| v.to_str().ok()),
                Some("no-cache")
            );
            assert!(!response.headers().contains_key("set-cookie"));
            let body = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
            if method == "HEAD" {
                assert!(body.is_empty());
            } else {
                assert!(
                    String::from_utf8(body.to_vec())?
                        == console_payroll_ui::native_account::render(
                            console_payroll_ui::native_account::Page::Public
                        )
                );
            }
        }
    }
    // Native credentials and parser errors retain configuration-before-parse
    // precedence. Neither mixed Bearer nor oversize may manufacture Public.
    let oversized = format!("theme={}", "z".repeat(16 * 1024 + 1));
    for headers in [
        vec![(
            "Cookie",
            "__Host-console_account_session=root-unavailable-marker",
        )],
        vec![
            (
                "Cookie",
                "__Host-console_account_session=root-unavailable-marker",
            ),
            ("Authorization", "Bearer root-unavailable-bearer"),
        ],
        vec![
            ("Cookie", oversized.as_str()),
            ("Authorization", "Bearer root-unavailable-bearer"),
        ],
    ] {
        for path in ["/", "/account", "/account/register"] {
            for (method, expected) in [
                ("GET", StatusCode::SERVICE_UNAVAILABLE),
                ("HEAD", StatusCode::METHOD_NOT_ALLOWED),
            ] {
                let mut builder = Request::builder().method(method).uri(path);
                for (name, value) in &headers {
                    builder = builder.header(*name, *value);
                }
                let response = app.clone().oneshot(builder.body(Body::empty())?).await?;
                assert_eq!(response.status(), expected);
                assert!(!response.headers().contains_key("set-cookie"));
                let body = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
                if method == "HEAD" {
                    assert!(body.is_empty());
                } else {
                    assert!(
                        String::from_utf8(body.to_vec())?
                            == console_payroll_ui::native_account::render(
                                console_payroll_ui::native_account::Page::Unavailable
                            )
                    );
                    // Literal assertions must independently detect a wrong retry
                    // destination even when the shared renderer matches itself.
                    let html = std::str::from_utf8(&body)?;
                    assert!(html.contains("<a class=\"button primary\" href=\"\">다시 시도</a>"));
                    assert!(html.contains("<a class=\"text-link\" href=\"/\">시작 화면으로</a>"));
                }
            }
        }
    }
    for uri in ["/work", "/organization", "/hr", "/payroll"] {
        let response = app
            .clone()
            .oneshot(Request::builder().uri(uri).body(Body::empty())?)
            .await?;
        assert_eq!(response.status(), StatusCode::OK, "{uri}");
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
        let text = String::from_utf8(body.to_vec())?;
        assert_eq!(text, console_payroll_ui::render_shell());
        assert!(
            !text.contains("291_520") && !text.to_ascii_lowercase().contains("payslip"),
            "{uri} leaked payroll: {text}"
        );
    }
    Ok(())
}

#[tokio::test]
async fn ui_pkg_serves_committed_hydrate_assets() -> Result<(), Box<dyn std::error::Error>> {
    let config = app_config(AppRole::Api)?;
    let state = AppState::new(config, DatabaseDependency::NotConfigured)?;
    let app = build_router(state);
    let cases: [(&str, &[u8], &[u8]); 2] = [
        (
            "/pkg/console_payroll_ui.js",
            b"text/javascript; charset=utf-8",
            console_payroll_ui::payroll_ui_js(),
        ),
        (
            "/pkg/console_payroll_ui_bg.wasm",
            b"application/wasm",
            console_payroll_ui::payroll_ui_wasm(),
        ),
    ];
    for (uri, mime, expected) in cases {
        let response = app
            .clone()
            .oneshot(Request::builder().uri(uri).body(Body::empty())?)
            .await?;
        assert_eq!(response.status(), StatusCode::OK, "{uri}");
        assert_eq!(
            response
                .headers()
                .get(http::header::CONTENT_TYPE)
                .map(http::HeaderValue::as_bytes),
            Some(mime),
            "{uri}"
        );
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
        assert_eq!(body.as_ref(), expected, "{uri}");
    }
    Ok(())
}

/// ADR-0042 option 1: HTML documents authenticate from a session cookie, not
/// from an access token in `localStorage`. The committed shell and hydrate
/// script must not grow a client token store while that option is proposed.
#[test]
fn ssr_shell_does_not_store_access_token_in_local_storage() {
    let shell = console_payroll_ui::render_shell();
    let js = String::from_utf8_lossy(console_payroll_ui::payroll_ui_js());
    for (label, text) in [("shell", shell.as_str()), ("hydrate js", js.as_ref())] {
        let lowered = text.to_ascii_lowercase();
        assert!(
            !lowered.contains("localstorage"),
            "{label} must not persist an access token in localStorage: {text}"
        );
    }
}

/// Source tripwire for ADR-0042 option 1. The recommended cookie is not minted
/// or consumed yet. `request-context` is the ADR-named extractor; scan its
/// production source (not `mod tests`) so a cookie fallback in
/// `resolve_principal` / `with_request_context` cannot hide behind unit tests
/// of `bearer_token()`.
#[test]
fn proposed_ssr_session_cookie_is_not_minted_or_consumed_yet() {
    let auth_rest = include_str!("../../crates/platform/auth-rest/src/lib.rs");
    let app = include_str!("../src/lib.rs");
    let request_context = production_rs(include_str!(
        "../../crates/platform/request-context/src/lib.rs"
    ));
    for (label, src) in [
        ("auth-rest", auth_rest),
        ("console-app", app),
        ("request-context", request_context),
    ] {
        assert!(
            !src.contains("console_session"),
            "ADR-0042 proposed session cookie appeared in {label}. Update \
             `cookie_does_not_authorize_json_api` if the live `/api/v1` deny \
             still holds, replace \
             `proposed_ssr_session_cookie_does_not_authorize_html_get_yet` \
             if HTML GET `/` now authenticates from the cookie, and un-ignore \
             `adr0042_ssr_session_cookie_contract` only for the remaining \
             proposed mint/HTML/replica assertions."
        );
    }
    assert!(
        !request_context.contains("header::COOKIE") && !request_context.contains("COOKIE"),
        "request-context production source grew a Cookie header read. Keep \
         `cookie_does_not_authorize_json_api` live; do not treat Cookie as a \
         Bearer for `/api/v1`."
    );
}

fn production_rs(src: &str) -> &str {
    src.split_once("#[cfg(test)]").map_or(src, |(prod, _)| prod)
}

fn app_config(role: AppRole) -> Result<AppConfig, console_app::AppError> {
    AppConfig::from_pairs([
        ("CONSOLE_APP_ROLE", role.to_string()),
        ("CONSOLE_HTTP_ADDR", "127.0.0.1:0".to_owned()),
    ])
}

mod authorized {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;
    use axum::body::to_bytes;
    use console_kernel_core::{OrgId, UserId};
    use console_ontology_canonical_adapter_postgres::company::{
        CompanyCommand, CompanyQuery, PgCompanyPort,
    };
    use console_ontology_canonical_adapter_postgres::employment::{
        EmploymentAttributes, EmploymentCommand, EmploymentQuery, PgEmploymentPort,
    };
    use console_ontology_canonical_adapter_postgres::org_unit::{
        OrgUnitCommand, OrgUnitQuery, PgOrgUnitPort,
    };
    use console_ontology_canonical_adapter_postgres::person::{
        PersonCommand, PersonQuery, PgPersonPort,
    };
    use console_ontology_canonical_domain::{CanonicalPort, CommandId, DispatchTarget};
    use console_payroll_adapter_postgres::pay_run::{PayRunCommand, PayRunQuery, PgPayRunPort};
    use console_platform_auth::{AccessTokenInput, JwtIssuer, JwtSettings};
    use http::header;
    use p256::ecdsa::SigningKey;
    use p256::elliptic_curve::rand_core::OsRng;
    use p256::pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding};
    use serde_json::json;
    use sqlx::PgPool;
    use time::OffsetDateTime;
    use time::macros::date;
    use uuid::Uuid;

    const TEST_ISSUER: &str = "console-platform-auth";
    const TEST_AUDIENCE: &str = "console-api";

    struct Keys {
        private_pem: String,
        public_pem: String,
    }

    fn keys() -> Keys {
        let signing_key = SigningKey::random(&mut OsRng);
        Keys {
            private_pem: signing_key
                .to_pkcs8_pem(LineEnding::LF)
                .unwrap()
                .to_string(),
            public_pem: signing_key
                .verifying_key()
                .to_public_key_pem(LineEnding::LF)
                .unwrap(),
        }
    }

    fn bearer(keys: &Keys, org: OrgId, user: UserId, role: &str) -> String {
        let issuer = JwtIssuer::from_es256_pem(
            JwtSettings {
                issuer: TEST_ISSUER.to_owned(),
                audience: TEST_AUDIENCE.to_owned(),
                access_token_ttl: time::Duration::minutes(15),
            },
            keys.private_pem.as_bytes(),
            keys.public_pem.as_bytes(),
        )
        .unwrap();
        issuer
            .issue_access_token(AccessTokenInput {
                subject: user,
                org_id: org,
                roles: vec![role.to_owned()],
                branches: Vec::new(),
                platform: false,
                view_as: false,
                read_only: false,
                display_name: None,
                feature_grants: Vec::new(),
                authz_subject_version: 0,
                authz_policy_version: 0,
                session_generation: 0,
                issued_at: OffsetDateTime::now_utc(),
            })
            .unwrap()
    }

    async fn runtime_role_pool(owner: &PgPool) -> PgPool {
        PgPoolOptions::new()
            .max_connections(4)
            .after_connect(|conn, _| {
                Box::pin(async move {
                    sqlx::query("SET ROLE console_rt").execute(conn).await?;
                    Ok(())
                })
            })
            .connect_with(owner.connect_options().as_ref().clone())
            .await
            .unwrap()
    }

    async fn jwt_app_state(owner_pool: &PgPool, public_key_pem: String) -> AppState {
        // The session verifier reads current legacy context and Account fencing
        // through its actual restricted Auth login, independent of token issuance.
        let auth_database = console_platform_test_support::login_test_pool(
            owner_pool,
            console_platform_test_support::TestDatabaseLogin::Auth,
        )
        .await;
        let runtime_pool = console_platform_test_support::login_test_pool(
            owner_pool,
            console_platform_test_support::TestDatabaseLogin::Business,
        )
        .await;
        let config = AppConfig::from_pairs([
            (
                "CONSOLE_DATABASE_DURABILITY",
                r#"{"mode":"local_development"}"#.to_owned(),
            ),
            ("CONSOLE_APP_ROLE", AppRole::Api.to_string()),
            ("CONSOLE_HTTP_ADDR", "127.0.0.1:0".to_owned()),
            ("CONSOLE_JWT_ISSUER", TEST_ISSUER.to_owned()),
            ("CONSOLE_JWT_AUDIENCE", TEST_AUDIENCE.to_owned()),
            ("CONSOLE_JWT_PUBLIC_KEY_PEM", public_key_pem),
        ])
        .unwrap();
        AppState::new(config, DatabaseDependency::Postgres(runtime_pool))
            .unwrap()
            .with_auth_database(auth_database)
    }

    async fn seed_user(pool: &PgPool, org: OrgId, user: UserId, role: &str) {
        sqlx::query("INSERT INTO users (id, display_name, roles, org_id) VALUES ($1, $2, $3, $4)")
            .bind(*user.as_uuid())
            .bind(format!("ui-{role}"))
            .bind(vec![role.to_owned()])
            .bind(*org.as_uuid())
            .execute(pool)
            .await
            .unwrap();
    }

    async fn seed_run(pool: &PgPool, org: OrgId, actor: UserId) -> Uuid {
        let pay_run = PgPayRunPort::new(
            runtime_role_pool(pool).await,
            tokio::runtime::Handle::current(),
        );
        let created = {
            let port = pay_run.clone();
            let command = PayRunCommand {
                org_id: org,
                command_id: CommandId::from_uuid(Uuid::new_v4()),
                actor_id: actor,
                query: PayRunQuery::CreateRun {
                    run_id: Uuid::new_v4(),
                    period_start: date!(2026 - 06 - 01),
                    period_end: date!(2026 - 06 - 30),
                    connector: Some("m2".to_owned()),
                    job: Some("payroll_draft".to_owned()),
                },
                action_key: "create_run".to_owned(),
                object_type_id: Uuid::nil(),
            };
            tokio::task::spawn_blocking(move || port.execute(&command))
                .await
                .unwrap()
                .expect("payroll.create_run as console_rt")
        };
        assert_eq!(created.target(), DispatchTarget::PayrollCreateRun);
        created.result()["draft_run_id"]
            .as_str()
            .expect("CreateRun must name draft_run_id")
            .parse()
            .unwrap()
    }

    struct SeededHeads {
        person_id: Uuid,
        org_unit_id: Uuid,
        employment_id: Uuid,
    }

    async fn seed_heads(pool: &PgPool, org: OrgId, actor: UserId) -> SeededHeads {
        let runtime = runtime_role_pool(pool).await;
        let handle = tokio::runtime::Handle::current();
        let company = PgCompanyPort::new(runtime.clone(), handle.clone());
        let company_cmd = CompanyCommand {
            org_id: org,
            command_id: CommandId::from_uuid(Uuid::new_v4()),
            actor_id: actor,
            query: CompanyQuery {
                attributes: json!({ "legal_name": "KNL" }),
            },
            action_key: "revise".to_owned(),
            object_type_id: Uuid::nil(),
        };
        tokio::task::spawn_blocking(move || company.execute(&company_cmd))
            .await
            .unwrap()
            .expect("company.revise as console_rt");

        let units = PgOrgUnitPort::new(runtime.clone(), handle.clone());
        let unit_cmd = OrgUnitCommand {
            org_id: org,
            command_id: CommandId::from_uuid(Uuid::new_v4()),
            actor_id: actor,
            query: OrgUnitQuery::Create {
                source: None,
                attributes: json!({ "name": "본사", "kind": "site" }),
            },
            action_key: "create_org_unit".to_owned(),
            object_type_id: Uuid::nil(),
        };
        let created_unit = tokio::task::spawn_blocking(move || units.execute(&unit_cmd))
            .await
            .unwrap()
            .expect("organization.create_org_unit as console_rt");
        let org_unit_id = created_unit.result()["org_unit_id"]
            .as_str()
            .expect("create_org_unit must name org_unit_id")
            .parse()
            .unwrap();

        let employee_id: Uuid = sqlx::query_scalar(
            "INSERT INTO employees \
             (org_id, company, name, source_filename, source_sheet, source_row, source_key) \
             VALUES ($1, 'KNL', '홍길동', 'seed.xlsx', 'Sheet1', 1, $2) RETURNING id",
        )
        .bind(*org.as_uuid())
        .bind(format!("ui-employment-{}", actor.as_uuid()))
        .fetch_one(pool)
        .await
        .unwrap();

        let persons = PgPersonPort::new(runtime.clone(), handle.clone());
        let person_cmd = PersonCommand {
            org_id: org,
            command_id: CommandId::from_uuid(Uuid::new_v4()),
            actor_id: actor,
            query: PersonQuery::Create {
                employee_id: Some(employee_id),
                attributes: json!({ "legal_name": "홍길동", "display_name": "홍길동" }),
            },
            action_key: "create_person".to_owned(),
            object_type_id: Uuid::nil(),
        };
        let created_person = tokio::task::spawn_blocking(move || persons.execute(&person_cmd))
            .await
            .unwrap()
            .expect("people.create_person as console_rt");
        let person_id = created_person.result()["person_id"]
            .as_str()
            .expect("create_person must name person_id")
            .parse()
            .unwrap();

        let employments = PgEmploymentPort::new(runtime, handle);
        let appoint_cmd = EmploymentCommand {
            org_id: org,
            command_id: CommandId::from_uuid(Uuid::new_v4()),
            actor_id: actor,
            query: EmploymentQuery::Appoint {
                employee_id,
                valid_from: OffsetDateTime::from_unix_timestamp(1_800_000_000).unwrap(),
                attributes: EmploymentAttributes {
                    company: "KNL".to_owned(),
                    org_unit_id: Some(org_unit_id),
                    job_position_id: None,
                    employment_status: "ACTIVE".to_owned(),
                },
            },
            action_key: "revise".to_owned(),
            object_type_id: Uuid::nil(),
        };
        let created_employment =
            tokio::task::spawn_blocking(move || employments.execute(&appoint_cmd))
                .await
                .unwrap()
                .expect("hr.appoint as console_rt");
        let employment_id = created_employment.result()["employment_id"]
            .as_str()
            .expect("hr.appoint must name employment_id")
            .parse()
            .unwrap();
        SeededHeads {
            person_id,
            org_unit_id,
            employment_id,
        }
    }

    /// Legacy console_refresh never authorizes a shipping document. The public
    /// landing now offers native Account entry, while Bearer remains the
    /// positive control for the same Company's authorized run and island.
    /// Supplying an access token under the legacy cookie name is deliberately
    /// generous: the real refresh cookie is path-scoped away from root. Native
    /// __Host-console_account_* credentials have a separate accepted owner.
    #[sqlx::test(migrations = false)]
    async fn browser_navigation_reaches_no_authorized_screen_adr_0042(pool: PgPool) {
        console_platform_test_support::prepare_account_test_database(&pool).await;
        let keys = keys();
        let org = OrgId::knl();
        let admin = UserId::new();
        seed_user(&pool, org, admin, "SUPER_ADMIN").await;
        let run = seed_run(&pool, org, admin).await;
        let service = build_router(jwt_app_state(&pool, keys.public_pem.clone()).await);
        let token = bearer(&keys, org, admin, "SUPER_ADMIN");

        // One principal, over the transport an HTTP client can use.
        let (status, with_header) = get_ui(service.clone(), Some(&token)).await;
        assert_eq!(status, StatusCode::OK, "{with_header}");
        assert!(
            with_header.contains(&format!("data-run-id=\"{run}\"")),
            "header transport must reach the authorized run: {with_header}"
        );
        assert!(
            with_header.contains("leptos-island"),
            "header transport must reach the hydrated island: {with_header}"
        );

        // Unrelated cookies are not alternate authority. The legacy Bearer
        // response, including its CSP/hydration envelope, stays byte-identical.
        let baseline = service
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(baseline.status(), StatusCode::OK);
        let baseline_headers = baseline.headers().clone();
        let baseline_body = to_bytes(baseline.into_body(), usize::MAX).await.unwrap();
        assert!(baseline_body.as_ref() == with_header.as_bytes());
        for cookie in [
            "theme=root-legacy-theme-marker",
            "console_refresh=root-non-native-marker",
            "console_session=root-proposed-marker",
        ] {
            for method in ["GET", "HEAD"] {
                let response = service
                    .clone()
                    .oneshot(
                        Request::builder()
                            .method(method)
                            .uri("/")
                            .header(header::AUTHORIZATION, format!("Bearer {token}"))
                            .header(header::COOKIE, cookie)
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .unwrap();
                assert_eq!(response.status(), StatusCode::OK);
                for name in [
                    header::CONTENT_TYPE,
                    header::CONTENT_SECURITY_POLICY,
                    header::CACHE_CONTROL,
                    header::PRAGMA,
                    header::VARY,
                ] {
                    assert!(
                        response
                            .headers()
                            .get_all(&name)
                            .iter()
                            .eq(baseline_headers.get_all(&name).iter()),
                        "legacy response envelope changed"
                    );
                }
                assert!(!response.headers().contains_key(header::SET_COOKIE));
                let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
                if method == "HEAD" {
                    assert!(body.is_empty());
                } else {
                    assert!(body == baseline_body);
                }
            }
        }

        // The same principal, over what a browser navigation can carry.
        // Generous on purpose: this hands the browser a real access token in
        // the only cookie the system sets at all -- a cookie that is
        // path-scoped away from `/` and never holds an access token. Even so
        // no authorized screen is disclosed; the public landing offers Account entry.
        let navigation = Request::builder()
            .uri("/")
            .header(
                header::USER_AGENT,
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36",
            )
            .header(header::ACCEPT, "text/html,application/xhtml+xml")
            .header(header::COOKIE, format!("console_refresh={token}"))
            .header("upgrade-insecure-requests", "1")
            .body(Body::empty())
            .unwrap();
        let response = service.oneshot(navigation).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let navigated = String::from_utf8(bytes.to_vec()).unwrap();

        assert_eq!(
            navigated,
            console_payroll_ui::native_account::render(
                console_payroll_ui::native_account::Page::Public
            ),
            "Legacy refresh cookies must render only the public entry, never authorize shipping work."
        );
        assert!(
            !navigated.contains(&run.to_string()) && !navigated.contains("leptos-island"),
            "the public entry must leak neither the run nor the island: {navigated}"
        );
    }

    /// The proposed console_session name remains non-authorizing, even with a
    /// valid access JWT. Root may show Public without granting any shipping
    /// data; native Account cookie entry does not activate this old proposal.
    /// Keep the separate live JSON API cookie-denial and Bearer controls.
    #[sqlx::test(migrations = false)]
    async fn proposed_ssr_session_cookie_does_not_authorize_html_get_yet(pool: PgPool) {
        console_platform_test_support::prepare_account_test_database(&pool).await;
        let keys = keys();
        let org = OrgId::knl();
        let admin = UserId::new();
        seed_user(&pool, org, admin, "SUPER_ADMIN").await;
        let run = seed_run(&pool, org, admin).await;
        let service = build_router(jwt_app_state(&pool, keys.public_pem.clone()).await);
        let token = bearer(&keys, org, admin, "SUPER_ADMIN");

        let (status, with_header) = get_ui(service.clone(), Some(&token)).await;
        assert_eq!(status, StatusCode::OK, "{with_header}");
        assert!(
            with_header.contains(&format!("data-run-id=\"{run}\"")),
            "header transport must still reach the authorized run: {with_header}"
        );

        let (status, navigated) = adr0042_ssr_session_cookie::get_html_with_cookie(
            service,
            "/",
            &format!(
                "{}={token}",
                adr0042_ssr_session_cookie::PROPOSED_SSR_SESSION_COOKIE
            ),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{navigated}");
        assert_eq!(
            navigated,
            console_payroll_ui::native_account::render(
                console_payroll_ui::native_account::Page::Public
            ),
            "Proposed console_session cookies must remain non-authorizing public entry; native Account cookies have a separate owner."
        );
        assert!(
            !navigated.contains(&run.to_string()) && !navigated.contains("leptos-island"),
            "the public entry must leak neither the run nor the island: {navigated}"
        );
    }

    async fn get_ui(app: axum::Router, token: Option<&str>) -> (StatusCode, String) {
        get_ui_path(app, "/", token).await
    }

    async fn get_ui_path(
        app: axum::Router,
        uri: &str,
        token: Option<&str>,
    ) -> (StatusCode, String) {
        let mut builder = Request::builder().uri(uri);
        if let Some(token) = token {
            builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
        }
        let response = app
            .oneshot(builder.body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    #[sqlx::test(migrations = false)]
    async fn ui_shell_omits_runs_unless_payroll_run_read(pool: PgPool) {
        console_platform_test_support::prepare_account_test_database(&pool).await;
        let keys = keys();
        let org = OrgId::knl();
        let super_admin = UserId::new();
        seed_user(&pool, org, super_admin, "SUPER_ADMIN").await;
        let member = UserId::new();
        seed_user(&pool, org, member, "MEMBER").await;
        let run = seed_run(&pool, org, super_admin).await;

        let service = build_router(jwt_app_state(&pool, keys.public_pem.clone()).await);

        let (status, unauth) = get_ui(service.clone(), None).await;
        assert_eq!(status, StatusCode::OK, "{unauth}");
        assert_eq!(
            unauth,
            console_payroll_ui::native_account::render(
                console_payroll_ui::native_account::Page::Public
            )
        );
        assert!(
            !unauth.contains("/pkg/"),
            "public entry must not load WASM: {unauth}"
        );

        let (status, member_html) =
            get_ui(service.clone(), Some(&bearer(&keys, org, member, "MEMBER"))).await;
        assert_eq!(status, StatusCode::OK, "{member_html}");
        assert_eq!(member_html, console_payroll_ui::render_shell());
        assert!(
            !member_html.contains(&run.to_string()),
            "MEMBER must not see the run id: {member_html}"
        );
        assert!(
            !member_html.contains("/pkg/"),
            "MEMBER shell must not load WASM: {member_html}"
        );

        let (status, admin_html) = get_ui(
            service,
            Some(&bearer(&keys, org, super_admin, "SUPER_ADMIN")),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{admin_html}");
        assert_ne!(admin_html, console_payroll_ui::render_shell());
        assert!(
            admin_html.contains(&format!("data-run-id=\"{run}\"")),
            "SUPER_ADMIN must see the authorized run: {admin_html}"
        );
        assert!(
            admin_html.contains("/pkg/console_payroll_ui.js"),
            "authorized shell must preload bindgen js: {admin_html}"
        );
        let lowered = admin_html.to_ascii_lowercase();
        assert!(!lowered.contains("won"), "won leaked: {admin_html}");
        assert!(
            !admin_html.contains("291_520"),
            "golden won leaked: {admin_html}"
        );
        assert!(!lowered.contains("payslip"), "payslip leaked: {admin_html}");
    }

    async fn grant_group_viewer(pool: &PgPool, org: OrgId, user: UserId) {
        sqlx::query(
            "INSERT INTO group_role_grants (group_id, user_id, group_role) \
             SELECT group_id, $1, 'GROUP_VIEWER' FROM organizations WHERE id = $2",
        )
        .bind(*user.as_uuid())
        .bind(*org.as_uuid())
        .execute(pool)
        .await
        .unwrap();
    }

    #[sqlx::test(migrations = false)]
    async fn ui_shipping_screens_deny_by_omission(pool: PgPool) {
        console_platform_test_support::prepare_account_test_database(&pool).await;
        let keys = keys();
        let org = OrgId::knl();
        let super_admin = UserId::new();
        seed_user(&pool, org, super_admin, "SUPER_ADMIN").await;
        grant_group_viewer(&pool, org, super_admin).await;
        let member = UserId::new();
        seed_user(&pool, org, member, "MEMBER").await;
        let run = seed_run(&pool, org, super_admin).await;
        let heads = seed_heads(&pool, org, super_admin).await;

        let service = build_router(jwt_app_state(&pool, keys.public_pem.clone()).await);
        let admin = bearer(&keys, org, super_admin, "SUPER_ADMIN");
        let member_tok = bearer(&keys, org, member, "MEMBER");

        for uri in ["/", "/organization", "/hr", "/payroll"] {
            let (status, html) = get_ui_path(service.clone(), uri, Some(&member_tok)).await;
            assert_eq!(status, StatusCode::OK, "{uri} {html}");
            assert_eq!(html, console_payroll_ui::render_shell(), "{uri}");
            assert!(
                !html.contains("data-screen="),
                "MEMBER must omit shipping screens at {uri}: {html}"
            );
        }

        let (status, org_html) = get_ui_path(service.clone(), "/organization", Some(&admin)).await;
        assert_eq!(status, StatusCode::OK, "{org_html}");
        assert!(
            org_html.contains("data-screen=\"organization\"")
                && org_html.contains(&format!("data-org-id=\"{}\"", org.as_uuid()))
                && org_html.contains("data-legal-name=")
                && org_html.contains(&format!("data-org-unit-id=\"{}\"", heads.org_unit_id))
                && org_html.contains(&format!("href=\"/api/v1/companies/{}\"", org.as_uuid()))
                && org_html.contains(&format!("href=\"/api/v1/org-units/{}\"", heads.org_unit_id))
                && !org_html.contains("data-slug"),
            "SUPER_ADMIN organization screen must render published Company/OrgUnit Heads: {org_html}"
        );
        assert!(
            !org_html.contains("/pkg/"),
            "organization SSR must not load WASM: {org_html}"
        );

        let (status, hr_html) = get_ui_path(service.clone(), "/hr", Some(&admin)).await;
        assert_eq!(status, StatusCode::OK, "{hr_html}");
        assert!(
            hr_html.contains("data-screen=\"hr\"")
                && hr_html.contains(&format!("data-person-id=\"{}\"", heads.person_id))
                && hr_html.contains("data-legal-name=")
                && hr_html.contains(&format!("href=\"/api/v1/persons/{}\"", heads.person_id))
                && hr_html.contains(&format!("data-employment-id=\"{}\"", heads.employment_id))
                && hr_html.contains(&format!(
                    "href=\"/api/v1/employments/{}\"",
                    heads.employment_id
                ))
                && hr_html.contains("data-appointed-on=")
                && hr_html.contains("data-job-position-id=")
                && !hr_html.contains("/api/v1/job-positions/")
                && !hr_html.contains("data-employee-"),
            "SUPER_ADMIN HR screen must render published Person and Employment Heads: {hr_html}"
        );
        assert!(
            !hr_html.to_ascii_lowercase().contains("phone"),
            "HR screen leaked phone: {hr_html}"
        );
        assert!(
            !hr_html.contains("/pkg/"),
            "HR SSR must not load WASM: {hr_html}"
        );

        let (status, payroll_html) = get_ui_path(service.clone(), "/payroll", Some(&admin)).await;
        assert_eq!(status, StatusCode::OK, "{payroll_html}");
        assert!(
            payroll_html.contains("data-screen=\"payroll\"")
                && payroll_html.contains(&format!("data-run-id=\"{run}\"")),
            "SUPER_ADMIN payroll screen must render authorized runs: {payroll_html}"
        );
        assert!(
            payroll_html.contains("/pkg/console_payroll_ui.js"),
            "payroll island must hydrate via committed WASM: {payroll_html}"
        );
        let lowered = payroll_html.to_ascii_lowercase();
        assert!(!lowered.contains("won"), "won leaked: {payroll_html}");
        assert!(!lowered.contains("group-switcher"), "{payroll_html}");
        assert!(!lowered.contains("comms-rail"), "{payroll_html}");
    }

    async fn list_read_audits(pool: &PgPool, actor: UserId) -> i64 {
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM audit_events \
             WHERE action = 'payroll_run.list_read' AND actor = $1",
        )
        .bind(*actor.as_uuid())
        .fetch_one(pool)
        .await
        .unwrap()
    }

    fn assert_ui_invariants(html: &str) {
        let lowered = html.to_ascii_lowercase();
        assert!(!lowered.contains("won"), "won leaked: {html}");
        assert!(!html.contains("291_520"), "golden won leaked: {html}");
        assert!(!lowered.contains("payslip"), "payslip leaked: {html}");
        assert!(!lowered.contains("phone"), "directory phone leaked: {html}");
        assert!(!lowered.contains("group-switcher"), "{html}");
        assert!(!lowered.contains("comms-rail"), "{html}");
        assert!(
            !html.contains("type=\"file\"") && !html.contains("자료실"),
            "import/export is not the data-entry base: {html}"
        );
        assert!(
            !html.contains("webpack") && !html.contains("vite") && !html.contains("innerHTML"),
            "must stay Rust-native Leptos SSR: {html}"
        );
        assert!(
            !html.contains("/api/v1/job-positions/"),
            "must not invent JobPosition routes: {html}"
        );
        assert!(
            !html.contains("type=\"date\"")
                && !html.contains("name=\"as_of\"")
                && !html.contains("name=\"from\"")
                && !html.contains("name=\"to\""),
            "current-slice directory must not invent a temporal picker: {html}"
        );
    }

    /// ADR-0025 §4 persona real-backend E2E on `/`, `/organization`, `/hr`, `/payroll`.
    #[sqlx::test(migrations = false)]
    async fn ui_persona_e2e(pool: PgPool) {
        console_platform_test_support::prepare_account_test_database(&pool).await;
        let keys = keys();
        let org = OrgId::knl();
        let member = UserId::new();
        let admin = UserId::new();
        let executive = UserId::new();
        let super_admin = UserId::new();
        seed_user(&pool, org, member, "MEMBER").await;
        seed_user(&pool, org, admin, "ADMIN").await;
        seed_user(&pool, org, executive, "EXECUTIVE").await;
        seed_user(&pool, org, super_admin, "SUPER_ADMIN").await;
        grant_group_viewer(&pool, org, admin).await;
        grant_group_viewer(&pool, org, executive).await;
        grant_group_viewer(&pool, org, super_admin).await;
        let run = seed_run(&pool, org, super_admin).await;
        let heads = seed_heads(&pool, org, super_admin).await;

        let other_org = OrgId::from_uuid(Uuid::from_u128(0xb2));
        sqlx::query("INSERT INTO organizations (id, slug, name) VALUES ($1, $2, $3)")
            .bind(*other_org.as_uuid())
            .bind("persona-e2e-other")
            .bind("Persona E2E other org")
            .execute(&pool)
            .await
            .unwrap();
        let foreign = UserId::new();
        seed_user(&pool, other_org, foreign, "SUPER_ADMIN").await;
        grant_group_viewer(&pool, other_org, foreign).await;

        let service = build_router(jwt_app_state(&pool, keys.public_pem.clone()).await);
        let member_tok = bearer(&keys, org, member, "MEMBER");
        let admin_tok = bearer(&keys, org, admin, "ADMIN");
        let exec_tok = bearer(&keys, org, executive, "EXECUTIVE");
        let super_tok = bearer(&keys, org, super_admin, "SUPER_ADMIN");
        let foreign_tok = bearer(&keys, other_org, foreign, "SUPER_ADMIN");
        let routes = ["/", "/organization", "/hr", "/payroll"];
        let org_id = org.as_uuid().to_string();
        let run_id = run.to_string();

        for uri in routes {
            let (status, html) = get_ui_path(service.clone(), uri, None).await;
            assert_eq!(status, StatusCode::OK, "unauth {uri} {html}");
            let expected = if uri == "/" {
                console_payroll_ui::native_account::render(
                    console_payroll_ui::native_account::Page::Public,
                )
            } else {
                console_payroll_ui::render_shell()
            };
            assert_eq!(html, expected, "{uri}");
            assert!(!html.contains(&run_id), "unauth saw run at {uri}: {html}");
            assert!(
                !html.contains("/pkg/"),
                "unauth loads WASM at {uri}: {html}"
            );
            assert_ui_invariants(&html);

            let (status, html) = get_ui_path(service.clone(), uri, Some(&member_tok)).await;
            assert_eq!(status, StatusCode::OK, "MEMBER {uri} {html}");
            assert_eq!(html, console_payroll_ui::render_shell(), "{uri}");
            assert!(!html.contains(&run_id), "MEMBER saw run at {uri}: {html}");
            assert_ui_invariants(&html);
        }

        let (status, admin_org) =
            get_ui_path(service.clone(), "/organization", Some(&admin_tok)).await;
        assert_eq!(status, StatusCode::OK, "{admin_org}");
        assert_eq!(
            admin_org,
            console_payroll_ui::render_shell(),
            "ADMIN without org-wide EmployeeDirectoryRead must omit Company/OrgUnit Heads: {admin_org}"
        );

        let (status, admin_hr) = get_ui_path(service.clone(), "/hr", Some(&admin_tok)).await;
        assert_eq!(status, StatusCode::OK, "{admin_hr}");
        assert_eq!(
            admin_hr,
            console_payroll_ui::render_shell(),
            "ADMIN without org-wide EmployeeDirectoryRead must omit Person/Employment Heads: {admin_hr}"
        );

        let (status, admin_pay) = get_ui_path(service.clone(), "/payroll", Some(&admin_tok)).await;
        assert_eq!(status, StatusCode::OK, "{admin_pay}");
        assert_eq!(
            admin_pay,
            console_payroll_ui::render_shell(),
            "ADMIN payroll route must deny-by-omission: {admin_pay}"
        );

        let (status, exec_home) = get_ui_path(service.clone(), "/", Some(&exec_tok)).await;
        assert_eq!(status, StatusCode::OK, "{exec_home}");
        assert!(
            exec_home.contains("data-screen=\"organization\"")
                && exec_home.contains("data-screen=\"hr\"")
                && exec_home.contains("data-screen=\"payroll\""),
            "EXECUTIVE home must mount every authorized body: {exec_home}"
        );
        assert!(
            exec_home.contains("href=\"/organization\"")
                && exec_home.contains("href=\"/hr\"")
                && exec_home.contains("href=\"/payroll\"")
                && exec_home.contains("조직")
                && exec_home.contains("인사")
                && exec_home.contains("급여"),
            "EXECUTIVE nav must expose authorized screens: {exec_home}"
        );
        assert!(
            exec_home.contains(&format!("data-org-id=\"{org_id}\""))
                && exec_home.contains(&format!("data-org-unit-id=\"{}\"", heads.org_unit_id))
                && exec_home.contains(&format!("data-person-id=\"{}\"", heads.person_id))
                && exec_home.contains(&format!("data-employment-id=\"{}\"", heads.employment_id))
                && exec_home.contains(&format!("data-run-id=\"{run_id}\""))
                && exec_home.contains("data-legal-name=")
                && exec_home.contains("data-display-name=")
                && exec_home.contains(&format!("href=\"/api/v1/companies/{org_id}\""))
                && exec_home.contains(&format!("href=\"/api/v1/org-units/{}\"", heads.org_unit_id))
                && exec_home.contains(&format!("href=\"/api/v1/persons/{}\"", heads.person_id))
                && exec_home.contains(&format!(
                    "href=\"/api/v1/employments/{}\"",
                    heads.employment_id
                ))
                && !exec_home.contains("data-slug")
                && !exec_home.contains("data-employee-"),
            "EXECUTIVE markup must carry published Head identifiers: {exec_home}"
        );
        assert!(
            exec_home.contains("/pkg/console_payroll_ui.js") && exec_home.contains("charset"),
            "payroll island hydrates via committed WASM; charset stays SSR: {exec_home}"
        );
        assert_ui_invariants(&exec_home);

        let (status, exec_org) =
            get_ui_path(service.clone(), "/organization", Some(&exec_tok)).await;
        assert_eq!(status, StatusCode::OK, "{exec_org}");
        assert!(
            exec_org.contains("data-screen=\"organization\"")
                && !exec_org.contains("data-screen=\"payroll\"")
                && !exec_org.contains("/pkg/")
                && exec_org.contains("href=\"/hr\"")
                && exec_org.contains("href=\"/payroll\""),
            "focused org keeps authorized nav and omits payroll WASM: {exec_org}"
        );
        assert_ui_invariants(&exec_org);

        let (status, exec_hr) = get_ui_path(service.clone(), "/hr", Some(&exec_tok)).await;
        assert_eq!(status, StatusCode::OK, "{exec_hr}");
        assert!(
            exec_hr.contains("data-screen=\"hr\"")
                && exec_hr.contains(&format!("data-person-id=\"{}\"", heads.person_id))
                && exec_hr.contains(&format!("data-employment-id=\"{}\"", heads.employment_id))
                && exec_hr.contains("data-legal-name=")
                && exec_hr.contains("data-appointed-on=")
                && !exec_hr.contains("data-employee-")
                && !exec_hr.contains("/api/v1/job-positions/")
                && !exec_hr.contains("/pkg/"),
            "EXECUTIVE HR is SSR Person+Employment Head, not an island: {exec_hr}"
        );
        assert_ui_invariants(&exec_hr);

        let (status, exec_pay) = get_ui_path(service.clone(), "/payroll", Some(&exec_tok)).await;
        assert_eq!(status, StatusCode::OK, "{exec_pay}");
        assert!(
            exec_pay.contains("data-screen=\"payroll\"")
                && exec_pay.contains(&format!("data-run-id=\"{run_id}\""))
                && exec_pay.contains("href=\"/organization\"")
                && exec_pay.contains("href=\"/hr\""),
            "focused payroll stays reachable from org/HR nav: {exec_pay}"
        );
        assert!(
            !exec_pay.contains("method=\"post\""),
            "shipping screens are read projections; UI mutations stay HOLD: {exec_pay}"
        );
        assert_ui_invariants(&exec_pay);

        let exec_audits = list_read_audits(&pool, executive).await;
        assert!(
            exec_audits > 0,
            "EXECUTIVE payroll listing must write payroll_run.list_read (got {exec_audits})"
        );
        assert_eq!(
            list_read_audits(&pool, member).await,
            0,
            "MEMBER must not audit a payroll list read they cannot perform"
        );
        assert_eq!(
            list_read_audits(&pool, admin).await,
            0,
            "ADMIN without org-wide PayrollRunRead must not audit payroll list reads"
        );

        let (status, foreign_html) = get_ui_path(service.clone(), "/", Some(&foreign_tok)).await;
        assert_eq!(status, StatusCode::OK, "{foreign_html}");
        assert!(
            !foreign_html.contains(&run_id) && !foreign_html.contains(&org_id),
            "other-org SUPER_ADMIN must not see KNL identifiers: {foreign_html}"
        );
        assert_ui_invariants(&foreign_html);

        let (status, super_html) = get_ui_path(service.clone(), "/", Some(&super_tok)).await;
        assert_eq!(status, StatusCode::OK, "{super_html}");
        assert!(
            super_html.contains(&format!("data-run-id=\"{run_id}\""))
                && super_html.contains("data-screen=\"organization\"")
                && super_html.contains("data-screen=\"hr\""),
            "SUPER_ADMIN must see the same authorized contract rows: {super_html}"
        );
        assert_ui_invariants(&super_html);
    }

    #[cfg(feature = "test-browser")]
    #[sqlx::test(migrations = false)]
    async fn authorized_payroll_served_wasm_filters_only_current_rows_in_real_browser(
        pool: PgPool,
    ) {
        use futures::FutureExt;
        use serde_json::Value;
        use sha2::{Digest, Sha256};
        use std::collections::{BTreeMap, BTreeSet};
        use std::path::PathBuf;
        use std::process::Stdio;
        use std::sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        };
        use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};

        async fn rows(pool: &PgPool) -> BTreeMap<String, String> {
            let tables: Vec<String> = sqlx::query_scalar("SELECT relname::text FROM pg_class WHERE relnamespace='public'::regnamespace AND relkind IN ('r','p') ORDER BY relname COLLATE \"C\"").fetch_all(pool).await.unwrap();
            assert!(!tables.is_empty() && tables.len() <= 1024);
            let mut captured = BTreeMap::new();
            for table in tables {
                let quoted = table.replace('"', "\"\"");
                let sql = format!(
                    "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text COLLATE \"C\"),'[]'::jsonb)::text FROM public.\"{quoted}\" t"
                );
                let value: String = sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
                    .fetch_one(pool)
                    .await
                    .unwrap();
                assert!(captured.insert(table, value).is_none());
            }
            assert!(
                captured.contains_key("_sqlx_migrations")
                    && captured.contains_key("audit_events")
                    && captured.contains_key("payroll_draft_runs")
                    && captured.contains_key("accounts")
            );
            captured
        }
        fn audit_delta(before: &str, after: &str) -> Vec<Value> {
            use serde_json::value::RawValue;
            let before: Vec<&RawValue> = serde_json::from_str(before).unwrap();
            let after: Vec<&RawValue> = serde_json::from_str(after).unwrap();
            let mut retained: BTreeSet<_> = before.iter().map(|r| r.get()).collect();
            assert_eq!(retained.len(), before.len());
            let mut seen = BTreeSet::new();
            let mut added = Vec::new();
            for row in after {
                assert!(seen.insert(row.get()));
                if !retained.remove(row.get()) {
                    added.push(serde_json::from_str(row.get()).unwrap());
                }
            }
            assert!(
                retained.is_empty(),
                "prior audit history was changed or omitted"
            );
            added
        }
        async fn event(reader: &mut tokio::io::BufReader<tokio::process::ChildStdout>) -> Value {
            let mut line = String::new();
            let size = tokio::time::timeout(Duration::from_secs(100), reader.read_line(&mut line))
                .await
                .unwrap()
                .unwrap();
            assert!(size > 0 && size < 4096, "bounded browser protocol required");
            serde_json::from_str(&line).unwrap()
        }
        fn alive(pid: u32) -> bool {
            assert!(pid > 1);
            std::process::Command::new("/bin/kill")
                .args(["-0", &pid.to_string()])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .unwrap()
                .success()
        }

        let driver = PathBuf::from(
            std::env::var_os("CONSOLE_HYDRATION_BROWSER_DRIVER").expect("reviewed driver required"),
        );
        let expected = std::env::var("CONSOLE_HYDRATION_BROWSER_SHA256")
            .expect("reviewed driver hash required");
        assert!(
            driver.is_absolute()
                && std::fs::symlink_metadata(&driver)
                    .unwrap()
                    .file_type()
                    .is_file()
        );
        let driver_bytes = std::fs::read(&driver).unwrap();
        assert_eq!(hex::encode(Sha256::digest(&driver_bytes)), expected);
        let output = PathBuf::from(
            std::env::var_os("CONSOLE_HYDRATION_BROWSER_OUTPUT").expect("fresh output required"),
        );
        assert!(output.is_absolute() && !output.exists());
        console_platform_test_support::prepare_account_test_database(&pool).await;
        let keys = keys();
        let org = OrgId::knl();
        let admin = UserId::new();
        let member = UserId::new();
        seed_user(&pool, org, admin, "SUPER_ADMIN").await;
        seed_user(&pool, org, member, "MEMBER").await;
        let first = seed_run(&pool, org, admin).await;
        let port = PgPayRunPort::new(
            runtime_role_pool(&pool).await,
            tokio::runtime::Handle::current(),
        );
        let command = PayRunCommand {
            org_id: org,
            command_id: CommandId::from_uuid(Uuid::new_v4()),
            actor_id: admin,
            query: PayRunQuery::CreateRun {
                run_id: Uuid::new_v4(),
                period_start: date!(2026 - 06 - 01),
                period_end: date!(2026 - 06 - 30),
                connector: Some("m2".to_owned()),
                job: Some("hydration-distinct-presentation".to_owned()),
            },
            action_key: "create_run".to_owned(),
            object_type_id: Uuid::nil(),
        };
        let created = tokio::task::spawn_blocking(move || port.execute(&command))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(created.target(), DispatchTarget::PayrollCreateRun);
        let second: Uuid = created.result()["draft_run_id"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        assert_ne!(first, second);
        // TEST_ONLY rendering fixture, as in retained PayRun port tests. This does
        // not claim that a payroll calculation or legal transition was completed.
        assert_eq!(
            sqlx::query(
                "UPDATE payroll_draft_runs SET status='CALCULATED' WHERE id=$1 AND status='BLOCKED_LEGAL_GATE'"
            )
            .bind(second)
            .execute(&pool)
            .await
            .unwrap()
            .rows_affected(),
            1
        );
        let statuses: Vec<(Uuid, String)> =
            sqlx::query_as("SELECT id,status FROM payroll_draft_runs WHERE id=ANY($1) ORDER BY id")
                .bind(vec![first, second])
                .fetch_all(&pool)
                .await
                .unwrap();
        assert_eq!(statuses.len(), 2);
        assert!(
            statuses
                .iter()
                .any(|(id, status)| *id == first && status == "BLOCKED_LEGAL_GATE")
                && statuses
                    .iter()
                    .any(|(id, status)| *id == second && status == "CALCULATED")
        );
        let foreign_org = OrgId::from_uuid(Uuid::new_v4());
        sqlx::query(
            "INSERT INTO organizations (id,slug,name) VALUES ($1,$2,'Hydration empty Company')",
        )
        .bind(*foreign_org.as_uuid())
        .bind(format!("hydrate-{}", foreign_org.as_uuid().simple()))
        .execute(&pool)
        .await
        .unwrap();
        let foreign = UserId::new();
        seed_user(&pool, foreign_org, foreign, "SUPER_ADMIN").await;
        let state = jwt_app_state(&pool, keys.public_pem.clone()).await;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (stop, stopped) = tokio::sync::oneshot::channel();
        let admin_token = bearer(&keys, org, admin, "SUPER_ADMIN");
        // TEST_ONLY real foreign-origin sink. A credential-free TCP/HTTP request
        // below must prove observation before the redirect's zero-request claim.
        let sink_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let sink_address = sink_listener.local_addr().unwrap();
        let sink_url = format!("http://{sink_address}/__test__/hydration/sink");
        let sink_counts = Arc::new([AtomicUsize::new(0), AtomicUsize::new(0)]);
        let observed_sink = sink_counts.clone();
        let sink_router = axum::Router::new().fallback(move |headers: http::HeaderMap| {
            let counts = observed_sink.clone();
            async move {
                counts[0].fetch_add(1, Ordering::SeqCst);
                if headers.contains_key(http::header::AUTHORIZATION) {
                    counts[1].fetch_add(1, Ordering::SeqCst);
                }
                (StatusCode::OK, "TEST_ONLY_SINK_OBSERVED")
            }
        });
        let (sink_stop, sink_stopped) = tokio::sync::oneshot::channel();
        let mut sink_server = tokio::spawn(async move {
            axum::serve(sink_listener, sink_router)
                .with_graceful_shutdown(async {
                    let _ = sink_stopped.await;
                })
                .await
        });
        let source_counts = Arc::new([AtomicUsize::new(0), AtomicUsize::new(0)]);
        let observed_source = source_counts.clone();
        let expected_authorization = format!("Bearer {admin_token}");
        let target = sink_url.clone();
        // This additional fixture route never replaces a production response.
        let router = build_router(state.clone()).route(
            "/__test__/hydration/redirect",
            axum::routing::get(move |headers: http::HeaderMap| {
                let counts = observed_source.clone();
                let expected = expected_authorization.clone();
                let target = target.clone();
                async move {
                    counts[0].fetch_add(1, Ordering::SeqCst);
                    if headers
                        .get(http::header::AUTHORIZATION)
                        .and_then(|v| v.to_str().ok())
                        == Some(expected.as_str())
                    {
                        counts[1].fetch_add(1, Ordering::SeqCst);
                    }
                    (StatusCode::FOUND, [(http::header::LOCATION, target)])
                }
            }),
        );
        let mut server = tokio::spawn(async move {
            axum::serve(
                listener,
                router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
            )
            .with_graceful_shutdown(async {
                let _ = stopped.await;
            })
            .await
        });
        let before = rows(&pool).await;
        let js_hash = hex::encode(Sha256::digest(console_payroll_ui::payroll_ui_js()));
        let wasm_hash = hex::encode(Sha256::digest(console_payroll_ui::payroll_ui_wasm()));
        let payload = json!({"origin":format!("http://{address}"),"admin":admin_token,"redirect_sink":sink_url,"member":bearer(&keys,org,member,"MEMBER"),"foreign":bearer(&keys,foreign_org,foreign,"SUPER_ADMIN"),"runs":statuses.iter().map(|(id,status)|json!({"id":id,"status":status})).collect::<Vec<_>>(),"assets":{"js":js_hash,"wasm":wasm_hash}});
        let mut child = tokio::process::Command::new("node")
            .arg(&driver)
            .arg(&output)
            .env_remove("DEBUG")
            .env_remove("PWDEBUG")
            .env_remove("NODE_DEBUG")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .unwrap();
        let mut input = child.stdin.take().unwrap();
        let mut reader = tokio::io::BufReader::new(child.stdout.take().unwrap());
        let mut owned_pid = None;
        let mut checkpoint_ok = false;
        let mut sink_positive = false;
        let outcome = std::panic::AssertUnwindSafe(async {
            let mut positive = tokio::time::timeout(Duration::from_secs(5), tokio::net::TcpStream::connect(sink_address)).await.unwrap().unwrap();
            let request = format!("GET /__test__/hydration/sink HTTP/1.1\r\nHost: {sink_address}\r\nConnection: close\r\n\r\n");
            positive.write_all(request.as_bytes()).await.unwrap();
            let mut response = Vec::new();
            tokio::time::timeout(Duration::from_secs(5), positive.read_to_end(&mut response)).await.unwrap().unwrap();
            assert!(response.starts_with(b"HTTP/1.1 200 ") && response.windows(b"TEST_ONLY_SINK_OBSERVED".len()).any(|w| w == b"TEST_ONLY_SINK_OBSERVED"));
            assert_eq!(sink_counts[0].load(Ordering::SeqCst), 1);
            assert_eq!(sink_counts[1].load(Ordering::SeqCst), 0);
            sink_counts[0].store(0, Ordering::SeqCst);
            sink_positive = true;
            let mut bytes = serde_json::to_vec(&payload).unwrap();
            bytes.push(b'\n');
            input.write_all(&bytes).await.unwrap();
            input.flush().await.unwrap();
            bytes.fill(0);
            let owned = event(&mut reader).await;
            assert!(
                owned["kind"] == "BROWSER_OWNED"
                    && owned["executable_sha256"]
                        == match (std::env::consts::OS, std::env::consts::ARCH) {
                ("macos", "aarch64") => "a0bfe7b4da4787b66058477d696cd1d09065d25f06a548947722b9af77ee8282",
                ("linux", "x86_64") => "ded93a9c9a53a1ae040f08124badcca95c938e9d5015ff340c3b5538c41bf39e",
                _ => panic!("unsupported reviewed browser platform"),
            }
            );
            let pid = u32::try_from(owned["pid"].as_u64().unwrap()).unwrap();
            assert!(alive(pid));
            owned_pid = Some(pid);
            let result = event(&mut reader).await;
            assert!(
                result["kind"] == "RESULT" && result["status"] == "PASSED",
                "actual hydration browser proof failed; inspect sanitized result"
            );
            assert_eq!(source_counts[0].load(Ordering::SeqCst), 1);
            assert_eq!(source_counts[1].load(Ordering::SeqCst), 1);
            assert_eq!(sink_counts[0].load(Ordering::SeqCst), 0);
            assert_eq!(sink_counts[1].load(Ordering::SeqCst), 0);
            let after = rows(&pool).await;
            assert_eq!(
                before.keys().collect::<Vec<_>>(),
                after.keys().collect::<Vec<_>>()
            );
            for (name, value) in &before {
                if name != "audit_events" {
                    assert!(
                        after[name] == *value,
                        "browser changed non-audit durable facts"
                    );
                }
            }
            let delta = audit_delta(&before["audit_events"], &after["audit_events"]);
            assert_eq!(
                delta.len(),
                3,
                "exact authorized document read audit count required"
            );
            let mut actors = BTreeMap::new();
            let mut new_audit_ids = BTreeSet::new();
            for row in delta {
                assert_eq!(row.as_object().unwrap().keys().map(String::as_str).collect::<BTreeSet<_>>(), BTreeSet::from([
                    "id", "actor", "action", "target_type", "target_id", "branch_id", "before_snap", "after_snap", "trace_id", "span_id", "occurred_at", "created_at", "org_id", "ip", "user_agent", "auth_method", "device", "classification_badges", "anomaly", "reason"
                ]));
                for name in ["branch_id", "before_snap", "after_snap", "ip", "user_agent", "auth_method", "device", "classification_badges", "anomaly", "reason"] {
                    assert!(row[name].is_null(), "plain list audit contains unexpected context");
                }
                let audit_id = Uuid::parse_str(row["id"].as_str().unwrap()).unwrap();
                assert!(!audit_id.is_nil() && new_audit_ids.insert(audit_id));
                assert!(
                    row["action"] == "payroll_run.list_read"
                        && row["target_type"] == "payroll_draft_run"
                        && row["target_id"] == "query"
                );
                let actor: Uuid = row["actor"].as_str().unwrap().parse().unwrap();
                let expected_org = if actor == *admin.as_uuid() {
                    *org.as_uuid()
                } else {
                    assert_eq!(actor, *foreign.as_uuid());
                    *foreign_org.as_uuid()
                };
                assert_eq!(row["org_id"], json!(expected_org));
                for (name, length) in [("trace_id", 32), ("span_id", 16)] {
                    let value = row[name].as_str().unwrap();
                    assert!(
                        value.len() == length
                            && value.bytes().all(|b| b.is_ascii_hexdigit())
                            && value.bytes().any(|b| b != b'0')
                    );
                }
                assert!(row["occurred_at"].is_string() && row["created_at"].is_string());
                *actors.entry(actor).or_insert(0usize) += 1;
            }
            assert_eq!(
                actors,
                BTreeMap::from([(*admin.as_uuid(), 2), (*foreign.as_uuid(), 1)])
            );
            checkpoint_ok = true;
        })
        .catch_unwind()
        .await;
        if outcome.is_err() {
            let _ = input.write_all(b"{\"kind\":\"ABORT\"}\n").await;
            let _ = input.flush().await;
        }
        drop(input);
        let child_status = tokio::time::timeout(Duration::from_secs(25), child.wait()).await;
        if child_status.is_err() {
            let _ = child.kill().await;
        }
        let _ = stop.send(());
        let server_clean = matches!(
            tokio::time::timeout(Duration::from_secs(5), &mut server).await,
            Ok(Ok(Ok(())))
        );
        if !server_clean {
            server.abort();
            let _ = server.await;
        }
        let _ = sink_stop.send(());
        let sink_clean = matches!(
            tokio::time::timeout(Duration::from_secs(5), &mut sink_server).await,
            Ok(Ok(Ok(())))
        );
        if !sink_clean {
            sink_server.abort();
            let _ = sink_server.await;
        }
        state.shutdown_realtime().await;
        let source_unchanged = std::fs::read(&driver).is_ok_and(|bytes| bytes == driver_bytes);
        let browser_exited = owned_pid.is_some_and(|pid| !alive(pid));
        let exit_ok = matches!(child_status,Ok(Ok(status)) if status.success());
        let receipt = json!({"kind":"INDEPENDENT_HYDRATION_DATABASE_EFFECTS","driver_sha256":expected,"driver_source_unchanged":source_unchanged,"browser_pid":owned_pid,"browser_exited":browser_exited,"driver_exit_success":exit_ok,"server_stopped":server_clean,"redirect_control":{"sink_stopped":sink_clean,"sink_positive_observed_then_reset":sink_positive,"source_requests":source_counts[0].load(Ordering::SeqCst),"source_expected_authorization":source_counts[1].load(Ordering::SeqCst),"sink_requests":sink_counts[0].load(Ordering::SeqCst),"sink_authorization":sink_counts[1].load(Ordering::SeqCst)},"exact_durable_effects_accepted":checkpoint_ok,"expected_authorized_read_audits":3,"embedded_assets":{"js":js_hash,"wasm":wasm_hash},"limits":"Fixture Bearer only; TEST_ONLY status projection; no native Account/Company/payroll workflow or release acceptance"});
        if output.is_dir() {
            std::fs::write(
                output.join("owner-receipt.json"),
                serde_json::to_vec_pretty(&receipt).unwrap(),
            )
            .unwrap();
        }
        assert!(
            source_unchanged
                && server_clean
                && sink_clean
                && (owned_pid.is_none() || browser_exited),
            "owned cleanup/source verification failed"
        );
        if let Err(panic) = outcome {
            std::panic::resume_unwind(panic);
        }
        assert!(exit_ok && browser_exited && checkpoint_ok);
    }

    /// Proposed-until-accepted ADR-0042 option 1 assertions. The ADR decides
    /// nothing. Live `/api/v1` Cookie deny is `cookie_does_not_authorize_json_api`,
    /// not this ignored module.
    mod adr0042_ssr_session_cookie {
        use super::*;
        use console_platform_provisioning::BootstrapCredentialStore;
        use http::Response;
        use serde_json::{Value, json};
        use time::Duration as TimeDuration;

        pub(super) const PROPOSED_SSR_SESSION_COOKIE: &str = "console_session";
        const PRODUCTION_REFRESH_COOKIE: &str = "console_refresh";
        const HTML_DOCUMENT_ROUTES: [&str; 4] = ["/", "/organization", "/hr", "/payroll"];
        const JSON_API_PATH: &str = "/api/v1/payroll/runs";
        /// Refresh TTL is 30 days. A session cookie is not that refresh token.
        const REFRESH_TTL_SECS: i64 = 60 * 60 * 24 * 30;

        pub(super) async fn get_html_with_cookie(
            app: axum::Router,
            uri: &str,
            cookie: &str,
        ) -> (StatusCode, String) {
            let response = app
                .oneshot(
                    Request::builder()
                        .uri(uri)
                        .header(
                            header::USER_AGENT,
                            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36",
                        )
                        .header(header::ACCEPT, "text/html,application/xhtml+xml")
                        .header(header::COOKIE, cookie)
                        .header("upgrade-insecure-requests", "1")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            let status = response.status();
            let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            (status, String::from_utf8(bytes.to_vec()).unwrap())
        }

        /// Present deny: assembled-router `/api/v1` is Bearer-only. Option 1
        /// would mint `console_session` with Path=/ so a browser sends it to
        /// JSON API too; the server must still ignore it. Keep this live after
        /// a session cookie exists.
        #[sqlx::test(migrations = false)]
        async fn cookie_does_not_authorize_json_api(pool: PgPool) {
            console_platform_test_support::prepare_account_test_database(&pool).await;
            let keys = keys();
            let org = OrgId::knl();
            let admin = UserId::new();
            seed_user(&pool, org, admin, "SUPER_ADMIN").await;
            let _run = seed_run(&pool, org, admin).await;
            let service = build_router(jwt_app_state(&pool, keys.public_pem.clone()).await);
            let token = bearer(&keys, org, admin, "SUPER_ADMIN");

            for cookie in [
                format!("{PROPOSED_SSR_SESSION_COOKIE}={token}"),
                format!("{PRODUCTION_REFRESH_COOKIE}={token}"),
            ] {
                let response = service
                    .clone()
                    .oneshot(
                        Request::builder()
                            .uri(JSON_API_PATH)
                            .header(header::ACCEPT, "application/json")
                            .header(header::COOKIE, cookie.clone())
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .unwrap();
                assert_eq!(
                    response.status(),
                    StatusCode::UNAUTHORIZED,
                    "/api/v1/* remains Bearer-only; cookie {cookie} must not authorize JSON API"
                );
            }

            let bearer_ok = service
                .oneshot(
                    Request::builder()
                        .uri(JSON_API_PATH)
                        .header(header::ACCEPT, "application/json")
                        .header(header::AUTHORIZATION, format!("Bearer {token}"))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(
                bearer_ok.status(),
                StatusCode::OK,
                "Bearer still authorizes /api/v1: {}",
                bearer_ok.status()
            );
        }

        fn jwt_app_state_with_auth(runtime_pool: PgPool, keys: &Keys) -> AppState {
            let config = AppConfig::from_pairs([
                (
                    "CONSOLE_DATABASE_DURABILITY",
                    r#"{"mode":"local_development"}"#.to_owned(),
                ),
                ("CONSOLE_APP_ROLE", AppRole::Api.to_string()),
                ("CONSOLE_HTTP_ADDR", "127.0.0.1:0".to_owned()),
                ("CONSOLE_JWT_ISSUER", TEST_ISSUER.to_owned()),
                ("CONSOLE_JWT_AUDIENCE", TEST_AUDIENCE.to_owned()),
                ("CONSOLE_JWT_PRIVATE_KEY_PEM", keys.private_pem.clone()),
                ("CONSOLE_JWT_PUBLIC_KEY_PEM", keys.public_pem.clone()),
                ("CONSOLE_WEBAUTHN_RP_ID", "example.com".to_owned()),
                (
                    "CONSOLE_WEBAUTHN_RP_ORIGIN",
                    "https://auth.example.com".to_owned(),
                ),
                ("CONSOLE_WEBAUTHN_RP_NAME", "Console".to_owned()),
                ("CONSOLE_COOKIE_SECURE", "true".to_owned()),
            ])
            .unwrap();
            AppState::new(config, DatabaseDependency::Postgres(runtime_pool)).unwrap()
        }

        fn set_cookie_named(response: &Response<Body>, name: &str) -> Option<String> {
            let prefix = format!("{name}=");
            response
                .headers()
                .get_all(header::SET_COOKIE)
                .iter()
                .filter_map(|value| value.to_str().ok())
                .find(|value| value.starts_with(&prefix))
                .map(ToOwned::to_owned)
        }

        fn cookie_attr<'a>(set_cookie: &'a str, name: &str) -> Option<&'a str> {
            set_cookie.split(';').find_map(|part| {
                let part = part.trim();
                part.split_once('=')
                    .and_then(|(key, value)| key.eq_ignore_ascii_case(name).then_some(value))
            })
        }

        fn cookie_has_flag(set_cookie: &str, name: &str) -> bool {
            set_cookie
                .split(';')
                .any(|part| part.trim().eq_ignore_ascii_case(name))
        }

        fn cookie_value<'a>(set_cookie: &'a str, name: &str) -> &'a str {
            set_cookie
                .strip_prefix(&format!("{name}="))
                .and_then(|rest| rest.split(';').next())
                .map(str::trim)
                .unwrap()
        }

        /// Proposed-until-accepted: mint attributes, HTML document GETs, replica
        /// share. Not executed until a session cookie exists. `/api/v1` Cookie
        /// deny is `cookie_does_not_authorize_json_api` (live). Not a CNPG=3,
        /// PITR, or live-exposure claim.
        #[ignore = "ADR-0042 proposed; SSR session cookie not implemented"]
        #[sqlx::test(migrations = "../crates/platform/db/migrations")]
        async fn adr0042_ssr_session_cookie_contract(pool: PgPool) {
            let keys = keys();
            let org = OrgId::knl();
            let admin = UserId::new();
            seed_user(&pool, org, admin, "SUPER_ADMIN").await;
            seed_heads(&pool, org, admin).await;
            let run = seed_run(&pool, org, admin).await;
            let runtime = runtime_role_pool(&pool).await;
            let replica_a = build_router(jwt_app_state_with_auth(runtime.clone(), &keys));
            let replica_b = build_router(jwt_app_state_with_auth(runtime, &keys));

            let issued = BootstrapCredentialStore
                .issue_for_zero_credential_user(
                    &pool,
                    *admin.as_uuid(),
                    org,
                    OffsetDateTime::now_utc(),
                    TimeDuration::hours(24),
                )
                .await
                .expect("bootstrap OTP for cookie-mode login");
            let login = replica_a
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/api/v1/auth/otp/redeem")
                        .method("POST")
                        .header(header::CONTENT_TYPE, "application/json")
                        .header("x-auth-transport", "cookie")
                        .body(Body::from(
                            json!({ "otp": issued.token.as_str() }).to_string(),
                        ))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(login.status(), StatusCode::OK, "cookie-mode login");

            let session_set = set_cookie_named(&login, PROPOSED_SSR_SESSION_COOKIE).expect(
                "ADR-0042 option 1: cookie-mode login must mint console_session beside console_refresh",
            );
            assert!(
                cookie_has_flag(&session_set, "HttpOnly"),
                "session cookie must be HttpOnly: {session_set}"
            );
            assert!(
                cookie_has_flag(&session_set, "Secure"),
                "session cookie must be Secure: {session_set}"
            );
            assert_eq!(
                cookie_attr(&session_set, "SameSite"),
                Some("Lax"),
                "session cookie must be SameSite=Lax so a top-level navigation to `/` carries it: {session_set}"
            );
            assert_eq!(
                cookie_attr(&session_set, "Path"),
                Some("/"),
                "session cookie Path must be `/` so the browser sends it to HTML documents, not Path=/api/v1/auth: {session_set}"
            );
            let max_age = cookie_attr(&session_set, "Max-Age")
                .and_then(|raw| raw.parse::<i64>().ok())
                .expect("session cookie TTL must be bounded by Max-Age");
            assert!(
                max_age > 0 && max_age < REFRESH_TTL_SECS,
                "session cookie is not a refresh token; Max-Age must be positive and shorter than the 30-day refresh TTL, got {max_age}: {session_set}"
            );

            let session_value = cookie_value(&session_set, PROPOSED_SSR_SESSION_COOKIE);
            assert!(!session_value.is_empty(), "{session_set}");
            if let Some(refresh_set) = set_cookie_named(&login, PRODUCTION_REFRESH_COOKIE) {
                assert_ne!(
                    session_value,
                    cookie_value(&refresh_set, PRODUCTION_REFRESH_COOKIE),
                    "session cookie must not reuse the refresh token value"
                );
            }

            let login_body: Value =
                serde_json::from_slice(&to_bytes(login.into_body(), usize::MAX).await.unwrap())
                    .unwrap();
            let access_token = login_body["access_token"]
                .as_str()
                .expect("access token remains a Bearer for /api/v1");
            assert_ne!(
                session_value, access_token,
                "session cookie is not the access token; HTML must not need localStorage for it"
            );

            let cookie_header = format!("{PROPOSED_SSR_SESSION_COOKIE}={session_value}");
            for uri in HTML_DOCUMENT_ROUTES {
                let (status, html) =
                    get_html_with_cookie(replica_a.clone(), uri, &cookie_header).await;
                assert_eq!(status, StatusCode::OK, "{uri} {html}");
                assert_ne!(
                    html,
                    console_payroll_ui::render_shell(),
                    "HTML GET {uri} must authenticate from the session cookie, not render the empty shell: {html}"
                );
            }
            let (status, home) = get_html_with_cookie(replica_a.clone(), "/", &cookie_header).await;
            assert_eq!(status, StatusCode::OK, "{home}");
            assert!(
                home.contains(&format!("data-run-id=\"{run}\"")),
                "session-cookie navigation must reach the authorized run: {home}"
            );

            let (status, replica_html) = get_html_with_cookie(replica_b, "/", &cookie_header).await;
            assert_eq!(status, StatusCode::OK, "{replica_html}");
            assert!(
                replica_html.contains(&format!("data-run-id=\"{run}\"")),
                "two API replicas must share a session store or verify a signed cookie; this is not a CNPG=3 claim: {replica_html}"
            );
        }
    }
}
