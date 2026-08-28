//! Payroll `Layer::Ui` surface. SSR HTML for `/_ui`; no payroll math.
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

const PKG_JS: &str = "/_ui/pkg/console_payroll_ui.js";
const PKG_WASM: &str = "/_ui/pkg/console_payroll_ui_bg.wasm";
const ISLAND_BOOTSTRAP: &str = concat!(
    include_str!("island_script.js"),
    "(\"/_ui\", \"pkg\", \"console_payroll_ui\", \"console_payroll_ui_bg\");"
);

/// Contract-shaped run summary for SSR composition. Field names match
/// `PayrollRunSummary.yaml` required keys; values are already-authorized.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunSummary {
    pub id: String,
    pub period_start: String,
    pub period_end: String,
    pub source_label: String,
    pub status: String,
    pub calculation_enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[component]
pub fn Shell() -> impl IntoView {
    view! {
        <html>
            <head>
                <meta charset="utf-8" />
            </head>
            <body></body>
        </html>
    }
}

#[island]
pub fn AuthorizedRuns(runs: Vec<RunSummary>) -> impl IntoView {
    runs.into_iter()
        .map(|run| {
            view! {
                <span
                    data-run-id=run.id
                    data-period-start=run.period_start
                    data-period-end=run.period_end
                    data-source-label=run.source_label
                    data-status=run.status
                    data-calculation-enabled=run.calculation_enabled.to_string()
                    data-created-at=run.created_at
                    data-updated-at=run.updated_at
                ></span>
            }
        })
        .collect_view()
}

#[component]
pub fn AuthorizedShell(runs: Vec<RunSummary>) -> impl IntoView {
    view! {
        <html>
            <head>
                <meta charset="utf-8" />
                <link rel="modulepreload" href=PKG_JS />
                <link rel="preload" href=PKG_WASM r#as="fetch" r#type="application/wasm" />
                <script type="module">{ISLAND_BOOTSTRAP}</script>
            </head>
            <body>
                <AuthorizedRuns runs=runs />
            </body>
        </html>
    }
}

pub fn render_shell() -> String {
    let mut html = String::from("<!DOCTYPE html>");
    html.push_str(&Shell().to_html());
    html
}

pub fn render_shell_with(runs: &[RunSummary]) -> String {
    if runs.is_empty() {
        return render_shell();
    }
    let runs = runs.to_vec();
    let mut html = String::from("<!DOCTYPE html>");
    html.push_str(&view! { <AuthorizedShell runs=runs /> }.to_html());
    html
}

#[cfg(feature = "ssr")]
mod ssr {
    use super::{RunSummary, render_shell, render_shell_with};
    use axum::Router;
    use axum::http::header;
    use axum::response::{Html, IntoResponse};
    use axum::routing::get;

    pub fn html_shell() -> Html<String> {
        Html(render_shell())
    }

    pub fn html_shell_with(runs: &[RunSummary]) -> Html<String> {
        Html(render_shell_with(runs))
    }

    pub fn payroll_ui_js() -> &'static [u8] {
        include_bytes!("../pkg/console_payroll_ui.js")
    }

    pub fn payroll_ui_wasm() -> &'static [u8] {
        include_bytes!("../pkg/console_payroll_ui_bg.wasm")
    }

    async fn pkg_js() -> impl IntoResponse {
        (
            [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
            payroll_ui_js(),
        )
    }

    async fn pkg_wasm() -> impl IntoResponse {
        (
            [(header::CONTENT_TYPE, "application/wasm")],
            payroll_ui_wasm(),
        )
    }

    pub fn pkg_router<S>() -> Router<S>
    where
        S: Clone + Send + Sync + 'static,
    {
        Router::new()
            .route("/pkg/console_payroll_ui.js", get(pkg_js))
            .route("/pkg/console_payroll_ui_bg.wasm", get(pkg_wasm))
    }
}

#[cfg(feature = "ssr")]
pub use ssr::{html_shell, html_shell_with, payroll_ui_js, payroll_ui_wasm, pkg_router};

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    leptos::mount::hydrate_islands();
}

