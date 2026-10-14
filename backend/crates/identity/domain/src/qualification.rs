//! Pure source references; current applicability and human facts belong to owners.
use console_kernel_core::{AccountId, OrgId};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualificationRefV1 {
    pub person_id: Uuid,
    pub person_revision: u64,
    pub person_receipt_id: Uuid,
    pub assertion_id: Uuid,
    pub assertion_revision: u64,
    pub assertion_receipt_id: Uuid,
    pub current_pointer_revision: u64,
    pub observation_id: Uuid,
    pub observation_revision: u64,
    pub observation_digest: [u8; 32],
    pub target_confirmation_receipt_id: Uuid,
    pub trust_adoption_id: Uuid,
    pub trust_adoption_revision: u64,
    pub trust_adoption_receipt_id: Uuid,
    pub conflict_witness_id: Uuid,
    pub conflict_witness_revision: u64,
    pub conflict_witness_digest: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePinV1 {
    pub source_id: Uuid,
    pub revision: u64,
    pub receipt_id: Uuid,
    pub content_digest: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualifiedPersonSnapshotV1 {
    pub company: OrgId,
    pub account: AccountId,
    pub human_anchor: Uuid,
    pub qualification: QualificationRefV1,
    pub person_content_digest: [u8; 32],
    pub assertion_content_digest: [u8; 32],
    pub current_pointer_content_digest: [u8; 32],
    pub conflict_receipt_id: Uuid,
    pub evidence_availability: SourcePinV1,
    pub source_impact: SourcePinV1,
    pub individual_valid_until: Option<OffsetDateTime>,
}
