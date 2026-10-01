//! Transport-only interruption tests. No fake product store or database claim.
use super::*;
use axum::body::{Body, to_bytes};
use console_kernel_core::OrgId;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use tower::ServiceExt;
use tower_http::timeout::TimeoutLayer;
use uuid::Uuid;
const COMPANY: &str = "11111111-1111-4111-8111-111111111111";
const COMMAND: &str = "22222222-2222-4222-8222-222222222222";
struct Dropped(Arc<AtomicUsize>);
impl Drop for Dropped {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}
fn fixture(captured: bool, count: Arc<AtomicUsize>) -> Router {
    let endpoint = any(move |marker: Option<Extension<RecoveryLocator>>| {
        let count = count.clone();
        async move {
            let _dropped = Dropped(count);
            if captured {
                if let Some(Extension(marker)) = marker {
                    marker.retain_requested(
                        DirectoryRequestRef::new(
                            OrgId::from_uuid(Uuid::parse_str(COMPANY).unwrap()),
                            Uuid::parse_str(COMMAND).unwrap(),
                        )
                        .unwrap(),
                    );
                }
            }
            std::future::pending::<()>().await;
            "unreachable private payload"
        }
    });
    with_transport(console_platform_request_context::with_http_error_envelope(
        Router::new()
            .route(PREPARE, endpoint.clone())
            .route(LIST, endpoint.clone())
            .route("/unrelated", endpoint)
            .layer(TimeoutLayer::with_status_code(
                StatusCode::REQUEST_TIMEOUT,
                std::time::Duration::from_millis(1),
            )),
    ))
}

#[tokio::test]
async fn queried_people_reads_keep_no_referrer_on_all_transport_outcomes() {
    let path = format!("/companies/{COMPANY}/people?employee_number=%FF");
    for status in [
        StatusCode::OK,
        StatusCode::BAD_REQUEST,
        StatusCode::NOT_FOUND,
        StatusCode::SERVICE_UNAVAILABLE,
    ] {
        for method in ["GET", "HEAD"] {
            let router =
                with_transport(Router::new().route(LIST, any(move || async move { status })));
            let response = router
                .oneshot(
                    axum::http::Request::builder()
                        .method(method)
                        .uri(&path)
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), status);
            assert_eq!(response.headers()[header::REFERRER_POLICY], "no-referrer");
            if method == "HEAD" {
                assert!(
                    to_bytes(response.into_body(), 1024)
                        .await
                        .unwrap()
                        .is_empty()
                );
            }
        }
    }
    for method in ["GET", "HEAD"] {
        let response = fixture(false, Arc::new(AtomicUsize::new(0)))
            .oneshot(
                axum::http::Request::builder()
                    .method(method)
                    .uri(&path)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);
        assert_eq!(response.headers()[header::REFERRER_POLICY], "no-referrer");
        if method == "HEAD" {
            assert!(
                to_bytes(response.into_body(), 32768)
                    .await
                    .unwrap()
                    .is_empty()
            );
        }
    }
}
#[tokio::test]
async fn people_timeout_retains_only_requested_locator_after_capture_and_cancels_handler() {
    let count = Arc::new(AtomicUsize::new(0));
    for (i, (captured, method)) in [(false, "POST"), (true, "POST"), (true, "HEAD")]
        .into_iter()
        .enumerate()
    {
        let response = fixture(captured, count.clone())
            .oneshot(
                axum::http::Request::builder()
                    .method(method)
                    .uri(format!("/companies/{COMPANY}/people/requests"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);
        assert_eq!(count.load(Ordering::SeqCst), i + 1);
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
        assert_eq!(response.headers()[header::PRAGMA], "no-cache");
        assert_eq!(response.headers()[header::REFERRER_POLICY], "same-origin");
        assert!(
            response.headers()[header::CONTENT_TYPE]
                .to_str()
                .unwrap()
                .starts_with("text/html")
        );
        assert!(!response.headers().contains_key(header::SET_COOKIE));
        let body = to_bytes(response.into_body(), 32768).await.unwrap();
        if method == "HEAD" {
            assert!(body.is_empty());
            continue;
        }
        let text = std::str::from_utf8(&body).unwrap();
        assert!(!text.contains("unreachable private payload"));
        assert_eq!(text.contains("data-people-outcome=\"uncertain\""), captured);
        assert_eq!(
            text.contains(&format!("/companies/{COMPANY}/people/requests/{COMMAND}")),
            captured
        );
        assert!(!text.contains("data-people-outcome=\"committed\""));
        assert!(!text.contains("csrf_proof"));
    }
}
#[tokio::test]
async fn people_timeout_does_not_rewrite_unrelated_routes_or_known_response() {
    let count = Arc::new(AtomicUsize::new(0));
    let response = fixture(true, count)
        .oneshot(
            axum::http::Request::builder()
                .uri("/unrelated")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);
    assert!(
        response.headers()[header::CONTENT_TYPE]
            .to_str()
            .unwrap()
            .starts_with("application/json")
    );
    let router = with_transport(Router::new().route(
        PREPARE,
        any(|| async { (StatusCode::CONFLICT, "known-conflict") }),
    ));
    let response = router
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/companies/{COMPANY}/people/requests"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);
    assert_eq!(
        &to_bytes(response.into_body(), 1024).await.unwrap()[..],
        b"known-conflict"
    );
}
