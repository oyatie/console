#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! `PayRunPort` proven against a REAL PostgreSQL as the genuine runtime role
//! `console_rt` — never the BYPASSRLS superuser the `#[sqlx::test]` pool
//! connects as, which sees every row and would green-light a broken
//! `org_isolation` policy.
//! `a_foreign_tenant_is_invisible_and_unwritable_to_the_runtime_role` is what
//! makes that claim observable: it is the only test that crosses a tenant
//! boundary, it asserts through the OWNER pool that the foreign rows genuinely
//! EXIST first, and it dies when `org_isolation` is loosened to `USING (true)`.
//!
//! WHY `#[sqlx::test]` IS NOT OPTIONAL HERE. Migration 0196 refuses a superuser
//! applier unless `CURRENT_DATABASE()` matches `^_sqlx_test_[A-Za-z0-9_]{52}$`
//! with the `console.sqlx_test_bootstrap` marker set, so the schema itself
//! admits exactly one applier and a hand-rolled `sqlx::migrate!` harness is not
//! a design choice that was available.
//!
//! WHY `execute` IS CALLED FROM `spawn_blocking`. `CanonicalPort::execute` is
//! SYNCHRONOUS — `canonical-domain` declares it so and this lane may not edit
//! that crate — so the adapter bridges to `sqlx` with `Handle::block_on`, which
//! panics inside an async context. A `spawn_blocking` thread is not one.
//!
//! WHAT MAKES `submit`/`decide` MEANINGFUL RATHER THAN CEREMONIAL. The contract
//! says `PayRunPort` WRAPS the existing payroll writer instead of adding a
//! second one, so those two targets call `lifecycle::submit_run_in_tx` and
//! `lifecycle::decide_run_in_tx` — statements this crate already owned. The
//! tests below drive the port and then assert the columns THOSE statements
//! write (`submitted_by`, `decided_by`, `decision_reason`, `approved_at`), and
//! `a_decider_who_submitted_the_run_is_refused` shows the pre-existing
//! segregation-of-duties check still firing through the port. A port that had
//! quietly restated the SQL would have to reproduce all of that by accident.

use console_kernel_core::{OrgId, UserId};
use console_ontology_canonical_adapter_postgres::company::{
    CompanyCommand, CompanyQuery, PgCompanyPort,
};
use console_ontology_canonical_adapter_postgres::employment::{
    EmploymentAttributes, EmploymentCommand, EmploymentQuery, NewEmployeeRecord, PgEmploymentPort,
    insert_employee_record,
};
use console_ontology_canonical_adapter_postgres::job_position::{
    JobPositionCommand, JobPositionQuery, PgJobPositionPort,
};
use console_ontology_canonical_adapter_postgres::org_unit::{
    OrgUnitCommand, OrgUnitQuery, PgOrgUnitPort,
};
use console_ontology_canonical_adapter_postgres::person::{
    PersonCommand, PersonQuery, PgPersonPort,
};
use console_ontology_canonical_domain::{
    CanonicalPort, CommandId, CommandReceipt, DispatchTarget, ObjectKey, PayRunPort, ReceiptOwner,
};
use console_payroll_adapter_postgres::lifecycle::LifecycleError;
use console_payroll_adapter_postgres::pay_run::{
    PayRunCommand, PayRunError, PayRunQuery, PgPayRunPort, StageDraftError, stage_draft_run_in_tx,
    stage_draft_run_returning_id_in_tx,
};
use console_platform_test_support::{runtime_role_pool, seed_org_and_super_admin};
use console_workflow_domain::{PayrollDraftStaging, StagePayrollDraft};
use serde_json::json;
use sqlx::{PgPool, Row};
use time::macros::date;
use time::{OffsetDateTime, Time};
use uuid::Uuid;

const ORG: Uuid = Uuid::from_u128(0xe3b0_0000_0000_0000_0000_0000_0000_0011);
const FOREIGN_ORG: Uuid = Uuid::from_u128(0xe3b0_0000_0000_0000_0000_0000_0000_0012);

/// The port must satisfy the NAMED trait, not merely `CanonicalPort`. The
/// blanket impl in `canonical-domain` makes `PayRunPort` an alias for
/// `CanonicalPort<Object = PayRun>`, so this bound stops holding the moment the
/// adapter is retargeted at a different object.
fn assert_implements_pay_run_port<P: PayRunPort>() {}

