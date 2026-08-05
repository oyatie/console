# Pre-mortem — <lane>
- Outcome / non-goals:
- Exact base SHA and writable roots:
- Failure scenarios and blast radius:
- Detection signals and test evidence:
- Rollback and stop conditions:
- Owner / reviewers / revisit date:

## Reasoning-lens evidence example

<!-- REASONING-LENS-EVIDENCE:START -->
```json
{
  "lens_contract": "v1",
  "lens_contract_digest": "ac1e7d6b8150808ef73e5e3cd1a1e54d2f37eb43e84aaa1370dbbaaff3c44373",
  "task_class": "planning",
  "risk_class": "standard",
  "risk_domains": [],
  "selected_lenses": [
    "Cartesian doubt",
    "Red Team"
  ],
  "task_fit": {
    "Cartesian doubt": "EXAMPLE: Separated verified constraints from assumptions.",
    "Red Team": "EXAMPLE: Identified hostile inputs and credible failure paths."
  },
  "mandatory_lens_exceptions": {},
  "findings": [
    "EXAMPLE: The lane needs an explicit stop condition before implementation."
  ],
  "decisions_changed_or_rejected": [],
  "lens_set_changes": []
}
```
<!-- REASONING-LENS-EVIDENCE:END -->
