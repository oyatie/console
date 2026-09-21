use super::*;

// Additive include inside existing company_setup module, alongside
// effect-tests-current.rs. No new authority fixture: all native rows come from
// the mounted real submit after genuine Account enrollment/designation.
const IDENTITY_CATALOG_VERSION: &str = "native-company-identity-2026-09-19.1";
const IDENTITY_MANIFEST_BYTEA: &str =
    "\\x0d3d0c3bc0357c0394b02400295f77231178cd5dc22a668a90880fc92a089935";

pub(super) fn identity_rows(rows: &BTreeMap<String, String>, table: &str, org: Uuid) -> Vec<Value> {
    serde_json::from_str::<Vec<Value>>(&rows[table])
        .unwrap()
        .into_iter()
        .filter(|row| row["org_id"] == json!(org))
        .collect()
}

pub(super) fn identity_graph_matches(
    rows: &BTreeMap<String, String>,
    r: &Committed,
    operator: Uuid,
) -> bool {
    let check = || -> Option<()> {
        let one = |table| {
            let mut values = identity_rows(rows, table, r.company);
            (values.len() == 1).then(|| values.remove(0))
        };
        let role = one("policy_roles")?;
        let assignment = one("user_role_assignments")?;
        let rr = one("policy_role_revisions")?;
        let ar = one("policy_assignment_revisions")?;
        let head = one("company_authority_heads")?;
        let binding = one("company_enrollment_effect_bindings")?;
        let receipt = one("company_enrollment_receipts")?;
        let actor = one("company_actors")?;
        let install = one("native_company_catalog_installs")?;
        if role["subject_protocol"] != "NATIVE_ACCOUNT"
            || assignment["subject_protocol"] != "NATIVE_ACCOUNT"
            || rr["subject_protocol"] != "NATIVE_ACCOUNT"
            || ar["subject_protocol"] != "NATIVE_ACCOUNT"
            || role["native_current_revision"] != 1
            || assignment["native_current_revision"] != 1
            || rr["revision"] != 1
            || ar["revision"] != 1
            || head["epoch"] != 1
            || role["role_key"] != "native_company_administration"
            || role["display_name"] != "회사 초기 관리자"
            || !role["description"].is_null()
            || role["is_system"] != true
            || role["created_at"] != binding["started_at"]
            || role["updated_at"] != binding["started_at"]
            || assignment["created_at"] != binding["started_at"]
            || install["installed_at"] != binding["started_at"]
            || install["catalog_version"] != IDENTITY_CATALOG_VERSION
            || install["manifest_digest"] != IDENTITY_MANIFEST_BYTEA
            || rr["catalog_version"] != IDENTITY_CATALOG_VERSION
            || rr["manifest_digest"] != IDENTITY_MANIFEST_BYTEA
            || role["status"] != "ACTIVE"
            || rr["state"] != "ACTIVE"
            || ar["state"] != "ACTIVE"
            || role["created_by"] != Value::Null
            || role["updated_by"] != Value::Null
            || assignment["user_id"] != Value::Null
            || assignment["assigned_by"] != Value::Null
            || role["created_by_account_id"] != json!(operator)
            || role["updated_by_account_id"] != json!(operator)
            || assignment["assigned_by_account_id"] != json!(operator)
            || assignment["account_id"] != json!(r.administrator)
            || ar["account_id"] != json!(r.administrator)
            || assignment["role_id"] != role["id"]
            || rr["role_id"] != role["id"]
            || ar["role_id"] != role["id"]
            || ar["role_revision"] != rr["revision"]
            || ar["assignment_id"] != assignment["id"]
            || receipt["root_assignment_id"] != assignment["id"]
            || receipt["root_revision"] != ar["revision"]
            || ar["ceiling_digest"] != rr["clause_digest"]
            || actor["account_id"] != json!(r.administrator)
            || actor["admission_receipt_id"] != json!(r.receipt)
            || actor["entitlement_ref"]
                != json!({"kind":"COMPANY_ENROLLMENT_V1","account_id":operator,
                "command_id":r.command,"org_id":r.company,"receipt_id":r.receipt})
            || !identity_rows(rows, "policy_role_permissions", r.company).is_empty()
            || !identity_rows(rows, "policy_role_conditions", r.company).is_empty()
        {
            return None;
        }
        for row in [&role, &assignment, &rr, &ar, &head, &install] {
            if row["origin_account_id"] != json!(operator)
                || row["origin_command_id"] != json!(r.command)
                || row["origin_receipt_id"] != json!(r.receipt)
            {
                return None;
            }
        }
        for row in [&rr, &ar] {
            if row["actor_account_id"] != json!(operator)
                || row["session_id"] != binding["session_id"]
                || row["valid_from"] != binding["started_at"]
                || !row["valid_until"].is_null()
                || row["created_at"] != binding["started_at"]
            {
                return None;
            }
        }
        let clauses = identity_rows(rows, "policy_capability_clauses", r.company);
        let fields = identity_rows(rows, "policy_capability_clause_fields", r.company);
        let objects = identity_rows(rows, "native_company_object_refs", r.company);
        let actions = identity_rows(rows, "native_company_action_refs", r.company);
        let properties = identity_rows(rows, "native_company_property_refs", r.company);
        if clauses.len() != 7
            || fields.len() != 16
            || objects.len() != 2
            || actions.len() != 5
            || properties.len() != 10
        {
            return None;
        }
        for mapping in objects
            .iter()
            .chain(actions.iter())
            .chain(properties.iter())
        {
            if mapping["catalog_version"] != IDENTITY_CATALOG_VERSION
                || mapping["manifest_digest"] != IDENTITY_MANIFEST_BYTEA
            {
                return None;
            }
        }
        let actual_types = identity_rows(rows, "ont_object_types", r.company);
        let actual_actions = identity_rows(rows, "ont_action_types", r.company);
        let actual_properties = identity_rows(rows, "ont_property_defs", r.company);
        for mapping in &objects {
            if mapping["schema_revision"] != 1
                || !actual_types.iter().any(|row| {
                    row["id"] == mapping["object_type_id"]
                        && row["stable_key"] == mapping["object_key"]
                        && row["schema_version"] == mapping["schema_revision"]
                        && row["lifecycle_state"] == "published"
                })
            {
                return None;
            }
        }
        for mapping in &actions {
            if mapping["registration_revision"] != 1
                || !actual_actions.iter().any(|row| {
                    row["id"] == mapping["action_type_id"]
                        && row["object_type_id"] == mapping["object_type_id"]
                        && row["dispatch_target"] == mapping["action_key"]
                })
            {
                return None;
            }
        }
        for mapping in &properties {
            let key = mapping["property_key"].as_str()?;
            let (prefix, key) = key.split_once('.')?;
            let object_key = match prefix {
                "company" => "company_workspace",
                "assignment" => "company_policy_assignment",
                _ => return None,
            };
            if mapping["schema_revision"] != 1
                || !objects.iter().any(|o| {
                    o["object_key"] == object_key
                        && o["object_type_id"] == mapping["object_type_id"]
                })
                || !actual_properties.iter().any(|row| {
                    row["id"] == mapping["property_id"]
                        && row["object_type_id"] == mapping["object_type_id"]
                        && row["key"] == key
                })
            {
                return None;
            }
        }
        let candidates: Vec<Value> =
            serde_json::from_str(&rows["account_context_candidates"]).ok()?;
        let candidates: Vec<_> = candidates
            .iter()
            .filter(|c| c["context_id"] == json!(r.company))
            .collect();
        if candidates.len() != 1 {
            return None;
        }
        let candidate = candidates[0];
        if candidate["account_id"] != json!(r.administrator)
            || candidate["context_kind"] != "COMPANY"
            || candidate["state"] != "CURRENT"
            || candidate["source_revision"] != 1
            || !candidate["incarnation"].is_null()
            || candidate["source_key"]
                != json!({"kind":"COMPANY_POLICY","org_id":r.company,"assignment_id":assignment["id"],"revision":"1"})
        {
            return None;
        }
        let expected = [
            (1, "context.discover", false, 2),
            (2, "company.identity.read", false, 2),
            (3, "company.policy.read", false, 8),
            (4, "company.policy.assign", false, 0),
            (5, "company.policy.revoke", false, 0),
            (6, "context.discover", true, 2),
            (7, "company.identity.read", true, 2),
        ];
        for (index, key, delegable, count) in expected {
            let selected: Vec<_> = clauses
                .iter()
                .filter(|c| c["clause_index"] == index)
                .collect();
            if selected.len() != 1 {
                return None;
            }
            let c = selected[0];
            let selected: Vec<_> = actions.iter().filter(|a| a["action_key"] == key).collect();
            if selected.len() != 1 {
                return None;
            }
            let a = selected[0];
            if c["role_id"] != role["id"]
                || c["role_revision"] != rr["revision"]
                || c["effect"] != "ALLOW"
                || c["delegable"] != delegable
                || c["resource_org_id"] != json!(r.company)
                || c["action_type_id"] != a["action_type_id"]
                || c["action_object_type_id"] != a["object_type_id"]
                || c["registration_revision"] != a["registration_revision"]
                || c["manifest_digest"] != a["manifest_digest"]
                || c["manifest_digest"] != install["manifest_digest"]
                || rr["manifest_digest"] != install["manifest_digest"]
                || c["valid_from"] != binding["started_at"]
                || !c["valid_until"].is_null()
            {
                return None;
            }
            let selected: Vec<_> = fields
                .iter()
                .filter(|f| f["clause_index"] == index)
                .collect();
            if selected.len() != count {
                return None;
            }
            let mut keys = BTreeSet::new();
            for f in selected {
                if f["role_id"] != role["id"]
                    || f["role_revision"] != rr["revision"]
                    || f["object_type_id"] != c["action_object_type_id"]
                    || f["manifest_digest"] != install["manifest_digest"]
                {
                    return None;
                }
                let matching: Vec<_> = properties
                    .iter()
                    .filter(|p| {
                        p["property_id"] == f["property_id"]
                            && p["object_type_id"] == f["object_type_id"]
                            && p["schema_revision"] == f["schema_revision"]
                            && p["manifest_digest"] == f["manifest_digest"]
                            && p["content_digest"] == f["content_digest"]
                    })
                    .collect();
                if matching.len() != 1 || !keys.insert(matching[0]["property_key"].as_str()?) {
                    return None;
                }
            }
            let expected: BTreeSet<_> = if count == 2 {
                ["company.name", "company.slug"].into_iter().collect()
            } else if count == 8 {
                [
                    "assignment.account_id",
                    "assignment.scope",
                    "assignment.actions",
                    "assignment.fields",
                    "assignment.valid_from",
                    "assignment.valid_until",
                    "assignment.state",
                    "assignment.revision",
                ]
                .into_iter()
                .collect()
            } else {
                BTreeSet::new()
            };
            if keys != expected {
                return None;
            }
        }
        Some(())
    };
    check().is_some()
}