async fn seed_org_and_user(owner_pool: &PgPool, org: Uuid, tag: &str) -> UserId {
    sqlx::query(
        "INSERT INTO organizations (id, slug, name) VALUES ($1, $2, $3) \
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(org)
    .bind(format!("org-{tag}"))
    .bind(format!("Org {tag}"))
    .execute(owner_pool)
    .await
    .unwrap();
    let user_id = UserId::new();
    sqlx::query("INSERT INTO users (id, display_name, roles, org_id) VALUES ($1, $2, $3, $4)")
        .bind(*user_id.as_uuid())
        .bind(format!("User {tag}"))
        .bind(["SUPER_ADMIN"].as_slice())
        .bind(org)
        .execute(owner_pool)
        .await
        .unwrap();
    user_id
}

/// The tenant, two distinct actors (segregation of duties needs two), and the
/// port built on a `console_rt` pool.
async fn fixture(owner_pool: &PgPool) -> (OrgId, UserId, UserId, PgPayRunPort) {
    let submitter = seed_org_and_user(owner_pool, ORG, "payrun").await;
    let decider = seed_org_and_user(owner_pool, ORG, "payrun2").await;
    let runtime_pool = runtime_role_pool(owner_pool).await;
    let port = PgPayRunPort::new(runtime_pool, tokio::runtime::Handle::current());
    (OrgId::from_uuid(ORG), submitter, decider, port)
}

/// Drive the SYNCHRONOUS `execute` off the runtime's worker thread.
async fn execute(
    port: &PgPayRunPort,
    command: PayRunCommand,
) -> Result<CommandReceipt, PayRunError> {
    let port = port.clone();
    tokio::task::spawn_blocking(move || port.execute(&command))
        .await
        .unwrap()
}

fn command(org: OrgId, actor: UserId, query: PayRunQuery) -> PayRunCommand {
    PayRunCommand {
        org_id: org,
        command_id: CommandId::from_uuid(Uuid::new_v4()),
        actor_id: actor,
        query,
        action_key: "revise".to_owned(),
        object_type_id: Uuid::nil(),
    }
}

fn create(run_id: Uuid) -> PayRunQuery {
    PayRunQuery::CreateRun {
        run_id,
        period_start: date!(2026 - 06 - 01),
        period_end: date!(2026 - 06 - 30),
        connector: Some("m2".to_owned()),
        job: Some("payroll_draft".to_owned()),
    }
}

/// The row the natural key resolves to, read through the OWNER pool so a test
/// asserting "it landed" is never satisfied by RLS hiding the answer.
async fn run_by_label(owner_pool: &PgPool, run_id: Uuid) -> Option<(Uuid, String, String)> {
    sqlx::query("SELECT id, status, source_label FROM payroll_draft_runs WHERE source_label = $1")
        .bind(format!("workflow_runtime_m2:run:{run_id}"))
        .fetch_optional(owner_pool)
        .await
        .unwrap()
        .map(|row| (row.get("id"), row.get("status"), row.get("source_label")))
}

async fn count(owner_pool: &PgPool, sql: &'static str, org: Uuid) -> i64 {
    sqlx::query_scalar(sql)
        .bind(org)
        .fetch_one(owner_pool)
        .await
        .unwrap()
}

const COUNT_RUNS: &str = "SELECT count(*)::bigint FROM payroll_draft_runs WHERE org_id = $1";

mod complete_roster_submission_tests {
    use super::*;
    use console_attendance_adapter_postgres::PgAttendanceStore;
    use console_attendance_application::{CallerScope, CloseMonth};
    use console_kernel_core::ErrorKind;
    use console_ontology_canonical_domain::CanonicalPortError;
    use console_payroll_adapter_postgres::lifecycle::{
        calculate_run_in_tx, close_attendance_in_tx,
    };
    use console_platform_db::with_org_conn;

    // Deliberate lower-boundary fixture: production employee import does not
    // write this payroll object or verify these figures. Neither this fixture
    // nor its successful control establishes native source entry, daily work
    // coverage, minimum-wage applicability or qualified Korean payroll math.
    async fn calculated_roster(
        owner: &PgPool,
        complete_bob_source: bool,
    ) -> (OrgId, UserId, PgPayRunPort, Uuid) {
        let (org, submitter, decider, port) = fixture(owner).await;
        assert_ne!(submitter, decider);
        let import_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO data_import_runs \
             (id, org_id, entity_type, status, source_filename, source_format, \
              source_sha256, pay_period_start, pay_period_end) \
             VALUES ($1, $2, 'employee_hr', 'DRY_RUN', 'roster.xlsx', 'xlsx', \
                     repeat('a', 64), DATE '2026-06-01', DATE '2026-06-30')",
        )
        .bind(import_id)
        .bind(ORG)
        .execute(owner)
        .await
        .unwrap();
        let payroll_source = json!({
            "monthly_gross_pay_won": 3_000_000,
            "nts_tax_row": {
                "table_version": "NTS-간이세액표-fixture-row-v1",
                "monthly_income_tax_won": 74_350,
                "local_income_tax_won": 7_430,
            },
        });
        let mut expected_sources = Vec::new();
        for (source_row, source_key, name) in [
            (1_i32, "alice-2026-06", "Alice"),
            (2_i32, "bob-2026-06", "Bob"),
        ] {
            let employee_id: Uuid = sqlx::query_scalar(
                "INSERT INTO employees \
                 (org_id, company, name, source_filename, source_sheet, source_row, source_key) \
                 VALUES ($1, 'KNL', $2, 'roster.xlsx', 's', $3, $4) RETURNING id",
            )
            .bind(ORG)
            .bind(name)
            .bind(source_row)
            .bind(source_key)
            .fetch_one(owner)
            .await
            .unwrap();
            let mut canonical = json!({"source_key": source_key});
            if source_row == 1 || complete_bob_source {
                canonical["payroll"] = payroll_source.clone();
            }
            let row_id: Uuid = sqlx::query_scalar(
                "INSERT INTO data_import_rows \
                 (org_id, run_id, source_sheet, source_row, source_key, row_status, \
                  raw_row, canonical_row) \
                 VALUES ($1, $2, 's', $3, $4, 'CANDIDATE', $5, $6) RETURNING id",
            )
            .bind(ORG)
            .bind(import_id)
            .bind(source_row)
            .bind(format!("filename:roster.xlsx|sheet:s|row:{source_row}"))
            .bind(json!({"기본급": "3000000", "소득세": "74350", "근무시간": "209"}))
            .bind(canonical)
            .fetch_one(owner)
            .await
            .unwrap();
            expected_sources.push((employee_id, source_key, row_id));
        }
        assert_ne!(expected_sources[0].0, expected_sources[1].0);
        assert_ne!(expected_sources[0].2, expected_sources[1].2);

        // Satisfy 0166's real writer/audit guards; an APPLIED INSERT or an
        // unarmed UPDATE would either be rejected or silently change no row.
        let mut tx = owner.begin().await.unwrap();
        sqlx::query("SET LOCAL ROLE console_leave_definer")
            .execute(&mut *tx)
            .await
            .unwrap();
        sqlx::query("SELECT set_config('app.current_org', $1, true)")
            .bind(ORG.to_string())
            .execute(&mut *tx)
            .await
            .unwrap();
        let applied = sqlx::query(
            "UPDATE data_import_runs \
             SET status = 'APPLIED', applied_by = $3, applied_at = now(), updated_at = now() \
             WHERE org_id = $1 AND id = $2",
        )
        .bind(ORG)
        .bind(import_id)
        .bind(*submitter.as_uuid())
        .execute(&mut *tx)
        .await
        .unwrap();
        assert_eq!(applied.rows_affected(), 1);
        sqlx::query(
            "INSERT INTO audit_events \
             (actor, action, target_type, target_id, before_snap, after_snap, \
              trace_id, span_id, occurred_at, org_id) \
             VALUES ($1, 'data_import.apply', 'data_import_run', $2, NULL, '{}'::jsonb, \
                     '0123456789abcdef0123456789abcdef', '0123456789abcdef', now(), $3)",
        )
        .bind(*submitter.as_uuid())
        .bind(import_id.to_string())
        .bind(ORG)
        .execute(&mut *tx)
        .await
        .unwrap();
        tx.commit().await.unwrap();
        let applied: bool = sqlx::query_scalar(
            "SELECT r.status = 'APPLIED' AND r.applied_by = $2 AND r.applied_at IS NOT NULL \
                    AND (SELECT count(*) FROM audit_events a \
                         WHERE a.org_id = r.org_id AND a.target_id = r.id::text \
                           AND a.action = 'data_import.apply' AND a.actor = r.applied_by) = 1 \
             FROM data_import_runs r WHERE r.id = $1",
        )
        .bind(import_id)
        .bind(*submitter.as_uuid())
        .fetch_one(owner)
        .await
        .unwrap();
        assert!(applied, "the guarded import must commit before CreateRun");

        let created = execute(&port, command(org, submitter, create(Uuid::new_v4())))
            .await
            .expect("CreateRun must materialize the real two-line roster as console_rt");
        assert_eq!(created.target(), DispatchTarget::PayrollCreateRun);
        let run_id: Uuid = created.result()["draft_run_id"]
            .as_str()
            .expect("CreateRun must return its successor's run identity")
            .parse()
            .unwrap();
        let lines = sqlx::query(
            "SELECT id, employee_id, employee_source_key, source_data_import_row_ids, \
                    payroll_source_row_count, attendance_source_row_count, \
                    gross_pay_source_present, nts_tax_row_status \
             FROM payroll_draft_lines WHERE run_id = $1 ORDER BY employee_source_key",
        )
        .bind(run_id)
        .fetch_all(owner)
        .await
        .unwrap();
        assert_eq!(lines.len(), 2);
        for (line, (employee_id, source_key, row_id)) in lines.iter().zip(&expected_sources) {
            assert_eq!(line.get::<Uuid, _>("employee_id"), *employee_id);
            assert_eq!(line.get::<String, _>("employee_source_key"), *source_key);
            assert_eq!(
                line.get::<Vec<Uuid>, _>("source_data_import_row_ids"),
                vec![*row_id]
            );
            assert_eq!(line.get::<i32, _>("payroll_source_row_count"), 1);
            assert_eq!(line.get::<i32, _>("attendance_source_row_count"), 1);
            assert!(line.get::<bool, _>("gross_pay_source_present"));
            assert_eq!(
                line.get::<String, _>("nts_tax_row_status"),
                "REQUIRED_NOT_SUPPLIED"
            );
        }
        // Fixture-only owner override reaches the missing-amounts branch even
        // though Bob's gross/readiness flags are true. It is not verification.
        let marked = sqlx::query(
            "UPDATE payroll_draft_lines SET nts_tax_row_status = 'VERIFIED_SOURCE_ROW' \
             WHERE run_id = $1",
        )
        .bind(run_id)
        .execute(owner)
        .await
        .unwrap();
        assert_eq!(marked.rows_affected(), 2);

        let runtime_pool = runtime_role_pool(owner).await;
        let attendance = PgAttendanceStore::new(runtime_pool.clone());
        let close = attendance
            .close_month(
                &CallerScope {
                    org_id: ORG,
                    user_id: *submitter.as_uuid(),
                    branch_ids: vec![],
                    org_wide: true,
                },
                CloseMonth {
                    month: "2026-06".to_owned(),
                    branch_scope: None,
                    attest: true,
                },
            )
            .await
            .expect("Attendance owner must create the org-wide June close and payroll lock");
        assert_eq!(close.month, date!(2026 - 06 - 01));
        assert_eq!(close.branch_id, None);
        let lock_id = close
            .period_lock_id
            .expect("org-wide close must own a payroll lock");
        let close_and_lock: bool = sqlx::query_scalar(
            "SELECT c.org_id = $1 AND c.month = DATE '2026-06-01' AND c.branch_id IS NULL \
                    AND c.checks = '{\"open_exceptions\":0,\"pending_leave\":0,\"already_closed\":false}'::jsonb \
                    AND l.id = $3 AND l.org_id = c.org_id AND l.domain = 'payroll' \
                    AND l.period_start = DATE '2026-06-01' AND l.period_end = DATE '2026-06-30' \
                    AND l.unlocked_at IS NULL \
             FROM attendance_month_closes c JOIN period_locks l ON l.id = c.period_lock_id \
             WHERE c.id = $2",
        )
        .bind(ORG)
        .bind(close.id)
        .bind(lock_id)
        .fetch_one(owner)
        .await
        .unwrap();
        assert!(
            close_and_lock,
            "read back the committed Attendance-owned close/lock"
        );
        let outcome = with_org_conn::<_, _, LifecycleError>(&runtime_pool, org, move |tx| {
            Box::pin(async move {
                close_attendance_in_tx(tx, run_id, *submitter.as_uuid(), OffsetDateTime::now_utc())
                    .await?;
                calculate_run_in_tx(tx, run_id).await
            })
        })
        .await
        .expect("the payroll owner must calculate, without a synthetic CALCULATED state");
        assert_eq!(outcome.version, 1);
        assert_eq!(
            outcome.calculated_lines,
            if complete_bob_source { 2 } else { 1 }
        );
        assert_eq!(
            outcome.blocked_lines,
            if complete_bob_source { 0 } else { 1 }
        );
        assert_eq!(outcome.exceptions_created, 0);
        let calculations: Vec<(Uuid, i32, bool)> = sqlx::query_as(
            "SELECT line_id, version, payable FROM payroll_line_calculations \
             WHERE run_id = $1 ORDER BY line_id",
        )
        .bind(run_id)
        .fetch_all(owner)
        .await
        .unwrap();
        assert_eq!(calculations.len(), if complete_bob_source { 2 } else { 1 });
        assert!(
            calculations
                .iter()
                .all(|(_, version, payable)| *version == 1 && !payable)
        );
        for (index, line) in lines.iter().enumerate() {
            let line_id = line.get::<Uuid, _>("id");
            let should_calculate = index == 0 || complete_bob_source;
            assert_eq!(
                calculations
                    .iter()
                    .filter(|(id, _, _)| *id == line_id)
                    .count(),
                usize::from(should_calculate)
            );
            let (status, blockers): (String, serde_json::Value) = sqlx::query_as(
                "SELECT calculation_status, blockers FROM payroll_draft_lines WHERE id = $1",
            )
            .bind(line_id)
            .fetch_one(owner)
            .await
            .unwrap();
            assert_eq!(
                status,
                if should_calculate {
                    "READY_FOR_REVIEW"
                } else {
                    "BLOCKED_LEGAL_GATE"
                }
            );
            assert_eq!(
                blockers,
                if should_calculate {
                    json!([])
                } else {
                    json!(["SOURCE_AMOUNTS_NOT_MATERIALIZED"])
                }
            );
        }
        (org, submitter, port, run_id)
    }

    // One owner snapshot captures every stored run/roster/calculation byte and
    // all receipt/approval consumption records. Denial logs are not success
    // audits; the existing canonical and REST success action names are counted.
    async fn snapshot(owner: &PgPool, run_id: Uuid) -> String {
        sqlx::query_scalar(
            "SELECT jsonb_build_object( \
               'run', (SELECT to_jsonb(r) FROM payroll_draft_runs r WHERE r.id = $1), \
               'lines', (SELECT coalesce(jsonb_agg(to_jsonb(l) ORDER BY l.id), '[]'::jsonb) \
                         FROM payroll_draft_lines l WHERE l.run_id = $1), \
               'calculations', (SELECT coalesce(jsonb_agg(to_jsonb(c) ORDER BY c.id), '[]'::jsonb) \
                                FROM payroll_line_calculations c WHERE c.run_id = $1), \
               'receipts', (SELECT coalesce(jsonb_agg(to_jsonb(r) ORDER BY r.command_id), '[]'::jsonb) \
                            FROM ont_action_command_receipts r WHERE r.org_id = $2), \
               'success_audits', (SELECT count(*) FROM audit_events \
                                  WHERE org_id = $2 AND action IN ('payroll.submit_run', 'payroll_run.submit')), \
               'approval_consumptions', (SELECT coalesce(jsonb_agg(to_jsonb(c) ORDER BY c.id), '[]'::jsonb) \
                                          FROM gov_approval_consumptions c WHERE c.org_id = $2), \
               'exception_count', (SELECT count(*) FROM payroll_run_exceptions WHERE run_id = $1) \
             )::text",
        )
        .bind(run_id)
        .bind(ORG)
        .fetch_one(owner)
        .await
        .unwrap()
    }

    #[sqlx::test(migrations = "../../platform/db/migrations")]
    async fn partial_calculated_roster_cannot_submit_as_runtime_role(owner: PgPool) {
        let (org, submitter, port, run_id) = calculated_roster(&owner, false).await;
        let before = snapshot(&owner, run_id).await;
        let facts: serde_json::Value = serde_json::from_str(&before).unwrap();
        assert_eq!(facts["run"]["status"], "CALCULATED");
        assert!(facts["run"]["submitted_by"].is_null());
        assert!(facts["run"]["submitted_at"].is_null());
        assert_eq!(facts["receipts"].as_array().unwrap().len(), 1);
        assert_eq!(facts["success_audits"], 0);
        assert_eq!(facts["approval_consumptions"], json!([]));
        assert_eq!(
            facts["exception_count"], 0,
            "no exception may mask the source guard"
        );
        let submit = command(org, submitter, PayRunQuery::SubmitRun { run_id });
        for attempt in 1..=2 {
            let error = execute(&port, submit.clone())
                .await
                .expect_err("PARTIAL_CALCULATED_ROSTER_SUBMITTED: Bob has no calculation and a named source blocker");
            match error {
                PayRunError::Blocked(blockers) => assert!(!blockers.is_empty()),
                error => assert_eq!(error.into_kernel_error().kind, ErrorKind::Conflict),
            }
            assert_eq!(
                snapshot(&owner, run_id).await,
                before,
                "refused submit attempt {attempt} must preserve all roster/calculation bytes and mint no receipt, success audit or approval consumption",
            );
        }
    }

    #[sqlx::test(migrations = "../../platform/db/migrations")]
    async fn complete_calculated_roster_can_submit_as_runtime_role(owner: PgPool) {
        let (org, submitter, port, run_id) = calculated_roster(&owner, true).await;
        let before: serde_json::Value =
            serde_json::from_str(&snapshot(&owner, run_id).await).unwrap();
        assert_eq!(before["run"]["status"], "CALCULATED");
        let submit = command(org, submitter, PayRunQuery::SubmitRun { run_id });
        let receipt = execute(&port, submit.clone())
            .await
            .expect("the source guard must permit a complete two-line calculated roster");
        assert_eq!(receipt.target(), DispatchTarget::PayrollSubmitRun);
        let committed = snapshot(&owner, run_id).await;
        let after: serde_json::Value = serde_json::from_str(&committed).unwrap();
        assert_eq!(after["run"]["status"], "SUBMITTED");
        assert_eq!(after["run"]["submitted_by"], json!(submitter.as_uuid()));
        assert!(after["run"]["submitted_at"].is_string());
        assert_eq!(after["lines"], before["lines"]);
        assert_eq!(after["calculations"], before["calculations"]);
        assert_eq!(after["receipts"].as_array().unwrap().len(), 2);
        assert_eq!(
            after["approval_consumptions"],
            before["approval_consumptions"]
        );
        let replay = execute(&port, submit)
            .await
            .expect("same submit command must replay");
        assert_eq!(replay.result(), receipt.result());
        assert_eq!(snapshot(&owner, run_id).await, committed);
    }
}

/// Move a staged run to `CALCULATED` as the table owner. Submitting requires it,
/// and the port deliberately does NOT own a "calculate" target — `calculate_run_in_tx`
/// already exists in this crate and is not one of the contract's three PayRun
/// dispatch targets.
async fn mark_calculated(owner_pool: &PgPool, id: Uuid) {
    sqlx::query("UPDATE payroll_draft_runs SET status = 'CALCULATED' WHERE id = $1")
        .bind(id)
        .execute(owner_pool)
        .await
        .unwrap();
}

// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn the_contract_identity_is_copied_verbatim_and_the_port_is_the_named_one(
    owner_pool: PgPool,
) {
    assert_implements_pay_run_port::<PgPayRunPort>();

    // The six tables, verbatim from `canonical-domain`. All six already exist
    // (0074 and 0186); this lane created none of them, which is why the check is
    // that the DATABASE holds each one rather than that a migration added it.
    let expected = [
        "payroll_draft_runs",
        "payroll_draft_lines",
        "payroll_line_calculations",
        "payroll_run_exceptions",
        "payroll_disbursements",
        "payroll_payslip_deliveries",
    ];
    assert_eq!(
        ObjectKey::PayRun.owned_tables(),
        expected,
        "the contract's PayRun table list moved; this suite is written against it"
    );
    assert_eq!(
        ObjectKey::PayRun.owner_crate(),
        "console-payroll-adapter-postgres",
        "this crate is the contract's named owner"
    );
    for table in expected {
        let present: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM information_schema.tables \
             WHERE table_schema = 'public' AND table_name = $1)",
        )
        .bind(table)
        .fetch_one(&owner_pool)
        .await
        .unwrap();
        assert!(
            present,
            "{table} must already exist — this lane adds no migration"
        );
    }

    // The three dispatch targets, each bound to PayRun by the contract.
    for target in [
        DispatchTarget::PayrollCreateRun,
        DispatchTarget::PayrollSubmitRun,
        DispatchTarget::PayrollDecideRun,
    ] {
        assert_eq!(target.object(), ObjectKey::PayRun);
    }
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn preflight_is_pure_and_blocks_what_the_database_would_only_refuse_later(
    owner_pool: PgPool,
) {
    let (org, actor, _, port) = fixture(&owner_pool).await;
    let before = count(&owner_pool, COUNT_RUNS, ORG).await;

    // An inverted period is 0074's `payroll_draft_runs_valid_period` CHECK; the
    // preflight refuses it without opening a transaction.
    let inverted = PayRunQuery::CreateRun {
        run_id: Uuid::new_v4(),
        period_start: date!(2026 - 06 - 30),
        period_end: date!(2026 - 06 - 01),
        connector: None,
        job: None,
    };
    assert!(!PgPayRunPort::preflight(&inverted).is_ok());

    // A nil run id, an unknown decision, and a REJECT with no reason.
    assert!(!PgPayRunPort::preflight(&create(Uuid::nil())).is_ok());
    assert!(
        !PgPayRunPort::preflight(&PayRunQuery::DecideRun {
            run_id: Uuid::new_v4(),
            decision: "MAYBE".to_owned(),
            reason: None,
        })
        .is_ok()
    );
    assert!(
        !PgPayRunPort::preflight(&PayRunQuery::DecideRun {
            run_id: Uuid::new_v4(),
            decision: "REJECT".to_owned(),
            reason: Some("   ".to_owned()),
        })
        .is_ok()
    );
    assert!(PgPayRunPort::preflight(&create(Uuid::new_v4())).is_ok());

    // A blocked preflight never reaches the database: no run, no receipt.
    let error = execute(&port, command(org, actor, inverted))
        .await
        .unwrap_err();
    assert!(matches!(error, PayRunError::Blocked(_)), "{error:?}");
    assert_eq!(
        count(&owner_pool, COUNT_RUNS, ORG).await,
        before,
        "a blocked preflight must write nothing"
    );
    let receipts: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM ont_action_command_receipts WHERE org_id = $1",
    )
    .bind(ORG)
    .fetch_one(&owner_pool)
    .await
    .unwrap();
    assert_eq!(receipts, 0, "a blocked preflight must mint no receipt");
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn a_pay_run_is_created_through_the_port_and_the_receipt_names_the_owner(owner_pool: PgPool) {
    let (org, actor, _, port) = fixture(&owner_pool).await;
    let run_id = Uuid::new_v4();

    assert!(
        run_by_label(&owner_pool, run_id).await.is_none(),
        "the natural key must resolve to nothing before the port runs — otherwise \
         the assertion below is satisfied by a row this test did not create"
    );

    let receipt = execute(&port, command(org, actor, create(run_id)))
        .await
        .unwrap();

    let (id, status, label) = run_by_label(&owner_pool, run_id)
        .await
        .expect("the port must have staged the run");
    assert_eq!(status, "BLOCKED_LEGAL_GATE");
    assert_eq!(label, format!("workflow_runtime_m2:run:{run_id}"));

    // The legal gate is what `calculation_enabled = FALSE` is for: nothing may
    // calculate off a freshly staged run.
    let enabled: bool =
        sqlx::query_scalar("SELECT calculation_enabled FROM payroll_draft_runs WHERE id = $1")
            .bind(id)
            .fetch_one(&owner_pool)
            .await
            .unwrap();
    assert!(!enabled, "a staged run must not be calculation-enabled");

    // `source_summary` carries the provenance the outbox drain used to build
    // with `jsonb_build_object`.
    let summary: serde_json::Value =
        sqlx::query_scalar("SELECT source_summary FROM payroll_draft_runs WHERE id = $1")
            .bind(id)
            .fetch_one(&owner_pool)
            .await
            .unwrap();
    assert_eq!(summary["run_id"].as_str().unwrap(), run_id.to_string());
    assert_eq!(summary["connector"].as_str().unwrap(), "m2");
    assert_eq!(summary["job"].as_str().unwrap(), "payroll_draft");

    assert_eq!(receipt.target(), DispatchTarget::PayrollCreateRun);
    assert_eq!(
        receipt.owner(),
        ReceiptOwner::Canonical(ObjectKey::PayRun),
        "the receipt must be owned by PayRun, not by ontology.action"
    );
    assert_eq!(receipt.org_id(), org);
    assert_eq!(receipt.actor_id(), actor);
    assert!(receipt.result()["created"].as_bool().unwrap());

    // The stored row matches the returned receipt byte for byte on the digest.
    let stored: (Vec<u8>, serde_json::Value) = sqlx::query_as(
        "SELECT payload_digest, receipt FROM ont_action_command_receipts \
         WHERE org_id = $1 AND command_id = $2",
    )
    .bind(ORG)
    .bind(receipt.command_id().as_uuid())
    .fetch_one(&owner_pool)
    .await
    .unwrap();
    assert_eq!(stored.0, receipt.payload_digest().to_vec());
    assert_eq!(
        stored.0.len(),
        32,
        "0177's CHECK sizes the digest at 32 bytes"
    );
    assert_eq!(&stored.1, receipt.result());
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn a_repeat_of_the_same_command_replays_the_receipt_and_stages_no_second_run(
    owner_pool: PgPool,
) {
    let (org, actor, _, port) = fixture(&owner_pool).await;
    let run_id = Uuid::new_v4();
    let first = command(org, actor, create(run_id));

    let receipt = execute(&port, first.clone()).await.unwrap();
    assert_eq!(count(&owner_pool, COUNT_RUNS, ORG).await, 1);

    // The SAME command id with the SAME payload: replay, not a second write.
    let replay = execute(&port, first.clone()).await.unwrap();
    assert_eq!(
        replay, receipt,
        "a repeat must replay the stored receipt verbatim"
    );
    assert_eq!(
        count(&owner_pool, COUNT_RUNS, ORG).await,
        1,
        "the same command twice must not double-write the run"
    );
    let receipts: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM ont_action_command_receipts WHERE org_id = $1",
    )
    .bind(ORG)
    .fetch_one(&owner_pool)
    .await
    .unwrap();
    assert_eq!(receipts, 1, "a replay must not mint a second receipt");

    // A DIFFERENT command id for the same run is NOT a replay — it is a fresh
    // command — and it must still not double-write, because the natural key is
    // the second, independent idempotency mechanism. `created` is false, so the
    // receipt records that this call staged nothing.
    let second = command(org, actor, create(run_id));
    let receipt2 = execute(&port, second).await.unwrap();
    assert!(
        !receipt2.result()["created"].as_bool().unwrap(),
        "the natural key must absorb a second create of the same run"
    );
    assert_eq!(
        count(&owner_pool, COUNT_RUNS, ORG).await,
        1,
        "ON CONFLICT DO NOTHING on (org_id, period_start, period_end, source_label)"
    );

    // The same command id with a DIFFERENT payload is a conflict, never a replay.
    let mut tampered = first;
    tampered.query = PayRunQuery::CreateRun {
        run_id,
        period_start: date!(2026 - 07 - 01),
        period_end: date!(2026 - 07 - 31),
        connector: Some("m2".to_owned()),
        job: Some("payroll_draft".to_owned()),
    };
    let error = execute(&port, tampered).await.unwrap_err();
    assert!(matches!(error, PayRunError::DigestConflict(_)), "{error:?}");
}

/// A FRESH command id reusing the same run + period with a DIFFERENT
/// connector/job is a payload mismatch, not a silent absorb. The natural-key
/// conflict arm must refuse it and leave the stored provenance untouched.
#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn a_changed_provenance_on_the_same_run_and_period_is_refused(owner_pool: PgPool) {
    let (org, actor, _, port) = fixture(&owner_pool).await;
    let run_id = Uuid::new_v4();

    execute(&port, command(org, actor, create(run_id)))
        .await
        .expect("the first create must stage the run");
    assert_eq!(count(&owner_pool, COUNT_RUNS, ORG).await, 1);

    // Same run_id + period, but a different connector/job: the requested
    // provenance can never be stored on the existing row, so the port must
    // refuse instead of returning `created:false` plus success (which would let
    // a caller act on a `draft_run_id` whose provenance differs from the ask).
    let changed = command(
        org,
        actor,
        PayRunQuery::CreateRun {
            run_id,
            period_start: date!(2026 - 06 - 01),
            period_end: date!(2026 - 06 - 30),
            connector: Some("other-connector".to_owned()),
            job: Some("other-job".to_owned()),
        },
    );
    let error = execute(&port, changed).await.unwrap_err();
    assert!(
        matches!(error, PayRunError::ProvenanceConflict),
        "a changed provenance on an existing run must be refused: {error:?}"
    );

    // The refused request must not rewrite the row's provenance nor mint a run.
    let summary: serde_json::Value =
        sqlx::query_scalar("SELECT source_summary FROM payroll_draft_runs WHERE org_id = $1")
            .bind(ORG)
            .fetch_one(&owner_pool)
            .await
            .unwrap();
    assert_eq!(summary["connector"].as_str().unwrap(), "m2");
    assert_eq!(summary["job"].as_str().unwrap(), "payroll_draft");
    assert_eq!(count(&owner_pool, COUNT_RUNS, ORG).await, 1);
}

