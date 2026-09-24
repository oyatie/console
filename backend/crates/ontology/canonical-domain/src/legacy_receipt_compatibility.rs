//! Reader-first compatibility for the existing Company-global receipt store.
//! These pure checks do not authenticate SQL sources or validate payload digests.
use crate::{DispatchTarget, ReceiptOwner};
use serde_json::Value;
use uuid::Uuid;

/// Call before decoding actor_id. SQL must use `to_jsonb(r)->...` so a 0229
/// database lacking both columns is readable. SQL NULL is None; JSON null is
/// Some(Value::Null). Never coalesce them or use `->>` for this projection.
#[must_use]
pub fn legacy_user_receipt_shape(kind: Option<&Value>, account: Option<&Value>) -> bool {
    matches!((kind, account), (None, None))
        || matches!((kind, account), (Some(Value::String(kind)), Some(Value::Null)) if kind == "USER")
}

pub struct LegacyReceiptBinding<'a> {
    pub actor: Option<Uuid>,
    pub owner: &'a str,
    pub target: Option<&'a str>,
    pub action_key: Option<&'a str>,
    pub object_type_id: Option<Uuid>,
}
pub struct LegacyReceiptExpectation<'a> {
    pub actor: Uuid,
    pub target: Option<DispatchTarget>,
    pub action_key: &'a str,
    pub object_type_id: Uuid,
}

/// Run after attribution shape and before the ORIGINAL digest check. A true
/// result is provisional: old digest and result-target validation remain required.
/// Migration0223 gave historical canonical receipts ontology.action/NULL;
/// immutable rows cannot be relabelled. Only that exact pair is compatible.
#[must_use]
pub fn legacy_receipt_binding(
    stored: &LegacyReceiptBinding<'_>,
    expected: &LegacyReceiptExpectation<'_>,
) -> bool {
    let owner_matches = match expected.target {
        Some(target) => {
            (stored.owner == ReceiptOwner::Canonical(target.object()).as_str()
                && stored.target == Some(target.as_str()))
                || (stored.owner == ReceiptOwner::OntologyAction.as_str()
                    && stored.target.is_none())
        }
        None => stored.owner == ReceiptOwner::OntologyAction.as_str() && stored.target.is_none(),
    };
    owner_matches
        && stored.actor == Some(expected.actor)
        && stored
            .action_key
            .is_none_or(|key| key == expected.action_key)
        && stored
            .object_type_id
            .is_none_or(|id| id == expected.object_type_id)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LegacyReceiptTargetError {
    /// Preserve canonical owners' existing UnreadableReceipt envelope.
    Unreadable,
    /// Use the caller's existing nondisclosing conflict envelope.
    Conflict,
}

/// Run after the existing digest check and before any replay or audit repair.
/// The REST canonical peek also checks this before bypassing spent-approval
/// gates; the actual owning port must still compare its original digest.
pub fn legacy_receipt_result_target(
    expected: Option<DispatchTarget>,
    result: &Value,
) -> Result<(), LegacyReceiptTargetError> {
    match expected {
        Some(expected) => {
            let actual = result
                .get("target")
                .and_then(Value::as_str)
                .and_then(|value| value.parse::<DispatchTarget>().ok())
                .ok_or(LegacyReceiptTargetError::Unreadable)?;
            if actual == expected {
                Ok(())
            } else {
                Err(LegacyReceiptTargetError::Conflict)
            }
        }
        None if result.get("target").is_none() => Ok(()),
        None => Err(LegacyReceiptTargetError::Conflict),
    }
}

#[cfg(test)]
#[path = "legacy_receipt_compatibility_tests.rs"]
mod tests;
