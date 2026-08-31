//! Generate OpenAPI schema YAML from the canonical semantic manifest.
//!
//! ADR-0031: the contracts crate is the internal contract; composed
//! `openapi.yaml` is a deliverable. This module is the first slice of that
//! compiler: the thirteen DispatchTarget Input schemas, two nested write bags,
//! and the six PRODUCT Head schemas (links + actions injected) are emitted
//! from [`SEMANTIC_MANIFEST`] and merged by [`crate::compose_document_with_owned`].
//! Face YAML must not also own those names.
//!
//! The same manifest emits the serde codecs included by ontology REST
//! `typed_action.rs` ([`generated_typed_action_rs`]). Dual-maintained Input
//! structs are a second contract.

use crate::OwnedNamedYaml;
use std::fmt;

/// Crate source name used on duplicate/dangling errors for generated schemas.
pub const SEMANTIC_SOURCE: &str = "console-contracts-semantic";

/// 13 Inputs + EmploymentAttributesInput + OrgUnitSourceBinding + 6 Heads.
pub const GENERATED_SCHEMA_COUNT: usize = 21;

/// 13 Inputs + EmploymentAttributesInput + OrgUnitSourceBinding (execute codecs).
pub const CODEC_SCHEMA_COUNT: usize = 15;

/// Thirteen DispatchTarget action keys.
pub const DISPATCH_TARGET_COUNT: usize = 13;

const SEMANTIC_MANIFEST: &str = include_str!("semantic_manifest.json");
const RESULT_REF: &str = "#/components/schemas/OntologyActionExecuteOutcome";

#[derive(Clone, Debug)]
enum Json {
    Null,
    Bool(bool),
    Number(String),
    String(String),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
}

impl Json {
    fn as_object(&self) -> Option<&[(String, Json)]> {
        match self {
            Json::Object(fields) => Some(fields),
            _ => None,
        }
    }

    fn as_array(&self) -> Option<&[Json]> {
        match self {
            Json::Array(items) => Some(items),
            _ => None,
        }
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            Json::String(text) => Some(text),
            _ => None,
        }
    }

    fn get(&self, key: &str) -> Option<&Json> {
        self.as_object()?
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value)
    }
}

/// Failure to parse the committed manifest or to emit a required schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticError(String);

impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for SemanticError {}

/// Emit owned schema bodies for compose, in stable name order.
///
/// # Errors
///
/// Returns when the committed JSON is not the expected object, when a required
/// Head/Input body is missing, or when generation would drop below
/// [`GENERATED_SCHEMA_COUNT`].
pub fn generated_schema_yaml() -> Result<Vec<OwnedNamedYaml>, SemanticError> {
    let root = parse_json(SEMANTIC_MANIFEST)?;
    let schemas = root
        .get("schemas")
        .and_then(Json::as_object)
        .ok_or_else(|| SemanticError("semantic manifest schemas must be an object".to_owned()))?;
    let objects = root
        .get("objects")
        .and_then(Json::as_array)
        .ok_or_else(|| SemanticError("semantic manifest objects must be an array".to_owned()))?;
    let links = root
        .get("links")
        .and_then(Json::as_array)
        .ok_or_else(|| SemanticError("semantic manifest links must be an array".to_owned()))?;
    let actions = root
        .get("actions")
        .and_then(Json::as_array)
        .ok_or_else(|| SemanticError("semantic manifest actions must be an array".to_owned()))?;

    let head_names: Vec<String> = objects
        .iter()
        .filter_map(|object| object.get("name").and_then(Json::as_str).map(str::to_owned))
        .collect();

    let mut generated: Vec<OwnedNamedYaml> = Vec::new();
    for (name, schema) in schemas {
        let body_json = if head_names.iter().any(|head| head == name) {
            inject_head_contract(schema, name, objects, links, actions)?
        } else {
            schema.clone()
        };
        let mut body = String::new();
        emit_yaml(&body_json, 0, &mut body);
        if !body.ends_with('\n') {
            body.push('\n');
        }
        generated.push(OwnedNamedYaml {
            name: name.clone(),
            body,
            source: SEMANTIC_SOURCE,
        });
    }
    generated.sort_by(|left, right| left.name.cmp(&right.name));
    if generated.len() != GENERATED_SCHEMA_COUNT {
        return Err(SemanticError(format!(
            "semantic manifest generated {} schemas, expected {GENERATED_SCHEMA_COUNT}",
            generated.len()
        )));
    }
    Ok(generated)
}

