//! Native Group procedure pages. Callers supply finalized authorized projections;
//! this presentation neither establishes authority nor changes business state.
use super::native_workspace_header;
use leptos::prelude::*;

pub struct Scope {
    pub group: String,
    pub group_name: String,
    pub incarnation: String,
    pub revision: String,
    pub policy_revision: String,
    pub operator: String,
}

pub struct Content {
    pub title: String,
    pub method: String,
    pub intended_claimant_matching_procedure: String,
    pub account_possession_procedure: String,
    pub physical_human_evidence_procedure: String,
    pub duplicate_contradictory_claim_procedure: String,
    pub qualification_criteria_instruction: String,
    pub escalation_adjudication_procedure: String,
    pub evidence_minimization_retention_description: String,
    pub recipient_responsibility: String,
}

pub struct HeadReference {
    pub process: String,
    pub content_version: String,
    pub head_revision: String,
    pub content_digest: String,
    pub head_digest: String,
    /// Actual persisted state, distinct from current time-based usability.
    pub state: &'static str,
    pub expires_at: String,
}

pub struct Head {
    pub reference: HeadReference,
    pub causing_command: String,
    pub causing_receipt: String,
}

pub struct History {
    pub label: &'static str,
    pub recorded_at: String,
    pub content_version: String,
    pub head_revision: String,
    pub command: String,
    pub reason: Option<String>,
}

pub struct Current {
    pub scope: Scope,
    pub head: Head,
    pub content: Content,
    pub observed_at: String,
    pub usable: bool,
    pub can_replace: bool,
    pub can_suspend: bool,
    pub history: Vec<History>,
}

/// These are the original expected pins, never substituted with a newer state.
pub struct Expectations {
    pub group_revision: String,
    pub group_incarnation: String,
    pub policy_revision: String,
}

pub struct FieldError {
    pub field: &'static str,
    pub message: &'static str,
}

pub enum Input {
    Adopt {
        process: String,
        expected_prior_head_revision: String,
        replacing: bool,
        content: Content,
        expires_at_local: String,
        responsibility_accepted: bool,
    },
    Suspend {
        process: String,
        content_version: String,
        content_digest: String,
        expected_head_revision: String,
        expected_head_digest: String,
        reason: String,
    },
}

// The ephemeral Auth proof has no Debug or serialization implementation here.
pub struct Form {
    pub scope: Scope,
    pub command: String,
    pub expected: Expectations,
    pub proof: String,
    pub input: Input,
    pub current: Option<Head>,
    pub errors: Vec<FieldError>,
    pub form_error: Option<&'static str>,
}

/// Historical input and receipt data only. No current Group eligibility belongs
/// in this projection: a same-Account receipt survives designation loss.
pub struct OriginalRecord {
    pub group: String,
    pub incarnation: String,
    pub command: String,
    pub actor: String,
    pub process: String,
    pub operation_label: &'static str,
    pub accepted_at: String,
    pub intake_receipt: String,
    pub input_digest: String,
    pub expected: Expectations,
    pub content: Option<Content>,
    pub requested_expiry: Option<String>,
    pub suspension_reason: Option<String>,
}

pub enum Outcome {
    Pending,
    Terminal {
        code: &'static str,
        title: &'static str,
        description: &'static str,
        receipt: String,
        executed_at: String,
        before: Option<HeadReference>,
        after: Option<HeadReference>,
    },
}

pub struct Request {
    pub original: OriginalRecord,
    pub outcome: Outcome,
}

pub enum Page {
    Empty {
        scope: Scope,
        can_adopt: bool,
    },
    Current(Current),
    Form(Form),
    Request(Request),
    OwnRetry {
        group: String,
        command: String,
        proof: String,
    },
    Uncertain {
        group: String,
        command: String,
    },
    Problem {
        title: &'static str,
        description: &'static str,
    },
    NotVisible,
    Refused,
    Unavailable,
}

fn base(group: &str) -> String {
    format!("/groups/{group}/identity")
}

fn request_path(group: &str, command: &str) -> String {
    format!("{}/requests/{command}", base(group))
}

