use super::*;

// Pure projection over the complete real-database census. RawValue preserves
// old numeric/timestamp/string bytes; new IDs are checked independently.
pub(super) fn topology_backfill_matches(
    before: &BTreeMap<String, String>,
    after: &BTreeMap<String, String>,
) -> bool {
    use serde_json::value::RawValue;
    type Row<'a> = BTreeMap<String, &'a RawValue>;
    fn rows<'a>(census: &'a BTreeMap<String, String>, table: &str) -> Option<Vec<Row<'a>>> {
        serde_json::from_str(census.get(table)?).ok()
    }
    fn raw<'a>(row: &'a Row<'_>, key: &str) -> Option<&'a str> {
        Some(row.get(key)?.get())
    }
    fn uuid(row: &Row<'_>, key: &str) -> Option<Uuid> {
        let text: String = serde_json::from_str(raw(row, key)?).ok()?;
        let value: Uuid = text.parse().ok()?;
        (!value.is_nil() && value.hyphenated().to_string() == text).then_some(value)
    }
    let check = || -> Option<()> {
        let old_groups = rows(before, "groups")?;
        let old_members = rows(before, "group_memberships")?;
        let heads = rows(after, "group_authority_heads")?;
        let history = rows(after, "group_membership_revisions")?;
        let members = rows(after, "group_memberships")?;
        if heads.len() != old_groups.len()
            || history.len() != old_members.len()
            || members.len() != old_members.len()
        {
            return None;
        }
        let mut incarnations = BTreeSet::new();
        for group in &old_groups {
            let id = uuid(group, "id")?;
            let matching: Vec<_> = heads
                .iter()
                .filter(|h| uuid(h, "group_id") == Some(id))
                .collect();
            if matching.len() != 1 {
                return None;
            }
            let h = matching[0];
            if h.len() != 4
                || raw(h, "revision")? != "1"
                || raw(h, "state")? != r#""ACTIVE""#
                || !incarnations.insert(uuid(h, "incarnation")?)
            {
                return None;
            }
        }
        let mut membership_ids = BTreeSet::new();
        for old in &old_members {
            let group = uuid(old, "group_id")?;
            let org = uuid(old, "org_id")?;
            let matching: Vec<_> = members
                .iter()
                .filter(|m| uuid(m, "group_id") == Some(group) && uuid(m, "org_id") == Some(org))
                .collect();
            if matching.len() != 1 {
                return None;
            }
            let current = matching[0];
            let member = uuid(current, "membership_id")?;
            let incarnation = uuid(current, "incarnation")?;
            if !membership_ids.insert(member)
                || !incarnations.insert(incarnation)
                || raw(current, "current_revision")? != "1"
            {
                return None;
            }
            let matching: Vec<_> = history
                .iter()
                .filter(|h| uuid(h, "membership_id") == Some(member))
                .collect();
            if matching.len() != 1 {
                return None;
            }
            let h = matching[0];
            if h.len() != 13
                || uuid(h, "group_id") != Some(group)
                || uuid(h, "org_id") != Some(org)
                || uuid(h, "incarnation") != Some(incarnation)
                || raw(h, "revision")? != "1"
                || raw(h, "from_time")? != raw(old, "created_at")?
                || raw(h, "to_time")? != "null"
                || raw(h, "state")? != r#""ACTIVE""#
                || raw(h, "provenance_kind")? != r#""LEGACY_BACKFILL""#
            {
                return None;
            }
            for key in [
                "native_account_id",
                "legacy_actor_user_id",
                "command_id",
                "command_receipt",
            ] {
                if raw(h, key)? != "null" {
                    return None;
                }
            }
        }
        if rows(after, "company_enrollment_effect_bindings")?.len() != 0
            || rows(after, "platform_legacy_topology_receipts")?.len() != 0
        {
            return None;
        }
        Some(())
    };
    check().is_some()
}

pub(super) fn assert_topology_backfill(
    before: &BTreeMap<String, String>,
    after: &BTreeMap<String, String>,
) {
    assert!(
        topology_backfill_matches(before, after),
        "actual legacy topology backfill differs from exact finite contract"
    );
    // The genuine legacy owner prerequisite must have produced real topology.
    for table in ["groups", "group_memberships"] {
        assert_ne!(before[table], "[]", "empty migration prerequisite: {table}");
    }
    for (table, key, bad) in [
        ("group_authority_heads", "revision", json!(2)),
        ("group_authority_heads", "state", json!("RETIRED")),
        ("group_authority_heads", "incarnation", json!(Uuid::nil())),
        (
            "group_membership_revisions",
            "from_time",
            json!("2000-01-01T00:00:00+00:00"),
        ),
        (
            "group_membership_revisions",
            "provenance_kind",
            json!("COMPANY_ENROLLMENT_V1"),
        ),
        (
            "group_membership_revisions",
            "native_account_id",
            json!(Uuid::new_v4()),
        ),
        (
            "group_membership_revisions",
            "command_receipt",
            json!(Uuid::new_v4()),
        ),
        ("group_memberships", "current_revision", json!(2)),
        ("group_memberships", "membership_id", json!(Uuid::new_v4())),
    ] {
        let mut corrupt = after.clone();
        let mut rows: Vec<Value> = serde_json::from_str(&corrupt[table]).unwrap();
        assert_ne!(rows[0][key], bad, "ineffective backfill control");
        rows[0][key] = bad;
        corrupt.insert(table.into(), serde_json::to_string(&rows).unwrap());
        assert!(
            !topology_backfill_matches(before, &corrupt),
            "backfill oracle accepted {table}.{key}"
        );
    }
    for table in [
        "group_authority_heads",
        "group_membership_revisions",
        "group_memberships",
    ] {
        let mut corrupt = after.clone();
        corrupt.insert(table.into(), "[]".into());
        assert!(
            !topology_backfill_matches(before, &corrupt),
            "backfill oracle accepted omitted {table}"
        );
    }
}

