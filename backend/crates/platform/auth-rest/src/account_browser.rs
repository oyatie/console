//! BW31 native Account transport, composed through actual Provisioning and Auth
//! owners. Parsing never establishes authority, and responses remain private
//! until the owning transaction commits.
//! The HTTP composition must separately suppress auth headers/bodies from capture.

#[path = "account_entry.rs"]
mod entry;
pub use entry::{
    NativeAccountContext, NativeAccountEntry, NativeEntryError, NativeTerms, NativeTermsItem,
    native_account_entry,
};

use std::collections::BTreeSet;

use axum::Json;
use axum::body::{Body, to_bytes};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use console_platform_auth::{
    AccountAssurance, AuthenticationCeremony, PasskeyAuthenticationCredential,
    PasskeyRegistrationCredential, RegistrationCeremony,
};
use serde::{Deserialize, Deserializer, Serialize, de, ser::SerializeStruct};
use time::{OffsetDateTime, UtcOffset, format_description::well_known::Rfc3339};
use url::Url;
use uuid::Uuid;

pub(super) const START_BODY_LIMIT: usize = 4 * 1024;
pub(super) const FINISH_BODY_LIMIT: usize = 64 * 1024;
const COOKIE_HEADER_LIMIT: usize = 16 * 1024;
const PROOF_LIMIT: usize = 4 * 1024;
const ACCOUNT_RESPONSE_LIMIT: usize = 16 * 1024;
const JSON_DEPTH_LIMIT: usize = 32;
const CSRF_HEADER: &str = "x-console-csrf";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RegistrationStartInput {
    #[serde(deserialize_with = "digest")]
    pub terms_version: String,
}

pub(super) struct TermsAcknowledgment {
    pub terms_kind: String,
}

impl<'de> Deserialize<'de> for TermsAcknowledgment {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // Derived structs also accept positional arrays. This app-owned nested
        // wrapper requires an object while retaining duplicate-field rejection.
        struct Object;
        impl<'de> de::Visitor<'de> for Object {
            type Value = TermsAcknowledgment;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("an item acknowledgment object")
            }

            fn visit_map<M: de::MapAccess<'de>>(self, map: M) -> Result<Self::Value, M::Error> {
                #[derive(Deserialize)]
                #[serde(deny_unknown_fields)]
                struct Fields {
                    #[serde(deserialize_with = "key")]
                    terms_kind: String,
                    #[serde(rename = "accepted", deserialize_with = "accepted")]
                    _accepted: (),
                }
                let fields = Fields::deserialize(de::value::MapAccessDeserializer::new(map))?;
                Ok(TermsAcknowledgment {
                    terms_kind: fields.terms_kind,
                })
            }
        }
        deserializer.deserialize_map(Object)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RegistrationFinishInput {
    #[serde(deserialize_with = "canonical_uuid")]
    pub ceremony_id: Uuid,
    // The pinned library owns WebAuthn's nested fields and verification.
    pub credential: PasskeyRegistrationCredential,
    #[serde(deserialize_with = "digest")]
    pub accept_terms_version: String,
    #[serde(deserialize_with = "acknowledgments")]
    pub accept_items: Vec<TermsAcknowledgment>,
}

impl RegistrationFinishInput {
    /// Compare against the real immutable manifest selected by Auth authority.
    /// This does not resolve current terms, validate consent, or activate Account.
    pub(super) fn require_items(&self, required: &[&str]) -> Result<(), BrowserError> {
        let required_set: BTreeSet<_> = required.iter().copied().collect();
        if !(1..=8).contains(&required.len())
            || required_set.len() != required.len()
            || required.iter().any(|item| !valid_key(item))
        {
            return Err(BrowserError::AuthorityUnavailable);
        }
        let supplied: BTreeSet<_> = self
            .accept_items
            .iter()
            .map(|item| item.terms_kind.as_str())
            .collect();
        if supplied != required_set {
            return Err(BrowserError::InvalidRequest);
        }
        Ok(())
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LoginFinishInput {
    #[serde(deserialize_with = "canonical_uuid")]
    pub ceremony_id: Uuid,
    pub assertion: PasskeyAuthenticationCredential,
}

/// Forward the complete pinned request-options wrapper, including mediation.
pub(super) struct LoginStart<'a>(pub &'a AuthenticationCeremony);

impl Serialize for LoginStart<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if self.0.ceremony_id.is_nil() {
            return Err(serde::ser::Error::custom("invalid ceremony"));
        }
        let mut wire = serializer.serialize_struct("LoginStart", 2)?;
        wire.serialize_field("ceremony_id", &self.0.ceremony_id)?;
        wire.serialize_field("public_key_options", &self.0.challenge)?;
        wire.end()
    }
}

/// The exact empty object accepted by native login start and logout.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct EmptyInput {}

/// Forward the complete pinned creation wrapper. Resident-key adjustment and
/// the original verifier/state remain Auth owner's responsibility.
pub(super) struct RegistrationStart<'a>(pub &'a RegistrationCeremony);

impl Serialize for RegistrationStart<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if self.0.ceremony_id.is_nil() {
            return Err(serde::ser::Error::custom("invalid ceremony"));
        }
        let mut wire = serializer.serialize_struct("RegistrationStart", 2)?;
        wire.serialize_field("ceremony_id", &self.0.ceremony_id)?;
        wire.serialize_field("public_key_options", &self.0.challenge)?;
        wire.end()
    }
}

fn canonical_uuid<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Uuid, D::Error> {
    let text = String::deserialize(deserializer)?;
    let id = Uuid::parse_str(&text).map_err(|_| de::Error::custom("invalid UUID"))?;
    if id.is_nil() || id.hyphenated().to_string() != text {
        return Err(de::Error::custom("invalid UUID"));
    }
    Ok(id)
}

