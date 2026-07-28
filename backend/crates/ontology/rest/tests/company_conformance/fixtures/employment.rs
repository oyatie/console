//! LANE-EDITABLE — param bags only.
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
