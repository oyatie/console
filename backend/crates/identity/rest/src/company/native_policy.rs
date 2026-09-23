//! Native policy documents and form transport. Current authorization and every
//! durable transition remain in the retained application workflow.
mod form;
pub use form::NativePolicyGrantDraft;

use super::{CompanyRestState, command_id};
use axum::{
    body::to_bytes,
    extract::Request,
    http::{HeaderMap, StatusCode},
};
use console_identity_application::company_policy::{
    business::NativeBusinessOperationV1,
    workflow::{
        NativePolicyCommandRef, NativePolicyExecution, NativePolicyForm, NativePolicyStatus,
        NativePolicyWorkflowError, NativePolicyWorkflowStore, execute_native_policy_command,
        native_policy_command_status, native_policy_form, native_policy_validation_form,
        submit_native_policy_command,
    },
};
use console_kernel_core::{OrgId, TraceContext};
use console_platform_auth::account::{AccountEnrollmentCredentials, AccountFormProof};
use console_platform_auth_rest::AuthRestState;
use uuid::Uuid;

pub enum NativePolicySubmission {
    Validation {
        current: NativePolicyForm<AccountFormProof>,
        draft: NativePolicyGrantDraft,
    },
    Confirmed {
        selector: NativePolicyCommandRef,
    },
    Unconfirmed {
        selector: NativePolicyCommandRef,
    },
}

// A current form is returned only after its own final authorization succeeds.
// Neither this proof nor any request containing it is Debug/Serialize.
pub enum NativePolicyRequestDocument {
    NotVisible,
    Visible {
        status: NativePolicyStatus,
        current: NativePolicyForm<AccountFormProof>,
    },
}

pub enum NativePolicyPostTarget<'a> {
    Install,
    Grant,
    Revoke {
        assignment: &'a str,
    },
    Retry {
        operation: &'a str,
        command: &'a str,
    },
}

impl<S> CompanyRestState<S>
where
    S: NativePolicyWorkflowStore<
            Credentials = AccountEnrollmentCredentials,
            FormProof = AccountFormProof,
        >,
{
    pub async fn policy_form_document(
        &self,
        headers: &HeaderMap,
        company: &str,
        operation: &str,
    ) -> Result<NativePolicyForm<AccountFormProof>, StatusCode> {
        let selector = selector(company, operation, Uuid::new_v4())?;
        let credentials = self
            .auth
            .company_document_credentials(headers)
            .map_err(|e| e.status())?;
        let policy = self
            .policy
            .as_ref()
            .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
        native_policy_form(&self.store, policy.as_ref(), &credentials, selector)
            .await
            .map_err(workflow_status)
    }

    pub async fn policy_request_document(
        &self,
        headers: &HeaderMap,
        company: &str,
        operation: &str,
        command: &str,
    ) -> Result<NativePolicyRequestDocument, StatusCode> {
        let selector = selector(company, operation, route_id(command)?)?;
        let credentials = self
            .auth
            .company_document_credentials(headers)
            .map_err(|e| e.status())?;
        let policy = self
            .policy
            .as_ref()
            .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
        let status =
            native_policy_command_status(&self.store, policy.as_ref(), &credentials, selector)
                .await
                .map_err(workflow_status)?;
        if matches!(status, NativePolicyStatus::NotVisible) {
            return Ok(NativePolicyRequestDocument::NotVisible);
        }
        let current = native_policy_form(&self.store, policy.as_ref(), &credentials, selector)
            .await
            .map_err(workflow_status)?;
        Ok(NativePolicyRequestDocument::Visible { status, current })
    }

    pub async fn policy_submit_document(
        &self,
        request: Request,
        company: &str,
        target: NativePolicyPostTarget<'_>,
    ) -> Result<NativePolicySubmission, StatusCode> {
        let company = OrgId::from_uuid(route_id(company)?);
        if company == OrgId::platform() {
            return Err(StatusCode::NOT_FOUND);
        }
        let target = match target {
            NativePolicyPostTarget::Install => form::Target::Install,
            NativePolicyPostTarget::Grant => form::Target::Grant,
            NativePolicyPostTarget::Revoke { assignment } => form::Target::Revoke {
                assignment: route_id(assignment)?,
            },
            NativePolicyPostTarget::Retry { operation, command } => form::Target::Retry {
                operation: operation_kind(operation)?,
                command_id: route_id(command)?,
            },
        };
        let (credentials, input) = capture_post(&self.auth, request, company, target).await?;
        let policy = self
            .policy
            .as_ref()
            .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
        let trace = TraceContext::generate();
        let (selector, result) = match input {
            form::DocumentInput::GrantValidation(draft) => {
                let current = native_policy_validation_form(
                    &self.store,
                    policy.as_ref(),
                    &credentials,
                    draft.selector,
                )
                .await
                .map_err(workflow_status)?;
                return Ok(NativePolicySubmission::Validation { current, draft });
            }
            form::DocumentInput::Ready(form::Input::Command(command)) => {
                let selector = NativePolicyCommandRef::from_command(&command);
                let result = submit_native_policy_command(
                    &self.store,
                    policy.as_ref(),
                    &credentials,
                    &command,
                    &trace,
                )
                .await;
                (selector, result)
            }
            form::DocumentInput::Ready(form::Input::Retry(selector)) => {
                let result = execute_native_policy_command(
                    &self.store,
                    policy.as_ref(),
                    &credentials,
                    selector,
                    &trace,
                )
                .await;
                (selector, result)
            }
        };
        finish_submission(selector, result)
    }
}

