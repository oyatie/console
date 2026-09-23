//! Ordinary child tests for the real native policy request capture/outcome
//! helpers. Opaque credentials prove transport capture only, not authentication.
use super::form::{Input, Target};
use super::{NativePolicySubmission, capture_post, finish_submission};
use axum::{
    body::Body,
    extract::Request,
    http::{HeaderValue, Method, StatusCode, header},
};
use console_identity_application::company_policy::{
    business::{NativeBusinessOperationV1, NativeCompanyBusinessCommandV1},
    workflow::{
        NativePolicyAcceptedView, NativePolicyCommandRef, NativePolicyEffect,
        NativePolicyExecution, NativePolicyOutcome, NativePolicyRejection,
        NativePolicyTerminalView, NativePolicyWorkflowError,
    },
};
use console_kernel_core::OrgId;
use console_platform_auth_rest::{AuthRestConfig, AuthRestState};
use p256::{
    ecdsa::SigningKey,
    elliptic_curve::rand_core::OsRng,
    pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding},
};
use sqlx::postgres::PgPoolOptions;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

const ORIGIN: &str = "https://auth.example.com";
const COMPANY: &str = "11111111-1111-4111-8111-111111111111";
const COMMAND: &str = "abcdefab-1234-4123-8123-abcdefabcdef";
const RECIPIENT: &str = "fedcbafe-2345-4234-8234-fedcbafedcba";
const ASSIGNMENT: &str = "abcabcab-3456-4345-8345-abcabcabcabc";
const PROOF: &str = "capture-only-proof";
const SESSION: &str = "capture-only-not-a-signed-token";
fn id(value: &str) -> Uuid {
    Uuid::parse_str(value).unwrap()
}
fn company() -> OrgId {
    OrgId::from_uuid(id(COMPANY))
}
fn auth() -> AuthRestState {
    // Existing Auth capture fixture pattern; no connection is ever requested.
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://unused:unused@localhost/unused")
        .unwrap();
    auth_with_pool(pool)
}
fn auth_with_pool(pool: sqlx::PgPool) -> AuthRestState {
    let key = SigningKey::random(&mut OsRng);
    AuthRestState::new(
        pool,
        AuthRestConfig {
            rp_id: "example.com".into(),
            rp_origin: ORIGIN.into(),
            rp_name: "Console".into(),
            ceremony_ttl: Duration::minutes(5),
            jwt_issuer: "transport-test".into(),
            jwt_audience: "transport-test".into(),
            jwt_private_key_pem: key.to_pkcs8_pem(LineEnding::LF).unwrap().to_string(),
            jwt_public_key_pem: key
                .verifying_key()
                .to_public_key_pem(LineEnding::LF)
                .unwrap(),
            refresh_token_ttl: Duration::days(30),
            refresh_family_absolute_ttl: Duration::hours(24),
            cookie_secure: true,
        },
    )
    .unwrap()
}
fn target(operation: &str) -> Target {
    match operation {
        "install" => Target::Install,
        "grant" => Target::Grant,
        "revoke" => Target::Revoke {
            assignment: id(ASSIGNMENT),
        },
        "retry" => Target::Retry {
            operation: NativeBusinessOperationV1::Grant,
            command_id: id(COMMAND),
        },
        _ => panic!("invalid fixture operation"),
    }
}
fn path(operation: &str) -> String {
    let base = format!("/companies/{COMPANY}/policy/payroll-read");
    match operation {
        "install" => format!("{base}/catalog"),
        "grant" => format!("{base}/grants"),
        "revoke" => format!("{base}/grants/{ASSIGNMENT}/revoke"),
        "retry" => format!("{base}/requests/grant/{COMMAND}/retry"),
        _ => panic!("invalid fixture operation"),
    }
}
fn body(operation: &str) -> String {
    if operation == "retry" {
        return format!("csrf_proof={PROOF}");
    }
    let mut value = format!("csrf_proof={PROOF}&command_id={COMMAND}&expected_company_epoch=2");
    match operation {
        "install" => (),
        "grant" => value.push_str(&format!("&recipient_account_id={RECIPIENT}&expected_role_revision=&assignment_id=&expected_assignment_revision=&expires_at_local=2028-02-29T00%3A15")),
        "revoke" => value.push_str("&expected_role_revision=1&expected_assignment_revision=7"),
        _ => panic!("invalid fixture operation"),
    }
    value
}
fn request(operation: &str, body: impl Into<Body>) -> Request {
    Request::builder()
        .method(Method::POST)
        .uri(path(operation))
        .header(header::ORIGIN, ORIGIN)
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(
            header::COOKIE,
            format!("__Host-console_account_session={SESSION}"),
        )
        .header("sec-fetch-site", "same-origin")
        .header("sec-fetch-mode", "navigate")
        .header("sec-fetch-dest", "document")
        .body(body.into())
        .unwrap()
}
async fn refused(
    auth: &AuthRestState,
    company: OrgId,
    target: Target,
    request: Request,
    status: StatusCode,
) {
    match capture_post(auth, request, company, target).await {
        Err(actual) => assert_eq!(actual, status),
        Ok(_) => panic!("invalid original request was captured"),
    }
}
fn selector(operation: NativeBusinessOperationV1) -> NativePolicyCommandRef {
    NativePolicyCommandRef::new(company(), id(COMMAND), operation).unwrap()
}

