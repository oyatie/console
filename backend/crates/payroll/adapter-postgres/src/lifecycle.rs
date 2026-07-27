//! Run-lifecycle writes for the payroll console (migration 0186):
//! close → calculate → exception review → SoD approval → operator-attested
//! disbursement → release-gated payslip issuance.
//!
//! Every function here takes an already-armed transaction (the REST layer
//! wraps each mutation in `mnt_platform_db::with_audits`, which binds
//! `app.current_org` before the closure runs), so Postgres RLS scopes every
//! statement. `Option::None` / [`LifecycleError::NotFound`] is returned for a
//! run that does not exist OR belongs to another org — RLS makes the two
//! indistinguishable (deny-by-omission).
//!
//! # Truthfulness invariants
//! - A per-line calculation is attempted ONLY when the readiness row says
//!   `gross_pay_source_present && nts_tax_row_status = 'VERIFIED_SOURCE_ROW'`
//!   AND the linked immutable source ledger rows
//!   (`data_import_rows.canonical_row.payroll`) actually carry the figures.
//!   Anything else records a truthful blocker; income tax is never estimated.
//! - Stored calculations are always drafts (`payable = FALSE`); nothing here
//!   flips that flag — only the 노무사/세무사 release gate charter may.
//! - Disbursement statuses are operator attestations (no bank API exists).
//! - Payslip issuance is hard-gated by
//!   [`mnt_payroll_domain::validate_release_gate`] over the release-gate
//!   record registered in `payroll_draft_runs.legal_basis.release_gate`;
//!   an unregistered gate is a 409, never a fake payslip.

use mnt_kernel_core::KernelError;
use mnt_payroll_domain::{
    DeductionCode, GoldenPayrollCase, LineCalculationInput, PayrollReleaseGateInput,
    ProfessionalReviewerKind, ProfessionalValidation, VerifiedNtsTaxRow, build_line_calculation,
    payroll_sources_verified_on, validate_release_gate,
};
use mnt_platform_db::DbError;
use serde::Serialize;
use serde_json::{Value, json};
use sqlx::{Postgres, Row, Transaction};
use time::format_description::well_known::Iso8601;
use time::{Date, OffsetDateTime};
use uuid::Uuid;

