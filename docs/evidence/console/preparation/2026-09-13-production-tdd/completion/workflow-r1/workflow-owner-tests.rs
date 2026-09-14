//! Fifteen authored registered-owner tests. Compile/runtime status is recorded
//! separately; missing owners/fixtures must fail, never skip or pass by parser.
#![cfg(feature="test-postgres")]
use console_ontology_application::action30::*;
use console_ontology_adapter_postgres::action30 as ontology;
use console_payroll_adapter_postgres::action30 as payroll;
use console_governance_adapter_postgres::action30 as governance;
use console_workflow_runtime_adapter_postgres::action30 as workflow;
use console_platform_group::action30 as group;
use sqlx::PgPool;
use serde_json::json;
use uuid::Uuid;
use crate::{native_fixture::TestResult,binder_sequence::{bind_and_seal,completed,explicit_new_intent},
    workflow_fixture::{fixture,typed,value,assert_company,assert_group,assert_refusal}};

#[sqlx::test]
async fn gate_request_binds_actual_sealed_calculation_without_effect(pool:PgPool)->TestResult {
    let s=fixture(pool,"gate_request",1).await?;let f=s.company();
    let (requested,a,draft,run)=s.gate_request().await?;
    assert_eq!(value(&requested)?["state"],"PENDING");assert!(requested.gate.is_none());
    let binding=governance::read_execution_gate_binding(&f.pool,&f.submitter,&requested.request).await?;
    let actual=ontology::read_sealed_draft_attempt(&f.pool,&f.submitter,draft).await?;
    assert_eq!(binding.command,actual.command);assert_eq!(binding.attempt,actual.attempt);
    assert_eq!(binding.fingerprint,actual.fingerprint);assert_ne!(actual.attempt.attempt_id,a.attempt_id);
    assert_eq!(actual.gate_request_id,Some(requested.request.request_id));
    let before=f.snapshot(run).await?;
    assert!(payroll::read_run_control(&f.pool,&f.submitter,run).await?.calculation_revision.is_none());
    let replay=completed(governance::governance_execution_gate_request(&f.pool,&f.submitter,&a).await?)?;
    assert_company(&replay.receipt,&a);assert_eq!(value(&replay)?,value(&requested)?);
    assert_eq!(f.snapshot(run).await?,before);
    let count:i64=sqlx::query_scalar("SELECT count(*) FROM gov_approval_consumptions c JOIN gov_approvals a ON a.org_id=c.org_id AND a.id=c.approval_id WHERE a.org_id=$1 AND a.request_ref=$2")
        .bind(f.org).bind(requested.request.request_id).fetch_one(&f.readback).await?;
    assert_eq!(count,0);Ok(())
}

#[sqlx::test]
async fn gate_decide_requires_independence_and_never_executes_business(pool:PgPool)->TestResult {
    let s=fixture(pool,"gate_decide",1).await?;let f=s.company();
    let (requested,_,_,run)=s.gate_request().await?;
    let input:governance::GateDecide=typed(json!({"request":requested.request,"decision":"PERMIT","reason":"Permit this exact current calculation intent"}))?;
    let target=UntrustedActionTargetSelection::existing_execution_gate(requested.request.request_id);
    let own=bind_and_seal(&f.pool,&f.submitter,&target,&input).await?;
    let before=f.snapshot(run).await?;
    let refused=governance::governance_execution_gate_decide(&f.pool,&f.submitter,&own).await?;
    assert_refusal(&refused);assert_eq!(f.snapshot(run).await?,before);
    let a=bind_and_seal(&f.pool,&f.reviewer,&target,&input).await?;
    let actual=completed(governance::governance_execution_gate_decide(&f.pool,&f.reviewer,&a).await?)?;
    assert_company(&actual.receipt,&a);assert_eq!(value(&actual)?["state"],"PERMITTED");assert!(actual.gate.is_some());
    assert_eq!(actual.request.request_id,requested.request.request_id);
    assert_eq!(f.snapshot(run).await?,before,"permit may not calculate or consume itself");
    let replay=completed(governance::governance_execution_gate_decide(&f.pool,&f.reviewer,&a).await?)?;
    assert_eq!(value(&replay)?,value(&actual)?);Ok(())
}

