use super::*;

// PROPOSED: include inside the existing company_setup module after review.
// No test fixture inserts a successful business effect. All positive setup uses
// actual enrollment/session/designation and the mounted Company HTTP owner.

const NATIVE_PRIVATE_CALLS: &[(&str, &str)] = &[
    (
        "public.company_enrollment_binding_v1(uuid,uuid)",
        "SELECT * FROM public.company_enrollment_binding_v1(NULL::uuid,NULL::uuid)",
    ),
    (
        "public.company_enrollment_topology_v1(uuid,uuid)",
        "SELECT * FROM public.company_enrollment_topology_v1(NULL::uuid,NULL::uuid)",
    ),
    (
        "ontology_api.install_native_company_catalog_v1(uuid,uuid,text,text)",
        "SELECT * FROM ontology_api.install_native_company_catalog_v1(NULL::uuid,NULL::uuid,NULL::text,NULL::text)",
    ),
    (
        "ont_policy_api.install_native_company_policy_v1(uuid,uuid,text,text)",
        "SELECT * FROM ont_policy_api.install_native_company_policy_v1(NULL::uuid,NULL::uuid,NULL::text,NULL::text)",
    ),
    (
        "public.identity_enroll_company_administration_v1(uuid,uuid)",
        "SELECT * FROM public.identity_enroll_company_administration_v1(NULL::uuid,NULL::uuid)",
    ),
    (
        "public.company_enrollment_audit_v1(uuid,uuid,text,text)",
        "SELECT public.company_enrollment_audit_v1(NULL::uuid,NULL::uuid,NULL::text,NULL::text)",
    ),
    (
        "public.company_enrollment_assert_closure_v1(uuid,uuid)",
        "SELECT public.company_enrollment_assert_closure_v1(NULL::uuid,NULL::uuid)",
    ),
    (
        "public.group_authority_lock_shared_v1(uuid)",
        "SELECT * FROM public.group_authority_lock_shared_v1(NULL::uuid)",
    ),
    (
        "public.group_authority_lock_exclusive_v1(uuid)",
        "SELECT * FROM public.group_authority_lock_exclusive_v1(NULL::uuid)",
    ),
    (
        "public.platform_create_organization_core_v1(uuid,uuid,text,text)",
        "SELECT public.platform_create_organization_core_v1(NULL::uuid,NULL::uuid,NULL::text,NULL::text)",
    ),
];

const PRIVATE_RAW_TOPOLOGY_CALLS: &[(&str, &str)] = &[
    (
        "public.platform_create_organization(text,text)",
        "SELECT public.platform_create_organization(NULL::text,NULL::text)",
    ),
    (
        "public.platform_assign_org_to_group(uuid,uuid)",
        "SELECT public.platform_assign_org_to_group(NULL::uuid,NULL::uuid)",
    ),
    (
        "public.platform_remove_org_from_group(uuid,uuid)",
        "SELECT public.platform_remove_org_from_group(NULL::uuid,NULL::uuid)",
    ),
    (
        "public.platform_set_organization_status(uuid,text)",
        "SELECT public.platform_set_organization_status(NULL::uuid,NULL::text)",
    ),
    (
        "public.platform_remove_organization(uuid)",
        "SELECT public.platform_remove_organization(NULL::uuid)",
    ),
    (
        "public.platform_create_group(text,text)",
        "SELECT public.platform_create_group(NULL::text,NULL::text)",
    ),
    (
        "public.platform_update_group(uuid,text,text)",
        "SELECT public.platform_update_group(NULL::uuid,NULL::text,NULL::text)",
    ),
    (
        "public.platform_update_group(uuid,text,text,text)",
        "SELECT public.platform_update_group(NULL::uuid,NULL::text,NULL::text,NULL::text)",
    ),
    (
        "public.platform_create_group_account(uuid,uuid,text,text,text[],text,uuid)",
        "SELECT public.platform_create_group_account(NULL::uuid,NULL::uuid,NULL::text,NULL::text,NULL::text[],NULL::text,NULL::uuid)",
    ),
    (
        "public.platform_revoke_group_role(uuid,uuid,text)",
        "SELECT public.platform_revoke_group_role(NULL::uuid,NULL::uuid,NULL::text)",
    ),
];

