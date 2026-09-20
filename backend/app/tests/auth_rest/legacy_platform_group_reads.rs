//! Initial real mounted Group owner RED only; no future API dependency.
use super::*;

const GROUP_PATH: &str = "/api/platform/groups";
const GROUP_KEYS: &[&str] = &[
    "id",
    "slug",
    "name",
    "status",
    "created_at",
    "updated_at",
    "member_count",
    "members",
];
const MEMBER_KEYS: &[&str] = &["id", "slug", "name", "status"];
fn exact_group_keys(value: &Value, keys: &[&str]) -> bool {
    value.as_object().is_some_and(|row| {
        row.keys().map(String::as_str).collect::<BTreeSet<_>>() == keys.iter().copied().collect()
    })
}
fn group_matches(actual: &Value, expected: &Value) -> bool {
    actual == expected
        && actual.as_array().is_some_and(|groups| {
            !groups.is_empty()
                && groups.iter().all(|group| {
                    exact_group_keys(group, GROUP_KEYS)
                        && group["id"]
                            .as_str()
                            .is_some_and(|id| Uuid::parse_str(id).is_ok_and(|id| !id.is_nil()))
                        && group["members"].as_array().is_some_and(|members| {
                            group["member_count"].as_u64() == Some(members.len() as u64)
                                && members.iter().all(|member| {
                                    exact_group_keys(member, MEMBER_KEYS)
                                        && member["id"].as_str().is_some_and(|id| {
                                            Uuid::parse_str(id).is_ok_and(|id| {
                                                !id.is_nil() && id != *OrgId::platform().as_uuid()
                                            })
                                        })
                                })
                        })
                })
        })
}
fn wire_time(at: OffsetDateTime) -> String {
    at.format(&time::format_description::well_known::Rfc3339)
        .unwrap()
}

// Read independent base rows, never platform_list_groups, group_from_row or owner output.
async fn expected_groups(pool: &PgPool) -> Value {
    let sentinel: Uuid =
        sqlx::query_scalar("SELECT group_id FROM public.organizations WHERE id=$1")
            .bind(OrgId::platform().as_uuid())
            .fetch_one(pool)
            .await
            .unwrap();
    let groups = sqlx::query("SELECT id,slug,name,status,created_at,updated_at FROM public.groups")
        .fetch_all(pool)
        .await
        .unwrap();
    let companies =
        sqlx::query("SELECT id,slug,name,status,group_id,created_at FROM public.organizations")
            .fetch_all(pool)
            .await
            .unwrap();
    let mut groups: Vec<_> = groups
        .iter()
        .filter(|row| row.get::<Uuid, _>("id") != sentinel)
        .collect();
    groups.sort_by_key(|row| {
        (
            row.get::<OffsetDateTime, _>("created_at"),
            row.get::<Uuid, _>("id"),
        )
    });
    let mut result = Vec::new();
    for group in groups {
        let id: Uuid = group.get("id");
        let mut members: Vec<_> = companies
            .iter()
            .filter(|row| {
                row.get::<Uuid, _>("id") != *OrgId::platform().as_uuid()
                    && row.get::<Uuid, _>("group_id") == id
            })
            .collect();
        members.sort_by_key(|row| {
            (
                row.get::<OffsetDateTime, _>("created_at"),
                row.get::<Uuid, _>("id"),
            )
        });
        let members:Vec<_>=members.into_iter().map(|row| json!({"id":row.get::<Uuid,_>("id"),"slug":row.get::<String,_>("slug"),"name":row.get::<String,_>("name"),"status":row.get::<String,_>("status")})).collect();
        result.push(json!({"id":id,"slug":group.get::<String,_>("slug"),"name":group.get::<String,_>("name"),"status":group.get::<String,_>("status"),"created_at":wire_time(group.get("created_at")),"updated_at":wire_time(group.get("updated_at")),"member_count":members.len(),"members":members}));
    }
    json!(result)
}

