# Ledger: W0-INSP inspection domain wire + CI reachability

**Date:** 2026-08-06  
**Allowlist:** backend/crates/inspection/domain/**, .github/workflows/ci.yml, scripts/check-ci-preflight.mjs, docs/program/executed-tests-baseline.json

## Outcome

Pure domain unit tests for inspection cycle/schedule/round wire tags and
fail-closed `interval_days`. Named `console-inspection-domain` in Domain crates
unit CI + preflight package list + run proofDigest so tests cannot execute nowhere.

## Verification

```
cargo test -p console-inspection-domain --lib   # 3 passed
node scripts/check-ci-preflight.mjs             # passed
node scripts/check-executed-tests.mjs           # inspection not dark
node --test scripts/check-ci-preflight.test.mjs # 53 passed
```

<!-- REASONING-LENS-EVIDENCE:START -->
```json
{
  "lens_contract": "v1",
  "lens_contract_digest": "ac1e7d6b8150808ef73e5e3cd1a1e54d2f37eb43e84aaa1370dbbaaff3c44373",
  "task_class": "implementation",
  "risk_class": "standard",
  "risk_domains": [],
  "selected_lenses": [
    "Essentialism / YAGNI",
    "Chesterton's Fence",
    "Pragmatism",
    "Red Team",
    "Blast-radius / cell-based",
    "Zero-trust / defense-in-depth"
  ],
  "task_fit": {
    "Essentialism / YAGNI": "Three pure unit tests + CI package list/digest update only.",
    "Chesterton's Fence": "Domain-unit package list and proofDigest are the reachability fence; update deliberately.",
    "Pragmatism": "AC: 3 tests green; check-ci-preflight + check-executed-tests green.",
    "Red Team": "Unknown tags fail closed; local-only cargo cannot hide dark suites.",
    "Blast-radius / cell-based": "inspection domain + domain-unit CI list + preflight digest.",
    "Zero-trust / defense-in-depth": "Fail-closed wire parse + executed-tests reachability."
  },
  "mandatory_lens_exceptions": {},
  "findings": [
    "Inspection domain wire enums had zero unit tests and were dark until named in Domain crates CI."
  ],
  "decisions_changed_or_rejected": [],
  "lens_set_changes": []
}
```
<!-- REASONING-LENS-EVIDENCE:END -->