fn route_id(value: &str) -> Result<Uuid, StatusCode> {
    command_id(value).map_err(|_| StatusCode::NOT_FOUND)
}
fn operation_kind(value: &str) -> Result<NativeBusinessOperationV1, StatusCode> {
    match value {
        "install" => Ok(NativeBusinessOperationV1::Install),
        "grant" => Ok(NativeBusinessOperationV1::Grant),
        "revoke" => Ok(NativeBusinessOperationV1::Revoke),
        _ => Err(StatusCode::NOT_FOUND),
    }
}
fn selector(
    company: &str,
    operation: &str,
    command: Uuid,
) -> Result<NativePolicyCommandRef, StatusCode> {
    NativePolicyCommandRef::new(
        OrgId::from_uuid(route_id(company)?),
        command,
        operation_kind(operation)?,
    )
    .map_err(|_| StatusCode::NOT_FOUND)
}

async fn capture_post(
    auth: &AuthRestState,
    request: Request,
    company: OrgId,
    target: form::Target,
) -> Result<(AccountEnrollmentCredentials, form::DocumentInput), StatusCode> {
    let (parts, body) = request.into_parts();
    let body = to_bytes(body, form::MAX_BODY_BYTES)
        .await
        .map_err(|_| StatusCode::PAYLOAD_TOO_LARGE)?;
    let parsed = form::parse_document(company, target, &body).map_err(|error| match error {
        form::FormError::Invalid => StatusCode::BAD_REQUEST,
        form::FormError::TooLarge => StatusCode::PAYLOAD_TOO_LARGE,
    })?;
    let credentials = auth
        .company_form_mutation_credentials(&parts.method, &parts.headers, &parsed.proof)
        .map_err(|e| e.status())?;
    Ok((credentials, parsed.input))
}
fn finish_submission(
    selector: NativePolicyCommandRef,
    result: Result<NativePolicyExecution, NativePolicyWorkflowError>,
) -> Result<NativePolicySubmission, StatusCode> {
    match result {
        Ok(_) => Ok(NativePolicySubmission::Confirmed { selector }),
        Err(NativePolicyWorkflowError::Unconfirmed) => {
            Ok(NativePolicySubmission::Unconfirmed { selector })
        }
        Err(error) => Err(workflow_status(error)),
    }
}
fn workflow_status(error: NativePolicyWorkflowError) -> StatusCode {
    use NativePolicyWorkflowError as E;
    match error {
        E::InvalidInput => StatusCode::UNPROCESSABLE_ENTITY,
        E::AuthenticationInvalid => StatusCode::UNAUTHORIZED,
        E::CsrfInvalid => StatusCode::FORBIDDEN,
        E::NotFound => StatusCode::NOT_FOUND,
        E::Conflict => StatusCode::CONFLICT,
        E::Capacity => StatusCode::TOO_MANY_REQUESTS,
        E::Unavailable | E::Unconfirmed => StatusCode::SERVICE_UNAVAILABLE,
    }
}

#[cfg(test)]
mod tests;
