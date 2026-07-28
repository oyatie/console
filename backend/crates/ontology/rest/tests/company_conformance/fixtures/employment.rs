//! LANE-EDITABLE — this type's `declare`, and its param bags.
//!
//! Target contract:
//!   * `job_position_id` and `org_unit_id` are `InstanceId` strings. The target
//!     does NOT accept them as mere attributes: it walks the engine's traversal
//!     surface and asserts the employment actually LINKS to that position
//!     (CC-04) and, after the transfer, to the NEW unit only (CC-11).
//!   * `base_salary` is a JSON integer. The target NEVER sends a pay total — it
//!     sums these and asserts the pay run DERIVED the same number (CC-10), so
//!     the two people must carry DIFFERENT non-zero salaries.
//!   * This same bag is reused for the TRANSFER (CC-06): a full param bag on
//!     every revision, because `apply_edits` nulls any property whose param is
//!     absent (`ontology/application/src/lib.rs:288,296`).

use crate::harness::Harness;
use console_ontology_domain::InstanceId;
use serde_json::{Value, json};

/// Base salary per person. Distinct on purpose: CC-10 runs the pay cycle twice
/// over different populations, so no constant can satisfy both totals.
pub fn base_salary(person: &str) -> i64 {
    match person {
        "김정비" => 3_600_000,
        _ => 4_200_000,
    }
}

pub fn hire(person: &str, job_position: InstanceId, org_unit: InstanceId) -> Value {
    json!({
        "person_name": person,
        "job_position_id": job_position.to_string(),
        "org_unit_id": org_unit.to_string(),
        "base_salary": base_salary(person),
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