/// The three targets are ONE lifecycle, traversed with nothing but what the port
/// itself returns.
///
/// The test below this one reaches `run_by_label` through the OWNER pool to learn
/// the row id before submitting. That back channel is behind the port, behind RLS,
/// and no caller on the action surface has it -- so it proved the statements work
/// while hiding the fact that a caller could not reach them. A create whose own
/// successors cannot resolve its output shipped green underneath it.
///
/// This one is the honest oracle: whatever `CreateRun` hands back is the only
/// input `SubmitRun` is allowed. RED before `draft_run_id` existed, because the
/// receipt carried only the caller's correlation id and `run_head` keys the
/// PRIMARY KEY -- `SubmitRun` returned `not_found` for every value create ever
/// produced.
#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn the_create_receipt_alone_is_enough_to_submit(owner_pool: PgPool) {
    let (org, submitter, _decider, port) = fixture(&owner_pool).await;
    let run_id = Uuid::new_v4();

    let created = execute(&port, command(org, submitter, create(run_id)))
        .await
        .expect("create");

    // Everything below uses ONLY the receipt. No owner pool, no source_label lookup.
    let receipt = created.result();
    let draft_run_id: Uuid = receipt
        .get("draft_run_id")
        .and_then(serde_json::Value::as_str)
        .expect("CreateRun must name the row a later SubmitRun can resolve")
        .parse()
        .expect("draft_run_id must be a UUID");

    assert_ne!(
        draft_run_id, run_id,
        "these are deliberately different id spaces: run_id is the workflow drain's natural key, \
         draft_run_id is the payroll_draft_runs primary key. If they are ever equal this test has \
         stopped proving anything."
    );

    mark_calculated(&owner_pool, draft_run_id).await;

    let submit = execute(
        &port,
        command(
            org,
            submitter,
            PayRunQuery::SubmitRun {
                run_id: draft_run_id,
            },
        ),
    )
    .await
    .expect("submit must resolve the id the create receipt handed back");
    assert_eq!(submit.target(), DispatchTarget::PayrollSubmitRun);
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn submit_and_decide_drive_the_statements_this_crate_already_owned(owner_pool: PgPool) {
    let (org, submitter, decider, port) = fixture(&owner_pool).await;
    natural_person_guard_tests::bind_distinct_actors(&owner_pool, org, submitter, decider).await;
    let run_id = Uuid::new_v4();
    execute(&port, command(org, submitter, create(run_id)))
        .await
        .unwrap();
    let (id, _, _) = run_by_label(&owner_pool, run_id).await.unwrap();
    mark_calculated(&owner_pool, id).await;

    let submit = execute(
        &port,
        command(org, submitter, PayRunQuery::SubmitRun { run_id: id }),
    )
    .await
    .unwrap();
    assert_eq!(submit.target(), DispatchTarget::PayrollSubmitRun);

    // `submitted_by`/`submitted_at` are columns only `lifecycle::submit_run_in_tx`
    // writes. Their presence is what proves the port CALLED it.
    let row = sqlx::query(
        "SELECT status, submitted_by, submitted_at FROM payroll_draft_runs WHERE id = $1",
    )
    .bind(id)
    .fetch_one(&owner_pool)
    .await
    .unwrap();
    assert_eq!(row.get::<String, _>("status"), "SUBMITTED");
    assert_eq!(
        row.get::<Option<Uuid>, _>("submitted_by"),
        Some(*submitter.as_uuid())
    );
    assert!(
        row.get::<Option<time::OffsetDateTime>, _>("submitted_at")
            .is_some()
    );

    let decide = execute(
        &port,
        command(
            org,
            decider,
            PayRunQuery::DecideRun {
                run_id: id,
                decision: "APPROVE".to_owned(),
                reason: Some("6월 급여 승인".to_owned()),
            },
        ),
    )
    .await
    .unwrap();
    assert_eq!(decide.target(), DispatchTarget::PayrollDecideRun);

    let row = sqlx::query(
        "SELECT status, decided_by, decision_reason, approved_by, approved_at \
         FROM payroll_draft_runs WHERE id = $1",
    )
    .bind(id)
    .fetch_one(&owner_pool)
    .await
    .unwrap();
    assert_eq!(row.get::<String, _>("status"), "APPROVED");
    assert_eq!(
        row.get::<Option<Uuid>, _>("decided_by"),
        Some(*decider.as_uuid())
    );
    assert_eq!(
        row.get::<Option<String>, _>("decision_reason").as_deref(),
        Some("6월 급여 승인")
    );
    assert_eq!(
        row.get::<Option<Uuid>, _>("approved_by"),
        Some(*decider.as_uuid())
    );
    assert!(
        row.get::<Option<time::OffsetDateTime>, _>("approved_at")
            .is_some()
    );
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn a_decider_who_submitted_the_run_is_refused(owner_pool: PgPool) {
    let (org, submitter, _, port) = fixture(&owner_pool).await;
    let run_id = Uuid::new_v4();
    execute(&port, command(org, submitter, create(run_id)))
        .await
        .unwrap();
    let (id, _, _) = run_by_label(&owner_pool, run_id).await.unwrap();
    mark_calculated(&owner_pool, id).await;
    execute(
        &port,
        command(org, submitter, PayRunQuery::SubmitRun { run_id: id }),
    )
    .await
    .unwrap();

    // Coworker won amounts that genuinely exist on this run, so a refuse that
    // echoed line calculations would be observable. Distinct from the 3_000_000
    // golden-case figures used elsewhere in this crate.
    const COWORKER_GROSS_WON: i64 = 4_192_837;
    const COWORKER_NET_WON: i64 = 3_508_126;
    let coworker_line: Uuid = sqlx::query_scalar(
        "INSERT INTO payroll_draft_lines \
             (org_id, run_id, employee_source_key, employee_display_name, employee_company) \
         VALUES ($1, $2, 'coworker-src', 'Coworker Kim', 'KNL') RETURNING id",
    )
    .bind(ORG)
    .bind(id)
    .fetch_one(&owner_pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO payroll_line_calculations \
             (org_id, run_id, line_id, version, gross_won, deductions, \
              total_deductions_won, net_won, tax_table_version) \
         VALUES ($1, $2, $3, 1, $4, '[]'::jsonb, $5, $6, 'v1')",
    )
    .bind(ORG)
    .bind(id)
    .bind(coworker_line)
    .bind(COWORKER_GROSS_WON)
    .bind(COWORKER_GROSS_WON - COWORKER_NET_WON)
    .bind(COWORKER_NET_WON)
    .execute(&owner_pool)
    .await
    .unwrap();
    let stored_net: i64 =
        sqlx::query_scalar("SELECT net_won FROM payroll_line_calculations WHERE line_id = $1")
            .bind(coworker_line)
            .fetch_one(&owner_pool)
            .await
            .unwrap();
    assert_eq!(
        stored_net, COWORKER_NET_WON,
        "the coworker won amounts must exist before the refuse is rendered"
    );

    // The pre-existing segregation-of-duties check, reached THROUGH the port.
    let error = execute(
        &port,
        command(
            org,
            submitter,
            PayRunQuery::DecideRun {
                run_id: id,
                decision: "APPROVE".to_owned(),
                reason: None,
            },
        ),
    )
    .await
    .unwrap_err();
    assert!(
        matches!(error, PayRunError::Lifecycle(LifecycleError::SodViolation)),
        "{error:?}"
    );

    let shown = format!("{error} {error:?}");
    for (amount, grouped) in [
        (COWORKER_GROSS_WON, "4,192,837"),
        (COWORKER_NET_WON, "3,508,126"),
    ] {
        let digits = amount.to_string();
        assert!(
            !shown.contains(&digits) && !shown.contains(grouped),
            "a SoD refuse must not carry coworker payroll won amounts, found {amount} in {shown}"
        );
    }

    let row = sqlx::query("SELECT status, decided_by FROM payroll_draft_runs WHERE id = $1")
        .bind(id)
        .fetch_one(&owner_pool)
        .await
        .unwrap();
    assert_eq!(
        row.get::<String, _>("status"),
        "SUBMITTED",
        "a refused decision must not move the run"
    );
    assert_eq!(
        row.get::<Option<Uuid>, _>("decided_by"),
        None,
        "a refused decision must not stamp decided_by"
    );

    // A failed command mints no receipt, so the client may retry the same id.
    let receipts: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM ont_action_command_receipts WHERE org_id = $1",
    )
    .bind(ORG)
    .fetch_one(&owner_pool)
    .await
    .unwrap();
    assert_eq!(receipts, 2, "only the create and the submit are receipted");
}

/// The staging seam the JOB outbox drain reaches this crate through. The drain
/// itself is proven end to end by
/// `console-workflow-runtime-adapter-postgres`'s `payroll_drain_period_lock`
/// and `console-app`'s `m2_real_engine_drive`, both of which now inject THIS
/// port; what is proven here is the seam's own contract: idempotent on the
/// natural key, `false` rather than an error on a repeat.
#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn the_workflow_staging_seam_is_idempotent_on_the_natural_key(owner_pool: PgPool) {
    let (org, _, _, port) = fixture(&owner_pool).await;
    let run_id = Uuid::new_v4();
    let draft = StagePayrollDraft {
        org,
        outbox_event_id: Uuid::new_v4(),
        run_id,
        period_start: Some(date!(2026 - 06 - 01)),
        period_end: Some(date!(2026 - 06 - 30)),
        connector: Some("m2".to_owned()),
        job: Some("payroll_draft".to_owned()),
    };
    assert_eq!(
        draft.source_label(),
        format!("workflow_runtime_m2:run:{run_id}"),
        "the natural key is spelled once, in the domain crate both sides share"
    );

    assert!(
        port.stage(draft.clone()).await.unwrap(),
        "the first stage creates"
    );
    assert_eq!(count(&owner_pool, COUNT_RUNS, ORG).await, 1);

    // A different outbox event id for the same run — what a re-emitted event
    // looks like — must still collide on the natural key.
    let mut replay = draft.clone();
    replay.outbox_event_id = Uuid::new_v4();
    assert!(
        !port.stage(replay).await.unwrap(),
        "a restage must return false, not error and not double-write"
    );
    assert_eq!(
        count(&owner_pool, COUNT_RUNS, ORG).await,
        1,
        "the crash-between-stage-and-ack retry must be a no-op"
    );

    // An absent period is passed through as NULL rather than defaulted, so the
    // column's NOT NULL refuses it exactly as the old `(payload->>'…')::date`
    // form did. Silently defaulting a payroll period would be the dangerous
    // alternative.
    let mut undated = draft;
    undated.run_id = Uuid::new_v4();
    undated.period_start = None;
    undated.period_end = None;
    let error = port.stage(undated).await.unwrap_err();
    assert!(
        error.to_string().contains("null value")
            || error.to_string().to_lowercase().contains("not-null")
            || error.to_string().contains("23502"),
        "a periodless draft must be refused by NOT NULL, got: {error}"
    );
    assert_eq!(count(&owner_pool, COUNT_RUNS, ORG).await, 1);
}

/// The staging write itself must re-run the freeze-window gate, so a payroll
/// period lock that closed AFTER the drain's phase-1 read (but before the
/// staging transaction) still refuses the draft instead of staging it.
#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn the_workflow_staging_seam_refuses_a_locked_period(owner_pool: PgPool) {
    let (org, _, _, port) = fixture(&owner_pool).await;

    // An active payroll freeze window overlapping the draft's June period.
    sqlx::query(
        "INSERT INTO period_locks (org_id, domain, period_start, period_end, reason) \
         VALUES ($1, 'payroll', DATE '2026-06-01', DATE '2026-06-30', '6월 급여 마감')",
    )
    .bind(ORG)
    .execute(&owner_pool)
    .await
    .unwrap();

    let draft = StagePayrollDraft {
        org,
        outbox_event_id: Uuid::new_v4(),
        run_id: Uuid::new_v4(),
        period_start: Some(date!(2026 - 06 - 01)),
        period_end: Some(date!(2026 - 06 - 30)),
        connector: Some("m2".to_owned()),
        job: Some("payroll_draft".to_owned()),
    };

    let error = port
        .stage(draft)
        .await
        .expect_err("a locked period must refuse the staging write");
    assert!(
        error.to_string().contains("locked"),
        "the refusal must name the locked period, got: {error}"
    );
    assert_eq!(count(&owner_pool, COUNT_RUNS, ORG).await, 0);
}

/// An active payroll freeze window must refuse the CANONICAL `payroll.create_run`
/// path, not only the import drain seam. `stage_draft_run_returning_id_in_tx` is
/// the writer `create_run` uses; both must surface [`PayRunError::PeriodLocked`]
/// / [`StageDraftError::PeriodLocked`]. The drain path stays gated. Nothing here
/// may flip `payable`.
#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn canonical_create_run_refuses_a_locked_period(owner_pool: PgPool) {
    let (org, actor, _, port) = fixture(&owner_pool).await;

    sqlx::query(
        "INSERT INTO period_locks (org_id, domain, period_start, period_end, reason) \
         VALUES ($1, 'payroll', DATE '2026-06-01', DATE '2026-06-30', '6월 급여 마감')",
    )
    .bind(ORG)
    .execute(&owner_pool)
    .await
    .unwrap();

    let error = execute(&port, command(org, actor, create(Uuid::new_v4())))
        .await
        .expect_err("a locked period must refuse payroll.create_run");
    assert!(
        matches!(error, PayRunError::PeriodLocked),
        "canonical create must surface PeriodLocked, got: {error:?}"
    );

    let runtime_pool = runtime_role_pool(&owner_pool).await;
    let mut tx = runtime_pool.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.current_org', $1, true)")
        .bind(ORG.to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
    let draft = StagePayrollDraft {
        org,
        outbox_event_id: Uuid::new_v4(),
        run_id: Uuid::new_v4(),
        period_start: Some(date!(2026 - 06 - 01)),
        period_end: Some(date!(2026 - 06 - 30)),
        connector: Some("m2".to_owned()),
        job: Some("payroll_draft".to_owned()),
    };
    let staged = stage_draft_run_returning_id_in_tx(&mut tx, *org.as_uuid(), &draft).await;
    assert!(
        matches!(staged, Err(StageDraftError::PeriodLocked)),
        "canonical staging must refuse a locked period: {staged:?}"
    );
    tx.rollback().await.unwrap();

    let drain = port
        .stage(StagePayrollDraft {
            org,
            outbox_event_id: Uuid::new_v4(),
            run_id: Uuid::new_v4(),
            period_start: Some(date!(2026 - 06 - 01)),
            period_end: Some(date!(2026 - 06 - 30)),
            connector: Some("m2".to_owned()),
            job: Some("payroll_draft".to_owned()),
        })
        .await
        .expect_err("the drain path must remain gated on the freeze window");
    assert!(
        drain.to_string().contains("locked"),
        "drain refusal must name the locked period, got: {drain}"
    );

    assert_eq!(
        count(&owner_pool, COUNT_RUNS, ORG).await,
        0,
        "a locked period must stage no draft run"
    );
    let receipts: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM ont_action_command_receipts WHERE org_id = $1",
    )
    .bind(ORG)
    .fetch_one(&owner_pool)
    .await
    .unwrap();
    assert_eq!(receipts, 0, "a refused create must mint no receipt");

    let payable: i64 =
        sqlx::query_scalar("SELECT count(*)::bigint FROM payroll_line_calculations WHERE payable")
            .fetch_one(&owner_pool)
            .await
            .unwrap();
    assert_eq!(payable, 0, "a refused create must not flip payable");
}