fn digest<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    let text = String::deserialize(deserializer)?;
    if text.len() != 64
        || !text
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(de::Error::custom("invalid digest"));
    }
    Ok(text)
}

fn valid_key(text: &str) -> bool {
    (1..=64).contains(&text.len())
        && text.as_bytes()[0].is_ascii_lowercase()
        && text.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'.' | b'-')
        })
}

fn key<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    let text = String::deserialize(deserializer)?;
    if !valid_key(&text) {
        return Err(de::Error::custom("invalid terms kind"));
    }
    Ok(text)
}

fn accepted<'de, D: Deserializer<'de>>(deserializer: D) -> Result<(), D::Error> {
    if !bool::deserialize(deserializer)? {
        return Err(de::Error::custom("item acknowledgment required"));
    }
    Ok(())
}

fn acknowledgments<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<TermsAcknowledgment>, D::Error> {
    let items = Vec::<TermsAcknowledgment>::deserialize(deserializer)?;
    let mut kinds = BTreeSet::new();
    if !(1..=8).contains(&items.len()) || items.iter().any(|item| !kinds.insert(&item.terms_kind)) {
        return Err(de::Error::custom("invalid item acknowledgments"));
    }
    Ok(items)
}

/// Call after header admission, before any owner mutation. No raw parse error
/// leaves this boundary. Limits apply even without a Content-Length header.
pub(super) async fn read_json<T: de::DeserializeOwned>(
    body: Body,
    limit: usize,
) -> Result<T, BrowserError> {
    if !matches!(limit, START_BODY_LIMIT | FINISH_BODY_LIMIT) {
        return Err(BrowserError::AuthorityUnavailable);
    }
    let bytes = to_bytes(body, limit)
        .await
        .map_err(|_| BrowserError::RequestTooLarge)?;
    if bytes
        .iter()
        .find(|byte| !matches!(byte, b' ' | b'\t' | b'\r' | b'\n'))
        != Some(&b'{')
    {
        return Err(BrowserError::InvalidRequest);
    }
    check_json_depth(&bytes)?;
    serde_json::from_slice(&bytes).map_err(|_| BrowserError::InvalidRequest)
}

// Bounded lexical precheck only; serde remains the JSON parser. Braces inside
// strings, including escaped quotes/backslashes, cannot inflate nesting depth.
fn check_json_depth(bytes: &[u8]) -> Result<(), BrowserError> {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for &byte in bytes {
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
        } else {
            match byte {
                b'"' => in_string = true,
                b'{' | b'[' => {
                    depth += 1;
                    if depth > JSON_DEPTH_LIMIT {
                        return Err(BrowserError::InvalidRequest);
                    }
                }
                b'}' | b']' => {
                    depth = depth.checked_sub(1).ok_or(BrowserError::InvalidRequest)?;
                }
                _ => {}
            }
        }
    }
    if depth != 0 || in_string {
        return Err(BrowserError::InvalidRequest);
    }
    Ok(())
}

pub(super) fn admit_post<'a>(
    headers: &'a HeaderMap,
    rp_origin: &Url,
) -> Result<BrowserCookies<'a>, BrowserError> {
    admit_origin(headers, rp_origin, true)?;
    if one_header(
        headers,
        header::CONTENT_TYPE.as_str(),
        BrowserError::InvalidRequest,
    )? != Some("application/json")
    {
        return Err(BrowserError::InvalidRequest);
    }
    parse_cookies(headers)
}

pub(super) fn admit_read<'a>(
    headers: &'a HeaderMap,
    rp_origin: &Url,
) -> Result<BrowserCookies<'a>, BrowserError> {
    admit_origin(headers, rp_origin, false)?;
    parse_cookies(headers)
}

pub(super) fn admit_proof_fetch<'a>(
    headers: &'a HeaderMap,
    rp_origin: &Url,
) -> Result<BrowserCookies<'a>, BrowserError> {
    admit_origin(headers, rp_origin, false)?;
    if csrf_header(headers)? != "fetch" {
        return Err(BrowserError::CsrfInvalid);
    }
    parse_cookies(headers)
}

fn admit_origin(headers: &HeaderMap, rp_origin: &Url, required: bool) -> Result<(), BrowserError> {
    if rp_origin.scheme() != "https" || rp_origin.host_str().is_none() {
        return Err(BrowserError::AuthorityUnavailable);
    }
    let origin = one_header(
        headers,
        header::ORIGIN.as_str(),
        BrowserError::RequestOriginDenied,
    )?;
    if origin.is_none() && required
        || origin.is_some_and(|origin| origin != rp_origin.origin().ascii_serialization())
    {
        return Err(BrowserError::RequestOriginDenied);
    }
    if one_header(headers, "sec-fetch-site", BrowserError::RequestOriginDenied)?
        .is_some_and(|site| site != "same-origin")
    {
        return Err(BrowserError::RequestOriginDenied);
    }
    Ok(())
}

fn one_header<'a>(
    headers: &'a HeaderMap,
    name: &str,
    refusal: BrowserError,
) -> Result<Option<&'a str>, BrowserError> {
    let mut values = headers.get_all(name).iter();
    let first = values.next();
    if values.next().is_some() {
        return Err(refusal);
    }
    first
        .map(|value| value.to_str().map_err(|_| refusal))
        .transpose()
}

/// Obtaining the bounded header is not CSRF verification. The mutation owner
/// verifies its signature/time and exact live Account/family/generation binding.
pub(super) fn csrf_header(headers: &HeaderMap) -> Result<&str, BrowserError> {
    let mut size = 0usize;
    for value in headers.get_all(CSRF_HEADER) {
        size = size.saturating_add(value.as_bytes().len());
        if size > PROOF_LIMIT {
            return Err(BrowserError::RequestTooLarge);
        }
    }
    one_header(headers, CSRF_HEADER, BrowserError::CsrfInvalid)?
        .filter(|value| !value.is_empty())
        .ok_or(BrowserError::CsrfInvalid)
}

