//! Actual PostgreSQL source/family histories; child of credential tests.
use super::*;
use console_platform_auth::RefreshTokenUseError;
use futures::FutureExt;

pub(in super::super) async fn role_wait(
    pool: &PgPool,
    blocker: i32,
    role: &str,
    fragment: &str,
) -> Option<i32> {
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(4);
    loop {
        let pid:Option<i32>=sqlx::query_scalar("SELECT pid FROM pg_catalog.pg_stat_activity WHERE datname=current_database() AND usename=$3 AND wait_event_type='Lock' AND $1=ANY(pg_catalog.pg_blocking_pids(pid)) AND strpos(query,$2)>0 ORDER BY pid LIMIT 1")
            .bind(blocker).bind(fragment).bind(role).fetch_optional(pool).await.unwrap();
        if pid.is_some() || tokio::time::Instant::now() >= deadline {
            return pid;
        }
        tokio::time::sleep(std::time::Duration::from_millis(15)).await;
    }
}
pub(in super::super) async fn elapsed(pool: &PgPool, deadline: OffsetDateTime) -> bool {
    tokio::time::timeout(std::time::Duration::from_secs(10), async {
        loop {
            if db_now(pool).await >= deadline {
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(15)).await;
        }
    })
    .await
    .is_ok()
}
pub(in super::super) fn rows(snapshot: &Rows, table: &str) -> Vec<Value> {
    serde_json::from_str(&snapshot[table]).unwrap()
}
pub(in super::super) fn same_except(before: &Rows, after: &Rows, allowed: &[&str]) -> bool {
    before.keys().eq(after.keys())
        && before.iter().all(|(table, bytes)| {
            allowed.contains(&table.as_str()) || after.get(table) == Some(bytes)
        })
}
fn at(value: &Value, expected: OffsetDateTime) -> bool {
    value.as_str().is_some_and(|v| {
        OffsetDateTime::parse(v, &time::format_description::well_known::Rfc3339)
            .is_ok_and(|v| v == expected)
    })
}
#[expect(
    clippy::too_many_arguments,
    reason = "Keep independent source, family, event and read evidence explicit in this test oracle"
)]
pub(in super::super) fn revocation_delta(
    before: &Rows,
    after: &Rows,
    actor: UserId,
    family: Uuid,
    reused_token: Uuid,
    metadata_count: usize,
    reason: &str,
    now: OffsetDateTime,
    start: OffsetDateTime,
    end: OffsetDateTime,
    read: bool,
) -> bool {
    if !same_except(
        before,
        after,
        &[
            "auth_refresh_token_families",
            "auth_refresh_tokens",
            "audit_events",
        ],
    ) {
        return false;
    }
    for table in ["auth_refresh_token_families", "auth_refresh_tokens"] {
        let old = rows(before, table);
        let new = rows(after, table);
        if old.len() != new.len() {
            return false;
        }
        let mut changed = 0;
        for old in old {
            let Some(actual) = new.iter().find(|r| r["id"] == old["id"]) else {
                return false;
            };
            let targeted = if table == "auth_refresh_token_families" {
                old["id"] == json!(family)
            } else {
                old["family_id"] == json!(family)
            };
            if !targeted {
                if old != *actual {
                    return false;
                }
                continue;
            }
            changed += 1;
            if !old["revoked_at"].is_null() || !at(&actual["revoked_at"], now) {
                return false;
            }
            let mut expected = old.clone();
            expected["revoked_at"] = actual["revoked_at"].clone();
            if table == "auth_refresh_token_families" {
                if !old["revoked_reason"].is_null() {
                    return false;
                }
                expected["revoked_reason"] = json!(reason);
            } else if reason == "reuse_detected" && old["id"] == json!(reused_token) {
                if !old["reuse_detected_at"].is_null() || !at(&actual["reuse_detected_at"], now) {
                    return false;
                }
                expected["reuse_detected_at"] = actual["reuse_detected_at"].clone();
            }
            if expected != *actual {
                return false;
            }
        }
        if changed == 0 || (table == "auth_refresh_token_families" && changed != 1) {
            return false;
        }
    }
    let old: Vec<&RawValue> = serde_json::from_str(&before["audit_events"]).unwrap();
    let new: Vec<&RawValue> = serde_json::from_str(&after["audit_events"]).unwrap();
    let old_set: BTreeSet<_> = old.iter().map(|r| r.get()).collect();
    let new_set: BTreeSet<_> = new.iter().map(|r| r.get()).collect();
    if old_set.len() != old.len()
        || new_set.len() != new.len()
        || !old_set.is_subset(&new_set)
        || new.len() != old.len() + 1 + usize::from(read)
    {
        return false;
    }
    let mut revocations = 0;
    let mut reads = 0;
    let prior: Vec<Value> = old
        .iter()
        .map(|r| serde_json::from_str(r.get()).unwrap())
        .collect();
    let mut ids: BTreeSet<String> = prior
        .iter()
        .filter_map(|r| r["id"].as_str().map(str::to_owned))
        .collect();
    let mut traces: BTreeSet<String> = prior
        .iter()
        .filter_map(|r| r["trace_id"].as_str().map(str::to_owned))
        .collect();
    for raw in new_set.difference(&old_set) {
        let row: Value = serde_json::from_str(raw).unwrap();
        let (Some(id), Some(trace)) = (row["id"].as_str(), row["trace_id"].as_str()) else {
            return false;
        };
        if !ids.insert(id.to_owned()) || !traces.insert(trace.to_owned()) {
            return false;
        }
        if row["action"] == "platform.tenant.list" {
            if !audit_matches(&row, actor, metadata_count, start, end) {
                return false;
            }
            reads += 1;
        } else {
            let action = if reason == "logout" {
                "auth.logout"
            } else {
                "auth.refresh.reuse_detected"
            };
            let expected_snap = if reason == "logout" {
                json!({"family_id":family,"revoked_reason":reason})
            } else {
                json!({"family_id":family,"revoked_reason":reason,"reused_token_id":reused_token})
            };
            if row["action"] != action
                || row["target_type"] != "auth_refresh_token_family"
                || row["target_id"] != json!(family.to_string())
                || row["org_id"] != json!(OrgId::platform())
                || row["after_snap"] != expected_snap
                || !at(&row["occurred_at"], now)
            {
                return false;
            }
            // Validate original event-specific fields above, then reuse the exact20-field oracle.
            let mut normalized = row;
            normalized["action"] = json!("platform.tenant.list");
            normalized["target_type"] = json!("organizations");
            normalized["target_id"] = json!("list");
            normalized["org_id"] = Value::Null;
            normalized["after_snap"] = json!({"count":0});
            if !audit_matches(&normalized, actor, 0, start, end) {
                return false;
            }
            revocations += 1;
        }
    }
    revocations == 1 && reads == usize::from(read)
}
async fn assert_http_result(
    f: &Fixture,
    token: &str,
    response: http::Response<Body>,
    status: StatusCode,
) {
    let (actual, headers, body) = response_parts(response).await;
    let mut secrets = f.secrets.clone();
    secrets.push(token.to_owned());
    assert!(actual == status && private_response(&headers, &body, &secrets));
    let value: Value = serde_json::from_slice(&body).unwrap();
    if status == StatusCode::OK {
        assert!(metadata_matches(&value, &f.expected));
    } else {
        assert!(value == json!({"error":{"code":"unauthorized","message":"invalid bearer token"}}));
    }
}
async fn revoker_first(pool: PgPool, reuse: bool) {
    let f = fixture(&pool).await;
    let auth = login_test_pool(&pool, TestDatabaseLogin::Auth).await;
    let (a, access, _) = issue_bound(&pool, &f, &auth).await;
    let (_, sibling, _) = issue_bound(&pool, &f, &auth).await;
    direct_positive(&pool, &f, &access, Duration::days(30)).await;
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
    let reader = request_task(&f.router, &access, Method::GET);
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
    direct_positive(&pool, &f, &sibling, Duration::days(30)).await;
    assert_http_result(&f, &access, result.unwrap(), StatusCode::UNAUTHORIZED).await;
    assert!(
        waited_cleanly(blocker, reader_pid, pending, clean),
        "actual owner waits for canonical revocation commit"
    );
    assert!(
        revocation_delta(
            &before,
            &after,
            f.actor,
            a.family_id,
            a.token_id,
            f.expected.as_array().unwrap().len(),
            if reuse { "reuse_detected" } else { "logout" },
            now,
            start,
            end,
            false
        ),
        "only exact canonical revocation effects; no list audit"
    );
    auth.close().await;
    close(f).await;
}
#[sqlx::test(migrations = false)]
async fn list_revoker_first_logout_retains_source_until_commit(pool: PgPool) {
    revoker_first(pool, false).await;
}
#[sqlx::test(migrations = false)]
async fn list_revoker_first_reuse_retains_source_until_commit(pool: PgPool) {
    revoker_first(pool, true).await;
}