#[sqlx::test]
async fn publication_release_preserves_exact_source_and_is_nonpayable(pool:PgPool)->TestResult {
    let s=fixture(pool,"publication_release",1).await?;let f=s.company();
    let (actual,a,approved)=s.publication().await?;
    assert_eq!(value(&actual)?["purpose"],"NONPAYABLE_REVIEW");
    let p=&actual.publications[0];assert_eq!(p.org_id,f.org);
    let content=payroll::read_publication_record(&f.pool,&f.submitter,p).await?;
    assert_eq!(content.completion,approved.completion);
    let targets=payroll::read_complete_publication_targets(&f.pool,&f.submitter,approved.created.run_id,&approved.completion).await?;
    let exact=targets.iter().find(|t|t.subject==content.content.subject).ok_or("published subject absent from native membership")?;
    assert_eq!(content.result,exact.result);
    assert_eq!(content.projection,s.config.projection);
    let slot=payroll::read_publication_slot(&f.pool,&f.submitter,p).await?;
    assert_eq!(slot.current_publication,*p);assert_eq!(value(&slot)?["currentness"],"CURRENT");
    let before=f.snapshot(approved.created.run_id).await?;
    let replay=completed(payroll::payroll_review_release(&f.pool,&f.submitter,&a).await?)?;
    assert_company(&replay.receipt,&a);assert_eq!(value(&actual)?,value(&replay)?);
    assert_eq!(f.snapshot(approved.created.run_id).await?,before);
    let payable:bool=sqlx::query_scalar("SELECT bool_or(payable) FROM payroll_line_calculations WHERE org_id=$1 AND run_id=$2")
        .bind(f.org).bind(approved.created.run_id).fetch_one(&f.readback).await?;
    assert!(!payable);Ok(())
}

#[sqlx::test]
async fn publication_replace_preserves_history_and_rejects_stale_slot(pool:PgPool)->TestResult {
    let s=fixture(pool,"publication_replace",1).await?;let f=s.company();
    let (published,_,approved)=s.publication().await?;let old=published.publications[0].clone();
    let old_content=payroll::read_publication_record(&f.pool,&f.submitter,&old).await?;
    let input:payroll::PublicationReplace=typed(json!({"predecessor":old,"projection":s.config.replacement_projection,"reason":"Use the other registered field projection"}))?;
    let target=UntrustedActionTargetSelection::existing_publication(old.publication_id);
    let a=bind_and_seal(&f.pool,&f.submitter,&target,&input).await?;
    let actual=completed(payroll::payroll_review_replace_projection(&f.pool,&f.submitter,&a).await?)?;
    assert_company(&actual.receipt,&a);assert_eq!(actual.publications.len(),1);
    let new=&actual.publications[0];assert_ne!(new.publication_id,old.publication_id);
    assert_eq!(new.slot_id,old.slot_id);assert_eq!(new.slot_generation,old.slot_generation+1);
    let new_content=payroll::read_publication_record(&f.pool,&f.submitter,new).await?;
    assert_eq!(new_content.content.subject,old_content.content.subject);assert_eq!(new_content.result,old_content.result);
    assert_eq!(new_content.completion,old_content.completion);assert_eq!(new_content.projection,s.config.replacement_projection);
    assert_eq!(value(&payroll::read_publication_record(&f.pool,&f.submitter,&old).await?)?,value(&old_content)?);
    let before=f.snapshot(approved.created.run_id).await?;
    let new_intent=explicit_new_intent::<payroll::PublicationReplace>(&f.pool,&f.submitter,&target).await?;
    let stale=bind_and_seal(&f.pool,&f.submitter,&new_intent,&input).await?;
    assert_ne!(stale.command_id,a.command_id);
    assert_refusal(&payroll::payroll_review_replace_projection(&f.pool,&f.submitter,&stale).await?);
    assert_eq!(f.snapshot(approved.created.run_id).await?,before);Ok(())
}