async fn seed_group_content(pool: &PgPool) -> (Vec<Uuid>, Vec<Uuid>, OffsetDateTime) {
    let mut companies: Vec<Uuid> =
        sqlx::query_scalar("SELECT id FROM public.organizations WHERE id<>$1 ORDER BY id")
            .bind(OrgId::platform().as_uuid())
            .fetch_all(pool)
            .await
            .unwrap();
    assert_eq!(companies.len(), 2, "genuine parent two Company fixture");
    for index in 0..2 {
        let id:Uuid=sqlx::query_scalar("INSERT INTO public.organizations(slug,name,status) VALUES($1,$2,'ACTIVE') RETURNING id")
            .bind(format!("group-topology-company-{index}"))
            .bind(format!("대한민국 그룹 회사 이름 {index} {}", "긴 조직 이름 ".repeat(8)))
            .fetch_one(pool).await.unwrap();
        companies.push(id);
    }
    let mut groups = Vec::new();
    for company in &companies {
        groups.push(
            sqlx::query_scalar::<_, Uuid>("SELECT group_id FROM public.organizations WHERE id=$1")
                .bind(company)
                .fetch_one(pool)
                .await
                .unwrap(),
        );
    }
    assert_eq!(
        groups.iter().collect::<BTreeSet<_>>().len(),
        4,
        "four genuine Group-of-one roots"
    );
    let moved: Uuid = sqlx::query_scalar("SELECT platform_assign_org_to_group($1,$2)")
        .bind(groups[0])
        .bind(companies[1])
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(moved, companies[1], "real owning topology fixture mutation");
    let at = db_now(pool).await - Duration::days(3);
    // Equal creation times require stable UUID tie-breaks for both levels.
    for (index, status) in ["ACTIVE", "SUSPENDED", "ARCHIVED", "ACTIVE"]
        .into_iter()
        .enumerate()
    {
        sqlx::query("UPDATE public.organizations SET status=$1,created_at=$2 WHERE id=$3")
            .bind(status)
            .bind(at)
            .bind(companies[index])
            .execute(pool)
            .await
            .unwrap();
        sqlx::query("UPDATE public.groups SET status=$1,created_at=$2,name=$3 WHERE id=$4")
            .bind(status)
            .bind(at)
            .bind(format!("그룹 {index} 본사 조직 {}", "긴 이름 ".repeat(10)))
            .bind(groups[index])
            .execute(pool)
            .await
            .unwrap();
    }
    let memberships: Vec<(Uuid, Uuid)> =
        sqlx::query_as("SELECT org_id,group_id FROM public.group_memberships WHERE org_id=ANY($1)")
            .bind(&companies)
            .fetch_all(pool)
            .await
            .unwrap();
    assert_eq!(
        memberships.len(),
        4,
        "each Company retains exactly one real Group membership"
    );
    for (index, company) in companies.iter().enumerate() {
        let group = if index == 1 { groups[0] } else { groups[index] };
        assert!(
            memberships.contains(&(*company, group)),
            "identity and membership fixture agree"
        );
    }
    (companies, groups, at)
}

#[sqlx::test(migrations = false)]
async fn mounted_group_list_current_member_denies_and_recovers(pool: PgPool) {
    let f = fixture(&pool).await;
    let (companies, groups, at) = seed_group_content(&pool).await;
    let expected = expected_groups(&pool).await;
    assert!(
        group_matches(&expected, &expected),
        "independent exact8/4 projection prerequisite"
    );
    assert_eq!(
        expected.as_array().unwrap().len(),
        4,
        "four Groups include an empty Group"
    );
    for (index, id) in groups.iter().enumerate() {
        let row = expected
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["id"] == json!(id))
            .unwrap();
        let mut members = match index {
            0 => vec![companies[0], companies[1]],
            1 => vec![],
            _ => vec![companies[index]],
        };
        members.sort();
        assert_eq!(row["member_count"], json!(members.len()));
        let actual: Vec<Uuid> = row["members"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| Uuid::parse_str(row["id"].as_str().unwrap()).unwrap())
            .collect();
        assert_eq!(actual, members, "known membership and tied UUID ordering");
        assert_eq!(row["created_at"], json!(wire_time(at)));
        assert_eq!(
            row["status"],
            json!(["ACTIVE", "SUSPENDED", "ARCHIVED", "ACTIVE"][index])
        );
    }
    let mut sorted = groups.clone();
    sorted.sort();
    let ids: Vec<Uuid> = expected
        .as_array()
        .unwrap()
        .iter()
        .map(|row| Uuid::parse_str(row["id"].as_str().unwrap()).unwrap())
        .collect();
    assert_eq!(ids, sorted, "Group tied creation ordering");
    let count = expected.as_array().unwrap().len();
    let before = all_rows(&pool).await;
    let start = db_now(&pool).await;
    let (positive_status, positive) = observe_groups(&f).await;
    let end = db_now(&pool).await;
    let after = all_rows(&pool).await;
    assert!(
        positive_status == StatusCode::OK && group_matches(&positive, &expected),
        "actual owner positive projection prerequisite"
    );
    set_role(&pool, f.actor, "MEMBER").await;
    let denied_before = all_rows(&pool).await;
    let (denied_status, denied) = observe_groups(&f).await;
    let denied_after = all_rows(&pool).await;
    set_role(&pool, f.actor, "SUPER_ADMIN").await;
    let recovery_before = all_rows(&pool).await;
    let recovery_start = db_now(&pool).await;
    let (recovery_status, recovery) = observe_groups(&f).await;
    let recovery_end = db_now(&pool).await;
    let recovery_after = all_rows(&pool).await;
    f.state.shutdown_realtime().await;
    f.business.close().await;
    assert!(
        recovery_status == StatusCode::OK && group_matches(&recovery, &expected),
        "same original bearer exact Group recovery prerequisite"
    );
    // Preserve strict real DB audit windows below; old host-clock behavior cannot mask this owner RED.
    assert!(
        denied_status == StatusCode::FORBIDDEN,
        "PLATFORM_GROUP_LIST_CURRENT_ROLE_REQUIRED: current MEMBER disclosed cross-Company Group topology"
    );
    assert_eq!(
        denied,
        json!({"error":{"code":"forbidden","message":"platform principal cannot list groups"}})
    );
    assert!(
        denied_before == denied_after,
        "refused Group read has no audit or other durable effect"
    );
    assert!(
        group_read_delta(&before, &after, f.actor, count, start, end),
        "exact positive Group audit/allstate"
    );
    assert!(
        group_read_delta(
            &recovery_before,
            &recovery_after,
            f.actor,
            count,
            recovery_start,
            recovery_end
        ),
        "exact recovery Group audit/allstate"
    );
}

