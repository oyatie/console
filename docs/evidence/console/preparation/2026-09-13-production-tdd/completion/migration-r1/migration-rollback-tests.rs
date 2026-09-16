//! Append as a child of account_migration.rs, sharing its exact225 legacy fixture.
//! Native support modules and real process/snapshot producers are source-pinned.
use super::{seed_legacy,apply_current_and_require_account_catalog,material_snapshot};
use crate::{native_fixture::{build_native_deployment,TestResult},binder_sequence::{bind_and_seal,completed},
    real_process_producer::Process,postgres_snapshot_producer, browser_auth_producer::login_at};
use console_app::{account_migration as migration,serving_admission as admission};
use console_ontology_application::action30::*;
use console_payroll_adapter_postgres::action30 as payroll;
use console_platform_auth::RefreshTokenStore;
use serde_json::{json,Value};
use sqlx::PgPool;
use time::{OffsetDateTime,Duration};

async fn retained(pool:&PgPool,org:uuid::Uuid)->TestResult<Value> {
    // Exact immutable/control objects protected by the approved rollback contract.
    // This deliberately excludes transport-attempt diagnostics and read audits.
    let mut result=serde_json::Map::new();
    for table in ["ont_action_command_receipts","retained_units","retained_payload_locations","content_pins","domain_adoptions","protected_inventory_headers","protected_inventory_members","ont_source_submissions","ont_source_submission_bindings","payroll_input_revisions","payroll_input_units","payroll_calculation_batches","payroll_calculation_units"] {
        let rows:Vec<Value>=sqlx::query_scalar(sqlx::AssertSqlSafe(format!("SELECT to_jsonb(t) FROM public.{table} t WHERE org_id=$1 ORDER BY to_jsonb(t)::text"))).bind(org).fetch_all(pool).await?;
        result.insert(table.into(),Value::Array(rows));
    }
    Ok(Value::Object(result))
}

#[sqlx::test(migrations=false)]
async fn rollback_before_business_write_preserves_permanent_legacy_fence(pool:PgPool)->TestResult {
    let f=seed_legacy(&pool).await;
    apply_current_and_require_account_catalog(&pool,&f).await;
    let manifest=migration::load_installed_manifest(&f.migrate_config).await?;
    let paused=migration::pause_legacy_admission(&f.migrate_config,&manifest).await?;
    let drained=migration::drain_legacy_work(&f.migrate_config,&paused).await?;
    let fence=migration::commit_account_cutover(&f.migrate_config,&drained).await?;
    let before=material_snapshot(&pool,&f.subjects,&f.audit_ids).await;
    let release=admission::load_installed_release(&f.migrate_config).await?;
    let pause=admission::pause_and_drain_effects(&f.migrate_config).await?;
    let compatibility=admission::enter_compatible_read_only(&f.migrate_config,&release,&pause).await?;
    assert!(compatibility.command_service().is_none());
    assert_eq!(material_snapshot(&pool,&f.subjects,&f.audit_ids).await,before);
    let after:(uuid::Uuid,i64,bool)=sqlx::query_as("SELECT id,generation,permanent FROM legacy_authentication_fences WHERE id=$1").bind(fence.id()).fetch_one(&pool).await?;
    assert_eq!(after,(fence.id(),fence.generation(),true));
    assert!(matches!(RefreshTokenStore.rotate(&pool,&f.revoked_token,OffsetDateTime::now_utc(),Duration::hours(2),Duration::days(1)).await,
        Err(console_platform_auth::RefreshTokenUseError::FamilyRevoked)));
    Ok(())
}

