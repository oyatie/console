//! Native directory pages composed from current authorized projections.
//! No client storage, identity matching, employment decisions, or mutations.
use super::native_workspace_header::{self, NavigationMode};
use leptos::prelude::*;

pub struct Scope {
    pub company: String,
    pub company_name: Option<String>,
    pub company_link: bool,
    pub directory_link: bool,
    pub can_create: bool,
    pub payroll_link: bool,
    pub policy_link: bool,
}

pub struct Record {
    pub employee_id: String,
    pub person_id: String,
    pub legal_name: Option<String>,
    pub employee_number: Option<String>,
    pub person_version: String,
    pub registered_at: String,
}

/// Expected registrations select the exact state reviewed with the form.
/// They are checked by the owner again; hidden fields are not authority.
pub struct Expectations {
    pub company_epoch: String,
    pub object_type_id: String,
    pub action_type_id: String,
    pub action_revision: String,
    pub schema_revision: String,
    pub legal_name_property_id: String,
    pub employee_number_property_id: String,
}

pub struct Registration {
    pub command: String,
    pub proof: String,
    pub expected: Expectations,
    pub legal_name: String,
    pub employee_number: String,
    pub name_error: Option<&'static str>,
    pub number_error: Option<&'static str>,
    pub form_error: Option<&'static str>,
}

pub struct Request {
    pub command: String,
    pub legal_name: String,
    pub employee_number: String,
    pub accepted_at: String,
    pub deadline: String,
    pub intake_receipt: String,
    pub expected_company_epoch: String,
    pub outcome: Outcome,
}

pub enum Outcome {
    Pending {
        proof: String,
    },
    Committed {
        employee_id: String,
        person_id: String,
        receipt: String,
        registered_at: String,
    },
    Rejected {
        reason: &'static str,
    },
    Conflicting {
        reason: &'static str,
    },
    Cancelled,
    Expired,
}

pub enum Page {
    Directory {
        scope: Scope,
        records: Vec<Record>,
        next_after: Option<String>,
    },
    Registration {
        scope: Scope,
        form: Registration,
    },
    RegistrationConflict {
        scope: Scope,
        command: String,
        legal_name: String,
        employee_number: String,
    },
    Request {
        scope: Scope,
        request: Request,
    },
    RequestNotVisible {
        scope: Scope,
        command: String,
    },
    Detail {
        scope: Scope,
        record: Record,
    },
    Uncertain {
        company: String,
        command: String,
    },
    Refused,
    Unavailable,
}

fn directory(company: &str) -> String {
    format!("/companies/{company}/people")
}

fn consequences() -> AnyView {
    view! {
        <section class="panel people-consequences" aria-labelledby="people-consequence-heading">
            <h2 id="people-consequence-heading">"등록하면 무엇이 달라지나요?"</h2>
            <div class="policy-panel-body">
                <p>"사람 목록에 등록합니다. 고용과 발령은 별도로 승인해야 합니다."</p>
                <p class="supporting">"이름과 사번으로 사람을 구분합니다. 이름이 같아도 다른 사람의 기록과 합치지 않습니다."</p>
            </div>
        </section>
    }.into_any()
}

// Presentation distinguishes absence from a present but visually blank value.
// This never normalizes or changes the authorized stored projection.
fn record_name(value: Option<&str>) -> &str {
    match value {
        None => "이름 미등록",
        Some(value) if value.trim().is_empty() => "공백으로 저장된 이름",
        Some(value) => value,
    }
}
fn record_number(value: Option<&str>) -> &str {
    match value {
        None => "사번 미등록",
        Some(value) if value.trim().is_empty() => "공백으로 저장된 사번",
        Some(value) => value,
    }
}

