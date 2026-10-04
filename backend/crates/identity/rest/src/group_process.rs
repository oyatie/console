//! Group document transport. The retained application owner supplies every
//! authorized projection; transport identifiers never establish authority.
use std::sync::{Arc, OnceLock};

mod form;
pub use form::{Draft as GroupProcessDraft, DraftInput as GroupProcessDraftInput, RawContent};

use axum::{
    body::to_bytes,
    extract::Request,
    http::{HeaderMap, StatusCode},
};
use console_identity_application::group_process::{
    GroupId, GroupProcessActionV1, GroupProcessError, GroupProcessForm, GroupProcessLandingViewV1,
    GroupProcessNavigationViewV1, GroupProcessOwnRetryFormV1, GroupProcessRouteSelectorV1,
    GroupProcessStatus, GroupProcessStore, discover_group_process_navigation,
    group_process_retry_form, group_process_status, read_group_process_landing,
};
use console_identity_application::group_process::{
    GroupProcessFormTargetV1, group_process_form_for, group_process_validation_form,
    retry_group_process, submit_group_process,
};
use console_kernel_core::TraceContext;
use console_platform_auth::account::{AccountEnrollmentCredentials, AccountFormProof};
use console_platform_auth_rest::AuthRestState;
use console_platform_authz::group_process::GroupProcessPolicy;
use console_platform_request_context::TrustedClientIp;
use uuid::Uuid;

/// Requested route only; neither authority nor proof of an owner outcome.
#[derive(Clone, Default)]
pub struct RecoveryLocator(Arc<OnceLock<GroupProcessRouteSelectorV1>>);
impl RecoveryLocator {
    pub fn get(&self) -> Option<GroupProcessRouteSelectorV1> {
        self.0.get().copied()
    }
    pub fn retain_requested(&self, requested: GroupProcessRouteSelectorV1) {
        let _ = self.0.set(requested);
    }
}

#[derive(Clone)]
pub struct GroupRestState<S> {
    store: S,
    auth: AuthRestState,
    policy: Arc<GroupProcessPolicy>,
}

