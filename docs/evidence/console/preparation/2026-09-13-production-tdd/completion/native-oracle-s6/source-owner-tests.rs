//! Twelve concrete macro-expanded source tests: each of nine fixed business
//! registrations has positive/current, hostile-input, replay and real ACK-loss
//! assertions; three additional histories race distinct revoke intentions.
use console_ontology_application::action30::*;
use console_ontology_application::action30::registered_inputs::*;
use console_payroll_adapter_postgres::action30 as payroll;
use crate::{native_fixture::{fixture,Fixture,TestResult},binder_sequence::{bind_and_seal,bind_and_seal_with_reason,completed,explicit_new_intent},commit_ack_relay::CommitAckRelay};
use sqlx::PgPool;
fn receipt(result:&payroll::SourceEffectResult,attempt:&AttemptRef) {
 assert_eq!(result.command.org_id,attempt.org_id);assert_eq!(result.command.command_id,attempt.command_id);
 match &result.receipt{ReceiptRef::Company{command,..}=>assert_eq!(command,&result.command),_=>panic!("Company source got Group receipt")}
 assert!(!result.effect_ordinals.is_empty());
}
fn refused<T>(r:&OwnerExecution<T>){assert!(matches!(r,OwnerExecution::Observation(ResultObservation::DefinitiveNoEffect{..})));}
macro_rules! family_tests {
 ($proposal_test:ident,$verification_test:ident,$revocation_test:ident,$race_test:ident,
  $family:literal,$proposal:ident,$verify:ident,$revoke:ident,$propose_owner:ident,$verify_owner:ident,$revoke_owner:ident,$reader:ident,$payload:expr)=>{
 #[sqlx::test(migrations="../crates/platform/db/migrations")]
 async fn $proposal_test(pool:PgPool)->TestResult {
  let f=fixture(pool,stringify!($proposal_test)).await?;
  let payload:payroll::AdmissionPayload=($payload)(&f);
  let selection=payroll::read_source_collection_selection(&f.pool,&f.submitter,$family).await?;
  let selection=explicit_new_intent::<$proposal>(&f.pool,&f.submitter,&selection).await?;
  let attempt=bind_and_seal(&f.pool,&f.submitter,&selection,&$proposal(payload.clone())).await?;
  let relay=CommitAckRelay::start(&f.pool).await?;
  let unknown=payroll::$propose_owner(&relay.pool,&f.submitter,&attempt).await;
  assert!(unknown.is_err(),"actual transport ACK was cut");relay.assert_cut()?;relay.stop().await?;
  let actual=completed(payroll::$propose_owner(&f.pool,&f.submitter,&attempt).await?)?;receipt(&actual,&attempt);
  let control=payroll::$reader(&f.pool,&f.submitter,&actual.command,&actual.effect_ordinals).await?;
  assert_eq!(control.state,payroll::AdmissionState::Proposed);
  let before=payroll::read_source_family_snapshot(&f.pool,&f.submitter,$family).await?;
  let replay=completed(payroll::$propose_owner(&f.pool,&f.submitter,&attempt).await?)?;
  assert_eq!(actual.receipt,replay.receipt);assert_eq!(actual.effect_ordinals,replay.effect_ordinals);
  assert_eq!(before,payroll::read_source_family_snapshot(&f.pool,&f.submitter,$family).await?);
  // Known actual head, deliberately nonexistent next revision. Owner may
  // reject while preparing/sealing, but no transport error counts as refusal.
  let mut wrong=serde_json::to_value(&payload)?;
  let mut head=serde_json::to_value(&control.admission_ref)?;
  let revision=head["revision"].as_str().ok_or("retained revision must be decimal text")?.parse::<u64>()?;
  head["revision"]=serde_json::json!((revision+1).to_string());wrong["supersedes_ref"]=head;
  let target=explicit_new_intent::<$proposal>(&f.pool,&f.submitter,&selection).await?;
  crate::assert_registered_refusal!(&f.pool,&f.submitter,&target,&$proposal(serde_json::from_value(wrong)?),payroll::$propose_owner);
  assert_eq!(before,payroll::read_source_family_snapshot(&f.pool,&f.submitter,$family).await?);
  let bytes=payroll::read_retained_source_proposal_bytes(&f.pool,&f.submitter,&control.admission_ref).await?;
  assert!(!bytes.is_empty());
  assert_eq!(bytes,payroll::read_retained_source_proposal_bytes(&f.pool,&f.submitter,&control.admission_ref).await?);Ok(())
 }
 #[sqlx::test(migrations="../crates/platform/db/migrations")]
 async fn $verification_test(pool:PgPool)->TestResult {
  let f=fixture(pool,stringify!($verification_test)).await?;
  let selection=payroll::read_source_collection_selection(&f.pool,&f.submitter,$family).await?;
  let selection=explicit_new_intent::<$proposal>(&f.pool,&f.submitter,&selection).await?;
  let a=bind_and_seal(&f.pool,&f.submitter,&selection,&$proposal(($payload)(&f))).await?;
  let proposed=completed(payroll::$propose_owner(&f.pool,&f.submitter,&a).await?)?;
  let c=payroll::$reader(&f.pool,&f.reviewer,&proposed.command,&proposed.effect_ordinals).await?;
  let good=payroll::VerifyPayload{admission_ref:c.admission_ref.clone(),admission_digest:c.admission_digest.clone(),
   expected_head_revision:c.head_revision,reviewer_credential_ref:c.current_reviewer_credential_ref.clone(),attestation:true};
  assert!(good.reviewer_credential_ref.is_some());
  let alias=payroll::$reader(&f.pool,&f.same_human_other_account,&proposed.command,&proposed.effect_ordinals).await?;
  assert!(alias.current_reviewer_credential_ref.is_some(),"same Human has actual professional credential");
  let mut same_human=good.clone();same_human.reviewer_credential_ref=alias.current_reviewer_credential_ref;
  // Positive control uses the SAME alias and exact verification owner on
  // another actual proposal authored by the independent reviewer Human.
  let other_selection=explicit_new_intent::<$proposal>(&f.pool,&f.reviewer,&selection).await?;
  let other=bind_and_seal(&f.pool,&f.reviewer,&other_selection,&$proposal(($payload)(&f))).await?;
  let other_proposed=completed(payroll::$propose_owner(&f.pool,&f.reviewer,&other).await?)?;
  let other_control=payroll::$reader(&f.pool,&f.same_human_other_account,&other_proposed.command,&other_proposed.effect_ordinals).await?;
  let alias_positive=payroll::VerifyPayload{admission_ref:other_control.admission_ref.clone(),
   admission_digest:other_control.admission_digest,expected_head_revision:other_control.head_revision,
   reviewer_credential_ref:other_control.current_reviewer_credential_ref,attestation:true};
  assert!(alias_positive.reviewer_credential_ref.is_some());
  let positive=bind_and_seal(&f.pool,&f.same_human_other_account,&other_control.exact_registered_target,&$verify(alias_positive)).await?;
  let verified_by_alias=completed(payroll::$verify_owner(&f.pool,&f.same_human_other_account,&positive).await?)?;
  receipt(&verified_by_alias,&positive);
  assert_eq!(payroll::$reader(&f.pool,&f.same_human_other_account,&verified_by_alias.command,&verified_by_alias.effect_ordinals).await?.state,payroll::AdmissionState::Verified);
  let before=payroll::read_source_family_snapshot(&f.pool,&f.submitter,$family).await?;
  crate::assert_registered_refusal!(&f.pool,&f.same_human_other_account,&c.exact_registered_target,&$verify(same_human),payroll::$verify_owner);
  assert_eq!(before,payroll::read_source_family_snapshot(&f.pool,&f.submitter,$family).await?);
  let mut stale=good.clone();stale.expected_head_revision=stale.expected_head_revision.checked_add(1).unwrap();
  let target=explicit_new_intent::<$verify>(&f.pool,&f.reviewer,&c.exact_registered_target).await?;
  crate::assert_registered_refusal!(&f.pool,&f.reviewer,&target,&$verify(stale),payroll::$verify_owner);
  assert_eq!(before,payroll::read_source_family_snapshot(&f.pool,&f.submitter,$family).await?);
  let target=explicit_new_intent::<$verify>(&f.pool,&f.reviewer,&c.exact_registered_target).await?;
  let a=bind_and_seal(&f.pool,&f.reviewer,&target,&$verify(good)).await?;
  let relay=CommitAckRelay::start(&f.pool).await?;
  assert!(payroll::$verify_owner(&relay.pool,&f.reviewer,&a).await.is_err());relay.assert_cut()?;relay.stop().await?;
  let verified=completed(payroll::$verify_owner(&f.pool,&f.reviewer,&a).await?)?;receipt(&verified,&a);
  let current=payroll::$reader(&f.pool,&f.reviewer,&verified.command,&verified.effect_ordinals).await?;
  assert_eq!(current.state,payroll::AdmissionState::Verified);assert!(current.head_revision>c.head_revision);
  assert!(payroll::read_current_usable_source_projection(&f.pool,&f.reviewer,&current.admission_ref).await?.is_some());
  let snapshot=payroll::read_source_family_snapshot(&f.pool,&f.reviewer,$family).await?;
  assert_eq!(completed(payroll::$verify_owner(&f.pool,&f.reviewer,&a).await?)?.receipt,verified.receipt);
  assert_eq!(snapshot,payroll::read_source_family_snapshot(&f.pool,&f.reviewer,$family).await?);Ok(())
 }
 #[sqlx::test(migrations="../crates/platform/db/migrations")]
 async fn $revocation_test(pool:PgPool)->TestResult {
  let f=fixture(pool,stringify!($revocation_test)).await?;
  let selection=payroll::read_source_collection_selection(&f.pool,&f.submitter,$family).await?;
  let selection=explicit_new_intent::<$proposal>(&f.pool,&f.submitter,&selection).await?;
  let a=bind_and_seal(&f.pool,&f.submitter,&selection,&$proposal(($payload)(&f))).await?;
  let proposed=completed(payroll::$propose_owner(&f.pool,&f.submitter,&a).await?)?;
  let c=payroll::$reader(&f.pool,&f.reviewer,&proposed.command,&proposed.effect_ordinals).await?;
  let v=bind_and_seal(&f.pool,&f.reviewer,&c.exact_registered_target,&$verify(payroll::VerifyPayload{admission_ref:c.admission_ref.clone(),admission_digest:c.admission_digest,
   expected_head_revision:c.head_revision,reviewer_credential_ref:c.current_reviewer_credential_ref,attestation:true})).await?;
  let verified=completed(payroll::$verify_owner(&f.pool,&f.reviewer,&v).await?)?;
  let c=payroll::$reader(&f.pool,&f.reviewer,&verified.command,&verified.effect_ordinals).await?;
  let input=payroll::RevokePayload{head_ref:c.admission_ref.clone(),evidence_refs:vec![f.facts.sources.evidence.clone()]};
  let mut wrong=input.clone();wrong.head_ref.revision=wrong.head_ref.revision.checked_add(1).unwrap();
  let before=payroll::read_source_family_snapshot(&f.pool,&f.reviewer,$family).await?;
  crate::assert_registered_refusal!(&f.pool,&f.reviewer,&c.exact_registered_target,&$revoke(wrong),payroll::$revoke_owner; reason = Some("Synthetic stale-source revocation check"));
  assert_eq!(before,payroll::read_source_family_snapshot(&f.pool,&f.reviewer,$family).await?);
  // Blank common reason is a real invalid user input; same current source,
  // reviewer, credential and payload subsequently succeed with explicit reason.
  let empty_target=explicit_new_intent::<$revoke>(&f.pool,&f.reviewer,&c.exact_registered_target).await?;
  crate::assert_registered_refusal!(&f.pool,&f.reviewer,&empty_target,&$revoke(input.clone()),payroll::$revoke_owner; reason = Some("   "));
  assert_eq!(before,payroll::read_source_family_snapshot(&f.pool,&f.reviewer,$family).await?);
  let target=explicit_new_intent::<$revoke>(&f.pool,&f.reviewer,&c.exact_registered_target).await?;
  let a=bind_and_seal_with_reason(&f.pool,&f.reviewer,&target,&$revoke(input),Some("Synthetic revoke source while preserving history")).await?;
  let relay=CommitAckRelay::start(&f.pool).await?;
  assert!(payroll::$revoke_owner(&relay.pool,&f.reviewer,&a).await.is_err());relay.assert_cut()?;relay.stop().await?;
  let revoked=completed(payroll::$revoke_owner(&f.pool,&f.reviewer,&a).await?)?;receipt(&revoked,&a);
  // Independent exact Source24 SQL readback, not the serializer under test.
  // Actual successful owner command identifies the stored submission; no guessed
  // source unit and no diagnostic/tenant-facing raw-byte endpoint is added.
  let (command_bytes,input_bytes):(Vec<u8>,Vec<u8>)=sqlx::query_as(
   "SELECT command_bytes,input_bytes FROM public.ont_source_submissions WHERE org_id=$1 AND command_id=$2 AND attempt_id=$3")
   .bind(f.org).bind(a.command_id).bind(a.attempt_id).fetch_one(&f.readback).await?;
  let normalized:serde_json::Value=serde_json::from_slice(&command_bytes)?;
  assert_eq!(normalized["reason"],"Synthetic revoke source while preserving history");
  assert_eq!(normalized["command_id"],a.command_id.to_string());
  let retained_input:serde_json::Value=serde_json::from_slice(&input_bytes)?;
  assert!(retained_input.get("reason").is_none(),"source19 payload must not duplicate common reason");

  let current=payroll::$reader(&f.pool,&f.reviewer,&revoked.command,&revoked.effect_ordinals).await?;
  assert_eq!(current.state,payroll::AdmissionState::Revoked);
  assert!(payroll::read_current_usable_source_projection(&f.pool,&f.reviewer,&current.admission_ref).await?.is_none());
  let after=payroll::read_source_family_snapshot(&f.pool,&f.reviewer,$family).await?;
  assert_eq!(completed(payroll::$revoke_owner(&f.pool,&f.reviewer,&a).await?)?.receipt,revoked.receipt);
  // Historical success remains replayable after revocation but cannot restore
  // current usability. Receipt lookup precedes NEW-use source guards.
  assert_eq!(completed(payroll::$verify_owner(&f.pool,&f.reviewer,&v).await?)?.receipt,verified.receipt);
  assert_eq!(after,payroll::read_source_family_snapshot(&f.pool,&f.reviewer,$family).await?);Ok(())
 }
 #[sqlx::test(migrations="../crates/platform/db/migrations")]
 async fn $race_test(pool:PgPool)->TestResult {
  let f=fixture(pool,stringify!($race_test)).await?;
  let selection=payroll::read_source_collection_selection(&f.pool,&f.submitter,$family).await?;
  let selection=explicit_new_intent::<$proposal>(&f.pool,&f.submitter,&selection).await?;
  let a=bind_and_seal(&f.pool,&f.submitter,&selection,&$proposal(($payload)(&f))).await?;
  let proposed=completed(payroll::$propose_owner(&f.pool,&f.submitter,&a).await?)?;
  let c=payroll::$reader(&f.pool,&f.reviewer,&proposed.command,&proposed.effect_ordinals).await?;
  let v=bind_and_seal(&f.pool,&f.reviewer,&c.exact_registered_target,&$verify(payroll::VerifyPayload{admission_ref:c.admission_ref.clone(),admission_digest:c.admission_digest,
   expected_head_revision:c.head_revision,reviewer_credential_ref:c.current_reviewer_credential_ref,attestation:true})).await?;
  let verified=completed(payroll::$verify_owner(&f.pool,&f.reviewer,&v).await?)?;
  let c=payroll::$reader(&f.pool,&f.reviewer,&verified.command,&verified.effect_ordinals).await?;
  let input=$revoke(payroll::RevokePayload{head_ref:c.admission_ref.clone(),evidence_refs:vec![f.facts.sources.evidence.clone()]});
  let one=bind_and_seal_with_reason(&f.pool,&f.reviewer,&c.exact_registered_target,&input,Some("Synthetic concurrent source revocation")).await?;
  let target=explicit_new_intent::<$revoke>(&f.pool,&f.reviewer,&c.exact_registered_target).await?;
  let two=bind_and_seal_with_reason(&f.pool,&f.reviewer,&target,&input,Some("Synthetic concurrent source revocation")).await?;
  assert_ne!(one.command_id,two.command_id);assert_ne!(one.attempt_id,two.attempt_id);
  let (a,b)=tokio::join!(payroll::$revoke_owner(&f.pool,&f.reviewer,&one),payroll::$revoke_owner(&f.pool,&f.reviewer,&two));
  let results=[a?,b?];assert_eq!(results.iter().filter(|r|matches!(r,OwnerExecution::Completed(_))).count(),1);
  assert_eq!(results.iter().filter(|r|matches!(r,OwnerExecution::Observation(ResultObservation::DefinitiveNoEffect{..}))).count(),1);
  let winner=results.into_iter().find_map(|r|match r{OwnerExecution::Completed(x)=>Some(x),_=>None}).unwrap();
  let current=payroll::$reader(&f.pool,&f.reviewer,&winner.command,&winner.effect_ordinals).await?;
  assert_eq!(current.state,payroll::AdmissionState::Revoked);assert_eq!(current.head_revision,c.head_revision+1);Ok(())
 }
 };}
