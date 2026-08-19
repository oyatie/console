> **EXECUTABLE-CONTRACT INVENTORY / NON-AUTHORITY.** Path-stable lists below are machine-checked by `npm run check:foundation-gates` against `.github/workflows/ci.yml` and root `package.json`. This page does not authorize product scope, roadmap order, issue closure, release, or production readiness. Current authority: [`README.md`](../README.md) and [`docs/current/`](current/).

# CI Gates

Refreshed: 2026-08-17 from `.github/workflows/ci.yml`, `.github/workflows/security.yml`, and root `package.json` on `main` at `7705578e`.

The workflow is the executable inventory. This file is the short, machine-checked mirror. Pre-2026-08-17 per-gate prose is historical and lives in git history at `main:docs/CI-GATES.md` (the 69KB essay). Do not treat that history as dispatch authority.

Current `main` branch protection requires exactly three GitHub-Actions-app-bound contexts: `Required / CI`, `Required / Security`, and `authenticate-console-authority`.

`npm run check:foundation-gates` machine-checks the two lists below against the workflow and package manifests.

### Backend console-gate binaries run by CI

- `console-gate-audit-coverage`
- `console-gate-dev-auth-absence`
- `console-gate-fabricated-branch`
- `console-gate-iac-tier`
- `console-gate-layer-boundary`
- `console-gate-migration-safety`
- `console-gate-personal-data-classification`
- `console-gate-pii-no-logs`
- `console-gate-rls-arming`
- `console-gate-tenant-isolation`
- `console-gate-writer-ownership`

### Root package scripts run by CI

- `check:adrs`
- `check:ci-preflight`
- `check:doc-citations`
- `check:doc-manifest`
- `check:gate-input-provenance`
- `test:gate-input-provenance`
- `check:doc-links`
- `check:executed-tests`
- `check:js-test-reachability`
- `test:js-test-reachability`
- `check:foundation-gates`
- `check:g004-identity-foundation`
- `check:g005-workflow-lifecycle`
- `check:g006-asset-dispatch-lifecycle`
- `check:g007-collaboration-mobile-lifecycle`
- `check:g008-payroll-readiness`
- `check:k8s`
- `check:platform-contract-drift`
- `check:test-credentials`
- `check:package-lock`
- `check:payroll-release-gate`
- `check:people-hr-maturity`
- `check:pr473-migration-operational`
- `check:production-hardening`
- `check:request-body-contract`
- `check:undeclared-imports`
- `check:workflow-runtime-m2-cedar-guards`
- `check:workflow-runtime-m2-drainer`
- `check:workflow-runtime-m2-runtime`
- `check:workflow-runtime-m2-strangler`
- `check:workflow-runtime-spine`
- `test:adrs`
- `test:employee-import-contract`
- `test:executed-tests-baseline`
- `test:lane-receipt`
- `test:ontology-write-precondition`
- `test:production-hardening`
- `test:text-gate`

Local verification for a new clone is the sequence in [`docs/current/DELIVERY.md`](current/DELIVERY.md), not a partial subset of the lists above.

## Deployment gates

`check:k8s` renders manifests only; CI warns rather than fails when no live cluster is
reachable. Enforcement of the rendered NetworkPolicy is a separate, opt-in preflight, and
`scripts/check-production-hardening.mjs` requires this exact invocation to be documented
here — a deployment checklist that omits it reads as if rendering were sufficient:

```bash
CONSOLE_NETWORKPOLICY_PREFLIGHT=require npm run check:k8s:networkpolicy
```

`deploy/README.md` carries the same line for the deployment checklist.

## Backend gates

`console-gate-vendor-lockin` exists under `backend/ci/gates/` and is **not** in the CI-run list above.

### Honest unreachable ADR check

[`ADR-0020`](decisions/ADR-0020-korean-institutional-connectivity-coverage-factory.md) Verification names `check:korean-institutional-connectivity`. That script exists in root `package.json`. No workflow in `.github/workflows/` invokes it as of 2026-08-17. Open PR 796 documents the same fact and deliberately does not delete the script, pending an ADR amendment. The ADR's stated enforcement is currently unreachable. This page does not delete or rewire that gate.

`package.json` still defines other `check:` scripts that no workflow invokes. This inventory lists only scripts `ci.yml` actually runs. Do not treat an unused `check:` name as coverage.
