//! Mounted self-passkey reader: separate exact-family and response-privacy leaves.
//! Bound credentials exist only in this fixture; production mint remains unbound.
use super::*;
use account_browser::deployment_operator_designation::company_setup::all_rows;
use account_fence_transport::{issue_family_fence::assert_issue_delta, refresh_complete_snapshot};
use console_platform_auth::{JwtSettings, JwtVerifier, RefreshTokenIssue, RefreshTokenStore};
use console_platform_test_support::login_test_pool;
use std::collections::{BTreeMap, BTreeSet};

const PATH: &str = "/api/v1/auth/passkeys";
type Rows = BTreeMap<String, String>;

struct Fixture {
    legacy: LegacyFenceFixture,
    auth: PgPool,
    a: RefreshTokenIssue,
    b: RefreshTokenIssue,
    access_a: String,
    access_b: String,
    expected: Value,
    control_expected: Value,
    credential_secrets: Vec<String>,
}

async fn db_now(pool: &PgPool) -> OffsetDateTime {
    sqlx::query_scalar("SELECT pg_catalog.clock_timestamp()")
        .fetch_one(pool)
        .await
        .unwrap()
}

fn unchanged_except(before: &Rows, after: &Rows, allowed: &[&str]) {
    assert!(before.keys().eq(after.keys()), "table census changed");
    for (table, rows) in before {
        if !allowed.contains(&table.as_str()) {
            assert!(
                after.get(table) == Some(rows),
                "unexpected table effect: {table}"
            );
        }
    }
}

async fn expected_summary(pool: &PgPool, subject: UserId) -> Value {
    let rows: Vec<(Uuid, OffsetDateTime, Option<OffsetDateTime>)> = sqlx::query_as(
        "SELECT id,created_at,last_used_at FROM public.auth_webauthn_credentials WHERE user_id=$1 ORDER BY created_at DESC",
    )
    .bind(subject.as_uuid())
    .fetch_all(pool)
    .await
    .unwrap();
    assert_eq!(rows.len(), 1, "real nonempty enrollment prerequisite");
    Value::Array(rows.into_iter().map(|(id, created, used)| json!({
        "id": id,
        "created_at": created.format(&time::format_description::well_known::Rfc3339).unwrap(),
        "last_used_at": used.map(|t| t.format(&time::format_description::well_known::Rfc3339).unwrap()),
    })).collect())
}

fn bound_access(
    legacy: &LegacyFenceFixture,
    issue: &RefreshTokenIssue,
    now: OffsetDateTime,
) -> String {
    let public = legacy
        .signing_key
        .verifying_key()
        .to_public_key_pem(LineEnding::LF)
        .unwrap();
    let verifier = JwtVerifier::from_es256_public_pem(
        JwtSettings {
            issuer: TEST_ISSUER.to_owned(),
            audience: TEST_AUDIENCE.to_owned(),
            access_token_ttl: Duration::minutes(15),
        },
        public.as_bytes(),
    )
    .unwrap();
    let mut claims =
        serde_json::to_value(verifier.verify_access_token(&legacy.access).unwrap()).unwrap();
    assert!(
        claims.get("legacy_session").is_none(),
        "production issuance must remain unbound"
    );
    assert!(claims["sub"] == json!(issue.user_id.to_string()));
    assert!(claims["org"] == json!(issue.org_id.to_string()));
    // Fixture-only lease starts at genuine family issuance. Keep the original
    // verified token's expiration, issuer, audience, subject and role claims.
    claims["iat"] = json!(now.unix_timestamp());
    claims["nbf"] = json!(now.unix_timestamp());
    assert!(claims["exp"].as_i64().unwrap() > now.unix_timestamp());
    claims["legacy_session"] = json!({"version":1,"family_id":issue.family_id,
        "home_org":issue.org_id,"kind":"direct"});
    let private = legacy.signing_key.to_pkcs8_pem(LineEnding::LF).unwrap();
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::ES256),
        &claims,
        &jsonwebtoken::EncodingKey::from_ec_pem(private.as_bytes()).unwrap(),
    )
    .unwrap();
    let verified = verifier.verify_access_token(&token).unwrap();
    assert!(verified.legacy_session.unwrap().family_id == issue.family_id);
    token
}