#[sqlx::test(migrations = false)]
async fn list_reader_first_holds_subject_and_exact_family_until_projection_commit(pool: PgPool) {
    let f = fixture(&pool).await;
    let auth = login_test_pool(&pool, TestDatabaseLogin::Auth).await;
    let (a, access, _) = issue_bound(&pool, &f, &auth).await;
    let (_, sibling, _) = issue_bound(&pool, &f, &auth).await;
    direct_positive(&pool, &f, &access, Duration::days(30)).await;
    let before = all_rows(&pool).await;
    let start = db_now(&pool).await;
    let mut observer = pool.acquire().await.unwrap();
    let mut projection = pool.begin().await.unwrap();
    let blocker: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *projection)
        .await
        .unwrap();
    sqlx::query("LOCK TABLE public.groups IN ACCESS EXCLUSIVE MODE")
        .execute(&mut *projection)
        .await
        .unwrap();
    let reader = request_task(&f.router, &access, Method::GET);
    let reader_pid = role_wait(&pool, blocker, "console_rt", "platform_list_organizations").await;
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
    assert_http_result(&f, &access, result.unwrap(), StatusCode::OK).await;
    assert!(
        reader_pid.is_some() && family_wait.is_some() && revoke_wait.is_some() && pending && clean,
        "observed metadata, exact-family SHARE and canonical source-guard exclusion required"
    );
    assert!(
        revocation_delta(
            &before,
            &after,
            f.actor,
            a.family_id,
            a.token_id,
            f.expected.as_array().unwrap().len(),
            "logout",
            now,
            start,
            end,
            true
        ),
        "one completed list audit plus one exact canonical logout"
    );
    direct_denied(&pool, &f, &access, Duration::days(30), "unauthorized").await;
    direct_positive(&pool, &f, &sibling, Duration::days(30)).await;
    auth.close().await;
    close(f).await;
}