#[derive(Clone, Copy)]
pub(super) enum CookieName {
    Session,
    Refresh,
    Enrollment,
    Login,
}

impl CookieName {
    fn attributes(self) -> (&'static str, &'static str, Option<i64>) {
        match self {
            Self::Session => ("__Host-console_account_session", "Lax", Some(900)),
            Self::Refresh => ("__Host-console_account_refresh", "Strict", None),
            Self::Enrollment => ("__Host-console_account_enrollment", "Strict", Some(300)),
            Self::Login => ("__Host-console_account_login", "Strict", Some(300)),
        }
    }
}

/// Syntactic state only. A malformed or cryptographically invalid access value
/// must never become Absent or permit refresh/parser fallback. No Debug output.
pub(super) enum CookieValue<'a> {
    Absent,
    Malformed,
    Supplied(&'a str),
}

pub(super) struct BrowserCookies<'a> {
    pub session: CookieValue<'a>,
    pub refresh: CookieValue<'a>,
    pub enrollment: CookieValue<'a>,
    pub login: CookieValue<'a>,
}

fn parse_cookies(headers: &HeaderMap) -> Result<BrowserCookies<'_>, BrowserError> {
    if headers.contains_key(header::AUTHORIZATION) {
        return Err(BrowserError::AmbiguousCredentials);
    }
    let mut total = 0usize;
    for value in headers.get_all(header::COOKIE) {
        // Include the separator needed to combine multiple Cookie field lines.
        total = total
            .saturating_add(value.as_bytes().len())
            .saturating_add(2);
        if total > COOKIE_HEADER_LIMIT + 2 {
            return Err(BrowserError::RequestTooLarge);
        }
    }
    let mut cookies = BrowserCookies {
        session: CookieValue::Absent,
        refresh: CookieValue::Absent,
        enrollment: CookieValue::Absent,
        login: CookieValue::Absent,
    };
    for header in headers.get_all(header::COOKIE) {
        let text = header.to_str().map_err(|_| BrowserError::InvalidRequest)?;
        for pair in text.split(';') {
            let pair = pair.trim_start_matches([' ', '\t']);
            let (name, value) = pair
                .split_once('=')
                .map_or((pair, None), |(name, value)| (name, Some(value)));
            let slot = match name.trim_end_matches([' ', '\t']) {
                "__Host-console_account_session" => &mut cookies.session,
                "__Host-console_account_refresh" => &mut cookies.refresh,
                "__Host-console_account_enrollment" => &mut cookies.enrollment,
                "__Host-console_account_login" => &mut cookies.login,
                _ => continue, // Legacy console_refresh is never an Account credential.
            };
            if !matches!(slot, CookieValue::Absent) {
                return Err(BrowserError::AmbiguousCredentials);
            }
            *slot = match value {
                Some(value)
                    if name == name.trim_end_matches([' ', '\t']) && valid_cookie_value(value) =>
                {
                    CookieValue::Supplied(value)
                }
                _ => CookieValue::Malformed,
            };
        }
    }
    Ok(cookies)
}

fn valid_cookie_value(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= COOKIE_HEADER_LIMIT
        && value.bytes().all(
            |byte| matches!(byte, 0x21 | 0x23..=0x2b | 0x2d..=0x3a | 0x3c..=0x5b | 0x5d..=0x7e),
        )
}

/// Auth supplies the actual signed/stored expiry. For refresh this must already
/// be min(token expiry, family absolute expiry); no lifetime is inferred here.
pub(super) fn issue_cookie(
    name: CookieName,
    value: &str,
    now: OffsetDateTime,
    expires_at: OffsetDateTime,
) -> Result<HeaderValue, BrowserError> {
    if !valid_cookie_value(value) {
        return Err(BrowserError::AuthorityUnavailable);
    }
    let (name, same_site, cap) = name.attributes();
    let remaining = (expires_at - now).whole_seconds();
    let age = cap.map_or(remaining, |cap| remaining.min(cap));
    if age <= 0 {
        return Err(BrowserError::AuthorityUnavailable);
    }
    render_cookie(name, same_site, value, age)
}

/// Use only after a validated successful flow; refusal rendering never calls it.
pub(super) fn clear_cookie(name: CookieName) -> Result<HeaderValue, BrowserError> {
    let (name, same_site, _) = name.attributes();
    render_cookie(name, same_site, "", 0)
}

fn render_cookie(
    name: &str,
    same_site: &str,
    value: &str,
    age: i64,
) -> Result<HeaderValue, BrowserError> {
    let text =
        format!("{name}={value}; Secure; HttpOnly; Path=/; SameSite={same_site}; Max-Age={age}");
    if text.len() > COOKIE_HEADER_LIMIT {
        return Err(BrowserError::AuthorityUnavailable);
    }
    let mut header =
        HeaderValue::from_str(&text).map_err(|_| BrowserError::AuthorityUnavailable)?;
    header.set_sensitive(true);
    Ok(header)
}

/// Values come from a successful live owner projection, never from JWT claims
/// alone or client-selected identity. This DTO does not enroll any action.
#[derive(Serialize)]
pub(super) struct AccountProjection {
    pub account_id: Uuid,
    pub session: AccountSession,
    pub permitted_self_actions: Vec<AccountActionRef>,
}

#[derive(Serialize)]
pub(super) struct AccountSession {
    pub assurance: AccountAssurance,
    #[serde(serialize_with = "utc_instant")]
    pub expires_at: OffsetDateTime,
}

#[derive(Serialize)]
pub(super) struct AccountActionRef {
    pub action_key: String,
    pub registration_revision: String,
}

