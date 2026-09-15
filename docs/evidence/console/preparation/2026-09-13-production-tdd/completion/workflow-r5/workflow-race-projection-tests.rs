//! Additional focused races/current-disclosure probes. Same owner/fixture path.
use crate::{
    binder_sequence::{bind_and_seal, completed, explicit_new_intent},
    native_fixture::TestResult,
    workflow_fixture::{assert_company, assert_refusal, fixture, typed, value},
};
use console_identity_adapter_postgres::account13 as identity;
use console_ontology_application::action30::*;
use console_payroll_adapter_postgres::action30 as payroll;
use console_workflow_runtime_adapter_postgres::action30 as workflow;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test(migrations = "../crates/platform/db/migrations")]
async fn publication_concurrent_same_predecessor_has_one_exact_successor(
    pool: PgPool,
) -> TestResult {
    let s = fixture(pool, "publication_race", 1).await?;
    let f = s.company();
    let (published, release, _) = s.publication().await?;
    let old = published.publications[0].clone();
    let input: payroll::PublicationReplace = typed(
        json!({"predecessor":old,"projection":s.config.replacement_projection,"reason":"Concurrent explicit replacement"}),
    )?;
    let target = UntrustedActionTargetSelection::existing_publication(old.publication_id);
    let left = bind_and_seal(&f.pool, &f.submitter, &target, &input).await?;
    let new =
        explicit_new_intent::<payroll::PublicationReplace>(&f.pool, &f.submitter, &target).await?;
    let right = bind_and_seal(&f.pool, &f.submitter, &new, &input).await?;
    assert_ne!(left.attempt_id, right.attempt_id);
    assert_ne!(left.command_id, right.command_id);
    let (x, y) = tokio::join!(
        payroll::payroll_review_replace_projection(&f.pool, &f.submitter, &left),
        payroll::payroll_review_replace_projection(&f.pool, &f.submitter, &right)
    );
    let (winner, winner_attempt, loser, loser_attempt) = match (x?, y?) {
        (OwnerExecution::Completed(w), l @ OwnerExecution::Observation(_)) => (w, &left, l, &right),
        (l @ OwnerExecution::Observation(_), OwnerExecution::Completed(w)) => (w, &right, l, &left),
        _ => panic!("one exact same-predecessor replacement must win"),
    };
    assert_company(&winner.receipt, winner_attempt);
    assert_refusal(&loser);
    if let OwnerExecution::Observation(ResultObservation::DefinitiveNoEffect { receipt, .. }) =
        &loser
    {
        assert_company(receipt, loser_attempt);
    }
    let after = s
        .publication_snapshot(&[release.command_id, left.command_id, right.command_id])
        .await?;
    assert_eq!(after["publications"].as_array().unwrap().len(), 2);
    assert_eq!(after["slots"].as_array().unwrap().len(), 1);
    assert_eq!(after["controls"].as_array().unwrap().len(), 2);
    assert_eq!(winner.publications.len(), 1);
    assert_eq!(
        after["slots"][0]["current_publication_id"],
        json!(winner.publications[0].publication_id)
    );
    let original = payroll::read_publication_record(&f.pool, &f.submitter, &old).await?;
    assert_eq!(original.publication.publication_id, old.publication_id);
    let replay = completed(
        payroll::payroll_review_replace_projection(&f.pool, &f.submitter, winner_attempt).await?,
    )?;
    assert_eq!(value(&replay)?, value(&winner)?);
    assert_eq!(
        s.publication_snapshot(&[release.command_id, left.command_id, right.command_id])
            .await?,
        after
    );
    Ok(())
}

