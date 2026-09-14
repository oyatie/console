//! Eleven isolated real-owner/browser cases. Missing owners are prerequisites.
use crate::{native_fixture::{NativeDeployment,TestResult,build_native_deployment},binder_sequence::{completed,bind_and_seal,explicit_new_intent},
 source_foundation_producer::{ArtifactChannel,upload_source},browser_auth_producer::{self,Browser}};
use console_ontology_application::action30::*;
use console_ontology_adapter_postgres::action30 as ontology;
use console_payroll_adapter_postgres::action30 as payroll;
use console_governance_adapter_postgres::action30 as governance;
use console_leave_adapter_postgres::account13 as workforce;
use console_identity_adapter_postgres::account13 as identity;
use console_app::ui30::{self,UiTarget};
use serde_json::{json,Value};use sqlx::PgPool;use uuid::Uuid;
#[derive(Clone,Copy)]pub enum Kind{Draft,Scoped,Historical,ReadOnly}
pub struct Prepared{pub deployment:NativeDeployment,pub target:UiTarget,pub browser:Browser,pub data:Value,pub provenance:Value}
fn marker()->String{format!("CONSOLEPRIVATE{}",&Uuid::new_v4().simple().to_string().to_uppercase()[..24])}

pub async fn draft(pool:PgPool)->TestResult<Prepared>{
 let mut d=build_native_deployment(pool,"browser-editable-wage",1).await?;let f=&d.companies[0];
 // This is editable acknowledged input for a proposed correction, never a
 // supported1.2m payroll calculation or an already committed business wage.
 let mut input=f.facts.wage_correction.clone();input.amount_won=1_200_000;
 let selected=explicit_new_intent::<payroll::WageCorrect>(&f.pool,&f.submitter,&f.facts.wage_selection).await?;
 let binding=ontology::read_registered_draft_binding::<payroll::WageCorrect>(&f.pool,&f.submitter,&selected).await?;
 let started=completed(ontology::draft_start(&f.pool,&f.submitter,UnadmittedControl{command_id:Uuid::new_v4(),input:DraftStart{
  action:binding.action,target:binding.target,schema:binding.schema.clone(),custody:binding.custody,intent_slot:binding.intent_slot,
 }}).await?)?;
 let saved=completed(ontology::draft_save(&f.pool,&f.submitter,UnadmittedControl{command_id:Uuid::new_v4(),input:DraftSave{
  draft_id:started.draft_id,expected_revision_id:started.revision_id,editing_token:started.editing_token.clone(),
  patches:compile_registered_input_patches(&binding.schema_snapshot,&input)?,
 }}).await?)?;
 let current=ontology::read_acknowledged_registered_input::<payroll::WageCorrect>(&f.pool,&f.submitter,saved.draft_id).await?;
 assert_eq!(current.revision_id,saved.revision_id);assert_eq!(current.input.amount_won,1_200_000);
 let company=identity::read_company_profile(&d.pool,&d.operator,f.org).await?;
 let target=UiTarget::Draft{draft_id:saved.draft_id};
 let provenance=json!({"draft_start":started,"draft_save":saved,"acknowledged":current,"company":company});
 let browser=browser_auth_producer::login(&d.runtime.client,&d.runtime.origin,&mut d.accounts.submitter).await?;
 Ok(Prepared{deployment:d,target,browser,data:json!({"company_label":company.name}),provenance})
}

