//! Deterministic OpenAPI composition from per-face fragments.
//!
//! Each REST face owns a [`Fragment`]: the paths it serves and the schemas it
//! introduces. [`compose`] merges fragments into one document body.
//!
//! Three properties are the whole point of this crate:
//!
//! * **Duplicates are errors.** A path, operation or schema key contributed
//!   twice is a [`DuplicateKey`], never last-writer-wins. Silent overwrite is
//!   how a face loses an endpoint from the published contract without any
//!   diff showing it.
//! * **Output is a function of the fragment set.** Keys are emitted in sorted
//!   order and bodies are re-indented from scratch, so the bytes do not depend
//!   on registration order or on how an author indented a raw string.
//! * **Every schema `$ref` resolves.** A ref whose target no fragment defines
//!   and no fragment declares external is a [`DanglingRef`]; a target that is
//!   not an OpenAPI component key at all is a [`MalformedRef`]; a ref into a
//!   component section OpenAPI does not define — a typo of `schemas`, say — is
//!   an [`UnknownSection`]. Without this, a schema deleted from a fragment is
//!   silent: the paths keep pointing at it and the composed document simply
//!   loses the definition.
//!
//! A ref into a component section OpenAPI DOES define but [`Fragment`] does not
//! model — `responses`, `parameters`, … — is neither resolved nor rejected
//! here: the published document supplies those sections, and the faces ship
//! such refs today. [`compose`] cannot see the published document, so it is not
//! the control for them; only the section NAME is checkable from a fragment
//! alone, and that is what it checks.
//!
//! Fragment bodies are raw YAML text rather than a typed model: the faces
//! already author this YAML by hand, and a typed re-implementation of OpenAPI
//! would be a second source of truth to keep in sync.
//! ponytail: raw-text bodies, upgrade to a typed model only if we ever need to
//! query the composed document rather than emit it.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

// ---------------------------------------------------------------------------
// Fragment model
// ---------------------------------------------------------------------------

/// One face's contribution to the OpenAPI document.
#[derive(Debug, Clone, Copy)]
pub struct Fragment {
    /// Crate that owns this fragment; used to name both sides of a collision.
    pub source: &'static str,
    pub paths: &'static [PathItem],
    pub schemas: &'static [NamedYaml],
    /// Schemas this face `$ref`s but does not own — shared components such as
    /// `Uuid` or `ErrorBody`. Anything referenced and not listed here must be
    /// defined by the composed set, so dropping an owned schema cannot pass
    /// unless the same diff also hands ownership away in this list.
    ///
    /// ponytail: a face can still launder a dropped schema by adding it here,
    /// which costs a second visible edit but is not caught. That escape closes
    /// on its own once a shared fragment actually defines `Uuid`, `Timestamp`
    /// and `ErrorBody`: every face's list becomes empty, and a laundered name
    /// is dangling again because no fragment defines it.
    pub external_schemas: &'static [&'static str],
}

/// One path and every operation served on it. A path is owned by exactly one
/// fragment: two faces contributing methods to the same path is a duplicate.
#[derive(Debug, Clone, Copy)]
pub struct PathItem {
    pub path: &'static str,
    pub operations: &'static [Operation],
}

/// One HTTP operation. `body` is the YAML beneath the method key
/// (`operationId`, `responses`, …) at any indentation.
#[derive(Debug, Clone, Copy)]
pub struct Operation {
    pub method: &'static str,
    pub body: &'static str,
}

/// A `components/schemas` entry. `body` is the YAML beneath the schema name.
#[derive(Debug, Clone, Copy)]
pub struct NamedYaml {
    pub name: &'static str,
    pub body: &'static str,
}

// ---------------------------------------------------------------------------
// Collisions
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DuplicateKind {
    Path,
    Operation,
    Schema,
}

impl DuplicateKind {
    fn label(self) -> &'static str {
        match self {
            DuplicateKind::Path => "path",
            DuplicateKind::Operation => "operation",
            DuplicateKind::Schema => "schema",
        }
    }
}

/// A key contributed more than once. `first` and `second` name the owning
/// crates so an operator can route the conflict without reading the fragments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateKey {
    pub kind: DuplicateKind,
    pub key: String,
    pub first: &'static str,
    pub second: &'static str,
}

impl fmt::Display for DuplicateKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "duplicate {} key '{}': contributed by both {} and {}",
            self.kind.label(),
            self.key,
            self.first,
            self.second
        )
    }
}

impl std::error::Error for DuplicateKey {}

/// A `$ref` to a schema the composed set does not define and no fragment
/// declares external.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DanglingRef {
    pub schema: String,
    /// Crate whose fragment body contains the ref.
    pub source: &'static str,
}

impl fmt::Display for DanglingRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} refs schema '{}', which no fragment defines and no fragment \
             lists in external_schemas",
            self.source, self.schema
        )
    }
}

impl std::error::Error for DanglingRef {}