#[cfg(test)]
mod tests {
    use super::*;

    const PAYROLL_RUN_SUMMARY_SCHEMA: &str =
        include_str!("../../rest/openapi/schemas/PayrollRunSummary.yaml");

    fn sample_run() -> RunSummary {
        RunSummary {
            id: "00000000-0000-0000-0000-000000000001".to_owned(),
            period_start: "2026-06-01".to_owned(),
            period_end: "2026-06-30".to_owned(),
            source_label: "workflow_runtime_m2:run:example".to_owned(),
            status: "BLOCKED_LEGAL_GATE".to_owned(),
            calculation_enabled: false,
            created_at: "2026-06-01T00:00:00Z".to_owned(),
            updated_at: "2026-06-01T00:00:00Z".to_owned(),
        }
    }

    fn yaml_required_keys(schema: &str) -> Vec<&str> {
        let mut keys = Vec::new();
        let mut in_required = false;
        for line in schema.lines() {
            let trimmed = line.trim();
            if trimmed == "required:" {
                in_required = true;
                continue;
            }
            if in_required {
                if let Some(key) = trimmed.strip_prefix("- ") {
                    keys.push(key);
                } else if trimmed.ends_with(':') {
                    break;
                }
            }
        }
        keys
    }

    #[test]
    fn empty_runs_match_unauthenticated_shell() {
        assert_eq!(render_shell_with(&[]), render_shell());
        assert!(
            !render_shell().contains("data-run-id"),
            "empty shell must omit run markup: {}",
            render_shell()
        );
        assert!(
            !render_shell().contains("/_ui/pkg/"),
            "empty shell must not load WASM: {}",
            render_shell()
        );
    }

    #[test]
    fn authorized_runs_carry_contract_keys_and_omit_won() {
        let html = render_shell_with(&[sample_run()]);
        let lowered = html.to_ascii_lowercase();
        assert!(
            html.contains("data-run-id=\"00000000-0000-0000-0000-000000000001\""),
            "{html}"
        );
        for key in yaml_required_keys(PAYROLL_RUN_SUMMARY_SCHEMA) {
            let attr = if key == "id" {
                "data-run-id".to_owned()
            } else {
                format!("data-{}", key.replace('_', "-"))
            };
            assert!(
                html.contains(&attr),
                "authorized markup must carry contract key {key} as {attr}: {html}"
            );
        }
        assert!(!lowered.contains("won"), "won leaked: {html}");
        assert!(!html.contains("291_520"), "golden won leaked: {html}");
        assert!(!lowered.contains("payslip"), "payslip leaked: {html}");
        assert!(
            html.contains("rel=\"modulepreload\"") && html.contains(PKG_JS),
            "authorized markup must preload bindgen js: {html}"
        );
        assert!(
            html.contains(PKG_WASM),
            "authorized markup must preload wasm: {html}"
        );
        assert!(
            html.contains("(\"/_ui\", \"pkg\", \"console_payroll_ui\", \"console_payroll_ui_bg\")"),
            "authorized markup must invoke leptos island_script: {html}"
        );
        let component = island_component(&html).unwrap_or("");
        assert!(
            component.starts_with("AuthorizedRuns_"),
            "island data-component must be the wasm-bindgen export: {html}"
        );
        let js = std::str::from_utf8(payroll_ui_js()).unwrap_or("");
        assert!(
            js.contains(&format!("export function {component}")),
            "committed bindgen js must export {component}"
        );
        assert_eq!(
            &payroll_ui_wasm()[..4],
            b"\0asm",
            "committed wasm must be a Wasm module"
        );
        assert!(
            !render_shell().contains("AuthorizedRuns"),
            "empty shell must not emit an island: {}",
            render_shell()
        );
    }

    fn island_component(html: &str) -> Option<&str> {
        html.split_once("data-component=\"")
            .and_then(|(_, rest)| rest.split_once('"').map(|(id, _)| id))
    }
}
