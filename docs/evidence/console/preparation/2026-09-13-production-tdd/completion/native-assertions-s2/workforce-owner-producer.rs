//! Canonical workforce and profile prerequisites through ordinary guarded owners.
//! All returned refs are committed owner results/current reads; no business DML.
use console_platform_request_context::account::AuthenticatedCompanyContext;
use console_platform_auth::account_session::VerifiedAccountSession;
use console_identity_adapter_postgres::account13 as identity;
use console_leave_adapter_postgres::account13 as workforce;
use console_payroll_adapter_postgres::action30 as payroll;
use sqlx::PgPool;
use uuid::Uuid;
use crate::native_fixture::TestResult;

#[derive(Clone)]
pub struct WorkforceFacts {
    pub person_id:Uuid, pub reviewer_person_id:Uuid,
    pub employee_id:Uuid, pub employment:payroll::EmploymentCoverage,
    pub scope:payroll::ScopeBasisRef,
}
pub async fn enroll_workforce(
    pool:&PgPool,auth:&AuthenticatedCompanyContext,operator:&VerifiedAccountSession,
    submitter_account:Uuid,reviewer_account:Uuid,provider:&identity::IdentityProviderRevision,
)->TestResult<WorkforceFacts> {
    let subject=identity::read_current_account_human_binding(pool,operator,auth.company_id(),submitter_account,provider).await?;
    let reviewer=identity::read_current_account_human_binding(pool,operator,auth.company_id(),reviewer_account,provider).await?;
    assert_eq!(subject.account_id,submitter_account); assert_eq!(reviewer.account_id,reviewer_account);
    assert_ne!(subject.human_ref,reviewer.human_ref);
    let subject_person=workforce::ensure_person_for_human_binding(pool,auth,workforce::PersonEnrollment {
        command_id:Uuid::new_v4(),subject_binding:subject.reference.clone(),display_name:"Synthetic payroll subject".into(),
    }).await?;
    let reviewer_person=workforce::ensure_person_for_human_binding(pool,auth,workforce::PersonEnrollment {
        command_id:Uuid::new_v4(),subject_binding:reviewer.reference.clone(),display_name:"Synthetic independent reviewer".into(),
    }).await?;
    assert_eq!(subject_person.human_ref,subject.human_ref);
    assert_eq!(reviewer_person.human_ref,reviewer.human_ref);
    // Account-aware entry into existing create_employee/apply_employment_change
    // transaction participants. Resolves verified Human to canonical Person and
    // binds Account entitlement under current owner guards and Company locks.
    let command_id=Uuid::new_v4();
    let enrolled=workforce::enroll_employee(pool,auth,workforce::EmployeeEnrollment {
        command_id,subject_binding:subject.reference,person:subject_person.reference,display_name:"Synthetic payroll subject".into(),
        employee_number:format!("NATIVE-{}",Uuid::new_v4()),
        starts_on:time::macros::date!(2026-05-31),employment_kind:workforce::EmploymentKind::Regular,
    }).await?;
    assert_eq!(enrolled.original_command_id,command_id);
    let entitlement=identity::read_current_account_employee_entitlement(pool,operator,auth.company_id(),submitter_account).await?;
    assert_eq!(entitlement.person_id,enrolled.person_id);
    assert_eq!(entitlement.employee_id,enrolled.employee_id);
    assert_eq!(entitlement.employment_id,enrolled.employment_id);
    let employment=payroll::read_complete_employment_coverage(pool,auth,enrolled.employee_id,
        &payroll::InclusivePeriod{start:time::macros::date!(2026-06-01),end:time::macros::date!(2026-06-30)}).await?;
    assert_eq!(employment.employment_id,enrolled.employment_id);
    assert!(!employment.segments.is_empty());
    let scope=payroll::create_scope_basis(pool,auth,payroll::CreateScopeBasis {
        command_id:Uuid::new_v4(),employee_ids:vec![enrolled.employee_id],reason:"Synthetic one-subject acceptance population".into(),
    }).await?;
    Ok(WorkforceFacts{person_id:enrolled.person_id,reviewer_person_id:reviewer_person.person_id,
        employee_id:enrolled.employee_id,employment,scope:scope.reference})
}
