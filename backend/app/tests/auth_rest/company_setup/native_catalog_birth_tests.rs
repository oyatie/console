use super::*;

// Preserve this retained historical oracle's original schema predicate.
pub(super) fn native_catalog_attribution_matches(
    rows: &BTreeMap<String, String>,
    r: &Committed,
    operator: Uuid,
) -> bool {
    birth_oracles::native_catalog_attribution_with_schema(
        rows,
        r,
        operator,
        "ontology-runtime-filter-v1",
    )
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