fn inject_head_contract(
    schema: &Json,
    name: &str,
    objects: &[Json],
    links: &[Json],
    actions: &[Json],
) -> Result<Json, SemanticError> {
    let Json::Object(fields) = schema else {
        return Err(SemanticError(format!(
            "head schema {name} must be a JSON object"
        )));
    };
    let mut out: Vec<(String, Json)> = fields
        .iter()
        .filter(|(key, _)| key != "links" && key != "actions")
        .cloned()
        .collect();

    let declared_links: Vec<Json> = links
        .iter()
        .filter(|link| link.get("from").and_then(Json::as_str) == Some(name))
        .cloned()
        .collect();
    out.push(("links".to_owned(), Json::Array(declared_links)));

    let object = objects
        .iter()
        .find(|item| item.get("name").and_then(Json::as_str) == Some(name))
        .ok_or_else(|| SemanticError(format!("no object entry for head {name}")))?;
    let action_keys = object
        .get("actions")
        .and_then(Json::as_array)
        .ok_or_else(|| SemanticError(format!("object {name} actions must be an array")))?;
    let mut declared_actions = Vec::new();
    for key_json in action_keys {
        let key = key_json
            .as_str()
            .ok_or_else(|| SemanticError(format!("object {name} action key must be a string")))?;
        let action = actions
            .iter()
            .find(|item| item.get("action_key").and_then(Json::as_str) == Some(key))
            .ok_or_else(|| SemanticError(format!("missing action {key} for head {name}")))?;
        declared_actions.push(action_contract(action)?);
    }
    out.push(("actions".to_owned(), Json::Array(declared_actions)));
    Ok(Json::Object(out))
}

fn action_contract(action: &Json) -> Result<Json, SemanticError> {
    let action_key = action
        .get("action_key")
        .and_then(Json::as_str)
        .ok_or_else(|| SemanticError("action_key must be a string".to_owned()))?;
    let object_key = action
        .get("object_key")
        .and_then(Json::as_str)
        .ok_or_else(|| SemanticError(format!("{action_key} object_key must be a string")))?;
    let input = action
        .get("input")
        .and_then(Json::as_str)
        .ok_or_else(|| SemanticError(format!("{action_key} input must be a string")))?;
    let four_eyes = action
        .get("four_eyes")
        .and_then(Json::as_str)
        .ok_or_else(|| SemanticError(format!("{action_key} four_eyes must be a string")))?;
    let edits = action
        .get("edits")
        .cloned()
        .ok_or_else(|| SemanticError(format!("{action_key} edits must be an array")))?;
    let permissions = action
        .get("permissions")
        .cloned()
        .unwrap_or_else(|| Json::Array(vec![Json::String("role_manage".to_owned())]));
    let concurrency = action
        .get("concurrency")
        .cloned()
        .ok_or_else(|| SemanticError(format!("{action_key} concurrency must be an object")))?;
    Ok(Json::Object(vec![
        ("action_key".to_owned(), Json::String(action_key.to_owned())),
        ("object_key".to_owned(), Json::String(object_key.to_owned())),
        (
            "input".to_owned(),
            Json::Object(vec![(
                "$ref".to_owned(),
                Json::String(format!("#/components/schemas/{input}")),
            )]),
        ),
        (
            "result".to_owned(),
            Json::Object(vec![(
                "$ref".to_owned(),
                Json::String(RESULT_REF.to_owned()),
            )]),
        ),
        ("permissions".to_owned(), permissions),
        ("four_eyes".to_owned(), Json::String(four_eyes.to_owned())),
        ("edits".to_owned(), edits),
        ("concurrency".to_owned(), concurrency),
    ]))
}

