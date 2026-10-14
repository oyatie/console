//! Complete canonical qualification decoding, private to Company policy.
use super::AccountId;
use super::business::{invalid, read_revision, read_uuid, take};
use console_identity_domain::qualification::{
    QualificationRefV1, QualifiedPersonSnapshotV1, SourcePinV1,
};
use console_kernel_core::{KernelError, OrgId};
use time::{OffsetDateTime, UtcOffset};

const PREFIX: &[u8] = b"console.identity.qualified-person-use\0\0\x01";
const MIN_MICROS: i64 = -62_135_596_800_000_000;
const MAX_MICROS: i64 = 253_402_300_799_999_999;

pub(super) fn read_account(input: &mut &[u8]) -> Result<AccountId, KernelError> {
    AccountId::from_uuid(read_uuid(input)?).map_err(|_| invalid())
}

pub(super) fn read_digest(input: &mut &[u8]) -> Result<[u8; 32], KernelError> {
    take(input, 32)?.try_into().map_err(|_| invalid())
}

pub(super) fn read_instant(input: &mut &[u8]) -> Result<OffsetDateTime, KernelError> {
    let micros = i64::from_be_bytes(take(input, 8)?.try_into().map_err(|_| invalid())?);
    if !(MIN_MICROS..=MAX_MICROS).contains(&micros) {
        return Err(invalid());
    }
    OffsetDateTime::from_unix_timestamp_nanos(i128::from(micros) * 1_000)
        .map(|value| value.to_offset(UtcOffset::UTC))
        .map_err(|_| invalid())
}

fn read_qualification(input: &mut &[u8]) -> Result<QualificationRefV1, KernelError> {
    Ok(QualificationRefV1 {
        person_id: read_uuid(input)?,
        person_revision: read_revision(input)?,
        person_receipt_id: read_uuid(input)?,
        assertion_id: read_uuid(input)?,
        assertion_revision: read_revision(input)?,
        assertion_receipt_id: read_uuid(input)?,
        current_pointer_revision: read_revision(input)?,
        observation_id: read_uuid(input)?,
        observation_revision: read_revision(input)?,
        observation_digest: read_digest(input)?,
        target_confirmation_receipt_id: read_uuid(input)?,
        trust_adoption_id: read_uuid(input)?,
        trust_adoption_revision: read_revision(input)?,
        trust_adoption_receipt_id: read_uuid(input)?,
        conflict_witness_id: read_uuid(input)?,
        conflict_witness_revision: read_revision(input)?,
        conflict_witness_digest: read_digest(input)?,
    })
}

fn read_pin(input: &mut &[u8]) -> Result<SourcePinV1, KernelError> {
    Ok(SourcePinV1 {
        source_id: read_uuid(input)?,
        revision: read_revision(input)?,
        receipt_id: read_uuid(input)?,
        content_digest: read_digest(input)?,
    })
}

pub(super) fn decode_snapshot(mut input: &[u8]) -> Result<QualifiedPersonSnapshotV1, KernelError> {
    if !matches!(input.len(), 601 | 609) || take(&mut input, PREFIX.len())? != PREFIX {
        return Err(invalid());
    }
    let company = OrgId::from_uuid(read_uuid(&mut input)?);
    if company == OrgId::platform() {
        return Err(invalid());
    }
    let mut value = QualifiedPersonSnapshotV1 {
        company,
        account: read_account(&mut input)?,
        human_anchor: read_uuid(&mut input)?,
        qualification: read_qualification(&mut input)?,
        person_content_digest: read_digest(&mut input)?,
        assertion_content_digest: read_digest(&mut input)?,
        current_pointer_content_digest: read_digest(&mut input)?,
        conflict_receipt_id: read_uuid(&mut input)?,
        evidence_availability: read_pin(&mut input)?,
        source_impact: read_pin(&mut input)?,
        individual_valid_until: None,
    };
    value.individual_valid_until = match take(&mut input, 1)? {
        [0] => None,
        [1] => Some(read_instant(&mut input)?),
        _ => return Err(invalid()),
    };
    if !input.is_empty() || value.human_anchor != value.qualification.person_id {
        return Err(invalid());
    }
    Ok(value)
}
