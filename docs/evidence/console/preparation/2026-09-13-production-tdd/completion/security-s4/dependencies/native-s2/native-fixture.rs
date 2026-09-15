//! Fresh ordinary deployment composition. No seed index, invented success rows,
//! business INSERTs or fabricated authenticated contexts.
use console_ontology_application::action30::*;
use console_payroll_adapter_postgres::action30 as payroll;
use console_identity_adapter_postgres::account13 as identity;
use console_platform_auth::{JwtVerifier,account_session::VerifiedAccountSession};
use console_platform_request_context::{account::{resolve_company_context,AuthenticatedCompanyContext},worker::{resolve_worker_context,AuthenticatedWorkerContext},serving::{admit_current_process,ServingAdmissionBinding}};
use sqlx::PgPool;
use uuid::Uuid;
use std::sync::Arc;
use crate::{binder_sequence::{bind_and_seal,completed,completed_attempt},native_owner_producer::{NativeUserChoices,complete_native},
 account_enrollment_producer::{enroll_fixture_accounts,AccountBootstrapPhase},
 ordinary_issuer_bootstrap::{enroll_authority_and_companies,FixtureIssuer},
 publish_then_root_producer::publish_genesis_then_enroll_operator,
 deployment_runtime_producer::{start_runtime,Runtime},workforce_owner_producer::{enroll_workforce,WorkforceFacts},
 source_foundation_producer::{produce_source_foundations,ArtifactChannel,SourceFoundations},
 payroll_source_facts_producer::{produce_payroll_sources,PayrollSourceFacts},
 attendance_owner_producer::{produce_attendance,AttendanceFacts}};
