//! LANE-EDITABLE — param bags only.
//!
//! Target contract: the bag MUST carry `legal_name` (the target uses it as the
//! instance title and asserts `head.title == legal_name`).

use serde_json::{Value, json};

pub fn found(legal_name: &str) -> Value {
    json!({
        "legal_name": legal_name,
        "registration_number": "123-45-67890",
        "founded_on": "2026-07-10",
    })
}
