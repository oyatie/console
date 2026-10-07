//! Shared exact Company birth oracles; no transport, fixtures or test discovery.
use super::Committed;
use serde_json::{Value, json};
use sqlx::PgPool;
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

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

pub(super) async fn identity_digests_match(
    pool: &PgPool,
    rows: &BTreeMap<String, String>,
    r: &Committed,
) {
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

pub(super) fn native_catalog_attribution_with_schema(
    rows: &BTreeMap<String, String>,
    r: &Committed,
    operator: Uuid,
    expected_schema: &str,
) -> bool {
    let check = || -> Option<()> {
        let bindings = identity_rows(rows, "company_enrollment_effect_bindings", r.company);
        if bindings.len() != 1 {
            return None;
        }
        let binding = &bindings[0];
        let types = identity_rows(rows, "ont_object_types", r.company);
        let installs = identity_rows(rows, "ont_builtin_catalog_installs", r.company);
        let catalogs = identity_rows(rows, "cedar_policy_catalog_entries", r.company);
        let policies = identity_rows(rows, "ont_object_policies", r.company);
        if types.len() != 2 || installs.len() != 1 || catalogs.len() != 2 || policies.len() != 2 {
            return None;
        }
        for row in types
            .iter()
            .chain(installs.iter())
            .chain(catalogs.iter())
            .chain(policies.iter())
        {
            if row["attribution_protocol"] != "NATIVE_ACCOUNT"
                || row["org_id"] != json!(r.company)
                || row["origin_account_id"] != json!(operator)
                || row["origin_command_id"] != json!(r.command)
                || row["origin_receipt_id"] != json!(r.receipt)
            {
                return None;
            }
        }
        for row in types.iter().chain(catalogs.iter()).chain(policies.iter()) {
            if row.get("created_by") != Some(&Value::Null)
                || row["created_by_account_id"] != json!(operator)
                || row["created_at"] != binding["started_at"]
            {
                return None;
            }
        }
        for row in &catalogs {
            if row.get("updated_by") != Some(&Value::Null)
                || row["updated_by_account_id"] != json!(operator)
                || row["updated_at"] != binding["started_at"]
                || row["effect"] != "forbid"
                || row["status"] != "enforced"
                || row.get("generated_policy_text") != Some(&Value::Null)
                || row["schema_version"] != expected_schema
            {
                return None;
            }
        }
        let install = &installs[0];
        if install.get("installed_by") != Some(&Value::Null)
            || install["installed_by_account_id"] != json!(operator)
            || install["installed_at"] != binding["started_at"]
            || install["catalog_version"] != "native-company-identity-2026-09-19.1"
            || install["manifest_digest"]
                != "\\x0d3d0c3bc0357c0394b02400295f77231178cd5dc22a668a90880fc92a089935"
        {
            return None;
        }
        let keys = identity_rows(rows, "ont_object_type_key_revisions", r.company);
        if keys.len() != 2 {
            return None;
        }
        for object in &types {
            let key = object["stable_key"].as_str()?;
            if !["company_workspace", "company_policy_assignment"].contains(&key)
                || object["updated_at"] != binding["started_at"]
            {
                return None;
            }
            let key_rows: Vec<_> = keys.iter().filter(|k| k["stable_key"] == key).collect();
            if key_rows.len() != 1 {
                return None;
            }
            let k = key_rows[0];
            let validator: Uuid = k["validator_id"].as_str()?.parse().ok()?;
            if validator.is_nil()
                || k["revision"] != 1
                || k["created_at"] != binding["started_at"]
                || k["updated_at"] != binding["started_at"]
            {
                return None;
            }
            let attached: Vec<_> = policies
                .iter()
                .filter(|p| p["object_type_id"] == object["id"])
                .collect();
            if attached.len() != 1 {
                return None;
            }
            let policy = attached[0];
            if policy["effect"] != "forbid" {
                return None;
            }
            let sources: Vec<_> = catalogs
                .iter()
                .filter(|c| c["id"] == policy["cedar_policy_id"])
                .collect();
            if sources.len() != 1 {
                return None;
            }
            if sources[0]["normalized_row"]
                != json!({"effect":"forbid","action":"view","resource_type":key,"conditions":[]})
            {
                return None;
            }
        }
        for table in [
            "ont_property_defs",
            "ont_action_types",
            "ont_link_types",
            "ont_analytics",
        ] {
            for child in identity_rows(rows, table, r.company) {
                if !types.iter().any(|t| t["id"] == child["object_type_id"])
                    || child["created_at"] != binding["started_at"]
                {
                    return None;
                }
            }
        }
        let audits = identity_rows(rows, "audit_events", r.company);
        if audits.len() != 5 {
            return None;
        }
        let mut covered = BTreeSet::new();
        let mut trace = None;
        let mut span = None;
        for audit in &audits {
            if audit["actor"] != json!(operator)
                || audit["occurred_at"] != binding["started_at"]
                || audit["after_snap"]["enrollment"]
                    != json!({"account_id":operator,"command_id":r.command,"receipt_id":r.receipt,"session_id":binding["session_id"]})
            {
                return None;
            }
            let t = audit["trace_id"].as_str()?;
            let s = audit["span_id"].as_str()?;
            if t.len() != 32
                || s.len() != 16
                || !t
                    .bytes()
                    .chain(s.bytes())
                    .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
                || t.bytes().all(|b| b == b'0')
                || s.bytes().all(|b| b == b'0')
            {
                return None;
            }
            if trace.is_some_and(|v| v != t) || span.is_some_and(|v| v != s) {
                return None;
            }
            trace = Some(t);
            span = Some(s);
            let action = audit["action"].as_str()?;
            let target = audit["target_id"].as_str()?;
            if !covered.insert((action, target)) {
                return None;
            }
            match action {
                "ontology.object_type.builtin_install" | "ontology.object_policy.attach" => {
                    let expected = if action == "ontology.object_type.builtin_install" {
                        "ont_object_types"
                    } else {
                        "ont_object_policies"
                    };
                    if audit["target_type"] != expected
                        || !types.iter().any(|o| o["id"].as_str() == Some(target))
                    {
                        return None;
                    }
                }
                "company.enroll" => {
                    if target != r.company.to_string() {
                        return None;
                    }
                }
                _ => return None,
            }
        }
        for object in &types {
            let id = object["id"].as_str()?;
            for action in [
                "ontology.object_type.builtin_install",
                "ontology.object_policy.attach",
            ] {
                if !covered.contains(&(action, id)) {
                    return None;
                }
            }
        }
        Some(())
    };
    check().is_some()
}

pub(super) fn assert_native_catalog_content(rows: &BTreeMap<String, String>, org: Uuid) {
    let manifest: Value =
        serde_json::from_str(include_str!("native-company-catalog-v1.json")).unwrap();
    let same_org = |table: &str| -> Vec<Value> {
        serde_json::from_str::<Vec<Value>>(&rows[table])
            .unwrap()
            .into_iter()
            .filter(|row| row["org_id"] == json!(org))
            .collect()
    };
    let types = same_org("ont_object_types");
    let properties = same_org("ont_property_defs");
    let actions = same_org("ont_action_types");
    assert_eq!(types.len(), 2);
    assert_eq!(properties.len(), 10);
    assert_eq!(actions.len(), 5);
    assert!(same_org("ont_link_types").is_empty());
    assert!(same_org("ont_analytics").is_empty());
    let mut ids = BTreeSet::new();
    for logical in manifest["object_types"].as_array().unwrap() {
        let physical: Vec<_> = types
            .iter()
            .filter(|row| row["stable_key"] == logical["stable_key"])
            .collect();
        assert_eq!(physical.len(), 1);
        let physical = physical[0];
        let id: Uuid = physical["id"].as_str().unwrap().parse().unwrap();
        assert!(!id.is_nil() && ids.insert(id));
        assert_eq!(physical["schema_version"], 1);
        assert_eq!(physical["lifecycle_state"], "published");
        assert!(
            physical["created_by"].is_null(),
            "native registry borrowed a legacy User identity"
        );
        for key in [
            "stable_key",
            "title",
            "title_property_key",
            "backing_kind",
            "backing_table",
            "primary_key_property",
        ] {
            assert_eq!(physical[key], logical[key], "object content differs: {key}");
        }
        for property in logical["properties"].as_array().unwrap() {
            let mapped: Vec<_> = properties
                .iter()
                .filter(|row| row["object_type_id"] == json!(id) && row["key"] == property["key"])
                .collect();
            assert_eq!(mapped.len(), 1);
            let row = mapped[0];
            let property_id: Uuid = row["id"].as_str().unwrap().parse().unwrap();
            assert!(!property_id.is_nil() && ids.insert(property_id));
            assert_eq!(row["type"], property["field_type"]);
            for key in [
                "key",
                "title",
                "config",
                "backing_column",
                "required",
                "in_property_policy",
            ] {
                assert_eq!(row[key], property[key], "property content differs: {key}");
            }
        }
        for action in logical["actions"].as_array().unwrap() {
            let mapped: Vec<_> = actions
                .iter()
                .filter(|row| {
                    row["object_type_id"] == json!(id) && row["stable_key"] == action["stable_key"]
                })
                .collect();
            assert_eq!(mapped.len(), 1);
            let row = mapped[0];
            let action_id: Uuid = row["id"].as_str().unwrap().parse().unwrap();
            assert!(!action_id.is_nil() && ids.insert(action_id));
            for key in [
                "stable_key",
                "title",
                "params_schema",
                "edits",
                "submission_criteria",
                "side_effects",
                "dispatch",
                "dispatch_target",
                "control_points",
            ] {
                assert_eq!(row[key], action[key], "action content differs: {key}");
            }
        }
    }
    assert_eq!(
        ids.len(),
        17,
        "physical type/action/property identities must be distinct"
    );
}