pub type TestResult<T=()> = Result<T,Box<dyn std::error::Error+Send+Sync>>;
pub struct SupportedSubjectFacts {
 pub workforce:WorkforceFacts,pub sources:SourceFoundations,pub payroll_sources:PayrollSourceFacts,pub attendance:AttendanceFacts,
}
pub struct FixtureFacts {
 pub second_supported:Option<SupportedSubjectFacts>,
 pub employee_id:Uuid,pub employment:payroll::EmploymentCoverage,pub scope:payroll::ScopeBasisRef,pub profile:payroll::ProfileRef,
 pub calendar_rule:payroll::QualificationRef,
 pub calendar:payroll::CalendarRef,pub assignment:payroll::CalendarAssignmentRef,
 pub coverage:payroll::AttendanceCoverageRef,pub close_basis:Option<payroll::AttendanceCloseBasisRef>,
 pub workforce:WorkforceFacts,pub unprepared_workforce:WorkforceFacts,pub sources:SourceFoundations,pub payroll_sources:PayrollSourceFacts,pub attendance:AttendanceFacts,
 pub choices:NativeUserChoices,pub blocked_choices:NativeUserChoices,
 pub wage_selection:UntrustedActionTargetSelection,pub wage_correction:payroll::WageCorrect,
}
pub struct Fixture {
 pub pool:PgPool,pub readback:PgPool,pub org:Uuid,
 pub submitter:AuthenticatedCompanyContext,pub reviewer:AuthenticatedCompanyContext,
 pub same_human_other_account:AuthenticatedCompanyContext,pub worker:AuthenticatedWorkerContext,
 pub facts:FixtureFacts,
 // Keep actual local router/object storage alive even when fixture() extracts a Company.
 pub runtime:Arc<Runtime>,pub account_token:String,
}
pub struct NativeDeployment {
 pub pool:PgPool,pub readback:PgPool,pub verifier:Arc<JwtVerifier>,pub serving:ServingAdmissionBinding,
 pub group_id:Uuid,pub operator:VerifiedAccountSession,pub accounts:AccountBootstrapPhase,
 pub companies:Vec<Fixture>,pub runtime:Arc<Runtime>,
}
pub async fn fixture(pool:PgPool,scenario:&str)->TestResult<Fixture> {
 let mut deployment=build_native_deployment(pool,scenario,1).await?;
 deployment.companies.pop().ok_or_else(||"missing actual Company".into())
}
pub struct IdentityCompany {
 pub org:Uuid,pub submitter:AuthenticatedCompanyContext,pub reviewer:AuthenticatedCompanyContext,
 pub same_human_other_account:AuthenticatedCompanyContext,pub worker:AuthenticatedWorkerContext,
}
pub struct IdentityDeployment {
 pub pool:PgPool,pub readback:PgPool,pub verifier:Arc<JwtVerifier>,pub serving:ServingAdmissionBinding,
 pub group_id:Uuid,pub provider:identity::IdentityProviderRevision,pub operator:VerifiedAccountSession,
 pub accounts:AccountBootstrapPhase,pub companies:Vec<IdentityCompany>,pub runtime:Arc<Runtime>,
}
pub async fn build_native_deployment(pool:PgPool,scenario:&str,company_count:usize)->TestResult<NativeDeployment> {
 finish_business_fixture(build_identity_deployment(pool,scenario,company_count).await?).await
}
pub async fn build_identity_deployment(pool:PgPool,scenario:&str,company_count:usize)->TestResult<IdentityDeployment> {
 assert!((1..=6).contains(&company_count));
 let runtime=Arc::new(start_runtime(&pool,scenario).await?);
 build_identity_deployment_with_runtime(runtime,scenario,company_count).await
}
pub async fn build_identity_deployment_with_runtime(runtime:Arc<Runtime>,scenario:&str,company_count:usize)->TestResult<IdentityDeployment> {
 assert!((1..=6).contains(&company_count));
 let operator=publish_genesis_then_enroll_operator(&runtime.custody,&runtime.startup,&runtime.passkeys,&runtime.verifier,
  &runtime.origin,&runtime.config.account_root_secret).await?;
 let accounts=enroll_fixture_accounts(&runtime.auth,&runtime.passkeys,&runtime.verifier,&runtime.origin).await?;
 let issuer=FixtureIssuer::from_config(runtime.config.synthetic_issuer.issuer.clone(),runtime.config.synthetic_issuer.key_id.clone(),
  runtime.config.synthetic_issuer.public_pem.clone(),&runtime.config.synthetic_issuer.private_pem)?;
 let account_ids=[accounts.submitter.account_id,accounts.reviewer.account_id,accounts.same_human_alias.account_id,accounts.transfer_recipient.account_id];
 let mut grants=Vec::new();
 for account_id in account_ids {
  // Exact already deployed registration/projection keys are untrusted choices;
  // per-Company action refs are resolved AFTER each Company exists in B6.
  grants.push(identity::CompanyGrantChoices{account_id,
   action_keys:serde_json::from_str::<Vec<String>>(include_str!("registered-action-keys.json"))?.into_iter().filter(|key|!key.starts_with("group.operation.")).collect(),
   field_projection_key:"native30.standard".into(),scope_intent:identity::ScopeIntent::Company,
   valid_from:time::OffsetDateTime::now_utc(),valid_to:time::OffsetDateTime::now_utc()+time::Duration::hours(1),
   reason:"Explicit TEST_ONLY acceptance operator grants".into()});
 }
 let names=(0..company_count).map(|i|format!("{scenario}-{i}")).collect::<Vec<_>>();
 let bootstrap=enroll_authority_and_companies(&runtime.runtime,&operator,&accounts,&issuer,&names,&grants).await?;
 let serving=admit_current_process(&runtime.runtime).await?;
 let mut companies=Vec::new();
 for company in &bootstrap.companies {
  // Ordinary operator grants setup privileges as registered capabilities after
  // Company existence; this cannot grant arbitrary action refs or bypass Cedar.
  for account in [accounts.submitter.account_id,accounts.reviewer.account_id] {
   let keys=if account==accounts.reviewer.account_id {vec!["source.credential.authenticity.verify"]}
    else {vec!["source.credential.authenticity.verify","workforce.person.enroll","workforce.employee.enroll","payroll.scope.create","payroll.profile.create",
     "attendance.manual.record_historical","ontology.schema.publish","docs.retention.manage"]};
   let grant=identity::read_registered_capability_grant_plan(&runtime.runtime,&operator,company.org_id,account,&keys).await?;
   let expected=identity::read_company_policy_control(&runtime.runtime,&operator,company.org_id).await?;
   identity::apply_company_capability_grant(&runtime.runtime,&operator,identity::ApplyCompanyCapabilityGrant {
    command_id:Uuid::new_v4(),org_id:company.org_id,expected,plan:grant,
   }).await?;
  }
  let workload_plan=identity::read_registered_workload_grant_plan(&runtime.runtime,&operator,company.org_id,&runtime.config.workload.key_id,
   &["artifact.verify_version","native.advance","draft.expire","group.child.claim","group.child.dispatch"]).await?;
  let expected=identity::read_company_policy_control(&runtime.runtime,&operator,company.org_id).await?;
  identity::apply_workload_company_grant(&runtime.runtime,&operator,identity::ApplyWorkloadCompanyGrant {
   command_id:Uuid::new_v4(),org_id:company.org_id,expected,plan:workload_plan,
  }).await?;
  let selection=identity::read_current_company_selection(&runtime.runtime,&operator,company.org_id).await?;
  let submitter=resolve_company_context(&runtime.runtime,&runtime.verifier,&accounts.submitter.account_access_token,&selection,&serving).await?;
  let reviewer=resolve_company_context(&runtime.runtime,&runtime.verifier,&accounts.reviewer.account_access_token,&selection,&serving).await?;
  let same_human_other_account=resolve_company_context(&runtime.runtime,&runtime.verifier,&accounts.same_human_alias.account_access_token,&selection,&serving).await?;
  // Ordinary workload challenge authentication. Configured private key signs
  // actual challenge; issuance owner verifies key, audience, current registered
  // workload grant and Company before minting a short-lived worker credential.
  let challenge=identity::begin_workload_authentication(&runtime.runtime,&runtime.config.workload.key_id,company.org_id).await?;
  let proof=runtime.config.workload.sign(challenge.signing_bytes())?;
  let credential=identity::finish_workload_authentication(&runtime.runtime,challenge,&proof).await?;
  let worker=resolve_worker_context(&runtime.runtime,&credential.access_token,&selection,&serving).await?;
  assert_eq!(submitter.company_id(),company.org_id);
  companies.push(IdentityCompany{org:company.org_id,submitter,reviewer,same_human_other_account,worker});
 }
 Ok(IdentityDeployment{pool:runtime.runtime.clone(),readback:runtime.readback.clone(),verifier:runtime.verifier.clone(),serving,
  group_id:bootstrap.group.group_id,provider:bootstrap.provider,operator,accounts,companies,runtime})
}
pub async fn finish_business_fixture(identity:IdentityDeployment)->TestResult<NativeDeployment> {
 let IdentityDeployment{runtime,operator,accounts,serving,provider,group_id,companies:identity_companies,..}=identity;
 let issuer=FixtureIssuer::from_config(runtime.config.synthetic_issuer.issuer.clone(),runtime.config.synthetic_issuer.key_id.clone(),
  runtime.config.synthetic_issuer.public_pem.clone(),&runtime.config.synthetic_issuer.private_pem)?;
 let mut companies=Vec::new();
 for company in identity_companies {
  let IdentityCompany{org,submitter,reviewer,same_human_other_account,worker}=company;
  let workforce=enroll_workforce(&runtime.runtime,&submitter,&operator,accounts.submitter.account_id,accounts.reviewer.account_id,&provider).await?;
  let unprepared=enroll_workforce(&runtime.runtime,&submitter,&operator,accounts.transfer_recipient.account_id,accounts.reviewer.account_id,&provider).await?;
  let credential_scope=payroll::create_scope_basis(&runtime.runtime,&submitter,payroll::CreateScopeBasis {
   command_id:Uuid::new_v4(),employee_ids:vec![workforce.employee_id,unprepared.employee_id],reason:"Exact two-subject professional scope".into(),
  }).await?.reference;
  let channel=ArtifactChannel{client:&runtime.client,origin:&runtime.origin,token:&accounts.submitter.account_access_token};
  let sources=produce_source_foundations(&runtime.runtime,&submitter,&reviewer,&worker,&operator,&workforce,&issuer,
   &provider,&credential_scope,&channel,&runtime.config.compiled_bundle_manifest).await?;
  let period=payroll::InclusivePeriod{start:time::macros::date!(2026-06-01),end:time::macros::date!(2026-06-30)};
  let attendance=produce_attendance(&runtime.runtime,&submitter,&worker,&channel,workforce.employee_id,workforce.employment.employment_id,
   &sources.calendar.admission_ref,&workforce.scope,period.clone(),false).await?;
  let payroll_sources=produce_payroll_sources(&runtime.runtime,&submitter,&reviewer,&workforce,&sources,&attendance.coverage).await?;
  // Explicit second ordinary subject has NO wage/coverage/eligibility facts.
  // Its separately owned scope yields a known missing-source blocked population.

  let create_selection=payroll::read_run_collection_selection(&runtime.runtime,&submitter).await?;
  let source_refs=payroll::read_complete_native_source_refs(&runtime.runtime,&submitter,&workforce.scope,&period).await?;
  assert!(!source_refs.is_empty());
  let evidence=console_docs_adapter_postgres::action30::read_exact_evidence(&runtime.runtime,&submitter,&sources.evidence).await?;
  let choices=NativeUserChoices{create_selection,create:payroll::RunCreate{correlation_id:Uuid::new_v4(),period:period.clone(),
   pay_date:time::macros::date!(2026-06-27),profile:sources.profile.clone(),scope:workforce.scope.clone()},
   source_refs,preparation_evidence:vec![evidence.clone()],review_evidence:vec![evidence],
   review_decision:console_governance_adapter_postgres::action30::ReviewDecision::Approve,review_reason:"Synthetic accepted evidence".into()};
  let blocked_profile=payroll::create_supported_profile(&runtime.runtime,&submitter,payroll::CreateSupportedProfile {
   command_id:Uuid::new_v4(),rule_bundle:sources.rule_bundle.clone(),qualification:sources.bundle.admission_ref.clone(),
   scope_basis:unprepared.scope.clone(),period:period.clone(),pay_date:time::macros::date!(2026-06-27),currency:"KRW".into(),monthly_standard_hours:209,
  }).await?;
  let mut blocked_choices=choices.clone();blocked_choices.create.profile=blocked_profile.reference;blocked_choices.create.scope=unprepared.scope.clone();blocked_choices.source_refs.clear();
  let wage_selection=payroll::read_wage_selection(&runtime.runtime,&submitter,&payroll_sources.wage).await?;
  let wage_correction=payroll::WageCorrect{base_wage_id:payroll_sources.wage.base_wage_id,amount_won:3_100_000,
   wage_kind:payroll::WageKind::Monthly,monthly_standard_hours:209,source_note:"Synthetic concurrent wage correction".into(),evidence_refs:vec![sources.evidence.clone()]};
  let facts=FixtureFacts{second_supported:None,employee_id:workforce.employee_id,employment:workforce.employment.clone(),scope:workforce.scope.clone(),profile:sources.profile.clone(),
   calendar_rule:sources.calendar.admission_ref.clone(),calendar:attendance.calendar.clone(),assignment:attendance.assignment.clone(),
   coverage:attendance.coverage.clone(),close_basis:attendance.close_basis.clone(),workforce,unprepared_workforce:unprepared,sources,payroll_sources,attendance,
   choices,blocked_choices,wage_selection,wage_correction};
  companies.push(Fixture{pool:runtime.runtime.clone(),readback:runtime.readback.clone(),org,submitter,reviewer,same_human_other_account,worker,facts,
   runtime:runtime.clone(),account_token:accounts.submitter.account_access_token.clone()});
 }
 Ok(NativeDeployment{pool:runtime.runtime.clone(),readback:runtime.readback.clone(),verifier:runtime.verifier.clone(),serving,
  group_id,operator,accounts,companies,runtime})
}
impl Fixture {
    pub async fn snapshot(&self,run:Uuid)->TestResult<serde_json::Value> {
        Ok(sqlx::query_scalar::<_,serde_json::Value>(include_str!("business-readback.sql")).bind(self.org).bind(run).fetch_one(&self.readback).await?)
    }
    pub async fn staged(&self,blocked:bool)->TestResult<(payroll::RunResult,NativeUserChoices)> {
        let mut choices=if blocked {self.facts.blocked_choices.clone()} else {self.facts.choices.clone()};
        choices.create.correlation_id=Uuid::new_v4();
        choices.create_selection=crate::binder_sequence::explicit_new_intent::<payroll::RunCreate>(&self.pool,&self.submitter,&choices.create_selection).await?;
        let attempt=bind_and_seal(&self.pool,&self.submitter,&choices.create_selection,&choices.create).await?;
        let run=completed_attempt(payroll::payroll_create_run(&self.pool,&self.submitter,&attempt).await?,&attempt)?;
        assert_eq!(run.state,payroll::RunState::Staged);assert!(!run.payable);
        Ok((run,choices))
    }
    pub async fn prepare_input(&self,run:Uuid,choices:&NativeUserChoices)->TestResult<payroll::NativePrepare> {
        let c=payroll::read_preparation_control(&self.pool,&self.submitter,run).await?;
        Ok(payroll::NativePrepare{expected:c.expected,expected_membership_revision:c.expected_membership_revision,scope_basis_ref:c.scope_basis_ref,profile_ref:c.profile_ref,source_refs:choices.source_refs.clone(),evidence_refs:choices.preparation_evidence.clone()})
    }
    pub async fn prepared(&self,blocked:bool)->TestResult<(payroll::RunResult,AttemptRef)> {
        let (run,choices)=self.staged(blocked).await?;
        let input=self.prepare_input(run.run_id,&choices).await?;
        let attempt=bind_and_seal(&self.pool,&self.submitter,&UntrustedActionTargetSelection::existing_run(run.run_id),&input).await?;
        let initial=payroll::payroll_prepare_inputs(&self.pool,&self.submitter,&attempt).await?;
        let result=complete_native(&self.pool,&self.submitter,&self.worker,&attempt,initial).await?;
        Ok((result,attempt))
    }
    pub async fn closed(&self)->TestResult<(payroll::RunResult,AttemptRef)> {
        let (run,_)=self.prepared(false).await?;
        let c=payroll::read_attendance_close_control(&self.pool,&self.submitter,run.run_id).await?;
        let input=payroll::CloseAttendance{expected:c.expected,close_basis:c.close_basis,attest:true};
        let attempt=bind_and_seal(&self.pool,&self.submitter,&UntrustedActionTargetSelection::existing_run(run.run_id),&input).await?;
        Ok((completed(payroll::payroll_close_attendance(&self.pool,&self.submitter,&attempt).await?)?,attempt))
    }
    pub async fn calculated(&self)->TestResult<(payroll::RunResult,AttemptRef)> {
        let (run,_)=self.closed().await?;
        let input=payroll::RunCalculate{expected:payroll::read_run_control(&self.pool,&self.submitter,run.run_id).await?};
        let attempt=bind_and_seal(&self.pool,&self.submitter,&UntrustedActionTargetSelection::existing_run(run.run_id),&input).await?;
        let initial=payroll::payroll_calculate_run(&self.pool,&self.submitter,&attempt).await?;
        Ok((complete_native(&self.pool,&self.submitter,&self.worker,&attempt,initial).await?,attempt))
    }
    pub async fn review_open(&self)->TestResult<payroll::ReviewResult> {
        let (run,_)=self.calculated().await?;
        let input=payroll::ReviewOpen{expected:payroll::read_run_control(&self.pool,&self.submitter,run.run_id).await?};
        let attempt=bind_and_seal(&self.pool,&self.submitter,&UntrustedActionTargetSelection::existing_run(run.run_id),&input).await?;
        Ok(completed(payroll::payroll_open_review_cycle(&self.pool,&self.submitter,&attempt).await?)?)
    }
}