fn directory_body(scope: &Scope, records: Vec<Record>, next_after: Option<String>) -> AnyView {
    let path = directory(&scope.company);
    let empty = records.is_empty();
    let entries = records.into_iter().map(|record| {
        let needs_identity = record.legal_name.as_deref().is_none_or(|name| name.trim().is_empty());
        let name = record_name(record.legal_name.as_deref()).to_owned();
        let number = record_number(record.employee_number.as_deref()).to_owned();
        let label = needs_identity.then(|| format!("{name} · 목록 기록 {}", record.employee_id));
        view! {
            <li class="directory-entry" data-people-record=record.employee_id.clone() data-people-person=record.person_id>
                <a class="directory-name" href=format!("{path}/{}",record.employee_id) aria-label=label>{name}</a>
                <dl><dt>"사번"</dt><dd>{number}</dd>
                    {needs_identity.then(|| view! {<dt>"목록 기록"</dt><dd>{record.employee_id.clone()}</dd>})}
                </dl>
                <span class="directory-state">"목록에 등록됨"</span>
            </li>
        }
    }).collect_view();
    view! {
        <section class="panel" aria-labelledby="people-directory-heading">
            <div class="people-list-heading"><h2 id="people-directory-heading">"사람 목록"</h2>
                {(!empty && scope.can_create).then(||view! {<a class="policy-button" href=format!("{path}/new")>"사람 등록"</a>})}
            </div>
            {if empty {view! {
                <div class="people-empty"><h3>"표시할 사람이 없습니다"</h3>
                    <p>"현재 목록에 표시할 등록 정보가 없습니다."</p>
                    {scope.can_create.then(||view! {<a class="policy-button" href=format!("{path}/new")>"사람 등록"</a>})}
                </div>
            }.into_any()} else {view! {<ul class="directory-list">{entries}</ul>}.into_any()}}
            {next_after.map(|after|view! {
                <nav class="people-pagination" aria-label="사람 목록 페이지">
                    <a class="policy-button secondary" href=format!("{path}?after_employee_id={after}")>"다음 사람 보기"</a>
                </nav>
            })}
        </section>
    }.into_any()
}

fn validation_summary(form: &Registration) -> AnyView {
    let has_error =
        form.name_error.is_some() || form.number_error.is_some() || form.form_error.is_some();
    if !has_error {
        return ().into_any();
    }
    view! {
        <section id="people-input-errors" class="people-validation" role="alert" tabindex="-1" autofocus>
            <h3>"입력한 내용을 확인해 주세요"</h3>
            <p>"이번 제출로 새 요청을 접수하지 않았습니다. 작성한 내용은 아래에 남아 있습니다."</p>
            {form.form_error.map(|message|view! {<p>{message}</p>})}
            <ul>
                {form.name_error.map(|message|view! {<li><a href="#people-name">"이름: "{message}</a></li>})}
                {form.number_error.map(|message|view! {<li><a href="#people-number">"사번: "{message}</a></li>})}
            </ul>
        </section>
    }.into_any()
}

fn registration_body(scope: &Scope, form: Registration) -> AnyView {
    let path = directory(&scope.company);
    let error_summary = validation_summary(&form);
    view! {
        <div class="people-task-layout">
            <aside aria-label="등록의 영향">{consequences()}</aside>
            <section class="panel people-input-panel"><h2>"등록할 사람"</h2><div class="policy-panel-body">
                {error_summary}
                <form method="post" action=format!("{path}/requests") data-people-operation="prepare" autocomplete="off">
                    <input type="hidden" name="csrf_proof" value=form.proof/>
                    <input type="hidden" name="command_id" value=form.command/>
                    <input type="hidden" name="expected_company_epoch" value=form.expected.company_epoch/>
                    <input type="hidden" name="object_type_id" value=form.expected.object_type_id/>
                    <input type="hidden" name="action_type_id" value=form.expected.action_type_id/>
                    <input type="hidden" name="expected_action_revision" value=form.expected.action_revision/>
                    <input type="hidden" name="expected_schema_revision" value=form.expected.schema_revision/>
                    <input type="hidden" name="legal_name_property_id" value=form.expected.legal_name_property_id/>
                    <input type="hidden" name="employee_number_property_id" value=form.expected.employee_number_property_id/>
                    <label for="people-name">"이름"</label>
                    <input id="people-name" name="legal_name" type="text" required value=form.legal_name
                        aria-invalid=form.name_error.map(|_|"true")
                        aria-describedby=if form.name_error.is_some(){"people-name-help people-name-error"}else{"people-name-help"}/>
                    <p id="people-name-help" class="supporting">"공식 기록에 사용할 이름을 입력하세요. 최대 200자입니다."</p>
                    {form.name_error.map(|message|view! {<p id="people-name-error" class="people-field-error">{message}</p>})}
                    <label for="people-number">"사번"</label>
                    <input id="people-number" name="employee_number" type="text" required value=form.employee_number
                        aria-invalid=form.number_error.map(|_|"true")
                        aria-describedby=if form.number_error.is_some(){"people-number-help people-number-error"}else{"people-number-help"}/>
                    <p id="people-number-help" class="supporting">"이 회사에서 사람을 구분할 고유한 번호입니다. 최대 64자이며, 등록된 번호와 중복될 수 없습니다."</p>
                    {form.number_error.map(|message|view! {<p id="people-number-error" class="people-field-error">{message}</p>})}
                    <div class="policy-actions"><button class="policy-button" type="submit">"등록 내용 확인"</button>
                        {scope.directory_link.then(||view! {<a href=path.clone()>"목록으로 돌아가기"</a>})}
                    </div>
                </form>
            </div></section>
        </div>
    }.into_any()
}

