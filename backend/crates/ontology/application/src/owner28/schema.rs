//! Fixed local normalized schemas. No network, file retrieval, caller schema,
//! registration currentness, or capability construction is involved.
use super::{CodecError, OnceLock};
use jsonschema::{Keyword, Retrieve, Uri, ValidationError, Validator, paths::Location};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

pub(super) const SOURCE_ACTIONS: &[&str] = &[
    "calendar.publish",
    "calendar.assign",
    "attendance.resolve_coverage",
    "attendance.create_close_basis",
    "source.propose_qualification",
    "source.verify_qualification",
    "source.revoke_qualification",
    "eligibility.propose",
    "eligibility.verify",
    "eligibility.revoke",
    "payroll.admit_tax_evidence",
    "payroll.verify_tax_evidence",
    "payroll.revoke_tax_evidence",
];
const BASE: &str = "https://console.invalid/owner28/";
const DOCUMENTS: &[(&str, &str)] = &[
    (
        "2026-09-12-native-manifest-contract-18/native-types.schema.json",
        include_str!(
            "../owner28_schemas/2026-09-12-native-manifest-contract-18/native-types.schema.json"
        ),
    ),
    (
        "2026-09-12-source-payload-contract-19/source-types.schema.json",
        include_str!(
            "../owner28_schemas/2026-09-12-source-payload-contract-19/source-types.schema.json"
        ),
    ),
    (
        "2026-09-13-integrated-design-30/action-types.schema.json",
        include_str!("../owner28_schemas/2026-09-13-integrated-design-30/action-types.schema.json"),
    ),
    (
        "2026-09-13-integrated-design-30/control-types.schema.json",
        include_str!(
            "../owner28_schemas/2026-09-13-integrated-design-30/control-types.schema.json"
        ),
    ),
    (
        "2026-09-13-integrated-design-30/journal-types.schema.json",
        include_str!(
            "../owner28_schemas/2026-09-13-integrated-design-30/journal-types.schema.json"
        ),
    ),
    (
        "2026-09-13-integrated-design-30/types.schema.json",
        include_str!("../owner28_schemas/2026-09-13-integrated-design-30/types.schema.json"),
    ),
    (
        "2026-09-13-submission-custody-contract-24/submission-types.schema.json",
        include_str!(
            "../owner28_schemas/2026-09-13-submission-custody-contract-24/submission-types.schema.json"
        ),
    ),
];
pub(super) struct Binding {
    pub owner: String,
    pub object_kind: String,
    pub input: Validator,
}
pub(super) struct Schemas {
    pub input_schema: Validator,
    pub bindings: BTreeMap<String, Binding>,
}
struct FixedSchemas(BTreeMap<String, Value>);
impl Retrieve for FixedSchemas {
    fn retrieve(
        &self,
        uri: &Uri<String>,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        self.0
            .get(uri.as_str())
            .cloned()
            .ok_or_else(|| "unregistered schema resource".into())
    }
}
struct DecimalBound {
    bound: i64,
    minimum: bool,
}
impl<'i> Keyword<'i> for DecimalBound {
    fn validate(&self, instance: &'i Value) -> Result<(), ValidationError<'i>> {
        if self.is_valid(instance) {
            Ok(())
        } else {
            Err(ValidationError::custom("checked decimal bound"))
        }
    }
    fn is_valid(&self, instance: &'i Value) -> bool {
        // These extension keywords constrain the string alternatives. The
        // standard schema handles type/null alternatives independently.
        instance.as_str().is_none_or(|text| {
            text.parse::<i64>().is_ok_and(|n| {
                if self.minimum {
                    n >= self.bound
                } else {
                    n <= self.bound
                }
            })
        })
    }
}
fn minimum<'a>(
    _parent: &'a Map<String, Value>,
    value: &'a Value,
    _path: Location,
) -> Result<Box<dyn for<'i> Keyword<'i>>, ValidationError<'a>> {
    bound(value, true)
}
fn maximum<'a>(
    _parent: &'a Map<String, Value>,
    value: &'a Value,
    _path: Location,
) -> Result<Box<dyn for<'i> Keyword<'i>>, ValidationError<'a>> {
    bound(value, false)
}
fn bound(
    value: &Value,
    minimum: bool,
) -> Result<Box<dyn for<'i> Keyword<'i>>, ValidationError<'_>> {
    let bound = value
        .as_i64()
        .ok_or_else(|| ValidationError::schema("invalid checked decimal bound"))?;
    Ok(Box::new(DecimalBound { bound, minimum }))
}
fn compile(reference: &str, resources: &BTreeMap<String, Value>) -> Result<Validator, CodecError> {
    jsonschema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .should_validate_formats(true)
        .with_retriever(FixedSchemas(resources.clone()))
        .with_keyword("x-decimal-min", minimum)
        .with_keyword("x-decimal-max", maximum)
        .build(&serde_json::json!({"$ref":reference}))
        .map_err(|_| CodecError("registered schema initialization failed"))
}
pub(super) fn registered() -> Result<&'static Schemas, CodecError> {
    static SCHEMAS: OnceLock<Result<Schemas, CodecError>> = OnceLock::new();
    SCHEMAS.get_or_init(build).as_ref().map_err(|error| *error)
}
fn build() -> Result<Schemas, CodecError> {
    let mut resources = BTreeMap::new();
    // Index each immutable definition separately. The validator otherwise
    // eagerly fetches references in unrelated definitions of a whole document.
    // All refs remain exact; missing reachable resources fail closed.
    for (path, bytes) in DOCUMENTS {
        let document: Value =
            serde_json::from_str(bytes).map_err(|_| CodecError("registered schema JSON"))?;
        for (name, definition) in document["$defs"]
            .as_object()
            .ok_or(CodecError("registered definitions"))?
        {
            let mut definition = definition.clone();
            rewrite_references(&mut definition, path)?;
            let uri = definition_uri(path, &format!("#/$defs/{name}"))?;
            resources.insert(uri, definition);
        }
    }
    let input_schema = compile(
        &definition_uri(
            "2026-09-13-integrated-design-30/action-types.schema.json",
            "#/$defs/InputSchemaRef",
        )?,
        &resources,
    )?;
    let manifest: Value =
        serde_json::from_str(include_str!("../owner28_schemas/owner-bindings.json"))
            .map_err(|_| CodecError("owner binding manifest"))?;
    let mut bindings = BTreeMap::new();
    for binding in manifest["bindings"]
        .as_array()
        .ok_or(CodecError("owner binding list"))?
    {
        let get = |key: &str| {
            binding[key]
                .as_str()
                .ok_or(CodecError("owner binding field"))
        };
        let action = get("key")?;
        // Compile the whole fixed registration inventory, including bodies
        // whose execution uses specialized codecs. Dispatch excludes those
        // separately; no uncompiled registration is silently ignored.
        let input = get("input")?;
        let reference = definition_uri(
            "2026-09-13-integrated-design-30/action-types.schema.json",
            input,
        )?;
        let value = Binding {
            owner: get("owner")?.to_owned(),
            object_kind: get("registered_object_kind")?.to_owned(),
            input: compile(&reference, &resources)?,
        };
        if bindings.insert(action.to_owned(), value).is_some() {
            return Err(CodecError("duplicate owner binding"));
        }
    }
    Ok(Schemas {
        input_schema,
        bindings,
    })
}

