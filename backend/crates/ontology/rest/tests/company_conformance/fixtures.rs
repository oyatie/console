//! OWNED index — a lane appends ONE `pub mod` line and owns ONE file below.
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
