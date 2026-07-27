#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]
//! Authenticated, runtime-role HTTP contract for the payroll run lifecycle
//! (CAP-PAYROLL-CONSOLE). Crosses the crate's assembled router (the same
//! `with_request_context` middleware the app mounts) on a genuine `console_rt`
//! served pool (RLS enforced), with a real ES256 signature chain, through the
//! full close → calculate → exception → SoD decision → disbursement →
//! release-gated payslip issuance pipeline; asserts PBAC denial without
//! leakage, cross-tenant invisibility, and the audit readback.
//!
//! NOTE: chartered home was `backend/app/tests/payroll_run_api.rs`, but
//! `console-app` does not compile on this branch (facilities/production rest
//! lanes are mid-refactor), so the identical proof runs here against the
//! crate router the app mounts verbatim.

use axum::body::{Body, to_bytes};
use http::{Request, StatusCode, header};
use console_kernel_core::{OrgId, UserId};
use console_payroll_adapter_postgres::PgPayrollStore;
use console_payroll_rest::{PayrollRestState, router};
use console_platform_auth::{AccessTokenInput, JwtIssuer, JwtSettings, JwtVerifier};
use p256::ecdsa::SigningKey;
use p256::elliptic_curve::rand_core::OsRng;
use p256::pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding};
use serde_json::{Value, json};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use time::format_description::well_known::Rfc3339;
use time::macros::date;
use time::{Duration, OffsetDateTime};
use tower::ServiceExt;
use uuid::Uuid;

