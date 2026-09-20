//! Child of retained list histories. Real credentials enter the public owner directly.
//! Signing altered/bound fixture claims does not enable production bound mint.
use super::*;
use console_platform_auth::{RefreshTokenIssue, RefreshTokenStore};
use console_platform_authz::platform_policy::PlatformPolicy;
use console_platform_provisioning::{OrganizationSummary, PlatformProvisioner, ProvisioningError};

fn signed(f: &Fixture, claims: &Value) -> String {
    jsonwebtoken::encode(
        &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::ES256),
        claims,
        &jsonwebtoken::EncodingKey::from_ec_pem(
            f.key.to_pkcs8_pem(LineEnding::LF).unwrap().as_bytes(),
        )
        .unwrap(),
    )
    .unwrap()
}
fn original_claims(f: &Fixture) -> Value {
    serde_json::to_value(f.verifier.verify_access_token(&f.access).unwrap()).unwrap()
}
fn metadata(rows: &[OrganizationSummary]) -> Value {
    Value::Array(
        rows.iter()
            .map(|r| {
                json!({"id":r.id,"slug":r.slug,"name":r.name,
        "status":r.status,"group_id":r.group_id,"group_slug":r.group_slug,"group_name":r.group_name,
        "created_at":r.created_at.format(&time::format_description::well_known::Rfc3339).unwrap(),
        "updated_at":r.updated_at.format(&time::format_description::well_known::Rfc3339).unwrap()})
            })
            .collect(),
    )
}
async fn direct(
    f: &Fixture,
    token: &str,
    ttl: Duration,
) -> Result<Vec<OrganizationSummary>, ProvisioningError> {
    PlatformProvisioner::new(Duration::hours(1))
        .list_tenants(
            &f.business,
            &f.verifier,
            token,
            ttl,
            &PlatformPolicy::compile_current().unwrap(),
        )
        .await
}
async fn direct_positive(pool: &PgPool, f: &Fixture, token: &str, ttl: Duration) {
    let before = all_rows(pool).await;
    let start = db_now(pool).await;
    let result = direct(f, token, ttl).await.unwrap();
    let end = db_now(pool).await;
    assert!(
        metadata_matches(&metadata(&result), &f.expected),
        "independent exact direct-owner metadata"
    );
    assert!(
        exact_read_delta(
            &before,
            &all_rows(pool).await,
            f.actor,
            result.len(),
            start,
            end
        ),
        "direct owner appends one exact audit and changes no other state"
    );
}
async fn direct_denied(pool: &PgPool, f: &Fixture, token: &str, ttl: Duration, kind: &str) {
    let before = all_rows(pool).await;
    let result = direct(f, token, ttl).await;
    assert!(
        match kind {
            "unauthorized" => matches!(result, Err(ProvisioningError::PlatformListUnauthorized)),
            "forbidden" => matches!(result, Err(ProvisioningError::PlatformListForbidden)),
            "unavailable" => matches!(result, Err(ProvisioningError::PlatformListUnavailable)),
            _ => false,
        },
        "finite direct-owner refusal: {kind}"
    );
    assert!(
        before == all_rows(pool).await,
        "refused owner discloses no rows and has zero durable effects"
    );
}
async fn close(f: Fixture) {
    f.state.shutdown_realtime().await;
    f.business.close().await;
}

