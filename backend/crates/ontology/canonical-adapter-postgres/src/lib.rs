//! The owning Postgres write adapter for four canonical objects.
//!
//! `backend/crates/ontology/canonical-domain/src/lib.rs` names this crate,
//! verbatim, as `owner` for exactly four of its six `ObjectKey`s:
//!
//! | `ObjectKey`   | key             | owned tables |
//! |---------------|-----------------|--------------|
//! | `Company`     | `company`       | `organizations`, `company_revisions` |
//! | `OrgUnit`     | `org_unit`      | `org_units`, `org_unit_revisions`, `org_unit_source_bindings` |
//! | `JobPosition` | `job_position`  | `job_positions`, `job_position_revisions` |
//! | `Person`      | `person`        | `persons`, `person_revisions`, `employee_person_bindings` |
//!
//! The other two objects are owned elsewhere and no part of them belongs here:
//! `Employment` by `console-orgchange-adapter-postgres` (documented INTERIM in
//! the contract, retargeted by the `EmploymentPort` lane) and `PayRun` by
//! `console-payroll-adapter-postgres`, which already exists and is wrapped
//! rather than replaced. `ObjectKey` is locked at six by
//! `six_projected_stable_object_keys_verbatim`.
//!
//! `console-gate-writer-ownership` reads that registry and rejects production
//! DML against an owned table held by any crate other than its owner. This
//! crate is where that DML is permitted to live.
//!
//! # The one-word-away trap
//!
//! `backend/crates/ontology/adapter-postgres` is
//! `console-ontology-adapter-postgres` — the ontology METAMODEL adapter. It is
//! real, populated, sits in this same directory, and is ONE hyphenated word
//! (`canonical`) away from this crate's name. It is NOT the owner of anything
//! above. Canonical DML added there compiles fine and enforces nothing, because
//! the gate matches the owner name exactly. If you are about to write a
//! canonical write and the path you have open does not contain
//! `canonical-adapter-postgres`, you have the wrong crate.
//!
//! There is also no `backend/crates/company` and no per-object crate: every
//! port is a MODULE in this one shared crate, which is why the module lines
//! below are pre-declared. Filling them is each lane's work.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod company;
pub mod job_position;
pub mod org_unit;
pub mod person;
