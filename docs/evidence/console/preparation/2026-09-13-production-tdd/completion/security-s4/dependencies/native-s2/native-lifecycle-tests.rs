//! Full native lifecycle/current-vs-history assertions for the fixed payroll owners.
use console_ontology_application::action30::*;
use console_payroll_adapter_postgres::action30 as payroll;
use crate::{native_fixture::{fixture,TestResult},binder_sequence::{bind_and_seal,completed,explicit_new_intent},
 native_owner_producer::{complete_native,produce_native_and_review},commit_ack_relay::CommitAckRelay};
use sqlx::PgPool;
use uuid::Uuid;
fn rejected<R>(r:&OwnerExecution<R>){assert!(matches!(r,OwnerExecution::Observation(ResultObservation::DefinitiveNoEffect{..})));}

#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn contract_wage_create_replays_exact_subject_fact_without_duplicate(pool:PgPool)->TestResult {
 let f=fixture(pool,"wage_create").await?;let facts=&f.facts.payroll_sources;
 let before=payroll::read_contract_wage_history(&f.pool,&f.submitter,f.facts.employee_id).await?;
 let replay=completed(payroll::payroll_create_contract_wage(&f.pool,&f.submitter,&facts.wage_attempt).await?)?;
 assert_eq!(replay.fact,facts.wage);assert_eq!(replay.fact.org_id,f.org);
 assert_eq!(before,payroll::read_contract_wage_history(&f.pool,&f.submitter,f.facts.employee_id).await?);
 let mut stale=facts.wage_input.clone();stale.subject_source_generation=stale.subject_source_generation.checked_add(1).unwrap();
 let current=payroll::read_contract_wage_creation_control(&f.pool,&f.submitter,f.facts.employee_id,f.facts.employment.employment_id).await?;
 let target=explicit_new_intent::<payroll::WageCreate>(&f.pool,&f.submitter,&current.selection).await?;
 let bad=bind_and_seal(&f.pool,&f.submitter,&target,&stale).await?;
 rejected(&payroll::payroll_create_contract_wage(&f.pool,&f.submitter,&bad).await?);
 assert_eq!(before,payroll::read_contract_wage_history(&f.pool,&f.submitter,f.facts.employee_id).await?);Ok(())
}
#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn contract_wage_correction_recovers_commit_and_preserves_original(pool:PgPool)->TestResult {
 let f=fixture(pool,"wage_correct").await?;
 let original=payroll::read_contract_wage_fact(&f.pool,&f.submitter,&f.facts.payroll_sources.wage).await?;
 let a=bind_and_seal(&f.pool,&f.submitter,&f.facts.wage_selection,&f.facts.wage_correction).await?;
 let relay=CommitAckRelay::start(&f.pool).await?;
 assert!(payroll::payroll_correct_contract_wage(&relay.pool,&f.submitter,&a).await.is_err());relay.assert_cut()?;relay.stop().await?;
 let actual=completed(payroll::payroll_correct_contract_wage(&f.pool,&f.submitter,&a).await?)?;
 assert_eq!(actual.fact.base_wage_id,f.facts.payroll_sources.wage.base_wage_id);
 assert!(actual.fact.correction_revision>f.facts.payroll_sources.wage.correction_revision);
 assert_eq!(original,payroll::read_contract_wage_fact(&f.pool,&f.submitter,&f.facts.payroll_sources.wage).await?);
 let after=payroll::read_contract_wage_history(&f.pool,&f.submitter,f.facts.employee_id).await?;
 assert_eq!(completed(payroll::payroll_correct_contract_wage(&f.pool,&f.submitter,&a).await?)?.receipt,actual.receipt);
 assert_eq!(after,payroll::read_contract_wage_history(&f.pool,&f.submitter,f.facts.employee_id).await?);Ok(())
}
#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn native_create_is_header_only_and_original_command_replays(pool:PgPool)->TestResult {
 let f=fixture(pool,"native_create").await?;
 let input=f.facts.choices.create.clone();let selection=&f.facts.choices.create_selection;
 let a=bind_and_seal(&f.pool,&f.submitter,selection,&input).await?;
 let relay=CommitAckRelay::start(&f.pool).await?;
 assert!(payroll::payroll_create_run(&relay.pool,&f.submitter,&a).await.is_err());relay.assert_cut()?;relay.stop().await?;
 let actual=completed(payroll::payroll_create_run(&f.pool,&f.submitter,&a).await?)?;
 let control=payroll::read_run_control(&f.pool,&f.submitter,actual.run_id).await?;
 assert_eq!(control.run_revision,1);assert!(control.input_revision.is_none());assert!(control.calculation_revision.is_none());
 assert!(control.close_basis.is_none());assert!(control.review_cycle_id.is_none());
 let snapshot=f.snapshot(actual.run_id).await?;
 for field in ["lines","inputs","units","batches","outcomes","money"] {assert!(snapshot[field].as_array().unwrap().is_empty(),"header-only create {field}");}
 assert_eq!(snapshot["run"]["native_pay_date"],serde_json::to_value(&input.pay_date)?);
 assert_eq!(completed(payroll::payroll_create_run(&f.pool,&f.submitter,&a).await?)?.receipt,actual.receipt);
 assert_eq!(snapshot,f.snapshot(actual.run_id).await?);Ok(())
}
#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn native_prepare_recovers_lost_building_ack_on_original_subject(pool:PgPool)->TestResult {
 let f=fixture(pool,"native_building_ack").await?;let (run,choices)=f.staged(false).await?;
 let input=f.prepare_input(run.run_id,&choices).await?;
 let a=bind_and_seal(&f.pool,&f.submitter,&UntrustedActionTargetSelection::existing_run(run.run_id),&input).await?;
 let relay=CommitAckRelay::start(&f.pool).await?;
 assert!(payroll::payroll_prepare_inputs(&relay.pool,&f.submitter,&a).await.is_err());relay.assert_cut()?;relay.stop().await?;
 // Native initial transaction commits BUILDING, not a terminal monetary effect.
 let resumed=payroll::payroll_prepare_inputs(&f.pool,&f.submitter,&a).await?;
 assert!(matches!(&resumed,OwnerExecution::Observation(ResultObservation::Pending{phase:PreparationPhase::Building,..})));
 let terminal=complete_native(&f.pool,&f.submitter,&f.worker,&a,resumed).await?;
 let before=f.snapshot(run.run_id).await?;
 assert_eq!(before["inputs"].as_array().unwrap().len(),1);assert!(before["money"].as_array().unwrap().is_empty());
 assert_eq!(completed(payroll::payroll_prepare_inputs(&f.pool,&f.submitter,&a).await?)?.receipt,terminal.receipt);
 assert_eq!(before,f.snapshot(run.run_id).await?);Ok(())
}
#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn native_reopen_preserves_history_and_current_projection_excludes_old_money(pool:PgPool)->TestResult {
 let f=fixture(pool,"native_reopen").await?;let (calculated,calculation_attempt)=f.calculated().await?;
 let old=payroll::read_run_control(&f.pool,&f.submitter,calculated.run_id).await?;
 assert!(old.input_revision.is_some());assert!(old.calculation_revision.is_some());assert!(old.close_basis.is_some());assert!(old.review_cycle_id.is_none());
 let before=f.snapshot(calculated.run_id).await?;
 let input=payroll::RunReopen{expected:old.clone(),reason:"Explicit current-source refresh".into()};
 let a=bind_and_seal(&f.pool,&f.submitter,&UntrustedActionTargetSelection::existing_run(calculated.run_id),&input).await?;
 let reopened=completed(payroll::payroll_reopen_inputs(&f.pool,&f.submitter,&a).await?)?;
 assert_eq!(reopened.state,payroll::RunState::Staged);
 let now=payroll::read_run_control(&f.pool,&f.submitter,calculated.run_id).await?;
 assert!(now.run_revision>old.run_revision);assert!(now.calculation_revision.is_none());assert!(now.close_basis.is_none());assert!(now.review_cycle_id.is_none());
 let after=f.snapshot(calculated.run_id).await?;
 for field in ["inputs","units","batches","outcomes","money"] {assert_eq!(before[field],after[field],"immutable historical {field}");}
 assert!(payroll::read_current_native_projection(&f.pool,&f.submitter,calculated.run_id).await?.results.is_empty());
 assert_eq!(completed(payroll::payroll_calculate_run(&f.pool,&f.submitter,&calculation_attempt).await?)?.receipt,calculated.receipt);
 assert!(payroll::read_current_native_projection(&f.pool,&f.submitter,calculated.run_id).await?.results.is_empty());
 assert_eq!(completed(payroll::payroll_reopen_inputs(&f.pool,&f.submitter,&a).await?)?.receipt,reopened.receipt);
 // Fresh preparation uses current controls, then current projection joins the
 // new membership only. Old input rows stay byte-identical and addressable.
 let current=f.prepare_input(calculated.run_id,&f.facts.choices).await?;
 let target=explicit_new_intent::<payroll::NativePrepare>(&f.pool,&f.submitter,&UntrustedActionTargetSelection::existing_run(calculated.run_id)).await?;
 let fresh=bind_and_seal(&f.pool,&f.submitter,&target,&current).await?;
 let prepared=complete_native(&f.pool,&f.submitter,&f.worker,&fresh,payroll::payroll_prepare_inputs(&f.pool,&f.submitter,&fresh).await?).await?;
 assert!(prepared.input.as_ref().unwrap().input_revision>old.input_revision.unwrap());
 let projection=payroll::read_current_native_projection(&f.pool,&f.submitter,calculated.run_id).await?;
 assert!(projection.results.is_empty());assert!(!projection.units.is_empty());
 assert!(projection.units.iter().all(|u|u.input_revision==prepared.input.as_ref().unwrap().input_revision));Ok(())
}
#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn native_close_replay_does_not_unlock_or_replace_operational_basis(pool:PgPool)->TestResult {
 let f=fixture(pool,"native_close_replay").await?;let (closed,a)=f.closed().await?;
 let before=f.snapshot(closed.run_id).await?;
 let control=payroll::read_run_control(&f.pool,&f.submitter,closed.run_id).await?;assert!(control.close_basis.is_some());
 assert_eq!(completed(payroll::payroll_close_attendance(&f.pool,&f.submitter,&a).await?)?.receipt,closed.receipt);
 assert_eq!(before,f.snapshot(closed.run_id).await?);
 let lock=console_attendance_adapter_postgres::action30::read_current_operational_close(&f.pool,&f.submitter,&f.facts.scope,&f.facts.choices.create.period).await?;
 assert_eq!(lock.reference,f.facts.attendance.close_input.as_ref().unwrap().operational_close_ref);Ok(())
}
#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn native_cancel_review_is_explicit_and_generic_reopen_refuses(pool:PgPool)->TestResult {
 let f=fixture(pool,"native_cancel_review").await?;let cycle=f.review_open().await?;
 let run=cycle.cycle.run_id;
 let expected=payroll::read_run_control(&f.pool,&f.submitter,run).await?;assert_eq!(expected.review_cycle_id,Some(cycle.cycle.cycle_id));
 let target=UntrustedActionTargetSelection::existing_run(run);
 let bad=bind_and_seal(&f.pool,&f.submitter,&target,&payroll::RunReopen{expected:expected.clone(),reason:"Must not bypass REVIEWING protocol".into()}).await?;
 let before=payroll::read_review_cycle_history(&f.pool,&f.submitter,run).await?;
 rejected(&payroll::payroll_reopen_inputs(&f.pool,&f.submitter,&bad).await?);
 assert_eq!(before,payroll::read_review_cycle_history(&f.pool,&f.submitter,run).await?);
 let input=payroll::ReviewCancel{cycle:cycle.cycle.clone(),reason:"Explicit cancellation retains proposals".into()};
 let a=bind_and_seal(&f.pool,&f.submitter,&target,&input).await?;
 let actual=completed(payroll::payroll_cancel_review_cycle(&f.pool,&f.submitter,&a).await?)?;
 assert_eq!(actual.state,payroll::RunState::Staged);
 let after=payroll::read_review_cycle_history(&f.pool,&f.submitter,run).await?;
 assert_eq!(after.cycles,before.cycles);assert_eq!(after.bases,before.bases);assert_eq!(after.decisions,before.decisions);
 assert!(payroll::read_run_control(&f.pool,&f.submitter,run).await?.review_cycle_id.is_none());
 assert_eq!(completed(payroll::payroll_cancel_review_cycle(&f.pool,&f.submitter,&a).await?)?.receipt,actual.receipt);Ok(())
}
#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn approved_successor_is_unique_staged_and_preserves_old_decisions(pool:PgPool)->TestResult {
 let f=fixture(pool,"approved_successor").await?;
 let approved=produce_native_and_review(&f.pool,&f.submitter,&f.reviewer,&f.worker,f.facts.choices.clone()).await?;
 let run=approved.created.run_id;let target=UntrustedActionTargetSelection::existing_run(run);
 let expected=payroll::read_run_control(&f.pool,&f.submitter,run).await?;
 let old=payroll::read_review_cycle_history(&f.pool,&f.submitter,run).await?;
 let input=payroll::ApprovedSupersede{expected,reason:"Explicit nonpayable correction successor".into(),
  evidence:f.facts.choices.review_evidence.clone(),affected_basis_refs:payroll::read_complete_review_bases(&f.pool,&f.submitter,&approved.review.cycle).await?};
 let one=bind_and_seal(&f.pool,&f.submitter,&target,&input).await?;
 let target2=explicit_new_intent::<payroll::ApprovedSupersede>(&f.pool,&f.submitter,&target).await?;
 let two=bind_and_seal(&f.pool,&f.submitter,&target2,&input).await?;
 let (a,b)=tokio::join!(payroll::payroll_supersede_approved_draft(&f.pool,&f.submitter,&one),payroll::payroll_supersede_approved_draft(&f.pool,&f.submitter,&two));
 let mut winners=Vec::new();for (result,attempt) in [(a?,one),(b?,two)] {match result{OwnerExecution::Completed(r)=>winners.push((r,attempt)),other=>rejected(&other)}}
 assert_eq!(winners.len(),1);let (success,a)=winners.pop().unwrap();
 assert_eq!(success.predecessor_run_id,run);assert_ne!(success.successor_run_id,run);
 let current=payroll::read_run_control(&f.pool,&f.submitter,success.successor_run_id).await?;
 assert_eq!(current.run_revision,1);assert!(current.input_revision.is_none());assert!(current.calculation_revision.is_none());assert!(current.close_basis.is_none());assert!(current.review_cycle_id.is_none());
 let after=payroll::read_review_cycle_history(&f.pool,&f.submitter,run).await?;
 assert_eq!(old.bases,after.bases);assert_eq!(old.decisions,after.decisions);assert_eq!(old.completions,after.completions);
 assert_eq!(completed(payroll::payroll_supersede_approved_draft(&f.pool,&f.submitter,&a).await?)?.edge_id,success.edge_id);
 let edges=payroll::read_correction_edges(&f.pool,&f.submitter,run).await?;assert_eq!(edges.len(),1);Ok(())
}

