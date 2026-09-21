#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use super::*;
use crate::AuthRestConfig;
use p256::ecdsa::SigningKey;
use p256::elliptic_curve::rand_core::OsRng;
use p256::pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding};
use sqlx::postgres::PgPoolOptions;
use time::Duration;

const ORIGIN: &str = "https://auth.example.com";
const OPAQUE: &str = "capture-only-not-a-signed-token";

fn state() -> AuthRestState {
    // Transport-only tests. Lazy pool never connects; successful capture is
    // explicitly not current Account or Company authority.
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://unused:unused@localhost/unused")
        .unwrap();
    let key = SigningKey::random(&mut OsRng);
    AuthRestState::new(
        pool,
        AuthRestConfig {
            rp_id: "example.com".into(),
            rp_origin: ORIGIN.into(),
            rp_name: "Console".into(),
            ceremony_ttl: Duration::minutes(5),
            jwt_issuer: "test-capture".into(),
            jwt_audience: "test-capture".into(),
            jwt_private_key_pem: key.to_pkcs8_pem(LineEnding::LF).unwrap().to_string(),
            jwt_public_key_pem: key
                .verifying_key()
                .to_public_key_pem(LineEnding::LF)
                .unwrap(),
            refresh_token_ttl: Duration::days(30),
            refresh_family_absolute_ttl: Duration::hours(24),
            cookie_secure: true,
        },
    )
    .unwrap()
}

fn headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::COOKIE,
        HeaderValue::from_str(&format!("__Host-console_account_session={OPAQUE}")).unwrap(),
    );
    headers
}

fn mutation() -> HeaderMap {
    let mut h = headers();
    h.insert(header::ORIGIN, HeaderValue::from_static(ORIGIN));
    h.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    h.insert("sec-fetch-site", HeaderValue::from_static("same-origin"));
    h.insert(CSRF_HEADER, HeaderValue::from_static("capture-only-proof"));
    h
}

async fn api_error(
    result: Result<AccountEnrollmentCredentials, Box<Response>>,
    status: StatusCode,
    code: &str,
) {
    let response = match result {
        Ok(_) => panic!("capture unexpectedly accepted invalid transport"),
        Err(response) => response,
    };
    assert_eq!(response.status(), status);
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    assert_eq!(response.headers()[header::PRAGMA], "no-cache");
    assert_eq!(
        response.headers()[header::X_CONTENT_TYPE_OPTIONS],
        "nosniff"
    );
    assert!(
        response.headers()[header::VARY]
            .to_str()
            .unwrap()
            .contains("Cookie")
    );
    assert!(!response.headers().contains_key(header::SET_COOKIE));
    let bytes = to_bytes(response.into_body(), 4096).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["error"]["code"], code);
    assert!(!String::from_utf8(bytes.to_vec()).unwrap().contains(OPAQUE));
}

fn document_error(
    result: Result<AccountEnrollmentCredentials, NativeEntryError>,
    status: StatusCode,
) {
    match result {
        Ok(_) => panic!("document capture unexpectedly accepted invalid transport"),
        Err(error) => assert_eq!(error.status(), status),
    }
}

#[tokio::test]
async fn company_capture_preserves_distinct_api_and_document_grammars() {
    let state = state();
    assert!(state.company_api_read_credentials(&headers()).is_ok());
    assert!(state.company_document_credentials(&headers()).is_ok());
    for site in ["none", "same-origin", "same-site", "cross-site"] {
        let mut h = headers();
        h.insert("sec-fetch-site", HeaderValue::from_static(site));
        h.insert("sec-fetch-mode", HeaderValue::from_static("navigate"));
        h.insert("sec-fetch-dest", HeaderValue::from_static("document"));
        h.insert("sec-fetch-user", HeaderValue::from_static("?1"));
        assert!(state.company_document_credentials(&h).is_ok());
        if site == "same-origin" {
            assert!(state.company_api_read_credentials(&h).is_ok());
        } else {
            api_error(
                state.company_api_read_credentials(&h),
                StatusCode::FORBIDDEN,
                "request_origin_denied",
            )
            .await;
        }
    }
    let mut partial = headers();
    partial.insert("sec-fetch-site", HeaderValue::from_static("none"));
    document_error(
        state.company_document_credentials(&partial),
        StatusCode::BAD_REQUEST,
    );
    let mut non_document = headers();
    non_document.insert("sec-fetch-site", HeaderValue::from_static("same-origin"));
    non_document.insert("sec-fetch-mode", HeaderValue::from_static("cors"));
    non_document.insert("sec-fetch-dest", HeaderValue::from_static("empty"));
    document_error(
        state.company_document_credentials(&non_document),
        StatusCode::FORBIDDEN,
    );
    let mut wrong_origin = headers();
    wrong_origin.insert(
        header::ORIGIN,
        HeaderValue::from_static("https://other.example.com"),
    );
    api_error(
        state.company_api_read_credentials(&wrong_origin),
        StatusCode::FORBIDDEN,
        "request_origin_denied",
    )
    .await;
    document_error(
        state.company_document_credentials(&wrong_origin),
        StatusCode::FORBIDDEN,
    );
}

