// Include inside native_policy_startup_tests after its existing source include.
// Additive actual startup/ACL regression; no browser or business-flow claim.
async fn system_acl_metadata(pool: &PgPool) -> Value {
    sqlx::query_scalar("SELECT jsonb_build_object('number',a.attnum,'name',a.attname,'acl_is_null',a.attacl IS NULL,'table_owner',pg_get_userbyid(c.relowner),'owner_select',has_column_privilege('console_account_owner',c.oid,a.attnum,'SELECT'),'public_select',EXISTS(SELECT 1 FROM aclexplode(a.attacl) x WHERE x.grantee=0 AND x.privilege_type='SELECT'),'acl',(SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(x.grantor),CASE WHEN x.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(x.grantee) END,x.privilege_type,x.is_grantable) ORDER BY pg_get_userbyid(x.grantor),CASE WHEN x.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(x.grantee) END,x.privilege_type) FROM aclexplode(a.attacl) x)) FROM pg_attribute a JOIN pg_class c ON c.oid=a.attrelid WHERE a.attrelid='public.audit_events'::regclass AND a.attname='xmin' AND a.attnum<0 AND NOT a.attisdropped")
        .fetch_one(pool).await.unwrap()
}

async fn system_acl_capture(pool: &PgPool) -> (Value, String, Option<bool>) {
    let mut tx = pool.begin().await.unwrap();
    sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
        .execute(tx.as_mut())
        .await
        .unwrap();
    sqlx::raw_sql(CLASSIFIER_SESSION)
        .execute(tx.as_mut())
        .await
        .unwrap();
    let capture = sqlx::query_as(include_str!(
        "../../../../ops/postgres-capture-native-company-policy-custody.sql"
    ))
    .fetch_one(tx.as_mut())
    .await
    .unwrap();
    tx.commit().await.unwrap();
    capture
}

fn assert_system_acl_captured(snapshot: &Value, metadata: &Value) {
    let tables: Vec<_> = snapshot["tables"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["name"] == "audit_events")
        .collect();
    assert_eq!(tables.len(), 1);
    let columns: Vec<_> = tables[0]["column_security"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["name"] == "xmin")
        .collect();
    if metadata["acl_is_null"] == true {
        assert!(
            columns.is_empty(),
            "null system ACL should not add a snapshot row"
        );
    } else {
        assert_eq!(
            columns.len(),
            1,
            "negative attnum ACL missing from actual capture"
        );
        assert_eq!(
            *columns[0],
            json!({
                "number": metadata["number"], "name": "xmin",
                "acl_is_null": false, "acl": metadata["acl"]
            })
        );
    }
}