const DEFAULT_LIMIT: i64 = 100;
const MAX_LIMIT: i64 = 500;
/// Cap on per-check blocking refs returned by the preflight (the note carries
/// the full count).
const PREFLIGHT_REF_CAP: i64 = 50;

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct PreflightCheck {
    pub key: &'static str,
    pub label_ko: &'static str,
    pub ok: bool,
    pub warn: bool,
    pub note: Option<String>,
    pub blocking_refs: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClosePreflight {
    pub checks: Vec<PreflightCheck>,
    pub can_close: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct RunCalcSummary {
    pub version: i32,
    pub calculated_at: OffsetDateTime,
    pub calculated_lines: i64,
    pub blocked_lines: i64,
    /// Always `false` until the 노무사/세무사 release gate flips stored rows.
    pub payable: bool,
    pub kernel_rate_table: String,
    /// `None` unless EVERY line of the run calculated — never a partial sum
    /// presented as a total.
    pub total_net_won: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PayrollException {
    pub id: Uuid,
    pub run_id: Uuid,
    pub line_id: Option<Uuid>,
    pub employee_id: Option<Uuid>,
    pub employee_display_name: String,
    pub kind: String,
    pub severity: String,
    pub amount_delta_won: Option<i64>,
    pub summary_ko: String,
    pub detail: Value,
    pub linked_refs: Value,
    pub status: String,
    pub resolved_by: Option<Uuid>,
    pub resolved_at: Option<OffsetDateTime>,
    pub resolved_reason: Option<String>,
    pub carried_from_run_id: Option<Uuid>,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExceptionPage {
    pub items: Vec<PayrollException>,
    pub total: i64,
    pub open: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Disbursement {
    pub id: Uuid,
    pub run_id: Uuid,
    pub scheduled_at: OffsetDateTime,
    pub status: String,
    pub attested_by: Option<Uuid>,
    pub attested_at: Option<OffsetDateTime>,
    pub reason: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize)]
pub struct PayslipDeliveryItem {
    pub line_id: Uuid,
    pub employee_id: Uuid,
    pub inbox_doc_id: Uuid,
    pub issued_at: OffsetDateTime,
    /// Read back from `inbox_docs.confirmed_at`. The vault currently only
    /// permits receipt confirmation on `legal_notice` documents, so payslip
    /// acknowledgement stays `None` until that vault capability lands —
    /// reported truthfully rather than simulated.
    pub acknowledged_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PayslipDeliverySummary {
    pub run_id: Uuid,
    pub issued: i64,
    pub acknowledged: i64,
    pub items: Vec<PayslipDeliveryItem>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// One deliverable line for payslip issuance: the latest stored calculation
/// joined to the employee's linked user account.
#[derive(Debug, Clone)]
pub struct IssuanceLine {
    pub line_id: Uuid,
    pub employee_id: Uuid,
    pub employee_display_name: String,
    pub recipient_user_id: Uuid,
    pub version: i32,
    pub gross_won: i64,
    pub deductions: Value,
    pub total_deductions_won: i64,
    pub net_won: i64,
    pub tax_table_version: String,
}

#[derive(Debug, Clone)]
pub struct RunHead {
    pub id: Uuid,
    pub period_start: Date,
    pub period_end: Date,
    pub source_label: String,
    pub status: String,
    pub submitted_by: Option<Uuid>,
    pub legal_basis: Value,
}

#[derive(Debug, Clone)]
pub struct CalculationOutcome {
    pub version: i32,
    pub calculated_lines: i64,
    pub blocked_lines: i64,
    pub exceptions_created: i64,
}

// ---------------------------------------------------------------------------
// Error surface
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum LifecycleError {
    /// Not found or other-org (RLS-indistinguishable).
    #[error("payroll run not found")]
    NotFound,
    #[error("attendance close preflight is blocked")]
    PreflightBlocked(ClosePreflight),
    #[error("{0}")]
    InvalidState(String),
    #[error("{0} exceptions are still open")]
    ExceptionsOpen(i64),
    #[error("the decider must not be the submitter (separation of duties)")]
    SodViolation,
    #[error("exception is already resolved")]
    AlreadyResolved,
    #[error("{0}")]
    InvalidTransition(String),
    #[error("payslip release gate is not satisfied: {0}")]
    LegalGate(String),
    #[error("{0}")]
    Validation(String),
    #[error(transparent)]
    Db(#[from] DbError),
}

impl From<sqlx::Error> for LifecycleError {
    fn from(value: sqlx::Error) -> Self {
        Self::Db(DbError::Sqlx(value))
    }
}

impl From<KernelError> for LifecycleError {
    fn from(value: KernelError) -> Self {
        Self::Validation(value.message)
    }
}

impl From<LifecycleError> for crate::PgPayrollError {
    fn from(value: LifecycleError) -> Self {
        match value {
            LifecycleError::Db(err) => Self::Db(err),
            LifecycleError::NotFound => Self::Domain(KernelError::not_found("run not found")),
            other => Self::Domain(KernelError::internal(other.to_string())),
        }
    }
}

// ---------------------------------------------------------------------------
// Run head loading
// ---------------------------------------------------------------------------

async fn run_head(
    tx: &mut Transaction<'_, Postgres>,
    run_id: Uuid,
    for_update: bool,
) -> Result<Option<RunHead>, LifecycleError> {
    let sql_locked = "SELECT id, period_start, period_end, source_label, status, submitted_by, \
         legal_basis FROM payroll_draft_runs WHERE id = $1 FOR UPDATE";
    let sql_plain = "SELECT id, period_start, period_end, source_label, status, submitted_by, \
         legal_basis FROM payroll_draft_runs WHERE id = $1";
    let row = sqlx::query(if for_update { sql_locked } else { sql_plain })
        .bind(run_id)
        .fetch_optional(tx.as_mut())
        .await?;
    row.map(|row| {
        Ok(RunHead {
            id: row.try_get("id")?,
            period_start: row.try_get("period_start")?,
            period_end: row.try_get("period_end")?,
            source_label: row.try_get("source_label")?,
            status: row.try_get("status")?,
            submitted_by: row.try_get("submitted_by")?,
            legal_basis: row.try_get("legal_basis")?,
        })
    })
    .transpose()
}

fn invalid_state(action: &str, status: &str) -> LifecycleError {
    LifecycleError::InvalidState(format!("cannot {action} a run in status {status}"))
}

// ---------------------------------------------------------------------------
// Close preflight + close
// ---------------------------------------------------------------------------

/// The three §4-29 preflight checks derivable from real tables today:
/// attendance-material coverage per line, an active payroll period lock
/// covering the run period, and (soft warn) pending leave requests
/// overlapping the period. `can_close` is the AND of the hard checks.
pub async fn close_preflight_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    run_id: Uuid,
) -> Result<Option<ClosePreflight>, LifecycleError> {
    let Some(run) = run_head(tx, run_id, false).await? else {
        return Ok(None);
    };
    Ok(Some(preflight_for(tx, &run).await?))
}

async fn preflight_for(
    tx: &mut Transaction<'_, Postgres>,
    run: &RunHead,
) -> Result<ClosePreflight, LifecycleError> {
    // 1. Every roster line must carry attendance material (source rows or
    //    direct events). Lines without any are the blocking refs (fix-links).
    let missing_total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM payroll_draft_lines \
         WHERE run_id = $1 AND attendance_source_row_count = 0 AND attendance_event_count = 0",
    )
    .bind(run.id)
    .fetch_one(tx.as_mut())
    .await?;
    let missing_refs: Vec<Uuid> = sqlx::query_scalar(
        "SELECT id FROM payroll_draft_lines \
         WHERE run_id = $1 AND attendance_source_row_count = 0 AND attendance_event_count = 0 \
         ORDER BY employee_company, employee_display_name LIMIT $2",
    )
    .bind(run.id)
    .bind(PREFLIGHT_REF_CAP)
    .fetch_all(tx.as_mut())
    .await?;
    let attendance = PreflightCheck {
        key: "attendance_material",
        label_ko: "근태 원천 확보",
        ok: missing_total == 0,
        warn: false,
        note: (missing_total > 0).then(|| format!("근태 원천 누락 {missing_total}명")),
        blocking_refs: missing_refs,
    };

    // 2. An active payroll period lock must cover the whole run period.
    let lock_active: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM period_locks \
         WHERE domain = 'payroll' AND unlocked_at IS NULL \
           AND period_start <= $1 AND period_end >= $2)",
    )
    .bind(run.period_start)
    .bind(run.period_end)
    .fetch_one(tx.as_mut())
    .await?;
    let period_lock = PreflightCheck {
        key: "period_lock",
        label_ko: "급여 기간 동결",
        ok: lock_active,
        warn: false,
        note: (!lock_active).then(|| "급여 동결창(기간 잠금) 미설정".to_owned()),
        blocking_refs: Vec::new(),
    };

    // 3. Soft warn: pending leave requests overlapping the period may still
    //    change attendance figures. Never blocks — surfaced for the attestor.
    let pending_leave: Vec<Uuid> = sqlx::query_scalar(
        "SELECT id FROM leave_requests \
         WHERE status = 'pending' AND start_date <= $1 AND end_date >= $2 \
         ORDER BY start_date LIMIT $3",
    )
    .bind(run.period_end)
    .bind(run.period_start)
    .bind(PREFLIGHT_REF_CAP)
    .fetch_all(tx.as_mut())
    .await?;
    let leave = PreflightCheck {
        key: "pending_leave",
        label_ko: "미결 휴가 신청",
        ok: true,
        warn: !pending_leave.is_empty(),
        note: (!pending_leave.is_empty()).then(|| format!("미결 휴가 {}건", pending_leave.len())),
        blocking_refs: pending_leave,
    };

    let can_close = attendance.ok && period_lock.ok;
    Ok(ClosePreflight {
        checks: vec![attendance, period_lock, leave],
        can_close,
    })
}

/// Statuses a run may be closed from. The three 0074-era pre-close states are
/// all admissible so real staged rows can enter the lifecycle.
const CLOSEABLE: [&str; 3] = ["STAGED", "BLOCKED_LEGAL_GATE", "READY_FOR_REVIEW"];

pub async fn close_attendance_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    run_id: Uuid,
    actor: Uuid,
    now: OffsetDateTime,
) -> Result<Value, LifecycleError> {
    let Some(run) = run_head(tx, run_id, true).await? else {
        return Err(LifecycleError::NotFound);
    };
    if !CLOSEABLE.contains(&run.status.as_str()) {
        return Err(invalid_state("close attendance for", &run.status));
    }
    let preflight = preflight_for(tx, &run).await?;
    if !preflight.can_close {
        return Err(LifecycleError::PreflightBlocked(preflight));
    }

    let receipt = json!({
        "checks": serde_json::to_value(&preflight.checks).map_err(json_internal)?,
        "attested_by": actor,
        "attested_at": now.format(&Iso8601::DEFAULT).map_err(|err| {
            LifecycleError::Validation(format!("failed to format attestation time: {err}"))
        })?,
    });
    sqlx::query(
        "UPDATE payroll_draft_runs \
         SET status = 'ATTENDANCE_CLOSED', close_receipt = $2, updated_at = now() \
         WHERE id = $1",
    )
    .bind(run_id)
    .bind(&receipt)
    .execute(tx.as_mut())
    .await?;
    Ok(receipt)
}

fn json_internal(err: serde_json::Error) -> LifecycleError {
    LifecycleError::Validation(format!("failed to serialize receipt: {err}"))
}

// ---------------------------------------------------------------------------
// Calculation
// ---------------------------------------------------------------------------

#[derive(PartialEq, Eq)]
struct SourceAmounts {
    gross_won: i64,
    pension_standard_monthly_income_won: Option<i64>,
    tax_row: VerifiedNtsTaxRow,
}

/// Extract the payroll figures from an imported source ledger row's
/// `canonical_row`. The canonical key contract (documented in the module
/// evidence): `payroll.monthly_gross_pay_won`,
/// `payroll.pension_standard_monthly_income_won` (optional),
/// `payroll.nts_tax_row.{table_version, monthly_income_tax_won,
/// local_income_tax_won}` — all figures verbatim from the verified source.
/// Deterministically choose the payroll figures among a line's linked
/// canonical rows. Byte-identical duplicates (re-imports of the same source)
/// are fine; two DIFFERENT figure sets for one line mean the figure to pay is
/// ambiguous — a truthful blocker, never an arbitrary row-order pick.
fn select_source_amounts(canonical_rows: &[Value]) -> Result<SourceAmounts, &'static str> {
    let mut found = canonical_rows.iter().filter_map(extract_source_amounts);
    let Some(first) = found.next() else {
        return Err("SOURCE_AMOUNTS_NOT_MATERIALIZED");
    };
    if found.any(|other| other != first) {
        return Err("SOURCE_AMOUNTS_CONFLICTING");
    }
    Ok(first)
}

fn extract_source_amounts(canonical_row: &Value) -> Option<SourceAmounts> {
    let payroll = canonical_row.get("payroll")?;
    let tax = payroll.get("nts_tax_row")?;
    Some(SourceAmounts {
        gross_won: payroll.get("monthly_gross_pay_won")?.as_i64()?,
        pension_standard_monthly_income_won: payroll
            .get("pension_standard_monthly_income_won")
            .and_then(Value::as_i64),
        tax_row: VerifiedNtsTaxRow {
            table_version: tax.get("table_version")?.as_str()?.to_owned(),
            monthly_income_tax_won: tax.get("monthly_income_tax_won")?.as_i64()?,
            local_income_tax_won: tax.get("local_income_tax_won")?.as_i64()?,
        },
    })
}

const fn deduction_code_str(code: DeductionCode) -> &'static str {
    match code {
        DeductionCode::NationalPension => "NATIONAL_PENSION",
        DeductionCode::HealthInsurance => "HEALTH_INSURANCE",
        DeductionCode::LongTermCare => "LONG_TERM_CARE",
        DeductionCode::EmploymentInsurance => "EMPLOYMENT_INSURANCE",
        DeductionCode::IncomeTax => "INCOME_TAX",
        DeductionCode::LocalIncomeTax => "LOCAL_INCOME_TAX",
    }
}