fn request_body(scope: &Scope, request: Request) -> AnyView {
    let root = directory(&scope.company);
    let path = format!("{root}/requests/{}", request.command);
    let (state, title, description, employee, person, receipt, at, actions) = match request.outcome {
        Outcome::Pending { proof } => (
            "pending", "등록 내용을 확인하세요", "요청을 접수했습니다. 등록 확정을 선택하면 사람 목록에 반영됩니다.",
            None, None, None, None, view! {
                <p class="supporting">"처리 기한: "{request.deadline.clone()}</p>
                <div class="policy-actions">
                    <form method="post" action=format!("{path}/execute") data-people-operation="execute">
                        <input type="hidden" name="command_id" value=request.command.clone()/>
                        <input type="hidden" name="csrf_proof" value=proof.clone()/>
                        <button class="policy-button" type="submit">"등록 확정"</button>
                    </form>
                    <form method="post" action=format!("{path}/cancel") data-people-operation="cancel">
                        <input type="hidden" name="command_id" value=request.command.clone()/>
                        <input type="hidden" name="csrf_proof" value=proof/>
                        <button class="policy-button secondary" type="submit">"등록 요청 취소"</button>
                    </form>
                </div>
            }.into_any(),
        ),
        Outcome::Committed { employee_id, person_id, receipt, registered_at } => {
            let link = scope.directory_link.then(||view! {
                <a class="policy-button" href=format!("{root}/{employee_id}")>"등록한 사람 보기"</a>
            });
            ("committed", "사람 등록을 완료했습니다", "사람 목록에 등록했습니다. 이 등록으로 고용이나 발령이 생성되지는 않았습니다.",
                Some(employee_id), Some(person_id), Some(receipt), Some(registered_at), link.into_any())
        },
        Outcome::Rejected { reason } => ("rejected", "등록하지 못했습니다", reason, None, None, None, None, ().into_any()),
        Outcome::Conflicting { reason } => ("conflicting", "변경된 내용을 확인해 주세요", reason, None, None, None, None, ().into_any()),
        Outcome::Cancelled => ("cancelled", "등록 요청을 취소했습니다", "이 요청으로 사람을 등록하지 않았습니다.", None, None, None, None, ().into_any()),
        Outcome::Expired => ("expired", "등록 요청의 처리 기한이 지났습니다", "이 요청으로 사람을 등록하지 않았습니다. 현재 내용을 확인하고 새 등록 요청을 작성하세요.", None, None, None, None, ().into_any()),
    };
    view! {
        <section class="panel people-request" data-people-outcome=state data-people-command=request.command.clone()
            data-people-employee=employee data-people-person=person>
            <h2>{title}</h2><div class="policy-panel-body">
                <p class="people-outcome-description">{description}</p>
                <dl class="people-accepted"><dt>"이름"</dt><dd>{request.legal_name}</dd><dt>"사번"</dt><dd>{request.employee_number}</dd></dl>
                <p class="supporting">"내가 제출한 등록 요청의 내용입니다."</p>
                {actions}
                {(!matches!(state,"pending"|"committed") && scope.can_create).then(||view! {
                    <div class="policy-actions"><a class="policy-button" href=format!("{root}/new")>"새 등록 요청 작성"</a></div>
                })}
                <details class="policy-provenance"><summary>"이 요청의 처리 기록"</summary>
                    <dl class="record-meta"><dt>"요청"</dt><dd>{request.command.clone()}</dd>
                        <dt>"접수 시각"</dt><dd>{request.accepted_at}</dd><dt>"처리 기한"</dt><dd>{request.deadline}</dd>
                        <dt>"접수 기록"</dt><dd>{request.intake_receipt}</dd><dt>"요청 당시 회사 버전"</dt><dd>{request.expected_company_epoch}</dd>
                        {receipt.map(|value|view! {<dt>"완료 기록"</dt><dd>{value}</dd>})}
                        {at.map(|value|view! {<dt>"등록 시각"</dt><dd>{value}</dd>})}
                    </dl><a href=path>"이 요청 다시 열기"</a>
                </details>
            </div>
        </section>
    }.into_any()
}