family_tests!(qualification_propose_recovers_actual_commit,qualification_verify_current_and_independent,qualification_revoke_keeps_history,qualification_revoke_race_one_winner,
 "QUALIFICATION",QualificationProposeInput,QualificationVerifyInput,QualificationRevokeInput,
 source_propose_qualification,source_verify_qualification,source_revoke_qualification,read_qualification_admission_control_by_effect,
 |f:&Fixture| f.facts.sources.calendar_payload.clone());
family_tests!(eligibility_propose_recovers_actual_commit,eligibility_verify_current_and_independent,eligibility_revoke_keeps_history,eligibility_revoke_race_one_winner,
 "ELIGIBILITY",EligibilityProposeInput,EligibilityVerifyInput,EligibilityRevokeInput,
 eligibility_propose,eligibility_verify,eligibility_revoke,read_eligibility_admission_control_by_effect,
 |f:&Fixture| f.facts.payroll_sources.eligibility_payload.clone());
family_tests!(tax_propose_recovers_actual_commit,tax_verify_current_and_independent,tax_revoke_keeps_history,tax_revoke_race_one_winner,
 "TAX_EVIDENCE",TaxEvidenceProposeInput,TaxEvidenceVerifyInput,TaxEvidenceRevokeInput,
 payroll_admit_tax_evidence,payroll_verify_tax_evidence,payroll_revoke_tax_evidence,read_tax_evidence_admission_control_by_effect,
 |f:&Fixture| f.facts.payroll_sources.tax_payload.clone());
