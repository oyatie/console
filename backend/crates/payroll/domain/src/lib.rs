//! Korean payroll domain kernel.
//!
//! This crate intentionally contains pure, source-versioned data and guardrail
//! math only. It must not call external services, read environment variables, or
//! silently estimate tax-table values. Production payroll release remains gated
//! by licensed 노무사/세무사 validation artifacts.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use console_kernel_core::KernelError;
use time::Date;
use time::macros::date;

const PPM_DENOMINATOR: i128 = 1_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OfficialSource {
    pub authority: &'static str,
    pub title: &'static str,
    pub url: &'static str,
    pub retrieved_on: Date,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectivePeriod {
    pub from: Date,
    pub to_exclusive: Option<Date>,
}

impl EffectivePeriod {
    #[must_use]
    pub const fn new(from: Date, to_exclusive: Option<Date>) -> Self {
        Self { from, to_exclusive }
    }

    #[must_use]
    pub fn contains(self, day: Date) -> bool {
        day >= self.from && self.to_exclusive.is_none_or(|to| day < to)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContributionCode {
    NationalPension,
    HealthInsurance,
    LongTermCare,
    EmploymentUnemployment,
    IndustrialAccident,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContributionBasis {
    MonthlyStandardIncome,
    MonthlyRemuneration,
    IndustryTariff,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundingRule {
    FloorWon,
    ExternalTable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContributionRate {
    pub code: ContributionCode,
    pub period: EffectivePeriod,
    /// Parts per million of the contribution basis. 47,500 ppm = 4.75%.
    pub employee_ppm: Option<u32>,
    /// Parts per million of the contribution basis when a fixed employer share
    /// exists. `None` means employer cost needs a separate official tariff.
    pub employer_ppm: Option<u32>,
    pub basis: ContributionBasis,
    pub rounding: RoundingRule,
    pub source: OfficialSource,
    pub notes: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonthlyBaseLimit {
    pub period: EffectivePeriod,
    pub minimum_won: i64,
    pub maximum_won: i64,
    pub source: OfficialSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MinimumWageRate {
    pub period: EffectivePeriod,
    pub hourly_won: i64,
    pub daily_8h_won: i64,
    pub monthly_209h_won: i64,
    pub source: OfficialSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NtsWithholdingTaxRow {
    pub table_version: &'static str,
    pub monthly_income_tax_won: i64,
    pub local_income_tax_won: i64,
    pub source: OfficialSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PayrollDraftInput {
    pub pay_date: Date,
    pub monthly_remuneration_won: i64,
    pub pension_standard_monthly_income_won: Option<i64>,
    pub nts_tax_row: Option<NtsWithholdingTaxRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeductionLine {
    pub code: DeductionCode,
    pub label_ko: &'static str,
    pub amount_won: i64,
    pub source_url: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeductionCode {
    NationalPension,
    HealthInsurance,
    LongTermCare,
    EmploymentInsurance,
    IncomeTax,
    LocalIncomeTax,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PayrollDraft {
    pub pay_date: Date,
    pub gross_wage_won: i64,
    pub taxable_income_tax_table_version: &'static str,
    pub lines: Vec<DeductionLine>,
    pub total_employee_deductions_won: i64,
    pub net_pay_won: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeverancePayInput {
    pub hire_date: Date,
    pub exit_date: Date,
    pub average_wage_period_start: Date,
    pub average_wage_period_end: Date,
    pub average_wage_calendar_days: i64,
    pub average_wage_total_won: i64,
    /// 1-day 통상임금 (ordinary wage) in won.
    ///
    /// MANDATORY (not `Option`) by design: 근로기준법 시행령 제2조② requires
    /// severance to use the HIGHER of the average wage and the ordinary wage, so
    /// every caller must supply this. A plain field makes omission a *compile*
    /// error rather than a silent average-wage-only fall back — which is exactly
    /// the under-calculation bug that hurts the absence→exit population whose
    /// depressed 3-month window yields an artificially low average wage. The
    /// caller is responsible for deriving this daily figure from the monthly
    /// 통상임금 (e.g. via the 기준시간 209h rule); the kernel keeps that policy out.
    /// A non-positive value is rejected in `build_severance_pay_draft`.
    pub ordinary_daily_wage_won: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeverancePayDraft {
    pub hire_date: Date,
    pub exit_date: Date,
    pub service_days: i64,
    pub average_wage_period_start: Date,
    pub average_wage_period_end: Date,
    pub average_wage_calendar_days: i64,
    pub average_wage_total_won: i64,
    pub average_daily_wage_milliwon: i64,
    pub ordinary_daily_wage_won: i64,
    /// The daily wage that actually governed severance: max(average, ordinary).
    /// Auditable proof of which statutory basis won.
    pub statutory_daily_wage_milliwon: i64,
    pub statutory_30_day_wage_won: i64,
    pub severance_pay_won: i64,
    pub source: OfficialSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfessionalReviewerKind {
    LaborAttorney,
    TaxAccountant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoldenPayrollCase {
    pub case_id: String,
    pub rate_table_version: String,
    pub professionally_validated: bool,
    /// The case's own declared kernel inputs, embedded WHOLE rather than
    /// copied field-by-field: [`build_line_calculation`] consumes exactly this
    /// type, so when the kernel's input vocabulary moves, every golden-case
    /// construction site fails to COMPILE and must be re-signed. A copied
    /// field set would rot silently while still reading green.
    pub inputs: LineCalculationInput,
    pub expected_total_employee_deductions_won: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfessionalValidation {
    pub reviewer_kind: ProfessionalReviewerKind,
    pub reviewed_on: Date,
    pub artifact_sha256: String,
    pub reviewer_reference: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PayrollReleaseGateInput {
    pub rate_table_version: String,
    pub official_source_urls: Vec<String>,
    pub golden_cases: Vec<GoldenPayrollCase>,
    pub professional_validation: Option<ProfessionalValidation>,
}

/// The persisted lifecycle states already admitted by
/// `payroll_draft_runs.status` (migration 0074).
///
/// This is deliberately separate from tax calculation: a lifecycle decision
/// records whether a run may advance toward review, approval, and issuance;
/// it never manufactures a payable amount or an external bank transfer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayrollRunStatus {
    Staged,
    BlockedLegalGate,
    ReadyForReview,
    Approved,
    Issued,
    Void,
}

impl PayrollRunStatus {
    /// Parse only the storage values defined by migration 0074.
    pub fn parse(value: &str) -> Result<Self, KernelError> {
        match value {
            "STAGED" => Ok(Self::Staged),
            "BLOCKED_LEGAL_GATE" => Ok(Self::BlockedLegalGate),
            "READY_FOR_REVIEW" => Ok(Self::ReadyForReview),
            "APPROVED" => Ok(Self::Approved),
            "ISSUED" => Ok(Self::Issued),
            "VOID" => Ok(Self::Void),
            _ => Err(KernelError::validation("unknown payroll run status")),
        }
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Staged => "STAGED",
            Self::BlockedLegalGate => "BLOCKED_LEGAL_GATE",
            Self::ReadyForReview => "READY_FOR_REVIEW",
            Self::Approved => "APPROVED",
            Self::Issued => "ISSUED",
            Self::Void => "VOID",
        }
    }
}

/// A close-to-payslip command.
///
/// Approval is a pure transition after close prerequisites have been proven.
/// Calculation and issuance remain fail-closed until their regulated evidence
/// contracts are available to the persistence and transport layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayrollRunCommand {
    Calculate,
    Approve { approver_is_creator: bool },
    MarkIssued,
}

/// Facts which must all hold before a payroll run can advance.
///
/// `attendance_month_closes` is intentionally an org-level close. A branch
/// close cannot attest a whole payroll population, and a payroll lock must be
/// active for the *same* organization and exact calendar month.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PayrollClosePrerequisites {
    pub period_start: Date,
    pub period_end: Date,
    pub org_month_close_present: bool,
    pub active_exact_payroll_lock_present: bool,
    pub unresolved_attendance_exception_count: u64,
}

/// The deterministic outcome of one lifecycle command. `idempotent` means a
/// retry observed the terminal state that the command itself requests and made
/// no new transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PayrollRunTransition {
    pub status: PayrollRunStatus,
    pub idempotent: bool,
}

/// Fail closed unless an existing run has an exact org/month attendance close,
/// its active payroll period lock, and no unresolved attendance exceptions.
///
/// It permits only the approval transition: `Calculate` and `MarkIssued` return
/// `Conflict` unconditionally.
///
/// # NOTHING IN PRODUCTION CALLS THIS
///
/// Read the previous sentence as a statement about this function only, never
/// about the system. This guard has **no non-test caller**: every call site is
/// inside this file's `#[cfg(test)] mod tests`. The doc comment here previously
/// said it was "intentionally reusable by every persistence/transport adapter"
/// and that calculation and issuance "remain blocked" — and both readings were
/// false at the system level:
///
/// - **Calculation is not blocked.** It ships and works, through
///   `console_payroll_adapter_postgres::lifecycle::calculate_run_in_tx`, a
///   parallel path added later that never consults this function. That path
///   gates on a raw string compare, `run.status != "ATTENDANCE_CLOSED"`, as do
///   the submit, decide, schedule-disbursement and attest steps.
/// - **Issuance is not gated on step-up.** No step-up mechanism exists anywhere
///   in the payroll crates; the only occurrence of the phrase in the whole
///   payroll tree is a test name below. The mechanism does exist elsewhere
///   (`verify_step_up` in the inbox, financial and identity REST crates) and is
///   simply not wired here.
///
/// So this is a pure FSM kept for its shape, not an enforced boundary. Wiring
/// production through it — or deleting it — is a decision this comment does not
/// make. What it must not do is let a reader mistake it for the guard that
/// blocks payment, because the release gate is consulted in exactly one place
/// (`load_payslip_issuance_in_tx`, after `status == "PAID"`), which withholds
/// the 임금명세서 and not the money.
pub fn transition_payroll_run(
    current: PayrollRunStatus,
    command: PayrollRunCommand,
    prerequisites: PayrollClosePrerequisites,
) -> Result<PayrollRunTransition, KernelError> {
    // A confirmed prior transition is a retry-safe no-op. In particular, a
    // later attendance correction or lock amendment must not turn a network
    // retry into a second mutation or a contradictory failure.
    if let (PayrollRunStatus::Approved, PayrollRunCommand::Approve { .. }) = (current, command) {
        return Ok(PayrollRunTransition {
            status: current,
            idempotent: true,
        });
    }
    validate_close_prerequisites(prerequisites)?;

    let (status, idempotent) = match command {
        PayrollRunCommand::Calculate => {
            return Err(KernelError::conflict(
                "payroll calculation is blocked until immutable validated release-gate evidence is persisted",
            ));
        }
        PayrollRunCommand::Approve {
            approver_is_creator,
        } => match current {
            PayrollRunStatus::ReadyForReview => {
                if approver_is_creator {
                    return Err(KernelError::forbidden(
                        "payroll run creator cannot approve the same run",
                    ));
                }
                (PayrollRunStatus::Approved, false)
            }
            PayrollRunStatus::Approved => unreachable!("handled as retry above"),
            PayrollRunStatus::Staged
            | PayrollRunStatus::BlockedLegalGate
            | PayrollRunStatus::Issued
            | PayrollRunStatus::Void => {
                return Err(KernelError::invalid_transition(
                    "payroll approval requires a run ready for review",
                ));
            }
        },
        PayrollRunCommand::MarkIssued => {
            return Err(KernelError::conflict(
                "payroll issuance is blocked until step-up authorization, audit evidence, and an immutable issuance artifact are persisted",
            ));
        }
    };
    Ok(PayrollRunTransition { status, idempotent })
}

fn validate_close_prerequisites(
    prerequisites: PayrollClosePrerequisites,
) -> Result<(), KernelError> {
    if !is_exact_calendar_month(prerequisites.period_start, prerequisites.period_end) {
        return Err(KernelError::validation(
            "payroll run period must be one exact calendar month",
        ));
    }
    if !prerequisites.org_month_close_present {
        return Err(KernelError::conflict(
            "payroll run requires an org-level attendance close for its exact month",
        ));
    }
    if !prerequisites.active_exact_payroll_lock_present {
        return Err(KernelError::conflict(
            "payroll run requires an active payroll period lock for its exact month",
        ));
    }
    if prerequisites.unresolved_attendance_exception_count != 0 {
        return Err(KernelError::conflict(
            "payroll run is blocked by unresolved attendance exceptions",
        ));
    }
    Ok(())
}

fn is_exact_calendar_month(period_start: Date, period_end: Date) -> bool {
    if period_start.day() != 1 {
        return false;
    }
    let Some(day_after_end) = period_end.next_day() else {
        return false;
    };
    if day_after_end.day() != 1 {
        return false;
    }
    match period_start.month() {
        time::Month::January => {
            day_after_end.year() == period_start.year()
                && day_after_end.month() == time::Month::February
        }
        time::Month::February => {
            day_after_end.year() == period_start.year()
                && day_after_end.month() == time::Month::March
        }
        time::Month::March => {
            day_after_end.year() == period_start.year()
                && day_after_end.month() == time::Month::April
        }
        time::Month::April => {
            day_after_end.year() == period_start.year() && day_after_end.month() == time::Month::May
        }
        time::Month::May => {
            day_after_end.year() == period_start.year()
                && day_after_end.month() == time::Month::June
        }
        time::Month::June => {
            day_after_end.year() == period_start.year()
                && day_after_end.month() == time::Month::July
        }
        time::Month::July => {
            day_after_end.year() == period_start.year()
                && day_after_end.month() == time::Month::August
        }
        time::Month::August => {
            day_after_end.year() == period_start.year()
                && day_after_end.month() == time::Month::September
        }
        time::Month::September => {
            day_after_end.year() == period_start.year()
                && day_after_end.month() == time::Month::October
        }
        time::Month::October => {
            day_after_end.year() == period_start.year()
                && day_after_end.month() == time::Month::November
        }
        time::Month::November => {
            day_after_end.year() == period_start.year()
                && day_after_end.month() == time::Month::December
        }
        time::Month::December => {
            day_after_end.year() == period_start.year() + 1
                && day_after_end.month() == time::Month::January
        }
    }
}

#[must_use]
pub const fn payroll_sources_verified_on() -> Date {
    date!(2026 - 06 - 27)
}

#[must_use]
pub const fn moel_retirement_pay_source() -> OfficialSource {
    OfficialSource {
        authority: "Ministry of Employment and Labor",
        title: "Retirement pay average wage formula",
        url: "https://www.moel.go.kr/faq/faqView.do?seqRepeat=89",
        retrieved_on: date!(2026 - 07 - 03),
    }
}

#[must_use]
pub const fn nhis_qualification_loss_form_source() -> OfficialSource {
    OfficialSource {
        authority: "National Health Insurance Service",
        title: "4-insurance workplace subscriber qualification loss report",
        url: "https://www.nhis.or.kr/static/html/wbdb/f/wbdbf0201.html",
        retrieved_on: date!(2026 - 07 - 03),
    }
}

#[must_use]
pub const fn nps_source() -> OfficialSource {
    OfficialSource {
        authority: "국민연금공단",
        title: "사업장가입자 보험료율 및 기준소득월액 상·하한액",
        url: "https://www.nps.or.kr/pnsinfo/ntpsklg/getOHAF0038M0.do",
        retrieved_on: payroll_sources_verified_on(),
    }
}

#[must_use]
pub const fn nhis_source() -> OfficialSource {
    OfficialSource {
        authority: "국민건강보험공단",
        title: "2026년도 보험료율 인상 안내",
        url: "https://edi.nhis.or.kr/portal/images/popup/20251204_pop01longdesc.html",
        retrieved_on: payroll_sources_verified_on(),
    }
}

#[must_use]
pub const fn nts_source() -> OfficialSource {
    OfficialSource {
        authority: "국세청",
        title: "근로소득 원천징수방법(간이세액표)",
        url: "https://www.nts.go.kr/nts/cm/cntnts/cntntsView.do?cntntsId=7862&mi=6583",
        retrieved_on: payroll_sources_verified_on(),
    }
}

#[must_use]
pub const fn minimum_wage_source() -> OfficialSource {
    OfficialSource {
        authority: "최저임금위원회",
        title: "연도별 최저임금 결정현황",
        url: "https://www.minimumwage.go.kr/minWage/policy/decisionMain.do",
        retrieved_on: payroll_sources_verified_on(),
    }
}

#[must_use]
pub fn statutory_contribution_rates() -> Vec<ContributionRate> {
    vec![
        ContributionRate {
            code: ContributionCode::NationalPension,
            period: EffectivePeriod::new(date!(2026 - 01 - 01), Some(date!(2027 - 01 - 01))),
            employee_ppm: Some(47_500),
            employer_ppm: Some(47_500),
            basis: ContributionBasis::MonthlyStandardIncome,
            rounding: RoundingRule::FloorWon,
            source: nps_source(),
            notes: "2026 total 국민연금 rate is 9.5%, split equally for workplace subscribers.",
        },
        ContributionRate {
            code: ContributionCode::HealthInsurance,
            period: EffectivePeriod::new(date!(2026 - 01 - 01), Some(date!(2027 - 01 - 01))),
            employee_ppm: Some(35_950),
            employer_ppm: Some(35_950),
            basis: ContributionBasis::MonthlyRemuneration,
            rounding: RoundingRule::FloorWon,
            source: nhis_source(),
            notes: "2026 workplace 건강보험 total 7.19%, split 50/50.",
        },
        ContributionRate {
            code: ContributionCode::LongTermCare,
            period: EffectivePeriod::new(date!(2026 - 01 - 01), Some(date!(2027 - 01 - 01))),
            employee_ppm: Some(4_724),
            employer_ppm: Some(4_724),
            basis: ContributionBasis::MonthlyRemuneration,
            rounding: RoundingRule::FloorWon,
            source: nhis_source(),
            notes: "2026 장기요양 total 0.9448% of remuneration, represented as a 50/50 employee/employer split.",
        },
        ContributionRate {
            code: ContributionCode::EmploymentUnemployment,
            period: EffectivePeriod::new(date!(2026 - 01 - 01), Some(date!(2027 - 01 - 01))),
            employee_ppm: Some(9_000),
            employer_ppm: Some(9_000),
            basis: ContributionBasis::MonthlyRemuneration,
            rounding: RoundingRule::FloorWon,
            source: OfficialSource {
                authority: "법제처/고용노동부",
                title: "고용보험 실업급여 보험료율",
                url: "https://www.law.go.kr/LSW/lsInfoP.do?efYd=20251001&lsiSeq=280527#0000",
                retrieved_on: payroll_sources_verified_on(),
            },
            notes: "Employee unemployment-insurance share only; employer vocational/stabilization add-ons require separate company-size rules.",
        },
        ContributionRate {
            code: ContributionCode::IndustrialAccident,
            period: EffectivePeriod::new(date!(2026 - 01 - 01), Some(date!(2027 - 01 - 01))),
            employee_ppm: Some(0),
            employer_ppm: None,
            basis: ContributionBasis::IndustryTariff,
            rounding: RoundingRule::ExternalTable,
            source: OfficialSource {
                authority: "근로복지공단/고용노동부",
                title: "산재보험 업종별 보험료율 고시",
                url: "https://total.comwel.or.kr/",
                retrieved_on: payroll_sources_verified_on(),
            },
            notes: "산재보험 is employer-side and industry-tariff based; this kernel must not guess it.",
        },
    ]
}

#[must_use]
pub fn national_pension_base_limits() -> Vec<MonthlyBaseLimit> {
    vec![
        MonthlyBaseLimit {
            period: EffectivePeriod::new(date!(2025 - 07 - 01), Some(date!(2026 - 07 - 01))),
            minimum_won: 400_000,
            maximum_won: 6_370_000,
            source: nps_source(),
        },
        MonthlyBaseLimit {
            period: EffectivePeriod::new(date!(2026 - 07 - 01), Some(date!(2027 - 07 - 01))),
            minimum_won: 410_000,
            maximum_won: 6_590_000,
            source: nps_source(),
        },
    ]
}

#[must_use]
pub fn minimum_wage_rates() -> Vec<MinimumWageRate> {
    vec![MinimumWageRate {
        period: EffectivePeriod::new(date!(2026 - 01 - 01), Some(date!(2027 - 01 - 01))),
        hourly_won: 10_320,
        daily_8h_won: 82_560,
        monthly_209h_won: 2_156_880,
        source: minimum_wage_source(),
    }]
}

pub fn contribution_rate_on(
    code: ContributionCode,
    day: Date,
) -> Result<ContributionRate, KernelError> {
    statutory_contribution_rates()
        .into_iter()
        .find(|rate| rate.code == code && rate.period.contains(day))
        .ok_or_else(|| KernelError::validation(format!("missing payroll rate {code:?} for {day}")))
}

pub fn national_pension_limit_on(day: Date) -> Result<MonthlyBaseLimit, KernelError> {
    national_pension_base_limits()
        .into_iter()
        .find(|limit| limit.period.contains(day))
        .ok_or_else(|| KernelError::validation(format!("missing pension base limit for {day}")))
}

pub fn minimum_wage_on(day: Date) -> Result<MinimumWageRate, KernelError> {
    minimum_wage_rates()
        .into_iter()
        .find(|rate| rate.period.contains(day))
        .ok_or_else(|| KernelError::validation(format!("missing minimum wage for {day}")))
}

pub fn build_employee_payroll_draft(input: PayrollDraftInput) -> Result<PayrollDraft, KernelError> {
    if input.monthly_remuneration_won < 0 {
        return Err(KernelError::validation(
            "monthly remuneration must be non-negative",
        ));
    }
    let tax_row = input.nts_tax_row.ok_or_else(|| {
        KernelError::validation(
            "NTS withholding tax table row is required; payroll must not estimate income tax",
        )
    })?;
    if tax_row.monthly_income_tax_won < 0 || tax_row.local_income_tax_won < 0 {
        return Err(KernelError::validation(
            "NTS tax row amounts must be non-negative",
        ));
    }

    let lines = employee_deduction_lines(
        input.pay_date,
        input.monthly_remuneration_won,
        input.pension_standard_monthly_income_won,
        tax_row.monthly_income_tax_won,
        tax_row.local_income_tax_won,
        tax_row.source.url,
    )?;
    let (total_employee_deductions_won, net_pay_won) =
        deduction_totals(&lines, input.monthly_remuneration_won)?;

    Ok(PayrollDraft {
        pay_date: input.pay_date,
        gross_wage_won: input.monthly_remuneration_won,
        taxable_income_tax_table_version: tax_row.table_version,
        lines,
        total_employee_deductions_won,
        net_pay_won,
    })
}

/// An NTS 간이세액표 row whose figures were materialized from a verified
/// imported source ledger row (`data_import_rows.canonical_row`), so its table
/// version is a runtime string — unlike [`NtsWithholdingTaxRow`], whose
/// `&'static` fields exist for in-crate verified constants. The amounts are
/// still verbatim source figures: this type never carries an estimate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedNtsTaxRow {
    pub table_version: String,
    pub monthly_income_tax_won: i64,
    pub local_income_tax_won: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineCalculationInput {
    pub pay_date: Date,
    pub gross_won: i64,
    pub pension_standard_monthly_income_won: Option<i64>,
    pub tax_row: VerifiedNtsTaxRow,
}

/// One employee's draft deduction breakdown for persistence in
/// `payroll_line_calculations`. Always a DRAFT: the release gate
/// ([`validate_release_gate`]) is what makes a stored calculation payable,
/// never this function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineCalculation {
    pub gross_won: i64,
    pub lines: Vec<DeductionLine>,
    pub total_employee_deductions_won: i64,
    pub net_won: i64,
    pub tax_table_version: String,
}

/// [`build_employee_payroll_draft`] for a source-materialized tax row: same
/// statutory 4-insurance math from the in-crate verified rate tables, income
/// tax verbatim from the supplied verified NTS row. Refuses (like the draft
/// builder) to compute anything without that row.
pub fn build_line_calculation(input: LineCalculationInput) -> Result<LineCalculation, KernelError> {
    if input.gross_won < 0 {
        return Err(KernelError::validation(
            "monthly remuneration must be non-negative",
        ));
    }
    if input.tax_row.monthly_income_tax_won < 0 || input.tax_row.local_income_tax_won < 0 {
        return Err(KernelError::validation(
            "NTS tax row amounts must be non-negative",
        ));
    }
    if input.tax_row.table_version.trim().is_empty() {
        return Err(KernelError::validation(
            "NTS tax table version must be supplied with the verified source row",
        ));
    }

    let lines = employee_deduction_lines(
        input.pay_date,
        input.gross_won,
        input.pension_standard_monthly_income_won,
        input.tax_row.monthly_income_tax_won,
        input.tax_row.local_income_tax_won,
        nts_source().url,
    )?;
    let (total_employee_deductions_won, net_won) = deduction_totals(&lines, input.gross_won)?;

    Ok(LineCalculation {
        gross_won: input.gross_won,
        lines,
        total_employee_deductions_won,
        net_won,
        tax_table_version: input.tax_row.table_version,
    })
}

/// The six employee-side deduction lines: statutory 4-insurance contributions
/// from the in-crate verified rate tables + the two income-tax lines verbatim
/// from a verified NTS row (never estimated).
fn employee_deduction_lines(
    pay_date: Date,
    monthly_remuneration_won: i64,
    pension_standard_monthly_income_won: Option<i64>,
    monthly_income_tax_won: i64,
    local_income_tax_won: i64,
    tax_source_url: &'static str,
) -> Result<Vec<DeductionLine>, KernelError> {
    let pension_limit = national_pension_limit_on(pay_date)?;
    let pension_basis = pension_standard_monthly_income_won
        .unwrap_or(monthly_remuneration_won)
        .clamp(pension_limit.minimum_won, pension_limit.maximum_won);

    let pension = employee_amount(
        contribution_rate_on(ContributionCode::NationalPension, pay_date)?,
        pension_basis,
    )?;
    let health = employee_amount(
        contribution_rate_on(ContributionCode::HealthInsurance, pay_date)?,
        monthly_remuneration_won,
    )?;
    let long_term_care = employee_amount(
        contribution_rate_on(ContributionCode::LongTermCare, pay_date)?,
        monthly_remuneration_won,
    )?;
    let employment = employee_amount(
        contribution_rate_on(ContributionCode::EmploymentUnemployment, pay_date)?,
        monthly_remuneration_won,
    )?;

    Ok(vec![
        deduction(
            DeductionCode::NationalPension,
            "국민연금",
            pension,
            nps_source().url,
        ),
        deduction(
            DeductionCode::HealthInsurance,
            "건강보험",
            health,
            nhis_source().url,
        ),
        deduction(
            DeductionCode::LongTermCare,
            "장기요양보험",
            long_term_care,
            nhis_source().url,
        ),
        deduction(
            DeductionCode::EmploymentInsurance,
            "고용보험",
            employment,
            contribution_rate_on(ContributionCode::EmploymentUnemployment, pay_date)?
                .source
                .url,
        ),
        deduction(
            DeductionCode::IncomeTax,
            "근로소득세",
            monthly_income_tax_won,
            tax_source_url,
        ),
        deduction(
            DeductionCode::LocalIncomeTax,
            "지방소득세",
            local_income_tax_won,
            tax_source_url,
        ),
    ])
}

fn deduction_totals(lines: &[DeductionLine], gross_won: i64) -> Result<(i64, i64), KernelError> {
    let total = lines
        .iter()
        .map(|line| line.amount_won)
        .try_fold(0_i64, |total, amount| {
            total
                .checked_add(amount)
                .ok_or_else(|| KernelError::validation("deduction total overflow"))
        })?;
    let net = gross_won
        .checked_sub(total)
        .ok_or_else(|| KernelError::validation("deductions exceed gross wage"))?;
    Ok((total, net))
}

pub fn build_severance_pay_draft(
    input: SeverancePayInput,
) -> Result<SeverancePayDraft, KernelError> {
    if input.exit_date < input.hire_date {
        return Err(KernelError::validation(
            "exit date must be on or after hire date",
        ));
    }
    if input.average_wage_period_end < input.average_wage_period_start {
        return Err(KernelError::validation(
            "average wage period end must be on or after start",
        ));
    }
    if input.average_wage_period_end > input.exit_date {
        return Err(KernelError::validation(
            "average wage period must not end after the exit date",
        ));
    }
    if input.average_wage_calendar_days <= 0 {
        return Err(KernelError::validation(
            "average wage calendar days must be positive",
        ));
    }
    if input.average_wage_total_won <= 0 {
        return Err(KernelError::validation(
            "average wage total must be positive",
        ));
    }
    // FAIL LOUD: the 통상임금 floor cannot be applied without the ordinary wage.
    // Reject rather than silently compute severance from the average wage alone,
    // which under-pays the absence→exit population this feature targets.
    if input.ordinary_daily_wage_won <= 0 {
        return Err(KernelError::validation(
            "ordinary daily wage (통상임금) is required and must be positive; severance must never fall back to the average-wage-only figure",
        ));
    }

    let service_days = i64::from(
        input
            .exit_date
            .to_julian_day()
            .saturating_sub(input.hire_date.to_julian_day())
            + 1,
    );
    if service_days < 365 {
        return Err(KernelError::validation(
            "statutory severance pay requires at least one year of service",
        ));
    }

    let average_daily_wage_milliwon = checked_i128_to_i64(
        checked_mul_i128(input.average_wage_total_won, 1_000)?
            / i128::from(input.average_wage_calendar_days),
    )?;

    // 통상임금 floor (근로기준법 시행령 제2조②): severance uses the HIGHER of the
    // 1-day average wage and the 1-day ordinary wage. Decide which governs by
    // cross-multiplying (ordinary_daily > average_daily  <=>  ordinary_daily *
    // calendar_days > average_wage_total), so the comparison never loses the
    // fractional part that flooring `average_daily_wage_milliwon` would drop.
    let ordinary_governs = checked_mul_i128(
        input.ordinary_daily_wage_won,
        input.average_wage_calendar_days,
    )? > i128::from(input.average_wage_total_won);

    let statutory_daily_wage_milliwon = if ordinary_governs {
        checked_i128_to_i64(checked_mul_i128(input.ordinary_daily_wage_won, 1_000)?)?
    } else {
        average_daily_wage_milliwon
    };

    // Fold both paths into one formula by choosing the numerator base and its
    // divisor. Ordinary path uses exact won (divisor 1); average path keeps the
    // existing high-precision direct form (divisor = calendar_days) so its
    // rounding is byte-for-byte unchanged when it governs.
    let (base_won, base_divisor) = if ordinary_governs {
        (input.ordinary_daily_wage_won, 1_i64)
    } else {
        (
            input.average_wage_total_won,
            input.average_wage_calendar_days,
        )
    };

    let statutory_30_day_wage_won =
        checked_i128_to_i64(checked_mul_i128(base_won, 30)? / i128::from(base_divisor))?;
    let severance_pay_won = checked_i128_to_i64(
        checked_mul_i128(base_won, 30)?
            .checked_mul(i128::from(service_days))
            .ok_or_else(|| KernelError::validation("severance calculation overflow"))?
            / i128::from(base_divisor)
            / 365,
    )?;

    Ok(SeverancePayDraft {
        hire_date: input.hire_date,
        exit_date: input.exit_date,
        service_days,
        average_wage_period_start: input.average_wage_period_start,
        average_wage_period_end: input.average_wage_period_end,
        average_wage_calendar_days: input.average_wage_calendar_days,
        average_wage_total_won: input.average_wage_total_won,
        average_daily_wage_milliwon,
        ordinary_daily_wage_won: input.ordinary_daily_wage_won,
        statutory_daily_wage_milliwon,
        statutory_30_day_wage_won,
        severance_pay_won,
        source: moel_retirement_pay_source(),
    })
}

pub fn validate_release_gate(input: &PayrollReleaseGateInput) -> Result<(), KernelError> {
    if input.rate_table_version.trim().is_empty() {
        return Err(KernelError::validation(
            "payroll rate table version is required",
        ));
    }
    if input.official_source_urls.is_empty() {
        return Err(KernelError::validation(
            "at least one official source URL is required",
        ));
    }
    if input.golden_cases.is_empty() {
        return Err(KernelError::validation(
            "at least one payroll golden case is required",
        ));
    }
    if let Some(case) = input
        .golden_cases
        .iter()
        .find(|case| case.rate_table_version != input.rate_table_version)
    {
        return Err(KernelError::validation(format!(
            "golden case {} uses mismatched rate table version",
            case.case_id
        )));
    }
    if let Some(case) = input
        .golden_cases
        .iter()
        .find(|case| !case.professionally_validated)
    {
        return Err(KernelError::validation(format!(
            "golden case {} lacks professional validation",
            case.case_id
        )));
    }
    let validation = input.professional_validation.as_ref().ok_or_else(|| {
        KernelError::validation("노무사/세무사 professional validation is required")
    })?;
    if validation.artifact_sha256.len() != 64
        || !validation
            .artifact_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(KernelError::validation(
            "professional validation artifact_sha256 must be a 64-character hex digest",
        ));
    }
    if validation.reviewer_reference.trim().is_empty() {
        return Err(KernelError::validation(
            "professional validation reviewer reference is required",
        ));
    }
    // The golden case is RE-EXECUTED, not merely stored: until this loop
    // existed the professionals' signed figure was compared to nothing, so a
    // golden case could not fail. LAST on purpose — every check above returns
    // on its first failure, so running the arithmetic earlier would surface a
    // mismatch message in place of the specific pre-existing condition that
    // actually regressed.
    //
    // Strictly stricter, and it admits a THIRD failure class to the gate:
    // `build_line_calculation`'s own refusals (blank tax table_version,
    // negative amounts, a pay_date outside the in-crate rate windows) now fail
    // the GATE, not just line calculation. That is deliberate — a case whose
    // declared inputs the kernel cannot execute is not a case that passed.
    for case in &input.golden_cases {
        // .clone() is mandatory: build_line_calculation takes its input by
        // value and moves tax_row.table_version, while this function only
        // borrows &PayrollReleaseGateInput.
        let computed = build_line_calculation(case.inputs.clone()).map_err(|err| {
            KernelError::validation(format!(
                "golden case {} could not be recomputed: {}",
                case.case_id, err.message
            ))
        })?;
        if computed.total_employee_deductions_won != case.expected_total_employee_deductions_won {
            return Err(KernelError::validation(format!(
                "golden case {} expects total employee deductions {} but the payroll kernel computed {}",
                case.case_id,
                case.expected_total_employee_deductions_won,
                computed.total_employee_deductions_won
            )));
        }
    }
    Ok(())
}

fn employee_amount(rate: ContributionRate, basis_won: i64) -> Result<i64, KernelError> {
    let ppm = rate.employee_ppm.ok_or_else(|| {
        KernelError::validation(format!(
            "payroll rate {:?} has no employee share",
            rate.code
        ))
    })?;
    amount_by_ppm_floor_won(basis_won, ppm)
}

fn amount_by_ppm_floor_won(base_won: i64, ppm: u32) -> Result<i64, KernelError> {
    if base_won < 0 {
        return Err(KernelError::validation("rate base must be non-negative"));
    }
    let amount = i128::from(base_won)
        .checked_mul(i128::from(ppm))
        .ok_or_else(|| KernelError::validation("payroll rate multiplication overflow"))?
        / PPM_DENOMINATOR;
    i64::try_from(amount).map_err(|_| KernelError::validation("payroll amount overflow"))
}

fn checked_mul_i128(left: i64, right: i64) -> Result<i128, KernelError> {
    i128::from(left)
        .checked_mul(i128::from(right))
        .ok_or_else(|| KernelError::validation("payroll amount multiplication overflow"))
}

fn checked_i128_to_i64(amount: i128) -> Result<i64, KernelError> {
    i64::try_from(amount).map_err(|_| KernelError::validation("payroll amount overflow"))
}

fn deduction(
    code: DeductionCode,
    label_ko: &'static str,
    amount_won: i64,
    source_url: &'static str,
) -> DeductionLine {
    DeductionLine {
        code,
        label_ko,
        amount_won,
        source_url,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn closed_june_prerequisites() -> PayrollClosePrerequisites {
        PayrollClosePrerequisites {
            period_start: date!(2026 - 06 - 01),
            period_end: date!(2026 - 06 - 30),
            org_month_close_present: true,
            active_exact_payroll_lock_present: true,
            unresolved_attendance_exception_count: 0,
        }
    }

    // These two tests pin the FSM's own refusals. They are named for the
    // function, not for the system, because `transition_payroll_run` has no
    // production caller — see its doc comment. A name like
    // `calculation_is_blocked_without_...` asserted a system property this
    // repository does not have, and would have become a CI-endorsed falsehood
    // the moment this target was wired into a workflow.
    #[test]
    fn fsm_refuses_calculate_command_unconditionally() {
        let error = transition_payroll_run(
            PayrollRunStatus::Staged,
            PayrollRunCommand::Calculate,
            closed_june_prerequisites(),
        )
        .unwrap_err();

        assert_eq!(error.kind, console_kernel_core::ErrorKind::Conflict);
        assert!(
            error
                .message
                .contains("immutable validated release-gate evidence")
        );
    }

    // The refusal message names step-up, audit evidence and an immutable
    // artifact. None of those three is implemented on the production issuance
    // path — this asserts the STRING, which is why the name says so.
    #[test]
    fn fsm_refuses_mark_issued_command_citing_unimplemented_controls() {
        let error = transition_payroll_run(
            PayrollRunStatus::Approved,
            PayrollRunCommand::MarkIssued,
            closed_june_prerequisites(),
        )
        .unwrap_err();

        assert_eq!(error.kind, console_kernel_core::ErrorKind::Conflict);
        assert!(error.message.contains("step-up authorization"));
        assert!(error.message.contains("audit evidence"));
        assert!(error.message.contains("immutable issuance artifact"));
    }

    #[test]
    fn approval_requires_exact_closed_month_and_is_idempotent_after_persistence() {
        let prerequisites = closed_june_prerequisites();
        let approved = transition_payroll_run(
            PayrollRunStatus::ReadyForReview,
            PayrollRunCommand::Approve {
                approver_is_creator: false,
            },
            prerequisites,
        )
        .unwrap();
        assert_eq!(approved.status, PayrollRunStatus::Approved);
        assert!(!approved.idempotent);

        let retry = transition_payroll_run(
            PayrollRunStatus::Approved,
            PayrollRunCommand::Approve {
                approver_is_creator: false,
            },
            PayrollClosePrerequisites {
                org_month_close_present: false,
                ..prerequisites
            },
        )
        .unwrap();
        assert!(retry.idempotent);
    }

    #[test]
    fn approval_fails_closed_on_missing_close_lock_or_exception() {
        let prerequisites = closed_june_prerequisites();
        for missing in [
            PayrollClosePrerequisites {
                org_month_close_present: false,
                ..prerequisites
            },
            PayrollClosePrerequisites {
                active_exact_payroll_lock_present: false,
                ..prerequisites
            },
            PayrollClosePrerequisites {
                unresolved_attendance_exception_count: 1,
                ..prerequisites
            },
            PayrollClosePrerequisites {
                period_start: date!(2026 - 06 - 02),
                ..prerequisites
            },
        ] {
            assert!(
                transition_payroll_run(
                    PayrollRunStatus::ReadyForReview,
                    PayrollRunCommand::Approve {
                        approver_is_creator: false,
                    },
                    missing,
                )
                .is_err()
            );
        }
    }

    #[test]
    fn approval_rejects_creator_self_approval() {
        let error = transition_payroll_run(
            PayrollRunStatus::ReadyForReview,
            PayrollRunCommand::Approve {
                approver_is_creator: true,
            },
            closed_june_prerequisites(),
        )
        .unwrap_err();
        assert_eq!(error.kind, console_kernel_core::ErrorKind::Forbidden);
    }

    /// The declared kernel inputs behind the 373,302 figure every golden-case
    /// fixture in this crate already stores. Pinned green independently by
    /// `builds_employee_deduction_draft_from_effective_rates_and_supplied_nts_row`.
    fn golden_case_inputs() -> LineCalculationInput {
        LineCalculationInput {
            pay_date: date!(2026 - 06 - 27),
            gross_won: 3_000_000,
            pension_standard_monthly_income_won: None,
            tax_row: VerifiedNtsTaxRow {
                table_version: "NTS-간이세액표-fixture-row-v1".to_owned(),
                monthly_income_tax_won: 74_350,
                local_income_tax_won: 7_430,
            },
        }
    }

    fn fixture_tax_row() -> NtsWithholdingTaxRow {
        NtsWithholdingTaxRow {
            table_version: "NTS-간이세액표-fixture-row-v1",
            monthly_income_tax_won: 74_350,
            local_income_tax_won: 7_430,
            source: nts_source(),
        }
    }

    #[test]
    fn selects_2026_rates_and_effective_dated_pension_limits() {
        let june_limit = national_pension_limit_on(date!(2026 - 06 - 27)).unwrap();
        assert_eq!(june_limit.minimum_won, 400_000);
        assert_eq!(june_limit.maximum_won, 6_370_000);

        let july_limit = national_pension_limit_on(date!(2026 - 07 - 01)).unwrap();
        assert_eq!(july_limit.minimum_won, 410_000);
        assert_eq!(july_limit.maximum_won, 6_590_000);

        let pension =
            contribution_rate_on(ContributionCode::NationalPension, date!(2026 - 06 - 27)).unwrap();
        assert_eq!(pension.employee_ppm, Some(47_500));
        assert_eq!(pension.employer_ppm, Some(47_500));

        let minimum_wage = minimum_wage_on(date!(2026 - 06 - 27)).unwrap();
        assert_eq!(minimum_wage.hourly_won, 10_320);
        assert_eq!(minimum_wage.monthly_209h_won, 2_156_880);
    }

    #[test]
    fn refuses_to_estimate_income_tax_without_an_nts_table_row() {
        let result = build_employee_payroll_draft(PayrollDraftInput {
            pay_date: date!(2026 - 06 - 27),
            monthly_remuneration_won: 3_000_000,
            pension_standard_monthly_income_won: None,
            nts_tax_row: None,
        });

        assert!(result.is_err());
        assert!(format!("{:?}", result.err().unwrap()).contains("NTS withholding tax table row"));
    }

    #[test]
    fn builds_employee_deduction_draft_from_effective_rates_and_supplied_nts_row() {
        let draft = build_employee_payroll_draft(PayrollDraftInput {
            pay_date: date!(2026 - 06 - 27),
            monthly_remuneration_won: 3_000_000,
            pension_standard_monthly_income_won: None,
            nts_tax_row: Some(fixture_tax_row()),
        })
        .unwrap();

        assert_eq!(line_amount(&draft, DeductionCode::NationalPension), 142_500);
        assert_eq!(line_amount(&draft, DeductionCode::HealthInsurance), 107_850);
        assert_eq!(line_amount(&draft, DeductionCode::LongTermCare), 14_172);
        assert_eq!(
            line_amount(&draft, DeductionCode::EmploymentInsurance),
            27_000
        );
        assert_eq!(line_amount(&draft, DeductionCode::IncomeTax), 74_350);
        assert_eq!(line_amount(&draft, DeductionCode::LocalIncomeTax), 7_430);
        assert_eq!(draft.total_employee_deductions_won, 373_302);
        assert_eq!(draft.net_pay_won, 2_626_698);
        assert_eq!(
            draft.taxable_income_tax_table_version,
            "NTS-간이세액표-fixture-row-v1"
        );
    }

    #[test]
    fn line_calculation_matches_draft_builder_amounts_exactly() {
        let draft = build_employee_payroll_draft(PayrollDraftInput {
            pay_date: date!(2026 - 06 - 27),
            monthly_remuneration_won: 3_000_000,
            pension_standard_monthly_income_won: None,
            nts_tax_row: Some(fixture_tax_row()),
        })
        .unwrap();
        let calc = build_line_calculation(LineCalculationInput {
            pay_date: date!(2026 - 06 - 27),
            gross_won: 3_000_000,
            pension_standard_monthly_income_won: None,
            tax_row: VerifiedNtsTaxRow {
                table_version: "NTS-간이세액표-fixture-row-v1".to_owned(),
                monthly_income_tax_won: 74_350,
                local_income_tax_won: 7_430,
            },
        })
        .unwrap();

        assert_eq!(calc.lines, draft.lines);
        assert_eq!(
            calc.total_employee_deductions_won,
            draft.total_employee_deductions_won
        );
        assert_eq!(calc.net_won, draft.net_pay_won);
        assert_eq!(calc.tax_table_version, "NTS-간이세액표-fixture-row-v1");
    }

    #[test]
    fn line_calculation_refuses_blank_table_version_and_negative_tax() {
        let base = LineCalculationInput {
            pay_date: date!(2026 - 06 - 27),
            gross_won: 3_000_000,
            pension_standard_monthly_income_won: None,
            tax_row: VerifiedNtsTaxRow {
                table_version: "  ".to_owned(),
                monthly_income_tax_won: 74_350,
                local_income_tax_won: 7_430,
            },
        };
        assert!(build_line_calculation(base.clone()).is_err());

        let mut negative = base;
        negative.tax_row.table_version = "v1".to_owned();
        negative.tax_row.monthly_income_tax_won = -1;
        assert!(build_line_calculation(negative).is_err());
    }

    #[test]
    fn caps_national_pension_basis_by_effective_limit() {
        let draft = build_employee_payroll_draft(PayrollDraftInput {
            pay_date: date!(2026 - 07 - 01),
            monthly_remuneration_won: 10_000_000,
            pension_standard_monthly_income_won: None,
            nts_tax_row: Some(fixture_tax_row()),
        })
        .unwrap();

        assert_eq!(line_amount(&draft, DeductionCode::NationalPension), 313_025);
    }

    #[test]
    fn release_gate_requires_validated_golden_case_and_professional_artifact() {
        let unvalidated = PayrollReleaseGateInput {
            rate_table_version: "KR-2026-official-rates-v1".to_string(),
            official_source_urls: vec![nps_source().url.to_string(), nhis_source().url.to_string()],
            golden_cases: vec![GoldenPayrollCase {
                case_id: "golden-fixture-unvalidated".to_string(),
                rate_table_version: "KR-2026-official-rates-v1".to_string(),
                professionally_validated: false,
                inputs: golden_case_inputs(),
                expected_total_employee_deductions_won: 373_302,
            }],
            professional_validation: None,
        };
        assert!(validate_release_gate(&unvalidated).is_err());

        let validated = PayrollReleaseGateInput {
            rate_table_version: "KR-2026-official-rates-v1".to_string(),
            official_source_urls: vec![
                nps_source().url.to_string(),
                nhis_source().url.to_string(),
                nts_source().url.to_string(),
                minimum_wage_source().url.to_string(),
            ],
            golden_cases: vec![GoldenPayrollCase {
                case_id: "golden-fixture-professionally-reviewed".to_string(),
                rate_table_version: "KR-2026-official-rates-v1".to_string(),
                professionally_validated: true,
                inputs: golden_case_inputs(),
                expected_total_employee_deductions_won: 373_302,
            }],
            professional_validation: Some(ProfessionalValidation {
                reviewer_kind: ProfessionalReviewerKind::LaborAttorney,
                reviewed_on: date!(2026 - 06 - 27),
                artifact_sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                    .to_string(),
                reviewer_reference: "licensed-reviewer-record".to_string(),
            }),
        };
        validate_release_gate(&validated).unwrap();
    }

    /// A gate input that satisfies every condition. Each test below mutates
    /// EXACTLY ONE field of it, so a failure names one defect.
    fn satisfied_release_gate() -> PayrollReleaseGateInput {
        PayrollReleaseGateInput {
            rate_table_version: "KR-2026-official-rates-v1".to_string(),
            official_source_urls: vec![nps_source().url.to_string(), nhis_source().url.to_string()],
            golden_cases: vec![GoldenPayrollCase {
                case_id: "GC-FIXTURE-A".to_string(),
                rate_table_version: "KR-2026-official-rates-v1".to_string(),
                professionally_validated: true,
                inputs: golden_case_inputs(),
                expected_total_employee_deductions_won: 373_302,
            }],
            professional_validation: Some(ProfessionalValidation {
                reviewer_kind: ProfessionalReviewerKind::LaborAttorney,
                reviewed_on: date!(2026 - 06 - 27),
                // Placeholder digest, and it STAYS a placeholder.
                artifact_sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                    .to_string(),
                reviewer_reference: "licensed-reviewer-record".to_string(),
            }),
        }
    }

    #[test]
    fn release_gate_rejects_a_golden_case_total_the_kernel_does_not_recompute() {
        // THE LOAD-BEARING TEST. Until this passes, a golden case is a stored
        // assertion that cannot fail: the professionals' signed figure is never
        // compared to anything the kernel produces. Only the EXPECTATION moves;
        // the declared inputs are the ones that really compute 373,302.
        let mut disagrees = satisfied_release_gate();
        disagrees.golden_cases[0].expected_total_employee_deductions_won = 373_303;

        let error = validate_release_gate(&disagrees).unwrap_err();

        assert_eq!(
            error.message,
            "golden case GC-FIXTURE-A expects total employee deductions 373303 \
             but the payroll kernel computed 373302"
        );
    }

    #[test]
    fn release_gate_accepts_a_golden_case_whose_declared_inputs_recompute_to_its_expected_total() {
        validate_release_gate(&satisfied_release_gate()).unwrap();
    }

    #[test]
    fn release_gate_rejects_a_blank_rate_table_version() {
        // ONE field, like every other test here. Blanking the golden case's
        // copy too was unreachable setup — the blank check returns before the
        // mismatch check is reached — and it cost the test its grip on that
        // precedence: with both blank they matched, so a reordering that put
        // the mismatch check first went unnoticed. Leaving the case's version
        // intact makes this record mismatched as well as blank, so the pinned
        // message now asserts WHICH refusal wins.
        let mut blank = satisfied_release_gate();
        blank.rate_table_version = "  ".to_string();

        assert_eq!(
            validate_release_gate(&blank).unwrap_err().message,
            "payroll rate table version is required"
        );
    }

    #[test]
    fn release_gate_rejects_an_empty_official_source_url_list() {
        let mut no_urls = satisfied_release_gate();
        no_urls.official_source_urls = vec![];

        assert_eq!(
            validate_release_gate(&no_urls).unwrap_err().message,
            "at least one official source URL is required"
        );
    }

    #[test]
    fn release_gate_rejects_a_case_marked_not_professionally_validated() {
        let mut unvalidated = satisfied_release_gate();
        unvalidated.golden_cases[0].professionally_validated = false;

        assert_eq!(
            validate_release_gate(&unvalidated).unwrap_err().message,
            "golden case GC-FIXTURE-A lacks professional validation"
        );
    }

    #[test]
    fn release_gate_rejects_an_absent_professional_validation() {
        let mut unsigned = satisfied_release_gate();
        unsigned.professional_validation = None;

        assert_eq!(
            validate_release_gate(&unsigned).unwrap_err().message,
            "노무사/세무사 professional validation is required"
        );
    }

    #[test]
    fn release_gate_rejects_an_artifact_digest_that_is_not_64_hex() {
        let mut not_hex = satisfied_release_gate();
        not_hex
            .professional_validation
            .as_mut()
            .unwrap()
            .artifact_sha256 = "z".repeat(64);

        assert_eq!(
            validate_release_gate(&not_hex).unwrap_err().message,
            "professional validation artifact_sha256 must be a 64-character hex digest"
        );
    }

    #[test]
    fn release_gate_names_the_case_whose_declared_inputs_cannot_be_recomputed() {
        // 2027-03-01 is inside the pension base-limit window (ends 2027-07-01)
        // but outside the contribution-rate window (ends 2027-01-01), so the
        // kernel refuses rather than mismatching. The gate must attribute that
        // refusal to a case_id; only the gate-owned prefix is pinned here, not
        // the inner kernel text.
        let mut unrecomputable = satisfied_release_gate();
        unrecomputable.golden_cases[0].inputs.pay_date = date!(2027 - 03 - 01);

        let message = validate_release_gate(&unrecomputable).unwrap_err().message;

        assert!(
            message.starts_with("golden case GC-FIXTURE-A could not be recomputed: "),
            "expected a case-attributed recomputation failure, got: {message}"
        );
    }

    #[test]
    fn release_gate_recomputes_every_golden_case_and_not_only_the_first() {
        // Line coverage cannot see this: the loop body's lines are already
        // executed by every single-case test. `for case in &input.golden_cases`
        // narrowed to `.first()` or `[0]` would keep them green while every
        // case after the first went unchecked — a signed batch where only the
        // first figure is ever re-derived.
        let mut two_cases = satisfied_release_gate();
        let mut second = two_cases.golden_cases[0].clone();
        second.case_id = "GC-FIXTURE-B".to_string();
        second.expected_total_employee_deductions_won = 373_303;
        two_cases.golden_cases.push(second);

        assert_eq!(
            validate_release_gate(&two_cases).unwrap_err().message,
            "golden case GC-FIXTURE-B expects total employee deductions 373303 \
             but the payroll kernel computed 373302"
        );
    }

    #[test]
    fn release_gate_rejects_a_gate_carrying_no_golden_case_at_all() {
        // The recomputation loop is VACUOUS over an empty list — it iterates
        // nothing and returns Ok. This check is therefore the only thing
        // between "the gate re-executes the signed figures" and "the gate
        // passed having compared nothing", and it had no test of its own.
        let mut caseless = satisfied_release_gate();
        caseless.golden_cases.clear();

        assert_eq!(
            validate_release_gate(&caseless).unwrap_err().message,
            "at least one payroll golden case is required"
        );
    }

    #[test]
    fn release_gate_rejects_a_case_signed_against_a_different_rate_table_version() {
        // Recomputation cannot catch this. `rate_table_version` is a label: it
        // reaches no kernel input, so a case signed against last year's table
        // still recomputes to its own expected total and passes the arithmetic.
        // Only this string compare notices.
        let mut mismatched = satisfied_release_gate();
        mismatched.golden_cases[0].rate_table_version = "KR-2025-official-rates-v1".to_string();

        assert_eq!(
            validate_release_gate(&mismatched).unwrap_err().message,
            "golden case GC-FIXTURE-A uses mismatched rate table version"
        );
    }

    #[test]
    fn release_gate_rejects_a_blank_reviewer_reference() {
        // The remaining unexercised pre-existing condition. A 노무사/세무사
        // sign-off whose reviewer cannot be identified is an anonymous one.
        let mut anonymous = satisfied_release_gate();
        anonymous
            .professional_validation
            .as_mut()
            .unwrap()
            .reviewer_reference = "   ".to_string();

        assert_eq!(
            validate_release_gate(&anonymous).unwrap_err().message,
            "professional validation reviewer reference is required"
        );
    }

    #[test]
    fn release_gate_recomputes_a_case_on_its_declared_pension_standard_monthly_income() {
        // `pension_standard_monthly_income_won` is the one optional kernel
        // input, and it is the only one that changes a figure without changing
        // the gross. 2,000,000 sits inside the 2026-06 base window
        // (400,000..6,370,000), so the clamp does not erase it: the 국민연금
        // line drops by 47,500 and the total with it. A gate that ignored the
        // declared basis would compute 373,302 here.
        let mut capped = satisfied_release_gate();
        capped.golden_cases[0]
            .inputs
            .pension_standard_monthly_income_won = Some(2_000_000);
        capped.golden_cases[0].expected_total_employee_deductions_won = 325_802;

        validate_release_gate(&capped).unwrap();

        // ...and the gross-based figure is now the WRONG answer for this case.
        capped.golden_cases[0].expected_total_employee_deductions_won = 373_302;
        assert_eq!(
            validate_release_gate(&capped).unwrap_err().message,
            "golden case GC-FIXTURE-A expects total employee deductions 373302 \
             but the payroll kernel computed 325802"
        );
    }

    #[test]
    fn builds_severance_pay_from_moel_average_wage_formula() {
        // Ordinary daily wage (90,000) is BELOW the average daily wage (100,000),
        // so the average-wage path governs and the historical figures must be
        // byte-for-byte unchanged.
        let draft = build_severance_pay_draft(SeverancePayInput {
            hire_date: date!(2024 - 01 - 01),
            exit_date: date!(2026 - 06 - 30),
            average_wage_period_start: date!(2026 - 04 - 01),
            average_wage_period_end: date!(2026 - 06 - 30),
            average_wage_calendar_days: 91,
            average_wage_total_won: 9_100_000,
            ordinary_daily_wage_won: 90_000,
        })
        .unwrap();

        assert_eq!(draft.service_days, 912);
        assert_eq!(draft.average_daily_wage_milliwon, 100_000_000);
        // max(average 100,000, ordinary 90,000) => average governs.
        assert_eq!(draft.statutory_daily_wage_milliwon, 100_000_000);
        assert_eq!(draft.statutory_30_day_wage_won, 3_000_000);
        assert_eq!(draft.severance_pay_won, 7_495_890);
        assert_eq!(draft.source, moel_retirement_pay_source());
    }

    #[test]
    fn ordinary_wage_floor_governs_when_three_month_window_is_depressed() {
        // Absence→exit population: unpaid leave halved the 3-month wage total, so
        // the average daily wage collapses to 50,000/day. The employee's ordinary
        // daily wage is still 100,000. 통상임금 floor must govern and roughly double
        // the severance versus the average-wage-only figure.
        let draft = build_severance_pay_draft(SeverancePayInput {
            hire_date: date!(2024 - 01 - 01),
            exit_date: date!(2026 - 06 - 30),
            average_wage_period_start: date!(2026 - 04 - 01),
            average_wage_period_end: date!(2026 - 06 - 30),
            average_wage_calendar_days: 91,
            average_wage_total_won: 4_550_000, // depressed window: 50,000/day
            ordinary_daily_wage_won: 100_000,
        })
        .unwrap();

        // Average path would have produced only the depressed figure.
        assert_eq!(draft.average_daily_wage_milliwon, 50_000_000);
        let average_only_severance = 4_550_000_i128 * 30 * 912 / 91 / 365;
        assert_eq!(average_only_severance, 3_747_945);

        // Ordinary wage floor governs: max(50,000, 100,000) = 100,000.
        assert_eq!(draft.statutory_daily_wage_milliwon, 100_000_000);
        assert_eq!(draft.statutory_30_day_wage_won, 3_000_000);
        assert_eq!(draft.severance_pay_won, 7_495_890);
        assert!(
            draft.severance_pay_won > i64::try_from(average_only_severance).unwrap(),
            "ordinary-wage floor must exceed the depressed average-wage figure"
        );
    }

    #[test]
    fn severance_pay_refuses_short_service_or_missing_wage_basis() {
        let short_service = build_severance_pay_draft(SeverancePayInput {
            hire_date: date!(2026 - 01 - 01),
            exit_date: date!(2026 - 06 - 30),
            average_wage_period_start: date!(2026 - 04 - 01),
            average_wage_period_end: date!(2026 - 06 - 30),
            average_wage_calendar_days: 91,
            average_wage_total_won: 9_100_000,
            ordinary_daily_wage_won: 100_000,
        });
        assert!(short_service.is_err());

        let missing_wage = build_severance_pay_draft(SeverancePayInput {
            hire_date: date!(2024 - 01 - 01),
            exit_date: date!(2026 - 06 - 30),
            average_wage_period_start: date!(2026 - 04 - 01),
            average_wage_period_end: date!(2026 - 06 - 30),
            average_wage_calendar_days: 91,
            average_wage_total_won: 0,
            ordinary_daily_wage_won: 100_000,
        });
        assert!(missing_wage.is_err());
    }

    #[test]
    fn severance_pay_refuses_missing_ordinary_wage_floor() {
        // FAIL LOUD: absent/non-positive ordinary wage must be rejected, never
        // silently degraded to average-wage-only.
        let missing_ordinary = build_severance_pay_draft(SeverancePayInput {
            hire_date: date!(2024 - 01 - 01),
            exit_date: date!(2026 - 06 - 30),
            average_wage_period_start: date!(2026 - 04 - 01),
            average_wage_period_end: date!(2026 - 06 - 30),
            average_wage_calendar_days: 91,
            average_wage_total_won: 9_100_000,
            ordinary_daily_wage_won: 0,
        });
        assert!(missing_ordinary.is_err());
        assert!(format!("{:?}", missing_ordinary.err().unwrap()).contains("ordinary daily wage"));
    }

    fn line_amount(draft: &PayrollDraft, code: DeductionCode) -> i64 {
        draft
            .lines
            .iter()
            .find(|line| line.code == code)
            .unwrap()
            .amount_won
    }
}