pub async fn calculate_run_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    run_id: Uuid,
) -> Result<CalculationOutcome, LifecycleError> {
    let Some(run) = run_head(tx, run_id, true).await? else {
        return Err(LifecycleError::NotFound);
    };
    if run.status != "ATTENDANCE_CLOSED" {
        return Err(invalid_state("calculate", &run.status));
    }
    // The transient state is real: any concurrent reader inside this
    // transaction window sees CALCULATING.
    sqlx::query(
        "UPDATE payroll_draft_runs SET status = 'CALCULATING', updated_at = now() WHERE id = $1",
    )
    .bind(run_id)
    .execute(tx.as_mut())
    .await?;

    let version: i32 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(version), 0) + 1 FROM payroll_line_calculations WHERE run_id = $1",
    )
    .bind(run_id)
    .fetch_one(tx.as_mut())
    .await?;

    let lines = sqlx::query(
        "SELECT id, employee_id, employee_display_name, gross_pay_source_present, \
                nts_tax_row_status, overtime_hours::float8 AS overtime_hours, \
                source_data_import_row_ids \
         FROM payroll_draft_lines WHERE run_id = $1 \
         ORDER BY employee_company, employee_display_name, id FOR UPDATE",
    )
    .bind(run_id)
    .fetch_all(tx.as_mut())
    .await?;

    let mut calculated: i64 = 0;
    let mut blocked: i64 = 0;
    let mut exceptions_created: i64 = 0;

    for row in &lines {
        let line_id: Uuid = row.try_get("id")?;
        let employee_id: Option<Uuid> = row.try_get("employee_id")?;
        let display_name: String = row.try_get("employee_display_name")?;
        let gross_present: bool = row.try_get("gross_pay_source_present")?;
        let nts_status: String = row.try_get("nts_tax_row_status")?;
        let overtime_hours: Option<f64> = row.try_get("overtime_hours")?;
        let import_row_ids: Vec<Uuid> = row.try_get("source_data_import_row_ids")?;

        let mut blockers: Vec<String> = Vec::new();
        if !gross_present {
            blockers.push("GROSS_PAY_SOURCE_MISSING".to_owned());
        }
        if nts_status != "VERIFIED_SOURCE_ROW" {
            blockers.push("NTS_TAX_ROW_UNVERIFIED".to_owned());
        }

        let mut amounts: Option<SourceAmounts> = None;
        if blockers.is_empty() {
            let canonical_rows: Vec<Value> = if import_row_ids.is_empty() {
                Vec::new()
            } else {
                sqlx::query_scalar("SELECT canonical_row FROM data_import_rows WHERE id = ANY($1)")
                    .bind(&import_row_ids)
                    .fetch_all(tx.as_mut())
                    .await?
            };
            // The readiness flags promised a verified source; if the linked
            // ledger rows carry no payroll figures (NOT_MATERIALIZED) or two
            // different figure sets (CONFLICTING), that is a truthful blocker
            // — never an estimate, never an arbitrary pick.
            match select_source_amounts(&canonical_rows) {
                Ok(selected) => amounts = Some(selected),
                Err(blocker) => blockers.push(blocker.to_owned()),
            }
        }

        if let Some(amounts) = amounts {
            match build_line_calculation(LineCalculationInput {
                pay_date: run.period_end,
                gross_won: amounts.gross_won,
                pension_standard_monthly_income_won: amounts.pension_standard_monthly_income_won,
                tax_row: amounts.tax_row,
            }) {
                Ok(calc) => {
                    let deductions: Vec<Value> = calc
                        .lines
                        .iter()
                        .map(|line| {
                            json!({
                                "code": deduction_code_str(line.code),
                                "label_ko": line.label_ko,
                                "amount_won": line.amount_won,
                                "source_url": line.source_url,
                            })
                        })
                        .collect();
                    sqlx::query(
                        "INSERT INTO payroll_line_calculations \
                         (org_id, run_id, line_id, version, gross_won, deductions, \
                          total_deductions_won, net_won, tax_table_version) \
                         SELECT l.org_id, l.run_id, l.id, $2, $3, $4, $5, $6, $7 \
                         FROM payroll_draft_lines l WHERE l.id = $1",
                    )
                    .bind(line_id)
                    .bind(version)
                    .bind(calc.gross_won)
                    .bind(Value::Array(deductions))
                    .bind(calc.total_employee_deductions_won)
                    .bind(calc.net_won)
                    .bind(&calc.tax_table_version)
                    .execute(tx.as_mut())
                    .await?;
                    calculated += 1;
                    sqlx::query(
                        "UPDATE payroll_draft_lines SET calculation_status = 'READY_FOR_REVIEW', \
                         blockers = '[]'::jsonb, updated_at = now() WHERE id = $1",
                    )
                    .bind(line_id)
                    .execute(tx.as_mut())
                    .await?;
                }
                Err(err) => {
                    blockers.push(format!("CALCULATION_REJECTED: {}", err.message));
                    blocked += 1;
                    write_line_blockers(tx, line_id, &blockers).await?;
                }
            }
        } else {
            blocked += 1;
            write_line_blockers(tx, line_id, &blockers).await?;
        }

        // Real attendance-derived signal: overtime hours on the roster line
        // mean the 연장수당 must be confirmed against the gross source. The
        // delta is NOT derivable from a verified source, so it stays NULL.
        if overtime_hours.unwrap_or(0.0) > 0.0 {
            let inserted = sqlx::query(
                "INSERT INTO payroll_run_exceptions \
                 (org_id, run_id, line_id, employee_id, employee_display_name, kind, severity, \
                  amount_delta_won, summary_ko, detail, linked_refs) \
                 SELECT l.org_id, l.run_id, l.id, l.employee_id, l.employee_display_name, \
                        'OVERTIME_ALLOWANCE', 'warn', NULL, $2, $3, $4 \
                 FROM payroll_draft_lines l WHERE l.id = $1 \
                 ON CONFLICT (org_id, run_id, line_id, kind) WHERE line_id IS NOT NULL \
                 DO NOTHING",
            )
            .bind(line_id)
            .bind(format!(
                "연장 {:.1}시간 — 연장수당 반영 확인 필요",
                overtime_hours.unwrap_or(0.0)
            ))
            .bind(json!({ "overtime_hours": overtime_hours }))
            .bind(json!([
                { "kind": "payroll_line", "code": display_name, "id": line_id },
                { "kind": "employee", "code": display_name, "id": employee_id },
            ]))
            .execute(tx.as_mut())
            .await?;
            exceptions_created += i64::try_from(inserted.rows_affected()).unwrap_or(0);
        }
    }

    // Carry-forward: HELD exceptions from the previous run of the same series
    // are re-opened on this run (carried_from_run_id keeps the lineage).
    let carried = sqlx::query(
        "INSERT INTO payroll_run_exceptions \
         (org_id, run_id, line_id, employee_id, employee_display_name, kind, severity, \
          amount_delta_won, summary_ko, detail, linked_refs, carried_from_run_id) \
         SELECT e.org_id, $1, NULL, e.employee_id, e.employee_display_name, e.kind, \
                e.severity, e.amount_delta_won, e.summary_ko, e.detail, e.linked_refs, e.run_id \
         FROM payroll_run_exceptions e \
         WHERE e.status = 'HELD' \
           AND e.run_id = (SELECT r.id FROM payroll_draft_runs r \
                           WHERE r.source_label = $2 AND r.period_end < $3 \
                           ORDER BY r.period_end DESC LIMIT 1) \
           AND NOT EXISTS (SELECT 1 FROM payroll_run_exceptions x \
                           WHERE x.run_id = $1 AND x.carried_from_run_id = e.run_id \
                             AND x.kind = e.kind \
                             AND x.employee_id IS NOT DISTINCT FROM e.employee_id)",
    )
    .bind(run_id)
    .bind(&run.source_label)
    .bind(run.period_start)
    .execute(tx.as_mut())
    .await?;
    exceptions_created += i64::try_from(carried.rows_affected()).unwrap_or(0);

    sqlx::query(
        "UPDATE payroll_draft_runs SET status = 'CALCULATED', updated_at = now() WHERE id = $1",
    )
    .bind(run_id)
    .execute(tx.as_mut())
    .await?;

    Ok(CalculationOutcome {
        version,
        calculated_lines: calculated,
        blocked_lines: blocked,
        exceptions_created,
    })
}

