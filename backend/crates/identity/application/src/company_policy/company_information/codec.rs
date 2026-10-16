//! Exact finite wire grammar; source and authorization validation are separate.
use super::*;
use crate::company_policy::business::{read_revision, read_uuid, take};

impl NativeCompanyInformationCommandV1 {
    pub fn encode(&self, actor: AccountId) -> Vec<u8> {
        let payload = match self.input {
            Input::Grant(_) => 527,
            Input::Revoke(_) => 24,
        };
        let mut bytes = Vec::with_capacity(128 + self.reason.len() + payload);
        bytes.extend_from_slice(PREFIX);
        bytes.extend_from_slice(actor.as_uuid().as_bytes());
        bytes.extend_from_slice(self.company.as_uuid().as_bytes());
        bytes.extend_from_slice(self.command_id.as_bytes());
        bytes.extend_from_slice(&self.expected_company_epoch.to_be_bytes());
        bytes.extend_from_slice(&MANIFEST);
        bytes.push(match self.input {
            Input::Grant(_) => 2,
            Input::Revoke(_) => 3,
        });
        bytes.extend_from_slice(&(self.reason.len() as u16).to_be_bytes());
        bytes.extend_from_slice(self.reason.as_bytes());
        match &self.input {
            Input::Grant(grant) => {
                bytes.extend_from_slice(grant.recipient.as_uuid().as_bytes());
                write_assignment(&mut bytes, grant.parent);
                write_time(&mut bytes, grant.valid_from);
                write_time(&mut bytes, grant.valid_until);
                bytes.push(2);
                for action in &grant.actions {
                    bytes.extend_from_slice(action.org_id().as_uuid().as_bytes());
                    bytes.extend_from_slice(action.action_type_id().as_bytes());
                    bytes.extend_from_slice(action.object_type_id().as_bytes());
                    bytes.extend_from_slice(&action.registration_revision().to_be_bytes());
                    bytes.extend_from_slice(action.manifest_digest());
                    bytes.extend_from_slice(self.company.as_uuid().as_bytes());
                    bytes.push(2);
                    for field in &grant.fields {
                        bytes.extend_from_slice(field.org_id().as_uuid().as_bytes());
                        bytes.extend_from_slice(field.object_type_id().as_bytes());
                        bytes.extend_from_slice(field.property_id().as_bytes());
                        bytes.extend_from_slice(&field.schema_revision().to_be_bytes());
                    }
                    write_time(&mut bytes, grant.valid_from);
                    bytes.push(1);
                    write_time(&mut bytes, grant.valid_until);
                    bytes.push(0);
                }
            }
            Input::Revoke(assignment) => write_assignment(&mut bytes, *assignment),
        }
        bytes
    }

    pub fn decode(mut bytes: &[u8]) -> Result<(AccountId, Self), KernelError> {
        if bytes.len() > 2703 || take(&mut bytes, PREFIX.len())? != PREFIX {
            return Err(invalid());
        }
        let actor = AccountId::from_uuid(read_uuid(&mut bytes)?).map_err(|_| invalid())?;
        let company = OrgId::from_uuid(read_uuid(&mut bytes)?);
        let command = read_uuid(&mut bytes)?;
        let epoch = read_revision(&mut bytes)?;
        if take(&mut bytes, 32)? != MANIFEST {
            return Err(invalid());
        }
        let operation = take(&mut bytes, 1)?[0];
        let length =
            u16::from_be_bytes(take(&mut bytes, 2)?.try_into().map_err(|_| invalid())?) as usize;
        if length == 0 || length > 2048 {
            return Err(invalid());
        }
        let reason = std::str::from_utf8(take(&mut bytes, length)?)
            .map_err(|_| invalid())?
            .to_owned();
        let command = match operation {
            2 => {
                let recipient =
                    AccountId::from_uuid(read_uuid(&mut bytes)?).map_err(|_| invalid())?;
                let parent = read_assignment(&mut bytes)?;
                let valid_from = read_time(&mut bytes)?;
                let valid_until = read_time(&mut bytes)?;
                if take(&mut bytes, 1)? != [2] {
                    return Err(invalid());
                }
                let (discover, fields) = read_clause(&mut bytes, company, valid_from, valid_until)?;
                let (identity, identity_fields) =
                    read_clause(&mut bytes, company, valid_from, valid_until)?;
                if fields != identity_fields {
                    return Err(invalid());
                }
                let grant = NativeCompanyInformationGrantV1::new(
                    recipient,
                    parent,
                    valid_from,
                    valid_until,
                    [discover, identity],
                    fields,
                )?;
                Self::grant(command, company, epoch, reason, grant)?
            }
            3 => Self::revoke(
                command,
                company,
                epoch,
                reason,
                read_assignment(&mut bytes)?,
            )?,
            _ => return Err(invalid()),
        };
        if !bytes.is_empty() {
            return Err(invalid());
        }
        Ok((actor, command))
    }
}

fn read_assignment(bytes: &mut &[u8]) -> Result<CompanyInformationAssignmentRefV1, KernelError> {
    CompanyInformationAssignmentRefV1::new(read_uuid(bytes)?, read_revision(bytes)?)
}
fn write_assignment(bytes: &mut Vec<u8>, assignment: CompanyInformationAssignmentRefV1) {
    bytes.extend_from_slice(assignment.assignment_id.as_bytes());
    bytes.extend_from_slice(&assignment.revision.to_be_bytes());
}
fn read_time(bytes: &mut &[u8]) -> Result<OffsetDateTime, KernelError> {
    let micros = i64::from_be_bytes(take(bytes, 8)?.try_into().map_err(|_| invalid())?);
    let value = OffsetDateTime::from_unix_timestamp_nanos(i128::from(micros) * 1_000)
        .map_err(|_| invalid())?;
    validate_time(value)?;
    Ok(value)
}
fn write_time(bytes: &mut Vec<u8>, value: OffsetDateTime) {
    // Validated finite whole microseconds make the signed conversion exact.
    bytes.extend_from_slice(&((value.unix_timestamp_nanos() / 1_000) as i64).to_be_bytes());
}
fn read_clause(
    bytes: &mut &[u8],
    company: OrgId,
    valid_from: OffsetDateTime,
    valid_until: OffsetDateTime,
) -> Result<(ActionRef, [PropertyRef; 2]), KernelError> {
    let org = OrgId::from_uuid(read_uuid(bytes)?);
    let action_type = read_uuid(bytes)?;
    let object_type = read_uuid(bytes)?;
    let revision = read_revision(bytes)?;
    if take(bytes, 32)? != MANIFEST || org != company {
        return Err(invalid());
    }
    let action =
        ActionRef::new(org, action_type, object_type, revision, MANIFEST).map_err(|_| invalid())?;
    if read_uuid(bytes)? != *company.as_uuid() || take(bytes, 1)? != [2] {
        return Err(invalid());
    }
    let fields = [read_property(bytes)?, read_property(bytes)?];
    if read_time(bytes)? != valid_from
        || take(bytes, 1)? != [1]
        || read_time(bytes)? != valid_until
        || take(bytes, 1)? != [0]
    {
        return Err(invalid());
    }
    Ok((action, fields))
}
fn read_property(bytes: &mut &[u8]) -> Result<PropertyRef, KernelError> {
    PropertyRef::new(
        OrgId::from_uuid(read_uuid(bytes)?),
        read_uuid(bytes)?,
        read_uuid(bytes)?,
        read_revision(bytes)?,
    )
    .map_err(|_| invalid())
}