// Pure evidence-oracle fixture only: no database writes or alternate oracle.
fn current_schema_topology_backfill_fixture() -> (BTreeMap<String, String>, BTreeMap<String, String>)
{
    let group = "10000000-0000-4000-8000-000000000001";
    let org = "10000000-0000-4000-8000-000000000002";
    let membership = "10000000-0000-4000-8000-000000000003";
    let head_incarnation = "10000000-0000-4000-8000-000000000004";
    let incarnation = "10000000-0000-4000-8000-000000000005";
    let created_at = "2026-09-19T09:08:07.123456+09:00";
    let groups = json!([{"id": group}]);
    let old_members = json!([{"group_id": group, "org_id": org, "created_at": created_at}]);
    let heads = json!([{"group_id": group, "revision": 1,
        "incarnation": head_incarnation, "state": "ACTIVE"}]);
    let members = json!([{"group_id": group, "org_id": org, "created_at": created_at,
        "membership_id": membership, "current_revision": 1, "incarnation": incarnation}]);
    let history = json!([{"group_id": group, "org_id": org, "membership_id": membership,
        "revision": 1, "incarnation": incarnation, "from_time": created_at, "to_time": null,
        "state": "ACTIVE", "provenance_kind": "LEGACY_BACKFILL", "native_account_id": null,
        "legacy_actor_user_id": null, "force_actor_user_id": null,
        "command_id": null, "command_receipt": null}]);
    for rows in [&groups, &old_members, &heads, &members, &history] {
        assert_eq!(rows.as_array().unwrap().len(), 1);
    }
    assert_eq!(history[0].as_object().unwrap().len(), 14);
    assert_eq!(heads[0]["group_id"], groups[0]["id"]);
    assert_eq!(history[0]["group_id"], old_members[0]["group_id"]);
    assert_eq!(history[0]["org_id"], old_members[0]["org_id"]);
    assert_eq!(history[0]["membership_id"], members[0]["membership_id"]);
    assert_eq!(history[0]["incarnation"], members[0]["incarnation"]);
    assert_eq!(history[0]["from_time"], old_members[0]["created_at"]);
    assert_eq!(history[0]["revision"], json!(1));
    assert_eq!(history[0]["from_time"], json!(created_at));
    assert!(history[0]["force_actor_user_id"].is_null());
    let before = BTreeMap::from([
        ("groups".into(), groups.to_string()),
        ("group_memberships".into(), old_members.to_string()),
    ]);
    let after = BTreeMap::from([
        ("group_authority_heads".into(), heads.to_string()),
        ("group_memberships".into(), members.to_string()),
        ("group_membership_revisions".into(), history.to_string()),
        ("company_enrollment_effect_bindings".into(), "[]".into()),
        ("platform_legacy_topology_receipts".into(), "[]".into()),
    ]);
    assert_eq!(after["company_enrollment_effect_bindings"], "[]");
    assert_eq!(after["platform_legacy_topology_receipts"], "[]");
    (before, after)
}

#[test]
fn topology_oracle_accepts_current_fourteen_field_legacy_backfill() {
    let (before, after) = current_schema_topology_backfill_fixture();
    assert!(
        topology_backfill_matches(&before, &after),
        "current full-row LEGACY_BACKFILL includes a null force_actor_user_id"
    );
}

#[test]
fn topology_oracle_rejects_omitted_force_actor_field() {
    let (before, mut after) = current_schema_topology_backfill_fixture();
    let mut history: Vec<Value> =
        serde_json::from_str(&after["group_membership_revisions"]).unwrap();
    assert_eq!(
        history[0]
            .as_object_mut()
            .unwrap()
            .remove("force_actor_user_id"),
        Some(Value::Null)
    );
    assert_eq!(history[0].as_object().unwrap().len(), 13);
    after.insert(
        "group_membership_revisions".into(),
        serde_json::to_string(&history).unwrap(),
    );
    assert!(
        !topology_backfill_matches(&before, &after),
        "omitted current-schema column must not qualify as a complete census"
    );
}

#[test]
fn topology_oracle_rejects_nonnull_force_actor_for_legacy_backfill() {
    let (before, mut after) = current_schema_topology_backfill_fixture();
    let mut history: Vec<Value> =
        serde_json::from_str(&after["group_membership_revisions"]).unwrap();
    assert!(history[0]["force_actor_user_id"].is_null());
    history[0]["force_actor_user_id"] = json!("10000000-0000-4000-8000-000000000006");
    assert_eq!(history[0].as_object().unwrap().len(), 14);
    after.insert(
        "group_membership_revisions".into(),
        serde_json::to_string(&history).unwrap(),
    );
    assert!(
        !topology_backfill_matches(&before, &after),
        "LEGACY_BACKFILL cannot claim a force-removal actor"
    );
}