#[sqlx::test(migrations = "../crates/platform/db/migrations")]
async fn publication_replay_after_current_grant_revocation_is_not_disclosed(
    pool: PgPool,
) -> TestResult {
    let s = fixture(pool, "publication_revoked_replay", 1).await?;
    let f = s.company();
    let (_, a, _) = s.publication().await?;
    let before = s.publication_snapshot(&[a.command_id]).await?;
    let grants = identity::read_current_company_grants_for_account(
        &s.deployment.pool,
        &s.deployment.operator,
        f.org,
        s.deployment.accounts.submitter.account_id,
    )
    .await?;
    assert!(!grants.is_empty());
    for grant in grants {
        let expected = identity::read_company_policy_control(
            &s.deployment.pool,
            &s.deployment.operator,
            f.org,
        )
        .await?;
        identity::revoke_company_grant(
            &s.deployment.pool,
            &s.deployment.operator,
            identity::RevokeCompanyGrant {
                command_id: Uuid::new_v4(),
                org_id: f.org,
                grant_id: grant.grant_id,
                expected,
                reason: "Current disclosure revoked by ordinary operator".into(),
            },
        )
        .await?;
    }
    // Precise concealed authorization refusal, not any error or a fabricated
    // projected result. It must not return old unrestricted PublicationResult.
    match payroll::payroll_review_release(&f.pool, &f.submitter, &a).await {
        Ok(OwnerExecution::Observation(ResultObservation::Unavailable { error })) => {
            assert_eq!(value(&error)?["code"], "not_found")
        }
        Err(error) => assert_eq!(error.public_code(), "not_found"),
        _ => panic!("revoked current disclosure must not return original publication fields"),
    }
    assert_eq!(s.publication_snapshot(&[a.command_id]).await?, before);
    Ok(())
}

#[sqlx::test(migrations = "../crates/platform/db/migrations")]
async fn transfer_stop_races_effect_preserving_one_truthful_assignment(pool: PgPool) -> TestResult {
    let s = fixture(pool, "transfer_stop_race", 1).await?;
    let f = s.company();
    let (plan, _, task) = s.transfer().await?;
    let before = workflow::read_assignment(&f.pool, &f.reviewer, &task).await?;
    let target = UntrustedActionTargetSelection::existing_transfer_plan(plan.plan.plan_id);
    let execute = bind_and_seal(
        &f.pool,
        &f.reviewer,
        &target,
        &workflow::TransferExecute {
            plan: plan.plan.clone(),
            task: task.clone(),
        },
    )
    .await?;
    let stop = bind_and_seal(
        &f.pool,
        &f.reviewer,
        &target,
        &workflow::TransferStop {
            plan: plan.plan.clone(),
            reason: "Concurrent explicit stop".into(),
        },
    )
    .await?;
    let (effect, stopping) = tokio::join!(
        workflow::work_transfer_execute(&f.pool, &f.reviewer, &execute),
        workflow::work_transfer_stop(&f.pool, &f.reviewer, &stop)
    );
    let effect = effect?;
    let stopping = stopping?;
    let after = workflow::read_current_assignment(&f.pool, &f.reviewer, task.task_id).await?;
    match effect {
        OwnerExecution::Completed(ref result) => {
            assert_company(&result.receipt, &execute);
            assert_eq!(value(&result.task_outcomes[0])?["state"], "TRANSFERRED");
            assert_eq!(
                after.account_id,
                s.deployment.accounts.transfer_recipient.account_id
            );
            assert_eq!(after.assignment_epoch, before.assignment_epoch + 1);
        }
        ref refusal => {
            assert_refusal(refusal);
            assert_eq!(value(&after)?, value(&before)?);
            let stopped = completed(stopping)?;
            assert_company(&stopped.receipt, &stop);
            assert_eq!(value(&stopped)?["state"], "STOPPED");
            return Ok(());
        }
    }
    // Effect-first may complete the one-target plan before stale stop. A precise
    // no-effect or STOPPED result is truthful; neither can undo committed work.
    match stopping {
        OwnerExecution::Completed(ref result) => {
            assert_company(&result.receipt, &stop);
            assert_eq!(value(result)?["state"], "STOPPED");
        }
        ref refusal => assert_refusal(refusal),
    }
    let replay = completed(workflow::work_transfer_execute(&f.pool, &f.reviewer, &execute).await?)?;
    assert_company(&replay.receipt, &execute);
    assert_eq!(
        value(&workflow::read_current_assignment(&f.pool, &f.reviewer, task.task_id).await?)?,
        value(&after)?
    );
    Ok(())
}