#[sqlx::test]
async fn case_request_retains_real_content_and_actual_accountable_task(pool:PgPool)->TestResult {
    let s=fixture(pool,"case_request",1).await?;let f=s.company();
    let (actual,a,items,publication)=s.case().await?;
    assert_eq!(value(&actual)?["state"],"OPEN");assert_eq!(actual.case.org_id,f.org);
    assert!(actual.unassigned_exception_id.is_none());let task=actual.task.clone().ok_or("actual case task missing")?;
    let assignment=workflow::read_assignment(&f.pool,&f.reviewer,&task).await?;
    assert_eq!(assignment.account_id,s.deployment.accounts.reviewer.account_id);
    let content=payroll::read_case_content(&f.pool,&f.submitter,&actual.case).await?;
    assert_eq!(value(&content.items)?,value(&items)?);assert_eq!(value(&content.target)?,json!({"kind":"PUBLICATION","publication":publication}));
    let replay=completed(payroll::payroll_review_request(&f.pool,&f.submitter,&a).await?)?;
    assert_eq!(value(&replay)?,value(&actual)?);
    let after=workflow::read_assignment(&f.pool,&f.reviewer,&task).await?;
    assert_eq!(value(&after)?,value(&assignment)?);Ok(())
}

#[sqlx::test]
async fn case_append_preserves_prior_items_and_refuses_stale_revision(pool:PgPool)->TestResult {
    let s=fixture(pool,"case_append",1).await?;let f=s.company();
    let (opened,_,old_items,_)=s.case().await?;let added=s.item("Also explain the rounding shown")?;
    let old_content=payroll::read_case_content(&f.pool,&f.submitter,&opened.case).await?;
    let input:payroll::CaseAppend=typed(json!({"expected":opened.case,"items":[added],"narrative":"My additional acknowledged question"}))?;
    let target=UntrustedActionTargetSelection::existing_case(opened.case.case_id);
    let a=bind_and_seal(&f.pool,&f.submitter,&target,&input).await?;
    let actual=completed(payroll::payroll_review_request_append(&f.pool,&f.submitter,&a).await?)?;
    assert_company(&actual.receipt,&a);assert_eq!(actual.case.case_id,opened.case.case_id);
    assert_eq!(actual.case.content_revision,opened.case.content_revision+1);
    let content=payroll::read_case_content(&f.pool,&f.submitter,&actual.case).await?;
    assert_eq!(content.items.len(),2);assert_eq!(value(&content.items[0])?,value(&old_items[0])?);
    assert_eq!(value(&content.items[1])?,value(&added)?);
    assert_eq!(value(&payroll::read_case_content(&f.pool,&f.submitter,&opened.case).await?)?,value(&old_content)?);
    let fresh=explicit_new_intent::<payroll::CaseAppend>(&f.pool,&f.submitter,&target).await?;
    let stale=bind_and_seal(&f.pool,&f.submitter,&fresh,&input).await?;
    assert_refusal(&payroll::payroll_review_request_append(&f.pool,&f.submitter,&stale).await?);
    assert_eq!(payroll::read_case_control(&f.pool,&f.submitter,actual.case.case_id).await?,actual.case);Ok(())
}

#[sqlx::test]
async fn case_withdraw_retains_immutable_content_and_replays_once(pool:PgPool)->TestResult {
    let s=fixture(pool,"case_withdraw",1).await?;let f=s.company();
    let (opened,_,_,_)=s.case().await?;
    let content=payroll::read_case_content(&f.pool,&f.submitter,&opened.case).await?;
    let input:payroll::CaseWithdraw=typed(json!({"expected":opened.case,"reason":"Explicit requester withdrawal"}))?;
    let a=bind_and_seal(&f.pool,&f.submitter,&UntrustedActionTargetSelection::existing_case(opened.case.case_id),&input).await?;
    let actual=completed(payroll::payroll_review_request_withdraw(&f.pool,&f.submitter,&a).await?)?;
    assert_company(&actual.receipt,&a);assert_eq!(value(&actual)?["state"],"WITHDRAWN");
    assert_eq!(actual.case.case_id,opened.case.case_id);
    assert_eq!(value(&payroll::read_case_content(&f.pool,&f.submitter,&opened.case).await?)?,value(&content)?);
    let replay=completed(payroll::payroll_review_request_withdraw(&f.pool,&f.submitter,&a).await?)?;
    assert_eq!(value(&replay)?,value(&actual)?);Ok(())
}

