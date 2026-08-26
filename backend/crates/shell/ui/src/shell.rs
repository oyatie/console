use leptos::prelude::*;

use console_ontology_ui::{CompanyPage, OrgPage, OrgSnapshot, PeoplePage};
use console_payroll_ui::{
    ApprovalsPage, AttendanceHandoffPage, EssPage, PayRunDetail, PayrollSnapshot, RunDetailPage,
    RunsPage,
};

use crate::caps::{normalize_path, path_allowed};
use crate::hub::WorkHubPage;
use crate::login::LoginPage;
use crate::nav::NavBar;
use crate::not_found::NotFoundPage;
use crate::policy::{PolicyFoldPage, PolicyGrant};
use crate::session::Session;
use crate::style::CSS;

const WASM_SCRIPT: &str = "/ui/pkg/console_shell_hydrate.js";

#[derive(Clone, Debug, Default)]
pub struct ShellData {
    pub session: Option<Session>,
    pub org: OrgSnapshot,
    pub payroll: PayrollSnapshot,
    pub grants: Vec<PolicyGrant>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderedPage {
    pub status: u16,
    pub html: String,
}

#[must_use]
pub fn not_found_html() -> String {
    wrap_document("페이지 없음", None, "/", view! { <NotFoundPage /> })
}

#[must_use]
pub fn login_html() -> String {
    wrap_document("로그인", None, "/login", view! { <LoginPage /> })
}

#[must_use]
pub fn render_path(path: &str, data: &ShellData) -> RenderedPage {
    let path = normalize_path(path);
    match data.session.as_ref() {
        None => {
            if path == "/" || path == "/login" {
                RenderedPage {
                    status: 200,
                    html: login_html(),
                }
            } else {
                RenderedPage {
                    status: 404,
                    html: not_found_html(),
                }
            }
        }
        Some(session) => {
            if !path_allowed(path, &session.caps) {
                return RenderedPage {
                    status: 404,
                    html: not_found_html(),
                };
            }
            let Some(body) = authed_body(path, session, data) else {
                return RenderedPage {
                    status: 404,
                    html: not_found_html(),
                };
            };
            RenderedPage {
                status: 200,
                html: wrap_document(title_for(path), Some(session), path, body),
            }
        }
    }
}

fn title_for(path: &str) -> &'static str {
    if path == "/" {
        "작업 허브"
    } else if path == "/company" {
        "회사"
    } else if path == "/org" {
        "조직"
    } else if path == "/people" {
        "구성원"
    } else if path == "/policy" {
        "권한 폴드"
    } else if path == "/attendance" {
        "근태 인수"
    } else if path == "/payroll/runs" {
        "급여 실행"
    } else if path.starts_with("/payroll/runs/") {
        "급여 실행 상세"
    } else if path == "/payroll/me" {
        "내 급여"
    } else if path == "/approvals" {
        "결재 수신함"
    } else {
        "콘솔"
    }
}

fn authed_body(path: &str, session: &Session, data: &ShellData) -> Option<impl IntoView> {
    if path == "/" {
        return Some(
            view! {
                <WorkHubPage
                    caps=session.caps.clone()
                    org=data.org.clone()
                    payroll=data.payroll.clone()
                />
            }
            .into_any(),
        );
    }
    if path == "/company" {
        return Some(view! { <CompanyPage head=data.org.company.clone() /> }.into_any());
    }
    if path == "/org" {
        return Some(
            view! {
                <OrgPage
                    units=data.org.org_units.clone()
                    jobs=data.org.job_positions.clone()
                />
            }
            .into_any(),
        );
    }
    if path == "/people" {
        return Some(
            view! {
                <PeoplePage
                    people=data.org.people.clone()
                    employments=data.org.employments.clone()
                />
            }
            .into_any(),
        );
    }
    if path == "/policy" {
        return Some(view! { <PolicyFoldPage grants=data.grants.clone() /> }.into_any());
    }
    if path == "/attendance" {
        return Some(
            view! {
                <AttendanceHandoffPage period=data.payroll.attendance.clone() />
            }
            .into_any(),
        );
    }
    if path == "/payroll/runs" {
        return Some(view! { <RunsPage runs=data.payroll.runs.clone() /> }.into_any());
    }
    if let Some(run_id) = path.strip_prefix("/payroll/runs/") {
        let detail = data
            .payroll
            .selected
            .clone()
            .filter(|detail| detail.run.id == run_id)
            .or_else(|| {
                data.payroll
                    .runs
                    .iter()
                    .find(|run| run.id == run_id)
                    .map(|run| PayRunDetail {
                        run: run.clone(),
                        lines: Vec::new(),
                        calculation_version: None,
                        total_net_won: None,
                        total_net_lineage: None,
                        exceptions: Vec::new(),
                        payable: false,
                    })
            })?;
        return Some(
            view! {
                <RunDetailPage
                    detail=detail
                    actor_id=session.user_id.clone()
                    can_manage=session.caps.payroll_manage
                />
            }
            .into_any(),
        );
    }
    if path == "/payroll/me" {
        return Some(view! { <EssPage payslip=data.payroll.my_payslip.clone() /> }.into_any());
    }
    if path == "/approvals" {
        return Some(
            view! {
                <ApprovalsPage
                    items=data.payroll.inbox.clone()
                    actor_id=session.user_id.clone()
                />
            }
            .into_any(),
        );
    }
    None
}

fn wrap_document(
    title: &str,
    session: Option<&Session>,
    current: &str,
    body: impl IntoView,
) -> String {
    let inner = match session {
        Some(session) => view! {
            <NavBar session=session.clone() current=current.to_owned() />
            <main>{body}</main>
        }
        .into_any(),
        None => view! { <main>{body}</main> }.into_any(),
    };
    let page = view! {
        <html lang="ko">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <title>{title.to_owned()}</title>
                <style>{CSS}</style>
                <script type="module" src=WASM_SCRIPT></script>
            </head>
            <body>{inner}</body>
        </html>
    };
    format!("<!DOCTYPE html>{}", view_to_html(page))
}

#[cfg(feature = "ssr")]
fn view_to_html(view: impl IntoView) -> String {
    view.into_view().to_html()
}

#[cfg(not(feature = "ssr"))]
fn view_to_html(_view: impl IntoView) -> String {
    String::new()
}