impl AccountProjection {
    fn validate(&self) -> Result<(), BrowserError> {
        if self.account_id.is_nil()
            || self.permitted_self_actions.len() > 16
            || self.permitted_self_actions.iter().any(|action| {
                !valid_key(&action.action_key)
                    || !action
                        .registration_revision
                        .parse::<i64>()
                        .is_ok_and(|revision| {
                            revision > 0 && revision.to_string() == action.registration_revision
                        })
            })
        {
            return Err(BrowserError::AuthorityUnavailable);
        }
        Ok(())
    }

    pub(super) fn response(&self) -> Result<Response, BrowserError> {
        self.validate()?;
        account_response(StatusCode::OK, self)
    }

    /// The caller may attach prepared cookies only after the real transaction
    /// commits. Failure here is not permission to replay a consumed ceremony.
    pub(super) fn established_response(
        &self,
        status: StatusCode,
    ) -> Result<Response, BrowserError> {
        #[derive(Serialize)]
        struct Established<'a> {
            account: &'a AccountProjection,
        }
        if !matches!(status, StatusCode::OK | StatusCode::CREATED) {
            return Err(BrowserError::AuthorityUnavailable);
        }
        self.validate()?;
        account_response(status, &Established { account: self })
    }
}

#[derive(Serialize)]
pub(super) struct CsrfProof<'a> {
    pub csrf_proof: &'a str,
    #[serde(serialize_with = "utc_instant")]
    pub expires_at: OffsetDateTime,
}

impl CsrfProof<'_> {
    pub(super) fn response(&self) -> Result<Response, BrowserError> {
        if self.csrf_proof.is_empty() || self.csrf_proof.len() > PROOF_LIMIT {
            return Err(BrowserError::AuthorityUnavailable);
        }
        account_response(StatusCode::OK, self)
    }
}

fn utc_instant<S: serde::Serializer>(
    instant: &OffsetDateTime,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let text = instant
        .to_offset(UtcOffset::UTC)
        .format(&Rfc3339)
        .map_err(|_| serde::ser::Error::custom("invalid instant"))?;
    serializer.serialize_str(&text)
}

fn account_response<T: Serialize>(status: StatusCode, body: &T) -> Result<Response, BrowserError> {
    let bytes = serde_json::to_vec(body).map_err(|_| BrowserError::AuthorityUnavailable)?;
    if bytes.len() > ACCOUNT_RESPONSE_LIMIT {
        return Err(BrowserError::AuthorityUnavailable);
    }
    let mut response = (status, bytes).into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    private_response(&mut response);
    Ok(response)
}

pub(super) fn private_response(response: &mut Response) {
    let headers = response.headers_mut();
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(header::PRAGMA, HeaderValue::from_static("no-cache"));
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.append(header::VARY, HeaderValue::from_static("Cookie, Origin"));
}

#[derive(Clone, Copy)]
pub(super) enum BrowserError {
    AmbiguousCredentials,
    AuthenticationInvalid,
    EnrollmentInvalid,
    RequestOriginDenied,
    CsrfInvalid,
    AlreadyAuthenticated,
    TermsAcceptanceRequired,
    TermsChanged,
    RequestTooLarge,
    InvalidRequest,
    RateLimited,
    AuthorityUnavailable,
    NavigationUnavailable,
}

impl IntoResponse for BrowserError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            Self::AmbiguousCredentials => (
                StatusCode::BAD_REQUEST,
                "ambiguous_credentials",
                "Credential transport is ambiguous.",
            ),
            Self::AuthenticationInvalid => (
                StatusCode::UNAUTHORIZED,
                "authentication_invalid",
                "Authentication is invalid.",
            ),
            Self::EnrollmentInvalid => (
                StatusCode::UNAUTHORIZED,
                "enrollment_invalid",
                "Enrollment is invalid.",
            ),
            Self::RequestOriginDenied => (
                StatusCode::FORBIDDEN,
                "request_origin_denied",
                "Request origin is denied.",
            ),
            Self::CsrfInvalid => (
                StatusCode::FORBIDDEN,
                "csrf_invalid",
                "Request proof is invalid.",
            ),
            Self::AlreadyAuthenticated => (
                StatusCode::CONFLICT,
                "already_authenticated",
                "Account is already authenticated.",
            ),
            Self::TermsAcceptanceRequired => (
                StatusCode::FORBIDDEN,
                "terms_acceptance_required",
                "Terms acceptance is required.",
            ),
            Self::TermsChanged => (StatusCode::CONFLICT, "terms_changed", "Terms have changed."),
            Self::RequestTooLarge => (
                StatusCode::PAYLOAD_TOO_LARGE,
                "request_too_large",
                "Request exceeds the supported size.",
            ),
            Self::InvalidRequest => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_request",
                "Request is invalid.",
            ),
            Self::RateLimited => (
                StatusCode::TOO_MANY_REQUESTS,
                "rate_limited",
                "Request rate is exceeded.",
            ),
            Self::AuthorityUnavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "authority_unavailable",
                "Account authority is unavailable.",
            ),
            Self::NavigationUnavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "navigation_unavailable",
                "Navigation authority is unavailable.",
            ),
        };
        #[derive(Serialize)]
        struct ErrorBody {
            code: &'static str,
            message: &'static str,
        }
        #[derive(Serialize)]
        struct Envelope {
            error: ErrorBody,
        }
        let mut response = (
            status,
            Json(Envelope {
                error: ErrorBody { code, message },
            }),
        )
            .into_response();
        private_response(&mut response);
        response
    }
}

// Native HTTP owners. All credentials stay private until transaction commit.
use axum::extract::{Extension, RawQuery, State};
use console_platform_auth::AccountCsrfTokenInput;
use console_platform_auth::account::{
    AccountLiveSession, AccountOperationError, account_contexts_empty_in_tx,
    account_csrf_session_in_tx, account_login_consent_in_tx, account_now_in_tx,
    ensure_account_session_fresh_in_tx, live_account_session_in_tx, logout_account_session_in_tx,
};
use console_platform_provisioning::{
    AccountRegistrationFinishInput, AccountRegistrationStartInput, ProvisioningError,
    finish_account_registration_in_tx, start_account_registration_in_tx,
};
use console_platform_request_context::TrustedClientIp;
use sqlx::{Postgres, Transaction};

