//! V7 caller REDs against real HTTP, current source, and issued legacy families.
use super::legacy_platform_list_credentials::{issue_bound, no_raw_input_echo};
use super::*;
use console_platform_auth::RefreshTokenStore;

fn path(target: Uuid, force: bool) -> String {
    format!("/api/platform/orgs/{target}?delete_data={force}")
}

fn removal(
    f: &Fixture,
    token: &str,
    target: Uuid,
    force: bool,
) -> JoinHandle<http::Response<Body>> {
    let router = f.router.clone();
    let token = token.to_owned();
    tokio::spawn(async move {
        router
            .oneshot(
                Request::builder()
                    .method(Method::DELETE)
                    .uri(path(target, force))
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap()
    })
}

async fn private_error(
    response: http::Response<Body>,
    status: StatusCode,
    code: &str,
    secrets: &[String],
) -> bool {
    let (actual, headers, body) = response_parts(response).await;
    let value = serde_json::from_slice::<Value>(&body).ok();
    let inputs: Vec<_> = secrets
        .iter()
        .map(|value| value.as_bytes().to_vec())
        .collect();
    actual == status
        && no_raw_input_echo(&headers, &body, &inputs)
        && value.is_some_and(|v| {
            v.as_object().is_some_and(|fields| fields.len() == 1)
                && v["error"]
                    .as_object()
                    .is_some_and(|fields| fields.len() == 2)
                && v["error"]["code"] == code
                && v["error"]["message"].as_str().is_some_and(|text| {
                    !text.is_empty()
                        && text.len() <= 256
                        && !text.contains("SQLSTATE")
                        && !text.contains("auth_legacy")
                        && !text.contains("platform_topology")
                        && !text.contains("platform_force")
                        && !text.contains("permission denied")
                        && !text.contains("public.")
                })
        })
}

async fn missing_target(pool: &PgPool) -> Uuid {
    let target = Uuid::new_v4();
    let absent: bool =
        sqlx::query_scalar("SELECT NOT EXISTS(SELECT 1 FROM public.organizations WHERE id=$1)")
            .bind(target)
            .fetch_one(pool)
            .await
            .unwrap();
    assert!(absent && !target.is_nil() && target != *OrgId::platform().as_uuid());
    target
}

async fn owner_wait(pool: &PgPool, blocker: i32, force: bool) -> Option<i32> {
    let role = if force {
        "console_platform_force_cmd"
    } else {
        "console_rt"
    };
    let old = if force {
        "platform_force_remove_organization_command("
    } else {
        "platform_get_organization("
    };
    let current = if force {
        "platform_prepare_force_organization_removal_v1("
    } else {
        "platform_prepare_organization_removal_v1("
    };
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        // Auth middleware uses console_auth_rt; only a dispatched owner can use this LOGIN.
        let pid: Option<i32> = sqlx::query_scalar("SELECT pid FROM pg_catalog.pg_stat_activity WHERE datname=current_database() AND usename=$2 AND state='active' AND wait_event_type='Lock' AND $1=ANY(pg_catalog.pg_blocking_pids(pid)) AND (strpos(query,$3)>0 OR strpos(query,$4)>0) ORDER BY pid LIMIT 1")
            .bind(blocker).bind(role).bind(old).bind(current).fetch_optional(pool).await.unwrap();
        if pid.is_some() || tokio::time::Instant::now() >= deadline {
            return pid;
        }
        tokio::time::sleep(std::time::Duration::from_millis(15)).await;
    }
}

