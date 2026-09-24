//! Native People route composition and authorized presentation, with no SQL.
use axum::{
    Router,
    extract::{Extension, MatchedPath, Path, RawQuery, Request, State, rejection::PathRejection},
    http::{HeaderMap, HeaderValue, Method, StatusCode, header},
    middleware::{Next, from_fn},
    response::{IntoResponse, Response},
    routing::any,
};
use console_identity_adapter_postgres::PgOrgStore;
use console_identity_rest::company::CompanyRestState;
use console_ontology_application::people::{workflow::*, *};
use console_ontology_canonical_adapter_postgres::native_directory::{
    DirectoryCedarDecision, PgNativeDirectoryStore,
};
use console_ontology_rest::native_people::{
    NativePeopleRestState, PostTarget, RecoveryLocator, Submission,
};
use console_payroll_ui::native_people as ui;
use console_platform_auth::account::AccountFormProof;
use console_platform_request_context::TrustedClientIp;
use time::OffsetDateTime;

type PeopleOwner = NativePeopleRestState<PgNativeDirectoryStore, DirectoryCedarDecision>;
#[derive(Clone)]
pub(super) struct PeopleState {
    // Root sets Some only for the verified activated native-directory profile.
    // Absence is unavailable; there is no fallback store or deployment shortcut.
    pub(super) owner: Option<PeopleOwner>,
    pub(super) company: Option<CompanyRestState<PgOrgStore>>,
    pub(super) payroll: super::native_payroll::Navigation,
}
pub(super) fn state(
    app: &super::AppState,
    pool: &sqlx::PgPool,
    payroll: console_payroll_rest::PayrollRestState,
) -> PeopleState {
    let ready = app
        .serving_custody_profile
        .is_some_and(super::account_custody::VerifiedCustodyProfile::supports_native_directory);
    let owner = if ready {
        match (
            &app.auth_rest,
            &app.jwt_verifier,
            &app.native_policy_issuer,
            &app.config.auth_rest,
            console_platform_authz::company_policy::CompanyPolicy::new(),
        ) {
            (Some(auth), Some(verifier), Some(issuer), Some(config), Ok(policy)) => {
                PgNativeDirectoryStore::new(
                    pool.clone(),
                    verifier.clone(),
                    issuer.clone(),
                    config.refresh_family_absolute_ttl,
                )
                .ok()
                .map(|store| {
                    NativePeopleRestState::new(
                        store,
                        DirectoryCedarDecision::new(std::sync::Arc::new(policy)),
                        auth.clone(),
                    )
                })
            }
            _ => None,
        }
    } else {
        None
    };
    PeopleState {
        owner,
        company: app.company_rest.clone(),
        payroll: super::native_payroll::Navigation(
            app.serving_custody_profile
                .is_some_and(super::account_custody::VerifiedCustodyProfile::supports_policy)
                .then_some(payroll),
        ),
    }
}
#[derive(serde::Deserialize)]
struct Route {
    org_id: String,
    employee: Option<String>,
    command: Option<String>,
}
const LIST: &str = "/companies/{org_id}/people";
const NEW: &str = "/companies/{org_id}/people/new";
const PREPARE: &str = "/companies/{org_id}/people/requests";
const REQUEST: &str = "/companies/{org_id}/people/requests/{command}";
const EXECUTE: &str = "/companies/{org_id}/people/requests/{command}/execute";
const CANCEL: &str = "/companies/{org_id}/people/requests/{command}/cancel";
const DETAIL: &str = "/companies/{org_id}/people/{employee}";
pub(super) fn router(state: PeopleState) -> Router {
    Router::new()
        .route(LIST, any(read))
        .route(NEW, any(read))
        .route(REQUEST, any(read))
        .route(DETAIL, any(read))
        .route(PREPARE, any(write))
        .route(EXECUTE, any(write))
        .route(CANCEL, any(write))
        .with_state(state)
}
/// Same outer envelope pattern as native_payroll::with_transport. Mount outside
/// the shared TimeoutLayer so interrupted commands retain their recovery URL.
pub(super) fn with_transport(router: Router) -> Router {
    router.layer(from_fn(timeout_response))
}
async fn timeout_response(mut request: Request, next: Next) -> Response {
    let applies = request.extensions().get::<MatchedPath>().is_some_and(|p| {
        matches!(
            p.as_str(),
            LIST | NEW | PREPARE | REQUEST | EXECUTE | CANCEL | DETAIL
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
        response = match recovery.get() {
            Some(locator) => document(
                ui::Page::Uncertain {
                    company: locator.company().to_string(),
                    command: locator.command_id().to_string(),
                },
                StatusCode::REQUEST_TIMEOUT,
            ),
            None => document(ui::Page::Unavailable, StatusCode::REQUEST_TIMEOUT),
        };
    }
    if head {
        *response.body_mut() = axum::body::Body::empty();
    }
    response
}
pub(super) async fn navigation(
    state: &PeopleState,
    headers: &HeaderMap,
    company: &str,
) -> Result<(bool, bool), StatusCode> {
    let Some(owner) = state.owner.as_ref() else {
        return Ok((false, false));
    };
    let read = owner
        .navigation(headers, company, DirectoryAction::Read)
        .await?;
    let create = owner
        .navigation(headers, company, DirectoryAction::Create)
        .await?;
    Ok((read, create))
}
async fn scope(
    state: &PeopleState,
    headers: &HeaderMap,
    company: &str,
) -> Result<ui::Scope, StatusCode> {
    let (directory_link, can_create) = navigation(state, headers, company).await?;
    let payroll_link = state.payroll.visible(headers, company).await?;
    let identity = match state.company.as_ref() {
        Some(owner) => match owner.company_document(headers, company).await {
            Ok(identity) => Some(identity),
            Err(StatusCode::NOT_FOUND | StatusCode::FORBIDDEN) => None,
            Err(status) => return Err(status),
        },
        None => None,
    };
    Ok(ui::Scope {
        company: company.to_owned(),
        company_name: identity.as_ref().map(|i| i.name.clone()),
        company_link: identity.is_some(),
        directory_link,
        can_create,
        payroll_link,
        policy_link: identity.as_ref().is_some_and(|i| i.show_policy_navigation),
    })
}
async fn read(
    State(state): State<PeopleState>,
    path: Result<Path<Route>, PathRejection>,
    matched: MatchedPath,
    RawQuery(query): RawQuery,
    headers: HeaderMap,
    method: Method,
    client: Option<Extension<TrustedClientIp>>,
    recovery: Option<Extension<RecoveryLocator>>,
) -> Response {
    if method != Method::GET {
        return error(StatusCode::METHOD_NOT_ALLOWED);
    }
    let Ok(Path(route)) = path else {
        return error(StatusCode::NOT_FOUND);
    };
    let Some(owner) = state.owner.as_ref() else {
        return error(StatusCode::SERVICE_UNAVAILABLE);
    };
    // Navigation happens first. The displayed owner result is reauthorized and
    // committed last, so no identity/proof is retained across navigation waits.
    let scope = match scope(&state, &headers, &route.org_id).await {
        Ok(scope) => scope,
        Err(status) => return error(status),
    };
    let result: Result<ui::Page, StatusCode> = match matched.as_str() {
        LIST => owner
            .list(&headers, &route.org_id, query.as_deref())
            .await
            .map(|page| ui::Page::Directory {
                scope,
                records: page.records.into_iter().map(record).collect(),
                next_after: page.next_after.map(|id| id.to_string()),
            }),
        NEW => owner
            .registration(
                &headers,
                client.map(|Extension(ip)| ip),
                &route.org_id,
                query.as_deref(),
            )
            .await
            .map(|form| ui::Page::Registration {
                scope,
                form: registration(form, String::new(), String::new(), None, None, None),
            }),
        DETAIL => match route.employee.as_deref() {
            Some(employee) => match owner
                .detail(&headers, &route.org_id, employee, query.as_deref())
                .await
            {
                Ok(Some(entry)) => Ok(ui::Page::Detail {
                    scope,
                    record: record(entry),
                }),
                Ok(None) => Err(StatusCode::NOT_FOUND),
                Err(status) => Err(status),
            },
            None => Err(StatusCode::NOT_FOUND),
        },
        REQUEST => match route.command.as_deref() {
            Some(command) => match owner
                .request(
                    &headers,
                    client.map(|Extension(ip)| ip),
                    &route.org_id,
                    command,
                    query.as_deref(),
                    recovery.as_ref().map(|Extension(value)| value),
                )
                .await
            {
                Ok(result) if matches!(result.status, DirectoryStatus::NotVisible) => {
                    return document(
                        ui::Page::RequestNotVisible {
                            scope,
                            command: command.to_owned(),
                        },
                        StatusCode::NOT_FOUND,
                    );
                }
                Ok(result) => request_page(scope, result),
                Err(status) => Err(status),
            },
            None => Err(StatusCode::NOT_FOUND),
        },
        _ => Err(StatusCode::NOT_FOUND),
    };
    match result {
        Ok(page) => document(page, StatusCode::OK),
        Err(status) => error(status),
    }
}
async fn write(
    State(state): State<PeopleState>,
    path: Result<Path<Route>, PathRejection>,
    matched: MatchedPath,
    request: Request,
) -> Response {
    if request.method() != Method::POST {
        return error(StatusCode::METHOD_NOT_ALLOWED);
    }
    let Ok(Path(route)) = path else {
        return error(StatusCode::NOT_FOUND);
    };
    let Some(owner) = state.owner.as_ref() else {
        return error(StatusCode::SERVICE_UNAVAILABLE);
    };
    let target = match (matched.as_str(), route.command.as_deref()) {
        (PREPARE, None) => PostTarget::Prepare,
        (EXECUTE, Some(command)) => PostTarget::Execute(command),
        (CANCEL, Some(command)) => PostTarget::Cancel(command),
        _ => return error(StatusCode::NOT_FOUND),
    };
    let scope = match scope(&state, request.headers(), &route.org_id).await {
        Ok(scope) => scope,
        Err(status) => return error(status),
    };
    match owner.submit(request, &route.org_id, target).await {
        Ok(Submission::Confirmed(locator)) => redirect(locator),
        Ok(Submission::Unconfirmed(locator)) => document(
            ui::Page::Uncertain {
                company: locator.company().to_string(),
                command: locator.command_id().to_string(),
            },
            StatusCode::SERVICE_UNAVAILABLE,
        ),
        Ok(Submission::Validation {
            current,
            draft,
            problem,
        }) => {
            let (status, form_error) = match problem {
                Some(DirectoryWorkflowError::Capacity) => (
                    StatusCode::TOO_MANY_REQUESTS,
                    Some(
                        "처리 중인 등록 요청이 한도에 도달했습니다. 기존 요청을 완료하거나 취소한 뒤 같은 요청을 다시 제출하세요.",
                    ),
                ),
                Some(DirectoryWorkflowError::Conflict) => (
                    StatusCode::CONFLICT,
                    Some(
                        "요청 당시의 회사 설정이나 요청 내용이 현재 기록과 일치하지 않습니다. 원래 요청의 결과를 확인하고 새 요청을 작성하세요.",
                    ),
                ),
                _ => (StatusCode::UNPROCESSABLE_ENTITY, None),
            };
            document(
                ui::Page::Registration {
                    scope,
                    form: registration(
                        current,
                        draft.legal_name,
                        draft.employee_number,
                        draft.name_error,
                        draft.number_error,
                        form_error,
                    ),
                },
                status,
            )
        }
        Err(status) => error(status),
    }
}
fn registration(
    form: DirectoryForm<AccountFormProof>,
    legal_name: String,
    employee_number: String,
    name_error: Option<DirectoryInputProblem>,
    number_error: Option<DirectoryInputProblem>,
    form_error: Option<&'static str>,
) -> ui::Registration {
    let e = form.expected;
    ui::Registration {
        command: form.locator.command_id().to_string(),
        proof: form.proof.as_str().to_owned(),
        expected: ui::Expectations {
            company_epoch: e.company_epoch.to_string(),
            object_type_id: e.object_type_id.to_string(),
            action_type_id: e.action_type_id.to_string(),
            action_revision: e.action_revision.to_string(),
            schema_revision: e.schema_revision.to_string(),
            legal_name_property_id: e.legal_name_property_id.to_string(),
            employee_number_property_id: e.employee_number_property_id.to_string(),
        },
        legal_name,
        employee_number,
        name_error: name_error.map(input_problem),
        number_error: number_error.map(input_problem),
        form_error,
    }
}
fn input_problem(problem: DirectoryInputProblem) -> &'static str {
    match problem {
        DirectoryInputProblem::Required => "입력해 주세요.",
        DirectoryInputProblem::TooLong => "허용된 글자 수를 초과했습니다.",
        DirectoryInputProblem::ControlCharacter => "줄바꿈이나 제어 문자는 사용할 수 없습니다.",
    }
}
fn record(entry: DirectoryRecord) -> ui::Record {
    ui::Record {
        employee_id: entry.employee_id.to_string(),
        person_id: entry.person_id.to_string(),
        legal_name: entry.legal_name,
        employee_number: entry.employee_number,
        person_version: entry.person_version.to_string(),
        registered_at: date(entry.registered_at),
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
fn request_page(
    scope: ui::Scope,
    result: DirectoryRecovery<AccountFormProof>,
) -> Result<ui::Page, StatusCode> {
    let (accepted, outcome) = match result.status {
        DirectoryStatus::NotVisible => return Err(StatusCode::NOT_FOUND),
        DirectoryStatus::Pending(accepted) => {
            let proof = result.proof.ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
            (
                accepted,
                ui::Outcome::Pending {
                    proof: proof.as_str().to_owned(),
                },
            )
        }
        DirectoryStatus::Terminal(terminal) => {
            if result.proof.is_some() {
                return Err(StatusCode::SERVICE_UNAVAILABLE);
            }
            let outcome = match terminal.outcome() {
                DirectoryTerminalOutcomeV1::Committed => ui::Outcome::Committed {
                    employee_id: terminal
                        .employee_id()
                        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?
                        .to_string(),
                    person_id: terminal
                        .person_id()
                        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?
                        .to_string(),
                    receipt: terminal
                        .canonical_command_id()
                        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?
                        .to_string(),
                    registered_at: date(terminal.terminal_at()),
                },
                DirectoryTerminalOutcomeV1::Rejected(DirectoryRejectionV1::RevisionConflict) => {
                    ui::Outcome::Conflicting {
                        reason: "회사 설정이 변경되어 등록하지 않았습니다. 현재 설정으로 새 요청을 작성하세요.",
                    }
                }
                DirectoryTerminalOutcomeV1::Rejected(
                    DirectoryRejectionV1::EmployeeNumberConflict,
                ) => ui::Outcome::Rejected {
                    reason: "이미 등록된 사번입니다. 사번을 확인하고 새 요청을 작성하세요.",
                },
                DirectoryTerminalOutcomeV1::Rejected(DirectoryRejectionV1::CommandConflict) => {
                    ui::Outcome::Conflicting {
                        reason: "같은 요청 식별자로 다른 처리가 기록되어 등록하지 않았습니다. 원래 요청을 확인하세요.",
                    }
                }
                DirectoryTerminalOutcomeV1::Cancelled => ui::Outcome::Cancelled,
                DirectoryTerminalOutcomeV1::Expired => ui::Outcome::Expired,
            };
            (terminal.accepted().clone(), outcome)
        }
    };
    let command = accepted.command();
    Ok(ui::Page::Request {
        scope,
        request: ui::Request {
            command: command.command_id().to_string(),
            legal_name: command.input().legal_name().to_owned(),
            employee_number: command.input().employee_number().to_owned(),
            accepted_at: date(accepted.accepted_at()),
            deadline: date(accepted.execution_not_after()),
            intake_receipt: accepted.intake_receipt_id().to_string(),
            expected_company_epoch: command.expected().company_epoch.to_string(),
            outcome,
        },
    })
}
fn redirect(locator: DirectoryRequestRef) -> Response {
    let mut response = StatusCode::SEE_OTHER.into_response();
    let path = format!(
        "/companies/{}/people/requests/{}",
        locator.company(),
        locator.command_id()
    );
    let Ok(location) = HeaderValue::from_str(&path) else {
        return error(StatusCode::SERVICE_UNAVAILABLE);
    };
    response.headers_mut().insert(header::LOCATION, location);
    for (name, value) in [
        (header::CACHE_CONTROL, "no-store"),
        (header::PRAGMA, "no-cache"),
        (header::VARY, "Cookie, Origin"),
        (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
    ] {
        response
            .headers_mut()
            .insert(name, HeaderValue::from_static(value));
    }
    response
}
fn error(status: StatusCode) -> Response {
    let page = if status.is_server_error() || status == StatusCode::REQUEST_TIMEOUT {
        ui::Page::Unavailable
    } else {
        ui::Page::Refused
    };
    console_platform_request_context::preserve_native_html_error(document(page, status))
}

fn document(page: ui::Page, status: StatusCode) -> Response {
    let response = ui::document(page, status);
    if status.is_client_error() || status.is_server_error() {
        console_platform_request_context::preserve_native_html_error(response)
    } else {
        response
    }
}

#[cfg(test)]
#[path = "native_people_timeout_tests.rs"]
mod timeout_tests;
