//! Initial health owner RED; parent genuine login fixture and all original list tests retained.
use super::*;

const OPS_PATH: &str = "/api/platform/ops";
const HEALTH_KEYS: &[&str] = &[
    "id",
    "slug",
    "name",
    "status",
    "group_id",
    "group_slug",
    "group_name",
    "user_count",
    "active_user_count",
    "active_work_orders",
    "open_work_orders",
    "last_activity_at",
    "route_adoption",
    "zero_legacy_release_cycles",
];
const ADOPTION_KEYS: &[&str] = &[
    "release_cycle",
    "console_route_events",
    "legacy_route_events",
    "rum_error_events",
    "rum_perf_p95_ms",
    "last_event_at",
];

fn exact_keys(value: &Value, keys: &[&str]) -> bool {
    value.as_object().is_some_and(|row| {
        row.keys().map(String::as_str).collect::<BTreeSet<_>>() == keys.iter().copied().collect()
    })
}
fn health_matches(actual: &Value, expected: &Value) -> bool {
    exact_keys(actual, &["tenants"])
        && actual == expected
        && actual["tenants"].as_array().is_some_and(|rows| {
            rows.len() >= 2
                && rows.iter().all(|row| {
                    exact_keys(row, HEALTH_KEYS)
                        && row["id"].as_str().is_some_and(|id| {
                            Uuid::parse_str(id)
                                .is_ok_and(|id| !id.is_nil() && id != *OrgId::platform().as_uuid())
                        })
                        && row["route_adoption"].as_array().is_some_and(|cycles| {
                            cycles.iter().all(|cycle| exact_keys(cycle, ADOPTION_KEYS))
                        })
                })
        })
}
fn wire_time(at: OffsetDateTime) -> String {
    at.format(&time::format_description::well_known::Rfc3339)
        .unwrap()
}