fn definition_uri(document: &str, reference: &str) -> Result<String, CodecError> {
    let (file, definition) = reference
        .split_once("#/$defs/")
        .ok_or(CodecError("registered reference syntax"))?;
    if definition.is_empty() || definition.contains('/') || definition.contains('%') {
        return Err(CodecError("registered definition name"));
    }
    let path = if file.is_empty() {
        document.to_owned()
    } else {
        let parent = document.rsplit_once('/').map_or("", |(parent, _)| parent);
        format!("{parent}/{file}")
    };
    let mut normalized = Vec::new();
    for component in path.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                normalized
                    .pop()
                    .ok_or(CodecError("registered reference escape"))?;
            }
            part if part.contains(':') || part.contains('#') || part.contains('%') => {
                return Err(CodecError("registered reference syntax"));
            }
            part => normalized.push(part),
        }
    }
    Ok(format!(
        "{BASE}{}/definitions/{definition}",
        normalized.join("/")
    ))
}
fn rewrite_references(value: &mut Value, document: &str) -> Result<(), CodecError> {
    match value {
        Value::Object(fields) => {
            if let Some(reference) = fields.get_mut("$ref") {
                *reference = Value::String(definition_uri(
                    document,
                    reference
                        .as_str()
                        .ok_or(CodecError("registered reference"))?,
                )?);
            }
            for child in fields.values_mut() {
                rewrite_references(child, document)?;
            }
        }
        Value::Array(values) => {
            for child in values {
                rewrite_references(child, document)?;
            }
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
#[path = "schema_tests.rs"]
mod tests;
