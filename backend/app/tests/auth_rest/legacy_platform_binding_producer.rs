//! Proposed child module of app/tests/auth_rest.rs. Not compiled or admitted.
//! Genuine production HTTP producers; existing SoftPasskey harness limitations apply.
use super::*;
use account_browser::deployment_operator_designation::company_setup::all_rows;
use console_platform_auth::{JwtSettings, JwtVerifier, LegacySessionKind};
use console_platform_test_support::login_test_pool;

struct DirectFixture {
    router: axum::Router,
    key: SigningKey,
    verifier: JwtVerifier,
    subject: UserId,
    authenticator: WebauthnAuthenticator<SoftPasskey>,
    credential: String,
    otp_access: String,
    otp_refresh: String,
    login: TokenPairResponse,
}

async fn direct_fixture(pool: &PgPool, platform: bool, group_role: bool) -> DirectFixture {
    prepare_http_database(pool).await;
    let org = if platform {
        OrgId::platform()
    } else {
        OrgId::knl()
    };
    let subject = UserId::new();
    sqlx::query("INSERT INTO public.users(id,display_name,roles,org_id) VALUES($1,'Direct session producer', $2, $3)")
        .bind(subject.as_uuid()).bind(vec![if platform {"SUPER_ADMIN"} else {"MEMBER"}])
        .bind(org.as_uuid()).execute(pool).await.unwrap();
    if group_role {
        assert!(!platform, "group fixture is an ordinary Company subject");
        let group: Uuid =
            sqlx::query_scalar("SELECT group_id FROM public.organizations WHERE id=$1")
                .bind(org.as_uuid())
                .fetch_one(pool)
                .await
                .unwrap();
        sqlx::query("INSERT INTO public.group_role_grants(group_id,user_id,group_role) VALUES($1,$2,'GROUP_ADMIN')")
            .bind(group).bind(subject.as_uuid()).execute(pool).await.unwrap();
    }
    let key = SigningKey::random(&mut OsRng);
    let public = key
        .verifying_key()
        .to_public_key_pem(LineEnding::LF)
        .unwrap();
    let router = build_router(
        app_state(
            pool.clone(),
            key.to_pkcs8_pem(LineEnding::LF).unwrap().to_string(),
            public.clone(),
        )
        .await
        .unwrap(),
    );
    let business = login_test_pool(pool, TestDatabaseLogin::Business).await;
    let actual: (String, String) = sqlx::query_as("SELECT session_user::text,current_user::text")
        .fetch_one(&business)
        .await
        .unwrap();
    assert!(actual == ("console_rt".into(), "console_rt".into()));
    let issue = BootstrapCredentialStore
        .issue_for_zero_credential_user(
            &business,
            *subject.as_uuid(),
            org,
            OffsetDateTime::now_utc(),
            Duration::hours(1),
        )
        .await
        .unwrap();
    let redeemed: OtpRedeemResponse = post_json(
        router.clone(),
        "/api/v1/auth/otp/redeem",
        None,
        json!({"otp":issue.token.as_str()}),
        StatusCode::OK,
    )
    .await;
    assert!(redeemed.requires_passkey_setup);
    let mut authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let credential = enroll_passkey(&router, &mut authenticator, &redeemed.access_token).await;
    let login = usernameless_login(&router, &mut authenticator, &credential).await;
    assert!(!login.requires_passkey_setup);
    let verifier = JwtVerifier::from_es256_public_pem(
        JwtSettings {
            issuer: TEST_ISSUER.into(),
            audience: TEST_AUDIENCE.into(),
            access_token_ttl: Duration::minutes(15),
        },
        public.as_bytes(),
    )
    .unwrap();
    business.close().await;
    DirectFixture {
        router,
        key,
        verifier,
        subject,
        authenticator,
        credential,
        otp_access: redeemed.access_token,
        otp_refresh: redeemed.refresh_token.unwrap(),
        login,
    }
}

