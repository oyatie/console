//! SSR-only Payroll collection. Inputs are committed, authorized owner projections.
use super::native_workspace_header::{self, NavigationMode};
use leptos::prelude::*;
use serde_json::Value;

pub struct CompanyIdentity {
    pub name: String,
    pub slug: String,
}
pub struct Collection {
    pub people_navigation: (bool, bool),
    pub company: String,
    pub identity: Option<CompanyIdentity>,
    pub items: Vec<Run>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
pub struct Run {
    pub id: String,
    pub period_start: String,
    pub period_end: String,
    pub source_label: String,
    pub status: String,
    pub calculation_enabled: bool,
    pub created_by: Option<String>,
    pub approved_by: Option<String>,
    pub approved_at: Option<String>,
    pub close_receipt: Option<Value>,
    pub submitted_by: Option<String>,
    pub submitted_at: Option<String>,
    pub decided_by: Option<String>,
    pub decided_at: Option<String>,
    pub decision_reason: Option<String>,
    pub approval_ref: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
pub enum Page {
    Runs(Collection),
    NotVisible,
    AuthenticationRequired,
    Unavailable,
    InvalidRequest,
}

fn field(name: &'static str, label: &'static str, value: impl IntoView) -> AnyView {
    view! { <dt>{label}</dt><dd data-field=name>{value}</dd> }.into_any()
}
fn optional(value: Option<String>) -> String {
    value.unwrap_or_else(|| "기록 없음".into())
}
fn value_text(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        _ => value.to_string(),
    }
}
fn receipt(value: Option<Value>) -> AnyView {
    let Some(value) = value else {
        return view! { "기록 없음" }.into_any();
    };
    let checks = value.get("checks").and_then(Value::as_array).map(|checks| {
        checks.iter().map(|check| {
            let text = |key| check.get(key).map(value_text).unwrap_or_else(|| "기록 없음".into());
            let outcome = match check.get("ok").and_then(Value::as_bool) {
                Some(true) => "통과", Some(false) => "통과하지 않음", None => "결과 미확인",
            };
            let warning = match check.get("warn").and_then(Value::as_bool) {
                Some(true) => "주의", Some(false) => "주의 표시 없음", None => "주의 여부 미확인",
            };
            view! { <li class="payroll-check">
                <div class="payroll-check-heading"><strong>{text("label_ko")}</strong><span>{outcome}" · "{warning}</span></div>
                <p>{text("note")}</p><p class="payroll-reference">"확인 항목: "{text("key")}</p>
                <p class="payroll-reference">"관련 기록: "{text("blocking_refs")}</p>
            </li> }.into_any()
        }).collect::<Vec<_>>()
    });
    let attestation = value.get("attested_by").map(|actor| {
        view! {
            <p class="payroll-reference">"확인 계정: "{value_text(actor)}</p>
        }
    });
    let at = value.get("attested_at").map(|at| {
        view! {
            <p class="payroll-reference">"확인 시각: "{value_text(at)}</p>
        }
    });
    // Keep unknown and malformed historical shapes exactly as authorized data.
    // JSON strings remain escaped text, never markup or navigable object links.
    let raw = value.to_string();
    view! { <div class="payroll-evidence">
        {checks.map(|items| view! { <ul class="payroll-checks">{items}</ul> })}
        {attestation}{at}
        <details class="payroll-raw"><summary>"증빙 원문 전체"</summary><pre>{raw}</pre></details>
    </div> }
    .into_any()
}
fn status(raw: &str) -> String {
    let label = match raw {
        "SUBMITTED" => "검토 대기",
        "APPROVED" => "승인",
        "REJECTED" => "반려",
        "STAGED" => "자료 준비",
        "READY_FOR_REVIEW" => "검토 준비",
        "BLOCKED_LEGAL_GATE" => "확인 필요",
        "ATTENDANCE_CLOSED" => "근태 마감",
        "CALCULATING" => "산정 중",
        "CALCULATED" => "산정됨",
        _ => return raw.to_owned(),
    };
    format!("{label} · {raw}")
}
fn run_card(run: Run) -> AnyView {
    let heading = format!("payroll-run-{}", run.id);
    let period = format!("{} — {}", run.period_start, run.period_end);
    let source = run.source_label.clone();
    let state = status(&run.status);
    let calculation = if run.calculation_enabled {
        "산정 가능"
    } else {
        "산정 불가"
    };
    let fields = vec![
        field("id", "회차 식별자", run.id.clone()),
        field("period_start", "산정 시작", run.period_start),
        field("period_end", "산정 종료", run.period_end),
        field("source_label", "자료 이름", run.source_label),
        field("status", "상태", status(&run.status)),
        field("calculation_enabled", "산정 가능 여부", calculation),
        field("created_by", "작성 계정", optional(run.created_by)),
        field("approved_by", "승인 계정", optional(run.approved_by)),
        field("approved_at", "승인 시각", optional(run.approved_at)),
        field(
            "close_receipt",
            "근태 마감 증빙",
            receipt(run.close_receipt),
        ),
        field("submitted_by", "제출 계정", optional(run.submitted_by)),
        field("submitted_at", "제출 시각", optional(run.submitted_at)),
        field("decided_by", "결정 계정", optional(run.decided_by)),
        field("decided_at", "결정 시각", optional(run.decided_at)),
        field(
            "decision_reason",
            "결정 사유",
            optional(run.decision_reason),
        ),
        field("approval_ref", "승인 기록", optional(run.approval_ref)),
        field("created_at", "작성 시각", run.created_at),
        field("updated_at", "최근 변경", run.updated_at),
    ];
    view! { <article class="panel payroll-run" data-run-id=run.id aria-labelledby=heading.clone()>
        <div class="payroll-run-heading"><div><p class="page-eyebrow">{period}</p><h2 id=heading.clone()>{source}</h2></div>
            <span class="payroll-status">{state}</span></div>
        <div class="payroll-run-context"><span>{calculation}</span><span>"근태 증빙 · 제출 · 승인 기록"</span></div>
        <details class="payroll-inspector"><summary>"증빙과 처리 이력 확인"</summary><dl class="record-meta">{fields}</dl></details>
    </article> }.into_any()
}
fn listing(page: Collection) -> AnyView {
    let base = format!("/companies/{}/payroll", page.company);
    let size = page.limit;
    let previous = (page.offset > 0).then(|| {
        let offset = page.offset.saturating_sub(size).max(0);
        view! { <a class="payroll-page-link" rel="prev" href=format!("{base}?limit={size}&offset={offset}")>"이전 페이지"</a> }
    });
    let next = page.offset.checked_add(size).filter(|offset| *offset < page.total).map(|offset| {
        view! { <a class="payroll-page-link" rel="next" href=format!("{base}?limit={size}&offset={offset}")>"다음 페이지"</a> }
    });
    let summary = if page.items.is_empty() {
        format!("전체 {}회차 · 이 페이지에 표시할 회차 없음", page.total)
    } else {
        let count = i64::try_from(page.items.len()).unwrap_or(i64::MAX);
        format!(
            "전체 {}회차 · {}–{}",
            page.total,
            page.offset.saturating_add(1),
            page.offset.saturating_add(count)
        )
    };
    let empty = if page.items.is_empty() {
        Some(if page.total == 0 {
            view! { <section class="panel payroll-empty" data-state="empty"><span class="payroll-empty-mark" aria-hidden="true">"▤"</span>
                <h2>"등록된 급여 회차가 없습니다"</h2><p>"이 회사에 기록된 급여 회차가 생기면 산정 기간과 준비 상태, 검토 이력을 여기에서 확인할 수 있습니다."</p>
            </section> }.into_any()
        } else {
            let recovery = (page.offset >= page.total).then(|| view! {
                <a class="payroll-page-link" href=format!("{base}?limit={size}&offset=0")>"첫 페이지"</a>
            });
            view! { <section class="panel payroll-empty" data-state="empty-window"><h2>"이 페이지에 표시할 급여 회차가 없습니다"</h2>
                <p>"목록이 변경되었을 수 있습니다. 다른 페이지에서 현재 목록을 다시 확인하세요."</p>{recovery}
            </section> }.into_any()
        })
    } else {
        None
    };
    let identity = match page.identity {
        Some(identity) => view! { <a class="payroll-company" href=format!("/companies/{}",page.company)>{identity.name}<span>{identity.slug}</span></a> }.into_any(),
        None => view! { <div class="payroll-company">"회사 범위"<span class="payroll-reference">{page.company}</span></div> }.into_any(),
    };
    view! { <section class="payroll-collection" data-screen="payroll" aria-label="회사 급여 회차">
        <div class="payroll-toolbar">{identity}<form method="get" action=base>
            <label for="payroll-page-size">"페이지당 회차"</label>
            <select id="payroll-page-size" name="limit">
                {[25_i64,50,100,250,500].into_iter().map(|value|view! { <option value=value.to_string() selected={value==size}>{value}</option> }).collect::<Vec<_>>()}
                {(![25,50,100,250,500].contains(&size)).then(||view! {<option value=size.to_string() selected>{size}</option>})}
            </select><input type="hidden" name="offset" value="0"/><button type="submit">"적용"</button>
        </form></div>
        <p class="payroll-window" data-pagination-summary>{summary}</p>
        <div class="payroll-runs">{empty}{page.items.into_iter().map(run_card).collect::<Vec<_>>()}</div>
        <nav class="payroll-pagination" aria-label="급여 목록 페이지">{previous}{next}</nav>
    </section> }.into_any()
}
fn problem(title: &'static str, description: &'static str) -> AnyView {
    view! { <section class="panel payroll-empty"><h2>{title}</h2><p>{description}</p><a class="payroll-page-link" href="/account">"내 업무 공간 확인"</a></section> }.into_any()
}
pub fn render(page: Page) -> String {
    let company = match &page {
        Page::Runs(page) => Some((page.company.clone(), page.people_navigation)),
        _ => None,
    };
    let content = match page {
        Page::Runs(page) => listing(page),
        Page::NotVisible => problem(
            "급여 목록을 열 수 없습니다",
            "내 계정에서 접근할 수 있는 회사 업무 공간을 확인하세요.",
        ),
        Page::AuthenticationRequired => problem(
            "다시 로그인해 주세요",
            "계정을 확인한 뒤 업무 공간에서 급여 목록을 다시 열어 주세요.",
        ),
        Page::Unavailable => problem(
            "지금 급여 목록을 확인할 수 없습니다",
            "연결이 복구된 뒤 같은 주소에서 다시 확인해 주세요.",
        ),
        Page::InvalidRequest => problem(
            "목록 주소를 확인해 주세요",
            "업무 공간에서 급여 목록을 다시 열어 주세요.",
        ),
    };
    let payroll_shortcut = company
        .as_ref()
        .map(|(id, _)| (format!("/companies/{id}/payroll"), true));
    let header = native_workspace_header::render(
        |mode| {
            company.as_ref().filter(|(_, (people_read, people_create))| *people_read || *people_create || mode == NavigationMode::Desktop).map(|(id, (people_read, people_create))|view! {
                <p class="nav-group">"사람과 조직"</p><nav aria-label="사람과 조직 탐색">
                    {people_read.then(||view! {<a href=format!("/companies/{id}/people")>"사람"</a>})}
                    {people_create.then(||view! {<a href=format!("/companies/{id}/people/new")>"사람 등록"</a>})}
                    {(mode == NavigationMode::Desktop).then(||view! {<a href=format!("/companies/{id}/payroll") aria-current="page">"급여"</a>})}
                </nav>
            }.into_any()).unwrap_or_else(|| ().into_any())
        },
        payroll_shortcut,
    );
    let html = view! {
        <html lang="ko"><head><meta charset="utf-8"/><meta name="viewport" content="width=device-width, initial-scale=1"/>
            <title>"급여 · Console"</title><link rel="stylesheet" href="/assets/workspace.css"/></head>
        <body class="workspace payroll-workspace"><a class="skip-link" href="#main-content">"본문 바로가기"</a>
            {header}
            <main id="main-content" tabindex="-1"><div class="page-heading"><p class="page-eyebrow">"사람과 조직 / 급여"</p><h1>"급여"</h1>
                <p class="page-description">"산정 기간별 준비 상태와 증빙, 검토 기록을 확인하세요."</p></div>{content}</main>
        </body></html>
    }.to_html();
    format!("<!DOCTYPE html>{html}")
}
pub fn document(page: Page, status: axum::http::StatusCode) -> axum::response::Response {
    super::ssr::private_document(
        render(page),
        status,
        "default-src 'self'; script-src 'none'; style-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'",
    )
}
