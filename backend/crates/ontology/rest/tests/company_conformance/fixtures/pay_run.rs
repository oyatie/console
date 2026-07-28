//! LANE-EDITABLE — param bags only.
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
