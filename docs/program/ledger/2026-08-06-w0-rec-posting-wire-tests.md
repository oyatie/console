# Ledger: W0-REC recruiting posting wire tests

**Date:** 2026-08-06  
**Allowlist:** backend/crates/recruiting/domain/src/lib.rs

## Outcome

PostingStatus/Scope/EmploymentType/AssessmentScore wire roundtrips; from_db vs from_input error kinds.

## Verification

`backend/crates/recruiting/domain/src/lib.rs` baseline 4 → 5

```
cargo test -p console-recruiting-domain --lib
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
    "Essentialism / YAGNI": "One pure posting/employment vocab test.",
    "Chesterton's Fence": "from_db Internal vs from_input Validation retained.",
    "Pragmatism": "AC: 5 domain tests green (was 4).",
    "Red Team": "Unknown tags fail closed by origin.",
    "Blast-radius / cell-based": "recruiting/domain only.",
    "Zero-trust / defense-in-depth": "Fail-closed posting vocabulary."
  },
  "mandatory_lens_exceptions": {},
  "findings": [
    "PostingStatus/Scope/EmploymentType/AssessmentScore lacked explicit input-vs-internal fail-closed tests."
  ],
  "decisions_changed_or_rejected": [],
  "lens_set_changes": []
}
```
<!-- REASONING-LENS-EVIDENCE:END -->

## Authority tip

T is the signed authority tip for this candidate train. C prebinds this ledger blob.
