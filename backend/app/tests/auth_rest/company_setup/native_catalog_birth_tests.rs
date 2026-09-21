use super::*;

pub(super) fn native_catalog_attribution_matches(
    rows: &BTreeMap<String, String>,
    r: &Committed,
    operator: Uuid,
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
                || row["schema_version"] != "ontology-runtime-filter-v1"
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

pub(super) fn assert_native_catalog_attribution(
    rows: &BTreeMap<String, String>,
    r: &Committed,
    operator: Uuid,
) {
    assert!(
        native_catalog_attribution_matches(rows, r, operator),
        "canonical native catalog attribution/audit closure differs"
    );
    for (table, key, bad) in [
        ("ont_object_types", "created_by", json!(operator)),
        (
            "ont_object_types",
            "created_by_account_id",
            json!(Uuid::new_v4()),
        ),
        (
            "ont_builtin_catalog_installs",
            "installed_by",
            json!(operator),
        ),
        (
            "ont_builtin_catalog_installs",
            "attribution_protocol",
            json!("LEGACY_USER"),
        ),
        (
            "cedar_policy_catalog_entries",
            "updated_by_account_id",
            json!(Uuid::new_v4()),
        ),
        (
            "cedar_policy_catalog_entries",
            "origin_receipt_id",
            json!(Uuid::new_v4()),
        ),
        (
            "cedar_policy_catalog_entries",
            "normalized_row",
            json!({"effect":"permit","action":"view","resource_type":"company_workspace","conditions":[]}),
        ),
        (
            "ont_object_policies",
            "cedar_policy_id",
            json!(Uuid::new_v4()),
        ),
        (
            "ont_object_policies",
            "created_at",
            json!("2000-01-01T00:00:00Z"),
        ),
        ("ont_object_type_key_revisions", "revision", json!(2)),
        (
            "ont_property_defs",
            "created_at",
            json!("2000-01-01T00:00:00Z"),
        ),
        (
            "audit_events",
            "target_id",
            json!(Uuid::new_v4().to_string()),
        ),
        (
            "audit_events",
            "trace_id",
            json!("11111111111111111111111111111111"),
        ),
        ("audit_events", "occurred_at", json!("2000-01-01T00:00:00Z")),
    ] {
        let mut corrupt = rows.clone();
        let mut values: Vec<Value> = serde_json::from_str(&corrupt[table]).unwrap();
        let row = values
            .iter_mut()
            .find(|v| v["org_id"] == json!(r.company))
            .unwrap();
        assert_ne!(
            row[key], bad,
            "ineffective native corruption control {table}.{key}"
        );
        row[key] = bad;
        corrupt.insert(table.into(), serde_json::to_string(&values).unwrap());
        assert!(
            !native_catalog_attribution_matches(&corrupt, r, operator),
            "native oracle accepted {table}.{key}"
        );
    }
    let mut corrupt = rows.clone();
    let mut audits: Vec<Value> = serde_json::from_str(&corrupt["audit_events"]).unwrap();
    let mut marker = audits
        .iter()
        .find(|a| a["org_id"] == json!(r.company))
        .unwrap()
        .clone();
    marker["id"] = json!(Uuid::new_v4());
    marker["action"] = json!("ontology.builtin_catalog.install");
    audits.push(marker);
    corrupt.insert(
        "audit_events".into(),
        serde_json::to_string(&audits).unwrap(),
    );
    assert!(
        !native_catalog_attribution_matches(&corrupt, r, operator),
        "native oracle accepted extra legacy marker audit"
    );
}
