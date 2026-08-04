# Runnable rollback — <change>
- Candidate and deployment revision:
- Trigger / stop condition:
- Exact commands and required approvals:
- Readback and verification evidence:
- Data/schema considerations and owner:

## Reasoning-lens evidence example

<!-- REASONING-LENS-EVIDENCE:START -->
```json
{
  "lens_contract": "v1",
  "lens_contract_digest": "ac1e7d6b8150808ef73e5e3cd1a1e54d2f37eb43e84aaa1370dbbaaff3c44373",
  "task_class": "planning",
  "risk_class": "high",
  "risk_domains": [
    "approval",
    "release",
    "production"
  ],
  "selected_lenses": [
    "Red Team",
    "Operability / Day-2",
    "Blast-radius / cell-based",
    "Zero-trust / defense-in-depth"
  ],
  "task_fit": {
    "Red Team": "EXAMPLE: Modeled misuse and rollback failure paths.",
    "Operability / Day-2": "EXAMPLE: Defined executable recovery and post-rollback ownership.",
    "Blast-radius / cell-based": "EXAMPLE: Limited rollback effects to an independently recoverable boundary.",
    "Zero-trust / defense-in-depth": "EXAMPLE: Required independent approval and readback at each boundary."
  },
  "mandatory_lens_exceptions": {},
  "findings": [
    "EXAMPLE: Rollback must stop unless approval, execution, and readback evidence agree."
  ],
  "decisions_changed_or_rejected": [],
  "lens_set_changes": []
}
```
<!-- REASONING-LENS-EVIDENCE:END -->
