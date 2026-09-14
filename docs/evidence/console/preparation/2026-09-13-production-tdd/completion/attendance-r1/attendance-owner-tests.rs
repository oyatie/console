//! Application assertions over actual ordinary-produced source facts.
use console_attendance_adapter_postgres::action30 as attendance;
use console_ontology_application::action30::*;
use crate::binder_sequence::{bind_and_seal, completed, explicit_new_intent, ProducerStop};
use crate::native_fixture::{fixture, build_attendance_fixture, TestResult};
use sqlx::PgPool;

fn denied<R:std::fmt::Debug>(r:OwnerExecution<R>) {
    assert!(matches!(r,OwnerExecution::Observation(ResultObservation::DefinitiveNoEffect{..})),"expected exact current validation refusal: {r:?}");
}

#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn calendar_published_bytes_and_same_attempt_replay_are_immutable(pool:PgPool)->TestResult {
    let f=fixture(pool,"calendar_immutable").await?;let facts=&f.facts.attendance;
    let before=attendance::read_calendar_history(&f.pool,&f.submitter,&facts.calendar).await?;
    assert_eq!(before.revisions.len(),1);assert!(before.complete);
    assert_eq!(before.revisions[0].input,facts.calendar_input);
    let replay=completed(attendance::calendar_publish(&f.pool,&f.submitter,&facts.calendar_attempt).await?)?;
    assert_eq!(replay.calendar.as_ref(),Some(&facts.calendar));
    match &replay.receipt {ReceiptRef::Company{command,..}=>assert_eq!(command.command_id,facts.calendar_attempt.command_id),_=>panic!("Company calendar receipt")}
    assert_eq!(before,attendance::read_calendar_history(&f.pool,&f.submitter,&facts.calendar).await?);
    Ok(())
}

#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn calendar_duplicate_weekday_and_wrong_derived_minutes_refuse(pool:PgPool)->TestResult {
    let f=fixture(pool,"calendar_invalid").await?;
    let before=attendance::read_calendar_collection(&f.pool,&f.submitter).await?;
    for bad_minutes in [false,true] {
        let mut value=serde_json::to_value(&f.facts.attendance.calendar_input)?;
        if bad_minutes {value["weekly"][0]["expected_work_minutes"]=serde_json::json!("479");}
        else {value["weekly"][1]["iso_weekday"]=serde_json::json!("1");}
        let input:attendance::CalendarPublish=serde_json::from_value(value)?;
        let target=explicit_new_intent::<attendance::CalendarPublish>(&f.pool,&f.submitter,&UntrustedActionTargetSelection::company()).await?;
        match bind_and_seal(&f.pool,&f.submitter,&target,&input).await {
            Ok(attempt)=>denied(attendance::calendar_publish(&f.pool,&f.submitter,&attempt).await?),
            Err(ProducerStop::Observation(ResultObservation::DefinitiveNoEffect{..}))=>{},
            Err(error)=>return Err(error.into()),
        }
    }
    assert_eq!(before,attendance::read_calendar_collection(&f.pool,&f.submitter).await?);Ok(())
}

#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn calendar_assignment_overlap_refuses_without_second_interval(pool:PgPool)->TestResult {
    let f=fixture(pool,"calendar_overlap").await?;let facts=&f.facts.attendance;
    let before=attendance::read_calendar_assignments(&f.pool,&f.submitter,f.facts.employment.employment_id).await?;
    assert_eq!(before.assignments.len(),1);assert!(before.complete);
    let replay=completed(attendance::calendar_assign(&f.pool,&f.submitter,&facts.assignment_attempt).await?)?;
    assert_eq!(replay.assignment.as_ref(),Some(&facts.assignment));
    let target=explicit_new_intent::<attendance::CalendarAssign>(&f.pool,&f.submitter,&UntrustedActionTargetSelection::existing_employment(f.facts.employment.employment_id)).await?;
    let duplicate=bind_and_seal(&f.pool,&f.submitter,&target,&facts.assignment_input).await?;
    denied(attendance::calendar_assign(&f.pool,&f.submitter,&duplicate).await?);
    assert_eq!(before,attendance::read_calendar_assignments(&f.pool,&f.submitter,f.facts.employment.employment_id).await?);Ok(())
}

#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn attendance_coverage_accounts_for_every_actual_event_and_qualified_break(pool:PgPool)->TestResult {
    let f=fixture(pool,"attendance_complete").await?;let facts=&f.attendance;
    let actual=attendance::read_coverage(&f.pool,&f.submitter,&facts.coverage).await?;
    assert_eq!(actual.outcome,attendance::CoverageOutcome::Resolved);assert!(actual.blockers.is_empty());
    assert_eq!(actual.events.len(),facts.actual_events.len());
    let ids:std::collections::BTreeSet<_>=actual.events.iter().map(|e|e.attendance_record_id).collect();
    assert_eq!(ids.len(),actual.events.len());
    let expected:std::collections::BTreeSet<_>=facts.actual_events.iter().map(|e|e.attendance_record_id).collect();
    assert_eq!(ids,expected);
    // June2026 fixture has22 weekdays and30 calendar dates. This arithmetic is
    // independently literal; the resolver's totals do not serve as their oracle.
    assert_eq!(actual.days.len(),30);
    assert_eq!(actual.days.iter().filter(|d|d.work_minutes==480).count(),22);
    assert_eq!(actual.days.iter().map(|d|d.work_minutes).sum::<u64>(),10560);
    assert_eq!(actual.days.iter().map(|d|d.paid_minutes).sum::<u64>(),10560);
    let replay=completed(attendance::attendance_resolve_coverage(&f.pool,&f.submitter,&facts.coverage_attempt).await?)?;
    assert_eq!(replay.coverage.as_ref(),Some(&facts.coverage));
    assert_eq!(actual,attendance::read_coverage(&f.pool,&f.submitter,&facts.coverage).await?);Ok(())
}