#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn native_review_revision_preserves_only_exact_unaffected_approval(pool:PgPool)->TestResult {
 use console_governance_adapter_postgres::action30 as governance;
 let f=fixture(pool,"native_review_revise").await?;let opened=f.review_open().await?;
 let corrections=crate::review_correction_producer::correct_review_sources(&f).await?;
 let changed=payroll::read_correction_affected_review_bases(&f.pool,&f.submitter,&opened.cycle,&corrections).await?;
 assert!(!changed.is_empty());
 let all=payroll::read_complete_review_bases(&f.pool,&f.submitter,&opened.cycle).await?;
 let unchanged=all.iter().find(|b|!changed.iter().any(|a|a.basis_id==b.basis_id)).ok_or("fixture requires real unaffected Common obligation")?.clone();
 let target=UntrustedActionTargetSelection::existing_review_basis(unchanged.basis_id);
 let a=bind_and_seal(&f.pool,&f.submitter,&target,&governance::ReviewSubmit{basis:unchanged.clone(),attest:true,submitted_evidence:f.facts.choices.review_evidence.clone()}).await?;
 let submitted=completed(governance::review_submit(&f.pool,&f.submitter,&a).await?)?;
 let a=bind_and_seal(&f.pool,&f.reviewer,&target,&governance::ReviewDecide{request:submitted.request.unwrap(),basis:unchanged.clone(),decision:governance::ReviewDecision::Approve,reason:"Only this unchanged Common obligation".into()}).await?;
 let accepted=completed(governance::review_decide(&f.pool,&f.reviewer,&a).await?)?;
 assert!(accepted.completion.is_none());
 let old=payroll::read_review_cycle_history(&f.pool,&f.submitter,opened.cycle.run_id).await?;
 let input=payroll::ReviewRevise{affected_bases:changed.clone(),committed_corrections:corrections,reason:"Actual corrected source refs require selective revision".into()};
 let target=UntrustedActionTargetSelection::existing_run(opened.cycle.run_id);
 let attempt=bind_and_seal(&f.pool,&f.submitter,&target,&input).await?;
 let revised=crate::review_correction_producer::complete_review_revision(&f,&attempt,payroll::payroll_revise_review_cycle(&f.pool,&f.submitter,&attempt).await?).await?;
 assert_ne!(revised.cycle.cycle_id,opened.cycle.cycle_id);
 let now=payroll::read_complete_review_bases(&f.pool,&f.submitter,&revised.cycle).await?;
 assert!(now.iter().any(|b|b==&unchanged));
 assert!(changed.iter().all(|old|now.iter().all(|new|new.basis_id!=old.basis_id)));
 let after=payroll::read_review_cycle_history(&f.pool,&f.submitter,opened.cycle.run_id).await?;
 assert!(old.decisions.iter().all(|old|after.decisions.contains(old)));
 let reused=payroll::read_current_cycle_accepted_decisions(&f.pool,&f.submitter,&revised.cycle).await?;
 assert!(reused.iter().any(|d|Some(d.decision_id)==accepted.decision_id));
 assert_eq!(completed(payroll::payroll_revise_review_cycle(&f.pool,&f.submitter,&attempt).await?)?.receipt,revised.receipt);
 let stale_target=explicit_new_intent::<payroll::ReviewRevise>(&f.pool,&f.submitter,&target).await?;
 let stale=bind_and_seal(&f.pool,&f.submitter,&stale_target,&input).await?;
 rejected(&payroll::payroll_revise_review_cycle(&f.pool,&f.submitter,&stale).await?);Ok(())
}

