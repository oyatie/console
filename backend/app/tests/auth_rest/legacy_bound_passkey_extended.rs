//! Child of legacy_bound_passkey_reads: real mounted routes and retained fixtures.
use super::*;
use console_platform_auth::{AccessTokenInput, JwtIssuer, RefreshTokenUseError};
use futures::FutureExt;

fn verifier(f: &Fixture) -> JwtVerifier {
    JwtVerifier::from_es256_public_pem(
        JwtSettings {
            issuer: TEST_ISSUER.to_owned(),
            audience: TEST_AUDIENCE.to_owned(),
            access_token_ttl: Duration::minutes(15),
        },
        f.legacy
            .signing_key
            .verifying_key()
            .to_public_key_pem(LineEnding::LF)
            .unwrap()
            .as_bytes(),
    )
    .unwrap()
}
fn claims(f: &Fixture, token: &str) -> Value {
    serde_json::to_value(verifier(f).verify_access_token(token).unwrap()).unwrap()
}
fn sign(f: &Fixture, claims: &Value) -> String {
    jsonwebtoken::encode(
        &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::ES256),
        claims,
        &jsonwebtoken::EncodingKey::from_ec_pem(
            f.legacy
                .signing_key
                .to_pkcs8_pem(LineEnding::LF)
                .unwrap()
                .as_bytes(),
        )
        .unwrap(),
    )
    .unwrap()
}
async fn accepted(f: &Fixture, router: &axum::Router, token: &str, expected: &Value) {
    let mut secrets = f.secrets();
    secrets.push(token.to_owned());
    if let Ok(verified) = verifier(f).verify_access_token(token)
        && let Some(binding) = verified.legacy_session
    {
        secrets.push(binding.family_id.to_string());
    }
    let body = response_json(
        get_legacy_raw(router, PATH, token).await,
        StatusCode::OK,
        &secrets,
        true,
    )
    .await;
    assert!(&body == expected, "exact independently stored summary");
}
async fn refused(f: &Fixture, router: &axum::Router, token: &str) {
    let mut secrets = f.secrets();
    secrets.push(token.to_owned());
    if let Ok(verified) = verifier(f).verify_access_token(token)
        && let Some(binding) = verified.legacy_session
    {
        secrets.push(binding.family_id.to_string());
    }
    let body = response_json(
        get_legacy_raw(router, PATH, token).await,
        StatusCode::UNAUTHORIZED,
        &secrets,
        true,
    )
    .await;
    assert!(body == error_body("unauthorized", "invalid bearer token"));
}

async fn family_ttl_router(pool: &PgPool, f: &Fixture, ttl_seconds: u64) -> axum::Router {
    // Same production startup and admitted transports as app_state; the single
    // changed server policy is the existing refresh-family lifetime setting.
    let mut pairs = vec![
        (
            "CONSOLE_DATABASE_DURABILITY",
            r#"{"mode":"local_development"}"#.to_owned(),
        ),
        ("CONSOLE_APP_ROLE", AppRole::Api.to_string()),
        ("CONSOLE_HTTP_ADDR", "127.0.0.1:0".to_owned()),
        ("CONSOLE_JWT_ISSUER", TEST_ISSUER.to_owned()),
        ("CONSOLE_JWT_AUDIENCE", TEST_AUDIENCE.to_owned()),
        (
            "CONSOLE_JWT_PRIVATE_KEY_PEM",
            f.legacy
                .signing_key
                .to_pkcs8_pem(LineEnding::LF)
                .unwrap()
                .to_string(),
        ),
        (
            "CONSOLE_JWT_PUBLIC_KEY_PEM",
            f.legacy
                .signing_key
                .verifying_key()
                .to_public_key_pem(LineEnding::LF)
                .unwrap(),
        ),
        ("CONSOLE_WEBAUTHN_RP_ID", "example.com".to_owned()),
        ("CONSOLE_WEBAUTHN_RP_ORIGIN", TEST_ORIGIN.to_owned()),
        ("CONSOLE_WEBAUTHN_RP_NAME", "Console".to_owned()),
        (
            "CONSOLE_REFRESH_FAMILY_ABSOLUTE_TTL_SECS",
            ttl_seconds.to_string(),
        ),
    ];
    pairs.extend(account_transport_urls(pool));
    build_router(
        AppState::from_config(AppConfig::from_pairs(pairs).unwrap())
            .await
            .unwrap(),
    )
}

