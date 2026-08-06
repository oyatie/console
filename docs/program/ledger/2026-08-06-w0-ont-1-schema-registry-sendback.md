# Ledger: W0-ONT-1 ontology schema/registry wire + reviewer send-back

**Date:** 2026-08-06  
**Epic:** console-g1n / lane-ssf-ont  
**Allowlist:** backend/crates/ontology/domain/**

## Outcome

Unit tests: schema/registry wire roundtrips; fail-closed unknown tags (contrast
FieldKind); ReviewPending→Draft send-back.

## Verification

cargo test -p console-ontology-domain --lib  # 14 passed
Locked executed-tests-baseline ontology domain attrs.

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
    "Essentialism / YAGNI": "Domain pure tests only.",
    "Chesterton's Fence": "FieldKind contrast retained.",
    "Pragmatism": "14 unit tests green.",
    "Red Team": "Unknown tags fail closed.",
    "Operability / Day-2": "Baseline locked.",
    "Blast-radius / cell-based": "ontology/domain only.",
    "Telemetry-first": "cargo counts.",
    "Zero-trust / defense-in-depth": "Send-back cannot invent runtime status."
  },
  "mandatory_lens_exceptions": {},
  "findings": [
    "Schema/registry wire and reviewer send-back lacked pure domain coverage."
  ],
  "decisions_changed_or_rejected": [],
  "lens_set_changes": []
}
```
<!-- REASONING-LENS-EVIDENCE:END -->

## Authority tip

T is the signed authority tip for this candidate train. C prebinds this ledger blob.
