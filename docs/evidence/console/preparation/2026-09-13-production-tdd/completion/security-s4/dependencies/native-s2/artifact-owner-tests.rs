//! Domain-specific artifact custody assertions against the actual HTTP upload and
//! worker verifier. Redirect/current-policy/egress tests live in security packet.
use console_ontology_application::action30::*;
use console_docs_adapter_postgres::action30 as docs;
use console_ontology_adapter_postgres::action30 as ontology;
use crate::{native_fixture::{fixture,Fixture,TestResult,build_native_deployment},
 binder_sequence::{bind_and_seal,completed,explicit_new_intent},source_foundation_producer::{ArtifactChannel,upload_source}};
use sqlx::PgPool;
use uuid::Uuid;
use sha2::{Digest,Sha256};
async fn pending_upload(f:&Fixture,bytes:Vec<u8>,digest:String)->TestResult<(AttemptRef,docs::UploadTicket,reqwest::StatusCode)> {
 let c=docs::read_upload_request_control(&f.pool,&f.submitter,"payroll.source.evidence").await?;
 let target=explicit_new_intent::<docs::UploadRequest>(&f.pool,&f.submitter,&c.selection).await?;
 let binding=ontology::read_registered_draft_binding::<docs::UploadRequest>(&f.pool,&f.submitter,&target).await?;
 let input=docs::UploadRequest{target:binding.target,purpose:c.purpose,mime_type:"application/json".into(),expected_bytes:bytes.len().try_into()?,expected_digest:Some(digest)};
 let a=bind_and_seal(&f.pool,&f.submitter,&target,&input).await?;
 let ticket=completed(docs::artifact_request_upload(&f.pool,&f.submitter,&a).await?)?;
 let url=f.runtime.origin.join(&ticket.upload_path)?;assert_eq!(url.origin(),f.runtime.origin.origin());
 let status=f.runtime.client.put(url).bearer_auth(&f.account_token).header("content-type","application/json").body(bytes).send().await?.status();
 Ok((a,ticket,status))
}
#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn artifact_upload_request_and_worker_verification_replay_exact_receipts(pool:PgPool)->TestResult {
 let f=fixture(pool,"artifact_receipts").await?;
 let channel=ArtifactChannel{client:&f.runtime.client,origin:&f.runtime.origin,token:&f.account_token};
 let actual=upload_source(&f.pool,&f.submitter,&f.worker,&channel,"application/json",b"{\"synthetic\":true}".to_vec()).await?;
 let before=docs::read_unit_version_history(&f.pool,&f.submitter,&actual.unit).await?;
 let replay=completed(docs::artifact_request_upload(&f.pool,&f.submitter,&actual.upload_attempt).await?)?;
 assert_eq!(replay.ticket_id,actual.ticket.ticket_id);assert_eq!(replay.target_unit,actual.unit);assert_eq!(replay.receipt,actual.ticket.receipt);
 let replay=completed(docs::artifact_verify_version(&f.pool,&f.worker,UnadmittedWorkerControl{command_id:actual.verify_command_id,input:actual.verify_input}).await?)?;
 assert_eq!(replay.attestation_id,actual.attestation.attestation_id);assert_eq!(replay.receipt,actual.attestation.receipt);
 assert_eq!(before,docs::read_unit_version_history(&f.pool,&f.submitter,&actual.unit).await?);Ok(())
}
#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn artifact_wrong_declared_digest_cannot_create_verified_source(pool:PgPool)->TestResult {
 let f=fixture(pool,"artifact_wrong_digest").await?;
 let bytes=b"{\"actual\":1}".to_vec();assert_ne!(hex::encode(Sha256::digest(&bytes)),"f".repeat(64));
 let (_,ticket,status)=pending_upload(&f,bytes,"f".repeat(64)).await?;
 if status.is_success() {
  let control=docs::read_uploaded_verification_control(&f.pool,&f.worker,ticket.ticket_id).await?;
  let value=docs::artifact_verify_version(&f.pool,&f.worker,UnadmittedWorkerControl{command_id:Uuid::new_v4(),input:docs::VerifyUpload {
   ticket_id:ticket.ticket_id,unit:control.unit,location_revision:control.location_revision,verification_job_id:control.verification_job_id,
  }}).await?;
  match value {OwnerExecution::Completed(failed)=>assert_eq!(failed.state,docs::AttestationState::Failed),
   OwnerExecution::Observation(ResultObservation::DefinitiveNoEffect{..})=>{},other=>panic!("digest failure cannot be verified/pending: {other:?}")};
 } else {assert!(matches!(status.as_u16(),400|409|422),"only classified digest refusal, no transport/auth/server error");}
 let attestations=docs::read_ticket_attestations(&f.pool,&f.submitter,ticket.ticket_id).await?;
 assert!(attestations.iter().all(|a|a.state!=docs::AttestationState::Verified));
 assert!(docs::read_optional_verified_source_artifact(&f.pool,&f.submitter,ticket.ticket_id).await?.is_none());Ok(())
}
#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn artifact_reused_ticket_cannot_verify_changed_bytes_or_location_revision(pool:PgPool)->TestResult {
 let f=fixture(pool,"artifact_old_ticket").await?;
 let channel=ArtifactChannel{client:&f.runtime.client,origin:&f.runtime.origin,token:&f.account_token};
 let original=b"{\"original\":1}".to_vec();
 let actual=upload_source(&f.pool,&f.submitter,&f.worker,&channel,"application/json",original.clone()).await?;
 let before=docs::read_unit_version_history(&f.pool,&f.submitter,&actual.unit).await?;
 let url=f.runtime.origin.join(&actual.ticket.upload_path)?;assert_eq!(url.origin(),f.runtime.origin.origin());
 let status=f.runtime.client.put(url).bearer_auth(&f.account_token).header("content-type","application/json")
  .body(b"{\"changed\":2}".to_vec()).send().await?.status();
 assert!(matches!(status.as_u16(),409|410|422));
 let mut wrong=actual.verify_input.clone();wrong.location_revision=wrong.location_revision.checked_add(1).unwrap();
 let refused=docs::artifact_verify_version(&f.pool,&f.worker,UnadmittedWorkerControl{command_id:Uuid::new_v4(),input:wrong}).await?;
 assert!(matches!(refused,OwnerExecution::Observation(ResultObservation::DefinitiveNoEffect{..})));
 assert_eq!(docs::read_verified_artifact_digest(&f.pool,&f.submitter,&actual.source_artifact).await?,hex::encode(Sha256::digest(original)));
 assert_eq!(before,docs::read_unit_version_history(&f.pool,&f.submitter,&actual.unit).await?);Ok(())
}
#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn artifact_parallel_worker_commands_append_one_verification(pool:PgPool)->TestResult {
 let f=fixture(pool,"artifact_worker_race").await?;let bytes=b"{\"worker_race\":true}".to_vec();
 let (_,ticket,status)=pending_upload(&f,bytes.clone(),hex::encode(Sha256::digest(bytes))).await?;assert!(status.is_success());
 let c=docs::read_uploaded_verification_control(&f.pool,&f.worker,ticket.ticket_id).await?;
 let input=docs::VerifyUpload{ticket_id:ticket.ticket_id,unit:c.unit,location_revision:c.location_revision,verification_job_id:c.verification_job_id};
 let one=Uuid::new_v4();let two=Uuid::new_v4();
 let (a,b)=tokio::join!(docs::artifact_verify_version(&f.pool,&f.worker,UnadmittedWorkerControl{command_id:one,input:input.clone()}),
  docs::artifact_verify_version(&f.pool,&f.worker,UnadmittedWorkerControl{command_id:two,input}));
 let values=[a?,b?];assert_eq!(values.iter().filter(|v|matches!(v,OwnerExecution::Completed(_))).count(),1);
 assert_eq!(values.iter().filter(|v|matches!(v,OwnerExecution::Observation(ResultObservation::DefinitiveNoEffect{..}))).count(),1);
 let actual=docs::read_ticket_attestations(&f.pool,&f.submitter,ticket.ticket_id).await?;
 assert_eq!(actual.iter().filter(|a|a.state==docs::AttestationState::Verified).count(),1);Ok(())
}
#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn artifact_worker_boundary_refuses_account_token_before_owner_dispatch(pool:PgPool)->TestResult {
 use console_platform_request_context::{worker::resolve_worker_context,ContextError};
 let d=build_native_deployment(pool,"artifact_worker_identity",1).await?;let f=&d.companies[0];
 let selection=console_identity_adapter_postgres::account13::read_current_company_selection(&f.pool,&d.operator,f.org).await?;
 let refused=resolve_worker_context(&f.pool,&d.accounts.submitter.account_access_token,&selection,&d.serving).await;
 assert!(matches!(refused,Err(ContextError::InvalidCredential{..})),"ACCOUNT_V1 cannot produce a worker context");
 Ok(())
}