fn consequences() -> AnyView {
    view! {
        <section class="panel group-consequences" aria-labelledby="group-consequences-heading">
            <h2 id="group-consequences-heading">"이 절차가 정하는 범위"</h2>
            <div class="policy-panel-body">
                <p>"신청자와 계정, 확인 자료를 어떤 순서와 기준으로 대조할지 정합니다. 확인 담당자는 실제로 확인한 사실과 남은 불확실성을 구분해야 합니다."</p>
                <p class="policy-limit">"절차를 등록해도 계정과 사람의 동일성이 확인되거나 검증 담당자 권한이 연결되지는 않습니다. 고용·발령·급여 권한도 변경되지 않습니다."</p>
            </div>
        </section>
    }.into_any()
}

fn scope_summary(scope: &Scope) -> AnyView {
    view! {
        <section class="panel"><h2>"업무 범위와 담당"</h2><div class="policy-panel-body">
            <dl class="record-meta"><dt>"그룹"</dt><dd><strong>{scope.group_name.clone()}</strong></dd>
                <dt>"작업 담당"</dt><dd>"현재 지정된 운영 계정"</dd>
            </dl>
            <p class="supporting">"그룹 범위의 확인 절차입니다. 개별 회사의 사람·고용 기록과 권한은 별도로 관리합니다."</p>
            <details class="policy-provenance"><summary>"그룹·계정 식별 정보"</summary>
                <dl class="record-meta"><dt>"그룹"</dt><dd>{scope.group.clone()}</dd>
                    <dt>"그룹 식별 버전"</dt><dd>{scope.incarnation.clone()}</dd>
                    <dt>"그룹 기록 버전"</dt><dd>{scope.revision.clone()}</dd>
                    <dt>"정책 기록 버전"</dt><dd>{scope.policy_revision.clone()}</dd>
                    <dt>"담당 계정"</dt><dd>{scope.operator.clone()}</dd>
                </dl>
            </details>
        </div></section>
    }.into_any()
}

fn head_summary(head: &Head) -> AnyView {
    view! {
        <section class="panel"><h2>"비교할 절차 기록"</h2><div class="policy-panel-body">
            <dl class="record-meta"><dt>"저장 상태"</dt><dd>{if head.reference.state == "SUSPENDED" {"중단됨"} else {"등록됨"}}</dd>
                <dt>"유효 기한"</dt><dd>{head.reference.expires_at.clone()}</dd>
                <dt>"내용 버전"</dt><dd>{head.reference.content_version.clone()}</dd>
                <dt>"상태 버전"</dt><dd>{head.reference.head_revision.clone()}</dd>
            </dl>
            <p class="supporting">"제출 시 원래 기대한 기록과 현재 기록을 다시 비교합니다. 다른 작업이 먼저 확정되면 이 요청은 적용되지 않을 수 있습니다."</p>
        </div></section>
    }.into_any()
}

fn procedure(content: &Content) -> AnyView {
    let fields = [
        ("신청자 대조", &content.intended_claimant_matching_procedure),
        ("계정 소유 확인", &content.account_possession_procedure),
        (
            "실제 사람 확인 자료",
            &content.physical_human_evidence_procedure,
        ),
        (
            "중복·상충 주장 처리",
            &content.duplicate_contradictory_claim_procedure,
        ),
        (
            "검증 업무 제안 기준",
            &content.qualification_criteria_instruction,
        ),
        (
            "추가 검토와 판단",
            &content.escalation_adjudication_procedure,
        ),
        (
            "자료 최소화와 보관",
            &content.evidence_minimization_retention_description,
        ),
        ("확인 담당자의 책임", &content.recipient_responsibility),
    ];
    view! {
        <dl class="group-procedure"><dt>"확인 방법"</dt><dd>"대면·계정·자료 확인"</dd>
            {fields.into_iter().map(|(label, text)| view! {<dt>{label}</dt><dd>{text.clone()}</dd>}).collect_view()}
        </dl>
    }.into_any()
}

fn empty(scope: Scope, can_adopt: bool) -> AnyView {
    let destination = format!("{}/processes/new", base(&scope.group));
    view! {
        <div class="group-workflow-layout">
            <div class="workflow-main"><section class="panel group-empty"><h2>"아직 등록된 확인 절차가 없습니다"</h2><div class="policy-panel-body">
                <p>"신청자 대조, 계정 소유 확인, 자료 취급과 예외 처리 기준을 먼저 정하세요. 등록된 절차는 같은 주소에서 다시 열어 확인할 수 있습니다."</p>
                {can_adopt.then(||view! {<a class="policy-button" href=destination>"확인 절차 등록"</a>})}
            </div></section>{consequences()}</div>
            <aside class="workflow-guide" aria-label="그룹 업무 범위">{scope_summary(&scope)}</aside>
        </div>
    }.into_any()
}