#[sqlx::test(migrations = false)]
async fn rotation_keeps_exact_family_access_but_real_refresh_reuse_revokes_it(pool: PgPool) {
    let f = fixture(&pool).await;
    let before_all = all_rows(&pool).await;
    let before = refresh_complete_snapshot(&pool).await;
    let now = db_now(&f.auth).await;
    let replacement = RefreshTokenStore
        .rotate(
            &pool,
            &f.auth,
            f.a.token.as_str(),
            now,
            Duration::hours(1),
            Duration::days(1),
        )
        .await
        .unwrap();
    assert!(replacement.family_id == f.a.family_id && replacement.token_id != f.a.token_id);
    let after = refresh_complete_snapshot(&pool).await;
    super::super::account_fence_transport::assert_exact_refresh_delta(
        &before,
        &after,
        f.legacy.subject,
        f.a.token.as_str(),
        replacement.token.as_str(),
    );
    unchanged_except(
        &before_all,
        &all_rows(&pool).await,
        &["auth_refresh_tokens", "audit_events"],
    );
    let rotated = all_rows(&pool).await;
    accepted(&f, &f.legacy.router, &f.access_a, &f.expected).await;
    accepted(&f, &f.legacy.router, &f.access_b, &f.expected).await;
    assert!(
        rotated == all_rows(&pool).await,
        "access reads after rotation are effect-free"
    );
    let before = refresh_complete_snapshot(&pool).await;
    let now = db_now(&f.auth).await;
    let result = RefreshTokenStore
        .rotate(
            &pool,
            &f.auth,
            f.a.token.as_str(),
            now,
            Duration::hours(1),
            Duration::days(1),
        )
        .await;
    assert!(matches!(result, Err(RefreshTokenUseError::ReuseDetected)));
    let after = refresh_complete_snapshot(&pool).await;
    unchanged_except(
        &rotated,
        &all_rows(&pool).await,
        &[
            "auth_refresh_token_families",
            "auth_refresh_tokens",
            "audit_events",
        ],
    );
    for roster in ["families", "tokens"] {
        let old = before[roster].as_array().unwrap();
        let new = after[roster].as_array().unwrap();
        assert_eq!(old.len(), new.len());
        for row in old {
            let actual = new.iter().find(|r| r["id"] == row["id"]).unwrap();
            let belongs = if roster == "families" {
                row["id"] == json!(f.a.family_id)
            } else {
                row["family_id"] == json!(f.a.family_id)
            };
            if !belongs {
                assert!(actual == row, "reuse modified unrelated row");
                continue;
            }
            let mut expected = row.clone();
            expected["revoked_at"] = actual["revoked_at"].clone();
            assert!(!actual["revoked_at"].is_null());
            if roster == "families" {
                expected["revoked_reason"] = json!("reuse_detected");
            }
            if roster == "tokens" && row["id"] == json!(f.a.token_id) {
                expected["reuse_detected_at"] = actual["reuse_detected_at"].clone();
                assert!(!actual["reuse_detected_at"].is_null());
            }
            assert!(expected == *actual, "unexpected reuse terminal change");
        }
    }
    let exact:bool=sqlx::query_scalar("SELECT f.revoked_at=$2 AND f.revoked_reason='reuse_detected' AND (SELECT bool_and(t.revoked_at=$2 AND (CASE WHEN t.id=$3 THEN t.reuse_detected_at=$2 ELSE t.reuse_detected_at IS NULL END)) FROM public.auth_refresh_tokens t WHERE t.family_id=f.id) FROM public.auth_refresh_token_families f WHERE f.id=$1")
        .bind(f.a.family_id).bind(now).bind(f.a.token_id).fetch_one(&pool).await.unwrap();
    assert!(exact);
    let old = before["audit"].as_array().unwrap();
    let new = after["audit"].as_array().unwrap();
    assert_eq!(new.len(), old.len() + 1);
    assert!(old.iter().all(|r| new.contains(r)));
    let added: Vec<_> = new.iter().filter(|r| !old.contains(r)).collect();
    assert_eq!(added.len(), 1);
    assert!(added[0]["action"] == "auth.refresh.reuse_detected");
    assert!(
        added[0]["after_snap"]
            == json!({"family_id":f.a.family_id,"revoked_reason":"reuse_detected","reused_token_id":f.a.token_id})
    );
    // Reuse exactly the already reviewed complete audit metadata oracle, after
    // separately checking the only two event-specific fields we normalize.
    let mut normalized = added[0].clone();
    normalized["action"] = json!("auth.logout");
    normalized["after_snap"] = json!({"family_id":f.a.family_id,"revoked_reason":"logout"});
    assert!(logout_audit_matches(
        &normalized,
        f.legacy.subject,
        f.a.family_id,
        now,
        db_now(&pool).await
    ));
    let revoked = all_rows(&pool).await;
    refused(&f, &f.legacy.router, &f.access_a).await;
    accepted(&f, &f.legacy.router, &f.access_b, &f.expected).await;
    accepted(
        &f,
        &f.legacy.router,
        &f.legacy.control_access,
        &f.control_expected,
    )
    .await;
    assert!(revoked == all_rows(&pool).await);
    f.auth.close().await;
}