#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn attendance_missing_real_clock_out_is_blocked_without_close_basis(pool:PgPool)->TestResult {
    let f=build_attendance_fixture(pool,"attendance_incomplete",true).await?;
    let facts=&f.attendance;
    let actual=attendance::read_coverage(&f.pool,&f.submitter,&facts.coverage).await?;
    assert_eq!(actual.outcome,attendance::CoverageOutcome::Blocked);
    assert!(actual.blockers.iter().any(|b|b.code==attendance::BlockerCode::IncompleteAttendance));
    assert_eq!(actual.events.len(),43);assert_eq!(facts.actual_events.len(),43);
    assert!(facts.close_basis.is_none());assert!(facts.close_attempt.is_none());Ok(())
}

#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn attendance_changed_package_digest_cannot_create_coverage(pool:PgPool)->TestResult {
    let f=fixture(pool,"attendance_digest").await?;let facts=&f.facts.attendance;
    let before=attendance::read_coverage_collection(&f.pool,&f.submitter,f.facts.employment.employment_id).await?;
    let mut value=serde_json::to_value(&facts.coverage_input)?;
    let old=value["source_package_ref"]["event_set_digest"].as_str().unwrap();
    let wrong=if old=="00".repeat(32){"11".repeat(32)}else{"00".repeat(32)};
    value["source_package_ref"]["event_set_digest"]=serde_json::json!(wrong);
    let input:attendance::CoverageResolve=serde_json::from_value(value)?;
    let target=explicit_new_intent::<attendance::CoverageResolve>(&f.pool,&f.submitter,&UntrustedActionTargetSelection::existing_employment(f.facts.employment.employment_id)).await?;
    match bind_and_seal(&f.pool,&f.submitter,&target,&input).await {
        Ok(attempt)=>denied(attendance::attendance_resolve_coverage(&f.pool,&f.submitter,&attempt).await?),
        Err(ProducerStop::Observation(ResultObservation::DefinitiveNoEffect{..}))=>{},
        Err(error)=>return Err(error.into()),
    }
    assert_eq!(before,attendance::read_coverage_collection(&f.pool,&f.submitter,f.facts.employment.employment_id).await?);Ok(())
}

#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn attendance_close_basis_has_exact_membership_and_replays_its_receipt(pool:PgPool)->TestResult {
    let f=fixture(pool,"attendance_close_exact").await?;let facts=&f.facts.attendance;
    let basis=facts.close_basis.as_ref().expect("complete actual coverage closes");
    let before=attendance::read_close_basis(&f.pool,&f.submitter,basis).await?;
    assert_eq!(before.members.len(),1);assert!(before.complete);assert_eq!(before.members[0].coverage,facts.coverage);
    assert_eq!(before.members[0].employee_id,f.facts.employee_id);
    let attempt=facts.close_attempt.as_ref().unwrap();
    let replay=completed(attendance::attendance_create_close_basis(&f.pool,&f.submitter,attempt).await?)?;
    assert_eq!(replay.close_basis.as_ref(),Some(basis));assert_eq!(before,attendance::read_close_basis(&f.pool,&f.submitter,basis).await?);Ok(())
}

#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn attendance_close_basis_cannot_omit_or_invent_member(pool:PgPool)->TestResult {
    let f=fixture(pool,"attendance_close_omit").await?;let facts=&f.facts.attendance;
    let before=attendance::read_close_basis_collection(&f.pool,&f.submitter,&f.facts.scope).await?;
    for count in ["0","2"] {
        let mut value=serde_json::to_value(facts.close_input.as_ref().unwrap())?;value["member_count"]=serde_json::json!(count);
        let input:attendance::CloseBasisCreate=serde_json::from_value(value)?;
        let target=explicit_new_intent::<attendance::CloseBasisCreate>(&f.pool,&f.submitter,&UntrustedActionTargetSelection::company()).await?;
        match bind_and_seal(&f.pool,&f.submitter,&target,&input).await {
            Ok(attempt)=>denied(attendance::attendance_create_close_basis(&f.pool,&f.submitter,&attempt).await?),
            Err(ProducerStop::Observation(ResultObservation::DefinitiveNoEffect{..}))=>{},
            Err(error)=>return Err(error.into()),
        }
    }
    assert_eq!(before,attendance::read_close_basis_collection(&f.pool,&f.submitter,&f.facts.scope).await?);Ok(())
}
