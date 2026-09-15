//! First-slice consultation action, not full labor-law or leave product coverage.
use console_leave_adapter_postgres::action30 as leave;
use console_ontology_application::action30::*;
use crate::binder_sequence::{bind_and_seal,completed,explicit_new_intent};
use crate::native_fixture::{fixture,Fixture,TestResult};
use sqlx::PgPool;
use uuid::Uuid;

async fn requested(f:&Fixture)->TestResult<leave::LeaveRequestView> {
    // Account-aware sibling adapter over the existing self-filing leave owner.
    // It resolves current historical Person/Employment and scope itself; input
    // cannot choose another employee or invent approver/charge authority.
    let request=leave::create_leave_request(&f.pool,&f.submitter,UnadmittedControl{
        command_id:Uuid::new_v4(),input:leave::NewLeaveInput {
            leave_type:leave::LeaveType::Annual,
            start_date:time::macros::date!(2026-06-15),end_date:time::macros::date!(2026-06-15),
            partial_day_period:None,reason:"Synthetic request requiring timing discussion".into(),
        },
    }).await?;
    assert_eq!(request.subject_employee_id,f.facts.employee_id);
    assert_eq!(request.state,leave::RequestState::Pending);
    Ok(request)
}

fn consultation(request:&leave::LeaveRequestView)->leave::CurrentLeaveDecision {
    leave::CurrentLeaveDecision {
        request_id:request.id,expected_request_revision:request.revision,
        charge_resolution:None,decision:leave::Decision::TimeChangeConsult,
        reason:"Discuss another date; retain the employee request".into(),
        consultation_proposal_dates:vec![time::macros::date!(2026-06-16)],supporting_evidence:vec![],
    }
}

#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn leave_timing_consultation_keeps_original_request_and_replays_once(pool:PgPool)->TestResult {
    let f=fixture(pool,"leave_consult").await?;let request=requested(&f).await?;
    let before=leave::read_leave_request_history(&f.pool,&f.submitter,request.id).await?;
    let balances=leave::read_own_charge_ledger(&f.pool,&f.submitter).await?;
    let target=UntrustedActionTargetSelection::existing_leave_request(request.id);
    let attempt=bind_and_seal(&f.pool,&f.reviewer,&target,&consultation(&request)).await?;
    let first=completed(leave::leave_decide(&f.pool,&f.reviewer,&attempt).await?)?;
    let replay=completed(leave::leave_decide(&f.pool,&f.reviewer,&attempt).await?)?;
    assert_eq!(first.receipt,replay.receipt);
    let after=leave::read_leave_request_history(&f.pool,&f.submitter,request.id).await?;
    assert!(before.complete && after.complete);
    assert_eq!(before.original_request,after.original_request);
    assert_eq!(after.consultations.len(),before.consultations.len()+1);
    assert_eq!(after.consultations.last().unwrap().proposal_dates,vec![time::macros::date!(2026-06-16)]);
    assert_ne!(after.current_state,leave::RequestState::Denied);
    assert_eq!(balances,leave::read_own_charge_ledger(&f.pool,&f.submitter).await?);
    assert!(after.consultations.last().unwrap().charge_resolution.is_none());Ok(())
}

#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn leave_old_request_revision_cannot_overwrite_new_consultation(pool:PgPool)->TestResult {
    let f=fixture(pool,"leave_stale_consult").await?;let request=requested(&f).await?;
    let input=consultation(&request);let target=UntrustedActionTargetSelection::existing_leave_request(request.id);
    let original=bind_and_seal(&f.pool,&f.reviewer,&target,&input).await?;
    let new_target=explicit_new_intent::<leave::CurrentLeaveDecision>(&f.pool,&f.reviewer,&target).await?;
    let stale=bind_and_seal(&f.pool,&f.reviewer,&new_target,&input).await?;
    assert_ne!(stale.command_id,original.command_id);
    completed(leave::leave_decide(&f.pool,&f.reviewer,&original).await?)?;
    let before=leave::read_leave_request_history(&f.pool,&f.submitter,request.id).await?;
    let refused=leave::leave_decide(&f.pool,&f.reviewer,&stale).await?;
    assert!(matches!(refused,OwnerExecution::Observation(ResultObservation::DefinitiveNoEffect{..})));
    assert_eq!(before,leave::read_leave_request_history(&f.pool,&f.submitter,request.id).await?);Ok(())
}

#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn leave_two_consultations_at_same_revision_have_one_current_effect(pool:PgPool)->TestResult {
    let f=fixture(pool,"leave_concurrent_consult").await?;let request=requested(&f).await?;
    let target=UntrustedActionTargetSelection::existing_leave_request(request.id);
    let one=bind_and_seal(&f.pool,&f.reviewer,&target,&consultation(&request)).await?;
    let selected=explicit_new_intent::<leave::CurrentLeaveDecision>(&f.pool,&f.reviewer,&target).await?;
    let mut changed=consultation(&request);changed.consultation_proposal_dates=vec![time::macros::date!(2026-06-17)];
    let two=bind_and_seal(&f.pool,&f.reviewer,&selected,&changed).await?;
    let (a,b)=tokio::join!(leave::leave_decide(&f.pool,&f.reviewer,&one),leave::leave_decide(&f.pool,&f.reviewer,&two));
    let mut successes=0;for value in [a?,b?]{match value{OwnerExecution::Completed(_)=>successes+=1,OwnerExecution::Observation(ResultObservation::DefinitiveNoEffect{..})=>{},other=>panic!("unclassified consultation result: {other:?}")}}
    assert_eq!(successes,1);
    let after=leave::read_leave_request_history(&f.pool,&f.submitter,request.id).await?;
    assert_eq!(after.consultations.len(),1);assert_eq!(after.original_request.start_date,request.start_date);Ok(())
}
