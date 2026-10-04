//! Transport interruption regression proposal over the real Group parser and
//! retained PgOrgStore. Technical form values are never inserted business data.
//! Explicit AppState injection below proves transport composition only: it is
//! not verified serving startup, successful Auth, policy, or owner acceptance.
use super::*;
use axum::body::{Body, to_bytes};
use axum::http::Request as HttpRequest;
use tower::ServiceExt;

const GROUP: &str = "11111111-1111-4111-8111-111111111111";
#[cfg(feature = "test-postgres")]
const COMMAND: &str = "22222222-2222-4222-8222-222222222222";
#[cfg(feature = "test-postgres")]
const PROCESS: &str = "33333333-3333-4333-8333-333333333333";
#[cfg(feature = "test-postgres")]
const INCARNATION: &str = "44444444-4444-4444-8444-444444444444";
#[cfg(feature = "test-postgres")]
const DECOY: &str = "55555555-5555-4555-8555-555555555555";
const CSP: &str = "default-src 'self'; script-src 'none'; style-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'";

fn private_headers(response: &Response) -> Result<(), String> {
    for (name, value) in [
        (header::CONTENT_TYPE, "text/html; charset=utf-8"),
        (header::CACHE_CONTROL, "no-store"),
        (header::PRAGMA, "no-cache"),
        (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        (header::VARY, "Authorization, Cookie, Origin"),
        (header::REFERRER_POLICY, "same-origin"),
        (header::CONTENT_SECURITY_POLICY, CSP),
    ] {
        let values: Vec<_> = response.headers().get_all(&name).iter().collect();
        if values.len() != 1 || values[0].as_bytes() != value.as_bytes() {
            return Err(format!(
                "native Group header missing, changed, or duplicated: {name}"
            ));
        }
    }
    if response.headers().contains_key(header::SET_COOKIE) {
        return Err("transport recovery may not mint or expose a cookie".into());
    }
    Ok(())
}

#[cfg(feature = "test-postgres")]
async fn html(response: Response) -> String {
    private_headers(&response).expect("exact private native Group document headers");
    let body = to_bytes(response.into_body(), 65_536).await.unwrap();
    let text = String::from_utf8(body.to_vec()).unwrap();
    assert!(text.starts_with("<!DOCTYPE html>"));
    for secret in [
        "opaque-transport-session",
        "opaque-transport-proof",
        "csrf_proof",
        "<form",
    ] {
        assert!(
            !text.contains(secret),
            "recovery document leaked input or an actionable form"
        );
    }
    assert!(!text.contains("data-group-process-outcome=\"committed\""));
    text
}

#[cfg(not(feature = "test-postgres"))]
#[tokio::test]
async fn private_error_oracle_has_positive_corruption_and_unrelated_controls() {
    let control = || ui::document(ui::Page::Unavailable, StatusCode::PAYLOAD_TOO_LARGE);
    private_headers(&control()).expect("real native document positive control");
    for name in [
        header::CONTENT_TYPE,
        header::CACHE_CONTROL,
        header::PRAGMA,
        header::X_CONTENT_TYPE_OPTIONS,
        header::VARY,
        header::REFERRER_POLICY,
        header::CONTENT_SECURITY_POLICY,
    ] {
        let mut missing = control();
        missing.headers_mut().remove(&name);
        assert!(private_headers(&missing).is_err());
        let mut changed = control();
        changed
            .headers_mut()
            .insert(name.clone(), HeaderValue::from_static("corrupt"));
        assert!(private_headers(&changed).is_err());
        let mut duplicate = control();
        let original = duplicate.headers()[&name].clone();
        duplicate.headers_mut().append(name, original);
        assert!(private_headers(&duplicate).is_err());
    }
    let mut cookie = control();
    cookie
        .headers_mut()
        .insert(header::SET_COOKIE, HeaderValue::from_static("unexpected=1"));
    assert!(private_headers(&cookie).is_err());
    for status in [StatusCode::PAYLOAD_TOO_LARGE, StatusCode::TOO_MANY_REQUESTS] {
        let router = console_platform_request_context::with_http_error_envelope(
            Router::new().route("/unrelated", any(move || async move { status })),
        );
        let response = router
            .oneshot(
                HttpRequest::builder()
                    .uri("/unrelated")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), status);
        assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");
        assert!(private_headers(&response).is_err());
    }
}

