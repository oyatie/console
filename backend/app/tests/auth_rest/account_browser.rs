//! Test candidate bound to BW31 5f9f3b15704deaf8e81a682e4475af489d164210.
//! No test-created schema/roles or substitute handlers. Terms publication rows
//! are prerequisites, not publisher-authority/CAS proof. SoftPasskey's UV flag,
//! nonresident registration and allow-list/userHandle accommodations below are
//! synthetic crypto fixtures, NOT browser presence, residency or discovery proof.

use super::*;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

const PACKET: &str = "docs/evidence/console/design/2026-09-13-account-browser-wire-31";
const MANIFEST_DIGEST: &str = "3643e74a128a2fb3facce194264bcb40487dc79548b4d3b734b1de90be76fe52";
const ACCESS: &str = "__Host-console_account_session";
const REFRESH: &str = "__Host-console_account_refresh";
const ENROLLMENT: &str = "__Host-console_account_enrollment";
const LOGIN: &str = "__Host-console_account_login";
const RECEIPT: &str = "31313131-3131-4131-8131-313131313131";

fn artifact_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(PACKET)
}

fn manifest() -> Value {
    serde_json::from_slice(&std::fs::read(artifact_root().join("fixtures/manifest.json")).unwrap())
        .unwrap()
}

fn exact_keys(value: &Value, keys: &[&str]) {
    let actual: BTreeSet<_> = value
        .as_object()
        .expect("object wrapper")
        .keys()
        .map(String::as_str)
        .collect();
    assert!(
        actual == keys.iter().copied().collect(),
        "unexpected wrapper keys"
    );
}

#[derive(Clone, Default)]
struct Cookies(BTreeMap<String, String>);

impl Cookies {
    fn header(&self) -> String {
        self.0
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join("; ")
    }

    fn absorb(&mut self, headers: &http::HeaderMap) {
        let mut seen = BTreeSet::new();
        for value in headers.get_all(header::SET_COOKIE) {
            let mut parts = value.to_str().unwrap().split(';').map(str::trim);
            let (name, secret) = parts.next().unwrap().split_once('=').unwrap();
            assert!(
                [ACCESS, REFRESH, ENROLLMENT, LOGIN].contains(&name),
                "unexpected cookie name"
            );
            assert!(seen.insert(name.to_owned()), "duplicate Set-Cookie name");
            let attrs: BTreeSet<_> = parts.map(str::to_ascii_lowercase).collect();
            assert!(
                attrs.contains("secure") && attrs.contains("httponly") && attrs.contains("path=/")
            );
            assert!(!attrs.iter().any(|a| a.starts_with("domain=")));
            assert!(attrs.contains(if name == ACCESS {
                "samesite=lax"
            } else {
                "samesite=strict"
            }));
            let age: i64 = attrs
                .iter()
                .find_map(|a| a.strip_prefix("max-age="))
                .expect("bounded cookie lifetime")
                .parse()
                .unwrap();
            if secret.is_empty() {
                assert_eq!(age, 0);
                self.0.remove(name);
            } else {
                assert!(age > 0);
                if name == ACCESS {
                    assert!(age <= 900);
                }
                if name == ENROLLMENT || name == LOGIN {
                    assert!(age <= 300);
                }
                self.0.insert(name.to_owned(), secret.to_owned());
            }
        }
    }
}

struct Response {
    status: StatusCode,
    headers: http::HeaderMap,
    bytes: Vec<u8>,
}

impl Response {
    fn json(&self, expected: StatusCode) -> Value {
        assert_eq!(self.status, expected, "BW31 HTTP contract not satisfied");
        serde_json::from_slice(&self.bytes).expect("JSON response, no fallback HTML")
    }

    fn private(&self) {
        assert!(
            self.headers
                .get(header::CACHE_CONTROL)
                .unwrap()
                .to_str()
                .unwrap()
                .split(',')
                .any(|s| s.trim() == "no-store")
        );
        assert_eq!(self.headers.get(header::PRAGMA).unwrap(), "no-cache");
    }

    fn error(&self, status: StatusCode, code: &str) {
        let value = self.json(status);
        exact_keys(&value, &["error"]);
        exact_keys(&value["error"], &["code", "message"]);
        assert_eq!(value["error"]["code"], code);
        assert!(
            value["error"]["message"]
                .as_str()
                .is_some_and(|m| !m.is_empty() && m.len() <= 256)
        );
        assert!(
            !self.headers.contains_key(header::SET_COOKIE),
            "refusal changed browser identity"
        );
        self.private();
    }
}

