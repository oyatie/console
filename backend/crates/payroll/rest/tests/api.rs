#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! HTTP-level proofs over the real `console-payroll-rest` router, driven on a
//! genuine non-owner `console_rt` pool (RLS actually enforced).
//!
//! Proves:
//!  * `GET /api/v1/payroll/payslips/me` is draft READINESS, not issued
//!    명세서: no vault document fields (`kind`, `legal_basis`,
//!    `confirmed_at`, inbox doc id) and no won amounts;
//!  * it is self-scoped, not `PayrollRunRead`-gated, mirroring
//!    `GET /api/v1/hr/attendance-records/me`: MEMBER without that capability
//!    still reads own draft lines (200); an account with no linked employee
//!    reads an empty page (200), never a 403; a linked employee reads ONLY
//!    their own draft lines, never a coworker's, including when a foreign
//!    `employee_id` query param is supplied;
//!  * `GET /api/v1/payroll/runs` / `/runs/{id}` are EXECUTIVE/SUPER_ADMIN-only
//!    admin reads — MEMBER and a branch-scoped ADMIN are 403;
//!  * another org's runs are invisible to a SUPER_ADMIN of THIS org (RLS).

use axum::body::{Body, to_bytes};
use console_kernel_core::{AuditAction, AuditEvent, OrgId, TraceContext, UserId};
use console_payroll_adapter_postgres::PgPayrollStore;
use console_payroll_rest::{PAYROLL_MY_PAYSLIPS_PATH, PAYROLL_RUNS_PATH, PayrollRestState, router};
use console_platform_auth::{AccessTokenInput, JwtIssuer, JwtSettings, JwtVerifier};
use console_platform_db::{DbError, with_audit};
use console_platform_test_support::runtime_role_pool;
use http::{Request, StatusCode, header};
use p256::ecdsa::SigningKey;
use p256::elliptic_curve::rand_core::OsRng;
use p256::pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding};
use serde_json::{Value, json};
use sqlx::PgPool;
use std::collections::BTreeSet;
use time::macros::date;
use time::{Duration, OffsetDateTime};
use tower::ServiceExt;
use uuid::Uuid;

const TEST_ISSUER: &str = "console-platform-auth";
const TEST_AUDIENCE: &str = "console-api";

struct Keys {
    private_pem: String,
    public_pem: String,
}

struct JsonResponse {
    status: StatusCode,
    json: Value,
}

fn keys() -> Keys {
    let signing_key = SigningKey::random(&mut OsRng);
    Keys {
        private_pem: signing_key
            .to_pkcs8_pem(LineEnding::LF)
            .unwrap()
            .to_string(),
        public_pem: signing_key
            .verifying_key()
            .to_public_key_pem(LineEnding::LF)
            .unwrap(),
    }
}

fn bearer(keys: &Keys, user_id: UserId, org: OrgId, role: &str) -> String {
    let issuer = JwtIssuer::from_es256_pem(
        JwtSettings {
            issuer: TEST_ISSUER.to_owned(),
            audience: TEST_AUDIENCE.to_owned(),
            access_token_ttl: Duration::minutes(15),
        },
        keys.private_pem.as_bytes(),
        keys.public_pem.as_bytes(),
    )
    .unwrap();
    issuer
        .issue_access_token(AccessTokenInput {
            subject: user_id,
            org_id: org,
            roles: vec![role.to_owned()],
            branches: Vec::new(),
            platform: false,
            view_as: false,
            read_only: false,
            display_name: None,
            feature_grants: Vec::new(),
            authz_subject_version: 0,
            authz_policy_version: 0,
            session_generation: 0,
            issued_at: OffsetDateTime::now_utc(),
        })
        .unwrap()
}

