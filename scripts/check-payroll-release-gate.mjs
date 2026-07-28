import { createTextGate } from "./lib/text-gate.mjs";

const { requireIncludes, requireMatches, requireAbsent, reportGate } = createTextGate({
  includeFailure: ({ path, needle, label }) => `${path} is missing ${label}: ${needle}`,
  matchFailure: ({ path, pattern, label }) => `${path} does not satisfy ${label}: ${pattern}`,
  absentFailure: ({ path, pattern, label }) => `${path} violates ${label}: ${pattern}`,
  passLabel: (label, kind) => `${label} ${kind === "absent" ? "absent" : "present"}`,
});

requireIncludes(
  "docs/specs/payroll.md",
  "Status: first regulated-kernel slice",
  "regulated payroll spec status",
);
requireIncludes(
  "docs/specs/payroll.md",
  "Production payroll calculations are disabled unless all are true",
  "production payroll release gate",
);
requireIncludes(
  "docs/specs/payroll.md",
  "G028 production-control contract",
  "G028 production-control contract",
);
requireIncludes(
  "docs/specs/payroll.md",
  "generic employee import/export can only preview masked values",
  "generic HR payroll masking rule",
);
requireIncludes(
  "docs/specs/payroll.md",
  "payroll/wage-statement mail may exist as an audited work-mail object",
  "payroll receipt mail boundary",
);
requireIncludes(
  "docs/specs/payroll.md",
  "passkey step-up",
  "payroll signing-equivalent step-up rule",
);

requireIncludes(
  "backend/crates/payroll/domain/src/lib.rs",
  "This crate intentionally contains pure, source-versioned data and guardrail",
  "pure payroll kernel boundary",
);
requireIncludes(
  "backend/crates/payroll/domain/src/lib.rs",
  "NTS withholding tax table row is required; payroll must not estimate income tax",
  "no estimated income tax path",
);
requireIncludes(
  "backend/crates/payroll/domain/src/lib.rs",
  "노무사/세무사 professional validation is required",
  "professional validation fail-closed gate",
);
requireIncludes(
  "backend/crates/payroll/domain/src/lib.rs",
  "at least one payroll golden case is required",
  "golden case release gate",
);
requireIncludes(
  "backend/crates/payroll/domain/src/lib.rs",
  "artifact_sha256 must be a 64-character hex digest",
  "professional artifact digest validation",
);

requireMatches(
  "package.json",
  /"check:payroll"\s*:\s*"node scripts\/check-payroll-domain\.mjs"/,
  "payroll domain script",
);
requireMatches(
  "package.json",
  /"check:payroll-release-gate"\s*:\s*"node scripts\/check-payroll-release-gate\.mjs"/,
  "payroll release-gate script",
);
requireIncludes(
  ".github/workflows/ci.yml",
  "npm run check:payroll-release-gate",
  "CI payroll release-gate wiring",
);

reportGate("payroll release gate check passed");