/// The staging INSERT itself must re-check the freeze-window gate ATOMICALLY: a
/// lock that commits after the drain's phase-1 read (which saw an open period)
/// but before the write must still be refused — not slipped past by the
/// READ COMMITTED gap between a separate SELECT and the INSERT.
#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn a_lock_committed_after_the_gate_read_but_before_the_write_is_refused(owner_pool: PgPool) {
    let (org, _, _, _) = fixture(&owner_pool).await;
    let runtime_pool = runtime_role_pool(&owner_pool).await;

    // Re-open the staging transaction, armed for the tenant exactly as
    // `with_org_conn` arms it. The drain's phase-1 read already ran in an
    // earlier transaction and saw an open period; here the write re-checks.
    let mut tx = runtime_pool.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.current_org', $1, true)")
        .bind(ORG.to_string())
        .execute(&mut *tx)
        .await
        .unwrap();

    // Phase-1's gate read would pass: no active payroll lock for June 2026.
    let open: bool = sqlx::query_scalar(
        "SELECT NOT EXISTS ( \
             SELECT 1 FROM period_locks \
             WHERE domain = 'payroll' AND unlocked_at IS NULL \
               AND period_start <= DATE '2026-06-30' AND period_end >= DATE '2026-06-01' \
         )",
    )
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    assert!(open, "the period is open at the time of the phase-1 read");

    // A concurrent period lock commits AFTER that read but BEFORE the write.
    sqlx::query(
        "INSERT INTO period_locks (org_id, domain, period_start, period_end, reason) \
         VALUES ($1, 'payroll', DATE '2026-06-01', DATE '2026-06-30', 'mid-drain lock')",
    )
    .bind(ORG)
    .execute(&owner_pool)
    .await
    .unwrap();

    // The staging write must re-check the gate in the SAME statement and refuse.
    let draft = StagePayrollDraft {
        org,
        outbox_event_id: Uuid::new_v4(),
        run_id: Uuid::new_v4(),
        period_start: Some(date!(2026 - 06 - 01)),
        period_end: Some(date!(2026 - 06 - 30)),
        connector: Some("m2".to_owned()),
        job: Some("payroll_draft".to_owned()),
    };
    let result = stage_draft_run_in_tx(&mut tx, *org.as_uuid(), &draft).await;
    assert!(
        matches!(result, Err(StageDraftError::PeriodLocked)),
        "a lock committed between the read and the write must be refused: {result:?}"
    );

    tx.rollback().await.unwrap();
    assert_eq!(
        count(&owner_pool, COUNT_RUNS, ORG).await,
        0,
        "the refused write must stage nothing"
    );
}

