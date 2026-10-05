//! Native Group page composition over finalized owner projections. No SQL or
//! business transition belongs in this presentation adapter.
use axum::{
    Router,
    extract::{Extension, MatchedPath, Path, Request, State, rejection::PathRejection},
    http::{HeaderMap, HeaderValue, Method, StatusCode, header},
    middleware::{Next, from_fn},
    response::Response,
    routing::any,
};
use console_identity_adapter_postgres::PgOrgStore;
use console_identity_application::group_process::*;
use console_identity_rest::group_process::{
    GroupFormTarget, GroupPostTarget, GroupProcessDraft, GroupProcessDraftInput, GroupRestState,
    GroupSubmission, RawContent, RecoveryLocator,
};
use console_payroll_ui::native_group_process as ui;
use console_platform_auth::account::AccountFormProof;
use console_platform_request_context::TrustedClientIp;

type Owner = GroupRestState<PgOrgStore>;
#[derive(Clone)]
struct NativeGroupState {
    owner: Owner,
    runtime: crate::AppState,
}
const LANDING: &str = "/groups/{group}/identity";
const NEW: &str = "/groups/{group}/identity/processes/new";
const ADOPT: &str = "/groups/{group}/identity/processes";
const REPLACE: &str = "/groups/{group}/identity/processes/{process}/replace";
const SUSPEND: &str = "/groups/{group}/identity/processes/{process}/suspend";
const REQUEST: &str = "/groups/{group}/identity/requests/{command}";
const RETRY: &str = "/groups/{group}/identity/requests/{command}/retry";

#[derive(serde::Deserialize)]
struct Route {
    group: String,
    process: Option<String>,
    command: Option<String>,
}

pub(super) fn router(owner: Owner, runtime: crate::AppState) -> Router {
    Router::new()
        .route(LANDING, any(handle))
        .route(NEW, any(handle))
        .route(ADOPT, any(handle))
        .route(REPLACE, any(handle))
        .route(SUSPEND, any(handle))
        .route(REQUEST, any(handle))
        .route(RETRY, any(handle))
        .with_state(NativeGroupState { owner, runtime })
}

/// Shared native composition admission; this conveys no owner authority.
pub(super) async fn current_custody(runtime: &crate::AppState) -> Result<(), StatusCode> {
    let Some(startup) = runtime.serving_custody_profile else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };
    if !startup.supports_native_group_process() {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
    let crate::DatabaseDependency::Postgres(pool) = &runtime.database else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };
    match crate::account_custody::verify(pool).await {
        Ok(current) if current == startup => Ok(()),
        _ => Err(StatusCode::SERVICE_UNAVAILABLE),
    }
}

async fn guard(runtime: &crate::AppState, headers: &HeaderMap) -> Result<(), StatusCode> {
    use console_platform_auth_rest::NativeAccountEntry;
    let Some(auth) = &runtime.auth_rest else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };
    match console_platform_auth_rest::native_account_entry(auth, headers, false).await {
        Ok(NativeAccountEntry::Active { .. }) => current_custody(runtime).await,
        Ok(NativeAccountEntry::SignIn) => Err(StatusCode::UNAUTHORIZED),
        Err(error) => Err(error.status()),
        Ok(_) => Err(StatusCode::SERVICE_UNAVAILABLE),
    }
}

/// Preserve native interruption feedback outside the shared timeout/envelope.
pub(super) fn with_transport(router: Router) -> Router {
    router.layer(from_fn(timeout_response))
}
async fn timeout_response(mut request: Request, next: Next) -> Response {
    let applies = request
        .extensions()
        .get::<MatchedPath>()
        .is_some_and(|path| {
            matches!(
                path.as_str(),
                LANDING | NEW | ADOPT | REPLACE | SUSPEND | REQUEST | RETRY
            )
        });
    if !applies {
        return next.run(request).await;
    }
    let head = request.method() == Method::HEAD;
    let recovery = RecoveryLocator::default();
    request.extensions_mut().insert(recovery.clone());
    let mut response = next.run(request).await;
    if response.status() == StatusCode::REQUEST_TIMEOUT {
        let page = match recovery.get() {
            Some(requested) => ui::Page::Uncertain {
                group: requested.group().as_uuid().to_string(),
                command: requested.command_id().to_string(),
            },
            None => ui::Page::Unavailable,
        };
        response = ui::document(page, StatusCode::REQUEST_TIMEOUT);
    }
    if head {
        *response.body_mut() = axum::body::Body::empty();
    }
    response
}