// ---------------------------------------------------------------------------
// JSON parser (zero-dep; the contracts crate has no serde_json)
// ---------------------------------------------------------------------------

struct Cursor<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            bytes: input.as_bytes(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<u8> {
        let byte = self.peek()?;
        self.pos += 1;
        Some(byte)
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.pos += 1;
        }
    }

    fn err(&self, msg: &str) -> SemanticError {
        SemanticError(format!(
            "semantic manifest JSON: {msg} at byte {}",
            self.pos
        ))
    }
}

fn parse_json(input: &str) -> Result<Json, SemanticError> {
    let mut cur = Cursor::new(input);
    cur.skip_ws();
    let value = parse_value(&mut cur)?;
    cur.skip_ws();
    if cur.pos != cur.bytes.len() {
        return Err(cur.err("trailing bytes after the top-level value"));
    }
    Ok(value)
}

fn parse_value(cur: &mut Cursor<'_>) -> Result<Json, SemanticError> {
    cur.skip_ws();
    match cur.peek() {
        Some(b'{') => parse_object(cur),
        Some(b'[') => parse_array(cur),
        Some(b'"') => parse_string(cur).map(Json::String),
        Some(b't') => parse_ident(cur, b"true", Json::Bool(true)),
        Some(b'f') => parse_ident(cur, b"false", Json::Bool(false)),
        Some(b'n') => parse_ident(cur, b"null", Json::Null),
        Some(b'-' | b'0'..=b'9') => parse_number(cur),
        Some(byte) => Err(cur.err(&format!("unexpected 0x{byte:02x}"))),
        None => Err(cur.err("unexpected end")),
    }
}

fn parse_ident(cur: &mut Cursor<'_>, token: &[u8], value: Json) -> Result<Json, SemanticError> {
    for expected in token {
        match cur.bump() {
            Some(byte) if byte == *expected => {}
            _ => return Err(cur.err("invalid literal")),
        }
    }
    Ok(value)
}

fn parse_number(cur: &mut Cursor<'_>) -> Result<Json, SemanticError> {
    let start = cur.pos;
    if cur.peek() == Some(b'-') {
        cur.pos += 1;
    }
    if cur.peek() == Some(b'0') {
        cur.pos += 1;
    } else if matches!(cur.peek(), Some(b'1'..=b'9')) {
        while matches!(cur.peek(), Some(b'0'..=b'9')) {
            cur.pos += 1;
        }
    } else {
        return Err(cur.err("invalid number"));
    }
    if cur.peek() == Some(b'.') {
        cur.pos += 1;
        if !matches!(cur.peek(), Some(b'0'..=b'9')) {
            return Err(cur.err("invalid number"));
        }
        while matches!(cur.peek(), Some(b'0'..=b'9')) {
            cur.pos += 1;
        }
    }
    if matches!(cur.peek(), Some(b'e' | b'E')) {
        cur.pos += 1;
        if matches!(cur.peek(), Some(b'+' | b'-')) {
            cur.pos += 1;
        }
        if !matches!(cur.peek(), Some(b'0'..=b'9')) {
            return Err(cur.err("invalid number"));
        }
        while matches!(cur.peek(), Some(b'0'..=b'9')) {
            cur.pos += 1;
        }
    }
    let raw = std::str::from_utf8(&cur.bytes[start..cur.pos])
        .map_err(|_| cur.err("number is not utf-8"))?;
    Ok(Json::Number(raw.to_owned()))
}