async fn issue(
    pool: &PgPool,
    auth: &PgPool,
    subject: UserId,
) -> (RefreshTokenIssue, OffsetDateTime) {
    let before_all = all_rows(pool).await;
    let before = refresh_complete_snapshot(pool).await;
    let now = db_now(auth).await;
    let ttl = Duration::hours(1);
    let mut tx = auth.begin().await.unwrap();
    let issued = RefreshTokenStore
        .issue_family_in_tx(&mut tx, *subject.as_uuid(), OrgId::knl(), now, ttl)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let after = refresh_complete_snapshot(pool).await;
    assert_issue_delta(pool, &before, &after, &issued, subject, now, ttl).await;
    unchanged_except(
        &before_all,
        &all_rows(pool).await,
        &[
            "auth_refresh_token_families",
            "auth_refresh_tokens",
            "audit_events",
        ],
    );
    let exact: bool = sqlx::query_scalar("SELECT id=$1 AND user_id=$2 AND org_id=$3 AND protocol='LEGACY_COMPANY' AND created_at=$4 AND revoked_at IS NULL AND revoked_reason IS NULL AND account_security_generation IS NULL AND auth_time IS NULL AND assurance IS NULL FROM public.auth_refresh_token_families WHERE id=$1")
        .bind(issued.family_id).bind(subject.as_uuid()).bind(OrgId::knl().as_uuid()).bind(now)
        .fetch_one(pool).await.unwrap();
    assert!(
        exact,
        "genuine returned family must have exact legacy tuple"
    );
    (issued, now)
}

async fn fixture(pool: &PgPool) -> Fixture {
    let legacy = legacy_fence_fixture(pool).await;
    let auth = login_test_pool(pool, TestDatabaseLogin::Auth).await;
    let business = login_test_pool(pool, TestDatabaseLogin::Business).await;
    for (connection, role) in [(&auth, "console_auth_rt"), (&business, "console_rt")] {
        let identity: (String, String) =
            sqlx::query_as("SELECT session_user::text,current_user::text")
                .fetch_one(connection)
                .await
                .unwrap();
        assert!(
            identity == (role.to_owned(), role.to_owned()),
            "restricted runtime prerequisite"
        );
    }
    business.close().await;
    // Another genuine subject has a different nonempty credential projection.
    let mut control_authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
    enroll_passkey(
        &legacy.router,
        &mut control_authenticator,
        &legacy.control_access,
    )
    .await;
    let expected = expected_summary(pool, legacy.subject).await;
    let control_expected = expected_summary(pool, legacy.control).await;
    assert!(
        expected != control_expected,
        "subject isolation positive control"
    );
    let mut credential_secrets = Vec::new();
    for subject in [legacy.subject, legacy.control] {
        for (_, credential_id, passkey_json) in fence_credential_snapshot(pool, subject).await {
            credential_secrets.push(credential_id);
            credential_secrets.push(
                serde_json::to_string(&serde_json::from_str::<Value>(&passkey_json).unwrap())
                    .unwrap(),
            );
            credential_secrets.push(passkey_json);
        }
    }
    assert!(credential_secrets.len() >= 6 && credential_secrets.iter().all(|v| !v.is_empty()));
    let (a, a_now) = issue(pool, &auth, legacy.subject).await;
    let (b, b_now) = issue(pool, &auth, legacy.subject).await;
    assert!(a.family_id != b.family_id && a.token_id != b.token_id);
    let access_a = bound_access(&legacy, &a, a_now);
    let access_b = bound_access(&legacy, &b, b_now);
    Fixture {
        legacy,
        auth,
        a,
        b,
        access_a,
        access_b,
        expected,
        control_expected,
        credential_secrets,
    }
}

impl Fixture {
    fn secrets(&self) -> Vec<String> {
        let mut secrets = self.credential_secrets.clone();
        secrets.extend([
            self.access_a.clone(),
            self.access_b.clone(),
            self.legacy.access.clone(),
            self.legacy.control_access.clone(),
            self.legacy.body_refresh.clone(),
            self.legacy.cookie_refresh.clone(),
            self.a.token.as_str().to_owned(),
            self.b.token.as_str().to_owned(),
            self.a.family_id.to_string(),
            self.b.family_id.to_string(),
        ]);
        secrets
    }
}

