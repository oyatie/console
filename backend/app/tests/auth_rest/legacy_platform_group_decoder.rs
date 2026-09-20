//! Functional Group member JSON decoding regression in the disposable SQLx database.
//! No production hook, migration, permission change or external transport.
use super::*;
use futures::FutureExt;

const BAD_MEMBER_ID: &str = "group-decoder-fixture-not-a-uuid";
const MEMBER_ID_EXPRESSION: &str = "'id', o.id,";

// Same pg_proc/ACL snapshot pattern as the existing ops function fixture, scoped
// to this one unchanged-signature projection and its exact dependency metadata.
async fn group_function_catalog(pool: &PgPool) -> Value {
    sqlx::query_scalar(r#"
        SELECT jsonb_build_object(
          'function', (to_jsonb(p)-'proacl') || jsonb_build_object('acl',
            (SELECT jsonb_agg(to_jsonb(a) ORDER BY grantor,grantee,privilege_type,is_grantable)
             FROM aclexplode(COALESCE(p.proacl,acldefault('f',p.proowner))) a)),
          'dependencies', (SELECT COALESCE(jsonb_agg(to_jsonb(d) ORDER BY classid,objid,objsubid,refclassid,refobjid,refobjsubid,deptype),'[]'::jsonb)
            FROM pg_depend d WHERE (classid='pg_proc'::regclass AND objid=p.oid) OR (refclassid='pg_proc'::regclass AND refobjid=p.oid)),
          'shared_dependencies', (SELECT COALESCE(jsonb_agg(to_jsonb(d) ORDER BY dbid,classid,objid,objsubid,refclassid,refobjid,deptype),'[]'::jsonb)
            FROM pg_shdepend d WHERE dbid=(SELECT oid FROM pg_database WHERE datname=current_database()) AND classid='pg_proc'::regclass AND objid=p.oid),
          'descriptions', (SELECT COALESCE(jsonb_agg(to_jsonb(d) ORDER BY objoid,classoid,objsubid),'[]'::jsonb)
            FROM pg_description d WHERE classoid='pg_proc'::regclass AND objoid=p.oid),
          'labels', (SELECT COALESCE(jsonb_agg(to_jsonb(l) ORDER BY objoid,classoid,objsubid,provider),'[]'::jsonb)
            FROM pg_seclabel l WHERE classoid='pg_proc'::regclass AND objoid=p.oid)
        ) FROM pg_proc p WHERE p.oid='public.platform_list_groups()'::regprocedure
    "#).fetch_one(pool).await.unwrap()
}

// Deliberately decode members only as generic JSON: SQL must succeed and return
// the malformed typed field, not fail first in the GroupSummary Rust decoder.
async fn raw_group_projection(pool: &PgPool) -> Result<Value, sqlx::Error> {
    let rows = sqlx::query("SELECT id,slug,name,status,created_at,updated_at,member_count,members FROM public.platform_list_groups()")
        .fetch_all(pool).await?;
    let mut result = Vec::new();
    for row in rows {
        let members: sqlx::types::Json<Value> = row.try_get("members")?;
        result.push(json!({
            "id":row.try_get::<Uuid,_>("id")?,
            "slug":row.try_get::<String,_>("slug")?,
            "name":row.try_get::<String,_>("name")?,
            "status":row.try_get::<String,_>("status")?,
            "created_at":wire_time(row.try_get("created_at")?),
            "updated_at":wire_time(row.try_get("updated_at")?),
            "member_count":row.try_get::<i64,_>("member_count")?,
            "members":members.0
        }));
    }
    Ok(json!(result))
}

#[derive(Clone, Copy)]
enum DecodeEntry {
    Direct,
    Get,
    Head,
}
enum DecodeResult {
    Direct(Result<Vec<GroupSummary>, ProvisioningError>),
    Http(http::Response<Body>),
}
fn decode_task(f: &Fixture, entry: DecodeEntry) -> tokio::task::JoinHandle<DecodeResult> {
    let pool = f.business.clone();
    let verifier = f.verifier.clone();
    let access = f.access.clone();
    let router = f.router.clone();
    tokio::spawn(async move {
        match entry {
            DecodeEntry::Direct => DecodeResult::Direct(
                PlatformProvisioner::new(Duration::hours(1))
                    .list_groups(
                        &pool,
                        &verifier,
                        &access,
                        Duration::days(30),
                        &PlatformPolicy::compile_current().unwrap(),
                    )
                    .await,
            ),
            DecodeEntry::Get | DecodeEntry::Head => DecodeResult::Http(
                router
                    .oneshot(
                        Request::builder()
                            .uri(GROUP_PATH)
                            .method(if matches!(entry, DecodeEntry::Head) {
                                Method::HEAD
                            } else {
                                Method::GET
                            })
                            .header(header::AUTHORIZATION, format!("Bearer {access}"))
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .unwrap(),
            ),
        }
    })
}
async fn decode_refusal(f: &Fixture, entry: DecodeEntry, result: DecodeResult) -> bool {
    match (entry, result) {
        (DecodeEntry::Direct, DecodeResult::Direct(result)) => {
            matches!(result, Err(ProvisioningError::PlatformGroupListUnavailable))
        }
        (DecodeEntry::Get | DecodeEntry::Head, DecodeResult::Http(response)) => {
            let (status, headers, body) = response_parts(response).await;
            let mut secrets = f.secrets.clone();
            secrets.push(BAD_MEMBER_ID.to_owned());
            secrets.push("platform_list_groups".to_owned());
            let payload = if matches!(entry, DecodeEntry::Head) {
                body.is_empty()
            } else {
                serde_json::from_slice::<Value>(&body).ok()
                    == Some(
                        json!({"error":{"code":"service_unavailable","message":"platform group list is unavailable"}}),
                    )
            };
            status == StatusCode::SERVICE_UNAVAILABLE
                && payload
                && private_response(&headers, &body, &secrets)
        }
        _ => false,
    }
}

#[sqlx::test(migrations = false)]
async fn actual_group_member_decode_failure_is_finite_atomic_and_recovers(pool: PgPool) {
    let (f, expected) = populated_fixture(&pool).await;
    owner_positive(&pool, &f, &f.access, &expected).await;
    mounted_token(&pool, &f, &f.access, StatusCode::OK, &expected).await;
    let before = all_rows(&pool).await;
    assert_eq!(
        raw_group_projection(&f.business).await.unwrap(),
        expected,
        "genuine Business raw SQL positive before substitution"
    );
    assert!(
        all_rows(&pool).await == before,
        "raw SQL control adds no owner audit or business mutation"
    );
    let original_definition: String = sqlx::query_scalar(
        "SELECT pg_get_functiondef('public.platform_list_groups()'::regprocedure)",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let original_catalog = group_function_catalog(&pool).await;
    let original_source = original_catalog["function"]["prosrc"].as_str().unwrap();
    assert_eq!(
        original_definition.matches(MEMBER_ID_EXPRESSION).count(),
        1,
        "exact frozen member expression prerequisite"
    );
    assert_eq!(
        original_source.matches(MEMBER_ID_EXPRESSION).count(),
        1,
        "exact one projection source expression"
    );
    let group_index = expected
        .as_array()
        .unwrap()
        .iter()
        .position(|g| g["member_count"] == 2)
        .unwrap();
    let member =
        Uuid::parse_str(expected[group_index]["members"][0]["id"].as_str().unwrap()).unwrap();
    let replacement = format!(
        "'id', CASE WHEN o.id = '{member}'::uuid THEN '{BAD_MEMBER_ID}' ELSE o.id::text END,"
    );
    let faulty_definition = original_definition.replacen(MEMBER_ID_EXPRESSION, &replacement, 1);
    let mut faulty_catalog = original_catalog.clone();
    faulty_catalog["function"]["prosrc"] =
        json!(original_source.replacen(MEMBER_ID_EXPRESSION, &replacement, 1));
    let mut faulty_expected = expected.clone();
    faulty_expected[group_index]["members"][0]["id"] = json!(BAD_MEMBER_ID);
    assert!(
        !group_matches(&faulty_expected, &expected),
        "strict existing projection oracle rejects typed corruption"
    );

    // All handles and barriers live outside panic capture so restoration cannot
    // run while an abandoned owning query still holds the projection function.
    let mut observer = pool.acquire().await.unwrap();
    let mut blocker: Option<sqlx::Transaction<'_, sqlx::Postgres>> = None;
    let mut task: Option<tokio::task::JoinHandle<DecodeResult>> = None;
    let mut owned_pids = Vec::new();
    let mut outcomes = Vec::new();
    let captured = std::panic::AssertUnwindSafe(async {
        // Original function text plus one fixed typed-UUID fixture substitution;
        // no credentials, external SQL input, ACL changes or historical edits.
        sqlx::raw_sql(sqlx::AssertSqlSafe(&faulty_definition))
            .execute(&pool)
            .await
            .unwrap();
        assert_eq!(
            group_function_catalog(&pool).await,
            faulty_catalog,
            "only the exact projection body changes; all identity/ACL/dependencies stay equal"
        );
        let raw = raw_group_projection(&f.business).await.unwrap();
        assert_eq!(
            raw, faulty_expected,
            "SQL succeeds and only one real member id is invalid typed JSON"
        );
        assert!(
            all_rows(&pool).await == before,
            "successful raw control preserves all state/history"
        );
        for entry in [DecodeEntry::Direct, DecodeEntry::Get, DecodeEntry::Head] {
            blocker = Some(pool.begin().await.unwrap());
            let tx = blocker.as_mut().unwrap();
            let pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
                .fetch_one(&mut **tx)
                .await
                .unwrap();
            owned_pids.push(pid);
            sqlx::raw_sql(
                "SET LOCAL lock_timeout='2s'; LOCK TABLE public.groups IN ACCESS EXCLUSIVE MODE",
            )
            .execute(&mut **tx)
            .await
            .unwrap();
            task = Some(decode_task(&f, entry));
            let waiter = waiting_on(
                &pool,
                pid,
                "platform_list_groups",
                std::time::Duration::from_secs(2),
            )
            .await;
            if let Some(pid) = waiter {
                owned_pids.push(pid);
            }
            let pending = !task.as_ref().unwrap().is_finished();
            // Release the actual SQL barrier before joining; the now successful
            // projection must reach the real Rust nested-member decoder.
            blocker.take().unwrap().rollback().await.unwrap();
            let completed = finish(task.take().unwrap()).await;
            let clean = clean_pids(&mut observer, &owned_pids).await;
            let refusal = match completed {
                Ok(result) => decode_refusal(&f, entry, result).await,
                Err(_) => false,
            };
            outcomes.push(
                waited_cleanly(pid, waiter, pending, clean)
                    && refusal
                    && all_rows(&pool).await == before
                    && group_function_catalog(&pool).await == faulty_catalog,
            );
        }
        assert_eq!(
            raw_group_projection(&f.business).await.unwrap(),
            faulty_expected,
            "same corrupted projection still succeeds as raw JSON after all owner attempts"
        );
    })
    .catch_unwind()
    .await;

    // Finally: bounded join/cancellation and actual transaction/PID cleanup
    // precede DDL restoration. Errors are captured until restoration completes.
    let released = match blocker.take() {
        Some(tx) => tx.rollback().await.is_ok(),
        None => true,
    };
    if let Some(pending) = task.take() {
        pending.abort();
        let _ = finish(pending).await;
    }
    let cleaned = clean_pids(&mut observer, &owned_pids).await;
    let mut restore = pool.begin().await.unwrap();
    sqlx::raw_sql("SET LOCAL lock_timeout='2s'; SET LOCAL statement_timeout='5s'")
        .execute(&mut *restore)
        .await
        .unwrap();
    sqlx::raw_sql(sqlx::AssertSqlSafe(&original_definition))
        .execute(&mut *restore)
        .await
        .unwrap();
    restore.commit().await.unwrap();
    let restored_definition: String = sqlx::query_scalar(
        "SELECT pg_get_functiondef('public.platform_list_groups()'::regprocedure)",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let restored_catalog = group_function_catalog(&pool).await;
    let restored_state = all_rows(&pool).await;
    assert!(
        released && cleaned,
        "owned barriers/tasks/PIDs cleaned before DDL restore"
    );
    assert_eq!(
        restored_definition, original_definition,
        "exact projection source restored"
    );
    assert_eq!(
        restored_catalog, original_catalog,
        "exact projection identity/ACL/catalog/dependencies restored"
    );
    assert!(
        restored_state == before,
        "no owner audit or public row changed on decode failure"
    );
    // Reopen with the same original credential and database after exact restore.
    assert_eq!(
        raw_group_projection(&f.business).await.unwrap(),
        expected,
        "restored actual raw projection positive"
    );
    owner_positive(&pool, &f, &f.access, &expected).await;
    mounted_token(&pool, &f, &f.access, StatusCode::OK, &expected).await;
    close_groups(f).await;
    if let Err(panic) = captured {
        std::panic::resume_unwind(panic);
    }
    assert!(
        outcomes.len() == 3 && outcomes.into_iter().all(|ok| ok),
        "actual direct/GET/HEAD decode503, observed SQL and exact no-effect histories required"
    );
}
