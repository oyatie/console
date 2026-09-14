//! Real equal-amount wage correction and newly admitted exact eligibility/tax refs.
//! Equal money does not imply equal source meaning or transferable human approval.
use console_ontology_application::action30::*;
use console_payroll_adapter_postgres::action30 as payroll;
use crate::{native_fixture::{Fixture,TestResult},binder_sequence::{bind_and_seal,completed},
 source_owner_producer::{produce_verified_eligibility,produce_verified_tax_evidence}};
pub async fn correct_review_sources(f:&Fixture)->TestResult<Vec<ReceiptRef>> {
 let mut correction=f.facts.wage_correction.clone();correction.amount_won=3_000_000;
 correction.source_note="Synthetic corrected provenance, identical amount".into();
 let a=bind_and_seal(&f.pool,&f.submitter,&f.facts.wage_selection,&correction).await?;
 let wage=completed(payroll::payroll_correct_contract_wage(&f.pool,&f.submitter,&a).await?)?;
 let mut eligibility=serde_json::to_value(&f.facts.payroll_sources.eligibility_payload)?;
 eligibility["payload"]["wage_ref"]=serde_json::to_value(&wage.fact)?;
 eligibility["supersedes_ref"]=serde_json::to_value(&f.facts.payroll_sources.eligibility.admission_ref)?;
 let selection=payroll::read_source_collection_selection(&f.pool,&f.submitter,"ELIGIBILITY").await?;
 let eligible=produce_verified_eligibility(&f.pool,&f.submitter,&f.reviewer,&selection,serde_json::from_value(eligibility)?).await?;
 let mut tax=serde_json::to_value(&f.facts.payroll_sources.tax_payload)?;
 tax["payload"]["assessment_ref"]=serde_json::to_value(&eligible.admission_ref)?;
 tax["supersedes_ref"]=serde_json::to_value(&f.facts.payroll_sources.tax.admission_ref)?;
 let selection=payroll::read_source_collection_selection(&f.pool,&f.submitter,"TAX_EVIDENCE").await?;
 let tax=produce_verified_tax_evidence(&f.pool,&f.submitter,&f.reviewer,&selection,serde_json::from_value(tax)?).await?;
 Ok(vec![wage.receipt,eligible.last_effect_receipt,tax.last_effect_receipt])
}
pub async fn complete_review_revision(f:&Fixture,a:&AttemptRef,initial:OwnerExecution<payroll::ReviewResult>)->TestResult<payroll::ReviewResult> {
 let work=async {
  let mut value=initial;
  for _ in 0..64 {
   match value {
    OwnerExecution::Completed(actual)=>{
     match &actual.receipt {ReceiptRef::Company{command,..}=>{assert_eq!(command.org_id,a.org_id);assert_eq!(command.command_id,a.command_id);},_=>return Err("wrong receipt namespace".into())}
     return Ok(actual)
    },
    OwnerExecution::Observation(ref observed @ ResultObservation::Pending{..})=>{
     let subject=observed.subject().ok_or("missing pending subject")?.clone();
     match &subject {ReconciliationSubject::CompanyCommand{command} if command.org_id==a.org_id&&command.command_id==a.command_id=>{},
      ReconciliationSubject::Attempt{attempt} if attempt==a=>{},_=>return Err("changed original review revision subject".into())}
     payroll::advance_native_preparation_once(&f.pool,&f.worker,&subject).await?;
     value=payroll::observe_review_revision_command(&f.pool,&f.submitter,&subject).await?;
    },
    OwnerExecution::Observation(other)=>return Err(crate::binder_sequence::ProducerStop::Observation(other).into()),
   }
  }
  Err::<payroll::ReviewResult,Box<dyn std::error::Error+Send+Sync>>("bounded revision progression exhausted".into())
 };
 tokio::time::timeout(std::time::Duration::from_secs(60),work).await?
}
