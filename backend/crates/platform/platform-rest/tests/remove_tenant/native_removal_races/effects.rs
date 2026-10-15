//! Complete public-row effects for the two owning topology operations.

use super::super::*;
use super::support::Fixture;
use std::collections::BTreeMap;

pub(super) type Census = BTreeMap<String, Vec<Value>>;

pub(super) async fn census(owner: &PgPool) -> Census {
    let tables: Vec<String> = sqlx::query_scalar("SELECT c.relname::text FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='public' AND c.relkind IN('r','p') ORDER BY c.relname")
        .fetch_all(owner).await.unwrap();
    let mut result = BTreeMap::new();
    for table in tables {
        let sql = sqlx::AssertSqlSafe(format!(
            "SELECT coalesce(jsonb_agg(value),'[]'::jsonb) FROM (SELECT to_jsonb(t) AS value FROM public.\"{}\" t) captured",
            table.replace('"', "\"\"")
        ));
        let rows: Value = sqlx::query_scalar(sql).fetch_one(owner).await.unwrap();
        let mut rows = rows.as_array().unwrap().clone();
        rows.sort_by_key(Value::to_string);
        result.insert(table, rows);
    }
    result
}

pub(super) enum Transition {
    Reactivate,
    Assign,
}

fn added_audit(before: &[Value], after: &[Value]) -> Value {
    assert_eq!(
        after.len(),
        before.len() + 1,
        "exactly one owning operation must be audited"
    );
    assert!(
        before.iter().all(|row| after.contains(row)),
        "acknowledged audit history changed"
    );
    let added: Vec<_> = after.iter().filter(|row| !before.contains(row)).collect();
    assert_eq!(added.len(), 1);
    added[0].clone()
}

pub(super) async fn assert_transition(f: &Fixture, before: &Census, transition: Transition) {
    let after = census(&f.owner).await;
    assert!(
        before.keys().eq(after.keys()),
        "complete table census changed"
    );
    let mut expected = before.clone();
    let actual = after["organizations"]
        .iter()
        .find(|row| row["id"] == f.target.to_string())
        .unwrap();
    let row = expected
        .get_mut("organizations")
        .unwrap()
        .iter_mut()
        .find(|row| row["id"] == f.target.to_string())
        .unwrap();
    row["updated_at"] = actual["updated_at"].clone();
    let (action, target_type, target_id, snapshot) = match transition {
        Transition::Reactivate => {
            row["status"] = Value::String("ACTIVE".into());
            (
                "platform.tenant.status",
                "organizations",
                f.target.to_string(),
                serde_json::json!({"status":"ACTIVE"}),
            )
        }
        Transition::Assign => {
            row["group_id"] = serde_json::json!(f.destination);
            let actual = after["group_memberships"]
                .iter()
                .find(|row| row["org_id"] == f.target.to_string())
                .unwrap();
            let membership = expected
                .get_mut("group_memberships")
                .unwrap()
                .iter_mut()
                .find(|row| row["org_id"] == f.target.to_string())
                .unwrap();
            membership["group_id"] = serde_json::json!(f.destination);
            membership["created_at"] = actual["created_at"].clone();
            (
                "platform.group.assign_org",
                "group_memberships",
                format!("{}:{}", f.destination, f.target),
                serde_json::json!({"group_id":f.destination,"org_id":f.target}),
            )
        }
    };
    let audit = added_audit(&before["audit_events"], &after["audit_events"]);
    assert_eq!(audit["action"], action);
    assert_eq!(audit["target_type"], target_type);
    assert_eq!(audit["target_id"], target_id);
    assert_eq!(audit["org_id"], f.target.to_string());
    assert_eq!(audit["actor"], f.actor.as_uuid().to_string());
    assert!(audit["before_snap"].is_null() && audit["branch_id"].is_null());
    assert_eq!(audit["after_snap"], snapshot);
    expected.get_mut("audit_events").unwrap().push(audit);
    assert_census(expected, &after);
}

fn assert_census(mut expected: Census, after: &Census) {
    for rows in expected.values_mut() {
        rows.sort_by_key(Value::to_string);
    }
    assert!(
        &expected == after,
        "complete target/foreign/Account/credential/audit effects differ"
    );
}

pub(super) async fn assert_removed(f: &Fixture, before: &Census, force: bool) {
    let after = census(&f.owner).await;
    assert!(!org_exists(&f.owner, f.target).await);
    assert!(org_exists(&f.owner, f.foreign).await);
    let mut expected = before.clone();
    for (table, rows) in &mut expected {
        if table == "audit_events" {
            for row in rows {
                if row["org_id"] == f.target.to_string() {
                    row["org_id"] = serde_json::json!(OrgId::platform().as_uuid());
                    row["actor"] = Value::Null;
                    row["branch_id"] = Value::Null;
                }
            }
        } else {
            rows.retain(|row| {
                row["org_id"] != f.target.to_string()
                    && !(table == "organizations" && row["id"] == f.target.to_string())
            });
        }
    }
    let audit = added_audit(&expected["audit_events"], &after["audit_events"]);
    assert_eq!(
        audit["action"],
        if force {
            "platform.tenant.force_remove"
        } else {
            "platform.tenant.remove"
        }
    );
    assert_eq!(audit["target_type"], "organizations");
    assert_eq!(audit["target_id"], f.target.to_string());
    assert_eq!(audit["actor"], f.actor.as_uuid().to_string());
    assert!(audit["org_id"].is_null() && audit["branch_id"].is_null());
    let original = before["organizations"]
        .iter()
        .find(|row| row["id"] == f.target.to_string())
        .unwrap();
    if force {
        let mut wiped = serde_json::Map::new();
        for table in [
            "users",
            "registry_customers",
            "registry_sites",
            "registry_equipment",
            "work_orders",
            "financial_rental_quotes",
            "financial_purchase_requests",
            "messenger_threads",
            "evidence_media",
            "audit_events",
        ] {
            let count = before[table]
                .iter()
                .filter(|row| row["org_id"] == f.target.to_string())
                .count();
            wiped.insert(table.into(), serde_json::json!(count));
        }
        assert_eq!(
            audit["before_snap"],
            serde_json::json!({"org_id":f.target,"slug":original["slug"],"name":original["name"],"wiped":wiped})
        );
        assert!(audit["after_snap"].is_null());
    } else {
        assert!(audit["before_snap"].is_null());
        assert_eq!(
            audit["after_snap"],
            serde_json::json!({"org_id":f.target,"slug":original["slug"]})
        );
    }
    expected.get_mut("audit_events").unwrap().push(audit);
    assert_census(expected, &after);
}
