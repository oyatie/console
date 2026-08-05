# Workflow experiment — <name>
- Baseline fixture and prompt/workflow hash:
- Authority settings and changed variable:
- Quality findings, latency, and resource use:
- Outcome and acceptance evidence:
- Decision: retain, revise, or reject:

## Reasoning-lens evidence example

<!-- REASONING-LENS-EVIDENCE:START -->
```json
{
  "lens_contract": "v1",
  "lens_contract_digest": "ac1e7d6b8150808ef73e5e3cd1a1e54d2f37eb43e84aaa1370dbbaaff3c44373",
  "task_class": "verification",
  "risk_class": "standard",
  "risk_domains": [],
  "selected_lenses": [
    "Pragmatism",
    "Telemetry-first"
  ],
  "task_fit": {
    "Pragmatism": "EXAMPLE: Compared the workflow against the real acceptance outcome.",
    "Telemetry-first": "EXAMPLE: Captured quality, latency, resource, and failure signals."
  },
  "mandatory_lens_exceptions": {},
  "findings": [
    "EXAMPLE: The experiment needs repeatable evidence before the workflow is retained."
  ],
  "decisions_changed_or_rejected": [],
  "lens_set_changes": []
}
```
<!-- REASONING-LENS-EVIDENCE:END -->
