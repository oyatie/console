# Ledger: W0-POL-2 policy domain fail-closed pure tests

**Date:** 2026-08-06  
**Epic:** console-a80  
**Allowlist:** backend/crates/policy/domain/**

## Outcome

Unit tests: review lifecycle never maps to runtime-enforced catalog status;
condition/principal constructors reject incomplete or unknown authority material.

## Verification

cargo test -p console-policy-domain --lib  # 7 passed
Locked executed-tests-baseline policy domain attrs (5→7).

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
    "Essentialism / YAGNI": "Two sparse pure domain tests only.",
    "Chesterton's Fence": "Review never runtime-enforced retained.",
    "Pragmatism": "7 unit tests green.",
    "Red Team": "Unknown wire and incomplete constructors fail closed.",
    "Operability / Day-2": "Baseline locked.",
    "Blast-radius / cell-based": "policy/domain only.",
    "Telemetry-first": "cargo counts.",
    "Zero-trust / defense-in-depth": "No smuggled enforced/shadow review wires."
  },
  "mandatory_lens_exceptions": {},
  "findings": [
    "Review→catalog mapping lacked pure coverage against runtime-enforced statuses.",
    "Condition/principal constructors lacked incomplete-input fail-closed tests."
  ],
  "decisions_changed_or_rejected": [],
  "lens_set_changes": []
}
```
<!-- REASONING-LENS-EVIDENCE:END -->