fn parse_string(cur: &mut Cursor<'_>) -> Result<String, SemanticError> {
    if cur.bump() != Some(b'"') {
        return Err(cur.err("expected string"));
    }
    let mut out = String::new();
    loop {
        match cur.bump() {
            Some(b'"') => return Ok(out),
            Some(b'\\') => match cur.bump() {
                Some(b'"') => out.push('"'),
                Some(b'\\') => out.push('\\'),
                Some(b'/') => out.push('/'),
                Some(b'b') => out.push('\u{0008}'),
                Some(b'f') => out.push('\u{000c}'),
                Some(b'n') => out.push('\n'),
                Some(b'r') => out.push('\r'),
                Some(b't') => out.push('\t'),
                Some(b'u') => {
                    let mut hex = 0u32;
                    for _ in 0..4 {
                        let byte = cur.bump().ok_or_else(|| cur.err("truncated \\u escape"))?;
                        hex <<= 4;
                        hex |= match byte {
                            b'0'..=b'9' => u32::from(byte - b'0'),
                            b'a'..=b'f' => u32::from(byte - b'a') + 10,
                            b'A'..=b'F' => u32::from(byte - b'A') + 10,
                            _ => return Err(cur.err("invalid \\u escape")),
                        };
                    }
                    let ch =
                        char::from_u32(hex).ok_or_else(|| cur.err("invalid unicode scalar"))?;
                    out.push(ch);
                }
                _ => return Err(cur.err("invalid escape")),
            },
            Some(byte) if byte < 0x20 => return Err(cur.err("unescaped control in string")),
            Some(_byte) => {
                // Restart at this byte so a multi-byte UTF-8 sequence is total.
                cur.pos -= 1;
                let rest = std::str::from_utf8(&cur.bytes[cur.pos..])
                    .map_err(|_| cur.err("string is not utf-8"))?;
                let ch = rest
                    .chars()
                    .next()
                    .ok_or_else(|| cur.err("empty string slice"))?;
                out.push(ch);
                cur.pos += ch.len_utf8();
            }
            None => return Err(cur.err("unterminated string")),
        }
    }
}

fn parse_array(cur: &mut Cursor<'_>) -> Result<Json, SemanticError> {
    if cur.bump() != Some(b'[') {
        return Err(cur.err("expected array"));
    }
    cur.skip_ws();
    if cur.peek() == Some(b']') {
        cur.pos += 1;
        return Ok(Json::Array(Vec::new()));
    }
    let mut items = Vec::new();
    loop {
        items.push(parse_value(cur)?);
        cur.skip_ws();
        match cur.bump() {
            Some(b']') => return Ok(Json::Array(items)),
            Some(b',') => {
                cur.skip_ws();
            }
            _ => return Err(cur.err("expected comma or end of array")),
        }
    }
}

fn parse_object(cur: &mut Cursor<'_>) -> Result<Json, SemanticError> {
    if cur.bump() != Some(b'{') {
        return Err(cur.err("expected object"));
    }
    cur.skip_ws();
    if cur.peek() == Some(b'}') {
        cur.pos += 1;
        return Ok(Json::Object(Vec::new()));
    }
    let mut fields = Vec::new();
    loop {
        cur.skip_ws();
        let key = parse_string(cur)?;
        cur.skip_ws();
        if cur.bump() != Some(b':') {
            return Err(cur.err("expected colon"));
        }
        let value = parse_value(cur)?;
        fields.push((key, value));
        cur.skip_ws();
        match cur.bump() {
            Some(b'}') => return Ok(Json::Object(fields)),
            Some(b',') => {}
            _ => return Err(cur.err("expected comma or end of object")),
        }
    }
}

// ---------------------------------------------------------------------------
// YAML emitter
// ---------------------------------------------------------------------------