// Derive expected values from persisted base rows, never either challenged SQL function,
// a production projection struct, or a response from the owner under test.
async fn expected_health(pool: &PgPool) -> Value {
    let companies = sqlx::query("SELECT o.id,o.slug,o.name,o.status,o.group_id,g.slug AS group_slug,g.name AS group_name FROM public.organizations o LEFT JOIN public.groups g ON g.id=o.group_id WHERE o.id<>$1 ORDER BY o.created_at,o.id")
        .bind(OrgId::platform().as_uuid()).fetch_all(pool).await.unwrap();
    let users = sqlx::query("SELECT org_id,is_active,created_at FROM public.users")
        .fetch_all(pool)
        .await
        .unwrap();
    let work = sqlx::query("SELECT org_id,status,updated_at FROM public.work_orders")
        .fetch_all(pool)
        .await
        .unwrap();
    let audits =
        sqlx::query("SELECT org_id,occurred_at FROM public.audit_events WHERE org_id IS NOT NULL")
            .fetch_all(pool)
            .await
            .unwrap();
    let telemetry = sqlx::query("SELECT org_id,release_cycle,event_kind,route_surface,duration_ms,occurred_at FROM public.console_route_telemetry").fetch_all(pool).await.unwrap();
    let mut result = Vec::new();
    for company in companies {
        let id: Uuid = company.get("id");
        let people: Vec<_> = users
            .iter()
            .filter(|row| row.get::<Uuid, _>("org_id") == id)
            .collect();
        let orders: Vec<_> = work
            .iter()
            .filter(|row| row.get::<Uuid, _>("org_id") == id)
            .collect();
        let last_activity = people
            .iter()
            .map(|row| row.get::<OffsetDateTime, _>("created_at"))
            .chain(
                orders
                    .iter()
                    .map(|row| row.get::<OffsetDateTime, _>("updated_at")),
            )
            .chain(
                audits
                    .iter()
                    .filter(|row| row.get::<Uuid, _>("org_id") == id)
                    .map(|row| row.get::<OffsetDateTime, _>("occurred_at")),
            )
            .max();
        let mut cycles: BTreeMap<String, Vec<&sqlx::postgres::PgRow>> = BTreeMap::new();
        for row in &telemetry {
            if row.get::<Uuid, _>("org_id") == id {
                cycles
                    .entry(row.get("release_cycle"))
                    .or_default()
                    .push(row);
            }
        }
        let mut adoption = Vec::new();
        for (cycle, events) in cycles {
            let console = events
                .iter()
                .filter(|row| {
                    row.get::<String, _>("event_kind") == "route_selection"
                        && row.get::<String, _>("route_surface") == "console"
                })
                .count();
            let legacy = events
                .iter()
                .filter(|row| {
                    row.get::<String, _>("event_kind") == "route_selection"
                        && row.get::<String, _>("route_surface") == "legacy"
                })
                .count();
            let errors = events
                .iter()
                .filter(|row| row.get::<String, _>("event_kind") == "rum_error")
                .count();
            let mut durations: Vec<i32> = events
                .iter()
                .filter(|row| {
                    matches!(
                        row.get::<String, _>("event_kind").as_str(),
                        "route_selection" | "rum_perf"
                    )
                })
                .filter_map(|row| row.get::<Option<i32>, _>("duration_ms"))
                .collect();
            durations.sort_unstable();
            // Discrete nearest rank, not interpolation or max. Twenty observations
            // deliberately place the95th percentile below the maximum.
            let p95 = if durations.is_empty() {
                None
            } else {
                Some(durations[(durations.len() * 95).div_ceil(100) - 1])
            };
            let last = events
                .iter()
                .map(|row| row.get::<OffsetDateTime, _>("occurred_at"))
                .max()
                .unwrap();
            adoption.push((last, cycle.clone(), json!({"release_cycle":cycle,"console_route_events":console,
                "legacy_route_events":legacy,"rum_error_events":errors,"rum_perf_p95_ms":p95,"last_event_at":wire_time(last)})));
        }
        adoption.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.cmp(&a.1)));
        let zero_legacy = adoption
            .iter()
            .filter(|(_, _, row)| {
                row["console_route_events"].as_u64().unwrap() > 0 && row["legacy_route_events"] == 0
            })
            .count();
        let adoption: Vec<_> = adoption.into_iter().map(|(_, _, row)| row).collect();
        result.push(json!({"id":id,"slug":company.get::<String,_>("slug"),"name":company.get::<String,_>("name"),
            "status":company.get::<String,_>("status"),"group_id":company.get::<Option<Uuid>,_>("group_id"),
            "group_slug":company.get::<Option<String>,_>("group_slug"),"group_name":company.get::<Option<String>,_>("group_name"),
            "user_count":people.len(),"active_user_count":people.iter().filter(|row| row.get::<bool,_>("is_active")).count(),
            "active_work_orders":orders.iter().filter(|row| matches!(row.get::<String,_>("status").as_str(),"IN_PROGRESS"|"REPORT_SUBMITTED"|"ADMIN_REVIEW"|"ASSIGNED")).count(),
            "open_work_orders":orders.iter().filter(|row| !matches!(row.get::<String,_>("status").as_str(),"FINAL_COMPLETED"|"REJECTED"|"ARCHIVED"|"CANCELLED")).count(),
            "last_activity_at":last_activity.map(wire_time),"route_adoption":adoption,"zero_legacy_release_cycles":zero_legacy}));
    }
    json!({"tenants":result})
}

