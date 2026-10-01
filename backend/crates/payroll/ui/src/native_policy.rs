//! Server-rendered policy tasks. Every value is a currently authorized projection.
use super::native_workspace_header::{self, NavigationMode};
use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Subject {
    PayrollRead,
    PeopleCatalog,
    PeopleRead,
    PeopleCreate,
}
impl Subject {
    pub fn base(self, company: &str) -> String {
        let catalog = if self == Self::PayrollRead {
            "payroll-read"
        } else {
            "people-directory"
        };
        format!("/companies/{company}/policy/{catalog}")
    }
    fn action_base(self, company: &str) -> String {
        let base = self.base(company);
        match self {
            Self::PeopleRead => format!("{base}/read"),
            Self::PeopleCreate => format!("{base}/create"),
            _ => base,
        }
    }
    fn title(self) -> &'static str {
        match self {
            Self::PayrollRead => "급여 목록 열람 권한",
            Self::PeopleCatalog => "사람 등록·열람 권한",
            Self::PeopleRead => "사람 열람 권한",
            Self::PeopleCreate => "사람 등록 권한",
        }
    }
    pub fn label(self, op: Operation) -> &'static str {
        match (self, op) {
            (Self::PayrollRead, op) => op.label(),
            (_, Operation::Install) => "권한 설정 준비",
            (Self::PeopleRead, Operation::Grant) => "사람 열람 권한 연결",
            (Self::PeopleRead, Operation::Revoke) => "사람 열람 권한 회수",
            (Self::PeopleCreate, Operation::Grant) => "사람 등록 권한 연결",
            (Self::PeopleCreate, Operation::Revoke) => "사람 등록 권한 회수",
            (Self::PeopleCatalog, _) => "권한 설정 준비",
        }
    }
    fn key(self, op: Operation) -> &'static str {
        if self == Self::PayrollRead {
            return op.key();
        }
        match op {
            Operation::Install => "InstallPeopleDirectoryCatalogV1",
            Operation::Grant => "GrantPeopleDirectoryV1",
            Operation::Revoke => "RevokePeopleDirectoryV1",
        }
    }
    fn instruction(self, op: Operation) -> &'static str {
        match (self, op) {
            (Self::PayrollRead, Operation::Install) => {
                "급여 목록 열람 기능을 이 회사에 준비합니다. 준비가 끝난 뒤 대상 계정과 종료 시각을 확인하고 권한을 연결하세요."
            }
            (Self::PayrollRead, Operation::Grant) => {
                "대상 계정과 공개되는 정보를 확인한 뒤, 필요한 기간만 열람 권한을 연결하세요."
            }
            (Self::PayrollRead, Operation::Revoke) => {
                "이 연결을 회수하면 대상 계정은 이 권한으로 회사의 급여 목록을 열 수 없습니다. 이전 처리 기록은 보존됩니다."
            }
            (_, Operation::Install) => {
                "사람 등록과 열람 권한을 각각 관리할 수 있도록 준비합니다. 준비만으로 어떤 계정에도 권한이 연결되지 않습니다."
            }
            (Self::PeopleCreate, Operation::Grant) => {
                "대상 계정과 등록 범위를 확인한 뒤, 필요한 기간만 사람 등록 권한을 연결하세요."
            }
            (Self::PeopleCreate, Operation::Revoke) => {
                "이 권한으로 새 사람을 등록하거나 본인의 등록 요청을 확인할 수 없게 됩니다. 확정된 등록과 처리 기록은 보존됩니다."
            }
            (_, Operation::Grant) => {
                "대상 계정과 공개되는 정보를 확인한 뒤, 필요한 기간만 사람 열람 권한을 연결하세요."
            }
            (_, Operation::Revoke) => {
                "이 권한으로 사람 목록과 상세 정보를 열 수 없게 됩니다. 등록 권한은 별도로 관리하며, 이전 처리 기록은 보존됩니다."
            }
        }
    }
}
pub struct PolicyAction {
    pub subject: Subject,
    pub operation: Operation,
}
pub fn people_action_links(company: &str, actions: &[PolicyAction]) -> AnyView {
    actions.iter().map(|a| view! {
        <a class="policy-button secondary" href=format!("{}/{}", a.subject.action_base(company), a.operation.path())>{a.subject.label(a.operation)}</a>
    }).collect_view().into_any()
}

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
    pub subject: Subject,
    pub people_actions: Vec<PolicyAction>,
    pub people_navigation: (bool, bool),
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
    fn navigation(&self) -> (String, bool, bool, bool, (bool, bool)) {
        (
            self.company.clone(),
            self.company_link,
            self.policy_link,
            self.payroll_link,
            self.people_navigation,
        )
    }
}
#[component]
fn Consequences(subject: Subject) -> impl IntoView {
    let (heading, detail, limit) = match subject {
        Subject::PayrollRead => (
            "이 권한의 공개 범위",
            "권한이 연결되어 유효하고 현재 정책이 허용할 때, 대상 관리 계정이 선택한 회사의 급여 목록과 목록에 포함된 모든 항목을 확인할 수 있습니다. 근태 마감 증빙 전체와 결정 사유도 포함됩니다.",
            "급여의 상세 내역·수정·지급·내보내기 권한은 연결되지 않습니다. 다른 회사에는 적용되지 않습니다.",
        ),
        Subject::PeopleCatalog => (
            "등록과 열람을 각각 관리합니다",
            "이 회사의 사람 등록과 열람 기능을 준비합니다. 각 권한은 대상 계정과 기간을 확인한 뒤 따로 연결합니다.",
            "설정 준비만으로 사람 정보가 공개되거나 등록 권한이 연결되지는 않습니다.",
        ),
        Subject::PeopleRead => (
            "이 권한의 공개 범위",
            "권한이 연결되어 유효하고 현재 정책이 허용할 때, 대상 관리 계정이 이 회사에 등록된 사람의 이름, 사번, 식별자와 등록 기록을 확인할 수 있습니다.",
            "등록·고용 변경·급여 권한은 포함되지 않습니다. 다른 회사에는 적용되지 않습니다.",
        ),
        Subject::PeopleCreate => (
            "이 권한의 공개 범위",
            "권한이 연결되어 유효하고 현재 정책이 허용할 때, 대상 관리 계정이 이 회사에 이름과 사번으로 사람을 등록하고 본인이 접수한 등록 요청의 처리 결과를 확인할 수 있습니다.",
            "사람 목록 열람이나 고용·급여 권한은 별도입니다. 다른 회사에는 적용되지 않습니다.",
        ),
    };
    view! {
        <section class="panel policy-consequences" aria-labelledby="disclosure-heading">
            <h2 id="disclosure-heading">{heading}</h2><div class="policy-panel-body">
                <p>{detail}</p><p class="policy-limit">{limit}</p>
            </div>
        </section>
    }
}
fn current(scope: &Scope) -> AnyView {
    if scope.subject == Subject::PeopleCatalog {
        return view! {
            <section class="panel"><h2>"현재 권한 설정"</h2><div class="policy-panel-body">
                <p>{if scope.installed { "사람 등록·열람 기능이 준비되었습니다. 각 권한의 현재 상태를 열어 확인하세요." } else { "사람 등록·열람 기능을 준비해야 합니다." }}</p>
                <ul>{scope.people_actions.iter().map(|action| view! { <li>{action.subject.title()}": "{if matches!(action.operation, Operation::Revoke) { "연결됨" } else { "연결 가능" }}</li> }).collect_view()}</ul>
            </div></section>
        }.into_any();
    }
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
        None => view! { <p class="policy-state" data-policy-current-state="NONE">"연결된 권한 없음"</p> }.into_any(),
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
                <dt>"회사"</dt><dd><strong>{scope.company_name.clone().unwrap_or_else(|| "선택한 회사".into())}</strong></dd>
                <dt>"권한 대상"</dt><dd>"회사 등록 시 지정된 관리 계정"</dd>
                <dt>"작업 담당"</dt><dd>"현재 로그인 계정"</dd>
            </dl>
            <p class="supporting">"회사 등록 시 지정된 관리 계정에 한해 연결할 수 있습니다."</p>
            <details class="policy-provenance"><summary>"회사·계정 식별 정보"</summary>
                <dl class="record-meta">
                    <dt>"회사"</dt><dd data-policy-company=scope.company.clone()>{scope.company.clone()}</dd>
                    <dt>"그룹"</dt><dd>{scope.group.clone()}</dd>
                    <dt>"권한 대상"</dt><dd data-policy-recipient=scope.recipient.clone()>{scope.recipient.clone()}</dd>
                    <dt>"현재 담당 계정"</dt><dd data-policy-operator=scope.operator.clone()>{scope.operator.clone()}</dd>
                </dl>
            </details>
        </div></section>
    }.into_any()
}
fn next_action(scope: &Scope) -> AnyView {
    if scope.subject != Subject::PayrollRead && scope.installed {
        return people_action_links(&scope.company, &scope.people_actions);
    }
    let root = if scope.installed {
        scope.subject.action_base(&scope.company)
    } else {
        scope.subject.base(&scope.company)
    };
    let operation = if !scope.installed {
        Operation::Install
    } else if scope.assignment.as_ref().is_none_or(|a| a.can_grant) {
        Operation::Grant
    } else {
        Operation::Revoke
    };
    view! { <a class="policy-button secondary" href=format!("{root}/{}", operation.path())>{scope.subject.label(operation)}</a> }.into_any()
}