#[sqlx::test(migrations = false)]
async fn exact_unknown_cross_subject_cross_home_and_native_families_refuse(pool: PgPool) {
    let f = fixture(&pool).await;
    let (_, native_family) =
        super::super::account_browser::native_family_for_legacy_reader(&pool).await;
    let before = all_rows(&pool).await;
    let original = claims(&f, &f.access_a);
    for family in [Uuid::new_v4(), native_family] {
        let mut changed = original.clone();
        changed["legacy_session"]["family_id"] = json!(family);
        refused(&f, &f.legacy.router, &sign(&f, &changed)).await;
    }
    let mut changed = original.clone();
    changed["sub"] = json!(f.legacy.control.to_string());
    refused(&f, &f.legacy.router, &sign(&f, &changed)).await;
    let mut changed = original.clone();
    let other = Uuid::new_v4();
    changed["org"] = json!(other);
    changed["legacy_session"]["home_org"] = json!(other);
    refused(&f, &f.legacy.router, &sign(&f, &changed)).await;
    let mut changed = original;
    changed["sub"] = json!(Uuid::new_v4());
    refused(&f, &f.legacy.router, &sign(&f, &changed)).await;
    accepted(&f, &f.legacy.router, &f.access_b, &f.expected).await;
    assert!(before == all_rows(&pool).await);
    f.auth.close().await;
}

#[sqlx::test(migrations = false)]
async fn malformed_present_binding_never_falls_back_to_historical_access(pool: PgPool) {
    let f = fixture(&pool).await;
    let before = all_rows(&pool).await;
    let original = claims(&f, &f.access_a);
    for binding in [
        Value::Null,
        json!([]),
        json!({}),
        json!({"version":1,"family_id":f.a.family_id,"home_org":OrgId::knl(),"kind":"direct","unknown":true}),
        json!({"version":2,"family_id":f.a.family_id,"home_org":OrgId::knl(),"kind":"direct"}),
    ] {
        let mut changed = original.clone();
        changed["legacy_session"] = binding;
        refused(&f, &f.legacy.router, &sign(&f, &changed)).await;
    }
    accepted(&f, &f.legacy.router, &f.legacy.access, &f.expected).await;
    accepted(&f, &f.legacy.router, &f.access_b, &f.expected).await;
    assert!(before == all_rows(&pool).await);
    f.auth.close().await;
}

