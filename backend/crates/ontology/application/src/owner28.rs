//! Bounded canonical framing of plain normalized action submissions.
//!
//! These values establish schema/byte validity only. They are not authenticated
//! contexts, current registrations, approved gates, or sealed owner capabilities.
//! The command owner must admit all referenced state separately before execution.
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor},
};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fmt,
    io::{Read, Write},
    sync::OnceLock,
};
use uuid::Uuid;

pub(crate) mod schema;
const TAG: &[u8] = b"console.owner28.submission.v1\0";
const MAX_FRAME: usize = 8_388_608;
const ORDINARY_FRAME: usize = 65_536;
const RELEASE: &str = "payroll.review.release";

/// A fixed, non-disclosing protocol failure. No input bytes enter diagnostics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CodecError(&'static str);
impl fmt::Display for CodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}
impl std::error::Error for CodecError {}
fn require(ok: bool, message: &'static str) -> Result<(), CodecError> {
    if ok { Ok(()) } else { Err(CodecError(message)) }
}

/// Plain structurally validated bytes; never permission or currentness evidence.
///
/// Deserialize bounds accepted material, not allocations made by an arbitrary
/// external deserializer before callbacks. Use [`decode_owner_submission`] for
/// untrusted streams: it caps frame bytes before allocating/parsing the body.
#[derive(Clone, PartialEq, Eq)]
pub struct NormalizedOwnerSubmission(Value);
impl fmt::Debug for NormalizedOwnerSubmission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NormalizedOwnerSubmission")
            .finish_non_exhaustive()
    }
}
impl Serialize for NormalizedOwnerSubmission {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for NormalizedOwnerSubmission {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let mut remaining = MAX_FRAME;
        let value = BoundedValue {
            depth: 1,
            remaining: &mut remaining,
        }
        .deserialize(deserializer)?;
        validate(&value).map_err(de::Error::custom)?;
        Ok(Self(value))
    }
}

/// Domain-separated identity of the exact canonical command, not its frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EncodedSummary {
    pub command_fingerprint: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodedSubmission {
    pub command: NormalizedOwnerSubmission,
}

/// Write the canonical owner frame after validating the whole plain command.
/// # Errors
/// Refuses malformed/bounded input or an incomplete output write.
pub fn encode_owner_submission<W: Write>(
    command: &NormalizedOwnerSubmission,
    sink: &mut W,
) -> Result<EncodedSummary, CodecError> {
    let body = validate(&command.0)?;
    let length = u32::try_from(body.len()).map_err(|_| CodecError("owner frame size"))?;
    sink.write_all(TAG)
        .and_then(|()| sink.write_all(&length.to_be_bytes()))
        .and_then(|()| sink.write_all(&body))
        .map_err(|_| CodecError("owner stream write failed"))?;
    let mut hash = Sha256::new();
    hash.update(b"console.command.ccf1\0");
    hash.update(&body);
    Ok(EncodedSummary {
        command_fingerprint: hex_digest(hash.finalize()),
    })
}