#[sqlx::test(migrations = false)]
async fn mounted_removal_rechecks_current_policy_after_actual_owner_wait(pool: PgPool) {
    let f = fixture(&pool).await;
    let auth = login_test_pool(&pool, TestDatabaseLogin::Auth).await;
    let (_, access, _) = issue_bound(&pool, &f, &auth).await;
    let mut secrets = f.secrets.clone();
    secrets.push(access.clone());
    let mut findings = Vec::new();
    let mut prerequisites = Vec::new();
    for force in [false, true] {
        let target = missing_target(&pool).await;
        let before = all_rows(&pool).await;
        let allowed = finish(removal(&f, &access, target, force)).await.unwrap();
        let positive = private_error(allowed, StatusCode::NOT_FOUND, "not_found", &secrets).await;
        let positive_unchanged = before == all_rows(&pool).await;
        assert!(
            positive && positive_unchanged,
            "V7_CALLER_PREREQUISITE: valid bound bearer reaches actual no-effect missing-target404"
        );
        let mut observer = pool.acquire().await.unwrap();
        let mut barrier = pool.begin().await.unwrap();
        let blocker: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut *barrier)
            .await
            .unwrap();
        sqlx::query("LOCK TABLE public.organizations IN ACCESS EXCLUSIVE MODE")
            .execute(&mut *barrier)
            .await
            .unwrap();
        let request = removal(&f, &access, target, force);
        let waiter = owner_wait(&pool, blocker, force).await;
        let pending = !request.is_finished();
        set_role(&pool, f.actor, "MEMBER").await;
        barrier.rollback().await.unwrap();
        let response = finish(request).await;
        let denied_rows = all_rows(&pool).await;
        set_role(&pool, f.actor, "SUPER_ADMIN").await;
        let restored = before == all_rows(&pool).await;
        let recovery = finish(removal(&f, &access, target, force)).await.unwrap();
        let recovered = private_error(recovery, StatusCode::NOT_FOUND, "not_found", &secrets).await
            && before == all_rows(&pool).await;
        let pids: Vec<_> = std::iter::once(blocker).chain(waiter).collect();
        let clean = clean_pids(&mut observer, &pids).await;
        let (denied, actual_status) = match response {
            Ok(response) => {
                let status = response.status().as_u16();
                (
                    private_error(response, StatusCode::FORBIDDEN, "forbidden", &secrets).await,
                    Some(status),
                )
            }
            Err(_) => (false, None),
        };
        if !waited_cleanly(blocker, waiter, pending, clean) || actual_status.is_none() {
            prerequisites.push(format!(
                "force={force} blocker={blocker} waiter={waiter:?} pending={pending} clean={clean}"
            ));
        }
        let role_delta = only_role_changed(&before, &denied_rows, f.actor);
        if !(role_delta && restored && recovered && denied) {
            findings.push(format!("force={force} actual_status={actual_status:?} private403={denied} role_delta={role_delta} restored={restored} recovered={recovered}"));
        }
    }
    auth.close().await;
    f.state.shutdown_realtime().await;
    f.business.close().await;
    assert!(
        prerequisites.is_empty(),
        "V7_CALLER_PREREQUISITE: {}",
        prerequisites.join("; ")
    );
    assert!(findings.is_empty(), "{}", findings.join("; "));
}

