//! Canonical family races and final audit-wait expiry for the separate ops owner.
use super::super::super::legacy_platform_list_credentials::{
    elapsed, revocation_delta, role_wait, rows,
};
use super::legacy_platform_ops_transport::ops_task;
use super::*;
use console_platform_auth::RefreshTokenUseError;

pub(super) async fn assert_ops_response(
    f: &Fixture,
    token: &str,
    response: http::Response<Body>,
    expected_status: StatusCode,
    expected: &Value,
) {
    let (status, headers, body) = response_parts(response).await;
    let mut secrets = f.secrets.clone();
    secrets.push(token.to_owned());
    assert!(status == expected_status && private_response(&headers, &body, &secrets));
    let value: Value = serde_json::from_slice(&body).unwrap();
    if expected_status == StatusCode::OK {
        assert!(health_matches(&value, expected));
    } else {
        assert!(
            expected_status == StatusCode::UNAUTHORIZED
                && value
                    == json!({"error":{"code":"unauthorized","message":"invalid bearer token"}})
        );
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Preserve explicit independent source/family/event/read witnesses in the existing combined oracle"
)]
fn health_revocation_delta(
    before: &Rows,
    after: &Rows,
    actor: UserId,
    family: Uuid,
    token: Uuid,
    count: usize,
    reason: &str,
    now: OffsetDateTime,
    start: OffsetDateTime,
    end: OffsetDateTime,
    read: bool,
) -> bool {
    if !read {
        return revocation_delta(
            before, after, actor, family, token, count, reason, now, start, end, false,
        );
    }
    let (Some(old), Some(new)) = (before.get("audit_events"), after.get("audit_events")) else {
        return false;
    };
    let (Ok(old), Ok(new)) = (
        serde_json::from_str::<Vec<&RawValue>>(old),
        serde_json::from_str::<Vec<&RawValue>>(new),
    ) else {
        return false;
    };
    let prior: BTreeSet<_> = old.iter().map(|row| row.get()).collect();
    let mut mapped = Vec::new();
    let mut health_reads = 0;
    for raw in new {
        let Ok(mut event) = serde_json::from_str::<Value>(raw.get()) else {
            return false;
        };
        if !prior.contains(raw.get()) && event["action"] == "platform.tenant.health" {
            // Validate ALL20 original health fields and actual timestamps first.
            // Adapt only this already-validated new event's two discriminator fields.
            if !health_audit_matches(&event, actor, count, start, end) {
                return false;
            }
            health_reads += 1;
            event["action"] = json!("platform.tenant.list");
            event["target_id"] = json!("list");
            mapped.push(event.to_string());
        } else {
            mapped.push(raw.get().to_owned());
        }
    }
    if health_reads != 1 {
        return false;
    }
    let mut adapted = after.clone();
    adapted.insert("audit_events".to_owned(), format!("[{}]", mapped.join(",")));
    // Unchanged original oracle still checks every revocation/token/sibling field,
    // every original historical audit byte, exact event count and all other tables.
    revocation_delta(
        before, &adapted, actor, family, token, count, reason, now, start, end, true,
    )
}

async fn revoker_first(pool: PgPool, reuse: bool) {
    let (f, expected) = populated_fixture(&pool).await;
    let auth = login_test_pool(&pool, TestDatabaseLogin::Auth).await;
    let (a, access, _) = issue_bound(&pool, &f, &auth).await;
    let (_, sibling, _) = issue_bound(&pool, &f, &auth).await;
    owner_positive(&pool, &f, &access, &expected).await;
    if reuse {
        let next = RefreshTokenStore
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
        assert!(next.family_id == a.family_id && next.token_id != a.token_id);
    }
    let before = all_rows(&pool).await;
    let start = db_now(&pool).await;
    let mut observer = pool.acquire().await.unwrap();
    let mut revoker = auth.begin().await.unwrap();
    let blocker: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *revoker)
        .await
        .unwrap();
    let now = db_now(&pool).await;
    if reuse {
        let result = RefreshTokenStore
            .rotate_in_tx(
                &mut revoker,
                a.token.as_str(),
                now,
                Duration::hours(1),
                Duration::days(30),
            )
            .await
            .unwrap();
        assert!(
            matches!(result, Err(RefreshTokenUseError::ReuseDetected)),
            "canonical reused-token terminal result"
        );
    } else {
        RefreshTokenStore
            .revoke_family_for_logout_in_tx(&mut revoker, a.token.as_str(), now)
            .await
            .unwrap();
    }
    let reader = ops_task(&f.router, &access, Method::GET);
    let reader_pid = role_wait(
        &pool,
        blocker,
        "console_rt",
        "auth_legacy_platform_source_material_v1",
    )
    .await;
    let pending = !reader.is_finished();
    revoker.commit().await.unwrap();
    let result = finish(reader).await;
    let pids: Vec<_> = std::iter::once(blocker).chain(reader_pid).collect();
    let clean = clean_pids(&mut observer, &pids).await;
    let end = db_now(&pool).await;
    let after = all_rows(&pool).await;
    owner_positive(&pool, &f, &sibling, &expected).await;
    assert_ops_response(
        &f,
        &access,
        result.unwrap(),
        StatusCode::UNAUTHORIZED,
        &expected,
    )
    .await;
    assert!(
        waited_cleanly(blocker, reader_pid, pending, clean),
        "actual owner waits for canonical revocation commit"
    );
    assert!(
        health_revocation_delta(
            &before,
            &after,
            f.actor,
            a.family_id,
            a.token_id,
            expected["tenants"].as_array().unwrap().len(),
            if reuse { "reuse_detected" } else { "logout" },
            now,
            start,
            end,
            false
        ),
        "only exact canonical revocation effects; no ops audit"
    );
    auth.close().await;
    close_ops(f).await;
}
#[sqlx::test(migrations = false)]
async fn ops_revoker_first_logout_retains_source_until_commit(pool: PgPool) {
    revoker_first(pool, false).await;
}
#[sqlx::test(migrations = false)]
async fn ops_revoker_first_reuse_retains_source_until_commit(pool: PgPool) {
    revoker_first(pool, true).await;
}

