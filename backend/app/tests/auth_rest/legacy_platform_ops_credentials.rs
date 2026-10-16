//! Ops acceptance through original credentials and the reviewed direct owning API.
//! This proposed API packet is NOT the mounted initial owner-RED admission probe.
use super::super::legacy_platform_list_credentials::{issue_bound, original_claims, signed};
use super::*;
use console_platform_auth::RefreshTokenStore;
use console_platform_authz::platform_policy::PlatformPolicy;
use console_platform_provisioning::{PlatformProvisioner, ProvisioningError, TenantHealth};

fn direct_projection(rows: &[TenantHealth]) -> Value {
    // Serialize the actual owner output, converting only its typed timestamps to
    // the frozen HTTP time encoding. Expected values still come from raw base rows.
    let mut value = json!({"tenants":rows});
    for (record, source) in value["tenants"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .zip(rows)
    {
        record["last_activity_at"] = json!(source.last_activity_at.map(wire_time));
        for (cycle, metric) in record["route_adoption"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .zip(&source.route_adoption)
        {
            cycle["last_event_at"] = json!(wire_time(metric.last_event_at));
        }
    }
    value
}
async fn ops_direct(
    f: &Fixture,
    token: &str,
    ttl: Duration,
) -> Result<Vec<TenantHealth>, ProvisioningError> {
    PlatformProvisioner::new(Duration::hours(1))
        .list_tenant_health(
            &f.business,
            &f.verifier,
            token,
            ttl,
            &PlatformPolicy::compile_current().unwrap(),
        )
        .await
}
async fn owner_positive(pool: &PgPool, f: &Fixture, token: &str, expected: &Value) {
    let before = all_rows(pool).await;
    let start = db_now(pool).await;
    let result = ops_direct(f, token, Duration::days(30)).await.unwrap();
    let end = db_now(pool).await;
    let after = all_rows(pool).await;
    assert!(
        health_matches(&direct_projection(&result), expected),
        "direct owner complete14/6field projection"
    );
    assert!(
        health_read_delta(&before, &after, f.actor, result.len(), start, end),
        "direct owner exact one health audit and no other effects"
    );
}
async fn owner_denied(pool: &PgPool, f: &Fixture, token: &str, ttl: Duration, kind: &str) {
    let before = all_rows(pool).await;
    let result = ops_direct(f, token, ttl).await;
    let after = all_rows(pool).await;
    assert!(
        match kind {
            "unauthorized" => matches!(result, Err(ProvisioningError::PlatformHealthUnauthorized)),
            "forbidden" => matches!(result, Err(ProvisioningError::PlatformHealthForbidden)),
            "unavailable" => matches!(result, Err(ProvisioningError::PlatformHealthUnavailable)),
            _ => false,
        },
        "finite direct ops refusal: {kind}"
    );
    assert!(
        before == after,
        "refused direct owner publishes no rows, audit or other effects"
    );
}
async fn mounted_token(
    pool: &PgPool,
    f: &Fixture,
    token: &str,
    expected_status: StatusCode,
    expected: &Value,
) {
    let before = all_rows(pool).await;
    let start = db_now(pool).await;
    let response = f
        .router
        .clone()
        .oneshot(
            Request::builder()
                .uri(OPS_PATH)
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let end = db_now(pool).await;
    let after = all_rows(pool).await;
    let (status, headers, body) = response_parts(response).await;
    let mut secrets = f.secrets.clone();
    secrets.push(token.to_owned());
    assert!(
        status == expected_status && private_response(&headers, &body, &secrets),
        "mounted ops credential status and privacy"
    );
    let value: Value = serde_json::from_slice(&body).unwrap();
    if expected_status == StatusCode::OK {
        assert!(
            health_matches(&value, expected),
            "mounted exact health/adoption projection"
        );
        assert!(health_read_delta(
            &before,
            &after,
            f.actor,
            expected["tenants"].as_array().unwrap().len(),
            start,
            end
        ));
    } else {
        assert!(
            expected_status == StatusCode::UNAUTHORIZED
                && value
                    == json!({"error":{"code":"unauthorized","message":"invalid bearer token"}}),
            "finite401 without projection"
        );
        assert!(
            before == after,
            "mounted refusal has no audit or other durable effects"
        );
    }
}
async fn populated_fixture(pool: &PgPool) -> (Fixture, Value) {
    let f = fixture(pool).await;
    let _ = seed_ops_content(pool).await;
    let expected = expected_health(pool).await;
    assert!(
        health_matches(&expected, &expected),
        "actual nonempty asymmetric14/6field fixture"
    );
    (f, expected)
}
async fn close_ops(f: Fixture) {
    f.state.shutdown_realtime().await;
    f.business.close().await;
}

#[sqlx::test(migrations = false)]
async fn direct_ops_owner_reverifies_raw_credentials_and_current_role(pool: PgPool) {
    let (f, expected) = populated_fixture(&pool).await;
    owner_positive(&pool, &f, &f.access, &expected).await;
    set_role(&pool, f.actor, "MEMBER").await;
    owner_denied(&pool, &f, &f.access, Duration::days(30), "forbidden").await;
    set_role(&pool, f.actor, "SUPER_ADMIN").await;
    owner_positive(&pool, &f, &f.access, &expected).await;
    let claims = original_claims(&f);
    let wrong_key = SigningKey::random(&mut OsRng);
    let wrong_signature = jsonwebtoken::encode(
        &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::ES256),
        &claims,
        &jsonwebtoken::EncodingKey::from_ec_pem(
            wrong_key.to_pkcs8_pem(LineEnding::LF).unwrap().as_bytes(),
        )
        .unwrap(),
    )
    .unwrap();
    for token in [&wrong_signature, "not.a.valid.jwt", ""] {
        owner_denied(&pool, &f, token, Duration::days(30), "unauthorized").await;
    }
    for fault in [
        "ordinary_company",
        "view_as",
        "read_only",
        "tenant_context",
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
                bad["org"] = json!(OrgId::knl());
                bad["platform"] = json!(false);
            }
            "view_as" => bad["view_as"] = json!(true),
            "read_only" => bad["read_only"] = json!(true),
            "tenant_context" => {
                bad["tenant_context"] =
                    json!({"target_org":OrgId::knl(),"context_id":Uuid::new_v4()})
            }
            "group_context" => bad["group_context_id"] = json!(Uuid::new_v4()),
            "actor_home" => bad["actor_home_org"] = json!(OrgId::platform()),
            "scope_level" => bad["scope_level"] = json!("GROUP"),
            "nil_subject" => bad["sub"] = json!(Uuid::nil()),
            "null_binding" => bad["legacy_session"] = Value::Null,
            "empty_binding" => bad["legacy_session"] = json!({}),
            "future_binding_version" => {
                bad["legacy_session"] = json!({"version":2,"family_id":Uuid::new_v4(),"home_org":OrgId::platform(),"kind":"direct"})
            }
            _ => unreachable!(),
        }
        owner_denied(
            &pool,
            &f,
            &signed(&f, &bad),
            Duration::days(30),
            "unauthorized",
        )
        .await;
    }
    for ttl in [Duration::ZERO, Duration::seconds(-1)] {
        owner_denied(&pool, &f, &f.access, ttl, "unavailable").await;
        // Original signed-source verification takes precedence over invalid serverTTL.
        owner_denied(&pool, &f, &wrong_signature, ttl, "unauthorized").await;
    }
    owner_positive(&pool, &f, &f.access, &expected).await;
    close_ops(f).await;
}