// DB correlation uses the exact returned refresh token's hash, never newest
// family, jti, row order, JWT-provided family lookup, or a test-created family.
async fn exact_bound_pair(pool: &PgPool, f: &DirectFixture, access: &str, refresh: &str) -> Uuid {
    let c = f.verifier.verify_access_token(access).unwrap();
    assert!(
        c.sub == f.subject.to_string()
            && c.org == OrgId::platform().to_string()
            && c.platform
            && !c.view_as
            && !c.read_only
            && c.tenant_context.is_none()
    );
    let b = c
        .legacy_session
        .as_ref()
        .expect("actual platform producer omitted binding");
    type FamilyRow = (
        Uuid,
        Uuid,
        Uuid,
        OffsetDateTime,
        Option<OffsetDateTime>,
        String,
        Option<i64>,
        Option<OffsetDateTime>,
        Option<String>,
    );
    let row:FamilyRow = sqlx::query_as(
        "SELECT f.id,f.user_id,f.org_id,f.created_at,f.revoked_at,f.protocol,f.account_security_generation,f.auth_time,f.assurance FROM public.auth_refresh_tokens t JOIN public.auth_refresh_token_families f ON f.id=t.family_id AND f.user_id=t.user_id AND f.org_id=t.org_id WHERE t.token_hash=sha256(convert_to($1,'UTF8')) AND t.used_at IS NULL AND t.revoked_at IS NULL AND t.expires_at>clock_timestamp()")
        .bind(refresh).fetch_one(pool).await.unwrap();
    assert!(
        b.version == 1
            && !b.family_id.is_nil()
            && b.family_id == row.0
            && b.home_org == *OrgId::platform().as_uuid()
            && b.kind == LegacySessionKind::Direct
    );
    assert!(
        row.1 == *f.subject.as_uuid()
            && row.2 == *OrgId::platform().as_uuid()
            && row.4.is_none()
            && row.5 == "LEGACY_USER"
            && row.6.is_none()
            && row.7.is_none()
            && row.8.is_none()
    );
    assert!(c.iat >= row.3.unix_timestamp() && c.nbf >= c.iat && c.exp > c.nbf);
    row.0
}

// Test-only historical-shape successor. No stored receipt, command, migration
// or existing token bytes change. Every claim except additive binding stays exact.
// Kept pub(super) only so independently reviewed legacy fixtures can reuse it.
pub(super) fn historical_access(key: &SigningKey, verifier: &JwtVerifier, token: &str) -> String {
    let original = serde_json::to_value(verifier.verify_access_token(token).unwrap()).unwrap();
    let mut old = original.clone();
    old.as_object_mut().unwrap().remove("legacy_session");
    let historical = jsonwebtoken::encode(
        &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::ES256),
        &old,
        &jsonwebtoken::EncodingKey::from_ec_pem(
            key.to_pkcs8_pem(LineEnding::LF).unwrap().as_bytes(),
        )
        .unwrap(),
    )
    .unwrap();
    let verified =
        serde_json::to_value(verifier.verify_access_token(&historical).unwrap()).unwrap();
    assert!(
        verified == old,
        "historical fixture changed a retained claim"
    );
    let mut restored = verified;
    if let Some(binding) = original.get("legacy_session") {
        restored["legacy_session"] = binding.clone();
    }
    assert!(restored == original, "only the additive binding may differ");
    historical
}

async fn rotate(f: &DirectFixture, token: &str) -> TokenPairResponse {
    post_json(
        f.router.clone(),
        "/api/v1/auth/token/refresh",
        None,
        json!({"refresh_token":token}),
        StatusCode::OK,
    )
    .await
}

#[sqlx::test(migrations = false)]
async fn platform_direct_otp_login_and_both_refresh_transports_bind_exact_family(pool: PgPool) {
    let mut f = direct_fixture(&pool, true, false).await;
    let otp_family = exact_bound_pair(&pool, &f, &f.otp_access, &f.otp_refresh).await;
    let login_family = exact_bound_pair(
        &pool,
        &f,
        &f.login.access_token,
        f.login.refresh_token.as_deref().unwrap(),
    )
    .await;
    assert!(
        otp_family != login_family,
        "separate real producers must create separate families"
    );
    // Newer live login family already exists: refreshing old OTP must retain old family.
    let rotated = rotate(&f, &f.otp_refresh).await;
    assert!(rotated.refresh_token.as_deref() != Some(f.otp_refresh.as_str()));
    assert!(
        exact_bound_pair(
            &pool,
            &f,
            &rotated.access_token,
            rotated.refresh_token.as_deref().unwrap()
        )
        .await
            == otp_family
    );
    let cookie_login =
        cookie_mode_usernameless_login(&f.router, &mut f.authenticator, &f.credential).await;
    assert!(cookie_login.status() == StatusCode::OK);
    let cookie = console_refresh_set_cookie(&cookie_login).unwrap();
    let body = body_json(cookie_login).await;
    assert!(body["refresh_token"].is_null());
    let family = exact_bound_pair(
        &pool,
        &f,
        body["access_token"].as_str().unwrap(),
        cookie_token(&cookie),
    )
    .await;
    assert!(family != otp_family && family != login_family);
    let response = post_cookie_mode(
        f.router.clone(),
        "/api/v1/auth/token/refresh",
        Some(cookie_token(&cookie)),
        json!({}),
    )
    .await;
    assert!(response.status() == StatusCode::OK);
    let replacement = console_refresh_set_cookie(&response).unwrap();
    let body = body_json(response).await;
    assert!(body["refresh_token"].is_null() && cookie_token(&replacement) != cookie_token(&cookie));
    assert!(
        exact_bound_pair(
            &pool,
            &f,
            body["access_token"].as_str().unwrap(),
            cookie_token(&replacement)
        )
        .await
            == family
    );
}

