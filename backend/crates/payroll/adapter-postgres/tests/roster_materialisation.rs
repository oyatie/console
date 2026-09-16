#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The production roster writer, proven against a real PostgreSQL.
//!
//! `payroll_draft_lines` had no production writer: its only writer was a
//! hand-run SQL script, so every run production code created had an empty
//! roster and — once the close preflight required `roster_total > 0` — could
//! never close. `roster::materialise_roster_in_tx` is the port of that script.
//!
//! These tests exist because the port could go wrong in ways that all look like
//! success: a roster built from material nobody applied, from the wrong month,
//! from blank cells, or one line per spreadsheet row instead of per person. Each
//! is a NEAR-MISS fixture with an exact-count assertion, not a smoke test.

#[path = "roster_materialisation/seed.rs"]
mod seed;

use console_payroll_adapter_postgres::lifecycle::{calculate_run_in_tx, close_attendance_in_tx};
use seed::{
    PERIOD_END, PERIOD_START, attendance_row, materialise, roster, seed_employee, seed_import,
    seed_org_and_run,
};
use sqlx::PgPool;
use time::macros::date;

/// POSITIVE CONTROL. Without this, every refusal below could be produced by a
/// writer that writes nothing at all.
#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn an_applied_in_period_row_becomes_exactly_one_roster_line(pool: PgPool) {
    let f = seed_org_and_run(&pool).await;
    seed_employee(&pool, f.org, "emp-1", "홍길동").await;
    seed_import(
        &pool,
        f.org,
        "APPLIED",
        "CANDIDATE",
        (PERIOD_START, PERIOD_END),
        "emp-1",
        attendance_row(),
    )
    .await;

    let diag: (i64, i64, Option<String>, Option<String>) = sqlx::query_as(
        "SELECT (SELECT count(*) FROM data_import_runs WHERE org_id=$1 AND status='APPLIED'), \
                (SELECT count(*) FROM data_import_rows WHERE org_id=$1), \
                (SELECT canonical_row->>'source_key' FROM data_import_rows WHERE org_id=$1 LIMIT 1), \
                (SELECT source_key FROM employees WHERE org_id=$1 LIMIT 1)",
    ).bind(f.org).fetch_one(&pool).await.unwrap();
    assert_eq!(
        materialise(&pool, &f).await,
        1,
        "one employee, one line; diag={diag:?}"
    );
    let lines = roster(&pool, &f).await;
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].0, "emp-1");
    assert!(
        lines[0].2 > 0,
        "attendance material must be counted: {lines:?}"
    );
}

/// TWO import rows for the SAME person still make ONE line.
///
/// `data_import_rows.source_key` is `filename:…|sheet:…|row:…`, so a writer that
/// grouped on it would produce a roster of spreadsheet rows rather than people —
/// and a two-sheet workbook would silently double the roster.
#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn two_rows_for_one_person_make_one_line(pool: PgPool) {
    let f = seed_org_and_run(&pool).await;
    seed_employee(&pool, f.org, "emp-1", "홍길동").await;
    for _ in 0..2 {
        seed_import(
            &pool,
            f.org,
            "APPLIED",
            "CANDIDATE",
            (PERIOD_START, PERIOD_END),
            "emp-1",
            attendance_row(),
        )
        .await;
    }
    assert_eq!(
        materialise(&pool, &f).await,
        1,
        "one PERSON, not one per row"
    );
    assert_eq!(roster(&pool, &f).await.len(), 1);
}

/// Material declared for a DIFFERENT period is not this run's material.
///
/// The near-miss is deliberate: a period that OVERLAPS but is not equal. A writer
/// using overlap instead of equality would admit it.
#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn a_different_period_is_not_this_runs_material(pool: PgPool) {
    let f = seed_org_and_run(&pool).await;
    seed_employee(&pool, f.org, "emp-1", "홍길동").await;
    seed_import(
        &pool,
        f.org,
        "APPLIED",
        "CANDIDATE",
        (date!(2026 - 01 - 01), date!(2026 - 12 - 31)),
        "emp-1",
        attendance_row(),
    )
    .await;
    assert_eq!(
        materialise(&pool, &f).await,
        0,
        "an overlapping period is not an equal one"
    );
    assert!(roster(&pool, &f).await.is_empty());
}