/// A stored `source_summary` whose `connector`/`job` is missing or non-string is
/// noncanonical (the unconstrained JSONB column admits it) and must be REFUSED
/// as a provenance mismatch, never normalized to absence and absorbed.
#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn a_noncanonical_stored_provenance_is_refused_not_absorbed(owner_pool: PgPool) {
    let (org, _, _, _) = fixture(&owner_pool).await;
    let runtime_pool = runtime_role_pool(&owner_pool).await;

    let draft = StagePayrollDraft {
        org,
        outbox_event_id: Uuid::new_v4(),
        run_id: Uuid::new_v4(),
        period_start: Some(date!(2026 - 06 - 01)),
        period_end: Some(date!(2026 - 06 - 30)),
        connector: Some("m2".to_owned()),
        job: Some("payroll_draft".to_owned()),
    };

    // Stage once, then corrupt the stored connector to a non-string JSON number.
    let mut tx = runtime_pool.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.current_org', $1, true)")
        .bind(ORG.to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
    let created = stage_draft_run_in_tx(&mut tx, *org.as_uuid(), &draft)
        .await
        .unwrap();
    assert!(created, "the first stage must create the row");
    tx.commit().await.unwrap();

    sqlx::query(
        "UPDATE payroll_draft_runs \
         SET source_summary = jsonb_set(source_summary, '{connector}', '42'::jsonb) \
         WHERE org_id = $1 AND source_label = $2",
    )
    .bind(ORG)
    .bind(draft.source_label())
    .execute(&owner_pool)
    .await
    .unwrap();

    // A retry with the SAME request must refuse the noncanonical stored value.
    let mut tx = runtime_pool.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.current_org', $1, true)")
        .bind(ORG.to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
    let result = stage_draft_run_in_tx(&mut tx, *org.as_uuid(), &draft).await;
    assert!(
        matches!(result, Err(StageDraftError::ProvenanceMismatch)),
        "a non-string stored connector must be refused: {result:?}"
    );
    tx.rollback().await.unwrap();
}

/// An already-staged draft must be acknowledged (idempotent `Ok(false)`) even
/// after the period is locked — the freeze gate applies only to a NEW write, so
/// a crash between phase 2 (stage) and phase 3 (ack) followed by a lock must not
/// strand the event PENDING forever.
#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn an_existing_draft_is_acknowledged_after_the_period_is_locked(owner_pool: PgPool) {
    let (org, _, _, _) = fixture(&owner_pool).await;
    let runtime_pool = runtime_role_pool(&owner_pool).await;

    let draft = StagePayrollDraft {
        org,
        outbox_event_id: Uuid::new_v4(),
        run_id: Uuid::new_v4(),
        period_start: Some(date!(2026 - 06 - 01)),
        period_end: Some(date!(2026 - 06 - 30)),
        connector: Some("m2".to_owned()),
        job: Some("payroll_draft".to_owned()),
    };

    // Phase 2: stage the draft (created).
    let mut tx = runtime_pool.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.current_org', $1, true)")
        .bind(ORG.to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
    let created = stage_draft_run_in_tx(&mut tx, *org.as_uuid(), &draft)
        .await
        .unwrap();
    assert!(created, "the first stage must create the row");
    tx.commit().await.unwrap();

    // The period is locked AFTER the draft was staged (crash-before-ack).
    sqlx::query(
        "INSERT INTO period_locks (org_id, domain, period_start, period_end, reason) \
         VALUES ($1, 'payroll', DATE '2026-06-01', DATE '2026-06-30', '6월 급여 마감')",
    )
    .bind(ORG)
    .execute(&owner_pool)
    .await
    .unwrap();

    // The retry must be an idempotent ack, not a PeriodLocked refusal.
    let mut tx = runtime_pool.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.current_org', $1, true)")
        .bind(ORG.to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
    let created = stage_draft_run_in_tx(&mut tx, *org.as_uuid(), &draft)
        .await
        .unwrap();
    assert!(
        !created,
        "the existing draft must be acknowledged, not re-created or refused"
    );
    tx.commit().await.unwrap();
    assert_eq!(count(&owner_pool, COUNT_RUNS, ORG).await, 1);
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn a_foreign_tenant_is_invisible_and_unwritable_to_the_runtime_role(owner_pool: PgPool) {
    let (org, actor, _, port) = fixture(&owner_pool).await;
    let foreign_actor = seed_org_and_user(&owner_pool, FOREIGN_ORG, "foreign").await;

    // Seed the FOREIGN tenant's run through the BYPASSRLS owner pool.
    let foreign_run = Uuid::new_v4();
    let foreign_id: Uuid = sqlx::query_scalar(
        "INSERT INTO payroll_draft_runs \
             (org_id, period_start, period_end, source_label, status) \
         VALUES ($1, DATE '2026-06-01', DATE '2026-06-30', $2, 'BLOCKED_LEGAL_GATE') \
         RETURNING id",
    )
    .bind(FOREIGN_ORG)
    .bind(format!("workflow_runtime_m2:run:{foreign_run}"))
    .fetch_one(&owner_pool)
    .await
    .unwrap();

    // NON-VACUOUS: the row genuinely EXISTS before the boundary is tested. Without
    // this the "0 rows" assertion below would also pass against an empty table.
    assert_eq!(
        count(&owner_pool, COUNT_RUNS, FOREIGN_ORG).await,
        1,
        "payroll_draft_runs must hold exactly the foreign tenant's row before the \
         boundary is tested"
    );

    // READ: a console_rt session armed for THIS org counts zero of them.
    let runtime_pool = runtime_role_pool(&owner_pool).await;
    let mut conn = runtime_pool.acquire().await.unwrap();
    sqlx::query("SELECT set_config('app.current_org', $1, false)")
        .bind(ORG.to_string())
        .execute(&mut *conn)
        .await
        .unwrap();
    let visible: i64 = sqlx::query_scalar("SELECT count(*)::bigint FROM payroll_draft_runs")
        .fetch_one(&mut *conn)
        .await
        .unwrap();
    assert_eq!(
        visible, 0,
        "the foreign tenant's run must be invisible to a session armed for another org"
    );

    // WRITE: staging into the foreign tenant while armed for this one is refused
    // by the policy's WITH CHECK, not by application filtering.
    let error = sqlx::query(
        "INSERT INTO payroll_draft_runs \
             (org_id, period_start, period_end, source_label, status) \
         VALUES ($1, DATE '2026-06-01', DATE '2026-06-30', 'cross-tenant', \
                 'BLOCKED_LEGAL_GATE')",
    )
    .bind(FOREIGN_ORG)
    .execute(&mut *conn)
    .await
    .unwrap_err();
    let db = error.as_database_error().unwrap();
    assert_eq!(db.code().as_deref(), Some("42501"), "{error}");
    assert!(
        db.message().contains("row-level security"),
        "expected the RLS refusal, got: {}",
        db.message()
    );
    drop(conn);

    // And the port itself, armed for THIS org, writes into THIS org only.
    let run_id = Uuid::new_v4();
    execute(&port, command(org, actor, create(run_id)))
        .await
        .unwrap();
    assert_eq!(count(&owner_pool, COUNT_RUNS, ORG).await, 1);
    assert_eq!(
        count(&owner_pool, COUNT_RUNS, FOREIGN_ORG).await,
        1,
        "the foreign tenant's row count must be untouched by this org's port"
    );

    // Submit/decide on the foreign PRIMARY KEY must omit (NotFound), not write.
    // CALCULATED / SUBMITTED are the states those targets WOULD advance if the
    // port could see the row — InvalidState on BLOCKED_LEGAL_GATE would also
    // "not write" while proving the foreign id was visible.
    mark_calculated(&owner_pool, foreign_id).await;
    let submit_err = execute(
        &port,
        command(org, actor, PayRunQuery::SubmitRun { run_id: foreign_id }),
    )
    .await
    .unwrap_err();
    assert!(
        matches!(submit_err, PayRunError::Lifecycle(LifecycleError::NotFound)),
        "submit of a foreign run must omit, got {submit_err:?}"
    );
    let row = sqlx::query(
        "SELECT status, submitted_by, decided_by FROM payroll_draft_runs WHERE id = $1",
    )
    .bind(foreign_id)
    .fetch_one(&owner_pool)
    .await
    .unwrap();
    assert_eq!(row.get::<String, _>("status"), "CALCULATED");
    assert_eq!(row.get::<Option<Uuid>, _>("submitted_by"), None);
    assert_eq!(row.get::<Option<Uuid>, _>("decided_by"), None);

    sqlx::query(
        "UPDATE payroll_draft_runs \
         SET status = 'SUBMITTED', submitted_by = $2, submitted_at = now() \
         WHERE id = $1",
    )
    .bind(foreign_id)
    .bind(*foreign_actor.as_uuid())
    .execute(&owner_pool)
    .await
    .unwrap();
    let decide_err = execute(
        &port,
        command(
            org,
            actor,
            PayRunQuery::DecideRun {
                run_id: foreign_id,
                decision: "APPROVE".to_owned(),
                reason: None,
            },
        ),
    )
    .await
    .unwrap_err();
    assert!(
        matches!(decide_err, PayRunError::Lifecycle(LifecycleError::NotFound)),
        "decide of a foreign run must omit, got {decide_err:?}"
    );
    let row = sqlx::query(
        "SELECT status, submitted_by, decided_by FROM payroll_draft_runs WHERE id = $1",
    )
    .bind(foreign_id)
    .fetch_one(&owner_pool)
    .await
    .unwrap();
    assert_eq!(row.get::<String, _>("status"), "SUBMITTED");
    assert_eq!(
        row.get::<Option<Uuid>, _>("submitted_by"),
        Some(*foreign_actor.as_uuid())
    );
    assert_eq!(
        row.get::<Option<Uuid>, _>("decided_by"),
        None,
        "a foreign decide omit must not stamp decided_by"
    );
    let receipts: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM ont_action_command_receipts WHERE org_id = $1",
    )
    .bind(ORG)
    .fetch_one(&owner_pool)
    .await
    .unwrap();
    assert_eq!(
        receipts, 1,
        "a foreign omit must mint no receipt in this org"
    );
    let foreign_receipts: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM ont_action_command_receipts WHERE org_id = $1",
    )
    .bind(FOREIGN_ORG)
    .fetch_one(&owner_pool)
    .await
    .unwrap();
    assert_eq!(foreign_receipts, 0, "a foreign omit must mint no receipt");
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn a_stored_receipt_naming_no_dispatch_target_is_refused(owner_pool: PgPool) {
    let (org, actor, _, port) = fixture(&owner_pool).await;
    let cmd = command(org, actor, create(Uuid::new_v4()));
    let receipt = execute(&port, cmd.clone()).await.unwrap();
    let command_uuid = *cmd.command_id.as_uuid();

    // Stand a hostile row where the good one was, carrying the SAME digest — so
    // the replay gets PAST the digest comparison and the refusal below is the
    // target read, not a `DigestConflict` — but a receipt naming no dispatch
    // target, which is the shape an `ontology.action` row has. 0177's trigger
    // refuses UPDATE and DELETE per row and TRUNCATE is statement-level, so this
    // is the only way a test can replace the row.
    sqlx::query("TRUNCATE ont_action_command_receipts")
        .execute(&owner_pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO ont_action_command_receipts \
             (org_id, command_id, actor_id, payload_digest, receipt, created_at) \
         VALUES ($1, $2, $3, $4, $5, now())",
    )
    .bind(ORG)
    .bind(command_uuid)
    .bind(actor.as_uuid())
    .bind(receipt.payload_digest().as_slice())
    .bind(serde_json::json!({ "run_id": receipt.result()["run_id"].clone() }))
    .execute(&owner_pool)
    .await
    .unwrap();

    let error = execute(&port, cmd).await.unwrap_err();
    assert!(
        matches!(error, PayRunError::UnreadableReceipt(id, _) if id == command_uuid),
        "a receipt naming no target must be refused, never replayed: {error:?}"
    );
}

/// Empty-tenant Company/OrgUnit/JobPosition/Person/`hr.appoint`, then the same
/// PayRun create → submit → decide path. Fixture `INSERT INTO organizations`
/// plus a PayRun port is not this path. No won arithmetic.
#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn empty_tenant_pay_run_lifecycle_sits_on_canonical_org_tree(owner_pool: PgPool) {
    let (org, submitter, decider, port, appointed) =
        empty_tenant_pay_run_fixture(&owner_pool).await;
    natural_person_guard_tests::bind_distinct_actors(&owner_pool, org, submitter, decider).await;
    assert_eq!(appointed.target(), DispatchTarget::HrAppoint);

    let run_id = Uuid::new_v4();
    let created = execute(&port, command(org, submitter, create(run_id)))
        .await
        .expect("create_run");
    assert_eq!(created.target(), DispatchTarget::PayrollCreateRun);
    let draft_run_id: Uuid = created.result()["draft_run_id"]
        .as_str()
        .expect("CreateRun must name draft_run_id")
        .parse()
        .unwrap();
    let enabled: bool =
        sqlx::query_scalar("SELECT calculation_enabled FROM payroll_draft_runs WHERE id = $1")
            .bind(draft_run_id)
            .fetch_one(&owner_pool)
            .await
            .unwrap();
    assert!(
        !enabled,
        "a staged run on the canonical tree must not be calculation-enabled"
    );

    mark_calculated(&owner_pool, draft_run_id).await;
    let submit = execute(
        &port,
        command(
            org,
            submitter,
            PayRunQuery::SubmitRun {
                run_id: draft_run_id,
            },
        ),
    )
    .await
    .expect("submit_run");
    assert_eq!(submit.target(), DispatchTarget::PayrollSubmitRun);
    let status: String = sqlx::query_scalar("SELECT status FROM payroll_draft_runs WHERE id = $1")
        .bind(draft_run_id)
        .fetch_one(&owner_pool)
        .await
        .unwrap();
    assert_eq!(status, "SUBMITTED");

    let decide = execute(
        &port,
        command(
            org,
            decider,
            PayRunQuery::DecideRun {
                run_id: draft_run_id,
                decision: "APPROVE".to_owned(),
                reason: Some("empty-tenant 승인".to_owned()),
            },
        ),
    )
    .await
    .expect("decide_run");
    assert_eq!(decide.target(), DispatchTarget::PayrollDecideRun);
    let row = sqlx::query("SELECT status, decided_by FROM payroll_draft_runs WHERE id = $1")
        .bind(draft_run_id)
        .fetch_one(&owner_pool)
        .await
        .unwrap();
    assert_eq!(row.get::<String, _>("status"), "APPROVED");
    assert_eq!(
        row.get::<Option<Uuid>, _>("decided_by"),
        Some(*decider.as_uuid())
    );

    let instances: i64 =
        sqlx::query_scalar("SELECT count(*)::bigint FROM ont_instances WHERE org_id = $1")
            .bind(*org.as_uuid())
            .fetch_one(&owner_pool)
            .await
            .unwrap();
    assert_eq!(instances, 0, "PayRun must not mint ont_instances");
}

async fn empty_tenant_pay_run_fixture(
    owner_pool: &PgPool,
) -> (OrgId, UserId, UserId, PgPayRunPort, CommandReceipt) {
    let submitter = seed_org_and_super_admin(owner_pool, ORG, "payrun-tree").await;
    let decider = seed_org_and_user(owner_pool, ORG, "payrun-tree-decider").await;
    let runtime_pool = runtime_role_pool(owner_pool).await;
    let handle = tokio::runtime::Handle::current();
    let company = PgCompanyPort::new(runtime_pool.clone(), handle.clone());
    let units = PgOrgUnitPort::new(runtime_pool.clone(), handle.clone());
    let positions = PgJobPositionPort::new(runtime_pool.clone(), handle.clone());
    let persons = PgPersonPort::new(runtime_pool.clone(), handle.clone());
    let employment = PgEmploymentPort::new(runtime_pool.clone(), handle.clone());
    let port = PgPayRunPort::new(runtime_pool.clone(), handle);
    let org = OrgId::from_uuid(ORG);

    execute_sync(
        &company,
        CompanyCommand {
            org_id: org,
            command_id: CommandId::from_uuid(Uuid::new_v4()),
            actor_id: submitter,
            query: CompanyQuery {
                attributes: json!({ "legal_name": "주식회사 아크메" }),
            },
            action_key: "company.revise".to_owned(),
            object_type_id: Uuid::nil(),
        },
    )
    .await
    .unwrap();

    let unit_receipt = execute_sync(
        &units,
        OrgUnitCommand {
            org_id: org,
            command_id: CommandId::from_uuid(Uuid::new_v4()),
            actor_id: submitter,
            query: OrgUnitQuery::Create {
                source: None,
                attributes: json!({ "name": "영업본부", "kind": "site" }),
            },
            action_key: "create_org_unit".to_owned(),
            object_type_id: Uuid::nil(),
        },
    )
    .await
    .unwrap();
    let sales: Uuid = unit_receipt.result()["org_unit_id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();

    let position_receipt = execute_sync(
        &positions,
        JobPositionCommand {
            org_id: org,
            command_id: CommandId::from_uuid(Uuid::new_v4()),
            actor_id: submitter,
            query: JobPositionQuery::Create {
                org_unit_id: sales,
                attributes: json!({ "title": "백엔드 엔지니어" }),
            },
            action_key: "create_job_position".to_owned(),
            object_type_id: Uuid::nil(),
        },
    )
    .await
    .unwrap();
    let engineer: Uuid = position_receipt.result()["job_position_id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();

    let employee_id = Uuid::new_v4();
    let sales_text = sales.to_string();
    let engineer_text = engineer.to_string();
    let mut tx = runtime_pool.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.current_org', $1, true)")
        .bind(ORG.to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
    insert_employee_record(
        &mut tx,
        ORG,
        NewEmployeeRecord {
            employee_id,
            company: "ACME",
            name: "김직원",
            employee_number: "E-RUN-1",
            org_unit: &sales_text,
            position: &engineer_text,
            worksite_name: "서울",
        },
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    execute_sync(
        &persons,
        PersonCommand {
            org_id: org,
            command_id: CommandId::from_uuid(Uuid::new_v4()),
            actor_id: submitter,
            query: PersonQuery::Create {
                employee_id: Some(employee_id),
                attributes: json!({ "legal_name": "김직원" }),
            },
            action_key: "create_person".to_owned(),
            object_type_id: Uuid::nil(),
        },
    )
    .await
    .unwrap();

    let appointed = execute_sync(
        &employment,
        EmploymentCommand {
            org_id: org,
            command_id: CommandId::from_uuid(Uuid::new_v4()),
            actor_id: submitter,
            query: EmploymentQuery::Appoint {
                employee_id,
                valid_from: OffsetDateTime::new_utc(date!(2026 - 01 - 01), Time::MIDNIGHT),
                attributes: EmploymentAttributes {
                    company: "ACME".to_owned(),
                    org_unit_id: Some(sales),
                    job_position_id: Some(engineer),
                    employment_status: "ACTIVE".to_owned(),
                },
            },
            action_key: "appoint".to_owned(),
            object_type_id: Uuid::nil(),
        },
    )
    .await
    .unwrap();

    (org, submitter, decider, port, appointed)
}

async fn execute_sync<P: CanonicalPort + Clone + Send + 'static>(
    port: &P,
    command: P::Command,
) -> Result<CommandReceipt, P::Error>
where
    P::Command: Send + 'static,
    P::Error: Send + 'static,
{
    let port = port.clone();
    tokio::task::spawn_blocking(move || port.execute(&command))
        .await
        .unwrap()
}

/// PostgreSQL row-lock SELECT needs an UPDATE privilege, while the existing
/// immutable trigger must continue rejecting actual binding updates.
#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn actual_business_login_can_lock_person_bindings_without_mutating_them(owner_pool: PgPool) {
    use console_platform_test_support::{TestDatabaseLogin, login_test_pool};

    let mut binding_ids = Vec::new();
    for (org, tag) in [(ORG, "binding-lock"), (FOREIGN_ORG, "binding-foreign")] {
        let user = seed_org_and_user(&owner_pool, org, tag).await;
        let employee: Uuid = sqlx::query_scalar(
            "INSERT INTO employees (org_id, company, name, source_filename, source_sheet, source_row, source_key) \
             VALUES ($1, 'ACME', $2, 'fixture', 'fixture', 1, $2) RETURNING id",
        )
        .bind(org).bind(tag).fetch_one(&owner_pool).await.unwrap();
        let person: Uuid =
            sqlx::query_scalar("INSERT INTO persons (org_id) VALUES ($1) RETURNING id")
                .bind(org)
                .fetch_one(&owner_pool)
                .await
                .unwrap();
        sqlx::query(
            "INSERT INTO employee_person_bindings (org_id, employee_id, person_id, actor_id, payload_digest) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(org).bind(employee).bind(person).bind(user.as_uuid())
        .bind([7_u8; 32].as_slice()).execute(&owner_pool).await.unwrap();
        binding_ids.push((org, employee, person));
    }
    let before: Vec<serde_json::Value> = sqlx::query_scalar(
        "SELECT to_jsonb(b) FROM employee_person_bindings b ORDER BY org_id, employee_id",
    )
    .fetch_all(&owner_pool)
    .await
    .unwrap();
    assert_eq!(
        before.len(),
        2,
        "own and foreign bindings must really exist"
    );
    let actual = login_test_pool(&owner_pool, TestDatabaseLogin::Business).await;
    let mut tx = actual.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.current_org', $1, true)")
        .bind(ORG.to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
    let locked: Vec<(Uuid, Uuid)> = sqlx::query_as(
        "SELECT employee_id, person_id FROM employee_person_bindings ORDER BY org_id, employee_id FOR SHARE",
    ).fetch_all(&mut *tx).await.expect("PERSON_BINDING_ROW_LOCK_REQUIRED: actual Business login must acquire row locks");
    assert_eq!(locked, vec![(binding_ids[0].1, binding_ids[0].2)]);
    let privileges: Vec<(String, bool)> = sqlx::query_as(
        "SELECT attname::text, has_column_privilege(current_user, attrelid, attnum, 'UPDATE') \
         FROM pg_attribute WHERE attrelid='public.employee_person_bindings'::regclass \
         AND attnum>0 AND NOT attisdropped ORDER BY attnum",
    )
    .fetch_all(&mut *tx)
    .await
    .unwrap();
    assert_eq!(privileges.len(), 6);
    for (column, allowed) in privileges {
        assert_eq!(
            allowed,
            column == "employee_id",
            "unexpected UPDATE grant for {column}"
        );
    }
    let whole_table: bool = sqlx::query_scalar(
        "SELECT has_table_privilege(current_user, 'public.employee_person_bindings', 'UPDATE')",
    )
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    assert!(
        !whole_table,
        "row locking must not grant whole-table UPDATE"
    );
    tx.rollback().await.unwrap();

    for (statement, code, message) in [
        (
            "UPDATE employee_person_bindings SET employee_id=employee_id",
            "P0001",
            "row is immutable",
        ),
        (
            "UPDATE employee_person_bindings SET person_id=person_id",
            "42501",
            "permission denied",
        ),
        (
            "ALTER TABLE employee_person_bindings DISABLE TRIGGER trg_employee_person_bindings_immutable",
            "42501",
            "must be owner",
        ),
    ] {
        let mut tx = actual.begin().await.unwrap();
        sqlx::query("SELECT set_config('app.current_org', $1, true)")
            .bind(ORG.to_string())
            .execute(&mut *tx)
            .await
            .unwrap();
        let error = sqlx::query(statement)
            .execute(&mut *tx)
            .await
            .expect_err("binding mutation must fail");
        let database = error.as_database_error().expect("database refusal");
        assert_eq!(database.code().as_deref(), Some(code));
        assert!(database.message().contains(message), "{database}");
        tx.rollback().await.unwrap();
    }
    let after: Vec<serde_json::Value> = sqlx::query_scalar(
        "SELECT to_jsonb(b) FROM employee_person_bindings b ORDER BY org_id, employee_id",
    )
    .fetch_all(&owner_pool)
    .await
    .unwrap();
    assert_eq!(
        after, before,
        "locking and refused mutations preserve every binding byte"
    );
    actual.close().await;
}

// Append as a child module of pay_run_port_as_runtime_role.rs.
// Tests execute the existing canonical owner and shared lifecycle transaction.
// Fixture calculation is explicit; these tests do not qualify wage calculation.
mod natural_person_guard_tests {
    use super::*;
    use console_payroll_adapter_postgres::lifecycle::decide_run_in_tx;
    use console_platform_test_support::{TestDatabaseLogin, login_test_pool};
    use serde_json::Value;
    use sqlx::{Postgres, Transaction};
    use std::collections::BTreeMap;
    use std::time::Duration;

    async fn arm(pool: &PgPool) -> Transaction<'static, Postgres> {
        let mut tx = pool.begin().await.unwrap();
        sqlx::query("SELECT set_config('app.current_org', $1, true)")
            .bind(ORG.to_string())
            .execute(&mut *tx)
            .await
            .unwrap();
        tx
    }

    async fn employee(owner: &PgPool, org: OrgId, tag: &str) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query("INSERT INTO employees (id, org_id, company, name, source_filename, source_sheet, source_row, source_key) VALUES ($1, $2, 'ACME', $3, 'fixture', 'fixture', 1, $4)")
            .bind(id).bind(*org.as_uuid()).bind(tag).bind(id.to_string())
            .execute(owner).await.unwrap();
        id
    }

    async fn person(
        pool: &PgPool,
        org: OrgId,
        actor: UserId,
        employee: Uuid,
        existing: Option<Uuid>,
    ) -> Uuid {
        let port = PgPersonPort::new(pool.clone(), tokio::runtime::Handle::current());
        let attributes = json!({"legal_name": "검증 참여자"});
        let query = match existing {
            Some(person_id) => PersonQuery::Revise {
                person_id,
                employee_id: Some(employee),
                attributes,
            },
            None => PersonQuery::Create {
                employee_id: Some(employee),
                attributes,
            },
        };
        let receipt = execute_sync(
            &port,
            PersonCommand {
                org_id: org,
                command_id: CommandId::from_uuid(Uuid::new_v4()),
                actor_id: actor,
                query,
                action_key: if existing.is_some() {
                    "revise_person"
                } else {
                    "create_person"
                }
                .to_owned(),
                object_type_id: Uuid::nil(),
            },
        )
        .await
        .unwrap();
        receipt.result()["person_id"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap()
    }

    // Fixture successor called ONLY by the two existing positive decision
    // journeys; no old negative or assertion is replaced.
    pub(super) async fn bind_distinct_actors(
        owner: &PgPool,
        org: OrgId,
        first: UserId,
        second: UserId,
    ) {
        let pool = login_test_pool(owner, TestDatabaseLogin::Business).await;
        let mut persons = Vec::new();
        for actor in [first, second] {
            let employee = employee(owner, org, "independent reviewer").await;
            persons.push(person(&pool, org, actor, employee, None).await);
            sqlx::query("UPDATE users SET employee_id = $1 WHERE id = $2 AND org_id = $3")
                .bind(employee)
                .bind(*actor.as_uuid())
                .bind(*org.as_uuid())
                .execute(owner)
                .await
                .unwrap();
        }
        assert_ne!(persons[0], persons[1]);
        pool.close().await;
    }

    struct Fixture {
        org: OrgId,
        submitter: UserId,
        decider: UserId,
        employees: [Uuid; 3],
        persons: [Uuid; 2],
        pool: PgPool,
        port: PgPayRunPort,
    }

    async fn fixture(owner: &PgPool, same: bool) -> Fixture {
        let submitter = seed_org_and_user(owner, ORG, "person-submitter").await;
        let decider = seed_org_and_user(owner, ORG, "person-decider").await;
        let org = OrgId::from_uuid(ORG);
        let pool = login_test_pool(owner, TestDatabaseLogin::Business).await;
        let e1 = employee(owner, org, "first employment").await;
        let e2 = employee(owner, org, "second employment").await;
        let e3 = employee(owner, org, "unassigned alternate employment").await;
        let p1 = person(&pool, org, submitter, e1, None).await;
        let p2 = person(&pool, org, decider, e2, same.then_some(p1)).await;
        assert_eq!(person(&pool, org, submitter, e3, Some(p1)).await, p1);
        for (user, employee) in [(submitter, e1), (decider, e2)] {
            let changed =
                sqlx::query("UPDATE users SET employee_id = $1 WHERE id = $2 AND org_id = $3")
                    .bind(employee)
                    .bind(*user.as_uuid())
                    .bind(ORG)
                    .execute(owner)
                    .await
                    .unwrap();
            assert_eq!(changed.rows_affected(), 1);
        }
        let resolved: Vec<(Uuid, Uuid, Uuid)> = sqlx::query_as("SELECT u.id, u.employee_id, b.person_id FROM users u JOIN employee_person_bindings b ON b.org_id=u.org_id AND b.employee_id=u.employee_id WHERE u.org_id=$1 ORDER BY u.id")
            .bind(ORG).fetch_all(owner).await.unwrap();
        assert_eq!(resolved.len(), 2);
        assert_ne!(resolved[0].0, resolved[1].0);
        assert_ne!(resolved[0].1, resolved[1].1);
        assert_eq!(resolved[0].2 == resolved[1].2, same);
        let port = PgPayRunPort::new(pool.clone(), tokio::runtime::Handle::current());
        Fixture {
            org,
            submitter,
            decider,
            employees: [e1, e2, e3],
            persons: [p1, p2],
            pool,
            port,
        }
    }

    async fn submitted(owner: &PgPool, f: &Fixture) -> Uuid {
        let label = Uuid::new_v4();
        let receipt = execute(&f.port, command(f.org, f.submitter, create(label)))
            .await
            .unwrap();
        let id: Uuid = receipt.result()["draft_run_id"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        mark_calculated(owner, id).await;
        execute(
            &f.port,
            command(f.org, f.submitter, PayRunQuery::SubmitRun { run_id: id }),
        )
        .await
        .unwrap();
        id
    }

    fn decision(f: &Fixture, id: Uuid, decision: &str) -> PayRunCommand {
        command(
            f.org,
            f.decider,
            PayRunQuery::DecideRun {
                run_id: id,
                decision: decision.to_owned(),
                reason: Some("독립 검토".to_owned()),
            },
        )
    }

    // Every durable public row, including all payroll tables, receipts, audit,
    // and identity history. Names come from pg_catalog and are quoted.
    async fn census(owner: &PgPool) -> BTreeMap<String, Value> {
        let tables: Vec<String> = sqlx::query_scalar("SELECT c.relname::text FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='public' AND c.relkind IN ('r','p') ORDER BY c.relname")
            .fetch_all(owner).await.unwrap();
        let mut result = BTreeMap::new();
        for table in tables {
            let quoted = table.replace('"', "\"\"");
            let sql = format!(
                "SELECT coalesce(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM public.\"{quoted}\" t"
            );
            // SqlSafeStr: table is pg_catalog metadata and every identifier quote is doubled.
            let rows: Value = sqlx::query_scalar(sqlx::AssertSqlSafe(sql.as_str()))
                .fetch_one(owner)
                .await
                .unwrap();
            result.insert(table, rows);
        }
        result
    }

    fn assert_sod(error: PayRunError) {
        assert!(
            matches!(error, PayRunError::Lifecycle(LifecycleError::SodViolation)),
            "{error:?}"
        );
    }

    #[sqlx::test(migrations = "../../platform/db/migrations")]
    async fn distinct_users_and_employees_for_one_person_cannot_approve_or_reject(owner: PgPool) {
        let f = fixture(&owner, true).await;
        for verb in ["APPROVE", "REJECT"] {
            let id = submitted(&owner, &f).await;
            let command = decision(&f, id, verb);
            let before = census(&owner).await;
            assert_sod(execute(&f.port, command.clone()).await.unwrap_err());
            assert_eq!(
                census(&owner).await,
                before,
                "{verb} must have zero durable effects"
            );
            assert_sod(execute(&f.port, command).await.unwrap_err());
            assert_eq!(
                census(&owner).await,
                before,
                "repeat denial must have zero effects"
            );
        }
    }

    #[sqlx::test(migrations = "../../platform/db/migrations")]
    async fn unresolved_or_foreign_person_identity_refuses_both_decisions(owner: PgPool) {
        let f = fixture(&owner, false).await;
        let foreign = seed_org_and_user(&owner, FOREIGN_ORG, "foreign-reviewer").await;
        for verb in ["APPROVE", "REJECT"] {
            for case in [
                "missing_submitter",
                "submitter_employee",
                "decider_employee",
                "submitter_binding",
                "decider_binding",
                "foreign_decider",
                "absent_decider",
            ] {
                let id = submitted(&owner, &f).await;
                // Fixture mutations commit before the command so full durable
                // census can detect any accidentally committed side effect.
                let employee_index = usize::from(case == "decider_binding");
                let original_binding: Value = sqlx::query_scalar("SELECT to_jsonb(b) FROM employee_person_bindings b WHERE org_id=$1 AND employee_id=$2")
                    .bind(ORG).bind(f.employees[employee_index]).fetch_one(&owner).await.unwrap();
                let actor = match case {
                    "missing_submitter" => {
                        sqlx::query("UPDATE payroll_draft_runs SET submitted_by=NULL WHERE id=$1")
                            .bind(id)
                            .execute(&owner)
                            .await
                            .unwrap();
                        f.decider
                    }
                    "submitter_employee" | "decider_employee" => {
                        let user = if case == "submitter_employee" {
                            f.submitter
                        } else {
                            f.decider
                        };
                        sqlx::query("UPDATE users SET employee_id=NULL WHERE id=$1")
                            .bind(*user.as_uuid())
                            .execute(&owner)
                            .await
                            .unwrap();
                        f.decider
                    }
                    "submitter_binding" | "decider_binding" => {
                        sqlx::query("DELETE FROM employee_person_bindings WHERE org_id=$1 AND employee_id=$2").bind(ORG).bind(f.employees[employee_index]).execute(&owner).await.unwrap();
                        f.decider
                    }
                    "foreign_decider" => foreign,
                    "absent_decider" => UserId::new(),
                    _ => unreachable!(),
                };
                let before = census(&owner).await;
                let cmd = command(
                    f.org,
                    actor,
                    PayRunQuery::DecideRun {
                        run_id: id,
                        decision: verb.to_owned(),
                        reason: Some("검토".to_owned()),
                    },
                );
                assert_sod(execute(&f.port, cmd).await.unwrap_err());
                assert_eq!(census(&owner).await, before, "{verb}/{case}");
                match case {
                    "submitter_employee" | "decider_employee" => {
                        let index = usize::from(case == "decider_employee");
                        let user = [f.submitter, f.decider][index];
                        sqlx::query("UPDATE users SET employee_id=$1 WHERE id=$2")
                            .bind(f.employees[index])
                            .bind(*user.as_uuid())
                            .execute(&owner)
                            .await
                            .unwrap();
                    }
                    "submitter_binding" | "decider_binding" => {
                        sqlx::query("INSERT INTO employee_person_bindings SELECT * FROM jsonb_populate_record(NULL::employee_person_bindings, $1)").bind(original_binding).execute(&owner).await.unwrap();
                    }
                    _ => {}
                }
            }
        }
    }

    #[sqlx::test(migrations = "../../platform/db/migrations")]
    async fn distinct_people_can_decide_and_exact_command_replay_has_no_effect(owner: PgPool) {
        let f = fixture(&owner, false).await;
        for verb in ["APPROVE", "REJECT"] {
            let id = submitted(&owner, &f).await;
            let command = decision(&f, id, verb);
            let receipt = execute(&f.port, command.clone()).await.unwrap();
            assert_eq!(receipt.target(), DispatchTarget::PayrollDecideRun);
            let row = sqlx::query("SELECT status, submitted_by, decided_by, decided_at, decision_reason, approved_by, approved_at FROM payroll_draft_runs WHERE id=$1").bind(id).fetch_one(&owner).await.unwrap();
            assert_eq!(
                row.get::<String, _>("status"),
                if verb == "APPROVE" {
                    "APPROVED"
                } else {
                    "REJECTED"
                }
            );
            assert_eq!(row.get::<Uuid, _>("submitted_by"), *f.submitter.as_uuid());
            assert_eq!(row.get::<Uuid, _>("decided_by"), *f.decider.as_uuid());
            assert!(row.get::<Option<OffsetDateTime>, _>("decided_at").is_some());
            assert_eq!(row.get::<String, _>("decision_reason"), "독립 검토");
            assert_eq!(
                row.get::<Option<Uuid>, _>("approved_by"),
                (verb == "APPROVE").then_some(*f.decider.as_uuid())
            );
            assert_eq!(
                row.get::<Option<OffsetDateTime>, _>("approved_at")
                    .is_some(),
                verb == "APPROVE"
            );
            let after = census(&owner).await;
            let replay = execute(&f.port, command).await.unwrap();
            assert_eq!(replay, receipt);
            assert_eq!(census(&owner).await, after);
        }
    }

    async fn pid(tx: &mut Transaction<'_, Postgres>) -> i32 {
        sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut **tx)
            .await
            .unwrap()
    }

    async fn wait_blocked(owner: &PgPool, waiter: i32, holder: i32) {
        tokio::time::timeout(Duration::from_secs(20), async {
            loop {
                let blocked: bool =
                    sqlx::query_scalar("SELECT $2 = ANY(pg_catalog.pg_blocking_pids($1))")
                        .bind(waiter)
                        .bind(holder)
                        .fetch_one(owner)
                        .await
                        .unwrap();
                if blocked {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .expect("actual PostgreSQL blocking edge must be observed");
    }

    async fn change_identity(tx: &mut Transaction<'_, Postgres>, kind: &str, f: &Fixture) {
        let index = usize::from(!kind.starts_with("submitter_"));
        let actor = [f.submitter, f.decider][index];
        if kind.ends_with("user") {
            let changed = sqlx::query("UPDATE users SET employee_id=$1 WHERE id=$2 AND org_id=$3")
                .bind(if index == 0 {
                    f.employees[0]
                } else {
                    f.employees[2]
                })
                .bind(*actor.as_uuid())
                .bind(ORG)
                .execute(&mut **tx)
                .await
                .unwrap();
            assert_eq!(changed.rows_affected(), 1);
        } else {
            let deleted = sqlx::query(
                "DELETE FROM employee_person_bindings WHERE org_id=$1 AND employee_id=$2",
            )
            .bind(ORG)
            .bind(f.employees[index])
            .execute(&mut **tx)
            .await
            .unwrap();
            assert_eq!(deleted.rows_affected(), 1);
            sqlx::query("INSERT INTO employee_person_bindings (org_id, employee_id, person_id, actor_id, payload_digest) VALUES ($1,$2,$3,$4,decode(repeat('ab',32),'hex'))")
                .bind(ORG).bind(f.employees[index]).bind(f.persons[0]).bind(*actor.as_uuid()).execute(&mut **tx).await.unwrap();
        }
    }

    #[sqlx::test(migrations = "../../platform/db/migrations")]
    async fn decision_retains_user_and_binding_locks_until_commit_or_rollback(owner: PgPool) {
        let f = std::sync::Arc::new(fixture(&owner, false).await);
        for kind in ["user", "binding", "submitter_user", "submitter_binding"] {
            for commit in [false, true] {
                let id = submitted(&owner, &f).await;
                let before = census(&owner).await;
                let mut reader = arm(&f.pool).await;
                let reader_pid = pid(&mut reader).await;
                decide_run_in_tx(
                    &mut reader,
                    id,
                    *f.decider.as_uuid(),
                    "APPROVE",
                    Some("검토"),
                )
                .await
                .unwrap();
                let mut writer = arm(&f.pool).await;
                let writer_pid = pid(&mut writer).await;
                let writer_f = f.clone();
                let task = tokio::spawn(async move {
                    change_identity(&mut writer, kind, &writer_f).await;
                    writer.rollback().await.unwrap();
                });
                wait_blocked(&owner, writer_pid, reader_pid).await;
                assert!(!task.is_finished());
                if commit {
                    reader.commit().await.unwrap();
                } else {
                    reader.rollback().await.unwrap();
                }
                tokio::time::timeout(Duration::from_secs(20), task)
                    .await
                    .unwrap()
                    .unwrap();
                if commit {
                    let status: String =
                        sqlx::query_scalar("SELECT status FROM payroll_draft_runs WHERE id=$1")
                            .bind(id)
                            .fetch_one(&owner)
                            .await
                            .unwrap();
                    assert_eq!(status, "APPROVED");
                    let mut after = census(&owner).await;
                    // Direct shared owner does not claim canonical receipts/audit.
                    let before_runs = before.get("payroll_draft_runs").unwrap().clone();
                    let after_runs = after
                        .insert("payroll_draft_runs".to_owned(), before_runs)
                        .unwrap();
                    assert_ne!(after_runs, before["payroll_draft_runs"]);
                    assert_eq!(after, before, "no identity/receipt/audit changes");
                } else {
                    assert_eq!(census(&owner).await, before);
                }
            }
        }
    }

    #[sqlx::test(migrations = "../../platform/db/migrations")]
    async fn identity_writer_winning_first_cannot_leave_a_stale_approval(owner: PgPool) {
        // Fresh fixture per schema is not needed: restore only the disposable
        // identity writer after each case, using the original immutable row.
        let f = std::sync::Arc::new(fixture(&owner, false).await);
        for kind in ["user", "binding"] {
            let id = submitted(&owner, &f).await;
            let original: Value = sqlx::query_scalar("SELECT to_jsonb(b) FROM employee_person_bindings b WHERE org_id=$1 AND employee_id=$2")
                .bind(ORG).bind(f.employees[1]).fetch_one(&owner).await.unwrap();
            let before = census(&owner).await;
            let mut writer = arm(&f.pool).await;
            let writer_pid = pid(&mut writer).await;
            change_identity(&mut writer, kind, &f).await;
            let mut reader = arm(&f.pool).await;
            let reader_pid = pid(&mut reader).await;
            let actor = *f.decider.as_uuid();
            let task = tokio::spawn(async move {
                let result =
                    decide_run_in_tx(&mut reader, id, actor, "APPROVE", Some("검토")).await;
                assert!(
                    matches!(result, Err(LifecycleError::SodViolation)),
                    "{result:?}"
                );
                reader.rollback().await.unwrap();
            });
            wait_blocked(&owner, reader_pid, writer_pid).await;
            writer.commit().await.unwrap();
            tokio::time::timeout(Duration::from_secs(20), task)
                .await
                .unwrap()
                .unwrap();
            let mut after = census(&owner).await;
            for table in ["users", "employee_person_bindings"] {
                after.insert(table.to_owned(), before[table].clone());
            }
            assert_eq!(
                after, before,
                "writer-first denial must not affect payroll, receipts, audit, or any unrelated row"
            );
            let status: String =
                sqlx::query_scalar("SELECT status FROM payroll_draft_runs WHERE id=$1")
                    .bind(id)
                    .fetch_one(&owner)
                    .await
                    .unwrap();
            assert_eq!(status, "SUBMITTED");
            let count: i64 = sqlx::query_scalar("SELECT count(*) FROM ont_action_command_receipts WHERE org_id=$1 AND target='payroll.decide_run'")
                .bind(ORG).fetch_one(&owner).await.unwrap();
            assert_eq!(count, 0);
            if kind == "user" {
                sqlx::query("UPDATE users SET employee_id=$1 WHERE id=$2")
                    .bind(f.employees[1])
                    .bind(*f.decider.as_uuid())
                    .execute(&owner)
                    .await
                    .unwrap();
            } else {
                let mut restore = owner.begin().await.unwrap();
                sqlx::query(
                    "DELETE FROM employee_person_bindings WHERE org_id=$1 AND employee_id=$2",
                )
                .bind(ORG)
                .bind(f.employees[1])
                .execute(&mut *restore)
                .await
                .unwrap();
                sqlx::query("INSERT INTO employee_person_bindings SELECT * FROM jsonb_populate_record(NULL::employee_person_bindings, $1)").bind(original).execute(&mut *restore).await.unwrap();
                restore.commit().await.unwrap();
            }
        }
    }

    #[sqlx::test(migrations = "../../platform/db/migrations")]
    async fn identity_lock_timeout_rolls_back_then_canonical_retry_and_replay_succeed(
        owner: PgPool,
    ) {
        let f = fixture(&owner, false).await;
        let id = submitted(&owner, &f).await;
        let before = census(&owner).await;
        let mut holder = arm(&f.pool).await;
        sqlx::query("SELECT id FROM users WHERE id=$1 FOR UPDATE")
            .bind(*f.decider.as_uuid())
            .fetch_one(&mut *holder)
            .await
            .unwrap();
        let mut reader = arm(&f.pool).await;
        sqlx::query("SET LOCAL lock_timeout='200ms'")
            .execute(&mut *reader)
            .await
            .unwrap();
        let result = decide_run_in_tx(
            &mut reader,
            id,
            *f.decider.as_uuid(),
            "APPROVE",
            Some("검토"),
        )
        .await;
        match result {
            Err(LifecycleError::Db(console_platform_db::DbError::Sqlx(error))) => assert_eq!(
                error.as_database_error().and_then(|e| e.code()).as_deref(),
                Some("55P03")
            ),
            other => panic!("expected exact lock timeout, got {other:?}"),
        }
        reader.rollback().await.unwrap();
        holder.rollback().await.unwrap();
        assert_eq!(census(&owner).await, before);
        let command = decision(&f, id, "APPROVE");
        let receipt = execute(&f.port, command.clone()).await.unwrap();
        let after = census(&owner).await;
        assert_eq!(execute(&f.port, command).await.unwrap(), receipt);
        assert_eq!(census(&owner).await, after);
    }
}

// Append inside the EXISTING corresponding integration-test file; not a new fixture framework.
// Isolated actual runtime owner tests; fixture SQL does not provision browser business data.
#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn receipt_reader_binds_present_action_and_object_without_resealing(owner_pool: PgPool) {
    let (org, actor, _decider, port) = fixture(&owner_pool).await;
    let accepted = command(org, actor, create(Uuid::new_v4()));
    let original = execute(&port, accepted.clone()).await.unwrap();
    let receipt_before: serde_json::Value = sqlx::query_scalar(
        "SELECT to_jsonb(r) FROM ont_action_command_receipts r WHERE org_id=$1 AND command_id=$2",
    )
    .bind(*org.as_uuid())
    .bind(*accepted.command_id.as_uuid())
    .fetch_one(&owner_pool)
    .await
    .unwrap();
    for change in 0..2 {
        let mut retry = accepted.clone();
        if change == 0 {
            retry.action_key = "different_accepted_wrapper".to_owned();
        } else {
            retry.object_type_id = Uuid::from_u128(0xeeee);
        }
        let refused = execute(&port, retry).await.unwrap_err();
        assert!(
            matches!(refused, PayRunError::DigestConflict(id) if id == *accepted.command_id.as_uuid()),
            "change {change}: {refused:?}"
        );
        let receipt_after: serde_json::Value = sqlx::query_scalar("SELECT to_jsonb(r) FROM ont_action_command_receipts r WHERE org_id=$1 AND command_id=$2")
            .bind(*org.as_uuid()).bind(*accepted.command_id.as_uuid()).fetch_one(&owner_pool).await.unwrap();
        assert_eq!(
            receipt_after, receipt_before,
            "refusal rewrote stored receipt"
        );
    }
    assert_eq!(
        execute(&port, accepted).await.unwrap(),
        original,
        "unchanged command still replays"
    );
}