/// Read one bounded frame and require exact canonical bytes and EOF.
/// # Errors
/// Refuses truncation, oversized frames, malformed/noncanonical JSON or trailing bytes.
pub fn decode_owner_submission<R: Read>(source: &mut R) -> Result<DecodedSubmission, CodecError> {
    let mut tag = [0; 30];
    source
        .read_exact(&mut tag)
        .map_err(|_| CodecError("owner stream truncated"))?;
    require(tag == TAG, "owner frame tag")?;
    let mut length = [0; 4];
    source
        .read_exact(&mut length)
        .map_err(|_| CodecError("owner stream truncated"))?;
    let length = u32::from_be_bytes(length) as usize;
    require(length <= MAX_FRAME - TAG.len() - 4, "owner frame size")?;
    let mut bytes = vec![0; length];
    source
        .read_exact(&mut bytes)
        .map_err(|_| CodecError("owner stream truncated"))?;
    let mut extra = [0; 1];
    require(
        source
            .read(&mut extra)
            .map_err(|_| CodecError("owner stream read failed"))?
            == 0,
        "owner trailing bytes",
    )?;
    let command: NormalizedOwnerSubmission =
        serde_json::from_slice(&bytes).map_err(|_| CodecError("invalid owner command"))?;
    require(validate(&command.0)? == bytes, "noncanonical owner command")?;
    Ok(DecodedSubmission { command })
}
fn hex_digest(bytes: impl AsRef<[u8]>) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.as_ref().len() * 2);
    for &byte in bytes.as_ref() {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 15)]));
    }
    output
}
fn digest(tag: &[u8], value: &Value) -> Result<String, CodecError> {
    let mut hash = Sha256::new();
    hash.update(tag);
    hash.update(canonical(value)?);
    Ok(hex_digest(hash.finalize()))
}
pub(crate) fn canonical(value: &Value) -> Result<Vec<u8>, CodecError> {
    // Explicitly sort, independent of serde_json's preserve_order feature union.
    fn sorted(value: &Value) -> Value {
        match value {
            Value::Object(fields) => Value::Object(
                fields
                    .iter()
                    .collect::<BTreeMap<_, _>>()
                    .into_iter()
                    .map(|(key, value)| (key.clone(), sorted(value)))
                    .collect(),
            ),
            Value::Array(values) => Value::Array(values.iter().map(sorted).collect()),
            other => other.clone(),
        }
    }
    serde_json::to_vec(&sorted(value)).map_err(|_| CodecError("owner canonical encoding"))
}
fn uuid(value: &Value) -> bool {
    value.as_str().is_some_and(|text| {
        Uuid::parse_str(text).is_ok_and(|id| !id.is_nil() && id.hyphenated().to_string() == text)
    })
}
fn positive(value: &Value) -> bool {
    value.as_str().is_some_and(|text| {
        text.parse::<i64>()
            .is_ok_and(|n| n > 0 && n.to_string() == text)
    })
}
fn hex(value: &Value) -> bool {
    value.as_str().is_some_and(|text| {
        text.len() == 64
            && text
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    })
}
fn exact(value: &Value, keys: &[&str]) -> bool {
    value
        .as_object()
        .is_some_and(|map| map.len() == keys.len() && keys.iter().all(|key| map.contains_key(*key)))
}
fn validate(command: &Value) -> Result<Vec<u8>, CodecError> {
    require(
        exact(
            command,
            &[
                "protocol",
                "org_id",
                "command_id",
                "actor_account_id",
                "owner_key",
                "owner_action",
                "object_type_id",
                "action_type_id",
                "input_schema_ref",
                "codec_version",
                "target",
                "body",
                "expected",
                "reason",
                "attempt",
                "gate_ref",
                "action_registration_revision",
                "action_registration_manifest_digest",
            ],
        ),
        "owner envelope fields",
    )?;
    require(
        command["protocol"] == "CCF1" && command["codec_version"] == "action30-v1",
        "owner codec dispatch",
    )?;
    let action = command["owner_action"]
        .as_str()
        .ok_or(CodecError("owner action"))?;
    require(
        !action.starts_with("group.")
            && action != "attempt.execute"
            && !schema::SOURCE_ACTIONS.contains(&action),
        "specialized owner codec required",
    )?;
    validate_tree(command, action, &command["org_id"], &mut Vec::new(), 1)?;
    let bytes = canonical(command)?;
    require(
        bytes.len() + TAG.len() + 4
            <= if action == RELEASE {
                MAX_FRAME
            } else {
                ORDINARY_FRAME
            },
        "owner frame size",
    )?;
    let schemas = schema::registered()?;
    let binding = schemas
        .bindings
        .get(action)
        .ok_or(CodecError("unregistered owner action"))?;
    require(
        command["owner_key"] == binding.owner,
        "owner dispatch mismatch",
    )?;
    for key in [
        "org_id",
        "command_id",
        "actor_account_id",
        "object_type_id",
        "action_type_id",
    ] {
        require(uuid(&command[key]), "owner identity")?;
    }
    require(
        positive(&command["action_registration_revision"])
            && hex(&command["action_registration_manifest_digest"]),
        "registration shape",
    )?;
    require(
        schemas.input_schema.is_valid(&command["input_schema_ref"])
            && binding.input.is_valid(&command["body"]),
        "registered input schema",
    )?;
    let target = &command["target"];
    require(
        target["object_kind"] == binding.object_kind,
        "registered target kind",
    )?;
    match target["kind"].as_str() {
        Some("Existing") => require(
            exact(target, &["kind", "object_kind", "object_id"]) && uuid(&target["object_id"]),
            "existing target",
        )?,
        Some("Create") => {
            require(
                exact(
                    target,
                    &["kind", "object_kind", "allocated_object_id", "scope_ref"],
                ) && uuid(&target["allocated_object_id"]),
                "create target",
            )?;
            let scope = &target["scope_ref"];
            require(
                exact(scope, &["org_id", "object_kind", "object_id", "revision"])
                    && uuid(&scope["org_id"])
                    && uuid(&scope["object_id"])
                    && positive(&scope["revision"])
                    && scope["object_kind"].as_str().is_some_and(|s| !s.is_empty()),
                "create scope",
            )?;
        }
        _ => return Err(CodecError("target discriminant")),
    }

    let input_digest = digest(
        b"console.action30.registered-input\0",
        &serde_json::json!({"kind":action,"body":command["body"]}),
    )?;
    require(
        command["expected"]
            == serde_json::json!({"kind":"BODY_EXACT","registered_input_digest":input_digest}),
        "registered input digest",
    )?;
    require(
        command["reason"]
            == command["body"]
                .get("reason")
                .unwrap_or(&Value::Null)
                .clone(),
        "reason mismatch",
    )?;
    let gate = &command["gate_ref"];
    require(
        gate.is_null()
            || (exact(gate, &["request_id", "request_revision"])
                && uuid(&gate["request_id"])
                && gate["request_revision"] == "1"),
        "stable gate shape",
    )?;
    let attempt = &command["attempt"];
    require(
        attempt.is_null()
            || (exact(
                attempt,
                &[
                    "attempt_id",
                    "draft_id",
                    "source_revision_id",
                    "editing_epoch",
                    "assignment_generation",
                ],
            ) && ["attempt_id", "draft_id", "source_revision_id"]
                .iter()
                .all(|key| uuid(&attempt[*key]))
                && positive(&attempt["editing_epoch"])
                && (attempt["assignment_generation"].is_null()
                    || positive(&attempt["assignment_generation"]))),
        "attempt shape",
    )?;
    Ok(bytes)
}
fn validate_tree(
    value: &Value,
    action: &str,
    org: &Value,
    path: &mut Vec<String>,
    depth: usize,
) -> Result<(), CodecError> {
    require(depth <= 8, "owner depth")?;
    match value {
        Value::Object(fields) => {
            require(fields.len() <= 128, "owner field count")?;
            for (key, value) in fields {
                require(key.is_ascii() && !key.contains('\0'), "owner field key")?;
                if key == "org_id" {
                    require(value == org, "cross-company input")?;
                }
                path.push(key.clone());
                validate_tree(
                    value,
                    action,
                    org,
                    path,
                    depth + usize::from(value.is_object() || value.is_array()),
                )?;
                path.pop();
            }
        }
        Value::Array(values) => {
            let release_targets =
                action == RELEASE && path.iter().map(String::as_str).eq(["body", "targets"]);
            require(
                values.len() <= if release_targets { 2000 } else { 256 },
                "owner list count",
            )?;
            for (i, value) in values.iter().enumerate() {
                path.push(i.to_string());
                validate_tree(
                    value,
                    action,
                    org,
                    path,
                    depth + usize::from(value.is_object() || value.is_array()),
                )?;
                path.pop();
            }
        }
        Value::String(text) => require(!text.contains('\0'), "owner NUL")?,
        Value::Null | Value::Bool(_) => {}
        Value::Number(_) => return Err(CodecError("owner JSON number")),
    }
    Ok(())
}

