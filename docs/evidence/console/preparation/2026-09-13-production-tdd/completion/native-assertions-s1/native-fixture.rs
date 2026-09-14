//! Actual fixture composition over proposed fixed owners; not a fake owner.
//! Preexisting authority/source seed is untrusted input, checked by real resolvers
//! and owner binders. Account/identity/issuer/docs bootstrap source is separately
//! enumerated SOURCE_NOT_AUTHORED; merely loading this file does not satisfy it.
use console_ontology_application::action30::*;
use console_payroll_adapter_postgres::action30 as payroll;
use console_platform_request_context::{account::{resolve_company_context,AuthenticatedCompanyContext},worker::{resolve_worker_context,AuthenticatedWorkerContext}};
use console_platform_auth::account_session::configured_account_verifier;
use console_platform_request_context::serving::admit_current_process;
use sqlx::{PgPool,postgres::PgPoolOptions};
use uuid::Uuid;
use crate::{binder_sequence::{bind_and_seal,completed},native_owner_producer::{NativeUserChoices,complete_native}};
pub type TestResult<T=()> = Result<T,Box<dyn std::error::Error+Send+Sync>>;
#[derive(serde::Deserialize)]
pub struct Seed {
    pub company: UntrustedCompanySelection,
    pub choices: NativeUserChoices,
    pub blocked_choices: NativeUserChoices,
    pub wage_selection: UntrustedActionTargetSelection,
    pub wage_correction: payroll::WageCorrect,
}
pub struct Fixture {
    pub pool: PgPool, pub readback: PgPool, pub org: Uuid,
    pub submitter: AuthenticatedCompanyContext,
    pub reviewer: AuthenticatedCompanyContext,
    pub same_human_other_account: AuthenticatedCompanyContext,
    pub worker: AuthenticatedWorkerContext,
    pub seed: Seed,
}
#[derive(serde::Deserialize)]
pub struct SeedEntry { company_id:Uuid, seed_path:std::path::PathBuf }
pub async fn fixture(scenario:&str)->TestResult<Fixture> {
    // Missing configuration is a visible fixture prerequisite error, never skip/PASS.
    let entries:std::collections::BTreeMap<String,SeedEntry>=serde_json::from_slice(&std::fs::read(std::env::var("CONSOLE_NATIVE_OWNER_SEED_INDEX")?)?)?;
    let companies:std::collections::BTreeSet<_>=entries.values().map(|e|e.company_id).collect();
    if companies.len()!=entries.len(){return Err("each scenario requires a distinct actual Company/population".into());}
    let entry=entries.get(scenario).ok_or("missing isolated scenario seed")?;
    let seed:Seed=serde_json::from_slice(&std::fs::read(&entry.seed_path)?)?;
    let pool=PgPoolOptions::new().max_connections(8).connect(&std::env::var("DATABASE_URL")?).await?;
    let readback=PgPoolOptions::new().max_connections(2).connect(&std::env::var("CONSOLE_NATIVE_READBACK_DATABASE_URL")?).await?;
    let verifier=configured_account_verifier().await?;
    let serving=admit_current_process(&pool).await?;
    let submitter=resolve_company_context(&pool,&verifier,&std::env::var("CONSOLE_NATIVE_SUBMITTER_TOKEN")?,&seed.company,&serving).await?;
    let reviewer=resolve_company_context(&pool,&verifier,&std::env::var("CONSOLE_NATIVE_REVIEWER_TOKEN")?,&seed.company,&serving).await?;
    let same_human_other_account=resolve_company_context(&pool,&verifier,&std::env::var("CONSOLE_NATIVE_SAME_HUMAN_TOKEN")?,&seed.company,&serving).await?;
    let worker=resolve_worker_context(&pool,&std::env::var("CONSOLE_NATIVE_WORKER_TOKEN")?,&seed.company,&serving).await?;
    let org=submitter.company_id(); // public read accessor to verified context, not constructor
    if org!=entry.company_id{return Err("actual authenticated Company differs from scenario seed identity".into());}
    Ok(Fixture{pool,readback,org,submitter,reviewer,same_human_other_account,worker,seed})
}
impl Fixture {
    pub async fn snapshot(&self,run:Uuid)->TestResult<serde_json::Value> {
        Ok(sqlx::query_scalar::<_,serde_json::Value>(include_str!("business-readback.sql")).bind(self.org).bind(run).fetch_one(&self.readback).await?)
    }
    pub async fn staged(&self,blocked:bool)->TestResult<(payroll::RunResult,NativeUserChoices)> {
        let mut choices=if blocked {self.seed.blocked_choices.clone()} else {self.seed.choices.clone()};
        choices.create.correlation_id=Uuid::new_v4();
        let attempt=bind_and_seal(&self.pool,&self.submitter,&choices.create_selection,&choices.create).await?;
        let run=completed(payroll::payroll_create_run(&self.pool,&self.submitter,&attempt).await?)?;
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