async fn seed_ops_content(pool: &PgPool) -> (Vec<Uuid>, OffsetDateTime) {
    let companies: Vec<Uuid> = sqlx::query_scalar(
        "SELECT id FROM public.organizations WHERE id<>$1 ORDER BY created_at,id",
    )
    .bind(OrgId::platform().as_uuid())
    .fetch_all(pool)
    .await
    .unwrap();
    assert!(
        companies.len() == 2,
        "exact two real Company fixture prerequisite"
    );
    let groups: Vec<Uuid> = sqlx::query_scalar(
        "SELECT group_id FROM public.organizations WHERE id=ANY($1) ORDER BY id",
    )
    .bind(&companies)
    .fetch_all(pool)
    .await
    .unwrap();
    assert!(
        groups.len() == 2 && groups[0] != groups[1],
        "distinct actual Group prerequisite"
    );
    let at = db_now(pool).await - Duration::days(3);
    // An actual creation-order tie exercises the stable Company id tiebreaker.
    sqlx::query("UPDATE public.organizations SET created_at=$1 WHERE id=ANY($2)")
        .bind(at)
        .bind(&companies)
        .execute(pool)
        .await
        .unwrap();
    for (index, org) in companies.iter().copied().enumerate() {
        let mut requester = Uuid::nil();
        for person in 0..(index + 2) {
            let id: Uuid = sqlx::query_scalar("INSERT INTO public.users(display_name,roles,org_id,is_active,created_at) VALUES($1,ARRAY['MEMBER'],$2,$3,$4) RETURNING id")
                .bind(format!("운영 건강 검증 회사{index} 담당자{person}"))
                .bind(org).bind(person != 1).bind(at+Duration::hours(person as i64))
                .fetch_one(pool).await.unwrap();
            if person == 0 {
                requester = id;
            }
        }
        let region: Uuid = sqlx::query_scalar(
            "INSERT INTO public.regions(name,org_id) VALUES($1,$2) RETURNING id",
        )
        .bind(format!("운영 건강 지역{index}"))
        .bind(org)
        .fetch_one(pool)
        .await
        .unwrap();
        let branch: Uuid = sqlx::query_scalar(
            "INSERT INTO public.branches(region_id,name,org_id) VALUES($1,$2,$3) RETURNING id",
        )
        .bind(region)
        .bind(format!("운영 건강 지점{index}"))
        .bind(org)
        .fetch_one(pool)
        .await
        .unwrap();
        let customer: Uuid = sqlx::query_scalar("INSERT INTO public.registry_customers(branch_id,name,org_id) VALUES($1,$2,$3) RETURNING id")
            .bind(branch).bind(format!("운영 건강 고객{index}")).bind(org).fetch_one(pool).await.unwrap();
        let site: Uuid = sqlx::query_scalar("INSERT INTO public.registry_sites(branch_id,customer_id,name,org_id) VALUES($1,$2,$3,$4) RETURNING id")
            .bind(branch).bind(customer).bind(format!("운영 건강 현장{index}")).bind(org).fetch_one(pool).await.unwrap();
        let equipment: Uuid = sqlx::query_scalar("INSERT INTO public.registry_equipment(branch_id,customer_id,site_id,equipment_no,management_no,manufacturer_code,kind_code,power_code,status,specification,ton_text,model,source_sheet,source_row,org_id) VALUES($1,$2,$3,$4,$5,'A','B','C','임대','좌식','2.5','GTS25DE','ops-test',1,$6) RETURNING id")
            .bind(branch).bind(customer).bind(site).bind(format!("ABC12-{:04}",index+8100))
            .bind(format!("ops-{index}")).bind(org).fetch_one(pool).await.unwrap();
        let statuses: &[&str] = if index == 0 {
            &[
                "ASSIGNED",
                "IN_PROGRESS",
                "REPORT_SUBMITTED",
                "ADMIN_REVIEW",
                "ON_HOLD",
                "FINAL_COMPLETED",
                "REJECTED",
                "ARCHIVED",
                "CANCELLED",
            ]
        } else {
            &["RECEIVED", "IN_PROGRESS", "DELAYED"]
        };
        for (sequence, status) in statuses.iter().enumerate() {
            sqlx::query("INSERT INTO public.work_orders(request_no,branch_id,equipment_id,customer_id,site_id,requested_by,status,symptom,org_id,updated_at) VALUES($1,$2,$3,$4,$5,$6,$7,'운영 건강 검증',$8,$9)")
                .bind(format!("20260920-{:03}",index*100+sequence+1)).bind(branch).bind(equipment)
                .bind(customer).bind(site).bind(requester).bind(status).bind(org)
                .bind(at+Duration::hours(8+sequence as i64)).execute(pool).await.unwrap();
        }
        for cycle in ["a-clean", "b-mixed", "c-null"] {
            let samples = if cycle == "a-clean" { 20 } else { 3 };
            for sample in 0..samples {
                let kind = if cycle == "a-clean" {
                    if sample == 0 {
                        "route_selection"
                    } else {
                        "rum_perf"
                    }
                } else if sample == 0 {
                    "rum_error"
                } else {
                    "route_selection"
                };
                let surface = if cycle == "b-mixed" && sample == 2 {
                    "legacy"
                } else {
                    "console"
                };
                let duration: Option<i32> = if cycle == "c-null" {
                    None
                } else if kind == "rum_error" {
                    Some(600000)
                } else {
                    Some((index as i32 + 1) * 100 + sample * 10)
                };
                sqlx::query("INSERT INTO public.console_route_telemetry(org_id,user_id,event_kind,route_surface,route_path,release_cycle,duration_ms,occurred_at) VALUES($1,$2,$3,$4,'/people',$5,$6,$7)")
                    .bind(org).bind(requester).bind(kind).bind(surface).bind(cycle).bind(duration)
                    // b/c tie at newer time; descending cycle tiebreaker is observable.
                    .bind(at+Duration::hours(if cycle=="a-clean" {20} else {21}))
                    .execute(pool).await.unwrap();
            }
        }
        // Company-scoped audit is deliberately the latest business activity for one
        // Company. Platform/NULL audits added by requests must never affect it.
        if index == 1 {
            sqlx::query("INSERT INTO public.audit_events(actor,action,target_type,target_id,trace_id,span_id,occurred_at,org_id) VALUES($1,'ops.fixture','organizations',$2,'fedcba0987654321fedcba0987654321','fedcba0987654321',$3,$4)")
                .bind(requester).bind(org.to_string()).bind(at+Duration::days(1)).bind(org)
                .execute(pool).await.unwrap();
        }
    }
    (companies, at)
}