#[sqlx::test(migrations = false)]
async fn platform_direct_handoff_poll_binds_consumed_family_once(pool: PgPool) {
    let mut f = direct_fixture(&pool, true, false).await;
    let handoff: Value = post_json(
        f.router.clone(),
        "/api/v1/auth/device-login/start",
        None,
        json!({}),
        StatusCode::OK,
    )
    .await;
    let url = Url::parse(handoff["approve_url"].as_str().unwrap()).unwrap();
    let approve = url::form_urlencoded::parse(
        url.fragment()
            .expect("real approve URL fragment")
            .as_bytes(),
    )
    .find(|(key, _)| key == "desktop_approve")
    .expect("real approve-token fragment contract")
    .1
    .into_owned();
    let start: LoginStartResponse = post_json(
        f.router.clone(),
        "/api/v1/auth/passkey/login/start",
        None,
        json!({}),
        StatusCode::OK,
    )
    .await;
    let assertion = f
        .authenticator
        .do_authentication(
            Url::parse(TEST_ORIGIN).unwrap(),
            inject_allow_credential(start.challenge, &f.credential),
        )
        .unwrap();
    let approved = post_raw(
        f.router.clone(),
        "/api/v1/auth/device-login/approve",
        None,
        json!({"approve_token":approve,"ceremony_id":start.ceremony_id,"credential":assertion}),
    )
    .await;
    assert!(approved.status() == StatusCode::NO_CONTENT);
    let response: Value = post_json(
        f.router.clone(),
        "/api/v1/auth/device-login/poll",
        None,
        json!({"poll_token":handoff["poll_token"]}),
        StatusCode::OK,
    )
    .await;
    assert!(response["status"] == "approved");
    let family = exact_bound_pair(
        &pool,
        &f,
        response["access_token"].as_str().unwrap(),
        response["refresh_token"].as_str().unwrap(),
    )
    .await;
    let correlated:i64=sqlx::query_scalar("SELECT count(*) FROM public.audit_events WHERE action='auth.device_login.consume' AND actor=$1 AND after_snap->>'refresh_family_id'=$2")
        .bind(f.subject.as_uuid()).bind(family.to_string()).fetch_one(&pool).await.unwrap();
    assert!(correlated == 1);
    let before = all_rows(&pool).await;
    let duplicate = post_raw(
        f.router.clone(),
        "/api/v1/auth/device-login/poll",
        None,
        json!({"poll_token":handoff["poll_token"]}),
    )
    .await;
    assert!(duplicate.status() == StatusCode::UNAUTHORIZED);
    let after = all_rows(&pool).await;
    for table in [
        "auth_refresh_token_families",
        "auth_refresh_tokens",
        "auth_device_login_handoffs",
    ] {
        assert!(
            before[table] == after[table],
            "duplicate handoff added credential effect"
        );
    }
}

#[sqlx::test(migrations = false)]
async fn platform_historical_unbound_refresh_upgrades_without_rewriting_original(pool: PgPool) {
    let f = direct_fixture(&pool, true, false).await;
    let historical = historical_access(&f.key, &f.verifier, &f.login.access_token);
    let original = f.verifier.verify_access_token(&historical).unwrap();
    assert!(original.legacy_session.is_none());
    let response = get_legacy_raw(&f.router, "/api/platform/orgs", &historical).await;
    assert!(
        response.status() == StatusCode::OK,
        "historical current source read remains accepted"
    );
    let family = exact_bound_pair(
        &pool,
        &f,
        &f.login.access_token,
        f.login.refresh_token.as_deref().unwrap(),
    )
    .await;
    let rotated = rotate(&f, f.login.refresh_token.as_deref().unwrap()).await;
    assert!(
        exact_bound_pair(
            &pool,
            &f,
            &rotated.access_token,
            rotated.refresh_token.as_deref().unwrap()
        )
        .await
            == family
    );
    let retained = f.verifier.verify_access_token(&historical).unwrap();
    assert!(
        retained == original && retained.legacy_session.is_none(),
        "refresh reinterpreted original token"
    );
    assert!(
        get_legacy_raw(&f.router, "/api/platform/orgs", &historical)
            .await
            .status()
            == StatusCode::OK
    );
    // Existing PlatformPolicy tests retain old ReadProjection Allow/Effect
    // UpgradeRequired. This leaf proves real refresh upgrade, not a new effect owner.
}

