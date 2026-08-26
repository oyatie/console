//! HTML Content-Security-Policy for the SSR shell.
//!
//! `style-src` punches a hole only for the compile-time light CSS: the sha256 is
//! of [`crate::style::CSS`] as `<style>` text. Recompute it if CSS changes.

pub const CONTENT_SECURITY_POLICY: &str = "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self' 'sha256-FKoONoBmRjvoh6yE/ycjJ+aGVPb9+sgqwCIW64M0v7E='; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; object-src 'none'";

pub const CSP_HEADER_NAME: &str = "content-security-policy";

/// Base64 SHA-256 of [`crate::style::CSS`] (the exact `<style>` text content).
pub const STYLE_SRC_SHA256: &str = "sha256-FKoONoBmRjvoh6yE/ycjJ+aGVPb9+sgqwCIW64M0v7E=";

#[must_use]
pub fn csp_header() -> (&'static str, &'static str) {
    (CSP_HEADER_NAME, CONTENT_SECURITY_POLICY)
}

#[must_use]
pub fn csp_allows_wasm_eval_not_js_eval() -> bool {
    let script_src = directive(CONTENT_SECURITY_POLICY, "script-src");
    let without_wasm = script_src.replace("'wasm-unsafe-eval'", "");
    script_src.contains("'wasm-unsafe-eval'") && !without_wasm.contains("'unsafe-eval'")
}

#[must_use]
pub fn csp_allows_hashed_style_not_unsafe_inline() -> bool {
    let style_src = directive(CONTENT_SECURITY_POLICY, "style-src");
    let without_hashes: String = style_src
        .split_whitespace()
        .filter(|part| !part.starts_with("'sha256-"))
        .collect::<Vec<_>>()
        .join(" ");
    CONTENT_SECURITY_POLICY.contains("default-src 'self'")
        && style_src.contains("'self'")
        && style_src.contains(STYLE_SRC_SHA256)
        && !without_hashes.contains("unsafe-inline")
        && !CONTENT_SECURITY_POLICY.contains("unsafe-inline")
}

fn directive<'a>(policy: &'a str, name: &str) -> &'a str {
    policy
        .split(name)
        .nth(1)
        .and_then(|rest| rest.split(';').next())
        .unwrap_or("")
}
