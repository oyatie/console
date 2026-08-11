#!/usr/bin/env bash
# Cursor adapter: local admit on push/PR create; forge preflight on gh pr merge.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
INPUT="$(cat || true)"

CMD="$(python3 -c 'import json,sys
try:
  d=json.load(sys.stdin)
except Exception:
  d={}
print(d.get("command") or d.get("toolInput",{}).get("command") or d.get("tool_input",{}).get("command") or "")
' <<<"$INPUT" 2>/dev/null || true)"

deny() {
  python3 -c 'import json,sys; print(json.dumps({"permission":"deny","user_message":sys.argv[1],"agent_message":sys.argv[1]}))' "$1"
  exit 0
}

# Merge / heavy forge ops require live gh auth (ops.gh-auth-stale)
if printf '%s' "$CMD" | grep -Eq 'gh[[:space:]]+pr[[:space:]]+merge'; then
  if ! bash "$ROOT/scripts/cursor/preflight-forge.sh" >/tmp/console-preflight-forge.out 2>/tmp/console-preflight-forge.err; then
    deny "ops.gh-auth-stale: $(head -c 800 /tmp/console-preflight-forge.err)"
  fi
fi

OUT="$(printf '%s' "$INPUT" | "$ROOT/scripts/hooks/pre-tool-push-admission.sh" || true)"

if printf '%s' "$OUT" | grep -q '"decision"[[:space:]]*:[[:space:]]*"deny"'; then
  REASON="$(printf '%s' "$OUT" | python3 -c 'import json,sys
try:
  d=json.load(sys.stdin); print(d.get("reason","admission denied"))
except Exception:
  print("admission denied")
' 2>/dev/null || echo "admission denied")"
  deny "$REASON"
fi

echo '{"permission":"allow"}'
exit 0