/// Material nobody applied is not material.
#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn unapplied_and_errored_material_is_refused(pool: PgPool) {
    let f = seed_org_and_run(&pool).await;
    seed_employee(&pool, f.org, "emp-1", "홍길동").await;
    for (status, row_status) in [
        ("DRY_RUN", "CANDIDATE"),
        ("PREVIEWED", "CANDIDATE"),
        ("FAILED", "CANDIDATE"),
        ("APPLIED", "ERROR"),
    ] {
        seed_import(
            &pool,
            f.org,
            status,
            row_status,
            (PERIOD_START, PERIOD_END),
            "emp-1",
            attendance_row(),
        )
        .await;
    }
    assert_eq!(
        materialise(&pool, &f).await,
        0,
        "only APPLIED, non-ERROR rows are material"
    );
    assert!(roster(&pool, &f).await.is_empty());
}

/// A row whose cells are blank says nothing, so it is not source material.
///
/// The columns are PRESENT — this is the near-miss for a key-presence test.
#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn a_row_of_blank_cells_is_not_source_material(pool: PgPool) {
    let f = seed_org_and_run(&pool).await;
    seed_employee(&pool, f.org, "emp-1", "홍길동").await;
    seed_import(
        &pool,
        f.org,
        "APPLIED",
        "CANDIDATE",
        (PERIOD_START, PERIOD_END),
        "emp-1",
        serde_json::json!({ "출근": "", "근무시간": "  ", "기본급": "" }),
    )
    .await;
    assert_eq!(
        materialise(&pool, &f).await,
        0,
        "present-but-blank columns are not evidence"
    );
}

/// An employee with leave but no imported material gets no line.
///
/// The script admitted them via `OR leave_remaining > 0`. Such a line carries no
/// attendance evidence, so it can only ever block the close.
#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn an_employee_with_no_imported_material_is_not_admitted(pool: PgPool) {
    let f = seed_org_and_run(&pool).await;
    seed_employee(&pool, f.org, "emp-1", "홍길동").await;
    // NOTE ON THE NEAR-MISS. Ideally this employee would carry
    // `leave_remaining > 0`, since that is the exact disjunct the script used to
    // admit them on. `employees.leave_remaining` cannot be set from a test: the
    // write raises 42501 `leave_write.command_required`, because leave balances
    // are command-governed. So this proves the weaker, still load-bearing half —
    // an employee with NO imported material gets no line — and the disjunct's
    // absence is additionally pinned by the reviewer note in roster.rs.
    assert_eq!(
        materialise(&pool, &f).await,
        0,
        "an employee with no imported material must not be admitted to the roster"
    );
}

/// Re-materialising is idempotent and does not duplicate the roster.
#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn re_materialising_updates_rather_than_duplicates(pool: PgPool) {
    let f = seed_org_and_run(&pool).await;
    seed_employee(&pool, f.org, "emp-1", "홍길동").await;
    seed_import(
        &pool,
        f.org,
        "APPLIED",
        "CANDIDATE",
        (PERIOD_START, PERIOD_END),
        "emp-1",
        attendance_row(),
    )
    .await;
    assert_eq!(materialise(&pool, &f).await, 1);
    assert_eq!(
        materialise(&pool, &f).await,
        1,
        "the second pass updates the same line"
    );
    assert_eq!(roster(&pool, &f).await.len(), 1, "no duplicate line");
}

