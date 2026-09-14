//! Authored runtime assertions against exact proposed owners. Not compiled/executed.
//! Missing imports/fixtures are prerequisite failures, never product RED evidence.
#[path="binder-sequence.rs"] mod binder_sequence;
#[path="native-owner-producer.rs"] mod native_owner_producer;
#[path="native-fixture.rs"] mod native_fixture;
use console_ontology_application::action30::*;
use console_payroll_adapter_postgres::action30 as payroll;
use console_governance_adapter_postgres::action30 as governance;
use console_ontology_adapter_postgres::action30 as ontology;
use binder_sequence::{bind_and_seal,completed};
use native_owner_producer::{complete_native,produce_native_and_review};
use native_fixture::{fixture,TestResult};
use uuid::Uuid;

#[tokio::test]
async fn native_owner_chain_produces_complete_nonpayable_review()->TestResult {
    let f=fixture("native_owner_chain_produces_complete_nonpayable_review").await?;
    let mut choices=f.seed.choices.clone();choices.create.correlation_id=Uuid::new_v4();
    let actual=produce_native_and_review(&f.pool,&f.submitter,&f.reviewer,&f.worker,choices).await?;
    assert_eq!(actual.created.run_id,actual.calculated.run_id);
    assert_eq!(actual.prepared.state,payroll::RunState::Staged);
    assert_eq!(actual.closed.state,payroll::RunState::AttendanceClosed);
    assert_eq!(actual.calculated.state,payroll::RunState::Calculated);
    assert!(!actual.created.payable && !actual.closed.payable && !actual.calculated.payable);
    assert_eq!(actual.review.cycle.cycle_id,actual.completion.cycle_id);
    assert_eq!(actual.review.cycle.membership_digest,actual.completion.membership_digest);
    let durable=f.snapshot(actual.created.run_id).await?;
    let inputs=durable["units"].as_array().expect("actual unit rows");
    let outcomes=durable["outcomes"].as_array().expect("actual outcome rows");
    let money=durable["money"].as_array().expect("actual money rows");
    assert!(!inputs.is_empty());assert_eq!(inputs.len(),outcomes.len());
    assert_eq!(money.len(),outcomes.iter().filter(|x|x["state"]=="SUCCESS").count());
    assert!(money.iter().all(|x|x["payable"]==false));
    assert!(durable["lines"].as_array().unwrap().iter().all(|x|x["native_protocol"]=="NP1" && x["source_data_import_row_ids"].as_array().is_some_and(|a|a.is_empty())));
    for out in outcomes {
        let input=inputs.iter().find(|x|x["employee_id"]==out["employee_id"] && x["input_revision"]==out["input_revision"]).expect("exact input membership");
        assert_eq!(input["line_id"],out["line_id"]);
        assert_eq!(input["custody_digest"],out["input_unit_digest"]);
        if out["state"]=="SUCCESS" {
            let result=money.iter().find(|x|x["id"]==out["result_id"]).expect("actual immutable existing result");
            assert_eq!(result["native_employee_id"],out["employee_id"]);
            assert_eq!(result["native_input_revision"],out["input_revision"]);
            assert_eq!(result["line_id"],out["line_id"]);
        } else {assert!(out["result_id"].is_null());assert!(!out["blockers"].as_array().unwrap().is_empty());}
    }
    Ok(())
}

#[tokio::test]
async fn native_first_prepare_is_durable_pending_before_terminal_effects()->TestResult {
    let f=fixture("native_first_prepare_is_durable_pending_before_terminal_effects").await?;let (run,choices)=f.staged(false).await?;
    let input=f.prepare_input(run.run_id,&choices).await?;
    let attempt=bind_and_seal(&f.pool,&f.submitter,&UntrustedActionTargetSelection::existing_run(run.run_id),&input).await?;
    let initial=payroll::payroll_prepare_inputs(&f.pool,&f.submitter,&attempt).await?;
    let subject=match &initial {
        OwnerExecution::Observation(ResultObservation::Pending{subject,phase:PreparationPhase::Building,..})=>subject.clone(),
        other=>panic!("expected actual first BUILDING PENDING, got {other:?}"),
    };
    let observed=payroll::observe_native_command(&f.pool,&f.submitter,&subject).await?;
    assert!(matches!(observed,OwnerExecution::Observation(ResultObservation::Pending{..})));
    let before=f.snapshot(run.run_id).await?;
    assert!(before["run"]["current_input_revision"].is_null());
    assert!(before["money"].as_array().unwrap().is_empty());
    let terminal=complete_native(&f.pool,&f.submitter,&f.worker,&attempt,initial).await?;
    assert!(terminal.input.is_some());
    let after=f.snapshot(run.run_id).await?;
    assert!(!after["run"]["current_input_revision"].is_null());
    assert!(after["money"].as_array().unwrap().is_empty());
    Ok(())
}