fn app(pool: PgPool, keys: &Keys) -> axum::Router {
    let verifier = JwtVerifier::from_es256_public_pem(
        JwtSettings {
            issuer: TEST_ISSUER.to_owned(),
            audience: TEST_AUDIENCE.to_owned(),
            access_token_ttl: Duration::minutes(15),
        },
        keys.public_pem.as_bytes(),
    )
    .unwrap();
    let store = PgPayrollStore::new(pool);
    router(PayrollRestState::new(store, Some(verifier)))
}

async fn get(service: axum::Router, uri: &str, token: &str) -> JsonResponse {
    let request = Request::builder()
        .uri(uri)
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let response = service.oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json = serde_json::from_slice(&bytes).unwrap_or_else(|_| json!({}));
    JsonResponse { status, json }
}

async fn get_unauthenticated(service: axum::Router, uri: &str) -> StatusCode {
    let request = Request::builder()
        .uri(uri)
        .method("GET")
        .body(Body::empty())
        .unwrap();
    service.oneshot(request).await.unwrap().status()
}

fn test_audit_event(
    action: &str,
    target_type: &str,
    target_id: impl ToString,
    org: Uuid,
) -> AuditEvent {
    AuditEvent::new(
        None,
        AuditAction::new(action).unwrap(),
        target_type,
        target_id.to_string(),
        TraceContext::generate(),
        OffsetDateTime::now_utc(),
    )
    .with_org(OrgId::from_uuid(org))
}

async fn seed_org(owner_pool: &PgPool, org: Uuid, tag: &str) {
    let event = test_audit_event("test.seed_org", "organization", org, org);
    let tag = tag.to_owned();
    with_audit(owner_pool, event, |tx| {
        Box::pin(async move {
            sqlx::query(
                "INSERT INTO organizations (id, slug, name) VALUES ($1, $2, $3) ON CONFLICT (id) DO NOTHING",
            )
            .bind(org)
            .bind(format!("org-{}", tag.to_lowercase()))
            .bind(format!("Org {tag}"))
            .execute(tx.as_mut())
            .await
            .map_err(DbError::Sqlx)?;
            Ok::<(), DbError>(())
        })
    })
    .await
    .unwrap();
}

async fn seed_user(owner_pool: &PgPool, user_id: UserId, org: Uuid, role: &str) {
    let event = test_audit_event("test.seed_user", "user", *user_id.as_uuid(), org);
    let role = role.to_owned();
    with_audit(owner_pool, event, |tx| {
        Box::pin(async move {
            sqlx::query(
                "INSERT INTO users (id, display_name, roles, org_id) VALUES ($1, $2, $3, $4)",
            )
            .bind(*user_id.as_uuid())
            .bind(format!("user-{role}-{}", user_id.as_uuid()))
            .bind(vec![role])
            .bind(org)
            .execute(tx.as_mut())
            .await
            .map_err(DbError::Sqlx)?;
            Ok::<(), DbError>(())
        })
    })
    .await
    .unwrap();
}

async fn seed_employee(owner_pool: &PgPool, org: Uuid, name: &str) -> Uuid {
    let id = Uuid::new_v4();
    let event = test_audit_event("test.seed_employee", "employee", id, org);
    let name = name.to_owned();
    with_audit(owner_pool, event, |tx| {
        Box::pin(async move {
            sqlx::query(
                "INSERT INTO employees \
                 (id, org_id, company, name, source_filename, source_sheet, source_row, source_key) \
                 VALUES ($1, $2, 'KNL', $3, 'roster.xlsx', 'Sheet1', 1, $4)",
            )
            .bind(id)
            .bind(org)
            .bind(name)
            .bind(format!("emp-{id}"))
            .execute(tx.as_mut())
            .await
            .map_err(DbError::Sqlx)?;
            Ok::<(), DbError>(())
        })
    })
    .await
    .unwrap();
    id
}