pub async fn review(pool:PgPool,read_only:bool)->TestResult<Prepared>{
 let markers=[marker(),marker(),marker(),marker()];
 let mut d=crate::two_subject_producer::build_two_supported_deployment_with_subject_note(pool,"browser-scoped-two-subjects",&markers[2]).await?;
 authorize_profile_setup(&mut d,&["workforce.person.profile.update","workforce.employee.profile.update"]).await?;
 let f=&d.companies[0];let second=f.facts.second_supported.as_ref().ok_or("actual second supported subject missing")?;
 // Ordinary workforce CAS writes only display/profile fields, preserving actual
 // Human/Person/Employment binding. No native result or eligibility row edited.
 let person=workforce::read_person_profile(&f.pool,&f.submitter,second.workforce.person_id).await?;
 let renamed=workforce::update_person_profile(&f.pool,&f.submitter,workforce::PersonProfileUpdate{command_id:Uuid::new_v4(),
  person_id:person.person_id,expected_revision:person.revision,display_name:markers[0].clone()}).await?;
 let employee=workforce::read_employee_profile(&f.pool,&f.submitter,second.workforce.employee_id).await?;
 let numbered=workforce::update_employee_profile(&f.pool,&f.submitter,workforce::EmployeeProfileUpdate{command_id:Uuid::new_v4(),
  employee_id:employee.employee_id,expected_revision:employee.revision,employee_number:markers[1].clone()}).await?;
 let person=workforce::read_person_profile(&f.pool,&f.submitter,second.workforce.person_id).await?;
 let employee=workforce::read_employee_profile(&f.pool,&f.submitter,second.workforce.employee_id).await?;
 let wage=payroll::read_contract_wage_fact(&f.pool,&f.submitter,&second.payroll_sources.wage).await?;
 assert_eq!(person.display_name,markers[0]);assert_eq!(employee.employee_number,markers[1]);assert_eq!(wage.source_note,markers[2]);
 let channel=ArtifactChannel{client:&f.runtime.client,origin:&f.runtime.origin,token:&f.account_token};
 let visible_text="Synthetic exact source explanation for the permitted subject";
 let visible=upload_source(&f.pool,&f.submitter,&f.worker,&channel,"application/json",serde_json::to_vec(&json!({"narrative":visible_text}))?).await?;
 let hidden=upload_source(&f.pool,&f.submitter,&f.worker,&channel,"application/json",serde_json::to_vec(&json!({"narrative":markers[3]}))?).await?;
 let (run,_)=f.calculated().await?;
 let open_input=payroll::ReviewOpen{expected:payroll::read_run_control(&f.pool,&f.submitter,run.run_id).await?};
 let open_attempt=bind_and_seal(&f.pool,&f.submitter,&UntrustedActionTargetSelection::existing_run(run.run_id),&open_input).await?;
 let opened=completed(payroll::payroll_open_review_cycle(&f.pool,&f.submitter,&open_attempt).await?)?;
 let bases=payroll::read_complete_review_bases(&f.pool,&f.submitter,&opened.cycle).await?;
 let first=payroll::read_subject_review_basis(&f.pool,&f.submitter,&opened.cycle,f.facts.employee_id).await?;
 let other=payroll::read_subject_review_basis(&f.pool,&f.submitter,&opened.cycle,second.workforce.employee_id).await?;
 assert!(bases.contains(&first)&&bases.contains(&other));assert_ne!(first.basis_id,other.basis_id);
 let mut receipts=Vec::new();
 for (basis,evidence) in [(&first,&visible.source_artifact),(&other,&hidden.source_artifact)]{
  let exact=console_docs_adapter_postgres::action30::read_exact_evidence(&f.pool,&f.submitter,evidence).await?;
  let a=bind_and_seal(&f.pool,&f.submitter,&UntrustedActionTargetSelection::existing_review_basis(basis.basis_id),
   &governance::ReviewSubmit{basis:basis.clone(),attest:true,submitted_evidence:vec![exact]}).await?;
  receipts.push(completed(governance::review_submit(&f.pool,&f.submitter,&a).await?)?);
 }
 let target=UiTarget::ReviewBasis{basis:first.clone()};
 // Exact actual resource/property refs are resolved by fixed owning reads. Only
 // first-subject evidence and this decision's controls are granted; never cycle
 // global coverage, hidden subject or unrestricted Company field access.
 let allowed=payroll::read_review_basis_policy_resources(&f.pool,&f.submitter,&first).await?;
 let denied=vec![person.display_name_field.clone(),employee.employee_number_field.clone(),wage.source_note_field.clone(),
  console_docs_adapter_postgres::action30::read_json_narrative_field_resource(&f.pool,&f.submitter,&hidden.source_artifact).await?];
 restrict(&d,&allowed,read_only).await?;
 let selection=identity::read_current_company_selection(&d.pool,&d.operator,f.org).await?;
 let actor=console_platform_request_context::account::resolve_company_context(&d.pool,&d.verifier,&d.accounts.reviewer.account_access_token,&selection,&d.serving).await?;
 for field in &denied{assert_eq!(identity::read_current_field_decision(&f.pool,&actor,field).await?,identity::FieldDecision::Deny);}
 let displayed=ui30::read_target_presentation(&f.pool,&actor,&target).await?;
 assert_eq!(displayed.gross_won,Some(3_000_000));assert_eq!(displayed.evidence_narrative.as_deref(),Some(visible_text));
 let provenance=json!({"person_update":renamed,"employee_update":numbered,"person":person,"employee":employee,"wage":wage,
  "visible_artifact":visible.source_artifact,"hidden_artifact":hidden.source_artifact,"review_open":opened,"submissions":receipts,"denied_fields":denied,
  "run_id":run.run_id,"actual_readback":f.snapshot(run.run_id).await?,"current_presentation":displayed});
 let data=json!({"company_label":displayed.company_label,"visible_subject_label":displayed.subject_label,
  "visible_evidence_narrative":visible_text,"canaries":markers.to_vec()});
 let browser=browser_auth_producer::login(&d.runtime.client,&d.runtime.origin,&mut d.accounts.reviewer).await?;
 Ok(Prepared{deployment:d,target,browser,data,provenance})
}