async fn audit_expiry(pool: PgPool, bound: bool) {
    let f = fixture(&pool).await;
    let auth = login_test_pool(&pool, TestDatabaseLogin::Auth).await;
    let (_, bound_access, _) = issue_bound(&pool, &f, &auth).await;
    let original = if bound { &bound_access } else { &f.access };
    direct_positive(&pool, &f, original, Duration::days(30)).await;
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
    let reader = request_task(&f.router, &access, Method::GET);
    let waiter = role_wait(&pool, pid, "console_rt", "INSERT INTO audit_events").await;
    let pending = !reader.is_finished();
    let expired = elapsed(&pool, OffsetDateTime::from_unix_timestamp(expires).unwrap()).await;
    blocker.rollback().await.unwrap();
    let result = finish(reader).await;
    let pids: Vec<_> = std::iter::once(pid).chain(waiter).collect();
    let clean = clean_pids(&mut observer, &pids).await;
    let after = all_rows(&pool).await;
    assert_http_result(&f, &access, result.unwrap(), StatusCode::UNAUTHORIZED).await;
    assert!(
        waited_cleanly(pid, waiter, pending, clean) && expired,
        "audit INSERT wait and real DB expiry must precede release"
    );
    assert!(
        before == after,
        "post-audit lifetime refusal rolls back appended audit and all effects"
    );
    direct_positive(&pool, &f, original, Duration::days(30)).await;
    auth.close().await;
    close(f).await;
}
#[sqlx::test(migrations = false)]
async fn historical_list_expiry_during_actual_audit_insert_discards_projection(pool: PgPool) {
    audit_expiry(pool, false).await;
}
#[sqlx::test(migrations = false)]
async fn bound_list_expiry_during_actual_audit_insert_discards_projection(pool: PgPool) {
    audit_expiry(pool, true).await;
}

