//! Pure editing over fixed registered structure and plain caller-supplied scope.
//! No authenticated capability, current registration, reference resolution,
//! business submission, public wire, or persistent snapshot is constructed here.
use crate::owner28;
use serde::{Deserialize, Deserializer, Serialize, de};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};
use uuid::Uuid;

mod apply;
mod schema;
pub(crate) use apply::apply_draft_patches;
pub(crate) use schema::DraftSchema;

const PATCH_BYTES: usize = 65_536;
const PATCH_COUNT: usize = 256;
const FIELD_COUNT: usize = 128;
const ADDRESS_DEPTH: usize = 8;
const CELL_DEPTH: usize = 16;

/// Fixed diagnostic: input values and references never enter errors.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct DraftError(&'static str);
impl fmt::Display for DraftError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}
impl std::error::Error for DraftError {}
impl From<owner28::CodecError> for DraftError {
    fn from(_: owner28::CodecError) -> Self {
        Self("draft schema or input unavailable")
    }
}
fn check(ok: bool, message: &'static str) -> Result<(), DraftError> {
    if ok { Ok(()) } else { Err(DraftError(message)) }
}
macro_rules! redacted_debug {
    ($($name:ty),+ $(,)?) => {$ (impl fmt::Debug for $name {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(stringify!($name)) }
    })+};
}
macro_rules! plain_value {
    ($name:ident) => {
        #[derive(Clone, PartialEq, Eq, Serialize)]
        #[serde(transparent)]
        pub(crate) struct $name(Value);
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let value = owner28::deserialize_bounded(deserializer, PATCH_BYTES)
                    .map_err(|_| de::Error::custom("invalid draft value"))?;
                match owner28::schema::draft_definition(stringify!($name), &value) {
                    Ok(true) => Ok(Self(value)),
                    _ => Err(de::Error::custom("invalid draft value")),
                }
            }
        }
        redacted_debug!($name);
    };
}
plain_value!(FieldValue);
plain_value!(InputSchemaRef);
plain_value!(GovernedRevision);

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FieldAddress {
    pub field_id: Uuid,
    pub parent_item_ids: Vec<Uuid>,
    pub item_id: Option<Uuid>,
}
impl FieldAddress {
    fn validate(&self) -> Result<(), DraftError> {
        check(
            !self.field_id.is_nil() && self.parent_item_ids.len() <= ADDRESS_DEPTH,
            "invalid draft address",
        )?;
        let mut seen = BTreeSet::new();
        for id in self.parent_item_ids.iter().chain(self.item_id.iter()) {
            check(
                !id.is_nil() && seen.insert(*id),
                "invalid draft item identity",
            )?;
        }
        Ok(())
    }
    fn ancestors(&self) -> Vec<Uuid> {
        let mut ids = self.parent_item_ids.clone();
        ids.extend(self.item_id);
        ids
    }
}
#[derive(Clone)]
pub(crate) struct FieldBinding {
    pub field_id: Uuid,
    pub owner_schema_path: Vec<String>,
}
#[derive(Clone, PartialEq, Eq)]
pub(crate) enum DraftCell {
    Unset,
    Incomplete { text: String },
    Value(FieldValue),
    Record(BTreeMap<Uuid, DraftCell>),
    List(Vec<DraftItem>),
}
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct DraftItem {
    pub item_id: Uuid,
    pub cell: DraftCell,
}
#[derive(Clone, PartialEq, Eq)]
pub(crate) enum DraftProvenance {
    User,
    Source { reference: GovernedRevision },
}
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct DraftSnapshot {
    pub schema_ref: InputSchemaRef,
    pub cells: BTreeMap<Uuid, DraftCell>,
    pub provenance: BTreeMap<FieldAddress, DraftProvenance>,
    pub basis_refs: Vec<GovernedRevision>,
}
#[derive(Clone)]
pub(crate) struct DraftEditScope {
    pub editable: BTreeSet<FieldAddress>,
    pub permitted_anchors: BTreeSet<FieldAddress>,
}
pub(crate) struct DraftSnapshotChange {
    pub snapshot: DraftSnapshot,
    pub changed: bool,
}
#[derive(Clone, PartialEq, Eq)]
pub(crate) enum Patch {
    Set {
        address: FieldAddress,
        value: FieldValue,
    },
    Clear {
        address: FieldAddress,
    },
    Incomplete {
        address: FieldAddress,
        text: String,
    },
    Insert {
        address: FieldAddress,
        after: Option<Uuid>,
        value: FieldValue,
    },
    Remove {
        address: FieldAddress,
    },
    Move {
        address: FieldAddress,
        after: Option<Uuid>,
    },
    Create {
        address: FieldAddress,
        after: Option<Uuid>,
        container: ContainerKind,
    },
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ContainerKind {
    Record,
    List,
}
redacted_debug!(
    FieldAddress,
    FieldBinding,
    DraftCell,
    DraftItem,
    DraftProvenance,
    DraftSnapshot,
    DraftEditScope,
    DraftSnapshotChange,
    Patch,
    ContainerKind
);

impl Patch {
    fn address(&self) -> &FieldAddress {
        match self {
            Self::Set { address, .. }
            | Self::Clear { address }
            | Self::Incomplete { address, .. }
            | Self::Insert { address, .. }
            | Self::Remove { address }
            | Self::Move { address, .. }
            | Self::Create { address, .. } => address,
        }
    }
    fn wire(&self) -> Value {
        use serde_json::json;
        match self {
            Self::Set { address, value } => json!({"kind":"SET","address":address,"value":value}),
            Self::Clear { address } => json!({"kind":"CLEAR","address":address}),
            Self::Incomplete { address, text } => {
                json!({"kind":"SET_INCOMPLETE","address":address,"editor_text":text})
            }
            Self::Insert {
                address,
                after,
                value,
            } => {
                json!({"kind":"INSERT_ITEM","address":address,"after_item_id":after,"value":value})
            }
            Self::Remove { address } => json!({"kind":"REMOVE_ITEM","address":address}),
            Self::Move { address, after } => {
                json!({"kind":"MOVE_ITEM","address":address,"after_item_id":after})
            }
            Self::Create {
                address,
                after,
                container,
            } => {
                json!({"kind":"CREATE_CONTAINER","address":address,"after_item_id":after,"container_kind":match container {ContainerKind::Record=>"RECORD",ContainerKind::List=>"LIST"}})
            }
        }
    }
    fn parse(value: &Value) -> Result<Self, DraftError> {
        check(
            owner28::schema::draft_definition("Patch", value)?,
            "invalid draft patch",
        )?;
        let address: FieldAddress = serde_json::from_value(value["address"].clone())
            .map_err(|_| DraftError("invalid draft address"))?;
        address.validate()?;
        let field_value = || {
            serde_json::from_value(value["value"].clone())
                .map_err(|_| DraftError("invalid draft value"))
        };
        let after = || {
            serde_json::from_value::<Option<Uuid>>(value["after_item_id"].clone())
                .map_err(|_| DraftError("invalid draft anchor"))
        };
        Ok(match value["kind"].as_str() {
            Some("SET") => Self::Set {
                address,
                value: field_value()?,
            },
            Some("CLEAR") => Self::Clear { address },
            Some("SET_INCOMPLETE") => Self::Incomplete {
                address,
                text: value["editor_text"]
                    .as_str()
                    .ok_or(DraftError("invalid editor text"))?
                    .to_owned(),
            },
            Some("INSERT_ITEM") => Self::Insert {
                address,
                after: after()?,
                value: field_value()?,
            },
            Some("REMOVE_ITEM") => Self::Remove { address },
            Some("MOVE_ITEM") => Self::Move {
                address,
                after: after()?,
            },
            Some("CREATE_CONTAINER") => Self::Create {
                address,
                after: after()?,
                container: match value["container_kind"].as_str() {
                    Some("RECORD") => ContainerKind::Record,
                    Some("LIST") => ContainerKind::List,
                    _ => return Err(DraftError("invalid draft container")),
                },
            },
            _ => return Err(DraftError("invalid draft operation")),
        })
    }
}

pub(crate) fn parse_draft_patches(bytes: &[u8]) -> Result<Vec<Patch>, DraftError> {
    let raw = owner28::parse_bounded(bytes, PATCH_BYTES)?;
    let values = raw
        .as_array()
        .ok_or(DraftError("invalid draft patch array"))?;
    check(
        !values.is_empty() && values.len() <= PATCH_COUNT,
        "draft patch count",
    )?;
    values.iter().map(Patch::parse).collect()
}
