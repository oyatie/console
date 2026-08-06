# Console agent MEMORY (bounded, Hermes-style)

Curated process facts only. Prefer promoting durable rules into
`.grok/harness/*` and `.grok/workflows/*` (Bun doctrine).

## Standing rules

- Soft reds/blocks must land on `lane-board.live.json` (`ops.soft-red-silence`).
- Tip prebind: C prebinds T-final ledger blob; never naive `generate --write` after tip.
- Autonomy: agent review **APPROVE** + Required CI/Security → merge; fix until APPROVE.
- Ultragoal active → Stop hook loops into `/workflow program-control` or `ralph`.
- Forbidden control-plane CLIs: `omc`, `omx`, `gjc`, `hermes` (ideas only).
- [2026-08-06] class `ops.soft-red-silence`: hermes loop smoke