fn detail_body(record: Record) -> AnyView {
    view! {
        <section class="panel people-detail-card" data-people-record=record.employee_id.clone() data-people-person=record.person_id.clone()>
            <h2>"사람 목록 등록 정보"</h2><div class="policy-panel-body">
                <span class="directory-state">"목록에 등록됨"</span>
                <dl class="people-accepted"><dt>"사번"</dt><dd>{record_number(record.employee_number.as_deref()).to_owned()}</dd>
                    <dt>"등록 시각"</dt><dd>{record.registered_at}</dd><dt>"사람 기록 버전"</dt><dd>{record.person_version}</dd>
                </dl>
                <p class="supporting">"사람 목록의 정보입니다. 이 화면의 등록 상태는 고용이나 발령 상태를 나타내지 않습니다."</p>
                <details class="policy-provenance"><summary>"기록 식별 정보"</summary>
                    <dl class="record-meta"><dt>"목록 기록"</dt><dd>{record.employee_id.clone()}</dd><dt>"사람 기록"</dt><dd>{record.person_id.clone()}</dd></dl>
                </details>
            </div>
        </section>
    }.into_any()
}

pub fn render(page: Page) -> String {
    let directory_current = matches!(&page, Page::Directory { .. });
    let registration_current = matches!(&page, Page::Registration { .. });
    let scope = match &page {
        Page::Directory { scope, .. }
        | Page::Registration { scope, .. }
        | Page::RegistrationConflict { scope, .. }
        | Page::Request { scope, .. }
        | Page::RequestNotVisible { scope, .. }
        | Page::Detail { scope, .. } => Some(scope),
        _ => None,
    };
    let title = match &page {
        Page::Directory { .. } => "사람".to_owned(),
        Page::Registration { .. } => "사람 등록".to_owned(),
        Page::RegistrationConflict { .. } => "등록 내용을 다시 확인하세요".to_owned(),
        Page::Request { .. } => "사람 등록 요청".to_owned(),
        Page::RequestNotVisible { .. } => "요청 상태를 확인할 수 없습니다".to_owned(),
        Page::Detail { record, .. } => record_name(record.legal_name.as_deref()).to_owned(),
        Page::Uncertain { .. } => "등록 결과 확인".to_owned(),
        Page::Refused => "이 페이지를 열 수 없습니다".to_owned(),
        Page::Unavailable => "지금 정보를 불러올 수 없습니다".to_owned(),
    };
    let company = scope.map(|s| {
        view! {
        <p class="page-eyebrow">{s.company_name.clone().unwrap_or_else(||"선택한 회사".into())}</p>
    }.into_any()
    });
    let payroll_shortcut = scope
        .filter(|s| s.payroll_link)
        .map(|s| (format!("/companies/{}/payroll", s.company), false));
    let header = native_workspace_header::render(
        |mode| {
            scope.map(|s| {
        let id = &s.company;
        view! {
            <p class="nav-group">"사람과 조직"</p><nav aria-label="사람과 조직 탐색">
                {s.directory_link.then(||view! {<a href=directory(id) aria-current=directory_current.then_some("page")>"사람"</a>})}
                {s.can_create.then(||view! {<a href=format!("{}/new",directory(id)) aria-current=registration_current.then_some("page")>"사람 등록"</a>})}
                {(s.payroll_link && mode == NavigationMode::Desktop).then(||view! {<a href=format!("/companies/{id}/payroll")>"급여"</a>})}
            </nav>
            {(s.company_link || s.policy_link).then(||view! {
                <p class="nav-group">"관리"</p><nav aria-label="회사 관리">
                    {s.company_link.then(||view! {<a href=format!("/companies/{id}")>"회사 업무 공간"</a>})}
                    {s.policy_link.then(||view! {<a href=format!("/companies/{id}/policy")>"권한 관리"</a>})}
                </nav>
            })}
        }.into_any()
    }).unwrap_or_else(|| ().into_any())
        },
        payroll_shortcut,
    );
    let content = match page {
        Page::Directory { scope, records, next_after } => directory_body(&scope, records, next_after),
        Page::Registration { scope, form } => registration_body(&scope, form),
        Page::RegistrationConflict { scope, command, legal_name, employee_number } => view! {
            <section class="panel" data-people-outcome="prepare-conflict" role="alert" tabindex="-1" autofocus>
                <h2>"등록 내용을 다시 확인하세요"</h2><div class="policy-panel-body">
                    <p>"회사 설정이나 제출 내용이 현재 기록과 일치하지 않습니다. 요청 상태를 확인한 뒤 새 요청을 작성하세요."</p>
                    <dl class="people-accepted" aria-label="제출한 내용">
                        <dt>"이름"</dt><dd>{legal_name}</dd><dt>"사번"</dt><dd>{employee_number}</dd>
                    </dl>
                    <div class="policy-actions">
                        <a class="policy-button" href=format!("{}/requests/{command}",directory(&scope.company))>"요청 상태 확인"</a>
                        {scope.can_create.then(||view! {
                            <a href=format!("{}/new",directory(&scope.company))>"새 등록 요청 작성"</a>
                        })}
                    </div>
                </div>
            </section>
        }.into_any(),
        Page::Request { scope, request } => request_body(&scope, request),
        Page::RequestNotVisible { scope, command } => view! {
            <section class="panel" data-people-outcome="not-visible"><h2>"요청 상태를 확인할 수 없습니다"</h2>
                <div class="policy-panel-body">
                    <p>"현재 권한으로 이 요청의 처리 결과를 확인할 수 없습니다. 이 화면은 등록이 실패했거나 취소되었다는 뜻이 아닙니다."</p>
                    <p>"새 요청을 작성하기 전에 기존 요청의 처리 여부를 확인하세요."</p>
                    <div class="policy-actions">
                        <a class="policy-button" href=format!("{}/requests/{command}",directory(&scope.company))>"같은 요청 상태 다시 확인"</a>
                        {scope.can_create.then(||view! {
                            <a href=format!("{}/new",directory(&scope.company))>"새 등록 요청 작성"</a>
                        })}
                    </div>
                </div>
            </section>
        }.into_any(),
        Page::Detail { record, .. } => detail_body(record),
        Page::Uncertain { company, command } => view! {
            <section class="panel" data-people-outcome="uncertain"><h2>"처리 결과를 아직 확인하지 못했습니다"</h2>
                <div class="policy-panel-body"><p>"요청이 처리되었을 수 있습니다. 새로 등록하기 전에 같은 요청의 결과를 확인하세요."</p>
                    <a class="policy-button" href=format!("{}/requests/{command}",directory(&company))>"같은 요청 결과 다시 확인"</a>
                </div>
            </section>
        }.into_any(),
        Page::Refused => view! {<section class="panel"><div class="policy-panel-body"><p>"현재 접근할 수 있는 업무 공간을 확인하세요."</p><a href="/account">"내 업무 공간 확인"</a></div></section>}.into_any(),
        Page::Unavailable => view! {<section class="panel"><div class="policy-panel-body"><p>"잠시 후 다시 열어 주세요. 제출한 요청이 있다면 원래 요청 주소에서 처리 결과를 확인하세요."</p><a href="/account">"내 업무 공간 확인"</a></div></section>}.into_any(),
    };
    let html = view! {
        <html lang="ko"><head><meta charset="utf-8"/><meta name="viewport" content="width=device-width, initial-scale=1"/>
            <title>{format!("{title} · Console")}</title><link rel="stylesheet" href="/assets/workspace.css"/>
        </head><body class="workspace native-people-workspace">
            <a class="skip-link" href="#main-content">"본문 바로가기"</a>
            {header}
            <main id="main-content" tabindex="-1"><div class="page-heading">{company}<h1>{title}</h1></div>{content}</main>
        </body></html>
    }.to_html();
    format!("<!DOCTYPE html>{html}")
}

#[cfg(feature = "ssr")]
pub fn document(page: Page, status: axum::http::StatusCode) -> axum::response::Response {
    let mut response = super::ssr::private_document(
        render(page),
        status,
        "default-src 'self'; script-src 'none'; style-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'",
    );
    response.headers_mut().insert(
        axum::http::header::REFERRER_POLICY,
        axum::http::HeaderValue::from_static("same-origin"),
    );
    response
}