async fn identity_digests_match(pool: &PgPool, rows: &BTreeMap<String, String>, r: &Committed) {
    let clauses = identity_rows(rows, "policy_capability_clauses", r.company);
    let fields = identity_rows(rows, "policy_capability_clause_fields", r.company);
    let mut expected_role = Vec::new();
    let start: String=sqlx::query_scalar("SELECT to_char(started_at AT TIME ZONE 'UTC','YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"') FROM public.company_enrollment_effect_bindings WHERE org_id=$1")
        .bind(r.company).fetch_one(pool).await.unwrap();
    for index in 1..=7 {
        let c = clauses.iter().find(|c| c["clause_index"] == index).unwrap();
        let mut refs: Vec<_> = fields
            .iter()
            .filter(|f| f["clause_index"] == index)
            .map(|f| {
                json!({
            "org_id":r.company,"object_type_id":f["object_type_id"],"property_id":f["property_id"],
            "schema_revision":f["schema_revision"].as_i64().unwrap().to_string()})
            })
            .collect();
        refs.sort_by(|a, b| {
            (
                a["object_type_id"].as_str(),
                a["property_id"].as_str(),
                a["schema_revision"].as_str(),
            )
                .cmp(&(
                    b["object_type_id"].as_str(),
                    b["property_id"].as_str(),
                    b["schema_revision"].as_str(),
                ))
        });
        let envelope = json!({"kind":"COMPANY_CAPABILITY_CLAUSE_V1","action":{
            "org_id":r.company,"action_type_id":c["action_type_id"],"object_type_id":c["action_object_type_id"],
            "registration_revision":c["registration_revision"].as_i64().unwrap().to_string(),
            "manifest_digest":c["manifest_digest"].as_str().unwrap().strip_prefix("\\x").unwrap()},
            "resource":{"kind":"COMPANY","org_id":r.company},"fields":refs,
            "valid_from":start,"valid_until":null,"delegable":c["delegable"]});
        let digest: String =
            sqlx::query_scalar("SELECT encode(sha256(convert_to($1::jsonb::text,'UTF8')),'hex')")
                .bind(envelope)
                .fetch_one(pool)
                .await
                .unwrap();
        assert!(
            c["clause_digest"] == json!(format!("\\x{digest}")),
            "clause digest is not exact correlated envelope"
        );
        expected_role.push(json!({"clause_index":index,"clause_digest":digest}));
    }
    let digest: String =
        sqlx::query_scalar("SELECT encode(sha256(convert_to($1::jsonb::text,'UTF8')),'hex')")
            .bind(json!(expected_role))
            .fetch_one(pool)
            .await
            .unwrap();
    for (mapping_table, physical_table, id_field, keys) in [
        (
            "native_company_object_refs",
            "ont_object_types",
            "object_type_id",
            &[
                "stable_key",
                "title",
                "title_property_key",
                "backing_kind",
                "backing_table",
                "primary_key_property",
                "schema_version",
                "lifecycle_state",
            ][..],
        ),
        (
            "native_company_action_refs",
            "ont_action_types",
            "action_type_id",
            &[
                "stable_key",
                "title",
                "params_schema",
                "edits",
                "submission_criteria",
                "side_effects",
                "dispatch",
                "dispatch_target",
                "control_points",
            ][..],
        ),
        (
            "native_company_property_refs",
            "ont_property_defs",
            "property_id",
            &[
                "key",
                "title",
                "type",
                "config",
                "backing_column",
                "required",
                "in_property_policy",
            ][..],
        ),
    ] {
        let physical = identity_rows(rows, physical_table, r.company);
        for mapping in identity_rows(rows, mapping_table, r.company) {
            let row = physical
                .iter()
                .find(|p| p["id"] == mapping[id_field])
                .unwrap();
            let content: serde_json::Map<String, Value> = keys
                .iter()
                .map(|key| ((*key).to_owned(), row[*key].clone()))
                .collect();
            let expected: String = sqlx::query_scalar(
                "SELECT encode(sha256(convert_to($1::jsonb::text,'UTF8')),'hex')",
            )
            .bind(Value::Object(content))
            .fetch_one(pool)
            .await
            .unwrap();
            assert!(
                mapping["content_digest"] == json!(format!("\\x{expected}")),
                "physical mapping content differs"
            );
        }
    }
    let rr = identity_rows(rows, "policy_role_revisions", r.company);
    assert!(
        rr[0]["clause_digest"] == json!(format!("\\x{digest}")),
        "role digest omits/reorders a clause"
    );
}

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