// Parse before schema evaluation, reject duplicate keys before inserting into a
// Value, and bound container depth/cardinality even on malformed public input.
struct BoundedValue<'a> {
    depth: usize,
    remaining: &'a mut usize,
}
impl BoundedValue<'_> {
    fn spend<E: de::Error>(&mut self, amount: usize) -> Result<(), E> {
        *self.remaining = self
            .remaining
            .checked_sub(amount)
            .ok_or_else(|| E::custom("owner parse budget"))?;
        Ok(())
    }
}
impl<'de> DeserializeSeed<'de> for BoundedValue<'_> {
    type Value = Value;
    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Value, D::Error> {
        deserializer.deserialize_any(self)
    }
}
impl<'de> Visitor<'de> for BoundedValue<'_> {
    type Value = Value;
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("bounded nonnumeric owner JSON")
    }
    fn visit_i64<E: de::Error>(self, _: i64) -> Result<Value, E> {
        Err(E::custom("owner JSON number"))
    }
    fn visit_u64<E: de::Error>(self, _: u64) -> Result<Value, E> {
        Err(E::custom("owner JSON number"))
    }
    fn visit_f64<E: de::Error>(self, _: f64) -> Result<Value, E> {
        Err(E::custom("owner JSON number"))
    }
    fn visit_i128<E: de::Error>(self, _: i128) -> Result<Value, E> {
        Err(E::custom("owner JSON number"))
    }
    fn visit_u128<E: de::Error>(self, _: u128) -> Result<Value, E> {
        Err(E::custom("owner JSON number"))
    }
    fn visit_bool<E: de::Error>(mut self, value: bool) -> Result<Value, E> {
        self.spend(4)?;
        Ok(Value::Bool(value))
    }
    fn visit_unit<E: de::Error>(mut self) -> Result<Value, E> {
        self.spend(4)?;
        Ok(Value::Null)
    }
    fn visit_none<E: de::Error>(mut self) -> Result<Value, E> {
        self.spend(4)?;
        Ok(Value::Null)
    }
    fn visit_str<E: de::Error>(mut self, value: &str) -> Result<Value, E> {
        if value.contains('\0') || value.len() > MAX_FRAME {
            return Err(E::custom("owner string"));
        }
        self.spend(value.len().saturating_add(2))?;
        Ok(Value::String(value.to_owned()))
    }
    fn visit_string<E: de::Error>(mut self, value: String) -> Result<Value, E> {
        if value.contains('\0') || value.len() > MAX_FRAME {
            return Err(E::custom("owner string"));
        }
        self.spend(value.len().saturating_add(2))?;
        Ok(Value::String(value))
    }
    fn visit_map<M: MapAccess<'de>>(mut self, mut access: M) -> Result<Value, M::Error> {
        if self.depth > 8 {
            return Err(de::Error::custom("owner depth"));
        }
        self.spend(2)?;
        let mut fields = Map::new();
        while let Some(key) = access.next_key::<String>()? {
            if fields.len() >= 128
                || !key.is_ascii()
                || key.contains('\0')
                || fields.contains_key(&key)
            {
                return Err(de::Error::custom("owner fields"));
            }
            self.spend(key.len().saturating_add(3))?;
            let value = access.next_value_seed(BoundedValue {
                depth: self.depth + 1,
                remaining: &mut *self.remaining,
            })?;
            fields.insert(key, value);
        }
        Ok(Value::Object(fields))
    }
    fn visit_seq<S: SeqAccess<'de>>(mut self, mut access: S) -> Result<Value, S::Error> {
        if self.depth > 8 {
            return Err(de::Error::custom("owner depth"));
        }
        self.spend(2)?;
        let mut values = Vec::new();
        while let Some(value) = access.next_element_seed(BoundedValue {
            depth: self.depth + 1,
            remaining: &mut *self.remaining,
        })? {
            if values.len() >= 2000 {
                return Err(de::Error::custom("owner list count"));
            }
            values.push(value);
        }
        Ok(Value::Array(values))
    }
}

#[cfg(test)]
#[path = "owner28_limits_tests.rs"]
mod limits_tests;

// Shared bounded parsing for plain editor values; existing submission semantics
// stay unchanged. Callers select a smaller budget before any untrusted parsing.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "Pure draft owner awaits its separately admitted integration caller"
    )
)]
pub(crate) fn deserialize_bounded<'de, D: Deserializer<'de>>(
    deserializer: D,
    limit: usize,
) -> Result<Value, D::Error> {
    let mut remaining = limit;
    BoundedValue {
        depth: 1,
        remaining: &mut remaining,
    }
    .deserialize(deserializer)
}
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "Pure draft owner awaits its separately admitted integration caller"
    )
)]
pub(crate) fn parse_bounded(bytes: &[u8], limit: usize) -> Result<Value, CodecError> {
    require(bytes.len() <= limit, "owner input size")?;
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let value = deserialize_bounded(&mut deserializer, limit)
        .map_err(|_| CodecError("invalid bounded owner input"))?;
    deserializer
        .end()
        .map_err(|_| CodecError("owner trailing input"))?;
    Ok(value)
}