#[tokio::test]
async fn original_real_form_requests_capture_all_four_typed_shapes() {
    let auth = auth();
    for operation in ["install", "grant", "revoke", "retry"] {
        let prepared = capture_post(
            &auth,
            request(operation, body(operation)),
            company(),
            target(operation),
        )
        .await
        .unwrap_or_else(|_| panic!("valid original form transport rejected"));
        // Credentials remain opaque and are not inspected, logged or treated as
        // current authority. Existing owners must authenticate them later.
        let (_credentials, input) = prepared;
        match match input {
            super::form::DocumentInput::Ready(input) => input,
            super::form::DocumentInput::GrantValidation(_) => {
                panic!("valid input became validation")
            }
        } {
            Input::Command(command) => {
                assert_ne!(operation, "retry");
                assert_eq!(command.command_id(), id(COMMAND));
                assert_eq!(command.company(), company());
                assert_eq!(command.expected_company_epoch(), 2);
                let expected = match operation {
                    "install" => NativeBusinessOperationV1::Install,
                    "grant" => NativeBusinessOperationV1::Grant,
                    "revoke" => NativeBusinessOperationV1::Revoke,
                    _ => unreachable!(),
                };
                assert_eq!(command.operation(), expected);
            }
            Input::Retry(selected) => {
                assert_eq!(operation, "retry");
                assert_eq!(selected, selector(NativeBusinessOperationV1::Grant));
            }
        }
    }
}