#[sqlx::test]
async fn case_respond_requires_complete_items_then_resolves_with_real_receipt(pool:PgPool)->TestResult {
    let s=fixture(pool,"case_respond",1).await?;let f=s.company();
    let (opened,_,items,_)=s.case().await?;let task=opened.task.clone().ok_or("real handler task missing")?;
    let input:payroll::CaseRespond=typed(json!({"expected":opened.case,"task":task,"dispositions":[{
        "kind":"ANSWERED","item_id":items[0].item_id,"evidence":[],"answer":"The retained reviewed result is explained without changing its source."}],
        "explanation":"All submitted questions answered; this does not authorize payment."}))?;
    let target=UntrustedActionTargetSelection::existing_case(opened.case.case_id);
    // Use a wrong existing-format issue UUID, not an empty structurally invalid
    // array: this reaches exact owner issue membership validation.
    let mut malformed=value(&input)?;malformed["dispositions"][0]["item_id"]=json!(Uuid::new_v4());
    let wrong:payroll::CaseRespond=typed(malformed)?;
    let bad=bind_and_seal(&f.pool,&f.reviewer,&target,&wrong).await?;
    assert_refusal(&payroll::payroll_review_respond(&f.pool,&f.reviewer,&bad).await?);
    assert_eq!(payroll::read_case_control(&f.pool,&f.reviewer,opened.case.case_id).await?,opened.case);
    let fresh=explicit_new_intent::<payroll::CaseRespond>(&f.pool,&f.reviewer,&target).await?;
    let a=bind_and_seal(&f.pool,&f.reviewer,&fresh,&input).await?;
    let actual=completed(payroll::payroll_review_respond(&f.pool,&f.reviewer,&a).await?)?;
    assert_company(&actual.receipt,&a);assert_eq!(value(&actual)?["state"],"RESOLVED");
    let response=actual.response_id.ok_or("actual response missing")?;
    let content=payroll::read_response_content(&f.pool,&f.submitter,response).await?;
    assert_eq!(content.dispositions.len(),items.len());assert_eq!(content.dispositions[0].item_id(),items[0].item_id);
    assert_eq!(value(&content.dispositions[0])?["kind"],"ANSWERED");
    let assignment=workflow::read_assignment(&f.pool,&f.reviewer,&task).await?;
    assert_eq!(value(&assignment)?["state"],"TERMINAL");
    let charge=group::read_task_charge(&s.deployment.pool,&s.group_auth(&s.deployment.accounts.submitter.account_access_token).await?,f.org,task.task_id).await?;
    assert_eq!(value(&charge)?["state"],"RELEASED");
    let replay=completed(payroll::payroll_review_respond(&f.pool,&f.reviewer,&a).await?)?;
    assert_eq!(value(&replay)?,value(&actual)?);Ok(())
}