async fn nonempty_self_keys(pool: &PgPool, f: &DirectFixture, access: &str) {
    let before = all_rows(pool).await;
    let keys: Value = get_legacy_raw(&f.router, "/api/v1/auth/passkeys", access)
        .await
        .into_json(StatusCode::OK)
        .await;
    type StoredKey = (Uuid, OffsetDateTime, Option<OffsetDateTime>);
    let expected:Vec<StoredKey>=sqlx::query_as("SELECT id,created_at,last_used_at FROM public.auth_webauthn_credentials WHERE user_id=$1 ORDER BY id")
        .bind(f.subject.as_uuid()).fetch_all(pool).await.unwrap();
    assert!(expected.len() == 1 && keys.as_array().is_some_and(|rows| rows.len() == 1));
    let row = &keys[0];
    assert!(row["id"] == json!(expected[0].0));
    let parse = |v: &Value| {
        v.as_str().and_then(|s| {
            OffsetDateTime::parse(s, &time::format_description::well_known::Rfc3339).ok()
        })
    };
    assert!(
        parse(&row["created_at"]) == Some(expected[0].1),
        "created timestamp differs from stored credential"
    );
    assert!(
        match expected[0].2 {
            Some(at) => parse(&row["last_used_at"]) == Some(at),
            None => row["last_used_at"].is_null(),
        },
        "last-used timestamp differs from stored credential"
    );
    let mut names: Vec<_> = row
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    names.sort_unstable();
    assert!(names == vec!["created_at", "id", "last_used_at"]);
    assert!(
        before == all_rows(pool).await,
        "self projection changed durable rows"
    );
}

#[sqlx::test(migrations = false)]
async fn platform_direct_self_passkeys_revocation_preserves_historical_and_other_family(
    pool: PgPool,
) {
    let f = direct_fixture(&pool, true, false).await;
    let historical = historical_access(&f.key, &f.verifier, &f.login.access_token);
    let family = exact_bound_pair(
        &pool,
        &f,
        &f.login.access_token,
        f.login.refresh_token.as_deref().unwrap(),
    )
    .await;
    nonempty_self_keys(&pool, &f, &historical).await;
    nonempty_self_keys(&pool, &f, &f.login.access_token).await;
    nonempty_self_keys(&pool, &f, &f.otp_access).await;
    let logout = post_raw(
        f.router.clone(),
        "/api/v1/auth/logout",
        None,
        json!({"refresh_token":f.login.refresh_token.as_deref().unwrap()}),
    )
    .await;
    assert!(logout.status() == StatusCode::OK);
    let revoked: bool = sqlx::query_scalar(
        "SELECT revoked_at IS NOT NULL FROM public.auth_refresh_token_families WHERE id=$1",
    )
    .bind(family)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(revoked);
    let before = all_rows(&pool).await;
    let refused = get_legacy_raw(&f.router, "/api/v1/auth/passkeys", &f.login.access_token).await;
    assert!(refused.status() == StatusCode::UNAUTHORIZED);
    assert!(before == all_rows(&pool).await);
    nonempty_self_keys(&pool, &f, &historical).await;
    nonempty_self_keys(&pool, &f, &f.otp_access).await;
}

#[sqlx::test(migrations = false)]
async fn ordinary_group_role_issuance_and_refresh_stay_unbound(pool: PgPool) {
    let f = direct_fixture(&pool, false, true).await;
    for access in [&f.otp_access, &f.login.access_token] {
        let c = f.verifier.verify_access_token(access).unwrap();
        assert!(
            c.org == OrgId::knl().to_string()
                && !c.platform
                && !c.view_as
                && !c.read_only
                && c.legacy_session.is_none()
                && c.group_roles == vec!["GROUP_ADMIN"]
                && c.tenant_context.is_none()
        );
    }
    let rotated = rotate(&f, f.login.refresh_token.as_deref().unwrap()).await;
    let c = f
        .verifier
        .verify_access_token(&rotated.access_token)
        .unwrap();
    assert!(c.legacy_session.is_none() && !c.platform && c.group_roles == vec!["GROUP_ADMIN"]);
    nonempty_self_keys(&pool, &f, &rotated.access_token).await;
    // Ordinary non-group coverage remains existing legacy_fence_fixture and
    // otp_first_signin_then_passkey_enrollment_then_usernameless_login, unchanged.
}

#[path = "legacy_platform_binding_producer/signing_rollback.rs"]
mod signing_rollback;
