//! Thin HTTP composition for native Company policy documents.
use axum::{
    extract::{Extension, Path, Request, State},
    http::{HeaderMap, HeaderValue, Method, StatusCode, header},
    response::Response,
};
use console_identity_adapter_postgres::PgOrgStore;
use console_identity_application::company_policy::{
    business::NativeBusinessOperationV1,
    workflow::{
        NativePolicyAcceptedView, NativePolicyAssignmentState, NativePolicyCommandRef,
        NativePolicyEffect, NativePolicyFormView, NativePolicyOutcome, NativePolicyRejection,
        NativePolicyStatus,
    },
};
use console_identity_rest::company::{
    CompanyRestState, NativePolicyPostTarget, NativePolicyRequestDocument, NativePolicySubmission,
};
use console_payroll_ui::native_policy as ui;
use console_platform_request_context::TrustedClientIp;
use time::OffsetDateTime;

type CompanyState = CompanyRestState<PgOrgStore>;
fn operation(op: NativeBusinessOperationV1) -> ui::Operation {
    match op {
        NativeBusinessOperationV1::Install => ui::Operation::Install,
        NativeBusinessOperationV1::Grant => ui::Operation::Grant,
        NativeBusinessOperationV1::Revoke => ui::Operation::Revoke,
    }
}
fn date(at: OffsetDateTime) -> String {
    let at = at.to_offset(time::macros::offset!(+9));
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02} KST",
        at.year(),
        u8::from(at.month()),
        at.day(),
        at.hour(),
        at.minute(),
        at.second()
    )
}
fn result_path(selector: NativePolicyCommandRef) -> String {
    format!(
        "/companies/{}/policy/payroll-read/requests/{}/{}",
        selector.company(),
        operation(selector.operation()).path(),
        selector.command_id()
    )
}
async fn identity_context(
    state: &CompanyState,
    headers: &HeaderMap,
    company: &str,
) -> Result<Option<console_identity_application::company_policy::CompanyIdentityView>, StatusCode> {
    match state.company_document(headers, company).await {
        Ok(value) => Ok(Some(value)),
        Err(StatusCode::NOT_FOUND | StatusCode::FORBIDDEN) => Ok(None),
        Err(status) => Err(status),
    }
}
fn scope(
    view: NativePolicyFormView,
    identity: Option<console_identity_application::company_policy::CompanyIdentityView>,
    payroll_link: bool,
) -> ui::Scope {
    let company = view.selector.company().to_string();
    let now = OffsetDateTime::now_utc();
    ui::Scope {
        company_name: identity.as_ref().map(|v| v.name.clone()),
        group: view.group_id.to_string(),
        company_link: identity.is_some(),
        policy_link: identity.as_ref().is_some_and(|v| v.show_policy_navigation),
        payroll_link,
        company,
        operator: view.acting_account_id.as_uuid().to_string(),
        recipient: view.administrative_account_id.as_uuid().to_string(),
        epoch: view.company_epoch.to_string(),
        installed: view.installed_object_type_id.is_some(),
        assignment: view.assignment.map(|a| {
            let revoked = matches!(a.state, NativePolicyAssignmentState::Revoked);
            ui::Assignment {
                id: a.expectation.assignment_id.to_string(),
                role_revision: a.expectation.role_revision.to_string(),
                revision: a.expectation.assignment_revision.to_string(),
                state: if revoked { "REVOKED" } else { "ACTIVE" },
                label: if revoked {
                    "회수됨"
                } else if a.valid_until <= now {
                    "열람 기간 종료"
                } else {
                    "열람 권한 연결됨"
                },
                from: date(a.valid_from),
                until: date(a.valid_until),
                can_grant: revoked || a.valid_until <= now,
                can_revoke: !revoked,
            }
        }),
    }
}
fn original(accepted: &NativePolicyAcceptedView) -> ui::OriginalRecord {
    ui::OriginalRecord {
        accepted_at: date(accepted.accepted_at),
        deadline: date(accepted.execution_not_after),
        intake_receipt: accepted.intake_receipt_id.to_string(),
        expected_company_epoch: accepted.input.expected_company_epoch().to_string(),
        expected_assignment: accepted.input.assignment_expectation().map(|a| {
            (
                a.assignment_id.to_string(),
                a.assignment_revision.to_string(),
            )
        }),
        requested_until: accepted.input.expires_at().map(date),
        effect_period: None,
        effect_epochs: None,
    }
}
pub(super) fn error(status: StatusCode) -> Response {
    let page = match status {
        StatusCode::FORBIDDEN | StatusCode::NOT_FOUND => ui::Page::Refused,
        StatusCode::UNAUTHORIZED => ui::Page::Problem {
            state: "authentication",
            title: "로그인이 필요합니다",
            description: "내 계정에 다시 로그인한 뒤 요청을 확인하세요.",
        },
        StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY => ui::Page::Problem {
            state: "invalid",
            title: "입력한 요청을 확인해 주세요",
            description: "입력 형식을 확인한 뒤 다시 제출하세요.",
        },
        StatusCode::PAYLOAD_TOO_LARGE => ui::Page::Problem {
            state: "too-large",
            title: "입력한 내용이 너무 깁니다",
            description: "입력한 내용을 줄인 뒤 다시 제출하세요.",
        },
        StatusCode::CONFLICT => ui::Page::Problem {
            state: "conflict",
            title: "요청과 현재 상태가 다릅니다",
            description: "기존 요청의 처리 기록을 확인하고 현재 권한 상태에서 다시 시작하세요.",
        },
        StatusCode::TOO_MANY_REQUESTS => ui::Page::Problem {
            state: "capacity",
            title: "처리 중인 요청이 많습니다",
            description: "접수한 요청의 결과를 먼저 확인한 뒤 다시 시도하세요.",
        },
        StatusCode::METHOD_NOT_ALLOWED => ui::Page::Problem {
            state: "method",
            title: "지원하지 않는 요청입니다",
            description: "업무 공간의 작업 화면에서 다시 시작하세요.",
        },
        _ => ui::Page::Unavailable,
    };
    let response = ui::document(page, status);
    if status == StatusCode::PAYLOAD_TOO_LARGE {
        console_platform_request_context::preserve_native_html_error(response)
    } else {
        response
    }
}
pub(super) async fn preflight(
    State(state): State<CompanyState>,
    Extension(payroll): Extension<crate::native_payroll::Navigation>,
    Path((company, op)): Path<(String, String)>,
    headers: HeaderMap,
    method: Method,
    client: Option<Extension<TrustedClientIp>>,
) -> Response {
    if method != Method::GET {
        return error(StatusCode::METHOD_NOT_ALLOWED);
    }
    let payroll_link = match payroll.visible(&headers, &company).await {
        Ok(value) => value,
        Err(status) => return error(status),
    };
    let form = match state
        .policy_form_document(&headers, client.map(|Extension(ip)| ip), &company, &op)
        .await
    {
        Ok(value) => value,
        Err(status) => return error(status),
    };
    let identity = match identity_context(&state, &headers, &company).await {
        Ok(value) => value,
        Err(status) => return error(status),
    };
    let selector = form.view.selector;
    ui::document(
        ui::Page::Form(ui::Form {
            scope: scope(form.view, identity, payroll_link),
            operation: operation(selector.operation()),
            command: selector.command_id().to_string(),
            proof: form.proof.as_str().to_owned(),
            validation: None,
        }),
        StatusCode::OK,
    )
}
pub(super) async fn request_document(
    State(state): State<CompanyState>,
    Extension(payroll): Extension<crate::native_payroll::Navigation>,
    Path((company, op, command)): Path<(String, String, String)>,
    headers: HeaderMap,
    method: Method,
    client: Option<Extension<TrustedClientIp>>,
) -> Response {
    if method != Method::GET {
        return error(StatusCode::METHOD_NOT_ALLOWED);
    }
    let payroll_link = match payroll.visible(&headers, &company).await {
        Ok(value) => value,
        Err(status) => return error(status),
    };
    let result = match state
        .policy_request_document(
            &headers,
            client.map(|Extension(ip)| ip),
            &company,
            &op,
            &command,
        )
        .await
    {
        Ok(value) => value,
        Err(status) => return error(status),
    };
    let NativePolicyRequestDocument::Visible {
        status,
        current,
        proof,
    } = result
    else {
        return ui::document(ui::Page::NotVisible, StatusCode::NOT_FOUND);
    };
    let identity = match identity_context(&state, &headers, &company).await {
        Ok(value) => value,
        Err(status) => return error(status),
    };
    let selector = current.selector;
    let (outcome, original) = match status {
        NativePolicyStatus::NotVisible => {
            return ui::document(ui::Page::NotVisible, StatusCode::NOT_FOUND);
        }
        NativePolicyStatus::AcceptedPending(accepted) => {
            let Some(proof) = proof else {
                return error(StatusCode::SERVICE_UNAVAILABLE);
            };
            (
                ui::Outcome::Pending {
                    accepted_at: date(accepted.accepted_at),
                    deadline: date(accepted.execution_not_after),
                    proof: proof.as_str().to_owned(),
                },
                original(&accepted),
            )
        }
        NativePolicyStatus::AcceptedExpired(accepted) => (
            ui::Outcome::Expired {
                accepted_at: date(accepted.accepted_at),
            },
            original(&accepted),
        ),
        NativePolicyStatus::Terminal(terminal) => {
            let mut record = original(&terminal.accepted);
            record.effect_epochs = Some((
                terminal.epoch_before.to_string(),
                terminal.epoch_after.to_string(),
            ));
            let receipt = terminal.receipt_id.to_string();
            let at = date(terminal.executed_at);
            let outcome = match terminal.outcome {
                NativePolicyOutcome::Committed(effect) => {
                    let (title, description) = match effect {
                        NativePolicyEffect::Installed { .. } => {
                            ("설정 준비 완료", "계정에 열람 권한은 연결되지 않았습니다")
                        }
                        NativePolicyEffect::Granted { assignment, .. } => {
                            record.effect_period =
                                Some((date(assignment.valid_from), date(assignment.valid_until)));
                            (
                                "열람 권한을 연결했습니다",
                                "이 요청에서 확정된 열람 기간입니다. 현재 권한 상태도 함께 확인하세요.",
                            )
                        }
                        NativePolicyEffect::Revoked { .. } => (
                            "열람 권한을 회수했습니다",
                            "이 요청으로 연결한 권한을 회수했습니다. 이전 처리 기록은 보존됩니다.",
                        ),
                    };
                    ui::Outcome::Committed {
                        title,
                        description,
                        receipt,
                        at,
                    }
                }
                NativePolicyOutcome::Rejected(reason) => ui::Outcome::Rejected {
                    receipt,
                    at,
                    description: match reason {
                        NativePolicyRejection::IntakeExpired => {
                            "요청의 처리 기한이 지나 적용되지 않았습니다."
                        }
                        NativePolicyRejection::RevisionConflict => {
                            "요청을 작성한 뒤 회사 권한이 변경되었습니다. 현재 상태를 확인하고 다시 시작하세요."
                        }
                        NativePolicyRejection::GrantExpiryInvalid => {
                            "선택한 종료 시각은 처리 시점 이후 30일 이내여야 합니다."
                        }
                        NativePolicyRejection::RecipientIneligible => {
                            "대상 계정은 현재 이 권한을 받을 수 없습니다."
                        }
                    },
                },
            };
            (outcome, record)
        }
    };
    ui::document(
        ui::Page::Result {
            scope: scope(current, identity, payroll_link),
            operation: operation(selector.operation()),
            command: selector.command_id().to_string(),
            outcome,
            original,
        },
        StatusCode::OK,
    )
}
async fn submit(
    state: CompanyState,
    company: String,
    target: NativePolicyPostTarget<'_>,
    request: Request,
) -> Response {
    let identity = match identity_context(&state, request.headers(), &company).await {
        Ok(value) => value,
        Err(status) => return error(status),
    };
    match state
        .policy_submit_document(request, &company, target)
        .await
    {
        Ok(NativePolicySubmission::Validation { current, draft }) => {
            let current_assignment = current.view.assignment.as_ref().map(|a| a.expectation);
            let current_matches = current.view.company_epoch == draft.expected_company_epoch
                && current.view.administrative_account_id == draft.recipient_account_id
                && current_assignment == draft.assignment;
            let validation = ui::GrantValidation {
                expected_epoch: draft.expected_company_epoch.to_string(),
                recipient: draft.recipient_account_id.as_uuid().to_string(),
                assignment: draft.assignment.map(|a| {
                    (
                        a.role_revision.to_string(),
                        a.assignment_id.to_string(),
                        a.assignment_revision.to_string(),
                    )
                }),
                expires_at_local: draft.expires_at_local,
                current_matches,
            };
            ui::document(
                ui::Page::Form(ui::Form {
                    scope: scope(current.view, identity, false),
                    operation: operation(draft.selector.operation()),
                    command: draft.selector.command_id().to_string(),
                    proof: current.proof.as_str().to_owned(),
                    validation: Some(validation),
                }),
                StatusCode::UNPROCESSABLE_ENTITY,
            )
        }
        Ok(NativePolicySubmission::Confirmed { selector }) => {
            let mut response = ui::document(ui::Page::NotVisible, StatusCode::SEE_OTHER);
            match HeaderValue::from_str(&result_path(selector)) {
                Ok(location) => {
                    response.headers_mut().insert(header::LOCATION, location);
                    response
                }
                Err(_) => error(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        Ok(NativePolicySubmission::Unconfirmed { selector }) => ui::document(
            ui::Page::Uncertain {
                result_path: result_path(selector),
            },
            StatusCode::SERVICE_UNAVAILABLE,
        ),
        Err(status) => error(status),
    }
}
pub(super) async fn install(
    State(state): State<CompanyState>,
    Path(company): Path<String>,
    request: Request,
) -> Response {
    submit(state, company, NativePolicyPostTarget::Install, request).await
}
pub(super) async fn grant(
    State(state): State<CompanyState>,
    Path(company): Path<String>,
    request: Request,
) -> Response {
    submit(state, company, NativePolicyPostTarget::Grant, request).await
}
pub(super) async fn revoke(
    State(state): State<CompanyState>,
    Path((company, assignment)): Path<(String, String)>,
    request: Request,
) -> Response {
    submit(
        state,
        company,
        NativePolicyPostTarget::Revoke {
            assignment: &assignment,
        },
        request,
    )
    .await
}
pub(super) async fn retry(
    State(state): State<CompanyState>,
    Path((company, operation, command)): Path<(String, String, String)>,
    request: Request,
) -> Response {
    submit(
        state,
        company,
        NativePolicyPostTarget::Retry {
            operation: &operation,
            command: &command,
        },
        request,
    )
    .await
}
