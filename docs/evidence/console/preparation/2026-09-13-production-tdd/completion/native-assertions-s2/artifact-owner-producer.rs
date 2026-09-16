//! Actual request-upload -> upload bytes -> exact worker verification sequence.
//! Proposed fixed upload worker control read resolves real stored ticket/job state.
use console_ontology_application::action30::*;
use console_docs_adapter_postgres::action30 as docs;
use console_platform_request_context::{account::AuthenticatedCompanyContext,worker::AuthenticatedWorkerContext};
use sqlx::PgPool;
use uuid::Uuid;
use sha2::{Digest,Sha256};
use crate::{binder_sequence::{bind_and_seal,completed,explicit_new_intent},native_fixture::TestResult};

pub struct VerifiedArtifact {
    pub unit:UnitRef,
    pub attestation:docs::AttestationResult,
    pub source_artifact:docs::ArtifactRef,
    pub upload_attempt:AttemptRef,pub ticket:docs::UploadTicket,pub verify_command_id:Uuid,pub verify_input:docs::VerifyUpload,
}
pub async fn upload_and_verify(
    pool:&PgPool,auth:&AuthenticatedCompanyContext,worker:&AuthenticatedWorkerContext,
    client:&reqwest::Client,base_url:&url::Url,account_token:&str,
    selection:&UntrustedActionTargetSelection,purpose:GovernedRevision,
    mime:&str,bytes:Vec<u8>,
)->TestResult<VerifiedArtifact> {
    let selection=explicit_new_intent::<docs::UploadRequest>(pool,auth,selection).await?;
    let binding=console_ontology_adapter_postgres::action30::read_registered_draft_binding::<docs::UploadRequest>(pool,auth,&selection).await?;
    // CREATE target allocation belongs to the persisted explicit NEW intent;
    // construct UploadRequest only after reading its actual allocated target.
    let target=binding.target;
    let expected_digest=hex::encode(Sha256::digest(&bytes));
    let request=docs::UploadRequest{target,purpose,mime_type:mime.into(),expected_bytes:bytes.len().try_into()?,expected_digest:Some(expected_digest.clone())};
    let attempt=bind_and_seal(pool,auth,&selection,&request).await?;
    let ticket=completed(docs::artifact_request_upload(pool,auth,&attempt).await?)?;
    match &ticket.receipt {ReceiptRef::Company{command,..}=>{assert_eq!(command.org_id,attempt.org_id);assert_eq!(command.command_id,attempt.command_id);},_=>panic!("Company upload returned Group receipt")}
    assert!(ticket.max_bytes>=bytes.len().try_into()?);
    // Upload only to the exact owner-issued bounded local path and configured
    // same origin. Do not accept arbitrary URI/ref callbacks from fixture input.
    let url=base_url.join(&ticket.upload_path)?;
    assert_eq!(url.origin(),base_url.origin());
    let response=client.put(url).bearer_auth(account_token).header("content-type",mime).body(bytes).send().await?;
    assert!(response.status().is_success(),"actual upload must complete");
    // Fixed docs owner reads actual completed location revision and queued job;
    // there is no caller-created verified state, fake UUID or guessed revision.
    let control=docs::read_uploaded_verification_control(pool,worker,ticket.ticket_id).await?;
    assert_eq!(control.unit,ticket.target_unit);
    let command_id=Uuid::new_v4();
    let verify_input=docs::VerifyUpload{ticket_id:ticket.ticket_id,unit:control.unit.clone(),location_revision:control.location_revision,verification_job_id:control.verification_job_id};
    let verified=completed(docs::artifact_verify_version(pool,worker,UnadmittedWorkerControl {
        command_id,input:verify_input.clone(),
    }).await?)?;
    assert_eq!(verified.state,docs::AttestationState::Verified);
    match &verified.receipt {ReceiptRef::Company{command,..}=>{assert_eq!(command.command_id,command_id);assert_eq!(command.org_id,ticket.target_unit.org_id);},_=>panic!("Company artifact returned Group receipt")}
    // Fixed source19 ArtifactRef resolver takes exact current docs attestation,
    // internally joins actual stored unit/location/length/digest/verification.
    // It cannot create arbitrary refs or mark an upload verified.
    let source_artifact=docs::read_verified_source_artifact(pool,auth,&verified).await?;
    let retained=docs::read_verified_artifact_digest(pool,auth,&source_artifact).await?;
    assert_eq!(retained,expected_digest);
    Ok(VerifiedArtifact{unit:ticket.target_unit.clone(),attestation:verified,source_artifact,upload_attempt:attempt,ticket,verify_command_id:command_id,verify_input})
}