#[tokio::test]
async fn original_method_origin_media_metadata_and_proof_namespace_are_not_rewritten() {
    let auth = auth();
    let mut wrong_method = request("install", body("install"));
    *wrong_method.method_mut() = Method::PUT;
    refused(
        &auth,
        company(),
        target("install"),
        wrong_method,
        StatusCode::BAD_REQUEST,
    )
    .await;
    for (name, value, expected) in [
        (
            "origin",
            "https://attacker.example.com",
            StatusCode::FORBIDDEN,
        ),
        ("content-type", "application/json", StatusCode::BAD_REQUEST),
        (
            "content-type",
            "application/x-www-form-urlencoded; charset=utf-8",
            StatusCode::BAD_REQUEST,
        ),
        ("sec-fetch-site", "same-site", StatusCode::FORBIDDEN),
        ("sec-fetch-mode", "cors", StatusCode::FORBIDDEN),
        ("sec-fetch-dest", "iframe", StatusCode::FORBIDDEN),
        ("x-console-csrf", "second-proof", StatusCode::BAD_REQUEST),
    ] {
        let mut req = request("install", body("install"));
        req.headers_mut()
            .insert(name, HeaderValue::from_str(value).unwrap());
        refused(&auth, company(), target("install"), req, expected).await;
    }
    let mut missing_origin = request("install", body("install"));
    missing_origin.headers_mut().remove(header::ORIGIN);
    refused(
        &auth,
        company(),
        target("install"),
        missing_origin,
        StatusCode::FORBIDDEN,
    )
    .await;
    let mut partial = request("install", body("install"));
    partial.headers_mut().remove("sec-fetch-dest");
    refused(
        &auth,
        company(),
        target("install"),
        partial,
        StatusCode::BAD_REQUEST,
    )
    .await;
    let mut absent = request("install", body("install"));
    for name in ["sec-fetch-site", "sec-fetch-mode", "sec-fetch-dest"] {
        absent.headers_mut().remove(name);
    }
    assert!(
        capture_post(&auth, absent, company(), target("install"))
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn ambiguous_or_missing_access_is_never_replaced_by_refresh_or_api_credentials() {
    let auth = auth();
    let mut missing = request("install", body("install"));
    missing.headers_mut().remove(header::COOKIE);
    refused(
        &auth,
        company(),
        target("install"),
        missing,
        StatusCode::UNAUTHORIZED,
    )
    .await;
    let mut refresh = request("install", body("install"));
    refresh.headers_mut().insert(
        header::COOKIE,
        HeaderValue::from_static("__Host-console_account_refresh=capture-only-refresh"),
    );
    refused(
        &auth,
        company(),
        target("install"),
        refresh,
        StatusCode::UNAUTHORIZED,
    )
    .await;
    let mut bearer = request("install", body("install"));
    bearer.headers_mut().insert(
        header::AUTHORIZATION,
        HeaderValue::from_static("Bearer capture-only-api"),
    );
    refused(
        &auth,
        company(),
        target("install"),
        bearer,
        StatusCode::BAD_REQUEST,
    )
    .await;
    let mut duplicate = request("install", body("install"));
    duplicate
        .headers_mut()
        .append(header::ORIGIN, HeaderValue::from_static(ORIGIN));
    refused(
        &auth,
        company(),
        target("install"),
        duplicate,
        StatusCode::BAD_REQUEST,
    )
    .await;
}

#[tokio::test]
async fn actual_body_bytes_and_decoded_fields_are_checked_before_owner_handoff() {
    let auth = auth();
    let mut oversized = request("install", vec![b'x'; 16385]);
    oversized
        .headers_mut()
        .insert(header::CONTENT_LENGTH, HeaderValue::from_static("1"));
    refused(
        &auth,
        company(),
        target("install"),
        oversized,
        StatusCode::PAYLOAD_TOO_LARGE,
    )
    .await;
    for malformed in [
        format!("{}&%63srf_proof=duplicate", body("install")),
        body("install").replace(PROOF, "%FF"),
        body("install").replace(PROOF, ""),
        format!("{}&actor={RECIPIENT}", body("install")),
    ] {
        refused(
            &auth,
            company(),
            target("install"),
            request("install", malformed),
            StatusCode::BAD_REQUEST,
        )
        .await;
    }
    let proof_overflow = body("install").replace(PROOF, &"p".repeat(4097));
    refused(
        &auth,
        company(),
        target("install"),
        request("install", proof_overflow),
        StatusCode::PAYLOAD_TOO_LARGE,
    )
    .await;
    let fully_encoded = body("install").replace(PROOF, &"%70".repeat(4096));
    assert!(
        capture_post(
            &auth,
            request("install", fully_encoded),
            company(),
            target("install")
        )
        .await
        .is_ok()
    );
}

#[tokio::test]
async fn retry_body_cannot_replace_original_input_or_typed_route_selector() {
    let auth = auth();
    for field in [
        format!("command_id={COMMAND}"),
        "expected_company_epoch=2".into(),
        format!("recipient_account_id={RECIPIENT}"),
        "expires_at_local=2028-02-29T00%3A15".into(),
        "operation=grant".into(),
        format!("assignment_id={ASSIGNMENT}"),
    ] {
        refused(
            &auth,
            company(),
            target("retry"),
            request("retry", format!("{}&{field}", body("retry"))),
            StatusCode::BAD_REQUEST,
        )
        .await;
    }
    let other_command = Uuid::from_u128(42);
    let mut original = request("retry", body("retry"));
    *original.uri_mut() =
        format!("/companies/{COMPANY}/policy/payroll-read/requests/grant/{other_command}/retry")
            .parse()
            .unwrap();
    let (_, input) = capture_post(
        &auth,
        original,
        company(),
        Target::Retry {
            operation: NativeBusinessOperationV1::Grant,
            command_id: other_command,
        },
    )
    .await
    .unwrap_or_else(|_| panic!("valid typed original selector rejected"));
    match match input {
        super::form::DocumentInput::Ready(input) => input,
        super::form::DocumentInput::GrantValidation(_) => panic!("valid input became validation"),
    } {
        Input::Retry(actual) => assert_eq!(
            actual,
            NativePolicyCommandRef::new(company(), other_command, NativeBusinessOperationV1::Grant)
                .unwrap()
        ),
        Input::Command(_) => panic!("retry created new command"),
    }
}

#[test]
fn unconfirmed_result_preserves_exact_original_command_and_never_offers_held_resubmit() {
    for operation in [
        NativeBusinessOperationV1::Install,
        NativeBusinessOperationV1::Grant,
        NativeBusinessOperationV1::Revoke,
    ] {
        match finish_submission(
            selector(operation),
            Err(NativePolicyWorkflowError::Unconfirmed),
        ) {
            Ok(NativePolicySubmission::Unconfirmed { selector: actual }) => {
                assert_eq!(actual, selector(operation))
            }
            _ => panic!("unknown result was represented as confirmation or replacement submission"),
        }
    }
}

#[test]
fn workflow_failures_produce_neutral_unscoped_response_variants() {
    use NativePolicyWorkflowError as E;
    for (cause, expected) in [
        (E::InvalidInput, StatusCode::UNPROCESSABLE_ENTITY),
        (E::AuthenticationInvalid, StatusCode::UNAUTHORIZED),
        (E::CsrfInvalid, StatusCode::FORBIDDEN),
        (E::NotFound, StatusCode::NOT_FOUND),
        (E::Conflict, StatusCode::CONFLICT),
        (E::Capacity, StatusCode::TOO_MANY_REQUESTS),
        (E::Unavailable, StatusCode::SERVICE_UNAVAILABLE),
    ] {
        match finish_submission(selector(NativeBusinessOperationV1::Grant), Err(cause)) {
            Err(status) => assert_eq!(status, expected),
            _ => panic!("failure retained scoped outcome or recovery material"),
        }
    }
}

#[test]
fn durable_terminal_success_or_rejection_redirects_to_same_persistent_result() {
    let input = NativeCompanyBusinessCommandV1::install(id(COMMAND), company(), 2).unwrap();
    let selected = NativePolicyCommandRef::from_command(&input);
    let accepted_at = OffsetDateTime::from_unix_timestamp(1800000000).unwrap();
    let accepted = NativePolicyAcceptedView {
        input,
        intake_receipt_id: Uuid::from_u128(101),
        accepted_at,
        execution_not_after: accepted_at + Duration::days(7),
    };
    for outcome in [
        NativePolicyOutcome::Committed(NativePolicyEffect::Installed {
            object_type_id: Uuid::from_u128(102),
        }),
        NativePolicyOutcome::Rejected(NativePolicyRejection::IntakeExpired),
        NativePolicyOutcome::Rejected(NativePolicyRejection::RevisionConflict),
    ] {
        let epoch_after = if matches!(outcome, NativePolicyOutcome::Committed(_)) {
            3
        } else {
            2
        };
        let executed_at = if matches!(
            outcome,
            NativePolicyOutcome::Rejected(NativePolicyRejection::IntakeExpired)
        ) {
            accepted_at + Duration::days(7)
        } else {
            accepted_at
        };
        let terminal = NativePolicyTerminalView {
            accepted: accepted.clone(),
            receipt_id: Uuid::from_u128(103),
            executed_at,
            epoch_before: 2,
            epoch_after,
            outcome,
        };
        for inserted in [true, false] {
            match finish_submission(
                selected,
                Ok(NativePolicyExecution {
                    inserted,
                    terminal: terminal.clone(),
                }),
            ) {
                Ok(NativePolicySubmission::Confirmed { selector: actual }) => {
                    assert_eq!(actual, selected)
                }
                _ => panic!("durable terminal outcome failed persistent redirect"),
            }
        }
    }
}

include!("expiry_validation_capture_tests.rs");


// Scripted material tests production REST routing only, not live Auth or SQL
// custody. The real Account/Company owner test covers those independently.
mod read_routing {
    use super::*;
    use crate::company::CompanyRestState;
    use console_identity_application::company_policy::{
        AccountId, CompanyPolicyDecisionPort, CurrentNativeBootstrapAuthority,
        NativeBootstrapProjectionRow, NativePolicySourceBinding, workflow::*,
    };
    use console_kernel_core::TraceContext;
    use console_platform_auth::account::{AccountEnrollmentCredentials, AccountFormProof};
    use console_platform_authz::company_policy::CompanyPolicy;
    use console_platform_request_context::TrustedClientIp;
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct Store { events: Mutex<Vec<&'static str>> }
    struct Scope<'a> {
        store: &'a Store,
        current: bool,
        authority: CurrentNativeBootstrapAuthority,
    }
    impl Store {
        fn record(&self, event: &'static str) { self.events.lock().unwrap().push(event); }
    }
    impl NativePolicyWorkflowStore for Store {
        type Credentials = AccountEnrollmentCredentials;
        type FormProof = AccountFormProof;
        type Scope<'a> = Scope<'a>;
        async fn lock<'a>(&'a self, _: &'a Self::Credentials, request: NativePolicyScopeRequest<'a>)
            -> Result<Scope<'a>, NativePolicyWorkflowError> {
            assert_eq!(request.selector(), selector(NativeBusinessOperationV1::Install));
            let current = match request {
                NativePolicyScopeRequest::Status(_) => { self.record("lock:status"); false },
                NativePolicyScopeRequest::Current(_) => { self.record("lock:current"); true },
                _ => panic!("proof/workflow reached after refusal or on inert receipt"),
            };
            let at = OffsetDateTime::from_unix_timestamp(1_800_000_000).unwrap();
            let uid = Uuid::from_u128;
            let binding = NativePolicySourceBinding {
                account: AccountId::from_uuid(uid(1)).unwrap(), session_id: uid(2),
                account_security_generation: 1, source_xid: 9001, source_backend_pid: 123, observed_at: at,
            };
            let authority = CurrentNativeBootstrapAuthority::from_retained_projection(&binding,
                NativeBootstrapProjectionRow {
                    actor_account_id: uid(1), session_id: uid(2), account_security_generation: 1,
                    designation_system_identifier: "7610000000000000001".into(),
                    designation_database_name: "scripted_transport".into(), designation_database_oid: 16384,
                    designation_revision: 3, designation_receipt_id: uid(4), org_id: id(COMPANY),
                    current_group_id: uid(12), group_revision: 2, group_incarnation: uid(13),
                    membership_id: uid(14), membership_revision: 2, membership_incarnation: uid(15),
                    company_epoch: 1, current_policy_receipt_id: None,
                    origin_account_id: uid(31), origin_command_id: uid(32), origin_receipt_id: uid(33),
                    administrative_account_id: uid(34), company_actor_admission_receipt_id: uid(33),
                    birth_assignment_id: uid(35), birth_role_id: uid(36), observed_at: at,
                    source_xid: "9001".into(), source_backend_pid: 123,
                }).unwrap();
            Ok(Scope { store: self, current, authority })
        }
    }
    impl Drop for Scope<'_> {
        fn drop(&mut self) { self.store.record("release"); }
    }
    impl NativePolicyWorkflowScope for Scope<'_> {
        type FormProof = AccountFormProof;
        fn authority(&self) -> &CurrentNativeBootstrapAuthority { &self.authority }
        async fn current(&mut self) -> Result<NativePolicyFormView, NativePolicyWorkflowError> {
            assert!(self.current);
            self.store.record("current");
            Ok(NativePolicyFormView {
                selector: selector(NativeBusinessOperationV1::Install), group_id: Uuid::from_u128(12),
                company_epoch: 1, acting_account_id: AccountId::from_uuid(Uuid::from_u128(1)).unwrap(),
                administrative_account_id: AccountId::from_uuid(Uuid::from_u128(34)).unwrap(),
                installed_object_type_id: None, assignment: None,
            })
        }
        async fn form(&mut self) -> Result<NativePolicyForm<AccountFormProof>, NativePolicyWorkflowError> {
            panic!("inert receipt issued a proof")
        }
        async fn accept(&mut self, _: &TraceContext) -> Result<NativePolicyAcceptance, NativePolicyWorkflowError> {
            panic!("read accepted a command")
        }
        async fn execute(&mut self, _: &TraceContext) -> Result<NativePolicyExecution, NativePolicyWorkflowError> {
            panic!("read executed a command")
        }
        async fn status(&mut self) -> Result<NativePolicyStatus, NativePolicyWorkflowError> {
            assert!(!self.current);
            self.store.record("status:expired");
            let at = self.authority.source().observed_at;
            Ok(NativePolicyStatus::AcceptedExpired(NativePolicyAcceptedView {
                input: NativeCompanyBusinessCommandV1::install(id(COMMAND), company(), 1).unwrap(),
                intake_receipt_id: Uuid::from_u128(99),
                accepted_at: at - Duration::days(7), execution_not_after: at,
            }))
        }
        async fn finish<P: CompanyPolicyDecisionPort + ?Sized>(self, _: &P, proof: Option<&AccountFormProof>)
            -> Result<(), NativePolicyWorkflowError> {
            assert!(proof.is_none());
            self.store.record("finish:none");
            Ok(())
        }
    }
    fn state(pool: sqlx::PgPool) -> CompanyRestState<Store> {
        CompanyRestState { store: Store::default(), auth: auth_with_pool(pool),
            policy: Some(Arc::new(CompanyPolicy::new().unwrap())) }
    }
    fn headers() -> axum::http::HeaderMap { request("install", "").headers().clone() }
    fn client() -> Option<TrustedClientIp> { Some(TrustedClientIp::new("203.0.113.50".parse().unwrap())) }

    #[tokio::test]
    async fn expired_receipt_uses_current_without_proof_or_metering() {
        let pool = PgPoolOptions::new().connect_lazy("postgres://unused:unused@localhost/unused").unwrap();
        pool.close().await; // Any ordinary awaited limiter call must now fail.
        let state = state(pool);
        match state.policy_request_document(&headers(), client(), COMPANY, "install", COMMAND).await.unwrap() {
            super::super::NativePolicyRequestDocument::Visible { status, current, proof } => {
                assert!(matches!(status, NativePolicyStatus::AcceptedExpired(_)));
                assert_eq!(current.selector, selector(NativeBusinessOperationV1::Install));
                assert!(proof.is_none());
            },
            _ => panic!("scripted expired receipt lost its visible current context"),
        }
        assert_eq!(*state.store.events.lock().unwrap(), ["lock:status", "status:expired", "finish:none", "release",
            "lock:current", "current", "finish:none", "release"]);
    }

    #[sqlx::test(migrations = "../../platform/db/migrations")]
    async fn form_metering_refuses_before_owner_and_invalid_capture_preserves_quota(pool: sqlx::PgPool) {
        let state = state(pool.clone());
        // Explicit infrastructure saturation fault only, never business fixture data.
        // Cover a possible minute rollover, with bounded wall-clock witnesses below.
        let started = OffsetDateTime::now_utc();
        let window = started.unix_timestamp().div_euclid(60) * 60;
        for epoch in [window, window + 60] {
            sqlx::query("INSERT INTO auth_rate_limit(client_key,endpoint,window_start,attempts) VALUES ('ip:203.0.113.50','account_csrf',$1,60)")
                .bind(OffsetDateTime::from_unix_timestamp(epoch).unwrap()).execute(&pool).await.unwrap();
        }
        let mut invalid = headers();
        invalid.remove(header::COOKIE);
        assert!(matches!(state.policy_form_document(&invalid, client(), COMPANY, "install").await, Err(StatusCode::UNAUTHORIZED)));
        let counts: Vec<i32> = sqlx::query_scalar("SELECT attempts FROM auth_rate_limit ORDER BY window_start")
            .fetch_all(&pool).await.unwrap();
        assert_eq!(counts, [60, 60], "invalid capture consumed quota");
        let mut valid = headers();
        valid.insert("x-forwarded-for", "198.51.100.99".parse().unwrap());
        assert!(matches!(state.policy_form_document(&valid, client(), COMPANY, "install").await, Err(StatusCode::TOO_MANY_REQUESTS)));
        let finished = OffsetDateTime::now_utc();
        assert!(started <= finished && finished - started < Duration::seconds(30), "quota fixture interval invalid");
        let rows: Vec<(String,String,OffsetDateTime,i32)> = sqlx::query_as("SELECT client_key,endpoint,window_start,attempts FROM auth_rate_limit ORDER BY window_start")
            .fetch_all(&pool).await.unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows.iter().map(|row| row.3).sum::<i32>(), 121);
        for (i, row) in rows.iter().enumerate() {
            assert_eq!((&*row.0, &*row.1), ("ip:203.0.113.50", "account_csrf"));
            assert_eq!(row.2.unix_timestamp(), window + i as i64 * 60);
            assert!([60,61].contains(&row.3));
        }
        assert!(state.store.events.lock().unwrap().is_empty());
        let unavailable_pool = PgPoolOptions::new().connect_lazy("postgres://unused:unused@localhost/unused").unwrap();
        unavailable_pool.close().await;
        let unavailable = self::state(unavailable_pool);
        assert!(matches!(unavailable.policy_form_document(&headers(), client(), COMPANY, "install").await, Err(StatusCode::SERVICE_UNAVAILABLE)));
        assert!(unavailable.store.events.lock().unwrap().is_empty());
    }
}
