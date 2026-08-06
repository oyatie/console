# Ledger: W0-DISP dispatch domain wire tests

**Date:** 2026-08-06  
**Allowlist:** backend/crates/dispatch/domain/src/lib.rs

## Outcome

Domain pure tests for wire/db vocabulary fail-closed roundtrips. HOLDs not cleared.

## Verification

test_attribute_baseline `backend/crates/dispatch/domain/src/lib.rs`: 3 → 5

```
cargo test -p console-dispatch-domain --lib
# 5 passed
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
    "Essentialism / YAGNI": "Pure unit tests only for console-dispatch-domain; no schema/API.",
    "Chesterton's Fence": "Existing wire tags retained; unknown rejected.",
    "Pragmatism": "AC: 5 domain unit tests green (was 3).",
    "Red Team": "Fail-closed parse; no HOLD clear.",
    "Blast-radius / cell-based": "backend/crates/dispatch/domain/src/lib.rs + ledger/baseline only.",
    "Zero-trust / defense-in-depth": "Fail-closed wire vocabulary."
  },
  "mandatory_lens_exceptions": {},
  "findings": [
    "console-dispatch-domain needed explicit wire roundtrip/fail-closed coverage."
  ],
  "decisions_changed_or_rejected": [],
  "lens_set_changes": []
}
```
<!-- REASONING-LENS-EVIDENCE:END -->

## Authority tip

T is the signed authority tip for this candidate train. C prebinds this ledger blob.
