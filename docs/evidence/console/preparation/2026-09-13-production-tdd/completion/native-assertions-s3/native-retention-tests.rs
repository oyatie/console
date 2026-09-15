//! Native-specific RR17/EP26 retention identity and live protection assertions.
//! Uses shared ordinary retention owner; does not invent a native cleanup bypass.
use crate::native_fixture::{fixture,Fixture,TestResult};
use console_ontology_application::action30::*;
use console_ontology_adapter_postgres::retention;
use console_payroll_adapter_postgres::action30 as payroll;
use sqlx::PgPool;
use uuid::Uuid;
use sha2::{Digest,Sha256};
fn value<T:serde::Serialize>(v:&T)->TestResult<serde_json::Value>{Ok(serde_json::to_value(v)?)}
async fn actual_streams(f:&Fixture,run:Uuid)->TestResult<payroll::NativeRetainedStreams> {
 let streams=payroll::read_native_retained_streams(&f.pool,&f.submitter,run).await?;
 assert_eq!(streams.semantic_manifest,streams.custody_manifest);
 assert_ne!(streams.semantic_manifest.unit,streams.outcome_manifest.unit);
 assert_eq!(streams.semantic_manifest.unit.org_id,f.org);
 let input=retention::read_exact_retained_payload(&f.pool,&f.submitter,&streams.semantic_manifest).await?;
 assert_eq!(hex::encode(Sha256::digest(&input)),streams.semantic_manifest.evidence_digest);
 assert_eq!(input.len() as u64,streams.semantic_manifest.encoded_size);
 let output=retention::read_exact_retained_payload(&f.pool,&f.submitter,&streams.outcome_manifest).await?;
 assert_eq!(hex::encode(Sha256::digest(&output)),streams.outcome_manifest.evidence_digest);
 Ok(streams)
}
#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn native_aliases_share_one_real_copy_but_keep_distinct_live_obligations(pool:PgPool)->TestResult {
 let f=fixture(pool,"native_alias_protection").await?;let(run,_)=f.calculated().await?;
 let streams=actual_streams(&f,run.run_id).await?;
 let unit=&streams.semantic_manifest.unit;
 let before=retention::read_unit_protection_snapshot(&f.pool,&f.submitter,unit).await?;
 assert_eq!(&before.unit,unit);assert!(!before.locations.is_empty());
 // Every physically admitted location is counted once; multiple real replicas
 // are valid. Semantic/custody alias must not register duplicate locations.
 let ids=before.locations.iter().map(|x|serde_json::to_string(&x.location_ref)).collect::<Result<std::collections::BTreeSet<_>,_>>()?;
 assert_eq!(ids.len(),before.locations.len());
 let aliases=retention::read_native_stream_alias_bindings(&f.pool,&f.submitter,unit).await?;
 assert_eq!(aliases.semantic_locations,aliases.custody_locations);
 assert_eq!(aliases.semantic_locations.len(),before.locations.len());
 assert!(!before.domain_pins.is_empty());
 let policy=retention::read_unit_control(&f.pool,&f.submitter,unit).await?;
 let first=retention::place_hold(&f.pool,&f.submitter,UnadmittedControl{command_id:Uuid::new_v4(),input:retention::HoldProposal{
  target:policy.target,expected_policy_revision:policy.policy_revision,reason:"Synthetic independent preservation purpose A".into(),
 }}).await?;
 let policy=retention::read_unit_control(&f.pool,&f.submitter,unit).await?;
 let second=retention::place_hold(&f.pool,&f.submitter,UnadmittedControl{command_id:Uuid::new_v4(),input:retention::HoldProposal{
  target:policy.target,expected_policy_revision:policy.policy_revision,reason:"Synthetic independent preservation purpose B".into(),
 }}).await?;
 assert_ne!(first.reference,second.reference);
 let held=retention::read_unit_protection_snapshot(&f.pool,&f.submitter,unit).await?;
 let a=held.hold_pins.iter().find(|x|x.hold_ref==first.reference).ok_or("first actual hold pin absent")?;
 let b=held.hold_pins.iter().find(|x|x.hold_ref==second.reference).ok_or("second actual hold pin absent")?;
 assert_ne!(a.obligation_ref,b.obligation_ref);assert_ne!(a.purpose_ref,b.purpose_ref);
 assert_eq!(a.unit,*unit);assert_eq!(b.unit,*unit);
 assert_eq!(value(&held.locations)?,value(&before.locations)?);
 assert_eq!(value(&held.domain_pins)?,value(&before.domain_pins)?);
 let control=retention::read_unit_control(&f.pool,&f.submitter,unit).await?;
 let request=UnadmittedControl{command_id:Uuid::new_v4(),input:retention::HoldRelease{
  hold:first.reference.clone(),expected_policy_revision:control.policy_revision,reason:"Conclude only preservation purpose A".into(),
 }};
 let released=retention::release_hold(&f.pool,&f.submitter,request.clone()).await?;
 let replay=retention::release_hold(&f.pool,&f.submitter,request).await?;
 assert_eq!(value(&released)?,value(&replay)?);
 let partial=retention::read_unit_protection_snapshot(&f.pool,&f.submitter,unit).await?;
 assert!(!partial.hold_pins.iter().any(|x|x.hold_ref==first.reference));
 let surviving=partial.hold_pins.iter().find(|x|x.hold_ref==second.reference).ok_or("partial release lost unrelated hold")?;
 assert_eq!(value(surviving)?,value(b)?);
 assert_eq!(value(&partial.domain_pins)?,value(&before.domain_pins)?);
 assert_eq!(value(&partial.locations)?,value(&before.locations)?);
 assert_eq!(actual_streams(&f,run.run_id).await?.semantic_manifest,streams.semantic_manifest);
 Ok(())
}
#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn native_late_hold_invalidates_disposal_preflight_without_claiming_bytes_removed(pool:PgPool)->TestResult {
 let f=fixture(pool,"native_late_hold_disposal").await?;let(run,_)=f.calculated().await?;
 let streams=actual_streams(&f,run.run_id).await?;let unit=&streams.semantic_manifest.unit;
 let before=retention::read_unit_protection_snapshot(&f.pool,&f.submitter,unit).await?;
 let preflight=retention::read_disposal_eligibility(&f.pool,&f.submitter,unit).await?;
 assert!(!preflight.eligible,"current native input still has mandatory domain protection");
 assert!(preflight.blockers.iter().any(|x|x.code=="active_domain_obligation"));
 let old=retention::read_unit_control(&f.pool,&f.submitter,unit).await?;
 let held=retention::place_hold(&f.pool,&f.submitter,UnadmittedControl{command_id:Uuid::new_v4(),input:retention::HoldProposal{
  target:old.target.clone(),expected_policy_revision:old.policy_revision.clone(),reason:"Actual late hold after disposal preflight".into(),
 }}).await?;
 let fresh=retention::read_disposal_eligibility(&f.pool,&f.submitter,unit).await?;
 assert!(!fresh.eligible);assert!(fresh.blockers.iter().any(|x|x.hold_ref.as_ref()==Some(&held.reference)));
 for (generation,policy) in [(old.generation,old.policy_revision),(fresh.generation,fresh.policy_revision)] {
  let result=retention::prepare_disposal(&f.pool,&f.submitter,UnadmittedControl{command_id:Uuid::new_v4(),input:retention::DisposalPreparation{
   unit:unit.clone(),expected_generation:generation,expected_policy_revision:policy,
  }}).await?;
  match result {OwnerExecution::Observation(ResultObservation::DefinitiveNoEffect{reason,..})=>assert!(matches!(reason.code.as_str(),
   "stale_retention_generation"|"stale_retention_policy"|"active_hold"|"active_domain_obligation")),_=>panic!("held native content admitted to disposal")}
 }
 let after=retention::read_unit_protection_snapshot(&f.pool,&f.submitter,unit).await?;
 assert_eq!(value(&after.locations)?,value(&before.locations)?);
 assert_eq!(value(&after.domain_pins)?,value(&before.domain_pins)?);
 assert!(after.hold_pins.iter().any(|p|p.hold_ref==held.reference));
 assert_eq!(after.disposal_jobs.len(),before.disposal_jobs.len());
 assert_eq!(after.disposal_receipts.len(),before.disposal_receipts.len());
 assert_eq!(actual_streams(&f,run.run_id).await?.semantic_manifest,streams.semantic_manifest);
 Ok(())
}