/// Closed route selection; permission is checked by the owner on every read.
#[derive(Clone, Copy)]
pub enum GroupFormTarget<'a> {
    Adopt,
    Replace { process: &'a str },
    Suspend { process: &'a str },
}

pub enum GroupPostTarget<'a> {
    Adopt,
    Suspend { process: &'a str },
    Retry { command: &'a str },
}
pub enum GroupSubmission {
    Validation {
        current: Box<GroupProcessForm<AccountFormProof>>,
        draft: GroupProcessDraft,
    },
    Confirmed {
        requested: GroupProcessRouteSelectorV1,
    },
    Unconfirmed {
        requested: GroupProcessRouteSelectorV1,
    },
}

impl<S> GroupRestState<S>
where
    S: GroupProcessStore<Credentials = AccountEnrollmentCredentials, FormProof = AccountFormProof>,
{
    pub fn new(store: S, auth: AuthRestState) -> Result<Self, GroupProcessError> {
        Ok(Self {
            store,
            auth,
            policy: Arc::new(GroupProcessPolicy::new()?),
        })
    }

    #[must_use]
    pub fn with_auth(mut self, auth: AuthRestState) -> Self {
        self.auth = auth;
        self
    }

    /// Keep the domain error until Account composition can discard the whole
    /// document, including projections obtained earlier from Company owners.
    pub async fn navigation_document(
        &self,
        headers: &HeaderMap,
    ) -> Result<Vec<GroupProcessNavigationViewV1>, GroupProcessError> {
        let credentials = self
            .auth
            .company_document_credentials(headers)
            .map_err(|error| {
                if error.status() == StatusCode::UNAUTHORIZED {
                    GroupProcessError::AuthenticationInvalid
                } else {
                    GroupProcessError::Unavailable
                }
            })?;
        discover_group_process_navigation(&self.store, self.policy.as_ref(), &credentials).await
    }

    pub async fn landing_document(
        &self,
        headers: &HeaderMap,
        group: &str,
    ) -> Result<GroupProcessLandingViewV1, StatusCode> {
        let group = route_group(group)?;
        let credentials = self
            .auth
            .company_document_credentials(headers)
            .map_err(|e| e.status())?;
        read_group_process_landing(&self.store, self.policy.as_ref(), &credentials, group)
            .await
            .map_err(workflow_status)
    }

    pub async fn form_document(
        &self,
        headers: &HeaderMap,
        client: Option<TrustedClientIp>,
        group: &str,
        target: GroupFormTarget<'_>,
    ) -> Result<GroupProcessForm<AccountFormProof>, StatusCode> {
        let group = route_group(group)?;
        let target = match target {
            GroupFormTarget::Adopt => GroupProcessFormTargetV1::Adopt,
            GroupFormTarget::Replace { process } => GroupProcessFormTargetV1::Replace {
                process_id: route_id(process)?,
            },
            GroupFormTarget::Suspend { process } => GroupProcessFormTargetV1::Suspend {
                process_id: route_id(process)?,
            },
        };
        let credentials = self
            .auth
            .company_document_credentials(headers)
            .map_err(|e| e.status())?;
        // Wrong or unauthorized routes issue no proof and consume no limiter.
        let landing =
            read_group_process_landing(&self.store, self.policy.as_ref(), &credentials, group)
                .await
                .map_err(workflow_status)?;
        let (head, actions) = match &landing {
            GroupProcessLandingViewV1::Empty(view) => (None, view.allowed_actions.as_slice()),
            GroupProcessLandingViewV1::Current(view) => {
                (view.head.as_ref(), view.allowed_actions.as_slice())
            }
        };
        target
            .require(head.map(|h| h.reference.process_id()), actions)
            .map_err(workflow_status)?;
        self.auth
            .limit_company_form(headers, client)
            .await
            .map_err(|e| e.status())?;
        group_process_form_for(
            &self.store,
            self.policy.as_ref(),
            &credentials,
            group,
            target,
        )
        .await
        .map_err(workflow_status)
    }

    pub async fn request_document(
        &self,
        headers: &HeaderMap,
        group: &str,
        command: &str,
        recovery: Option<&RecoveryLocator>,
    ) -> Result<GroupProcessStatus, StatusCode> {
        let requested = route_selector(group, command)?;
        let credentials = self
            .auth
            .company_document_credentials(headers)
            .map_err(|e| e.status())?;
        if let Some(recovery) = recovery {
            recovery.retain_requested(requested);
        }
        group_process_status(&self.store, self.policy.as_ref(), &credentials, requested)
            .await
            .map_err(workflow_status)
    }

    pub async fn retry_document(
        &self,
        headers: &HeaderMap,
        client: Option<TrustedClientIp>,
        group: &str,
        command: &str,
        recovery: Option<&RecoveryLocator>,
    ) -> Result<GroupProcessOwnRetryFormV1<AccountFormProof>, StatusCode> {
        let requested = route_selector(group, command)?;
        let credentials = self
            .auth
            .company_document_credentials(headers)
            .map_err(|e| e.status())?;
        if let Some(recovery) = recovery {
            recovery.retain_requested(requested);
        }
        let status =
            group_process_status(&self.store, self.policy.as_ref(), &credentials, requested)
                .await
                .map_err(workflow_status)?;
        if matches!(status, GroupProcessStatus::NotVisible) {
            return Err(StatusCode::NOT_FOUND);
        }
        self.auth
            .limit_company_form(headers, client)
            .await
            .map_err(|e| e.status())?;
        group_process_retry_form(&self.store, self.policy.as_ref(), &credentials, requested)
            .await
            .map_err(workflow_status)
    }

    pub async fn submit_document(
        &self,
        request: Request,
        group: &str,
        target: GroupPostTarget<'_>,
    ) -> Result<GroupSubmission, StatusCode> {
        let group = route_group(group)?;
        let target = match target {
            GroupPostTarget::Adopt => form::Target::Adopt,
            GroupPostTarget::Suspend { process } => form::Target::Suspend {
                process: route_id(process)?,
            },
            GroupPostTarget::Retry { command } => form::Target::Retry {
                requested: route_selector(&group.as_uuid().to_string(), command)?,
            },
        };
        let (parts, body) = request.into_parts();
        let body = to_bytes(body, target.max_body())
            .await
            .map_err(|_| StatusCode::PAYLOAD_TOO_LARGE)?;
        let parsed = form::parse(group, target, &body).map_err(|error| match error {
            form::FormError::Invalid => StatusCode::BAD_REQUEST,
            form::FormError::TooLarge => StatusCode::PAYLOAD_TOO_LARGE,
        })?;
        let credentials = self
            .auth
            .company_form_mutation_credentials(&parts.method, &parts.headers, &parsed.proof)
            .map_err(|error| error.status())?;
        let requested = match &parsed.input {
            form::Input::Command(command) => {
                GroupProcessRouteSelectorV1::new(group, command.command_id())
                    .map_err(workflow_status)?
            }
            form::Input::Validation(draft) => GroupProcessRouteSelectorV1::new(
                draft.original.group(),
                draft.original.command_id(),
            )
            .map_err(workflow_status)?,
            form::Input::Retry(requested) => *requested,
        };
        if let Some(recovery) = parts.extensions.get::<RecoveryLocator>() {
            recovery.retain_requested(requested);
        }
        let trace = TraceContext::generate();
        let result = match parsed.input {
            form::Input::Validation(draft) => {
                let current = group_process_validation_form(
                    &self.store,
                    self.policy.as_ref(),
                    &credentials,
                    draft.original,
                    draft.process_id,
                )
                .await
                .map_err(workflow_status)?;
                return Ok(GroupSubmission::Validation {
                    current: Box::new(current),
                    draft,
                });
            }
            form::Input::Command(command) => {
                submit_group_process(
                    &self.store,
                    self.policy.as_ref(),
                    &credentials,
                    &command,
                    &trace,
                )
                .await
            }
            form::Input::Retry(requested) => {
                retry_group_process(
                    &self.store,
                    self.policy.as_ref(),
                    &credentials,
                    requested,
                    &trace,
                )
                .await
            }
        };
        match result {
            Ok(_) => Ok(GroupSubmission::Confirmed { requested }),
            // This is an external observation, never a persisted guessed
            // outcome. Reopen the original locator to reconcile actual state.
            Err(GroupProcessError::Unconfirmed | GroupProcessError::Unavailable) => {
                Ok(GroupSubmission::Unconfirmed { requested })
            }
            Err(error) => Err(workflow_status(error)),
        }
    }
}

fn route_id(value: &str) -> Result<Uuid, StatusCode> {
    let id = Uuid::parse_str(value).map_err(|_| StatusCode::NOT_FOUND)?;
    if id.is_nil() || id.to_string() != value {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(id)
}
fn route_group(value: &str) -> Result<GroupId, StatusCode> {
    GroupId::from_uuid(route_id(value)?).map_err(|_| StatusCode::NOT_FOUND)
}
fn route_selector(group: &str, command: &str) -> Result<GroupProcessRouteSelectorV1, StatusCode> {
    GroupProcessRouteSelectorV1::new(route_group(group)?, route_id(command)?)
        .map_err(|_| StatusCode::NOT_FOUND)
}
fn workflow_status(error: GroupProcessError) -> StatusCode {
    match error {
        GroupProcessError::InvalidInput => StatusCode::UNPROCESSABLE_ENTITY,
        GroupProcessError::AuthenticationInvalid => StatusCode::UNAUTHORIZED,
        GroupProcessError::CsrfInvalid => StatusCode::FORBIDDEN,
        GroupProcessError::NotFound => StatusCode::NOT_FOUND,
        GroupProcessError::Conflict => StatusCode::CONFLICT,
        GroupProcessError::Unavailable | GroupProcessError::Unconfirmed => {
            StatusCode::SERVICE_UNAVAILABLE
        }
    }
}