#[sqlx::test(migrations = false)]
async fn direct_list_owner_reverifies_original_credential_and_current_roles(pool: PgPool) {
    let f = fixture(&pool).await;
    direct_positive(&pool, &f, &f.access, Duration::days(30)).await;
    set_role(&pool, f.actor, "MEMBER").await;
    direct_denied(&pool, &f, &f.access, Duration::days(30), "forbidden").await;
    set_role(&pool, f.actor, "SUPER_ADMIN").await;
    direct_positive(&pool, &f, &f.access, Duration::days(30)).await;
    let claims = original_claims(&f);
    let other_key = SigningKey::random(&mut OsRng);
    let wrong_signature = jsonwebtoken::encode(
        &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::ES256),
        &claims,
        &jsonwebtoken::EncodingKey::from_ec_pem(
            other_key.to_pkcs8_pem(LineEnding::LF).unwrap().as_bytes(),
        )
        .unwrap(),
    )
    .unwrap();
    direct_denied(
        &pool,
        &f,
        &wrong_signature,
        Duration::days(30),
        "unauthorized",
    )
    .await;
    direct_denied(
        &pool,
        &f,
        "not.a.valid.jwt",
        Duration::days(30),
        "unauthorized",
    )
    .await;
    for fault in [
        "ordinary_company",
        "view_as",
        "read_only",
        "malformed_tenant_context",
        "group_context",
        "actor_home",
        "scope_level",
        "nil_subject",
        "null_binding",
        "empty_binding",
        "future_binding_version",
    ] {
        let mut bad = claims.clone();
        match fault {
            "ordinary_company" => {
                bad["org"] = json!(OrgId::knl().to_string());
                bad["platform"] = json!(false);
            }
            "view_as" => bad["view_as"] = json!(true),
            "read_only" => bad["read_only"] = json!(true),
            "malformed_tenant_context" => {
                bad["tenant_context"] =
                    json!({"target_org":OrgId::knl(),"context_id":Uuid::new_v4()})
            }
            "group_context" => bad["group_context_id"] = json!(Uuid::new_v4()),
            "actor_home" => bad["actor_home_org"] = json!(OrgId::platform()),
            "scope_level" => bad["scope_level"] = json!("GROUP"),
            "nil_subject" => bad["sub"] = json!(Uuid::nil().to_string()),
            "null_binding" => bad["legacy_session"] = Value::Null,
            "empty_binding" => bad["legacy_session"] = json!({}),
            "future_binding_version" => {
                bad["legacy_session"] = json!({"version":2,"family_id":Uuid::new_v4(),"home_org":OrgId::platform(),"kind":"direct"})
            }
            _ => unreachable!(),
        }
        direct_denied(
            &pool,
            &f,
            &signed(&f, &bad),
            Duration::days(30),
            "unauthorized",
        )
        .await;
    }
    direct_denied(&pool, &f, &f.access, Duration::ZERO, "unavailable").await;
    direct_positive(&pool, &f, &f.access, Duration::days(30)).await;
    close(f).await;
}

