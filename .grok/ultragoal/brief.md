# Console ultragoal brief (template)

Replace with the durable initiative brief, then:

```text
/workflow ultragoal {"action":"create-goals","brief_path":".grok/ultragoal/brief.md"}
/goal <aggregate objective printed by handoff>
/workflow program-control
```

## Constraints (always)

- Authority: `docs/current/{PRODUCT,ROADMAP,DELIVERY}.md` only
- HOLDs fail-closed
- Soft reds/blocks always on lane board
- Autonomy: agent review APPROVE + Required CI/Security → merge
- Process defects → edit workflows/harness/tools (Bun + Hermes learn)
- Forbidden CLIs: omc, omx, gjc, hermes (ideas only)