async fn private_calls_refuse(pool: &PgPool) {
    for (_, statement) in NATIVE_PRIVATE_CALLS
        .iter()
        .chain(PRIVATE_RAW_TOPOLOGY_CALLS)
    {
        let error = sqlx::query(statement)
            .execute(pool)
            .await
            .expect_err("private owner routine was callable by a serving login");
        assert_eq!(
            error.as_database_error().and_then(|e| e.code()).as_deref(),
            Some("42501")
        );
    }
}

#[sqlx::test(migrations = false)]
async fn company_owner_private_surface_refuses_every_real_serving_login(pool: PgPool) {
    let (_app, _account, _cookies, startup, _) = designated(&pool).await;
    // Existence is a separate prerequisite: missing schema/function cannot satisfy
    // permission-denied assertions. Do not catch 42883 or 3F000 as a pass.
    for (signature, _) in NATIVE_PRIVATE_CALLS
        .iter()
        .chain(PRIVATE_RAW_TOPOLOGY_CALLS)
    {
        let exists: bool = sqlx::query_scalar("SELECT to_regprocedure($1) IS NOT NULL")
            .bind(signature)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!(
            exists,
            "exact Company owner routine prerequisite absent: {signature}"
        );
    }
    let before = all_rows(&pool).await;
    for login in [
        TestDatabaseLogin::Business,
        TestDatabaseLogin::Auth,
        TestDatabaseLogin::LeaveCommand,
        TestDatabaseLogin::OntologyCommand,
        TestDatabaseLogin::PlatformForceCommand,
    ] {
        let serving = login_test_pool(&pool, login).await;
        private_calls_refuse(&serving).await;
        let no_binding_read: bool = sqlx::query_scalar("SELECT NOT has_table_privilege(current_user,'public.company_enrollment_effect_bindings','SELECT') AND NOT has_any_column_privilege(current_user,'public.company_enrollment_effect_bindings','SELECT')")
            .fetch_one(&serving).await.unwrap();
        assert!(no_binding_read, "serving login may inspect private binding");
        for owner in [
            "console_account_owner",
            "console_ontology_writer",
            "console_app",
        ] {
            let can_assume: bool = sqlx::query_scalar(
                "SELECT pg_has_role(current_user,$1,'SET') OR pg_has_role(current_user,$1,'USAGE')",
            )
            .bind(owner)
            .fetch_one(&serving)
            .await
            .unwrap();
            assert!(!can_assume, "serving login may assume an owner");
        }
        serving.close().await;
    }
    private_calls_refuse(&startup).await;
    assert!(
        before == all_rows(&pool).await,
        "denied calls changed business state"
    );
    startup.close().await;
}

