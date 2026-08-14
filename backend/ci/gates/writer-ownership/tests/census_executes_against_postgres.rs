//! The DATABASE half, bound to an EXECUTED PostgreSQL — not to the script's text.
//!
//! `gate_detects_violation.rs` asserts things about the *characters* in
//! `ops/postgres-reconcile-topology.sh`. That is the same failure class as a
//! grep-shaped gate over Rust: changing `IF leaked IS NOT NULL THEN` to
//! `IF leaked IS NOT NULL AND false THEN` kills the entire deny-by-default
//! census and every text assertion stays green. This file runs the real script
//! against a real PostgreSQL and asserts on what the database ends up doing.
//!
//! ## What runs here
//!
//! `census_binds_to_an_executed_database` drives one container through:
//!
//! 1. NO canonical table exists and the run CLAIMS to enforce
//!    (`CONSOLE_TOPOLOGY_REQUIRE_CANONICAL_TABLES=1`) — the run must FAIL.
//! 2. Migrations applied, then enforcement — must PASS, and the SET of tables
//!    it examined must be exactly [`EXPECTED_REQUIRED_TABLES`]. Not "more than
//!    zero": renaming one canonical table away used to shrink the scope from
//!    eight to seven and still pass.
//! 3. [`PROBES`] — one disposable probe database each, one per shape a writer
//!    can take: a table grant, a COLUMN grant (invisible to
//!    `has_table_privilege`), a TRUNCATE-only grant (which has no column form),
//!    a grant on a PARTITIONED canonical table, one on a partition CHILD
//!    (whose relname the roster does not contain) and one on a relation a
//!    canonical table is made a CHILD of, an unexpected table OWNER, an owner
//!    that is already on the expected-WRITER ratchet (ownership is DDL
//!    authority, not DML, so the ratchet may not authorise it),
//!    membership in `pg_write_all_data` both INHERIT and NOINHERIT, an
//!    unexpected SUPERUSER, a canonical table RENAMED away, MOVED to another
//!    schema behind a decoy, parked in `information_schema`, and replaced by a
//!    VIEW.
//!
//!    A probe that needs a roster name in a shape migrations do not produce now
//!    TAKES the name — `DROP TABLE IF EXISTS ... CASCADE` and recreate. It used
//!    to borrow a roster name no migration had created yet, and that was moved
//!    twice (off `persons` at 0213, off `employment_heads` at 0214) before
//!    migration 0215 completed the roster and left nothing to borrow.
//!
//!    The tenth shape, membership in `console_app` itself, is REPAIRED by the
//!    reconcile rather than rejected by it — the exact-topology REVOKE strips
//!    every membership edge touching an application role before the canonical
//!    block runs — so phase 2b proves it by breaking that REVOKE instead.
//! 4. [`MUTATIONS`] — each one applied to a COPY of the script, run against the
//!    probe that is supposed to catch it, and required to FLIP that probe's
//!    verdict. A control that keeps its verdict when the control is broken is
//!    not testing anything. These used to live in a `mutation-check.sh` that
//!    nothing invoked, so they ran only when a human remembered. Two are
//!    measured against an UNARMED reconcile, with their own committed-script
//!    baseline; [`Mutation::armed`] says why the completed roster leaves them no
//!    armed configuration in which they flip anything.
//!
//! `pgtest_harness_executes_the_enforcement` and
//! `cargo_needs_postgres_harness_executes_the_enforcement` prove the WIRING by
//! running each harness and observing the enforcement's own output line. The
//! previous spelling was `harness_text.contains("canonical-enforce.sh")`, which
//! a `#` in front of the invocation satisfies — a grep-shaped gate rebuilt
//! inside the guard meant to prove the gate is not grep-shaped.
//!
//! ## Docker
//!
//! Every test here REQUIRES Docker and FAILS without it. It used to `return
//! Ok(())` after an `eprintln!` banner, but cargo captures stdio from PASSING
//! tests and replays it only for failures, so the banner was never printed: a
//! machine with no Docker reported `test result: ok. 2 passed`, byte-identical
//! to a fully verified run, having executed nothing. An unexecuted census must
//! not be able to look like a verified one, so it is a failure instead.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Same digest the two harnesses pin.
const IMAGE: &str =
    "postgres:18.4@sha256:65f70a152846cf504dff86e807007e9aeac98c3aeb7b62541b2c55ab9d264e56";

/// The canonical tables a fully migrated database has TODAY, and therefore the
/// exact set the census must examine. This list is a VERBATIM PIN, not a second
/// source: the census scope is DERIVED from the `required_tables` array in
/// `ops/postgres-reconcile-topology.sh` by [`required_tables_from_topology`],
/// and [`derived_required_tables_match_the_verbatim_roster`] fails the run when
/// the derived set and this pin diverge. A lane that lands a new canonical
/// table edits the shell array and this pin — two edits in two files, instead
/// of a fixed-arity const plus two shared arrays that four port lanes had to
/// hand-append in lockstep.
///
/// As of migration 0215 this is the WHOLE roster — every name in
/// `ObjectKey::owned_tables` now resolves to a real relation, so the set can no
/// longer grow without a seventh object key. `employee_person_bindings`,
/// `person_revisions` and `persons` landed with migration 0213 (the
/// `ObjectKey::Person` tables); `employment_heads`, `employment_revisions` and
/// `employment_source_bindings` with 0214 (the three `ObjectKey::Employment`
/// tables that did not already exist — `employees` is the fourth and predates
/// both); `company_revisions`, `org_units`, `org_unit_revisions`,
/// `org_unit_source_bindings`, `job_positions` and `job_position_revisions`
/// with 0215 (the six of `ObjectKey::Company`, `ObjectKey::OrgUnit` and
/// `ObjectKey::JobPosition` that did not already exist — `organizations` is the
/// seventh and predates all three migrations). Sorted, because [`examined_set`]
/// sorts what it reads back.
const EXPECTED_REQUIRED_TABLES: [&str; 20] = [
    "company_revisions",
    "employee_person_bindings",
    "employees",
    "employment_heads",
    "employment_revisions",
    "employment_source_bindings",
    "job_position_revisions",
    "job_positions",
    "org_unit_revisions",
    "org_unit_source_bindings",
    "org_units",
    "organizations",
    "payroll_disbursements",
    "payroll_draft_lines",
    "payroll_draft_runs",
    "payroll_line_calculations",
    "payroll_payslip_deliveries",
    "payroll_run_exceptions",
    "person_revisions",
    "persons",
];

