//! Eligibility/tax input source composition from ACTUAL completed predecessors.
use console_ontology_application::action30::*;
use crate::binder_sequence::{bind_and_seal,completed};
use console_payroll_adapter_postgres::action30 as payroll;
use console_platform_request_context::account::AuthenticatedCompanyContext;
use sqlx::PgPool;
use serde_json::json;
use uuid::Uuid;
use crate::{native_fixture::TestResult,source_foundation_producer::SourceFoundations,
 workforce_owner_producer::WorkforceFacts,source_owner_producer::{produce_verified_eligibility,produce_verified_tax_evidence}};
pub struct PayrollSourceFacts {
 pub wage:payroll::WageFactRef, pub wage_attempt:AttemptRef,pub wage_input:payroll::WageCreate,
 pub eligibility:payroll::EligibilityAdmissionControl,
 pub tax:payroll::TaxEvidenceAdmissionControl,
 pub eligibility_payload:payroll::AdmissionPayload,
 pub tax_payload:payroll::AdmissionPayload,
}
pub async fn produce_payroll_sources(pool:&PgPool,auth:&AuthenticatedCompanyContext,reviewer:&AuthenticatedCompanyContext,
 workforce:&WorkforceFacts,sources:&SourceFoundations,coverage:&payroll::AttendanceCoverageRef,
)->TestResult<PayrollSourceFacts> {
 produce_payroll_sources_with_note(pool,auth,reviewer,workforce,sources,coverage,"Synthetic existing domain fixture").await
}
pub async fn produce_payroll_sources_with_note(pool:&PgPool,auth:&AuthenticatedCompanyContext,reviewer:&AuthenticatedCompanyContext,
 workforce:&WorkforceFacts,sources:&SourceFoundations,coverage:&payroll::AttendanceCoverageRef,source_note:&str,
)->TestResult<PayrollSourceFacts> {
 let current=payroll::read_contract_wage_creation_control(pool,auth,workforce.employee_id,workforce.employment.employment_id).await?;
 let evidence=console_docs_adapter_postgres::action30::read_governed_artifact_revision(pool,auth,&sources.evidence).await?;
 let wage_input:payroll::WageCreate=serde_json::from_value(json!({
  "employee_id":workforce.employee_id,"employment_id":workforce.employment.employment_id,
  "amount_won":"3000000","wage_kind":"MONTHLY","monthly_standard_hours":209,
  "effective_from":"2026-06-01","source_note":source_note,
  "evidence_refs":[evidence],"subject_source_generation":current.subject_source_generation,
 }))?;
 let wage_attempt=bind_and_seal(pool,auth,&current.selection,&wage_input).await?;
 let wage=completed(payroll::payroll_create_contract_wage(pool,auth,&wage_attempt).await?)?;
 match &wage.receipt {ReceiptRef::Company{command,..}=>{assert_eq!(command.command_id,wage_attempt.command_id);assert_eq!(command.org_id,wage_attempt.org_id);},_=>panic!("Company wage returned Group receipt")}
 admit_payroll_sources(pool,auth,reviewer,workforce,sources,coverage,wage.fact,wage_attempt,wage_input,None,None).await
}
pub async fn admit_payroll_sources(pool:&PgPool,auth:&AuthenticatedCompanyContext,reviewer:&AuthenticatedCompanyContext,
 workforce:&WorkforceFacts,sources:&SourceFoundations,coverage:&payroll::AttendanceCoverageRef,
 wage:payroll::WageFactRef,wage_attempt:AttemptRef,wage_input:payroll::WageCreate,
 previous_eligibility:Option<payroll::EligibilityRef>,previous_tax:Option<payroll::TaxEvidenceRef>,
)->TestResult<PayrollSourceFacts> {
 let period=json!({"start":"2026-06-01","end":"2026-06-30"});
 let definition=&sources.applicability.admission_ref;
 let evidence=&sources.evidence;
 let selection=json!({"taxable_monthly_won":"3000000","dependent_count_including_self":"1","eligible_child_count":"0",
  "withholding_option":"P100","selection_fact_refs":[evidence],"income_definition_ref":definition});
 let insurance=["NATIONAL_PENSION","HEALTH","LONG_TERM_CARE","EMPLOYMENT"].iter().map(|scheme|
  json!({"scheme":scheme,"status":"APPLIES","qualification_ref":definition,"evidence_refs":[evidence]})).collect::<Vec<_>>();
 let obligations=["PRORATION","OVERTIME","NIGHT","HOLIDAY_PREMIUM","UNPAID_REDUCTION","RETRO_ADJUSTMENT","LATE_ABSENCE_EARLY_LEAVE"].iter().map(|family|
  json!({"family":family,"outcome":"NONE_DUE","qualification_ref":definition,"coverage_ref":coverage,"evidence_refs":[evidence]})).collect::<Vec<_>>();
 let assessed=json!({"amount_won":"3000000","definition_ref":definition,"evidence_refs":[evidence]});
 let eligibility_payload:payroll::AdmissionPayload=serde_json::from_value(json!({"family":"ELIGIBILITY","supersedes_ref":previous_eligibility,"payload":{
  "kind":"PAYROLL_ELIGIBILITY","employee_id":workforce.employee_id,"employment_coverage":workforce.employment,
  "period":period,"pay_date":"2026-06-27","profile_ref":sources.profile,"wage_ref":wage,"coverage_ref":coverage,
  "insurance":insurance,"pension_basis":assessed,"remuneration_basis":assessed,"taxable_basis":assessed,
  "obligations":obligations,"selection":selection,"outcome":"SUPPORTED","blockers":[]}}))?;
 let target=payroll::read_source_collection_selection(pool,auth,"ELIGIBILITY").await?;
 let eligibility=produce_verified_eligibility(pool,auth,reviewer,&target,eligibility_payload.clone()).await?;
 let locator=|column:&str|json!({"kind":"CSV_RECORD","record_ordinal":"1","column_name":column,"locator_schema_ref":sources.tax.admission_ref});
 let tax_payload:payroll::AdmissionPayload=serde_json::from_value(json!({"family":"TAX_EVIDENCE","supersedes_ref":previous_tax,"payload":{
  "kind":"TAX_EVIDENCE","source_qualification_ref":sources.tax.admission_ref,"artifact_ref":sources.tax_artifact,
  "table_edition":"NTS_FIXTURE_ROW_V1","assessment_ref":eligibility.admission_ref,
  "employment_coverage":workforce.employment,"period":period,"pay_date":"2026-06-27",
  "income_band":{"lower_inclusive_won":"3000000","upper_exclusive_won":"3010000"},"selection":selection,
  "amounts":{"amount_stage":"FINAL_WITHHOLDING","income_tax_won":"74350","local_tax_won":"7430",
   "income_locator":locator("income_tax_won"),"local_locator":locator("local_tax_won"),"rule_note_locators":[]}}}))?;
 let target=payroll::read_source_collection_selection(pool,auth,"TAX_EVIDENCE").await?;
 let tax=produce_verified_tax_evidence(pool,auth,reviewer,&target,tax_payload.clone()).await?;
 Ok(PayrollSourceFacts{wage,wage_attempt,wage_input,eligibility,tax,eligibility_payload,tax_payload})
}