#[sqlx::test(migrations = false)]
async fn company_birth_has_exact_catalog_actor_context_and_audit_effects(pool: PgPool) {
    let (app, operator, cookies, startup, _) = designated(&pool).await;
    // Independent recipient is an actual registered Account; the operator must
    // not receive incidental Company membership to satisfy attribution FKs.
    let (recipient, _) = enrolled(&app).await;
    let command = Uuid::new_v4();
    let input = enrollment(command, recipient.account);
    let before = all_rows(&pool).await;
    let result = committed(
        &submit(&app, &cookies, &input).await,
        StatusCode::CREATED,
        command,
        recipient.account,
        false,
    );
    durable(&pool, &result, &input, operator.account).await;
    let after = all_rows(&pool).await;
    assert_native_catalog_content(&after, result.company);
    assert_native_catalog_attribution(&after, &result, operator.account);
    for (table, count) in [
        ("organizations", 1),
        ("groups", 1),
        ("group_memberships", 1),
        ("company_actors", 1),
        ("policy_roles", 1),
        ("user_role_assignments", 1),
        ("policy_role_revisions", 1),
        ("policy_assignment_revisions", 1),
        ("ont_object_types", 2),
        ("ont_action_types", 5),
        ("ont_property_defs", 10),
        ("ont_object_policies", 2),
        ("cedar_policy_catalog_entries", 2),
        ("company_enrollment_effect_bindings", 1),
        ("company_enrollment_receipts", 1),
        ("company_enrollment_request_events", 2),
        ("audit_events", 5),
    ] {
        let added = added_rows(
            before
                .get(table)
                .expect("effect table missing before command"),
            after
                .get(table)
                .expect("effect table missing after command"),
        )
        .expect("prior effect history changed");
        assert_eq!(
            added.len(),
            count,
            "incorrect exact birth effect census for {table}"
        );
    }
    let actors = added_rows(&before["company_actors"], &after["company_actors"]).unwrap();
    assert_eq!(actors[0]["account_id"], json!(recipient.account));
    assert_eq!(actors[0]["admission_receipt_id"], json!(result.receipt));
    let evidence = &actors[0]["entitlement_ref"];
    assert_eq!(
        evidence,
        &json!({"kind":"COMPANY_ENROLLMENT_V1",
        "account_id":operator.account,"command_id":command,
        "org_id":result.company,"receipt_id":result.receipt})
    );
    assert_eq!(
        before["users"], after["users"],
        "native enrollment manufactured a legacy user"
    );
    let audits = added_rows(&before["audit_events"], &after["audit_events"]).unwrap();
    let mut actions: Vec<_> = audits
        .iter()
        .map(|v| v["action"].as_str().unwrap())
        .collect();
    actions.sort_unstable();
    assert_eq!(
        actions,
        vec![
            "company.enroll",
            "ontology.object_policy.attach",
            "ontology.object_policy.attach",
            "ontology.object_type.builtin_install",
            "ontology.object_type.builtin_install"
        ]
    );
    for audit in &audits {
        assert_eq!(audit["actor"], json!(operator.account));
        assert_eq!(audit["org_id"], json!(result.company));
        assert_eq!(
            audit["after_snap"]["enrollment"]["receipt_id"],
            json!(result.receipt)
        );
        assert_eq!(
            audit["after_snap"]["enrollment"]["command_id"],
            json!(command)
        );
    }
    let replay = committed(
        &submit(&app, &cookies, &input).await,
        StatusCode::OK,
        command,
        recipient.account,
        true,
    );
    assert_eq!(replay.receipt, result.receipt);
    assert!(
        after == all_rows(&pool).await,
        "exact replay added an effect"
    );
    startup.close().await;
}

// Replaces only the retained failure test's request-only exclusion block.
// The caller still compares every other public base table byte-for-byte.
pub(super) async fn assert_only_pending_company_intake(
    pool: &PgPool,
    before: &BTreeMap<String, String>,
    failed: &BTreeMap<String, String>,
    account: Uuid,
    command: Uuid,
) {
    let mut before_effects = before.clone();
    let mut failed_effects = failed.clone();
    let requests_before = before_effects
        .remove("company_enrollment_requests")
        .unwrap();
    let requests_after = failed_effects
        .remove("company_enrollment_requests")
        .unwrap();
    let events_before = before_effects
        .remove("company_enrollment_request_events")
        .unwrap();
    let events_after = failed_effects
        .remove("company_enrollment_request_events")
        .unwrap();
    assert!(
        before_effects == failed_effects,
        "failed composition retained partial topology/catalog/identity/context/receipt/audit"
    );
    let requests = added_rows(&requests_before, &requests_after).expect("request history changed");
    let events = added_rows(&events_before, &events_after).expect("intake event history changed");
    assert_eq!(requests.len(), 1);
    assert_eq!(events.len(), 1);
    let request = &requests[0];
    let event = &events[0];
    assert_eq!(request["account_id"], json!(account));
    assert_eq!(request["command_id"], json!(command));
    assert_eq!(request["state"], "PENDING");
    assert!(request["committed_receipt_id"].is_null() && request["terminal_at"].is_null());
    assert_eq!(request["codec_version"], 1);
    exact_keys(
        event,
        &[
            "account_id",
            "command_id",
            "event_revision",
            "from_state",
            "to_state",
            "occurred_at",
            "actor_account_id",
            "session_id",
            "reason_code",
        ],
    );
    assert_eq!(event["account_id"], json!(account));
    assert_eq!(event["command_id"], json!(command));
    assert_eq!(event["actor_account_id"], json!(account));
    assert_eq!(event["event_revision"], 1);
    assert!(event["from_state"].is_null());
    assert_eq!(event["to_state"], "PENDING");
    assert_eq!(event["reason_code"], "PREPARED");
    assert_eq!(event["occurred_at"], request["created_at"]);
    let families: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM public.auth_refresh_token_families WHERE user_id=$1 AND protocol='ACCOUNT_V1' AND revoked_at IS NULL")
        .bind(account).fetch_all(pool).await.unwrap();
    assert_eq!(
        families.len(),
        1,
        "fixture requires exactly its genuine current family"
    );
    assert_eq!(event["session_id"], json!(families[0]));
}

