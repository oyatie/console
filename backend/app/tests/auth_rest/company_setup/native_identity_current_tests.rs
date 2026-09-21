use super::*;

// Include inside company_setup; no successful authority is constructed in SQL.
#[sqlx::test(migrations = false)]
async fn identity_initial_projection_is_current_after_real_session_revocation(pool: PgPool) {
    let (app, operator, operator_cookies, startup, _) = designated(&pool).await;
    let (recipient, cookies) = enrolled(&app).await;
    let command = Uuid::new_v4();
    let input = enrollment(command, recipient.account);
    let result = committed(
        &submit(&app, &operator_cookies, &input).await,
        StatusCode::CREATED,
        command,
        recipient.account,
        false,
    );
    initial_ceiling(&app, &cookies, &result).await;
    let page = document(&app, &format!("/companies/{}", result.company), &cookies).await;
    native_entry_html(&page, StatusCode::OK);
    let discovery = document(&app, "/account", &cookies).await;
    assert!(
        native_entry_html(&discovery, StatusCode::OK)
            .contains(&format!("href=\"/companies/{}\"", result.company)),
        "actual authorized Company absent from populated Account discovery"
    );
    let other = document(
        &app,
        &format!("/companies/{}", result.company),
        &operator_cookies,
    )
    .await;
    native_entry_html(&other, StatusCode::NOT_FOUND);
    let before = all_rows(&pool).await;
    let csrf = proof(&app, &cookies).await;
    let logout = request(
        &app,
        "POST",
        "/api/v2/auth/logout",
        &cookies,
        Some(json!({})),
        &[("X-Console-CSRF", &csrf)],
    )
    .await;
    assert_eq!(logout.json(StatusCode::OK), json!({"outcome":"COMMITTED"}));
    logout.private();
    let revoked = all_rows(&pool).await;
    for table in [
        "policy_roles",
        "user_role_assignments",
        "policy_role_revisions",
        "policy_assignment_revisions",
        "policy_capability_clauses",
        "policy_capability_clause_fields",
        "native_company_catalog_installs",
        "company_enrollment_receipts",
        "company_enrollment_effect_bindings",
        "company_authority_heads",
    ] {
        assert!(
            before[table] == revoked[table],
            "session revocation rewrote business provenance: {table}"
        );
    }
    request(
        &app,
        "GET",
        &format!("/api/v2/companies/{}/policy", result.company),
        &cookies,
        None,
        &[],
    )
    .await
    .error(StatusCode::UNAUTHORIZED, "authentication_invalid");
    let account_page = document(&app, "/account", &cookies).await;
    let html = native_entry_html(&account_page, StatusCode::OK);
    assert!(html.contains("data-account-state=\"anonymous\""));
    assert!(!html.contains(&format!("href=\"/companies/{}\"", result.company)));
    assert!(
        revoked == all_rows(&pool).await,
        "denied read changed authority/history"
    );
    assert!(identity_graph_matches(&revoked, &result, operator.account));
    startup.close().await;
}

#[sqlx::test(migrations = false)]
async fn identity_legacy_reader_filter_and_global_truncate_guard_ignore_caller_scope(pool: PgPool) {
    let (app, account, cookies, startup, _) = designated(&pool).await;
    let command = Uuid::new_v4();
    let input = enrollment(command, account.account);
    let result = committed(
        &submit(&app, &cookies, &input).await,
        StatusCode::CREATED,
        command,
        account.account,
        false,
    );
    let before = all_rows(&pool).await;
    let native = identity_rows(&before, "policy_roles", result.company);
    assert_eq!(native.len(), 1);
    let native_role: Uuid = native[0]["id"].as_str().unwrap().parse().unwrap();
    let business = login_test_pool(&pool, TestDatabaseLogin::Business).await;
    console_platform_request_context::scope_org(OrgId::from_uuid(result.company), async {
        let store = console_identity_adapter_postgres::PgOrgStore::new(business.clone());
        assert!(
            store.list_policy_roles().await.unwrap().is_empty(),
            "legacy adapter disclosed native role"
        );
        assert_eq!(
            store
                .count_policy_role_assignments(native_role)
                .await
                .unwrap(),
            0,
            "legacy adapter counted native assignment"
        );
    })
    .await;
    business.close().await;
    // Owner-invariant probe, explicitly NOT proof of a serving TRUNCATE grant.
    // console_app really owns the old table; no serving role may assume it.
    sqlx::raw_sql(
        r#"
      CREATE SEQUENCE public.identity_truncate_witness;
      CREATE FUNCTION public.identity_truncate_witness() RETURNS trigger LANGUAGE plpgsql
      SECURITY DEFINER SET search_path=pg_catalog,pg_temp AS $$
      BEGIN PERFORM nextval('public.identity_truncate_witness'::regclass);RETURN NULL;END $$;
      CREATE TRIGGER a_identity_truncate_witness BEFORE TRUNCATE ON public.policy_role_permissions
      FOR EACH STATEMENT EXECUTE FUNCTION public.identity_truncate_witness();
    "#,
    )
    .execute(&pool)
    .await
    .unwrap();
    for (index, guc) in [None, Some(Uuid::new_v4().to_string())].iter().enumerate() {
        let mut tx = pool.begin().await.unwrap();
        sqlx::query("SET LOCAL ROLE console_app")
            .execute(tx.as_mut())
            .await
            .unwrap();
        sqlx::query("SELECT set_config('app.current_org',$1,true)")
            .bind(guc.as_deref().unwrap_or(""))
            .execute(tx.as_mut())
            .await
            .unwrap();
        let error = sqlx::query("TRUNCATE public.policy_role_permissions")
            .execute(tx.as_mut())
            .await
            .expect_err("global native-presence guard trusted caller Company filter");
        assert_eq!(
            error.as_database_error().and_then(|e| e.code()).as_deref(),
            Some("42501")
        );
        tx.rollback().await.unwrap();
        let (called, value): (bool, i64) =
            sqlx::query_as("SELECT is_called,last_value FROM public.identity_truncate_witness")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert!(
            called && value == i64::try_from(index + 1).unwrap(),
            "TRUNCATE guard boundary not reached"
        );
        assert!(before == all_rows(&pool).await);
    }
    sqlx::raw_sql("DROP TRIGGER a_identity_truncate_witness ON public.policy_role_permissions;DROP FUNCTION public.identity_truncate_witness();DROP SEQUENCE public.identity_truncate_witness;")
        .execute(&pool).await.unwrap();
    startup.close().await;
}
