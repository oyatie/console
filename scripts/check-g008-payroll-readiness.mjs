import { createTextGate } from "./lib/text-gate.mjs";
import { beginGate, emitProvenanceIfRequested } from "./lib/gate-inputs.mjs";

beginGate({
  gate: "check:g008-payroll-readiness",
  script: "scripts/check-g008-payroll-readiness.mjs",
  documentInputs: [
    "docs/specs/hr-payroll-readiness.md",
  ],
});

const { requireIncludes, requireMatches, requireNotIncludes, reportGate } = createTextGate();

requireIncludes(
  "docs/specs/hr-payroll-readiness.md",
  "Payroll calculation remains blocked until the release gate is professionally validated",
  "payroll readiness spec keeps regulated calculation blocked",
);
requireIncludes(
  "docs/specs/hr-payroll-readiness.md",
  "annual-leave usage-promotion workflow",
  "annual leave usage-promotion workflow is modeled",
);
requireIncludes(
  "docs/specs/hr-payroll-readiness.md",
  "messenger/mail/workflow notification is a workflow object",
  "future notification integration routes through workflow/comms objects",
);

requireIncludes(
  "backend/crates/platform/db/migrations/0074_create_payroll_readiness.sql",
  "CREATE TABLE payroll_draft_runs",
  "payroll draft run table exists",
);
requireIncludes(
  "backend/crates/platform/db/migrations/0074_create_payroll_readiness.sql",
  "CREATE TABLE payroll_draft_lines",
  "payroll draft line table exists",
);
requireIncludes(
  "backend/crates/platform/db/migrations/0074_create_payroll_readiness.sql",
  "CREATE TABLE annual_leave_obligations",
  "annual leave obligation table exists",
);
requireIncludes(
  "backend/crates/platform/db/migrations/0074_create_payroll_readiness.sql",
  "BLOCKED_LEGAL_GATE",
  "draft lines fail closed behind legal gate",
);
requireIncludes(
  "backend/crates/platform/db/migrations/0074_create_payroll_readiness.sql",
  "FORCE ROW LEVEL SECURITY",
  "payroll readiness tables force RLS",
);
requireIncludes(
  "backend/crates/platform/db/migrations/0074_create_payroll_readiness.sql",
  "GRANT SELECT, INSERT, UPDATE ON payroll_draft_runs TO console_rt",
  "runtime grants are explicit for draft runs",
);

requireIncludes(
  "scripts/stage_coss_group_payroll_readiness.sql",
  "COSS Group 2026-05 live import",
  "live import source label is explicit",
);
requireIncludes(
  "scripts/stage_coss_group_payroll_readiness.sql",
  "data_import_rows",
  "stage SQL derives from governed import ledger",
);
requireIncludes(
  "scripts/stage_coss_group_payroll_readiness.sql",
  "Payroll calculation remains blocked until an official NTS row and professional validation are attached",
  "stage SQL does not enable payable payroll",
);
requireIncludes(
  "scripts/stage_coss_group_payroll_readiness.sql",
  "annual_leave_obligations",
  "stage SQL creates annual leave review obligations",
);
requireIncludes(
  "scripts/stage_coss_group_payroll_readiness.sql",
  "data_import.payroll_readiness_stage",
  "stage SQL audits the live derivation",
);
requireMatches(
  "scripts/stage_coss_group_payroll_readiness.sql",
  /raw_row\?\|array\[/,
  "stage SQL classifies payroll/attendance source rows by allowlisted headers",
);
requireNotIncludes(
  "scripts/stage_coss_group_payroll_readiness.sql",
  "SELECT *",
  "stage SQL avoids broad raw selects",
);

// Provenance of the material that reaches payroll_draft_lines. `data_import_runs.status`
// admits PREVIEWED/DRY_RUN/APPLIED/FAILED and `data_import_rows.row_status` admits
// CANDIDATE/PRESERVED/ERROR (migration 0070). The stage SQL filtered on NEITHER: the only
// thing keeping unapplied rows out of a payroll roster was a human hand-typing one vetted
// source_filename. Measured against a live PostgreSQL: with these two predicates removed, a
// never-applied DRY_RUN and a run whose rows are all ERROR each materialise a roster line.
requireIncludes(
  "scripts/stage_coss_group_payroll_readiness.sql",
  "run.status = 'APPLIED'",
  "stage SQL admits only APPLIED import runs as payroll source material",
);
requireIncludes(
  "scripts/stage_coss_group_payroll_readiness.sql",
  "r.row_status <> 'ERROR'",
  "stage SQL never treats an ERROR import row as payroll source material",
);
// Re-running the stage must not rewind a run that has left the pre-close states. Without the
// guard the ON CONFLICT DO UPDATE reset status to BLOCKED_LEGAL_GATE unconditionally, so a
// re-stage over a PAID run rewound it -- measured: PAID -> BLOCKED_LEGAL_GATE.
requireIncludes(
  "scripts/stage_coss_group_payroll_readiness.sql",
  "WHERE payroll_draft_runs.status IN ('STAGED', 'BLOCKED_LEGAL_GATE', 'READY_FOR_REVIEW')",
  "stage SQL refuses to rewind a run past the pre-close states",
);

requireIncludes(
  "package.json",
  '"check:g008-payroll-readiness"',
  "package script is wired",
);
requireIncludes(
  ".github/workflows/ci.yml",
  "npm run check:g008-payroll-readiness",
  "CI runs G008 payroll readiness gate",
);

emitProvenanceIfRequested();
reportGate("G008 payroll readiness gate passed");