async fn handle(
    State(state): State<NativeGroupState>,
    path: Result<Path<Route>, PathRejection>,
    matched: MatchedPath,
    client: Option<Extension<TrustedClientIp>>,
    recovery: Option<Extension<RecoveryLocator>>,
    request: Request,
) -> Response {
    let Ok(Path(route)) = path else {
        return error(StatusCode::NOT_FOUND);
    };
    if request.method() == Method::POST {
        let target = match matched.as_str() {
            ADOPT => GroupPostTarget::Adopt,
            SUSPEND => match route.process.as_deref() {
                Some(process) => GroupPostTarget::Suspend { process },
                None => return error(StatusCode::NOT_FOUND),
            },
            RETRY => match route.command.as_deref() {
                Some(command) => GroupPostTarget::Retry { command },
                None => return error(StatusCode::NOT_FOUND),
            },
            _ => return method_refused(matched.as_str()),
        };
        let headers = request.headers().clone();
        if let Err(status) = guard(&state.runtime, &headers).await {
            return error(status);
        }
        let prepared = match state
            .owner
            .prepare_submit_document(request, &route.group, target)
            .await
        {
            Ok(prepared) => prepared,
            Err(status) => return error(status),
        };
        return submission(state.owner.submit_prepared_document(prepared).await);
    }
    if request.method() != Method::GET || matched.as_str() == ADOPT {
        return method_refused(matched.as_str());
    }
    read(
        state,
        route,
        matched.as_str(),
        request.headers(),
        client,
        recovery,
    )
    .await
}

async fn read(
    state: NativeGroupState,
    route: Route,
    matched: &str,
    headers: &HeaderMap,
    client: Option<Extension<TrustedClientIp>>,
    recovery: Option<Extension<RecoveryLocator>>,
) -> Response {
    let client = client.map(|Extension(ip)| ip);
    let recovery = recovery.as_ref().map(|Extension(locator)| locator);
    let owner = &state.owner;
    let result = match matched {
        LANDING => {
            async {
                let prepared = owner.prepare_landing_document(headers, &route.group)?;
                guard(&state.runtime, headers).await?;
                owner
                    .landing_prepared_document(prepared)
                    .await
                    .and_then(landing)
            }
            .await
        }
        NEW => {
            async {
                let prepared = owner.prepare_form_document(
                    headers,
                    client,
                    &route.group,
                    GroupFormTarget::Adopt,
                )?;
                guard(&state.runtime, headers).await?;
                owner
                    .form_prepared_document(prepared)
                    .await
                    .and_then(|value| form(value, false))
            }
            .await
        }
        REPLACE | SUSPEND => match route.process.as_deref() {
            Some(process) => {
                let suspend = matched == SUSPEND;
                let target = if suspend {
                    GroupFormTarget::Suspend { process }
                } else {
                    GroupFormTarget::Replace { process }
                };
                async {
                    let prepared =
                        owner.prepare_form_document(headers, client, &route.group, target)?;
                    guard(&state.runtime, headers).await?;
                    owner
                        .form_prepared_document(prepared)
                        .await
                        .and_then(|value| form(value, suspend))
                }
                .await
            }
            None => Err(StatusCode::NOT_FOUND),
        },
        REQUEST => match route.command.as_deref() {
            Some(command) => {
                async {
                    let prepared = owner.prepare_request_document(
                        headers,
                        None,
                        &route.group,
                        command,
                        recovery,
                    )?;
                    guard(&state.runtime, headers).await?;
                    owner
                        .request_prepared_document(prepared)
                        .await
                        .and_then(request)
                }
                .await
            }
            None => Err(StatusCode::NOT_FOUND),
        },
        RETRY => match route.command.as_deref() {
            Some(command) => {
                async {
                    let prepared = owner.prepare_request_document(
                        headers,
                        client,
                        &route.group,
                        command,
                        recovery,
                    )?;
                    guard(&state.runtime, headers).await?;
                    owner
                        .retry_prepared_document(prepared)
                        .await
                        .map(|form| ui::Page::OwnRetry {
                            group: form.original.group().as_uuid().to_string(),
                            command: form.original.command_id().to_string(),
                            proof: form.proof.as_str().to_owned(),
                        })
                }
                .await
            }
            None => Err(StatusCode::NOT_FOUND),
        },
        _ => Err(StatusCode::NOT_FOUND),
    };
    match result {
        Ok(page) => ui::document(page, StatusCode::OK),
        Err(status) => error(status),
    }
}