#[sqlx::test(migrations = false)]
async fn ops_reader_first_holds_subject_and_exact_family_through_adoption_commit(pool: PgPool) {
    let (f, expected) = populated_fixture(&pool).await;
    let auth = login_test_pool(&pool, TestDatabaseLogin::Auth).await;
    let (a, access, _) = issue_bound(&pool, &f, &auth).await;
    let (_, sibling, _) = issue_bound(&pool, &f, &auth).await;
    owner_positive(&pool, &f, &access, &expected).await;
    let before = all_rows(&pool).await;
    let start = db_now(&pool).await;
    let mut observer = pool.acquire().await.unwrap();
    let mut projection = pool.begin().await.unwrap();
    let blocker: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *projection)
        .await
        .unwrap();
    sqlx::query("LOCK TABLE public.console_route_telemetry IN ACCESS EXCLUSIVE MODE")
        .execute(&mut *projection)
        .await
        .unwrap();
    let reader = ops_task(&f.router, &access, Method::GET);
    let reader_pid = role_wait(
        &pool,
        blocker,
        "console_rt",
        "platform_console_route_adoption",
    )
    .await;
    let family = a.family_id;
    let family_pool = auth.clone();
    let family_probe = tokio::spawn(async move {
        let mut tx = family_pool.begin().await.unwrap();
        sqlx::query("SELECT set_config('app.current_org',$1,true)")
            .bind(OrgId::platform().to_string())
            .execute(&mut *tx)
            .await
            .unwrap();
        let id: Uuid = sqlx::query_scalar(
            "SELECT id FROM public.auth_refresh_token_families WHERE id=$1 FOR UPDATE",
        )
        .bind(family)
        .fetch_one(&mut *tx)
        .await
        .unwrap();
        tx.rollback().await.unwrap();
        id
    });
    let family_wait = if let Some(pid) = reader_pid {
        role_wait(&pool, pid, "console_auth_rt", "auth_refresh_token_families").await
    } else {
        None
    };
    let now = db_now(&pool).await;
    let revoke_pool = auth.clone();
    let token = a.token.as_str().to_owned();
    let revoker = tokio::spawn(async move {
        RefreshTokenStore
            .revoke_family_for_logout(&revoke_pool, &token, now)
            .await
    });
    let revoke_wait = if let Some(pid) = reader_pid {
        role_wait(
            &pool,
            pid,
            "console_auth_rt",
            "account_company_deactivation_guard_v1",
        )
        .await
    } else {
        None
    };
    let pending = !reader.is_finished() && !family_probe.is_finished() && !revoker.is_finished();
    projection.rollback().await.unwrap();
    let (result, probed, revoked) =
        tokio::join!(finish(reader), finish(family_probe), finish(revoker));
    let pids: Vec<_> = std::iter::once(blocker)
        .chain(reader_pid)
        .chain(family_wait)
        .chain(revoke_wait)
        .collect();
    let clean = clean_pids(&mut observer, &pids).await;
    let end = db_now(&pool).await;
    let after = all_rows(&pool).await;
    assert!(probed.unwrap() == a.family_id && revoked.unwrap().is_ok());
    assert_ops_response(&f, &access, result.unwrap(), StatusCode::OK, &expected).await;
    assert!(
        reader_pid.is_some() && family_wait.is_some() && revoke_wait.is_some() && pending && clean,
        "observed adoption, exact-family SHARE and canonical source-guard exclusion required"
    );
    assert!(
        health_revocation_delta(
            &before,
            &after,
            f.actor,
            a.family_id,
            a.token_id,
            expected["tenants"].as_array().unwrap().len(),
            "logout",
            now,
            start,
            end,
            true
        ),
        "one completed health audit plus one exact canonical logout"
    );
    owner_denied(&pool, &f, &access, Duration::days(30), "unauthorized").await;
    owner_positive(&pool, &f, &sibling, &expected).await;
    auth.close().await;
    close_ops(f).await;
}

