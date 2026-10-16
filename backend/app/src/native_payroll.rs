//! Composition and authorized view mapping only. Payroll owns the read.
use crate::{AppState, account_custody, payroll_runs_reader};
use axum::{
    Router,
    extract::{MatchedPath, Path, RawQuery, Request, State, rejection::PathRejection},
    http::{HeaderMap, HeaderValue, Method, StatusCode, header},
    middleware::{Next, from_fn},
    response::Response,
    routing::any,
};
use console_payroll_adapter_postgres::{PgNativePayrollRunsReadPort, PgPayrollStore};
use console_payroll_rest::PayrollRestState;
use console_payroll_ui::native_payroll::{Collection, CompanyIdentity, Page, Run, document};
use console_platform_authz::company_policy::CompanyPolicy;
use sqlx::PgPool;
use std::sync::Arc;

const DOCUMENT_PATH: &str = "/companies/{org_id}/payroll";

/// After the shared timeout/envelope, retain native error representation.
/// MatchedPath is server routing metadata, never a source of authority.
pub(super) fn with_transport(router: Router) -> Router {
    router.layer(from_fn(timeout_response))
}

async fn timeout_response(request: Request, next: Next) -> Response {
    let document_route = match request
        .extensions()
        .get::<MatchedPath>()
        .map(MatchedPath::as_str)
    {
        Some(DOCUMENT_PATH) => Some(true),
        Some(console_payroll_rest::NATIVE_PAYROLL_RUNS_PATH) => Some(false),
        _ => None,
    };
    let head = request.method() == Method::HEAD;
    let mut response = next.run(request).await;
    if response.status() != StatusCode::REQUEST_TIMEOUT {
        return response;
    }
    match document_route {
        Some(true) => response = document(Page::Unavailable, StatusCode::REQUEST_TIMEOUT),
        Some(false) => {
            for (name, value) in [
                (header::CACHE_CONTROL, "no-store"),
                (header::PRAGMA, "no-cache"),
                (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
                (header::VARY, "Cookie, Origin"),
            ] {
                response
                    .headers_mut()
                    .insert(name, HeaderValue::from_static(value));
            }
        }
        None => return response,
    }
    if head {
        *response.body_mut() = axum::body::Body::empty();
    }
    response
}

/// Composition availability only; the Payroll owner decides each viewer's access.
#[derive(Clone)]
pub(super) struct Navigation(pub(super) Option<PayrollRestState>);

impl Navigation {
    pub(super) async fn visible(
        &self,
        headers: &HeaderMap,
        company: &str,
    ) -> Result<bool, StatusCode> {
        match &self.0 {
            Some(state) => state.native_document_navigation(headers, company).await,
            None => Ok(false),
        }
    }
}

pub(super) fn state(state: &AppState, pool: &PgPool) -> PayrollRestState {
    let mut payroll = PayrollRestState::new(
        PgPayrollStore::new(pool.clone()),
        state.session_verification(),
        payroll_runs_reader(pool),
    );
    if state
        .serving_custody_profile
        .is_some_and(account_custody::VerifiedCustodyProfile::supports_policy)
        && let (Some(auth), Some(verifier), Some(config), Ok(policy)) = (
            &state.auth_rest,
            &state.jwt_verifier,
            &state.config.auth_rest,
            CompanyPolicy::new(),
        )
    {
        let pool = pool.clone();
        let verifier = verifier.clone();
        let ttl = config.refresh_family_absolute_ttl;
        let policy = Arc::new(policy);
        payroll = payroll.with_native_accounts(
            auth.clone(),
            Arc::new(move |credentials, company| {
                Box::new(PgNativePayrollRunsReadPort::new(
                    pool.clone(),
                    verifier.clone(),
                    ttl,
                    policy.clone(),
                    credentials,
                    company,
                ))
            }),
        );
    }
    payroll
}

pub(super) fn router(payroll: PayrollRestState) -> Router {
    console_payroll_rest::native_router(payroll.clone()).merge(
        Router::new()
            .route(DOCUMENT_PATH, any(collection))
            .with_state(payroll),
    )
}

#[cfg(test)]
#[path = "native_payroll_timeout_tests.rs"]
mod timeout_tests;

async fn collection(
    State(state): State<PayrollRestState>,
    people: Option<axum::Extension<super::native_people::PeopleState>>,
    path: Result<Path<String>, PathRejection>,
    RawQuery(query): RawQuery,
    headers: HeaderMap,
    method: Method,
) -> Response {
    if method != Method::GET {
        return document(Page::InvalidRequest, StatusCode::METHOD_NOT_ALLOWED);
    }
    let Ok(Path(company)) = path else {
        return document(Page::NotVisible, StatusCode::NOT_FOUND);
    };
    let people_navigation = match people {
        Some(axum::Extension(people)) => {
            match super::native_people::navigation(&people, &headers, &company).await {
                Ok(value) => value,
                Err(status) => {
                    return console_platform_request_context::preserve_native_html_error(document(
                        Page::Unavailable,
                        status,
                    ));
                }
            }
        }
        None => (false, false),
    };
    let result = match state
        .native_document(&headers, &company, query.as_deref())
        .await
    {
        Ok(result) => result,
        Err(status) => {
            let page = match status {
                StatusCode::UNAUTHORIZED => Page::AuthenticationRequired,
                StatusCode::SERVICE_UNAVAILABLE => Page::Unavailable,
                StatusCode::BAD_REQUEST => Page::InvalidRequest,
                _ => Page::NotVisible,
            };
            return console_platform_request_context::preserve_native_html_error(document(
                page, status,
            ));
        }
    };
    let Some(context) = result.company else {
        return document(Page::Unavailable, StatusCode::SERVICE_UNAVAILABLE);
    };
    let page = result.page;
    document(
        Page::Runs(Collection {
            company: context.id.to_string(),
            people_navigation,
            identity: context.identity.map(|identity| CompanyIdentity {
                name: identity.name,
                slug: identity.slug,
            }),
            total: page.total,
            limit: page.limit,
            offset: page.offset,
            items: page
                .items
                .into_iter()
                .map(|run| Run {
                    id: run.id.to_string(),
                    period_start: run.period_start.to_string(),
                    period_end: run.period_end.to_string(),
                    source_label: run.source_label,
                    status: run.status,
                    calculation_enabled: run.calculation_enabled,
                    created_by: run.created_by.map(|v| v.to_string()),
                    approved_by: run.approved_by.map(|v| v.to_string()),
                    approved_at: run.approved_at.map(|v| v.to_string()),
                    close_receipt: run.close_receipt,
                    submitted_by: run.submitted_by.map(|v| v.to_string()),
                    submitted_at: run.submitted_at.map(|v| v.to_string()),
                    decided_by: run.decided_by.map(|v| v.to_string()),
                    decided_at: run.decided_at.map(|v| v.to_string()),
                    decision_reason: run.decision_reason,
                    approval_ref: run.approval_ref.map(|v| v.to_string()),
                    created_at: run.created_at.to_string(),
                    updated_at: run.updated_at.to_string(),
                })
                .collect(),
        }),
        StatusCode::OK,
    )
}
