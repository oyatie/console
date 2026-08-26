//! Thin axum mount. App composition remains: resolve session from cookies/
//! headers (never WASM), load snapshots through ports, then call [`render_path`].

use crate::csp::{csp_header, CONTENT_SECURITY_POLICY};
use crate::shell::RenderedPage;

#[cfg(feature = "ssr")]
use crate::shell::{render_path, ShellData};

/// Remaining app-composition work (HOLD, not done here):
/// 1. `console-app` depends on this crate with `ssr`.
/// 2. Layer::App → Layer::Ui is now a legal edge.
/// 3. Implement `CompanyReadPort` / `PayrollReadPort` over existing postgres
///    adapters inside app (ui crates must not take that dependency).
/// 4. Serve `/ui/pkg/*.wasm` same-origin; session stays on cookies/headers.
/// 5. Re-authorize every `/ui/**` POST; `payroll.decide_run` SoD: decider ≠ submitter.
/// 6. Unauthenticated HTML fails closed to login — never embed JWTs in JS.
#[must_use]
pub fn composition_holds() -> &'static [&'static str] {
    &[
        "app-wires-ui-ports",
        "same-origin-wasm-pkg",
        "cookie-session-not-jwt-in-wasm",
        "post-reauthorize-and-sod",
        "persona-e2e",
    ]
}

#[must_use]
pub fn apply_csp_to_page(page: RenderedPage) -> (u16, [&'static str; 2], String) {
    let (name, value) = csp_header();
    debug_assert_eq!(value, CONTENT_SECURITY_POLICY);
    (page.status, [name, value], page.html)
}

#[cfg(feature = "ssr")]
pub use leptos_axum;

#[cfg(feature = "ssr")]
pub fn router_from_data(data: ShellData) -> axum::Router {
    use axum::routing::get;
    use axum::Router;

    let data = std::sync::Arc::new(data);
    Router::new()
        .route(
            "/",
            get({
                let data = data.clone();
                move || {
                    let data = data.clone();
                    async move { html_response(render_path("/", data.as_ref())) }
                }
            }),
        )
        .fallback({
            let data = data.clone();
            move |uri: axum::http::Uri| {
                let data = data.clone();
                async move { html_response(render_path(uri.path(), data.as_ref())) }
            }
        })
}

#[cfg(feature = "ssr")]
fn html_response(
    page: crate::shell::RenderedPage,
) -> (axum::http::StatusCode, [(axum::http::header::HeaderName, axum::http::HeaderValue); 1], axum::response::Html<String>)
{
    use axum::http::{header, HeaderValue, StatusCode};
    use axum::response::Html;

    let status = StatusCode::from_u16(page.status).unwrap_or(StatusCode::NOT_FOUND);
    let value = HeaderValue::from_static(CONTENT_SECURITY_POLICY);
    (status, [(header::CONTENT_SECURITY_POLICY, value)], Html(page.html))
}

#[cfg(feature = "ssr")]
pub fn render_for_headers(path: &str, data: &ShellData) -> RenderedPage {
    render_path(path, data)
}