const ISSUER: &str = "console-platform-auth";
const AUDIENCE: &str = "console-api";
const SHA256_FIXTURE: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn executive_drives_full_lifecycle_with_audit_readback(pool: PgPool) {
    let keys = Keys::generate();
    let rt = runtime_role_pool(&pool).await;
    let org = OrgId::knl();
    seed_org(&pool, org).await;

    let submitter = seed_user(&pool, org, "EXECUTIVE", None).await;
    let decider = seed_user(&pool, org, "EXECUTIVE", None).await;
    let submitter_token = keys.token(submitter, org, "EXECUTIVE");
    let decider_token = keys.token(decider, org, "EXECUTIVE");

    let employee = seed_employee(&pool, org, "Alice").await;
    let _recipient = seed_user(&pool, org, "MEMBER", Some(employee)).await;
    let run = seed_run(&pool, org).await;
    let import_row = seed_verified_import_row(&pool, org).await;
    seed_calculable_line(&pool, org, run, employee, import_row).await;

    // ---- Close preflight: fail-closed while the payroll period lock is
    // missing; the check names itself and close 409s with the receipt.
    let (status, preflight) = send(
        &rt,
        &keys,
        "GET",
        &format!("/api/v1/payroll/runs/{run}/close-preflight"),
        &submitter_token,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{preflight}");
    assert_eq!(preflight["can_close"], false);
    let lock_check = preflight["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|check| check["key"] == "period_lock")
        .expect("period_lock check present");
    assert_eq!(lock_check["ok"], false);

    let (status, blocked) = send(
        &rt,
        &keys,
        "POST",
        &format!("/api/v1/payroll/runs/{run}/close-attendance"),
        &submitter_token,
        Some(json!({"attest": true})),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{blocked}");
    assert_eq!(blocked["error"]["code"], "preflight_blocked");
    assert_eq!(blocked["error"]["details"]["can_close"], false);

    seed_period_lock(&pool, org).await;

    // Attestation is mandatory.
    let (status, unattested) = send(
        &rt,
        &keys,
        "POST",
        &format!("/api/v1/payroll/runs/{run}/close-attendance"),
        &submitter_token,
        Some(json!({"attest": false})),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{unattested}");

    let (status, closed) = send(
        &rt,
        &keys,
        "POST",
        &format!("/api/v1/payroll/runs/{run}/close-attendance"),
        &submitter_token,
        Some(json!({"attest": true})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{closed}");
    assert_eq!(closed["run"]["status"], "ATTENDANCE_CLOSED");
    assert_eq!(
        closed["run"]["close_receipt"]["attested_by"],
        json!(submitter.as_uuid()),
        "close receipt records the attestor"
    );

    // ---- Calculate: real kernel math from the verified source ledger row.
    let (status, calculated) = send(
        &rt,
        &keys,
        "POST",
        &format!("/api/v1/payroll/runs/{run}/calculate"),
        &submitter_token,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{calculated}");
    assert_eq!(calculated["run"]["status"], "CALCULATED");
    let calc = &calculated["calculation"];
    assert_eq!(calc["calculated_lines"], 1);
    assert_eq!(calc["blocked_lines"], 0);
    assert_eq!(
        calc["total_net_won"], 2_626_698,
        "3,000,000 gross with the June 2026 statutory tables + verified NTS row"
    );
    assert_eq!(calc["payable"], false, "draft until the release gate flips");
    assert_eq!(
        calculated["exceptions_open"], 1,
        "overtime hours raise 예외"
    );

    // ---- Submit is fail-closed while exceptions are open.
    let (status, refused) = send(
        &rt,
        &keys,
        "POST",
        &format!("/api/v1/payroll/runs/{run}/submit"),
        &submitter_token,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{refused}");
    assert_eq!(refused["error"]["code"], "exceptions_open");

    let (status, exceptions) = send(
        &rt,
        &keys,
        "GET",
        &format!("/api/v1/payroll/runs/{run}/exceptions"),
        &submitter_token,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{exceptions}");
    assert_eq!(exceptions["open"], 1);
    let exception = &exceptions["items"][0];
    assert_eq!(exception["kind"], "OVERTIME_ALLOWANCE");
    assert_eq!(exception["status"], "OPEN");
    assert!(
        exception["amount_delta_won"].is_null(),
        "no verified source for the delta — must be null, never an estimate"
    );
    let exception_id = exception["id"].as_str().unwrap().to_owned();

    // HOLD requires a reason; CONFIRM resolves; a second resolve conflicts.
    let (status, no_reason) = send(
        &rt,
        &keys,
        "POST",
        &format!("/api/v1/payroll/runs/{run}/exceptions/{exception_id}/resolve"),
        &submitter_token,
        Some(json!({"action": "HOLD"})),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{no_reason}");
    let (status, confirmed) = send(
        &rt,
        &keys,
        "POST",
        &format!("/api/v1/payroll/runs/{run}/exceptions/{exception_id}/resolve"),
        &submitter_token,
        Some(json!({"action": "CONFIRM"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{confirmed}");
    assert_eq!(confirmed["status"], "CONFIRMED");
    let (status, replay) = send(
        &rt,
        &keys,
        "POST",
        &format!("/api/v1/payroll/runs/{run}/exceptions/{exception_id}/resolve"),
        &submitter_token,
        Some(json!({"action": "CONFIRM"})),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{replay}");
    assert_eq!(replay["error"]["code"], "already_resolved");

    // ---- Submit, then SoD: the submitter may not decide.
    let (status, submitted) = send(
        &rt,
        &keys,
        "POST",
        &format!("/api/v1/payroll/runs/{run}/submit"),
        &submitter_token,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{submitted}");
    assert_eq!(submitted["run"]["status"], "SUBMITTED");

    let (status, sod) = send(
        &rt,
        &keys,
        "POST",
        &format!("/api/v1/payroll/runs/{run}/decision"),
        &submitter_token,
        Some(json!({"decision": "APPROVE"})),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{sod}");
    assert_eq!(sod["error"]["code"], "sod_violation");

    // ---- Reject (reason required) → withdraw → resubmit → approve.
    let (status, no_reason) = send(
        &rt,
        &keys,
        "POST",
        &format!("/api/v1/payroll/runs/{run}/decision"),
        &decider_token,
        Some(json!({"decision": "REJECT"})),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{no_reason}");
    let (status, rejected) = send(
        &rt,
        &keys,
        "POST",
        &format!("/api/v1/payroll/runs/{run}/decision"),
        &decider_token,
        Some(json!({"decision": "REJECT", "reason": "지급 계좌 재확인"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{rejected}");
    assert_eq!(rejected["run"]["status"], "REJECTED");

    let (status, withdrawn) = send(
        &rt,
        &keys,
        "POST",
        &format!("/api/v1/payroll/runs/{run}/withdraw"),
        &submitter_token,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{withdrawn}");
    assert_eq!(withdrawn["run"]["status"], "CALCULATED");

    let (status, _) = send(
        &rt,
        &keys,
        "POST",
        &format!("/api/v1/payroll/runs/{run}/submit"),
        &submitter_token,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, approved) = send(
        &rt,
        &keys,
        "POST",
        &format!("/api/v1/payroll/runs/{run}/decision"),
        &decider_token,
        Some(json!({"decision": "APPROVE"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{approved}");
    assert_eq!(approved["run"]["status"], "APPROVED");

    // ---- Disbursement: past date 422; then the operator-attested FSM.
    let past = (OffsetDateTime::now_utc() - Duration::hours(1))
        .format(&Rfc3339)
        .unwrap();
    let (status, _) = send(
        &rt,
        &keys,
        "POST",
        &format!("/api/v1/payroll/runs/{run}/schedule-disbursement"),
        &submitter_token,
        Some(json!({"scheduled_at": past})),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let future = (OffsetDateTime::now_utc() + Duration::days(1))
        .format(&Rfc3339)
        .unwrap();
    let (status, scheduled) = send(
        &rt,
        &keys,
        "POST",
        &format!("/api/v1/payroll/runs/{run}/schedule-disbursement"),
        &submitter_token,
        Some(json!({"scheduled_at": future})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{scheduled}");
    assert_eq!(scheduled["status"], "SCHEDULED");

    // PAID straight from SCHEDULED is an invalid operator attestation.
    let (status, skipped) = send(
        &rt,
        &keys,
        "POST",
        &format!("/api/v1/payroll/runs/{run}/disbursement/attest"),
        &submitter_token,
        Some(json!({"status": "PAID"})),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{skipped}");
    assert_eq!(skipped["error"]["code"], "invalid_transition");
    let (status, _) = send(
        &rt,
        &keys,
        "POST",
        &format!("/api/v1/payroll/runs/{run}/disbursement/attest"),
        &submitter_token,
        Some(json!({"status": "SUBMITTED_TO_BANK"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, paid) = send(
        &rt,
        &keys,
        "POST",
        &format!("/api/v1/payroll/runs/{run}/disbursement/attest"),
        &submitter_token,
        Some(json!({"status": "PAID"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{paid}");
    assert_eq!(paid["status"], "PAID");

    // ---- Payslip issuance is hard-gated by the registered release gate.
    let (status, gate) = send(
        &rt,
        &keys,
        "POST",
        &format!("/api/v1/payroll/runs/{run}/issue-payslips"),
        &submitter_token,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{gate}");
    assert_eq!(gate["error"]["code"], "legal_gate");

    register_release_gate(&pool, run).await;

    let (status, issued) = send(
        &rt,
        &keys,
        "POST",
        &format!("/api/v1/payroll/runs/{run}/issue-payslips"),
        &submitter_token,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{issued}");
    assert_eq!(issued["issued"], 1);
    assert_eq!(issued["acknowledged"], 0);

    // Replay after ISSUED is a typed conflict, and the vault holds exactly
    // one deduped payslip document sourced from this run.
    let (status, replay) = send(
        &rt,
        &keys,
        "POST",
        &format!("/api/v1/payroll/runs/{run}/issue-payslips"),
        &submitter_token,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{replay}");
    let docs: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM inbox_docs WHERE kind = 'payslip' AND source_id = $1",
    )
    .bind(run.to_string())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(docs, 1);

    let (status, delivery) = send(
        &rt,
        &keys,
        "GET",
        &format!("/api/v1/payroll/runs/{run}/payslip-delivery"),
        &decider_token,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{delivery}");
    assert_eq!(delivery["issued"], 1);
    assert_eq!(delivery["items"][0]["acknowledged_at"], Value::Null);

    let (status, final_detail) = send(
        &rt,
        &keys,
        "GET",
        &format!("/api/v1/payroll/runs/{run}"),
        &decider_token,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(final_detail["run"]["status"], "ISSUED");

    // ---- Audit readback: every lifecycle mutation left its event.
    for action in [
        "payroll_run.close",
        "payroll_run.calculate",
        "payroll_run.submit",
        "payroll_run.decide",
        "payroll_run.withdraw",
        "payroll_run.disburse_schedule",
        "payroll_run.disburse_attest",
        "payroll_run.payslip_issue",
    ] {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM audit_events WHERE action = $1 AND (target_id = $2 OR \
             target_type = 'payroll_disbursement')",
        )
        .bind(action)
        .bind(run.to_string())
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(count >= 1, "audit action {action} must be recorded");
    }
    let resolve_audits: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_events WHERE action = 'payroll_run.exception_resolve'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        resolve_audits, 1,
        "exactly the one successful resolve audits"
    );
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn lifecycle_writes_deny_without_leakage_and_cross_tenant_is_invisible(pool: PgPool) {
    let keys = Keys::generate();
    let rt = runtime_role_pool(&pool).await;
    let org_a = OrgId::knl();
    seed_org(&pool, org_a).await;
    let org_b = OrgId::from_uuid(Uuid::new_v4());
    sqlx::query("INSERT INTO organizations (id, slug, name) VALUES ($1, 'org-b', 'Org B')")
        .bind(*org_b.as_uuid())
        .execute(&pool)
        .await
        .unwrap();

    let run = seed_run(&pool, org_a).await;

    // MEMBER and built-in ADMIN (the write tier is EXECUTIVE/SUPER_ADMIN or a
    // custom org-wide PBAC grant): 403, and the same 403 whether or not the
    // run exists — no existence oracle.
    for role in ["MEMBER", "ADMIN"] {
        let user = seed_user(&pool, org_a, role, None).await;
        let token = keys.token(user, org_a, role);
        for target in [run, Uuid::new_v4()] {
            let (status, denied) = send(
                &rt,
                &keys,
                "POST",
                &format!("/api/v1/payroll/runs/{target}/calculate"),
                &token,
                None,
            )
            .await;
            assert_eq!(status, StatusCode::FORBIDDEN, "{role}: {denied}");
            assert_eq!(denied["error"]["code"], "forbidden");
        }
    }

    // An org-B EXECUTIVE: org A's run is indistinguishable from a missing one
    // on both the read and the write path (RLS deny-by-omission).
    let outsider = seed_user(&pool, org_b, "EXECUTIVE", None).await;
    let outsider_token = keys.token(outsider, org_b, "EXECUTIVE");
    for (method, uri) in [
        ("GET", format!("/api/v1/payroll/runs/{run}")),
        ("GET", format!("/api/v1/payroll/runs/{run}/close-preflight")),
        ("GET", format!("/api/v1/payroll/runs/{run}/exceptions")),
        ("POST", format!("/api/v1/payroll/runs/{run}/calculate")),
        ("POST", format!("/api/v1/payroll/runs/{run}/submit")),
    ] {
        let (status, body) = send(&rt, &keys, method, &uri, &outsider_token, None).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{method} {uri}: {body}");
        assert_eq!(body["error"]["code"], "not_found");
    }

    // The denied probes must not have advanced the run.
    let status: String = sqlx::query_scalar("SELECT status FROM payroll_draft_runs WHERE id = $1")
        .bind(run)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "STAGED");
    let probe_audits: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_events WHERE action LIKE 'payroll_run.%' AND target_id = $1",
    )
    .bind(run.to_string())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        probe_audits, 0,
        "denied probes must not commit audit events"
    );
}

// ---------------------------------------------------------------------------
// Harness
// ---------------------------------------------------------------------------

struct Keys {
    private_pem: String,
    public_pem: String,
}

impl Keys {
    fn generate() -> Self {
        let key = SigningKey::random(&mut OsRng);
        Self {
            private_pem: key.to_pkcs8_pem(LineEnding::LF).unwrap().to_string(),
            public_pem: key
                .verifying_key()
                .to_public_key_pem(LineEnding::LF)
                .unwrap(),
        }
    }

    fn token(&self, user: UserId, org: OrgId, role: &str) -> String {
        JwtIssuer::from_es256_pem(
            JwtSettings {
                issuer: ISSUER.into(),
                audience: AUDIENCE.into(),
                access_token_ttl: Duration::minutes(15),
            },
            self.private_pem.as_bytes(),
            self.public_pem.as_bytes(),
        )
        .unwrap()
        .issue_access_token(AccessTokenInput {
            subject: user,
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
}

async fn runtime_role_pool(owner: &PgPool) -> PgPool {
    PgPoolOptions::new()
        .max_connections(8)
        .after_connect(|conn, _| {
            Box::pin(async move {
                sqlx::query("SET ROLE console_rt").execute(conn).await?;
                Ok(())
            })
        })
        .connect_with(owner.connect_options().as_ref().clone())
        .await
        .unwrap()
}

fn app(pool: PgPool, public_key: &str) -> axum::Router {
    let verifier = JwtVerifier::from_es256_public_pem(
        JwtSettings {
            issuer: ISSUER.into(),
            audience: AUDIENCE.into(),
            access_token_ttl: Duration::minutes(15),
        },
        public_key.as_bytes(),
    )
    .unwrap();
    router(PayrollRestState::new(
        PgPayrollStore::new(pool),
        Some(verifier),
    ))
}

async fn send(
    pool: &PgPool,
    keys: &Keys,
    method: &str,
    uri: &str,
    token: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let request = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(
            body.map(|v| Body::from(serde_json::to_vec(&v).unwrap()))
                .unwrap_or_else(Body::empty),
        )
        .unwrap();
    let response = app(pool.clone(), &keys.public_pem)
        .oneshot(request)
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    };
    (status, json)
}

async fn seed_org(pool: &PgPool, org: OrgId) {
    sqlx::query(
        "INSERT INTO organizations (id, slug, name) VALUES ($1, 'knl', 'KNL') \
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(*org.as_uuid())
    .execute(pool)
    .await
    .unwrap();
}

async fn seed_user(pool: &PgPool, org: OrgId, role: &str, employee: Option<Uuid>) -> UserId {
    let user = UserId::new();
    sqlx::query(
        "INSERT INTO users (id, display_name, roles, is_active, org_id, employee_id) \
         VALUES ($1, $2, $3, true, $4, $5)",
    )
    .bind(*user.as_uuid())
    .bind(format!("payroll-{role}-{user}"))
    .bind(vec![role.to_owned()])
    .bind(*org.as_uuid())
    .bind(employee)
    .execute(pool)
    .await
    .unwrap();
    user
}

async fn seed_employee(pool: &PgPool, org: OrgId, name: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO employees \
         (id, org_id, company, name, source_filename, source_sheet, source_row, source_key) \
         VALUES ($1, $2, 'KNL', $3, 'roster.xlsx', 'Sheet1', 1, $4)",
    )
    .bind(id)
    .bind(*org.as_uuid())
    .bind(name)
    .bind(format!("emp-{id}"))
    .execute(pool)
    .await
    .unwrap();
    id
}

async fn seed_run(pool: &PgPool, org: OrgId) -> Uuid {
    sqlx::query_scalar(
        "INSERT INTO payroll_draft_runs (org_id, period_start, period_end, source_label, status) \
         VALUES ($1, $2, $3, '2026-06 정기급여', 'STAGED') RETURNING id",
    )
    .bind(*org.as_uuid())
    .bind(date!(2026 - 06 - 01))
    .bind(date!(2026 - 06 - 30))
    .fetch_one(pool)
    .await
    .unwrap()
}

/// One DRY_RUN import run + row whose `canonical_row.payroll` carries the
/// verified figures the calculate step reads verbatim.
async fn seed_verified_import_row(pool: &PgPool, org: OrgId) -> Uuid {
    let import_run: Uuid = sqlx::query_scalar(
        "INSERT INTO data_import_runs \
         (org_id, entity_type, status, source_filename, source_format, source_sha256) \
         VALUES ($1, 'employee_hr', 'DRY_RUN', 'payroll-source.xlsx', 'xlsx', $2) RETURNING id",
    )
    .bind(*org.as_uuid())
    .bind(SHA256_FIXTURE)
    .fetch_one(pool)
    .await
    .unwrap();
    sqlx::query_scalar(
        "INSERT INTO data_import_rows \
         (org_id, run_id, source_sheet, source_row, source_key, row_status, canonical_row) \
         VALUES ($1, $2, '급여', 2, 'alice-2026-06', 'CANDIDATE', $3) RETURNING id",
    )
    .bind(*org.as_uuid())
    .bind(import_run)
    .bind(json!({
        "payroll": {
            "monthly_gross_pay_won": 3_000_000,
            "nts_tax_row": {
                "table_version": "NTS-간이세액표-fixture-row-v1",
                "monthly_income_tax_won": 74_350,
                "local_income_tax_won": 7_430,
            },
        },
    }))
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn seed_calculable_line(
    pool: &PgPool,
    org: OrgId,
    run: Uuid,
    employee: Uuid,
    import_row: Uuid,
) {
    sqlx::query(
        "INSERT INTO payroll_draft_lines \
         (org_id, run_id, employee_id, employee_source_key, employee_display_name, \
          employee_company, attendance_source_row_count, overtime_hours, \
          gross_pay_source_present, nts_tax_row_status, source_data_import_row_ids) \
         VALUES ($1, $2, $3, $4, 'Alice', 'KNL', 1, 2.5, true, 'VERIFIED_SOURCE_ROW', $5)",
    )
    .bind(*org.as_uuid())
    .bind(run)
    .bind(employee)
    .bind(format!("src-{employee}"))
    .bind(vec![import_row])
    .execute(pool)
    .await
    .unwrap();
}

async fn seed_period_lock(pool: &PgPool, org: OrgId) {
    sqlx::query(
        "INSERT INTO period_locks (org_id, domain, period_start, period_end, reason) \
         VALUES ($1, 'payroll', $2, $3, '2026-06 급여 동결창')",
    )
    .bind(*org.as_uuid())
    .bind(date!(2026 - 06 - 01))
    .bind(date!(2026 - 06 - 30))
    .execute(pool)
    .await
    .unwrap();
}

/// Register a release-gate record that satisfies
/// `console_payroll_domain::validate_release_gate` on the run's `legal_basis`.
async fn register_release_gate(pool: &PgPool, run: Uuid) {
    sqlx::query("UPDATE payroll_draft_runs SET legal_basis = legal_basis || $2 WHERE id = $1")
        .bind(run)
        .bind(json!({
            "release_gate": {
                "rate_table_version": "statutory-rates-2026-06-27",
                "official_source_urls": [
                    "https://www.nps.or.kr/pnsinfo/ntpsklg/getOHAF0038M0.do",
                ],
                "golden_cases": [{
                    "case_id": "GC-2026-06-A",
                    "rate_table_version": "statutory-rates-2026-06-27",
                    "professionally_validated": true,
                    "expected_total_employee_deductions_won": 373_302,
                }],
                "professional_validation": {
                    "reviewer_kind": "labor_attorney",
                    "reviewed_on": "2026-07-01",
                    "artifact_sha256": SHA256_FIXTURE,
                    "reviewer_reference": "노무법인 검증 2026-07",
                },
            },
        }))
        .execute(pool)
        .await
        .unwrap();
}