async fn write_line_blockers(
    tx: &mut Transaction<'_, Postgres>,
    line_id: Uuid,
    blockers: &[String],
) -> Result<(), LifecycleError> {
    sqlx::query(
        "UPDATE payroll_draft_lines SET calculation_status = 'BLOCKED_LEGAL_GATE', \
         blockers = $2, updated_at = now() WHERE id = $1",
    )
    .bind(line_id)
    .bind(serde_json::to_value(blockers).map_err(json_internal)?)
    .execute(tx.as_mut())
    .await?;
    Ok(())
}

/// Aggregate of the latest calculation version for a run, or `None` if the
/// run was never calculated.
pub async fn latest_calc_summary_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    run_id: Uuid,
    lines_total: i64,
) -> Result<Option<RunCalcSummary>, LifecycleError> {
    let row = sqlx::query(
        "SELECT version, MAX(created_at) AS calculated_at, COUNT(*)::BIGINT AS calculated_lines, \
                BOOL_AND(payable) AS payable, SUM(net_won)::BIGINT AS net_sum \
         FROM payroll_line_calculations \
         WHERE run_id = $1 \
           AND version = (SELECT MAX(version) FROM payroll_line_calculations WHERE run_id = $1) \
         GROUP BY version",
    )
    .bind(run_id)
    .fetch_optional(tx.as_mut())
    .await?;
    row.map(|row| {
        let calculated_lines: i64 = row.try_get("calculated_lines")?;
        let net_sum: Option<i64> = row.try_get("net_sum")?;
        Ok(RunCalcSummary {
            version: row.try_get("version")?,
            calculated_at: row.try_get("calculated_at")?,
            calculated_lines,
            blocked_lines: (lines_total - calculated_lines).max(0),
            payable: row.try_get("payable")?,
            kernel_rate_table: format!("statutory-rates-{}", payroll_sources_verified_on()),
            total_net_won: if calculated_lines == lines_total {
                net_sum
            } else {
                None
            },
        })
    })
    .transpose()
}

