//! LANE-EDITABLE — param bags only.
//!
//! Target contract: the bag MUST carry `parent_org_unit_id`, set to the parent's
//! `InstanceId` string, or JSON null for the root. The type's
//! `parent_org_unit_id` property therefore MUST be declared `required: false` —
//! `validate_params` rejects a missing required param, and the root has no
//! parent.

use console_ontology_domain::InstanceId;
use serde_json::{Value, json};

pub fn unit(name: &str, parent: Option<InstanceId>) -> Value {
    json!({
        "name": name,
        "parent_org_unit_id": parent.map(|id| id.to_string()),
    })
}
