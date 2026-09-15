//! Only ordinary policy writes; all IDs below are locators, never authority.
use crate::native_fixture::{NativeDeployment, TestResult};
use console_identity_adapter_postgres::account13 as identity;
use console_platform_request_context::account::{
    AuthenticatedCompanyContext, resolve_company_context,
};
use uuid::Uuid;

pub async fn replace_projection(
    d: &NativeDeployment,
    company: usize,
    account: Uuid,
    actions: &[&str],
    projection: &str,
) -> TestResult {
    let f = &d.companies[company];
    let assignments:Vec<Uuid>=sqlx::query_scalar("SELECT r.assignment_id FROM policy_assignment_heads h JOIN policy_assignment_revisions r ON (r.org_id,r.assignment_id,r.revision)=(h.org_id,h.assignment_id,h.current_revision) WHERE r.org_id=$1 AND r.account_id=$2 ORDER BY r.assignment_id")
  .bind(f.org).bind(account).fetch_all(&d.readback).await?;
    for grant_id in assignments {
        let expected = identity::read_company_policy_control(&d.pool, &d.operator, f.org).await?;
        identity::revoke_company_grant(
            &d.pool,
            &d.operator,
            identity::RevokeCompanyGrant {
                command_id: Uuid::new_v4(),
                org_id: f.org,
                grant_id,
                expected,
                reason: "TEST_ONLY current field restriction".into(),
            },
        )
        .await?;
    }
    if !actions.is_empty() {
        let schema = identity::read_company_policy_schema(&d.pool, &d.operator, f.org).await?;
        let expected = identity::read_company_policy_control(&d.pool, &d.operator, f.org).await?;
        let actions = actions
            .iter()
            .map(|a| schema.action(a))
            .collect::<Result<Vec<_>, _>>()?;
        identity::apply_company_grant_plan(
            &d.pool,
            &d.operator,
            identity::ApplyCompanyGrantPlan {
                command_id: Uuid::new_v4(),
                org_id: f.org,
                expected,
                plan: identity::CompanyGrantPlan {
                    account_id: account,
                    scope: identity::PolicyScope::Company,
                    actions,
                    field_projection: schema.projection(projection)?,
                    valid_from: None,
                    valid_to: None,
                    reason: "TEST_ONLY registered projection".into(),
                },
            },
        )
        .await?;
    }
    Ok(())
}
pub async fn fresh_context(
    d: &NativeDeployment,
    company: usize,
    token: &str,
) -> TestResult<AuthenticatedCompanyContext> {
    let selection =
        identity::read_current_company_selection(&d.pool, &d.operator, d.companies[company].org)
            .await?;
    Ok(resolve_company_context(&d.pool, &d.verifier, token, &selection, &d.serving).await?)
}
