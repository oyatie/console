//! Compile only fixed owner definitions and the supplied stable field bindings.
//! This is an in-memory structural index, not a second validation language.
use super::*;
use crate::owner28::schema as fixed;
use jsonschema::Validator;

pub(super) struct Node {
    pub kind: Kind,
    source: Value,
}
pub(super) enum Kind {
    Scalar {
        kind: ScalarKind,
        validator: Validator,
    },
    Record(BTreeMap<Uuid, Node>),
    List {
        item: Box<Node>,
        maximum: usize,
    },
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ScalarKind {
    Text,
    Integer,
    Decimal,
    Boolean,
    Date,
    Instant,
    FieldValue,
}
impl ScalarKind {
    fn accepts(self, value: &FieldValue) -> bool {
        if value.0["kind"] == "NULL" {
            return true;
        }
        self == Self::FieldValue
            || value.0["kind"].as_str()
                == Some(match self {
                    Self::Text => "TEXT",
                    Self::Integer => "INTEGER",
                    Self::Decimal => "DECIMAL",
                    Self::Boolean => "BOOLEAN",
                    Self::Date => "DATE",
                    Self::Instant => "INSTANT",
                    Self::FieldValue => return true,
                })
    }
}
impl Node {
    pub fn scalar(&self) -> bool {
        matches!(self.kind, Kind::Scalar { .. })
    }
    pub fn validate_value(&self, value: &FieldValue) -> bool {
        let Kind::Scalar { kind, validator } = &self.kind else {
            return false;
        };
        if !kind.accepts(value) {
            return false;
        }
        if *kind == ScalarKind::FieldValue {
            validator.is_valid(&value.0)
        } else if value.0["kind"] == "NULL" {
            validator.is_valid(&Value::Null)
        } else {
            validator.is_valid(&value.0["value"])
        }
    }
    pub fn empty(&self, requested: ContainerKind) -> Result<DraftCell, DraftError> {
        match (&self.kind, requested) {
            (Kind::Record(children), ContainerKind::Record) => Ok(DraftCell::Record(
                children.keys().map(|id| (*id, DraftCell::Unset)).collect(),
            )),
            (Kind::List { .. }, ContainerKind::List) => Ok(DraftCell::List(Vec::new())),
            _ => Err(DraftError("draft container kind mismatch")),
        }
    }
}
#[derive(Default)]
struct Trie {
    id: Option<Uuid>,
    children: BTreeMap<String, Trie>,
}
pub(crate) struct DraftSchema {
    pub(super) schema_ref: InputSchemaRef,
    pub(super) root: BTreeMap<Uuid, Node>,
    pub(super) routes: BTreeMap<Uuid, Vec<Uuid>>,
    pub(super) work_budget: usize,
}
impl fmt::Debug for DraftSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DraftSchema")
    }
}
impl DraftSchema {
    pub(crate) fn compile_registered(
        action: &str,
        schema_ref: InputSchemaRef,
        fields: Vec<FieldBinding>,
    ) -> Result<Self, DraftError> {
        check(
            !fields.is_empty() && fields.len() <= FIELD_COUNT,
            "draft field count",
        )?;
        let mut trie = Trie::default();
        let mut identities = BTreeSet::new();
        let mut paths = BTreeMap::new();
        for field in &fields {
            check(
                !field.field_id.is_nil() && identities.insert(field.field_id),
                "duplicate or invalid draft field",
            )?;
            check(
                !field.owner_schema_path.is_empty()
                    && field.owner_schema_path.len() <= ADDRESS_DEPTH,
                "draft field path depth",
            )?;
            let mut current = &mut trie;
            for part in &field.owner_schema_path {
                check(
                    !part.is_empty() && part.len() <= 64 && part.is_ascii() && !part.contains('\0'),
                    "invalid draft field path",
                )?;
                current = current.children.entry(part.clone()).or_default();
            }
            check(
                current.id.replace(field.field_id).is_none(),
                "duplicate draft field path",
            )?;
            paths.insert(field.owner_schema_path.clone(), field.field_id);
        }
        let mut routes = BTreeMap::new();
        for field in &fields {
            let mut route = Vec::new();
            for end in 1..=field.owner_schema_path.len() {
                route.push(
                    *paths
                        .get(&field.owner_schema_path[..end])
                        .ok_or(DraftError("unbound draft parent field"))?,
                );
            }
            routes.insert(field.field_id, route);
        }
        let body = fixed::draft_body(action)?;
        let compiled = compile(&body, &trie, 0)?;
        let Kind::Record(root) = compiled.kind else {
            return Err(DraftError("draft owner body is not a record"));
        };
        Ok(Self {
            schema_ref,
            root,
            routes,
            work_budget: if action == "payroll.review.release" {
                8_388_608
            } else {
                65_536
            },
        })
    }
}

