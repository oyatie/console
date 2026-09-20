//! Read-only native Account document composition. API admission remains separate.
use super::*;
use sqlx::Acquire;

pub struct NativeTermsItem {
    pub kind: String,
    pub title: String,
    pub content_url: String,
    pub content: String,
}

pub struct NativeTerms {
    pub version: String,
    pub items: Vec<NativeTermsItem>,
}

pub enum NativeAccountContext {
    Empty,
    Unavailable,
}

pub enum NativeAccountEntry {
    SignIn,
    Registration(NativeTerms),
    Active {
        context: NativeAccountContext,
        can_logout: bool,
    },
}

#[derive(Clone, Copy)]
pub enum NativeEntryError {
    InvalidRequest,
    Forbidden,
    Unavailable,
    TooLarge,
}

impl NativeEntryError {
    pub const fn status(self) -> StatusCode {
        match self {
            Self::InvalidRequest => StatusCode::BAD_REQUEST,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
            Self::TooLarge => StatusCode::PAYLOAD_TOO_LARGE,
        }
    }
}

impl From<BrowserError> for NativeEntryError {
    fn from(error: BrowserError) -> Self {
        match error {
            BrowserError::AmbiguousCredentials
            | BrowserError::InvalidRequest
            | BrowserError::AuthenticationInvalid
            | BrowserError::EnrollmentInvalid => Self::InvalidRequest,
            BrowserError::RequestOriginDenied => Self::Forbidden,
            BrowserError::RequestTooLarge => Self::TooLarge,
            _ => Self::Unavailable,
        }
    }
}

/// Routing hint only; never establishes authentication or exposes credentials.
/// Malformed native values remain present and parser errors must reach the owner.
pub fn native_account_credentials_present(headers: &HeaderMap) -> Result<bool, NativeEntryError> {
    let cookies = parse_cookie_fields(headers)?;
    Ok([
        cookies.session,
        cookies.refresh,
        cookies.enrollment,
        cookies.login,
    ]
    .into_iter()
    .any(|cookie| !matches!(cookie, CookieValue::Absent)))
}

// Only a mounted GET document handler calls this. Never strip headers or invoke
// the stricter API owner with a fabricated same-origin request.
fn admit_document<'a>(
    headers: &'a HeaderMap,
    origin: &Url,
) -> Result<BrowserCookies<'a>, NativeEntryError> {
    if origin.scheme() != "https" || origin.host_str().is_none() {
        return Err(NativeEntryError::Unavailable);
    }
    let single = |name| {
        one_header(headers, name, BrowserError::AmbiguousCredentials)
            .map_err(NativeEntryError::from)
    };
    if single("origin")?.is_some_and(|value| value != origin.origin().ascii_serialization()) {
        return Err(NativeEntryError::Forbidden);
    }
    if single("sec-fetch-user")?.is_some_and(|value| value != "?1") {
        return Err(NativeEntryError::InvalidRequest);
    }
    match (
        single("sec-fetch-site")?,
        single("sec-fetch-mode")?,
        single("sec-fetch-dest")?,
    ) {
        (None, None, None) => {} // Explicit compatibility path; live authority still required.
        (Some(site), Some(mode), Some(destination)) => {
            if !matches!(site, "none" | "same-origin" | "same-site" | "cross-site")
                || !matches!(
                    mode,
                    "navigate" | "same-origin" | "no-cors" | "cors" | "websocket"
                )
                || !matches!(
                    destination,
                    "audio"
                        | "audioworklet"
                        | "document"
                        | "embed"
                        | "empty"
                        | "font"
                        | "frame"
                        | "iframe"
                        | "image"
                        | "manifest"
                        | "object"
                        | "paintworklet"
                        | "report"
                        | "script"
                        | "serviceworker"
                        | "sharedworker"
                        | "style"
                        | "track"
                        | "video"
                        | "worker"
                        | "xslt"
                )
            {
                return Err(NativeEntryError::InvalidRequest);
            }
            if mode != "navigate" || destination != "document" {
                return Err(NativeEntryError::Forbidden);
            }
        }
        _ => return Err(NativeEntryError::InvalidRequest),
    }
    let cookies = parse_cookies(headers)?;
    if [
        &cookies.session,
        &cookies.refresh,
        &cookies.enrollment,
        &cookies.login,
    ]
    .into_iter()
    .any(|cookie| matches!(cookie, CookieValue::Malformed))
    {
        return Err(NativeEntryError::InvalidRequest);
    }
    Ok(cookies)
}

/// Safe document projection only: no session/refresh/proof values, database
/// capability or legacy Company identity can cross this interface.
pub async fn native_account_entry(
    state: &AuthRestState,
    headers: &HeaderMap,
    registration: bool,
) -> Result<NativeAccountEntry, NativeEntryError> {
    let services = configured(state)?;
    let cookies = admit_document(headers, &services.rp_origin)?;
    let access = supplied(&cookies.session, BrowserError::AuthenticationInvalid)?;
    let mut tx = transaction(state).await?;
    let result: Result<Option<NativeAccountEntry>, BrowserError> = async {
        let Some(access) = access else {
            return Ok(None);
        };
        let session = live_account_session_in_tx(
            &mut tx,
            &services.jwt_verifier,
            access,
            services.refresh_family_absolute_ttl,
        )
        .await?;
        let projection = projection(&session);
        projection.validate()?;
        // Context proof may raise a SQL exception. Isolate that optional read
        // without releasing the parent transaction's Account/family guards.
        let mut context_tx = tx
            .begin()
            .await
            .map_err(|_| BrowserError::AuthorityUnavailable)?;
        let context = match account_contexts_empty_in_tx(&mut context_tx, &session).await {
            Ok(()) => {
                context_tx
                    .commit()
                    .await
                    .map_err(|_| BrowserError::AuthorityUnavailable)?;
                NativeAccountContext::Empty
            }
            Err(AccountOperationError::NavigationUnavailable) => {
                context_tx
                    .rollback()
                    .await
                    .map_err(|_| BrowserError::AuthorityUnavailable)?;
                NativeAccountContext::Unavailable
            }
            Err(error) => return Err(error.into()),
        };
        ensure_account_session_fresh_in_tx(&mut tx, &session).await?;
        Ok(Some(NativeAccountEntry::Active {
            context,
            can_logout: projection.permitted_self_actions.iter().any(|action| {
                action.action_key == "account.session.logout" && action.registration_revision == "1"
            }),
        }))
    }
    .await;
    let active = match result {
        Ok(value) => {
            tx.commit()
                .await
                .map_err(|_| NativeEntryError::Unavailable)?;
            value
        }
        Err(error) => {
            tx.rollback()
                .await
                .map_err(|_| NativeEntryError::Unavailable)?;
            // Expired/revoked access reveals no identity and never consumes refresh.
            if matches!(error, BrowserError::AuthenticationInvalid) {
                return Ok(NativeAccountEntry::SignIn);
            }
            return Err(error.into());
        }
    };
    if let Some(active) = active {
        return Ok(active);
    }
    if !registration || matches!(cookies.refresh, CookieValue::Supplied(_)) {
        return Ok(NativeAccountEntry::SignIn);
    }
    terms::entry_terms(state)
        .await
        .map(NativeAccountEntry::Registration)
        .map_err(|_| NativeEntryError::Unavailable)
}