fn current(current: Current) -> AnyView {
    let Current {
        scope,
        head,
        content,
        observed_at,
        usable,
        can_replace,
        can_suspend,
        history,
    } = current;
    let process_path = format!(
        "{}/processes/{}",
        base(&scope.group),
        head.reference.process
    );
    let state_label = if head.reference.state == "SUSPENDED" {
        "중단됨"
    } else if usable {
        "사용 가능"
    } else {
        "유효 기한 지남"
    };
    let receipt_path = request_path(&scope.group, &head.causing_command);
    view! {
        <div class="group-workflow-layout">
            <div class="workflow-main">
                <section class="panel" data-group-process-state=head.reference.state data-group-process-id=head.reference.process.clone()
                    data-group-process-version=head.reference.content_version.clone() data-group-process-head-revision=head.reference.head_revision.clone()>
                    <h2>"등록된 확인 절차"</h2><div class="policy-panel-body">
                        <div class="group-status-line"><span class="badge" data-status=if usable {"ACTIVE"} else {"BLOCKED"}>{state_label}</span>
                            <span class="supporting">"확인 시각: "{observed_at}</span></div>
                        <dl class="record-meta group-period"><dt>"유효 기한"</dt><dd>{head.reference.expires_at.clone()}</dd>
                            <dt>"내용 버전"</dt><dd>{head.reference.content_version.clone()}</dd><dt>"상태 버전"</dt><dd>{head.reference.head_revision.clone()}</dd></dl>
                        {(!usable).then(||view! {<p class="group-state-note">"이 절차로 새 확인 업무를 진행할 수 없습니다. 내용을 보완하려면 현재 상태를 기준으로 수정 등록하세요."</p>})}
                        {procedure(&content)}
                        <div class="policy-actions">
                            {can_replace.then(||view! {<a class="policy-button" href=format!("{process_path}/replace")>"확인 절차 수정 등록"</a>})}
                            {can_suspend.then(||view! {<a class="policy-button secondary" href=format!("{process_path}/suspend")>"확인 절차 중단"</a>})}
                        </div>
                        <details class="policy-provenance"><summary>"현재 기록 식별 정보"</summary>
                            <dl class="record-meta"><dt>"절차"</dt><dd>{head.reference.process.clone()}</dd><dt>"내용 지문"</dt><dd>{head.reference.content_digest}</dd>
                                <dt>"상태 지문"</dt><dd>{head.reference.head_digest}</dd><dt>"처리 기록"</dt><dd>{head.causing_receipt}</dd></dl>
                            <a href=receipt_path>"이 변경의 원래 요청"</a>
                        </details>
                    </div>
                </section>
                <section class="panel"><h2>"변경 기록"</h2><div class="policy-panel-body">
                    <p class="supporting">"중단은 내용을 덮어쓰지 않습니다. 수정 등록도 이전 요청과 내용을 보존합니다."</p>
                    <ol class="group-history">{history.into_iter().map(|record| view! {
                        <li><div class="group-history-heading"><strong>{record.label}</strong><span>{record.recorded_at}</span></div>
                            <p>"내용 버전 "{record.content_version}" · 상태 버전 "{record.head_revision}</p>
                            {record.reason.map(|reason|view! {<p class="group-preserved-text">{reason}</p>})}
                            <a href=request_path(&scope.group,&record.command)>"원래 요청 기록"</a>
                        </li>
                    }).collect_view()}</ol>
                </div></section>
            </div>
            <aside class="workflow-guide" aria-label="그룹 업무 범위와 적용 범위">{scope_summary(&scope)}{consequences()}</aside>
        </div>
    }.into_any()
}

