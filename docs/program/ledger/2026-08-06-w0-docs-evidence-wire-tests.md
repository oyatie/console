# Ledger: W0-DOCS evidence vocabulary wire tests

**Date:** 2026-08-06  
**Allowlist:** backend/crates/docs/domain/src/lib.rs

## Outcome

Evidence classification/source/WORM/TSA/custody/legal-hold/admissibility parse↔db_str fail-closed roundtrips.

## Verification

`backend/crates/docs/domain/src/lib.rs` baseline 5 → 6

```
cargo test -p console-docs-domain --lib
# 6 passed
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
    "Essentialism / YAGNI": "One pure evidence vocab roundtrip test.",
    "Chesterton's Fence": "Existing parse/as_db_str tags retained.",
    "Pragmatism": "AC: 6 domain tests green (was 5).",
    "Red Team": "Unknown tags fail closed.",
    "Blast-radius / cell-based": "docs/domain only.",
    "Zero-trust / defense-in-depth": "Fail-closed evidence custody vocabulary."
  },
  "mandatory_lens_exceptions": {},
  "findings": [
    "Evidence classification/source/WORM/TSA/custody/legal-hold/admissibility lacked full parse roundtrips."
  ],
  "decisions_changed_or_rejected": [],
  "lens_set_changes": []
}
```
<!-- REASONING-LENS-EVIDENCE:END -->