/// A `$ref` whose target is not an OpenAPI component key, so it can never name
/// a schema: an empty target, or one carrying a nested JSON pointer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MalformedRef {
    pub target: String,
    /// Crate whose fragment body contains the ref.
    pub source: &'static str,
}

impl fmt::Display for MalformedRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} refs '#/components/schemas/{}', whose target is not an OpenAPI \
             component key (`[A-Za-z0-9._-]+`)",
            self.source, self.target
        )
    }
}

impl std::error::Error for MalformedRef {}

/// A `$ref` into a component section OpenAPI does not define, so it names no
/// part of any document and can never resolve — almost always a typo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownSection {
    /// The segment after `#/components/`, e.g. `schema` for a typo of `schemas`.
    pub section: String,
    /// Crate whose fragment body contains the ref.
    pub source: &'static str,
}

impl fmt::Display for UnknownSection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} refs '#/components/{}/…', but OpenAPI defines no '{}' component \
             section, so the ref resolves in no document",
            self.source, self.section, self.section
        )
    }
}

impl std::error::Error for UnknownSection {}

/// The component sections OpenAPI 3.1 defines, which is the version
/// `backend/openapi/openapi.yaml` declares. Closed by the specification rather
/// than by observation: a section a later OpenAPI version adds is rejected
/// until it is listed here, which is the fail-closed direction.
const COMPONENT_SECTIONS: [&str; 10] = [
    "callbacks",
    "examples",
    "headers",
    "links",
    "parameters",
    "pathItems",
    "requestBodies",
    "responses",
    "schemas",
    "securitySchemes",
];

/// Everything wrong with one composition run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposeError {
    pub duplicates: Vec<DuplicateKey>,
    pub unknown_sections: Vec<UnknownSection>,
    pub malformed: Vec<MalformedRef>,
    pub dangling: Vec<DanglingRef>,
}

impl fmt::Display for ComposeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OpenAPI composition failed:")?;
        for duplicate in &self.duplicates {
            write!(f, "\n  {duplicate}")?;
        }
        for unknown in &self.unknown_sections {
            write!(f, "\n  {unknown}")?;
        }
        for malformed in &self.malformed {
            write!(f, "\n  {malformed}")?;
        }
        for dangling in &self.dangling {
            write!(f, "\n  {dangling}")?;
        }
        Ok(())
    }
}

impl std::error::Error for ComposeError {}

// ---------------------------------------------------------------------------
// Composition
// ---------------------------------------------------------------------------

