use super::*;

// Additive include inside existing company_setup module, alongside
// effect-tests-current.rs. No new authority fixture: all native rows come from
// the mounted real submit after genuine Account enrollment/designation.

#[sqlx::test(migrations = false)]
async fn identity_birth_exact_correlated_graph_and_digest_with_oracle_corruptions(pool: PgPool) {
    let (app, operator, cookies, startup, _) = designated(&pool).await;
    let (recipient, _) = enrolled(&app).await;
    let command = Uuid::new_v4();
    let input = enrollment(command, recipient.account);
    let before = all_rows(&pool).await;
    let r = committed(
        &submit(&app, &cookies, &input).await,
        StatusCode::CREATED,
        command,
        recipient.account,
        false,
    );
    let rows = all_rows(&pool).await;
    assert!(
        identity_graph_matches(&rows, &r, operator.account),
        "actual birth graph differs"
    );
    identity_digests_match(&pool, &rows, &r).await;
    assert_native_catalog_content(&rows, r.company);
    assert!(
        before["users"] == rows["users"],
        "native birth invented a legacy User"
    );
    let security = |snapshot: &BTreeMap<String, String>, account: Uuid| -> i64 {
        let values: Vec<Value> = serde_json::from_str(&snapshot["account_security"]).unwrap();
        values
            .iter()
            .find(|v| v["account_id"] == json!(account))
            .unwrap()["context_generation"]
            .as_i64()
            .unwrap()
    };
    assert_eq!(
        security(&rows, recipient.account),
        security(&before, recipient.account) + 1
    );
    assert_eq!(
        security(&rows, operator.account),
        security(&before, operator.account)
    );
    // Controls mutate only captured observations, never a successful fixture.
    for (table, field, bad) in [
        ("policy_roles", "native_current_revision", json!(2)),
        ("policy_roles", "role_key", json!("unexpected_role")),
        ("policy_roles", "display_name", json!("unexpected label")),
        (
            "policy_roles",
            "description",
            json!("unexpected description"),
        ),
        ("policy_roles", "is_system", json!(false)),
        (
            "policy_roles",
            "created_at",
            json!("2000-01-01T00:00:00+00:00"),
        ),
        (
            "policy_roles",
            "updated_at",
            json!("2000-01-01T00:00:00+00:00"),
        ),
        (
            "user_role_assignments",
            "created_at",
            json!("2000-01-01T00:00:00+00:00"),
        ),
        (
            "native_company_catalog_installs",
            "installed_at",
            json!("2000-01-01T00:00:00+00:00"),
        ),
        (
            "native_company_catalog_installs",
            "catalog_version",
            json!("wrong-version"),
        ),
        (
            "native_company_catalog_installs",
            "manifest_digest",
            json!("\\x00"),
        ),
        (
            "native_company_object_refs",
            "catalog_version",
            json!("wrong-version"),
        ),
        (
            "native_company_object_refs",
            "manifest_digest",
            json!("\\x00"),
        ),
        (
            "native_company_action_refs",
            "catalog_version",
            json!("wrong-version"),
        ),
        (
            "native_company_action_refs",
            "manifest_digest",
            json!("\\x00"),
        ),
        (
            "native_company_property_refs",
            "catalog_version",
            json!("wrong-version"),
        ),
        (
            "native_company_property_refs",
            "manifest_digest",
            json!("\\x00"),
        ),
        (
            "policy_role_revisions",
            "catalog_version",
            json!("wrong-version"),
        ),
        ("policy_role_revisions", "manifest_digest", json!("\\x00")),
        ("policy_role_revisions", "role_id", json!(Uuid::new_v4())),
        (
            "policy_assignment_revisions",
            "ceiling_digest",
            json!("\\x00"),
        ),
        (
            "policy_assignment_revisions",
            "account_id",
            json!(operator.account),
        ),
        ("policy_capability_clauses", "delegable", json!("true")),
        (
            "policy_capability_clauses",
            "action_type_id",
            json!(Uuid::new_v4()),
        ),
        (
            "policy_capability_clause_fields",
            "property_id",
            json!(Uuid::new_v4()),
        ),
        (
            "policy_capability_clause_fields",
            "content_digest",
            json!("\\x00"),
        ),
        (
            "native_company_action_refs",
            "action_key",
            json!("company.unregistered"),
        ),
        (
            "native_company_property_refs",
            "property_key",
            json!("unregistered.field"),
        ),
        ("company_authority_heads", "epoch", json!(2)),
        (
            "company_actors",
            "entitlement_ref",
            json!({"kind":"COMPANY_ENROLLMENT_V1"}),
        ),
    ] {
        let mut corrupt = rows.clone();
        let mut values: Vec<Value> = serde_json::from_str(&corrupt[table]).unwrap();
        values
            .iter_mut()
            .find(|v| v["org_id"] == json!(r.company))
            .unwrap()[field] = bad;
        corrupt.insert(table.to_owned(), serde_json::to_string(&values).unwrap());
        assert!(
            !identity_graph_matches(&corrupt, &r, operator.account),
            "oracle accepted corrupt {table}.{field}"
        );
    }
    {
        let mut corrupt = rows.clone();
        let mut values: Vec<Value> =
            serde_json::from_str(&corrupt["policy_capability_clauses"]).unwrap();
        let clause = values
            .iter_mut()
            .find(|v| v["org_id"] == json!(r.company))
            .unwrap();
        clause["delegable"] = json!(!clause["delegable"].as_bool().unwrap());
        corrupt.insert(
            "policy_capability_clauses".into(),
            serde_json::to_string(&values).unwrap(),
        );
        assert!(
            !identity_graph_matches(&corrupt, &r, operator.account),
            "oracle accepted actual delegation toggle"
        );
    }
    for table in [
        "policy_capability_clauses",
        "policy_capability_clause_fields",
        "policy_role_revisions",
        "policy_assignment_revisions",
    ] {
        let mut corrupt = rows.clone();
        let mut values: Vec<Value> = serde_json::from_str(&corrupt[table]).unwrap();
        let i = values
            .iter()
            .position(|v| v["org_id"] == json!(r.company))
            .unwrap();
        values.remove(i);
        corrupt.insert(table.to_owned(), serde_json::to_string(&values).unwrap());
        assert!(
            !identity_graph_matches(&corrupt, &r, operator.account),
            "oracle accepted omitted {table}"
        );
    }
    let replay = committed(
        &submit(&app, &cookies, &input).await,
        StatusCode::OK,
        command,
        recipient.account,
        true,
    );
    assert_eq!(replay.receipt, r.receipt);
    assert!(rows == all_rows(&pool).await);
    startup.close().await;
}

