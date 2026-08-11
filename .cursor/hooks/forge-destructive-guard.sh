#!/usr/bin/env bash
# Deny destructive forge (gh) surfaces and the git-push destructives that
# git-lock-enforcer does not already cover (-f, deletions, direct push to main).
# Everything else allows; push-admission.sh separately gates push/PR-create/merge.
set -euo pipefail
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

allow() {
  echo '{"permission":"allow"}'
  exit 0
}

# Escape hatch, same convention as git-lock-enforcer: session env var, or an
# explicit CURSOR_ALLOW_GIT_DANGEROUS=1 prefix on the command itself.
if [[ "${CURSOR_ALLOW_GIT_DANGEROUS:-}" == "1" ]] || printf '%s' "$CMD" | grep -q 'CURSOR_ALLOW_GIT_DANGEROUS=1'; then
  allow
fi

# --- gh destructive surfaces (no existing hook covers these) ---
if printf '%s' "$CMD" | grep -Eq 'gh[[:space:]]+repo[[:space:]]+(delete|rename)'; then
  deny "forge-guard: gh repo delete/rename is irreversible and reserved for humans. If explicitly approved, prefix CURSOR_ALLOW_GIT_DANGEROUS=1."
fi
if printf '%s' "$CMD" | grep -Eq 'gh[[:space:]]+secret([[:space:]]|$)'; then
  deny "forge-guard: gh secret touches repository credentials; agents must not read or write secrets. If explicitly approved, prefix CURSOR_ALLOW_GIT_DANGEROUS=1."
fi
if printf '%s' "$CMD" | grep -Eq 'gh[[:space:]]+auth[[:space:]]+(logout|refresh)'; then
  deny "forge-guard: gh auth logout/refresh would invalidate the shared gh session other lanes depend on (ops.gh-auth-stale). Use gh auth status to inspect."
fi
if printf '%s' "$CMD" | grep -Eq 'gh[[:space:]]+release[[:space:]]+delete'; then
  deny "forge-guard: gh release delete is destructive. If explicitly approved, prefix CURSOR_ALLOW_GIT_DANGEROUS=1."
fi
if printf '%s' "$CMD" | grep -Eq 'gh[[:space:]]+api' \
  && printf '%s' "$CMD" | grep -Eiq -- '(-X|--method)[[:space:]]*=?[[:space:]]*DELETE'; then
  deny "forge-guard: gh api with method DELETE is destructive. Reads/POST/PUT/PATCH are allowed; if deletion is explicitly approved, prefix CURSOR_ALLOW_GIT_DANGEROUS=1."
fi

# --- git push destructives NOT covered by git-lock-enforcer ---
# (git-lock-enforcer already denies long-form 'git push --force' without lease)
if printf '%s' "$CMD" | grep -Eq 'git[[:space:]]+push'; then
  if printf '%s' "$CMD" | grep -Eq 'git[[:space:]]+push[^;|&]*[[:space:]]-f([[:space:]]|$)' \
    && ! printf '%s' "$CMD" | grep -Eq -- '--force-with-lease|--force-if-includes'; then
    deny "forge-guard: git push -f (plain force) forbidden. Use --force-with-lease, and only after git rebase origin/main (BASE_LOCK restack rule)."
  fi
  if printf '%s' "$CMD" | grep -Eq 'git[[:space:]]+push[^;|&]*[[:space:]](--delete|-d|--mirror)([[:space:]]|$)'; then
    deny "forge-guard: git push --delete/-d/--mirror (remote deletion / mirror overwrite) forbidden. Branch cleanup belongs to the integration owner; prefix CURSOR_ALLOW_GIT_DANGEROUS=1 if explicitly approved."
  fi
  if printf '%s' "$CMD" | grep -Eq 'git[[:space:]]+push[^;|&]*[[:space:]][+]?:[^[:space:]]'; then
    deny "forge-guard: ':' deletion refspec removes a remote ref. Branch cleanup belongs to the integration owner; prefix CURSOR_ALLOW_GIT_DANGEROUS=1 if explicitly approved."
  fi
  if printf '%s' "$CMD" | grep -Eq 'git[[:space:]]+push[^;|&]*[[:space:]]([^[:space:]]*:)?(refs/heads/)?main([[:space:]]|$)'; then
    deny "forge-guard: direct push targeting main is forbidden (protected branch). Push a feature branch and open a PR (gh pr create); merges go through gh pr merge --squash after preflight."
  fi
fi

allow
