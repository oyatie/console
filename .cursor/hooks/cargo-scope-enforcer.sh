#!/usr/bin/env bash
# Scope Cargo; ban --workflow-only false greens.
set -euo pipefail
INPUT="$(cat || true)"
CMD="$(python3 -c 'import json,sys
try:
  d=json.load(sys.stdin)
except Exception:
  d={}
print(d.get("command") or d.get("toolInput",{}).get("command") or d.get("tool_input",{}).get("command") or "")
' <<<"$INPUT" 2>/dev/null || true)"

# Only care about cargo test / build invocations
if ! printf '%s' "$CMD" | grep -Eq '(^|[[:space:];|&])cargo[[:space:]]+(test|build|check|nextest)'; then
  echo '{"permission":"allow"}'
  exit 0
fi

if printf '%s' "$CMD" | grep -Eq -- '--workflow-only'; then
  python3 -c 'import json; print(json.dumps({
    "permission":"deny",
    "user_message":"BASE_LOCK: --workflow-only selects zero dark targets and exits 0 (false green). Use --only <name>.",
    "agent_message":"Denied --workflow-only. Use tools/ci/cargo_needs_postgres.sh --only <name> --num-threads=1."
  }))'
  exit 0
fi

# Bare cargo test without -p / --manifest-path / --package is a common false-scope
if printf '%s' "$CMD" | grep -Eq '(^|[[:space:];|&])cargo[[:space:]]+test([[:space:]]|$)' \
  && ! printf '%s' "$CMD" | grep -Eq -- '(-p[[:space:]]|--package[[:space:]]|--manifest-path[[:space:]]|cargo_needs_postgres)'; then
  python3 -c 'import json; print(json.dumps({
    "permission":"ask",
    "user_message":"Bare cargo test detected. Prefer scoped: cargo test --locked --manifest-path backend/Cargo.toml -p <pkg>",
    "agent_message":"Unscoped cargo test. Scope with -p <pkg> and --manifest-path backend/Cargo.toml."
  }))'
  exit 0
fi

echo '{"permission":"allow"}'
exit 0