async fn observe_ops(f: &Fixture) -> (StatusCode, Value) {
    let response = f
        .router
        .clone()
        .oneshot(
            Request::builder()
                .uri(OPS_PATH)
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
        "actual JSON ops prerequisite"
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

fn health_audit_matches(
    row: &Value,
    actor: UserId,
    count: usize,
    start: OffsetDateTime,
    end: OffsetDateTime,
) -> bool {
    if row["action"] != "platform.tenant.health"
        || row["target_type"] != "organizations"
        || row["target_id"] != "health"
    {
        return false;
    }
    // Check health-specific original fields first; reuse the unchanged strict20-field
    // list audit validator only for the identical remaining schema and time bounds.
    let mut normalized = row.clone();
    normalized["action"] = json!("platform.tenant.list");
    normalized["target_id"] = json!("list");
    audit_matches(&normalized, actor, count, start, end)
}
fn health_read_delta(
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
    health_audit_matches(&row, actor, count, start, end)
        && old.iter().all(|prior| {
            serde_json::from_str::<Value>(prior.get())
                .is_ok_and(|prior| prior["id"] != row["id"] && prior["trace_id"] != row["trace_id"])
        })
}

#[sqlx::test(migrations = false)]
async fn mounted_ops_current_member_denies_and_recovers(pool: PgPool) {
    let f = fixture(&pool).await;
    let (companies, seeded_at) = seed_ops_content(&pool).await;
    let expected = expected_health(&pool).await;
    assert!(
        health_matches(&expected, &expected),
        "complete raw-row health oracle prerequisite"
    );
    for (index, id) in companies.iter().enumerate() {
        let row = expected["tenants"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["id"] == json!(id))
            .unwrap();
        let base = (index + 1) * 100;
        let expected_cycles = json!([
            {"release_cycle":"c-null","console_route_events":2,"legacy_route_events":0,"rum_error_events":1,"rum_perf_p95_ms":null,"last_event_at":wire_time(seeded_at+Duration::hours(21))},
            {"release_cycle":"b-mixed","console_route_events":1,"legacy_route_events":1,"rum_error_events":1,"rum_perf_p95_ms":base+20,"last_event_at":wire_time(seeded_at+Duration::hours(21))},
            {"release_cycle":"a-clean","console_route_events":1,"legacy_route_events":0,"rum_error_events":0,"rum_perf_p95_ms":base+180,"last_event_at":wire_time(seeded_at+Duration::hours(20))}
        ]);
        assert!(
            row["route_adoption"] == expected_cycles && row["zero_legacy_release_cycles"] == 2,
            "persisted independent arithmetic must match known asymmetric fixture, including nonmax p95 and null/tied cycles"
        );
    }
    let count = expected["tenants"].as_array().unwrap().len();
    let before = all_rows(&pool).await;
    let start = db_now(&pool).await;
    let (positive_status, positive) = observe_ops(&f).await;
    let end = db_now(&pool).await;
    let after = all_rows(&pool).await;
    assert!(
        positive_status == StatusCode::OK && health_matches(&positive, &expected),
        "actual ops owner positive projection prerequisite"
    );
    set_role(&pool, f.actor, "MEMBER").await;
    let denied_before = all_rows(&pool).await;
    let (denied_status, denied) = observe_ops(&f).await;
    let denied_after = all_rows(&pool).await;
    set_role(&pool, f.actor, "SUPER_ADMIN").await;
    let recovery_before = all_rows(&pool).await;
    let recovery_start = db_now(&pool).await;
    let (recovery_status, recovery) = observe_ops(&f).await;
    let recovery_end = db_now(&pool).await;
    let recovery_after = all_rows(&pool).await;
    f.state.shutdown_realtime().await;
    f.business.close().await;
    assert!(
        recovery_status == StatusCode::OK && health_matches(&recovery, &expected),
        "same original bearer exact ops recovery prerequisite"
    );
    // This intentional owner RED precedes old caller-host-clock audit failure.
    // Strict positive and recovery DB-window assertions remain below, unchanged.
    assert!(
        denied_status == StatusCode::FORBIDDEN,
        "PLATFORM_OPS_CURRENT_ROLE_REQUIRED: current MEMBER disclosed cross-Company health"
    );
    assert!(
        denied
            == json!({"error":{"code":"forbidden","message":"platform principal cannot read tenant health"}}),
        "exact403 no health disclosure"
    );
    assert!(
        denied_before == denied_after,
        "denied health has no audit or other effect"
    );
    assert!(
        health_read_delta(&before, &after, f.actor, count, start, end),
        "exact positive health audit and allstate"
    );
    assert!(
        health_read_delta(
            &recovery_before,
            &recovery_after,
            f.actor,
            count,
            recovery_start,
            recovery_end
        ),
        "exact recovery health audit and allstate"
    );
}

fn health_projection_fixture() -> Value {
    let row = |offset: usize| {
        let cycle = |name: &str, duration: Option<i64>| {
            json!({"release_cycle":name,"console_route_events":offset+1,"legacy_route_events":0,
            "rum_error_events":offset,"rum_perf_p95_ms":duration,"last_event_at":"2026-09-20T00:00:00Z"})
        };
        json!({"id":Uuid::new_v4(),"slug":format!("company-{offset}"),"name":format!("회사{offset}"),"status":"ACTIVE",
            "group_id":Uuid::new_v4(),"group_slug":format!("group-{offset}"),"group_name":format!("그룹{offset}"),
            "user_count":offset+3,"active_user_count":offset+2,"active_work_orders":offset+1,"open_work_orders":offset+4,
            "last_activity_at":"2026-09-20T00:00:00Z","route_adoption":[cycle("b",None),cycle("a",Some(190+offset as i64))],"zero_legacy_release_cycles":2})
    };
    json!({"tenants":[row(0),row(1)]})
}

#[test]
fn ops_projection_oracle_rejects_omission_extra_reordering_and_each_field_corruption() {
    let expected = health_projection_fixture();
    assert!(health_matches(&expected, &expected));
    for key in HEALTH_KEYS {
        let mut missing = expected.clone();
        missing["tenants"][0].as_object_mut().unwrap().remove(*key);
        assert!(!health_matches(&missing, &expected), "missing health field");
        let mut changed = expected.clone();
        changed["tenants"][0][*key] = json!({"corrupt":true});
        assert!(!health_matches(&changed, &expected), "changed health field");
    }
    for key in ADOPTION_KEYS {
        let mut missing = expected.clone();
        missing["tenants"][0]["route_adoption"][0]
            .as_object_mut()
            .unwrap()
            .remove(*key);
        assert!(
            !health_matches(&missing, &expected),
            "missing adoption field"
        );
        let mut changed = expected.clone();
        changed["tenants"][0]["route_adoption"][0][*key] = json!({"corrupt":true});
        assert!(
            !health_matches(&changed, &expected),
            "changed adoption field"
        );
    }
    let mut corruptions = Vec::new();
    let mut x = expected.clone();
    x["tenants"].as_array_mut().unwrap().pop();
    corruptions.push(x);
    let mut x = expected.clone();
    x["tenants"].as_array_mut().unwrap().reverse();
    corruptions.push(x);
    let mut x = expected.clone();
    x["tenants"][0] = x["tenants"][1].clone();
    corruptions.push(x);
    let mut x = expected.clone();
    x["tenants"][0]["id"] = json!(OrgId::platform());
    corruptions.push(x);
    let mut x = expected.clone();
    x["tenants"][0]["route_adoption"]
        .as_array_mut()
        .unwrap()
        .reverse();
    corruptions.push(x);
    let mut x = expected.clone();
    x["tenants"][0]["route_adoption"]
        .as_array_mut()
        .unwrap()
        .pop();
    corruptions.push(x);
    let mut x = expected.clone();
    x["tenants"][0]["route_adoption"][0] = x["tenants"][1]["route_adoption"][0].clone();
    corruptions.push(x);
    let mut x = expected.clone();
    x["tenants"][0]["route_adoption"][1]["rum_perf_p95_ms"] = json!(200);
    corruptions.push(x);
    let mut x = expected.clone();
    x["tenants"][0]["private_salary"] = json!(1);
    corruptions.push(x);
    let mut x = expected.clone();
    x["tenants"][0]["route_adoption"][0]["secret"] = json!(1);
    corruptions.push(x);
    let mut x = expected.clone();
    x["secret"] = json!(1);
    corruptions.push(x);
    for changed in corruptions {
        assert!(
            !health_matches(&changed, &expected),
            "projection corruption escaped"
        );
    }
}

#[test]
fn ops_audit_allstate_oracle_rejects_omission_corruption_and_rewritten_history() {
    let actor = UserId::new();
    let at = OffsetDateTime::parse(
        "2026-09-20T00:00:00Z",
        &time::format_description::well_known::Rfc3339,
    )
    .unwrap();
    let start = at - Duration::seconds(1);
    let end = at + Duration::seconds(1);
    let prior = audit_fixture(actor, at - Duration::days(1));
    let mut added = audit_fixture(actor, at);
    added["action"] = json!("platform.tenant.health");
    added["target_id"] = json!("health");
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
    assert!(health_read_delta(&before, &after, actor, 2, start, end));
    assert!(!health_read_delta(&before, &before, actor, 2, start, end));
    for key in AUDIT_KEYS {
        let mut missing = added.clone();
        missing.as_object_mut().unwrap().remove(*key);
        assert!(
            !health_read_delta(&before, &delta(&missing), actor, 2, start, end),
            "missing audit field"
        );
        let mut corrupt = added.clone();
        corrupt[*key] = json!({"corrupt":true});
        assert!(
            !health_read_delta(&before, &delta(&corrupt), actor, 2, start, end),
            "corrupted audit field"
        );
    }
    for key in ["occurred_at", "created_at"] {
        let mut corrupt = added.clone();
        corrupt[key] = json!(wire_time(start - Duration::nanoseconds(1)));
        assert!(
            !health_read_delta(&before, &delta(&corrupt), actor, 2, start, end),
            "early timestamp"
        );
        corrupt[key] = json!(wire_time(end + Duration::nanoseconds(1)));
        assert!(
            !health_read_delta(&before, &delta(&corrupt), actor, 2, start, end),
            "late timestamp"
        );
    }
    for key in ["id", "trace_id"] {
        let mut corrupt = added.clone();
        corrupt[key] = prior[key].clone();
        assert!(
            !health_read_delta(&before, &delta(&corrupt), actor, 2, start, end),
            "reused audit identity"
        );
    }
    for (key, value) in [
        ("actor", json!(UserId::new())),
        ("after_snap", json!({"count":1})),
        ("org_id", json!(Uuid::new_v4())),
        ("action", json!("platform.tenant.list")),
        ("target_type", json!("users")),
        ("target_id", json!("list")),
        ("extra", json!(true)),
    ] {
        let mut corrupt = added.clone();
        corrupt[key] = value;
        assert!(
            !health_read_delta(&before, &delta(&corrupt), actor, 2, start, end),
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
            !health_read_delta(&before, &changed, actor, 2, start, end),
            "history/effect corruption escaped"
        );
    }
}
