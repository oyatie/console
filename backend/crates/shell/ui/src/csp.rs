//! HTML Content-Security-Policy for the SSR shell.

pub const CONTENT_SECURITY_POLICY: &str = "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; object-src 'none'";

pub const CSP_HEADER_NAME: &str = "content-security-policy";

#[must_use]
pub fn csp_header() -> (&'static str, &'static str) {
    (CSP_HEADER_NAME, CONTENT_SECURITY_POLICY)
}

#[must_use]
pub fn csp_allows_wasm_eval_not_js_eval() -> bool {
    let script_src = CONTENT_SECURITY_POLICY
        .split("script-src")
        .nth(1)
        .and_then(|rest| rest.split(';').next())
        .unwrap_or("");
    let without_wasm = script_src.replace("'wasm-unsafe-eval'", "");
    script_src.contains("'wasm-unsafe-eval'") && !without_wasm.contains("'unsafe-eval'")
}
