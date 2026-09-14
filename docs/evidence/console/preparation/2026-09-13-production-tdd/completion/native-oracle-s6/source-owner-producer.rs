//! Fixed source19/24 admission -> independent verification -> optional revocation.
//! Each next input is built from actual prior committed source owner control.
//! Control readers below are exact proposed owning reads, never fixture loaders.
use console_ontology_application::action30::*;
use console_ontology_application::action30::registered_inputs::*;
use console_payroll_adapter_postgres::action30 as payroll;
use console_platform_request_context::account::AuthenticatedCompanyContext;
use sqlx::PgPool;
use crate::{binder_sequence::{bind_and_seal,bind_and_seal_with_reason,completed,explicit_new_intent},native_fixture::TestResult};

fn exact_source_receipt(result:&payroll::SourceEffectResult,attempt:&AttemptRef) {
    assert_eq!(result.command.org_id,attempt.org_id);
    assert_eq!(result.command.command_id,attempt.command_id);
    match &result.receipt {
        ReceiptRef::Company{command,..}=>assert_eq!(*command,result.command),
        ReceiptRef::Group{..}=>panic!("Company source returned Group receipt"),
    }
    assert!(!result.effect_ordinals.is_empty());
}

pub async fn produce_verified_qualification(
    pool:&PgPool,submitter:&AuthenticatedCompanyContext,reviewer:&AuthenticatedCompanyContext,
    selection:&UntrustedActionTargetSelection,unsealed:payroll::AdmissionPayload,
)->TestResult<payroll::QualificationAdmissionControl> {
    // Registration-specific newtype fixes the action even where approved actions
    // share AdmissionPayload or VerifyPayload. It adds no wire/body fields.
    let credential_authenticity=serde_json::to_value(&unsealed)?["payload"]["kind"]=="REVIEWER_CREDENTIAL";
    let proposal=QualificationProposeInput(unsealed);
    let selection=explicit_new_intent::<QualificationProposeInput>(pool,submitter,selection).await?;
    let attempt=bind_and_seal(pool,submitter,&selection,&proposal).await?;
    let admitted=completed(payroll::source_propose_qualification(pool,submitter,&attempt).await?)?;
    exact_source_receipt(&admitted,&attempt);
    // Fixed owner reader joins exact source_effect command+declared ordinals to
    // the existing source21 family/head/revision. No caller-supplied success row.
    let control=payroll::read_qualification_admission_control_by_effect(pool,reviewer,&admitted.command,&admitted.effect_ordinals).await?;
    if credential_authenticity {
        assert!(control.current_reviewer_credential_ref.is_none(),"source19 credential authenticity has no recursive professional prerequisite");
        assert!(control.credential_operator_grant_ref.is_some(),"actual current Cedar operator grant readback");
    } else { assert!(control.current_reviewer_credential_ref.is_some()); }
    let verify_input=QualificationVerifyInput(payroll::VerifyPayload {
        admission_ref:control.admission_ref.clone(),admission_digest:control.admission_digest.clone(),
        expected_head_revision:control.head_revision,
        reviewer_credential_ref:control.current_reviewer_credential_ref.clone(),attestation:true,
    });
    let verify_target=control.exact_registered_target.clone();
    let verified_attempt=bind_and_seal(pool,reviewer,&verify_target,&verify_input).await?;
    let verified=completed(payroll::source_verify_qualification(pool,reviewer,&verified_attempt).await?)?;
    exact_source_receipt(&verified,&verified_attempt);
    let current=payroll::read_qualification_admission_control_by_effect(pool,reviewer,&verified.command,&verified.effect_ordinals).await?;
    assert_eq!(current.admission_ref,control.admission_ref);
    assert_eq!(current.state,payroll::AdmissionState::Verified);
    assert!(current.head_revision>control.head_revision);
    Ok(current)
}

pub async fn revoke_actual_qualification(
    pool:&PgPool,auth:&AuthenticatedCompanyContext,actual:&payroll::QualificationAdmissionControl,
    evidence:Vec<payroll::ArtifactRef>,
)->TestResult<payroll::SourceEffectResult> {
    let input=QualificationRevokeInput(payroll::RevokePayload{head_ref:actual.admission_ref.clone(),evidence_refs:evidence});
    let attempt=bind_and_seal_with_reason(pool,auth,&actual.exact_registered_target,&input,Some("Synthetic revoke actual source and retain immutable history")).await?;
    let revoked=completed(payroll::source_revoke_qualification(pool,auth,&attempt).await?)?;
    exact_source_receipt(&revoked,&attempt);
    let current=payroll::read_qualification_admission_control_by_effect(pool,auth,&revoked.command,&revoked.effect_ordinals).await?;
    assert_eq!(current.state,payroll::AdmissionState::Revoked);
    assert!(current.head_revision>actual.head_revision);
    Ok(revoked)
}

