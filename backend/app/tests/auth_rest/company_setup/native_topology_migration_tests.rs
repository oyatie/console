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