/// Native `employee_contract_wages` in force on the period end admit a roster
/// line with no import rows. Import-backed materialisation stays the other
/// admission path; this is not an import-shaped fixture.
#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn a_native_contract_wage_becomes_a_roster_line_without_import(pool: PgPool) {
    let f = seed_org_and_run(&pool).await;
    let employee_id = seed_employee(&pool, f.org, "emp-native", "김임금").await;
    seed::seed_monthly_contract_wage(&pool, f.org, employee_id, PERIOD_START, 3_000_000).await;

    let import_rows: i64 =
        sqlx::query_scalar("SELECT count(*) FROM data_import_rows WHERE org_id = $1")
            .bind(f.org)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(import_rows, 0, "this fixture must not plant import rows");
    assert_eq!(
        materialise(&pool, &f).await,
        1,
        "native wage must admit exactly one roster line"
    );
    let lines = roster(&pool, &f).await;
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].0, "emp-native");
}

/// Shipped close then calculate: native contract wage (no import rows) is
/// enough to close, then `calculate_run_in_tx` persists a versioned draft with
/// `payable` false. A second calculate replays that version without a second persist.
#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn calculate_uses_native_contract_wage_without_import_rows(pool: PgPool) {
    let f = seed_org_and_run(&pool).await;
    let employee_id = seed_employee(&pool, f.org, "emp-native", "김임금").await;
    let actor =
        seed::seed_monthly_contract_wage(&pool, f.org, employee_id, PERIOD_START, 3_000_000).await;
    assert_eq!(materialise(&pool, &f).await, 1);

    sqlx::query(
        "INSERT INTO period_locks (org_id, domain, period_start, period_end, reason) \
         VALUES ($1, 'payroll', $2, $3, 'native close')",
    )
    .bind(f.org)
    .bind(PERIOD_START)
    .bind(PERIOD_END)
    .execute(&pool)
    .await
    .unwrap();

    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.current_org', $1, true)")
        .bind(f.org.to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
    close_attendance_in_tx(&mut tx, f.run, actor, time::OffsetDateTime::now_utc())
        .await
        .expect("native wage line must close without forged ATTENDANCE_CLOSED");
    tx.commit().await.unwrap();

    let status: String = sqlx::query_scalar("SELECT status FROM payroll_draft_runs WHERE id = $1")
        .bind(f.run)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "ATTENDANCE_CLOSED");

    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.current_org', $1, true)")
        .bind(f.org.to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
    let first = calculate_run_in_tx(&mut tx, f.run)
        .await
        .expect("native calculate");
    tx.commit().await.unwrap();

    assert_eq!(first.version, 1);
    assert_eq!(first.calculated_lines, 1, "{first:?}");
    assert_eq!(first.blocked_lines, 0, "{first:?}");

    let row: (i64, bool, i32, String) = sqlx::query_as(
        "SELECT gross_won, payable, version, tax_table_version \
         FROM payroll_line_calculations WHERE run_id = $1",
    )
    .bind(f.run)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(row.0, 3_000_000);
    assert!(!row.1, "native drafts stay payable false");
    assert_eq!(row.2, 1);
    assert!(
        row.3.contains("INCOME_TAX_HOLD"),
        "must not claim a Korea wage-statement table: {}",
        row.3
    );

    let import_rows: i64 =
        sqlx::query_scalar("SELECT count(*) FROM data_import_rows WHERE org_id = $1")
            .bind(f.org)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(import_rows, 0);

    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.current_org', $1, true)")
        .bind(f.org.to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
    let replay = calculate_run_in_tx(&mut tx, f.run)
        .await
        .expect("second calculate must replay");
    tx.commit().await.unwrap();
    assert_eq!(replay.version, first.version);
    assert_eq!(replay.calculated_lines, first.calculated_lines);

    let persisted: i64 =
        sqlx::query_scalar("SELECT count(*) FROM payroll_line_calculations WHERE run_id = $1")
            .bind(f.run)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(persisted, 1, "replay must not insert a second version");
}