#[sqlx::test(migrations = false)]
async fn explicit_derived_shapes_refuse_even_with_live_sibling_family(pool: PgPool) {
    let f = fixture(&pool).await;
    let before = all_rows(&pool).await;
    let base = claims(&f, &f.access_a);
    let mut derived = Vec::new();
    let mut group = base.clone();
    group["roles"] = json!(["ADMIN"]);
    group["tenant_context"] = json!("group_admin");
    group["actor_home_org"] = json!(OrgId::knl().to_string());
    group["group_context_id"] = json!(Uuid::new_v4().to_string());
    group["group_roles"] = json!(["GROUP_ADMIN"]);
    group["legacy_session"]["kind"] = json!("group_admin");
    derived.push(group);
    for view in [true, false] {
        let mut platform = base.clone();
        platform["view_as"] = json!(view);
        platform["read_only"] = json!(view);
        platform["roles"] = json!(["SUPER_ADMIN"]);
        platform["legacy_session"]["home_org"] = json!(OrgId::platform());
        platform["legacy_session"]["kind"] = json!(if view {
            "platform_view_as"
        } else {
            "platform_tenant_context"
        });
        derived.push(platform);
    }
    for changed in derived {
        let token = sign(&f, &changed);
        assert!(
            verifier(&f).verify_access_token(&token).is_ok(),
            "valid signed derived shape prerequisite"
        );
        refused(&f, &f.legacy.router, &token).await;
        let mut historical = changed.clone();
        historical.as_object_mut().unwrap().remove("legacy_session");
        // Unmarked historical platform tenant-context has no shape identifying
        // delegation; test only the explicitly marked Group/view_as/read_only.
        if historical["tenant_context"] == "group_admin" || historical["view_as"] == true {
            let token = sign(&f, &historical);
            assert!(verifier(&f).verify_access_token(&token).is_ok());
            refused(&f, &f.legacy.router, &token).await;
        }
    }
    let mut historical = claims(&f, &f.legacy.access);
    historical["read_only"] = json!(true);
    let token = sign(&f, &historical);
    assert!(verifier(&f).verify_access_token(&token).is_ok());
    refused(&f, &f.legacy.router, &token).await;
    accepted(&f, &f.legacy.router, &f.access_b, &f.expected).await;
    assert!(before == all_rows(&pool).await);
    f.auth.close().await;
}

#[sqlx::test(migrations = false)]
async fn self_security_read_survives_role_changes_and_company_suspension(pool: PgPool) {
    let f = fixture(&pool).await;
    for roles in [Vec::<String>::new(), vec!["MEMBER".to_owned()]] {
        sqlx::query("UPDATE public.users SET roles=$2 WHERE id=$1")
            .bind(f.legacy.subject.as_uuid())
            .bind(&roles)
            .execute(&pool)
            .await
            .unwrap();
        let actual: Vec<String> = sqlx::query_scalar("SELECT roles FROM public.users WHERE id=$1")
            .bind(f.legacy.subject.as_uuid())
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!(actual == roles);
        let before = all_rows(&pool).await;
        accepted(&f, &f.legacy.router, &f.access_a, &f.expected).await;
        accepted(&f, &f.legacy.router, &f.legacy.access, &f.expected).await;
        assert!(before == all_rows(&pool).await);
    }
    sqlx::query("UPDATE public.organizations SET status='SUSPENDED' WHERE id=$1")
        .bind(OrgId::knl().as_uuid())
        .execute(&pool)
        .await
        .unwrap();
    let status: String = sqlx::query_scalar("SELECT status FROM public.organizations WHERE id=$1")
        .bind(OrgId::knl().as_uuid())
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "SUSPENDED");
    let before = all_rows(&pool).await;
    accepted(&f, &f.legacy.router, &f.access_a, &f.expected).await;
    accepted(&f, &f.legacy.router, &f.legacy.access, &f.expected).await;
    assert!(before == all_rows(&pool).await);
    f.auth.close().await;
}

#[sqlx::test(migrations = false)]
async fn inactive_or_fenced_current_self_refuses_with_unchanged_credentials(pool: PgPool) {
    let f = fixture(&pool).await;
    accepted(&f, &f.legacy.router, &f.access_a, &f.expected).await;
    sqlx::query("UPDATE public.users SET is_active=false WHERE id=$1")
        .bind(f.legacy.subject.as_uuid())
        .execute(&pool)
        .await
        .unwrap();
    let before = all_rows(&pool).await;
    refused(&f, &f.legacy.router, &f.access_a).await;
    refused(&f, &f.legacy.router, &f.legacy.access).await;
    accepted(
        &f,
        &f.legacy.router,
        &f.legacy.control_access,
        &f.control_expected,
    )
    .await;
    assert!(before == all_rows(&pool).await);
    sqlx::query("UPDATE public.users SET is_active=true WHERE id=$1")
        .bind(f.legacy.subject.as_uuid())
        .execute(&pool)
        .await
        .unwrap();
    accepted(&f, &f.legacy.router, &f.access_a, &f.expected).await;
    insert_account_fence(&pool, f.legacy.subject, "ACTIVE").await;
    let fenced = all_rows(&pool).await;
    refused(&f, &f.legacy.router, &f.access_a).await;
    refused(&f, &f.legacy.router, &f.legacy.access).await;
    accepted(
        &f,
        &f.legacy.router,
        &f.legacy.control_access,
        &f.control_expected,
    )
    .await;
    assert!(fenced == all_rows(&pool).await);
    f.auth.close().await;
}