#[tokio::test]
async fn native_prepare_replay_returns_same_receipt_without_second_input()->TestResult {
    let f=fixture("native_prepare_replay_returns_same_receipt_without_second_input").await?;let (first,attempt)=f.prepared(false).await?;
    let before=f.snapshot(first.run_id).await?;
    let replay=completed(payroll::payroll_prepare_inputs(&f.pool,&f.submitter,&attempt).await?)?;
    assert_receipt_for_attempt(&first.receipt,&attempt);assert_receipt_for_attempt(&replay.receipt,&attempt);
    assert_eq!(first.receipt,replay.receipt);assert_eq!(first.input,replay.input);
    assert_eq!(before,f.snapshot(first.run_id).await?);Ok(())
}

#[tokio::test]
async fn native_calculation_replay_returns_same_money_and_receipt()->TestResult {
    let f=fixture("native_calculation_replay_returns_same_money_and_receipt").await?;let (first,attempt)=f.calculated().await?;
    let before=f.snapshot(first.run_id).await?;
    let replay=completed(payroll::payroll_calculate_run(&f.pool,&f.submitter,&attempt).await?)?;
    assert_receipt_for_attempt(&first.receipt,&attempt);assert_receipt_for_attempt(&replay.receipt,&attempt);
    assert_eq!(first.receipt,replay.receipt);assert_eq!(first.batch,replay.batch);
    assert_eq!(before,f.snapshot(first.run_id).await?);Ok(())
}

#[tokio::test]
async fn native_same_draft_control_command_changed_input_refuses()->TestResult {
    let f=fixture("native_same_draft_control_command_changed_input_refuses").await?;let (run,choices)=f.staged(false).await?;
    let binding=ontology::read_registered_draft_binding::<payroll::NativePrepare>(&f.pool,&f.submitter,&UntrustedActionTargetSelection::existing_run(run.run_id)).await?;
    let command_id=Uuid::new_v4();
    let input=DraftStart{action:binding.action,target:binding.target,schema:binding.schema,custody:binding.custody,intent_slot:binding.intent_slot};
    let first=completed(ontology::draft_start(&f.pool,&f.submitter,UnadmittedControl{command_id,input:input.clone()}).await?)?;
    let replay=completed(ontology::draft_start(&f.pool,&f.submitter,UnadmittedControl{command_id,input:input.clone()}).await?)?;
    assert_eq!(first.draft_id,replay.draft_id);assert_eq!(first.receipt,replay.receipt);
    let mut changed=input;changed.intent_slot=format!("different-{}",Uuid::new_v4());
    let refusal=ontology::draft_start(&f.pool,&f.submitter,UnadmittedControl{command_id,input:changed}).await?;
    assert!(matches!(refusal,OwnerExecution::Observation(ResultObservation::DefinitiveNoEffect{..})));
    let continuation=ontology::read_draft_continuation(&f.pool,&f.submitter,first.draft_id).await?;
    assert_eq!(first.revision_id,continuation.revision_id);
    let _=choices;Ok(())
}

