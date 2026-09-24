//! Native People HTTP transport; application retains authorization and effects.
mod form;
use axum::{
    body::to_bytes,
    extract::Request,
    http::{HeaderMap, Method, StatusCode},
};
use console_kernel_core::TraceContext;
use console_ontology_application::people::workflow::*;
use console_platform_auth::account::{AccountEnrollmentCredentials, AccountFormProof};
use console_platform_auth_rest::AuthRestState;
use console_platform_request_context::TrustedClientIp;
pub use form::{Draft, PostTarget};
use std::sync::{Arc, OnceLock};
use uuid::Uuid;

/// Recovery routing only. Requested identifiers never establish authority and
/// are never evidence of an accepted or committed command.
#[derive(Clone, Default)]
pub struct RecoveryLocator(Arc<OnceLock<DirectoryRequestRef>>);
impl RecoveryLocator {
    pub fn get(&self) -> Option<DirectoryRequestRef> {
        self.0.get().copied()
    }
    pub fn retain_requested(&self, locator: DirectoryRequestRef) {
        let _ = self.0.set(locator);
    }
}

pub struct NativePeopleRestState<S, P> {
    store: Arc<S>,
    policy: Arc<P>,
    auth: AuthRestState,
}
impl<S, P> Clone for NativePeopleRestState<S, P> {
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
            policy: self.policy.clone(),
            auth: self.auth.clone(),
        }
    }
}
// Auth owns admission. The application requests it only when the retained
// status transaction will issue a Pending form proof; read-only outcomes never
// consume a CSRF bucket. No proof or current-authority result crosses this port.
struct StatusProofAdmission<'a> {
    auth: &'a AuthRestState,
    headers: &'a HeaderMap,
    client: Option<TrustedClientIp>,
}
impl DirectoryProofAdmission for StatusProofAdmission<'_> {
    async fn admit(&mut self) -> Result<(), DirectoryWorkflowError> {
        self.auth
            .limit_company_form(self.headers, self.client)
            .await
            .map_err(|error| {
                if error.status() == StatusCode::TOO_MANY_REQUESTS {
                    DirectoryWorkflowError::Capacity
                } else {
                    DirectoryWorkflowError::Unavailable
                }
            })
    }
}
pub enum Submission {
    Validation {
        current: DirectoryForm<AccountFormProof>,
        draft: Draft,
        problem: Option<DirectoryWorkflowError>,
    },
    Confirmed(DirectoryRequestRef),
    Unconfirmed(DirectoryRequestRef),
}
impl<S, P> NativePeopleRestState<S, P>
where
    S: DirectoryWorkflowStore<
            Credentials = AccountEnrollmentCredentials,
            FormProof = AccountFormProof,
        >,
    P: DirectoryDecisionPort<S::Authority>,
{
    pub fn new(store: S, policy: P, auth: AuthRestState) -> Self {
        Self {
            store: Arc::new(store),
            policy: Arc::new(policy),
            auth,
        }
    }
    fn credentials(&self, headers: &HeaderMap) -> Result<AccountEnrollmentCredentials, StatusCode> {
        self.auth
            .company_document_credentials(headers)
            .map_err(|e| e.status())
    }
    pub async fn navigation(
        &self,
        headers: &HeaderMap,
        company: &str,
        action: DirectoryAction,
    ) -> Result<bool, StatusCode> {
        let company = form::company(company)?;
        let credentials = self.credentials(headers)?;
        match directory_navigation(
            self.store.as_ref(),
            self.policy.as_ref(),
            &credentials,
            company,
            action,
        )
        .await
        {
            Ok(()) => Ok(true),
            Err(DirectoryWorkflowError::NotFound) => Ok(false),
            Err(error) => Err(owner_status(error)),
        }
    }
    pub async fn list(
        &self,
        headers: &HeaderMap,
        company: &str,
        query: Option<&str>,
    ) -> Result<DirectoryPage, StatusCode> {
        let company = form::company(company)?;
        let query = form::pagination(query)?;
        let credentials = self.credentials(headers)?;
        directory_list(
            self.store.as_ref(),
            self.policy.as_ref(),
            &credentials,
            company,
            query,
        )
        .await
        .map_err(owner_status)
    }
    pub async fn detail(
        &self,
        headers: &HeaderMap,
        company: &str,
        employee: &str,
        query: Option<&str>,
    ) -> Result<Option<DirectoryRecord>, StatusCode> {
        form::no_query(query)?;
        let company = form::company(company)?;
        let employee = form::id(employee)?;
        let credentials = self.credentials(headers)?;
        directory_detail(
            self.store.as_ref(),
            self.policy.as_ref(),
            &credentials,
            company,
            employee,
        )
        .await
        .map_err(owner_status)
    }
    pub async fn registration(
        &self,
        headers: &HeaderMap,
        client: Option<TrustedClientIp>,
        company: &str,
        query: Option<&str>,
    ) -> Result<DirectoryForm<AccountFormProof>, StatusCode> {
        form::no_query(query)?;
        let company = form::company(company)?;
        let credentials = self.credentials(headers)?;
        let locator = DirectoryRequestRef::new(company, Uuid::new_v4()).map_err(owner_status)?;
        self.auth
            .limit_company_form(headers, client)
            .await
            .map_err(|e| e.status())?;
        directory_form(
            self.store.as_ref(),
            self.policy.as_ref(),
            &credentials,
            locator,
            None,
        )
        .await
        .map_err(owner_status)
    }
    pub async fn request(
        &self,
        headers: &HeaderMap,
        client: Option<TrustedClientIp>,
        company: &str,
        command: &str,
        query: Option<&str>,
        recovery: Option<&RecoveryLocator>,
    ) -> Result<DirectoryRecovery<AccountFormProof>, StatusCode> {
        form::no_query(query)?;
        let locator = DirectoryRequestRef::new(form::company(company)?, form::id(command)?)
            .map_err(owner_status)?;
        let credentials = self.credentials(headers)?;
        let mut admission = StatusProofAdmission {
            auth: &self.auth,
            headers,
            client,
        };
        if let Some(recovery) = recovery {
            recovery.retain_requested(locator);
        }
        directory_status(
            self.store.as_ref(),
            self.policy.as_ref(),
            &credentials,
            locator,
            &TraceContext::generate(),
            &mut admission,
        )
        .await
        .map_err(owner_status)
    }
    pub async fn submit(
        &self,
        request: Request,
        company: &str,
        target: PostTarget<'_>,
    ) -> Result<Submission, StatusCode> {
        let company = form::company(company)?;
        if request.method() != Method::POST {
            return Err(StatusCode::METHOD_NOT_ALLOWED);
        }
        form::no_query(request.uri().query())?;
        let (parts, body) = request.into_parts();
        let bytes = to_bytes(body, form::MAX_BODY)
            .await
            .map_err(|_| StatusCode::PAYLOAD_TOO_LARGE)?;
        let parsed = form::parse(company, target, &bytes)?;
        let credentials = self
            .auth
            .company_form_mutation_credentials(&parts.method, &parts.headers, &parsed.proof)
            .map_err(|e| e.status())?;
        let locator = match &parsed.input {
            form::Input::Prepare { draft, .. } => draft.locator,
            form::Input::Execute(locator) | form::Input::Cancel(locator) => *locator,
        };
        if let Some(recovery) = parts.extensions.get::<RecoveryLocator>() {
            recovery.retain_requested(locator);
        }
        let trace = TraceContext::generate();
        match parsed.input {
            form::Input::Prepare { draft, submission } => {
                let result = match submission {
                    Some(input) => {
                        directory_prepare(
                            self.store.as_ref(),
                            self.policy.as_ref(),
                            &credentials,
                            &input,
                            &trace,
                        )
                        .await
                    }
                    None => return self.validation(&credentials, draft, None).await,
                };
                match result {
                    Ok(_) => Ok(Submission::Confirmed(draft.locator)),
                    Err(DirectoryWorkflowError::Unconfirmed) => {
                        Ok(Submission::Unconfirmed(draft.locator))
                    }
                    Err(
                        problem @ (DirectoryWorkflowError::Capacity
                        | DirectoryWorkflowError::Conflict),
                    ) => self.validation(&credentials, draft, Some(problem)).await,
                    Err(error) => Err(owner_status(error)),
                }
            }
            form::Input::Execute(locator) => finish(
                locator,
                directory_execute(
                    self.store.as_ref(),
                    self.policy.as_ref(),
                    &credentials,
                    locator,
                    &trace,
                )
                .await,
            ),
            form::Input::Cancel(locator) => finish(
                locator,
                directory_cancel(
                    self.store.as_ref(),
                    self.policy.as_ref(),
                    &credentials,
                    locator,
                    &trace,
                )
                .await,
            ),
        }
    }
    async fn validation(
        &self,
        credentials: &AccountEnrollmentCredentials,
        draft: Draft,
        problem: Option<DirectoryWorkflowError>,
    ) -> Result<Submission, StatusCode> {
        // Same original proof and expectations survive invalid input. No new proof,
        // command identity or authority is minted during validation redisplay.
        let current = directory_form(
            self.store.as_ref(),
            self.policy.as_ref(),
            credentials,
            draft.locator,
            Some(draft.expected),
        )
        .await
        .map_err(owner_status)?;
        Ok(Submission::Validation {
            current,
            draft,
            problem,
        })
    }
}
fn finish(
    locator: DirectoryRequestRef,
    result: Result<DirectoryExecution, DirectoryWorkflowError>,
) -> Result<Submission, StatusCode> {
    match result {
        Ok(_) => Ok(Submission::Confirmed(locator)),
        Err(DirectoryWorkflowError::Unconfirmed) => Ok(Submission::Unconfirmed(locator)),
        Err(error) => Err(owner_status(error)),
    }
}
fn owner_status(error: DirectoryWorkflowError) -> StatusCode {
    match error {
        DirectoryWorkflowError::InvalidInput => StatusCode::UNPROCESSABLE_ENTITY,
        DirectoryWorkflowError::AuthenticationInvalid => StatusCode::UNAUTHORIZED,
        DirectoryWorkflowError::CsrfInvalid => StatusCode::FORBIDDEN,
        DirectoryWorkflowError::NotFound => StatusCode::NOT_FOUND,
        DirectoryWorkflowError::Conflict => StatusCode::CONFLICT,
        DirectoryWorkflowError::Capacity => StatusCode::TOO_MANY_REQUESTS,
        DirectoryWorkflowError::Unavailable | DirectoryWorkflowError::Unconfirmed => {
            StatusCode::SERVICE_UNAVAILABLE
        }
    }
}