async fn restrict(d:&NativeDeployment,resources:&payroll::ReviewBasisPolicyResources,read_only:bool)->TestResult{
 let f=&d.companies[0];let account=d.accounts.reviewer.account_id;
 for grant in identity::read_current_company_grants_for_account(&d.pool,&d.operator,f.org,account).await?{
  let expected=identity::read_company_policy_control(&d.pool,&d.operator,f.org).await?;
  identity::revoke_company_grant(&d.pool,&d.operator,identity::RevokeCompanyGrant{command_id:Uuid::new_v4(),org_id:f.org,grant_id:grant.grant_id,expected,reason:"Exact browser subject projection".into()}).await?;
 }
 let schema=identity::read_company_policy_schema(&d.pool,&d.operator,f.org).await?;
 let actions=if read_only{schema.actions_for_review_read()?}else{schema.actions_for_review_read_and_decide()?};
 for r in &resources.individually_disclosable {
  let expected=identity::read_company_policy_control(&d.pool,&d.operator,f.org).await?;
  identity::apply_company_grant_plan(&d.pool,&d.operator,identity::ApplyCompanyGrantPlan{command_id:Uuid::new_v4(),org_id:f.org,expected,
   plan:identity::CompanyGrantPlan{account_id:account,scope:identity::PolicyScope::ExactResource{resource:r.object.clone(),schema:r.schema.clone()},
    actions:actions.clone(),field_projection:schema.projection("native30.standard")?,valid_from:None,valid_to:None,reason:"One actual subject and exact decision controls only".into()}}).await?;
 }
 Ok(())
}

pub async fn historical(pool:PgPool)->TestResult<Prepared>{
 let mut s=crate::workflow_fixture::fixture(pool,"browser-historical-own",1).await?;
 authorize_profile_setup(&mut s.deployment,&["workforce.employment.end"]).await?;
 let(publication,attempt,approved)=s.publication().await?;
 let f=s.company();let control=workforce::read_employment_change_control(&f.pool,&f.submitter,f.facts.employment.employment_id).await?;
 let ended=workforce::end_employment(&f.pool,&f.submitter,workforce::EmploymentEnd{command_id:Uuid::new_v4(),employment_id:f.facts.employment.employment_id,
  expected:control,ends_before:time::macros::date!(2026-07-01),reason:"Synthetic current termination after complete historical earning month".into()}).await?;
 let actual=workforce::read_employment_change_control(&f.pool,&f.submitter,f.facts.employment.employment_id).await?;
 assert_eq!(actual.ends_before,Some(time::macros::date!(2026-07-01)));
 let target=UiTarget::Publication{publication:publication.publications[0].clone()};
 let own_resources=payroll::read_own_publication_policy_resources(&f.pool,&f.submitter,&publication.publications[0]).await?;
 assert_eq!(own_resources.subject_person_id,f.facts.workforce.person_id);
 restrict_own_publication(&s.deployment,&own_resources).await?;
 let current=identity::read_current_company_selection(&s.deployment.pool,&s.deployment.operator,f.org).await?;
 let own=console_platform_request_context::account::resolve_company_context(&f.pool,&s.deployment.verifier,&s.deployment.accounts.submitter.account_access_token,&current,&s.deployment.serving).await?;
 let displayed=ui30::read_target_presentation(&f.pool,&own,&target).await?;
 assert_eq!(displayed.gross_won,Some(3_000_000));assert_eq!(displayed.purpose.as_deref(),Some("NONPAYABLE_REVIEW"));assert!(displayed.can_request_correction);assert!(!displayed.payable);
 let provenance=json!({"publication":publication,"attempt":attempt,"approval":approved.completion,"ended":ended,"employment":actual,"current_presentation":displayed,"run_id":approved.created.run_id,"actual_readback":f.snapshot(approved.created.run_id).await?});
 let data=json!({"historical_period":displayed.period_label,"company_label":displayed.company_label});
 let mut d=s.deployment;
 let browser=browser_auth_producer::login(&d.runtime.client,&d.runtime.origin,&mut d.accounts.submitter).await?;
 Ok(Prepared{deployment:d,target,browser,data,provenance})
}

