//! Read-only critic probe: runs the REAL gate `scan` over throwaway trees in
//! /tmp, mirroring gate_detects_violation.rs helpers exactly.

use std::fs;

const CANARY: &str = "pub const CANARY: &str = \"UPDATE organizations SET name = 'x'\";\n";
const PARSED_ONLY_CANARY: &str =
    "/// UPDATE org_units SET name = 'x'\npub struct ParsedCanary;\n";

fn probe(label: &str, body: &str) {
    let dir = std::env::temp_dir().join(format!("wo-critic-probe-{label}-{}", std::process::id()));
    if dir.exists() {
        fs::remove_dir_all(&dir).unwrap();
    }
    fs::create_dir_all(dir.join("intruder/src")).unwrap();
    fs::write(
        dir.join("intruder/Cargo.toml"),
        "[package]\nname = \"console-x-adapter-postgres\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(
        dir.join("intruder/src/lib.rs"),
        format!("{CANARY}{PARSED_ONLY_CANARY}pub mod employees {{}}\n{body}\n"),
    )
    .unwrap();
    let report = console_gate_writer_ownership::scan(&dir).unwrap();
    let charged = |t: &str| report.violations.iter().any(|v| v.table == t);
    println!(
        "[{label}] tree_read(organizations)={} parse_ok(org_units_not_charged)={} employees_write_charged={}",
        charged("organizations"),
        !charged("org_units"),
        charged("employees"),
    );
}

fn main() {
    // Control: the exact spelling the residual-11 pin says is now CLOSED.
    probe(
        "dash-in-quoted-data",
        "pub const SQL: &str = \"UPDATE t SET note = 'a -- b'; \
         UPDATE employees SET org_unit = 1\";",
    );
    // Hypothesis: sibling spelling — /* inside quoted data, */ inside a later
    // quoted string in the SAME Rust literal. Predicate does not route it to
    // the lexer; byte fallback deletes the span including the write.
    probe(
        "block-comment-in-quoted-data",
        "pub const SQL: &str = \"UPDATE t SET note = 'a /* b'; \
         UPDATE employees SET org_unit = 1; SELECT 'x */ y'\";",
    );
    // Boundary sanity: unterminated /* in quoted data must NOT hide the write
    // (past_block_comment returns None).
    probe(
        "unterminated-block-in-quoted-data",
        "pub const SQL: &str = \"UPDATE t SET note = 'a /* b'; \
         UPDATE employees SET org_unit = 1\";",
    );
}