#[sqlx::test(migrations = false)]
async fn identity_missing_actual_clause_field_cannot_commit_and_same_command_recovers(
    pool: PgPool,
) {
    let (app, account, cookies, startup, _) = designated(&pool).await;
    // Exact table/function presence is prerequisite; missing schema is not a
    // useful closure RED. Test only records closure after this sequence fires.
    sqlx::raw_sql(r#"
      CREATE SEQUENCE public.native_identity_omitted_field_seen;
      CREATE FUNCTION public.native_identity_omit_field() RETURNS trigger
      LANGUAGE plpgsql SECURITY DEFINER SET search_path=pg_catalog,pg_temp AS $$
      BEGIN
        IF NEW.clause_index=3 AND EXISTS(SELECT 1 FROM public.native_company_property_refs p
            WHERE p.org_id=NEW.org_id AND p.property_id=NEW.property_id AND p.property_key='assignment.state') THEN
          PERFORM nextval('public.native_identity_omitted_field_seen'::regclass); RETURN NULL;
        END IF; RETURN NEW;
      END $$;
      CREATE TRIGGER native_identity_omit_field BEFORE INSERT ON public.policy_capability_clause_fields
        FOR EACH ROW EXECUTE FUNCTION public.native_identity_omit_field();
    "#).execute(&pool).await.unwrap();
    let command = Uuid::new_v4();
    let input = enrollment(command, account.account);
    let before = all_rows(&pool).await;
    let response = submit(&app, &cookies, &input).await;
    let fired: bool = sqlx::query_scalar(
        "SELECT is_called AND last_value=1 FROM public.native_identity_omitted_field_seen",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::raw_sql("DROP TRIGGER native_identity_omit_field ON public.policy_capability_clause_fields; DROP FUNCTION public.native_identity_omit_field(); DROP SEQUENCE public.native_identity_omitted_field_seen;")
        .execute(&pool).await.unwrap();
    assert!(
        fired,
        "actual canonical field writer not reached; prerequisite failure is not closure evidence"
    );
    response.error(
        StatusCode::SERVICE_UNAVAILABLE,
        "company_enrollment_unavailable",
    );
    assert_only_pending_company_intake(
        &pool,
        &before,
        &all_rows(&pool).await,
        account.account,
        command,
    )
    .await;
    let r = committed(
        &submit(&app, &cookies, &input).await,
        StatusCode::CREATED,
        command,
        account.account,
        false,
    );
    let after = all_rows(&pool).await;
    assert!(identity_graph_matches(&after, &r, account.account));
    let replay = committed(
        &submit(&app, &cookies, &input).await,
        StatusCode::OK,
        command,
        account.account,
        true,
    );
    assert_eq!(r.receipt, replay.receipt);
    assert!(after == all_rows(&pool).await);
    startup.close().await;
}

#[sqlx::test(migrations = false)]
async fn identity_native_roots_and_history_refuse_real_business_dml(pool: PgPool) {
    let (app, account, cookies, startup, _) = designated(&pool).await;
    let command = Uuid::new_v4();
    let input = enrollment(command, account.account);
    let r = committed(
        &submit(&app, &cookies, &input).await,
        StatusCode::CREATED,
        command,
        account.account,
        false,
    );
    let before = all_rows(&pool).await;
    let business = login_test_pool(&pool, TestDatabaseLogin::Business).await;
    for statement in [
        "UPDATE public.policy_roles SET display_name=display_name WHERE org_id=$1",
        "DELETE FROM public.policy_roles WHERE org_id=$1",
        "UPDATE public.user_role_assignments SET role_id=role_id WHERE org_id=$1",
        "DELETE FROM public.user_role_assignments WHERE org_id=$1",
        "INSERT INTO public.policy_role_permissions(org_id,role_id,feature_key,permission_level) SELECT org_id,id,'login','allow' FROM public.policy_roles WHERE org_id=$1",
        "UPDATE public.policy_role_revisions SET state=state WHERE org_id=$1",
        "DELETE FROM public.policy_capability_clauses WHERE org_id=$1",
        "DELETE FROM public.policy_capability_clause_fields WHERE org_id=$1",
        "UPDATE public.policy_assignment_revisions SET revision=revision WHERE org_id=$1",
    ] {
        let mut tx = business.begin().await.unwrap();
        sqlx::query("SELECT set_config('app.current_org',$1,true)")
            .bind(r.company.to_string())
            .execute(tx.as_mut())
            .await
            .unwrap();
        let error = sqlx::query(statement)
            .bind(r.company)
            .execute(tx.as_mut())
            .await
            .expect_err("serving DML changed native authority");
        assert_eq!(
            error.as_database_error().and_then(|e| e.code()).as_deref(),
            Some("42501")
        );
        tx.rollback().await.unwrap();
        assert!(
            before == all_rows(&pool).await,
            "refused native statement changed durable rows"
        );
    }
    business.close().await;
    startup.close().await;
}