use super::{AuthRestState, AuthServices, RateLimitEndpoint, rate_limit, terms};

impl From<AccountOperationError> for BrowserError {
    fn from(error: AccountOperationError) -> Self {
        match error {
            AccountOperationError::AmbiguousCredentials => Self::AmbiguousCredentials,
            AccountOperationError::AuthenticationInvalid => Self::AuthenticationInvalid,
            AccountOperationError::EnrollmentInvalid => Self::EnrollmentInvalid,
            AccountOperationError::CsrfInvalid => Self::CsrfInvalid,
            AccountOperationError::TermsAcceptanceRequired => Self::TermsAcceptanceRequired,
            AccountOperationError::TermsChanged => Self::TermsChanged,
            AccountOperationError::AuthorityUnavailable => Self::AuthorityUnavailable,
            AccountOperationError::NavigationUnavailable => Self::NavigationUnavailable,
        }
    }
}

impl From<ProvisioningError> for BrowserError {
    fn from(error: ProvisioningError) -> Self {
        match error {
            ProvisioningError::Account(error) => error.into(),
            ProvisioningError::AccountTermsAcknowledgments => Self::InvalidRequest,
            // No SQL/crypto/input-bearing diagnostic can cross native transport.
            _ => Self::AuthorityUnavailable,
        }
    }
}

fn configured(state: &AuthRestState) -> Result<&AuthServices, BrowserError> {
    state
        .services
        .as_ref()
        .ok_or(BrowserError::AuthorityUnavailable)
}

async fn transaction(state: &AuthRestState) -> Result<Transaction<'_, Postgres>, BrowserError> {
    state
        .auth_database
        .as_ref()
        .ok_or(BrowserError::AuthorityUnavailable)?
        .begin()
        .await
        .map_err(|_| BrowserError::AuthorityUnavailable)
}

async fn complete_transaction(
    tx: Transaction<'_, Postgres>,
    response: Result<Response, BrowserError>,
) -> Result<Response, BrowserError> {
    match response {
        Ok(response) => {
            tx.commit()
                .await
                .map_err(|_| BrowserError::AuthorityUnavailable)?;
            Ok(response)
        }
        Err(error) => {
            tx.rollback()
                .await
                .map_err(|_| BrowserError::AuthorityUnavailable)?;
            Err(error)
        }
    }
}

fn supplied<'a>(
    cookie: &CookieValue<'a>,
    refusal: BrowserError,
) -> Result<Option<&'a str>, BrowserError> {
    match cookie {
        CookieValue::Absent => Ok(None),
        CookieValue::Malformed => Err(refusal),
        CookieValue::Supplied(value) => Ok(Some(value)),
    }
}

fn required<'a>(cookie: &CookieValue<'a>, refusal: BrowserError) -> Result<&'a str, BrowserError> {
    supplied(cookie, refusal)?.ok_or(refusal)
}

fn digest_bytes(value: &str) -> Result<[u8; 32], BrowserError> {
    if value.len() != 64 {
        return Err(BrowserError::InvalidRequest);
    }
    let mut bytes = [0; 32];
    for (slot, pair) in bytes.iter_mut().zip(value.as_bytes().as_chunks::<2>().0) {
        fn nibble(byte: u8) -> Result<u8, BrowserError> {
            match byte {
                b'0'..=b'9' => Ok(byte - b'0'),
                b'a'..=b'f' => Ok(byte - b'a' + 10),
                _ => Err(BrowserError::InvalidRequest),
            }
        }
        *slot = nibble(pair[0])? * 16 + nibble(pair[1])?;
    }
    Ok(bytes)
}

async fn limit(
    state: &AuthRestState,
    headers: &HeaderMap,
    client: Option<Extension<TrustedClientIp>>,
    endpoint: RateLimitEndpoint,
) -> Result<(), BrowserError> {
    rate_limit(
        &state.pool,
        headers,
        client.map(|Extension(ip)| ip),
        endpoint,
        OffsetDateTime::now_utc(),
    )
    .await
    .map_err(|error| {
        if error.status == StatusCode::TOO_MANY_REQUESTS {
            BrowserError::RateLimited
        } else {
            BrowserError::AuthorityUnavailable
        }
    })
}

async fn require_anonymous(
    tx: &mut Transaction<'_, Postgres>,
    services: &AuthServices,
    cookies: &BrowserCookies<'_>,
) -> Result<(), BrowserError> {
    let access = supplied(&cookies.session, BrowserError::AuthenticationInvalid)?;
    let refresh = supplied(&cookies.refresh, BrowserError::AuthenticationInvalid)?;
    if access.is_some() || refresh.is_some() {
        // A supplied invalid access cannot be treated as absence or rescued by a
        // refresh. The Auth owner enforces this rule while checking live state.
        account_csrf_session_in_tx(
            tx,
            &services.jwt_verifier,
            access,
            refresh,
            services.refresh_family_absolute_ttl,
        )
        .await?;
        return Err(BrowserError::AlreadyAuthenticated);
    }
    Ok(())
}

fn projection(session: &AccountLiveSession) -> AccountProjection {
    AccountProjection {
        account_id: session.account_id,
        session: AccountSession {
            assurance: session.assurance,
            expires_at: session.expires_at,
        },
        // This registration corresponds to the mounted native logout handler and
        // its actual Auth owner; additional self actions require their own owner.
        permitted_self_actions: vec![AccountActionRef {
            action_key: "account.session.logout".to_owned(),
            registration_revision: "1".to_owned(),
        }],
    }
}