#[sqlx::test(migrations = false)]
async fn historical_platform_self_stays_compatible_but_new_bound_platform_refuses(pool: PgPool) {
    let f = fixture(&pool).await;
    let subject = UserId::new();
    sqlx::query("INSERT INTO public.users(id,display_name,roles,org_id) VALUES($1,'Platform self reader',ARRAY['SUPER_ADMIN'],$2)")
        .bind(subject.as_uuid()).bind(OrgId::platform().as_uuid()).execute(&pool).await.unwrap();
    let key = &f.legacy.signing_key;
    let issuer = JwtIssuer::from_es256_pem(
        JwtSettings {
            issuer: TEST_ISSUER.to_owned(),
            audience: TEST_AUDIENCE.to_owned(),
            access_token_ttl: Duration::minutes(15),
        },
        key.to_pkcs8_pem(LineEnding::LF).unwrap().as_bytes(),
        key.verifying_key()
            .to_public_key_pem(LineEnding::LF)
            .unwrap()
            .as_bytes(),
    )
    .unwrap();
    let unbound = issuer
        .issue_access_token(AccessTokenInput {
            subject,
            org_id: OrgId::platform(),
            roles: vec!["SUPER_ADMIN".to_owned()],
            branches: vec![],
            platform: true,
            view_as: false,
            read_only: false,
            display_name: None,
            feature_grants: vec![],
            authz_subject_version: 0,
            authz_policy_version: 0,
            session_generation: 0,
            issued_at: db_now(&f.auth).await,
        })
        .unwrap();
    let now = db_now(&f.auth).await;
    let mut tx = f.auth.begin().await.unwrap();
    let issued = RefreshTokenStore
        .issue_family_in_tx(
            &mut tx,
            *subject.as_uuid(),
            OrgId::platform(),
            now,
            Duration::hours(1),
        )
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let mut changed = claims(&f, &unbound);
    changed["iat"] = json!(now.unix_timestamp());
    changed["nbf"] = json!(now.unix_timestamp());
    changed["legacy_session"] = json!({"version":1,"family_id":issued.family_id,"home_org":OrgId::platform(),"kind":"direct"});
    let bound = sign(&f, &changed);
    assert!(verifier(&f).verify_access_token(&bound).is_ok());
    let before = all_rows(&pool).await;
    accepted(&f, &f.legacy.router, &unbound, &json!([])).await;
    refused(&f, &f.legacy.router, &bound).await;
    let mut unmarked = claims(&f, &unbound);
    unmarked["platform"] = json!(false);
    unmarked["org"] = json!(OrgId::knl().to_string());
    refused(&f, &f.legacy.router, &sign(&f, &unmarked)).await;
    accepted(&f, &f.legacy.router, &f.access_b, &f.expected).await;
    assert!(before == all_rows(&pool).await);
    f.auth.close().await;
}

#[sqlx::test(migrations = false)]
async fn bound_signed_times_and_historical_expiry_use_fresh_strict_checks(pool: PgPool) {
    let f = fixture(&pool).await;
    let before = all_rows(&pool).await;
    let now = db_now(&f.auth).await.unix_timestamp();
    let original = claims(&f, &f.access_a);
    for (field, value) in [
        ("iat", now + 20),
        ("nbf", now + 20),
        ("exp", now),
        ("exp", i64::MAX),
        ("iat", i64::MIN),
    ] {
        let mut changed = original.clone();
        changed[field] = json!(value);
        refused(&f, &f.legacy.router, &sign(&f, &changed)).await;
    }
    let mut changed = original.clone();
    changed["iat"] = json!(original["iat"].as_i64().unwrap() - 1);
    refused(&f, &f.legacy.router, &sign(&f, &changed)).await;
    let mut historical = claims(&f, &f.legacy.access);
    historical["exp"] = json!(now);
    refused(&f, &f.legacy.router, &sign(&f, &historical)).await;
    let mut historical = claims(&f, &f.legacy.access);
    historical["iat"] = json!(now + 20);
    historical["nbf"] = json!(0);
    let historical = sign(&f, &historical);
    assert!(verifier(&f).verify_access_token(&historical).is_ok());
    accepted(&f, &f.legacy.router, &historical, &f.expected).await;
    accepted(&f, &f.legacy.router, &f.access_b, &f.expected).await;
    assert!(before == all_rows(&pool).await);
    f.auth.close().await;
}