async fn issue_bound(
    pool: &PgPool,
    f: &Fixture,
    auth: &PgPool,
) -> (RefreshTokenIssue, String, OffsetDateTime) {
    let now = db_now(pool).await;
    let mut tx = auth.begin().await.unwrap();
    let issue = RefreshTokenStore
        .issue_family_in_tx(
            &mut tx,
            *f.actor.as_uuid(),
            OrgId::platform(),
            now,
            Duration::hours(1),
        )
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let exact:bool=sqlx::query_scalar("SELECT f.user_id=$2 AND f.org_id=$3 AND f.protocol='LEGACY_COMPANY' AND f.created_at=$4 AND f.revoked_at IS NULL AND f.account_security_generation IS NULL AND f.auth_time IS NULL AND f.assurance IS NULL AND t.id=$5 AND t.user_id=f.user_id AND t.org_id=f.org_id AND t.issued_at=$4 AND t.expires_at=$6 AND t.used_at IS NULL AND t.revoked_at IS NULL FROM public.auth_refresh_token_families f JOIN public.auth_refresh_tokens t ON t.family_id=f.id WHERE f.id=$1")
        .bind(issue.family_id).bind(f.actor.as_uuid()).bind(OrgId::platform().as_uuid()).bind(now).bind(issue.token_id).bind(now+Duration::hours(1)).fetch_one(pool).await.unwrap();
    assert!(
        exact && issue.user_id == *f.actor.as_uuid() && issue.org_id == OrgId::platform(),
        "genuine exact legacy family issuance prerequisite"
    );
    let mut claims = original_claims(f);
    assert!(
        claims.get("legacy_session").is_none(),
        "production mint remains unbound"
    );
    claims["iat"] = json!(now.unix_timestamp());
    claims["nbf"] = json!(now.unix_timestamp());
    claims["legacy_session"] = json!({"version":1,"family_id":issue.family_id,"home_org":OrgId::platform(),"kind":"direct"});
    let access = signed(f, &claims);
    let verified = f.verifier.verify_access_token(&access).unwrap();
    assert!(verified.legacy_session.unwrap().family_id == issue.family_id);
    (issue, access, now)
}
async fn http_token(pool: &PgPool, f: &Fixture, token: &str, expected: StatusCode) {
    let before = all_rows(pool).await;
    let start = db_now(pool).await;
    let response = f
        .router
        .clone()
        .oneshot(
            Request::builder()
                .uri(PATH)
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let (status, headers, body) = response_parts(response).await;
    let end = db_now(pool).await;
    let mut secrets = f.secrets.clone();
    secrets.push(token.to_owned());
    assert!(
        status == expected && private_response(&headers, &body, &secrets),
        "mounted credential status and privacy"
    );
    let value: Value = serde_json::from_slice(&body).unwrap();
    if expected == StatusCode::OK {
        assert!(metadata_matches(&value, &f.expected));
        assert!(exact_read_delta(
            &before,
            &all_rows(pool).await,
            f.actor,
            f.expected.as_array().unwrap().len(),
            start,
            end
        ));
    } else {
        assert!(value == json!({"error":{"code":"unauthorized","message":"invalid bearer token"}}));
        assert!(
            before == all_rows(pool).await,
            "HTTP refusal has no durable effects"
        );
    }
}
#[sqlx::test(migrations = false)]
async fn bound_list_keeps_live_family_on_rotation_and_refuses_exact_revoked_sibling(pool: PgPool) {
    let f = fixture(&pool).await;
    let auth = login_test_pool(&pool, TestDatabaseLogin::Auth).await;
    let (a, access_a, _) = issue_bound(&pool, &f, &auth).await;
    let (b, access_b, _) = issue_bound(&pool, &f, &auth).await;
    assert!(a.family_id != b.family_id);
    direct_positive(&pool, &f, &access_a, Duration::days(30)).await;
    http_token(&pool, &f, &access_a, StatusCode::OK).await;
    let rotated = RefreshTokenStore
        .rotate(
            &f.business,
            &auth,
            a.token.as_str(),
            db_now(&pool).await,
            Duration::hours(1),
            Duration::days(30),
        )
        .await
        .unwrap();
    assert!(rotated.family_id == a.family_id && rotated.token_id != a.token_id);
    let rotated_exact:bool=sqlx::query_scalar("SELECT f.revoked_at IS NULL AND old.used_at IS NOT NULL AND old.replaced_by=$2 AND new.used_at IS NULL AND new.revoked_at IS NULL FROM public.auth_refresh_token_families f JOIN public.auth_refresh_tokens old ON old.family_id=f.id JOIN public.auth_refresh_tokens new ON new.family_id=f.id AND new.id=$2 WHERE f.id=$1 AND old.id=$3")
        .bind(a.family_id).bind(rotated.token_id).bind(a.token_id).fetch_one(&pool).await.unwrap();
    assert!(rotated_exact, "actual rotation kept same live family");
    direct_positive(&pool, &f, &access_a, Duration::days(30)).await;
    http_token(&pool, &f, &access_a, StatusCode::OK).await;
    let revoked_at = db_now(&pool).await;
    RefreshTokenStore
        .revoke_family_for_logout(&auth, rotated.token.as_str(), revoked_at)
        .await
        .unwrap();
    let revoked_exact:bool=sqlx::query_scalar("SELECT a.revoked_at=$3 AND a.revoked_reason='logout' AND b.revoked_at IS NULL AND b.revoked_reason IS NULL FROM public.auth_refresh_token_families a JOIN public.auth_refresh_token_families b ON b.id=$2 WHERE a.id=$1")
        .bind(a.family_id).bind(b.family_id).bind(revoked_at).fetch_one(&pool).await.unwrap();
    assert!(
        revoked_exact,
        "actual exact-family logout and live sibling prerequisite"
    );
    direct_denied(&pool, &f, &access_a, Duration::days(30), "unauthorized").await;
    http_token(&pool, &f, &access_a, StatusCode::UNAUTHORIZED).await;
    direct_positive(&pool, &f, &access_b, Duration::days(30)).await;
    http_token(&pool, &f, &access_b, StatusCode::OK).await;
    // Historical bearer does not retrospectively become bound to a chosen family.
    direct_positive(&pool, &f, &f.access, Duration::days(30)).await;
    auth.close().await;
    close(f).await;
}

#[sqlx::test(migrations = false)]
async fn direct_bound_list_lifetime_and_present_binding_faults_never_fall_back(pool: PgPool) {
    let f = fixture(&pool).await;
    let auth = login_test_pool(&pool, TestDatabaseLogin::Auth).await;
    let (a, access, created) = issue_bound(&pool, &f, &auth).await;
    direct_positive(&pool, &f, &access, Duration::days(30)).await;
    // Real family age, original signed token and server-selected TTL; no backdating DB rows.
    let now = db_now(&pool).await;
    assert!(now > created, "real elapsed family-age prerequisite");
    direct_denied(&pool, &f, &access, Duration::nanoseconds(1), "unauthorized").await;
    let claims = serde_json::to_value(f.verifier.verify_access_token(&access).unwrap()).unwrap();
    for fault in [
        "expired",
        "future_iat",
        "future_nbf",
        "inverted",
        "before_family",
        "unknown_family",
        "wrong_binding_home",
        "derived_binding",
        "null_binding",
        "missing_field",
        "unknown_field",
    ] {
        let mut bad = claims.clone();
        let now = db_now(&pool).await.unix_timestamp();
        match fault {
            "expired" => bad["exp"] = json!(now - 120),
            "future_iat" => bad["iat"] = json!(now + 120),
            "future_nbf" => bad["nbf"] = json!(now + 120),
            "inverted" => {
                bad["iat"] = json!(now);
                bad["nbf"] = json!(now - 1);
            }
            "before_family" => {
                bad["iat"] = json!(created.unix_timestamp() - 1);
                bad["nbf"] = bad["iat"].clone();
            }
            "unknown_family" => bad["legacy_session"]["family_id"] = json!(Uuid::new_v4()),
            "wrong_binding_home" => bad["legacy_session"]["home_org"] = json!(OrgId::knl()),
            "derived_binding" => bad["legacy_session"]["kind"] = json!("platform_view_as"),
            "null_binding" => bad["legacy_session"] = Value::Null,
            "missing_field" => {
                bad["legacy_session"]
                    .as_object_mut()
                    .unwrap()
                    .remove("version");
            }
            "unknown_field" => bad["legacy_session"]["unexpected"] = json!(true),
            _ => unreachable!(),
        }
        direct_denied(
            &pool,
            &f,
            &signed(&f, &bad),
            Duration::days(30),
            "unauthorized",
        )
        .await;
    }
    assert!(
        a.family_id
            == f.verifier
                .verify_access_token(&access)
                .unwrap()
                .legacy_session
                .unwrap()
                .family_id
    );
    direct_positive(&pool, &f, &access, Duration::days(30)).await;
    auth.close().await;
    close(f).await;
}

fn no_raw_input_echo(headers: &HeaderMap, body: &[u8], inputs: &[Vec<u8>]) -> bool {
    inputs.iter().all(|input| {
        !input.is_empty()
            && !body
                .windows(input.len())
                .any(|part| part == input.as_slice())
            && headers.iter().all(|(_, value)| {
                !value
                    .as_bytes()
                    .windows(input.len())
                    .any(|part| part == input.as_slice())
            })
    })
}

#[test]
fn raw_authorization_privacy_oracle_detects_non_utf8_echo() {
    let raw = b"Bearer \xff".to_vec();
    let inputs = vec![raw.clone()];
    let headers = HeaderMap::new();
    let body = b"private finite refusal";
    assert!(no_raw_input_echo(&headers, body, &inputs));
    let mut echoed_body = body.to_vec();
    echoed_body.extend_from_slice(&raw);
    assert!(!no_raw_input_echo(&headers, &echoed_body, &inputs));
    let mut echoed_headers = headers.clone();
    echoed_headers.insert("x-fixture", http::HeaderValue::from_bytes(&raw).unwrap());
    assert!(!no_raw_input_echo(&echoed_headers, body, &inputs));
    assert!(!no_raw_input_echo(&headers, body, &[Vec::new()]));
}

#[sqlx::test(migrations = false)]
async fn mounted_list_get_head_reject_ambiguous_headers_before_effects_and_accept_exact_bearer(
    pool: PgPool,
) {
    let f = fixture(&pool).await;
    for method in [Method::GET, Method::HEAD] {
        for case in [
            "absent",
            "cookie_only",
            "empty",
            "basic",
            "lowercase",
            "leading_space",
            "double_space",
            "trailing_space",
            "tab",
            "comma",
            "duplicate_same",
            "duplicate_conflict",
            "non_utf8",
        ] {
            let mut request = Request::builder()
                .method(method.clone())
                .uri(PATH)
                .body(Body::empty())
                .unwrap();
            let mut secrets = f.secrets.clone();
            let values: Vec<Vec<u8>> = match case {
                "absent" => vec![],
                "cookie_only" => {
                    request.headers_mut().insert(
                        header::COOKIE,
                        format!("access_token={}", f.access).parse().unwrap(),
                    );
                    vec![]
                }
                "empty" => vec![b"Bearer ".to_vec()],
                "basic" => vec![format!("Basic {}", f.access).into_bytes()],
                "lowercase" => vec![format!("bearer {}", f.access).into_bytes()],
                "leading_space" => vec![format!(" Bearer {}", f.access).into_bytes()],
                "double_space" => vec![format!("Bearer  {}", f.access).into_bytes()],
                "trailing_space" => vec![format!("Bearer {} ", f.access).into_bytes()],
                "tab" => vec![format!("Bearer\t{}", f.access).into_bytes()],
                "comma" => vec![format!("Bearer {},Bearer {}", f.access, f.access).into_bytes()],
                "duplicate_same" => vec![format!("Bearer {}", f.access).into_bytes(); 2],
                "duplicate_conflict" => vec![
                    format!("Bearer {}", f.access).into_bytes(),
                    b"Bearer other-private-credential".to_vec(),
                ],
                "non_utf8" => vec![b"Bearer \xff".to_vec()],
                _ => unreachable!(),
            };
            // Keep original bytes too: obs-text cannot be represented by the string oracle.
            let mut raw_inputs = values.clone();
            if let Some(cookie) = request.headers().get(header::COOKIE) {
                raw_inputs.push(cookie.as_bytes().to_vec());
            }
            for value in values {
                if let Ok(value) = std::str::from_utf8(&value)
                    && !value.is_empty()
                {
                    secrets.push(value.to_owned());
                }
                request.headers_mut().append(
                    header::AUTHORIZATION,
                    http::HeaderValue::from_bytes(&value).unwrap(),
                );
            }
            let before = all_rows(&pool).await;
            let response = f.router.clone().oneshot(request).await.unwrap();
            let (status, headers, body) = response_parts(response).await;
            assert!(
                no_raw_input_echo(&headers, &body, &raw_inputs),
                "raw current Authorization/Cookie input reflection: {case}"
            );
            assert!(
                status == StatusCode::UNAUTHORIZED && private_response(&headers, &body, &secrets),
                "exact early finite privacy: {case}"
            );
            if method == Method::HEAD {
                assert!(body.is_empty(), "HEAD refusal body must be empty");
            } else {
                assert!(
                    serde_json::from_slice::<Value>(&body).unwrap()
                        == json!({"error":{"code":"unauthorized","message":"invalid bearer token"}})
                );
            }
            assert!(
                before == all_rows(&pool).await,
                "early rejection must leave every table unchanged: {case}"
            );
        }
        let before = all_rows(&pool).await;
        let start = db_now(&pool).await;
        let response = f
            .router
            .clone()
            .oneshot(
                Request::builder()
                    .method(method.clone())
                    .uri(format!("{PATH}?display=all"))
                    .header(header::AUTHORIZATION, format!("Bearer {}", f.access))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let (status, headers, body) = response_parts(response).await;
        let end = db_now(&pool).await;
        assert!(status == StatusCode::OK && private_response(&headers, &body, &f.secrets));
        if method == Method::HEAD {
            assert!(body.is_empty());
        } else {
            assert!(metadata_matches(
                &serde_json::from_slice::<Value>(&body).unwrap(),
                &f.expected
            ));
        }
        assert!(
            exact_read_delta(
                &before,
                &all_rows(&pool).await,
                f.actor,
                f.expected.as_array().unwrap().len(),
                start,
                end
            ),
            "GET and actual HEAD fallback each execute one owned read"
        );
    }
    close(f).await;
}

#[tokio::test]
async fn list_transport_selection_preserves_sibling_routes_and_methods() {
    async fn sibling() -> http::Response<Body> {
        http::Response::builder()
            .status(StatusCode::IM_A_TEAPOT)
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::CACHE_CONTROL, "max-age=17")
            .header(header::ETAG, "fixture-validator")
            .header(header::VARY, "Origin")
            .body(Body::from("fixture sibling"))
            .unwrap()
    }
    let router =
        console_platform_rest::with_platform_list_transport(axum::Router::new().fallback(sibling));
    for (method, path) in [
        (Method::GET, "/api/platform/orgs/"),
        (Method::GET, "/api/platform/groups"),
        (Method::POST, PATH),
        (Method::PUT, PATH),
    ] {
        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(path)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let (status, headers, body) = response_parts(response).await;
        assert!(status == StatusCode::IM_A_TEAPOT && body == b"fixture sibling");
        assert!(
            headers[header::CACHE_CONTROL] == "max-age=17"
                && headers[header::ETAG] == "fixture-validator"
                && headers[header::VARY] == "Origin"
        );
    }
}

#[path = "legacy_platform_list_locks.rs"]
mod legacy_platform_list_locks;
