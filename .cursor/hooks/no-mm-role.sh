#!/usr/bin/env bash
# mm-role / claude -p / codex exec are a Grok transport accident — deny by default in Cursor.
set -euo pipefail
INPUT="$(cat || true)"
CMD="$(python3 -c 'import json,sys
try:
  d=json.load(sys.stdin)
except Exception:
  d={}
print(d.get("command") or d.get("toolInput",{}).get("command") or d.get("tool_input",{}).get("command") or "")
' <<<"$INPUT" 2>/dev/null || true)"

if [[ "${CURSOR_ALLOW_MM_ROLE:-}" == "1" ]]; then
  echo '{"permission":"allow"}'
  exit 0
fi

if printf '%s' "$CMD" | grep -Eq '(^|[/\s])mm-role([[:space:]]|$)|claude[[:space:]]+-p|codex[[:space:]]+exec'; then
  python3 -c 'import json; print(json.dumps({
    "permission":"deny",
    "user_message":"Use Cursor-native Task subagents (Grok 4.5 / Composer), not mm-role/claude -p/codex exec. Set CURSOR_ALLOW_MM_ROLE=1 only if you explicitly want CLI receipts.",
    "agent_message":"Transport ban: mm-role is a Grok auth workaround. Prefer Task(subagent) with cursor-grok-4.5-high."
  }))'
  exit 0
fi

echo '{"permission":"allow"}'
exit 0