#[sqlx::test(migrations = false)]
async fn native_policy_system_column_acl_drift_refuses_startup_and_recovers(pool: PgPool) {
    let (app, key, state) = configured_fixture(&pool, true).await;
    let config = account_browser_config(&pool, app._artifacts.root.clone(), &key);
    let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
    let outcome = AssertUnwindSafe(async {
        let before = all_rows(&pool).await;
        let original = system_acl_metadata(&pool).await;
        assert!(original["number"].as_i64().unwrap() < 0);
        assert_eq!(original["acl_is_null"], json!(false));
        assert_eq!(original["owner_select"], json!(true));
        assert_eq!(original["public_select"], json!(false));
        assert_eq!(
            original["acl"],
            json!([[
                original["table_owner"],
                "console_account_owner",
                "SELECT",
                false
            ]]),
            "baseline must have exactly the narrow owner grant, without grant option"
        );
        let baseline = system_acl_capture(&runtime).await;
        assert_system_acl_captured(&baseline.0, &original);
        assert_eq!(baseline.2, Some(true));
        assert_eq!(
            policy_classified(&runtime).await,
            "native_company_policy.finalized"
        );
        assert_eq!(ready_status(&state).await, StatusCode::OK);
        assert!(before == all_rows(&pool).await);

        for (label, mutation, restore, owner_select, public_select) in [
            (
                "required grant removed",
                "REVOKE SELECT(xmin) ON public.audit_events FROM console_account_owner",
                "GRANT SELECT(xmin) ON public.audit_events TO console_account_owner",
                false,
                false,
            ),
            (
                "unexpected PUBLIC grant",
                "GRANT SELECT(xmin) ON public.audit_events TO PUBLIC",
                "REVOKE SELECT(xmin) ON public.audit_events FROM PUBLIC",
                true,
                true,
            ),
        ] {
            let checked = AssertUnwindSafe(async {
                // Each statement is atomic; cleanup also runs when setup or a
                // later assertion panics. No business row is changed directly.
                sqlx::raw_sql(sqlx::AssertSqlSafe(mutation))
                    .execute(&pool)
                    .await
                    .unwrap();
                let changed = system_acl_metadata(&pool).await;
                assert_ne!(changed, original, "{label}: metadata fault absent");
                assert_eq!(changed["owner_select"], json!(owner_select), "{label}");
                assert_eq!(changed["public_select"], json!(public_select), "{label}");
                let actual_acl: BTreeSet<String> = changed["acl"]
                    .as_array()
                    .map(|rows| {
                        rows.iter()
                            .map(|row| serde_json::to_string(row).unwrap())
                            .collect()
                    })
                    .unwrap_or_default();
                let expected_acl: BTreeSet<String> = if public_select {
                    [
                        json!([
                            original["table_owner"],
                            "console_account_owner",
                            "SELECT",
                            false
                        ]),
                        json!([original["table_owner"], "PUBLIC", "SELECT", false]),
                    ]
                    .iter()
                    .map(|row| serde_json::to_string(row).unwrap())
                    .collect()
                } else {
                    BTreeSet::new()
                };
                assert_eq!(
                    actual_acl, expected_acl,
                    "{label}: exact system-column grant witness"
                );
                let captured = system_acl_capture(&runtime).await;
                assert_system_acl_captured(&captured.0, &changed);
                assert_ne!(
                    captured.0, baseline.0,
                    "{label}: actual snapshot omitted ACL drift"
                );
                assert_ne!(
                    captured.1, baseline.1,
                    "{label}: custody fingerprint ignored ACL drift"
                );
                assert_eq!(
                    policy_classified(&runtime).await,
                    "native_company_policy.profile_mismatch",
                    "{label}"
                );
                startup_refused(config.clone(), "native_company_policy.profile_mismatch").await;
                assert_eq!(
                    ready_status(&state).await,
                    StatusCode::SERVICE_UNAVAILABLE,
                    "{label}"
                );
                assert!(
                    before == all_rows(&pool).await,
                    "{label}: metadata probe changed business rows"
                );
            })
            .catch_unwind()
            .await;
            sqlx::raw_sql(sqlx::AssertSqlSafe(restore))
                .execute(&pool)
                .await
                .unwrap();
            assert_eq!(
                system_acl_metadata(&pool).await,
                original,
                "{label}: exact normalized ACL restoration"
            );
            assert_eq!(
                system_acl_capture(&runtime).await,
                baseline,
                "{label}: complete fingerprint restoration"
            );
            assert_eq!(
                policy_classified(&runtime).await,
                "native_company_policy.finalized"
            );
            assert_eq!(ready_status(&state).await, StatusCode::OK);
            assert!(
                before == all_rows(&pool).await,
                "{label}: cleanup changed business rows"
            );
            if let Err(panic) = checked {
                std::panic::resume_unwind(panic);
            }
        }
        // Restoration must support fresh actual AppState construction as well
        // as the retained state's readiness. Close it even if readiness panics.
        let restored = AppState::from_config(config).await.unwrap();
        let recovered = AssertUnwindSafe(async {
            assert_eq!(ready_status(&restored).await, StatusCode::OK);
            assert!(before == all_rows(&pool).await);
        })
        .catch_unwind()
        .await;
        restored.shutdown_realtime().await;
        if let Err(panic) = recovered {
            std::panic::resume_unwind(panic);
        }
    })
    .catch_unwind()
    .await;
    runtime.close().await;
    close_states(&[state], outcome).await;
}
