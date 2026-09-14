//! Actual native producer + original immutable AttemptRef. Revocation is through
//! ordinary current policy owners; stored result/receipt bytes are read separately
//! from the current disclosure outcome. No expected result or seed index loader.
use crate::{
    binder_sequence::completed,
    native_fixture::{TestResult, build_native_deployment},
};
use console_identity_adapter_postgres::account13 as identity;
use console_ontology_application::action30::{OwnerExecution, ResultObservation};
use console_payroll_adapter_postgres::action30 as payroll;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

async fn original_receipt(pool: &PgPool, org: Uuid, command: Uuid) -> TestResult<Value> {
    Ok(sqlx::query_scalar(
        "SELECT to_jsonb(r) FROM ont_action_command_receipts r WHERE org_id=$1 AND command_id=$2",
    )
    .bind(org)
    .bind(command)
    .fetch_one(pool)
    .await?)
}

#[sqlx::test(migrations = false)]
async fn native_original_command_replay_rechecks_disclosure_after_real_grant_revocation(
    pool: PgPool,
) -> TestResult {
    let deployment = build_native_deployment(pool, "security-native-replay-revocation", 1).await?;
    let f = &deployment.companies[0];
    let (run, attempt) = f.prepared(false).await?;
    let before = f.snapshot(run.run_id).await?;
    let receipt = original_receipt(&f.readback, f.org, attempt.command_id).await?;
    let positive =
        completed(payroll::payroll_prepare_inputs(&f.pool, &f.submitter, &attempt).await?)?;
    assert_eq!(positive.receipt, run.receipt);
    assert_eq!(f.snapshot(run.run_id).await?, before);
    // Independently read the actual current assignment heads produced by ordinary
    // B6 grants. IDs are locators only; current operator authorizes each revoke.
    let assignments:Vec<Uuid>=sqlx::query_scalar("SELECT r.assignment_id FROM policy_assignment_heads h JOIN policy_assignment_revisions r ON (r.org_id,r.assignment_id,r.revision)=(h.org_id,h.assignment_id,h.current_revision) WHERE r.org_id=$1 AND r.account_id=$2 ORDER BY r.assignment_id")
        .bind(f.org).bind(deployment.accounts.submitter.account_id).fetch_all(&f.readback).await?;
    assert!(
        !assignments.is_empty(),
        "ordinary fixture failed to create actual scoped assignments"
    );
    for grant_id in assignments {
        let expected =
            identity::read_company_policy_control(&f.pool, &deployment.operator, f.org).await?;
        identity::revoke_company_grant(
            &f.pool,
            &deployment.operator,
            identity::RevokeCompanyGrant {
                command_id: Uuid::new_v4(),
                org_id: f.org,
                grant_id,
                expected,
                reason: "TEST_ONLY current receipt disclosure revoked".into(),
            },
        )
        .await?;
    }
    let replay = payroll::payroll_prepare_inputs(&f.pool, &f.submitter, &attempt).await;
    // Only current authorized projection or explicit permission refusal is
    // permitted. Full historical RunResult cannot escape revoked disclosure.
    match replay {
        Err(payroll::PayrollActionError::PermissionDenied) => {}
        Err(payroll::PayrollActionError::StaleContext) => {}
        _ => panic!("revoked replay disclosed stored success or used unrelated failure"),
    }
    assert_eq!(
        f.snapshot(run.run_id).await?,
        before,
        "replay must not create second native effect"
    );
    assert_eq!(
        original_receipt(&f.readback, f.org, attempt.command_id).await?,
        receipt
    );
    let receipts: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM ont_action_command_receipts WHERE org_id=$1 AND command_id=$2",
    )
    .bind(f.org)
    .bind(attempt.command_id)
    .fetch_one(&f.readback)
    .await?;
    assert_eq!(receipts, 1);
    Ok(())
}