pub(super) async fn registration_start(
    State(state): State<AuthRestState>,
    headers: HeaderMap,
    client: Option<Extension<TrustedClientIp>>,
    body: Body,
) -> Result<Response, BrowserError> {
    let services = configured(&state)?;
    let cookies = admit_post(&headers, &services.rp_origin)?;
    let previous_nonce = supplied(&cookies.enrollment, BrowserError::EnrollmentInvalid)?;
    limit(
        &state,
        &headers,
        client,
        RateLimitEndpoint::AccountRegistrationStart,
    )
    .await?;
    let input: RegistrationStartInput = read_json(body, START_BODY_LIMIT).await?;
    let requested_terms = digest_bytes(&input.terms_version)?;
    let artifacts = terms::registration_artifacts(&state)
        .await
        .map_err(|_| BrowserError::AuthorityUnavailable)?;
    let origin = services.rp_origin.origin().ascii_serialization();
    let mut tx = transaction(&state).await?;
    let response = async {
        require_anonymous(&mut tx, services, &cookies).await?;
        let started = start_account_registration_in_tx(
            &mut tx,
            &services.passkeys,
            AccountRegistrationStartInput {
                terms: &artifacts,
                requested_terms,
                previous_nonce,
                origin: &origin,
            },
        )
        .await?;
        // The owner allocated the ceremony on the database clock. Cookie age
        // must use that same authority, including time spent constructing proof.
        let now = account_now_in_tx(&mut tx).await?;
        let mut response = account_response(StatusCode::OK, &RegistrationStart(&started.ceremony))?;
        response.headers_mut().append(
            header::SET_COOKIE,
            issue_cookie(
                CookieName::Enrollment,
                started.browser_nonce.as_str(),
                now,
                started.ceremony.expires_at,
            )?,
        );
        Ok(response)
    }
    .await;
    complete_transaction(tx, response).await
}

pub(super) async fn registration_finish(
    State(state): State<AuthRestState>,
    headers: HeaderMap,
    client: Option<Extension<TrustedClientIp>>,
    body: Body,
) -> Result<Response, BrowserError> {
    let services = configured(&state)?;
    let cookies = admit_post(&headers, &services.rp_origin)?;
    let nonce = required(&cookies.enrollment, BrowserError::EnrollmentInvalid)?;
    limit(
        &state,
        &headers,
        client,
        RateLimitEndpoint::AccountRegistrationFinish,
    )
    .await?;
    let input: RegistrationFinishInput = read_json(body, FINISH_BODY_LIMIT).await?;
    let accepted_terms = digest_bytes(&input.accept_terms_version)?;
    let artifacts = terms::registration_artifacts(&state)
        .await
        .map_err(|_| BrowserError::AuthorityUnavailable)?;
    input.require_items(&artifacts.required_kinds().collect::<Vec<_>>())?;
    let accepted_kinds: Vec<_> = input
        .accept_items
        .iter()
        .map(|item| item.terms_kind.as_str())
        .collect();
    let origin = services.rp_origin.origin().ascii_serialization();
    let mut tx = transaction(&state).await?;
    let response = async {
        require_anonymous(&mut tx, services, &cookies).await?;
        let issued = finish_account_registration_in_tx(
            &mut tx,
            &services.passkeys,
            &services.jwt_issuer,
            AccountRegistrationFinishInput {
                terms: &artifacts,
                accepted_terms,
                accepted_kinds: &accepted_kinds,
                ceremony_id: input.ceremony_id,
                browser_nonce: nonce,
                origin: &origin,
                credential: input.credential,
                refresh_ttl: services.refresh_token_ttl,
                absolute_family_ttl: services.refresh_family_absolute_ttl,
            },
        )
        .await?;
        // Project real live authority, including the actual signed expiry, after
        // activation and before committing; metadata is never inferred from TTL.
        let session = live_account_session_in_tx(
            &mut tx,
            &services.jwt_verifier,
            issued.access.as_str(),
            services.refresh_family_absolute_ttl,
        )
        .await?;
        // Final owner check returns the checked database time. No awaited work
        // intervenes between this check, projection and cookie-age calculation.
        let now = ensure_account_session_fresh_in_tx(&mut tx, &session).await?;
        let mut response = projection(&session).established_response(StatusCode::CREATED)?;
        for cookie in [
            issue_cookie(
                CookieName::Session,
                issued.access.as_str(),
                now,
                session.expires_at,
            )?,
            issue_cookie(
                CookieName::Refresh,
                issued.family.token.as_str(),
                now,
                issued
                    .family
                    .token_expires_at
                    .min(issued.family.family_expires_at),
            )?,
            clear_cookie(CookieName::Enrollment)?,
            clear_cookie(CookieName::Login)?,
        ] {
            response.headers_mut().append(header::SET_COOKIE, cookie);
        }
        Ok(response)
    }
    .await;
    complete_transaction(tx, response).await
}

pub(super) async fn me(
    State(state): State<AuthRestState>,
    headers: HeaderMap,
) -> Result<Response, BrowserError> {
    let services = configured(&state)?;
    let cookies = admit_read(&headers, &services.rp_origin)?;
    let access = required(&cookies.session, BrowserError::AuthenticationInvalid)?;
    let mut tx = transaction(&state).await?;
    let response = async {
        let session = live_account_session_in_tx(
            &mut tx,
            &services.jwt_verifier,
            access,
            services.refresh_family_absolute_ttl,
        )
        .await?;
        ensure_account_session_fresh_in_tx(&mut tx, &session).await?;
        projection(&session).response()
    }
    .await;
    complete_transaction(tx, response).await
}