type Fallible = Result<(), Box<dyn std::error::Error>>;
/// Derives the census scope from the `required_tables` array in the topology
/// script — the single production source — instead of a hand-maintained const.
/// Isolates the `DO $canonical$` block first (whole-line opener, same as
/// `canonical_block` in `gate_detects_violation.rs`), then the
/// `required_tables CONSTANT TEXT[] := ARRAY[...]` declaration inside that
/// block, then the `'name'` entries between its BEGIN/END markers. A decoy
/// marker or `ARRAY[` statement earlier in the shell script cannot win. Sorts
/// the names (the census compares against a sorted `examined_set`) and fails
/// loudly when the block, the statement, the markers, or the entries are
/// missing rather than examining a silently shrunk scope.
fn required_tables_from_topology(script: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let text =
        std::fs::read_to_string(script).map_err(|e| format!("read {}: {e}", script.display()))?;
    // Whole-line opener: `DO $canonical$` also appears in a comment earlier
    // ("Read back by the DO $canonical$ block below"). A bare substring split
    // would pull those preceding lines into "the block".
    let (_, rest) = text
        .split_once("\nDO $canonical$\n")
        .ok_or_else(|| format!("missing `DO $canonical$` block in {}", script.display()))?;
    let (block, _) = rest.split_once("$canonical$;").ok_or_else(|| {
        format!(
            "unterminated `DO $canonical$` block in {}",
            script.display()
        )
    })?;
    let statement = "required_tables CONSTANT TEXT[] := ARRAY[";
    let statement_at = block
        .find(statement)
        .ok_or_else(|| format!("missing `{statement}` in {}", script.display()))?;
    let begin = "-- canonical-writer-ownership: BEGIN required tables";
    let end = "-- canonical-writer-ownership: END required tables";
    let declared = &block[statement_at..];
    let start = declared
        .find(begin)
        .ok_or_else(|| format!("missing `{begin}` marker in {}", script.display()))?;
    let tail = &declared[start..];
    let stop = tail
        .find(end)
        .ok_or_else(|| format!("missing `{end}` marker in {}", script.display()))?;
    let mut names = Vec::new();
    for line in tail[..stop].lines() {
        let line = line.trim();
        if let Some(name) = line
            .strip_prefix('\'')
            .and_then(|rest| rest.split('\'').next())
            && !name.is_empty()
        {
            names.push(name.to_string());
        }
    }
    if names.is_empty() {
        return Err(format!("no required tables extracted from {}", script.display()).into());
    }
    names.sort();
    names.dedup();
    Ok(names)
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../..")
}

fn gate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn topology_script() -> PathBuf {
    repo_root().join("ops/postgres-reconcile-topology.sh")
}

fn run(program: &str, args: &[&str]) -> Result<(bool, String), Box<dyn std::error::Error>> {
    run_env(program, args, &[])
}

fn run_env(
    program: &str,
    args: &[&str],
    env: &[(&str, &str)],
) -> Result<(bool, String), Box<dyn std::error::Error>> {
    let mut command = Command::new(program);
    command.args(args);
    for (key, value) in env {
        command.env(key, value);
    }
    let output = command.output()?;
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    Ok((output.status.success(), text))
}