fn emit_yaml(value: &Json, indent: usize, out: &mut String) {
    match value {
        Json::Null => out.push_str("null"),
        Json::Bool(true) => out.push_str("true"),
        Json::Bool(false) => out.push_str("false"),
        Json::Number(number) => out.push_str(number),
        Json::String(text) => out.push_str(&yaml_quote(text)),
        Json::Array(items) => {
            if items.is_empty() {
                out.push_str("[]");
            } else {
                emit_block_array(items, indent, out);
            }
        }
        Json::Object(fields) => {
            if fields.is_empty() {
                out.push_str("{}");
            } else {
                emit_object(fields, indent, out);
            }
        }
    }
}

fn emit_object(fields: &[(String, Json)], indent: usize, out: &mut String) {
    for (index, (key, value)) in fields.iter().enumerate() {
        if index > 0 {
            out.push('\n');
            out.push_str(&" ".repeat(indent));
        }
        out.push_str(&yaml_key(key));
        out.push(':');
        emit_field_value(value, indent, out);
    }
}

fn emit_field_value(value: &Json, indent: usize, out: &mut String) {
    match value {
        Json::Object(inner) if !inner.is_empty() => {
            out.push('\n');
            out.push_str(&" ".repeat(indent + 2));
            emit_object(inner, indent + 2, out);
        }
        Json::Array(items) if !items.is_empty() => {
            out.push('\n');
            emit_block_array(items, indent + 2, out);
        }
        other => {
            out.push(' ');
            emit_yaml(other, indent + 2, out);
        }
    }
}

fn emit_block_array(items: &[Json], indent: usize, out: &mut String) {
    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        out.push_str(&" ".repeat(indent));
        out.push_str("- ");
        match item {
            Json::Object(fields) if !fields.is_empty() => {
                emit_object_list_item(fields, indent + 2, out);
            }
            other => emit_yaml(other, indent + 2, out),
        }
    }
}

fn emit_object_list_item(fields: &[(String, Json)], rest_indent: usize, out: &mut String) {
    for (index, (key, value)) in fields.iter().enumerate() {
        if index > 0 {
            out.push('\n');
            out.push_str(&" ".repeat(rest_indent));
        }
        out.push_str(&yaml_key(key));
        out.push(':');
        emit_field_value(value, rest_indent, out);
    }
}

fn yaml_key(key: &str) -> String {
    if key
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '$')
        && !key.is_empty()
    {
        key.to_owned()
    } else {
        yaml_quote(key)
    }
}