#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn native_review_open_rejects_each_wrong_nullable_expectation(pool:PgPool)->TestResult {
 let f=fixture(pool,"native_review_nullable").await?;let (run,_)=f.calculated().await?;
 let good=payroll::read_run_control(&f.pool,&f.submitter,run.run_id).await?;
 assert!(good.input_revision.is_some()&&good.calculation_revision.is_some()&&good.close_basis.is_some()&&good.review_cycle_id.is_none());
 let before=f.snapshot(run.run_id).await?;
 for field in ["input_revision","calculation_revision","close_basis","review_cycle_id"] {
  let mut wrong=good.clone();match field{"input_revision"=>wrong.input_revision=None,"calculation_revision"=>wrong.calculation_revision=None,
   "close_basis"=>wrong.close_basis=None,"review_cycle_id"=>wrong.review_cycle_id=Some(Uuid::new_v4()),_=>unreachable!()}
  let target=explicit_new_intent::<payroll::ReviewOpen>(&f.pool,&f.submitter,&UntrustedActionTargetSelection::existing_run(run.run_id)).await?;
  crate::assert_registered_refusal!(&f.pool,&f.submitter,&target,&payroll::ReviewOpen{expected:wrong},payroll::payroll_open_review_cycle);
  assert_eq!(before,f.snapshot(run.run_id).await?,"{field}");
 }
 let target=explicit_new_intent::<payroll::ReviewOpen>(&f.pool,&f.submitter,&UntrustedActionTargetSelection::existing_run(run.run_id)).await?;
 let a=bind_and_seal(&f.pool,&f.submitter,&target,&payroll::ReviewOpen{expected:good}).await?;
 let opened=completed(payroll::payroll_open_review_cycle(&f.pool,&f.submitter,&a).await?)?;
 assert_eq!(payroll::read_run_control(&f.pool,&f.submitter,run.run_id).await?.review_cycle_id,Some(opened.cycle.cycle_id));
 assert_eq!(completed(payroll::payroll_open_review_cycle(&f.pool,&f.submitter,&a).await?)?.receipt,opened.receipt);Ok(())
}