// ---------------------------------------------------------------------------
// Exceptions
// ---------------------------------------------------------------------------

fn exception_from_row(row: &sqlx::postgres::PgRow) -> Result<PayrollException, LifecycleError> {
    Ok(PayrollException {
        id: row.try_get("id")?,
        run_id: row.try_get("run_id")?,
        line_id: row.try_get("line_id")?,
        employee_id: row.try_get("employee_id")?,
        employee_display_name: row.try_get("employee_display_name")?,
        kind: row.try_get("kind")?,
        severity: row.try_get("severity")?,
        amount_delta_won: row.try_get("amount_delta_won")?,
        summary_ko: row.try_get("summary_ko")?,
        detail: row.try_get("detail")?,
        linked_refs: row.try_get("linked_refs")?,
        status: row.try_get("status")?,
        resolved_by: row.try_get("resolved_by")?,
        resolved_at: row.try_get("resolved_at")?,
        resolved_reason: row.try_get("resolved_reason")?,
        carried_from_run_id: row.try_get("carried_from_run_id")?,
        created_at: row.try_get("created_at")?,
    })
}

/// Unresolved rows first (the single sort point the design mandates), then
/// severity danger→warn→info, then age.
pub async fn list_exceptions_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    run_id: Uuid,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Option<ExceptionPage>, LifecycleError> {
    if run_head(tx, run_id, false).await?.is_none() {
        return Ok(None);
    }
    let limit = limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let offset = offset.unwrap_or(0).max(0);
    let total: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM payroll_run_exceptions WHERE run_id = $1")
            .bind(run_id)
            .fetch_one(tx.as_mut())
            .await?;
    let open: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM payroll_run_exceptions WHERE run_id = $1 AND status = 'OPEN'",
    )
    .bind(run_id)
    .fetch_one(tx.as_mut())
    .await?;
    let rows = sqlx::query(
        "SELECT id, run_id, line_id, employee_id, employee_display_name, kind, severity, \
                amount_delta_won, summary_ko, detail, linked_refs, status, resolved_by, \
                resolved_at, resolved_reason, carried_from_run_id, created_at \
         FROM payroll_run_exceptions WHERE run_id = $1 \
         ORDER BY (status <> 'OPEN'), \
                  CASE severity WHEN 'danger' THEN 0 WHEN 'warn' THEN 1 ELSE 2 END, \
                  created_at, id \
         LIMIT $2 OFFSET $3",
    )
    .bind(run_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(tx.as_mut())
    .await?;
    let items = rows
        .iter()
        .map(exception_from_row)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Some(ExceptionPage {
        items,
        total,
        open,
        limit,
        offset,
    }))
}

pub async fn resolve_exception_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    run_id: Uuid,
    exception_id: Uuid,
    actor: Uuid,
    action: &str,
    reason: Option<&str>,
) -> Result<PayrollException, LifecycleError> {
    let Some(run) = run_head(tx, run_id, true).await? else {
        return Err(LifecycleError::NotFound);
    };
    if run.status != "CALCULATED" {
        return Err(invalid_state("resolve exceptions for", &run.status));
    }
    let new_status = match action {
        "CONFIRM" => "CONFIRMED",
        "HOLD" => {
            if reason.is_none_or(|r| r.trim().is_empty()) {
                return Err(LifecycleError::Validation(
                    "a reason is required to hold an exception to the next run".to_owned(),
                ));
            }
            "HELD"
        }
        other => {
            return Err(LifecycleError::Validation(format!(
                "unknown resolve action: {other}"
            )));
        }
    };
    let current: Option<String> = sqlx::query_scalar(
        "SELECT status FROM payroll_run_exceptions WHERE id = $1 AND run_id = $2 FOR UPDATE",
    )
    .bind(exception_id)
    .bind(run_id)
    .fetch_optional(tx.as_mut())
    .await?;
    match current.as_deref() {
        None => return Err(LifecycleError::NotFound),
        Some("OPEN") => {}
        Some(_) => return Err(LifecycleError::AlreadyResolved),
    }
    let row = sqlx::query(
        "UPDATE payroll_run_exceptions \
         SET status = $3, resolved_by = $4, resolved_at = now(), resolved_reason = $5, \
             updated_at = now() \
         WHERE id = $1 AND run_id = $2 \
         RETURNING id, run_id, line_id, employee_id, employee_display_name, kind, severity, \
                   amount_delta_won, summary_ko, detail, linked_refs, status, resolved_by, \
                   resolved_at, resolved_reason, carried_from_run_id, created_at",
    )
    .bind(exception_id)
    .bind(run_id)
    .bind(new_status)
    .bind(actor)
    .bind(reason)
    .fetch_one(tx.as_mut())
    .await?;
    exception_from_row(&row)
}