fn docker_available() -> bool {
    Command::new("docker")
        .arg("version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

/// Docker is a requirement, not a nicety. See the module doc: skipping silently
/// is how a run that executed nothing certified the database half as green.
fn require_docker() {
    assert!(
        docker_available(),
        "Docker is REQUIRED by this test and is absent. This file is the only \
         thing that executes the canonical writer-ownership census; skipping it \
         would report a pass having verified nothing, which is exactly the \
         fail-open it exists to close. Run it where Docker is available."
    );
}

struct Container(String);

impl Drop for Container {
    fn drop(&mut self) {
        let _ = Command::new("docker")
            .args(["rm", "-fv", &self.0])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}

/// Six distinct 64-hex secrets, derived without a crypto dependency. The
/// reconcile refuses non-distinct passwords, and these never leave the
/// container's env file.
fn secrets() -> Vec<String> {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0u128, |d| d.as_nanos());
    (0..6u128)
        .map(|i| {
            let seed = nanos
                .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                .wrapping_add(i.wrapping_mul(0x1234_5678_9ABC_DEF1));
            format!("{seed:032x}{i:032x}")
        })
        .collect()
}

fn unique(prefix: &str) -> String {
    format!(
        "{prefix}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos())
    )
}

// ---------------------------------------------------------------------------
// The rogue probes, and the mutations that must flip each one
// ---------------------------------------------------------------------------

/// One planted violation and the reconcile's required reaction to it.
struct Probe {
    /// Probe-database suffix; also the label in failures.
    label: &'static str,
    /// Applied as the cluster admin AFTER migrations, BEFORE enforcement.
    sql: &'static str,
    /// The `topology.*` code the reconcile must raise.
    error: &'static str,
    /// The offending object or role the message must name.
    names: &'static str,
}

/// Every shape an unaccounted writer can take on a canonical table. Each is
/// planted on its own probe database, because planting them together would let
/// one detection carry the others.
///
/// Six of these used to be caught by six DIFFERENT guards — an ACL union, a
/// column-ACL union, an owner check, a `pg_auth_members` check, a `rolsuper`
/// exclusion. They are now all the same finding, because they are all the same
/// question: can this role write this relation?
const PROBES: [Probe; 16] = [
    Probe {
        // OWNERSHIP is not the DML question, and the census's ratchet is a
        // (role, relation) whitelist of who may WRITE. `console_rt` is on that
        // ratchet — it is the runtime login principal — so folding ownership
        // into the census made this plant invisible: the census excludes the
        // candidate by NAME before it ever asks the privilege question. An
        // owner also holds ALTER, DROP, TRUNCATE and, decisively for a table
        // whose tenant isolation is entirely RLS, `DISABLE ROW LEVEL SECURITY`
        // and `DROP POLICY`. The owner is pinned by its own list.
        label: "owner_is_a_ratcheted_writer",
        sql: "ALTER TABLE public.employees OWNER TO console_rt;\n",
        error: "topology.canonical_table_owner_failed",
        names: "employees:console_rt",
    },
    Probe {
        // Reachability runs DOWN to children and UP to parents. A canonical
        // table made the CHILD of a new relation is written through that
        // parent, which holds no privilege on `employees` at all — so the
        // deny-by-default census reports clean while the rows disappear.
        label: "inheritance_parent",
        sql: "DROP ROLE IF EXISTS console_rogue_parent;\n\
              CREATE ROLE console_rogue_parent NOLOGIN;\n\
              CREATE TABLE public.rogue_super_tbl (id uuid);\n\
              ALTER TABLE public.rogue_super_tbl OWNER TO console_app;\n\
              ALTER TABLE public.employees INHERIT public.rogue_super_tbl;\n\
              GRANT UPDATE, DELETE ON public.rogue_super_tbl TO console_rogue_parent;\n",
        error: "topology.canonical_writer_ownership_failed",
        names: "console_rogue_parent",
    },
    Probe {
        // The real, data-bearing table moves to another schema and a same-named
        // decoy is left in `public`. Nothing disappears, so a name-matched
        // census reports clean while the rogue keeps DML on the real rows.
        label: "schema_moved_away",
        sql: "DROP ROLE IF EXISTS console_rogue_schema;\n\
              CREATE ROLE console_rogue_schema NOLOGIN;\n\
              CREATE SCHEMA shadow;\n\
              ALTER TABLE public.employees SET SCHEMA shadow;\n\
              CREATE TABLE public.employees (id uuid);\n\
              ALTER TABLE public.employees OWNER TO console_app;\n\
              GRANT INSERT, UPDATE, DELETE ON shadow.employees TO console_rogue_schema;\n",
        error: "topology.canonical_roster_unresolved",
        names: "shadow.employees",
    },
    Probe {
        // The same shrink, parked in the ONE schema the previous design
        // hard-excluded from its search. A roster name is a name wherever it
        // lives, so the resolution has no exclusion list at all.
        label: "information_schema_park",
        sql: "DROP ROLE IF EXISTS console_rogue_infoschema;\n\
              CREATE ROLE console_rogue_infoschema NOLOGIN;\n\
              CREATE TABLE information_schema.persons (id uuid);\n\
              GRANT INSERT, UPDATE, DELETE ON information_schema.persons TO console_rogue_infoschema;\n",
        error: "topology.canonical_roster_unresolved",
        names: "information_schema.persons",
    },
    Probe {
        // DML held by MEMBERSHIP rather than by a grant. `pg_write_all_data`
        // writes every table in the cluster and appears in no `relacl`.
        label: "write_all_data",
        sql: "DROP ROLE IF EXISTS console_rogue_all_data;\n\
              CREATE ROLE console_rogue_all_data NOLOGIN;\n\
              GRANT pg_write_all_data TO console_rogue_all_data;\n",
        error: "topology.canonical_writer_ownership_failed",
        names: "console_rogue_all_data",
    },
    Probe {
        // The same membership, spelled NOINHERIT. `has_table_privilege` reports
        // FALSE for this role — it must SET ROLE first — and it can, so the
        // census asks whether the candidate can BECOME a holder, not whether it
        // inherits from one.
        label: "noinherit_member",
        sql: "DROP ROLE IF EXISTS console_rogue_noinherit;\n\
              CREATE ROLE console_rogue_noinherit NOLOGIN NOINHERIT;\n\
              GRANT pg_write_all_data TO console_rogue_noinherit;\n",
        error: "topology.canonical_writer_ownership_failed",
        names: "console_rogue_noinherit",
    },
    Probe {
        // A SUPERUSER writes every canonical table and is checked against no
        // ACL at all, so the previous census EXCLUDED superusers and reported
        // clean. The trusted set is now named instead.
        label: "superuser",
        sql: "DROP ROLE IF EXISTS console_rogue_super;\n\
              CREATE ROLE console_rogue_super SUPERUSER NOLOGIN;\n",
        error: "topology.canonical_writer_ownership_failed",
        names: "console_rogue_super",
    },
    Probe {
        label: "table_level",
        sql: "DROP ROLE IF EXISTS console_rogue_table;\n\
              CREATE ROLE console_rogue_table NOLOGIN;\n\
              GRANT UPDATE ON public.employees TO console_rogue_table;\n",
        error: "topology.canonical_writer_ownership_failed",
        names: "console_rogue_table",
    },
    Probe {
        // `has_table_privilege` answers FALSE for a column-only grant, which is
        // why the census asks `has_any_column_privilege` for the two DML
        // privileges PostgreSQL lets a grant scope to a column.
        label: "column_level",
        sql: "DROP ROLE IF EXISTS console_rogue_column;\n\
              CREATE ROLE console_rogue_column NOLOGIN;\n\
              GRANT UPDATE (org_unit) ON public.employees TO console_rogue_column;\n",
        error: "topology.canonical_writer_ownership_failed",
        names: "console_rogue_column",
    },
    Probe {
        // The other branch: TRUNCATE has no column form, so only the table
        // question can see it, and it empties the table.
        label: "truncate_only",
        sql: "DROP ROLE IF EXISTS console_rogue_truncate;\n\
              CREATE ROLE console_rogue_truncate NOLOGIN;\n\
              GRANT TRUNCATE ON public.employees TO console_rogue_truncate;\n",
        error: "topology.canonical_writer_ownership_failed",
        names: "console_rogue_truncate",
    },
    Probe {
        // relkind 'p'. Partitioning is already live in this tree, and a
        // relkind='r' filter drops a partitioned canonical table out of both the
        // REVOKE loop and the census.
        //
        // The plant used to require a roster name NO migration creates, and was
        // moved twice for it: off `employment_heads` when 0214 created that
        // table, onto `job_positions`, which 0215 has now created too. There is
        // no third name — 0215 completed the roster, so "the next roster name
        // that is still absent" is a set that is now permanently empty and
        // moving the plant again is not available to anyone. The plant TAKES the
        // name instead: `DROP TABLE ... CASCADE` first, then recreate it
        // partitioned. That works whether or not a migration created it, so no
        // future lane has to touch this probe again.
        label: "partitioned",
        sql: "DROP ROLE IF EXISTS console_rogue_partition;\n\
              CREATE ROLE console_rogue_partition NOLOGIN;\n\
              DROP TABLE IF EXISTS public.job_positions CASCADE;\n\
              CREATE TABLE public.job_positions (id uuid NOT NULL, org_id uuid NOT NULL)\n\
                PARTITION BY LIST (org_id);\n\
              ALTER TABLE public.job_positions OWNER TO console_app;\n\
              GRANT UPDATE ON public.job_positions TO console_rogue_partition;\n",
        error: "topology.canonical_writer_ownership_failed",
        names: "console_rogue_partition",
    },
    Probe {
        // The CHILD. Rows written through a partition land in the parent, and
        // the child carries a relname the roster does not contain, so it is
        // invisible to any census that matches on names. Reachability through
        // pg_inherits is the only thing that finds it.
        label: "partition_child",
        sql: "DROP ROLE IF EXISTS console_rogue_child;\n\
              CREATE ROLE console_rogue_child NOLOGIN;\n\
              DROP TABLE IF EXISTS public.job_positions CASCADE;\n\
              CREATE TABLE public.job_positions (id uuid NOT NULL, org_id uuid NOT NULL)\n\
                PARTITION BY LIST (org_id);\n\
              ALTER TABLE public.job_positions OWNER TO console_app;\n\
              CREATE TABLE public.job_positions_p1 PARTITION OF public.job_positions\n\
                FOR VALUES IN ('00000000-0000-0000-0000-000000000000');\n\
              ALTER TABLE public.job_positions_p1 OWNER TO console_app;\n\
              GRANT UPDATE ON public.job_positions_p1 TO console_rogue_child;\n",
        error: "topology.canonical_writer_ownership_failed",
        names: "console_rogue_child:job_positions_p1",
    },
    Probe {
        // The owner's DML is implicit: it is not in relacl and REVOKE cannot
        // remove it. The previous census EXCLUDED the owner for exactly that
        // reason, so re-owning a canonical table read as clean.
        label: "owner",
        sql: "DROP ROLE IF EXISTS console_rogue_owner;\n\
              CREATE ROLE console_rogue_owner NOLOGIN;\n\
              ALTER TABLE public.employees OWNER TO console_rogue_owner;\n",
        error: "topology.canonical_writer_ownership_failed",
        names: "console_rogue_owner",
    },
    Probe {
        // A partial loss of the roster. A non-empty count passes this: it falls
        // by one and the run reports success on what is left.
        label: "renamed_away",
        sql: "DROP ROLE IF EXISTS console_rogue_rename;\n\
              CREATE ROLE console_rogue_rename NOLOGIN;\n\
              ALTER TABLE public.employees RENAME TO employees_v2;\n\
              GRANT INSERT, UPDATE, DELETE ON public.employees_v2 TO console_rogue_rename;\n",
        error: "topology.canonical_roster_incomplete",
        names: "employees",
    },
    Probe {
        // Same shrink, spelled so the canonical NAME still resolves: a view is
        // writable through and is not a table.
        label: "view_swap_existing",
        sql: "ALTER TABLE public.employees RENAME TO employees_v2;\n\
              CREATE VIEW public.employees AS SELECT * FROM public.employees_v2;\n",
        error: "topology.canonical_roster_unresolved",
        names: "employees:v",
    },
    Probe {
        // A roster name landing as a VIEW instead of a table. Step 1a's relkind
        // branch is what refuses it, and it refuses it BEFORE the required-set
        // check of step 1c runs, which is why the expected error is
        // `unresolved` and not `incomplete`.
        //
        // Renamed from `view_swap_future`: it was moved off `persons` when 0213
        // created that table and onto `org_units`, which 0215 has now created
        // too. Like `partitioned`, it TAKES the name — `DROP TABLE ... CASCADE`
        // then `CREATE VIEW` — instead of borrowing one no migration has
        // claimed yet, because after 0215 there are none left to borrow.
        label: "view_swap_roster",
        sql: "DROP TABLE IF EXISTS public.org_units CASCADE;\n\
              CREATE VIEW public.org_units AS SELECT 1 AS id;\n",
        error: "topology.canonical_roster_unresolved",
        names: "org_units:v",
    },
];

/// A single-`sed` edit to the reconcile, the probe it must break, and the
/// verdict that probe must then return.
struct Mutation {
    label: &'static str,
    /// `sed` expression applied to a COPY; the committed file is never written.
    expression: &'static str,
    /// [`PROBES`] label, or `""` for the clean migrated database.
    probe: &'static str,
    /// What the probe must report once the control is broken. Every entry but
    /// `drop-expected-writer` flips a detection to a silent pass.
    expect_pass: bool,
    /// Whether this mutation is measured with
    /// `CONSOLE_TOPOLOGY_REQUIRE_CANONICAL_TABLES=1`.
    ///
    /// TRUE for all but two, and the exception is a consequence of migration
    /// 0215 rather than a convenience. Armed, step 1c demands that every
    /// `required_tables` entry exist as an `'r'`/`'p'` relation in `public`, and
    /// as of 0215 that list is the WHOLE roster — so 1c now refuses a roster
    /// name that has been turned into a view or dropped out of the relkind
    /// filter, and it refuses it no matter what step 1a or 1b do. Two controls
    /// therefore have no armed configuration in which breaking them changes the
    /// verdict, and pairing them with a probe that "fails either way" would be a
    /// mutation that has stopped testing what it names.
    ///
    /// Their unique coverage is the UNARMED reconcile — the one every automated
    /// path runs BEFORE migrations, where 1c does not execute at all — so that
    /// is where they are measured. The loop runs the committed script on the
    /// same probe unarmed first and requires it to FAIL, so the flip is between
    /// two runs that differ only in the mutation.
    armed: bool,
}

const MUTATIONS: [Mutation; 14] = [
    Mutation {
        // The owner pin, which the expected-writer ratchet may not widen.
        label: "neuter-owner-check",
        expression: "s/WHERE pg_get_userbyid(relation.relowner) <> ALL (expected_owners)/WHERE false/",
        probe: "owner_is_a_ratcheted_writer",
        expect_pass: true,
        armed: true,
    },
    Mutation {
        // The UP half of reachability.
        label: "no-inheritance-parents",
        expression: "s/SELECT inheritance.inhrelid AS held, inheritance.inhparent AS reached/SELECT NULL::oid AS held, NULL::oid AS reached/",
        probe: "inheritance_parent",
        expect_pass: true,
        armed: true,
    },
    Mutation {
        // The census still runs and still computes `leaked`, but can never raise.
        label: "invert-census",
        expression: "s/IF leaked IS NOT NULL THEN/IF leaked IS NOT NULL AND false THEN/",
        probe: "table_level",
        expect_pass: true,
        armed: true,
    },
    Mutation {
        // The privilege question itself.
        label: "neuter-privilege-question",
        expression: "s/                AND CASE/                AND false AND CASE/",
        probe: "table_level",
        expect_pass: true,
        armed: true,
    },
    Mutation {
        // Ask the TABLE question where a COLUMN grant lives.
        label: "drop-column-privilege-question",
        expression: "s/THEN has_any_column_privilege(holder.oid, relation.oid, privilege.name)/THEN has_table_privilege(holder.oid, relation.oid, privilege.name)/",
        probe: "column_level",
        expect_pass: true,
        armed: true,
    },
    Mutation {
        // ... and the branch that carries DELETE and TRUNCATE.
        label: "neuter-table-privilege-question",
        expression: "s/ELSE has_table_privilege(holder.oid, relation.oid, privilege.name)/ELSE false/",
        probe: "truncate_only",
        expect_pass: true,
        armed: true,
    },
    Mutation {
        // Ask only what the role INHERITS instead of what it can BECOME.
        label: "drop-membership-reachability",
        expression: "s/WHERE pg_has_role(candidate.oid, holder.oid, 'MEMBER')/WHERE candidate.oid = holder.oid/",
        probe: "noinherit_member",
        expect_pass: true,
        armed: true,
    },
    Mutation {
        // The exclusion the previous census actually shipped.
        label: "exclude-superusers",
        expression: "s/AND candidate.rolname <> session_user/AND NOT candidate.rolsuper/",
        probe: "superuser",
        expect_pass: true,
        armed: true,
    },
    Mutation {
        // console_rt loses its ratchet entry, so the CLEAN database must stop
        // passing. This is what proves the clean phase is not vacuous.
        label: "drop-expected-writer",
        expression: "/('console_rt', '\\*'),/d",
        probe: "",
        expect_pass: false,
        armed: true,
    },
    Mutation {
        label: "no-partition-children",
        expression: "s/SELECT inheritance.inhparent AS held, inheritance.inhrelid AS reached/SELECT NULL::oid AS held, NULL::oid AS reached/",
        probe: "partition_child",
        expect_pass: true,
        armed: true,
    },
    Mutation {
        label: "relkind-r-only",
        expression: "s/AND relation.relkind IN ('r', 'p')/AND relation.relkind = 'r'/g",
        probe: "partitioned",
        expect_pass: true,
        armed: false,
    },
    Mutation {
        label: "allow-any-schema",
        expression: "s/HAVING bool_or(namespace.nspname <> 'public')/HAVING bool_or(false)/",
        probe: "information_schema_park",
        expect_pass: true,
        armed: true,
    },
    Mutation {
        label: "allow-any-relkind",
        expression: "s/OR bool_or(relation.relkind NOT IN ('r', 'p'))/OR bool_or(false)/",
        probe: "view_swap_roster",
        expect_pass: true,
        armed: false,
    },
    Mutation {
        label: "empty-required-set",
        expression: "s/FROM unnest(required_tables) AS wanted(name)/FROM unnest(ARRAY[]::TEXT[]) AS wanted(name)/",
        probe: "renamed_away",
        expect_pass: true,
        armed: true,
    },
];

/// A rogue role granted MEMBERSHIP in `console_app`, the owner of every
/// canonical table. It inherits the owner's implicit DML: no `relacl` entry, no
/// REVOKE on the table that removes it, and `relowner` unchanged, so neither the
/// ACL census nor the owner guard can see it.
const MEMBERSHIP_PLANT: &str = "DROP ROLE IF EXISTS console_rogue_member;\n\
     CREATE ROLE console_rogue_member NOLOGIN;\n\
     GRANT console_app TO console_rogue_member;\n";

/// The exact-topology membership REVOKE, turned into a harmless SELECT so that
/// no membership edge is removed. This is the control that actually covers
/// [`MEMBERSHIP_PLANT`].
const KEEP_MEMBERSHIPS: &str = "s/SELECT format('REVOKE %I FROM %I', granted.rolname, member.rolname)/SELECT format('SELECT %L, %L', granted.rolname, member.rolname)/";

/// A copy of the reconcile with one `sed` expression applied, asserted to differ
/// from the committed file so that a moved anchor cannot silently test the
/// original.
fn mutant(
    workdir: &Path,
    label: &str,
    expression: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let path = workdir.join(format!("{label}.sh"));
    let output = Command::new("sed")
        .args([expression, &topology_script().display().to_string()])
        .output()?;
    assert!(
        output.status.success(),
        "{label}: sed failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    std::fs::write(&path, &output.stdout)?;
    assert_ne!(
        std::fs::read(&path)?,
        std::fs::read(topology_script())?,
        "{label}: the mutation changed nothing — its anchor moved, so it has been \
         silently testing the unmutated script"
    );
    Ok(path)
}

// ---------------------------------------------------------------------------
// The executed census
// ---------------------------------------------------------------------------

struct Harness {
    container: Container,
    workdir: PathBuf,
}

impl Harness {
    /// Runs the post-migration enforcement on a fresh probe database.
    /// `extra` is `--skip-migrations`, a path to SQL applied after migrations,
    /// or `None`. `armed` selects
    /// `CONSOLE_TOPOLOGY_REQUIRE_CANONICAL_TABLES`; every phase but the two
    /// unarmed mutations passes `true`, which is what every automated path does
    /// post-migration.
    fn enforce(
        &self,
        probe: &str,
        extra: Option<&str>,
        armed: bool,
    ) -> Result<(bool, String), Box<dyn std::error::Error>> {
        let script = gate_dir()
            .join("canonical-enforce.sh")
            .display()
            .to_string();
        let root = repo_root().display().to_string();
        let mut args = vec![
            script.as_str(),
            root.as_str(),
            self.container.0.as_str(),
            probe,
        ];
        if let Some(extra) = extra {
            args.push(extra);
        }
        run_env(
            "bash",
            &args,
            &[("CANONICAL_REQUIRE_TABLES", if armed { "1" } else { "0" })],
        )
    }

    /// Plants `sql` on its own probe database and enforces.
    fn probe(
        &self,
        label: &str,
        sql: &str,
        armed: bool,
    ) -> Result<(bool, String), Box<dyn std::error::Error>> {
        let file = self.workdir.join(format!("probe-{label}.sql"));
        std::fs::write(&file, sql)?;
        let path = file.display().to_string();
        self.enforce(&format!("census_probe_{label}"), Some(&path), armed)
    }

    /// Replaces the script inside the container. Passing the committed path
    /// restores it.
    fn install_script(&self, path: &Path) -> Fallible {
        let (ok, log) = run(
            "docker",
            &[
                "cp",
                &path.display().to_string(),
                &format!("{}:/topology.sh", self.container.0),
            ],
        )?;
        assert!(ok, "docker cp {} -> /topology.sh: {log}", path.display());
        Ok(())
    }
}

/// Boots one PostgreSQL container with the role topology in place.
fn boot() -> Result<Harness, Box<dyn std::error::Error>> {
    let name = unique("console-canonical-census");
    let database = "console_canonical_census";
    let values = secrets();
    let env_body = format!(
        "POSTGRES_DB={database}\nPOSTGRES_USER=console_census_admin\nPOSTGRES_PASSWORD={0}\n\
         POSTGRES_HOST=127.0.0.1\nPOSTGRES_PORT=5432\n\
         POSTGRES_ADMIN_USER=console_census_admin\nPOSTGRES_ADMIN_PASSWORD={0}\n\
         CONSOLE_APP_POSTGRES_PASSWORD={1}\nCONSOLE_RT_POSTGRES_PASSWORD={2}\n\
         CONSOLE_LEAVE_COMMAND_POSTGRES_PASSWORD={3}\n\
         CONSOLE_ONTOLOGY_COMMAND_POSTGRES_PASSWORD={4}\n\
         CONSOLE_PLATFORM_FORCE_COMMAND_POSTGRES_PASSWORD={5}\n",
        values[0], values[1], values[2], values[3], values[4], values[5]
    );
    let workdir = std::env::temp_dir().join(&name);
    std::fs::create_dir_all(&workdir)?;
    let env_file = workdir.join("topology.env");
    std::fs::write(&env_file, &env_body)?;

    let (started, log) = run(
        "docker",
        &[
            "run",
            "-d",
            "--rm",
            "--name",
            &name,
            "--env-file",
            &env_file.display().to_string(),
            IMAGE,
        ],
    )?;
    assert!(started, "could not start the census container: {log}");
    let container = Container(name);

    let mut healthy = false;
    for _ in 0..60 {
        let (ok, _) = run(
            "docker",
            &[
                "exec",
                &container.0,
                "pg_isready",
                "-h",
                "127.0.0.1",
                "-U",
                "console_census_admin",
                "-d",
                database,
            ],
        )?;
        if ok {
            healthy = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    assert!(healthy, "postgres never became healthy in {}", container.0);

    for (from, to) in [
        (topology_script().display().to_string(), "/topology.sh"),
        (env_file.display().to_string(), "/topology.env"),
    ] {
        let (ok, log) = run("docker", &["cp", &from, &format!("{}:{to}", container.0)])?;
        assert!(ok, "docker cp {from} -> {to}: {log}");
    }
    Ok(Harness { container, workdir })
}

/// The `[a,b,c]` set the enforcement's own summary line reports.
///
/// Read from the SUMMARY line only, never from the per-step NOTICEs: the
/// pre-migration topology run inside the probe legitimately reports zero, and
/// reading that one would be the very confusion this test exists to reject.
fn examined_set(log: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let line = log
        .lines()
        .find(|line| line.starts_with("canonical-enforce: enforced"))
        .ok_or_else(|| format!("no enforcement summary line in:\n{log}"))?;
    let inside = line
        .split_once('[')
        .and_then(|(_, tail)| tail.split_once(']'))
        .ok_or_else(|| format!("the summary line names no table set: {line}"))?
        .0;
    let mut names: Vec<String> = inside
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .collect();
    names.sort();
    Ok(names)
}

#[test]
fn derived_required_tables_match_the_verbatim_roster() -> Fallible {
    let derived = required_tables_from_topology(&topology_script())?;
    assert_eq!(
        derived,
        EXPECTED_REQUIRED_TABLES.map(str::to_string).to_vec(),
        "the shell `required_tables` array and the verbatim pin must stay the same \
         set; a new canonical table edits BOTH (shell first, then this pin), and \
         this test is the ratchet that refuses a drift"
    );
    Ok(())
}

/// A decoy `required_tables CONSTANT TEXT[] := ARRAY[` plus BEGIN/END markers
/// earlier in the shell script must not win. The production declaration lives
/// inside `DO $canonical$`; parsing the whole file (or the first ARRAY[
/// statement) lets a comment with the pinned 20 names keep the cheap ratchet
/// green while the real SQL array drifts.
#[test]
fn required_tables_parser_ignores_decoy_outside_canonical_block() -> Fallible {
    let path = std::env::temp_dir().join(format!(
        "writer-ownership-decoy-required-tables-{}.sh",
        std::process::id()
    ));
    std::fs::write(
        &path,
        r#"#!/bin/sh
required_tables CONSTANT TEXT[] := ARRAY[
      -- canonical-writer-ownership: BEGIN required tables
      'organizations',
      'employees',
      'persons',
      'person_revisions',
      'employee_person_bindings',
      'employment_heads',
      'employment_revisions',
      'employment_source_bindings',
      'company_revisions',
      'org_units',
      'org_unit_revisions',
      'org_unit_source_bindings',
      'job_positions',
      'job_position_revisions',
      'payroll_draft_runs',
      'payroll_draft_lines',
      'payroll_line_calculations',
      'payroll_run_exceptions',
      'payroll_disbursements',
      'payroll_payslip_deliveries'
      -- canonical-writer-ownership: END required tables
    ];
# Read back by the DO $canonical$ block below
DO $canonical$
DECLARE
    required_tables CONSTANT TEXT[] := ARRAY[
      -- canonical-writer-ownership: BEGIN required tables
      'organizations'
      -- canonical-writer-ownership: END required tables
    ];
END
$canonical$;
"#,
    )?;
    let derived = required_tables_from_topology(&path);
    let _ = std::fs::remove_file(&path);
    assert_eq!(
        derived?,
        vec!["organizations".to_string()],
        "the parser must read the declaration inside DO $canonical$, not a decoy ARRAY[ earlier in the script"
    );
    Ok(())
}

#[test]
fn census_binds_to_an_executed_database() -> Fallible {
    require_docker();
    let harness = boot()?;

    // Phase 1 — zero canonical tables while CLAIMING to enforce them.
    let (ok, log) = harness.enforce("census_probe_fresh", Some("--skip-migrations"), true)?;
    assert!(
        !ok,
        "a run that examines ZERO canonical tables while claiming to enforce them \
         MUST fail — this is exactly the fail-open every automated path shipped:\n{log}"
    );
    assert!(
        log.contains("topology.canonical_enforcement_examined_no_tables"),
        "it must fail for the RIGHT reason, not incidentally:\n{log}"
    );

    // Phase 2 — migrations, then enforcement, over an EXACT set of tables.
    let (ok, clean_log) = harness.enforce("census_probe_clean", None, true)?;
    assert!(
        ok,
        "the measured writer surface of a fully migrated database must pass the \
         census; if `console_rt` is missing from the expected-writer ratchet this \
         is where it shows:\n{clean_log}"
    );
    assert_eq!(
        examined_set(&clean_log)?,
        required_tables_from_topology(&topology_script())?,
        "the census must examine exactly the canonical tables a migrated database \
         has. A count assertion passes while the scope silently shrinks — a rename \
         took it from eight to seven and the run still reported success:\n{clean_log}"
    );

    // Phase 2b — DML held by MEMBERSHIP in the table owner. The reconcile
    // REPAIRS this one rather than rejecting it: the exact-topology REVOKE
    // strips every membership edge touching an application role, and
    // `topology.membership_readback_failed` pins the result — both long before
    // the canonical block runs. A guard for it inside the canonical block would
    // therefore examine nothing, so the proof has to be the mutation: break that
    // REVOKE and the same plant must fail.
    let (ok, log) = harness.probe("membership_repaired", MEMBERSHIP_PLANT, true)?;
    assert!(
        ok,
        "a membership edge into console_app is removed by the reconcile itself, so \
         the run must succeed:\n{log}"
    );
    harness.install_script(&mutant(
        &harness.workdir,
        "keep-memberships",
        KEEP_MEMBERSHIPS,
    )?)?;
    let (ok, log) = harness.probe("membership_kept", MEMBERSHIP_PLANT, true)?;
    harness.install_script(&topology_script())?;
    assert!(
        !ok && log.contains("topology.membership_readback_failed"),
        "with the exact-topology REVOKE broken, a rogue MEMBER of console_app must \
         fail the reconcile — that member inherits the owner's implicit DML on \
         every canonical table. It did not, so nothing removes the edge:\n{log}"
    );

    // Phase 3 — every shape of unaccounted writer, one probe database each.
    for probe in &PROBES {
        let (ok, log) = harness.probe(probe.label, probe.sql, true)?;
        assert!(!ok, "{}: this must fail the reconcile:\n{log}", probe.label);
        assert!(
            log.contains(probe.error),
            "{}: the failure must be `{}`, not something incidental:\n{log}",
            probe.label,
            probe.error
        );
        assert!(
            log.contains(probe.names),
            "{}: the failure must name `{}`:\n{log}",
            probe.label,
            probe.names
        );
    }

    // Phase 4 — break each control and prove the probe that covers it flips.
    for mutation in &MUTATIONS {
        let suffix = mutation.label.replace('-', "_");
        let probe = PROBES.iter().find(|probe| probe.label == mutation.probe);

        // The two unarmed mutations need their own BEFORE picture: phase 3 ran
        // this probe armed, where step 1c refuses it whatever these controls
        // do, so an armed baseline would make the flip a comparison between two
        // different questions. See `Mutation::armed`.
        assert!(
            mutation.armed || probe.is_some(),
            "{}: an unarmed mutation must name a probe — the clean database is not a \
             configuration in which these controls are the only thing looking",
            mutation.label
        );
        if let Some(probe) = probe.filter(|_| !mutation.armed) {
            let (ok, log) = harness.probe(&format!("unarmed_before_{suffix}"), probe.sql, false)?;
            assert!(
                !ok,
                "{}: with the committed script and this control INTACT, the `{}` \
                 probe must fail an UNARMED run — that is the configuration in \
                 which this control is the only thing looking. It passed, so the \
                 mutation below flips nothing:\n{log}",
                mutation.label, probe.label
            );
            assert!(
                log.contains(probe.error),
                "{}: the unarmed baseline must fail as `{}`:\n{log}",
                mutation.label,
                probe.error
            );
        }

        let mutant = mutant(&harness.workdir, mutation.label, mutation.expression)?;
        harness.install_script(&mutant)?;
        let (ok, log) = match probe {
            Some(probe) => harness.probe(&format!("mut_{suffix}"), probe.sql, mutation.armed)?,
            None => harness.enforce(&format!("census_probe_mut_{suffix}"), None, mutation.armed)?,
        };
        harness.install_script(&topology_script())?;

        assert_eq!(
            ok,
            mutation.expect_pass,
            "{}: breaking this control must flip the `{}` probe's verdict. It did \
             not, so nothing here tests that control:\n{log}",
            mutation.label,
            if mutation.probe.is_empty() {
                "clean"
            } else {
                mutation.probe
            }
        );
    }

    let _ = std::fs::remove_dir_all(&harness.workdir);
    Ok(())
}

// ---------------------------------------------------------------------------
// The wiring, proven by RUNNING each harness
// ---------------------------------------------------------------------------

/// The enforcement's own output line. Its presence is proof the enforcement
/// executed; the name of the script that would have produced it is not.
const ENFORCED: &str = "canonical-enforce: enforced";

#[test]
fn pgtest_harness_executes_the_enforcement() -> Fallible {
    require_docker();
    let root = repo_root().display().to_string();
    let script = repo_root()
        .join("tools/lanes/pgtest.sh")
        .display()
        .to_string();
    // `true` as the command under test: the harness still boots PostgreSQL,
    // reconciles and enforces, and nothing is compiled.
    let (ok, log) = run("bash", &[&script, &root, "true"])?;
    assert!(ok, "tools/lanes/pgtest.sh did not complete:\n{log}");
    assert!(
        log.contains(ENFORCED),
        "tools/lanes/pgtest.sh must EXECUTE the canonical enforcement. Commenting \
         the invocation out leaves the literal `canonical-enforce.sh` in the file, \
         so only its output proves it ran:\n{log}"
    );
    Ok(())
}

#[test]
fn cargo_needs_postgres_harness_executes_the_enforcement() -> Fallible {
    require_docker();
    let script = repo_root()
        .join("tools/ci/cargo_needs_postgres.sh")
        .display()
        .to_string();
    // A `--only` name no map entry carries: the harness boots, reconciles and
    // enforces, then exits on an empty selection without building anything.
    let (ok, log) = run(
        "bash",
        &[&script, "--only", "console-canonical-wiring-probe"],
    )?;
    assert!(
        log.contains(ENFORCED),
        "tools/ci/cargo_needs_postgres.sh must EXECUTE the canonical enforcement \
         before it selects targets:\n{log}"
    );
    assert!(
        !ok && log.contains("no map entries selected"),
        "the probe selection is expected to end the run, and did not:\n{log}"
    );
    Ok(())
}
