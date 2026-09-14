//! Ordinary owner fixture composition, with source bytes from actual persisted
//! event enumerations. No verified source, close or receipt rows are inserted.
use console_attendance_adapter_postgres::{action30 as attendance, records};
use console_payroll_adapter_postgres::action30 as payroll;
use console_ontology_application::action30::*;
use console_platform_request_context::{account::AuthenticatedCompanyContext, worker::AuthenticatedWorkerContext};
use crate::binder_sequence::{bind_and_seal, completed};
use crate::native_fixture::TestResult;
use crate::source_foundation_producer::{ArtifactChannel, upload_source};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{Datelike, Days, NaiveDate, TimeZone, Utc};

pub struct AttendanceFacts {
    pub calendar: payroll::CalendarRef,
    pub assignment: payroll::CalendarAssignmentRef,
    pub coverage: payroll::AttendanceCoverageRef,
    pub close_basis: Option<payroll::AttendanceCloseBasisRef>,
    pub calendar_attempt: AttemptRef,
    pub assignment_attempt: AttemptRef,
    pub coverage_attempt: AttemptRef,
    pub close_attempt: Option<AttemptRef>,
    pub calendar_input: attendance::CalendarPublish,
    pub assignment_input: attendance::CalendarAssign,
    pub coverage_input: attendance::CoverageResolve,
    pub close_input: Option<attendance::CloseBasisCreate>,
    pub actual_events: Vec<records::AttendanceEventRef>,
}

fn at(date:NaiveDate,h:u32,m:u32)->chrono::DateTime<Utc> {
    chrono::FixedOffset::east_opt(9*3600).unwrap().from_local_datetime(&date.and_hms_opt(h,m,0).unwrap()).single().unwrap().with_timezone(&Utc)
}

