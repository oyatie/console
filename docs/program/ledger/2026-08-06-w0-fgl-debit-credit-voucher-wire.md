# Ledger: W0-FGL debit/credit + voucher status wire roundtrips

**Date:** 2026-08-06  
**Bead:** console-a80 / Wave0 domain epic follow-on  
**Allowlist:** backend/crates/finance-gl/domain/**

## Outcome

Domain unit tests for fail-closed `from_db_str` roundtrips on
`DebitCredit` and `VoucherStatus`, including Korean labels, terminal-posted
matrix, and rejection of plausible aliases. HOLDs not cleared; no migrations;
pure domain only.

## Verification

`docs/program/executed-tests-baseline.json` test_attribute_baseline for domain
lib: 3 → 5.

```
cargo test -p console-finance-gl-domain --lib
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
    "Essentialism / YAGNI": "Two pure unit tests only; no schema or API change.",
    "Chesterton's Fence": "Existing as_db_str/from_db_str tags retained; aliases rejected.",
    "Pragmatism": "Measurable AC: 5 domain unit tests green.",
    "Red Team": "Unknown tags and lowercase aliases fail closed; no alternate write path; no HOLD clear.",
    "Blast-radius / cell-based": "finance-gl/domain only (+ ledger/baseline).",
    "Zero-trust / defense-in-depth": "Fail-closed parse on wire/db tags for voucher FSM vocabulary."
  },
  "mandatory_lens_exceptions": {},
  "findings": [
    "DebitCredit and VoucherStatus had FSM/balance coverage but lacked explicit wire roundtrip + alias rejection tests."
  ],
  "decisions_changed_or_rejected": [],
  "lens_set_changes": []
}
```
<!-- REASONING-LENS-EVIDENCE:END -->
