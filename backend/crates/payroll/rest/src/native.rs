//! Native Account transport; the application read owner retains all authority.
use super::PayrollRestState;
use axum::{
    Json, Router,
    extract::{Path, RawQuery, State, rejection::PathRejection},
    http::{HeaderMap, HeaderValue, Method, StatusCode, header},
    response::{IntoResponse, Response},
    routing::any,
};
use console_kernel_core::{ErrorKind, OrgId};
use console_payroll_application::read::{
    ListPayrollRuns, PayrollRunsReadError, PayrollRunsReadPort, PayrollRunsReadResult,
    list_payroll_runs, payroll_navigation,
};
use console_platform_auth::account::AccountEnrollmentCredentials;
use console_platform_auth_rest::AuthRestState;
use std::sync::Arc;
use uuid::Uuid;

pub const NATIVE_PAYROLL_RUNS_PATH: &str = "/api/v1/companies/{org_id}/payroll/runs";
pub type NativePayrollRunsReaderFactory =
    Arc<dyn Fn(AccountEnrollmentCredentials, OrgId) -> Box<dyn PayrollRunsReadPort> + Send + Sync>;

#[derive(Clone)]
pub(super) struct NativeAccounts {
    auth: AuthRestState,
    reader: NativePayrollRunsReaderFactory,
}

impl PayrollRestState {
    pub fn with_native_accounts(
        mut self,
        auth: AuthRestState,
        reader: NativePayrollRunsReaderFactory,
    ) -> Self {
        self.native = Some(NativeAccounts { auth, reader });
        self
    }

    pub async fn native_document(
        &self,
        headers: &HeaderMap,
        company: &str,
        query: Option<&str>,
    ) -> Result<PayrollRunsReadResult, StatusCode> {
        let native = self
            .native
            .as_ref()
            .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
        let credentials = native
            .auth
            .company_document_credentials(headers)
            .map_err(|error| error.status())?;
        native.listing(credentials, company, query).await
    }

    pub async fn native_document_navigation(
        &self,
        headers: &HeaderMap,
        company: &str,
    ) -> Result<bool, StatusCode> {
        let native = self
            .native
            .as_ref()
            .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
        let credentials = native
            .auth
            .company_document_credentials(headers)
            .map_err(|error| error.status())?;
        let company = company_id(company)?;
        match payroll_navigation((native.reader)(credentials, company).as_mut()).await {
            Ok(actual) if actual == company => Ok(true),
            Ok(_) => Err(StatusCode::SERVICE_UNAVAILABLE),
            Err(error) => match owner_status(error) {
                StatusCode::NOT_FOUND => Ok(false),
                status => Err(status),
            },
        }
    }
}

fn company_id(company: &str) -> Result<OrgId, StatusCode> {
    let id = Uuid::parse_str(company).map_err(|_| StatusCode::NOT_FOUND)?;
    let company_id = OrgId::from_uuid(id);
    if id.is_nil() || company_id == OrgId::platform() || id.to_string() != company {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(company_id)
}

fn owner_status(error: PayrollRunsReadError) -> StatusCode {
    match error {
        PayrollRunsReadError::AuthenticationInvalid => StatusCode::UNAUTHORIZED,
        PayrollRunsReadError::Authorization(error) if error.kind == ErrorKind::NotFound => {
            StatusCode::NOT_FOUND
        }
        PayrollRunsReadError::Authorization(_)
        | PayrollRunsReadError::Read(_)
        | PayrollRunsReadError::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
    }
}

impl NativeAccounts {
    async fn listing(
        &self,
        credentials: AccountEnrollmentCredentials,
        company: &str,
        query: Option<&str>,
    ) -> Result<PayrollRunsReadResult, StatusCode> {
        let company_id = company_id(company)?;
        let query = pagination(query)?;
        let result = list_payroll_runs((self.reader)(credentials, company_id).as_mut(), query)
            .await
            .map_err(owner_status)?;
        if result
            .company
            .as_ref()
            .is_none_or(|context| context.id != company_id)
        {
            return Err(StatusCode::SERVICE_UNAVAILABLE);
        }
        Ok(result)
    }
}

// Only the two integer selectors emitted by the native form are admitted.
// No ignored keys, duplicates, percent aliases, empty values or overflow.
fn pagination(raw: Option<&str>) -> Result<ListPayrollRuns, StatusCode> {
    let mut query = ListPayrollRuns {
        limit: None,
        offset: None,
    };
    let Some(raw) = raw.filter(|raw| !raw.is_empty()) else {
        return Ok(query);
    };
    if raw.len() > 128 {
        return Err(StatusCode::BAD_REQUEST);
    }
    for pair in raw.split('&') {
        let (key, value) = pair.split_once('=').ok_or(StatusCode::BAD_REQUEST)?;
        let target = match key {
            "limit" => &mut query.limit,
            "offset" => &mut query.offset,
            _ => return Err(StatusCode::BAD_REQUEST),
        };
        let digits = value.strip_prefix('-').unwrap_or(value);
        if target.is_some() || digits.is_empty() || !digits.bytes().all(|c| c.is_ascii_digit()) {
            return Err(StatusCode::BAD_REQUEST);
        }
        *target = Some(value.parse().map_err(|_| StatusCode::BAD_REQUEST)?);
    }
    Ok(query)
}

pub fn native_router(state: PayrollRestState) -> Router {
    Router::new()
        .route(NATIVE_PAYROLL_RUNS_PATH, any(runs))
        .with_state(state)
}

async fn runs(
    State(state): State<PayrollRestState>,
    path: Result<Path<String>, PathRejection>,
    RawQuery(query): RawQuery,
    headers: HeaderMap,
    method: Method,
) -> Response {
    // Axum GET also handles HEAD. Refuse it before credentials/owner/audit.
    if method != Method::GET {
        return error(StatusCode::METHOD_NOT_ALLOWED);
    }
    let Ok(Path(company)) = path else {
        return error(StatusCode::NOT_FOUND);
    };
    let Some(native) = state.native.as_ref() else {
        return error(StatusCode::SERVICE_UNAVAILABLE);
    };
    let credentials = match native.auth.company_api_read_credentials(&headers) {
        Ok(credentials) => credentials,
        Err(response) => return private(*response),
    };
    match native
        .listing(credentials, &company, query.as_deref())
        .await
    {
        Ok(result) => private(Json(result.page).into_response()),
        Err(status) => error(status),
    }
}

fn error(status: StatusCode) -> Response {
    let code = match status {
        StatusCode::NOT_FOUND => "not_found",
        StatusCode::UNAUTHORIZED => "authentication_invalid",
        StatusCode::BAD_REQUEST => "invalid_request",
        StatusCode::METHOD_NOT_ALLOWED => "method_not_allowed",
        _ => "payroll_unavailable",
    };
    private(
        (
            status,
            Json(serde_json::json!({"error":{"code":code,"message":code}})),
        )
            .into_response(),
    )
}

fn private(mut response: Response) -> Response {
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
    response
}
