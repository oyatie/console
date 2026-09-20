#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Existing shared mapper/body-limit owners; no new production marker API needed for RED.
use axum::body::{Body, Bytes, to_bytes};
use axum::extract::DefaultBodyLimit;
use axum::http::{HeaderValue, Request, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use console_platform_request_context::with_http_error_envelope;
use tower::ServiceExt;

// An unrelated server extension with a similar name cannot be the owner marker.
#[derive(Clone)]
struct NativeHtmlError;

#[tokio::test]
async fn untrusted_html_and_plain_rejections_keep_existing_json_envelopes() {
    for (status, types, expected) in [
        (
            StatusCode::PAYLOAD_TOO_LARGE,
            vec!["text/html"],
            "{\"error\":{\"code\":\"payload_too_large\",\"message\":\"request body too large\"}}",
        ),
        (
            StatusCode::PAYLOAD_TOO_LARGE,
            vec!["text/plain"],
            "{\"error\":{\"code\":\"payload_too_large\",\"message\":\"request body too large\"}}",
        ),
        (
            StatusCode::PAYLOAD_TOO_LARGE,
            vec![],
            "{\"error\":{\"code\":\"payload_too_large\",\"message\":\"request body too large\"}}",
        ),
        (
            StatusCode::PAYLOAD_TOO_LARGE,
            vec!["text/html", "application/json"],
            "{\"error\":{\"code\":\"payload_too_large\",\"message\":\"request body too large\"}}",
        ),
        (
            StatusCode::REQUEST_TIMEOUT,
            vec!["text/html"],
            "{\"error\":{\"code\":\"request_timeout\",\"message\":\"request timed out\"}}",
        ),
    ] {
        let app = with_http_error_envelope(axum::Router::new().route(
            "/account",
            get(move || {
                let types = types.clone();
                async move {
                    let mut response = (status, "untrusted body marker").into_response();
                    response.headers_mut().remove(header::CONTENT_TYPE);
                    for value in types {
                        response
                            .headers_mut()
                            .append(header::CONTENT_TYPE, HeaderValue::from_static(value));
                    }
                    response
                        .headers_mut()
                        .insert("x-console-native-error", HeaderValue::from_static("true"));
                    response.extensions_mut().insert(NativeHtmlError);
                    response
                }
            }),
        ));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/account")
                    .header(header::ACCEPT, "text/html")
                    .header("x-console-native-error", "true")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), status);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "application/json"
        );
        assert_eq!(
            to_bytes(response.into_body(), 4096).await.unwrap().as_ref(),
            expected.as_bytes()
        );
    }
}

#[tokio::test]
async fn existing_json_413_preserves_bytes_and_headers_with_native_looking_metadata() {
    for extra_html in [false, true] {
        let app = with_http_error_envelope(axum::Router::new().route(
            "/account",
            get(move || async move {
                let mut response = Response::new(Body::from(
                    "{\"error\":{\"code\":\"existing\",\"message\":\"exact existing JSON\"}}",
                ));
                *response.status_mut() = StatusCode::PAYLOAD_TOO_LARGE;
                response.headers_mut().insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("application/json"),
                );
                if extra_html {
                    response
                        .headers_mut()
                        .append(header::CONTENT_TYPE, HeaderValue::from_static("text/html"));
                }
                response
                    .headers_mut()
                    .insert("x-existing", HeaderValue::from_static("preserve"));
                response.extensions_mut().insert(NativeHtmlError);
                response
            }),
        ));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/account")
                    .header(header::ACCEPT, "text/html")
                    .header("x-console-native-error", "true")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
        let types: Vec<_> = response
            .headers()
            .get_all(header::CONTENT_TYPE)
            .iter()
            .collect();
        assert_eq!(types.len(), if extra_html { 2 } else { 1 });
        assert_eq!(types[0], "application/json");
        if extra_html {
            assert_eq!(types[1], "text/html");
        }
        assert_eq!(response.headers().get("x-existing").unwrap(), "preserve");
        assert_eq!(
            to_bytes(response.into_body(), 4096).await.unwrap().as_ref(),
            b"{\"error\":{\"code\":\"existing\",\"message\":\"exact existing JSON\"}}"
        );
    }
}

#[tokio::test]
async fn actual_body_limit_keeps_json_413_despite_html_request_hints() {
    let app = with_http_error_envelope(
        axum::Router::new()
            .route(
                "/api/body-limit-control",
                post(|_: Bytes| async {
                    panic!("oversized body reached handler");
                }),
            )
            .layer(DefaultBodyLimit::max(64)),
    );
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/body-limit-control")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ACCEPT, "text/html")
                .header("x-console-native-error", "true")
                .body(Body::from(vec![b'x'; 65]))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(
        response.headers().get(header::CONTENT_TYPE).unwrap(),
        "application/json"
    );
    assert_eq!(
        to_bytes(response.into_body(), 4096).await.unwrap().as_ref(),
        b"{\"error\":{\"code\":\"payload_too_large\",\"message\":\"request body too large\"}}"
    );
}
