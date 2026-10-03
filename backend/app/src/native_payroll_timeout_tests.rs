use super::*;
use axum::{
    body::{Body, to_bytes},
    http::{Request, header},
    routing::any,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use tower::ServiceExt;
use tower_http::timeout::TimeoutLayer;

struct Dropped(Arc<AtomicUsize>);
impl Drop for Dropped {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}
fn subject(counter: Arc<AtomicUsize>, immediate: bool) -> Router {
    let endpoint = any(move || {
        let counter = counter.clone();
        async move {
            let _guard = Dropped(counter);
            if immediate {
                return (StatusCode::CONFLICT, "ordinary-response");
            }
            std::future::pending::<()>().await;
            (StatusCode::OK, "unreachable-private-payload")
        }
    });
    let router = Router::new()
        .route("/companies/{org_id}/payroll", endpoint.clone())
        .route(
            console_payroll_rest::NATIVE_PAYROLL_RUNS_PATH,
            endpoint.clone(),
        )
        .route("/unrelated", endpoint)
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            std::time::Duration::from_millis(1),
        ));
    with_transport(console_platform_rest::with_platform_list_transport(
        console_platform_request_context::with_http_error_envelope(router),
    ))
}
#[cfg(not(feature = "test-postgres"))]
#[tokio::test]
async fn native_timeout_keeps_private_representation_cancels_and_preserves_other_responses() {
    let counter = Arc::new(AtomicUsize::new(0));
    let router = subject(counter.clone(), false);
    let company = "43210a78-23fa-4111-9222-001122334455";
    for (index, (path, html, method)) in [
        (format!("/companies/{company}/payroll"), true, "GET"),
        (
            format!("/api/v1/companies/{company}/payroll/runs"),
            false,
            "GET",
        ),
        (format!("/companies/{company}/payroll"), true, "HEAD"),
        (
            format!("/api/v1/companies/{company}/payroll/runs"),
            false,
            "HEAD",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(&path)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            counter.load(Ordering::SeqCst),
            index + 1,
            "timed-out handler future dropped"
        );
        assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);
        for (name, value) in [
            (header::CACHE_CONTROL, "no-store"),
            (header::PRAGMA, "no-cache"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ] {
            assert_eq!(response.headers().get_all(&name).iter().count(), 1);
            assert_eq!(response.headers().get(&name).unwrap(), value);
        }
        for name in [header::VARY, header::CONTENT_TYPE] {
            assert_eq!(response.headers().get_all(name).iter().count(), 1);
        }
        let vary = response
            .headers()
            .get(header::VARY)
            .unwrap()
            .to_str()
            .unwrap()
            .to_ascii_lowercase();
        for key in ["cookie", "origin"] {
            assert!(vary.split(',').any(|part| part.trim() == key));
        }
        assert!(!response.headers().contains_key(header::SET_COOKIE));
        let kind = response
            .headers()
            .get(header::CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap()
            .split(';')
            .next()
            .unwrap()
            .trim();
        assert_eq!(
            kind,
            if html {
                "text/html"
            } else {
                "application/json"
            }
        );
        if html {
            for name in [header::CONTENT_SECURITY_POLICY, header::REFERRER_POLICY] {
                assert_eq!(response.headers().get_all(name).iter().count(), 1);
            }
            assert_eq!(
                response.headers().get(header::REFERRER_POLICY).unwrap(),
                "no-referrer"
            );
            assert_eq!(
                response
                    .headers()
                    .get(header::CONTENT_SECURITY_POLICY)
                    .unwrap(),
                "default-src 'self'; script-src 'none'; style-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'"
            );
        }
        let body = to_bytes(response.into_body(), 256 * 1024).await.unwrap();
        if method == "HEAD" {
            assert!(body.is_empty());
            continue;
        }
        assert!(!body.is_empty());
        let text = std::str::from_utf8(&body).unwrap();
        for secret in [
            company,
            "unreachable-private-payload",
            "data-screen=\"payroll\"",
            "<form",
        ] {
            assert!(!text.contains(secret));
        }
        if html {
            assert!(text.starts_with(concat!("<!", "DOCTYPE html>")));
        } else {
            let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
            assert_eq!(
                value,
                serde_json::json!({"error":{"code":"request_timeout","message":"request timed out"}})
            );
        }
    }
    let response = router
        .oneshot(
            Request::builder()
                .uri("/unrelated")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);
    assert!(!response.headers().contains_key(header::CACHE_CONTROL));
    assert_eq!(counter.load(Ordering::SeqCst), 5);
    let body = to_bytes(response.into_body(), 4096).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        value,
        serde_json::json!({"error":{"code":"request_timeout","message":"request timed out"}})
    );
    let response = subject(counter.clone(), true)
        .oneshot(
            Request::builder()
                .uri(format!("/companies/{company}/payroll"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);
    assert_eq!(
        to_bytes(response.into_body(), 4096).await.unwrap().as_ref(),
        b"ordinary-response"
    );
    assert_eq!(counter.load(Ordering::SeqCst), 6);
}
