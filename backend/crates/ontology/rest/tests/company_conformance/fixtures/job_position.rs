//! LANE-EDITABLE — param bags only.
//!
//! Target contract: the bag MUST carry `org_unit_id` (an `InstanceId` string) and
//! `headcount` (a JSON integer — the target asserts `.as_i64()`, so the property
//! must be declared `integer`).

use console_ontology_domain::InstanceId;
use serde_json::{Value, json};

pub fn position(title: &str, org_unit: InstanceId, headcount: i64) -> Value {
    json!({
        "job_title": title,
        "org_unit_id": org_unit.to_string(),
        "headcount": headcount,
    })
}
