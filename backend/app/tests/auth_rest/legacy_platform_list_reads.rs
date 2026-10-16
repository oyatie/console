//! Initial actual-owner RED only. Later retained-lock/SQL/privacy histories remain required.
//! The mounted App, real Auth/Business logins and actual OTP/passkey login are prerequisites.
use super::*;
use account_browser::deployment_operator_designation::company_setup::all_rows;
use console_platform_auth::{JwtSettings, JwtVerifier};
use console_platform_test_support::login_test_pool;
use serde_json::value::RawValue;
use sqlx::Row;
use std::collections::{BTreeMap, BTreeSet};

const PATH: &str = "/api/platform/orgs";
type Rows = BTreeMap<String, String>;
const AUDIT_KEYS: &[&str] = &[
    "id",
    "actor",
    "action",
    "target_type",
    "target_id",
    "branch_id",
    "before_snap",
    "after_snap",
    "trace_id",
    "span_id",
    "occurred_at",
    "created_at",
    "org_id",
    "ip",
    "user_agent",
    "auth_method",
    "device",
    "classification_badges",
    "anomaly",
    "reason",
];

fn metadata_matches(actual: &Value, expected: &Value) -> bool {
    let Some(rows) = actual.as_array() else {
        return false;
    };
    !rows.is_empty()
        && actual == expected
        && rows.iter().all(|row| {
            row.as_object().is_some_and(|fields| fields.len() == 9)
                && row["id"].as_str().is_some_and(|id| {
                    Uuid::parse_str(id)
                        .is_ok_and(|id| !id.is_nil() && id != *OrgId::platform().as_uuid())
                })
        })
}

fn time_in_window(value: &Value, start: OffsetDateTime, end: OffsetDateTime) -> bool {
    value.as_str().is_some_and(|text| {
        OffsetDateTime::parse(text, &time::format_description::well_known::Rfc3339)
            .is_ok_and(|at| start <= at && at <= end)
    })
}

fn lower_hex(value: &Value, len: usize) -> bool {
    value.as_str().is_some_and(|text| {
        text.len() == len
            && text
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            && text.bytes().any(|b| b != b'0')
    })
}

fn audit_matches(
    row: &Value,
    actor: UserId,
    count: usize,
    start: OffsetDateTime,
    end: OffsetDateTime,
) -> bool {
    row.as_object().is_some_and(|fields| {
        fields.keys().map(String::as_str).collect::<BTreeSet<_>>()
            == AUDIT_KEYS.iter().copied().collect()
    }) && row["id"]
        .as_str()
        .is_some_and(|id| Uuid::parse_str(id).is_ok_and(|id| !id.is_nil()))
        && row["actor"] == json!(actor)
        && row["action"] == "platform.tenant.list"
        && row["target_type"] == "organizations"
        && row["target_id"] == "list"
        && row["after_snap"] == json!({"count": count})
        && [
            "branch_id",
            "before_snap",
            "org_id",
            "ip",
            "user_agent",
            "auth_method",
            "device",
            "classification_badges",
            "anomaly",
            "reason",
        ]
        .iter()
        .all(|key| row[*key].is_null())
        && lower_hex(&row["trace_id"], 32)
        && lower_hex(&row["span_id"], 16)
        && time_in_window(&row["occurred_at"], start, end)
        && time_in_window(&row["created_at"], start, end)
}