#[component]
fn ProcedureField(
    name: &'static str,
    label: &'static str,
    help: &'static str,
    value: String,
    error: Option<&'static str>,
) -> impl IntoView {
    let help_id = format!("{name}-help");
    let error_id = format!("{name}-error");
    let described_by = if error.is_some() {
        format!("{help_id} {error_id}")
    } else {
        help_id.clone()
    };
    // HTML parsing consumes the first LF immediately after <textarea>. Supply
    // that parser LF separately so the actual preserved value stays exact.
    let value = format!("\n{value}");
    view! {
        <div class="group-field"><label for=name>{label}</label><p id=help_id class="supporting">{help}</p>
            <textarea id=name name=name rows="3" required maxlength="2048" aria-describedby=described_by
                aria-invalid=error.is_some().to_string()>{value}</textarea>
            {error.map(|message|view! {<p class="group-field-error" id=error_id>{message}</p>})}
        </div>
    }
}

fn adopt_fields(
    process: String,
    expected_prior_head_revision: String,
    content: Content,
    expires_at_local: String,
    responsibility_accepted: bool,
    errors: &[FieldError],
) -> AnyView {
    let error = |field| {
        errors
            .iter()
            .find(|error| error.field == field)
            .map(|error| error.message)
    };
    let title_error = error("title");
    let method_error = error("method");
    let expiry_error = error("process_expiry");
    let responsibility_error = error("operator_responsibility");
    view! {
        <input type="hidden" name="process_id" value=process/>
        <input type="hidden" name="expected_prior_process_revision" value=expected_prior_head_revision/>
        <fieldset class="group-fieldset"><legend>"1. 절차 이름과 확인 방법"</legend>
            <div class="group-field"><label for="title">"절차 이름"</label>
                <input id="title" name="title" type="text" required maxlength="120" value=content.title
                    aria-invalid=title_error.is_some().to_string() aria-describedby=if title_error.is_some() {"title-help title-error"} else {"title-help"}/>
                <p id="title-help" class="supporting">"이 절차의 용도를 알아볼 수 있는 이름을 입력하세요. UTF-8 기준 120바이트 이내입니다."</p>
                {title_error.map(|message|view! {<p id="title-error" class="group-field-error">{message}</p>})}
            </div>
            <div class="group-field"><label for="method">"확인 방법"</label>
                <select id="method" name="method" required aria-invalid=method_error.is_some().to_string()
                    aria-describedby=if method_error.is_some() {"method-help method-error"} else {"method-help"}>
                    <option value="" selected=content.method.is_empty()>"확인 방법 선택"</option>
                    <option value="ATTENDED_ACCOUNT_AND_DOCUMENTARY_REVIEW_V1" selected=content.method == "ATTENDED_ACCOUNT_AND_DOCUMENTARY_REVIEW_V1">"대면·계정·자료 확인"</option>
                </select>
                <p id="method-help" class="supporting">"신청자와 직접 확인하며 계정 소유와 확인 자료를 함께 대조하는 방식입니다."</p>
                {method_error.map(|message|view! {<p id="method-error" class="group-field-error">{message}</p>})}
            </div>
        </fieldset>
        <fieldset class="group-fieldset"><legend>"2. 신청자와 계정 확인"</legend>
            <ProcedureField name="intended_claimant_matching_procedure" label="신청자 대조 절차" help="신청 의사와 자료의 당사자를 어떻게 대조할지 작성하세요." value=content.intended_claimant_matching_procedure error=error("intended_claimant_matching_procedure")/>
            <ProcedureField name="account_possession_procedure" label="계정 소유 확인 절차" help="신청자가 해당 계정을 직접 소유·사용하는지 확인할 방법을 작성하세요." value=content.account_possession_procedure error=error("account_possession_procedure")/>
            <ProcedureField name="physical_human_evidence_procedure" label="실제 사람 확인 자료" help="확인할 자료의 종류와 실제로 확인할 사실을 작성하세요. 계정만으로 동일인을 추정하지 마세요." value=content.physical_human_evidence_procedure error=error("physical_human_evidence_procedure")/>
            <ProcedureField name="qualification_criteria_instruction" label="검증 업무 제안 기준" help="어떤 사실이 확인되어야 검증 업무를 제안할 수 있는지 작성하세요." value=content.qualification_criteria_instruction error=error("qualification_criteria_instruction")/>
        </fieldset>
        <fieldset class="group-fieldset"><legend>"3. 예외 처리와 자료 책임"</legend>
            <ProcedureField name="duplicate_contradictory_claim_procedure" label="중복·상충 주장 처리" help="중복 신청이나 서로 다른 주장이 있을 때 보류·확인하는 방법을 작성하세요." value=content.duplicate_contradictory_claim_procedure error=error("duplicate_contradictory_claim_procedure")/>
            <ProcedureField name="escalation_adjudication_procedure" label="추가 검토와 판단 절차" help="해결되지 않은 의심과 상충을 누구에게 전달하고 어떤 근거로 판단할지 작성하세요." value=content.escalation_adjudication_procedure error=error("escalation_adjudication_procedure")/>
            <ProcedureField name="evidence_minimization_retention_description" label="자료 최소화와 보관" help="꼭 필요한 확인 기록, 접근 범위와 보관 기준을 작성하세요." value=content.evidence_minimization_retention_description error=error("evidence_minimization_retention_description")/>
            <ProcedureField name="recipient_responsibility" label="확인 담당자의 책임" help="확인한 사실·불확실성의 기록과 독립적인 검토에 관한 책임을 작성하세요." value=content.recipient_responsibility error=error("recipient_responsibility")/>
            <p class="supporting">"각 설명은 UTF-8 기준 2,048바이트 이내, 이름·방법·설명 전체는 16,384바이트 이내입니다."</p>
        </fieldset>
        <fieldset class="group-fieldset"><legend>"4. 유효 기한과 운영 책임"</legend>
            <div class="group-field"><label for="process_expiry">"유효 기한 (한국 표준시, UTC+09:00)"</label>
                <input id="process_expiry" name="process_expiry" type="text" required
                    pattern="[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}"
                    placeholder="YYYY-MM-DDTHH:MM:SS" autocomplete="off" value=expires_at_local
                    aria-invalid=expiry_error.is_some().to_string() aria-describedby=if expiry_error.is_some() {"expiry-help expiry-error"} else {"expiry-help"}/>
                <p id="expiry-help" class="supporting">"한국 표준시로 초까지 입력하세요 (예: 2026-12-31T18:00:00). 요청 접수 이후 365일 이내여야 하며 확정 시에도 기한이 남아 있어야 합니다."</p>
                {expiry_error.map(|message|view! {<p id="expiry-error" class="group-field-error">{message}</p>})}
            </div>
            <div class="group-field"><label class="group-checkbox" for="operator_responsibility">
                <input id="operator_responsibility" name="operator_responsibility" type="checkbox" value="1" required checked=responsibility_accepted
                    aria-invalid=responsibility_error.is_some().to_string() aria-describedby=if responsibility_error.is_some() {"responsibility-help responsibility-error"} else {"responsibility-help"}/>
                <span>"절차의 적합성, 운영과 자료 취급 책임을 확인했습니다."</span></label>
                <p id="responsibility-help" class="supporting">"절차의 등록은 사실 확인이나 권한 연결을 대신하지 않습니다."</p>
                {responsibility_error.map(|message|view! {<p id="responsibility-error" class="group-field-error">{message}</p>})}
            </div>
        </fieldset>
    }.into_any()
}

