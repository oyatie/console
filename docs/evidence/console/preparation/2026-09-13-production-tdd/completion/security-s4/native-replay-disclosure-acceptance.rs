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

#[sqlx::test(migrations = false)]
async fn native_original_calculation_replay_fresh_context_reduces_fields_without_second_effect(
    pool: PgPool,
) -> TestResult {
    use crate::security_grant_producer::fresh_context;
    let d = build_native_deployment(pool, "field-only-calculation-replay", 1).await?;
    let f = &d.companies[0];
    let (run, attempt) = f.calculated().await?;
    let snapshot = f.snapshot(run.run_id).await?;
    let receipt_bytes:String=sqlx::query_scalar("SELECT row_to_json(r)::text FROM ont_action_command_receipts r WHERE org_id=$1 AND command_id=$2").bind(f.org).bind(attempt.command_id).fetch_one(&f.readback).await?;
    let schema = payroll::read_native_projection_schema(&d.pool, &f.submitter, run.run_id).await?;
    let purpose = schema.resolve_registered_purpose("replay")?;
    let before = payroll::read_native_authorized_projection(
        &d.pool,
        &f.submitter,
        run.run_id,
        purpose.clone(),
    )
    .await?;
    let encoded = console_ontology_rest::projection::encode_authorized_projection(&before)?;
    assert!(serde_json::to_string(&encoded)?.contains("3000000"));
    let resource = payroll::read_native_line_policy_resource(
        &d.pool,
        &f.submitter,
        run.run_id,
        f.facts.employee_id,
    )
    .await?;
    let gross = resource.resolve_property_path(&["gross_won"])?;
    let address = resource.public_field_address(&gross)?;
    assert!(has_field(&encoded, &address.field_id));
    // Every assignment is loaded through the current ordinary policy owner. The
    // field revision command accepts only a projection, preserving action/scope/
    // population/validity and the original assignment identity by construction.
    let grants = identity::read_current_account_company_grants(
        &d.pool,
        &d.operator,
        f.org,
        d.accounts.submitter.account_id,
    )
    .await?;
    assert!(!grants.is_empty());
    let preserved: Vec<_> = grants.iter().map(|g| g.non_field_inputs()).collect();
    let mut removed = 0;
    for grant in &grants {
        let fields =
            identity::read_field_group_members(&d.pool, &d.operator, &grant.field_projection)
                .await?;
        let mut retained = fields.clone();
        retained.retain(|field| field != &gross);
        removed += fields.len() - retained.len();
        if retained == fields {
            continue;
        }
        let key = format!("test.replay.restricted.{}", grant.grant_id);
        let expected = identity::read_field_group_head(&d.pool, &d.operator, f.org, &key).await?;
        let replacement = identity::publish_field_group(
            &d.pool,
            &d.operator,
            identity::PublishFieldGroup {
                command_id: Uuid::new_v4(),
                org_id: f.org,
                key,
                expected,
                fields: retained,
                reason: "TEST_ONLY field-only current replay restriction".into(),
            },
        )
        .await?;
        let expected = identity::read_company_policy_control(&d.pool, &d.operator, f.org).await?;
        identity::revise_company_grant_field_projection(
            &d.pool,
            &d.operator,
            identity::ReviseCompanyGrantFieldProjection {
                command_id: Uuid::new_v4(),
                org_id: f.org,
                grant_id: grant.grant_id,
                expected,
                field_projection: replacement.reference,
                reason: "TEST_ONLY preserve all non-field authority".into(),
            },
        )
        .await?;
    }
    assert!(
        removed > 0,
        "actual gross permit must be removed; no wildcard omission"
    );
    let after_grants = identity::read_current_account_company_grants(
        &d.pool,
        &d.operator,
        f.org,
        d.accounts.submitter.account_id,
    )
    .await?;
    assert_eq!(
        after_grants
            .iter()
            .map(|g| g.non_field_inputs())
            .collect::<Vec<_>>(),
        preserved
    );
    let fresh = fresh_context(&d, 0, &d.accounts.submitter.account_access_token).await?;
    // Actual original calculation receipt remains readable under the fresh valid
    // context. No blanket Unavailable/stale-context branch can pass this case.
    let replay = completed(payroll::payroll_calculate_run(&d.pool, &fresh, &attempt).await?)?;
    assert_eq!(replay.receipt, run.receipt);
    let current =
        payroll::read_native_authorized_projection(&d.pool, &fresh, run.run_id, purpose).await?;
    let value = console_ontology_rest::projection::encode_authorized_projection(&current)?;
    fn has_field(v: &Value, id: &str) -> bool {
        match v {
            Value::Object(m) => {
                m.get("field_id").and_then(Value::as_str) == Some(id)
                    || m.values().any(|v| has_field(v, id))
            }
            Value::Array(a) => a.iter().any(|v| has_field(v, id)),
            _ => false,
        }
    }
    assert!(!has_field(&value, &address.field_id));
    assert!(!serde_json::to_string(&value)?.contains("3000000"));
    assert!(
        current
            .current_controls()
            .iter()
            .any(|c| c.target() == &resource.run.object),
        "still-authorized run control remains useful"
    );
    assert_eq!(f.snapshot(run.run_id).await?, snapshot);
    assert_eq!(sqlx::query_scalar::<_,String>("SELECT row_to_json(r)::text FROM ont_action_command_receipts r WHERE org_id=$1 AND command_id=$2").bind(f.org).bind(attempt.command_id).fetch_one(&f.readback).await?,receipt_bytes);
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM ont_action_command_receipts WHERE org_id=$1 AND command_id=$2"
        )
        .bind(f.org)
        .bind(attempt.command_id)
        .fetch_one(&f.readback)
        .await?,
        1
    );
    Ok(())
}
