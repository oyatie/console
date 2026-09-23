//! Server-rendered policy tasks. Every value is a currently authorized projection.
use leptos::prelude::*;

#[derive(Clone, Copy)]
pub enum Operation {
    Install,
    Grant,
    Revoke,
}
impl Operation {
    pub fn path(self) -> &'static str {
        match self {
            Self::Install => "install",
            Self::Grant => "grant",
            Self::Revoke => "revoke",
        }
    }
    fn key(self) -> &'static str {
        match self {
            Self::Install => "InstallPayrollReadCatalogV1",
            Self::Grant => "GrantPayrollReadV1",
            Self::Revoke => "RevokePayrollReadV1",
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::Install => "열람 권한 설정 준비",
            Self::Grant => "열람 권한 연결",
            Self::Revoke => "열람 권한 회수",
        }
    }
}
pub struct Assignment {
    pub id: String,
    pub role_revision: String,
    pub revision: String,
    pub state: &'static str,
    pub label: &'static str,
    pub from: String,
    pub until: String,
    pub can_grant: bool,
    pub can_revoke: bool,
}
pub struct Scope {
    pub company_name: Option<String>,
    pub group: String,
    pub company_link: bool,
    pub policy_link: bool,
    pub payroll_link: bool,
    pub company: String,
    pub operator: String,
    pub recipient: String,
    pub epoch: String,
    pub installed: bool,
    pub assignment: Option<Assignment>,
}
pub struct GrantValidation {
    pub expected_epoch: String,
    pub recipient: String,
    pub assignment: Option<(String, String, String)>,
    pub expires_at_local: String,
    pub current_matches: bool,
}
// Ephemeral form proof intentionally has no Debug or Serialize implementation.
pub struct Form {
    pub scope: Scope,
    pub operation: Operation,
    pub command: String,
    pub proof: String,
    pub validation: Option<GrantValidation>,
}
pub struct OriginalRecord {
    pub accepted_at: String,
    pub deadline: String,
    pub intake_receipt: String,
    pub expected_company_epoch: String,
    pub expected_assignment: Option<(String, String)>,
    pub requested_until: Option<String>,
    pub effect_period: Option<(String, String)>,
    pub effect_epochs: Option<(String, String)>,
}
pub enum Outcome {
    Committed {
        title: &'static str,
        description: &'static str,
        receipt: String,
        at: String,
    },
    Rejected {
        description: &'static str,
        receipt: String,
        at: String,
    },
    Pending {
        accepted_at: String,
        deadline: String,
        proof: String,
    },
    Expired {
        accepted_at: String,
    },
}
pub enum Page {
    Form(Form),
    Result {
        scope: Scope,
        operation: Operation,
        command: String,
        outcome: Outcome,
        original: OriginalRecord,
    },
    Uncertain {
        result_path: String,
    },
    Problem {
        state: &'static str,
        title: &'static str,
        description: &'static str,
    },
    NotVisible,
    Refused,
    Unavailable,
}
impl Scope {
    fn navigation(&self) -> (String, bool, bool, bool) {
        (
            self.company.clone(),
            self.company_link,
            self.policy_link,
            self.payroll_link,
        )
    }
}
fn base(company: &str) -> String {
    format!("/companies/{company}/policy/payroll-read")
}