fn yaml_quote(text: &str) -> String {
    let needs_quotes = text.is_empty()
        || matches!(
            text,
            "null"
                | "Null"
                | "NULL"
                | "~"
                | "true"
                | "false"
                | "True"
                | "False"
                | "TRUE"
                | "FALSE"
                | "yes"
                | "no"
                | "on"
                | "off"
        )
        || text.starts_with(['-', ':', '?', '*', '&', '!', '|', '>', '%', '@', '`'])
        || text.contains(':')
        || text.contains('#')
        || text.contains('\n')
        || text.contains('\r')
        || text.contains('\t')
        || text.starts_with(' ')
        || text.ends_with(' ')
        || looks_like_number(text);
    if !needs_quotes {
        return text.to_owned();
    }
    let mut out = String::from("\"");
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

fn looks_like_number(text: &str) -> bool {
    let bytes = text.as_bytes();
    if bytes.is_empty() {
        return false;
    }
    let mut i = 0;
    if bytes[0] == b'-' {
        i = 1;
        if i == bytes.len() {
            return false;
        }
    }
    i < bytes.len() && bytes[i].is_ascii_digit()
}

// ---------------------------------------------------------------------------
// Typed execute codecs (serde structs included by ontology REST typed_action.rs)
// ---------------------------------------------------------------------------

/// Emit the serde codecs for the thirteen DispatchTarget inputs and two nested
/// write bags. Fail closed when a property cannot be mapped to a codec type.
///
/// # Errors
///
/// Returns when the committed JSON is not the expected object, when a required
/// Input body is missing, when a field type is unmapped, or when generation
/// would drop below [`CODEC_SCHEMA_COUNT`].
pub fn generated_typed_action_rs() -> Result<String, SemanticError> {
    let root = parse_json(SEMANTIC_MANIFEST)?;
    let schemas = root
        .get("schemas")
        .and_then(Json::as_object)
        .ok_or_else(|| SemanticError("semantic manifest schemas must be an object".to_owned()))?;
    let objects = root
        .get("objects")
        .and_then(Json::as_array)
        .ok_or_else(|| SemanticError("semantic manifest objects must be an array".to_owned()))?;
    let actions = root
        .get("actions")
        .and_then(Json::as_array)
        .ok_or_else(|| SemanticError("semantic manifest actions must be an array".to_owned()))?;

    if actions.len() != DISPATCH_TARGET_COUNT {
        return Err(SemanticError(format!(
            "semantic manifest actions has {}, expected {DISPATCH_TARGET_COUNT}",
            actions.len()
        )));
    }

    let head_names: Vec<String> = objects
        .iter()
        .filter_map(|object| object.get("name").and_then(Json::as_str).map(str::to_owned))
        .collect();

    let input_names: Vec<String> = actions
        .iter()
        .map(|action| {
            action
                .get("input")
                .and_then(Json::as_str)
                .map(str::to_owned)
                .ok_or_else(|| SemanticError("action input must be a string".to_owned()))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut codec_names: Vec<String> = schemas
        .iter()
        .map(|(name, _)| name.clone())
        .filter(|name| !head_names.iter().any(|head| head == name))
        .collect();
    codec_names.sort();
    if codec_names.len() != CODEC_SCHEMA_COUNT {
        return Err(SemanticError(format!(
            "semantic manifest codec schemas {}, expected {CODEC_SCHEMA_COUNT}",
            codec_names.len()
        )));
    }
    for name in &input_names {
        if !codec_names.iter().any(|codec| codec == name) {
            return Err(SemanticError(format!(
                "action input {name} is not a codec schema in the semantic manifest"
            )));
        }
    }

    let mut out = String::from(
        "// @generated by console-openapi-gen from semantic_manifest.json. Do not edit.\n\n",
    );

    // Nested write bags first (referenced by Inputs), then Inputs in action order.
    let mut emit_order: Vec<String> = codec_names
        .iter()
        .filter(|name| !input_names.iter().any(|input| input == *name))
        .cloned()
        .collect();
    emit_order.extend(input_names.iter().cloned());

    for name in &emit_order {
        let schema = schemas
            .iter()
            .find(|(schema_name, _)| schema_name == name)
            .map(|(_, schema)| schema)
            .ok_or_else(|| SemanticError(format!("missing codec schema {name}")))?;
        emit_codec_struct(name, schema, &codec_names, &mut out)?;
        out.push('\n');
    }

    out.push_str(
        "fn decode_dispatch_target(target: DispatchTarget, params: &Value) -> Result<(), ActionError> {\n",
    );
    out.push_str("    match target {\n");
    for action in actions {
        let action_key = action
            .get("action_key")
            .and_then(Json::as_str)
            .ok_or_else(|| SemanticError("action_key must be a string".to_owned()))?;
        let input = action
            .get("input")
            .and_then(Json::as_str)
            .ok_or_else(|| SemanticError(format!("{action_key} input must be a string")))?;
        let variant = dispatch_variant(action_key)?;
        let one_line =
            format!("        DispatchTarget::{variant} => decode::<{input}>(target, params),");
        if one_line.len() > 100 {
            out.push_str("        DispatchTarget::");
            out.push_str(&variant);
            out.push_str(" => {\n            decode::<");
            out.push_str(input);
            out.push_str(">(target, params)\n        }\n");
        } else {
            out.push_str(&one_line);
            out.push('\n');
        }
    }
    out.push_str("    }\n");
    out.push_str("}\n");
    Ok(out)
}

fn dispatch_variant(action_key: &str) -> Result<String, SemanticError> {
    let mut out = String::new();
    let mut start = true;
    for ch in action_key.chars() {
        if ch == '.' || ch == '_' {
            start = true;
            continue;
        }
        if !ch.is_ascii_alphanumeric() {
            return Err(SemanticError(format!(
                "action_key {action_key:?} is not a DispatchTarget ident"
            )));
        }
        if start {
            out.push(ch.to_ascii_uppercase());
            start = false;
        } else {
            out.push(ch);
        }
    }
    if out.is_empty() {
        return Err(SemanticError(format!(
            "action_key {action_key:?} produced an empty DispatchTarget variant"
        )));
    }
    Ok(out)
}

fn emit_codec_struct(
    name: &str,
    schema: &Json,
    codec_names: &[String],
    out: &mut String,
) -> Result<(), SemanticError> {
    if primary_type_name(schema) != Some("object") {
        return Err(SemanticError(format!(
            "{name} codec schema must be type object"
        )));
    }
    if !matches!(schema.get("additionalProperties"), Some(Json::Bool(false))) {
        return Err(SemanticError(format!(
            "{name} codec schema must set additionalProperties: false"
        )));
    }
    let properties = schema
        .get("properties")
        .and_then(Json::as_object)
        .ok_or_else(|| {
            SemanticError(format!("{name} codec schema properties must be an object"))
        })?;
    let required: Vec<&str> = schema
        .get("required")
        .and_then(Json::as_array)
        .map(|items| items.iter().filter_map(Json::as_str).collect())
        .unwrap_or_default();

    out.push_str("#[allow(dead_code)]\n");
    out.push_str("#[derive(Deserialize)]\n");
    out.push_str("#[serde(deny_unknown_fields)]\n");
    out.push_str("struct ");
    out.push_str(name);
    out.push_str(" {\n");
    for (field, field_schema) in properties {
        let path = format!("{name}.{field}");
        let required = required.contains(&field.as_str());
        emit_codec_field(field, field_schema, required, codec_names, &path, out)?;
    }
    out.push_str("}\n");
    Ok(())
}

fn emit_codec_field(
    field: &str,
    schema: &Json,
    required: bool,
    codec_names: &[String],
    path: &str,
    out: &mut String,
) -> Result<(), SemanticError> {
    let (mapped, nullable) = map_field_type(schema, codec_names, path)?;
    if nullable && mapped.serde_with.is_some() {
        return Err(SemanticError(format!(
            "{path}: nullable date/datetime codecs need a dedicated serde option module"
        )));
    }
    let optional = !required || nullable;
    let ty = if optional {
        format!("Option<{}>", mapped.rust)
    } else {
        mapped.rust.clone()
    };

    let mut serde_attrs: Vec<String> = Vec::new();
    if !required {
        serde_attrs.push("default".to_owned());
    }
    if let Some(with) = mapped.serde_with {
        serde_attrs.push(format!("with = \"{with}\""));
    }
    if !serde_attrs.is_empty() {
        out.push_str("    #[serde(");
        out.push_str(&serde_attrs.join(", "));
        out.push_str(")]\n");
    }
    out.push_str("    ");
    out.push_str(field);
    out.push_str(": ");
    out.push_str(&ty);
    out.push_str(",\n");
    Ok(())
}

struct MappedType {
    rust: String,
    serde_with: Option<&'static str>,
}

fn map_field_type(
    schema: &Json,
    codec_names: &[String],
    path: &str,
) -> Result<(MappedType, bool), SemanticError> {
    let (inner, nullable) = split_nullable(schema, path)?;
    let mapped = map_non_null_type(inner, codec_names, path)?;
    Ok((mapped, nullable))
}

fn split_nullable<'a>(schema: &'a Json, path: &str) -> Result<(&'a Json, bool), SemanticError> {
    if let Some(one_of) = schema.get("oneOf").and_then(Json::as_array) {
        let mut non_null = None;
        let mut saw_null = false;
        for member in one_of {
            if is_null_schema(member) {
                saw_null = true;
            } else if non_null.is_none() {
                non_null = Some(member);
            } else {
                return Err(SemanticError(format!(
                    "{path}: oneOf with more than one non-null member is not a codec type"
                )));
            }
        }
        if saw_null {
            let inner =
                non_null.ok_or_else(|| SemanticError(format!("{path}: oneOf is only null")))?;
            return Ok((inner, true));
        }
        return Err(SemanticError(format!(
            "{path}: oneOf without null is not mapped to a codec type"
        )));
    }
    if let Some(types) = schema.get("type").and_then(Json::as_array) {
        let mut non_null = 0usize;
        let mut saw_null = false;
        for item in types {
            let name = item.as_str().ok_or_else(|| {
                SemanticError(format!("{path}: type union members must be strings"))
            })?;
            if name == "null" {
                saw_null = true;
            } else {
                non_null += 1;
            }
        }
        if saw_null {
            if non_null != 1 {
                return Err(SemanticError(format!(
                    "{path}: type union must be exactly one non-null type plus null"
                )));
            }
            return Ok((schema, true));
        }
    }
    Ok((schema, false))
}

fn is_null_schema(schema: &Json) -> bool {
    schema.get("type").and_then(Json::as_str) == Some("null")
}

fn map_non_null_type(
    schema: &Json,
    codec_names: &[String],
    path: &str,
) -> Result<MappedType, SemanticError> {
    if let Some(name) = schema_ref_name(schema) {
        if name == "Uuid" {
            return Ok(MappedType {
                rust: "Uuid".to_owned(),
                serde_with: None,
            });
        }
        if name == "Timestamp" {
            return Ok(MappedType {
                rust: "OffsetDateTime".to_owned(),
                serde_with: Some("time::serde::rfc3339"),
            });
        }
        if codec_names.iter().any(|codec| codec == name) {
            return Ok(MappedType {
                rust: name.to_owned(),
                serde_with: None,
            });
        }
        return Err(SemanticError(format!(
            "{path}: $ref {name} is not Uuid, Timestamp, or a codec schema"
        )));
    }
    match primary_type_name(schema) {
        Some("string") => {
            let format = schema.get("format").and_then(Json::as_str);
            match format {
                Some("date") => Ok(MappedType {
                    rust: "Date".to_owned(),
                    serde_with: Some("iso_date"),
                }),
                Some("date-time") => Err(SemanticError(format!(
                    "{path}: use Timestamp $ref for date-time, not format date-time"
                ))),
                Some(other) => Err(SemanticError(format!(
                    "{path}: string format {other} is not a codec type"
                ))),
                None => Ok(MappedType {
                    rust: "String".to_owned(),
                    serde_with: None,
                }),
            }
        }
        Some("object") => {
            if matches!(schema.get("additionalProperties"), Some(Json::Bool(true))) {
                Ok(MappedType {
                    rust: "serde_json::Map<String, Value>".to_owned(),
                    serde_with: None,
                })
            } else {
                Err(SemanticError(format!(
                    "{path}: inline object must be additionalProperties true (open bag) or a $ref to a named codec schema"
                )))
            }
        }
        Some(other) => Err(SemanticError(format!(
            "{path}: type {other} is not a codec type"
        ))),
        None => Err(SemanticError(format!(
            "{path}: codec field has no type or $ref"
        ))),
    }
}

fn schema_ref_name(schema: &Json) -> Option<&str> {
    schema
        .get("$ref")
        .and_then(Json::as_str)
        .and_then(|pointer| pointer.strip_prefix("#/components/schemas/"))
}

fn primary_type_name(schema: &Json) -> Option<&str> {
    match schema.get("type") {
        Some(Json::String(name)) => Some(name.as_str()),
        Some(Json::Array(items)) => items
            .iter()
            .filter_map(Json::as_str)
            .find(|name| *name != "null"),
        _ => None,
    }
}
