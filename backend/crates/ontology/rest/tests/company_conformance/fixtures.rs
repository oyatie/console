//! OWNED index — PRE-RESERVED. All five `pub mod` lines already exist below, so a
//! lane edits ONLY its own `fixtures/<type>.rs` and touches no shared file.
//!
//! Deliberate: "each lane appends one line here" is a claim about agent behaviour, not
//! a merge property. Two lanes appending to one file tail conflict in the same hunk,
//! and git cannot know the lines are independent. Pre-reserving makes the disjointness
//! structural instead of procedural. Adding a SIXTH type is a deliberate act that edits
//! this file once, outside any lane.
//!
//! Everything under `fixtures/` is per-type scenario DATA: the full required
//! param bag for the auto-attached generic `create` action. Nothing else.
//!
//! Two hard rules for a lane's file:
//!   1. The bag is the FULL required param set on EVERY call, including edits.
//!      `apply_edits` resolves an absent param to `Value::Null` and inserts it
//!      over the base (`ontology/application/src/lib.rs:288,296`).
//!   2. The bag contains ONLY declared params. `validate_params` rejects an
//!      undeclared key, and the target asserts every sent param round-trips into
//!      the stored attributes verbatim.
//!
//! The signatures are fixed by the target; the bodies are the lane's.

#[path = "fixtures/company.rs"]
pub mod company;
#[path = "fixtures/employment.rs"]
pub mod employment;
#[path = "fixtures/job_position.rs"]
pub mod job_position;
#[path = "fixtures/org_unit.rs"]
pub mod org_unit;
#[path = "fixtures/pay_run.rs"]
pub mod pay_run;