#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn compatible_reader_preserves_new_rows_pending_intent_and_original_replay(pool:PgPool)->TestResult {
    let mut d=build_native_deployment(pool,"rollback_new_rows",1).await?;
    let f=&d.companies[0];let (prepared,old_attempt)=f.prepared(false).await?;
    let (run,choices)=f.staged(false).await?;
    let input=f.prepare_input(run.run_id,&choices).await?;
    let pending=bind_and_seal(&f.pool,&f.submitter,&UntrustedActionTargetSelection::existing_run(run.run_id),&input).await?;
    assert!(matches!(payroll::payroll_prepare_inputs(&f.pool,&f.submitter,&pending).await?,OwnerExecution::Observation(ResultObservation::Pending{..})));
    let new_target=crate::binder_sequence::explicit_new_intent::<payroll::RunCreate>(&f.pool,&f.submitter,&choices.create_selection).await?;
    let mut new_input=choices.create.clone();new_input.correlation_id=uuid::Uuid::new_v4();
    let never_dispatched=bind_and_seal(&f.pool,&f.submitter,&new_target,&new_input).await?;
    let before=retained(&d.readback,f.org).await?;
    let run_before=f.snapshot(prepared.run_id).await?;
    let release=admission::load_installed_release(&d.runtime.migrate_config).await?;
    let pause=admission::pause_and_drain_effects(&d.runtime.migrate_config).await?;
    let compatibility=admission::enter_compatible_read_only(&d.runtime.migrate_config,&release,&pause).await?;
    assert!(compatibility.command_service().is_none());
    let config=compatibility.replica_config(&d.runtime.config)?;
    let mut process=Process::launch(&config.binary_path,&config.binary_sha256,&config.environment_pairs()?).await?;
    assert_eq!(process.ready(&d.runtime.client).await?,200);
    let browser=login_at(&d.runtime.client,&process.origin,&d.runtime.origin,&mut d.accounts.submitter).await?;
    let me=browser.request(&d.runtime.client,&process.origin,reqwest::Method::GET,"/api/v2/accounts/me",None).await?;
    assert_eq!(me.status(),200);
    let response=browser.request(&d.runtime.client,&process.origin,reqwest::Method::GET,&format!("/api/v1/payroll/runs/{}",prepared.run_id),None).await?;
    assert_eq!(response.status(),200);
    let body:Value=response.json().await?;assert_eq!(body["run"]["id"],json!(prepared.run_id));
    // Exact original already-completed work may reconcile during effect pause;
    // it cannot make another input or replace its receipt with a wrapper identity.
    let replay=completed(payroll::payroll_prepare_inputs(&f.pool,&f.submitter,&old_attempt).await?)?;
    assert_eq!(replay.receipt,prepared.receipt);
    assert_eq!(run_before,f.snapshot(prepared.run_id).await?);
    assert_eq!(before,retained(&d.readback,f.org).await?);
    let pending_after=console_ontology_adapter_postgres::action30::read_attempt_control(&f.pool,&f.submitter,&pending).await?;
    assert_ne!(pending_after.state,AttemptState::Cancelled);
    // A separate already-sealed new intent must remain effect-blocked. Existing
    // receipt read above is not a blanket grant to execute new commands.
    let outcome=payroll::payroll_prepare_inputs(&f.pool,&f.submitter,&pending).await?;
    assert!(matches!(outcome,OwnerExecution::Observation(ResultObservation::Pending{..})),"effect pause preserves pending intent; unrelated errors are not success");
    assert_eq!(before,retained(&d.readback,f.org).await?);
    let fresh=payroll::payroll_create_run(&f.pool,&f.submitter,&never_dispatched).await;
    assert!(matches!(fresh,Err(console_ontology_adapter_postgres::action30::OwnerActionError::ServingEffectsPaused)));
    assert_eq!(before,retained(&d.readback,f.org).await?);
    let created:i64=sqlx::query_scalar("SELECT count(*) FROM payroll_draft_runs WHERE org_id=$1 AND correlation_id=$2").bind(f.org).bind(new_input.correlation_id).fetch_one(&d.readback).await?;
    assert_eq!(created,0);
    let command:i64=sqlx::query_scalar("SELECT count(*) FROM ont_action_command_receipts WHERE org_id=$1 AND command_id=$2").bind(f.org).bind(never_dispatched.command_id).fetch_one(&d.readback).await?;
    assert_eq!(command,0);
    process.stop().await?;Ok(())
}

#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn restored_old_image_cannot_readmit_against_later_current_authority(pool:PgPool)->TestResult {
    let d=build_native_deployment(pool.clone(),"rollback_stale_image",1).await?;
    let snapshot=postgres_snapshot_producer::snapshot(&pool).await?;
    let f=&d.companies[0];let (later,attempt)=f.prepared(false).await?;
    let installed=admission::load_installed_release(&d.runtime.migrate_config).await?;
    let pause=admission::pause_and_drain_effects(&d.runtime.migrate_config).await?;
    let current=admission::advance_recovery_generation(&d.runtime.migrate_config,&pause).await?;
    let shadow=postgres_snapshot_producer::restore(&pool,&snapshot).await?;
    let count:i64=sqlx::query_scalar("SELECT count(*) FROM ont_action_command_receipts WHERE org_id=$1 AND command_id=$2").bind(f.org).bind(attempt.command_id).fetch_one(&shadow.pool).await?;
    assert_eq!(count,0,"actual restored image must precede later committed receipt");
    let original:i64=sqlx::query_scalar("SELECT count(*) FROM ont_action_command_receipts WHERE org_id=$1 AND command_id=$2").bind(f.org).bind(attempt.command_id).fetch_one(&d.readback).await?;
    assert_eq!(original,1);
    // Same externally held current authority/timeline endpoints, exact new data
    // database only. Restored rows cannot select their own trusted current head.
    let restore_config=d.runtime.migrate_config.for_restored_database_preserving_authority_source(shadow.connection.as_str())?;
    assert_eq!(restore_config.independent_authority_source(),d.runtime.migrate_config.independent_authority_source());
    let outcome=admission::admit_process(&restore_config,&installed).await;
    assert!(matches!(outcome,Err(admission::AdmissionError::IncompleteCurrentTimeline{..})|Err(admission::AdmissionError::StaleRecoveryGeneration{..})));
    let observed=admission::read_recovery_generation(&d.runtime.migrate_config).await?;assert_eq!(observed.generation(),current.generation());
    assert_eq!(completed(payroll::payroll_prepare_inputs(&f.pool,&f.submitter,&attempt).await?)?.receipt,later.receipt);
    shadow.close(&pool).await?;Ok(())
}