async fn seed_user_linked_to_employee(
    owner_pool: &PgPool,
    user_id: UserId,
    org: Uuid,
    employee: Uuid,
) {
    let event = test_audit_event("test.seed_user", "user", *user_id.as_uuid(), org);
    with_audit(owner_pool, event, |tx| {
        Box::pin(async move {
            sqlx::query(
                "INSERT INTO users (id, display_name, roles, org_id, employee_id) \
                 VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(*user_id.as_uuid())
            .bind(format!("linked-{}", user_id.as_uuid()))
            .bind(vec!["MEMBER".to_string()])
            .bind(org)
            .bind(employee)
            .execute(tx.as_mut())
            .await
            .map_err(DbError::Sqlx)?;
            Ok::<(), DbError>(())
        })
    })
    .await
    .unwrap();
}

async fn seed_run(owner_pool: &PgPool, org: Uuid, source_label: &str) -> Uuid {
    let run_id = Uuid::new_v4();
    let event = test_audit_event("test.seed_run", "payroll_draft_run", run_id, org);
    let source_label = source_label.to_owned();
    with_audit(owner_pool, event, |tx| {
        Box::pin(async move {
            sqlx::query(
                "INSERT INTO payroll_draft_runs (id, org_id, period_start, period_end, source_label) \
                 VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(run_id)
            .bind(org)
            .bind(date!(2026 - 06 - 01))
            .bind(date!(2026 - 06 - 30))
            .bind(source_label)
            .execute(tx.as_mut())
            .await
            .map_err(DbError::Sqlx)?;
            Ok::<(), DbError>(())
        })
    })
    .await
    .unwrap();
    run_id
}

async fn seed_line(owner_pool: &PgPool, org: Uuid, run_id: Uuid, employee: Uuid, name: &str) {
    let event = test_audit_event("test.seed_line", "payroll_draft_line", employee, org);
    let name = name.to_owned();
    with_audit(owner_pool, event, |tx| {
        Box::pin(async move {
            sqlx::query(
                "INSERT INTO payroll_draft_lines \
                 (org_id, run_id, employee_id, employee_source_key, employee_display_name, employee_company) \
                 VALUES ($1, $2, $3, $4, $5, 'KNL')",
            )
            .bind(org)
            .bind(run_id)
            .bind(employee)
            .bind(format!("src-{employee}"))
            .bind(name)
            .execute(tx.as_mut())
            .await
            .map_err(DbError::Sqlx)?;
            Ok::<(), DbError>(())
        })
    })
    .await
    .unwrap();
}

/// OpenAPI `MyPayrollLine` — hours/source-present booleans, never vault docs
/// and never a computed won amount.
const MY_PAYSLIP_READINESS_ITEM_KEYS: &[&str] = &[
    "calculation_status",
    "gross_pay_source_present",
    "holiday_hours",
    "leave_remaining",
    "leave_used",
    "net_pay_source_present",
    "night_hours",
    "overtime_hours",
    "period_end",
    "period_start",
    "regular_hours",
    "run_id",
    "run_status",
    "work_days",
];

/// Hour/leave counts the DTO may serialize (often JSON null). The seeded
/// INSERT does not populate them, so their absence is allowed.
const MY_PAYSLIP_OPTIONAL_HOUR_KEYS: &[&str] = &[
    "holiday_hours",
    "leave_remaining",
    "leave_used",
    "night_hours",
    "overtime_hours",
    "regular_hours",
    "work_days",
];

const VAULT_OR_ISSUED_PAYSLIP_KEYS: &[&str] = &[
    "kind",
    "legal_basis",
    "confirmed_at",
    "confirmed_by",
    "inbox_doc_id",
    "payload",
    "recipient_user_id",
];

const WON_AMOUNT_KEYS: &[&str] = &[
    "amount_won",
    "deductions",
    "earnings",
    "gross_pay_won",
    "gross_won",
    "net_pay_won",
    "net_won",
    "total_deductions_won",
];

fn object_keys(value: &Value) -> BTreeSet<&str> {
    value
        .as_object()
        .map(|map| map.keys().map(String::as_str).collect())
        .unwrap_or_default()
}

/// Item keys plus one nested object (or array-of-object) level.
fn object_keys_one_level(value: &Value) -> BTreeSet<&str> {
    let mut keys = BTreeSet::new();
    let Some(map) = value.as_object() else {
        return keys;
    };
    for (key, nested) in map {
        keys.insert(key.as_str());
        match nested {
            Value::Object(inner) => {
                keys.extend(inner.keys().map(String::as_str));
            }
            Value::Array(items) => {
                for nested in items {
                    if let Some(inner) = nested.as_object() {
                        keys.extend(inner.keys().map(String::as_str));
                    }
                }
            }
            _ => {}
        }
    }
    keys
}

fn collect_keys<'a>(value: &'a Value, keys: &mut BTreeSet<&'a str>) {
    match value {
        Value::Object(map) => {
            for (key, nested) in map {
                keys.insert(key.as_str());
                collect_keys(nested, keys);
            }
        }
        Value::Array(items) => {
            for nested in items {
                collect_keys(nested, keys);
            }
        }
        _ => {}
    }
}

fn assert_payslips_me_is_own_readiness(body: &Value, expected_run: Uuid, forbidden_ids: &[Uuid]) {
    assert_eq!(
        object_keys(body),
        BTreeSet::from(["items", "limit", "offset", "total"]),
        "page envelope must stay a readiness page, not a vault list: {body:?}"
    );
    assert_eq!(body["total"], 1, "{body:?}");
    let items = body["items"].as_array().expect("items array");
    assert_eq!(items.len(), 1, "{body:?}");
    let item = &items[0];
    let item_keys = object_keys(item);
    let allowed: BTreeSet<&str> = MY_PAYSLIP_READINESS_ITEM_KEYS.iter().copied().collect();
    let unexpected: Vec<&str> = item_keys.difference(&allowed).copied().collect();
    assert!(
        unexpected.is_empty(),
        "unexpected keys on /payslips/me item {unexpected:?}: {item}"
    );
    for key in MY_PAYSLIP_READINESS_ITEM_KEYS {
        if MY_PAYSLIP_OPTIONAL_HOUR_KEYS.contains(key) {
            continue;
        }
        assert!(
            item_keys.contains(key),
            "readiness field {key} must be present on /payslips/me: {item}"
        );
    }
    // Required DTO booleans stay present even when the seeded line has no
    // pay source. Hour keys are optional: the INSERT omits hour columns.
    assert!(
        item.get("gross_pay_source_present")
            .and_then(Value::as_bool)
            .is_some(),
        "gross_pay_source_present must be a boolean: {item}"
    );
    assert!(
        item.get("net_pay_source_present")
            .and_then(Value::as_bool)
            .is_some(),
        "net_pay_source_present must be a boolean: {item}"
    );
    assert_eq!(item["run_id"], expected_run.to_string());
    assert!(
        !item["period_start"].is_null() && !item["period_end"].is_null(),
        "readiness row must carry the run period: {item}"
    );

    // Item keys plus one nested object level: none may contain "won"
    // (`gross_won`, `net_won`, `amount_won`, `total_deductions_won`,
    // `monthly_remuneration_won`, …). This is not the issued 명세서
    // (`GET /api/v1/me/inbox-docs?filter=payslip`).
    for key in object_keys_one_level(item) {
        assert!(
            !key.contains("won"),
            "{key} contains 'won' on /payslips/me item (issued 명세서 is GET /api/v1/me/inbox-docs?filter=payslip): {item}"
        );
    }

    let mut keys = BTreeSet::new();
    collect_keys(body, &mut keys);
    for forbidden in VAULT_OR_ISSUED_PAYSLIP_KEYS
        .iter()
        .chain(WON_AMOUNT_KEYS.iter())
    {
        assert!(
            !keys.contains(forbidden),
            "{forbidden} must not appear on /payslips/me (issued 명세서 is GET /api/v1/me/inbox-docs?filter=payslip): {body:?}"
        );
    }
    for key in &keys {
        assert!(
            !key.contains("won"),
            "{key} contains 'won' on /payslips/me (issued 명세서 is GET /api/v1/me/inbox-docs?filter=payslip): {body:?}"
        );
    }

    let rendered = body.to_string();
    for id in forbidden_ids {
        assert!(
            !rendered.contains(&id.to_string()),
            "must not leak another person's id {id} via /payslips/me: {rendered}"
        );
    }
    assert!(
        !rendered.contains("Alice"),
        "must not leak employee_display_name via /payslips/me: {rendered}"
    );
    assert!(
        !rendered.contains("Bob"),
        "must not leak employee_display_name via /payslips/me: {rendered}"
    );
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn payslips_me_is_self_scoped_never_a_coworkers(pool: PgPool) {
    let keys = keys();
    let org = OrgId::knl();

    let run = seed_run(&pool, *org.as_uuid(), "shared-run").await;
    let alice_employee = seed_employee(&pool, *org.as_uuid(), "Alice").await;
    let bob_employee = seed_employee(&pool, *org.as_uuid(), "Bob").await;
    seed_line(&pool, *org.as_uuid(), run, alice_employee, "Alice").await;
    seed_line(&pool, *org.as_uuid(), run, bob_employee, "Bob").await;

    let alice_user = UserId::new();
    seed_user_linked_to_employee(&pool, alice_user, *org.as_uuid(), alice_employee).await;
    let bob_user = UserId::new();
    seed_user_linked_to_employee(&pool, bob_user, *org.as_uuid(), bob_employee).await;
    let admin_no_link = UserId::new();
    seed_user(&pool, admin_no_link, *org.as_uuid(), "ADMIN").await;

    let service = app(runtime_role_pool(&pool).await, &keys);
    let alice_token = bearer(&keys, alice_user, org, "MEMBER");
    let bob_token = bearer(&keys, bob_user, org, "MEMBER");

    let anon = get_unauthenticated(service.clone(), PAYROLL_MY_PAYSLIPS_PATH).await;
    assert_eq!(anon, StatusCode::UNAUTHORIZED);

    // MEMBER JWT carries empty feature_grants — no PayrollRunRead — and is
    // still 200 on /payslips/me. This is NOT the issued 명세서
    // (`GET /api/v1/me/inbox-docs?filter=payslip`); that vault is a
    // different crate. Admin listing must stay 403; own readiness 200.
    let member_runs = get(service.clone(), PAYROLL_RUNS_PATH, &alice_token).await;
    assert_eq!(
        member_runs.status,
        StatusCode::FORBIDDEN,
        "MEMBER without PayrollRunRead must not list runs: {:?}",
        member_runs.json
    );
    assert_eq!(member_runs.json["error"]["code"], "forbidden");

    let alice_read = get(service.clone(), PAYROLL_MY_PAYSLIPS_PATH, &alice_token).await;
    assert_eq!(alice_read.status, StatusCode::OK, "{:?}", alice_read.json);
    assert_payslips_me_is_own_readiness(
        &alice_read.json,
        run,
        &[bob_employee, *bob_user.as_uuid()],
    );

    // A foreign employee_id query param is ignored, not an IDOR.
    let alice_idor = get(
        service.clone(),
        &format!("{PAYROLL_MY_PAYSLIPS_PATH}?employee_id={bob_employee}"),
        &alice_token,
    )
    .await;
    assert_eq!(alice_idor.status, StatusCode::OK, "{:?}", alice_idor.json);
    assert_payslips_me_is_own_readiness(
        &alice_idor.json,
        run,
        &[bob_employee, *bob_user.as_uuid()],
    );

    let bob_read = get(service.clone(), PAYROLL_MY_PAYSLIPS_PATH, &bob_token).await;
    assert_eq!(bob_read.status, StatusCode::OK, "{:?}", bob_read.json);
    assert_payslips_me_is_own_readiness(
        &bob_read.json,
        run,
        &[alice_employee, *alice_user.as_uuid()],
    );

    // An ADMIN with no employee link reads an empty page, not a 403, and
    // never leaks Alice's or Bob's rows.
    let admin_read = get(
        service,
        PAYROLL_MY_PAYSLIPS_PATH,
        &bearer(&keys, admin_no_link, org, "ADMIN"),
    )
    .await;
    assert_eq!(
        admin_read.status,
        StatusCode::OK,
        "self-service read must not be forbidden: {:?}",
        admin_read.json
    );
    assert_eq!(admin_read.json["total"], 0);
    assert_eq!(
        admin_read.json["items"],
        json!([]),
        "unlinked account must not see anyone's draft lines: {:?}",
        admin_read.json
    );
    let mut admin_keys = BTreeSet::new();
    collect_keys(&admin_read.json, &mut admin_keys);
    for forbidden in VAULT_OR_ISSUED_PAYSLIP_KEYS {
        assert!(
            !admin_keys.contains(forbidden),
            "{forbidden} must not appear on the empty /payslips/me page: {:?}",
            admin_read.json
        );
    }
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn runs_admin_read_is_executive_and_super_admin_only(pool: PgPool) {
    let keys = keys();
    let org = OrgId::knl();
    seed_run(&pool, *org.as_uuid(), "run-1").await;

    let member = UserId::new();
    seed_user(&pool, member, *org.as_uuid(), "MEMBER").await;
    let admin = UserId::new();
    seed_user(&pool, admin, *org.as_uuid(), "ADMIN").await;
    let super_admin = UserId::new();
    seed_user(&pool, super_admin, *org.as_uuid(), "SUPER_ADMIN").await;

    let service = app(runtime_role_pool(&pool).await, &keys);

    let member_read = get(
        service.clone(),
        PAYROLL_RUNS_PATH,
        &bearer(&keys, member, org, "MEMBER"),
    )
    .await;
    assert_eq!(member_read.status, StatusCode::FORBIDDEN);

    // ADMIN's JWT carries no `branches` claim, which resolves to an EMPTY
    // branch scope (not All) — denied same as MEMBER, matching the
    // `authorize_org_wide` built-in-role behavior documented in the crate.
    let admin_read = get(
        service.clone(),
        PAYROLL_RUNS_PATH,
        &bearer(&keys, admin, org, "ADMIN"),
    )
    .await;
    assert_eq!(admin_read.status, StatusCode::FORBIDDEN);

    let super_admin_read = get(
        service,
        PAYROLL_RUNS_PATH,
        &bearer(&keys, super_admin, org, "SUPER_ADMIN"),
    )
    .await;
    assert_eq!(
        super_admin_read.status,
        StatusCode::OK,
        "{:?}",
        super_admin_read.json
    );
    assert_eq!(super_admin_read.json["total"], 1);
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn runs_are_org_isolated_over_http(pool: PgPool) {
    let keys = keys();
    let org = OrgId::knl();
    let other_org = Uuid::from_u128(0x5ea5_5ea5_5ea5_5ea5_5ea5_5ea5_5ea5_5ea5);
    seed_org(&pool, other_org, "OTHER").await;

    seed_run(&pool, *org.as_uuid(), "org-run").await;
    seed_run(&pool, other_org, "other-org-run").await;

    let super_admin = UserId::new();
    seed_user(&pool, super_admin, *org.as_uuid(), "SUPER_ADMIN").await;

    let service = app(runtime_role_pool(&pool).await, &keys);
    let read = get(
        service,
        PAYROLL_RUNS_PATH,
        &bearer(&keys, super_admin, org, "SUPER_ADMIN"),
    )
    .await;
    assert_eq!(read.status, StatusCode::OK, "{:?}", read.json);
    assert_eq!(
        read.json["total"], 1,
        "must see only this org's run, never the other org's"
    );
}
