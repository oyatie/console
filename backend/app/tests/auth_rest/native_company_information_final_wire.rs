//! One finite cached-statement/portal grammar for the four retained-read oracles.
#[path = "native_company_information_final_codec.rs"]
pub(super) mod codec;
use crate::native_pg_test_wire::invalid_wire;
use codec::{Column, Cursor, Kind};
use serde_json::Value;
use std::{
    collections::{BTreeMap, VecDeque},
    io::Result,
};
use uuid::Uuid;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Fault {
    Transparent,
    Material,
    SourceZero,
    Group,
    AuthZero,
}
#[derive(Clone)]
struct Statement {
    kind: Kind,
    columns: Vec<Column>,
}
#[derive(Clone)]
struct Portal {
    statement: String,
    formats: Vec<u16>,
    ordinal: usize,
}
struct Run {
    portal: Portal,
    kind: Kind,
    ordinal: usize,
    rows: usize,
    dropped: bool,
}
pub(super) struct Grammar {
    statements: BTreeMap<String, Statement>,
    portals: BTreeMap<String, Portal>,
    describes: VecDeque<String>,
    runs: VecDeque<Run>,
    simple: bool,
    simple_boundary: Option<&'static str>,
    frames: usize,
    ids: [Uuid; 4],
    group: Uuid,
    epoch: u64,
    policy_receipt: Option<Uuid>,
    pub(super) backend_pid: Option<i32>,
    pub(super) fault: Fault,
    pub(super) managers: usize,
    pub(super) auths: usize,
    manager_rows: Vec<Value>,
    auth_rows: Vec<Value>,
    completed: usize,
    error: bool,
    changed: usize,
    pub(super) commits: usize,
    commit_acks: usize,
    rollbacks: usize,
    rollback_acks: usize,
    idle: bool,
}
impl Grammar {
    pub(super) fn new(
        ids: [Uuid; 4],
        group: Uuid,
        epoch: u64,
        policy_receipt: Option<Uuid>,
    ) -> Self {
        Self {
            statements: BTreeMap::new(),
            portals: BTreeMap::new(),
            describes: VecDeque::new(),
            runs: VecDeque::new(),
            simple: false,
            simple_boundary: None,
            frames: 0,
            ids,
            group,
            epoch,
            policy_receipt,
            backend_pid: None,
            fault: Fault::Transparent,
            managers: 0,
            auths: 0,
            manager_rows: Vec::new(),
            auth_rows: Vec::new(),
            completed: 0,
            error: false,
            changed: 0,
            commits: 0,
            commit_acks: 0,
            rollbacks: 0,
            rollback_acks: 0,
            idle: false,
        }
    }
    pub(super) fn arm(&mut self, fault: Fault) -> Result<()> {
        if !self.proven() {
            return Err(invalid_wire());
        }
        self.fault = fault;
        self.managers = 0;
        self.auths = 0;
        self.manager_rows.clear();
        self.auth_rows.clear();
        self.completed = 0;
        self.error = false;
        self.changed = 0;
        self.commits = 0;
        self.commit_acks = 0;
        self.rollbacks = 0;
        self.rollback_acks = 0;
        Ok(())
    }
    fn bound(&mut self) -> Result<()> {
        self.frames += 1;
        if self.frames > 32768
            || self.statements.len() > 64
            || self.portals.len() > 8
            || self.runs.len() > 8
            || self.describes.len() > 8
        {
            return Err(invalid_wire());
        }
        Ok(())
    }
    pub(super) fn upload(&mut self, tag: u8, body: &[u8]) -> Result<Vec<u8>> {
        self.bound()?;
        let mut c = Cursor::new(body);
        match tag {
            b'P' => {
                let name = c.cstr()?;
                let sql = c.cstr()?;
                let kind = codec::kind(sql.as_bytes())?;
                let n = usize::from(c.u16()?);
                if n > 32 {
                    return Err(invalid_wire());
                }
                if kind != Kind::Other && n != if kind == Kind::Manager { 5 } else { 2 } {
                    return Err(invalid_wire());
                }
                for _ in 0..n {
                    let oid = c.u32()?;
                    if kind != Kind::Other && oid != 2950 {
                        return Err(invalid_wire());
                    }
                }
                c.done()?;
                if self.runs.iter().any(|r| r.portal.statement == name)
                    || self.describes.iter().any(|pending| *pending == name)
                    || self.statements.contains_key(&name)
                {
                    return Err(invalid_wire());
                }
                self.statements.insert(
                    name,
                    Statement {
                        kind,
                        columns: Vec::new(),
                    },
                );
            }
            b'B' => {
                let portal = c.cstr()?;
                let statement = c.cstr()?;
                let st = self.statements.get(&statement).ok_or_else(invalid_wire)?;
                // Parameter format count precedes parameter count; decode it independently.
                let fcount = usize::from(c.u16()?);
                if fcount > 32 {
                    return Err(invalid_wire());
                }
                let mut pf = Vec::new();
                for _ in 0..fcount {
                    let f = c.u16()?;
                    if f > 1 {
                        return Err(invalid_wire());
                    }
                    pf.push(f);
                }
                let n = usize::from(c.u16()?);
                if n > 32 || !(fcount == 0 || fcount == 1 || fcount == n) {
                    return Err(invalid_wire());
                }
                let pf = match fcount {
                    0 => vec![0; n],
                    1 => vec![pf[0]; n],
                    _ => pf,
                };
                let parameters_at = c.position();
                let original = codec::cells(&mut c, n)?;
                let result = codec::formats(&mut c, st.columns.len())?;
                c.done()?;
                let ordinal = match st.kind {
                    Kind::Manager => self.managers + 1,
                    Kind::Auth => self.auths + 1,
                    Kind::Other => 0,
                };
                if ordinal > 2 {
                    return Err(invalid_wire());
                }
                if st.kind != Kind::Other {
                    if st.columns.is_empty() {
                        return Err(invalid_wire());
                    }
                    let wanted = if st.kind == Kind::Manager { 5 } else { 2 };
                    if n != wanted {
                        return Err(invalid_wire());
                    }
                    for i in 0..if st.kind == Kind::Manager { 4 } else { 2 } {
                        if codec::uuid(&original[i], pf[i])? != Some(self.ids[i]) {
                            return Err(invalid_wire());
                        }
                    }
                    if st.kind == Kind::Manager {
                        let expected = if self.managers == 0 {
                            None
                        } else if self.managers == 1 {
                            Some(self.group)
                        } else {
                            return Err(invalid_wire());
                        };
                        if codec::uuid(&original[4], pf[4])? != expected {
                            return Err(invalid_wire());
                        }
                        if self.managers == 1 && self.manager_rows.len() != 1 {
                            return Err(invalid_wire());
                        }
                        if self.managers == 1 && self.fault == Fault::Group {
                            let mut bytes = *self.group.as_bytes();
                            bytes[0] ^= 0x80;
                            let replacement = Uuid::from_bytes(bytes);
                            if replacement.is_nil() || replacement == self.group {
                                return Err(invalid_wire());
                            }
                            let changed = codec::uuid_bytes(replacement, pf[4]);
                            let old = original[4].as_ref().ok_or_else(invalid_wire)?;
                            if changed.len() != old.len() {
                                return Err(invalid_wire());
                            }
                            let start = parameters_at
                                + 4
                                + original[..4]
                                    .iter()
                                    .map(|cell| 4 + cell.as_ref().map_or(0, Vec::len))
                                    .sum::<usize>();
                            let end = start + changed.len();
                            let mut out = body.to_vec();
                            if out.get(start..end) != Some(old.as_slice()) {
                                return Err(invalid_wire());
                            }
                            out[start..end].copy_from_slice(&changed);
                            self.portals.insert(
                                portal,
                                Portal {
                                    statement,
                                    formats: result,
                                    ordinal,
                                },
                            );
                            self.changed += 1;
                            return Ok(out);
                        }
                    }
                }
                self.portals.insert(
                    portal,
                    Portal {
                        statement,
                        formats: result,
                        ordinal,
                    },
                );
            }
            b'D' => {
                let kind = c.take(1)?[0];
                let name = c.cstr()?;
                c.done()?;
                let statement = match kind {
                    b'S' => name,
                    b'P' => self
                        .portals
                        .get(&name)
                        .ok_or_else(invalid_wire)?
                        .statement
                        .clone(),
                    _ => return Err(invalid_wire()),
                };
                if !self.statements.contains_key(&statement) {
                    return Err(invalid_wire());
                }
                self.describes.push_back(statement);
            }
            b'E' => {
                let portal = c.cstr()?;
                if c.u32()? != 0 {
                    return Err(invalid_wire());
                }
                c.done()?;
                let portal = self.portals.get(&portal).ok_or_else(invalid_wire)?.clone();
                let kind = self
                    .statements
                    .get(&portal.statement)
                    .ok_or_else(invalid_wire)?
                    .kind;
                let ordinal = match kind {
                    Kind::Manager => {
                        let ordered = match self.managers {
                            0 => {
                                self.auths == 0
                                    && self.manager_rows.is_empty()
                                    && self.auth_rows.is_empty()
                                    && self.completed == 0
                            }
                            1 => {
                                self.auths == 1
                                    && self.manager_rows.len() == 1
                                    && self.auth_rows.len() == 1
                                    && self.completed == 2
                            }
                            _ => false,
                        };
                        if !ordered || !self.runs.is_empty() || self.error {
                            return Err(invalid_wire());
                        }
                        self.managers += 1;
                        self.managers
                    }
                    Kind::Auth => {
                        let ordered = match self.auths {
                            0 => {
                                self.managers == 1
                                    && self.manager_rows.len() == 1
                                    && self.auth_rows.is_empty()
                                    && self.completed == 1
                            }
                            1 => {
                                self.managers == 2
                                    && self.manager_rows.len() == 2
                                    && self.auth_rows.len() == 1
                                    && self.completed == 3
                                    && self.fault != Fault::Group
                            }
                            _ => false,
                        };
                        if !ordered || !self.runs.is_empty() || self.error {
                            return Err(invalid_wire());
                        }
                        self.auths += 1;
                        self.auths
                    }
                    Kind::Other => 0,
                };
                if ordinal > 2 || ordinal != portal.ordinal || self.simple {
                    return Err(invalid_wire());
                }
                self.runs.push_back(Run {
                    portal,
                    kind,
                    ordinal,
                    rows: 0,
                    dropped: false,
                });
                self.idle = false;
            }
            b'Q' => {
                let sql = c.cstr()?;
                c.done()?;
                if codec::kind(sql.as_bytes())? != Kind::Other
                    || !self.runs.is_empty()
                    || self.simple
                {
                    return Err(invalid_wire());
                }
                if sql == "COMMIT" {
                    self.commits += 1;
                    self.simple_boundary = Some("COMMIT");
                } else if sql == "ROLLBACK" {
                    self.rollbacks += 1;
                    self.simple_boundary = Some("ROLLBACK");
                } else {
                    self.simple_boundary = None;
                }
                self.simple = true;
                self.idle = false;
            }
            b'C' => {
                let kind = c.take(1)?[0];
                let name = c.cstr()?;
                c.done()?;
                match kind {
                    b'S' => {
                        if self.runs.iter().any(|r| r.portal.statement == name) {
                            return Err(invalid_wire());
                        }
                        self.statements.remove(&name);
                    }
                    b'P' => {
                        self.portals.remove(&name);
                    }
                    _ => return Err(invalid_wire()),
                };
            }
            b'S' | b'H' | b'X' => {
                c.done()?;
            }
            b'p' => {} // Auth bytes are forwarded without interpretation or retention.
            _ => return Err(invalid_wire()),
        }
        Ok(body.to_vec())
    }
    pub(super) fn download(&mut self, tag: u8, body: &[u8]) -> Result<Option<Vec<u8>>> {
        self.bound()?;
        match tag {
            b'K' => {
                if body.len() != 8 || self.backend_pid.is_some() {
                    return Err(invalid_wire());
                }
                let pid = i32::from_be_bytes(body[..4].try_into().unwrap());
                if pid <= 0 {
                    return Err(invalid_wire());
                }
                self.backend_pid = Some(pid);
            }
            b't' => {
                let name = self.describes.front().ok_or_else(invalid_wire)?;
                let kind = self.statements[name].kind;
                let mut c = Cursor::new(body);
                let n = usize::from(c.u16()?);
                if n > 32 {
                    return Err(invalid_wire());
                }
                if kind != Kind::Other && n != if kind == Kind::Manager { 5 } else { 2 } {
                    return Err(invalid_wire());
                }
                for _ in 0..n {
                    let oid = c.u32()?;
                    if kind != Kind::Other && oid != 2950 {
                        return Err(invalid_wire());
                    }
                }
                c.done()?;
            }
            b'T' => {
                if let Some(name) = self.describes.pop_front() {
                    let st = self.statements.get_mut(&name).ok_or_else(invalid_wire)?;
                    st.columns = codec::schema(body, st.kind)?;
                } else if !self.simple {
                    return Err(invalid_wire());
                }
            }
            b'n' => {
                if !body.is_empty() {
                    return Err(invalid_wire());
                }
                if let Some(name) = self.describes.pop_front() {
                    if self.statements[&name].kind != Kind::Other {
                        return Err(invalid_wire());
                    }
                } else {
                    return Err(invalid_wire());
                }
            }
            b'D' => {
                if let Some(run) = self.runs.front_mut() {
                    run.rows += 1;
                    if run.kind != Kind::Other {
                        if run.rows != 1 {
                            return Err(invalid_wire());
                        }
                        let st = &self.statements[&run.portal.statement];
                        let mut cells = codec::row(body)?;
                        let row = codec::values(&cells, &st.columns, &run.portal.formats)?;
                        if run.kind == Kind::Manager {
                            for (key, id) in
                                ["actor_account_id", "session_id", "org_id", "command_id"]
                                    .into_iter()
                                    .zip(self.ids)
                            {
                                if row[key] != serde_json::json!(id) {
                                    return Err(invalid_wire());
                                }
                            }
                            if cells.iter().enumerate().any(|(i, c)| i != 6 && c.is_none())
                                || row["company_epoch"] != serde_json::json!(self.epoch)
                                || row["current_policy_receipt_id"]
                                    != serde_json::json!(self.policy_receipt)
                                || self.epoch == 0
                                || (self.epoch == 1) != self.policy_receipt.is_none()
                                || row["source_xid"].as_u64().is_none_or(|id| id == 0)
                                || row["observed_at"].as_i64().is_none()
                                || row["current_group_id"] != serde_json::json!(self.group)
                                || row["source_backend_pid"].as_i64()
                                    != self.backend_pid.map(i64::from)
                            {
                                return Err(invalid_wire());
                            }
                            if row["source_material"]["request"]
                                != serde_json::json!({"codec_version":4,"operation":"Grant","account_id":self.ids[0],"session_id":self.ids[1],"org_id":self.ids[2],"command_id":self.ids[3]})
                                || row["source_material"]["family"]["id"]
                                    != serde_json::json!(self.ids[1])
                                || row["source_material"]["account"]["id"]
                                    != serde_json::json!(self.ids[0])
                                || row["source_material"]["company_head"]["org_id"]
                                    != serde_json::json!(self.ids[2])
                                || row["source_material"]["company_head"]["epoch"]
                                    != row["company_epoch"]
                                || row["source_material"]["company_head"]
                                    .get("current_policy_receipt_id")
                                    .is_none()
                                || row["source_material"]["company_head"]["current_policy_receipt_id"]
                                    != row["current_policy_receipt_id"]
                            {
                                return Err(invalid_wire());
                            }
                            if let Some(first) = self.manager_rows.first() {
                                let mut a = first.clone();
                                let mut b = row.clone();
                                let at = a
                                    .as_object_mut()
                                    .unwrap()
                                    .remove("observed_at")
                                    .ok_or_else(invalid_wire)?;
                                let bt = b
                                    .as_object_mut()
                                    .unwrap()
                                    .remove("observed_at")
                                    .ok_or_else(invalid_wire)?;
                                if a != b || bt.as_i64() < at.as_i64() {
                                    return Err(invalid_wire());
                                }
                            }
                            self.manager_rows.push(row.clone());
                            if run.ordinal == 2 && self.fault == Fault::Material {
                                codec::material_drift(&mut cells, &run.portal.formats, &row)?;
                                self.changed += 1;
                                return Ok(Some(codec::write_cells(&cells)));
                            }
                        } else {
                            if cells
                                .iter()
                                .enumerate()
                                .any(|(i, c)| i != 10 && i != 11 && c.is_none())
                                || [
                                    "security_generation",
                                    "revision",
                                    "context_generation",
                                    "account_security_generation",
                                ]
                                .iter()
                                .any(|key| row[*key].as_u64().is_none_or(|v| v == 0))
                                || row["user_id"] != serde_json::json!(self.ids[0])
                                || row["security_state"] != "ACTIVE"
                                || row["protocol"] != "ACCOUNT_V1"
                                || row["assurance"] != "PASSKEY_PRIMARY"
                                || !row["revoked_at"].is_null()
                                || !row["org_id"].is_null()
                                || self.auth_rows.first().is_some_and(|first| *first != row)
                            {
                                return Err(invalid_wire());
                            }
                            self.auth_rows.push(row);
                        }
                        if run.ordinal == 2
                            && ((run.kind == Kind::Manager && self.fault == Fault::SourceZero)
                                || (run.kind == Kind::Auth && self.fault == Fault::AuthZero))
                        {
                            run.dropped = true;
                            self.changed += 1;
                            return Ok(None);
                        }
                    }
                } else if !self.simple {
                    return Err(invalid_wire());
                }
            }
            b'C' => {
                if let Some(run) = self.runs.pop_front() {
                    if run.kind != Kind::Other {
                        if run.rows != 1 || body != b"SELECT 1\0" {
                            return Err(invalid_wire());
                        }
                        self.completed += 1;
                        if run.dropped {
                            return Ok(Some(b"SELECT 0\0".to_vec()));
                        }
                    }
                } else if !self.simple {
                    return Err(invalid_wire());
                } else if let Some(boundary) = self.simple_boundary {
                    let expected: &[u8] = if boundary == "COMMIT" {
                        b"COMMIT\0"
                    } else {
                        b"ROLLBACK\0"
                    };
                    if body != expected {
                        return Err(invalid_wire());
                    }
                    if boundary == "COMMIT" {
                        self.commit_acks += 1;
                        if self.commit_acks != self.commits {
                            return Err(invalid_wire());
                        }
                    } else {
                        self.rollback_acks += 1;
                        if self.rollback_acks != self.rollbacks {
                            return Err(invalid_wire());
                        }
                    }
                }
            }
            b'E' => {
                let run = self.runs.pop_front().ok_or_else(invalid_wire)?;
                if self.fault != Fault::Group
                    || run.kind != Kind::Manager
                    || run.ordinal != 2
                    || run.rows != 0
                    || self.error
                {
                    return Err(invalid_wire());
                }
                let mut c = Cursor::new(body);
                let mut fields = BTreeMap::new();
                loop {
                    let k = c.take(1)?[0];
                    if k == 0 {
                        break;
                    }
                    let v = c.cstr()?;
                    if fields.insert(k, v).is_some() {
                        return Err(invalid_wire());
                    }
                }
                c.done()?;
                if fields.get(&b'C').map(String::as_str) != Some("40001")
                    || fields.get(&b'M').map(String::as_str)
                        != Some("company_information.lock_plan_changed")
                {
                    return Err(invalid_wire());
                }
                self.error = true;
            }
            b'Z' => {
                if body.len() != 1
                    || !matches!(body[0], b'I' | b'T' | b'E')
                    || !self.runs.is_empty()
                    || !self.describes.is_empty()
                    || (self.simple_boundary == Some("COMMIT") && self.commit_acks != self.commits)
                    || (self.simple_boundary == Some("ROLLBACK")
                        && self.rollback_acks != self.rollbacks)
                {
                    return Err(invalid_wire());
                }
                self.simple = false;
                self.simple_boundary = None;
                self.idle = body == b"I";
            }
            b'1' | b'2' | b'3' => {
                if !body.is_empty() {
                    return Err(invalid_wire());
                }
            }
            b'R' | b'S' | b'N' => {}
            _ => return Err(invalid_wire()),
        }
        Ok(Some(body.to_vec()))
    }
    pub(super) fn proven(&self) -> bool {
        let (rows, auths, completed) = if self.fault == Fault::Group {
            (1, 1, 2)
        } else {
            (2, 2, 4)
        };
        self.managers == 2
            && self.manager_rows.len() == rows
            && self.auths == auths
            && self.auth_rows.len() == auths
            && (1..=2).contains(&auths)
            && self.completed == completed
            && self.runs.is_empty()
            && self.portals.is_empty()
            && self.describes.is_empty()
            && !self.simple
            && self.idle
            && self.changed == usize::from(self.fault != Fault::Transparent)
            && self.error == (self.fault == Fault::Group)
            && self.commits == usize::from(self.fault == Fault::Transparent)
            && self.commit_acks == self.commits
            && self.rollbacks == usize::from(self.fault != Fault::Transparent)
            && self.rollback_acks == self.rollbacks
    }
}
