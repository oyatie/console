//! LANE-EDITABLE — this type's `declare`, and its param bags.
//!
//! Target contract: the bag carries `employment_ids` (a JSON array of
//! `InstanceId` strings — declare the property `multi_choice` or `json`, since
//! `check_field_shape` requires an array for `multi_choice` and accepts anything
//! for `json`) plus the period bounds.
//!
//! IT MUST NOT CARRY THE PAY TOTAL. `gross_total` is DERIVED from the referenced
//! employments' `base_salary`; the target asserts the computed value and REJECTS
//! a bag that sends it (CC-10). Sending it is the one edit that would turn a
//! derivation assertion back into a round-trip assertion — exactly the vacuity
//! this suite exists to prevent.

use crate::harness::Harness;
use console_ontology_domain::InstanceId;
use serde_json::{Value, json};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

pub fn cycle(
    employments: &[InstanceId],
    period_start: OffsetDateTime,
    period_end: OffsetDateTime,
) -> Value {
    json!({
        "employment_ids": employments.iter().map(ToString::to_string).collect::<Vec<_>>(),
        "period_start": period_start.format(&Rfc3339).unwrap_or_default(),
        "period_end": period_end.format(&Rfc3339).unwrap_or_default(),
    })
}

/// PRE-RESERVED, LANE-OWNED — no-op until this lane lands.
///
/// A no-op is the correct unbuilt state: the type is never created, so
/// `resolve_type` fails with the pinned `UNKNOWN_TYPE` signature and this type's
/// scenario ids stay RED for the one reason the target accepts. Anything else
/// here (a partial type, a draft that never publishes) produces a red that reads
/// like a lane defect — note that `get_object_type` orders by
/// `(lifecycle_state = 'published') DESC` with NO lifecycle filter
/// (`adapter-postgres/src/lib.rs:620-634`), so an unpublished draft still
/// RESOLVES and would turn a green that the engine cannot actually act on.
///
/// The lane replaces this body with the 4-step publish — `create_object_type`,
/// `transition_lifecycle(ReviewPending)`, `create_approval`/`decide_approval` of
/// kind `ontology.schema.publish`, `transition_lifecycle(Published)` — run
/// inside `scope_org(h.org, ..)` against `h.registry()`. Author NO action: the
/// publish auto-attaches the generic `create`, and a hand-authored one suppresses
/// it (`0165_ontology_object_type_key_revisions.sql:1024-1029`).
///
/// Four-eyes is enforced three ways, so `h.admin` requests and `h.approver`
/// decides: `gov_approvals CHECK (approver_id <> requested_by)`
/// (`0153_create_governance.sql:74`), `decide_approval`'s own check, and the
/// publish gate's `ga.requested_by = p_actor`. Approvals are single-use, so each
/// lane needs its own `request_ref`.
pub async fn declare(_h: &Harness) {}