async fn request(
    router: &axum::Router,
    method: &str,
    path: &str,
    cookies: &Cookies,
    body: Option<Value>,
    extra: &[(&str, &str)],
) -> Response {
    let mut req = Request::builder()
        .method(method)
        .uri(path)
        .header(header::ORIGIN, TEST_ORIGIN)
        .header("Sec-Fetch-Site", "same-origin");
    if !cookies.0.is_empty() {
        req = req.header(header::COOKIE, cookies.header());
    }
    for (name, value) in extra {
        req = req.header(*name, *value);
    }
    let data = if let Some(body) = body {
        req = req.header(header::CONTENT_TYPE, "application/json");
        Body::from(serde_json::to_vec(&body).unwrap())
    } else {
        Body::empty()
    };
    let mut req = req.body(data).unwrap();
    req.extensions_mut().insert(ConnectInfo(
        "127.0.0.1:41000".parse::<SocketAddr>().unwrap(),
    ));
    let response = router.clone().oneshot(req).await.unwrap();
    let (parts, body) = response.into_parts();
    Response {
        status: parts.status,
        headers: parts.headers,
        bytes: to_bytes(body, 256 * 1024).await.unwrap().to_vec(),
    }
}

async fn router(pool: &PgPool, root: PathBuf) -> axum::Router {
    let signing_key = SigningKey::random(&mut OsRng);
    let mut pairs = vec![
        ("CONSOLE_APP_ROLE", AppRole::Api.to_string()),
        ("CONSOLE_HTTP_ADDR", "127.0.0.1:0".to_owned()),
        ("CONSOLE_JWT_ISSUER", TEST_ISSUER.to_owned()),
        ("CONSOLE_JWT_AUDIENCE", TEST_AUDIENCE.to_owned()),
        (
            "CONSOLE_JWT_PRIVATE_KEY_PEM",
            signing_key
                .to_pkcs8_pem(LineEnding::LF)
                .unwrap()
                .to_string(),
        ),
        (
            "CONSOLE_JWT_PUBLIC_KEY_PEM",
            signing_key
                .verifying_key()
                .to_public_key_pem(LineEnding::LF)
                .unwrap(),
        ),
        ("CONSOLE_WEBAUTHN_RP_ID", "example.com".to_owned()),
        ("CONSOLE_WEBAUTHN_RP_ORIGIN", TEST_ORIGIN.to_owned()),
        ("CONSOLE_WEBAUTHN_RP_NAME", "Console".to_owned()),
        // Trusted internal release-directory wiring, not caller-controlled HTTP.
        // The index lives at fixtures/artifact-index.json under this package root.
        // Config parsing alone is NOT proof: exact served bytes are asserted.
        (
            "CONSOLE_ACCOUNT_TERMS_ARTIFACT_ROOT",
            root.to_str().unwrap().to_owned(),
        ),
    ];
    pairs.extend(account_transport_urls(pool));
    build_router(
        AppState::from_config(AppConfig::from_pairs(pairs).unwrap())
            .await
            .unwrap(),
    )
}

/// Fixture inserts ONLY the reviewed immutable publication/head data. Runtime
/// migration owns the actual tables, constraints, functions, roles and grants.
async fn seed_terms(pool: &PgPool) {
    let present: bool = sqlx::query_scalar("SELECT to_regclass('public.account_terms_head') IS NOT NULL AND to_regclass('public.account_terms_release_receipts') IS NOT NULL")
        .fetch_one(pool).await.unwrap();
    assert!(
        present,
        "BW31 terms schema prerequisite missing; enrollment assertions not reached"
    );
    let approval = serde_json::to_vec(&json!({
        "kind":"ACCOUNT_TERMS_PUBLICATION_APPROVAL", "approval_id":"32323232-3232-4232-8232-323232323232",
        "manifest_sha256":MANIFEST_DIGEST, "expected_revision":"0", "next_revision":"1", "fixture_only":true,
        "approved_by":"test_only.operator", "approved_at":"2026-09-13T00:00:00Z",
        "content_authority_refs":[hex::encode(Sha256::digest(b"TEST_ONLY content authority, never legal proof"))]
    })).unwrap();
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("INSERT INTO account_terms_release_receipts (id,previous_revision,revision,manifest_sha256,approved_release_ref,approval_bytes,recorded_at) VALUES ($1,NULL,1,$2,$3,$4,now())")
        .bind(Uuid::parse_str(RECEIPT).unwrap()).bind(hex::decode(MANIFEST_DIGEST).unwrap())
        .bind(json!({"kind":"OPERATOR_RELEASE_APPROVAL","approval_sha256":hex::encode(Sha256::digest(&approval))}))
        .bind(approval).execute(&mut *tx).await.unwrap();
    sqlx::query("INSERT INTO account_terms_head (id,manifest_sha256,revision,release_receipt_ref,updated_at) VALUES (1,$1,1,$2,now())")
        .bind(hex::decode(MANIFEST_DIGEST).unwrap()).bind(Uuid::parse_str(RECEIPT).unwrap()).execute(&mut *tx).await.unwrap();
    tx.commit().await.unwrap();
}