async fn authorize_profile_setup(d:&mut NativeDeployment,keys:&[&str])->TestResult{
 let org=d.companies[0].org;let account=d.accounts.submitter.account_id;
 let plan=identity::read_registered_capability_grant_plan(&d.pool,&d.operator,org,account,keys).await?;
 let expected=identity::read_company_policy_control(&d.pool,&d.operator,org).await?;
 identity::apply_company_capability_grant(&d.pool,&d.operator,identity::ApplyCompanyCapabilityGrant{command_id:Uuid::new_v4(),org_id:org,expected,plan}).await?;
 let selection=identity::read_current_company_selection(&d.pool,&d.operator,org).await?;
 use console_platform_request_context::account::resolve_company_context;
 let submitter=resolve_company_context(&d.pool,&d.verifier,&d.accounts.submitter.account_access_token,&selection,&d.serving).await?;
 let reviewer=resolve_company_context(&d.pool,&d.verifier,&d.accounts.reviewer.account_access_token,&selection,&d.serving).await?;
 let alias=resolve_company_context(&d.pool,&d.verifier,&d.accounts.same_human_alias.account_access_token,&selection,&d.serving).await?;
 let challenge=identity::begin_workload_authentication(&d.pool,&d.runtime.config.workload.key_id,org).await?;
 let proof=d.runtime.config.workload.sign(challenge.signing_bytes())?;
 let credential=identity::finish_workload_authentication(&d.pool,challenge,&proof).await?;
 let worker=console_platform_request_context::worker::resolve_worker_context(&d.pool,&credential.access_token,&selection,&d.serving).await?;
 let f=&mut d.companies[0];f.submitter=submitter;f.reviewer=reviewer;f.same_human_other_account=alias;f.worker=worker;Ok(())
}

async fn restrict_own_publication(d:&NativeDeployment,resources:&payroll::OwnPublicationPolicyResources)->TestResult{
 let f=&d.companies[0];let account=d.accounts.submitter.account_id;
 for grant in identity::read_current_company_grants_for_account(&d.pool,&d.operator,f.org,account).await?{
  let expected=identity::read_company_policy_control(&d.pool,&d.operator,f.org).await?;
  identity::revoke_company_grant(&d.pool,&d.operator,identity::RevokeCompanyGrant{command_id:Uuid::new_v4(),org_id:f.org,grant_id:grant.grant_id,expected,reason:"Exact browser subject projection".into()}).await?;
 }
 let schema=identity::read_company_policy_schema(&d.pool,&d.operator,f.org).await?;
 let actions=schema.actions_for_own_publication_read_and_request()?;
 for r in &resources.individually_disclosable {
  let expected=identity::read_company_policy_control(&d.pool,&d.operator,f.org).await?;
  identity::apply_company_grant_plan(&d.pool,&d.operator,identity::ApplyCompanyGrantPlan{command_id:Uuid::new_v4(),org_id:f.org,expected,
   plan:identity::CompanyGrantPlan{account_id:account,scope:identity::PolicyScope::ExactResource{resource:r.object.clone(),schema:r.schema.clone()},
    actions:actions.clone(),field_projection:schema.projection("native30.standard")?,valid_from:None,valid_to:None,reason:"Own actual historical publication and correction entry only".into()}}).await?;
 }
 Ok(())
}