#[sqlx::test(migrations = false)]
async fn bound_ops_rotation_preserves_access_and_exact_logout_refuses_only_selected_family(
    pool: PgPool,
) {
    let (f, expected) = populated_fixture(&pool).await;
    let auth = login_test_pool(&pool, TestDatabaseLogin::Auth).await;
    let (a, access_a, _) = issue_bound(&pool, &f, &auth).await;
    let (b, access_b, _) = issue_bound(&pool, &f, &auth).await;
    assert!(
        a.family_id != b.family_id,
        "two distinct real selected families"
    );
    owner_positive(&pool, &f, &access_a, &expected).await;
    mounted_token(&pool, &f, &access_a, StatusCode::OK, &expected).await;
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
    assert!(
        rotated.family_id == a.family_id && rotated.token_id != a.token_id,
        "canonical rotation keeps exact family"
    );
    let rotated_exact:bool=sqlx::query_scalar("SELECT f.revoked_at IS NULL AND old.used_at IS NOT NULL AND old.replaced_by=$2 AND new.used_at IS NULL AND new.revoked_at IS NULL FROM public.auth_refresh_token_families f JOIN public.auth_refresh_tokens old ON old.family_id=f.id JOIN public.auth_refresh_tokens new ON new.family_id=f.id AND new.id=$2 WHERE f.id=$1 AND old.id=$3")
        .bind(a.family_id).bind(rotated.token_id).bind(a.token_id).fetch_one(&pool).await.unwrap();
    assert!(rotated_exact, "persisted real rotation readback");
    owner_positive(&pool, &f, &access_a, &expected).await;
    mounted_token(&pool, &f, &access_a, StatusCode::OK, &expected).await;
    let revoked_at = db_now(&pool).await;
    RefreshTokenStore
        .revoke_family_for_logout(&auth, rotated.token.as_str(), revoked_at)
        .await
        .unwrap();
    let revoked_exact:bool=sqlx::query_scalar("SELECT a.revoked_at=$3 AND a.revoked_reason='logout' AND b.revoked_at IS NULL AND b.revoked_reason IS NULL FROM public.auth_refresh_token_families a JOIN public.auth_refresh_token_families b ON b.id=$2 WHERE a.id=$1")
        .bind(a.family_id).bind(b.family_id).bind(revoked_at).fetch_one(&pool).await.unwrap();
    assert!(
        revoked_exact,
        "committed exact selected-family revocation and live sibling"
    );
    owner_denied(&pool, &f, &access_a, Duration::days(30), "unauthorized").await;
    mounted_token(&pool, &f, &access_a, StatusCode::UNAUTHORIZED, &expected).await;
    owner_positive(&pool, &f, &access_b, &expected).await;
    mounted_token(&pool, &f, &access_b, StatusCode::OK, &expected).await;
    owner_positive(&pool, &f, &f.access, &expected).await;
    mounted_token(&pool, &f, &f.access, StatusCode::OK, &expected).await;
    auth.close().await;
    close_ops(f).await;
}