pub struct AttendanceFixture {
 pub pool:PgPool,pub readback:PgPool,pub org:Uuid,
 pub submitter:AuthenticatedCompanyContext,pub reviewer:AuthenticatedCompanyContext,pub worker:AuthenticatedWorkerContext,
 pub workforce:WorkforceFacts,pub sources:SourceFoundations,pub attendance:AttendanceFacts,
 pub runtime:Arc<Runtime>,pub account_token:String,
}
pub async fn build_attendance_fixture(pool:PgPool,scenario:&str,blocked:bool)->TestResult<AttendanceFixture> {
 // Native setup already creates a separately scoped real subject with no wage,
 // coverage or eligibility. Populate THAT subject's attendance explicitly, while
 // never asserting its incomplete coverage qualifies for supported payroll.
 let mut d=build_native_deployment(pool,scenario,1).await?;
 let f=d.companies.pop().ok_or("missing actual Company")?;
 let workforce=f.facts.unprepared_workforce.clone();
 let channel=ArtifactChannel{client:&f.runtime.client,origin:&f.runtime.origin,token:&f.account_token};
 let attendance=produce_attendance(&f.pool,&f.submitter,&f.worker,&channel,workforce.employee_id,workforce.employment.employment_id,
  &f.facts.calendar_rule,&workforce.scope,payroll::InclusivePeriod{start:time::macros::date!(2026-06-01),end:time::macros::date!(2026-06-30)},blocked).await?;
 Ok(AttendanceFixture{pool:f.pool,readback:f.readback,org:f.org,submitter:f.submitter,reviewer:f.reviewer,worker:f.worker,
  workforce,sources:f.facts.sources,attendance,runtime:f.runtime,account_token:f.account_token})
}