pub async fn produce_attendance(
    pool:&PgPool,auth:&AuthenticatedCompanyContext,worker:&AuthenticatedWorkerContext,
    channel:&ArtifactChannel<'_>,employee_id:Uuid,employment_id:Uuid,
    rule:&payroll::QualificationRef,scope:&payroll::ScopeBasisRef,
    period:payroll::InclusivePeriod,blocked:bool,
)->TestResult<AttendanceFacts> {
    let from:NaiveDate=period.start.to_string().parse()?;let end:NaiveDate=period.end.to_string().parse()?;
    let exclusive=end.checked_add_days(Days::new(1)).ok_or("period overflow")?;
    let mut weekly=Vec::new();
    for day in 1..=7 {
        let work=day<=5;
        weekly.push(serde_json::json!({"iso_weekday":day.to_string(),
            "day_kind":if work{"WORK_DUTY"}else{"NONWORKING"},
            "duties":if work{serde_json::json!([{"start_minute":"540","end_minute":"1080","breaks":[{"start_minute":"720","end_minute":"780","treatment":"UNPAID"}]}])}else{serde_json::json!([])},
            "expected_work_minutes":if work{"480"}else{"0"},
            "expected_paid_minutes":if work{"480"}else{"0"},"rule_ref":rule}));
    }
    let calendar_input:attendance::CalendarPublish=serde_json::from_value(serde_json::json!({
        "kind":"WORK_CALENDAR","covered_dates":{"from":from.to_string(),"to_exclusive":exclusive.to_string()},
        "zone":"Asia/Seoul","weekly":weekly,"overrides":[],"rule_qualification_ref":rule,
    }))?;
    let target=UntrustedActionTargetSelection::company();
    let calendar_attempt=bind_and_seal(pool,auth,&target,&calendar_input).await?;
    let published=completed(attendance::calendar_publish(pool,auth,&calendar_attempt).await?)?;
    let calendar=published.calendar.expect("actual calendar ref");
    let assignment_input:attendance::CalendarAssign=serde_json::from_value(serde_json::json!({
        "kind":"CALENDAR_ASSIGNMENT","employment_id":employment_id,"calendar_ref":calendar,
        "from_date":from.to_string(),"to_date_exclusive":exclusive.to_string(),"state":"ACTIVE",
    }))?;
    let assignment_attempt=bind_and_seal(pool,auth,&UntrustedActionTargetSelection::existing_employment(employment_id),&assignment_input).await?;
    let assigned=completed(attendance::calendar_assign(pool,auth,&assignment_attempt).await?)?;
    let assignment=assigned.assignment.expect("actual assignment ref");

    // Existing employee_attendance_records writer currently lives in app/hr.rs.
    // The Account-aware ordinary manual correction owner extends that responsible
    // writer; historical occurrence time requires its separate explicit policy.
    // It records the authentic current Account, server recorded_at, supplied
    // occurrence time and reason, advancing the actual ATTENDANCE source guard.
    let mut events=Vec::new();
    let mut date=from;
    let mut last_work=None;
    while date<=end {if date.weekday().number_from_monday()<=5{last_work=Some(date);}date=date.checked_add_days(Days::new(1)).ok_or("date overflow")?;}
    let last_work=last_work.ok_or("fixture requires a working day")?;
    // Establish an actual prior OFF_DUTY anchor via the same audited event owner.
    // The workforce fixture covers this prior date; no absence of records is
    // silently interpreted as an opening state.
    let prior_date=from.checked_sub_days(Days::new(1)).ok_or("date underflow")?;
    let mut prior_anchor=None;
    for (kind,h) in [(records::EventKind::ClockIn,9),(records::EventKind::ClockOut,18)] {
        let result=records::record_manual_event(pool,auth,UnadmittedControl{
            command_id:Uuid::new_v4(),input:records::ManualEventProposal{
                employee_id,employment_id,kind,occurred_at:at(prior_date,h,0),
                reason:"Synthetic actual prior attendance anchor".into(),qualification:rule.clone(),
            },
        }).await?;
        prior_anchor=Some(result.reference);
    }
    let prior_anchor=prior_anchor.expect("two actual prior events");
    date=from;
    while date<=end {
        if date.weekday().number_from_monday()<=5 {
            for (kind,h,m) in [(records::EventKind::ClockIn,9,0),(records::EventKind::ClockOut,18,0)] {
                if blocked && date==last_work && matches!(kind,records::EventKind::ClockOut){continue;}
                let event=records::record_manual_event(pool,auth,UnadmittedControl{
                    command_id:Uuid::new_v4(),input:records::ManualEventProposal {
                        employee_id,employment_id,kind,occurred_at:at(date,h,m),
                        reason:"Synthetic attendance source fixture".into(),qualification:rule.clone(),
                    },
                }).await?;
                events.push(event.reference);
            }
        }
        date=date.checked_add_days(Days::new(1)).ok_or("date overflow")?;
    }
    let enumeration=records::enumerate_coverage_source(pool,auth,employee_id,employment_id,&period,rule).await?;
    assert_eq!(enumeration.boundary.prior_anchor_ref.as_ref(),Some(&prior_anchor));
    assert_eq!(enumeration.events.len(),events.len());
    let expected:std::collections::BTreeSet<_>=events.iter().map(|x|x.attendance_record_id).collect();
    assert_eq!(expected.len(),events.len(),"ordinary producer returned duplicate event refs");
    let actual:std::collections::BTreeSet<_>=enumeration.events.iter().map(|x|x.reference.attendance_record_id).collect();
    assert_eq!(actual.len(),enumeration.events.len(),"enumerator duplicated events");assert_eq!(expected,actual);
    let event_bytes=attendance::encode_source19_events(&enumeration)?;
    let day_bytes=attendance::encode_source19_coverage_days(&enumeration)?;
    let event_artifact=upload_source(pool,auth,worker,channel,"application/x-ndjson",event_bytes).await?;
    let day_artifact=upload_source(pool,auth,worker,channel,"application/x-ndjson",day_bytes).await?;
    let coverage_input=attendance::CoverageResolve {
        kind:attendance::CoverageKind::AttendanceCoverage,employee_id,employment_id,period:period.clone(),
        source_package_ref:attendance::EventPackageRef { artifact_ref:event_artifact.source_artifact,
            format:attendance::EventFormat::Source19NdjsonV1,event_count:enumeration.event_count,event_set_digest:enumeration.event_set_digest },
        boundary:enumeration.boundary,outcome:enumeration.outcome,blockers:enumeration.blockers,
        exception_resolution_refs:enumeration.exception_resolution_refs,
        coverage_days_ref:attendance::CoverageDaysRef { artifact_ref:day_artifact.source_artifact,
            format:attendance::DayFormat::Source19CoverageDaysV1,day_count:enumeration.day_count,day_set_digest:enumeration.day_set_digest },
    };
    let coverage_attempt=bind_and_seal(pool,auth,&UntrustedActionTargetSelection::existing_employment(employment_id),&coverage_input).await?;
    let covered=completed(attendance::attendance_resolve_coverage(pool,auth,&coverage_attempt).await?)?;
    let coverage=covered.coverage.expect("actual source coverage ref");
    let check=attendance::read_coverage(pool,auth,&coverage).await?;
    assert_eq!(check.outcome==attendance::CoverageOutcome::Blocked,blocked);
    if blocked {assert!(!check.blockers.is_empty());}
    let mut close_basis=None;let mut close_attempt=None;let mut close_input=None;
    if !blocked {
        let close_control=attendance::read_operational_close_control(pool,auth,scope,&period).await?;
        let operational=attendance::close_operational_period(pool,auth,UnadmittedControl{command_id:Uuid::new_v4(),input:close_control}).await?;
        let members=attendance::enumerate_close_members(pool,auth,scope,&period,&operational.reference).await?;
        assert_eq!(members.rows.len(),1);assert_eq!(members.rows[0].coverage,coverage);
        let bytes=attendance::encode_source19_close_members(&members)?;
        let artifact=upload_source(pool,auth,worker,channel,"application/x-ndjson",bytes).await?;
        let input=attendance::CloseBasisCreate {
            kind:attendance::CloseKind::AttendanceCloseBasis,period,scope_basis_ref:scope.clone(),
            operational_close_ref:operational.reference,member_count:members.count,member_set_digest:members.digest,
            coverage_members_artifact_ref:artifact.source_artifact,attestation:true,
        };
        let attempt=bind_and_seal(pool,auth,&UntrustedActionTargetSelection::company(),&input).await?;
        let result=completed(attendance::attendance_create_close_basis(pool,auth,&attempt).await?)?;
        close_basis=Some(result.close_basis.expect("actual close basis"));close_attempt=Some(attempt);close_input=Some(input);
    }
    Ok(AttendanceFacts{calendar,assignment,coverage,close_basis,calendar_attempt,assignment_attempt,coverage_attempt,close_attempt,calendar_input,assignment_input,coverage_input,close_input,actual_events:events})
}