fn form(form: Form) -> AnyView {
    let Form {
        scope,
        command,
        expected,
        proof,
        input,
        current,
        errors,
        form_error,
    } = form;
    let group_base = base(&scope.group);
    let (action, button, suspend) = match &input {
        Input::Adopt { replacing, .. } => (
            format!("{group_base}/processes"),
            if *replacing {
                "확인 절차 수정 등록"
            } else {
                "확인 절차 등록"
            },
            false,
        ),
        Input::Suspend { process, .. } => (
            format!("{group_base}/processes/{process}/suspend"),
            "확인 절차 중단",
            true,
        ),
    };
    let input_fields = match input {
        Input::Suspend {
            process: _,
            content_version,
            content_digest,
            expected_head_revision,
            expected_head_digest,
            reason,
        } => {
            let reason_error = errors
                .iter()
                .find(|error| error.field == "reason")
                .map(|error| error.message);
            view! {
                <input type="hidden" name="process_version" value=content_version/>
                <input type="hidden" name="process_digest" value=content_digest/>
                <input type="hidden" name="expected_process_head_revision" value=expected_head_revision/>
                <input type="hidden" name="expected_process_head_digest" value=expected_head_digest/>
                <p>"중단이 확정되면 이 절차로 새 확인 업무를 진행할 수 없습니다. 등록 내용과 이전 처리 기록은 보존됩니다."</p>
                <ProcedureField name="reason" label="중단 사유" help="중단이 필요한 이유와 후속 검토에 필요한 사항을 작성하세요. UTF-8 기준 2,048바이트 이내입니다." value=reason error=reason_error/>
            }.into_any()
        }
        Input::Adopt {
            process,
            expected_prior_head_revision,
            content,
            expires_at_local,
            responsibility_accepted,
            ..
        } => adopt_fields(
            process,
            expected_prior_head_revision,
            content,
            expires_at_local,
            responsibility_accepted,
            &errors,
        ),
    };
    let invalid = !errors.is_empty() || form_error.is_some();
    let submission = view! {
                <form method="post" action=action>
                    <input type="hidden" name="command_id" value=command.clone()/>
                    <input type="hidden" name="expected_group_revision" value=expected.group_revision/>
                    <input type="hidden" name="expected_group_incarnation" value=expected.group_incarnation/>
                    <input type="hidden" name="expected_group_identity_policy_revision" value=expected.policy_revision/>
                    <input type="hidden" name="csrf_proof" value=proof/>
                    {input_fields}
                    <div class="group-submit"><p class="supporting">"제출한 내용과 기대한 기록을 다시 확인한 뒤 처리합니다. 접수·확정·거절은 원래 요청 기록에서 구분하여 확인할 수 있습니다."</p>
                        <div class="policy-actions"><button class=if suspend {"policy-button danger"} else {"policy-button"} type="submit">{button}</button>
                            <a href=group_base>"현재 절차로 돌아가기"</a></div>
                    </div>
                </form>
    }.into_any();
    let validation = invalid.then(|| {
        view! {<div class="group-validation" role="alert" tabindex="-1" autofocus>
            <h3>"입력 내용을 확인해 주세요"</h3><p>"이번 제출의 입력 내용을 확인해 주세요. 입력한 내용은 아래에 보존됩니다."</p>
            <p><a href=request_path(&scope.group, &command)>"원래 요청 결과 확인"</a></p>
            {form_error.map(|message|view! {<p>{message}</p>})}
            <ul>{errors.iter().map(|error|view! {<li><a href=format!("#{}",error.field)>{error.message}</a></li>}).collect_view()}</ul>
        </div>}.into_any()
    });
    view! {
        <div class="group-workflow-layout group-form-layout">
            <aside class="workflow-guide" aria-label="그룹 업무 범위와 제출 결과">{scope_summary(&scope)}{current.as_ref().map(head_summary)}{consequences()}</aside>
            <div class="workflow-main"><section class="panel group-form-panel"><h2>"제출할 내용"</h2><div class="policy-panel-body">
                {validation}
                {submission}
            </div></section></div>
        </div>
    }.into_any()
}

