//! Actual mounted infrastructure failures; no production fault switches.
use super::*;

async fn fault_catalog(pool: &PgPool) -> Value {
    sqlx::query_scalar(r#"
        SELECT jsonb_build_object(
          'functions', (SELECT jsonb_agg((to_jsonb(p)-'proacl') || jsonb_build_object('acl',
             (SELECT jsonb_agg(to_jsonb(a) ORDER BY grantor,grantee,privilege_type,is_grantable)
              FROM aclexplode(COALESCE(p.proacl,acldefault('f',p.proowner))) a)) ORDER BY p.oid)
             FROM pg_proc p WHERE p.pronamespace='public'::regnamespace AND p.proname IN
             ('auth_legacy_platform_source_material_v1','platform_list_organizations','platform_list_test_audit_failure')),
          'triggers', (SELECT jsonb_agg(to_jsonb(t) ORDER BY t.oid) FROM pg_trigger t
             WHERE t.tgrelid='public.audit_events'::regclass))
    "#).fetch_one(pool).await.unwrap()
}

#[sqlx::test(migrations = false)]
async fn mounted_list_infrastructure_503_is_private_atomic_and_recovers(pool: PgPool) {
    let f = fixture(&pool).await;
    positive(&pool, &f, &f.router).await;
    let faults = [
        (
            "REVOKE EXECUTE ON FUNCTION public.auth_legacy_platform_source_material_v1(uuid,uuid) FROM console_rt",
            "GRANT EXECUTE ON FUNCTION public.auth_legacy_platform_source_material_v1(uuid,uuid) TO console_rt",
            Some("public.auth_legacy_platform_source_material_v1(uuid,uuid)"),
        ),
        (
            "REVOKE EXECUTE ON FUNCTION public.platform_list_organizations() FROM console_rt",
            "GRANT EXECUTE ON FUNCTION public.platform_list_organizations() TO console_rt",
            Some("public.platform_list_organizations()"),
        ),
        (
            "CREATE FUNCTION public.platform_list_test_audit_failure() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION USING ERRCODE='P0002', MESSAGE='auth_legacy.subject_not_found'; END $$; CREATE TRIGGER platform_list_test_audit_failure BEFORE INSERT ON public.audit_events FOR EACH ROW EXECUTE FUNCTION public.platform_list_test_audit_failure()",
            "DROP TRIGGER platform_list_test_audit_failure ON public.audit_events; DROP FUNCTION public.platform_list_test_audit_failure()",
            None,
        ),
    ];
    for (install, restore, function) in faults {
        let original = fault_catalog(&pool).await;
        let rows = all_rows(&pool).await;
        if let Some(function) = function {
            let allowed: bool =
                sqlx::query_scalar("SELECT has_function_privilege('console_rt',$1,'EXECUTE')")
                    .bind(function)
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            assert!(
                allowed,
                "positive exact Business function grant prerequisite"
            );
        }
        sqlx::raw_sql(sqlx::AssertSqlSafe(install))
            .execute(&pool)
            .await
            .unwrap();
        let installed = fault_catalog(&pool).await;
        assert_ne!(
            original, installed,
            "actual fault must change its catalog boundary"
        );
        if let Some(function) = function {
            let allowed: bool =
                sqlx::query_scalar("SELECT has_function_privilege('console_rt',$1,'EXECUTE')")
                    .bind(function)
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            assert!(!allowed, "actual Business role loses the exact capability");
        }
        let mut outcomes = Vec::new();
        for method in [Method::GET, Method::HEAD] {
            let response = finish(request_task(&f.router, &f.access, method.clone())).await;
            let outcome = match response {
                Ok(response) => {
                    let (status, headers, body) = response_parts(response).await;
                    let payload = if method == Method::HEAD {
                        body.is_empty()
                    } else {
                        serde_json::from_slice::<Value>(&body).ok()
                            == Some(json!({"error":{
                                "code":"service_unavailable","message":"platform listing is unavailable"
                            }}))
                    };
                    status == StatusCode::SERVICE_UNAVAILABLE
                        && payload
                        && private_response(&headers, &body, &f.secrets)
                        && !body
                            .windows(b"auth_legacy.subject_not_found".len())
                            .any(|w| w == b"auth_legacy.subject_not_found")
                }
                Err(_) => false,
            };
            outcomes.push(
                outcome && all_rows(&pool).await == rows && fault_catalog(&pool).await == installed,
            );
        }
        // Restore before asserting response outcomes so diagnostic failures leave
        // no test-owned injection in the disposable database.
        sqlx::raw_sql(sqlx::AssertSqlSafe(restore))
            .execute(&pool)
            .await
            .unwrap();
        assert!(
            fault_catalog(&pool).await == original,
            "exact function/ACL/trigger catalog recovery"
        );
        assert!(
            all_rows(&pool).await == rows,
            "no projection audit or other durable mutation"
        );
        assert!(
            outcomes.into_iter().all(|ok| ok),
            "GET/HEAD must give finite private503 at each owning stage"
        );
        positive(&pool, &f, &f.router).await;
    }
    f.state.shutdown_realtime().await;
    f.business.close().await;
}
