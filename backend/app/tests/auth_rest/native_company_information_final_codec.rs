//! Finite PostgreSQL codecs; raw values stay in memory and never implement Debug.
use crate::native_pg_test_wire::invalid_wire;
use serde_json::{Value, json};
use std::io::Result;
use time::{Duration, OffsetDateTime, format_description::well_known::Rfc3339};
use uuid::Uuid;

pub(super) const MANAGER: &str = "SELECT actor_account_id, session_id, org_id, command_id, current_group_id, company_epoch, current_policy_receipt_id, context_generation, assignment_id, assignment_revision, role_id, role_revision, registered_clauses, company_name, company_slug, installed_object_type_id, observed_at, source_xid::text AS source_xid, source_backend_pid, source_material FROM public.identity_company_information_manager_current_v1($1,$2,$3,$4,$5) LIMIT 2";
pub(super) const AUTH: &str = "SELECT * FROM public.account_session_shared_material_v1($1,$2)";
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    Other,
    Manager,
    Auth,
}
pub(super) fn kind(sql: &[u8]) -> Result<Kind> {
    let text = std::str::from_utf8(sql).map_err(|_| invalid_wire())?;
    if text.len() > 16384 {
        return Err(invalid_wire());
    }
    let normalized = text.split_ascii_whitespace().collect::<Vec<_>>().join(" ");
    if normalized == MANAGER {
        return Ok(Kind::Manager);
    }
    if normalized == AUTH {
        return Ok(Kind::Auth);
    }
    let folded = text.to_ascii_lowercase();
    if folded.contains("identity_company_information_")
        || folded.contains("account_session_shared_material_v1")
    {
        return Err(invalid_wire());
    }
    Ok(Kind::Other)
}
pub(super) struct Cursor<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl<'a> Cursor<'a> {
    pub(super) fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, at: 0 }
    }
    pub(super) fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self.at.checked_add(n).ok_or_else(invalid_wire)?;
        let result = self.bytes.get(self.at..end).ok_or_else(invalid_wire)?;
        self.at = end;
        Ok(result)
    }
    pub(super) fn u16(&mut self) -> Result<u16> {
        Ok(u16::from_be_bytes(self.take(2)?.try_into().unwrap()))
    }
    pub(super) fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_be_bytes(self.take(4)?.try_into().unwrap()))
    }
    pub(super) fn cstr(&mut self) -> Result<String> {
        let n = self.bytes[self.at..]
            .iter()
            .position(|b| *b == 0)
            .ok_or_else(invalid_wire)?;
        if n > 16384 {
            return Err(invalid_wire());
        }
        let text = std::str::from_utf8(self.take(n)?)
            .map_err(|_| invalid_wire())?
            .to_owned();
        self.take(1)?;
        Ok(text)
    }
    pub(super) fn done(&self) -> Result<()> {
        if self.at == self.bytes.len() {
            Ok(())
        } else {
            Err(invalid_wire())
        }
    }
    pub(super) fn position(&self) -> usize {
        self.at
    }
}
pub(super) fn formats(c: &mut Cursor<'_>, count: usize) -> Result<Vec<u16>> {
    let n = usize::from(c.u16()?);
    if n > 64 || !(n == 0 || n == 1 || n == count) {
        return Err(invalid_wire());
    }
    let mut raw = Vec::new();
    for _ in 0..n {
        let f = c.u16()?;
        if f > 1 {
            return Err(invalid_wire());
        }
        raw.push(f);
    }
    Ok(match n {
        0 => vec![0; count],
        1 => vec![raw[0]; count],
        _ => raw,
    })
}
pub(super) type Cells = Vec<Option<Vec<u8>>>;
pub(super) fn cells(c: &mut Cursor<'_>, count: usize) -> Result<Cells> {
    if count > 64 {
        return Err(invalid_wire());
    }
    let mut result = Vec::new();
    for _ in 0..count {
        let n = c.u32()?;
        result.push(if n == u32::MAX {
            None
        } else {
            if n > 1024 * 1024 {
                return Err(invalid_wire());
            }
            Some(c.take(n as usize)?.to_vec())
        });
    }
    Ok(result)
}
pub(super) fn row(bytes: &[u8]) -> Result<Cells> {
    let mut c = Cursor::new(bytes);
    let n = usize::from(c.u16()?);
    let result = cells(&mut c, n)?;
    c.done()?;
    Ok(result)
}
pub(super) fn write_cells(cells: &Cells) -> Vec<u8> {
    let mut bytes = (cells.len() as u16).to_be_bytes().to_vec();
    for cell in cells {
        match cell {
            None => bytes.extend(u32::MAX.to_be_bytes()),
            Some(raw) => {
                bytes.extend((raw.len() as u32).to_be_bytes());
                bytes.extend(raw);
            }
        }
    }
    bytes
}
pub(super) fn uuid(raw: &Option<Vec<u8>>, format: u16) -> Result<Option<Uuid>> {
    let Some(raw) = raw else {
        return Ok(None);
    };
    let id = if format == 1 {
        Uuid::from_slice(raw).map_err(|_| invalid_wire())?
    } else {
        Uuid::parse_str(std::str::from_utf8(raw).map_err(|_| invalid_wire())?)
            .map_err(|_| invalid_wire())?
    };
    if id.is_nil() {
        return Err(invalid_wire());
    }
    Ok(Some(id))
}
pub(super) fn uuid_bytes(id: Uuid, format: u16) -> Vec<u8> {
    if format == 1 {
        id.as_bytes().to_vec()
    } else {
        id.to_string().into_bytes()
    }
}
#[derive(Clone)]
pub(super) struct Column {
    pub(super) name: String,
    pub(super) oid: u32,
}
pub(super) const MANAGER_SCHEMA: &[(&str, u32)] = &[
    ("actor_account_id", 2950),
    ("session_id", 2950),
    ("org_id", 2950),
    ("command_id", 2950),
    ("current_group_id", 2950),
    ("company_epoch", 20),
    ("current_policy_receipt_id", 2950),
    ("context_generation", 20),
    ("assignment_id", 2950),
    ("assignment_revision", 20),
    ("role_id", 2950),
    ("role_revision", 20),
    ("registered_clauses", 3802),
    ("company_name", 25),
    ("company_slug", 25),
    ("installed_object_type_id", 2950),
    ("observed_at", 1184),
    ("source_xid", 25),
    ("source_backend_pid", 23),
    ("source_material", 3802),
];
pub(super) const AUTH_SCHEMA: &[(&str, u32)] = &[
    ("security_state", 25),
    ("security_generation", 20),
    ("revision", 20),
    ("context_generation", 20),
    ("user_id", 2950),
    ("protocol", 25),
    ("account_security_generation", 20),
    ("auth_time", 1184),
    ("assurance", 25),
    ("created_at", 1184),
    ("revoked_at", 1184),
    ("org_id", 2950),
];
pub(super) fn schema(bytes: &[u8], kind: Kind) -> Result<Vec<Column>> {
    let mut c = Cursor::new(bytes);
    let n = usize::from(c.u16()?);
    if n > 64 {
        return Err(invalid_wire());
    }
    let mut result = Vec::new();
    for _ in 0..n {
        let name = c.cstr()?;
        c.take(6)?;
        let oid = c.u32()?;
        c.take(6)?;
        if c.u16()? > 1 {
            return Err(invalid_wire());
        }
        result.push(Column { name, oid });
    }
    c.done()?;
    let expected = match kind {
        Kind::Manager => MANAGER_SCHEMA,
        Kind::Auth => AUTH_SCHEMA,
        Kind::Other => return Ok(result),
    };
    if result.len() != expected.len()
        || result
            .iter()
            .zip(expected)
            .any(|(a, b)| a.name != b.0 || a.oid != b.1)
    {
        return Err(invalid_wire());
    }
    Ok(result)
}
pub(super) fn time(text: &str) -> Result<OffsetDateTime> {
    let mut text = text.replacen(' ', "T", 1);
    if text.ends_with("+00") {
        text.push_str(":00");
    }
    OffsetDateTime::parse(&text, &Rfc3339).map_err(|_| invalid_wire())
}
fn value(raw: &Option<Vec<u8>>, oid: u32, format: u16) -> Result<Value> {
    let Some(raw) = raw else {
        return Ok(Value::Null);
    };
    if oid == 2950 {
        return Ok(json!(
            uuid(&Some(raw.clone()), format)?.ok_or_else(invalid_wire)?
        ));
    }
    if oid == 3802 {
        let raw = if format == 1 {
            if raw.first() != Some(&1) {
                return Err(invalid_wire());
            }
            &raw[1..]
        } else {
            raw
        };
        return serde_json::from_slice(raw).map_err(|_| invalid_wire());
    }
    if oid == 25 {
        return Ok(json!(std::str::from_utf8(raw).map_err(|_| invalid_wire())?));
    }
    if oid == 1184 {
        let at = if format == 1 {
            let us = i64::from_be_bytes(raw.as_slice().try_into().map_err(|_| invalid_wire())?);
            if us == i64::MIN || us == i64::MAX {
                return Err(invalid_wire());
            }
            OffsetDateTime::from_unix_timestamp(946684800)
                .unwrap()
                .checked_add(Duration::microseconds(us))
                .ok_or_else(invalid_wire)?
        } else {
            time(std::str::from_utf8(raw).map_err(|_| invalid_wire())?)?
        };
        return Ok(json!(
            i64::try_from(at.unix_timestamp_nanos() / 1000).map_err(|_| invalid_wire())?
        ));
    }
    let n: i64 = if format == 1 {
        match oid {
            20 => i64::from_be_bytes(raw.as_slice().try_into().map_err(|_| invalid_wire())?),
            23 => i64::from(i32::from_be_bytes(
                raw.as_slice().try_into().map_err(|_| invalid_wire())?,
            )),
            _ => return Err(invalid_wire()),
        }
    } else {
        std::str::from_utf8(raw)
            .map_err(|_| invalid_wire())?
            .parse()
            .map_err(|_| invalid_wire())?
    };
    Ok(json!(n))
}
pub(super) fn values(cells: &Cells, schema: &[Column], formats: &[u16]) -> Result<Value> {
    if cells.len() != schema.len()
        || formats.len() != schema.len()
        || formats.iter().any(|f| *f > 1)
    {
        return Err(invalid_wire());
    }
    let mut result = serde_json::Map::new();
    for ((cell, column), format) in cells.iter().zip(schema).zip(formats) {
        let value = if column.name == "source_xid" {
            if column.oid != 25 || *format > 1 {
                return Err(invalid_wire());
            }
            let raw = cell.as_ref().ok_or_else(invalid_wire)?;
            let text = std::str::from_utf8(raw).map_err(|_| invalid_wire())?;
            if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
                return Err(invalid_wire());
            }
            let xid: u64 = text.parse().map_err(|_| invalid_wire())?;
            if xid == 0 {
                return Err(invalid_wire());
            }
            json!(xid)
        } else {
            value(cell, column.oid, *format)?
        };
        if result.insert(column.name.clone(), value).is_some() {
            return Err(invalid_wire());
        }
    }
    Ok(Value::Object(result))
}
pub(super) fn material_drift(cells: &mut Cells, formats: &[u16], original: &Value) -> Result<()> {
    let mut material = original["source_material"].clone();
    let before = material["company"]["updated_at"]
        .as_str()
        .ok_or_else(invalid_wire)?;
    let created = time(
        material["company"]["created_at"]
            .as_str()
            .ok_or_else(invalid_wire)?,
    )?;
    let changed = time(before)?
        .checked_add(Duration::microseconds(1))
        .ok_or_else(invalid_wire)?;
    let observed = original["observed_at"].as_i64().ok_or_else(invalid_wire)?;
    if changed < created || changed.unix_timestamp_nanos() / 1000 >= i128::from(observed) {
        return Err(invalid_wire());
    }
    material["company"]["updated_at"] =
        json!(changed.format(&Rfc3339).map_err(|_| invalid_wire())?);
    let mut restored = material.clone();
    restored["company"]["updated_at"] =
        original["source_material"]["company"]["updated_at"].clone();
    if restored != original["source_material"] || material == restored {
        return Err(invalid_wire());
    }
    let mut raw = serde_json::to_vec(&material).map_err(|_| invalid_wire())?;
    if formats[19] == 1 {
        raw.insert(0, 1);
    }
    cells[19] = Some(raw);
    Ok(())
}