fn original_record(original: &OriginalRecord) -> AnyView {
    view! {
        <details class="policy-provenance"><summary>"원래 요청의 접수 기록"</summary>
            <dl class="record-meta"><dt>"요청"</dt><dd>{original.command.clone()}</dd><dt>"그룹"</dt><dd>{original.group.clone()}</dd>
                <dt>"요청한 계정"</dt><dd>{original.actor.clone()}</dd><dt>"절차"</dt><dd>{original.process.clone()}</dd>
                <dt>"접수 기록"</dt><dd>{original.intake_receipt.clone()}</dd><dt>"입력 지문"</dt><dd>{original.input_digest.clone()}</dd>
                <dt>"그룹 식별 버전"</dt><dd>{original.incarnation.clone()}</dd>
                <dt>"기대한 그룹 버전"</dt><dd>{original.expected.group_revision.clone()}</dd>
                <dt>"기대한 정책 버전"</dt><dd>{original.expected.policy_revision.clone()}</dd>
            </dl>
        </details>
    }.into_any()
}

fn request(request: Request) -> AnyView {
    let Request { original, outcome } = request;
    let path = request_path(&original.group, &original.command);
    let (state, result) = match outcome {
        Outcome::Pending => ("pending", view! {
            <section class="panel group-result" data-group-process-outcome="pending"><h2>"요청이 접수되었으며 처리는 아직 확정되지 않았습니다"</h2><div class="policy-panel-body">
                <p>"이 화면을 열어도 절차가 변경되지는 않습니다. 원래 요청 결과 확인에서 같은 요청을 이어서 확인할 수 있습니다. 아직 처리되지 않은 요청은 현재 변경 권한과 절차의 유효 기한을 다시 확인합니다."</p>
            </div></section>
        }.into_any()),
        Outcome::Terminal { code, title, description, receipt, executed_at, before, after } => {
            let head = after.as_ref().or(before.as_ref());
            let process = head.map(|head|head.process.clone()).unwrap_or_else(||original.process.clone());
            let version = head.map(|head|head.content_version.clone());
            let revision = head.map(|head|head.head_revision.clone());
            let digest = head.map(|head|head.content_digest.clone());
            let head_digest = head.map(|head|head.head_digest.clone());
            ("terminal", view! {
                <section class="panel group-result" data-group-process-terminal-code=code data-group-process-command=original.command.clone()
                    data-group-process-result-receipt=receipt.clone() data-group-process-id=process
                    data-group-process-version=version data-group-process-head-revision=revision
                    data-group-process-digest=digest data-group-process-head-digest=head_digest>
                    <h2>{title}</h2><div class="policy-panel-body"><p>{description}</p>
                        <dl class="record-meta"><dt>"확정 시각"</dt><dd>{executed_at}</dd>
                            {head.map(|head|view! {<dt>"당시 내용 버전"</dt><dd>{head.content_version.clone()}</dd><dt>"당시 상태 버전"</dt><dd>{head.head_revision.clone()}</dd>})}
                        </dl>
                        <p class="supporting">"원래 요청의 확정 결과입니다. 이후 절차의 수정·중단이나 담당 지정 변경을 반영한 현재 상태는 아닙니다."</p>
                        <details class="policy-provenance"><summary>"확정 결과 식별 정보"</summary><dl class="record-meta"><dt>"처리 기록"</dt><dd>{receipt.clone()}</dd>
                            {head.map(|head|view! {<dt>"당시 내용 지문"</dt><dd>{head.content_digest.clone()}</dd><dt>"당시 상태 지문"</dt><dd>{head.head_digest.clone()}</dd>})}
                        </dl></details>
                    </div>
                </section>
            }.into_any())
        },
    };
    view! {
        <div class="group-receipt" data-group-process-outcome=state>
            {result}
            <section class="panel"><h2>"이 요청에서 제출한 내용"</h2><div class="policy-panel-body">
                <dl class="record-meta"><dt>"요청한 작업"</dt><dd>{original.operation_label}</dd><dt>"접수 시각"</dt><dd>{original.accepted_at.clone()}</dd>
                    {original.requested_expiry.as_ref().map(|expiry|view! {<dt>"요청한 유효 기한"</dt><dd>{expiry.clone()}</dd>})}
                </dl>
                {original.content.as_ref().map(|content|view! {<h3 class="group-subject">{content.title.clone()}</h3>{procedure(content)}})}
                {original.suspension_reason.as_ref().map(|reason|view! {<h3>"중단 사유"</h3><p class="group-preserved-text">{reason.clone()}</p>})}
                {original_record(&original)}
                <div class="policy-actions"><a class="policy-button" href=format!("{path}/retry")>"원래 요청 결과 확인"</a><a href=path>"이 요청 다시 열기"</a></div>
            </div></section>
        </div>
    }.into_any()
}