fn submission(result: Result<GroupSubmission, StatusCode>) -> Response {
    match result {
        Ok(GroupSubmission::Validation { current, draft }) => match validation(*current, draft) {
            Ok(page) => ui::document(page, StatusCode::UNPROCESSABLE_ENTITY),
            Err(status) => error(status),
        },
        Ok(GroupSubmission::Confirmed { requested }) => {
            let path = format!(
                "/groups/{}/identity/requests/{}",
                requested.group().as_uuid(),
                requested.command_id()
            );
            let mut response = ui::document(ui::Page::NotVisible, StatusCode::SEE_OTHER);
            match HeaderValue::from_str(&path) {
                Ok(location) => {
                    response.headers_mut().insert(header::LOCATION, location);
                    response
                }
                Err(_) => error(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        Ok(GroupSubmission::Unconfirmed { requested }) => ui::document(
            ui::Page::Uncertain {
                group: requested.group().as_uuid().to_string(),
                command: requested.command_id().to_string(),
            },
            StatusCode::SERVICE_UNAVAILABLE,
        ),
        Err(status) => error(status),
    }
}

fn validation(
    value: GroupProcessForm<AccountFormProof>,
    draft: GroupProcessDraft,
) -> Result<ui::Page, StatusCode> {
    let current = head(&value.view)?;
    let mut form_error = None;
    let errors = draft.errors.into_iter().filter_map(|field| {
        let message = match field {
            "content" => {
                form_error = Some("절차 내용의 합계가 UTF-8 기준 16,384바이트를 초과합니다. 내용을 줄인 뒤 다시 제출하세요.");
                return None;
            }
            "title" => "절차 이름을 UTF-8 기준 120바이트 이내로 작성하세요. 빈 값과 제어 문자는 사용할 수 없습니다.",
            "method" => "지원되는 확인 방법을 선택하세요.",
            "process_expiry" => "실제 날짜와 시각을 YYYY-MM-DDTHH:MM:SS 형식의 한국 표준시로 작성하세요.",
            _ => "내용을 UTF-8 기준 2,048바이트 이내로 작성하세요. 빈 값과 제어 문자는 사용할 수 없습니다.",
        };
        Some(ui::FieldError { field, message })
    }).collect();
    let input = match draft.input {
        GroupProcessDraftInput::Adopt {
            expected_prior_head_revision,
            expiry,
            content,
        } => ui::Input::Adopt {
            process: draft.process_id.to_string(),
            expected_prior_head_revision: expected_prior_head_revision.to_string(),
            replacing: expected_prior_head_revision != 0,
            content: raw_content(content),
            expires_at_local: expiry,
            responsibility_accepted: true,
        },
        GroupProcessDraftInput::Suspend {
            content_version,
            content_digest,
            expected_head_revision,
            expected_head_digest,
            reason,
        } => ui::Input::Suspend {
            process: draft.process_id.to_string(),
            content_version: content_version.to_string(),
            content_digest: hex(&content_digest),
            expected_head_revision: expected_head_revision.to_string(),
            expected_head_digest: hex(&expected_head_digest),
            reason,
        },
    };
    Ok(ui::Page::Form(ui::Form {
        scope: scope(
            value.view.context,
            value.view.policy_head.revision().unwrap_or(0),
        ),
        command: draft.original.command_id().to_string(),
        expected: ui::Expectations {
            group_revision: draft.expected_group_revision.to_string(),
            group_incarnation: draft.original.incarnation().as_uuid().to_string(),
            policy_revision: draft.expected_policy_revision.to_string(),
        },
        proof: value.proof.as_str().to_owned(),
        input,
        current,
        errors,
        form_error,
    }))
}
fn raw_content(value: RawContent) -> ui::Content {
    let [
        claimant,
        account,
        human,
        duplicate,
        qualification,
        escalation,
        retention,
        recipient,
    ] = value.prose;
    ui::Content {
        title: value.title,
        method: value.method,
        intended_claimant_matching_procedure: claimant,
        account_possession_procedure: account,
        physical_human_evidence_procedure: human,
        duplicate_contradictory_claim_procedure: duplicate,
        qualification_criteria_instruction: qualification,
        escalation_adjudication_procedure: escalation,
        evidence_minimization_retention_description: retention,
        recipient_responsibility: recipient,
    }
}

fn scope(context: GroupProcessContextV1, policy_revision: u64) -> ui::Scope {
    ui::Scope {
        group: context.group.as_uuid().to_string(),
        group_name: context.label,
        incarnation: context.incarnation.as_uuid().to_string(),
        revision: context.revision.to_string(),
        policy_revision: policy_revision.to_string(),
        operator: context.account.as_uuid().to_string(),
    }
}
fn content(value: &ProcessContentV1) -> ui::Content {
    ui::Content {
        title: value.title().to_owned(),
        method: value.method().as_str().to_owned(),
        intended_claimant_matching_procedure: value
            .intended_claimant_matching_procedure()
            .to_owned(),
        account_possession_procedure: value.account_possession_procedure().to_owned(),
        physical_human_evidence_procedure: value.physical_human_evidence_procedure().to_owned(),
        duplicate_contradictory_claim_procedure: value
            .duplicate_contradictory_claim_procedure()
            .to_owned(),
        qualification_criteria_instruction: value.qualification_criteria_instruction().to_owned(),
        escalation_adjudication_procedure: value.escalation_adjudication_procedure().to_owned(),
        evidence_minimization_retention_description: value
            .evidence_minimization_retention_description()
            .to_owned(),
        recipient_responsibility: value.recipient_responsibility().to_owned(),
    }
}
fn reference(value: &ProcessHeadReferenceV1) -> Result<ui::HeadReference, StatusCode> {
    Ok(ui::HeadReference {
        process: value.process_id().to_string(),
        content_version: value.content_version().to_string(),
        head_revision: value.head_revision().to_string(),
        content_digest: hex(value.content_digest()),
        head_digest: hex(value.head_digest()),
        state: value.state().as_str(),
        expires_at: date(value.expiry_us(), false)?,
    })
}
fn head(value: &GroupProcessCurrentView) -> Result<Option<ui::Head>, StatusCode> {
    let Some(current) = &value.head else {
        return Ok(None);
    };
    let mut history = value.history.iter().filter(|entry| entry.head == *current);
    let original = history.next().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    if history.next().is_some() {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
    Ok(Some(ui::Head {
        reference: reference(&current.reference)?,
        causing_command: original.command_id.to_string(),
        causing_receipt: current.causing_receipt.to_string(),
    }))
}
fn landing(value: GroupProcessLandingViewV1) -> Result<ui::Page, StatusCode> {
    match value {
        GroupProcessLandingViewV1::Empty(value) => Ok(ui::Page::Empty {
            scope: scope(value.context, 0),
            can_adopt: value.allowed_actions.contains(&GroupProcessActionV1::Adopt),
        }),
        GroupProcessLandingViewV1::Current(value) => {
            let current = head(&value)?.ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
            let content = content(
                value
                    .content
                    .as_ref()
                    .ok_or(StatusCode::SERVICE_UNAVAILABLE)?,
            );
            let usable = value.head.as_ref().is_some_and(|head| {
                head.reference.state() == ProcessStateV1::Active
                    && head.reference.expiry_us() > value.observed_at_us
            });
            let history = value
                .history
                .iter()
                .map(|entry| {
                    Ok(ui::History {
                        label: if entry.reason.is_some() {
                            "절차 중지"
                        } else if entry.head.reference.content_version() == 1 {
                            "절차 등록"
                        } else {
                            "절차 교체"
                        },
                        recorded_at: date(entry.occurred_at_us, false)?,
                        content_version: entry.head.reference.content_version().to_string(),
                        head_revision: entry.head.reference.head_revision().to_string(),
                        command: entry.command_id.to_string(),
                        reason: entry.reason.clone(),
                    })
                })
                .collect::<Result<Vec<_>, StatusCode>>()?;
            Ok(ui::Page::Current(ui::Current {
                observed_at: date(value.observed_at_us, false)?,
                usable,
                can_replace: value.allowed_actions.contains(&GroupProcessActionV1::Adopt),
                can_suspend: value
                    .allowed_actions
                    .contains(&GroupProcessActionV1::Suspend),
                scope: scope(value.context, value.policy_head.revision().unwrap_or(0)),
                head: current,
                content,
                history,
            }))
        }
    }
}
fn form(value: GroupProcessForm<AccountFormProof>, suspend: bool) -> Result<ui::Page, StatusCode> {
    let current = head(&value.view)?;
    let expected = ui::Expectations {
        group_revision: value.view.context.revision.to_string(),
        group_incarnation: value.view.context.incarnation.as_uuid().to_string(),
        policy_revision: value.view.policy_head.revision().unwrap_or(0).to_string(),
    };
    let input = if suspend {
        let head = value.view.head.as_ref().ok_or(StatusCode::NOT_FOUND)?;
        ui::Input::Suspend {
            process: value.process_id.to_string(),
            content_version: head.reference.content_version().to_string(),
            content_digest: hex(head.reference.content_digest()),
            expected_head_revision: head.reference.head_revision().to_string(),
            expected_head_digest: hex(head.reference.head_digest()),
            reason: String::new(),
        }
    } else {
        ui::Input::Adopt {
            process: value.process_id.to_string(),
            expected_prior_head_revision: value
                .view
                .head
                .as_ref()
                .map_or(0, |h| h.reference.head_revision())
                .to_string(),
            replacing: current.is_some(),
            content: value
                .view
                .content
                .as_ref()
                .map(content)
                .unwrap_or_else(empty_content),
            expires_at_local: value
                .view
                .head
                .as_ref()
                .map(|h| date(h.reference.expiry_us(), true))
                .transpose()?
                .unwrap_or_default(),
            responsibility_accepted: false,
        }
    };
    Ok(ui::Page::Form(ui::Form {
        scope: scope(
            value.view.context,
            value.view.policy_head.revision().unwrap_or(0),
        ),
        command: value.command_id.to_string(),
        expected,
        proof: value.proof.as_str().to_owned(),
        input,
        current,
        errors: Vec::new(),
        form_error: None,
    }))
}
fn empty_content() -> ui::Content {
    ui::Content {
        title: String::new(),
        method: ProcessMethodV1::AttendedAccountAndDocumentaryReview
            .as_str()
            .to_owned(),
        intended_claimant_matching_procedure: String::new(),
        account_possession_procedure: String::new(),
        physical_human_evidence_procedure: String::new(),
        duplicate_contradictory_claim_procedure: String::new(),
        qualification_criteria_instruction: String::new(),
        escalation_adjudication_procedure: String::new(),
        evidence_minimization_retention_description: String::new(),
        recipient_responsibility: String::new(),
    }
}
fn request(value: GroupProcessStatus) -> Result<ui::Page, StatusCode> {
    let (accepted, outcome) = match value {
        GroupProcessStatus::NotVisible => return Err(StatusCode::NOT_FOUND),
        GroupProcessStatus::AcceptedPending(value) => (value, ui::Outcome::Pending),
        GroupProcessStatus::Terminal(value) => {
            let result = &value.result;
            let (title, description) = match result.terminal_code() {
                ProcessTerminalCodeV1::Adopted => (
                    "절차가 등록되었습니다",
                    "등록한 내용과 책임, 적용 기한을 확인하세요.",
                ),
                ProcessTerminalCodeV1::Replaced => (
                    "절차가 교체되었습니다",
                    "새 내용이 적용됩니다. 이전 버전과 요청 기록은 보존됩니다.",
                ),
                ProcessTerminalCodeV1::Suspended => (
                    "절차가 중지되었습니다",
                    "중지 이유가 기록되었습니다. 이전 내용과 요청 기록은 보존됩니다.",
                ),
                ProcessTerminalCodeV1::RejectedStaleExpectation => (
                    "업무 상태가 변경되었습니다",
                    "원래 요청이 참조한 상태와 실제 처리 시점의 상태가 달랐습니다. 이 요청은 절차를 변경하지 않았습니다.",
                ),
                ProcessTerminalCodeV1::RejectedProcessExpired => (
                    "적용 기한이 지났습니다",
                    "이 요청은 절차를 변경하지 않았습니다. 현재 상태를 확인한 뒤 새 요청을 작성하세요.",
                ),
                ProcessTerminalCodeV1::RejectedAlreadySuspended => (
                    "이미 중지된 절차입니다",
                    "이 요청은 절차를 변경하지 않았습니다.",
                ),
            };
            let outcome = ui::Outcome::Terminal {
                code: result.terminal_code().as_str(),
                title,
                description,
                receipt: result.result_receipt().to_string(),
                executed_at: date(result.executed_at_us(), false)?,
                before: result.before_head().map(reference).transpose()?,
                after: result.after_head().map(reference).transpose()?,
            };
            (value.accepted, outcome)
        }
    };
    let input = &accepted.input;
    Ok(ui::Page::Request(ui::Request {
        original: ui::OriginalRecord {
            group: input.group().as_uuid().to_string(),
            incarnation: input.incarnation().as_uuid().to_string(),
            command: input.command_id().to_string(),
            actor: accepted.actor.as_uuid().to_string(),
            process: input.process_id().to_string(),
            operation_label: if input.adoption().is_some() {
                "절차 등록 · 교체"
            } else {
                "절차 중지"
            },
            accepted_at: date(accepted.accepted_at_us, false)?,
            intake_receipt: accepted.intake_receipt.to_string(),
            input_digest: hex(&accepted.input_digest),
            expected: ui::Expectations {
                group_revision: input.expected_group_revision().to_string(),
                group_incarnation: input.incarnation().as_uuid().to_string(),
                policy_revision: input.expected_policy_revision().to_string(),
            },
            content: input.adoption().map(|adopt| content(adopt.content())),
            requested_expiry: input
                .adoption()
                .map(|adopt| {
                    exact_time_us(adopt.expiry())
                        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)
                        .and_then(|us| date(us, false))
                })
                .transpose()?,
            suspension_reason: input
                .suspension()
                .map(|suspend| suspend.reason().to_owned()),
        },
        outcome,
    }))
}
fn date(us: i64, input: bool) -> Result<String, StatusCode> {
    let at = time_from_us(us)
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?
        .to_offset(time::macros::offset!(+9));
    if input {
        Ok(format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
            at.year(),
            u8::from(at.month()),
            at.day(),
            at.hour(),
            at.minute(),
            at.second()
        ))
    } else {
        Ok(format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02} KST",
            at.year(),
            u8::from(at.month()),
            at.day(),
            at.hour(),
            at.minute(),
            at.second()
        ))
    }
}
fn hex(value: &[u8; 32]) -> String {
    use std::fmt::Write;
    let mut output = String::with_capacity(64);
    for byte in value {
        let _ = write!(output, "{byte:02x}");
    }
    output
}
fn error(status: StatusCode) -> Response {
    let page = match status {
        StatusCode::NOT_FOUND => ui::Page::NotVisible,
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => ui::Page::Refused,
        StatusCode::METHOD_NOT_ALLOWED => ui::Page::Problem {
            title: "지원하지 않는 요청입니다",
            description: "업무 화면의 안내된 링크와 제출 버튼을 사용하세요.",
        },
        StatusCode::BAD_REQUEST => ui::Page::Problem {
            title: "제출 형식이 올바르지 않습니다",
            description: "원래 업무 화면으로 돌아가 내용을 확인한 뒤 다시 제출하세요.",
        },
        StatusCode::PAYLOAD_TOO_LARGE => ui::Page::Problem {
            title: "제출 내용이 너무 큽니다",
            description: "업무 화면에 안내된 입력 길이를 확인한 뒤 다시 제출하세요.",
        },
        _ => ui::Page::Unavailable,
    };
    console_platform_request_context::preserve_native_html_error(ui::document(page, status))
}

fn method_refused(matched: &str) -> Response {
    let allowed = match matched {
        ADOPT => "POST",
        SUSPEND | RETRY => "GET, POST",
        _ => "GET",
    };
    let mut response = error(StatusCode::METHOD_NOT_ALLOWED);
    response
        .headers_mut()
        .insert(header::ALLOW, HeaderValue::from_static(allowed));
    response
}

#[cfg(test)]
#[path = "native_group_process_transport_tests.rs"]
mod transport_tests;