pub(super) async fn contexts(
    State(state): State<AuthRestState>,
    headers: HeaderMap,
    RawQuery(query): RawQuery,
) -> Result<Response, BrowserError> {
    let services = configured(&state)?;
    let cookies = admit_read(&headers, &services.rp_origin)?;
    if query.is_some() {
        return Err(BrowserError::InvalidRequest);
    }
    let access = required(&cookies.session, BrowserError::AuthenticationInvalid)?;
    let mut tx = transaction(&state).await?;
    let response = async {
        let session = live_account_session_in_tx(
            &mut tx,
            &services.jwt_verifier,
            access,
            services.refresh_family_absolute_ttl,
        )
        .await?;
        // This owner establishes exact native birth, generation, source-writer
        // closure and actual absence under the retained Account lock. Any candidate
        // or unknown provenance is navigation_unavailable, never an empty fallback.
        account_contexts_empty_in_tx(&mut tx, &session).await?;
        // The Auth owner rechecks expiry after its birth/source reads.
        account_response(StatusCode::OK, &serde_json::json!({"contexts": []}))
    }
    .await;
    complete_transaction(tx, response).await
}

pub(super) async fn csrf(
    State(state): State<AuthRestState>,
    headers: HeaderMap,
    client: Option<Extension<TrustedClientIp>>,
) -> Result<Response, BrowserError> {
    let services = configured(&state)?;
    let cookies = admit_proof_fetch(&headers, &services.rp_origin)?;
    let access = supplied(&cookies.session, BrowserError::AuthenticationInvalid)?;
    let refresh = supplied(&cookies.refresh, BrowserError::AuthenticationInvalid)?;
    limit(&state, &headers, client, RateLimitEndpoint::AccountCsrf).await?;
    let mut tx = transaction(&state).await?;
    let response = async {
        let session = account_csrf_session_in_tx(
            &mut tx,
            &services.jwt_verifier,
            access,
            refresh,
            services.refresh_family_absolute_ttl,
        )
        .await?;
        let now = account_now_in_tx(&mut tx).await?;
        let proof = services
            .jwt_issuer
            .issue_account_csrf_token(AccountCsrfTokenInput {
                account_id: session.account_id,
                session_id: session.session_id,
                security_generation: session.security_generation,
                issued_at: now,
                family_expires_at: session.family_expires_at,
            })
            .map_err(|_| BrowserError::AuthorityUnavailable)?;
        // Signing does not extend the authenticated session or proof lifetime.
        let now = ensure_account_session_fresh_in_tx(&mut tx, &session).await?;
        let claims = services
            .jwt_verifier
            .verify_account_csrf_token(proof.as_str(), now)
            .map_err(|_| BrowserError::AuthorityUnavailable)?;
        CsrfProof {
            csrf_proof: proof.as_str(),
            expires_at: OffsetDateTime::from_unix_timestamp(claims.exp)
                .map_err(|_| BrowserError::AuthorityUnavailable)?,
        }
        .response()
    }
    .await;
    complete_transaction(tx, response).await
}

pub(super) async fn logout(
    State(state): State<AuthRestState>,
    headers: HeaderMap,
    client: Option<Extension<TrustedClientIp>>,
    body: Body,
) -> Result<Response, BrowserError> {
    let services = configured(&state)?;
    let cookies = admit_post(&headers, &services.rp_origin)?;
    let proof = csrf_header(&headers)?;
    let access = supplied(&cookies.session, BrowserError::AuthenticationInvalid)?;
    let refresh = supplied(&cookies.refresh, BrowserError::AuthenticationInvalid)?;
    limit(&state, &headers, client, RateLimitEndpoint::AccountLogout).await?;
    let _: EmptyInput = read_json(body, START_BODY_LIMIT).await?;
    let mut tx = transaction(&state).await?;
    let response = async {
        logout_account_session_in_tx(
            &mut tx,
            &services.jwt_verifier,
            access,
            refresh,
            proof,
            services.refresh_family_absolute_ttl,
        )
        .await?;
        let mut response =
            account_response(StatusCode::OK, &serde_json::json!({"outcome": "COMMITTED"}))?;
        for name in [CookieName::Session, CookieName::Refresh] {
            response
                .headers_mut()
                .append(header::SET_COOKIE, clear_cookie(name)?);
        }
        Ok(response)
    }
    .await;
    complete_transaction(tx, response).await
}

pub(super) async fn login_start(
    State(state): State<AuthRestState>,
    headers: HeaderMap,
    client: Option<Extension<TrustedClientIp>>,
    body: Body,
) -> Result<Response, BrowserError> {
    let services = configured(&state)?;
    admit_post(&headers, &services.rp_origin)?;
    limit(
        &state,
        &headers,
        client,
        RateLimitEndpoint::AccountLoginStart,
    )
    .await?;
    let _: EmptyInput = read_json(body, START_BODY_LIMIT).await?;
    let origin = services.rp_origin.origin().ascii_serialization();
    let mut tx = transaction(&state).await?;
    let response = async {
        let started = services
            .passkeys
            .start_account_login_in_tx(&mut tx, &origin)
            .await?;
        let now = account_now_in_tx(&mut tx).await?;
        let mut response = account_response(StatusCode::OK, &LoginStart(&started.ceremony))?;
        response.headers_mut().append(
            header::SET_COOKIE,
            issue_cookie(
                CookieName::Login,
                started.browser_nonce.as_str(),
                now,
                started.ceremony.expires_at,
            )?,
        );
        Ok(response)
    }
    .await;
    complete_transaction(tx, response).await
}