#[sqlx::test(migrations = false)]
async fn missing_enrollment_audit_cannot_commit_any_birth_effect(pool: PgPool) {
    let (app, account, cookies, startup, _) = designated(&pool).await;
    let command = Uuid::new_v4();
    let input = enrollment(command, account.account);
    let before = all_rows(&pool).await;
    // Omit rather than raise: only actual owner closure can refuse this commit.
    // The nontransactional sequence proves the exact audit boundary was reached.
    sqlx::raw_sql(
        r#"
        CREATE SEQUENCE public.company_setup_omitted_audit_seen;
        CREATE FUNCTION public.company_setup_omit_enrollment_audit() RETURNS trigger
        LANGUAGE plpgsql SECURITY DEFINER SET search_path=pg_catalog,pg_temp AS $$
        BEGIN
            IF NEW.action='company.enroll' THEN
                PERFORM nextval('public.company_setup_omitted_audit_seen'::regclass);
                RETURN NULL;
            END IF;
            RETURN NEW;
        END $$;
        CREATE TRIGGER company_setup_omit_enrollment_audit BEFORE INSERT ON public.audit_events
        FOR EACH ROW EXECUTE FUNCTION public.company_setup_omit_enrollment_audit();
    "#,
    )
    .execute(&pool)
    .await
    .unwrap();
    let response = submit(&app, &cookies, &input).await;
    let fired: bool = sqlx::query_scalar(
        "SELECT is_called AND last_value=1 FROM public.company_setup_omitted_audit_seen",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::raw_sql("DROP TRIGGER company_setup_omit_enrollment_audit ON public.audit_events; DROP FUNCTION public.company_setup_omit_enrollment_audit(); DROP SEQUENCE public.company_setup_omitted_audit_seen;")
        .execute(&pool).await.unwrap();
    assert!(
        fired,
        "actual enrollment audit was not reached; unrelated failure is not closure evidence"
    );
    response.error(
        StatusCode::SERVICE_UNAVAILABLE,
        "company_enrollment_unavailable",
    );
    let failed = all_rows(&pool).await;
    assert_only_pending_company_intake(&pool, &before, &failed, account.account, command).await;
    let result = committed(
        &submit(&app, &cookies, &input).await,
        StatusCode::CREATED,
        command,
        account.account,
        false,
    );
    durable(&pool, &result, &input, account.account).await;
    let complete = all_rows(&pool).await;
    let replay = committed(
        &submit(&app, &cookies, &input).await,
        StatusCode::OK,
        command,
        account.account,
        true,
    );
    assert_eq!(result.receipt, replay.receipt);
    assert!(
        complete == all_rows(&pool).await,
        "replay after fault repair added an effect"
    );
    startup.close().await;
}