#[tokio::test]
async fn company_capture_rejects_ambiguity_bounds_and_refresh_fallback() {
    let state = state();
    for cookie in [
        "",
        "__Host-console_account_refresh=valid-looking-refresh",
        "console_refresh=legacy-refresh",
    ] {
        let mut h = HeaderMap::new();
        h.insert(header::COOKIE, HeaderValue::from_str(cookie).unwrap());
        api_error(
            state.company_api_read_credentials(&h),
            StatusCode::UNAUTHORIZED,
            "authentication_invalid",
        )
        .await;
        document_error(
            state.company_document_credentials(&h),
            StatusCode::UNAUTHORIZED,
        );
    }
    let mut malformed = HeaderMap::new();
    malformed.insert(
        header::COOKIE,
        HeaderValue::from_static(
            "__Host-console_account_session=; __Host-console_account_refresh=present",
        ),
    );
    api_error(
        state.company_api_read_credentials(&malformed),
        StatusCode::UNAUTHORIZED,
        "authentication_invalid",
    )
    .await;
    document_error(
        state.company_document_credentials(&malformed),
        StatusCode::BAD_REQUEST,
    );
    for bearer in [false, true] {
        let mut h = headers();
        if bearer {
            h.insert(
                header::AUTHORIZATION,
                HeaderValue::from_static("Bearer ignored-is-not-allowed"),
            );
        } else {
            h.append(
                header::COOKIE,
                HeaderValue::from_static("__Host-console_account_session=second"),
            );
        }
        api_error(
            state.company_api_read_credentials(&h),
            StatusCode::BAD_REQUEST,
            "ambiguous_credentials",
        )
        .await;
        document_error(
            state.company_document_credentials(&h),
            StatusCode::BAD_REQUEST,
        );
    }
    let mut large = headers();
    large.append(
        header::COOKIE,
        HeaderValue::from_str(&format!("unused={}", "x".repeat(16 * 1024))).unwrap(),
    );
    api_error(
        state.company_api_read_credentials(&large),
        StatusCode::PAYLOAD_TOO_LARGE,
        "request_too_large",
    )
    .await;
    document_error(
        state.company_document_credentials(&large),
        StatusCode::PAYLOAD_TOO_LARGE,
    );
    // Preserve existing API behavior: unused malformed cookie is not access.
    // Document admission deliberately rejects any malformed native cookie.
    let mut ancillary = headers();
    ancillary.append(
        header::COOKIE,
        HeaderValue::from_static("__Host-console_account_login="),
    );
    assert!(state.company_api_read_credentials(&ancillary).is_ok());
    document_error(
        state.company_document_credentials(&ancillary),
        StatusCode::BAD_REQUEST,
    );
}

#[tokio::test]
async fn company_mutation_capture_requires_exact_post_headers_and_bounded_proof() {
    let state = state();
    assert!(state.company_api_mutation_credentials(&mutation()).is_ok());
    for missing in [
        header::ORIGIN.as_str(),
        header::CONTENT_TYPE.as_str(),
        CSRF_HEADER,
    ] {
        let mut h = mutation();
        h.remove(missing);
        let (status, code) = match missing {
            "origin" => (StatusCode::FORBIDDEN, "request_origin_denied"),
            "content-type" => (StatusCode::UNPROCESSABLE_ENTITY, "invalid_request"),
            _ => (StatusCode::FORBIDDEN, "csrf_invalid"),
        };
        api_error(state.company_api_mutation_credentials(&h), status, code).await;
    }
    let mut h = mutation();
    h.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json; charset=utf-8"),
    );
    api_error(
        state.company_api_mutation_credentials(&h),
        StatusCode::UNPROCESSABLE_ENTITY,
        "invalid_request",
    )
    .await;
    let mut h = mutation();
    h.insert(CSRF_HEADER, HeaderValue::from_static(""));
    api_error(
        state.company_api_mutation_credentials(&h),
        StatusCode::FORBIDDEN,
        "csrf_invalid",
    )
    .await;
    let mut h = mutation();
    h.append(CSRF_HEADER, HeaderValue::from_static("second"));
    api_error(
        state.company_api_mutation_credentials(&h),
        StatusCode::FORBIDDEN,
        "csrf_invalid",
    )
    .await;
    let mut h = mutation();
    h.insert(
        CSRF_HEADER,
        HeaderValue::from_str(&"p".repeat(4097)).unwrap(),
    );
    api_error(
        state.company_api_mutation_credentials(&h),
        StatusCode::PAYLOAD_TOO_LARGE,
        "request_too_large",
    )
    .await;
    let mut h = mutation();
    h.insert(
        CSRF_HEADER,
        HeaderValue::from_str(&"p".repeat(4096)).unwrap(),
    );
    assert!(state.company_api_mutation_credentials(&h).is_ok());
}

#[tokio::test]
async fn company_capture_disabled_auth_fails_closed_without_storage_access() {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://unused:unused@localhost/unused")
        .unwrap();
    let state = AuthRestState::disabled(pool);
    api_error(
        state.company_api_read_credentials(&headers()),
        StatusCode::SERVICE_UNAVAILABLE,
        "authority_unavailable",
    )
    .await;
    api_error(
        state.company_api_mutation_credentials(&mutation()),
        StatusCode::SERVICE_UNAVAILABLE,
        "authority_unavailable",
    )
    .await;
    document_error(
        state.company_document_credentials(&headers()),
        StatusCode::SERVICE_UNAVAILABLE,
    );
}