#[component]
fn Consequences() -> impl IntoView {
    view! {
        <section class="panel policy-consequences" aria-labelledby="disclosure-heading">
            <h2 id="disclosure-heading">"이 권한으로 볼 수 있는 정보"</h2>
            <div class="policy-panel-body">
                <p>"이 계정은 선택한 회사의 급여 목록과 목록에 포함된 모든 항목을 볼 수 있습니다. 근태 마감 증빙 전체와 결정 사유도 포함됩니다."</p>
                <p class="policy-limit">"급여의 상세 내역·수정·지급·내보내기 권한은 연결되지 않습니다. 다른 회사에는 적용되지 않습니다."</p>
            </div>
        </section>
    }
}
fn current(scope: &Scope) -> AnyView {
    let assignment = match &scope.assignment {
        Some(a) => view! {
            <div data-policy-current-state=a.state>
                <p class="policy-state">{a.label}</p>
                <dl class="record-meta"><dt>"시작"</dt><dd>{a.from.clone()}</dd><dt>"종료"</dt><dd>{a.until.clone()}</dd></dl>
                <details class="policy-provenance"><summary>"권한 기록 식별자"</summary>
                    <dl class="record-meta"><dt>"연결"</dt><dd>{a.id.clone()}</dd><dt>"기록 버전"</dt><dd>{a.revision.clone()}</dd></dl>
                </details>
            </div>
        }.into_any(),
        None => view! { <p class="policy-state" data-policy-current-state="NONE">"연결된 열람 권한 없음"</p> }.into_any(),
    };
    view! {
        <section class="panel"><h2>"현재 권한 상태"</h2><div class="policy-panel-body">
            <p class="supporting">"이 페이지를 열 때 다시 확인한 상태입니다. 이전 요청의 처리 결과와 구분해 확인하세요."</p>
            {assignment}
        </div></section>
    }.into_any()
}
fn identities(scope: &Scope) -> AnyView {
    view! {
        <section class="panel"><h2>"대상과 업무 범위"</h2><div class="policy-panel-body">
            <dl class="record-meta">
                <dt>"회사"</dt><dd data-policy-company=scope.company.clone()>{scope.company_name.clone().map(|name| view! {<strong>{name}</strong><br/>})}{scope.company.clone()}</dd>
                <dt>"그룹"</dt><dd>{scope.group.clone()}</dd>
                <dt>"권한 대상"</dt><dd data-policy-recipient=scope.recipient.clone()>{scope.recipient.clone()}</dd>
                <dt>"현재 담당 계정"</dt><dd data-policy-operator=scope.operator.clone()>{scope.operator.clone()}</dd>
            </dl>
            <p class="supporting">"회사 등록 시 지정된 관리 계정에 한해 연결할 수 있습니다."</p>
        </div></section>
    }.into_any()
}
fn next_action(scope: &Scope) -> AnyView {
    let root = base(&scope.company);
    let (path, label) = if !scope.installed {
        ("install", "열람 권한 설정 준비")
    } else if scope.assignment.as_ref().is_none_or(|a| a.can_grant) {
        ("grant", "열람 권한 연결")
    } else {
        ("revoke", "열람 권한 회수")
    };
    view! { <a class="policy-button secondary" href=format!("{root}/{path}")>{label}</a> }
        .into_any()
}
fn form_body(form: Form) -> AnyView {
    let Form {
        scope,
        operation,
        command,
        proof,
        validation,
    } = form;
    let root = base(&scope.company);
    let ready = match operation {
        Operation::Install => !scope.installed,
        Operation::Grant => {
            scope.installed && scope.assignment.as_ref().is_none_or(|a| a.can_grant)
        }
        Operation::Revoke => {
            scope.installed && scope.assignment.as_ref().is_some_and(|a| a.can_revoke)
        }
    };
    let ready = ready && validation.as_ref().is_none_or(|v| v.current_matches);
    let expected_epoch = validation
        .as_ref()
        .map(|v| v.expected_epoch.clone())
        .unwrap_or_else(|| scope.epoch.clone());
    let recipient = validation
        .as_ref()
        .map(|v| v.recipient.clone())
        .unwrap_or_else(|| scope.recipient.clone());
    let error = validation.as_ref().map(|v| view! {
        <div class="policy-validation" role="alert" tabindex="-1">
            <h3>"종료 날짜와 시각을 확인해 주세요"</h3>
            <p id="expiry-error">"한국 표준시 기준으로 유효한 날짜와 시각을 입력하세요. 이 입력으로 요청은 접수되지 않았습니다."</p>
            <p>"입력한 내용: "<code data-policy-retained-expiry>{v.expires_at_local.clone()}</code></p>
            {(!v.current_matches).then(|| view! {
                <p>"작성 중 권한 상태가 변경되었습니다. 입력한 내용은 아래에 보존됩니다. 현재 상태에서 새 요청을 시작하세요."</p>
                <dl class="record-meta"><dt>"원래 요청"</dt><dd>{command.clone()}</dd><dt>"원래 회사 버전"</dt><dd>{v.expected_epoch.clone()}</dd><dt>"원래 권한 대상"</dt><dd>{v.recipient.clone()}</dd>
                    {v.assignment.as_ref().map(|(role,id,revision)|view! {<dt>"원래 연결"</dt><dd>{id.clone()}" · "{role.clone()}" / "{revision.clone()}</dd>})}
                </dl>
            })}
        </div>
    }.into_any());
    let fields = match operation {
        Operation::Install => ().into_any(),
        Operation::Grant | Operation::Revoke => {
            let (role, id, revision) = if let Some(v) = &validation {
                v.assignment.clone().unwrap_or_default()
            } else {
                scope
                    .assignment
                    .as_ref()
                    .map(|a| (a.role_revision.clone(), a.id.clone(), a.revision.clone()))
                    .unwrap_or_default()
            };
            let invalid = validation.is_some();
            let expiry = validation
                .as_ref()
                .map(|v| v.expires_at_local.clone())
                .unwrap_or_default();
            view! {
                <input type="hidden" name="expected_role_revision" value=role/>
                <input type="hidden" name="expected_assignment_revision" value=revision/>
                {matches!(operation, Operation::Grant).then(|| view! {
                    <input type="hidden" name="assignment_id" value=id/>
                    <input type="hidden" name="recipient_account_id" value=recipient/>
                    <label class="policy-label" for="policy-expiry">"종료 날짜와 시각 (한국 표준시, UTC+09:00)"</label>
                    <input id="policy-expiry" name="expires_at_local" type="datetime-local" step="60" required value=expiry aria-invalid=invalid.to_string() aria-describedby=if invalid {"expiry-help expiry-error"} else {"expiry-help"}/>
                    <p id="expiry-help" class="supporting">"권한은 처리가 확정된 시점부터 시작됩니다. 종료 시각은 처리 시점 이후, 30일 이내로 선택하세요."</p>
                })}
            }.into_any()
        }
    };
    let action = match operation {
        Operation::Install => format!("{root}/catalog"),
        Operation::Grant => format!("{root}/grants"),
        Operation::Revoke => format!(
            "{root}/grants/{}/revoke",
            scope
                .assignment
                .as_ref()
                .map(|a| a.id.as_str())
                .unwrap_or_default()
        ),
    };
    let instruction = match operation {
        Operation::Install => {
            "급여 목록 열람 기능을 이 회사에 준비합니다. 준비가 끝난 뒤 대상 계정과 종료 시각을 확인하고 권한을 연결하세요."
        }
        Operation::Grant => {
            "대상 계정과 공개되는 정보를 확인한 뒤, 필요한 기간만 열람 권한을 연결하세요."
        }
        Operation::Revoke => {
            "이 연결을 회수하면 대상 계정은 이 권한으로 회사의 급여 목록을 열 수 없습니다. 이전 처리 기록은 보존됩니다."
        }
    };
    let task = if ready {
        view! {
            <form method="post" action=action data-policy-operation=operation.key()>
                <input type="hidden" name="command_id" value=command/>
                <input type="hidden" name="expected_company_epoch" value=expected_epoch/>
                <input type="hidden" name="csrf_proof" value=proof/>
                {fields}
                <div class="policy-actions"><button class=if matches!(operation,Operation::Revoke) {"policy-button danger"} else {"policy-button"} type="submit">{operation.label()}</button>
                    {scope.policy_link.then(|| view! {<a href=format!("/companies/{}/policy",scope.company)>"권한 관리로 돌아가기"</a>})}
                </div>
            </form>
        }.into_any()
    } else {
        view! { <p>"현재 권한 상태에 맞는 작업을 선택하세요."</p>{next_action(&scope)} }.into_any()
    };
    view! {
        <div data-policy-preflight-operation=operation.path() class="workflow-layout">
            <aside class="workflow-guide" aria-label="권한 대상과 현재 상태">{identities(&scope)}{current(&scope)}</aside>
            <div class="workflow-main">
                <Consequences/>
                <section class="panel"><h2>{operation.label()}</h2><div class="policy-panel-body"><p>{instruction}</p>{matches!(operation, Operation::Install).then(|| view! { <p>"설정을 준비해도 계정에 열람 권한이 연결되지는 않습니다."</p> })}{error}{task}</div></section>
            </div>
        </div>
    }.into_any()
}
fn result_body(
    scope: Scope,
    operation: Operation,
    command: String,
    outcome: Outcome,
    original: OriginalRecord,
) -> AnyView {
    let path = format!(
        "{}/requests/{}/{command}",
        base(&scope.company),
        operation.path()
    );
    let (state,title,description,receipt,at,extra) = match outcome {
        Outcome::Committed {title,description,receipt,at} => ("committed",title,description,Some(receipt),at,next_action(&scope)),
        Outcome::Rejected {description,receipt,at} => ("rejected","요청이 적용되지 않았습니다",description,Some(receipt),at,next_action(&scope)),
        Outcome::Pending {accepted_at,deadline,proof} => ("pending","접수한 요청을 이어서 처리하세요","요청을 접수했지만 권한 변경은 아직 확정되지 않았습니다.",None,accepted_at,view! {
            <p>"처리 기한: "{deadline}</p><form method="post" action=format!("{path}/retry") data-policy-recovery="accepted-retry">
                <input type="hidden" name="csrf_proof" value=proof/>
                <button class="policy-button" type="submit">"같은 요청 이어서 처리"</button>
            </form>
        }.into_any()),
        Outcome::Expired {accepted_at} => ("expired","이 요청의 처리 기한이 지났습니다","이 요청은 이어서 처리할 수 없습니다. 현재 권한 상태를 확인하고 필요한 작업을 새로 시작하세요.",None,accepted_at,next_action(&scope)),
    };
    let history = view! {
                        <details class="policy-provenance"><summary>"이 요청의 처리 기록"</summary>
                            <dl class="record-meta"><dt>"요청"</dt><dd>{command.clone()}</dd><dt>"기록 시각"</dt><dd>{at}</dd>
                                <dt>"접수 시각"</dt><dd>{original.accepted_at}</dd><dt>"처리 기한"</dt><dd>{original.deadline}</dd>
                                <dt>"접수 기록"</dt><dd>{original.intake_receipt}</dd><dt>"요청 당시 회사 버전"</dt><dd>{original.expected_company_epoch}</dd>
                                {original.expected_assignment.map(|(id,revision)|view! {<dt>"요청 당시 연결"</dt><dd>{id}" · "{revision}</dd>})}
                                {original.effect_epochs.map(|(before,after)|view! {<dt>"처리 전 회사 버전"</dt><dd>{before}</dd><dt>"처리 후 회사 버전"</dt><dd>{after}</dd>})}
                                {receipt.map(|id| view! { <dt>"처리 기록"</dt><dd data-policy-receipt=id.clone()>{id.clone()}</dd> })}
                            </dl>
                            <a href=path>"이 요청 다시 열기"</a>
                        </details>
    }.into_any();
    view! {
        <div class="workflow-layout" data-policy-result-layout>
            <aside class="workflow-guide" aria-label="권한 대상과 현재 상태">{identities(&scope)}{current(&scope)}</aside>
            <div class="workflow-main">
                <Consequences/>
                <section class="panel policy-result" data-policy-outcome=state data-policy-command=command.clone()>
                    <h2>{title}</h2><div class="policy-panel-body"><p>{description}</p>
                        {original.effect_period.map(|(from,until)| view! {<dl class="record-meta"><dt>"확정된 시작"</dt><dd>{from}</dd><dt>"확정된 종료"</dt><dd>{until}</dd></dl>})}
                        {original.requested_until.as_ref().map(|until|view! {<p>"이 요청에서 선택한 종료 시각: "<strong>{until.clone()}</strong></p>})}
                        <div class="policy-actions">{extra}</div>
                        {history}
                    </div>
                </section>
            </div>
        </div>
    }.into_any()
}
pub fn render(page: Page) -> String {
    let company = match &page {
        Page::Form(f) => Some(f.scope.navigation()),
        Page::Result { scope, .. } => Some(scope.navigation()),
        _ => None,
    };
    let content = match page {
        Page::Form(form)=>form_body(form),
        Page::Result{scope,operation,command,outcome,original}=>result_body(scope,operation,command,outcome,original),
        Page::Uncertain{result_path}=>view! { <section class="panel" data-policy-outcome="uncertain"><h2>"아직 처리 결과를 확인할 수 없습니다"</h2><div class="policy-panel-body"><p>"새 요청을 만들기 전에 같은 요청의 기록을 다시 확인하세요."</p><a class="policy-button" href=result_path>"같은 요청 결과 다시 확인"</a></div></section> }.into_any(),
        Page::Problem{state,title,description}=>view! {<section class="panel" data-policy-outcome=state><h2>{title}</h2><div class="policy-panel-body"><p>{description}</p><a href="/account">"내 업무 공간 확인"</a></div></section>}.into_any(),
        Page::NotVisible=>view! { <section class="panel" data-policy-outcome="not-visible"><h2>"현재 확인할 수 있는 처리 기록이 없습니다"</h2><p class="policy-panel-body">"기록이 보이지 않는다고 요청이 처리되지 않았다는 뜻은 아닙니다. 담당자에게 확인해 주세요."</p></section> }.into_any(),
        Page::Refused=>view! { <section class="panel" data-policy-outcome="denied"><h2>"이 요청을 열거나 변경할 권한이 없습니다"</h2><p class="policy-panel-body">"내 계정에서 접근할 수 있는 업무 공간을 확인하세요."</p></section> }.into_any(),
        Page::Unavailable=>view! { <section class="panel" data-policy-outcome="unavailable"><h2>"지금 요청을 확인할 수 없습니다"</h2><div class="policy-panel-body"><p>"연결이 복구된 뒤 다시 확인해 주세요."</p><a href="/account">"내 업무 공간 확인"</a></div></section> }.into_any(),
    };
    let nav = company.map(|(id, company_link, policy_link, payroll_link)| view! {
        <p class="nav-group">"관리"</p><nav aria-label="회사 업무 탐색">
            {company_link.then(||view! {<a href=format!("/companies/{id}")>"회사 업무 공간"</a>})}
            {policy_link.then(||view! {<a href=format!("/companies/{id}/policy") aria-current="page">"권한 관리"</a>})}
        </nav>
        {payroll_link.then(||view! {<p class="nav-group">"사람과 조직"</p><nav aria-label="급여 탐색"><a href=format!("/companies/{id}/payroll")>"급여"</a></nav>})}
    });
    let html=view! {
        <html lang="ko"><head><meta charset="utf-8"/><meta name="viewport" content="width=device-width, initial-scale=1"/>
            <title>"급여 목록 열람 권한 · Console"</title><link rel="stylesheet" href="/assets/workspace.css"/>
        </head><body class="workspace policy-workspace">
            <a class="skip-link" href="#main-content">"본문 바로가기"</a>
            <header class="app"><a class="brand" href="/"><span class="brand-mark" aria-hidden="true">"C"</span>"Console"</a>{nav}
                <nav class="policy-account-nav" aria-label="계정 탐색"><a href="/account">"내 계정 · 업무 공간 선택"</a></nav>
            </header>
            <main id="main-content" tabindex="-1"><div class="page-heading"><p class="page-eyebrow">"권한 관리 / 급여"</p><h1>"급여 목록 열람 권한"</h1>
                <p class="page-description">"누가, 어떤 정보를, 언제까지 볼 수 있는지 확인하고 관리하세요."</p>
            </div>{content}</main>
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
    // Native form POSTs need their same-origin Origin; no-referrer makes it null.
    response.headers_mut().insert(
        axum::http::header::REFERRER_POLICY,
        axum::http::HeaderValue::from_static("same-origin"),
    );
    response
}