// Inspect the fixed referenced definition for structure while retaining the
// original reference wrapper in the leaf validator, including sibling constraints.
fn head(raw: &Value) -> Result<(Value, Vec<String>), DraftError> {
    let mut value = raw.clone();
    let mut aliases = Vec::new();
    while let Some(reference) = value.get("$ref").and_then(Value::as_str) {
        check(
            aliases.len() < 32 && !aliases.iter().any(|seen| seen == reference),
            "recursive draft schema reference",
        )?;
        aliases.push(reference.to_owned());
        let mut target = fixed::draft_resource(reference)?.clone();
        for (key, sibling) in value
            .as_object()
            .ok_or(DraftError("invalid fixed schema"))?
        {
            if key != "$ref" && !target.get(key).is_some_and(|prior| prior != sibling) {
                target
                    .as_object_mut()
                    .ok_or(DraftError("invalid fixed schema"))?
                    .insert(key.clone(), sibling.clone());
            }
        }
        value = target;
    }
    Ok((value, aliases))
}
fn compile(raw: &Value, trie: &Trie, depth: usize) -> Result<Node, DraftError> {
    check(depth <= 32, "draft schema structure depth")?;
    let (shape, aliases) = head(raw)?;
    if aliases
        .iter()
        .any(|name| name.ends_with("/definitions/FieldValue"))
    {
        check(trie.children.is_empty(), "scalar draft field has children")?;
        return Ok(Node {
            kind: Kind::Scalar {
                kind: ScalarKind::FieldValue,
                validator: fixed::draft_validator(raw)?,
            },
            source: raw.clone(),
        });
    }
    if let Some(branches) = shape
        .get("oneOf")
        .or_else(|| shape.get("anyOf"))
        .and_then(Value::as_array)
    {
        let mut selected: Option<Node> = None;
        for branch in branches {
            let (branch_shape, _) = head(branch)?;
            if branch_shape.get("type").and_then(Value::as_str) == Some("null") {
                continue;
            }
            if let Ok(node) = compile(branch, trie, depth + 1) {
                if let Some(prior) = &selected {
                    check(prior.source == node.source, "ambiguous draft schema branch")?;
                } else {
                    selected = Some(node);
                }
            }
        }
        let mut node = selected.ok_or(DraftError("unresolved draft schema branch"))?;
        if let Kind::Scalar { validator, .. } = &mut node.kind {
            // The whole original union owns nullability and scalar constraints.
            *validator = fixed::draft_validator(raw)?;
        }
        node.source = raw.clone();
        return Ok(node);
    }
    let kind = shape.get("type").and_then(Value::as_str);
    let result = match kind {
        Some("array") => {
            let maximum = shape
                .get("maxItems")
                .and_then(Value::as_u64)
                .ok_or(DraftError("unbounded draft list schema"))?;
            check(maximum <= 2000, "draft list schema bound")?;
            let item = shape
                .get("items")
                .ok_or(DraftError("missing draft list element"))?;
            Kind::List {
                item: Box::new(compile(item, trie, depth + 1)?),
                maximum: usize::try_from(maximum)
                    .map_err(|_| DraftError("draft list schema bound"))?,
            }
        }
        Some("object") => {
            let properties = shape
                .get("properties")
                .and_then(Value::as_object)
                .ok_or(DraftError("unsupported draft record schema"))?;
            let mut children = BTreeMap::new();
            for (name, child) in &trie.children {
                let definition = properties
                    .get(name)
                    .ok_or(DraftError("unregistered draft property"))?;
                let id = child.id.ok_or(DraftError("unbound draft parent field"))?;
                children.insert(id, compile(definition, child, depth + 1)?);
            }
            Kind::Record(children)
        }
        Some("string" | "boolean") | None => {
            check(trie.children.is_empty(), "scalar draft field has children")?;
            let scalar = if kind == Some("boolean") {
                ScalarKind::Boolean
            } else if shape.get("format").and_then(Value::as_str) == Some("date") {
                ScalarKind::Date
            } else if aliases
                .iter()
                .any(|name| name.ends_with("/definitions/Time"))
            {
                ScalarKind::Instant
            } else if shape.get("x-decimal-min").is_some()
                || shape.get("x-decimal-max").is_some()
                || aliases.iter().any(|name| {
                    name.ends_with("/definitions/Minute") || name.ends_with("/definitions/Count")
                })
            {
                ScalarKind::Integer
            } else if aliases
                .iter()
                .any(|name| name.ends_with("/definitions/Decimal"))
            {
                ScalarKind::Decimal
            } else {
                check(
                    kind == Some("string")
                        || shape.get("const").is_some_and(Value::is_string)
                        || shape
                            .get("enum")
                            .and_then(Value::as_array)
                            .is_some_and(|values| {
                                !values.is_empty() && values.iter().all(Value::is_string)
                            }),
                    "unsupported draft scalar schema",
                )?;
                ScalarKind::Text
            };
            Kind::Scalar {
                kind: scalar,
                validator: fixed::draft_validator(raw)?,
            }
        }
        _ => return Err(DraftError("unsupported draft schema structure")),
    };
    Ok(Node {
        kind: result,
        source: raw.clone(),
    })
}

#[cfg(test)]
#[path = "schema_tests.rs"]
mod tests;