#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn review_withdraw_keeps_immutable_decision_and_replays_control_once(pool:PgPool)->TestResult {
 use console_governance_adapter_postgres::action30 as governance;
 let f=fixture(pool,"review_withdraw").await?;let cycle=f.review_open().await?;
 let bases=payroll::read_complete_review_bases(&f.pool,&f.submitter,&cycle.cycle).await?;assert!(bases.len()>=2);
 let basis=bases[0].clone();let target=UntrustedActionTargetSelection::existing_review_basis(basis.basis_id);
 let a=bind_and_seal(&f.pool,&f.submitter,&target,&governance::ReviewSubmit{basis:basis.clone(),attest:true,submitted_evidence:f.facts.choices.review_evidence.clone()}).await?;
 let proposed=completed(governance::review_submit(&f.pool,&f.submitter,&a).await?)?;let request=proposed.request.unwrap();
 let a=bind_and_seal(&f.pool,&f.reviewer,&target,&governance::ReviewDecide{request:request.clone(),basis:basis.clone(),decision:governance::ReviewDecision::Approve,reason:"Synthetic one-obligation approval".into()}).await?;
 let accepted=completed(governance::review_decide(&f.pool,&f.reviewer,&a).await?)?;assert!(accepted.completion.is_none());
 let input=governance::ReviewWithdraw{request:request.clone(),basis,decision_id:accepted.decision_id.unwrap(),reason:"Actual reviewer withdraws this approval".into(),evidence:f.facts.choices.review_evidence.clone()};
 let original=governance::read_review_decision_history(&f.pool,&f.reviewer,&request).await?;
 crate::assert_registered_refusal!(&f.pool,&f.submitter,&target,&input,governance::review_withdraw);
 let a=bind_and_seal(&f.pool,&f.reviewer,&target,&input).await?;
 let relay=CommitAckRelay::start(&f.pool).await?;
 assert!(governance::review_withdraw(&relay.pool,&f.reviewer,&a).await.is_err());relay.assert_cut()?;relay.stop().await?;
 let withdrawn=completed(governance::review_withdraw(&f.pool,&f.reviewer,&a).await?)?;
 assert_eq!(original,governance::read_review_decision_history(&f.pool,&f.reviewer,&request).await?);
 let controls=governance::read_request_control_history(&f.pool,&f.reviewer,&request).await?;
 assert!(payroll::read_current_cycle_accepted_decisions(&f.pool,&f.submitter,&cycle.cycle).await?.iter().all(|d|Some(d.decision_id)!=accepted.decision_id));
 assert_eq!(completed(governance::review_withdraw(&f.pool,&f.reviewer,&a).await?)?.receipt,withdrawn.receipt);
 assert_eq!(controls,governance::read_request_control_history(&f.pool,&f.reviewer,&request).await?);Ok(())
}