#[sqlx::test]
async fn transfer_submit_retains_task_and_selection_without_assigning(pool:PgPool)->TestResult {
    let s=fixture(pool,"transfer_submit",1).await?;let f=s.company();
    let (actual,a,task)=s.transfer().await?;
    let plan=workflow::read_transfer_plan(&f.pool,&f.reviewer,&actual.plan).await?;
    assert_eq!(plan.targets.len(),1);assert_eq!(plan.targets[0].task,task);
    assert_eq!(value(&plan.targets[0].selection)?,json!({"kind":"NOMINATED","account_id":s.deployment.accounts.transfer_recipient.account_id}));
    assert_eq!(value(&plan.targets[0].responsibility)?["kind"],"PERMANENT");
    let assignment=workflow::read_assignment(&f.pool,&f.reviewer,&task).await?;
    assert_eq!(assignment.account_id,s.deployment.accounts.reviewer.account_id);
    assert_eq!(assignment.assignment_epoch,task.assignment_epoch);
    let replay=completed(workflow::work_transfer_submit(&f.pool,&f.reviewer,&a).await?)?;
    assert_eq!(value(&replay)?,value(&actual)?);Ok(())
}

#[sqlx::test]
async fn transfer_execute_changes_one_assignment_and_charge_once(pool:PgPool)->TestResult {
    let s=fixture(pool,"transfer_execute",1).await?;let f=s.company();
    let (submitted,_,task)=s.transfer().await?;
    let before=workflow::read_assignment(&f.pool,&f.reviewer,&task).await?;
    let input=workflow::TransferExecute{plan:submitted.plan.clone(),task:task.clone()};
    let a=bind_and_seal(&f.pool,&f.reviewer,&UntrustedActionTargetSelection::existing_transfer_plan(submitted.plan.plan_id),&input).await?;
    let actual=completed(workflow::work_transfer_execute(&f.pool,&f.reviewer,&a).await?)?;
    assert_company(&actual.receipt,&a);assert_eq!(actual.task_outcomes.len(),1);
    assert_eq!(value(&actual.task_outcomes[0])?["state"],"TRANSFERRED");
    assert_company(actual.task_outcomes[0].receipt.as_ref().ok_or("task effect receipt missing")?,&a);
    let after=workflow::read_current_assignment(&f.pool,&f.reviewer,task.task_id).await?;
    assert_eq!(after.account_id,s.deployment.accounts.transfer_recipient.account_id);
    assert_eq!(after.assignment_epoch,before.assignment_epoch+1);
    let auth=s.group_auth(&s.deployment.accounts.submitter.account_access_token).await?;
    let charge=group::read_task_charge(&s.deployment.pool,&auth,f.org,task.task_id).await?;
    assert_eq!(charge.account_id,after.account_id);assert_eq!(charge.assignment_epoch,after.assignment_epoch);
    assert_eq!(charge.units,1);assert_eq!(value(&charge)?["state"],"ACTIVE");
    let replay=completed(workflow::work_transfer_execute(&f.pool,&f.reviewer,&a).await?)?;
    assert_eq!(value(&replay)?,value(&actual)?);
    assert_eq!(value(&workflow::read_current_assignment(&f.pool,&f.reviewer,task.task_id).await?)?,value(&after)?);
    assert_eq!(value(&group::read_task_charge(&s.deployment.pool,&auth,f.org,task.task_id).await?)?,value(&charge)?);Ok(())
}

#[sqlx::test]
async fn transfer_stop_fences_later_execution_and_keeps_assignment(pool:PgPool)->TestResult {
    let s=fixture(pool,"transfer_stop",1).await?;let f=s.company();
    let (submitted,_,task)=s.transfer().await?;
    let before=workflow::read_assignment(&f.pool,&f.reviewer,&task).await?;
    let target=UntrustedActionTargetSelection::existing_transfer_plan(submitted.plan.plan_id);
    let pending=bind_and_seal(&f.pool,&f.reviewer,&target,&workflow::TransferExecute{plan:submitted.plan.clone(),task:task.clone()}).await?;
    let stop=bind_and_seal(&f.pool,&f.reviewer,&target,&workflow::TransferStop{plan:submitted.plan.clone(),reason:"Stop this pending handover".into()}).await?;
    let actual=completed(workflow::work_transfer_stop(&f.pool,&f.reviewer,&stop).await?)?;
    assert_company(&actual.receipt,&stop);assert_eq!(value(&actual)?["state"],"STOPPED");
    assert_eq!(actual.plan.plan_id,submitted.plan.plan_id);assert!(actual.plan.control_epoch>submitted.plan.control_epoch);
    assert_refusal(&workflow::work_transfer_execute(&f.pool,&f.reviewer,&pending).await?);
    assert_eq!(value(&workflow::read_current_assignment(&f.pool,&f.reviewer,task.task_id).await?)?,value(&before)?);
    let replay=completed(workflow::work_transfer_stop(&f.pool,&f.reviewer,&stop).await?)?;
    assert_eq!(value(&replay)?,value(&actual)?);Ok(())
}

