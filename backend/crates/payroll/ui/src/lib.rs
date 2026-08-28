//! Payroll `Layer::Ui` surface. SSR HTML for `/_ui`; no payroll math.
//!
//! Unauthenticated markup is empty deny-by-omission. Authorized run summaries
//! are composed server-side from OpenAPI `PayrollRunSummary` required fields
//! (no won amounts). Islands/WASM hydration is not this slice.
use axum::Router;
use axum::response::Html;
use axum::routing::get;
use leptos::prelude::*;

/// Contract-shaped run summary for SSR composition. Field names match
/// `PayrollRunSummary.yaml` required keys; values are already-authorized.
#[derive(Clone, Debug, PartialEq, Eq)]
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

#[component]
pub fn AuthorizedShell(runs: Vec<RunSummary>) -> impl IntoView {
    view! {
        <html>
            <head>
                <meta charset="utf-8" />
            </head>
            <body>
                {runs
                    .into_iter()
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
                    .collect_view()}
            </body>
        </html>
    }
}

pub fn shell() -> impl IntoView {
    Shell()
}

pub fn render_shell() -> String {
    let mut html = String::from("<!DOCTYPE html>");
    html.push_str(&shell().to_html());
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

pub fn html_shell() -> Html<String> {
    Html(render_shell())
}

pub fn html_shell_with(runs: &[RunSummary]) -> Html<String> {
    Html(render_shell_with(runs))
}

async fn get_shell() -> Html<String> {
    html_shell()
}

pub fn router() -> Router {
    Router::new().route("/", get(get_shell))
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
    }
}