pub async fn exception_counts_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    run_id: Uuid,
) -> Result<(i64, i64), LifecycleError> {
    let row = sqlx::query(
        "SELECT COUNT(*)::BIGINT AS total, \
                COUNT(*) FILTER (WHERE status = 'OPEN')::BIGINT AS open \
         FROM payroll_run_exceptions WHERE run_id = $1",
    )
    .bind(run_id)
    .fetch_one(tx.as_mut())
    .await?;
    Ok((row.try_get("open")?, row.try_get("total")?))
}

// ---------------------------------------------------------------------------
// Submission / SoD decision / withdrawal
// ---------------------------------------------------------------------------

pub async fn submit_run_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    run_id: Uuid,
    actor: Uuid,
) -> Result<(), LifecycleError> {
    let Some(run) = run_head(tx, run_id, true).await? else {
        return Err(LifecycleError::NotFound);
    };
    if run.status != "CALCULATED" {
        return Err(invalid_state("submit", &run.status));
    }
    let (open, _) = exception_counts_in_tx(tx, run_id).await?;
    if open > 0 {
        return Err(LifecycleError::ExceptionsOpen(open));
    }
    sqlx::query(
        "UPDATE payroll_draft_runs \
         SET status = 'SUBMITTED', submitted_by = $2, submitted_at = now(), updated_at = now() \
         WHERE id = $1",
    )
    .bind(run_id)
    .bind(actor)
    .execute(tx.as_mut())
    .await?;
    Ok(())
}

pub async fn decide_run_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    run_id: Uuid,
    actor: Uuid,
    decision: &str,
    reason: Option<&str>,
) -> Result<(), LifecycleError> {
    let Some(run) = run_head(tx, run_id, true).await? else {
        return Err(LifecycleError::NotFound);
    };
    if run.status != "SUBMITTED" {
        return Err(invalid_state("decide", &run.status));
    }
    if run.submitted_by == Some(actor) {
        return Err(LifecycleError::SodViolation);
    }
    match decision {
        "APPROVE" => {
            sqlx::query(
                "UPDATE payroll_draft_runs \
                 SET status = 'APPROVED', decided_by = $2, decided_at = now(), \
                     decision_reason = $3, approved_by = $2, approved_at = now(), \
                     updated_at = now() \
                 WHERE id = $1",
            )
            .bind(run_id)
            .bind(actor)
            .bind(reason)
            .execute(tx.as_mut())
            .await?;
        }
        "REJECT" => {
            if reason.is_none_or(|r| r.trim().is_empty()) {
                return Err(LifecycleError::Validation(
                    "a reason is required to reject a payroll run".to_owned(),
                ));
            }
            sqlx::query(
                "UPDATE payroll_draft_runs \
                 SET status = 'REJECTED', decided_by = $2, decided_at = now(), \
                     decision_reason = $3, updated_at = now() \
                 WHERE id = $1",
            )
            .bind(run_id)
            .bind(actor)
            .bind(reason)
            .execute(tx.as_mut())
            .await?;
        }
        other => {
            return Err(LifecycleError::Validation(format!(
                "unknown decision: {other}"
            )));
        }
    }
    Ok(())
}

pub async fn withdraw_run_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    run_id: Uuid,
) -> Result<(), LifecycleError> {
    let Some(run) = run_head(tx, run_id, true).await? else {
        return Err(LifecycleError::NotFound);
    };
    if run.status != "REJECTED" {
        return Err(invalid_state("withdraw", &run.status));
    }
    // The rejection evidence lives in the audit trail; the columns reset so
    // the SoD CHECK admits a fresh submit/decide pair.
    sqlx::query(
        "UPDATE payroll_draft_runs \
         SET status = 'CALCULATED', submitted_by = NULL, submitted_at = NULL, \
             decided_by = NULL, decided_at = NULL, decision_reason = NULL, updated_at = now() \
         WHERE id = $1",
    )
    .bind(run_id)
    .execute(tx.as_mut())
    .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Disbursement (operator-attested; no bank API exists)
// ---------------------------------------------------------------------------

fn disbursement_from_row(row: &sqlx::postgres::PgRow) -> Result<Disbursement, LifecycleError> {
    Ok(Disbursement {
        id: row.try_get("id")?,
        run_id: row.try_get("run_id")?,
        scheduled_at: row.try_get("scheduled_at")?,
        status: row.try_get("status")?,
        attested_by: row.try_get("attested_by")?,
        attested_at: row.try_get("attested_at")?,
        reason: row.try_get("reason")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub async fn get_disbursement_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    run_id: Uuid,
) -> Result<Option<Disbursement>, LifecycleError> {
    let row = sqlx::query(
        "SELECT id, run_id, scheduled_at, status, attested_by, attested_at, reason, \
                created_at, updated_at \
         FROM payroll_disbursements WHERE run_id = $1",
    )
    .bind(run_id)
    .fetch_optional(tx.as_mut())
    .await?;
    row.as_ref().map(disbursement_from_row).transpose()
}

pub async fn schedule_disbursement_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    run_id: Uuid,
    scheduled_at: OffsetDateTime,
) -> Result<Disbursement, LifecycleError> {
    let Some(run) = run_head(tx, run_id, true).await? else {
        return Err(LifecycleError::NotFound);
    };
    if run.status != "APPROVED" {
        return Err(invalid_state("schedule disbursement for", &run.status));
    }
    let row = sqlx::query(
        "INSERT INTO payroll_disbursements (org_id, run_id, scheduled_at) \
         SELECT r.org_id, r.id, $2 FROM payroll_draft_runs r WHERE r.id = $1 \
         ON CONFLICT (run_id) DO NOTHING \
         RETURNING id, run_id, scheduled_at, status, attested_by, attested_at, reason, \
                   created_at, updated_at",
    )
    .bind(run_id)
    .bind(scheduled_at)
    .fetch_optional(tx.as_mut())
    .await?;
    let Some(row) = row else {
        return Err(LifecycleError::InvalidState(
            "a disbursement is already scheduled for this run".to_owned(),
        ));
    };
    sqlx::query(
        "UPDATE payroll_draft_runs SET status = 'DISBURSEMENT_SCHEDULED', updated_at = now() \
         WHERE id = $1",
    )
    .bind(run_id)
    .execute(tx.as_mut())
    .await?;
    disbursement_from_row(&row)
}

pub async fn attest_disbursement_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    run_id: Uuid,
    actor: Uuid,
    new_status: &str,
    reason: Option<&str>,
) -> Result<Disbursement, LifecycleError> {
    let Some(run) = run_head(tx, run_id, true).await? else {
        return Err(LifecycleError::NotFound);
    };
    let row = sqlx::query(
        "SELECT id, run_id, scheduled_at, status, attested_by, attested_at, reason, \
                created_at, updated_at \
         FROM payroll_disbursements WHERE run_id = $1 FOR UPDATE",
    )
    .bind(run_id)
    .fetch_optional(tx.as_mut())
    .await?;
    let Some(row) = row else {
        return Err(LifecycleError::InvalidState(
            "no disbursement is scheduled for this run".to_owned(),
        ));
    };
    let current = disbursement_from_row(&row)?;
    let allowed = matches!(
        (current.status.as_str(), new_status),
        ("SCHEDULED", "SUBMITTED_TO_BANK")
            | ("SUBMITTED_TO_BANK", "PAID")
            | ("SUBMITTED_TO_BANK", "FAILED")
            | ("FAILED", "SCHEDULED")
    );
    if !allowed {
        return Err(LifecycleError::InvalidTransition(format!(
            "cannot attest {new_status} from {}",
            current.status
        )));
    }
    if new_status == "FAILED" && reason.is_none_or(|r| r.trim().is_empty()) {
        return Err(LifecycleError::Validation(
            "a reason is required to attest a failed disbursement".to_owned(),
        ));
    }
    let row = sqlx::query(
        "UPDATE payroll_disbursements \
         SET status = $2, attested_by = $3, attested_at = now(), \
             reason = CASE WHEN $2 = 'FAILED' THEN $4 ELSE NULL END, updated_at = now() \
         WHERE run_id = $1 \
         RETURNING id, run_id, scheduled_at, status, attested_by, attested_at, reason, \
                   created_at, updated_at",
    )
    .bind(run_id)
    .bind(new_status)
    .bind(actor)
    .bind(reason)
    .fetch_one(tx.as_mut())
    .await?;
    if new_status == "PAID" {
        if run.status != "DISBURSEMENT_SCHEDULED" {
            return Err(invalid_state("mark paid", &run.status));
        }
        sqlx::query(
            "UPDATE payroll_draft_runs SET status = 'PAID', updated_at = now() WHERE id = $1",
        )
        .bind(run_id)
        .execute(tx.as_mut())
        .await?;
    }
    disbursement_from_row(&row)
}

// ---------------------------------------------------------------------------
// Payslip issuance (release-gated) + delivery readback
// ---------------------------------------------------------------------------

/// Parse the release-gate record registered at
/// `payroll_draft_runs.legal_basis.release_gate` and validate it with the
/// domain kernel. An absent or invalid record is a [`LifecycleError::LegalGate`].
pub fn validate_run_release_gate(legal_basis: &Value) -> Result<(), LifecycleError> {
    let Some(gate) = legal_basis.get("release_gate") else {
        return Err(LifecycleError::LegalGate(
            "no release-gate record is registered for this run (노무사/세무사 검증 필요)"
                .to_owned(),
        ));
    };
    let input = parse_release_gate(gate)?;
    validate_release_gate(&input).map_err(|err| LifecycleError::LegalGate(err.message))
}

fn gate_str(gate: &Value, key: &str) -> Result<String, LifecycleError> {
    gate.get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| LifecycleError::LegalGate(format!("release-gate record is missing {key}")))
}

