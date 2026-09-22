//! Native Account Company transport. Authorization and effects stay in the
//! application use cases and their retained owning transactions.
use std::sync::Arc;

use axum::{
    Json, Router,
    body::to_bytes,
    extract::{Path, Request, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use console_identity_application::{
    CompanyEnrollmentV1,
    company_enrollment::{
        CompanyEnrollmentError, CompanyEnrollmentReceipt, CompanyEnrollmentStatus,
        CompanyEnrollmentStore, cancel_company_enrollment, company_enrollment_status,
        enroll_company,
    },
    company_policy::{
        ActionRef, CompanyContextView, CompanyIdentityView, CompanyPolicyError, CompanyPolicyStore,
        CompanyPolicyView, PropertyRef, discover_company_context, discover_company_contexts,
        read_company_identity, read_company_policy,
    },
};
use console_kernel_core::{OrgId, TraceContext};
use console_platform_auth::account::AccountEnrollmentCredentials;
use console_platform_auth_rest::AuthRestState;
use console_platform_authz::company_policy::CompanyPolicy;
use serde_json::{Value, json};
use uuid::Uuid;

#[derive(Clone)]
pub struct CompanyRestState<S> {
    store: S,
    auth: AuthRestState,
    policy: Option<Arc<CompanyPolicy>>,
}

impl<S> CompanyRestState<S>
where
    S: CompanyEnrollmentStore<Credentials = AccountEnrollmentCredentials>
        + CompanyPolicyStore<Credentials = AccountEnrollmentCredentials>
        + Clone
        + Send
        + Sync
        + 'static,
{
    pub fn new(store: S, auth: AuthRestState) -> Self {
        Self {
            store,
            auth,
            policy: CompanyPolicy::new().ok().map(Arc::new),
        }
    }

    /// Only committed, currently authorized projections leave the read owners.
    pub async fn enrollment_document(
        &self,
        headers: &HeaderMap,
        command: &str,
    ) -> Result<EnrollmentDocument, StatusCode> {
        let credentials = self
            .auth
            .company_document_credentials(headers)
            .map_err(|e| e.status())?;
        let command = command_id(command).map_err(|_| StatusCode::NOT_FOUND)?;
        let status = company_enrollment_status(&self.store, &credentials, command)
            .await
            .map_err(|e| enrollment_error(e).status())?;
        let company = if let CompanyEnrollmentStatus::Committed { org_id, .. } = &status {
            let policy = self
                .policy
                .as_ref()
                .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
            match discover_company_context(
                &self.store,
                policy.as_ref(),
                &credentials,
                OrgId::from_uuid(*org_id),
            )
            .await
            {
                Ok(view) => Some(view),
                Err(CompanyPolicyError::NotFound) => None,
                Err(e) => return Err(policy_error(e).status()),
            }
        } else {
            None
        };
        // Only the status owner's command fence establishes absence. A current
        // eligible operator can explicitly re-enter unsent input at that same
        // command locator; neither browser history nor an optimistic 404 proves it.
        let reentry_account = if status == CompanyEnrollmentStatus::Missing {
            match console_platform_auth_rest::native_company_setup_entry(&self.auth, headers).await
            {
                Ok(console_platform_auth_rest::NativeAccountEntry::CompanySetup {
                    eligibility: console_platform_auth_rest::NativeCompanySetupEligibility::Eligible,
                    account_id,
                }) => Some(account_id),
                Err(error) => return Err(error.status()),
                _ => None,
            }
        } else {
            None
        };
        Ok(EnrollmentDocument {
            command,
            status,
            company,
            reentry_account,
        })
    }

    pub async fn company_document(
        &self,
        headers: &HeaderMap,
        company: &str,
    ) -> Result<CompanyIdentityView, StatusCode> {
        let credentials = self
            .auth
            .company_document_credentials(headers)
            .map_err(|e| e.status())?;
        let company = command_id(company).map_err(|_| StatusCode::NOT_FOUND)?;
        let policy = self
            .policy
            .as_ref()
            .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
        read_company_identity(
            &self.store,
            policy.as_ref(),
            &credentials,
            OrgId::from_uuid(company),
        )
        .await
        .map_err(|e| policy_error(e).status())
    }

    pub async fn context_documents(
        &self,
        headers: &HeaderMap,
    ) -> Result<Vec<CompanyContextView>, StatusCode> {
        let credentials = self
            .auth
            .company_document_credentials(headers)
            .map_err(|e| e.status())?;
        let policy = self
            .policy
            .as_ref()
            .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
        discover_company_contexts(&self.store, policy.as_ref(), &credentials)
            .await
            .map_err(|e| policy_error(e).status())
    }

    pub async fn policy_document(
        &self,
        headers: &HeaderMap,
        company: &str,
    ) -> Result<CompanyPolicyView, StatusCode> {
        let credentials = self
            .auth
            .company_document_credentials(headers)
            .map_err(|e| e.status())?;
        let company = command_id(company).map_err(|_| StatusCode::NOT_FOUND)?;
        let policy = self
            .policy
            .as_ref()
            .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
        read_company_policy(
            &self.store,
            policy.as_ref(),
            &credentials,
            OrgId::from_uuid(company),
        )
        .await
        .map_err(|e| policy_error(e).status())
    }
}

pub struct EnrollmentDocument {
    pub command: Uuid,
    pub status: CompanyEnrollmentStatus,
    pub company: Option<CompanyContextView>,
    pub reentry_account: Option<Uuid>,
}

/// Native credentials are intentionally outside legacy tenant middleware.
pub fn router<S>(state: CompanyRestState<S>) -> Router
where
    S: CompanyEnrollmentStore<Credentials = AccountEnrollmentCredentials>
        + CompanyPolicyStore<Credentials = AccountEnrollmentCredentials>
        + Clone
        + Send
        + Sync
        + 'static,
{
    Router::new()
        .route("/api/v2/companies/enroll", post(enroll))
        .route("/api/v2/companies/enrollments/{command_id}", get(status))
        .route(
            "/api/v2/companies/enrollments/{command_id}/cancel",
            post(cancel),
        )
        .route("/api/v2/companies/{org_id}/policy", get(policy))
        .with_state(state)
}

fn private(mut response: Response) -> Response {
    let headers = response.headers_mut();
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(header::PRAGMA, HeaderValue::from_static("no-cache"));
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.append(header::VARY, HeaderValue::from_static("Cookie, Origin"));
    response
}

fn error(status: StatusCode, code: &'static str) -> Response {
    private((status, Json(json!({"error":{"code":code,"message":code}}))).into_response())
}

fn enrollment_error(cause: CompanyEnrollmentError) -> Response {
    use CompanyEnrollmentError as E;
    let (status, code) = match cause {
        E::InvalidInput => (
            StatusCode::UNPROCESSABLE_ENTITY,
            "company_enrollment_invalid",
        ),
        E::AuthenticationInvalid => (StatusCode::UNAUTHORIZED, "authentication_invalid"),
        E::CsrfInvalid => (StatusCode::FORBIDDEN, "csrf_invalid"),
        E::Forbidden => (StatusCode::FORBIDDEN, "company_enrollment_forbidden"),
        E::Conflict => (StatusCode::CONFLICT, "command_conflict"),
        E::Capacity => (StatusCode::TOO_MANY_REQUESTS, "company_enrollment_capacity"),
        E::GroupUnavailable => (
            StatusCode::UNPROCESSABLE_ENTITY,
            "group_enrollment_unavailable",
        ),
        E::Unavailable => (
            StatusCode::SERVICE_UNAVAILABLE,
            "company_enrollment_unavailable",
        ),
        E::Unconfirmed => (StatusCode::CONFLICT, "outcome_unknown"),
    };
    error(status, code)
}

fn policy_error(cause: CompanyPolicyError) -> Response {
    let (status, code) = match cause {
        CompanyPolicyError::AuthenticationInvalid => {
            (StatusCode::UNAUTHORIZED, "authentication_invalid")
        }
        CompanyPolicyError::NotFound => (StatusCode::NOT_FOUND, "not_found"),
        CompanyPolicyError::Conflict => (StatusCode::CONFLICT, "context_changed"),
        CompanyPolicyError::MaterialUnavailable | CompanyPolicyError::EvaluatorUnavailable => {
            (StatusCode::SERVICE_UNAVAILABLE, "authority_unavailable")
        }
    };
    error(status, code)
}

fn command_id(value: &str) -> Result<Uuid, CompanyEnrollmentError> {
    let id = Uuid::parse_str(value).map_err(|_| CompanyEnrollmentError::InvalidInput)?;
    if id.is_nil() || id.to_string() != value {
        return Err(CompanyEnrollmentError::InvalidInput);
    }
    Ok(id)
}

fn receipt_json(command: Uuid, receipt: CompanyEnrollmentReceipt) -> Value {
    json!({
        "outcome":"COMMITTED", "original_command_id":command,
        "receipt_id":receipt.receipt_id, "org_id":receipt.org_id,
        "group_id":receipt.group_id, "administrative_account_id":receipt.administrative_account_id,
        "replayed":receipt.replayed, "result_path":format!("/account/companies/requests/{command}")
    })
}

fn status_response(command: Uuid, status: CompanyEnrollmentStatus) -> Response {
    let result_path = format!("/account/companies/requests/{command}");
    let body = match status {
        CompanyEnrollmentStatus::Missing => {
            return error(StatusCode::NOT_FOUND, "company_enrollment_not_found");
        }
        CompanyEnrollmentStatus::Committed {
            receipt_id,
            org_id,
            group_id,
            administrative_account_id,
        } => receipt_json(
            command,
            CompanyEnrollmentReceipt {
                receipt_id,
                org_id,
                group_id,
                administrative_account_id,
                replayed: true,
            },
        ),
        CompanyEnrollmentStatus::Pending(input) => json!({
            "outcome":"PENDING", "original_command_id":command, "result_path":result_path,
            "input": {"command_id":input.command_id(), "group_id":input.group_id(),
                "name":input.name(), "slug":input.slug(), "administrative_account_id":input.administrative_account_id()}
        }),
        CompanyEnrollmentStatus::Cancelled => {
            json!({"outcome":"CANCELLED", "original_command_id":command, "result_path":result_path})
        }
        CompanyEnrollmentStatus::Expired => {
            json!({"outcome":"EXPIRED", "original_command_id":command, "result_path":result_path})
        }
    };
    private(Json(body).into_response())
}

async fn enroll<S>(State(state): State<CompanyRestState<S>>, request: Request) -> Response
where
    S: CompanyEnrollmentStore<Credentials = AccountEnrollmentCredentials>
        + CompanyPolicyStore<Credentials = AccountEnrollmentCredentials>
        + Clone
        + Send
        + Sync
        + 'static,
{
    let credentials = match state
        .auth
        .company_api_mutation_credentials(request.headers())
    {
        Ok(value) => value,
        Err(response) => return *response,
    };
    let bytes = match to_bytes(request.into_body(), 4096).await {
        Ok(bytes) => bytes,
        Err(_) => {
            return error(
                StatusCode::PAYLOAD_TOO_LARGE,
                "company_enrollment_too_large",
            );
        }
    };
    let input = match CompanyEnrollmentV1::from_json_slice(&bytes) {
        Ok(input) => input,
        Err(_) => return enrollment_error(CompanyEnrollmentError::InvalidInput),
    };
    match enroll_company(
        &state.store,
        &credentials,
        &bytes,
        &TraceContext::generate(),
    )
    .await
    {
        Ok(receipt) => {
            let status = if receipt.replayed {
                StatusCode::OK
            } else {
                StatusCode::CREATED
            };
            private((status, Json(receipt_json(input.command_id(), receipt))).into_response())
        }
        Err(cause) => enrollment_error(cause),
    }
}

async fn status<S>(
    State(state): State<CompanyRestState<S>>,
    Path(command): Path<String>,
    headers: HeaderMap,
) -> Response
where
    S: CompanyEnrollmentStore<Credentials = AccountEnrollmentCredentials>
        + CompanyPolicyStore<Credentials = AccountEnrollmentCredentials>
        + Clone
        + Send
        + Sync
        + 'static,
{
    let credentials = match state.auth.company_api_read_credentials(&headers) {
        Ok(value) => value,
        Err(response) => return *response,
    };
    let command = match command_id(&command) {
        Ok(value) => value,
        Err(cause) => return enrollment_error(cause),
    };
    match company_enrollment_status(&state.store, &credentials, command).await {
        Ok(status) => status_response(command, status),
        Err(cause) => enrollment_error(cause),
    }
}

async fn cancel<S>(
    State(state): State<CompanyRestState<S>>,
    Path(command): Path<String>,
    request: Request,
) -> Response
where
    S: CompanyEnrollmentStore<Credentials = AccountEnrollmentCredentials>
        + CompanyPolicyStore<Credentials = AccountEnrollmentCredentials>
        + Clone
        + Send
        + Sync
        + 'static,
{
    let credentials = match state
        .auth
        .company_api_mutation_credentials(request.headers())
    {
        Ok(value) => value,
        Err(response) => return *response,
    };
    let command = match command_id(&command) {
        Ok(value) => value,
        Err(cause) => return enrollment_error(cause),
    };
    match to_bytes(request.into_body(), 4096).await {
        Ok(body) if body.is_empty() || body.as_ref() == b"{}" => (),
        Ok(_) => return enrollment_error(CompanyEnrollmentError::InvalidInput),
        Err(_) => {
            return error(
                StatusCode::PAYLOAD_TOO_LARGE,
                "company_enrollment_too_large",
            );
        }
    }
    match cancel_company_enrollment(&state.store, &credentials, command).await {
        Ok(status) => status_response(command, status),
        Err(cause) => enrollment_error(cause),
    }
}

fn action_json(value: &ActionRef) -> Value {
    let digest: String = value
        .manifest_digest()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    json!({"org_id":value.org_id(), "action_type_id":value.action_type_id(),
        "object_type_id":value.object_type_id(), "registration_revision":value.registration_revision(), "manifest_digest":digest})
}

fn property_json(value: &PropertyRef) -> Value {
    json!({"org_id":value.org_id(), "object_type_id":value.object_type_id(),
        "property_id":value.property_id(), "schema_revision":value.schema_revision()})
}

async fn policy<S>(
    State(state): State<CompanyRestState<S>>,
    Path(company): Path<String>,
    headers: HeaderMap,
) -> Response
where
    S: CompanyEnrollmentStore<Credentials = AccountEnrollmentCredentials>
        + CompanyPolicyStore<Credentials = AccountEnrollmentCredentials>
        + Clone
        + Send
        + Sync
        + 'static,
{
    let credentials = match state.auth.company_api_read_credentials(&headers) {
        Ok(value) => value,
        Err(response) => return *response,
    };
    let company = match command_id(&company) {
        Ok(value) => OrgId::from_uuid(value),
        Err(_) => return error(StatusCode::NOT_FOUND, "not_found"),
    };
    let Some(policy) = state.policy.as_ref() else {
        return policy_error(CompanyPolicyError::EvaluatorUnavailable);
    };
    match read_company_policy(&state.store, policy.as_ref(), &credentials, company).await {
        Ok(view) => {
            let c = view.initial_ceiling;
            private(Json(json!({"initial_ceiling": {
                "org_id":c.org_id, "account_id":c.account_id.as_uuid(),
                "action_keys":c.action_keys, "action_refs":c.action_refs.iter().map(action_json).collect::<Vec<_>>(),
                "delegable_action_keys":c.delegable_action_keys, "delegable_action_refs":c.delegable_action_refs.iter().map(action_json).collect::<Vec<_>>(),
                "company_property_keys":c.company_property_keys, "company_property_refs":c.company_property_refs.iter().map(property_json).collect::<Vec<_>>(),
                "delegable_company_property_keys":c.delegable_company_property_keys, "delegable_company_property_refs":c.delegable_company_property_refs.iter().map(property_json).collect::<Vec<_>>(),
                "future_registrations":c.future_registrations, "group_control":c.group_control
            }})).into_response())
        }
        Err(cause) => policy_error(cause),
    }
}