async fn audit_expiry(pool: PgPool, bound: bool) {
    let (f, expected) = populated_fixture(&pool).await;
    let auth = login_test_pool(&pool, TestDatabaseLogin::Auth).await;
    let (_, bound_access, _) = issue_bound(&pool, &f, &auth).await;
    let original = if bound { &bound_access } else { &f.access };
    owner_positive(&pool, &f, original, &expected).await;
    let mut claims =
        serde_json::to_value(f.verifier.verify_access_token(original).unwrap()).unwrap();
    let expires = db_now(&pool).await.unix_timestamp() + 5;
    claims["exp"] = json!(expires);
    let access = signed(&f, &claims);
    let before = all_rows(&pool).await;
    let mut observer = pool.acquire().await.unwrap();
    let mut blocker = pool.begin().await.unwrap();
    let pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *blocker)
        .await
        .unwrap();
    // SHARE admits reads but conflicts with the actual required audit INSERT RowExclusive lock.
    sqlx::query("LOCK TABLE public.audit_events IN SHARE MODE")
        .execute(&mut *blocker)
        .await
        .unwrap();
    let reader = ops_task(&f.router, &access, Method::GET);
    let waiter = role_wait(&pool, pid, "console_rt", "INSERT INTO audit_events").await;
    let pending = !reader.is_finished();
    let expired = elapsed(&pool, OffsetDateTime::from_unix_timestamp(expires).unwrap()).await;
    blocker.rollback().await.unwrap();
    let result = finish(reader).await;
    let pids: Vec<_> = std::iter::once(pid).chain(waiter).collect();
    let clean = clean_pids(&mut observer, &pids).await;
    let after = all_rows(&pool).await;
    assert_ops_response(
        &f,
        &access,
        result.unwrap(),
        StatusCode::UNAUTHORIZED,
        &expected,
    )
    .await;
    assert!(
        waited_cleanly(pid, waiter, pending, clean) && expired,
        "audit INSERT wait and real DB expiry must precede release"
    );
    assert!(
        before == after,
        "post-audit lifetime refusal rolls back appended audit and all effects"
    );
    owner_positive(&pool, &f, original, &expected).await;
    auth.close().await;
    close_ops(f).await;
}
#[sqlx::test(migrations = false)]
async fn historical_ops_expiry_during_actual_audit_insert_discards_projection(pool: PgPool) {
    audit_expiry(pool, false).await;
}
#[sqlx::test(migrations = false)]
async fn bound_ops_expiry_during_actual_audit_insert_discards_projection(pool: PgPool) {
    audit_expiry(pool, true).await;
}

