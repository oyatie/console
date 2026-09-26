//! Payroll `Layer::Ui` surface. SSR HTML for `/`; no payroll math.
pub mod native_account;
#[cfg(feature = "ssr")]
pub mod native_payroll;
#[cfg(feature = "ssr")]
pub mod native_people;
pub mod native_policy;
mod organization;
mod payroll_workspace;
mod people;

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

const PKG_JS: &str = "/pkg/console_payroll_ui.js";
const PKG_WASM: &str = "/pkg/console_payroll_ui_bg.wasm";
/// The whole stylesheet, inlined.
///
/// It is inlined rather than served because the unauthorized shell must carry
/// no `/pkg/` reference at all -- an external stylesheet there would be a
/// request an unauthenticated visitor makes, and the test that pins the empty
/// shell forbids it. Inlining also costs the authorized screens no extra
/// round trip. No `>` in any selector: `<style>` content is emitted raw, and
/// descendant selectors keep the sheet independent of that.
const STYLE: &str = concat!(
    include_str!("theme.css"),
    r#"
.workspace{display:grid;grid-template-columns:224px minmax(0,1fr);min-height:100dvh}
.skip-link{position:fixed;left:16px;top:12px;z-index:20;padding:12px 18px;background:var(--ink);
color:#fff;border-radius:8px;transform:translateY(-160%)}
.skip-link:focus{transform:none}
header.app{position:sticky;top:0;align-self:start;height:100dvh;padding:28px 16px;
background:var(--surface);border-right:1px solid var(--line)}
.brand{display:flex;align-items:center;gap:12px;padding:0 12px 30px;color:var(--ink);
font-size:20px;font-weight:750;letter-spacing:-.04em;text-decoration:none}
.brand-mark{display:grid;place-items:center;flex:none;width:32px;height:32px;border-radius:9px;
background:#eeb94b;color:#302714;font-size:20px;font-weight:800}
.nav-group{margin:12px 12px 8px;color:var(--muted);font-size:12px;font-weight:650;letter-spacing:.04em}
header.app nav{display:flex;flex-direction:column;gap:4px}
header.app nav a{display:flex;align-items:center;gap:12px;min-height:46px;padding:10px 12px;
border-radius:8px;color:var(--muted);font-weight:550;text-decoration:none}
header.app nav a:hover{background:var(--bg);color:var(--ink)}
header.app nav a[aria-current]{background:var(--accent-bg);color:#62410b;font-weight:700}
.nav-symbol{font-size:18px;width:22px;text-align:center}
.company-workspace .company-current{margin:0;padding:10px 12px;border-radius:8px;background:var(--accent-bg);color:var(--accent-hover);font-weight:650}
.company-workspace .company-account-nav{margin-top:32px}
.company-workspace h1{word-break:keep-all;overflow-wrap:anywhere}
.company-workspace .company-identifier{margin-top:16px;font-size:13px;color:var(--muted)}
.company-workspace .company-identifier summary{display:list-item;min-height:44px;padding:10px 0;cursor:pointer}
.company-workspace .company-identifier p{margin:0;padding:8px 0;font-variant-numeric:tabular-nums}
.company-workspace .company-next-task{display:grid;grid-template-columns:minmax(0,1fr) auto;align-items:center;gap:12px 24px;min-width:0;margin:0 0 28px;padding:20px 24px;background:var(--surface);border:1px solid var(--line);border-left:3px solid var(--accent);border-radius:12px}
.company-workspace .company-next-task h2{margin:0 0 6px;font-size:17px;color:var(--ink)}
.company-workspace .company-next-task p{margin:0;color:var(--muted);font-size:14px;word-break:keep-all;overflow-wrap:anywhere}
.company-workspace .company-group{margin:0 0 32px}
.company-workspace .company-group h2{margin:0 0 12px;font-size:15px;font-weight:650;color:var(--muted)}
.company-workspace .company-destinations{display:grid;grid-template-columns:repeat(auto-fit,minmax(min(100%,260px),1fr));gap:16px}
.company-workspace .company-destination{min-width:0;background:var(--surface);border:1px solid var(--line);border-radius:12px;box-shadow:0 2px 6px #20292304}
.company-workspace .company-payroll{border-top:3px solid var(--accent)}
.company-workspace .company-destination h3{margin:0;font-size:19px;line-height:1.45}
.company-workspace .company-destination a{display:block;min-height:56px;padding:18px 22px 10px;color:var(--ink);text-decoration:none;border-radius:11px 11px 0 0}
.company-workspace .company-destination a:hover{color:var(--accent);background:var(--surface-subtle)}
.company-workspace .company-destination p{margin:0;padding:0 22px 22px;color:var(--muted);font-size:14px;word-break:keep-all;overflow-wrap:anywhere}
.company-workspace .company-empty p{padding:0 22px 16px;color:var(--muted)}
@media(max-width:680px){.company-workspace .company-account-nav{margin-top:12px}.company-workspace .company-current{display:inline-block}.company-workspace .company-next-task{grid-template-columns:minmax(0,1fr);padding:16px}.company-workspace .company-next-task .policy-button{width:100%}.company-workspace .company-destination a{padding:16px 18px 10px}.company-workspace .company-destination p{padding:0 18px 18px}}

main{min-width:0;width:100%;max-width:1440px;padding:36px 40px 80px;margin:0 auto}
.page-heading{margin-bottom:28px;padding-bottom:24px;border-bottom:1px solid var(--line)}
.page-eyebrow{margin:0 0 6px;font-size:12px;font-weight:650;letter-spacing:.08em;color:var(--muted)}
h1{margin:0;font-size:28px;line-height:1.35;letter-spacing:-.04em;font-weight:700}
.page-description{margin:10px 0 0;color:var(--muted);max-width:65ch;word-break:keep-all;overflow-wrap:anywhere}
h2,h3,p,dd,a,summary{overflow-wrap:anywhere}
.panel{background:var(--surface);border:1px solid var(--line);border-radius:12px;margin:0 0 24px;min-width:0}
.panel h2{margin:0;padding:18px 22px;font-size:17px;line-height:1.45;font-weight:650;border-bottom:1px solid var(--line)}
.toolbar{display:flex;flex-wrap:wrap;align-items:center;gap:12px;padding:14px 22px;border-bottom:1px solid var(--line)}
.fl{font-size:14px;font-weight:600;color:var(--muted)}
select,input,textarea,button{font:inherit}
select{min-height:44px;padding:8px 12px;border:1px solid var(--control-border);border-radius:8px;background:var(--surface);color:var(--ink);max-width:100%}
.row{display:flex;align-items:baseline;gap:14px;padding:17px 22px;border-bottom:1px solid var(--line);color:var(--ink);text-decoration:none}
.row:last-child{border-bottom:0}
.row:hover{background:#fafbf8}
.row .name{font-weight:600;min-width:0}
.row .meta{flex:1 1 16ch;min-width:0;color:var(--muted);font-size:14px;overflow-wrap:anywhere}
.row .rev{flex:none;color:var(--muted);font-size:13px;font-variant-numeric:tabular-nums}
.badge{flex:none;max-width:100%;margin-left:auto;padding:4px 10px;border-radius:6px;background:var(--chip);
color:var(--chip-ink);font-size:12px;font-weight:650;overflow-wrap:anywhere}
.badge[data-status*='BLOCK']{background:var(--flag);color:var(--flag-ink)}
.state{margin:0;padding:32px 22px;color:var(--muted)}
.people-introduction{margin:0 0 20px}.people-introduction h2{margin:0 0 6px;font-size:18px;letter-spacing:-.02em}
.people-introduction p{margin:0;color:var(--muted);max-width:70ch}
.people-layout{display:grid;grid-template-columns:minmax(0,1.1fr) minmax(0,1fr);gap:24px;align-items:start}
.people-section h3{margin:0;padding:20px 22px 8px;font-size:17px;line-height:1.4}
.people-count{margin:0;padding:0 22px 18px;color:var(--muted);font-size:13px}
.person-inspector,.employment-inspector{min-width:0;border-top:1px solid var(--line);scroll-margin-top:24px}
.person-summary,.employment-summary{position:relative;display:flex;align-items:center;flex-wrap:wrap;gap:12px;
min-height:76px;padding:16px 20px;cursor:pointer;list-style:none}
.person-summary::-webkit-details-marker,.employment-summary::-webkit-details-marker{display:none}
.person-summary:hover,.employment-summary:hover{background:#fafbf8}
.person-summary::after,.employment-summary::after{content:'⌄';font-size:18px;color:var(--muted)}
.person-inspector[open] .person-summary::after,.employment-inspector[open] .employment-summary::after{content:'⌃'}
.person-inspector[open],.employment-inspector[open]{background:#fcfdfb}
.person-glyph{display:grid;place-items:center;flex:none;width:38px;height:38px;border-radius:12px;background:var(--accent-bg);color:#75521b;font-size:16px;font-weight:650}
.person-heading,.employment-heading{display:flex;flex:1 1 110px;min-width:0;flex-direction:column;gap:3px}
.person-secondary,.employment-heading span{font-size:14px;color:var(--muted)}
.person-revision{font-size:13px;color:var(--muted);font-variant-numeric:tabular-nums}
.person-disclosure{font-size:12px;color:var(--muted)}
.person-detail,.employment-detail{padding:4px 22px 22px}
.people-facts{display:grid;grid-template-columns:minmax(80px,.65fr) minmax(0,1.5fr);column-gap:16px;row-gap:12px;margin:12px 0 20px;font-size:14px}
.people-facts dt{color:var(--muted)}.people-facts dd{margin:0;overflow-wrap:anywhere;min-width:0}
.people-identity{font-variant-numeric:tabular-nums;font-size:13px}
.person-relationships{margin:18px 0;padding-top:16px;border-top:1px solid var(--line)}
.person-relationships h4{margin:0 0 8px;font-size:14px}.person-relationships ul{margin:0;padding-left:20px}
.person-relationships li{padding:6px 0}
.people-source,.people-retry{display:inline-block;padding:10px 0;min-height:44px;font-size:14px;font-weight:600}
.people-retry{margin:0 22px 18px}
@media(max-width:1150px){.people-layout{grid-template-columns:minmax(0,1fr);gap:0}}
@media(max-width:680px){.person-summary,.employment-summary{padding:16px}.person-detail,.employment-detail{padding:0 16px 16px}
.people-section h3{padding:18px 16px 8px}.people-count{padding:0 16px 16px}.people-facts{grid-template-columns:minmax(70px,.7fr) minmax(0,1.3fr);gap:10px}}
.workflow-layout{display:grid;grid-template-columns:minmax(0,1fr) 300px;gap:24px;align-items:start}
.workflow-main,.workflow-guide{min-width:0}.workflow-guide .panel{background:#fdfefa}
.section-intro{margin:0;padding:18px 22px;color:var(--muted);font-size:14px}
.record-list{margin:0;padding:0 22px 0 42px}
.record-card{padding:20px 0;border-bottom:1px solid var(--line)}.record-card:last-child{border-bottom:0}
.record-card::marker{color:var(--muted);font-variant-numeric:tabular-nums}
.record-title{margin:0 0 12px;font-size:16px;line-height:1.5}.record-title a{color:var(--ink)}
.record-meta{display:grid;grid-template-columns:90px minmax(0,1fr);gap:8px 16px;margin:0;font-size:14px}
.record-meta dt{color:var(--muted)}.record-meta dd{margin:0;min-width:0;overflow-wrap:anywhere}
.record-id,.record-revision{font-variant-numeric:tabular-nums}
.recovery-link{display:inline-block;min-height:44px;margin:0 22px 18px;padding:10px 0;font-weight:600;font-size:14px}
.hierarchy-note{margin:8px 22px 0;font-size:16px}.record-card .hierarchy-note{margin:0 0 14px;font-size:14px;color:var(--muted)}
.conflict-note{margin:0 0 12px;padding:8px 12px;border-left:3px solid #b4681d;background:var(--flag);color:var(--flag-ink);font-size:14px}
.guidance-list{margin:0;padding:10px 22px 22px 42px;color:var(--muted);font-size:14px}.guidance-list li{padding:6px 0}
.process-guide{display:grid;grid-template-columns:repeat(auto-fit,minmax(130px,1fr));gap:12px 28px;margin:0;padding:4px 22px 22px 42px;font-size:14px;font-weight:600}
.process-guide li{padding:10px 8px}.process-guide li::marker{color:var(--muted);font-variant-numeric:tabular-nums}
.review-notice{margin:0 22px;padding:14px;border:1px solid #e5cc92;border-radius:8px;background:var(--accent-bg);color:#62410b;font-size:14px;font-weight:600}
.person-detail,.employment-detail{scroll-margin-top:24px}
@media(max-width:1150px){.workflow-layout{grid-template-columns:minmax(0,1fr);gap:0}.workflow-guide{display:grid;grid-template-columns:repeat(auto-fit,minmax(240px,1fr));gap:0 24px}}
@media(max-width:680px){.section-intro{padding:16px}.record-list{padding:0 16px 0 36px}.record-meta{grid-template-columns:72px minmax(0,1fr);gap:8px 12px}
.guidance-list{padding:8px 16px 20px 36px}.process-guide{padding:0 16px 20px 36px}.review-notice{margin:0 16px}.workflow-guide{display:block}}
@media(min-width:1600px){main{padding-top:48px}}
@media(max-width:1000px){.workspace{grid-template-columns:188px minmax(0,1fr)}main{padding:28px 24px 64px}}
@media(max-width:680px){.workspace{display:block}header.app{position:static;height:auto;padding:16px;border-right:0;border-bottom:1px solid var(--line)}
.brand{padding:0 4px 12px;font-size:18px}.brand-mark{width:28px;height:28px}.nav-group{margin:0 4px 6px}
header.app nav{flex-direction:row;flex-wrap:wrap;gap:4px}header.app nav a{padding:8px 12px;min-height:44px}
main{padding:24px 16px 56px}.page-heading{margin-bottom:22px;padding-bottom:20px}h1{font-size:25px}
.panel h2{padding:16px}.toolbar{padding:12px 16px}.row{flex-wrap:wrap;gap:8px 12px;padding:16px}
.row .meta{flex:1 0 100%;order:3}.badge{margin-left:0}.state{padding:24px 16px}}
@media(prefers-reduced-motion:reduce){html{scroll-behavior:auto}}
"#,
    include_str!("native_policy.css"),
    include_str!("native_people.css"),
    include_str!("native_payroll.css")
);

const ISLAND_BOOTSTRAP: &str = concat!(
    include_str!("island_script.js"),
    "(\"\", \"pkg\", \"console_payroll_ui\", \"console_payroll_ui_bg\");"
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

/// Contract-shaped Company Head. Field names match OpenAPI `Company` required
/// keys; values are already-authorized. Same DTO as `GET /api/v1/companies`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompanyView {
    pub org_id: String,
    pub legal_name: String,
    pub reg_no: String,
    pub version: String,
}

/// Contract-shaped OrgUnit Head. Field names match OpenAPI `OrgUnit` required
/// keys; values are already-authorized. Same DTO as `GET /api/v1/org-units`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrgUnitView {
    pub id: String,
    pub name: String,
    pub parent_id: String,
    pub version: String,
}

/// Contract-shaped Person Head. Closed four-field projection matching OpenAPI
/// `Person`; values are already-authorized. Same DTO as `GET /api/v1/persons`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonView {
    pub id: String,
    pub display_name: String,
    pub legal_name: String,
    pub version: String,
}

/// Contract-shaped Employment Head. Field names match OpenAPI `Employment`
/// required keys; values are already-authorized. Same DTO as
/// `GET /api/v1/employments`. FKs stay ids (no invented JobPosition href).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmploymentView {
    pub id: String,
    pub version: String,
    pub appointed_on: String,
    pub person_id: String,
    pub org_unit_id: String,
    pub job_position_id: String,
}

/// One shipping-screen listing after server composition.
///
/// `Omitted` is deny-by-omission (unauth / forbidden). `Empty` is an authorized
/// listing that returned zero rows. `Failure` is an authorized listing that
/// failed. Unauthorized must never become `Empty` or `Failure`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ScreenSection<T> {
    #[default]
    Omitted,
    Empty,
    Failure,
    Rows(Vec<T>),
}

impl<T> ScreenSection<T> {
    /// Map an already-authorized listing. Non-empty rows always win; empty rows
    /// are `Empty` only when the same listing floor would have allowed the GET.
    #[must_use]
    pub fn from_authorized_listing(rows: Vec<T>, listing_authorized: bool) -> Self {
        if !rows.is_empty() {
            Self::Rows(rows)
        } else if listing_authorized {
            Self::Empty
        } else {
            Self::Omitted
        }
    }

    #[must_use]
    pub const fn is_offered(&self) -> bool {
        !matches!(self, Self::Omitted)
    }
}

/// Server-composed shipping screens. `Omitted` is omit, not a client decision.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ShippingScreens {
    pub companies: ScreenSection<CompanyView>,
    pub org_units: ScreenSection<OrgUnitView>,
    pub people: ScreenSection<PersonView>,
    pub employments: ScreenSection<EmploymentView>,
    pub runs: ScreenSection<RunSummary>,
}

/// Which shipping-UI body to render. Nav still only names authorized non-empty screens.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiScreen {
    Home,
    Organization,
    Hr,
    Payroll,
}

/// Presentation-only revision chip. The raw value stays in `data-version`;
/// this is the label a reader sees.
fn revision_label(version: &str) -> String {
    if version.is_empty() {
        String::new()
    } else {
        format!("v{version}")
    }
}

/// Calendar day of an effective-dated value. Employment carries an RFC 3339
/// instant, but a reader of an appointment wants the day, not the zero clock
/// time. Presentation only -- `data-appointed-on` keeps the exact value.
fn day_of(value: &str) -> String {
    value
        .split_once('T')
        .map_or_else(|| value.to_owned(), |(day, _)| day.to_owned())
}

#[component]
pub fn Shell() -> impl IntoView {
    view! {
        <html lang="ko">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <title>"Console"</title>
                <style>{STYLE}</style>
            </head>
            <body></body>
        </html>
    }
}

/// The one hydrated subtree on this surface.
///
/// Everything else the shell renders is static SSR. This island carries the
/// only client behavior: a status filter over the runs the server already
/// authorized and serialized into `data-props`. It is presentation only --
/// it changes which authorized rows are *visible*, never which rows exist. No
/// fetch, no server function, and no business rule runs here, so
/// deny-by-omission stays a server decision and the client gains nothing it
/// was not already sent.
#[island]
pub fn AuthorizedRuns(runs: Vec<RunSummary>) -> impl IntoView {
    // Empty means "전체". The options come from the rows in hand, so the
    // control can never name a status this actor was not sent.
    let selected = RwSignal::new(String::new());
    let mut statuses: Vec<String> = runs.iter().map(|run| run.status.clone()).collect();
    statuses.sort_unstable();
    statuses.dedup();
    view! {
        <div class="toolbar">
            <label class="fl" for="run-status">"상태"</label>
            <select
                id="run-status"
                data-run-status-filter=""
                on:change:target=move |ev| selected.set(ev.target().value())
            >
                <option value="">"전체"</option>
                {statuses
                    .into_iter()
                    .map(|status| {
                        let label = status.clone();
                        view! { <option value=status>{label}</option> }
                    })
                    .collect_view()}
            </select>
        </div>
        {runs
            .into_iter()
            .map(|run| {
                let href = format!("/api/v1/payroll/runs/{}", run.id);
                let period = format!("{}–{}", run.period_start, run.period_end);
                let source = run.source_label.clone();
                let badge = run.status.clone();
                let badge_key = run.status.clone();
                // Visibility only. SSR selects nothing, so every authorized row
                // is served unhidden and a client that never hydrates still
                // sees the whole authorized listing.
                let status = run.status.clone();
                let filtered_out = move || {
                    let selected = selected.get();
                    !selected.is_empty() && selected != status
                };
                view! {
                    <a class="row" href=href hidden=filtered_out>
                        <span
                            class="name"
                            data-run-id=run.id
                            data-period-start=run.period_start
                            data-period-end=run.period_end
                            data-source-label=run.source_label
                            data-status=run.status
                            data-calculation-enabled=run.calculation_enabled.to_string()
                            data-created-at=run.created_at
                            data-updated-at=run.updated_at
                        >
                            {period}
                        </span>
                        <span class="meta">{source}</span>
                        <span class="badge" data-status=badge_key>{badge}</span>
                    </a>
                }
            })
            .collect_view()}
    }
}

#[component]
fn ShippingNav(has_org: bool, has_hr: bool, has_payroll: bool, focus: UiScreen) -> impl IntoView {
    view! {
        <header class="app">
            <a class="brand" href="/"><span class="brand-mark" aria-hidden="true">"C"</span>"Console"</a>
            <p class="nav-group">"사람과 조직"</p>
            <nav aria-label="주요 탐색">
                {has_hr.then(|| {
                    let current = matches!(focus, UiScreen::Hr).then_some("page");
                    view! { <a href="/hr" aria-current=current><span class="nav-symbol" aria-hidden="true">"◎"</span>"사람과 고용"</a> }
                })}
                {has_org.then(|| {
                    let current = matches!(focus, UiScreen::Organization).then_some("page");
                    view! { <a href="/organization" aria-current=current><span class="nav-symbol" aria-hidden="true">"⊞"</span>"조직"</a> }
                })}
                {has_payroll.then(|| {
                    let current = matches!(focus, UiScreen::Payroll).then_some("page");
                    view! { <a href="/payroll" aria-current=current><span class="nav-symbol" aria-hidden="true">"▤"</span>"급여"</a> }
                })}
            </nav>
        </header>
    }
}

#[component]
pub fn AuthorizedShell(runs: Vec<RunSummary>) -> impl IntoView {
    let nav_payroll = !runs.is_empty();
    view! {
        <ShippingShell
            companies=ScreenSection::Omitted
            org_units=ScreenSection::Omitted
            people=ScreenSection::Omitted
            employments=ScreenSection::Omitted
            runs=ScreenSection::from_authorized_listing(runs, false)
            nav_org=false
            nav_hr=false
            nav_payroll=nav_payroll
            focus=UiScreen::Home
        />
    }
}

fn org_body(
    companies: ScreenSection<CompanyView>,
    org_units: ScreenSection<OrgUnitView>,
) -> impl IntoView {
    organization::body(companies, org_units)
}

fn hr_body(
    people: ScreenSection<PersonView>,
    employments: ScreenSection<EmploymentView>,
) -> impl IntoView {
    people::body(people, employments)
}

fn payroll_body(runs: ScreenSection<RunSummary>) -> impl IntoView {
    payroll_workspace::body(runs)
}

#[component]
fn ShippingShell(
    companies: ScreenSection<CompanyView>,
    org_units: ScreenSection<OrgUnitView>,
    people: ScreenSection<PersonView>,
    employments: ScreenSection<EmploymentView>,
    runs: ScreenSection<RunSummary>,
    nav_org: bool,
    nav_hr: bool,
    nav_payroll: bool,
    focus: UiScreen,
) -> impl IntoView {
    let hydrate_payroll = matches!(&runs, ScreenSection::Rows(rows) if !rows.is_empty());
    let (subject, description) = match focus {
        UiScreen::Home => (
            "업무 공간",
            "현재 접근할 수 있는 사람, 조직, 급여 기록을 확인하고 업무를 이어가세요.",
        ),
        UiScreen::Organization => (
            "조직",
            "법인과 조직 구조를 확인하세요. 서로 연결된 조직의 관계와 기록을 함께 살펴볼 수 있습니다.",
        ),
        UiScreen::Hr => (
            "사람과 고용",
            "사람의 기록과 연결된 고용 이력을 함께 확인하세요.",
        ),
        UiScreen::Payroll => ("급여", "급여 기간별 실행 기록과 검토할 자료를 확인하세요."),
    };
    view! {
        <html lang="ko">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <title>{format!("{subject} · Console")}</title>
                <style>{STYLE}</style>
                {hydrate_payroll
                    .then(|| {
                        view! {
                            <link rel="modulepreload" href=PKG_JS />
                            // `crossorigin` is not decoration. Without it the
                            // preload's credentials mode does not match the
                            // module's own fetch, so the browser discards the
                            // preload and the client downloads the whole
                            // bundle twice.
                            <link
                                rel="preload"
                                href=PKG_WASM
                                r#as="fetch"
                                r#type="application/wasm"
                                crossorigin="anonymous"
                            />
                            <script type="module">{ISLAND_BOOTSTRAP}</script>
                        }
                    })}
            </head>
            <body class="workspace">
                <a class="skip-link" href="#main-content">"본문으로 건너뛰기"</a>
                <ShippingNav
                    has_org=nav_org
                    has_hr=nav_hr
                    has_payroll=nav_payroll
                    focus=focus
                />
                <main id="main-content" tabindex="-1">
                    <div class="page-heading">
                        <p class="page-eyebrow">"CONSOLE WORKSPACE"</p>
                        <h1>{subject}</h1>
                        <p class="page-description">{description}</p>
                    </div>
                    {org_body(companies, org_units)}
                    {hr_body(people, employments)}
                    {payroll_body(runs)}
                </main>
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
    render_screens(
        &ShippingScreens {
            runs: ScreenSection::from_authorized_listing(runs.to_vec(), false),
            ..ShippingScreens::default()
        },
        UiScreen::Home,
    )
}

/// SSR compose org / HR / payroll. `Omitted` is deny-by-omission.
/// Nav names every offered screen; the body is the focused route.
pub fn render_screens(screens: &ShippingScreens, focus: UiScreen) -> String {
    let nav_org = screens.companies.is_offered() || screens.org_units.is_offered();
    let nav_hr = screens.people.is_offered() || screens.employments.is_offered();
    let nav_payroll = screens.runs.is_offered();
    let companies = match focus {
        UiScreen::Home | UiScreen::Organization => screens.companies.clone(),
        UiScreen::Hr | UiScreen::Payroll => ScreenSection::Omitted,
    };
    let org_units = match focus {
        UiScreen::Home | UiScreen::Organization => screens.org_units.clone(),
        UiScreen::Hr | UiScreen::Payroll => ScreenSection::Omitted,
    };
    let people = match focus {
        UiScreen::Home | UiScreen::Hr => screens.people.clone(),
        UiScreen::Organization | UiScreen::Payroll => ScreenSection::Omitted,
    };
    let employments = match focus {
        UiScreen::Home | UiScreen::Hr => screens.employments.clone(),
        UiScreen::Organization | UiScreen::Payroll => ScreenSection::Omitted,
    };
    let runs = match focus {
        UiScreen::Home | UiScreen::Payroll => screens.runs.clone(),
        UiScreen::Organization | UiScreen::Hr => ScreenSection::Omitted,
    };
    let focus_denied = match focus {
        UiScreen::Home => !nav_org && !nav_hr && !nav_payroll,
        UiScreen::Organization => !nav_org,
        UiScreen::Hr => !nav_hr,
        UiScreen::Payroll => !nav_payroll,
    };
    if focus_denied {
        return render_shell();
    }
    let mut html = String::from("<!DOCTYPE html>");
    html.push_str(
        &view! {
            <ShippingShell
                companies=companies
                org_units=org_units
                people=people
                employments=employments
                runs=runs
                nav_org=nav_org
                nav_hr=nav_hr
                nav_payroll=nav_payroll
                focus=focus
            />
        }
        .to_html(),
    );
    html
}

#[cfg(feature = "ssr")]
mod ssr {
    use super::{
        RunSummary, ShippingScreens, UiScreen, render_screens, render_shell, render_shell_with,
    };
    use axum::Router;
    use axum::http::{HeaderValue, StatusCode, header};
    use axum::response::{Html, IntoResponse, Response};
    use axum::routing::get;

    const SHELL_CSP: &str = "object-src 'none'; base-uri 'none'; frame-ancestors 'none'";

    pub(super) fn private_document(
        html: String,
        status: StatusCode,
        csp: &'static str,
    ) -> Response {
        let mut response = (status, Html(html)).into_response();
        for (name, value) in [
            (header::CACHE_CONTROL, "no-store"),
            (header::PRAGMA, "no-cache"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
            (header::VARY, "Authorization, Cookie, Origin"),
            (header::REFERRER_POLICY, "no-referrer"),
            (header::CONTENT_SECURITY_POLICY, csp),
        ] {
            response
                .headers_mut()
                .insert(name, HeaderValue::from_static(value));
        }
        response
    }

    pub fn html_shell() -> Response {
        private_document(render_shell(), StatusCode::OK, SHELL_CSP)
    }

    pub fn html_shell_with(runs: &[RunSummary]) -> Response {
        private_document(render_shell_with(runs), StatusCode::OK, SHELL_CSP)
    }

    pub fn html_shell_with_screens(screens: &ShippingScreens, focus: UiScreen) -> Response {
        private_document(render_screens(screens, focus), StatusCode::OK, SHELL_CSP)
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

    async fn native_account_js() -> impl IntoResponse {
        (
            [
                (header::CONTENT_TYPE, "text/javascript; charset=utf-8"),
                (header::CACHE_CONTROL, "no-store"),
                (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
            ],
            include_str!("native_account.js"),
        )
    }

    async fn workspace_css() -> impl IntoResponse {
        (
            [
                (header::CONTENT_TYPE, "text/css; charset=utf-8"),
                (header::CACHE_CONTROL, "no-store"),
                (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
            ],
            super::STYLE,
        )
    }

    async fn native_account_css() -> impl IntoResponse {
        (
            [
                (header::CONTENT_TYPE, "text/css; charset=utf-8"),
                (header::CACHE_CONTROL, "no-store"),
                (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
            ],
            concat!(
                include_str!("theme.css"),
                include_str!("native_account.css")
            ),
        )
    }

    pub fn pkg_router<S>() -> Router<S>
    where
        S: Clone + Send + Sync + 'static,
    {
        Router::new()
            .route("/assets/native-account.js", get(native_account_js))
            .route("/assets/native-account.css", get(native_account_css))
            .route("/assets/workspace.css", get(workspace_css))
            .route("/pkg/console_payroll_ui.js", get(pkg_js))
            .route("/pkg/console_payroll_ui_bg.wasm", get(pkg_wasm))
    }

    #[cfg(test)]
    #[allow(clippy::unwrap_used, clippy::panic)]
    mod theme_tests {
        use super::*;
        use std::future::Future;
        use std::task::{Context, Poll, Waker};

        fn ready<F: Future>(future: F) -> F::Output {
            let mut future = std::pin::pin!(future);
            match future
                .as_mut()
                .poll(&mut Context::from_waker(Waker::noop()))
            {
                Poll::Ready(value) => value,
                Poll::Pending => panic!("static CSS response unexpectedly needs asynchronous I/O"),
            }
        }

        fn definition<'a>(css: &'a str, property: &str) -> &'a str {
            let prefix = format!("{property}:");
            let values: Vec<_> = css
                .split(['{', '}', ';'])
                .filter_map(|part| part.trim().strip_prefix(&prefix))
                .map(str::trim)
                .collect();
            assert_eq!(
                values.len(),
                1,
                "theme property {property} must have one definition"
            );
            values[0]
        }

        #[test]
        fn native_account_asset_and_workspace_share_one_light_theme() {
            let response = ready(native_account_css()).into_response();
            assert_eq!(response.status(), axum::http::StatusCode::OK);
            assert_eq!(
                response.headers()[header::CONTENT_TYPE],
                "text/css; charset=utf-8"
            );
            assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
            assert_eq!(
                response.headers()[header::X_CONTENT_TYPE_OPTIONS],
                "nosniff"
            );
            let bytes = ready(axum::body::to_bytes(response.into_body(), 128 * 1024)).unwrap();
            let account = std::str::from_utf8(&bytes).unwrap();
            let document = crate::render_shell();
            let workspace = document
                .split_once("<style>")
                .unwrap()
                .1
                .split_once("</style>")
                .unwrap()
                .0;
            for property in [
                "--bg",
                "--surface",
                "--ink",
                "--muted",
                "--accent",
                "--focus",
                "--font-sans",
            ] {
                assert_eq!(
                    definition(account, property),
                    definition(workspace, property),
                    "Account and workspace disagree on {property}"
                );
            }
            for css in [account, workspace] {
                assert!(css.contains("color-scheme:light"));
                assert!(
                    !css.contains("@import"),
                    "theme must not fetch external fonts"
                );
            }
            assert!(
                !document.contains("/pkg/"),
                "public shell must remain unhydrated"
            );
        }
    }
}

#[cfg(feature = "ssr")]
pub use ssr::{
    html_shell, html_shell_with, html_shell_with_screens, payroll_ui_js, payroll_ui_wasm,
    pkg_router,
};

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    leptos::mount::hydrate_islands();
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    const PAYROLL_RUN_SUMMARY_SCHEMA: &str =
        include_str!("../../rest/openapi/schemas/PayrollRunSummary.yaml");
    const OPENAPI: &str = include_str!("../../../../openapi/openapi.yaml");

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

    fn yaml_schema_required_keys<'a>(doc: &'a str, schema: &str) -> Vec<&'a str> {
        let header = format!("    {schema}:\n");
        let rest = doc
            .split_once(&header)
            .unwrap_or_else(|| panic!("OpenAPI must declare schema {schema}"))
            .1;
        yaml_required_keys(rest)
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
            !render_shell().contains("/pkg/"),
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
            html.contains("(\"\", \"pkg\", \"console_payroll_ui\", \"console_payroll_ui_bg\")"),
            "authorized markup must invoke leptos island_script: {html}"
        );
        assert!(
            html.contains("href=\"/api/v1/payroll/runs/00000000-0000-0000-0000-000000000001\""),
            "payroll drill-through must use the existing run GET: {html}"
        );
        // Anchored on `>text<` so this proves the value is rendered content,
        // not merely a data-* attribute. Stronger than matching a concatenated
        // string, and independent of how the row lays the two values out.
        assert!(
            html.contains(">2026-06-01–2026-06-30<")
                && html.contains(">workflow_runtime_m2:run:example<"),
            "payroll drill-through must show human-safe period and source_label: {html}"
        );
        assert!(
            !html.contains("/api/v1/employees/") && !html.contains("/api/v1/users/"),
            "must not drill through privileged employee/user GET: {html}"
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
        // Close the binding: SSR -> JS -> wasm. Without this a fresh bindgen JS
        // paired with a stale wasm passes, because the JS carries the export
        // name and the wasm was only checked for its magic bytes.
        let needle = component.as_bytes();
        assert!(
            payroll_ui_wasm().windows(needle.len()).any(|w| w == needle),
            "committed wasm must contain the {component} export the SSR page names"
        );
        assert!(
            !render_shell().contains("AuthorizedRuns"),
            "empty shell must not emit an island: {}",
            render_shell()
        );
    }

    /// The island must own a presentation-only status filter.
    ///
    /// This is the only client behavior on the surface, and it is what makes
    /// selective hydration observable: without it the island re-renders markup
    /// identical to SSR and a broken bundle is indistinguishable from a working
    /// one. The filter changes visibility of rows the server already
    /// authorized; it never adds, removes, or requests a row, so deny-by-
    /// omission stays a server decision.
    #[test]
    fn authorized_runs_island_owns_a_presentation_only_status_filter() {
        let runs = vec![
            run_with("00000000-0000-0000-0000-000000000001", "DRAFT"),
            run_with("00000000-0000-0000-0000-000000000002", "BLOCKED_LEGAL_GATE"),
            run_with("00000000-0000-0000-0000-000000000003", "DRAFT"),
        ];
        let html = render_shell_with(&runs);
        let island = island_subtree(&html).unwrap_or("");
        assert!(
            !island.is_empty(),
            "authorized runs must be an island: {html}"
        );

        // Inside the island subtree, so hydration owns the control. A filter
        // rendered in the static shell would never receive an event listener.
        assert!(
            island.contains("data-run-status-filter"),
            "island must carry the status filter: {island}"
        );

        // Options are exactly the distinct statuses of the authorized rows plus
        // the all-option: no fixed enum, no status this actor was not sent.
        assert!(
            island.contains(r#"<option value="">"#),
            "filter must offer an all-option: {island}"
        );
        for status in ["DRAFT", "BLOCKED_LEGAL_GATE"] {
            assert!(
                island.contains(&format!(r#"<option value="{status}">"#)),
                "filter must offer authorized status {status}: {island}"
            );
        }
        assert_eq!(
            island.matches("<option").count(),
            3,
            "filter must offer the all-option and each distinct status once: {island}"
        );
        assert!(
            !island.contains("PAYABLE") && !island.contains("APPROVED"),
            "filter must not invent a status the actor was not sent: {island}"
        );

        // Every authorized row is served, none pre-hidden. Filtering is client
        // visibility over server-composed membership.
        assert_eq!(
            island.matches("data-run-id=").count(),
            3,
            "server must send every authorized row unfiltered: {island}"
        );
        // Scoped to each anchor's open tag: that keeps the serialized
        // `data-props` out of range -- a run whose `source_label` merely
        // contained the word must not fail this -- while keeping the form
        // Leptos actually emits in range. It renders a boolean attribute bare
        // (` hidden`, never ` hidden="..."`) and puts `class` last, so matching
        // on ` hidden=` or ` hidden>` would be a check that cannot fail.
        // Counted first so the loop cannot pass vacuously. The row count
        // asserted above does not imply an anchor exists -- rewriting the row
        // element would keep `data-run-id` at three and silently zero this.
        let anchors: Vec<&str> = island.split("<a ").skip(1).collect();
        assert_eq!(
            anchors.len(),
            3,
            "every authorized row must be an anchor: {island}"
        );
        for tag in anchors {
            let open = tag.split_once('>').map_or(tag, |(open, _)| open);
            assert!(
                !open.contains(" hidden"),
                "SSR must not pre-hide an authorized row: {island}"
            );
        }
        // The filter's only effect. `.row` sets `display:flex` and both
        // selectors have specificity (0,1,0), so without `!important` the
        // later rule wins on source order and the `hidden` attribute is inert:
        // the island hydrates, the control responds, and nothing disappears.
        // No Rust test can observe that symptom, so pin the rule itself.
        assert!(
            STYLE.contains("[hidden]{display:none!important}"),
            "the status filter works by toggling `hidden`; this rule is what makes it visible"
        );

        // The unauthorized shell gains nothing.
        assert!(
            !render_shell().contains("data-run-status-filter"),
            "empty shell must not carry the filter: {}",
            render_shell()
        );
    }

    fn run_with(id: &str, status: &str) -> RunSummary {
        RunSummary {
            id: id.to_owned(),
            status: status.to_owned(),
            ..sample_run()
        }
    }

    /// The `<leptos-island>` subtree only. Everything else on the page is static
    /// shell that never hydrates.
    fn island_subtree(html: &str) -> Option<&str> {
        let (_, rest) = html.split_once("<leptos-island ")?;
        rest.split_once("</leptos-island>").map(|(inner, _)| inner)
    }

    fn island_component(html: &str) -> Option<&str> {
        html.split_once("data-component=\"")
            .and_then(|(_, rest)| rest.split_once('"').map(|(id, _)| id))
    }

    fn data_attr(key: &str, id_alias: &str) -> String {
        if key == "id" {
            return format!("data-{id_alias}");
        }
        let mut out = String::from("data-");
        for (i, ch) in key.chars().enumerate() {
            if ch == '_' {
                out.push('-');
            } else if ch.is_ascii_uppercase() {
                if i > 0 {
                    out.push('-');
                }
                out.push(ch.to_ascii_lowercase());
            } else {
                out.push(ch);
            }
        }
        out
    }

    fn sample_company() -> CompanyView {
        CompanyView {
            org_id: "00000000-0000-0000-0000-0000000000aa".to_owned(),
            legal_name: "KNL".to_owned(),
            reg_no: "110111-0000000".to_owned(),
            version: "1".to_owned(),
        }
    }

    fn sample_org_unit() -> OrgUnitView {
        OrgUnitView {
            id: "00000000-0000-0000-0000-0000000000dd".to_owned(),
            name: "본사".to_owned(),
            parent_id: String::new(),
            version: "1".to_owned(),
        }
    }

    fn sample_person() -> PersonView {
        PersonView {
            id: "00000000-0000-0000-0000-0000000000bb".to_owned(),
            display_name: "홍길동".to_owned(),
            legal_name: "홍길동".to_owned(),
            version: "1".to_owned(),
        }
    }

    fn sample_employment() -> EmploymentView {
        EmploymentView {
            id: "00000000-0000-0000-0000-0000000000ee".to_owned(),
            version: "1".to_owned(),
            appointed_on: "2026-01-15T00:00:00Z".to_owned(),
            person_id: "00000000-0000-0000-0000-0000000000bb".to_owned(),
            org_unit_id: "00000000-0000-0000-0000-0000000000dd".to_owned(),
            job_position_id: String::new(),
        }
    }

    fn assert_not_directory_person(html: &str) {
        assert!(
            !html.contains("data-employee-")
                && !html.contains("data-slug")
                && !html.contains("data-account-status")
                && !html.contains("data-has-passkey")
                && !html.contains("data-branch-ids")
                && !html.contains("data-is-active")
                && !html.contains("employee_identity"),
            "Person Head must stay the closed four-field projection: {html}"
        );
    }

    fn assert_shipping_invariants(html: &str) {
        let lowered = html.to_ascii_lowercase();
        assert!(!lowered.contains("won"), "won leaked: {html}");
        assert!(!html.contains("291_520"), "golden won leaked: {html}");
        assert!(!lowered.contains("payslip"), "payslip leaked: {html}");
        assert!(
            !lowered.contains("group-switcher") && !lowered.contains("data-group-switch"),
            "Group switcher is not admitted unless it only displays authorized scope: {html}"
        );
        assert!(
            !lowered.contains("comms-rail") && !html.contains("data-comms"),
            "comms rail is out of this slice: {html}"
        );
        assert!(
            !html.contains("type=\"file\"")
                && !lowered.contains("import/export")
                && !html.contains("자료실"),
            "import/export is not the data-entry base: {html}"
        );
        assert!(
            !html.contains("webpack") && !html.contains("vite") && !html.contains("innerHTML"),
            "must stay Rust-native Leptos SSR, not a JS wrapper: {html}"
        );
        assert!(
            !html.contains("/api/v1/job-positions/"),
            "must not invent JobPosition routes: {html}"
        );
        assert!(
            !html.contains("type=\"date\"")
                && !html.contains("name=\"as_of\"")
                && !html.contains("name=\"from\"")
                && !html.contains("name=\"to\""),
            "current-slice directory must not invent a temporal picker: {html}"
        );
    }

    #[test]
    fn shipping_screens_empty_matches_unauthenticated_shell() {
        let empty = ShippingScreens::default();
        assert_eq!(render_screens(&empty, UiScreen::Home), render_shell());
        assert_eq!(
            render_screens(&empty, UiScreen::Organization),
            render_shell()
        );
        assert_eq!(render_screens(&empty, UiScreen::Hr), render_shell());
        assert_eq!(render_screens(&empty, UiScreen::Payroll), render_shell());
        assert!(
            !render_shell().contains("data-screen="),
            "empty shell must omit screen markup: {}",
            render_shell()
        );
        assert!(
            !render_shell().contains("href=\"/"),
            "empty shell must omit nav: {}",
            render_shell()
        );
        assert_shipping_invariants(&render_shell());

        let authorized_empty = ShippingScreens {
            companies: ScreenSection::Empty,
            org_units: ScreenSection::Empty,
            people: ScreenSection::Empty,
            employments: ScreenSection::Empty,
            runs: ScreenSection::Empty,
        };
        let empty_org = render_screens(&authorized_empty, UiScreen::Organization);
        assert_ne!(
            empty_org,
            render_shell(),
            "authorized-empty org must not reuse the silent deny shell"
        );
        assert!(
            empty_org.contains("data-screen=\"organization\"")
                && empty_org.contains("data-state=\"empty\"")
                && empty_org.contains("표시할 조직이 없습니다"),
            "authorized-empty org must mount Korean empty copy: {empty_org}"
        );
        assert!(
            !empty_org.contains("data-org-id"),
            "authorized-empty must not leak rows: {empty_org}"
        );
        assert_shipping_invariants(&empty_org);

        let empty_pay = render_screens(&authorized_empty, UiScreen::Payroll);
        assert_ne!(empty_pay, render_shell(), "{empty_pay}");
        assert!(
            empty_pay.contains("data-screen=\"payroll\"")
                && empty_pay.contains("data-state=\"empty\"")
                && empty_pay.contains("표시할 급여 이력이 없습니다")
                && !empty_pay.contains("/pkg/")
                && !empty_pay.contains("data-run-id"),
            "authorized-empty payroll is SSR empty copy, not omit and not WASM: {empty_pay}"
        );
        assert_eq!(
            render_screens(&ShippingScreens::default(), UiScreen::Payroll),
            render_shell(),
            "unauthorized payroll must stay deny-by-omission"
        );

        let failed = ShippingScreens {
            companies: ScreenSection::Failure,
            org_units: ScreenSection::Omitted,
            people: ScreenSection::Omitted,
            employments: ScreenSection::Omitted,
            runs: ScreenSection::Failure,
        };
        let fail_html = render_screens(&failed, UiScreen::Payroll);
        assert!(
            fail_html.contains("data-screen=\"payroll\"")
                && fail_html.contains("data-state=\"failure\"")
                && fail_html.contains("목록을 불러오지 못했습니다")
                && !fail_html.contains("data-run-id")
                && !fail_html.contains("/pkg/"),
            "listing failure marks the section without object ids: {fail_html}"
        );
        assert_shipping_invariants(&fail_html);
    }

    #[test]
    fn shipping_screens_organization_is_ssr_contracts_and_omits_wasm() {
        let screens = ShippingScreens {
            companies: ScreenSection::Rows(vec![sample_company()]),
            org_units: ScreenSection::Rows(vec![sample_org_unit()]),
            ..ShippingScreens::default()
        };
        let html = render_screens(&screens, UiScreen::Organization);
        assert_ne!(html, render_shell(), "{html}");
        assert!(
            html.contains("data-screen=\"organization\""),
            "organization body must be a mounted SSR screen: {html}"
        );
        for key in yaml_schema_required_keys(OPENAPI, "Company") {
            let attr = data_attr(key, "org-id");
            assert!(
                html.contains(&attr),
                "org markup must carry Company Head key {key} as {attr}: {html}"
            );
        }
        for key in yaml_schema_required_keys(OPENAPI, "OrgUnit") {
            let attr = data_attr(key, "org-unit-id");
            assert!(
                html.contains(&attr),
                "org markup must carry OrgUnit Head key {key} as {attr}: {html}"
            );
        }
        assert!(
            html.contains("data-org-id=\"00000000-0000-0000-0000-0000000000aa\""),
            "{html}"
        );
        assert!(
            html.contains("data-org-unit-id=\"00000000-0000-0000-0000-0000000000dd\""),
            "{html}"
        );
        assert!(
            html.contains("KNL") && html.contains("본사"),
            "org row must show human-safe Company legal_name and OrgUnit name: {html}"
        );
        assert!(
            html.contains("href=\"/api/v1/companies/00000000-0000-0000-0000-0000000000aa\"")
                && html.contains("href=\"/api/v1/org-units/00000000-0000-0000-0000-0000000000dd\""),
            "org must drill through published Company/OrgUnit instance GETs: {html}"
        );
        assert!(
            !html.contains("/api/v1/org-entities/")
                && !html.contains("/api/v1/employees/")
                && !html.contains("/api/v1/users/"),
            "must not invent privileged hrefs: {html}"
        );
        assert!(
            !html.contains("data-slug") && !html.contains("data-status="),
            "must not keep the OrgEntitySummary dual contract: {html}"
        );
        assert!(
            html.contains("href=\"/organization\""),
            "authorized org nav is SSR: {html}"
        );
        assert!(
            !html.contains("href=\"/hr\"") && !html.contains("href=\"/payroll\""),
            "nav must omit unauthorized/empty screens: {html}"
        );
        assert!(
            !html.contains("/pkg/"),
            "org read projection is SSR, not an island: {html}"
        );
        assert!(
            island_component(&html).is_none(),
            "org screen must not emit an island: {html}"
        );
        assert_shipping_invariants(&html);
    }

    #[test]
    fn shipping_screens_hr_is_ssr_contracts_and_omits_phone() {
        let screens = ShippingScreens {
            people: ScreenSection::Rows(vec![sample_person()]),
            employments: ScreenSection::Rows(vec![sample_employment()]),
            ..ShippingScreens::default()
        };
        let html = render_screens(&screens, UiScreen::Hr);
        assert_ne!(html, render_shell(), "{html}");
        assert!(
            html.contains("data-screen=\"hr\""),
            "HR body must be a mounted SSR screen: {html}"
        );
        for key in yaml_schema_required_keys(OPENAPI, "Person") {
            let attr = data_attr(key, "person-id");
            assert!(
                html.contains(&attr),
                "HR markup must carry Person Head key {key} as {attr}: {html}"
            );
        }
        assert_eq!(
            yaml_schema_required_keys(OPENAPI, "Person"),
            ["id", "version", "display_name", "legal_name"],
            "Person Head must stay the published four-field set"
        );
        for key in yaml_schema_required_keys(OPENAPI, "Employment") {
            let attr = data_attr(key, "employment-id");
            assert!(
                html.contains(&attr),
                "HR markup must carry Employment Head key {key} as {attr}: {html}"
            );
        }
        assert_eq!(
            yaml_schema_required_keys(OPENAPI, "Employment"),
            [
                "id",
                "version",
                "appointed_on",
                "person_id",
                "org_unit_id",
                "job_position_id"
            ],
            "Employment Head must stay the published six-field set"
        );
        assert!(
            html.contains("data-person-id=\"00000000-0000-0000-0000-0000000000bb\""),
            "{html}"
        );
        assert!(
            html.contains("data-employment-id=\"00000000-0000-0000-0000-0000000000ee\""),
            "{html}"
        );
        assert!(
            html.contains("홍길동"),
            "HR row must show human-safe display_name: {html}"
        );
        assert!(
            html.contains("href=\"/api/v1/persons/00000000-0000-0000-0000-0000000000bb\""),
            "HR must drill through the published Person instance GET: {html}"
        );
        assert!(
            html.contains("href=\"/api/v1/employments/00000000-0000-0000-0000-0000000000ee\""),
            "HR must drill through the published Employment instance GET: {html}"
        );
        assert!(
            html.contains("data-job-position-id=\"\""),
            "job_position_id stays an id attribute, including empty: {html}"
        );
        assert!(
            !html.contains("/api/v1/employees/") && !html.contains("/api/v1/users/"),
            "HR must not drill through privileged employee/user GET: {html}"
        );
        assert_not_directory_person(&html);
        assert!(
            html.contains("href=\"/hr\""),
            "authorized HR nav is SSR: {html}"
        );
        let lowered = html.to_ascii_lowercase();
        assert!(!lowered.contains("phone"), "directory phone leaked: {html}");
        assert!(
            !lowered.contains("salary")
                && !lowered.contains("bank_account")
                && !lowered.contains("base_pay")
                && !html.contains("data-rrn"),
            "Employment Head must not copy write-bag or PII: {html}"
        );
        assert!(
            !html.contains("/pkg/"),
            "HR read projection is SSR, not an island: {html}"
        );
        assert!(
            island_component(&html).is_none(),
            "HR screen must not emit an island: {html}"
        );
        assert_shipping_invariants(&html);
    }

    #[test]
    fn shipping_screens_home_composes_authorized_sections_only() {
        let screens = ShippingScreens {
            companies: ScreenSection::Rows(vec![sample_company()]),
            org_units: ScreenSection::Rows(vec![sample_org_unit()]),
            people: ScreenSection::Rows(vec![sample_person()]),
            employments: ScreenSection::Rows(vec![sample_employment()]),
            runs: ScreenSection::Rows(vec![sample_run()]),
        };
        let html = render_screens(&screens, UiScreen::Home);
        assert!(html.contains("data-screen=\"organization\""), "{html}");
        assert!(html.contains("data-screen=\"hr\""), "{html}");
        assert!(html.contains("data-screen=\"payroll\""), "{html}");
        assert!(html.contains("href=\"/organization\""), "{html}");
        assert!(html.contains("href=\"/hr\""), "{html}");
        assert!(html.contains("href=\"/payroll\""), "{html}");
        assert!(
            html.contains("data-org-id=\"00000000-0000-0000-0000-0000000000aa\"")
                && html.contains("data-org-unit-id=\"00000000-0000-0000-0000-0000000000dd\"")
                && html.contains("data-legal-name=")
                && html.contains("data-person-id=\"00000000-0000-0000-0000-0000000000bb\"")
                && html.contains("data-employment-id=\"00000000-0000-0000-0000-0000000000ee\"")
                && html.contains("data-run-id=\"00000000-0000-0000-0000-000000000001\""),
            "home must carry published Head identifiers: {html}"
        );
        assert_not_directory_person(&html);
        let component = island_component(&html).unwrap_or("");
        assert!(
            component.starts_with("AuthorizedRuns_"),
            "payroll interaction stays the AuthorizedRuns island: {html}"
        );
        assert!(
            html.contains("/pkg/console_payroll_ui.js"),
            "WASM hydrate only when the payroll island is composed: {html}"
        );
        assert_shipping_invariants(&html);

        let payroll_only = render_screens(
            &ShippingScreens {
                runs: ScreenSection::Rows(vec![sample_run()]),
                ..ShippingScreens::default()
            },
            UiScreen::Payroll,
        );
        assert!(
            payroll_only.contains("data-screen=\"payroll\""),
            "{payroll_only}"
        );
        assert!(
            !payroll_only.contains("data-screen=\"organization\"")
                && !payroll_only.contains("data-screen=\"hr\""),
            "focused payroll must omit other bodies: {payroll_only}"
        );
        assert_shipping_invariants(&payroll_only);
    }

    #[test]
    fn shipping_screens_focus_keeps_authorized_nav_and_omits_wasm_off_payroll() {
        let screens = ShippingScreens {
            companies: ScreenSection::Rows(vec![sample_company()]),
            org_units: ScreenSection::Rows(vec![sample_org_unit()]),
            people: ScreenSection::Rows(vec![sample_person()]),
            employments: ScreenSection::Rows(vec![sample_employment()]),
            runs: ScreenSection::Rows(vec![sample_run()]),
        };

        let org_html = render_screens(&screens, UiScreen::Organization);
        assert!(
            org_html.contains("data-screen=\"organization\""),
            "{org_html}"
        );
        assert!(
            !org_html.contains("data-screen=\"hr\"")
                && !org_html.contains("data-screen=\"payroll\""),
            "focused org must omit other bodies: {org_html}"
        );
        assert!(org_html.contains("href=\"/organization\""), "{org_html}");
        assert!(org_html.contains("href=\"/hr\""), "{org_html}");
        assert!(org_html.contains("href=\"/payroll\""), "{org_html}");
        assert!(
            !org_html.contains("/pkg/"),
            "org body must not hydrate WASM: {org_html}"
        );
        assert_shipping_invariants(&org_html);

        let payroll_html = render_screens(&screens, UiScreen::Payroll);
        assert!(
            payroll_html.contains("data-screen=\"payroll\""),
            "{payroll_html}"
        );
        assert!(
            !payroll_html.contains("data-screen=\"organization\"")
                && !payroll_html.contains("data-screen=\"hr\""),
            "focused payroll must omit other bodies: {payroll_html}"
        );
        assert!(
            payroll_html.contains("href=\"/organization\"")
                && payroll_html.contains("href=\"/hr\"")
                && payroll_html.contains("href=\"/payroll\""),
            "authorized nav must stay reachable from a focused screen: {payroll_html}"
        );
        assert!(
            payroll_html.contains("/pkg/console_payroll_ui.js"),
            "{payroll_html}"
        );
        assert_shipping_invariants(&payroll_html);

        let denied_payroll = render_screens(
            &ShippingScreens {
                companies: ScreenSection::Rows(vec![sample_company()]),
                ..ShippingScreens::default()
            },
            UiScreen::Payroll,
        );
        assert_eq!(
            denied_payroll,
            render_shell(),
            "unauthorized payroll route must omit, not leak sibling nav"
        );
    }
}

#[cfg(all(test, feature = "ssr"))]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod private_document_tests {
    use super::*;
    use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
    use axum::response::{IntoResponse, Response};
    use std::collections::BTreeSet;
    use std::future::Future;
    use std::task::{Context, Poll, Waker};

    fn frame_policy_is_closed(headers: &HeaderMap) -> bool {
        let policies: Vec<_> = headers
            .get_all(header::CONTENT_SECURITY_POLICY)
            .iter()
            .collect();
        if policies.len() != 1 {
            return false;
        }
        let Ok(policy) = policies[0].to_str() else {
            return false;
        };
        let directives: Vec<Vec<_>> = policy
            .split(';')
            .map(|part| part.split_ascii_whitespace().collect::<Vec<_>>())
            .filter(|parts| {
                parts
                    .first()
                    .is_some_and(|name| name.eq_ignore_ascii_case("frame-ancestors"))
            })
            .collect();
        directives.len() == 1 && directives[0].as_slice() == ["frame-ancestors", "'none'"]
    }

    fn assert_document(
        response: impl IntoResponse,
        status: StatusCode,
        expected: String,
    ) -> HeaderMap {
        let response: Response = response.into_response();
        assert_eq!(response.status(), status);
        let headers = response.headers().clone();
        for (name, expected) in [
            (header::CACHE_CONTROL, "no-store"),
            (header::PRAGMA, "no-cache"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
            (header::REFERRER_POLICY, "no-referrer"),
            (header::CONTENT_TYPE, "text/html; charset=utf-8"),
        ] {
            let values: Vec<_> = headers.get_all(name.clone()).iter().collect();
            assert_eq!(values.len(), 1, "missing or duplicate {name}");
            assert_eq!(values[0].to_str().unwrap(), expected, "{name}");
        }
        assert!(
            frame_policy_is_closed(&headers),
            "missing or ambiguous frame-ancestors denial"
        );
        assert!(!headers.contains_key(header::SET_COOKIE));
        let vary: BTreeSet<_> = headers
            .get_all(header::VARY)
            .iter()
            .flat_map(|value| value.to_str().unwrap().split(','))
            .map(|value| value.trim().to_ascii_lowercase())
            .collect();
        for key in ["authorization", "cookie", "origin"] {
            assert!(vary.contains(key), "missing Vary {key}");
        }
        // These actual owners return fully buffered Html<String>, not streams.
        // Polling the ready in-memory body needs no async runtime or dependency.
        let mut body = std::pin::pin!(axum::body::to_bytes(response.into_body(), 256 * 1024));
        let Poll::Ready(bytes) = body.as_mut().poll(&mut Context::from_waker(Waker::noop())) else {
            panic!("buffered SSR owner unexpectedly returned a pending body");
        };
        assert_eq!(
            bytes.unwrap().as_ref(),
            expected.as_bytes(),
            "response changed SSR document bytes"
        );
        headers
    }

    fn run() -> RunSummary {
        RunSummary {
            id: "00000000-0000-0000-0000-000000000001".into(),
            period_start: "2026-06-01".into(),
            period_end: "2026-06-30".into(),
            source_label: "검토 <source> & evidence".into(),
            status: "BLOCKED_LEGAL_GATE".into(),
            calculation_enabled: false,
            created_at: "2026-06-01T00:00:00Z".into(),
            updated_at: "2026-06-01T00:00:00Z".into(),
        }
    }

    #[test]
    fn ssr_private_shell_response_preserves_anonymous_document() {
        assert_document(html_shell(), StatusCode::OK, render_shell());
    }

    #[test]
    fn ssr_private_runs_response_preserves_empty_and_authorized_documents() {
        for runs in [vec![], vec![run()]] {
            assert_document(
                html_shell_with(&runs),
                StatusCode::OK,
                render_shell_with(&runs),
            );
        }
    }

    #[test]
    fn ssr_private_screens_response_preserves_all_focus_and_disclosure_states() {
        for runs in [
            ScreenSection::Omitted,
            ScreenSection::Empty,
            ScreenSection::Rows(vec![run()]),
        ] {
            let screens = ShippingScreens {
                runs,
                ..ShippingScreens::default()
            };
            for focus in [
                UiScreen::Home,
                UiScreen::Organization,
                UiScreen::Hr,
                UiScreen::Payroll,
            ] {
                assert_document(
                    html_shell_with_screens(&screens, focus),
                    StatusCode::OK,
                    render_screens(&screens, focus),
                );
            }
        }
    }

    #[test]
    fn ssr_private_native_documents_retain_status_bytes_and_restrictive_csp() {
        use native_account::{CompanySetupEligibility, ContextState, Page};
        let cases: [(fn() -> Page, StatusCode); 8] = [
            (|| Page::Public, StatusCode::OK),
            (|| Page::SignIn, StatusCode::OK),
            (
                || Page::Register {
                    version: "terms-v1".into(),
                    items: vec![native_account::TermsItem {
                        kind: "service".into(),
                        title: "이용 약관".into(),
                        content_url: "/terms/service".into(),
                        content: "검토 <terms> & consent".into(),
                    }],
                },
                StatusCode::OK,
            ),
            (
                || Page::Account {
                    context: ContextState::Empty,
                    can_logout: true,
                    company_setup: CompanySetupEligibility::Eligible,
                },
                StatusCode::OK,
            ),
            (
                || Page::Account {
                    context: ContextState::Unavailable,
                    can_logout: true,
                    company_setup: CompanySetupEligibility::Unavailable,
                },
                StatusCode::SERVICE_UNAVAILABLE,
            ),
            (
                || Page::CompanySetup {
                    account_id: "11111111-1111-4111-8111-111111111111".into(),
                    command_id: None,
                },
                StatusCode::OK,
            ),
            (|| Page::Refused, StatusCode::FORBIDDEN),
            (|| Page::Unavailable, StatusCode::SERVICE_UNAVAILABLE),
        ];
        for (page, status) in cases {
            let headers = assert_document(
                native_account::document(page(), status),
                status,
                native_account::render(page()),
            );
            assert_eq!(
                headers[header::CONTENT_SECURITY_POLICY],
                "default-src 'self'; script-src 'self'; style-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'"
            );
        }
    }

    #[test]
    fn ssr_private_frame_oracle_rejects_missing_permissive_and_duplicate_policies() {
        let mut headers = HeaderMap::new();
        assert!(!frame_policy_is_closed(&headers));
        for policy in [
            "frame-ancestors *",
            "frame-ancestors 'none' *",
            "frame-ancestors 'none'; frame-ancestors *",
            "default-src 'none'",
            "frame-ancestors",
        ] {
            headers.insert(
                header::CONTENT_SECURITY_POLICY,
                HeaderValue::from_str(policy).unwrap(),
            );
            assert!(!frame_policy_is_closed(&headers), "accepted {policy}");
        }
        headers.insert(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static("object-src 'none'; frame-ancestors 'none'"),
        );
        assert!(frame_policy_is_closed(&headers));
        headers.append(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static("frame-ancestors 'none'"),
        );
        assert!(!frame_policy_is_closed(&headers));
    }
}

#[cfg(all(test, feature = "ssr"))]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod people_tests;

#[cfg(all(test, feature = "ssr"))]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod workspace_tests;

#[cfg(all(test, feature = "ssr"))]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod workflow_presentation_tests;

#[cfg(all(test, feature = "ssr"))]
mod native_policy_validation_tests;

#[cfg(all(test, feature = "ssr"))]
mod native_payroll_tests;

#[cfg(all(test, feature = "ssr"))]
mod native_people_tests;

mod native_workspace_header;