pub(super) async fn login_finish(
    State(state): State<AuthRestState>,
    headers: HeaderMap,
    client: Option<Extension<TrustedClientIp>>,
    body: Body,
) -> Result<Response, BrowserError> {
    let services = configured(&state)?;
    let cookies = admit_post(&headers, &services.rp_origin)?;
    let nonce = required(&cookies.login, BrowserError::AuthenticationInvalid)?;
    limit(
        &state,
        &headers,
        client,
        RateLimitEndpoint::AccountLoginFinish,
    )
    .await?;
    let input: LoginFinishInput = read_json(body, FINISH_BODY_LIMIT).await?;
    let origin = services.rp_origin.origin().ascii_serialization();
    let mut tx = transaction(&state).await?;
    let response = async {
        // Fresh primary login can replace existing browser cookies. It never
        // merges Accounts, creates Company identity, or revokes other families.
        let primary = services
            .passkeys
            .finish_account_login_in_tx(
                &mut tx,
                input.ceremony_id,
                nonce,
                &origin,
                input.assertion,
                services.refresh_token_ttl,
                services.refresh_family_absolute_ttl,
            )
            .await?;
        let consent = account_login_consent_in_tx(&mut tx, &primary).await?;
        let artifacts = terms::historical_artifacts(&state, consent.manifest_sha256)
            .await
            .map_err(|_| BrowserError::AuthorityUnavailable)?;
        artifacts.require_historical_consent(&consent)?;
        let now = account_now_in_tx(&mut tx).await?;
        if primary.ceremony_expires_at <= now {
            return Err(BrowserError::AuthenticationInvalid);
        }
        let access = services
            .jwt_issuer
            .issue_account_access_token(console_platform_auth::AccountAccessTokenInput {
                account_id: primary.account_id,
                session_id: primary.family.family_id,
                security_generation: primary.security_generation,
                auth_time: primary.family.auth_time,
                assurance: AccountAssurance::PasskeyPrimary,
                issued_at: now,
                family_expires_at: primary.family.family_expires_at,
            })
            .map_err(|_| BrowserError::AuthorityUnavailable)?;
        let session = live_account_session_in_tx(
            &mut tx,
            &services.jwt_verifier,
            access.as_str(),
            services.refresh_family_absolute_ttl,
        )
        .await?;
        let now = ensure_account_session_fresh_in_tx(&mut tx, &session).await?;
        if primary.ceremony_expires_at <= now || primary.family.token_expires_at <= now {
            return Err(BrowserError::AuthenticationInvalid);
        }
        let mut response = projection(&session).established_response(StatusCode::OK)?;
        for cookie in [
            issue_cookie(
                CookieName::Session,
                access.as_str(),
                now,
                session.expires_at,
            )?,
            issue_cookie(
                CookieName::Refresh,
                primary.family.token.as_str(),
                now,
                primary
                    .family
                    .token_expires_at
                    .min(primary.family.family_expires_at),
            )?,
            clear_cookie(CookieName::Login)?,
            clear_cookie(CookieName::Enrollment)?,
        ] {
            response.headers_mut().append(header::SET_COOKIE, cookie);
        }
        Ok(response)
    }
    .await;
    complete_transaction(tx, response).await
}

pub(super) async fn refresh(
    State(state): State<AuthRestState>,
    headers: HeaderMap,
    client: Option<Extension<TrustedClientIp>>,
    body: Body,
) -> Result<Response, BrowserError> {
    let services = configured(&state)?;
    let cookies = admit_post(&headers, &services.rp_origin)?;
    let proof = csrf_header(&headers)?;
    let access = supplied(&cookies.session, BrowserError::AuthenticationInvalid)?;
    let refresh = required(&cookies.refresh, BrowserError::AuthenticationInvalid)?;
    limit(&state, &headers, client, RateLimitEndpoint::AccountRefresh).await?;
    let _: EmptyInput = read_json(body, START_BODY_LIMIT).await?;
    let mut tx = transaction(&state).await?;
    let response = async {
        let (family, source_session) = match services
            .refresh_tokens
            .rotate_account_in_tx(
                &mut tx,
                &services.jwt_verifier,
                access,
                refresh,
                proof,
                services.refresh_token_ttl,
                services.refresh_family_absolute_ttl,
            )
            .await?
        {
            Ok(issued) => issued,
            Err(console_platform_auth::RefreshTokenUseError::ReuseDetected) => {
                // Deliberate private refusal response inside Ok: the exact reuse
                // revocation/evidence commits before HTTP401 is released. Other
                // errors retain complete_transaction's existing rollback rule.
                return Ok(BrowserError::AuthenticationInvalid.into_response());
            }
            Err(_) => return Err(BrowserError::AuthorityUnavailable),
        };
        let now = account_now_in_tx(&mut tx).await?;
        let access = services
            .jwt_issuer
            .issue_account_access_token(console_platform_auth::AccountAccessTokenInput {
                account_id: source_session.account_id,
                session_id: source_session.session_id,
                security_generation: source_session.security_generation,
                auth_time: source_session.auth_time,
                assurance: source_session.assurance,
                issued_at: now,
                family_expires_at: source_session.family_expires_at,
            })
            .map_err(|_| BrowserError::AuthorityUnavailable)?;
        let session = live_account_session_in_tx(
            &mut tx,
            &services.jwt_verifier,
            access.as_str(),
            services.refresh_family_absolute_ttl,
        )
        .await?;
        let now = ensure_account_session_fresh_in_tx(&mut tx, &source_session).await?;
        services
            .jwt_verifier
            .verify_account_csrf_token(proof, now)
            .map_err(|_| BrowserError::CsrfInvalid)?;
        if session.expires_at <= now || family.token_expires_at <= now {
            return Err(BrowserError::AuthenticationInvalid);
        }
        let mut response = projection(&session).established_response(StatusCode::OK)?;
        for cookie in [
            issue_cookie(
                CookieName::Session,
                access.as_str(),
                now,
                session.expires_at,
            )?,
            issue_cookie(
                CookieName::Refresh,
                family.token.as_str(),
                now,
                family.token_expires_at,
            )?,
        ] {
            response.headers_mut().append(header::SET_COOKIE, cookie);
        }
        Ok(response)
    }
    .await;
    complete_transaction(tx, response).await
}