async fn fixture(pool: &PgPool) -> axum::Router {
    prepare_http_database(pool).await;
    seed_terms(pool).await;
    router(pool, artifact_root()).await
}

struct Attempt {
    account: Uuid,
    ceremony: Uuid,
    cookies: Cookies,
    finish: Value,
    authenticator: WebauthnAuthenticator<SoftPasskey>,
}

async fn start(router: &axum::Router) -> Attempt {
    let response = request(
        router,
        "POST",
        "/api/v2/auth/registration/start",
        &Cookies::default(),
        Some(json!({"terms_version":MANIFEST_DIGEST})),
        &[],
    )
    .await;
    let start = response.json(StatusCode::OK);
    response.private();
    exact_keys(&start, &["ceremony_id", "public_key_options"]);
    let options = &start["public_key_options"];
    exact_keys(options, &["publicKey"]);
    assert_eq!(
        options["publicKey"]["authenticatorSelection"]["residentKey"],
        "required"
    );
    assert_eq!(
        options["publicKey"]["authenticatorSelection"]["requireResidentKey"],
        true
    );
    assert_eq!(
        options["publicKey"]["authenticatorSelection"]["userVerification"],
        "required"
    );
    let account = Uuid::from_slice(
        &URL_SAFE_NO_PAD
            .decode(options["publicKey"]["user"]["id"].as_str().unwrap())
            .unwrap(),
    )
    .unwrap();
    assert!(!account.is_nil());
    let mut cookies = Cookies::default();
    cookies.absorb(&response.headers);
    assert!(
        cookies.0.len() == 1 && cookies.0.contains_key(ENROLLMENT),
        "start minted a session"
    );
    // Explicit CLIENT-ONLY accommodation: SoftPasskey cannot store resident keys.
    // Server challenge/state/signature inputs remain untouched. Never ship this
    // adjustment in the browser or describe this as resident-authenticator proof.
    let mut adapted = options.clone();
    adapted["publicKey"]["authenticatorSelection"]["residentKey"] = json!("discouraged");
    adapted["publicKey"]["authenticatorSelection"]["requireResidentKey"] = json!(false);
    let challenge: CreationChallengeResponse = serde_json::from_value(adapted).unwrap();
    let mut authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let credential = authenticator
        .do_registration(Url::parse(TEST_ORIGIN).unwrap(), challenge)
        .unwrap();
    let ceremony = Uuid::parse_str(start["ceremony_id"].as_str().unwrap()).unwrap();
    let acknowledgments: Vec<_> = manifest()["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| json!({"terms_kind":i["terms_kind"],"accepted":true}))
        .collect();
    Attempt {
        account,
        ceremony,
        cookies,
        finish: json!({"ceremony_id":ceremony,"credential":credential,"accept_terms_version":MANIFEST_DIGEST,"accept_items":acknowledgments}),
        authenticator,
    }
}

fn projection(value: &Value, account: Uuid) {
    exact_keys(value, &["account_id", "session", "permitted_self_actions"]);
    assert_eq!(value["account_id"], account.to_string());
    exact_keys(&value["session"], &["assurance", "expires_at"]);
    assert_eq!(value["session"]["assurance"], "PASSKEY_PRIMARY");
    let expiry = OffsetDateTime::parse(
        value["session"]["expires_at"].as_str().unwrap(),
        &time::format_description::well_known::Rfc3339,
    )
    .unwrap();
    assert!(
        expiry > OffsetDateTime::now_utc()
            && expiry <= OffsetDateTime::now_utc() + Duration::minutes(15)
    );
    assert_eq!(
        value["permitted_self_actions"],
        json!([{"action_key":"account.session.logout","registration_revision":"1"}])
    );
}

fn session(response: &Response, status: StatusCode, account: Uuid, cookies: &mut Cookies) {
    let value = response.json(status);
    response.private();
    exact_keys(&value, &["account"]);
    projection(&value["account"], account);
    cookies.absorb(&response.headers);
    assert!(cookies.0.contains_key(ACCESS) && cookies.0.contains_key(REFRESH));
    assert!(!cookies.0.contains_key(ENROLLMENT) && !cookies.0.contains_key(LOGIN));
    for secret in cookies.0.values() {
        assert!(
            !response
                .bytes
                .windows(secret.len())
                .any(|w| w == secret.as_bytes()),
            "credential in response body"
        );
    }
}

async fn finish(router: &axum::Router, attempt: &Attempt) -> Response {
    request(
        router,
        "POST",
        "/api/v2/auth/registration/finish",
        &attempt.cookies,
        Some(attempt.finish.clone()),
        &[],
    )
    .await
}

async fn enrolled(router: &axum::Router) -> (Attempt, Cookies) {
    let attempt = start(router).await;
    let response = finish(router, &attempt).await;
    let mut cookies = attempt.cookies.clone();
    session(
        &response,
        StatusCode::CREATED,
        attempt.account,
        &mut cookies,
    );
    (attempt, cookies)
}

/// Snapshot carries no plaintext token. Assert equality without printing rows.
async fn snapshot(pool: &PgPool, account: Uuid) -> Value {
    sqlx::query_scalar("SELECT jsonb_build_object('security',(SELECT to_jsonb(s) FROM account_security s WHERE account_id=$1), 'keys',(SELECT coalesce(jsonb_agg(to_jsonb(k) ORDER BY id),'[]') FROM auth_webauthn_credentials k WHERE user_id=$1), 'families',(SELECT coalesce(jsonb_agg(to_jsonb(f) ORDER BY id),'[]') FROM auth_refresh_token_families f WHERE user_id=$1), 'tokens',(SELECT coalesce(jsonb_agg(to_jsonb(t) ORDER BY id),'[]') FROM auth_refresh_tokens t WHERE user_id=$1), 'terms',(SELECT coalesce(jsonb_agg(to_jsonb(a) ORDER BY terms_kind),'[]') FROM account_terms_acceptances a WHERE account_id=$1), 'events',(SELECT coalesce(jsonb_agg(to_jsonb(e) ORDER BY id),'[]') FROM account_security_events e WHERE account_id=$1))")
        .bind(account).fetch_one(pool).await.unwrap()
}

async fn assert_committed(pool: &PgPool, attempt: &Attempt) {
    let value = snapshot(pool, attempt.account).await;
    assert_eq!(value["security"]["security_state"], "ACTIVE");
    assert_eq!(value["keys"].as_array().unwrap().len(), 1);
    assert_eq!(value["families"].as_array().unwrap().len(), 1);
    assert_eq!(value["families"][0]["protocol"], "ACCOUNT_V1");
    assert_eq!(value["terms"].as_array().unwrap().len(), 2);
    for item in manifest()["items"].as_array().unwrap() {
        let terms = value["terms"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["terms_kind"] == item["terms_kind"])
            .unwrap();
        assert_eq!(terms["terms_version"], MANIFEST_DIGEST);
        assert_eq!(terms["terms_release_receipt_id"], RECEIPT);
        assert_eq!(terms["terms_release_revision"], 1);
        assert_eq!(
            terms["terms_manifest_sha256"],
            format!("\\x{MANIFEST_DIGEST}")
        );
        assert_eq!(
            terms["content_sha256"],
            format!("\\x{}", item["content_sha256"].as_str().unwrap())
        );
        let event = value["events"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["id"] == terms["security_event_id"])
            .unwrap();
        assert_eq!(event["account_id"], attempt.account.to_string());
        assert_eq!(event["kind"], "TERMS_ACCEPTED");
        let expected = json!({"kind":"ACCOUNT_TERMS_RELEASE","receipt_id":RECEIPT,"revision":"1","manifest_sha256":MANIFEST_DIGEST});
        assert_eq!(event["evidence_ref"], expected);
        assert_eq!(event["payload"]["evidence"], expected);
        assert!(terms["accepted_at"].as_str().is_some());
    }
    let consumed: bool = sqlx::query_scalar(
        "SELECT consumed_at IS NOT NULL FROM auth_webauthn_ceremonies WHERE id=$1 AND user_id=$2",
    )
    .bind(attempt.ceremony)
    .bind(attempt.account)
    .fetch_one(pool)
    .await
    .unwrap();
    assert!(consumed);
    let invented: i64 = sqlx::query_scalar("SELECT (SELECT count(*) FROM users WHERE id=$1) + (SELECT count(*) FROM company_actors WHERE account_id=$1)")
        .bind(attempt.account).fetch_one(pool).await.unwrap();
    assert_eq!(invented, 0, "Account enrollment invented Company identity");
}

async fn proof(router: &axum::Router, cookies: &Cookies) -> String {
    let response = request(
        router,
        "GET",
        "/api/v2/auth/csrf",
        cookies,
        None,
        &[("X-Console-CSRF", "fetch")],
    )
    .await;
    let value = response.json(StatusCode::OK);
    response.private();
    exact_keys(&value, &["csrf_proof", "expires_at"]);
    assert!(!response.headers.contains_key(header::SET_COOKIE));
    value["csrf_proof"].as_str().unwrap().to_owned()
}

#[sqlx::test(migrations = false)]
async fn public_terms_without_authority_returns_503(pool: PgPool) {
    prepare_http_database(&pool).await;
    let router = router(&pool, artifact_root()).await;
    request(
        &router,
        "GET",
        "/api/v2/auth/terms",
        &Cookies::default(),
        None,
        &[],
    )
    .await
    .error(StatusCode::SERVICE_UNAVAILABLE, "authority_unavailable");
}

#[sqlx::test(migrations = false)]
async fn company_free_registration_me_and_empty_contexts(pool: PgPool) {
    let router = fixture(&pool).await;
    let before: (i64, i64, i64) = sqlx::query_as("SELECT (SELECT count(*) FROM organizations),(SELECT count(*) FROM employees),(SELECT count(*) FROM persons)").fetch_one(&pool).await.unwrap();
    let terms = request(
        &router,
        "GET",
        "/api/v2/auth/terms",
        &Cookies::default(),
        None,
        &[],
    )
    .await;
    let value = terms.json(StatusCode::OK);
    terms.private();
    exact_keys(&value, &["terms_version", "terms_revision", "manifest_url"]);
    assert_eq!(value["terms_version"], MANIFEST_DIGEST);
    assert_eq!(value["terms_revision"], "1");
    assert_eq!(
        value["manifest_url"],
        format!("/api/v2/auth/terms/manifests/{MANIFEST_DIGEST}")
    );
    let index: Value = serde_json::from_slice(
        &std::fs::read(artifact_root().join("fixtures/artifact-index.json")).unwrap(),
    )
    .unwrap();
    let mut artifacts = vec![index["manifest"].clone()];
    artifacts.extend(index["content"].as_array().unwrap().clone());
    for artifact in artifacts {
        let digest = artifact["sha256"].as_str().unwrap();
        let is_manifest = digest == MANIFEST_DIGEST;
        let path = format!(
            "/api/v2/auth/terms/{}/{digest}",
            if is_manifest { "manifests" } else { "content" }
        );
        let response = request(&router, "GET", &path, &Cookies::default(), None, &[]).await;
        assert_eq!(response.status, StatusCode::OK);
        let expected =
            std::fs::read(artifact_root().join(artifact["path"].as_str().unwrap())).unwrap();
        assert!(
            response.bytes == expected,
            "configured release artifact bytes differ"
        );
        assert_eq!(hex::encode(Sha256::digest(&response.bytes)), digest);
        assert_eq!(
            response.headers.get(header::CONTENT_TYPE).unwrap(),
            if is_manifest {
                "application/json; charset=utf-8"
            } else {
                "text/plain; charset=utf-8"
            }
        );
        if !is_manifest {
            assert_eq!(
                response.headers.get("x-content-type-options").unwrap(),
                "nosniff"
            );
        }
    }
    let (attempt, cookies) = enrolled(&router).await;
    assert_committed(&pool, &attempt).await;
    let me = request(&router, "GET", "/api/v2/accounts/me", &cookies, None, &[]).await;
    projection(&me.json(StatusCode::OK), attempt.account);
    me.private();
    let contexts = request(
        &router,
        "GET",
        "/api/v2/accounts/me/contexts",
        &cookies,
        None,
        &[],
    )
    .await;
    assert_eq!(contexts.json(StatusCode::OK), json!({"contexts":[]}));
    contexts.private();
    let after: (i64, i64, i64) = sqlx::query_as("SELECT (SELECT count(*) FROM organizations),(SELECT count(*) FROM employees),(SELECT count(*) FROM persons)").fetch_one(&pool).await.unwrap();
    assert_eq!(before, after);
}

#[sqlx::test(migrations = false)]
async fn acknowledgment_errors_leave_valid_attempt_completable(pool: PgPool) {
    let router = fixture(&pool).await;
    let attempt = start(&router).await;
    let before = snapshot(&pool, attempt.account).await;
    let accepted = attempt.finish["accept_items"].as_array().unwrap();
    let variants = [
        json!([]),
        json!([accepted[0]]),
        json!([accepted[0], accepted[0]]),
        json!([{"terms_kind":"unknown","accepted":true}]),
        json!([{"terms_kind":accepted[0]["terms_kind"],"accepted":false},accepted[1]]),
    ];
    for items in variants {
        let mut body = attempt.finish.clone();
        body["accept_items"] = items;
        request(
            &router,
            "POST",
            "/api/v2/auth/registration/finish",
            &attempt.cookies,
            Some(body),
            &[],
        )
        .await
        .error(StatusCode::UNPROCESSABLE_ENTITY, "invalid_request");
        assert!(
            snapshot(&pool, attempt.account).await == before,
            "invalid item acknowledgment partially committed"
        );
    }
    let response = finish(&router, &attempt).await;
    session(
        &response,
        StatusCode::CREATED,
        attempt.account,
        &mut attempt.cookies.clone(),
    );
    assert_committed(&pool, &attempt).await;
}

#[sqlx::test(migrations = false)]
async fn binding_errors_do_not_consume_valid_attempt(pool: PgPool) {
    let router = fixture(&pool).await;
    let attempt = start(&router).await;
    let other = start(&router).await;
    let before = snapshot(&pool, attempt.account).await;
    for cookies in [&Cookies::default(), &other.cookies] {
        request(
            &router,
            "POST",
            "/api/v2/auth/registration/finish",
            cookies,
            Some(attempt.finish.clone()),
            &[],
        )
        .await
        .error(StatusCode::UNAUTHORIZED, "enrollment_invalid");
        assert!(snapshot(&pool, attempt.account).await == before);
    }
    // A duplicate Origin is invalid too; request() already supplies the valid one.
    request(
        &router,
        "POST",
        "/api/v2/auth/registration/finish",
        &attempt.cookies,
        Some(attempt.finish.clone()),
        &[("Origin", "https://attacker.invalid")],
    )
    .await
    .error(StatusCode::FORBIDDEN, "request_origin_denied");
    let mut tampered = attempt.finish.clone();
    tampered["credential"]["response"]["clientDataJSON"] =
        other.finish["credential"]["response"]["clientDataJSON"].clone();
    request(
        &router,
        "POST",
        "/api/v2/auth/registration/finish",
        &attempt.cookies,
        Some(tampered),
        &[],
    )
    .await
    .error(StatusCode::UNAUTHORIZED, "enrollment_invalid");
    assert!(snapshot(&pool, attempt.account).await == before);
    session(
        &finish(&router, &attempt).await,
        StatusCode::CREATED,
        attempt.account,
        &mut attempt.cookies.clone(),
    );
    assert_committed(&pool, &attempt).await;
}

#[sqlx::test(migrations = false)]
async fn concurrent_same_attempt_commits_once_and_replay_cannot_remint(pool: PgPool) {
    let router = fixture(&pool).await;
    let attempt = start(&router).await;
    // Credential exists at the client but no finish was delivered: safe retry.
    let (left, right) = tokio::join!(finish(&router, &attempt), finish(&router, &attempt));
    let (success, refusal) = if left.status == StatusCode::CREATED {
        (left, right)
    } else {
        (right, left)
    };
    session(
        &success,
        StatusCode::CREATED,
        attempt.account,
        &mut attempt.cookies.clone(),
    );
    refusal.error(StatusCode::UNAUTHORIZED, "enrollment_invalid");
    assert_committed(&pool, &attempt).await;
    let before = snapshot(&pool, attempt.account).await;
    finish(&router, &attempt)
        .await
        .error(StatusCode::UNAUTHORIZED, "enrollment_invalid");
    assert!(
        snapshot(&pool, attempt.account).await == before,
        "replay changed committed evidence/session"
    );
}

async fn login_attempt(router: &axum::Router, attempt: &mut Attempt) -> (Cookies, Value) {
    let response = request(
        router,
        "POST",
        "/api/v2/auth/passkey/login/start",
        &Cookies::default(),
        Some(json!({})),
        &[],
    )
    .await;
    let value = response.json(StatusCode::OK);
    response.private();
    exact_keys(&value, &["ceremony_id", "public_key_options"]);
    assert_eq!(value["public_key_options"]["mediation"], "conditional");
    assert_eq!(
        value["public_key_options"]["publicKey"]["allowCredentials"],
        json!([])
    );
    let challenge: RequestChallengeResponse =
        serde_json::from_value(value["public_key_options"].clone()).unwrap();
    // CLIENT-ONLY SoftPasskey accommodations: known allow-list + unsigned handle.
    // Actual signature/server ceremony are unchanged; no discovery proof claimed.
    let challenge = inject_allow_credential(
        challenge,
        attempt.finish["credential"]["id"].as_str().unwrap(),
    );
    let assertion = attempt
        .authenticator
        .do_authentication(Url::parse(TEST_ORIGIN).unwrap(), challenge)
        .unwrap();
    let mut assertion = serde_json::to_value(assertion).unwrap();
    assertion["response"]["userHandle"] = json!(URL_SAFE_NO_PAD.encode(attempt.account.as_bytes()));
    let mut cookies = Cookies::default();
    cookies.absorb(&response.headers);
    assert!(cookies.0.len() == 1 && cookies.0.contains_key(LOGIN));
    (
        cookies,
        json!({"ceremony_id":value["ceremony_id"],"assertion":assertion}),
    )
}

#[sqlx::test(migrations = false)]
async fn lost_finish_response_recovers_same_account_and_rejects_swapped_handle(pool: PgPool) {
    let router = fixture(&pool).await;
    let mut attempt = start(&router).await;
    let committed = finish(&router, &attempt).await;
    assert_eq!(committed.status, StatusCode::CREATED);
    drop(committed); // Intentionally lose all session cookies and response bytes.
    let (cookies, input) = login_attempt(&router, &mut attempt).await;
    let other = start(&router).await;
    let before = snapshot(&pool, attempt.account).await;
    let mut swapped = input.clone();
    swapped["assertion"]["response"]["userHandle"] =
        json!(URL_SAFE_NO_PAD.encode(other.account.as_bytes()));
    request(
        &router,
        "POST",
        "/api/v2/auth/passkey/login/finish",
        &cookies,
        Some(swapped),
        &[],
    )
    .await
    .error(StatusCode::UNAUTHORIZED, "authentication_invalid");
    assert!(
        snapshot(&pool, attempt.account).await == before,
        "unsigned handle influenced credential owner"
    );
    let response = request(
        &router,
        "POST",
        "/api/v2/auth/passkey/login/finish",
        &cookies,
        Some(input),
        &[],
    )
    .await;
    let mut restored = cookies.clone();
    session(&response, StatusCode::OK, attempt.account, &mut restored);
    projection(
        &request(&router, "GET", "/api/v2/accounts/me", &restored, None, &[])
            .await
            .json(StatusCode::OK),
        attempt.account,
    );
    let keys: i64 =
        sqlx::query_scalar("SELECT count(*) FROM auth_webauthn_credentials WHERE user_id=$1")
            .bind(attempt.account)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(keys, 1);
}

#[sqlx::test(migrations = false)]
async fn csrf_refresh_and_logout_preserve_family_then_revoke(pool: PgPool) {
    let router = fixture(&pool).await;
    let (attempt, mut cookies) = enrolled(&router).await;
    let before = snapshot(&pool, attempt.account).await;
    let proof1 = proof(&router, &cookies).await;
    let proof2 = proof(&router, &cookies).await;
    assert!(
        snapshot(&pool, attempt.account).await == before,
        "proof GET mutated security state"
    );
    request(
        &router,
        "POST",
        "/api/v2/auth/logout",
        &cookies,
        Some(json!({})),
        &[],
    )
    .await
    .error(StatusCode::FORBIDDEN, "csrf_invalid");
    assert!(snapshot(&pool, attempt.account).await == before);
    let old_refresh = cookies.0[REFRESH].clone();
    let refreshed = request(
        &router,
        "POST",
        "/api/v2/auth/token/refresh",
        &cookies,
        Some(json!({})),
        &[("X-Console-CSRF", &proof1)],
    )
    .await;
    session(&refreshed, StatusCode::OK, attempt.account, &mut cookies);
    assert!(
        cookies.0[REFRESH] != old_refresh,
        "refresh cookie did not rotate"
    );
    let after = snapshot(&pool, attempt.account).await;
    assert!(
        before["families"] == after["families"],
        "refresh extended or replaced primary family"
    );
    let response = request(
        &router,
        "POST",
        "/api/v2/auth/logout",
        &cookies,
        Some(json!({})),
        &[("X-Console-CSRF", &proof2)],
    )
    .await;
    assert_eq!(
        response.json(StatusCode::OK),
        json!({"outcome":"COMMITTED"})
    );
    response.private();
    let revoked_cookies = cookies.clone();
    cookies.absorb(&response.headers);
    assert!(cookies.0.is_empty());
    request(
        &router,
        "GET",
        "/api/v2/accounts/me",
        &revoked_cookies,
        None,
        &[],
    )
    .await
    .error(StatusCode::UNAUTHORIZED, "authentication_invalid");
    let revoked = snapshot(&pool, attempt.account).await;
    assert!(!revoked["families"][0]["revoked_at"].is_null());
    request(
        &router,
        "POST",
        "/api/v2/auth/logout",
        &revoked_cookies,
        Some(json!({})),
        &[("X-Console-CSRF", &proof2)],
    )
    .await
    .error(StatusCode::UNAUTHORIZED, "authentication_invalid");
    assert!(snapshot(&pool, attempt.account).await == revoked);
}

#[sqlx::test(migrations = false)]
async fn csrf_refresh_only_is_read_only_and_cross_account_proof_is_denied(pool: PgPool) {
    let router = fixture(&pool).await;
    let (a, mut cookies) = enrolled(&router).await;
    let (b, other) = enrolled(&router).await;
    cookies.0.remove(ACCESS);
    let before_a = snapshot(&pool, a.account).await;
    let before_b = snapshot(&pool, b.account).await;
    let own = proof(&router, &cookies).await;
    let foreign = proof(&router, &other).await;
    request(&router, "GET", "/api/v2/accounts/me", &cookies, None, &[])
        .await
        .error(StatusCode::UNAUTHORIZED, "authentication_invalid");
    request(
        &router,
        "POST",
        "/api/v2/auth/token/refresh",
        &cookies,
        Some(json!({})),
        &[("X-Console-CSRF", &foreign)],
    )
    .await
    .error(StatusCode::FORBIDDEN, "csrf_invalid");
    assert!(
        snapshot(&pool, a.account).await == before_a
            && snapshot(&pool, b.account).await == before_b
    );
    let response = request(
        &router,
        "POST",
        "/api/v2/auth/token/refresh",
        &cookies,
        Some(json!({})),
        &[("X-Console-CSRF", &own)],
    )
    .await;
    session(&response, StatusCode::OK, a.account, &mut cookies);
}

#[sqlx::test(migrations = false)]
async fn browser_rejects_duplicate_cookie_bearer_and_proof_as_access(pool: PgPool) {
    let router = fixture(&pool).await;
    let (attempt, cookies) = enrolled(&router).await;
    let before = snapshot(&pool, attempt.account).await;
    let csrf = proof(&router, &cookies).await;
    request(
        &router,
        "GET",
        "/api/v2/accounts/me",
        &cookies,
        None,
        &[("Cookie", &format!("{ACCESS}={}", cookies.0[ACCESS]))],
    )
    .await
    .error(StatusCode::BAD_REQUEST, "ambiguous_credentials");
    request(
        &router,
        "GET",
        "/api/v2/accounts/me",
        &cookies,
        None,
        &[("Authorization", &format!("Bearer {}", cookies.0[ACCESS]))],
    )
    .await
    .error(StatusCode::BAD_REQUEST, "ambiguous_credentials");
    let mut confused = cookies.clone();
    confused.0.insert(ACCESS.to_owned(), csrf);
    request(&router, "GET", "/api/v2/accounts/me", &confused, None, &[])
        .await
        .error(StatusCode::UNAUTHORIZED, "authentication_invalid");
    request(
        &router,
        "GET",
        "/api/v2/auth/csrf",
        &confused,
        None,
        &[("X-Console-CSRF", "fetch")],
    )
    .await
    .error(StatusCode::UNAUTHORIZED, "authentication_invalid");
    request(
        &router,
        "POST",
        "/api/v2/auth/logout",
        &cookies,
        Some(json!({})),
        &[("X-Console-CSRF", &cookies.0[ACCESS])],
    )
    .await
    .error(StatusCode::FORBIDDEN, "csrf_invalid");
    assert!(snapshot(&pool, attempt.account).await == before);
    projection(
        &request(&router, "GET", "/api/v2/accounts/me", &cookies, None, &[])
            .await
            .json(StatusCode::OK),
        attempt.account,
    );
}

#[sqlx::test(migrations = false)]
async fn configured_missing_artifact_fails_closed(pool: PgPool) {
    prepare_http_database(&pool).await;
    seed_terms(&pool).await;
    let missing = artifact_root().join(format!("missing-{}", Uuid::new_v4()));
    assert!(!missing.exists());
    let before: i64 = sqlx::query_scalar("SELECT count(*) FROM accounts")
        .fetch_one(&pool)
        .await
        .unwrap();
    let router = router(&pool, missing).await;
    request(
        &router,
        "GET",
        "/api/v2/auth/terms",
        &Cookies::default(),
        None,
        &[],
    )
    .await
    .error(StatusCode::SERVICE_UNAVAILABLE, "authority_unavailable");
    request(
        &router,
        "POST",
        "/api/v2/auth/registration/start",
        &Cookies::default(),
        Some(json!({"terms_version":MANIFEST_DIGEST})),
        &[],
    )
    .await
    .error(StatusCode::SERVICE_UNAVAILABLE, "authority_unavailable");
    let accounts: i64 = sqlx::query_scalar("SELECT count(*) FROM accounts")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(accounts, before, "artifact failure created an Account");
}

#[test]
fn fixture_hashes_are_exact_and_test_only() {
    let bytes = std::fs::read(artifact_root().join("fixtures/manifest.json")).unwrap();
    assert_eq!(hex::encode(Sha256::digest(&bytes)), MANIFEST_DIGEST);
    assert_eq!(manifest()["fixture_only"], true);
    for (file, item) in [("service.txt", 0), ("privacy.txt", 1)] {
        let bytes = std::fs::read(artifact_root().join("fixtures").join(file)).unwrap();
        assert_eq!(
            hex::encode(Sha256::digest(&bytes)),
            manifest()["items"][item]["content_sha256"]
        );
    }
}

#[test]
fn strict_projection_oracle_refuses_wrong_account_and_hidden_fields() {
    let account = Uuid::new_v4();
    let expiry = (OffsetDateTime::now_utc() + Duration::minutes(5))
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap();
    let valid = json!({"account_id":account,"session":{"assurance":"PASSKEY_PRIMARY","expires_at":expiry},"permitted_self_actions":[{"action_key":"account.session.logout","registration_revision":"1"}]});
    projection(&valid, account);
    assert!(std::panic::catch_unwind(|| projection(&valid, Uuid::new_v4())).is_err());
    let mut leaked = valid.clone();
    leaked["access_token"] = json!("TEST_ONLY_TOKEN_CANARY");
    assert!(std::panic::catch_unwind(|| projection(&leaked, account)).is_err());
    leaked = valid;
    leaked["session"]["org_id"] = json!(Uuid::new_v4());
    assert!(std::panic::catch_unwind(|| projection(&leaked, account)).is_err());
}
