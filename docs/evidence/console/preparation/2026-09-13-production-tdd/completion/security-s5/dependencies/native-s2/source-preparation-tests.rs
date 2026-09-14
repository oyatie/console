//! Direct source.prepare_submission registration, including immutable exact head,
//! schema, repeated command, cross-draft substitution and changed-head sealing.
use console_ontology_application::action30::*;
use console_ontology_application::action30::registered_inputs::QualificationProposeInput;
use console_ontology_adapter_postgres::action30 as ontology;
use console_payroll_adapter_postgres::action30 as payroll;
use crate::{native_fixture::{fixture,Fixture,TestResult},binder_sequence::{completed,explicit_new_intent}};
use sqlx::PgPool;
use uuid::Uuid;
async fn saved(f:&Fixture)->TestResult<(SourcePreparationRequest,DraftSeal)> {
 let collection=payroll::read_source_collection_selection(&f.pool,&f.submitter,"QUALIFICATION").await?;
 let selected=explicit_new_intent::<QualificationProposeInput>(&f.pool,&f.submitter,&collection).await?;
 let b=ontology::read_registered_draft_binding::<QualificationProposeInput>(&f.pool,&f.submitter,&selected).await?;
 let started=completed(ontology::draft_start(&f.pool,&f.submitter,UnadmittedControl{command_id:Uuid::new_v4(),input:DraftStart{
  action:b.action,target:b.target,schema:b.schema.clone(),custody:b.custody,intent_slot:b.intent_slot}}).await?)?;
 let input=QualificationProposeInput(f.facts.sources.calendar_payload.clone());
 let ack=completed(ontology::draft_save(&f.pool,&f.submitter,UnadmittedControl{command_id:Uuid::new_v4(),input:DraftSave{
  draft_id:started.draft_id,expected_revision_id:started.revision_id,editing_token:started.editing_token,
  patches:compile_registered_input_patches(&b.schema_snapshot,&input)?}}).await?)?;
 Ok((SourcePreparationRequest{draft_id:ack.draft_id,expected_revision_id:ack.revision_id,editing_token:ack.editing_token.clone(),input_schema:b.schema},
  DraftSeal{draft_id:ack.draft_id,expected_revision_id:ack.revision_id,editing_token:ack.editing_token,prepared_submission_unit:None,gate_request_intent:b.gate_request_intent}))
}
#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn source_preparation_replay_retains_exact_schema_head_and_bytes(pool:PgPool)->TestResult {
 let f=fixture(pool,"source_prepare_replay").await?;let (input,_)=saved(&f).await?;let command_id=Uuid::new_v4();
 let first=completed(ontology::source_prepare_submission(&f.pool,&f.submitter,UnadmittedControl{command_id,input:input.clone()}).await?)?;
 assert_eq!(first.preparation.source.draft_id,input.draft_id);assert_eq!(first.preparation.source.revision_id,input.expected_revision_id);
 assert_eq!(first.preparation.source.org_id,f.org);assert_eq!(first.preparation.source.input_schema_ref,input.input_schema);
 let bytes=ontology::read_prepared_source_bytes(&f.pool,&f.submitter,&first.unit).await?;assert!(!bytes.is_empty());
 let replay=completed(ontology::source_prepare_submission(&f.pool,&f.submitter,UnadmittedControl{command_id,input}).await?)?;
 assert_eq!(first.unit,replay.unit);assert_eq!(first.receipt,replay.receipt);assert_eq!(first.preparation,replay.preparation);
 assert_eq!(bytes,ontology::read_prepared_source_bytes(&f.pool,&f.submitter,&replay.unit).await?);
 // SOURCE_INPUT24 is command-independent; sealed attempt allocated later.
 let decoded=ontology::decode_source24_input_for_readback(&bytes)?;
 assert_eq!(decoded.identity.source_input_digest,first.preparation.source.source_input_digest);
 assert_eq!(ontology::encode_source24_input(&decoded)?,bytes,"closed approved command-independent input format roundtrips exactly");Ok(())
}
#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn source_preparation_other_draft_unit_cannot_seal(pool:PgPool)->TestResult {
 let f=fixture(pool,"source_prepare_cross_draft").await?;let (one,_)=saved(&f).await?;let (two,mut seal)=saved(&f).await?;
 assert_ne!(one.draft_id,two.draft_id);
 let first=completed(ontology::source_prepare_submission(&f.pool,&f.submitter,UnadmittedControl{command_id:Uuid::new_v4(),input:one}).await?)?;
 let continuation=ontology::read_draft_continuation(&f.pool,&f.submitter,two.draft_id).await?;
 seal.editing_token=continuation.editing_token;seal.prepared_submission_unit=Some(first.unit.clone());
 let result=ontology::draft_seal(&f.pool,&f.submitter,UnadmittedControl{command_id:Uuid::new_v4(),input:seal}).await?;
 assert!(matches!(result,OwnerExecution::Observation(ResultObservation::DefinitiveNoEffect{..})));
 assert!(ontology::read_optional_sealed_attempt(&f.pool,&f.submitter,two.draft_id).await?.is_none());
 assert_eq!(ontology::read_prepared_source_identity(&f.pool,&f.submitter,&first.unit).await?,first.preparation.source);Ok(())
}
#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn source_preparation_changed_head_keeps_old_unit_but_cannot_seal_it(pool:PgPool)->TestResult {
 let f=fixture(pool,"source_prepare_stale_head").await?;let (old,mut seal)=saved(&f).await?;
 let prepared=completed(ontology::source_prepare_submission(&f.pool,&f.submitter,UnadmittedControl{command_id:Uuid::new_v4(),input:old.clone()}).await?)?;
 let original=ontology::read_prepared_source_bytes(&f.pool,&f.submitter,&prepared.unit).await?;
 let current=ontology::read_draft_continuation(&f.pool,&f.submitter,old.draft_id).await?;
 let schema=ontology::read_input_schema_snapshot(&f.pool,&f.submitter,&old.input_schema).await?;
 let mut value=serde_json::to_value(&f.facts.sources.calendar_payload)?;
 value["payload"]["citation"]["edition"]=serde_json::json!("NATIVE_DOMAIN_2026_CORRECTED");
 let changed=QualificationProposeInput(serde_json::from_value(value)?);
 let saved=completed(ontology::draft_save(&f.pool,&f.submitter,UnadmittedControl{command_id:Uuid::new_v4(),input:DraftSave{
  draft_id:old.draft_id,expected_revision_id:old.expected_revision_id,editing_token:current.editing_token,
  patches:compile_registered_input_patches(&schema,&changed)?}}).await?)?;
 assert_ne!(saved.revision_id,old.expected_revision_id);
 let refused=ontology::source_prepare_submission(&f.pool,&f.submitter,UnadmittedControl{command_id:Uuid::new_v4(),input:old.clone()}).await?;
 assert!(matches!(refused,OwnerExecution::Observation(ResultObservation::DefinitiveNoEffect{..})));
 seal.expected_revision_id=saved.revision_id;seal.editing_token=saved.editing_token;seal.prepared_submission_unit=Some(prepared.unit.clone());
 let refused=ontology::draft_seal(&f.pool,&f.submitter,UnadmittedControl{command_id:Uuid::new_v4(),input:seal}).await?;
 assert!(matches!(refused,OwnerExecution::Observation(ResultObservation::DefinitiveNoEffect{..})));
 assert_eq!(original,ontology::read_prepared_source_bytes(&f.pool,&f.submitter,&prepared.unit).await?);Ok(())
}