fn no_disclosure(headers: &http::HeaderMap, body: &[u8], secrets: &[String]) -> bool {
    !headers.contains_key(header::SET_COOKIE)
        && secrets.iter().all(|secret| {
            !secret.is_empty()
                && !body
                    .windows(secret.len())
                    .any(|bytes| bytes == secret.as_bytes())
                && headers.values().all(|value| {
                    !value
                        .as_bytes()
                        .windows(secret.len())
                        .any(|bytes| bytes == secret.as_bytes())
                })
        })
}

fn private_headers(headers: &http::HeaderMap) -> bool {
    for (name, expected) in [
        (header::CACHE_CONTROL, "no-store"),
        (header::PRAGMA, "no-cache"),
        (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
    ] {
        let values: Vec<_> = headers.get_all(name).iter().collect();
        if values.len() != 1 || values[0].as_bytes() != expected.as_bytes() {
            return false;
        }
    }
    let vary: Vec<_> = headers.get_all(header::VARY).iter().collect();
    if vary.len() != 1 {
        return false;
    }
    let Ok(vary) = vary[0].to_str() else {
        return false;
    };
    let names: Vec<_> = vary
        .split(',')
        .map(|s| s.trim().to_ascii_lowercase())
        .collect();
    names.len() == 3
        && names.into_iter().collect::<BTreeSet<_>>()
            == BTreeSet::from([
                "authorization".to_owned(),
                "cookie".to_owned(),
                "origin".to_owned(),
            ])
}

async fn response_json(
    response: http::Response<Body>,
    expected_status: StatusCode,
    secrets: &[String],
    require_private: bool,
) -> Value {
    assert_eq!(
        response.status(),
        expected_status,
        "SELF_PASSKEY_OWNER: wrong status"
    );
    if require_private {
        assert!(
            private_headers(response.headers()),
            "SELF_PASSKEY_PRIVACY: exact local private headers required"
        );
    }
    assert!(
        response
            .headers()
            .get(header::CONTENT_TYPE)
            .unwrap()
            .as_bytes()
            .starts_with(b"application/json")
    );
    let (parts, body) = response.into_parts();
    let bytes = to_bytes(body, 64 * 1024).await.unwrap();
    assert!(
        no_disclosure(&parts.headers, &bytes, secrets),
        "credential echo or Set-Cookie"
    );
    serde_json::from_slice(&bytes).expect("bounded JSON contract")
}

fn error_body(code: &str, message: &str) -> Value {
    json!({"error":{"code":code,"message":message}})
}

async fn read_matches(f: &Fixture, token: &str, expected: &Value) {
    let body = response_json(
        get_legacy_raw(&f.legacy.router, PATH, token).await,
        StatusCode::OK,
        &f.secrets(),
        false,
    )
    .await;
    assert!(&body == expected, "exact nonempty summary only");
}

fn logout_audit_matches(
    audit: &Value,
    subject: UserId,
    family: Uuid,
    now: OffsetDateTime,
    completed: OffsetDateTime,
) -> bool {
    let Some(object) = audit.as_object() else {
        return false;
    };
    let keys = [
        "id",
        "actor",
        "action",
        "target_type",
        "target_id",
        "branch_id",
        "before_snap",
        "after_snap",
        "trace_id",
        "span_id",
        "occurred_at",
        "created_at",
        "org_id",
        "ip",
        "user_agent",
        "auth_method",
        "device",
        "classification_badges",
        "anomaly",
        "reason",
    ];
    if object.keys().map(String::as_str).collect::<BTreeSet<_>>() != BTreeSet::from(keys) {
        return false;
    }
    if !audit["id"]
        .as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .is_some_and(|id| !id.is_nil() && id.get_version_num() == 4)
    {
        return false;
    }
    for (field, length) in [("trace_id", 32), ("span_id", 16)] {
        if !audit[field].as_str().is_some_and(|s| {
            s.len() == length
                && s.bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                && s.bytes().any(|b| b != b'0')
        }) {
            return false;
        }
    }
    let timestamp = |field: &str| {
        audit[field].as_str().and_then(|s| {
            OffsetDateTime::parse(s, &time::format_description::well_known::Rfc3339).ok()
        })
    };
    audit["actor"] == json!(subject)
        && audit["org_id"] == json!(OrgId::knl())
        && audit["action"] == "auth.logout"
        && audit["target_type"] == "auth_refresh_token_family"
        && audit["target_id"] == json!(family)
        && audit["after_snap"] == json!({"family_id":family,"revoked_reason":"logout"})
        && [
            "branch_id",
            "before_snap",
            "ip",
            "user_agent",
            "auth_method",
            "device",
            "classification_badges",
            "anomaly",
            "reason",
        ]
        .iter()
        .all(|key| audit[*key].is_null())
        && timestamp("occurred_at") == Some(now)
        && timestamp("created_at").is_some_and(|created| now <= created && created <= completed)
}

async fn logout_a(pool: &PgPool, f: &Fixture) {
    let before_all = all_rows(pool).await;
    let before = refresh_complete_snapshot(pool).await;
    let now = db_now(&f.auth).await;
    RefreshTokenStore
        .revoke_family_for_logout(&f.auth, f.a.token.as_str(), now)
        .await
        .unwrap();
    let after = refresh_complete_snapshot(pool).await;
    unchanged_except(
        &before_all,
        &all_rows(pool).await,
        &[
            "auth_refresh_token_families",
            "auth_refresh_tokens",
            "audit_events",
        ],
    );
    for (roster, id, fields) in [
        (
            "families",
            f.a.family_id,
            &["revoked_at", "revoked_reason"][..],
        ),
        ("tokens", f.a.token_id, &["revoked_at"][..]),
    ] {
        let old = before[roster].as_array().unwrap();
        let new = after[roster].as_array().unwrap();
        assert_eq!(old.len(), new.len());
        for row in old {
            let actual = new.iter().find(|r| r["id"] == row["id"]).unwrap();
            if row["id"] != json!(id) {
                assert!(actual == row, "logout changed unrelated row");
                continue;
            }
            let mut expected = row.clone();
            for field in fields {
                assert!(row[*field].is_null());
                assert!(!actual[*field].is_null());
                expected[*field] = actual[*field].clone();
            }
            assert!(&expected == actual, "logout changed nonterminal fields");
        }
    }
    let exact: bool = sqlx::query_scalar("SELECT a.revoked_at=$3 AND a.revoked_reason='logout' AND t.revoked_at=$3 AND b.revoked_at IS NULL AND b.revoked_reason IS NULL FROM public.auth_refresh_token_families a JOIN public.auth_refresh_tokens t ON t.family_id=a.id JOIN public.auth_refresh_token_families b ON b.id=$2 WHERE a.id=$1 AND t.id=$4")
        .bind(f.a.family_id).bind(f.b.family_id).bind(now).bind(f.a.token_id)
        .fetch_one(pool).await.unwrap();
    assert!(
        exact,
        "A revoked at exact owner time and B live prerequisite"
    );
    let old_audits = before["audit"].as_array().unwrap();
    let new_audits = after["audit"].as_array().unwrap();
    assert_eq!(new_audits.len(), old_audits.len() + 1);
    assert!(
        old_audits.iter().all(|row| new_audits.contains(row)),
        "retained audit changed"
    );
    let added: Vec<_> = new_audits
        .iter()
        .filter(|row| !old_audits.contains(row))
        .collect();
    assert_eq!(added.len(), 1);
    let audit = added[0];
    assert!(
        logout_audit_matches(
            audit,
            f.legacy.subject,
            f.a.family_id,
            now,
            db_now(pool).await
        ),
        "complete logout audit shape, attribution, metadata, trace and timestamps"
    );
}

#[sqlx::test(migrations = false)]
async fn revoked_exact_family_cannot_read_or_borrow_live_sibling(pool: PgPool) {
    let f = fixture(&pool).await;
    let before = all_rows(&pool).await;
    read_matches(&f, &f.access_a, &f.expected).await;
    read_matches(&f, &f.access_b, &f.expected).await;
    read_matches(&f, &f.legacy.control_access, &f.control_expected).await;
    assert!(
        before == all_rows(&pool).await,
        "positive reads must preserve complete state"
    );
    logout_a(&pool, &f).await;
    let revoked = all_rows(&pool).await;
    // This is the admission assertion. No cache-header assertion precedes it.
    // Current owner returns 200 despite persisted A-revoked/B-live facts.
    let denied = response_json(
        get_legacy_raw(&f.legacy.router, PATH, &f.access_a).await,
        StatusCode::UNAUTHORIZED,
        &f.secrets(),
        false,
    )
    .await;
    assert!(denied == error_body("unauthorized", "invalid bearer token"));
    read_matches(&f, &f.access_b, &f.expected).await;
    read_matches(&f, &f.legacy.control_access, &f.control_expected).await;
    assert!(
        revoked == all_rows(&pool).await,
        "all accepted/denied reads preserve complete state"
    );
    f.auth.close().await;
}

#[sqlx::test(migrations = false)]
async fn success_and_errors_have_local_private_headers(pool: PgPool) {
    let f = fixture(&pool).await;
    let before = all_rows(&pool).await;
    let body = response_json(
        get_legacy_raw(&f.legacy.router, PATH, &f.access_a).await,
        StatusCode::OK,
        &f.secrets(),
        true,
    )
    .await;
    assert!(body == f.expected);
    for (authorization, message) in [
        (None, "missing bearer token"),
        (
            Some("Basic privacy-probe"),
            "authorization header must use Bearer scheme",
        ),
        (Some("Bearer invalid-reader-token"), "invalid bearer token"),
    ] {
        let mut request = Request::builder().uri(PATH);
        let mut secrets = f.secrets();
        if let Some(value) = authorization {
            request = request.header(header::AUTHORIZATION, value);
            secrets.push(value.to_owned());
            if let Some((_, payload)) = value.split_once(' ') {
                secrets.push(payload.to_owned());
            }
        }
        let response = f
            .legacy
            .router
            .clone()
            .oneshot(request.body(Body::empty()).unwrap())
            .await
            .unwrap();
        let body = response_json(response, StatusCode::UNAUTHORIZED, &secrets, true).await;
        assert!(body == error_body("unauthorized", message));
    }
    // Explicitly configured, real restricted Auth pool closed after admission.
    // No SQL helper replacement, cluster-wide role mutation, or missing schema.
    let closed_auth = login_test_pool(&pool, TestDatabaseLogin::Auth).await;
    closed_auth.close().await;
    let key = &f.legacy.signing_key;
    let state = app_state(
        pool.clone(),
        key.to_pkcs8_pem(LineEnding::LF).unwrap().to_string(),
        key.verifying_key()
            .to_public_key_pem(LineEnding::LF)
            .unwrap(),
    )
    .await
    .unwrap()
    .with_auth_database(closed_auth);
    let outage = build_router(state);
    let body = response_json(
        get_legacy_raw(&outage, PATH, &f.access_a).await,
        StatusCode::SERVICE_UNAVAILABLE,
        &f.secrets(),
        true,
    )
    .await;
    assert!(body == error_body("service_unavailable", "session verification unavailable"));
    let mut invalid_secrets = f.secrets();
    invalid_secrets.push("invalid-reader-token".to_owned());
    let body = response_json(
        get_legacy_raw(&outage, PATH, "invalid-reader-token").await,
        StatusCode::UNAUTHORIZED,
        &invalid_secrets,
        true,
    )
    .await;
    assert!(body == error_body("unauthorized", "invalid bearer token"));
    // Original configured transport remains usable after the isolated fault.
    read_matches(&f, &f.access_b, &f.expected).await;
    assert!(
        before == all_rows(&pool).await,
        "privacy/error reads changed complete state"
    );
    f.auth.close().await;
}

#[test]
fn privacy_oracle_rejects_omitted_changed_and_duplicate_fields() {
    let mut good = http::HeaderMap::new();
    for (name, value) in [
        (header::CACHE_CONTROL, "no-store"),
        (header::PRAGMA, "no-cache"),
        (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        (header::VARY, "Authorization, Cookie, Origin"),
    ] {
        good.insert(name, http::HeaderValue::from_static(value));
    }
    assert!(private_headers(&good));
    for name in [
        header::CACHE_CONTROL,
        header::PRAGMA,
        header::X_CONTENT_TYPE_OPTIONS,
        header::VARY,
    ] {
        let mut omitted = good.clone();
        omitted.remove(&name);
        assert!(!private_headers(&omitted));
        let mut changed = good.clone();
        changed.insert(&name, http::HeaderValue::from_static("public"));
        assert!(!private_headers(&changed));
        let mut duplicate = good.clone();
        duplicate.append(&name, good[&name].clone());
        assert!(!private_headers(&duplicate));
    }
    for vary in [
        "Cookie, Origin",
        "Authorization, Cookie, Origin, *",
        "Authorization, Cookie, Cookie",
    ] {
        let mut changed = good.clone();
        changed.insert(header::VARY, http::HeaderValue::from_static(vary));
        assert!(!private_headers(&changed));
    }
    let mut reordered = good;
    reordered.insert(
        header::VARY,
        http::HeaderValue::from_static("origin, AUTHORIZATION, cookie"),
    );
    assert!(private_headers(&reordered), "structural Vary token set");
}

#[test]
fn disclosure_oracle_rejects_body_header_echo_and_cookie() {
    let headers = http::HeaderMap::new();
    let secrets = vec!["test-only-secret".to_owned()];
    assert!(no_disclosure(&headers, b"[]", &secrets));
    assert!(!no_disclosure(
        &headers,
        b"{\"error\":\"test-only-secret\"}",
        &secrets
    ));
    let mut echo = headers.clone();
    echo.insert(
        "x-debug",
        http::HeaderValue::from_static("test-only-secret"),
    );
    assert!(!no_disclosure(&echo, b"[]", &secrets));
    let mut cookie = headers;
    cookie.insert(header::SET_COOKIE, http::HeaderValue::from_static("x=y"));
    assert!(!no_disclosure(&cookie, b"[]", &secrets));
}

#[test]
fn logout_audit_oracle_rejects_missing_extra_and_corrupted_fields() {
    let subject = UserId::new();
    let family = Uuid::new_v4();
    let now = OffsetDateTime::from_unix_timestamp(1_790_000_000).unwrap();
    let trace = TraceContext::generate();
    let time = now
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap();
    let good = json!({"id":Uuid::new_v4(),"actor":subject,"action":"auth.logout",
        "target_type":"auth_refresh_token_family","target_id":family,"org_id":OrgId::knl(),
        "branch_id":null,"before_snap":null,"after_snap":{"family_id":family,"revoked_reason":"logout"},
        "trace_id":trace.trace_id(),"span_id":trace.span_id(),"occurred_at":time,"created_at":time,
        "ip":null,"user_agent":null,"auth_method":null,"device":null,"classification_badges":null,"anomaly":null,"reason":null});
    assert!(logout_audit_matches(&good, subject, family, now, now));
    for key in good.as_object().unwrap().keys() {
        let mut omitted = good.clone();
        omitted.as_object_mut().unwrap().remove(key);
        assert!(
            !logout_audit_matches(&omitted, subject, family, now, now),
            "missing audit field accepted: {key}"
        );
    }
    let mut extra = good.clone();
    extra["unexpected"] = json!(true);
    assert!(!logout_audit_matches(&extra, subject, family, now, now));
    for (key, value) in [
        ("id", json!(Uuid::nil())),
        ("actor", json!(Uuid::new_v4())),
        ("org_id", json!(Uuid::new_v4())),
        ("target_id", json!(Uuid::new_v4())),
        ("action", json!("auth.refresh.issue")),
        ("target_type", json!("user")),
        ("trace_id", json!("0".repeat(32))),
        ("span_id", json!("A".repeat(16))),
        (
            "after_snap",
            json!({"family_id":family,"revoked_reason":"reuse_detected"}),
        ),
        (
            "created_at",
            json!(
                (now + Duration::seconds(1))
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap()
            ),
        ),
        (
            "occurred_at",
            json!(
                (now - Duration::seconds(1))
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap()
            ),
        ),
    ] {
        let mut changed = good.clone();
        changed[key] = value;
        assert!(
            !logout_audit_matches(&changed, subject, family, now, now),
            "corrupt audit field accepted: {key}"
        );
    }
    for key in [
        "branch_id",
        "before_snap",
        "ip",
        "user_agent",
        "auth_method",
        "device",
        "classification_badges",
        "anomaly",
        "reason",
    ] {
        let mut changed = good.clone();
        changed[key] = json!("unexpected metadata");
        assert!(
            !logout_audit_matches(&changed, subject, family, now, now),
            "unexpected audit metadata accepted: {key}"
        );
    }
}