fn parse_release_gate(gate: &Value) -> Result<PayrollReleaseGateInput, LifecycleError> {
    let rate_table_version = gate_str(gate, "rate_table_version")?;
    let official_source_urls = gate
        .get("official_source_urls")
        .and_then(Value::as_array)
        .map(|urls| {
            urls.iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let golden_cases = gate
        .get("golden_cases")
        .and_then(Value::as_array)
        .map(|cases| {
            cases
                .iter()
                .map(|case| {
                    Ok(GoldenPayrollCase {
                        case_id: gate_str(case, "case_id")?,
                        rate_table_version: gate_str(case, "rate_table_version")?,
                        professionally_validated: case
                            .get("professionally_validated")
                            .and_then(Value::as_bool)
                            .unwrap_or(false),
                        expected_total_employee_deductions_won: case
                            .get("expected_total_employee_deductions_won")
                            .and_then(Value::as_i64)
                            .unwrap_or(0),
                    })
                })
                .collect::<Result<Vec<_>, LifecycleError>>()
        })
        .transpose()?
        .unwrap_or_default();
    let professional_validation = gate
        .get("professional_validation")
        .map(|validation| {
            let reviewer_kind = match gate_str(validation, "reviewer_kind")?.as_str() {
                "labor_attorney" => ProfessionalReviewerKind::LaborAttorney,
                "tax_accountant" => ProfessionalReviewerKind::TaxAccountant,
                other => {
                    return Err(LifecycleError::LegalGate(format!(
                        "unknown professional reviewer kind: {other}"
                    )));
                }
            };
            let reviewed_on = Date::parse(&gate_str(validation, "reviewed_on")?, &Iso8601::DEFAULT)
                .map_err(|err| {
                    LifecycleError::LegalGate(format!("invalid reviewed_on date: {err}"))
                })?;
            Ok(ProfessionalValidation {
                reviewer_kind,
                reviewed_on,
                artifact_sha256: gate_str(validation, "artifact_sha256")?,
                reviewer_reference: gate_str(validation, "reviewer_reference")?,
            })
        })
        .transpose()?;
    Ok(PayrollReleaseGateInput {
        rate_table_version,
        official_source_urls,
        golden_cases,
        professional_validation,
    })
}

/// Load the run + deliverable lines for payslip issuance. Guards status =
/// PAID and the release gate; a run with no deliverable calculated line is a
/// truthful invalid state (issuing nothing is not issuance).
pub async fn load_payslip_issuance_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    run_id: Uuid,
) -> Result<(RunHead, Vec<IssuanceLine>), LifecycleError> {
    let Some(run) = run_head(tx, run_id, false).await? else {
        return Err(LifecycleError::NotFound);
    };
    if run.status != "PAID" {
        return Err(invalid_state("issue payslips for", &run.status));
    }
    validate_run_release_gate(&run.legal_basis)?;
    let rows = sqlx::query(
        "SELECT DISTINCT ON (c.line_id) c.line_id, c.version, c.gross_won, c.deductions, \
                c.total_deductions_won, c.net_won, c.tax_table_version, \
                l.employee_id, l.employee_display_name, u.id AS recipient_user_id \
         FROM payroll_line_calculations c \
         JOIN payroll_draft_lines l ON l.id = c.line_id \
         JOIN users u ON u.employee_id = l.employee_id \
         WHERE c.run_id = $1 AND l.employee_id IS NOT NULL \
         ORDER BY c.line_id, c.version DESC",
    )
    .bind(run_id)
    .fetch_all(tx.as_mut())
    .await?;
    let lines =
        rows.iter()
            .map(|row| {
                Ok(IssuanceLine {
                    line_id: row.try_get("line_id")?,
                    employee_id: row.try_get::<Option<Uuid>, _>("employee_id")?.ok_or_else(
                        || LifecycleError::Validation("issuance line lost its employee".to_owned()),
                    )?,
                    employee_display_name: row.try_get("employee_display_name")?,
                    recipient_user_id: row.try_get("recipient_user_id")?,
                    version: row.try_get("version")?,
                    gross_won: row.try_get("gross_won")?,
                    deductions: row.try_get("deductions")?,
                    total_deductions_won: row.try_get("total_deductions_won")?,
                    net_won: row.try_get("net_won")?,
                    tax_table_version: row.try_get("tax_table_version")?,
                })
            })
            .collect::<Result<Vec<_>, LifecycleError>>()?;
    if lines.is_empty() {
        return Err(LifecycleError::InvalidState(
            "no calculated line with a linked user account exists to issue payslips for".to_owned(),
        ));
    }
    Ok((run, lines))
}

/// Record the delivered inbox docs and flip the run to ISSUED. Idempotent:
/// links insert `ON CONFLICT DO NOTHING`, so an interrupted issuance can be
/// re-driven to completion.
pub async fn record_payslip_deliveries_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    run_id: Uuid,
    deliveries: &[(Uuid, Uuid, Uuid)],
) -> Result<(), LifecycleError> {
    let Some(run) = run_head(tx, run_id, true).await? else {
        return Err(LifecycleError::NotFound);
    };
    if run.status != "PAID" {
        return Err(invalid_state("record payslip deliveries for", &run.status));
    }
    for (line_id, employee_id, inbox_doc_id) in deliveries {
        sqlx::query(
            "INSERT INTO payroll_payslip_deliveries \
             (org_id, run_id, line_id, employee_id, inbox_doc_id) \
             SELECT r.org_id, r.id, $2, $3, $4 FROM payroll_draft_runs r WHERE r.id = $1 \
             ON CONFLICT (org_id, run_id, line_id) DO NOTHING",
        )
        .bind(run_id)
        .bind(line_id)
        .bind(employee_id)
        .bind(inbox_doc_id)
        .execute(tx.as_mut())
        .await?;
    }
    sqlx::query(
        "UPDATE payroll_draft_runs SET status = 'ISSUED', updated_at = now() WHERE id = $1",
    )
    .bind(run_id)
    .execute(tx.as_mut())
    .await?;
    Ok(())
}