#[sqlx::test(migrations = false)]
async fn direct_and_mounted_ops_present_family_faults_do_not_fall_back_to_historical(pool: PgPool) {
    let (f, expected) = populated_fixture(&pool).await;
    let auth = login_test_pool(&pool, TestDatabaseLogin::Auth).await;
    let (family, access, created) = issue_bound(&pool, &f, &auth).await;
    owner_positive(&pool, &f, &access, &expected).await;
    mounted_token(&pool, &f, &access, StatusCode::OK, &expected).await;
    assert!(
        db_now(&pool).await > created,
        "actual family age prerequisite"
    );
    owner_denied(&pool, &f, &access, Duration::nanoseconds(1), "unauthorized").await;
    // A real live family owned by a different platform User/Account cannot be borrowed.
    let other = UserId::new();
    sqlx::query("INSERT INTO public.users(id,display_name,roles,org_id) VALUES($1,'다른 플랫폼 가족 소유자',ARRAY['SUPER_ADMIN'],$2)")
        .bind(other.as_uuid()).bind(OrgId::platform().as_uuid()).execute(&pool).await.unwrap();
    let root_present: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM public.accounts WHERE id=$1)")
            .bind(other.as_uuid())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(root_present, "automatic distinct Account root prerequisite");
    let foreign_created = db_now(&pool).await;
    let mut tx = auth.begin().await.unwrap();
    let foreign = RefreshTokenStore
        .issue_family_in_tx(
            &mut tx,
            *other.as_uuid(),
            OrgId::platform(),
            foreign_created,
            Duration::hours(1),
        )
        .await
        .unwrap();
    tx.commit().await.unwrap();
    assert!(
        foreign.family_id != family.family_id && foreign.user_id == *other.as_uuid(),
        "canonical foreign family belongs to another subject"
    );
    let foreign_exact: bool = sqlx::query_scalar("SELECT user_id=$2 AND org_id=$3 AND protocol='LEGACY_COMPANY' AND created_at=$4 AND revoked_at IS NULL FROM public.auth_refresh_token_families WHERE id=$1")
        .bind(foreign.family_id).bind(other.as_uuid()).bind(OrgId::platform().as_uuid()).bind(foreign_created)
        .fetch_one(&pool).await.unwrap();
    assert!(foreign_exact, "persisted live foreign family prerequisite");
    let claims = serde_json::to_value(f.verifier.verify_access_token(&access).unwrap()).unwrap();
    let mut foreign_claims = claims.clone();
    foreign_claims["sub"] = json!(other);
    foreign_claims["legacy_session"]["family_id"] = json!(foreign.family_id);
    foreign_claims["iat"] = json!(foreign_created.unix_timestamp());
    foreign_claims["nbf"] = foreign_claims["iat"].clone();
    let before = all_rows(&pool).await;
    let start = db_now(&pool).await;
    let foreign_result = ops_direct(&f, &signed(&f, &foreign_claims), Duration::days(30))
        .await
        .unwrap();
    let end = db_now(&pool).await;
    assert!(
        health_matches(&direct_projection(&foreign_result), &expected)
            && health_read_delta(
                &before,
                &all_rows(&pool).await,
                other,
                foreign_result.len(),
                start,
                end
            ),
        "foreign family is actually usable by its exact owning subject before borrowed-family refusal"
    );
    for fault in [
        "expired",
        "future_iat",
        "future_nbf",
        "inverted",
        "before_family",
        "unknown_family",
        "foreign_subject_family",
        "wrong_binding_home",
        "derived_binding",
        "nil_family",
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
            "foreign_subject_family" => {
                bad["legacy_session"]["family_id"] = json!(foreign.family_id);
                bad["iat"] = json!(foreign_created.unix_timestamp());
                bad["nbf"] = bad["iat"].clone();
            }
            "wrong_binding_home" => bad["legacy_session"]["home_org"] = json!(OrgId::knl()),
            "derived_binding" => bad["legacy_session"]["kind"] = json!("platform_view_as"),
            "nil_family" => bad["legacy_session"]["family_id"] = json!(Uuid::nil()),
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
        let token = signed(&f, &bad);
        owner_denied(&pool, &f, &token, Duration::days(30), "unauthorized").await;
        mounted_token(&pool, &f, &token, StatusCode::UNAUTHORIZED, &expected).await;
    }
    owner_positive(&pool, &f, &access, &expected).await;
    mounted_token(&pool, &f, &access, StatusCode::OK, &expected).await;
    auth.close().await;
    close_ops(f).await;
}

#[path = "legacy_platform_ops_transport.rs"]
mod legacy_platform_ops_transport;

#[path = "legacy_platform_ops_current_source.rs"]
mod legacy_platform_ops_current_source;

#[path = "legacy_platform_ops_family_histories.rs"]
mod legacy_platform_ops_family_histories;

#[path = "legacy_platform_ops_expiry_progress.rs"]
mod legacy_platform_ops_expiry_progress;

#[path = "legacy_platform_ops_commit_loss.rs"]
mod legacy_platform_ops_commit_loss;