#[sqlx::test]
async fn group_create_freezes_six_actual_companies_without_calculation(pool:PgPool)->TestResult {
    let s=fixture(pool,"group_create",6).await?;
    let (actual,identity,inputs)=s.preparing_group().await?;
    assert_group(&actual.receipt,&identity.command);assert_eq!(value(&actual)?["state"],"PREPARING");
    assert_eq!(actual.children.len(),6);
    let orgs:std::collections::BTreeSet<_>=actual.children.iter().map(|c|c.org_id).collect();
    assert_eq!(orgs.len(),6);assert_eq!(orgs,s.deployment.companies.iter().map(|c|c.org).collect());
    for (index,f) in s.deployment.companies.iter().enumerate(){
        let control=payroll::read_run_control(&f.pool,&f.submitter,inputs[index].0).await?;
        assert_eq!(control,inputs[index].1.expected);assert!(control.calculation_revision.is_none());
    }
    let auth=s.group_auth(&s.deployment.accounts.submitter.account_access_token).await?;
    let stored=group::read_group_operation(&s.deployment.pool,&auth,&actual.operation).await?;
    assert_eq!(stored.original_account_id,s.deployment.accounts.submitter.account_id);
    assert_eq!(stored.slots.len(),6);assert_eq!(stored.operation.manifest_digest,actual.operation.manifest_digest);Ok(())
}

#[sqlx::test]
async fn group_activate_requires_all_prepared_original_attempts(pool:PgPool)->TestResult {
    let s=fixture(pool,"group_activate",6).await?;
    let (parent,_,inputs)=s.preparing_group().await?;let attempts=s.prepare_children(&parent,&inputs).await?;
    let auth=s.group_auth(&s.deployment.accounts.submitter.account_access_token).await?;
    let incomplete_identity=s.group_identity::<group::GroupActivate>(&auth).await?;
    let incomplete=group::group_operation_activate(&s.deployment.pool,&auth,group::GroupRegisteredActionRequest{
        identity:incomplete_identity,input:group::GroupActivate{operation:parent.operation.clone(),prepared_attempts:attempts[..5].to_vec()}
    }).await?;assert_refusal(&incomplete);
    let identity=s.group_identity::<group::GroupActivate>(&auth).await?;
    let input=group::GroupActivate{operation:parent.operation.clone(),prepared_attempts:attempts.clone()};
    let actual=completed(group::group_operation_activate(&s.deployment.pool,&auth,group::GroupRegisteredActionRequest{identity:identity.clone(),input:input.clone()}).await?)?;
    assert_group(&actual.receipt,&identity.command);assert_eq!(value(&actual)?["state"],"ACTIVE");
    assert_eq!(actual.operation.operation_id,parent.operation.operation_id);
    assert_eq!(actual.operation.manifest_digest,parent.operation.manifest_digest);
    let stored=group::read_group_operation(&s.deployment.pool,&auth,&actual.operation).await?;
    let stored_attempts:std::collections::BTreeSet<_>=stored.slots.iter().map(|x|(x.org_id,x.attempt.attempt_id,x.attempt.command_id)).collect();
    assert_eq!(stored_attempts,attempts.iter().map(|x|(x.org_id,x.attempt_id,x.command_id)).collect());
    let replay=completed(group::group_operation_activate(&s.deployment.pool,&auth,group::GroupRegisteredActionRequest{identity:identity.clone(),input}).await?)?;
    assert_eq!(value(&replay)?,value(&actual)?);Ok(())
}

