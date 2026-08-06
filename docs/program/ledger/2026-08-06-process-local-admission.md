# Ledger: process local admission + CI failure classifier

**Date:** 2026-08-06
**Kind:** process upgrade

## Outcome

Add local admission gate (`npm run admit` / pre-tool-push) so product reds
are caught before push/PR. Classify hosted setup-only reds as
`ops.gha-infra-flake` so agents do not tip-restack or invent product fixes
during Actions outages.

## Verification

```
node --test scripts/local-admission.test.mjs \
  tools/ci/classify-ci-failure.test.mjs \
  tools/ci/assess-tip-contention.test.mjs \
  tools/ci/check-mjs-dark-suites.test.mjs
# 15 passed
```

<!-- REASONING-LENS-EVIDENCE:START -->
```json
{
  "lens_contract": "v1",
  "lens_contract_digest": "ac1e7d6b8150808ef73e5e3cd1a1e54d2f37eb43e84aaa1370dbbaaff3c44373",
  "task_class": "implementation",
  "risk_class": "high",
  "risk_domains": [
    "other"
  ],
  "selected_lenses": [
    "Essentialism / YAGNI",
    "Chesterton's Fence",
    "Pragmatism",
    "Red Team",
    "Operability / Day-2",
    "Blast-radius / cell-based",
    "Telemetry-first",
    "Zero-trust / defense-in-depth"
  ],
  "task_fit": {
    "Essentialism / YAGNI": "Process scripts only; no product domain crates.",
    "Chesterton's Fence": "Preserves hosted authority train; adds local preflight.",
    "Pragmatism": "15 node tests green; fail closed before push.",
    "Red Team": "No merge authority; classify infra flakes without product rewrite.",
    "Operability / Day-2": "ops.gha-infra-flake class for outage triage.",
    "Blast-radius / cell-based": "scripts/ and tools/ci only.",
    "Telemetry-first": "Failure class ids for retro.",
    "Zero-trust / defense-in-depth": "Local admit fails closed before remote CI."
  },
  "mandatory_lens_exceptions": {},
  "findings": [
    "Agents treated GHA Set-up-job Service Unavailable as product reds.",
    "No local gate prevented unsigned product-only tips from opening PRs."
  ],
  "decisions_changed_or_rejected": [],
  "lens_set_changes": []
}
```
<!-- REASONING-LENS-EVIDENCE:END -->

## Authority tip

T is the signed authority tip for this candidate train. C..T changes only this evidence ledger.