#[sqlx::test(migrations = false)]
async fn direct_list_current_inactive_and_account_fence_refuse_with_exact_recovery(pool: PgPool) {
    let f = fixture(&pool).await;
    direct_positive(&pool, &f, &f.access, Duration::days(30)).await;
    let original = all_rows(&pool).await;
    sqlx::query("UPDATE public.users SET is_active=false WHERE id=$1")
        .bind(f.actor.as_uuid())
        .execute(&pool)
        .await
        .unwrap();
    let inactive = all_rows(&pool).await;
    let failure = std::panic::AssertUnwindSafe(async {
        assert!(same_except(&original, &inactive, &["users"]));
        let mut expected = rows(&original, "users");
        let actual = rows(&inactive, "users");
        let mut changed = 0;
        for row in &mut expected {
            if row["id"] == json!(f.actor) {
                assert!(row["is_active"] == true);
                row["is_active"] = json!(false);
                changed += 1;
            }
        }
        assert!(
            changed == 1
                && expected.len() == actual.len()
                && expected.iter().all(|r| actual.contains(r))
        );
        direct_denied(&pool, &f, &f.access, Duration::days(30), "unauthorized").await;
    })
    .catch_unwind()
    .await;
    sqlx::query("UPDATE public.users SET is_active=true WHERE id=$1")
        .bind(f.actor.as_uuid())
        .execute(&pool)
        .await
        .unwrap();
    assert!(
        original == all_rows(&pool).await,
        "inactive source restored exactly"
    );
    if let Err(p) = failure {
        std::panic::resume_unwind(p);
    }
    direct_positive(&pool, &f, &f.access, Duration::days(30)).await;
    let original = all_rows(&pool).await;
    insert_account_fence(&pool, f.actor, "ACTIVE").await;
    let fenced = all_rows(&pool).await;
    let failure = std::panic::AssertUnwindSafe(async {
        assert!(same_except(&original, &fenced, &["account_security"]));
        let old = rows(&original, "account_security");
        let new = rows(&fenced, "account_security");
        assert!(new.len() == old.len() + 1 && old.iter().all(|r| new.contains(r)));
        let added: Vec<_> = new.iter().filter(|r| !old.contains(r)).collect();
        assert!(
            added.len() == 1
                && added[0]["account_id"] == json!(f.actor)
                && added[0]["security_state"] == "ACTIVE"
        );
        direct_denied(&pool, &f, &f.access, Duration::days(30), "unauthorized").await;
    })
    .catch_unwind()
    .await;
    sqlx::query("DELETE FROM public.account_security WHERE account_id=$1")
        .bind(f.actor.as_uuid())
        .execute(&pool)
        .await
        .unwrap();
    assert!(
        original == all_rows(&pool).await,
        "privileged fence fixture restored exactly"
    );
    if let Err(p) = failure {
        std::panic::resume_unwind(p);
    }
    direct_positive(&pool, &f, &f.access, Duration::days(30)).await;
    close(f).await;
}