#[sqlx::test(migrations = false)]
async fn mounted_removal_refuses_actual_logged_out_family_and_preserves_live_sibling(pool: PgPool) {
    let f = fixture(&pool).await;
    let auth = login_test_pool(&pool, TestDatabaseLogin::Auth).await;
    let (a, access_a, _) = issue_bound(&pool, &f, &auth).await;
    let (b, access_b, _) = issue_bound(&pool, &f, &auth).await;
    assert_ne!(a.family_id, b.family_id);
    let mut secrets = f.secrets.clone();
    secrets.extend([access_a.clone(), access_b.clone()]);
    let target = missing_target(&pool).await;
    let initial = all_rows(&pool).await;
    for force in [false, true] {
        for access in [&access_a, &access_b] {
            let response = finish(removal(&f, access, target, force)).await.unwrap();
            assert!(
                private_error(response, StatusCode::NOT_FOUND, "not_found", &secrets).await
                    && initial == all_rows(&pool).await,
                "V7_CALLER_PREREQUISITE: both genuine live families reach unchanged404 before logout"
            );
        }
    }
    let revoked_at = db_now(&pool).await;
    RefreshTokenStore
        .revoke_family_for_logout(&auth, a.token.as_str(), revoked_at)
        .await
        .unwrap();
    let exact: bool = sqlx::query_scalar("SELECT a.user_id=$3 AND b.user_id=$3 AND a.revoked_at=$4 AND a.revoked_reason='logout' AND b.revoked_at IS NULL AND b.revoked_reason IS NULL FROM public.auth_refresh_token_families a JOIN public.auth_refresh_token_families b ON b.id=$2 WHERE a.id=$1")
        .bind(a.family_id).bind(b.family_id).bind(f.actor.as_uuid()).bind(revoked_at).fetch_one(&pool).await.unwrap();
    assert!(
        exact,
        "actual Auth owner revoked exactly the selected issued family"
    );
    let before = all_rows(&pool).await;
    let mut findings = Vec::new();
    for force in [false, true] {
        for (access, expected, code) in [
            (&access_a, StatusCode::UNAUTHORIZED, "unauthorized"),
            (&access_b, StatusCode::NOT_FOUND, "not_found"),
        ] {
            let response = finish(removal(&f, access, target, force)).await.unwrap();
            let actual_status = response.status();
            let valid = private_error(response, expected, code, &secrets).await;
            let unchanged = before == all_rows(&pool).await;
            if !(valid && unchanged) {
                findings.push(format!(
                    "force={force} expected={expected} actual_status={actual_status} private_match={valid} unchanged={unchanged}"
                ));
            }
        }
    }
    auth.close().await;
    f.state.shutdown_realtime().await;
    f.business.close().await;
    assert!(findings.is_empty(), "{}", findings.join("; "));
}

#[sqlx::test(migrations = false)]
async fn mounted_removal_rejects_ambiguous_bearer_before_any_effect(pool: PgPool) {
    let f = fixture(&pool).await;
    let auth = login_test_pool(&pool, TestDatabaseLogin::Auth).await;
    let (_, access, _) = issue_bound(&pool, &f, &auth).await;
    let target = missing_target(&pool).await;
    let mut secrets = f.secrets.clone();
    secrets.push(access.clone());
    let before = all_rows(&pool).await;
    let mut findings = Vec::new();
    for force in [false, true] {
        let positive = finish(removal(&f, &access, target, force)).await.unwrap();
        assert!(
            private_error(positive, StatusCode::NOT_FOUND, "not_found", &secrets).await
                && before == all_rows(&pool).await,
            "V7_CALLER_PREREQUISITE: exact bearer reaches no-effect404 before malformed headers"
        );
        for case in ["double_space", "duplicate_same", "duplicate_conflict"] {
            let values = match case {
                "double_space" => vec![format!("Bearer  {access}")],
                "duplicate_same" => vec![format!("Bearer {access}"); 2],
                "duplicate_conflict" => vec![
                    format!("Bearer {access}"),
                    "Bearer conflicting-private-credential".into(),
                ],
                _ => unreachable!(),
            };
            let mut request = Request::builder()
                .method(Method::DELETE)
                .uri(path(target, force))
                .body(Body::empty())
                .unwrap();
            for value in &values {
                request
                    .headers_mut()
                    .append(header::AUTHORIZATION, value.parse().unwrap());
            }
            let response = f.router.clone().oneshot(request).await.unwrap();
            let actual_status = response.status();
            let mut inputs = secrets.clone();
            inputs.extend(values);
            let denied =
                private_error(response, StatusCode::UNAUTHORIZED, "unauthorized", &inputs).await;
            let unchanged = before == all_rows(&pool).await;
            if !(denied && unchanged) {
                findings.push(format!(
                    "force={force} case={case} actual_status={actual_status} private401={denied} unchanged={unchanged}"
                ));
            }
        }
    }
    auth.close().await;
    f.state.shutdown_realtime().await;
    f.business.close().await;
    assert!(findings.is_empty(), "{}", findings.join("; "));
}