pub async fn produce_verified_eligibility(
    pool:&PgPool,submitter:&AuthenticatedCompanyContext,reviewer:&AuthenticatedCompanyContext,
    selection:&UntrustedActionTargetSelection,unsealed:payroll::AdmissionPayload,
)->TestResult<payroll::EligibilityAdmissionControl> {
    // Registration-specific newtype fixes the action even where approved actions
    // share AdmissionPayload or VerifyPayload. It adds no wire/body fields.
    let proposal=EligibilityProposeInput(unsealed);
    let selection=explicit_new_intent::<EligibilityProposeInput>(pool,submitter,selection).await?;
    let attempt=bind_and_seal(pool,submitter,&selection,&proposal).await?;
    let admitted=completed(payroll::eligibility_propose(pool,submitter,&attempt).await?)?;
    exact_source_receipt(&admitted,&attempt);
    // Fixed owner reader joins exact source_effect command+declared ordinals to
    // the existing source21 family/head/revision. No caller-supplied success row.
    let control=payroll::read_eligibility_admission_control_by_effect(pool,reviewer,&admitted.command,&admitted.effect_ordinals).await?;
    assert!(control.current_reviewer_credential_ref.is_some());
    let verify_input=EligibilityVerifyInput(payroll::VerifyPayload {
        admission_ref:control.admission_ref.clone(),admission_digest:control.admission_digest.clone(),
        expected_head_revision:control.head_revision,
        reviewer_credential_ref:control.current_reviewer_credential_ref.clone(),attestation:true,
    });
    let verify_target=control.exact_registered_target.clone();
    let verified_attempt=bind_and_seal(pool,reviewer,&verify_target,&verify_input).await?;
    let verified=completed(payroll::eligibility_verify(pool,reviewer,&verified_attempt).await?)?;
    exact_source_receipt(&verified,&verified_attempt);
    let current=payroll::read_eligibility_admission_control_by_effect(pool,reviewer,&verified.command,&verified.effect_ordinals).await?;
    assert_eq!(current.admission_ref,control.admission_ref);
    assert_eq!(current.state,payroll::AdmissionState::Verified);
    assert!(current.head_revision>control.head_revision);
    Ok(current)
}

pub async fn revoke_actual_eligibility(
    pool:&PgPool,auth:&AuthenticatedCompanyContext,actual:&payroll::EligibilityAdmissionControl,
    evidence:Vec<payroll::ArtifactRef>,
)->TestResult<payroll::SourceEffectResult> {
    let input=EligibilityRevokeInput(payroll::RevokePayload{head_ref:actual.admission_ref.clone(),evidence_refs:evidence});
    let attempt=bind_and_seal_with_reason(pool,auth,&actual.exact_registered_target,&input,Some("Synthetic revoke actual source and retain immutable history")).await?;
    let revoked=completed(payroll::eligibility_revoke(pool,auth,&attempt).await?)?;
    exact_source_receipt(&revoked,&attempt);
    let current=payroll::read_eligibility_admission_control_by_effect(pool,auth,&revoked.command,&revoked.effect_ordinals).await?;
    assert_eq!(current.state,payroll::AdmissionState::Revoked);
    assert!(current.head_revision>actual.head_revision);
    Ok(revoked)
}

pub async fn produce_verified_tax_evidence(
    pool:&PgPool,submitter:&AuthenticatedCompanyContext,reviewer:&AuthenticatedCompanyContext,
    selection:&UntrustedActionTargetSelection,unsealed:payroll::AdmissionPayload,
)->TestResult<payroll::TaxEvidenceAdmissionControl> {
    // Registration-specific newtype fixes the action even where approved actions
    // share AdmissionPayload or VerifyPayload. It adds no wire/body fields.
    let proposal=TaxEvidenceProposeInput(unsealed);
    let selection=explicit_new_intent::<TaxEvidenceProposeInput>(pool,submitter,selection).await?;
    let attempt=bind_and_seal(pool,submitter,&selection,&proposal).await?;
    let admitted=completed(payroll::payroll_admit_tax_evidence(pool,submitter,&attempt).await?)?;
    exact_source_receipt(&admitted,&attempt);
    // Fixed owner reader joins exact source_effect command+declared ordinals to
    // the existing source21 family/head/revision. No caller-supplied success row.
    let control=payroll::read_tax_evidence_admission_control_by_effect(pool,reviewer,&admitted.command,&admitted.effect_ordinals).await?;
    assert!(control.current_reviewer_credential_ref.is_some());
    let verify_input=TaxEvidenceVerifyInput(payroll::VerifyPayload {
        admission_ref:control.admission_ref.clone(),admission_digest:control.admission_digest.clone(),
        expected_head_revision:control.head_revision,
        reviewer_credential_ref:control.current_reviewer_credential_ref.clone(),attestation:true,
    });
    let verify_target=control.exact_registered_target.clone();
    let verified_attempt=bind_and_seal(pool,reviewer,&verify_target,&verify_input).await?;
    let verified=completed(payroll::payroll_verify_tax_evidence(pool,reviewer,&verified_attempt).await?)?;
    exact_source_receipt(&verified,&verified_attempt);
    let current=payroll::read_tax_evidence_admission_control_by_effect(pool,reviewer,&verified.command,&verified.effect_ordinals).await?;
    assert_eq!(current.admission_ref,control.admission_ref);
    assert_eq!(current.state,payroll::AdmissionState::Verified);
    assert!(current.head_revision>control.head_revision);
    Ok(current)
}

pub async fn revoke_actual_tax_evidence(
    pool:&PgPool,auth:&AuthenticatedCompanyContext,actual:&payroll::TaxEvidenceAdmissionControl,
    evidence:Vec<payroll::ArtifactRef>,
)->TestResult<payroll::SourceEffectResult> {
    let input=TaxEvidenceRevokeInput(payroll::RevokePayload{head_ref:actual.admission_ref.clone(),evidence_refs:evidence});
    let attempt=bind_and_seal_with_reason(pool,auth,&actual.exact_registered_target,&input,Some("Synthetic revoke actual source and retain immutable history")).await?;
    let revoked=completed(payroll::payroll_revoke_tax_evidence(pool,auth,&attempt).await?)?;
    exact_source_receipt(&revoked,&attempt);
    let current=payroll::read_tax_evidence_admission_control_by_effect(pool,auth,&revoked.command,&revoked.effect_ordinals).await?;
    assert_eq!(current.state,payroll::AdmissionState::Revoked);
    assert!(current.head_revision>actual.head_revision);
    Ok(revoked)
}
