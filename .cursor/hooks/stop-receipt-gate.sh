#!/usr/bin/env bash
# stop hook: only ratchet Cursor-owned dirt (.cursor/**, scripts/cursor/**).
# Do NOT nag about unrelated lane dirt (e.g. #618 migrations on this worktree).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

# Paths this ratchet owns. Foreign dirty files are ownerLease — another lane's problem.
OWNED_DIRTY="$(git status --porcelain 2>/dev/null | grep -E '(\.cursor/|scripts/cursor/)' || true)"
if [[ -z "$OWNED_DIRTY" ]]; then
  echo '{}'
  exit 0
fi

OK=0
shopt -s nullglob
for f in .cursor/receipts/*.json; do
  # Prefer a build receipt for this lane
  case "$f" in
    *-critic.json) continue ;;
  esac
  if node scripts/cursor/validate-lane-receipt.mjs "$f" >/dev/null 2>&1; then
    OK=1
    break
  fi
done

if [[ "$OK" -eq 1 ]]; then
  echo '{}'
  exit 0
fi

python3 - <<'PY'
import json
print(json.dumps({
  "followup_message": (
    "Cursor-owned dirt (.cursor/** or scripts/cursor/**) without a valid build receipt. "
    "Before claiming the ratchet done: (1) enumerate known blockers and fix in ONE commit, "
    "(2) write .cursor/receipts/<lane>.json with enforcementPlacement/peripheralsUpdated "
    "(and commands/headSha if status=done), "
    "(3) node scripts/cursor/validate-lane-receipt.mjs .cursor/receipts/<lane>.json, "
    "(4) native Grok critic once — major blocks only if provenByExecution. "
    "Ignore unrelated worktree dirt (other PR lanes). No mm-role."
  )
}))
PY