#[sqlx::test(migrations = false)]
async fn existing_server_family_ttl_limits_bound_access_without_rewriting_family(pool: PgPool) {
    let f = fixture(&pool).await;
    let narrow = family_ttl_router(&pool, &f, 1).await;
    let created: OffsetDateTime =
        sqlx::query_scalar("SELECT created_at FROM public.auth_refresh_token_families WHERE id=$1")
            .bind(f.a.family_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        while db_now(&pool).await <= created + Duration::seconds(1) {
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("actual DB family deadline passes within bound");
    let before = all_rows(&pool).await;
    refused(&f, &narrow, &f.access_a).await;
    accepted(&f, &f.legacy.router, &f.access_a, &f.expected).await;
    accepted(&f, &narrow, &f.legacy.access, &f.expected).await;
    assert!(before == all_rows(&pool).await);
    f.auth.close().await;
}

#[sqlx::test(migrations = false)]
async fn missing_actual_projection_returns_503_preserves_rows_and_recovers(pool: PgPool) {
    let f = fixture(&pool).await;
    accepted(&f, &f.legacy.router, &f.access_a, &f.expected).await;
    let key = &f.legacy.signing_key;
    let fault_router = build_router(
        app_state(
            pool.clone(),
            key.to_pkcs8_pem(LineEnding::LF).unwrap().to_string(),
            key.verifying_key()
                .to_public_key_pem(LineEnding::LF)
                .unwrap(),
        )
        .await
        .unwrap()
        .with_auth_database(f.auth.clone()),
    );
    // This admitted Auth pool has issued real families but has not prepared the
    // credential-summary SELECT, so a preexisting cached query cannot hide the
    // intentionally renamed function's missing-name failure.
    let before = all_rows(&pool).await;
    let catalog:Value=sqlx::query_scalar("SELECT to_jsonb(p) FROM pg_catalog.pg_proc p WHERE p.oid='public.auth_legacy_self_passkeys_v1(uuid,uuid)'::regprocedure").fetch_one(&pool).await.unwrap();
    sqlx::query("ALTER FUNCTION public.auth_legacy_self_passkeys_v1(uuid,uuid) RENAME TO passkey_reader_projection_unavailable_fixture").execute(&pool).await.unwrap();
    let outcome = std::panic::AssertUnwindSafe(async {
        let body = response_json(
            get_legacy_raw(&fault_router, PATH, &f.access_a).await,
            StatusCode::SERVICE_UNAVAILABLE,
            &f.secrets(),
            true,
        )
        .await;
        assert!(body == error_body("service_unavailable", "session verification unavailable"));
        refused(&f, &fault_router, "invalid-reader-token").await;
        assert!(before == all_rows(&pool).await);
    })
    .catch_unwind()
    .await;
    sqlx::query("ALTER FUNCTION public.passkey_reader_projection_unavailable_fixture(uuid,uuid) RENAME TO auth_legacy_self_passkeys_v1").execute(&pool).await.unwrap();
    let restored:Value=sqlx::query_scalar("SELECT to_jsonb(p) FROM pg_catalog.pg_proc p WHERE p.oid='public.auth_legacy_self_passkeys_v1(uuid,uuid)'::regprocedure").fetch_one(&pool).await.unwrap();
    assert!(
        catalog == restored,
        "real projection/catalog restored exactly"
    );
    if let Err(panic) = outcome {
        std::panic::resume_unwind(panic);
    }
    accepted(&f, &fault_router, &f.access_a, &f.expected).await;
    assert!(before == all_rows(&pool).await);
    f.auth.close().await;
}

#[path = "legacy_bound_passkey_histories.rs"]
mod histories;