#[test]
fn health_combined_revocation_oracle_preserves_every_effect_and_original_audit_field() {
    let actor = UserId::new();
    let family = Uuid::new_v4();
    let sibling = Uuid::new_v4();
    let token = Uuid::new_v4();
    let sibling_token = Uuid::new_v4();
    let now = OffsetDateTime::from_unix_timestamp(1_789_862_400).unwrap();
    let stamp = now
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap();
    let mut prior = audit_fixture(actor, now);
    prior["action"] = json!("platform.tenant.health");
    prior["target_id"] = json!("health");
    prior["trace_id"] = json!("abcdef1234567890abcdef1234567890");
    let before=Rows::from([
        ("users".to_owned(),"[{\"retained\":true}]".to_owned()),
        ("auth_refresh_token_families".to_owned(),json!([
            {"id":family,"revoked_at":null,"revoked_reason":null,"retained":"original"},
            {"id":sibling,"revoked_at":null,"revoked_reason":null,"retained":"sibling"}]).to_string()),
        ("auth_refresh_tokens".to_owned(),json!([
            {"id":token,"family_id":family,"revoked_at":null,"reuse_detected_at":null,"retained":"original"},
            {"id":sibling_token,"family_id":sibling,"revoked_at":null,"reuse_detected_at":null,"retained":"sibling"}]).to_string()),
        ("audit_events".to_owned(),json!([prior.clone()]).to_string()),
    ]);
    for reason in ["logout", "reuse_detected"] {
        for read in [false, true] {
            let mut after = before.clone();
            let mut families = rows(&before, "auth_refresh_token_families");
            families[0]["revoked_at"] = json!(stamp);
            families[0]["revoked_reason"] = json!(reason);
            after.insert(
                "auth_refresh_token_families".to_owned(),
                json!(families).to_string(),
            );
            let mut tokens = rows(&before, "auth_refresh_tokens");
            tokens[0]["revoked_at"] = json!(stamp);
            if reason == "reuse_detected" {
                tokens[0]["reuse_detected_at"] = json!(stamp);
            }
            after.insert("auth_refresh_tokens".to_owned(), json!(tokens).to_string());
            let mut event = audit_fixture(actor, now);
            event["trace_id"] = json!("fedcba1234567890fedcba1234567890");
            event["action"] = json!(if reason == "logout" {
                "auth.logout"
            } else {
                "auth.refresh.reuse_detected"
            });
            event["target_type"] = json!("auth_refresh_token_family");
            event["target_id"] = json!(family.to_string());
            event["org_id"] = json!(OrgId::platform());
            event["after_snap"] = if reason == "logout" {
                json!({"family_id":family,"revoked_reason":reason})
            } else {
                json!({"family_id":family,"revoked_reason":reason,"reused_token_id":token})
            };
            let mut audit = vec![prior.clone(), event];
            if read {
                let mut health = audit_fixture(actor, now);
                health["action"] = json!("platform.tenant.health");
                health["target_id"] = json!("health");
                audit.push(health);
            }
            after.insert("audit_events".to_owned(), json!(audit).to_string());
            let check = |actual: &Rows| {
                health_revocation_delta(
                    &before,
                    actual,
                    actor,
                    family,
                    token,
                    2,
                    reason,
                    now,
                    now - Duration::seconds(1),
                    now + Duration::seconds(1),
                    read,
                )
            };
            assert!(check(&after), "positive combined oracle");
            let mut rewritten_history = after.clone();
            rewritten_history.insert(
                "audit_events".to_owned(),
                format!(
                    "[{},{}]",
                    serde_json::to_string_pretty(&prior).unwrap(),
                    audit
                        .iter()
                        .skip(1)
                        .map(Value::to_string)
                        .collect::<Vec<_>>()
                        .join(",")
                ),
            );
            assert!(
                !check(&rewritten_history),
                "historical health audit raw bytes must not be normalized"
            );
            if read {
                let mut list_spoof = audit.clone();
                list_spoof.last_mut().unwrap()["action"] = json!("platform.tenant.list");
                list_spoof.last_mut().unwrap()["target_id"] = json!("list");
                let mut wrong_kind = after.clone();
                wrong_kind.insert("audit_events".to_owned(), json!(list_spoof).to_string());
                assert!(
                    !check(&wrong_kind),
                    "valid list-shaped audit is not a health event"
                );
            }

            assert!(!check(&before), "omitted revocation");
            for table in [
                "auth_refresh_token_families",
                "auth_refresh_tokens",
                "audit_events",
            ] {
                let mut bad = after.clone();
                bad.insert(table.to_owned(), before[table].clone());
                assert!(!check(&bad), "omitted required effects");
                let mut bad = after.clone();
                bad.insert(table.to_owned(), "[]".to_owned());
                assert!(!check(&bad), "deleted history");
            }
            let mut bad = after.clone();
            bad.insert("users".to_owned(), "[]".to_owned());
            assert!(!check(&bad));
            let mut bad = after.clone();
            bad.remove("users");
            assert!(!check(&bad));
            for table in ["auth_refresh_token_families", "auth_refresh_tokens"] {
                for index in [0, 1] {
                    let mut values = rows(&after, table);
                    values[index]["retained"] = json!("corrupt");
                    let mut bad = after.clone();
                    bad.insert(table.to_owned(), json!(values).to_string());
                    assert!(!check(&bad));
                }
            }
            for index in 1..audit.len() {
                for key in AUDIT_KEYS {
                    let mut values = audit.clone();
                    values[index].as_object_mut().unwrap().remove(*key);
                    let mut bad = after.clone();
                    bad.insert("audit_events".to_owned(), json!(values).to_string());
                    assert!(!check(&bad), "omitted audit field");
                    let mut values = audit.clone();
                    values[index][*key] = json!("corrupt");
                    let mut bad = after.clone();
                    bad.insert("audit_events".to_owned(), json!(values).to_string());
                    assert!(!check(&bad), "corrupt audit field");
                }
            }
            for key in ["id", "trace_id"] {
                let mut values = audit.clone();
                values[1][key] = values[0][key].clone();
                let mut bad = after.clone();
                bad.insert("audit_events".to_owned(), json!(values).to_string());
                assert!(!check(&bad), "reused audit identity");
            }
        }
    }
}
