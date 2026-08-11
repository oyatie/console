#!/usr/bin/env bash
# Cursor sessionStart — inject process posture (portable from oyatie/console harnesses).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

# Best-effort beads prime (fail open — not all sessions have bd).
if command -v bd >/dev/null 2>&1; then
  bd prime --hook-json 2>/dev/null || true
fi

python3 - <<'PY'
import json
msg = (
  "Console Cursor ratchet active (.cursor/rules + hooks). "
  "Native Task only (no mm-role). "
  "Lane worktrees: ONLY <hub>/.worktrees/<name> (scripts/cursor/provision-lane-worktree.sh) — never ../console-lane-* siblings (External-File Protection / allow-edit prompts). "
  "Before first push: inventory ALL threads+prior findings → ONE fix commit. "
  "After that, only blocker or major+provenByExecution reopens the lane — not new unproven bot P2s. "
  "preflight-forge.sh before gh pr merge. "
  "Receipt: node scripts/cursor/validate-lane-receipt.mjs .cursor/receipts/<id>.json. "
  "Failure classes: .cursor/failure-classes-2026-08-10.md"
)
print(json.dumps({"additional_context": msg}))
PY