/// Merge `fragments` into one OpenAPI document body: a `paths:` block, plus a
/// `components:`/`schemas:` block when any fragment contributes a schema.
///
/// # Errors
///
/// Returns every duplicate path, operation and schema key found — not just the
/// first, so one CI run reports the whole conflict set. Once the keys are
/// unique, returns every malformed and every unresolvable schema `$ref`.
pub fn compose(fragments: &[&Fragment]) -> Result<String, ComposeError> {
    // Value carries the owning source so a later collision can name both sides.
    let mut paths: BTreeMap<&str, (&'static str, BTreeMap<String, String>)> = BTreeMap::new();
    let mut schemas: BTreeMap<&str, (&'static str, String)> = BTreeMap::new();
    let mut duplicates = Vec::new();

    for fragment in fragments {
        for item in fragment.paths {
            let mut operations: BTreeMap<String, String> = BTreeMap::new();
            for operation in item.operations {
                // Lowercase so `GET` and `get` collide instead of both emitting.
                let method = operation.method.to_ascii_lowercase();
                if operations
                    .insert(method.clone(), reindent(operation.body, 6))
                    .is_some()
                {
                    duplicates.push(DuplicateKey {
                        kind: DuplicateKind::Operation,
                        key: format!("{method} {}", item.path),
                        first: fragment.source,
                        second: fragment.source,
                    });
                }
            }

            if let Some((owner, _)) = paths.get(item.path) {
                duplicates.push(DuplicateKey {
                    kind: DuplicateKind::Path,
                    key: item.path.to_owned(),
                    first: owner,
                    second: fragment.source,
                });
                continue;
            }
            paths.insert(item.path, (fragment.source, operations));
        }

        for schema in fragment.schemas {
            if let Some((owner, _)) = schemas.get(schema.name) {
                duplicates.push(DuplicateKey {
                    kind: DuplicateKind::Schema,
                    key: schema.name.to_owned(),
                    first: owner,
                    second: fragment.source,
                });
                continue;
            }
            schemas.insert(schema.name, (fragment.source, reindent(schema.body, 6)));
        }
    }

    if !duplicates.is_empty() {
        // Ref resolution needs the complete schema set, which a collision denies.
        return Err(ComposeError {
            duplicates,
            unknown_sections: Vec::new(),
            malformed: Vec::new(),
            dangling: Vec::new(),
        });
    }

    let mut known: BTreeSet<&str> = schemas.keys().copied().collect();
    let mut unknown_sections = Vec::new();
    let mut malformed = Vec::new();
    let mut dangling = Vec::new();
    for fragment in fragments {
        known.extend(fragment.external_schemas.iter().copied());
    }
    for fragment in fragments {
        let bodies = fragment
            .paths
            .iter()
            .flat_map(|item| item.operations.iter().map(|operation| operation.body))
            .chain(fragment.schemas.iter().map(|schema| schema.body));
        for (section, target) in bodies.flat_map(component_refs) {
            if section != "schemas" {
                if !COMPONENT_SECTIONS.contains(&section) {
                    unknown_sections.push(UnknownSection {
                        section: section.to_owned(),
                        source: fragment.source,
                    });
                }
                // A section OpenAPI defines but `Fragment` does not model is
                // resolved by the published document, not by the fragment set.
                continue;
            }
            if !is_component_key(target) {
                malformed.push(MalformedRef {
                    target: target.to_owned(),
                    source: fragment.source,
                });
            } else if !known.contains(target) {
                dangling.push(DanglingRef {
                    schema: target.to_owned(),
                    source: fragment.source,
                });
            }
        }
    }
    if !unknown_sections.is_empty() || !malformed.is_empty() || !dangling.is_empty() {
        return Err(ComposeError {
            duplicates,
            unknown_sections,
            malformed,
            dangling,
        });
    }

    let mut out = String::from("paths:\n");
    for (path, (_, operations)) in &paths {
        out.push_str(&format!("  {path}:\n"));
        for (method, body) in operations {
            out.push_str(&format!("    {method}:\n"));
            out.push_str(body);
        }
    }

    if !schemas.is_empty() {
        out.push_str("components:\n  schemas:\n");
        for (name, (_, body)) in &schemas {
            out.push_str(&format!("    {name}:\n"));
            out.push_str(body);
        }
    }

    Ok(out)
}

/// Every `#/components/schemas/…` target named in `body`, whole.
///
/// A target runs to the YAML token that ends the scalar it sits in — a quote,
/// whitespace, or a flow-collection delimiter — NOT to the first character
/// outside `[A-Za-z0-9_]`. Component keys are `[A-Za-z0-9._-]+`, so stopping
/// at `.` or `-` would silently resolve `Todo-Summary` against `Todo`.
/// Anything left that is not a component key is reported by [`compose`] as a
/// [`MalformedRef`] rather than resolved.
///
/// Refs into other component sections (`responses`, …) are NOT yielded: they
/// are component references but not schema ones, and [`Fragment`] models only
/// schemas. A caller walking a transitive `$ref` closure with this therefore
/// stops at the boundary of a non-schema component and never enters its body —
/// which is what `console_todos_rest`'s drift test does. Crossing that boundary
/// needs `component_refs` below, which is private until a caller wants it.
///
/// Public so a face's drift test can walk the PUBLISHED document with the same
/// parser [`compose`] uses; a second ref parser in a test is a second source of
/// truth about what a ref is.
pub fn schema_refs(body: &str) -> impl Iterator<Item = &str> {
    component_refs(body).filter_map(|(section, target)| (section == "schemas").then_some(target))
}

/// Every `#/components/<section>/<target>` reference named in `body`, whole.
///
/// The one splitter behind [`schema_refs`] and [`compose`]'s section check, so
/// there is a single answer to "what is a ref" no matter which section it names.
/// A reference with no `/` after the section yields an empty target, which is
/// not a component key and so is reported rather than dropped.
fn component_refs(body: &str) -> impl Iterator<Item = (&str, &str)> {
    body.split("#/components/").skip(1).map(|rest| {
        let end = rest
            .find(|c: char| c.is_whitespace() || matches!(c, '\'' | '"' | ',' | ']' | '}'))
            .unwrap_or(rest.len());
        let reference = &rest[..end];
        reference.split_once('/').unwrap_or((reference, ""))
    })
}

/// Whether `target` is an OpenAPI component key: `^[a-zA-Z0-9._-]+$`.
fn is_component_key(target: &str) -> bool {
    !target.is_empty()
        && target
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
}

/// Re-emit `body` at exactly `indent` spaces: strip blank leading/trailing
/// lines, remove the author's common leading indentation, trim trailing
/// whitespace, and terminate every line with `\n`. This is what makes the
/// output independent of how the fragment was written.
fn reindent(body: &str, indent: usize) -> String {
    let lines: Vec<&str> = body
        .lines()
        .map(str::trim_end)
        .skip_while(|line| line.is_empty())
        .collect();
    let end = lines
        .iter()
        .rposition(|line| !line.is_empty())
        .map_or(0, |idx| idx + 1);
    let lines = &lines[..end];

    let common = lines
        .iter()
        .filter(|line| !line.is_empty())
        .map(|line| line.len() - line.trim_start_matches(' ').len())
        .min()
        .unwrap_or(0);

    let pad = " ".repeat(indent);
    let mut out = String::with_capacity(body.len() + lines.len() * indent);
    for line in lines {
        if line.is_empty() {
            out.push('\n');
        } else {
            out.push_str(&pad);
            out.push_str(&line[common..]);
            out.push('\n');
        }
    }
    out
}