async fn observe_groups(f: &Fixture) -> (StatusCode, Value) {
    let response = f
        .router
        .clone()
        .oneshot(
            Request::builder()
                .uri(GROUP_PATH)
                .header(header::AUTHORIZATION, format!("Bearer {}", f.access))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let (status, headers, bytes) = response_parts(response).await;
    assert!(
        headers
            .get(header::CONTENT_TYPE)
            .is_some_and(|v| v == "application/json"),
        "actual JSON Group prerequisite"
    );
    for secret in &f.secrets {
        assert!(
            !secret.is_empty() && !bytes.windows(secret.len()).any(|w| w == secret.as_bytes()),
            "no reflected credential body"
        );
        assert!(
            headers.iter().all(|(_, v)| !v
                .as_bytes()
                .windows(secret.len())
                .any(|w| w == secret.as_bytes())),
            "no reflected credential header"
        );
    }
    (status, serde_json::from_slice(&bytes).unwrap())
}

fn group_audit_matches(
    row: &Value,
    actor: UserId,
    count: usize,
    start: OffsetDateTime,
    end: OffsetDateTime,
) -> bool {
    if row["action"] != "platform.group.list"
        || row["target_type"] != "groups"
        || row["target_id"] != "list"
    {
        return false;
    }
    // Check Group-specific original fields first; reuse the unchanged strict20-field
    // list audit validator only for the identical remaining schema and time bounds.
    let mut normalized = row.clone();
    normalized["action"] = json!("platform.tenant.list");
    normalized["target_type"] = json!("organizations");
    audit_matches(&normalized, actor, count, start, end)
}
fn group_read_delta(
    before: &Rows,
    after: &Rows,
    actor: UserId,
    count: usize,
    start: OffsetDateTime,
    end: OffsetDateTime,
) -> bool {
    if before.is_empty()
        || !before.keys().eq(after.keys())
        || before
            .iter()
            .any(|(table, rows)| table != "audit_events" && after.get(table) != Some(rows))
    {
        return false;
    }
    let (Some(old), Some(new)) = (before.get("audit_events"), after.get("audit_events")) else {
        return false;
    };
    let (Ok(old), Ok(new)) = (
        serde_json::from_str::<Vec<&RawValue>>(old),
        serde_json::from_str::<Vec<&RawValue>>(new),
    ) else {
        return false;
    };
    if new.len() != old.len() + 1 {
        return false;
    }
    let old_bytes: BTreeSet<_> = old.iter().map(|row| row.get()).collect();
    let new_bytes: BTreeSet<_> = new.iter().map(|row| row.get()).collect();
    if old_bytes.len() != old.len()
        || new_bytes.len() != new.len()
        || !old_bytes.is_subset(&new_bytes)
    {
        return false;
    }
    let added: Vec<_> = new_bytes.difference(&old_bytes).collect();
    if added.len() != 1 {
        return false;
    }
    let Ok(row) = serde_json::from_str::<Value>(added[0]) else {
        return false;
    };
    group_audit_matches(&row, actor, count, start, end)
        && old.iter().all(|prior| {
            serde_json::from_str::<Value>(prior.get())
                .is_ok_and(|prior| prior["id"] != row["id"] && prior["trace_id"] != row["trace_id"])
        })
}

#[test]
fn group_audit_allstate_oracle_rejects_omission_corruption_and_rewritten_history() {
    let actor = UserId::new();
    let at = OffsetDateTime::parse(
        "2026-09-20T00:00:00Z",
        &time::format_description::well_known::Rfc3339,
    )
    .unwrap();
    let start = at - Duration::seconds(1);
    let end = at + Duration::seconds(1);
    let mut prior = audit_fixture(actor, at - Duration::days(1));
    prior["action"] = json!("platform.group.list");
    prior["target_type"] = json!("groups");
    let mut added = audit_fixture(actor, at);
    added["action"] = json!("platform.group.list");
    added["target_type"] = json!("groups");
    added["trace_id"] = json!("fedcba0987654321fedcba0987654321");
    let before = Rows::from([
        ("audit_events".into(), json!([prior.clone()]).to_string()),
        ("users".into(), "[{\"retained\":true}]".into()),
    ]);
    let delta = |event: &Value| {
        let mut result = before.clone();
        result.insert(
            "audit_events".into(),
            json!([prior.clone(), event]).to_string(),
        );
        result
    };
    let after = delta(&added);
    assert!(group_read_delta(&before, &after, actor, 2, start, end));
    assert!(!group_read_delta(&before, &before, actor, 2, start, end));
    for key in AUDIT_KEYS {
        let mut missing = added.clone();
        missing.as_object_mut().unwrap().remove(*key);
        assert!(
            !group_read_delta(&before, &delta(&missing), actor, 2, start, end),
            "missing audit field"
        );
        let mut corrupt = added.clone();
        corrupt[*key] = json!({"corrupt":true});
        assert!(
            !group_read_delta(&before, &delta(&corrupt), actor, 2, start, end),
            "corrupted audit field"
        );
    }
    for key in ["occurred_at", "created_at"] {
        let mut corrupt = added.clone();
        corrupt[key] = json!(wire_time(start - Duration::nanoseconds(1)));
        assert!(
            !group_read_delta(&before, &delta(&corrupt), actor, 2, start, end),
            "early timestamp"
        );
        corrupt[key] = json!(wire_time(end + Duration::nanoseconds(1)));
        assert!(
            !group_read_delta(&before, &delta(&corrupt), actor, 2, start, end),
            "late timestamp"
        );
    }
    for key in ["id", "trace_id"] {
        let mut corrupt = added.clone();
        corrupt[key] = prior[key].clone();
        assert!(
            !group_read_delta(&before, &delta(&corrupt), actor, 2, start, end),
            "reused audit identity"
        );
    }
    for (key, value) in [
        ("actor", json!(UserId::new())),
        ("after_snap", json!({"count":1})),
        ("org_id", json!(Uuid::new_v4())),
        ("action", json!("platform.tenant.list")),
        ("target_type", json!("users")),
        ("target_id", json!("health")),
        ("extra", json!(true)),
    ] {
        let mut corrupt = added.clone();
        corrupt[key] = value;
        assert!(
            !group_read_delta(&before, &delta(&corrupt), actor, 2, start, end),
            "plausible wrong audit field"
        );
    }
    let mut corruptions = Vec::new();
    let mut x = after.clone();
    x.remove("users");
    corruptions.push(x);
    let mut x = after.clone();
    x.insert("extra".into(), "[]".into());
    corruptions.push(x);
    let mut x = after.clone();
    x.insert("users".into(), "[]".into());
    corruptions.push(x);
    let mut x = after.clone();
    x.insert("audit_events".into(), json!([added.clone()]).to_string());
    corruptions.push(x);
    let mut x = after.clone();
    x.insert(
        "audit_events".into(),
        json!([prior.clone(), added.clone(), added.clone()]).to_string(),
    );
    corruptions.push(x);
    let mut x = after.clone();
    x.insert(
        "audit_events".into(),
        format!(
            "[{},{}]",
            serde_json::to_string_pretty(&prior).unwrap(),
            added
        ),
    );
    corruptions.push(x);
    let mut rewritten = prior.clone();
    rewritten["action"] = json!("rewritten");
    let mut x = after.clone();
    x.insert("audit_events".into(), json!([rewritten, added]).to_string());
    corruptions.push(x);
    for changed in corruptions {
        assert!(
            !group_read_delta(&before, &changed, actor, 2, start, end),
            "history/effect corruption escaped"
        );
    }
}

fn group_projection_fixture() -> Value {
    let member = |id: u128, status: &str| json!({"id":Uuid::from_u128(id),"slug":format!("company-{id}"),"name":format!("회사 {id} 긴 이름"),"status":status});
    let group = |id: u128, status: &str, members: Vec<Value>| json!({"id":Uuid::from_u128(id),"slug":format!("group-{id}"),"name":format!("그룹 {id} 긴 이름"),"status":status,"created_at":"2026-09-17T00:00:00Z","updated_at":"2026-09-18T00:00:00Z","member_count":members.len(),"members":members});
    json!([
        group(
            101,
            "ACTIVE",
            vec![member(201, "ACTIVE"), member(202, "SUSPENDED")]
        ),
        group(102, "SUSPENDED", vec![]),
        group(103, "ARCHIVED", vec![member(203, "ARCHIVED")])
    ])
}

#[test]
fn group_projection_oracle_rejects_every_field_topology_and_order_corruption() {
    let expected = group_projection_fixture();
    assert!(group_matches(&expected, &expected));
    for index in 0..expected.as_array().unwrap().len() {
        for key in GROUP_KEYS {
            let mut bad = expected.clone();
            bad[index].as_object_mut().unwrap().remove(*key);
            assert!(!group_matches(&bad, &expected), "missing Group field");
            let mut bad = expected.clone();
            bad[index][*key] = json!({"corrupt":true});
            assert!(!group_matches(&bad, &expected), "corrupt Group field");
        }
        for member in 0..expected[index]["members"].as_array().unwrap().len() {
            for key in MEMBER_KEYS {
                let mut bad = expected.clone();
                bad[index]["members"][member]
                    .as_object_mut()
                    .unwrap()
                    .remove(*key);
                assert!(!group_matches(&bad, &expected), "missing member field");
                let mut bad = expected.clone();
                bad[index]["members"][member][*key] = json!({"corrupt":true});
                assert!(!group_matches(&bad, &expected), "corrupt member field");
            }
        }
    }
    for fault in [
        "missing_group",
        "missing_empty",
        "extra_group",
        "duplicate_group",
        "group_order",
        "member_order",
        "missing_member",
        "duplicate_member",
        "extra_member",
        "wrong_count",
        "member_other_group",
        "sentinel_company",
        "sentinel_group",
        "group_extra_field",
        "member_extra_field",
        "wrapped_array",
        "malformed_members",
        "bad_member_uuid",
    ] {
        let mut bad = expected.clone();
        match fault {
            "missing_group" => {
                bad.as_array_mut().unwrap().pop();
            }
            "missing_empty" => {
                bad.as_array_mut().unwrap().remove(1);
            }
            "extra_group" => bad.as_array_mut().unwrap().push(expected[0].clone()),
            "duplicate_group" => bad[1] = bad[0].clone(),
            "group_order" => bad.as_array_mut().unwrap().reverse(),
            "member_order" => bad[0]["members"].as_array_mut().unwrap().reverse(),
            "missing_member" => {
                bad[0]["members"].as_array_mut().unwrap().pop();
            }
            "duplicate_member" => bad[0]["members"][1] = bad[0]["members"][0].clone(),
            "extra_member" => bad[1]["members"]
                .as_array_mut()
                .unwrap()
                .push(expected[0]["members"][0].clone()),
            "wrong_count" => bad[0]["member_count"] = json!(1),
            "member_other_group" => bad[0]["members"][0] = bad[2]["members"][0].clone(),
            "sentinel_company" => bad[0]["members"][0]["id"] = json!(OrgId::platform()),
            "sentinel_group" => {
                let mut sentinel = expected[1].clone();
                sentinel["id"] = json!(Uuid::from_u128(999));
                sentinel["slug"] = json!("platform-sentinel-group");
                bad.as_array_mut().unwrap().push(sentinel);
            }
            "group_extra_field" => bad[0]["private_salary"] = json!(1),
            "member_extra_field" => bad[0]["members"][0]["phone"] = json!("private"),
            "wrapped_array" => bad = json!({"groups":bad}),
            "malformed_members" => bad[0]["members"] = json!({"id":"not-array"}),
            "bad_member_uuid" => bad[0]["members"][0]["id"] = json!("not-a-uuid"),
            _ => unreachable!(),
        }
        assert!(
            !group_matches(&bad, &expected),
            "Group projection corruption: {fault}"
        );
    }
}

#[path = "legacy_platform_group_credentials.rs"]
mod legacy_platform_group_credentials;