pub fn render(page: Page) -> String {
    let scope = match &page {
        Page::Empty { scope, .. } => Some(scope),
        Page::Current(current) => Some(&current.scope),
        Page::Form(form) => Some(&form.scope),
        _ => None,
    };
    let title = match &page {
        Page::Current(current) => current.content.title.clone(),
        Page::Form(Form {
            input: Input::Adopt {
                replacing: true, ..
            },
            ..
        }) => "확인 절차 수정 등록".into(),
        Page::Form(Form {
            input: Input::Adopt { .. },
            ..
        }) => "확인 절차 등록".into(),
        Page::Form(Form {
            input: Input::Suspend { .. },
            ..
        }) => "확인 절차 중단".into(),
        Page::Request(_) => "그룹 신원 확인 요청".into(),
        Page::OwnRetry { .. } => "원래 요청 결과 확인".into(),
        Page::Uncertain { .. } => "요청 결과 확인".into(),
        Page::Problem { title, .. } => (*title).into(),
        Page::NotVisible => "이 기록을 확인할 수 없습니다".into(),
        Page::Refused => "이 페이지를 열 수 없습니다".into(),
        Page::Unavailable => "지금 정보를 불러올 수 없습니다".into(),
        Page::Empty { .. } => "그룹 신원 확인".into(),
    };
    let eyebrow = scope.map(|scope| {
        view! {<p class="page-eyebrow">"그룹 / "{scope.group_name.clone()}</p>}.into_any()
    });
    let header = native_workspace_header::render(
        |_| {
            scope.map(|scope|view! {
        <p class="nav-group">"관리"</p><nav aria-label="그룹 업무 탐색"><a href=base(&scope.group) aria-current="location">"그룹 신원 확인"</a></nav>
    }.into_any()).unwrap_or_else(||().into_any())
        },
        None,
    );
    let content = match page {
        Page::Empty { scope, can_adopt } => empty(scope, can_adopt),
        Page::Current(current_view) => current(current_view),
        Page::Form(form_view) => form(form_view),
        Page::Request(request_view) => request(request_view),
        Page::OwnRetry { group, command, proof } => {
            let path = request_path(&group, &command);
            view! {<section class="panel"><h2>"같은 요청의 결과를 확인합니다"</h2><div class="policy-panel-body">
                <p>"이미 확정된 요청은 원래 결과만 반환합니다. 아직 처리되지 않은 요청은 현재 권한, 원래 제출 내용과 절차의 유효 기한을 다시 확인한 뒤 처리합니다. 새로운 요청이나 다른 내용으로 바꾸지 않습니다."</p>
                <details class="policy-provenance"><summary>"원래 요청 식별 정보"</summary><dl class="record-meta"><dt>"요청"</dt><dd>{command}</dd><dt>"그룹"</dt><dd>{group}</dd></dl></details>
                <form method="post" action=format!("{path}/retry")><input type="hidden" name="csrf_proof" value=proof/>
                    <div class="policy-actions"><button class="policy-button" type="submit">"원래 요청 결과 확인"</button><a href=path.clone()>"원래 요청으로 돌아가기"</a></div>
                </form>
            </div></section>}.into_any()
        },
        Page::Uncertain { group, command } => view! {<section class="panel" data-group-process-outcome="uncertain"><h2>"처리 결과를 아직 확인하지 못했습니다"</h2><div class="policy-panel-body">
            <p>"요청이 접수되거나 처리되었을 수 있습니다. 새로 제출하기 전에 원래 요청의 기록을 확인하세요."</p>
            <a class="policy-button" href=request_path(&group,&command)>"같은 요청 기록 확인"</a>
        </div></section>}.into_any(),
        Page::Problem { title, description } => view! {<section class="panel" role="alert"><h2>{title}</h2><div class="policy-panel-body"><p>{description}</p><a href="/account">"내 업무 공간 확인"</a></div></section>}.into_any(),
        Page::NotVisible => view! {<section class="panel"><div class="policy-panel-body"><p>"현재 권한으로 이 기록을 확인할 수 없습니다. 기록이 보이지 않는다고 요청이 처리되지 않았다는 뜻은 아닙니다."</p><a href="/account">"내 업무 공간 확인"</a></div></section>}.into_any(),
        Page::Refused => view! {<section class="panel"><div class="policy-panel-body"><p>"현재 접근할 수 있는 업무 공간을 확인하세요."</p><a href="/account">"내 업무 공간 확인"</a></div></section>}.into_any(),
        Page::Unavailable => view! {<section class="panel"><div class="policy-panel-body"><p>"잠시 후 다시 열어 주세요. 제출한 요청이 있다면 원래 요청 주소에서 결과를 확인하세요."</p><a href="/account">"내 업무 공간 확인"</a></div></section>}.into_any(),
    };
    let html = view! {
        <html lang="ko"><head><meta charset="utf-8"/><meta name="viewport" content="width=device-width, initial-scale=1"/>
            <title>{format!("{title} · Console")}</title><link rel="stylesheet" href="/assets/workspace.css"/>
        </head><body class="workspace native-group-workspace">
            <a class="skip-link" href="#main-content">"본문 바로가기"</a>{header}
            <main id="main-content" tabindex="0"><div class="page-heading">{eyebrow}<h1>{title}</h1></div>{content}</main>
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