#[sqlx::test]
async fn group_resume_preserves_manifest_and_original_pending_commands(pool:PgPool)->TestResult {
    let mut s=fixture(pool,"group_resume",2).await?;
    let (parent,_,inputs)=s.preparing_group().await?;let attempts=s.prepare_children(&parent,&inputs).await?;
    let auth=s.group_auth(&s.deployment.accounts.submitter.account_access_token).await?;
    let activate=s.group_identity::<group::GroupActivate>(&auth).await?;
    let active=completed(group::group_operation_activate(&s.deployment.pool,&auth,group::GroupRegisteredActionRequest{identity:activate,
        input:group::GroupActivate{operation:parent.operation,prepared_attempts:attempts.clone()}}).await?)?;
    // Real fresh WebAuthn authentication through shared Account producer; never
    // rewrite session IDs or resurrect a stale context in memory.
    let fresh=s.fresh_submitter_login().await?;
    let fresh_auth=s.group_auth(&fresh).await?;
    let identity=s.group_identity::<group::GroupResume>(&fresh_auth).await?;
    let actual=completed(group::group_operation_resume(&s.deployment.pool,&fresh_auth,group::GroupRegisteredActionRequest{identity:identity.clone(),
        input:group::GroupResume{operation:active.operation.clone(),confirm_original_intent:true}}).await?)?;
    assert_group(&actual.receipt,&identity.command);assert_eq!(actual.operation.operation_id,active.operation.operation_id);
    assert_eq!(actual.operation.manifest_digest,active.operation.manifest_digest);
    assert!(actual.operation.dispatch_epoch>active.operation.dispatch_epoch);
    let stored=group::read_group_operation(&s.deployment.pool,&fresh_auth,&actual.operation).await?;
    assert_eq!(stored.original_account_id,s.deployment.accounts.submitter.account_id);
    assert_eq!(stored.slots.iter().map(|x|x.attempt.command_id).collect::<std::collections::BTreeSet<_>>(),
        attempts.iter().map(|x|x.command_id).collect());Ok(())
}

#[sqlx::test]
async fn group_stop_blocks_prepared_child_without_cross_company_effect(pool:PgPool)->TestResult {
    let s=fixture(pool,"group_stop",2).await?;
    let (parent,_,inputs)=s.preparing_group().await?;let attempts=s.prepare_children(&parent,&inputs).await?;
    let auth=s.group_auth(&s.deployment.accounts.submitter.account_access_token).await?;
    let activate=s.group_identity::<group::GroupActivate>(&auth).await?;
    let active=completed(group::group_operation_activate(&s.deployment.pool,&auth,group::GroupRegisteredActionRequest{identity:activate,
        input:group::GroupActivate{operation:parent.operation,prepared_attempts:attempts.clone()}}).await?)?;
    let identity=s.group_identity::<group::GroupStop>(&auth).await?;
    let stop_input=group::GroupStop{operation:active.operation.clone(),reason:"Stop undispatched selected calculations".into()};
    let stopped=completed(group::group_operation_stop(&s.deployment.pool,&auth,group::GroupRegisteredActionRequest{identity:identity.clone(),input:stop_input.clone()}).await?)?;
    assert_group(&stopped.receipt,&identity.command);assert_eq!(value(&stopped)?["state"],"STOPPED");
    for (index,f) in s.deployment.companies.iter().enumerate(){
        let before=f.snapshot(inputs[index].0).await?;
        // Direct ordinary owner route must discover stored parent provenance;
        // omitting parent from call arguments cannot bypass STOPPED.
        assert_refusal(&payroll::payroll_calculate_run(&f.pool,&f.submitter,&attempts[index]).await?);
        assert_eq!(f.snapshot(inputs[index].0).await?,before);
    }
    let replay=completed(group::group_operation_stop(&s.deployment.pool,&auth,group::GroupRegisteredActionRequest{identity:identity.clone(),input:stop_input}).await?)?;
    assert_eq!(value(&replay)?,value(&stopped)?);Ok(())
}