pub(in super::super) async fn root_catalog(pool: &PgPool) -> Value {
    sqlx::query_scalar("SELECT jsonb_build_object('table',(SELECT to_jsonb(c) FROM pg_catalog.pg_class c WHERE oid='public.accounts'::regclass),'triggers',(SELECT jsonb_agg(to_jsonb(t) ORDER BY oid) FROM pg_catalog.pg_trigger t WHERE tgrelid='public.accounts'::regclass),'constraints',(SELECT jsonb_agg(to_jsonb(c) ORDER BY oid) FROM pg_catalog.pg_constraint c WHERE conrelid='public.accounts'::regclass OR confrelid='public.accounts'::regclass))")
        .fetch_one(pool).await.unwrap()
}
#[sqlx::test(migrations = false)]
async fn direct_list_missing_account_root_is_unavailable_and_restores_exactly(pool: PgPool) {
    let f = fixture(&pool).await;
    direct_positive(&pool, &f, &f.access, Duration::days(30)).await;
    let before = all_rows(&pool).await;
    let catalog = root_catalog(&pool).await;
    let account: Value =
        sqlx::query_scalar("SELECT to_jsonb(a) FROM public.accounts a WHERE id=$1")
            .bind(f.actor.as_uuid())
            .fetch_one(&pool)
            .await
            .unwrap();
    // Privileged isolated corruption fixture, never a proposed production mutation route.
    let mut tx = pool.begin().await.unwrap();
    sqlx::raw_sql("SET CONSTRAINTS ALL IMMEDIATE; ALTER TABLE public.accounts DISABLE TRIGGER account_roots_immutable_v1; SET LOCAL session_replication_role=replica").execute(&mut *tx).await.unwrap();
    sqlx::query("DELETE FROM public.accounts WHERE id=$1")
        .bind(f.actor.as_uuid())
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::raw_sql("SET LOCAL session_replication_role=origin; ALTER TABLE public.accounts ENABLE ALWAYS TRIGGER account_roots_immutable_v1").execute(&mut *tx).await.unwrap();
    tx.commit().await.unwrap();
    let failure = std::panic::AssertUnwindSafe(async {
        assert!(
            catalog == root_catalog(&pool).await,
            "missing data must not masquerade as changed metadata"
        );
        let missing = all_rows(&pool).await;
        assert!(same_except(&before, &missing, &["accounts"]));
        let old = rows(&before, "accounts");
        let new = rows(&missing, "accounts");
        assert!(
            old.len() == new.len() + 1
                && new.iter().all(|r| old.contains(r))
                && old
                    .iter()
                    .filter(|r| !new.contains(r))
                    .all(|r| r["id"] == json!(f.actor))
        );
        direct_denied(&pool, &f, &f.access, Duration::days(30), "unavailable").await;
    })
    .catch_unwind()
    .await;
    sqlx::query(
        "INSERT INTO public.accounts SELECT (jsonb_populate_record(NULL::public.accounts,$1)).*",
    )
    .bind(account)
    .execute(&pool)
    .await
    .unwrap();
    assert!(
        before == all_rows(&pool).await && catalog == root_catalog(&pool).await,
        "Account data and exact metadata restore before recovery"
    );
    if let Err(p) = failure {
        std::panic::resume_unwind(p);
    }
    direct_positive(&pool, &f, &f.access, Duration::days(30)).await;
    close(f).await;
}

#[test]
fn combined_revocation_and_read_oracle_detects_missing_effects_and_corruption() {
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
                audit.push(audit_fixture(actor, now));
            }
            after.insert("audit_events".to_owned(), json!(audit).to_string());
            let check = |actual: &Rows| {
                revocation_delta(
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

#[path = "legacy_platform_list_commit_loss.rs"]
mod legacy_platform_list_commit_loss;

#[path = "legacy_platform_list_expiry_progress.rs"]
mod legacy_platform_list_expiry_progress;

pub(in super::super) use legacy_platform_list_expiry_progress::{
    configured_family_state, other_operator,
};

pub(in super::super) use legacy_platform_list_commit_loss::{
    WireEvidence, evidence, proven_ack_loss, relay,
};
