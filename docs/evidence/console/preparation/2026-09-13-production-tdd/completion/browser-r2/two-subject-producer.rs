//! Explicit same-Company two-supported-subject fixture. Both actual Employment
//! populations are fully produced through ordinary owners; no second result row
//! or target is guessed. Used for atomic hidden/forbidden publication target tests.
use console_payroll_adapter_postgres::action30 as payroll;
use console_attendance_adapter_postgres::action30 as attendance;
use console_ontology_application::action30::*;
use sqlx::PgPool;
use uuid::Uuid;
use crate::{native_fixture::{build_native_deployment,NativeDeployment,TestResult},
 source_foundation_producer::{ArtifactChannel,upload_source,signed_issuer_document},ordinary_issuer_bootstrap::FixtureIssuer,
 attendance_owner_producer::produce_attendance,payroll_source_facts_producer::{produce_payroll_sources_with_note,admit_payroll_sources},
 binder_sequence::{bind_and_seal,completed,explicit_new_intent}};
pub async fn build_two_supported_deployment(pool:PgPool,scenario:&str)->TestResult<NativeDeployment> {
 build_two_supported_deployment_with_subject_note(pool,scenario,"Synthetic existing domain fixture").await
}
pub async fn build_two_supported_deployment_with_subject_note(pool:PgPool,scenario:&str,second_wage_note:&str)->TestResult<NativeDeployment> {
 let mut d=build_native_deployment(pool,scenario,1).await?;
 let f=&mut d.companies[0];let second=f.facts.unprepared_workforce.clone();
 let scope=payroll::create_scope_basis(&f.pool,&f.submitter,payroll::CreateScopeBasis {
  command_id:Uuid::new_v4(),employee_ids:vec![f.facts.employee_id,second.employee_id],reason:"Exact two-supported-subject test scope".into(),
 }).await?.reference;
 let profile=payroll::create_supported_profile(&f.pool,&f.submitter,payroll::CreateSupportedProfile {
  command_id:Uuid::new_v4(),rule_bundle:f.facts.sources.rule_bundle.clone(),qualification:f.facts.sources.bundle.admission_ref.clone(),
  scope_basis:scope.clone(),period:f.facts.choices.create.period.clone(),pay_date:f.facts.choices.create.pay_date,
  currency:"KRW".into(),monthly_standard_hours:209,
 }).await?.reference;
 let issuer=FixtureIssuer::from_config(f.runtime.config.synthetic_issuer.issuer.clone(),f.runtime.config.synthetic_issuer.key_id.clone(),
  f.runtime.config.synthetic_issuer.public_pem.clone(),&f.runtime.config.synthetic_issuer.private_pem)?;
 let now=time::OffsetDateTime::now_utc().unix_timestamp();
 let evidence=serde_json::json!({"iss":issuer.issuer(),"aud":"console.payroll-source.v1","iat":now,"nbf":now,"exp":now+3600,"jti":Uuid::new_v4(),
  "classification":"SYNTHETIC","org_id":f.org,"employee_id":second.employee_id,"employment":second.employment,
  "profile":profile,"period":{"start":"2026-06-01","end":"2026-06-30"},"gross_won":"3000000","standard_hours":"209",
  "pension_basis_won":"3000000","remuneration_won":"3000000","taxable_won":"3000000",
  "dependent_count":"1","eligible_child_count":"0","withholding_option":"P100",
  "insurance":["NATIONAL_PENSION","HEALTH","LONG_TERM_CARE","EMPLOYMENT"],"income_tax_won":"74350","local_tax_won":"7430"});
 let channel=ArtifactChannel{client:&f.runtime.client,origin:&f.runtime.origin,token:&f.account_token};
 let actual=upload_source(&f.pool,&f.submitter,&f.worker,&channel,"application/json",signed_issuer_document(&issuer.sign(&evidence)?)?).await?;
 // Common rules/table are reusable admitted facts; personal eligibility evidence
 // is separately signed for the actual second Employment and shared profile.
 let mut second_sources=f.facts.sources.clone();second_sources.evidence=actual.source_artifact.clone();second_sources.profile=profile.clone();
 let second_attendance=produce_attendance(&f.pool,&f.submitter,&f.worker,&channel,second.employee_id,second.employment.employment_id,
  &f.facts.calendar_rule,&second.scope,f.facts.choices.create.period.clone(),false).await?;
 let second_payroll=produce_payroll_sources_with_note(&f.pool,&f.submitter,&f.reviewer,&second,&second_sources,&second_attendance.coverage,second_wage_note).await?;
 let mut first_evidence=evidence.clone();first_evidence["employee_id"]=serde_json::json!(f.facts.employee_id);
 first_evidence["employment"]=serde_json::to_value(&f.facts.employment)?;first_evidence["jti"]=serde_json::json!(Uuid::new_v4());
 let first_artifact=upload_source(&f.pool,&f.submitter,&f.worker,&channel,"application/json",signed_issuer_document(&issuer.sign(&first_evidence)?)?).await?;
 let mut first_sources=f.facts.sources.clone();first_sources.profile=profile.clone();first_sources.evidence=first_artifact.source_artifact.clone();
 let old=&f.facts.payroll_sources;
 let first_payroll=admit_payroll_sources(&f.pool,&f.submitter,&f.reviewer,&f.facts.workforce,&first_sources,&f.facts.coverage,
  old.wage.clone(),old.wage_attempt.clone(),old.wage_input.clone(),Some(old.eligibility.admission_ref.clone()),Some(old.tax.admission_ref.clone())).await?;
 let control=attendance::read_operational_close_control(&f.pool,&f.submitter,&scope,&f.facts.choices.create.period).await?;
 let operational=attendance::close_operational_period(&f.pool,&f.submitter,UnadmittedControl{command_id:Uuid::new_v4(),input:control}).await?;
 let members=attendance::enumerate_close_members(&f.pool,&f.submitter,&scope,&f.facts.choices.create.period,&operational.reference).await?;
 assert_eq!(members.rows.len(),2);
 let member_ids=members.rows.iter().map(|m|m.employee_id).collect::<std::collections::BTreeSet<_>>();
 assert_eq!(member_ids,std::collections::BTreeSet::from([f.facts.employee_id,second.employee_id]));
 let artifact=upload_source(&f.pool,&f.submitter,&f.worker,&channel,"application/x-ndjson",attendance::encode_source19_close_members(&members)?).await?;
 let input=attendance::CloseBasisCreate{kind:attendance::CloseKind::AttendanceCloseBasis,period:f.facts.choices.create.period.clone(),scope_basis_ref:scope.clone(),
  operational_close_ref:operational.reference,member_count:members.count,member_set_digest:members.digest,
  coverage_members_artifact_ref:artifact.source_artifact,attestation:true};
 let selected=explicit_new_intent::<attendance::CloseBasisCreate>(&f.pool,&f.submitter,&UntrustedActionTargetSelection::company()).await?;
 let a=bind_and_seal(&f.pool,&f.submitter,&selected,&input).await?;
 let close=completed(attendance::attendance_create_close_basis(&f.pool,&f.submitter,&a).await?)?;
 f.facts.close_basis=close.close_basis;
 f.facts.scope=scope.clone();f.facts.profile=profile.clone();
 f.facts.sources=first_sources;f.facts.payroll_sources=first_payroll;
 f.facts.choices.create.scope=scope.clone();f.facts.choices.create.profile=profile;
 let exact_first=console_docs_adapter_postgres::action30::read_exact_evidence(&f.pool,&f.submitter,&first_artifact.source_artifact).await?;
 let exact_second=console_docs_adapter_postgres::action30::read_exact_evidence(&f.pool,&f.submitter,&actual.source_artifact).await?;
 // Both current-profile signed statements are actual authenticated Docs reads;
 // no single-subject predecessor evidence survives in prepare/review choices.
 f.facts.choices.preparation_evidence=vec![exact_first.clone(),exact_second.clone()];
 f.facts.choices.review_evidence=vec![exact_first,exact_second];
 f.facts.choices.source_refs=payroll::read_complete_native_source_refs(&f.pool,&f.submitter,&scope,&f.facts.choices.create.period).await?;
 // Current owner read must expose BOTH fully admitted exact subject facts.
 let eligible=payroll::read_complete_supported_scope_subjects(&f.pool,&f.submitter,&scope,&f.facts.choices.create.period).await?;
 assert_eq!(eligible.len(),2);assert!(eligible.iter().any(|x|x.eligibility==second_payroll.eligibility.admission_ref));
 assert!(eligible.iter().any(|x|x.eligibility==f.facts.payroll_sources.eligibility.admission_ref));
 f.facts.second_supported=Some(crate::native_fixture::SupportedSubjectFacts{workforce:second,sources:second_sources,payroll_sources:second_payroll,attendance:second_attendance});
 Ok(d)
}