// Each mutation gets its own independently owner-produced run and original
// sealed command. No stored intent is edited; schema-valid mutation is present
// before ordinary real binding/sealing and must never be defaulted to current.
async fn refuse_changed_expectation(field:&str)->TestResult {
    let f=fixture(&format!("expected_{field}")).await?;let (run,_)=f.closed().await?;
    let good=payroll::read_run_control(&f.pool,&f.submitter,run.run_id).await?;
    let mut bad=good.clone();
    match field {
        "run_id"=>bad.run_id=Uuid::new_v4(),
        "run_revision"=>bad.run_revision=bad.run_revision.checked_add(1).unwrap(),
        "input_revision"=>bad.input_revision=Some(bad.input_revision.unwrap().checked_add(1).unwrap()),
        "input_revision_null"=>bad.input_revision=None,
        "calculation_revision"=>bad.calculation_revision=Some(1),
        "close_basis"=>bad.close_basis=None,
        "review_cycle_id"=>bad.review_cycle_id=Some(Uuid::new_v4()),
        _=>panic!("closed mutation set"),
    }
    assert_ne!(good,bad);
    let input=payroll::RunCalculate{expected:bad};
    let attempt=bind_and_seal(&f.pool,&f.submitter,&UntrustedActionTargetSelection::existing_run(run.run_id),&input).await?;
    let before=f.snapshot(run.run_id).await?;
    let refused=payroll::payroll_calculate_run(&f.pool,&f.submitter,&attempt).await?;
    assert!(matches!(refused,OwnerExecution::Observation(ResultObservation::DefinitiveNoEffect{..})),"{field} must refuse before BUILDING/effects");
    assert_eq!(before,f.snapshot(run.run_id).await?);Ok(())
}
macro_rules! expectation_test {($name:ident,$field:literal)=>{#[tokio::test] async fn $name()->TestResult{refuse_changed_expectation($field).await}};}
expectation_test!(native_expected_run_id_is_not_defaulted,"run_id");
expectation_test!(native_expected_run_revision_is_not_defaulted,"run_revision");
expectation_test!(native_expected_input_revision_is_not_defaulted,"input_revision");
expectation_test!(native_expected_input_revision_null_is_not_defaulted,"input_revision_null");
expectation_test!(native_expected_calculation_revision_is_not_defaulted,"calculation_revision");
expectation_test!(native_expected_close_basis_null_is_not_defaulted,"close_basis");
expectation_test!(native_expected_review_cycle_id_is_not_defaulted,"review_cycle_id");

#[tokio::test]
async fn native_owner_wage_change_after_building_prevents_old_activation()->TestResult {
    let f=fixture("native_owner_wage_change_after_building_prevents_old_activation").await?;let (run,choices)=f.staged(false).await?;
    let input=f.prepare_input(run.run_id,&choices).await?;
    let attempt=bind_and_seal(&f.pool,&f.submitter,&UntrustedActionTargetSelection::existing_run(run.run_id),&input).await?;
    let initial=payroll::payroll_prepare_inputs(&f.pool,&f.submitter,&attempt).await?;
    assert!(matches!(&initial,OwnerExecution::Observation(ResultObservation::Pending{phase:PreparationPhase::Building,..})));
    // Fixed real wage owner commits between BUILDING and activation; no trigger
    // disables or direct table edits. The seed correction concerns this population.
    let correction_attempt=bind_and_seal(&f.pool,&f.submitter,&f.seed.wage_selection,&f.seed.wage_correction).await?;
    let corrected=completed(payroll::payroll_correct_contract_wage(&f.pool,&f.submitter,&correction_attempt).await?)?;
    assert_receipt_for_attempt(&corrected.receipt,&correction_attempt);
    let progressed=complete_native(&f.pool,&f.submitter,&f.worker,&attempt,initial).await;
    assert!(matches!(progressed,Err(binder_sequence::ProducerStop::Observation(ResultObservation::DefinitiveNoEffect{..}))));
    let after=f.snapshot(run.run_id).await?;
    assert!(after["run"]["current_input_revision"].is_null());
    assert!(after["money"].as_array().unwrap().is_empty());Ok(())
}

#[tokio::test]
async fn native_blocked_population_cannot_close()->TestResult {
    let f=fixture("native_blocked_population_cannot_close").await?;let (run,_)=f.prepared(true).await?;
    let before=f.snapshot(run.run_id).await?;
    let units=before["units"].as_array().unwrap();assert!(!units.is_empty());
    assert!(units.iter().all(|u|u["resolution"]=="BLOCKED"),"fixture must be entirely blocked");
    // Close owner read may validly produce a typed blocker before sealing.
    let result=payroll::read_attendance_close_control(&f.pool,&f.submitter,run.run_id).await;
    assert!(matches!(result,Err(payroll::PayrollActionError::BlockedNativeInput{..})));
    assert_eq!(before,f.snapshot(run.run_id).await?);Ok(())
}

#[tokio::test]
async fn native_two_original_calculate_intents_have_one_current_winner()->TestResult {
    let f=fixture("native_two_original_calculate_intents_have_one_current_winner").await?;let (run,_)=f.closed().await?;
    let expected=payroll::read_run_control(&f.pool,&f.submitter,run.run_id).await?;
    let selection=UntrustedActionTargetSelection::existing_run(run.run_id);
    let one=bind_and_seal(&f.pool,&f.submitter,&selection,&payroll::RunCalculate{expected:expected.clone()}).await?;
    let two=bind_and_seal(&f.pool,&f.submitter,&selection,&payroll::RunCalculate{expected}).await?;
    let (a,b)=tokio::join!(payroll::payroll_calculate_run(&f.pool,&f.submitter,&one),payroll::payroll_calculate_run(&f.pool,&f.submitter,&two));
    // Progress both original subjects independently; conflicting activation must
    // have one terminal success and one definitive stale/no-effect observation.
    let aa=complete_native(&f.pool,&f.submitter,&f.worker,&one,a?).await;
    let bb=complete_native(&f.pool,&f.submitter,&f.worker,&two,b?).await;
    assert_eq!(usize::from(aa.is_ok())+usize::from(bb.is_ok()),1);
    let loser=if aa.is_err(){aa.err().unwrap()}else{bb.err().unwrap()};
    assert!(matches!(loser,binder_sequence::ProducerStop::Observation(ResultObservation::DefinitiveNoEffect{..})));
    let rows=f.snapshot(run.run_id).await?;
    assert_eq!(rows["batches"].as_array().unwrap().len(),1);
    assert_eq!(rows["money"].as_array().unwrap().len(),rows["outcomes"].as_array().unwrap().iter().filter(|u|u["state"]=="SUCCESS").count());Ok(())
}

#[tokio::test]
async fn review_partial_approvals_do_not_produce_completion()->TestResult {
    let f=fixture("review_partial_approvals_do_not_produce_completion").await?;let review=f.review_open().await?;
    let bases=payroll::read_complete_review_bases(&f.pool,&f.submitter,&review.cycle).await?;
    assert!(bases.len()>=2,"fixture prerequisite: at least two actual obligations");
    let basis=bases[0].clone();let target=UntrustedActionTargetSelection::existing_review_basis(basis.basis_id);
    let submit=governance::ReviewSubmit{basis:basis.clone(),attest:true,submitted_evidence:f.seed.choices.review_evidence.clone()};
    let attempt=bind_and_seal(&f.pool,&f.submitter,&target,&submit).await?;
    let submitted=completed(governance::review_submit(&f.pool,&f.submitter,&attempt).await?)?;
    let decision=governance::ReviewDecide{request:submitted.request.expect("actual request"),basis,decision:governance::ReviewDecision::Approve,reason:"Independent fixture review".into()};
    let attempt=bind_and_seal(&f.pool,&f.reviewer,&target,&decision).await?;
    let decided=completed(governance::review_decide(&f.pool,&f.reviewer,&attempt).await?)?;
    assert!(decided.decision_id.is_some());assert!(decided.completion.is_none());
    assert!(matches!(payroll::read_review_completion(&f.pool,&f.submitter,&review.cycle).await,Err(payroll::PayrollActionError::IncompleteReview{..})));
    Ok(())
}

#[tokio::test]
async fn review_second_account_of_same_human_cannot_self_approve()->TestResult {
    let f=fixture("review_second_account_of_same_human_cannot_self_approve").await?;let review=f.review_open().await?;
    let actor=governance::read_current_reviewer_identity(&f.pool,&f.submitter).await?;
    let alias=governance::read_current_reviewer_identity(&f.pool,&f.same_human_other_account).await?;
    assert_ne!(actor.account_id,alias.account_id);assert_eq!(actor.human_id,alias.human_id);
    let basis=payroll::read_complete_review_bases(&f.pool,&f.submitter,&review.cycle).await?.remove(0);
    let target=UntrustedActionTargetSelection::existing_review_basis(basis.basis_id);
    let submit=governance::ReviewSubmit{basis:basis.clone(),attest:true,submitted_evidence:f.seed.choices.review_evidence.clone()};
    let attempt=bind_and_seal(&f.pool,&f.submitter,&target,&submit).await?;
    let submitted=completed(governance::review_submit(&f.pool,&f.submitter,&attempt).await?)?;
    let request=submitted.request.expect("actual request");
    let before=governance::read_review_decision_history(&f.pool,&f.submitter,&request).await?;
    let decision=governance::ReviewDecide{request:request.clone(),basis,decision:governance::ReviewDecision::Approve,reason:"Same Human must be rejected".into()};
    let attempt=bind_and_seal(&f.pool,&f.same_human_other_account,&target,&decision).await?;
    let refused=governance::review_decide(&f.pool,&f.same_human_other_account,&attempt).await?;
    assert!(matches!(refused,OwnerExecution::Observation(ResultObservation::DefinitiveNoEffect{..})));
    assert_eq!(before,governance::read_review_decision_history(&f.pool,&f.submitter,&request).await?);
    let independent=bind_and_seal(&f.pool,&f.reviewer,&target,&decision).await?;
    let accepted=completed(governance::review_decide(&f.pool,&f.reviewer,&independent).await?)?;
    assert!(accepted.decision_id.is_some());Ok(())
}

fn assert_receipt_for_attempt(receipt:&ReceiptRef,attempt:&AttemptRef) {
    match receipt {
        ReceiptRef::Company { command, .. } => {assert_eq!(command.org_id,attempt.org_id);assert_eq!(command.command_id,attempt.command_id);},
        ReceiptRef::Group { .. } => panic!("Company native command returned Group receipt"),
    }
}
