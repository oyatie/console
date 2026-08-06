# Ledger: W0-BEN benefit domain wire tests

**Date:** 2026-08-06  
**Allowlist:** backend/crates/benefit/domain/src/lib.rs

## Outcome

Domain pure tests for BenefitCategory/ScopeKind/ConditionKind/Operator parse↔db_str fail-closed roundtrips.

## Verification

test_attribute_baseline `backend/crates/benefit/domain/src/lib.rs`: 3 → 4

```
cargo test -p console-benefit-domain --lib
# 4 passed
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
    "Essentialism / YAGNI": "One pure vocab roundtrip test module extension.",
    "Chesterton's Fence": "Existing parse/as_db_str retained.",
    "Pragmatism": "AC: 4 domain tests green (was 3).",
    "Red Team": "Fail-closed unknown tags; no HOLD clear.",
    "Blast-radius / cell-based": "benefit/domain only.",
    "Zero-trust / defense-in-depth": "Fail-closed benefit vocabulary."
  },
  "mandatory_lens_exceptions": {},
  "findings": [
    "Benefit domain parse/db_str lacked full vocabulary roundtrips."
  ],
  "decisions_changed_or_rejected": [],
  "lens_set_changes": []
}
```
<!-- REASONING-LENS-EVIDENCE:END -->