#[cfg(not(feature = "test-postgres"))]
#[tokio::test]
async fn real_group_private_413_and_429_survive_shared_envelope() {
    for status in [StatusCode::PAYLOAD_TOO_LARGE, StatusCode::TOO_MANY_REQUESTS] {
        let before = to_bytes(error(status).into_body(), 65_536).await.unwrap();
        let router = console_platform_request_context::with_http_error_envelope(
            Router::new().route(LANDING, any(move || async move { error(status) })),
        );
        let response = router
            .oneshot(
                HttpRequest::builder()
                    .uri(format!("/groups/{GROUP}/identity"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), status);
        private_headers(&response).expect("real Group error remains private HTML through envelope");
        if status == StatusCode::TOO_MANY_REQUESTS {
            assert_eq!(
                response
                    .headers()
                    .get_all(header::RETRY_AFTER)
                    .iter()
                    .count(),
                1
            );
            assert_eq!(response.headers()[header::RETRY_AFTER], "60");
        }
        assert_eq!(
            to_bytes(response.into_body(), 65_536).await.unwrap(),
            before
        );
    }
}

#[cfg(feature = "test-postgres")]
mod owner_interruption {
    use super::super::super::{
        AppConfig, AppState, DatabaseDependency, build_router, build_view_as_issuer,
    };
    use super::*;
    use futures::FutureExt;
    use p256::{
        ecdsa::SigningKey,
        elliptic_curve::rand_core::OsRng,
        pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding},
    };
    use sqlx::{PgPool, postgres::PgPoolOptions};
    use std::{
        panic::AssertUnwindSafe,
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
    };

    // No DATABASE_URL configuration, credentials, business setup, migration,
    // custody profile, or technical role is used to authorize this injection.
    // One connected disposable SQLx database pool is held, so the real owner
    // awaits its FIRST acquisition and can execute no business SQL.
    async fn fixture(test_pool: &PgPool) -> (AppState, PgPool) {
        let signing = SigningKey::random(&mut OsRng);
        let private = signing.to_pkcs8_pem(LineEnding::LF).unwrap().to_string();
        let public = signing
            .verifying_key()
            .to_public_key_pem(LineEnding::LF)
            .unwrap();
        let mut config = AppConfig::from_pairs([
            (
                "CONSOLE_DATABASE_DURABILITY",
                r#"{"mode":"local_development"}"#.into(),
            ),
            ("CONSOLE_APP_ROLE", "api".into()),
            ("CONSOLE_HTTP_ADDR", "127.0.0.1:0".into()),
            ("CONSOLE_JWT_ISSUER", "https://example.com".into()),
            ("CONSOLE_JWT_AUDIENCE", "console-transport-test".into()),
            ("CONSOLE_JWT_PRIVATE_KEY_PEM", private),
            ("CONSOLE_JWT_PUBLIC_KEY_PEM", public),
            ("CONSOLE_WEBAUTHN_RP_ID", "example.com".into()),
            ("CONSOLE_WEBAUTHN_RP_ORIGIN", "https://example.com".into()),
            ("CONSOLE_WEBAUTHN_RP_NAME", "Console".into()),
        ])
        .unwrap();
        config.request_timeout = std::time::Duration::from_millis(100);
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .min_connections(1)
            .acquire_timeout(std::time::Duration::from_secs(10))
            .connect_with(test_pool.connect_options().as_ref().clone())
            .await
            .unwrap();
        let mut state = AppState::new(config, DatabaseDependency::Postgres(pool.clone())).unwrap();
        let auth = state.auth_rest.as_ref().unwrap().clone();
        let config = state.config.auth_rest.as_ref().unwrap();
        let store = PgOrgStore::new(pool.clone()).with_native_account_policy(
            state.jwt_verifier.as_ref().unwrap().clone(),
            build_view_as_issuer(config).unwrap(),
            config.refresh_family_absolute_ttl,
        );
        state.group_rest = Some(GroupRestState::new(store, auth).unwrap());
        assert!(
            state.serving_custody_profile.is_none(),
            "no serving-profile acceptance claimed"
        );
        (state, pool)
    }

    fn document(method: &str, path: &str, body: Body) -> HttpRequest<Body> {
        HttpRequest::builder()
            .method(method)
            .uri(path)
            .header(
                header::COOKIE,
                "__Host-console_account_session=opaque-transport-session",
            )
            .header(header::ORIGIN, "https://example.com")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header("sec-fetch-site", "same-origin")
            .header("sec-fetch-mode", "navigate")
            .header("sec-fetch-dest", "document")
            .body(body)
            .unwrap()
    }

    fn form(kind: u8, invalid_title: bool) -> String {
        let mut pairs = vec![("csrf_proof", "opaque-transport-proof".to_owned())];
        if kind != 2 {
            pairs.extend([
                ("command_id", COMMAND.into()),
                ("expected_group_revision", "7".into()),
                ("expected_group_incarnation", INCARNATION.into()),
                (
                    "expected_group_identity_policy_revision",
                    if kind == 0 { "0" } else { "1" }.into(),
                ),
            ]);
            if kind == 0 {
                pairs.extend([
                    ("process_id", PROCESS.into()),
                    ("expected_prior_process_revision", "0".into()),
                    ("process_expiry", "2028-02-29T00:15:07".into()),
                    ("operator_responsibility", "1".into()),
                    (
                        "title",
                        if invalid_title {
                            ""
                        } else {
                            "독립 검증 절차"
                        }
                        .into(),
                    ),
                    (
                        "method",
                        "ATTENDED_ACCOUNT_AND_DOCUMENTARY_REVIEW_V1".into(),
                    ),
                ]);
                for name in [
                    "intended_claimant_matching_procedure",
                    "account_possession_procedure",
                    "physical_human_evidence_procedure",
                    "duplicate_contradictory_claim_procedure",
                    "qualification_criteria_instruction",
                    "escalation_adjudication_procedure",
                    "evidence_minimization_retention_description",
                    "recipient_responsibility",
                ] {
                    pairs.push((name, "보관할 원본 절차 입력".into()));
                }
            } else {
                pairs.extend([
                    ("process_version", "1".into()),
                    ("process_digest", "ab".repeat(32)),
                    ("expected_process_head_revision", "2".into()),
                    ("expected_process_head_digest", "cd".repeat(32)),
                    ("reason", "원래 절차의 중지 사유".into()),
                ]);
            }
        }
        assert_eq!(
            pairs.len(),
            if kind == 0 {
                19
            } else if kind == 1 {
                10
            } else {
                1
            }
        );
        let mut encoded = url::form_urlencoded::Serializer::new(String::new());
        for (key, value) in pairs {
            encoded.append_pair(key, &value);
        }
        encoded.finish()
    }

    struct Dropped(Arc<AtomicUsize>);
    impl Drop for Dropped {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    fn never_finishes(count: Arc<AtomicUsize>) -> Body {
        Body::from_stream(futures::stream::unfold(
            Dropped(count),
            |guard| async move {
                let _guard = guard;
                std::future::pending::<Option<(Result<axum::body::Bytes, std::io::Error>, Dropped)>>().await
            },
        ))
    }

    #[sqlx::test(migrations = false)]
    async fn real_group_before_parse_timeout_and_native_controls(test_pool: PgPool) {
        let (state, pool) = fixture(&test_pool).await;
        let held = pool.acquire().await.unwrap();
        assert_eq!(pool.size(), 1);
        assert_eq!(pool.num_idle(), 0);
        let result = AssertUnwindSafe(async {
            let router = build_router(state.clone());
            // Positive known-result and malformed controls run before RED.
            let duplicate = format!("{}&command_id={DECOY}", form(0, false));
            for body in ["command_id=%FF".to_owned(), duplicate] {
                let response = router.clone().oneshot(document("POST",
                    &format!("/groups/{GROUP}/identity/processes?command_id={DECOY}"),
                    Body::from(body))).await.unwrap();
                assert_eq!(response.status(), StatusCode::BAD_REQUEST);
                let text = html(response).await;
                assert!(!text.contains("data-group-process-outcome=\"uncertain\""));
                assert!(!text.contains(&format!("/identity/requests/{COMMAND}")));
                assert!(!text.contains(DECOY));
            }
            let mut refused = document("POST", &format!("/groups/{GROUP}/identity/processes"),
                Body::from(form(0, false)));
            refused.headers_mut().remove(header::COOKIE);
            let response = router.clone().oneshot(refused).await.unwrap();
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
            assert!(!html(response).await.contains("data-group-process-outcome=\"uncertain\""));
            // An invalid strict route must not become a recovery locator.
            let response = router.clone().oneshot(document("GET",
                &format!("/groups/{GROUP}/identity/requests/not-a-command?command_id={DECOY}"),
                Body::empty())).await.unwrap();
            assert_eq!(response.status(), StatusCode::NOT_FOUND);
            assert!(!html(response).await.contains(DECOY));
            let response = router.clone().oneshot(HttpRequest::builder().uri("/readyz")
                .body(Body::empty()).unwrap()).await.unwrap();
            assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);
            assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");
            let body = to_bytes(response.into_body(), 1024).await.unwrap();
            assert_eq!(serde_json::from_slice::<serde_json::Value>(&body).unwrap(),
                serde_json::json!({"error":{"code":"request_timeout","message":"request timed out"}}));
            let dropped = Arc::new(AtomicUsize::new(0));
            let response = router.clone().oneshot(document("POST",
                &format!("/groups/{GROUP}/identity/processes?command_id={COMMAND}&group={DECOY}"),
                never_finishes(dropped.clone()))).await.unwrap();
            assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);
            assert_eq!(dropped.load(Ordering::SeqCst), 1, "timeout drops incomplete request body");
            let text = html(response).await;
            assert!(!text.contains("data-group-process-outcome=\"uncertain\""));
            for hint in [GROUP, COMMAND, DECOY] { assert!(!text.contains(hint)); }
            // Actual body-bound refusal must retain the native Group413 page.
            let response = router.clone().oneshot(document("POST",
                &format!("/groups/{GROUP}/identity/processes"),
                Body::from(vec![b'x'; 131_073]))).await.unwrap();
            assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
            assert!(!html(response).await.contains("data-group-process-outcome=\"uncertain\""));
            // These mounted Group routes select a Group/process, not a command.
            for path in [
                format!("/groups/{GROUP}/identity"),
                format!("/groups/{GROUP}/identity/processes/new"),
                format!("/groups/{GROUP}/identity/processes/{PROCESS}/replace"),
                format!("/groups/{GROUP}/identity/processes/{PROCESS}/suspend"),
            ] {
                let response = router.clone().oneshot(document("GET", &path, Body::empty())).await.unwrap();
                assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);
                let text = html(response).await;
                assert!(!text.contains("data-group-process-outcome=\"uncertain\""));
                for hint in [GROUP, COMMAND, PROCESS] { assert!(!text.contains(hint)); }
            }
            let response = router.oneshot(document("HEAD",
                &format!("/groups/{GROUP}/identity/requests/{COMMAND}"), Body::empty())).await.unwrap();
            assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
            private_headers(&response).unwrap();
            assert_eq!(response.headers()[header::ALLOW], "GET");
            assert!(to_bytes(response.into_body(), 65_536).await.unwrap().is_empty());
        }).catch_unwind().await;
        drop(held);
        state.shutdown_realtime().await;
        let readmitted =
            match tokio::time::timeout(std::time::Duration::from_secs(1), pool.acquire()).await {
                Ok(Ok(connection)) => {
                    drop(connection);
                    true
                }
                _ => false,
            };
        pool.close().await;
        if let Err(error) = result {
            std::panic::resume_unwind(error);
        }
        assert!(
            readmitted,
            "cancelled transport leaves no acquisition obstruction"
        );
    }

    #[sqlx::test(migrations = false)]
    async fn real_group_typed_timeout_retains_exact_original_locator(test_pool: PgPool) {
        let (state, pool) = fixture(&test_pool).await;
        let held = pool.acquire().await.unwrap();
        assert_eq!(pool.size(), 1);
        assert_eq!(pool.num_idle(), 0);
        let result = AssertUnwindSafe(async {
            let router = build_router(state.clone());
            let cases = [
                (
                    "POST",
                    format!("/groups/{GROUP}/identity/processes?command_id={DECOY}"),
                    form(0, false),
                ),
                (
                    "POST",
                    format!("/groups/{GROUP}/identity/processes?command_id={DECOY}"),
                    form(0, true),
                ),
                (
                    "POST",
                    format!(
                        "/groups/{GROUP}/identity/processes/{PROCESS}/suspend?command_id={DECOY}"
                    ),
                    form(1, false),
                ),
                (
                    "POST",
                    format!("/groups/{GROUP}/identity/requests/{COMMAND}/retry?command_id={DECOY}"),
                    form(2, false),
                ),
                (
                    "GET",
                    format!("/groups/{GROUP}/identity/requests/{COMMAND}?command_id={DECOY}"),
                    String::new(),
                ),
                (
                    "GET",
                    format!("/groups/{GROUP}/identity/requests/{COMMAND}/retry?command_id={DECOY}"),
                    String::new(),
                ),
            ];
            for (method, path, body) in cases {
                let response = router
                    .clone()
                    .oneshot(document(method, &path, Body::from(body)))
                    .await
                    .unwrap();
                assert_eq!(
                    response.status(),
                    StatusCode::REQUEST_TIMEOUT,
                    "real parser/owner must reach blocking acquisition: {method} {path}"
                );
                let text = html(response).await;
                assert!(text.contains("data-group-process-outcome=\"uncertain\""));
                let href = format!("href=\"/groups/{GROUP}/identity/requests/{COMMAND}\"");
                assert_eq!(
                    text.matches(&href).count(),
                    1,
                    "exact original locator, no changed query or retry path"
                );
                assert!(!text.contains(DECOY));
                assert!(!text.contains("보관할 원본 절차 입력"));
                assert!(!text.contains("원래 절차의 중지 사유"));
            }
        })
        .catch_unwind()
        .await;
        drop(held);
        state.shutdown_realtime().await;
        let readmitted =
            match tokio::time::timeout(std::time::Duration::from_secs(1), pool.acquire()).await {
                Ok(Ok(connection)) => {
                    drop(connection);
                    true
                }
                _ => false,
            };
        pool.close().await;
        if let Err(error) = result {
            std::panic::resume_unwind(error);
        }
        assert!(
            readmitted,
            "cancelled transport leaves no acquisition obstruction"
        );
    }
}