pub async fn payslip_delivery_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    run_id: Uuid,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Option<PayslipDeliverySummary>, LifecycleError> {
    if run_head(tx, run_id, false).await?.is_none() {
        return Ok(None);
    }
    let limit = limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let offset = offset.unwrap_or(0).max(0);
    let counts = sqlx::query(
        "SELECT COUNT(*)::BIGINT AS issued, \
                COUNT(i.confirmed_at)::BIGINT AS acknowledged \
         FROM payroll_payslip_deliveries d \
         LEFT JOIN inbox_docs i ON i.id = d.inbox_doc_id \
         WHERE d.run_id = $1",
    )
    .bind(run_id)
    .fetch_one(tx.as_mut())
    .await?;
    let issued: i64 = counts.try_get("issued")?;
    let acknowledged: i64 = counts.try_get("acknowledged")?;
    let rows = sqlx::query(
        "SELECT d.line_id, d.employee_id, d.inbox_doc_id, d.issued_at, i.confirmed_at \
         FROM payroll_payslip_deliveries d \
         LEFT JOIN inbox_docs i ON i.id = d.inbox_doc_id \
         WHERE d.run_id = $1 \
         ORDER BY d.issued_at, d.line_id \
         LIMIT $2 OFFSET $3",
    )
    .bind(run_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(tx.as_mut())
    .await?;
    let items = rows
        .iter()
        .map(|row| {
            Ok(PayslipDeliveryItem {
                line_id: row.try_get("line_id")?,
                employee_id: row.try_get("employee_id")?,
                inbox_doc_id: row.try_get("inbox_doc_id")?,
                issued_at: row.try_get("issued_at")?,
                acknowledged_at: row.try_get("confirmed_at")?,
            })
        })
        .collect::<Result<Vec<_>, LifecycleError>>()?;
    Ok(Some(PayslipDeliverySummary {
        run_id,
        issued,
        acknowledged,
        items,
        total: issued,
        limit,
        offset,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payroll_row(gross: i64, income_tax: i64) -> Value {
        json!({
            "payroll": {
                "monthly_gross_pay_won": gross,
                "nts_tax_row": {
                    "table_version": "v1",
                    "monthly_income_tax_won": income_tax,
                    "local_income_tax_won": income_tax / 10,
                },
            },
        })
    }

    #[test]
    fn source_amount_selection_is_deterministic_and_fail_closed() {
        // No payroll-bearing row → truthful blocker.
        assert_eq!(
            select_source_amounts(&[]).err(),
            Some("SOURCE_AMOUNTS_NOT_MATERIALIZED")
        );
        assert_eq!(
            select_source_amounts(&[json!({"attendance": {}})]).err(),
            Some("SOURCE_AMOUNTS_NOT_MATERIALIZED")
        );

        // Byte-identical duplicates (re-import of the same source) are fine.
        let selected = select_source_amounts(&[
            json!({"attendance": {}}),
            payroll_row(3_000_000, 74_350),
            payroll_row(3_000_000, 74_350),
        ])
        .unwrap();
        assert_eq!(selected.gross_won, 3_000_000);
        assert_eq!(selected.tax_row.monthly_income_tax_won, 74_350);

        // Two DIFFERENT figure sets → ambiguity is a blocker, never a pick.
        assert_eq!(
            select_source_amounts(&[
                payroll_row(3_000_000, 74_350),
                payroll_row(3_100_000, 74_350)
            ])
            .err(),
            Some("SOURCE_AMOUNTS_CONFLICTING")
        );
    }
}