fn form_body(form: Form) -> AnyView {
    let Form {
        scope,
        operation,
        command,
        proof,
        validation,
    } = form;
    let root = scope.subject.action_base(&scope.company);
    let subject = scope.subject;
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
    let instruction = subject.instruction(operation);
    let task = if ready {
        view! {
            <form method="post" action=action data-policy-operation=subject.key(operation)>
                <input type="hidden" name="command_id" value=command/>
                <input type="hidden" name="expected_company_epoch" value=expected_epoch/>
                <input type="hidden" name="csrf_proof" value=proof/>
                {fields}
                <div class="policy-actions"><button class=if matches!(operation,Operation::Revoke) {"policy-button danger"} else {"policy-button"} type="submit">{subject.label(operation)}</button>
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
                <Consequences subject=scope.subject/>
                <section class="panel"><h2>{subject.label(operation)}</h2><div class="policy-panel-body"><p>{instruction}</p>{matches!(operation, Operation::Install).then(|| view! { <p>{if subject == Subject::PayrollRead { "설정을 준비해도 계정에 열람 권한이 연결되지는 않습니다." } else { "설정을 준비해도 계정에 권한이 연결되지는 않습니다." }}</p> })}{error}{task}</div></section>
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
        scope.subject.base(&scope.company),
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
                <Consequences subject=scope.subject/>
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
    let subject = match &page {
        Page::Form(f) => Some(f.scope.subject),
        Page::Result { scope, .. } => Some(scope.subject),
        _ => None,
    };
    let title = subject.map(Subject::title).unwrap_or("권한 관리");
    let eyebrow = if subject == Some(Subject::PayrollRead) {
        "권한 관리 / 급여"
    } else if subject.is_some() {
        "권한 관리 / 사람"
    } else {
        "권한 관리"
    };
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
    let payroll_shortcut = company.as_ref().and_then(|(id, _, _, payroll_link, _)| {
        payroll_link.then(|| (format!("/companies/{id}/payroll"), false))
    });
    let header = native_workspace_header::render(
        |mode| {
            company.as_ref().map(|(id, company_link, policy_link, payroll_link, (people_read, people_create))| view! {
        <p class="nav-group">"관리"</p><nav aria-label="회사 업무 탐색">
            {company_link.then(||view! {<a href=format!("/companies/{id}")>"회사 업무 공간"</a>})}
            {policy_link.then(||view! {<a href=format!("/companies/{id}/policy")>"권한 관리"</a>})}
        </nav>
        {(*payroll_link || *people_read || *people_create).then(||view! {<p class="nav-group">"사람과 조직"</p><nav aria-label="사람과 조직 탐색">{people_read.then(||view! {<a href=format!("/companies/{id}/people")>"사람"</a>})}{people_create.then(||view! {<a href=format!("/companies/{id}/people/new")>"사람 등록"</a>})}{(*payroll_link && mode == NavigationMode::Desktop).then(||view! {<a href=format!("/companies/{id}/payroll")>"급여"</a>})}</nav>})}
    }.into_any()).unwrap_or_else(|| ().into_any())
        },
        payroll_shortcut,
    );
    let html=view! {
        <html lang="ko"><head><meta charset="utf-8"/><meta name="viewport" content="width=device-width, initial-scale=1"/>
            <title>{format!("{title} · Console")}</title><link rel="stylesheet" href="/assets/workspace.css"/>
        </head><body class="workspace policy-workspace">
            <a class="skip-link" href="#main-content">"본문 바로가기"</a>
            {header}
            <main id="main-content" tabindex="-1"><div class="page-heading"><p class="page-eyebrow">{eyebrow}</p><h1>{title}</h1>
                <p class="page-description">"대상 계정과 업무 범위, 권한이 적용되는 기간을 확인하고 관리하세요."</p>
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