fn exact_read_delta(
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
    let Some(old) = before.get("audit_events") else {
        return false;
    };
    let Some(new) = after.get("audit_events") else {
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
    if !audit_matches(&row, actor, count, start, end) {
        return false;
    }
    old.iter().all(|prior| {
        serde_json::from_str::<Value>(prior.get())
            .is_ok_and(|prior| prior["id"] != row["id"] && prior["trace_id"] != row["trace_id"])
    })
}

// Diagnostics only: fixed field names, booleans, row counts and relative times.
// Never serialize a row, actor, token, trace, absolute timestamp or SQL error.
fn list_read_diagnostic(
    before: &Rows,
    after: &Rows,
    actor: UserId,
    count: usize,
    start: OffsetDateTime,
    end: OffsetDateTime,
) -> Value {
    let count_rows = |raw: Option<&String>| {
        raw.and_then(|raw| serde_json::from_str::<Vec<&RawValue>>(raw).ok())
            .map(|rows| rows.len())
    };
    let tables: BTreeSet<_> = before.keys().chain(after.keys()).collect();
    let changed: Vec<_> = tables
        .into_iter()
        .filter(|table| before.get(*table) != after.get(*table))
        .map(|table| {
            json!({"table":table,"before_count":count_rows(before.get(table)),
            "after_count":count_rows(after.get(table))})
        })
        .collect();
    let old = before
        .get("audit_events")
        .and_then(|raw| serde_json::from_str::<Vec<&RawValue>>(raw).ok());
    let new = after
        .get("audit_events")
        .and_then(|raw| serde_json::from_str::<Vec<&RawValue>>(raw).ok());
    let audit = match (old, new) {
        (Some(old), Some(new)) => {
            let old_bytes: BTreeSet<_> = old.iter().map(|r| r.get()).collect();
            let new_bytes: BTreeSet<_> = new.iter().map(|r| r.get()).collect();
            let added: Vec<_> = new_bytes.difference(&old_bytes).collect();
            let fields:Vec<_> = added.iter().take(4).map(|raw| {
                let Ok(row) = serde_json::from_str::<Value>(raw) else {
                    return json!({"parse_ok":false});
                };
                let expected_keys:BTreeSet<_> = AUDIT_KEYS.iter().copied().collect();
                let actual_keys:BTreeSet<_> = row.as_object().map(|r|r.keys().map(String::as_str).collect()).unwrap_or_default();
                let mut field_checks=serde_json::Map::new();
                for key in AUDIT_KEYS {
                    let matches = match *key {
                        "id" => row[*key].as_str().is_some_and(|v|Uuid::parse_str(v).is_ok_and(|id|!id.is_nil())),
                        "actor" => row[*key] == json!(actor),
                        "action" => row[*key] == "platform.tenant.list",
                        "target_type" => row[*key] == "organizations",
                        "target_id" => row[*key] == "list",
                        "after_snap" => row[*key] == json!({"count":count}),
                        "trace_id" => lower_hex(&row[*key],32),
                        "span_id" => lower_hex(&row[*key],16),
                        "occurred_at" | "created_at" => time_in_window(&row[*key],start,end),
                        _ => row[*key].is_null(),
                    };
                    field_checks.insert((*key).to_owned(),json!({"present":actual_keys.contains(key),"matches":matches}));
                }
                let times:serde_json::Map<String,Value> = ["occurred_at","created_at"].into_iter().map(|key| {
                    let parsed = row[key].as_str().and_then(|v|OffsetDateTime::parse(v,&time::format_description::well_known::Rfc3339).ok());
                    (key.to_owned(),match parsed {
                        Some(at) => json!({"parse_ok":true,"from_start_us":(at-start).whole_microseconds(),"to_end_us":(end-at).whole_microseconds()}),
                        None => json!({"parse_ok":false}),
                    })
                }).collect();
                let fresh_identity = old.iter().all(|prior|serde_json::from_str::<Value>(prior.get()).is_ok_and(|p|p["id"]!=row["id"] && p["trace_id"]!=row["trace_id"]));
                json!({"parse_ok":true,"keys_match":actual_keys==expected_keys,
                    "unexpected_key_count":actual_keys.difference(&expected_keys).count(),
                    "fields":field_checks,"time_deltas":times,"fresh_id_and_trace":fresh_identity,
                    "audit_oracle":audit_matches(&row,actor,count,start,end)})
            }).collect();
            json!({"parse_ok":true,"before_count":old.len(),"after_count":new.len(),
                "before_unique":old_bytes.len()==old.len(),"after_unique":new_bytes.len()==new.len(),
                "prior_rows_preserved":old_bytes.is_subset(&new_bytes),"added_count":added.len(),
                "added_diagnostics_first_four":fields})
        }
        _ => json!({"parse_ok":false}),
    };
    json!({"table_keys_match":before.keys().eq(after.keys()),"changed_tables":changed,
        "window_us":(end-start).whole_microseconds(),"audit":audit})
}

async fn db_now(pool: &PgPool) -> OffsetDateTime {
    sqlx::query_scalar("SELECT pg_catalog.clock_timestamp()")
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn expected_metadata(pool: &PgPool) -> Value {
    // Independent persisted oracle, never a call to the challenged raw function/use case.
    let rows = sqlx::query("SELECT o.id,o.slug,o.name,o.status,o.created_at,o.updated_at,g.id AS group_id,g.slug AS group_slug,g.name AS group_name FROM public.organizations o LEFT JOIN public.groups g ON g.id=o.group_id WHERE o.id<>$1 ORDER BY o.created_at,o.id")
        .bind(OrgId::platform().as_uuid()).fetch_all(pool).await.unwrap();
    Value::Array(
        rows.into_iter()
            .map(|row| {
                let created: OffsetDateTime = row.get("created_at");
                let updated: OffsetDateTime = row.get("updated_at");
                json!({"id": row.get::<Uuid,_>("id"), "slug":row.get::<String,_>("slug"),
            "name":row.get::<String,_>("name"), "status":row.get::<String,_>("status"),
            "group_id":row.get::<Option<Uuid>,_>("group_id"),
            "group_slug":row.get::<Option<String>,_>("group_slug"),
            "group_name":row.get::<Option<String>,_>("group_name"),
            "created_at":created.format(&time::format_description::well_known::Rfc3339).unwrap(),
            "updated_at":updated.format(&time::format_description::well_known::Rfc3339).unwrap()})
            })
            .collect(),
    )
}

async fn observe(router: &axum::Router, access: &str, secrets: &[String]) -> (StatusCode, Value) {
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .uri(PATH)
                .header(header::AUTHORIZATION, format!("Bearer {access}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let (parts, body) = response.into_parts();
    let raw = to_bytes(body, 1024 * 1024).await.unwrap();
    let body = std::str::from_utf8(&raw).unwrap();
    for secret in secrets
        .iter()
        .map(String::as_str)
        .chain(std::iter::once(access))
    {
        assert!(
            !secret.is_empty(),
            "privacy fixture secret must be nonempty"
        );
        assert!(
            !body.contains(secret),
            "response body reflected a credential"
        );
        assert!(
            parts.headers.iter().all(|(_, v)| !v
                .as_bytes()
                .windows(secret.len())
                .any(|w| w == secret.as_bytes())),
            "response header reflected a credential"
        );
    }
    assert!(
        parts
            .headers
            .get(header::CONTENT_TYPE)
            .is_some_and(|v| v == "application/json"),
        "actual JSON route prerequisite"
    );
    (status, serde_json::from_str(body).unwrap())
}

async fn set_role(pool: &PgPool, actor: UserId, role: &str) {
    let result = sqlx::query("UPDATE public.users SET roles=$2 WHERE id=$1")
        .bind(actor.as_uuid())
        .bind(vec![role])
        .execute(pool)
        .await
        .unwrap();
    assert!(
        result.rows_affected() == 1,
        "one persisted source role transition"
    );
    let roles: Vec<String> = sqlx::query_scalar("SELECT roles FROM public.users WHERE id=$1")
        .bind(actor.as_uuid())
        .fetch_one(pool)
        .await
        .unwrap();
    assert!(roles == vec![role], "committed source role readback");
}

#[sqlx::test(migrations = false)]
async fn mounted_platform_list_same_bearer_current_member_denies_and_restores(pool: PgPool) {
    prepare_http_database(&pool).await;
    let business = login_test_pool(&pool, TestDatabaseLogin::Business).await;
    let auth = login_test_pool(&pool, TestDatabaseLogin::Auth).await;
    for (connection, role) in [(&business, "console_rt"), (&auth, "console_auth_rt")] {
        let identity: (String, String) =
            sqlx::query_as("SELECT session_user::text,current_user::text")
                .fetch_one(connection)
                .await
                .unwrap();
        assert!(
            identity == (role.to_owned(), role.to_owned()),
            "real runtime login prerequisite"
        );
    }
    let actor = UserId::new();
    sqlx::query("INSERT INTO public.users(id,display_name,roles,org_id) VALUES($1,$2,$3,$4)")
        .bind(actor.as_uuid())
        .bind("플랫폼 목록 현재 권한 검증 담당자")
        .bind(vec!["SUPER_ADMIN"])
        .bind(OrgId::platform().as_uuid())
        .execute(&pool)
        .await
        .unwrap();
    let root_present: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM public.accounts WHERE id=$1)")
            .bind(actor.as_uuid())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(root_present, "actual legacy Account root prerequisite");
    let tied = db_now(&pool).await;
    let mut created = Vec::new();
    for (suffix, name) in [
        ("a", "서울 장기 근속 인사 급여 운영 회사"),
        ("b", "부산 현장 운영 회사"),
    ] {
        let id: Uuid = sqlx::query_scalar("INSERT INTO public.organizations(slug,name,status,created_at,updated_at) VALUES($1,$2,'ACTIVE',$3,$3) RETURNING id")
            .bind(format!("list-owner-{suffix}")).bind(name.repeat(3)).bind(tied)
            .fetch_one(&pool).await.unwrap();
        created.push(id);
    }
    let groups: Vec<Uuid> = sqlx::query_scalar(
        "SELECT group_id FROM public.organizations WHERE id=ANY($1) ORDER BY id",
    )
    .bind(&created)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert!(
        groups.len() == 2 && groups[0] != groups[1],
        "real distinct Group identity prerequisite"
    );
    let key = SigningKey::random(&mut OsRng);
    let private = key.to_pkcs8_pem(LineEnding::LF).unwrap().to_string();
    let public = key
        .verifying_key()
        .to_public_key_pem(LineEnding::LF)
        .unwrap();
    let state = app_state(pool.clone(), private, public.clone())
        .await
        .unwrap();
    let router = build_router(state.clone());
    let issued = BootstrapCredentialStore
        .issue_for_zero_credential_user(
            &business,
            *actor.as_uuid(),
            OrgId::platform(),
            db_now(&pool).await,
            Duration::hours(1),
        )
        .await
        .unwrap();
    let redeemed: OtpRedeemResponse = post_json(
        router.clone(),
        "/api/v1/auth/otp/redeem",
        None,
        json!({"otp":issued.token.as_str()}),
        StatusCode::OK,
    )
    .await;
    let mut authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let credential = enroll_passkey(&router, &mut authenticator, &redeemed.access_token).await;
    let mut login = usernameless_login(&router, &mut authenticator, &credential).await;
    let verifier = JwtVerifier::from_es256_public_pem(
        JwtSettings {
            issuer: TEST_ISSUER.to_owned(),
            audience: TEST_AUDIENCE.to_owned(),
            access_token_ttl: Duration::minutes(15),
        },
        public.as_bytes(),
    )
    .unwrap();
    // Historical-shape successor: preserve all verified claims and original
    // expiry; only remove the newly produced additive binding in test memory.
    login.access_token = crate::legacy_platform_binding_producer::historical_access(
        &key,
        &verifier,
        &login.access_token,
    );
    let claims = verifier.verify_access_token(&login.access_token).unwrap();
    assert!(
        claims.sub == actor.to_string()
            && claims.org == OrgId::platform().to_string()
            && claims.platform
            && !claims.view_as
            && !claims.read_only
            && claims.legacy_session.is_none(),
        "genuine unbound direct platform login prerequisite"
    );
    let secrets = vec![
        issued.token.as_str().to_owned(),
        redeemed.access_token,
        redeemed.refresh_token.unwrap(),
        login.refresh_token.unwrap(),
    ];
    let expected = expected_metadata(&pool).await;
    let count = expected.as_array().unwrap().len();
    assert!(
        count >= 2
            && created.iter().all(|id| expected
                .as_array()
                .unwrap()
                .iter()
                .any(|r| r["id"] == json!(id))),
        "nonempty real metadata fixture"
    );
    let before = all_rows(&pool).await;
    let start = db_now(&pool).await;
    let (positive_status, positive) = observe(&router, &login.access_token, &secrets).await;
    let end = db_now(&pool).await;
    let after = all_rows(&pool).await;
    assert!(
        positive_status == StatusCode::OK && metadata_matches(&positive, &expected),
        "actual owner positive prerequisite"
    );
    if !exact_read_delta(&before, &after, actor, count, start, end) {
        eprintln!(
            "PLATFORM_LIST_PREREQUISITE_DIAGNOSTIC {}",
            list_read_diagnostic(&before, &after, actor, count, start, end)
        );
    }

    set_role(&pool, actor, "MEMBER").await;
    let denied_before = all_rows(&pool).await;
    let (denied_status, denied_body) = observe(&router, &login.access_token, &secrets).await;
    let denied_after = all_rows(&pool).await;
    // Restore the exact fixture role and prove recovery before the intended RED assertion.
    set_role(&pool, actor, "SUPER_ADMIN").await;
    let recovery_before = all_rows(&pool).await;
    let recovery_start = db_now(&pool).await;
    let (recovery_status, recovery) = observe(&router, &login.access_token, &secrets).await;
    let recovery_end = db_now(&pool).await;
    let recovery_after = all_rows(&pool).await;
    state.shutdown_realtime().await;
    business.close().await;
    auth.close().await;
    assert!(
        recovery_status == StatusCode::OK && metadata_matches(&recovery, &expected),
        "same bearer restored source metadata prerequisite"
    );
    assert!(
        denied_status == StatusCode::FORBIDDEN,
        "PLATFORM_LIST_CURRENT_ROLE_REQUIRED: mounted owner disclosed metadata after committed MEMBER demotion"
    );
    assert!(
        exact_read_delta(&before, &after, actor, count, start, end),
        "exact positive list audit and no other effects prerequisite"
    );
    assert!(
        recovery_status == StatusCode::OK
            && metadata_matches(&recovery, &expected)
            && exact_read_delta(
                &recovery_before,
                &recovery_after,
                actor,
                count,
                recovery_start,
                recovery_end
            ),
        "same bearer restored source must recover through exact owner"
    );
    assert!(
        denied_body
            == json!({"error":{"code":"forbidden","message":"platform principal cannot list tenants"}}),
        "exact forbidden envelope, no metadata"
    );
    assert!(
        denied_before == denied_after,
        "denied read must append no audit or other effect"
    );
}

fn audit_fixture(actor: UserId, at: OffsetDateTime) -> Value {
    let mut row = serde_json::Map::new();
    for key in AUDIT_KEYS {
        row.insert((*key).to_owned(), Value::Null);
    }
    let mut row = Value::Object(row);
    row["id"] = json!(Uuid::new_v4());
    row["actor"] = json!(actor);
    row["action"] = json!("platform.tenant.list");
    row["target_type"] = json!("organizations");
    row["target_id"] = json!("list");
    row["after_snap"] = json!({"count":2});
    row["trace_id"] = json!("1234567890abcdef1234567890abcdef");
    row["span_id"] = json!("1234567890abcdef");
    let at = at
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap();
    row["occurred_at"] = json!(at);
    row["created_at"] = row["occurred_at"].clone();
    row
}

#[test]
fn list_metadata_oracle_rejects_missing_extra_reordered_and_private_fields() {
    let at = "2026-09-20T00:00:00Z";
    let row = |id: Uuid, group: Uuid| {
        json!({"id":id,"slug":id.to_string(),"name":"한글 회사 이름",
        "status":"ACTIVE","created_at":at,"updated_at":at,"group_id":group,
        "group_slug":group.to_string(),"group_name":"운영 그룹"})
    };
    let expected = json!([
        row(Uuid::new_v4(), Uuid::new_v4()),
        row(Uuid::new_v4(), Uuid::new_v4())
    ]);
    assert!(metadata_matches(&expected, &expected));
    assert!(
        !metadata_matches(&json!([]), &json!([])),
        "empty positive cannot pass"
    );
    let mut variants = vec![
        json!([]),
        json!({"items":expected.clone()}),
        json!([expected[0].clone()]),
    ];
    let mut reversed = expected.clone();
    reversed.as_array_mut().unwrap().reverse();
    variants.push(reversed);
    let mut duplicate = expected.clone();
    duplicate.as_array_mut().unwrap().push(expected[0].clone());
    variants.push(duplicate);
    for key in expected[0].as_object().unwrap().keys() {
        let mut missing = expected.clone();
        missing[0].as_object_mut().unwrap().remove(key);
        variants.push(missing);
        let mut changed = expected.clone();
        changed[0][key] = json!("corrupt");
        variants.push(changed);
    }
    let mut private = expected.clone();
    private[0]["salary"] = json!(1000000);
    variants.push(private);
    let mut sentinel = expected.clone();
    sentinel[0]["id"] = json!(OrgId::platform());
    variants.push(sentinel);
    for actual in variants {
        assert!(
            !metadata_matches(&actual, &expected),
            "metadata corruption escaped real oracle"
        );
    }
}

#[test]
fn list_audit_full_state_oracle_rejects_missing_effects_and_history_corruption() {
    let actor = UserId::new();
    let at = OffsetDateTime::from_unix_timestamp(1_789_862_400).unwrap();
    let row = audit_fixture(actor, at);
    let mut prior = audit_fixture(actor, at);
    prior["trace_id"] = json!("abcdef1234567890abcdef1234567890");
    let before = Rows::from([
        ("users".to_owned(), "[{\"id\":\"retained\"}]".to_owned()),
        (
            "audit_events".to_owned(),
            json!([prior.clone()]).to_string(),
        ),
    ]);
    let after = Rows::from([
        ("users".to_owned(), before["users"].clone()),
        (
            "audit_events".to_owned(),
            json!([prior.clone(), row.clone()]).to_string(),
        ),
    ]);
    let check = |actual: &Rows| {
        exact_read_delta(
            &before,
            actual,
            actor,
            2,
            at - Duration::seconds(1),
            at + Duration::seconds(1),
        )
    };
    assert!(check(&after), "positive full-state oracle control");
    assert!(!check(&before), "missing actual audit escaped");
    assert!(!exact_read_delta(
        &Rows::new(),
        &Rows::new(),
        actor,
        2,
        at,
        at
    ));
    for key in AUDIT_KEYS {
        let mut missing = row.clone();
        missing.as_object_mut().unwrap().remove(*key);
        let mut altered = row.clone();
        altered[*key] = json!("corrupt");
        for candidate in [missing, altered] {
            let mut corrupted = after.clone();
            corrupted.insert(
                "audit_events".to_owned(),
                json!([prior.clone(), candidate]).to_string(),
            );
            assert!(!check(&corrupted), "exact audit field corruption escaped");
        }
    }
    let mut extra = row.clone();
    extra["extra"] = json!(true);
    let mut wrong_count = row.clone();
    wrong_count["after_snap"] = json!({"count":3});
    let mut extra_snapshot = row.clone();
    extra_snapshot["after_snap"] = json!({"count":2,"secret":"private"});
    let mut reused_id = row.clone();
    reused_id["id"] = prior["id"].clone();
    let mut reused_trace = row.clone();
    reused_trace["trace_id"] = prior["trace_id"].clone();
    let mut expired = row.clone();
    expired["occurred_at"] = json!(
        (at - Duration::seconds(2))
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap()
    );
    for candidate in [
        extra,
        wrong_count,
        extra_snapshot,
        reused_id,
        reused_trace,
        expired,
    ] {
        let mut corrupted = after.clone();
        corrupted.insert(
            "audit_events".to_owned(),
            json!([prior.clone(), candidate]).to_string(),
        );
        assert!(!check(&corrupted), "audit semantic corruption escaped");
    }
    let mut variants = Vec::new();
    let mut added = after.clone();
    added.insert("unexpected_table".to_owned(), "[]".to_owned());
    variants.push(added);
    let mut missing = after.clone();
    missing.remove("users");
    variants.push(missing);
    let mut altered = after.clone();
    altered.insert("users".to_owned(), "[]".to_owned());
    variants.push(altered);
    for audit in [
        json!([row.clone()]),
        json!([prior.clone(), row.clone(), row.clone()]),
        json!([row.clone(), row.clone()]),
    ] {
        let mut corrupted = after.clone();
        corrupted.insert("audit_events".to_owned(), audit.to_string());
        variants.push(corrupted);
    }
    let mut changed_prior = prior.clone();
    changed_prior["target_id"] = json!("rewritten");
    let mut corrupted = after.clone();
    corrupted.insert(
        "audit_events".to_owned(),
        json!([changed_prior, row]).to_string(),
    );
    variants.push(corrupted);
    for actual in variants {
        assert!(
            !check(&actual),
            "history/table corruption escaped real oracle"
        );
    }
}

#[path = "legacy_platform_list_histories.rs"]
mod histories;

pub(super) use histories::{WireEvidence, evidence, proven_ack_loss, relay};
